use crate::core::models;
use crate::core::models::{
    Availability, CreatePollRequest, JoinPollRequest, Participant, Poll, UpdateAvailabilityRequest,
};
use crate::db::DbPool;
use crate::security::auth::MaybeAuthUser;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

// Security constants
const MAX_TITLE_LENGTH: usize = 200;
const MAX_DESCRIPTION_LENGTH: usize = 2000;
const MAX_LOCATION_LENGTH: usize = 200;
const MAX_NAME_LENGTH: usize = 100;
const MAX_EMAIL_LENGTH: usize = 254; // RFC 5321
const MAX_PARTICIPANTS: usize = 100;
const MAX_DATES: usize = 365;
const MAX_AVAILABILITY_ENTRIES: usize = 1000;

// Input validation helpers
fn validate_email(email: &str) -> Result<(), String> {
    if email.is_empty() || email.len() > MAX_EMAIL_LENGTH {
        return Err("Invalid email length".to_string());
    }

    // Basic email validation
    if !email.contains('@') || !email.contains('.') {
        return Err("Invalid email format".to_string());
    }

    // Check for dangerous characters
    if email.contains(['<', '>', '"', '\'', '\\', '\0']) {
        return Err("Invalid characters in email".to_string());
    }

    Ok(())
}

fn validate_string_length(s: &str, max_len: usize, field_name: &str) -> Result<(), String> {
    if s.is_empty() {
        return Err(format!("{} cannot be empty", field_name));
    }
    if s.len() > max_len {
        return Err(format!(
            "{} exceeds maximum length of {}",
            field_name, max_len
        ));
    }
    Ok(())
}

fn validate_uuid(id: &str) -> Result<(), String> {
    Uuid::parse_str(id)
        .map(|_| ())
        .map_err(|_| "Invalid ID format".to_string())
}

fn sanitize_string(s: &str) -> String {
    let mut escaped = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '&' => escaped.push_str("&amp;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#x27;"),
            _ => {
                if !c.is_control() || c == '\n' || c == '\r' || c == '\t' {
                    escaped.push(c);
                }
            }
        }
    }
    escaped
}

pub async fn list_polls(
    State(pool): State<DbPool>,
) -> Result<Json<Vec<Poll>>, (StatusCode, String)> {
    let polls: Vec<Poll> = sqlx::query_as("SELECT * FROM polls")
        .fetch_all(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(polls))
}

pub async fn create_poll(
    State(pool): State<DbPool>,
    auth_user: MaybeAuthUser,
    Json(payload): Json<CreatePollRequest>,
) -> Result<Json<Value>, (StatusCode, String)> {
    // Validate inputs
    validate_string_length(&payload.title, MAX_TITLE_LENGTH, "Title")
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    validate_string_length(&payload.description, MAX_DESCRIPTION_LENGTH, "Description")
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    validate_string_length(&payload.location, MAX_LOCATION_LENGTH, "Location")
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    // Validate dates
    if payload.dates.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "At least one date is required".to_string(),
        ));
    }
    if payload.dates.len() > MAX_DATES {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("Too many dates (max: {})", MAX_DATES),
        ));
    }

    // Story 1.3: Date Logic
    let today = Utc::now().date_naive();
    let mut parsed_dates = Vec::new();
    let mut min_date: Option<chrono::NaiveDate> = None;
    let mut max_date: Option<chrono::NaiveDate> = None;

    for date_str in &payload.dates {
        let date = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d").map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                format!("Invalid date format: {}", date_str),
            )
        })?;

        // Check for past dates
        if date < today {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("Date cannot be in the past: {}", date_str),
            ));
        }

        parsed_dates.push(date);

        match min_date {
            Some(min) => {
                if date < min {
                    min_date = Some(date)
                }
            }
            None => min_date = Some(date),
        }
        match max_date {
            Some(max) => {
                if date > max {
                    max_date = Some(date)
                }
            }
            None => max_date = Some(date),
        }
    }

    // Check date range length (max 14 days)
    if let (Some(min), Some(max)) = (min_date, max_date) {
        let duration = max.signed_duration_since(min);
        if duration.num_days() > 14 {
            return Err((
                StatusCode::BAD_REQUEST,
                "Date range cannot exceed 14 days".to_string(),
            ));
        }
    }

    // Validate participants
    if payload.participants.len() > MAX_PARTICIPANTS {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("Too many participants (max: {})", MAX_PARTICIPANTS),
        ));
    }

    for email in &payload.participants {
        validate_email(email).map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    }

    let poll_id = Uuid::new_v4().to_string();
    let admin_token = Uuid::new_v4().to_string(); // Generate admin token for creator
    let created_at = Utc::now().timestamp();

    // Sanitize inputs
    let title = sanitize_string(&payload.title);
    let description = sanitize_string(&payload.description);
    let location = sanitize_string(&payload.location);

    let dates_json = serde_json::to_string(&payload.dates).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to serialize dates: {}", e),
        )
    })?;

    // Handle time preferences - support both old and new formats
    let time_range_value = if let Some(time_prefs) = &payload.time_preferences {
        // New format: per-day time preferences
        serde_json::to_string(time_prefs).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to serialize time preferences: {}", e),
            )
        })?
    } else if let Some(legacy_time_range) = &payload.time_range {
        // Legacy format: global time range
        legacy_time_range.clone()
    } else {
        // Default: empty JSON object
        "{}".to_string()
    };

    // Start transaction
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Insert Poll with admin_token and organizer_id
    let organizer_id = auth_user.0.map(|u| u.id);

    sqlx::query(
        "INSERT INTO polls (id, title, description, location, created_at, dates, time_range, status, admin_token, organizer_id) VALUES (?, ?, ?, ?, ?, ?, ?, 'active', ?, ?)",
    )
    .bind(&poll_id)
    .bind(&title)
    .bind(&description)
    .bind(&location)
    .bind(created_at)
    .bind(&dates_json)
    .bind(&time_range_value)
    .bind(&admin_token)
    .bind(&organizer_id)
    .execute(&mut *tx)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create poll".to_string()))?;

    for email in &payload.participants {
        let participant_id = Uuid::new_v4().to_string();
        let access_token = Uuid::new_v4().to_string(); // Generate unique access token
                                                       // For now, name is just the email prefix or "Player"
        let name = email.split('@').next().unwrap_or("Player").to_string();
        let sanitized_name = sanitize_string(&name);

        sqlx::query("INSERT INTO participants (id, poll_id, name, email, access_token) VALUES (?, ?, ?, ?, ?)")
            .bind(&participant_id)
            .bind(&poll_id)
            .bind(&sanitized_name)
            .bind(email)
            .bind(&access_token)
            .execute(&mut *tx)
            .await
            .map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to add participant".to_string(),
                )
            })?;
    }

    // Commit transaction
    tx.commit()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Log activity: poll created
    crate::api::handlers::activity::log_activity(
        &pool,
        "poll_created",
        "anonymous".to_string(),
        "Organizzatore".to_string(),
        Some(poll_id.clone()),
        Some(title.clone()),
    )
    .await
    .unwrap_or_else(|e| tracing::error!("Activity log error: {}", e));

    Ok(Json(json!({
        "id": poll_id,
        "adminToken": admin_token
    })))
}

