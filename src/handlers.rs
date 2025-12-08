use crate::{
    db::DbPool,
    models,
    models::{
        Availability, CreatePollRequest, JoinPollRequest, Participant, Poll,
        UpdateAvailabilityRequest,
    },
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use serde::Deserialize;
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
    // Remove null bytes and control characters
    s.chars()
        .filter(|c| !c.is_control() || *c == '\n' || *c == '\r' || *c == '\t')
        .collect()
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
    auth_user: crate::auth::AuthUser,
    Json(payload): Json<CreatePollRequest>,
) -> Result<Json<Value>, (StatusCode, String)> {
    // Enforce DM role
    if auth_user.0.role != "dm" {
        return Err((
            StatusCode::FORBIDDEN,
            "Only Dungeon Masters can create sessions".to_string(),
        ));
    }

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

    sqlx::query(
        "INSERT INTO polls (id, title, description, location, created_at, dates, time_range) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&poll_id)
    .bind(&title)
    .bind(&description)
    .bind(&location)
    .bind(created_at)
    .bind(&dates_json)
    .bind(&time_range_value)
    .execute(&pool)
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
            .execute(&pool)
            .await
            .map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to add participant".to_string(),
                )
            })?;
    }

    // Log activity: poll created
    crate::activity_handlers::log_activity(
        &pool,
        "poll_created",
        "system".to_string(), // TODO: Use real user_id when auth is implemented
        "Organizzatore".to_string(), // TODO: Use real user name
        Some(poll_id.clone()),
        Some(title.clone()),
    )
    .await
    .unwrap_or_else(|e| tracing::error!("Activity log error: {}", e));

    Ok(Json(json!({ "id": poll_id })))
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
        sqlx::query_as("SELECT * FROM participants WHERE poll_id = ?")
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

    // Email is now required for authorization
    let email = payload.email.as_ref().ok_or((
        StatusCode::BAD_REQUEST,
        "Email is required to join this poll".to_string(),
    ))?;

    // Validate email
    validate_email(email).map_err(|e| (StatusCode::BAD_REQUEST, e))?;

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
    let existing_participant: Option<(String, String)> =
        sqlx::query_as("SELECT id, email FROM participants WHERE poll_id = ? AND email = ?")
            .bind(&poll_id)
            .bind(email)
            .fetch_optional(&pool)
            .await
            .map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Database error".to_string(),
                )
            })?;

    let (participant_id, access_token) = if let Some((id, _)) = existing_participant {
        // CASE 1: EXISTING PARTICIPANT (Claiming spot)
        // Update the participant's name
        let sanitized_name = sanitize_string(&payload.name);

        sqlx::query("UPDATE participants SET name = ? WHERE id = ?")
            .bind(&sanitized_name)
            .bind(&id)
            .execute(&pool)
            .await
            .map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to update participant".to_string(),
                )
            })?;

        // Retrieve existing access token
        let token: String =
            sqlx::query_scalar("SELECT access_token FROM participants WHERE id = ?")
                .bind(&id)
                .fetch_one(&pool)
                .await
                .map_err(|_| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Failed to retrieve access token".to_string(),
                    )
                })?;

        (id, token)
    } else {
        // CASE 2: NEW PARTICIPANT (Joining open poll)
        let new_id = Uuid::new_v4().to_string();
        let new_token = Uuid::new_v4().to_string();
        let sanitized_name = sanitize_string(&payload.name);

        sqlx::query(
            "INSERT INTO participants (id, poll_id, name, email, access_token) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&new_id)
        .bind(&poll_id)
        .bind(&sanitized_name)
        .bind(email)
        .bind(&new_token)
        .execute(&pool)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to join poll".to_string(),
            )
        })?;

        (new_id, new_token)
    };

    Ok(Json(json!({
        "id": participant_id,
        "access_token": access_token,
        "message": "Successfully joined the poll"
    })))
}

pub async fn update_availability(
    State(pool): State<DbPool>,
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

    // AUTHORIZATION CHECK: Validate access token
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

        if stored_token.is_none() || stored_token.as_ref() != Some(provided_token) {
            return Err((
                StatusCode::FORBIDDEN,
                "Invalid access token. You are not authorized to update this participant's availability.".to_string(),
            ));
        }
    } else {
        return Err((
            StatusCode::UNAUTHORIZED,
            "Access token is required to update availability.".to_string(),
        ));
    }

    // Transaction might be better here, but keeping it simple for now
    // First, clear existing availability for this participant in this poll
    sqlx::query("DELETE FROM availability WHERE poll_id = ? AND participant_id = ?")
        .bind(&poll_id)
        .bind(&participant_id)
        .execute(&pool)
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

        sqlx::query(
            "INSERT INTO availability (poll_id, participant_id, date, time_slot, status) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&poll_id)
        .bind(&participant_id)
        .bind(&entry.date)
        .bind(&entry.time_slot)
        .bind(&entry.status)
        .execute(&pool)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update availability".to_string()))?;
    }

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
    // CRITICAL SECURITY WARNING: This implementation does NOT verify the Google OAuth token!
    // In production, you MUST verify the token with Google's API before trusting it.
    // See: https://developers.google.com/identity/sign-in/web/backend-auth

    // TODO: Implement proper Google OAuth token verification:
    // 1. Send token to Google's tokeninfo endpoint
    // 2. Verify the token signature
    // 3. Check the token hasn't expired
    // 4. Verify the audience (client ID) matches your app
    // 5. Verify the issuer is Google

    eprintln!("WARNING: Google OAuth token is NOT being verified! This is a security risk!");

    // Validate inputs
    validate_email(&payload.email).map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    validate_string_length(&payload.name, MAX_NAME_LENGTH, "Name")
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    let admin_opt: Option<models::Admin> = sqlx::query_as("SELECT * FROM admins WHERE email = ?")
        .bind(&payload.email)
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
        // More secure domain validation - use environment variable or config in production
        let allowed_domains = vec!["ddscheduler.com", "example.com"];
        let email_domain = payload.email.split('@').nth(1).unwrap_or("");
        let default_admin_email = std::env::var("DEFAULT_ADMIN_EMAIL")
            .unwrap_or_else(|_| "admin@example.com".to_string());

        if !allowed_domains.iter().any(|d| email_domain == *d)
            && payload.email != default_admin_email
        {
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
            .bind(&payload.email)
            .bind("admin")
            .bind(now)
            .execute(&pool)
            .await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create user".to_string()))?;

        models::Admin {
            id,
            username: sanitized_name,
            password_hash: "".to_string(),
            email: Some(payload.email),
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

pub async fn delete_poll(
    State(pool): State<DbPool>,
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
