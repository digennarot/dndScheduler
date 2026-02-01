use crate::core::events::{
    AvailabilityEntryV1, Event, ParticipantJoinedV1, ParticipantUpdatedV1, PollCreatedV1,
    PollFinalizedV1, VoteUpdatedV1,
};
use crate::core::models::{
    CreatePollRequest, FinalizePollRequest, JoinPollRequest, UpdateAvailabilityRequest,
};
use crate::core::store::RedbEventStore;
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::core::projections::PollsProjection;

// Temporary state struct until we merge with main state
#[derive(Clone)]
pub struct EventAppState {
    pub event_store: Arc<RedbEventStore>,
    pub pool: crate::db::DbPool,
    pub projection: Arc<PollsProjection>,
    pub users_projection: Arc<crate::core::projections::UsersProjection>,
}

impl axum::extract::FromRef<EventAppState> for crate::db::DbPool {
    fn from_ref(state: &EventAppState) -> Self {
        state.pool.clone()
    }
}

impl axum::extract::FromRef<EventAppState> for Arc<PollsProjection> {
    fn from_ref(state: &EventAppState) -> Self {
        state.projection.clone()
    }
}

pub async fn create_poll_event(
    State(state): State<EventAppState>,
    Json(payload): Json<CreatePollRequest>,
) -> impl IntoResponse {
    let poll_id = Uuid::new_v4().to_string();

    let event = PollCreatedV1 {
        id: poll_id.clone(),
        title: payload.title,
        description: payload.description,
        location: payload.location,
        dates: payload.dates,
    };

    // Serialize event
    // Note: In a real app we'd have a helper to serialize the ENUM,
    // here we wrap it manually.
    let event_enum = Event::V1PollCreated(event);
    let event_data = bincode::serialize(&event_enum).unwrap(); // Handle error properly in prod

    // Append to stream "poll-{id}"
    let stream_id = format!("poll-{}", poll_id);
    match state.event_store.append(&stream_id, &event_data, 0).await {
        Ok(_) => {
            // Update Projection
            state.projection.apply(event_enum);

            (
                StatusCode::CREATED,
                Json(serde_json::json!({ "id": poll_id, "status": "created_via_events" })),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

pub async fn get_poll_event(
    State(state): State<EventAppState>,
    axum::extract::Path(poll_id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let stream_id = format!("poll-{}", poll_id);

    // 1. Read Raw Events
    let raw_events = match state.event_store.read_stream(&stream_id).await {
        Ok(events) => events,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        }
    };

    if raw_events.is_empty() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Poll not found" })),
        );
    }

    // 2. Deserialize Events
    let mut history = Vec::new();
    for raw in raw_events {
        match bincode::deserialize::<Event>(&raw) {
            Ok(event) => history.push(event),
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": format!("Deserialization Error: {}", e) })),
                )
            }
        }
    }

    // 3. Rebuild Aggregate
    match crate::core::models::PollAggregate::load_from_history(history) {
        Ok(aggregate) => (StatusCode::OK, Json(serde_json::json!(aggregate))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

pub async fn update_availability_event(
    State(state): State<EventAppState>,
    auth_user: crate::security::auth::MaybeAuthUser,
    axum::extract::Path((poll_id, participant_id)): axum::extract::Path<(String, String)>,
    Json(payload): Json<UpdateAvailabilityRequest>,
) -> impl IntoResponse {
    let stream_id = format!("poll-{}", poll_id);

    // 1. Validation (Hybrid: Check Participant existence in SQL)
    // We need name, email, and user_id to form event/authorize
    let participant = match sqlx::query!(
        "SELECT name, email, access_token, user_id FROM participants WHERE id = ? AND poll_id = ?",
        participant_id,
        poll_id
    )
    .fetch_optional(&state.pool)
    .await
    {
        Ok(Some(p)) => p,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Participant not found" })),
            )
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        }
    };

    // 2. Authorization Logic
    let mut authorized = false;

    // Check Token
    if let Some(token) = &payload.access_token {
        if let Some(stored) = &participant.access_token {
            if token == stored {
                authorized = true;
            }
        }
    }

    // Check Session (if not already authorized)
    if !authorized {
        if let Some(user) = auth_user.0 {
            if let Some(owner_id) = &participant.user_id {
                if owner_id == &user.id {
                    authorized = true;
                }
            }
        }
    }

    if !authorized {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "Unauthorized" })),
        );
    }

    // 3. Create Event
    let availability_v1: Vec<AvailabilityEntryV1> = payload
        .availability
        .clone()
        .into_iter()
        .map(|a| AvailabilityEntryV1 {
            date: a.date,
            slot: a.time_slot,
            status: a.status,
        })
        .collect();

    let event = VoteUpdatedV1 {
        participant_name: participant.name,
        participant_email: participant.email.unwrap_or_default(), // Should handle None better but MVP
        availability: availability_v1,
    };

    let event_enum = Event::V1VoteUpdated(event);
    let event_data = match bincode::serialize(&event_enum) {
        Ok(d) => d,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
        }
    };

    // 4. Concurrency Check
    // We strictly should read the stream to get expected version, but for high throughput
    // we might just retry or read-then-write.
    // For MVP, we pass '0' which implies we don't care about concurrency (WRONG)
    // or we read last version first.
    // `RedbEventStore` append checks current version. We need to know it.
    // Optimize: RedbEventStore.append could take defined "ExpectedVersion::Any" or we read it.
    // Let's read it quickly.

    // READ SIDE for Version
    // TODO: Optimize this by having `append` handle "Any" or return current version.
    // For now, let's implement a naive read-then-write loop or just read once.

    let current_version = match state.event_store.read_stream(&stream_id).await {
        Ok(events) => {
            // We don't need to deserialize, just count?
            // Actually `RedbEventStore::append` requires `expected_version`.
            // If we rely on the internal check of append, we must know version.
            // Our `read_stream` returns `Vec<Vec<u8>>`. The version is `len()`.
            // Because version starts at 0? No, version 1 is first event.
            // `make_key` uses version.
            events.len() as u64
        }
        Err(_) => 0,
    };

    match state
        .event_store
        .append(&stream_id, &event_data, current_version)
        .await
    {
        Ok(_) => {
            // Update Projection
            state.projection.apply_with_id(&poll_id, event_enum);

            // PROJECTION (Sync Dual Write): Update SQL Read Model
            if let Ok(mut tx) = state.pool.begin().await {
                // ... (SQL logic remains for dual write/legacy)
                let _ = sqlx::query(
                    "DELETE FROM availability WHERE poll_id = ? AND participant_id = ?",
                )
                .bind(&poll_id)
                .bind(&participant_id)
                .execute(&mut *tx)
                .await;

                for entry in &payload.availability {
                    let _ = sqlx::query(
                        "INSERT INTO availability (poll_id, participant_id, date, time_slot, status) VALUES (?, ?, ?, ?, ?)",
                    )
                    .bind(&poll_id)
                    .bind(&participant_id)
                    .bind(&entry.date)
                    .bind(&entry.time_slot)
                    .bind(&entry.status)
                    .execute(&mut *tx)
                    .await;
                }
                let _ = tx.commit().await;
            }

            (
                StatusCode::OK,
                Json(serde_json::json!({ "status": "vote_recorded", "mode": "dual_write" })),
            )
        }
        Err(e) => (
            StatusCode::CONFLICT,
            Json(serde_json::json!({ "error": format!("Concurrency/Write Error: {}", e) })),
        ),
    }
}