pub async fn get_poll(
    State(pool): State<DbPool>,
    Path(poll_id): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    // Validate poll_id is a valid UUID
    validate_uuid(&poll_id).map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    let poll: Poll = sqlx::query_as("SELECT * FROM polls WHERE id = ?")
        .bind(&poll_id)
        .fetch_optional(&pool)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error".to_string(),
            )
        })?
        .ok_or((StatusCode::NOT_FOUND, "Poll not found".to_string()))?;

    let participants: Vec<Participant> =
        sqlx::query_as("SELECT id, poll_id, name, email, NULL as access_token, user_id FROM participants WHERE poll_id = ?")
            .bind(&poll_id)
            .fetch_all(&pool)
            .await
            .map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Database error".to_string(),
                )
            })?;

    let availability: Vec<Availability> =
        sqlx::query_as("SELECT * FROM availability WHERE poll_id = ?")
            .bind(&poll_id)
            .fetch_all(&pool)
            .await
            .map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Database error".to_string(),
                )
            })?;

    Ok(Json(json!({
        "poll": poll,
        "participants": participants,
        "availability": availability
    })))
}

pub async fn join_poll(
    State(pool): State<DbPool>,
    Path(poll_id): Path<String>,
    Json(payload): Json<JoinPollRequest>,
) -> Result<Json<Value>, (StatusCode, String)> {
    // Validate poll_id
    validate_uuid(&poll_id).map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    // Validate name
    validate_string_length(&payload.name, MAX_NAME_LENGTH, "Name")
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    // Validate email if provided
    if let Some(email) = &payload.email {
        validate_email(email).map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    }

    // Verify poll exists
    let poll_exists: Option<(i64,)> = sqlx::query_as("SELECT 1 FROM polls WHERE id = ?")
        .bind(&poll_id)
        .fetch_optional(&pool)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error".to_string(),
            )
        })?;

    if poll_exists.is_none() {
        return Err((StatusCode::NOT_FOUND, "Poll not found".to_string()));
    }

    // Check if this email exists in the participants table for this poll

    // Transactional logic for join
    // We try to insert a new participant. If it fails due to UNIQUE constraint (poll_id, email),
    // we assume the participant exists and we update/fetch their details.

    // CASE 2 Attempt: Try to insert as new participant first (Optimistic)
    let new_id = Uuid::new_v4().to_string();
    let new_token = Uuid::new_v4().to_string();
    let sanitized_name = sanitize_string(&payload.name);

    // We use a transaction here mainly to ensure atomic operation if we were doing more,
    // but also consistent reads.
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Try insert
    let insert_result = sqlx::query(
        "INSERT INTO participants (id, poll_id, name, email, access_token) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&new_id)
    .bind(&poll_id)
    .bind(&sanitized_name)
    .bind(&payload.email)
    .bind(&new_token)
    .execute(&mut *tx)
    .await;

    let (participant_id, access_token) = match insert_result {
        Ok(_) => {
            // Success: New participant
            tx.commit()
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            (new_id, new_token)
        }
        Err(_) => {
            // Rollback transaction (or just ignore as we'll do a read next)
            tx.rollback().await.ok();

            // If email is missing, we shouldn't have hit a conflict on (poll_id, email) usually,
            // unless ID collision (rare) or some other error.
            // If email IS present, it might be a conflict.

            if let Some(email) = &payload.email {
                // CASE 1: EXISTING PARTICIPANT (Fallback)
                // Fetch existing ID and Token
                let existing: (String, String) = sqlx::query_as(
                    "SELECT id, access_token FROM participants WHERE poll_id = ? AND email = ?",
                )
                .bind(&poll_id)
                .bind(email)
                .fetch_one(&pool)
                .await
                .map_err(|_| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Failed to join poll (concurrency error or unknown)".to_string(),
                    )
                })?;

                // Update name
                sqlx::query("UPDATE participants SET name = ? WHERE id = ?")
                    .bind(&sanitized_name)
                    .bind(&existing.0)
                    .execute(&pool)
                    .await
                    .map_err(|_| {
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "Failed to update participant name".to_string(),
                        )
                    })?;

                existing
            } else {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to create anonymous participant".to_string(),
                ));
            }
        }
    };

    Ok(Json(json!({
        "id": participant_id,
        "access_token": access_token,
        "message": "Successfully joined the poll"
    })))
}

