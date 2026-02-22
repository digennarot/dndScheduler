use crate::core::events::{
    AvailabilityEntryV1, Event, ParticipantJoinedV1, ParticipantUpdatedV1, PollCreatedV2,
    PollFinalizedV1, VoteUpdatedV2,
};
use crate::core::models::{
    CreatePollRequest, FinalizePollRequest, JoinPollRequest, UpdateAvailabilityRequest, VoteRequest,
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
    pub key: axum_extra::extract::cookie::Key,
}

impl axum::extract::FromRef<EventAppState> for axum_extra::extract::cookie::Key {
    fn from_ref(state: &EventAppState) -> Self {
        state.key.clone()
    }
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
    let created_at = chrono::Utc::now().timestamp();

    let event = PollCreatedV2 {
        id: poll_id.clone(),
        title: payload.title,
        description: payload.description,
        location: payload.location,
        dates: payload.dates,
        created_at,
    };

    // Serialize event
    // Note: In a real app we'd have a helper to serialize the ENUM,
    // here we wrap it manually.
    let event_enum = Event::V2PollCreated(event);
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
    println!("EXTERING update_availability_event for poll: {}, pt: {}", poll_id, participant_id);
    let stream_id = format!("poll-{}", poll_id);

    // 1. Validation (Hybrid: Check Participant existence in SQL)
    let poll_view = state.projection.get(&poll_id);
    let participant = match poll_view {
        Some(view) => view.participants.iter().find(|p| p.id == participant_id).cloned(),
        None => None,
    };

    let participant = match participant {
        Some(p) => p,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Participant not found" })),
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

    let event = VoteUpdatedV2 {
        participant_name: participant.name.clone(),
        participant_email: participant.email.clone(),
        availability: availability_v1,
    };

    let event_enum = Event::V2VoteUpdated(event);
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

            // PROJECTION (Sync Dual Write): Update Redb Read Model
            let _ = crate::db::queries::poll_repo::PollRepo::upsert_vote(
                &state.pool,
                &poll_id,
                &participant_id,
                &participant.name,
                payload.availability.clone(),
            );

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

pub async fn vote_poll_event(
    State(state): State<EventAppState>,
    jar: axum_extra::extract::cookie::SignedCookieJar,
    axum::extract::Path(poll_id): axum::extract::Path<String>,
    Json(payload): Json<VoteRequest>,
) -> impl IntoResponse {
    let stream_id = format!("poll-{}", poll_id);
    let session_id = jar.get("dnd_session").map(|c| c.value().to_string());
    
    // Determine Participant ID (Session ID or New UUID for fallback)
    let participant_id = session_id.unwrap_or_else(|| Uuid::new_v4().to_string());

    let existing = state.projection.get(&poll_id).and_then(|view| {
        view.participants.iter().find(|p| p.id == participant_id).cloned()
    });

    let mut current_version = match state.event_store.read_stream(&stream_id).await {
        Ok(events) => events.len() as u64,
        Err(_) => 0,
    };

    let mut email_to_use = None;

    // If new participant, emit Joined Event
    if existing.is_none() {
        let join_event = ParticipantJoinedV1 {
            id: participant_id.clone(),
            poll_id: poll_id.clone(),
            name: payload.name.clone(),
            email: None,
            access_token: Uuid::new_v4().to_string(),
        };
        
        let event_enum = Event::V1ParticipantJoined(join_event);
        let event_data = bincode::serialize(&event_enum).unwrap();
        
        if let Ok(_) = state.event_store.append(&stream_id, &event_data, current_version).await {
            state.projection.apply(event_enum);
            current_version += 1;
            
            // Sync to SQL
            if let Ok(write_txn) = state.pool.begin_write() {
                let _ = crate::db::queries::poll_repo::PollRepo::add_participant(
                    &write_txn,
                    &participant_id,
                    &poll_id,
                    &payload.name,
                    None,
                    Some(&Uuid::new_v4().to_string()),
                );
                let _ = write_txn.commit();
            }
        }
    } else {
         if let Some(p) = &existing {
             email_to_use = p.email.clone();
             // Update name if changed?
             if p.name != payload.name {
                 let update_event = ParticipantUpdatedV1 {
                    id: participant_id.clone(),
                    poll_id: poll_id.clone(),
                    name: payload.name.clone(),
                    email: p.email.clone(),
                 };
                 let event_enum = Event::V1ParticipantUpdated(update_event);
                 let event_data = bincode::serialize(&event_enum).unwrap();
                 if let Ok(_) = state.event_store.append(&stream_id, &event_data, current_version).await {
                    state.projection.apply(event_enum);
                    current_version += 1;
                    // Update name in redb if changed
                    // Since PollRepo doesn't have an explicit update participant just by name (without availability)
                    // We'll rely on the projection. For completeness, upsert_vote handles changing name but here we might just have a participant update.
                    // Let's just update the participant in redb by adding them again with same ID.
                    if let Ok(write_txn) = state.pool.begin_write() {
                        let _ = crate::db::queries::poll_repo::PollRepo::add_participant(
                            &write_txn,
                            &participant_id,
                            &poll_id,
                            &payload.name,
                            p.email.as_deref(), // email
                            p.access_token.as_deref(),
                        );
                        let _ = write_txn.commit();
                    }
                 }
            }
        }
    }

    // Emit Vote Event
    let availability_v1: Vec<AvailabilityEntryV1> = payload.availability.iter().map(|a| AvailabilityEntryV1 {
        date: a.date.clone(),
        slot: a.time_slot.clone(),
        status: a.status.clone(),
    }).collect();

    let vote_event = VoteUpdatedV2 {
        participant_name: payload.name.clone(),
        participant_email: email_to_use, 
        availability: availability_v1.clone(),
    };

    let event_enum = Event::V2VoteUpdated(vote_event);
    let event_data = bincode::serialize(&event_enum).unwrap();

    match state.event_store.append(&stream_id, &event_data, current_version).await {
        Ok(_) => {
            state.projection.apply_with_id(&poll_id, event_enum);
            
            // Sync to Redb (Dual Write)
            let _ = crate::db::queries::poll_repo::PollRepo::upsert_vote(
                &state.pool,
                &poll_id,
                &participant_id,
                &payload.name,
                payload.availability.clone(),
            );
            
            (
                StatusCode::OK,
                Json(serde_json::json!({ "status": "voted", "participant_id": participant_id })),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
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

            // 4. Dual Write to Redb
            if is_update {
                access_token = state.projection.get(&poll_id).and_then(|v| {
                    v.participants.iter().find(|p| p.id == participant_id).and_then(|p| p.access_token.clone())
                }).unwrap_or_else(|| Uuid::new_v4().to_string());
                
                if let Ok(write_txn) = state.pool.begin_write() {
                    let _ = crate::db::queries::poll_repo::PollRepo::add_participant(
                        &write_txn,
                        &participant_id,
                        &poll_id,
                        &payload.name,
                        payload.email.as_deref(),
                        Some(&access_token),
                    );
                    let _ = write_txn.commit();
                }
            } else {
                if let Ok(write_txn) = state.pool.begin_write() {
                    let _ = crate::db::queries::poll_repo::PollRepo::add_participant(
                        &write_txn,
                        &participant_id,
                        &poll_id,
                        &payload.name,
                        payload.email.as_deref(),
                        Some(&access_token),
                    );
                    let _ = write_txn.commit();
                }
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
            let sql_res = crate::db::queries::poll_repo::PollRepo::finalize_poll(
                &state.pool,
                &poll_id,
                &payload.finalized_time,
                payload.notes.as_deref()
            );

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