pub async fn join_poll_event(
    State(state): State<EventAppState>,
    axum::extract::Path(poll_id): axum::extract::Path<String>,
    Json(payload): Json<JoinPollRequest>,
) -> impl IntoResponse {
    let stream_id = format!("poll-{}", poll_id);

    // 1. Read Stream & Rebuild Aggregate
    let current_version = match state.event_store.read_stream(&stream_id).await {
        Ok(events) => events.len() as u64,
        Err(_) => 0,
    };

    let history = match state.event_store.read_stream(&stream_id).await {
        Ok(raw_events) => raw_events
            .iter()
            .filter_map(|raw| bincode::deserialize::<Event>(raw).ok())
            .collect::<Vec<_>>(),
        Err(_) => Vec::new(),
    };

    let aggregate =
        crate::core::models::PollAggregate::load_from_history(history).unwrap_or_default();

    // 2. Determine Action (Join vs Update)
    let mut event_enum = None;
    let mut participant_id = String::new();
    let mut access_token = String::new();
    let mut is_update = false;

    if let Some(email) = &payload.email {
        if let Some(existing) = aggregate
            .participants
            .iter()
            .find(|p| p.email.as_ref() == Some(email))
        {
            // UPDATE EXISTING
            is_update = true;
            participant_id = existing.id.clone();
            // We'll fetch the real token from SQL later if needed for return value
            // Aggregate doesn't store token currently

            let update_event = ParticipantUpdatedV1 {
                id: participant_id.clone(),
                poll_id: poll_id.clone(),
                name: payload.name.clone(),
                email: Some(email.clone()),
            };
            event_enum = Some(Event::V1ParticipantUpdated(update_event));
        }
    }

    if event_enum.is_none() {
        // CREATE NEW
        participant_id = Uuid::new_v4().to_string();
        access_token = Uuid::new_v4().to_string();

        let join_event = ParticipantJoinedV1 {
            id: participant_id.clone(),
            poll_id: poll_id.clone(),
            name: payload.name.clone(),
            email: payload.email.clone(),
            access_token: access_token.clone(),
        };
        event_enum = Some(Event::V1ParticipantJoined(join_event));
    }

    let event_data = bincode::serialize(event_enum.as_ref().unwrap()).unwrap();

    // 3. Append to Event Store (Optimistic Lock)
    // 3. Append to Event Store (Optimistic Lock)
    match state
        .event_store
        .append(&stream_id, &event_data, current_version)
        .await
    {
        Ok(_) => {
            // Update Projection
            if let Some(ev) = event_enum {
                state.projection.apply(match ev {
                    Event::V1ParticipantJoined(e) => Event::V1ParticipantJoined(e),
                    Event::V1ParticipantUpdated(e) => Event::V1ParticipantUpdated(e),
                    _ => panic!("Unexpected event type"),
                });
            }

            // 4. Dual Write to SQL
            if is_update {
                // ... (SQL logic)
                let row: Option<(String,)> =
                    sqlx::query_as("SELECT access_token FROM participants WHERE id = ?")
                        .bind(&participant_id)
                        .fetch_optional(&state.pool)
                        .await
                        .unwrap_or(None);
                if let Some(r) = row {
                    access_token = r.0;
                }

                let _ = sqlx::query("UPDATE participants SET name = ? WHERE id = ?")
                    .bind(&payload.name)
                    .bind(&participant_id)
                    .execute(&state.pool)
                    .await;
            } else {
                // ... (SQL logic)
                let _ = sqlx::query(
                    "INSERT INTO participants (id, poll_id, name, email, access_token) VALUES (?, ?, ?, ?, ?)",
                )
                .bind(&participant_id)
                .bind(&poll_id)
                .bind(&payload.name)
                .bind(&payload.email)
                .bind(&access_token)
                .execute(&state.pool)
                .await;
            }

            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "id": participant_id,
                    "access_token": access_token,
                    "message": "Successfully joined (Event Sourced)"
                })),
            )
        }
        Err(e) => (
            StatusCode::CONFLICT,
            Json(serde_json::json!({ "error": format!("Concurrency Error: {}", e) })),
        ),
    }
}