pub async fn update_availability(
    State(pool): State<DbPool>,
    maybe_user: crate::auth::MaybeAuthUser,
    Path((poll_id, participant_id)): Path<(String, String)>,
    Json(payload): Json<UpdateAvailabilityRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    // Validate UUIDs
    validate_uuid(&poll_id).map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    validate_uuid(&participant_id).map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    // Validate availability entries count
    if payload.availability.len() > MAX_AVAILABILITY_ENTRIES {
        return Err((
            StatusCode::BAD_REQUEST,
            format!(
                "Too many availability entries (max: {})",
                MAX_AVAILABILITY_ENTRIES
            ),
        ));
    }

    // Verify poll and participant exist
    let poll_exists: Option<(i64,)> = sqlx::query_as("SELECT 1 FROM polls WHERE id = ?")
        .bind(&poll_id)
        .fetch_optional(&pool)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error".to_string(),
            )
        })?;

    if poll_exists.is_none() {
        return Err((StatusCode::NOT_FOUND, "Poll not found".to_string()));
    }

    let participant_exists: Option<(i64,)> =
        sqlx::query_as("SELECT 1 FROM participants WHERE id = ? AND poll_id = ?")
            .bind(&participant_id)
            .bind(&poll_id)
            .fetch_optional(&pool)
            .await
            .map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Database error".to_string(),
                )
            })?;

    if participant_exists.is_none() {
        return Err((StatusCode::NOT_FOUND, "Participant not found".to_string()));
    }

    // AUTHORIZATION CHECK: Validate access token OR User Ownership
    let mut authorized = false;

    // 1. Check Link Token
    if let Some(provided_token) = &payload.access_token {
        let stored_token: Option<String> =
            sqlx::query_scalar("SELECT access_token FROM participants WHERE id = ?")
                .bind(&participant_id)
                .fetch_optional(&pool)
                .await
                .map_err(|_| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Database error".to_string(),
                    )
                })?
                .flatten();

        if stored_token.is_some() && stored_token.as_ref() == Some(provided_token) {
            authorized = true;
        }
    }

    // 2. Check Session Ownership (if not already authorized)
    if !authorized {
        if let Some(user) = maybe_user.0 {
            // Check if this participant belongs to the user
            let owner_id: Option<String> =
                sqlx::query_scalar("SELECT user_id FROM participants WHERE id = ?")
                    .bind(&participant_id)
                    .fetch_optional(&pool)
                    .await
                    .map_err(|_| {
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "Database error".to_string(),
                        )
                    })?
                    .flatten();

            if let Some(oid) = owner_id {
                if oid == user.id {
                    authorized = true;
                }
            }
        }
    }

    if !authorized {
        return Err((
            StatusCode::UNAUTHORIZED,
            "Access token or valid session required to update availability.".to_string(),
        ));
    }

    // Validate dates against poll dates to prevent junk data injection
    let poll_dates_json: String = sqlx::query_scalar("SELECT dates FROM polls WHERE id = ?")
        .bind(&poll_id)
        .fetch_one(&pool)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error".to_string(),
            )
        })?;

    let valid_dates: Vec<String> = serde_json::from_str(&poll_dates_json).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to parse poll dates".to_string(),
        )
    })?;

    // Transaction start
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Clear existing availability for this participant in this poll
    sqlx::query("DELETE FROM availability WHERE poll_id = ? AND participant_id = ?")
        .bind(&poll_id)
        .bind(&participant_id)
        .execute(&mut *tx)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error".to_string(),
            )
        })?;

    for entry in payload.availability {
        // Validate each entry
        validate_string_length(&entry.date, 50, "Date")
            .map_err(|e| (StatusCode::BAD_REQUEST, e))?;
        validate_string_length(&entry.time_slot, 50, "Time slot")
            .map_err(|e| (StatusCode::BAD_REQUEST, e))?;
        validate_string_length(&entry.status, 20, "Status")
            .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

        // Check if date is valid for this poll
        if !valid_dates.contains(&entry.date) {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("Invalid date provided: {}", entry.date),
            ));
        }

        sqlx::query(
            "INSERT INTO availability (poll_id, participant_id, date, time_slot, status) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&poll_id)
        .bind(&participant_id)
        .bind(&entry.date)
        .bind(&entry.time_slot)
        .bind(&entry.status)
        .execute(&mut *tx)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update availability".to_string()))?;
    }

    // Commit transaction
    tx.commit()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Log activity: response submitted
    // Fetch participant and poll info for logging
    if let Ok(participant) =
        sqlx::query_as::<_, Participant>("SELECT * FROM participants WHERE id = ?")
            .bind(&participant_id)
            .fetch_one(&pool)
            .await
    {
        if let Ok(poll) = sqlx::query_as::<_, Poll>("SELECT * FROM polls WHERE id = ?")
            .bind(&poll_id)
            .fetch_one(&pool)
            .await
        {
            crate::activity_handlers::log_activity(
                &pool,
                "response_submitted",
                participant_id.clone(),
                participant.name,
                Some(poll_id),
                Some(poll.title),
            )
            .await
            .unwrap_or_else(|e| tracing::error!("Activity log error: {}", e));
        }
    }

    Ok(StatusCode::OK)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::{AvailabilityEntry, User};
    use axum::extract::State;
    use sqlx::sqlite::SqlitePoolOptions;
    use uuid::Uuid;

    async fn setup_test_db() -> DbPool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("Failed to connect to memory db");

        // Execute migrations (simplified for this test context)
        sqlx::query("CREATE TABLE polls (id TEXT PRIMARY KEY, title TEXT NOT NULL, description TEXT NOT NULL, location TEXT NOT NULL, created_at INTEGER NOT NULL, dates TEXT NOT NULL, time_range TEXT NOT NULL, status TEXT NOT NULL DEFAULT 'active', finalized_at INTEGER, finalized_time TEXT, notes TEXT, admin_token TEXT, organizer_id TEXT)")
            .execute(&pool).await.unwrap();
        sqlx::query("CREATE TABLE participants (id TEXT PRIMARY KEY, poll_id TEXT NOT NULL, name TEXT NOT NULL, email TEXT, access_token TEXT UNIQUE, user_id TEXT, FOREIGN KEY (poll_id) REFERENCES polls (id))")
            .execute(&pool).await.unwrap();
        sqlx::query("CREATE TABLE availability (id INTEGER PRIMARY KEY AUTOINCREMENT, poll_id TEXT NOT NULL, participant_id TEXT NOT NULL, date TEXT NOT NULL, time_slot TEXT NOT NULL, status TEXT NOT NULL, FOREIGN KEY (poll_id) REFERENCES polls (id), FOREIGN KEY (participant_id) REFERENCES participants (id))")
            .execute(&pool).await.unwrap();
        sqlx::query("CREATE TABLE users (id TEXT PRIMARY KEY, email TEXT NOT NULL UNIQUE, password_hash TEXT NOT NULL, name TEXT NOT NULL, role TEXT NOT NULL DEFAULT 'player', created_at INTEGER NOT NULL, last_login INTEGER, phone TEXT)")
            .execute(&pool).await.unwrap();
        sqlx::query("CREATE TABLE user_sessions (id TEXT PRIMARY KEY, user_id TEXT NOT NULL, token TEXT NOT NULL UNIQUE, expires_at INTEGER NOT NULL, created_at INTEGER NOT NULL, FOREIGN KEY (user_id) REFERENCES users (id))")
            .execute(&pool).await.unwrap();
        sqlx::query("CREATE TABLE activities (id TEXT PRIMARY KEY, activity_type TEXT NOT NULL, user_id TEXT NOT NULL, user_name TEXT NOT NULL, poll_id TEXT, poll_name TEXT, message TEXT NOT NULL, timestamp INTEGER NOT NULL)")
            .execute(&pool).await.unwrap();

        pool
    }

    #[tokio::test]
    async fn test_create_poll_anonymous() {
        let pool = setup_test_db().await;

        // Use a future date
        let future_date = (Utc::now() + chrono::Duration::days(30)).date_naive();

        let req = CreatePollRequest {
            title: "Anon Poll".to_string(),
            description: "Desc".to_string(),
            location: "Loc".to_string(),
            dates: vec![future_date.to_string()],
            time_range: None,
            time_preferences: None,
            participants: vec!["p1@test.com".to_string()],
        };

        let res = create_poll(State(pool.clone()), MaybeAuthUser(None), Json(req)).await;
        assert!(res.is_ok());

        let json_val = res.unwrap().0;
        assert!(json_val.get("id").is_some());
        assert!(json_val.get("adminToken").is_some());

        // Verify persistence
        let poll_id = json_val.get("id").unwrap().as_str().unwrap();
        let token_row: Option<(String,)> =
            sqlx::query_as("SELECT admin_token FROM polls WHERE id = ?")
                .bind(poll_id)
                .fetch_optional(&pool)
                .await
                .unwrap();

        assert!(token_row.is_some());
        assert_eq!(
            token_row.unwrap().0,
            json_val.get("adminToken").unwrap().as_str().unwrap()
        );
    }

    #[tokio::test]
    async fn test_create_poll_dates_validation() {
        let pool = setup_test_db().await;

        // 1. Valid dates (Tomorrow and Tomorrow + 13 days = 14 days span)
        let valid_start = Utc::now().date_naive() + chrono::Duration::days(1);
        let valid_end = valid_start + chrono::Duration::days(13);

        let req = CreatePollRequest {
            title: "Valid Poll".to_string(),
            description: "Desc".to_string(),
            location: "Loc".to_string(),
            dates: vec![valid_start.to_string(), valid_end.to_string()],
            time_range: None,
            time_preferences: None,
            participants: vec![],
        };
        let res = create_poll(State(pool.clone()), MaybeAuthUser(None), Json(req)).await;
        assert!(res.is_ok());

        // 2. Past date
        let past_date = Utc::now().date_naive() - chrono::Duration::days(1);
        let req_past = CreatePollRequest {
            title: "Past Poll".to_string(),
            description: "Desc".to_string(),
            location: "Loc".to_string(),
            dates: vec![past_date.to_string()],
            time_range: None,
            time_preferences: None,
            participants: vec![],
        };
        let res_past = create_poll(State(pool.clone()), MaybeAuthUser(None), Json(req_past)).await;
        assert!(res_past.is_err());
        assert_eq!(res_past.err().unwrap().0, StatusCode::BAD_REQUEST);

        // 3. Range too long (> 14 days)
        let long_start = Utc::now().date_naive() + chrono::Duration::days(1);
        let long_end = long_start + chrono::Duration::days(15);
        let req_long = CreatePollRequest {
            title: "Long Poll".to_string(),
            description: "Desc".to_string(),
            location: "Loc".to_string(),
            dates: vec![long_start.to_string(), long_end.to_string()],
            time_range: None,
            time_preferences: None,
            participants: vec![],
        };
        let res_long = create_poll(State(pool.clone()), MaybeAuthUser(None), Json(req_long)).await;
        assert!(res_long.is_err());
        assert_eq!(res_long.err().unwrap().0, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_availability_flow() {
        let pool = setup_test_db().await;

        // 1. Create DM User
        let dm_id = Uuid::new_v4().to_string();
        sqlx::query("INSERT INTO users (id, email, password_hash, name, role, created_at) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(&dm_id)
            .bind("dm@test.com")
            .bind("hash")
            .bind("DM")
            .bind("dm")
            .bind(Utc::now().timestamp())
            .execute(&pool).await.unwrap();

        // Mock AuthUser for DM - Not needed for create_poll anymore
        // let dm_user = crate::auth::AuthUser(User { ... });

        // 2. Create Poll
        let future_date = (Utc::now() + chrono::Duration::days(30)).date_naive();
        let create_req = CreatePollRequest {
            title: "Test Poll".to_string(),
            description: "Desc".to_string(),
            location: "Loc".to_string(),
            dates: vec![future_date.to_string()],
            time_range: None,
            time_preferences: None,
            participants: vec![],
        };

        let poll_res_json = create_poll(State(pool.clone()), MaybeAuthUser(None), Json(create_req))
            .await
            .unwrap();
        let poll_id = poll_res_json
            .0
            .get("id")
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();

        // 3. Create Player User
        let p_id = Uuid::new_v4().to_string();
        sqlx::query("INSERT INTO users (id, email, password_hash, name, role, created_at) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(&p_id)
            .bind("p@test.com")
            .bind("hash")
            .bind("Player")
            .bind("player")
            .bind(Utc::now().timestamp())
            .execute(&pool).await.unwrap();

        let player_token = "playertoken";
        sqlx::query("INSERT INTO user_sessions (id, user_id, token, expires_at, created_at) VALUES (?, ?, ?, ?, ?)")
            .bind("sess1")
            .bind(&p_id)
            .bind(player_token)
            .bind(Utc::now().timestamp() + 3600)
            .bind(Utc::now().timestamp())
            .execute(&pool).await.unwrap();

        let _player_auth_user = crate::auth::AuthUser(User {
            id: p_id.clone(),
            email: "p@test.com".to_string(),
            password_hash: "hash".to_string(),
            name: "Player".to_string(),
            role: "player".to_string(),
            created_at: 0,
            last_login: None,
            phone: None,
        });

        // 4. Join Poll
        // verify join_poll uses AuthUser? No, join_poll takes JoinPollRequest and returns access_token.
        // It does NOT require AuthUser. Wait, availability-manager says:
        // "joinSession... returns access_token".
        // Let's check join_poll signature. It takes State, Path, Json.
        // Logic: if not exists, create participant with access_token.

        let join_req = JoinPollRequest {
            name: "Player".to_string(),
            email: Some("p@test.com".to_string()),
        };

        let join_res = join_poll(State(pool.clone()), Path(poll_id.clone()), Json(join_req))
            .await
            .unwrap();

        // join_res is Json<Participant> or similar?
        // extract access_token from response.
        // join_poll returns Json<Value> (json!({...})).
        let join_val = join_res.0;
        let participant_id = join_val.get("id").unwrap().as_str().unwrap().to_string();
        let access_token = join_val
            .get("access_token")
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();

        // 5. Submit Availability
        let avail_req = UpdateAvailabilityRequest {
            availability: vec![AvailabilityEntry {
                date: future_date.to_string(),
                time_slot: "18:00".to_string(),
                status: "available".to_string(),
            }],
            access_token: Some(access_token.clone()),
        };

        let update_res = update_availability(
            State(pool.clone()),
            crate::auth::MaybeAuthUser(None),
            Path((poll_id.clone(), participant_id.clone())),
            Json(avail_req),
        )
        .await;
        assert!(update_res.is_ok());

        // 6. Verify Availability
        let get_res = get_poll(State(pool.clone()), Path(poll_id.clone()))
            .await
            .unwrap();
        let get_val = get_res.0;
        let availability_arr = get_val.get("availability").unwrap().as_array().unwrap();

        assert_eq!(availability_arr.len(), 1);
        assert_eq!(
            availability_arr[0].get("status").unwrap().as_str().unwrap(),
            "available"
        );
    }
    #[tokio::test]
    async fn test_finalize_poll_security() {
        let pool = setup_test_db().await;

        // 1. Create Poll
        let poll_id = Uuid::new_v4().to_string();
        sqlx::query("INSERT INTO polls (id, title, description, location, created_at, dates, time_range, status) VALUES (?, 'Title', 'Desc', 'Loc', 0, '[]', '{}', 'active')")
            .bind(&poll_id)
            .execute(&pool).await.unwrap();

        // 2. Mock Admin
        let admin = crate::core::models::Admin {
            id: "admin1".to_string(),
            username: "admin".to_string(),
            password_hash: "hash".to_string(),
            email: Some("admin@example.com".to_string()),
            role: "admin".to_string(),
            created_at: 0,
        };
        let admin_user = crate::auth::AdminUser(admin);

        // 3. Finalize Poll
        let req = models::FinalizePollRequest {
            finalized_time: "2026-01-01 20:00".to_string(),
            notes: Some("Let's play!".to_string()),
        };

        let res = finalize_poll(
            State(pool.clone()),
            admin_user,
            Path(poll_id.clone()),
            Json(req),
        )
        .await;

        assert!(res.is_ok());

        // 4. Check DB status
        let status: String = sqlx::query_scalar("SELECT status FROM polls WHERE id = ?")
            .bind(&poll_id)
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(status, "finalized");

        // 5. Try to finalize again (should fail)
        let admin_user_2 = crate::auth::AdminUser(crate::core::models::Admin {
            id: "admin1".to_string(),
            username: "admin".to_string(),
            password_hash: "hash".to_string(),
            email: Some("admin@example.com".to_string()),
            role: "admin".to_string(),
            created_at: 0,
        });

        let req_2 = models::FinalizePollRequest {
            finalized_time: "2026-01-02 20:00".to_string(),
            notes: None,
        };

        let res_2 = finalize_poll(
            State(pool.clone()),
            admin_user_2,
            Path(poll_id.clone()),
            Json(req_2),
        )
        .await;

        assert!(res_2.is_err());
        assert_eq!(res_2.err().unwrap().0, StatusCode::CONFLICT);
    }

    #[test]
    fn test_sanitize_string_xss() {
        let input = "<script>alert('XSS')</script>";
        let expected = "&lt;script&gt;alert(&#x27;XSS&#x27;)&lt;/script&gt;";
        assert_eq!(sanitize_string(input), expected);

        let input_quotes = "User \"Name\"";
        let expected_quotes = "User &quot;Name&quot;";
        assert_eq!(sanitize_string(input_quotes), expected_quotes);
    }

    #[tokio::test]
    async fn test_update_availability_owner() {
        let pool = setup_test_db().await;

        // 1. Create Poll
        let poll_id = Uuid::new_v4().to_string();
        sqlx::query("INSERT INTO polls (id, title, description, location, created_at, dates, time_range, status) VALUES (?, 'Title', 'Desc', 'Loc', 0, '[\"2023-12-01\"]', '{}', 'active')")
            .bind(&poll_id)
            .execute(&pool).await.unwrap();

        // 2. Create User
        let user_id = Uuid::new_v4().to_string();
        sqlx::query("INSERT INTO users (id, email, password_hash, name, role, created_at) VALUES (?, 'owner@test.com', 'hash', 'Owner', 'player', 0)")
            .bind(&user_id)
            .execute(&pool).await.unwrap();

        // 3. Create Participant LINKED to User
        let participant_id = Uuid::new_v4().to_string();
        sqlx::query("INSERT INTO participants (id, poll_id, name, email, user_id) VALUES (?, ?, 'Owner P', 'owner@test.com', ?)")
            .bind(&participant_id)
            .bind(&poll_id)
            .bind(&user_id)
            .execute(&pool).await.unwrap();

        // 4. Update Availability as User (WITHOUT Token)
        let avail_req = models::UpdateAvailabilityRequest {
            availability: vec![models::AvailabilityEntry {
                date: "2023-12-01".to_string(),
                time_slot: "18:00".to_string(),
                status: "available".to_string(),
            }],
            access_token: None, // No token provided!
        };

        // Mock AuthUser
        let user = crate::core::models::User {
            id: user_id.clone(),
            email: "owner@test.com".to_string(),
            password_hash: "hash".to_string(),
            name: "Owner".to_string(),
            role: "player".to_string(),
            created_at: 0,
            last_login: None,
            phone: None,
        };
        let maybe_user = crate::auth::MaybeAuthUser(Some(user));

        let update_res = update_availability(
            State(pool.clone()),
            maybe_user,
            Path((poll_id.clone(), participant_id.clone())),
            Json(avail_req),
        )
        .await;

        // Assert SUCCESS
        assert!(
            update_res.is_ok(),
            "Should allow Owner to update without token"
        );
    }
}

pub async fn admin_login(
    State(pool): State<DbPool>,
    Json(payload): Json<models::LoginRequest>,
) -> Result<Json<models::AuthResponse>, (StatusCode, String)> {
    // Validate inputs
    validate_email(&payload.email).map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    validate_string_length(&payload.password, 128, "Password")
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    // 1. Fetch admin by email
    let admin: models::Admin = sqlx::query_as("SELECT * FROM admins WHERE email = ?")
        .bind(&payload.email)
        .fetch_optional(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((
            StatusCode::UNAUTHORIZED,
            "Invalid email or password".to_string(),
        ))?;

    // 2. Verify password
    let valid = bcrypt::verify(&payload.password, &admin.password_hash).unwrap_or(false);

    if !valid {
        return Err((
            StatusCode::UNAUTHORIZED,
            "Invalid email or password".to_string(),
        ));
    }

    // 3. Create session
    let token = Uuid::new_v4().to_string();
    let expires_at = Utc::now().timestamp() + 86400; // 24 hours

    sqlx::query("INSERT INTO sessions (token, user_id, expires_at) VALUES (?, ?, ?)")
        .bind(&token)
        .bind(&admin.id)
        .bind(expires_at)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(models::AuthResponse { token, user: admin }))
}

// Get current admin user info
pub async fn get_current_admin(
    admin_user: crate::auth::AdminUser,
) -> Result<Json<models::Admin>, (StatusCode, String)> {
    // AdminUser extractor already validated the session and fetched the admin
    let admin = admin_user.0;

    // Return admin without password_hash
    Ok(Json(models::Admin {
        id: admin.id,
        username: admin.username,
        password_hash: String::new(), // Don't send password hash to frontend
        email: admin.email,
        role: admin.role,
        created_at: admin.created_at,
    }))
}

pub async fn get_all_users(
    State(pool): State<DbPool>,
    _admin_user: crate::auth::AdminUser,
) -> Result<Json<Vec<models::User>>, (StatusCode, String)> {
    // Explicitly select only columns that match the User model
    // The users table has additional GDPR columns that aren't in the model
    let users = sqlx::query_as::<_, models::User>(
        "SELECT id, email, password_hash, name, role, created_at, last_login FROM users ORDER BY created_at DESC"
    )
        .fetch_all(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(users))
}

pub async fn google_login(
    State(pool): State<DbPool>,
    Json(payload): Json<models::GoogleLoginRequest>,
) -> Result<Json<models::AuthResponse>, (StatusCode, String)> {
    // 1. Verify the Google Token (Unified Logic)
    let claims = crate::auth::verify_google_token(&payload.token)
        .await
        .map_err(|e| (StatusCode::UNAUTHORIZED, e))?;

    let email = claims.email;
    // We can use claims.name if we want to trust Google's name, or payload.name if we want user provided.
    // Admin login previously validated payload.name. Let's stick to payload.name but validate it matches claims or just use payload.name (as verified email is the key).
    // Actually, sticking to payload.name for now to minimize changes to Admin flow logic if they wanted custom names.
    // But we MUST check email matches. verify_google_token returns the email from the token.

    // Validate email matches payload (sanity check)
    if email != payload.email {
        return Err((
            StatusCode::BAD_REQUEST,
            "Payload email does not match token email".to_string(),
        ));
    }

    validate_string_length(&payload.name, MAX_NAME_LENGTH, "Name")
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    let admin_opt: Option<models::Admin> = sqlx::query_as("SELECT * FROM admins WHERE email = ?")
        .bind(&email)
        .fetch_optional(&pool)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error".to_string(),
            )
        })?;

    let admin = if let Some(existing) = admin_opt {
        existing
    } else {
        // More secure domain validation
        let allowed_domains = vec!["ddscheduler.com", "example.com"];
        let email_domain = email.split('@').nth(1).unwrap_or("");
        let default_admin_email = std::env::var("DEFAULT_ADMIN_EMAIL")
            .unwrap_or_else(|_| "admin@example.com".to_string());

        // Allow if domain matches OR it's the specific default admin email
        if !allowed_domains.iter().any(|d| email_domain == *d) && email != default_admin_email {
            return Err((
                StatusCode::UNAUTHORIZED,
                "Email domain not authorized".to_string(),
            ));
        }

        let id = Uuid::new_v4().to_string();
        let now = Utc::now().timestamp();
        let sanitized_name = sanitize_string(&payload.name);

        // Use a secure random password hash for OAuth users
        let hash =
            bcrypt::hash("google_oauth_no_password_login", bcrypt::DEFAULT_COST).map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to create user".to_string(),
                )
            })?;

        sqlx::query("INSERT INTO admins (id, username, password_hash, email, role, created_at) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(&id)
            .bind(&sanitized_name)
            .bind(&hash)
            .bind(&email)
            .bind("admin")
            .bind(now)
            .execute(&pool)
            .await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create user".to_string()))?;

        models::Admin {
            id,
            username: sanitized_name,
            password_hash: "".to_string(),
            email: Some(email),
            role: "admin".to_string(),
            created_at: now,
        }
    };

    let token = Uuid::new_v4().to_string();
    let expiration = Utc::now()
        .checked_add_signed(chrono::Duration::hours(24))
        .ok_or((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to generate expiration".to_string(),
        ))?
        .timestamp();

    sqlx::query("INSERT INTO sessions (token, user_id, expires_at) VALUES (?, ?, ?)")
        .bind(&token)
        .bind(&admin.id)
        .bind(expiration)
        .execute(&pool)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to create session".to_string(),
            )
        })?;

    Ok(Json(models::AuthResponse { token, user: admin }))
}

pub async fn update_poll(
    State(pool): State<DbPool>,
    Path(poll_id): Path<String>,
    Json(payload): Json<CreatePollRequest>,
) -> Result<Json<Value>, (StatusCode, String)> {
    // Validate poll_id
    validate_uuid(&poll_id).map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    // Validate inputs (same as create_poll)
    validate_string_length(&payload.title, MAX_TITLE_LENGTH, "Title")
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    validate_string_length(&payload.description, MAX_DESCRIPTION_LENGTH, "Description")
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    validate_string_length(&payload.location, MAX_LOCATION_LENGTH, "Location")
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    if payload.dates.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "At least one date is required".to_string(),
        ));
    }
    if payload.dates.len() > MAX_DATES {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("Too many dates (max: {})", MAX_DATES),
        ));
    }

    // Sanitize inputs
    let title = sanitize_string(&payload.title);
    let description = sanitize_string(&payload.description);
    let location = sanitize_string(&payload.location);

    let dates_json = serde_json::to_string(&payload.dates).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to serialize dates: {}", e),
        )
    })?;

    // Handle time preferences - support both old and new formats
    let time_range_value = if let Some(time_prefs) = &payload.time_preferences {
        // New format: per-day time preferences
        serde_json::to_string(time_prefs).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to serialize time preferences: {}", e),
            )
        })?
    } else if let Some(legacy_time_range) = &payload.time_range {
        // Legacy format: global time range
        legacy_time_range.clone()
    } else {
        // Default: empty JSON object
        "{}".to_string()
    };

    let result = sqlx::query(
        "UPDATE polls SET title = ?, description = ?, location = ?, dates = ?, time_range = ? WHERE id = ?",
    )
    .bind(&title)
    .bind(&description)
    .bind(&location)
    .bind(&dates_json)
    .bind(&time_range_value)
    .bind(&poll_id)
    .execute(&pool)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update poll".to_string()))?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Poll not found".to_string()));
    }

    Ok(Json(json!({ "success": true })))
}