pub async fn finalize_poll_event(
    State(state): State<EventAppState>,
    // Ensure Admin Auth
    _admin: crate::security::auth::AdminUser,
    axum::extract::Path(poll_id): axum::extract::Path<String>,
    Json(payload): Json<FinalizePollRequest>,
) -> impl IntoResponse {
    let stream_id = format!("poll-{}", poll_id);

    // 1. Validation
    if payload.finalized_time.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Finalized time cannot be empty" })),
        );
    }

    // 2. Read Stream & check status
    let current_version = match state.event_store.read_stream(&stream_id).await {
        Ok(events) => events.len() as u64,
        Err(_) => 0,
    };

    let history = match state.event_store.read_stream(&stream_id).await {
        Ok(raw_events) => raw_events
            .iter()
            .filter_map(|raw| bincode::deserialize::<Event>(raw).ok())
            .collect::<Vec<_>>(),
        Err(_) => Vec::new(),
    };

    if history.is_empty() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Poll not found" })),
        );
    }

    let aggregate =
        crate::core::models::PollAggregate::load_from_history(history).unwrap_or_default();

    if aggregate.status == "finalized" {
        return (
            StatusCode::CONFLICT,
            Json(serde_json::json!({ "error": "Poll is already finalized" })),
        );
    }

    // 3. Create Event
    let now = chrono::Utc::now().timestamp();
    let event = PollFinalizedV1 {
        id: poll_id.clone(),
        finalized_at: now,
        finalized_time: payload.finalized_time.clone(),
        notes: payload.notes.clone(),
    };

    let event_enum = Event::V1PollFinalized(event);
    let event_data = bincode::serialize(&event_enum).unwrap();

    // 4. Append to Event Store
    // 4. Append to Event Store
    match state
        .event_store
        .append(&stream_id, &event_data, current_version)
        .await
    {
        Ok(_) => {
            // Update Projection
            state.projection.apply(event_enum);

            // 5. Dual Write to SQL
            let sql_res = sqlx::query(
                "UPDATE polls SET status = 'finalized', finalized_at = ?, finalized_time = ?, notes = ? WHERE id = ?",
            )
            .bind(now)
            .bind(&payload.finalized_time)
            .bind(&payload.notes)
            .bind(&poll_id)
            .execute(&state.pool)
            .await;

            if let Err(e) = sql_res {
                tracing::error!("Failed to update SQL read model for finalize: {}", e);
            }

            // Log activity
            crate::activity_handlers::log_activity(
                &state.pool,
                "poll_finalized",
                "system".to_string(),
                "Organizzatore".to_string(),
                Some(poll_id),
                Some(aggregate.title),
            )
            .await
            .unwrap_or_else(|e| tracing::error!("Activity log error: {}", e));

            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "success": true,
                    "status": "finalized",
                    "finalizedAt": now,
                    "mode": "event_sourced"
                })),
            )
        }
        Err(e) => (
            StatusCode::CONFLICT,
            Json(serde_json::json!({ "error": format!("Concurrency Error: {}", e) })),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::JoinPollRequest;
    use axum::extract::State;
    use sqlx::sqlite::SqlitePoolOptions;
    use uuid::Uuid;

    async fn setup_test_db() -> crate::db::DbPool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("Failed to connect to memory db");

        // Initialize Schema (Simplified for test)
        sqlx::query("CREATE TABLE polls (id TEXT PRIMARY KEY, title TEXT NOT NULL, description TEXT NOT NULL, location TEXT NOT NULL, created_at INTEGER NOT NULL, dates TEXT NOT NULL, time_range TEXT NOT NULL, status TEXT NOT NULL DEFAULT 'active', finalized_at INTEGER, finalized_time TEXT, notes TEXT, admin_token TEXT, organizer_id TEXT, recurrence_rule TEXT)")
            .execute(&pool).await.unwrap();
        sqlx::query("CREATE TABLE participants (id TEXT PRIMARY KEY, poll_id TEXT NOT NULL, name TEXT NOT NULL, email TEXT, access_token TEXT UNIQUE, user_id TEXT, FOREIGN KEY (poll_id) REFERENCES polls (id))")
            .execute(&pool).await.unwrap();
        sqlx::query("CREATE TABLE availability (id INTEGER PRIMARY KEY, poll_id TEXT NOT NULL, participant_id TEXT NOT NULL, date TEXT NOT NULL, time_slot TEXT NOT NULL, status TEXT NOT NULL, FOREIGN KEY (poll_id) REFERENCES polls (id), FOREIGN KEY (participant_id) REFERENCES participants (id))")
            .execute(&pool).await.unwrap();

        pool
    }

    async fn setup_test_event_store() -> (Arc<RedbEventStore>, String) {
        let file = format!("test_app_handler_{}.redb", Uuid::new_v4());
        let store = Arc::new(RedbEventStore::new(&file).unwrap());
        (store, file)
    }

    #[tokio::test]
    async fn test_join_poll_event_flow() {
        let pool = setup_test_db().await;
        let (event_store, file) = setup_test_event_store().await;

        let projection = Arc::new(crate::core::projections::PollsProjection::new());
        let state = EventAppState {
            event_store: event_store.clone(),
            pool: pool.clone(),
            projection: projection.clone(),
            users_projection: Arc::new(crate::core::projections::UsersProjection::new()),
        };

        // 1. Create Poll (SQL only needed for FK check description in Dual Write)
        let poll_id = Uuid::new_v4().to_string();
        sqlx::query("INSERT INTO polls (id, title, description, location, created_at, dates, time_range) VALUES (?, 'T', 'D', 'L', 0, '[]', '{}')")
            .bind(&poll_id)
            .execute(&pool).await.unwrap();

        // 2. Join Poll (New)
        let req = JoinPollRequest {
            name: "New User".to_string(),
            email: Some("new@test.com".to_string()),
        };

        let res = join_poll_event(
            State(state.clone()),
            axum::extract::Path(poll_id.clone()),
            Json(req),
        )
        .await
        .into_response();
        assert_eq!(res.status(), StatusCode::OK);

        // Extract body to verify token
        let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        let participant_id = body_json.get("id").unwrap().as_str().unwrap().to_string();
        let access_token = body_json
            .get("access_token")
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();

        // 3. Verify SQL Persistence
        let row = sqlx::query!(
            "SELECT name, access_token FROM participants WHERE id = ?",
            participant_id
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(row.name, "New User");
        assert_eq!(row.access_token, Some(access_token.clone()));

        // 4. Verify Event Persistence
        let stream_id = format!("poll-{}", poll_id);
        let events = event_store.read_stream(&stream_id).await.unwrap();
        assert_eq!(events.len(), 1);
        let event: Event = bincode::deserialize(&events[0]).unwrap();
        match event {
            Event::V1ParticipantJoined(e) => {
                assert_eq!(e.name, "New User");
                assert_eq!(e.email, Some("new@test.com".to_string()));
            }
            _ => panic!("Wrong event type"),
        }

        // 5. Join Again (Update)
        let req_update = JoinPollRequest {
            name: "Updated User".to_string(),
            email: Some("new@test.com".to_string()), // Same email
        };

        let res_upd = join_poll_event(
            State(state.clone()),
            axum::extract::Path(poll_id.clone()),
            Json(req_update),
        )
        .await
        .into_response();
        assert_eq!(res_upd.status(), StatusCode::OK);

        let body_bytes_upd = axum::body::to_bytes(res_upd.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_json_upd: serde_json::Value = serde_json::from_slice(&body_bytes_upd).unwrap();
        let token_upd = body_json_upd.get("access_token").unwrap().as_str().unwrap();

        assert_eq!(token_upd, access_token); // Should match old token

        // Verify SQL Update
        let row_upd = sqlx::query!("SELECT name FROM participants WHERE id = ?", participant_id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(row_upd.name, "Updated User");

        // Verify Event Update
        let events_upd = event_store.read_stream(&stream_id).await.unwrap();
        assert_eq!(events_upd.len(), 2);
        let event_2: Event = bincode::deserialize(&events_upd[1]).unwrap();
        match event_2 {
            Event::V1ParticipantUpdated(e) => {
                assert_eq!(e.name, "Updated User");
            }
            _ => panic!("Wrong event type"),
        }

        std::fs::remove_file(file).ok();
    }

    #[tokio::test]
    async fn test_finalize_poll_event_flow() {
        use crate::core::models;
        use axum::extract::Path;
        use chrono::Utc;

        let pool = setup_test_db().await;
        let (event_store, file) = setup_test_event_store().await;

        let projection = Arc::new(crate::core::projections::PollsProjection::new());
        let state = EventAppState {
            event_store: event_store.clone(),
            pool: pool.clone(),
            projection: projection.clone(),
            users_projection: Arc::new(crate::core::projections::UsersProjection::new()),
        };

        // 1. Create Poll (SQL and Event)
        let poll_id = Uuid::new_v4().to_string();
        sqlx::query("INSERT INTO polls (id, title, description, location, created_at, dates, time_range) VALUES (?, 'T', 'D', 'L', 0, '[]', '{}')")
            .bind(&poll_id)
            .execute(&pool).await.unwrap();

        // Seed Event Store
        let created_event = crate::core::events::PollCreatedV1 {
            id: poll_id.clone(),
            title: "T".to_string(),
            description: "D".to_string(),
            location: "L".to_string(),
            dates: vec![],
        };
        let event_data =
            bincode::serialize(&crate::core::events::Event::V1PollCreated(created_event)).unwrap();
        event_store
            .append(&format!("poll-{}", poll_id), &event_data, 0)
            .await
            .unwrap();

        // 2. Finalize Poll
        let req = models::FinalizePollRequest {
            finalized_time: "2023-10-27T10:00:00Z".to_string(),
            notes: Some("It's happening!".to_string()),
        };

        let admin = crate::core::models::Admin {
            id: Uuid::new_v4().to_string(),
            username: "Admin".to_string(),
            password_hash: "hash".to_string(),
            email: Some("admin@test.com".to_string()),
            role: "admin".to_string(),
            created_at: Utc::now().timestamp(),
        };
        let admin_user = crate::security::auth::AdminUser(admin);

        let res = finalize_poll_event(
            State(state.clone()),
            admin_user,
            Path(poll_id.clone()),
            Json(req),
        )
        .await
        .into_response();
        assert_eq!(res.status(), StatusCode::OK);

        // Verify SQL
        let row = sqlx::query!(
            "SELECT status, finalized_time FROM polls WHERE id = ?",
            poll_id
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(row.status, "finalized");
        assert_eq!(row.finalized_time, Some("2023-10-27T10:00:00Z".to_string()));

        // Verify Event
        let stream_id = format!("poll-{}", poll_id);
        let events = event_store.read_stream(&stream_id).await.unwrap();
        assert_eq!(events.len(), 2); // PollCreated + PollFinalized
        let event: Event = bincode::deserialize(&events[1]).unwrap();
        match event {
            Event::V1PollFinalized(e) => {
                assert_eq!(e.finalized_time, "2023-10-27T10:00:00Z");
            }
            _ => panic!("Wrong event type"),
        }

        std::fs::remove_file(file).ok();
    }

    #[tokio::test]
    async fn test_update_availability_event_flow() {
        use crate::core::models;
        use axum::extract::Path;

        let pool = setup_test_db().await;
        let (event_store, file) = setup_test_event_store().await;

        let projection = Arc::new(crate::core::projections::PollsProjection::new());
        let state = EventAppState {
            event_store: event_store.clone(),
            pool: pool.clone(),
            projection: projection.clone(),
            users_projection: Arc::new(crate::core::projections::UsersProjection::new()),
        };

        let poll_id = Uuid::new_v4().to_string();
        let participant_id = Uuid::new_v4().to_string();
        let token = Uuid::new_v4().to_string();
        let dates = serde_json::json!(["2023-10-27"]).to_string();

        // SeePoll Setup
        sqlx::query("INSERT INTO polls (id, title, description, location, created_at, dates, time_range) VALUES (?, 'T', 'D', 'L', 0, ?, '{}')")
            .bind(&poll_id)
            .bind(&dates)
            .execute(&pool).await.unwrap();

        sqlx::query("INSERT INTO participants (id, poll_id, name, email, access_token) VALUES (?, ?, 'User', 'u@test.com', ?)")
            .bind(&participant_id)
            .bind(&poll_id)
            .bind(&token)
            .execute(&pool).await.unwrap();

        let req = models::UpdateAvailabilityRequest {
            availability: vec![models::AvailabilityEntry {
                date: "2023-10-27".to_string(),
                time_slot: "10:00".to_string(),
                status: "available".to_string(),
            }],
            access_token: Some(token.clone()),
        };

        let res = update_availability_event(
            State(state.clone()),
            crate::security::auth::MaybeAuthUser(None),
            Path((poll_id.clone(), participant_id.clone())),
            Json(req),
        )
        .await
        .into_response();
        assert_eq!(res.status(), StatusCode::OK);

        // Verify SQL
        let row = sqlx::query!(
            "SELECT status FROM availability WHERE poll_id = ? AND participant_id = ?",
            poll_id,
            participant_id
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(row.status, "available");

        // Verify Event
        let stream_id = format!("poll-{}", poll_id);
        let events = event_store.read_stream(&stream_id).await.unwrap();
        let event: Event = bincode::deserialize(&events[0]).unwrap();
        match event {
            Event::V1VoteUpdated(e) => {
                assert_eq!(e.availability.len(), 1);
                assert_eq!(e.availability[0].status, "available");
            }
            _ => panic!("Wrong event type"),
        }

        std::fs::remove_file(file).ok();
    }
}