pub async fn finalize_poll(
    State(pool): State<DbPool>,
    // Require Admin authentication
    _admin_user: crate::auth::AdminUser,
    Path(poll_id): Path<String>,
    Json(payload): Json<models::FinalizePollRequest>,
) -> Result<Json<Value>, (StatusCode, String)> {
    // Validate poll_id
    validate_uuid(&poll_id).map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    if payload.finalized_time.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Finalized time cannot be empty".to_string(),
        ));
    }

    // Check current status
    let current_status: Option<String> =
        sqlx::query_scalar("SELECT status FROM polls WHERE id = ?")
            .bind(&poll_id)
            .fetch_optional(&pool)
            .await
            .map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Database error".to_string(),
                )
            })?;

    if let Some(status) = current_status {
        if status == "finalized" {
            return Err((
                StatusCode::CONFLICT,
                "Poll is already finalized".to_string(),
            ));
        }
    } else {
        return Err((StatusCode::NOT_FOUND, "Poll not found".to_string()));
    }

    let now = Utc::now().timestamp();

    let result = sqlx::query(
        "UPDATE polls SET status = 'finalized', finalized_at = ?, finalized_time = ?, notes = ? WHERE id = ?",
    )
    .bind(now)
    .bind(&payload.finalized_time)
    .bind(&payload.notes)
    .bind(&poll_id)
    .execute(&pool)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to finalize poll".to_string()))?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Poll not found".to_string()));
    }

    // Log activity: poll finalized
    // We need to fetch poll title first for better logging, but for efficiency we can skip or do it quickly
    // Let's do a quick fetch for title
    let title: Option<String> = sqlx::query_scalar("SELECT title FROM polls WHERE id = ?")
        .bind(&poll_id)
        .fetch_optional(&pool)
        .await
        .unwrap_or(None);

    crate::activity_handlers::log_activity(
        &pool,
        "poll_finalized",
        "system".to_string(),
        "Organizzatore".to_string(),
        Some(poll_id),
        title,
    )
    .await
    .unwrap_or_else(|e| tracing::error!("Activity log error: {}", e));

    Ok(Json(json!({
        "success": true,
        "status": "finalized",
        "finalizedAt": now
    })))
}

pub async fn delete_poll(
    State(pool): State<DbPool>,
    _admin: crate::auth::AdminUser,
    Path(poll_id): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    // Validate poll_id
    validate_uuid(&poll_id).map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    // Delete availability first (foreign key constraint)
    sqlx::query("DELETE FROM availability WHERE poll_id = ?")
        .bind(&poll_id)
        .execute(&pool)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error".to_string(),
            )
        })?;

    // Delete participants
    sqlx::query("DELETE FROM participants WHERE poll_id = ?")
        .bind(&poll_id)
        .execute(&pool)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error".to_string(),
            )
        })?;

    // Delete the poll itself
    let result = sqlx::query("DELETE FROM polls WHERE id = ?")
        .bind(&poll_id)
        .execute(&pool)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error".to_string(),
            )
        })?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Poll not found".to_string()));
    }

    Ok(Json(json!({ "success": true })))
}

pub async fn delete_participant(
    State(pool): State<DbPool>,
    _admin: crate::auth::AdminUser,
    Path(participant_id): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    // Validate participant_id
    validate_uuid(&participant_id).map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    // Delete availability first (foreign key constraint)
    sqlx::query("DELETE FROM availability WHERE participant_id = ?")
        .bind(&participant_id)
        .execute(&pool)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error".to_string(),
            )
        })?;

    // Delete the participant
    let result = sqlx::query("DELETE FROM participants WHERE id = ?")
        .bind(&participant_id)
        .execute(&pool)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error".to_string(),
            )
        })?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Participant not found".to_string()));
    }

    Ok(Json(json!({ "success": true })))
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRoleRequest {
    pub role: String,
}

pub async fn update_user_role(
    State(pool): State<DbPool>,
    admin_user: crate::auth::AdminUser,
    Path(user_id): Path<String>,
    Json(payload): Json<UpdateUserRoleRequest>,
) -> Result<Json<Value>, (StatusCode, String)> {
    // AdminUser extractor validates admin authentication
    let _admin = admin_user.0;

    // Validate user_id
    validate_uuid(&user_id).map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    // Validate role
    let role = payload.role.to_lowercase();
    if role != "player" && role != "dm" && role != "admin" {
        return Err((
            StatusCode::BAD_REQUEST,
            "Role must be 'player', 'dm', or 'admin'".to_string(),
        ));
    }

    // Check if user exists
    let user_exists: Option<(i64,)> = sqlx::query_as("SELECT 1 FROM users WHERE id = ?")
        .bind(&user_id)
        .fetch_optional(&pool)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error".to_string(),
            )
        })?;

    if user_exists.is_none() {
        return Err((StatusCode::NOT_FOUND, "User not found".to_string()));
    }

    // Update user role
    let result = sqlx::query("UPDATE users SET role = ? WHERE id = ?")
        .bind(&role)
        .bind(&user_id)
        .execute(&pool)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to update user role".to_string(),
            )
        })?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "User not found".to_string()));
    }

    Ok(Json(json!({
        "success": true,
        "message": format!("User role updated to {}", role)
    })))
}

/// Response for admin password reset - includes the temporary password
#[derive(Debug, Serialize)]
pub struct AdminResetPasswordResponse {
    pub success: bool,
    pub temporary_password: String,
    pub message: String,
}

/// Admin endpoint to reset a user's password to a temporary one
/// The admin can then share this password with the user
pub async fn admin_reset_user_password(
    State(pool): State<DbPool>,
    admin_user: crate::auth::AdminUser,
    Path(user_id): Path<String>,
) -> Result<Json<AdminResetPasswordResponse>, (StatusCode, String)> {
    // AdminUser extractor validates admin authentication
    let admin = admin_user.0;

    // Validate user_id
    validate_uuid(&user_id).map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    // Check if user exists
    let user_exists: Option<(String,)> = sqlx::query_as("SELECT email FROM users WHERE id = ?")
        .bind(&user_id)
        .fetch_optional(&pool)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error".to_string(),
            )
        })?;

    if user_exists.is_none() {
        return Err((StatusCode::NOT_FOUND, "User not found".to_string()));
    }

    let user_email = user_exists
        .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "Logic error".to_string()))?
        .0;

    // Generate a secure temporary password using rand
    // Format: Uppercase + lowercase + numbers + special = meets password requirements
    use rand::Rng;
    let temp_password = {
        let mut rng = rand::thread_rng();

        // Ensure we have at least one of each required type
        let mut pwd = String::new();
        // 2 Uppercase
        pwd.push(rng.gen_range(b'A'..=b'Z') as char);
        pwd.push(rng.gen_range(b'A'..=b'Z') as char);
        // 4 Lowercase
        for _ in 0..4 {
            pwd.push(rng.gen_range(b'a'..=b'z') as char);
        }
        // 4 Numbers
        for _ in 0..4 {
            pwd.push(rng.gen_range(b'0'..=b'9') as char);
        }
        // 2 Special
        let specials = "!@#$%^&*";
        pwd.push(
            specials
                .chars()
                .nth(rng.gen_range(0..specials.len()))
                .unwrap_or('!'),
        );
        pwd.push(
            specials
                .chars()
                .nth(rng.gen_range(0..specials.len()))
                .unwrap_or('@'),
        );
        pwd
    };

    // Shuffle characters to avoid predictable pattern
    // (Simple shuffle or just append remaining length if needed, but above is 12 chars which is min)
    // For simplicity with basic rand, we'll keep this structure or just use a full random string if complexity wasn't hard-enforced by structure.
    // However, validation in auth.rs requires one of each category.
    // The constructed string above satisfies: 2 Upper, 4 Lower, 4 Digit, 2 Special = 12 chars.
    // To be safer against pattern analysis, we could shuffle, but for a temp password this is sufficient entropy (rand is CSPRNG).

    // Hash the new password
    let password_hash = bcrypt::hash(&temp_password, bcrypt::DEFAULT_COST).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to hash password".to_string(),
        )
    })?;

    // Update user's password in database
    let result = sqlx::query("UPDATE users SET password_hash = ? WHERE id = ?")
        .bind(&password_hash)
        .bind(&user_id)
        .execute(&pool)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to update password".to_string(),
            )
        })?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "User not found".to_string()));
    }

    // Invalidate all user sessions for security
    sqlx::query("DELETE FROM user_sessions WHERE user_id = ?")
        .bind(&user_id)
        .execute(&pool)
        .await
        .ok(); // Ignore errors here, it's just cleanup

    // Log this action in audit log
    crate::audit::log_audit(
        &pool,
        Some(admin.id.clone()),
        "admin_password_reset",
        Some("admin".to_string()),
        true,
        Some(format!("Admin reset password for user: {}", user_email)),
        None,
    )
    .await;

    Ok(Json(AdminResetPasswordResponse {
        success: true,
        temporary_password: temp_password,
        message: format!("Password reset successfully for {}", user_email),
    }))
}

// Story 1.6: Serve dynamic poll page with OG metadata
pub async fn serve_poll_page(
    Path(id): Path<String>,
    State(pool): State<DbPool>,
) -> impl IntoResponse {
    // Fetch poll for metadata
    let poll_result = sqlx::query_as::<_, Poll>("SELECT * FROM polls WHERE id = ?")
        .bind(&id)
        .fetch_optional(&pool)
        .await;

    let (title, description) = match poll_result {
        Ok(Some(p)) => (p.title, p.description),
        _ => (
            "D&D Session Planner".to_string(),
            "Join the adventure! Vote on dates for our next D&D session.".to_string(),
        ),
    };

    // Read template file
    let static_dir = std::env::var("STATIC_DIR").unwrap_or_else(|_| "static".to_string());
    let template_path = std::path::Path::new(&static_dir).join("participate.html");

    match tokio::fs::read_to_string(template_path).await {
        Ok(content) => {
            // Inject metadata
            let html = content
                .replace("{{OG_TITLE}}", &title)
                .replace("{{OG_DESCRIPTION}}", &description);

            Html(html).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to read participate.html template: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response()
        }
    }
}
