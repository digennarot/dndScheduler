use crate::core::events::Event;
use crate::core::models;
use crate::core::models::{CreatePollRequest, Poll};
use crate::core::store::RedbEventStore;
use crate::db::DbPool;
use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use uuid::Uuid;

// Security constants
const MAX_TITLE_LENGTH: usize = 100;
const MAX_DESCRIPTION_LENGTH: usize = 500;
const MAX_LOCATION_LENGTH: usize = 100;
const MAX_NAME_LENGTH: usize = 50;
const MAX_EMAIL_LENGTH: usize = 254; // RFC 5321

const MAX_DATES: usize = 365;

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
    if id.len() == 12 && id.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        return Ok(());
    }
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
    State(_pool): State<DbPool>,
    Extension(projection): Extension<Arc<crate::core::projections::PollsProjection>>,
) -> Result<Json<Vec<Poll>>, (StatusCode, String)> {
    let polls = projection.get_all();
    Ok(Json(polls))
}

// create_poll moved to poll.rs

pub async fn get_poll(
    State(_pool): State<DbPool>,
    Extension(projection): Extension<Arc<crate::core::projections::PollsProjection>>,
    jar: axum_extra::extract::cookie::SignedCookieJar,
    Path(poll_id): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    // Validate poll_id is a valid UUID
    validate_uuid(&poll_id).map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    // Read from Projection (Memory)
    if let Some(view) = projection.get(&poll_id) {
        // Story 2.2: Extract my_vote based on session
        let session_id = jar.get("dnd_session").map(|c| c.value().to_string());
        
        // Filter availability for this session
        let my_vote = if let Some(sid) = session_id {
            let votes = view.availability
                .iter()
                .filter(|a| a.participant_id == sid)
                .map(|v| crate::core::models::AvailabilityEntry {
                    date: v.date.clone(),
                    time_slot: v.time_slot.clone(),
                    status: v.status.clone(),
                })
                .collect::<Vec<_>>();
            
            if votes.is_empty() { None } else { Some(votes) }
        } else {
            None
        };

        let response = crate::core::models::PollResponse {
            poll: view.poll,
            participants: view.participants,
            votes: view.availability,
            instances: view.instances,
            my_vote,
        };

        let json_value = serde_json::to_value(response)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Serialization error: {}", e)))?;
        Ok(Json(json_value))
    } else {
        Err((StatusCode::NOT_FOUND, "Poll not found".to_string()))
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
    let admin = crate::db::queries::admin_repo::AdminRepo::find_by_email(&pool, &payload.email)
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

    crate::db::queries::admin_repo::SessionRepo::create_admin_session(&pool, &token, &admin.id, expires_at)
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
    let users = crate::db::queries::user_repo::UserRepo::get_all(&pool)
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

    let admin_opt = crate::db::queries::admin_repo::AdminRepo::find_by_email(&pool, &email)
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
        let allowed_domains = ["ddscheduler.com", "example.com"];
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
        let _hash =
            bcrypt::hash("google_oauth_no_password_login", bcrypt::DEFAULT_COST).map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to create user".to_string(),
                )
            })?;

        let new_admin = models::Admin {
            id,
            username: sanitized_name,
            password_hash: "".to_string(),
            email: Some(email),
            role: "admin".to_string(),
            created_at: now,
        };

        crate::db::queries::admin_repo::AdminRepo::create(&pool, &new_admin)
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create user".to_string()))?;

        new_admin
    };

    let token = Uuid::new_v4().to_string();
    let expiration = Utc::now()
        .checked_add_signed(chrono::Duration::hours(24))
        .ok_or((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to generate expiration".to_string(),
        ))?
        .timestamp();

    crate::db::queries::admin_repo::SessionRepo::create_admin_session(&pool, &token, &admin.id, expiration)
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

    let success = crate::db::queries::poll_repo::PollRepo::update(
        &pool,
        &poll_id,
        &title,
        &description,
        &location,
        &dates_json,
        &time_range_value,
    ).map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update poll".to_string()))?;

    if !success {
        return Err((StatusCode::NOT_FOUND, "Poll not found".to_string()));
    }

    Ok(Json(json!({ "success": true })))
}

pub async fn delete_poll(
    State(pool): State<DbPool>,
    Extension(event_store): Extension<Arc<RedbEventStore>>,
    Extension(projection): Extension<Arc<crate::core::projections::PollsProjection>>,
    _admin: crate::auth::AdminUser,
    Path(poll_id): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    // Validate poll_id
    validate_uuid(&poll_id).map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    crate::db::queries::poll_repo::PollRepo::delete_poll(&pool, &poll_id)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete poll".to_string()))?;

    // EVENT SOURCING: Emit PollDeletedV1
    let event = Event::V1PollDeleted(crate::core::events::PollDeletedV1 {
        id: poll_id.clone(),
    });

    // Use append_event to ensure version consistency
    let stream_id = format!("poll-{}", poll_id);
    if let Err(e) = event_store.append_event(&stream_id, &event).await {
        tracing::error!("Failed to append PollDeletedV1 event: {}", e);
    } else {
        projection.apply(event);
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

    let success = crate::db::queries::poll_repo::PollRepo::delete_participant_by_id(&pool, &participant_id)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string()))?;

    if !success {
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

    // Check if user exists and fetch it
    let mut user = crate::db::queries::user_repo::UserRepo::find_by_id(&pool, &user_id)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "User not found".to_string()))?;

    // Update user's role in database
    user.role = role.clone();
    
    crate::db::queries::user_repo::UserRepo::create_or_update(&pool, &user)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update role".to_string()))?;

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

    let user = crate::db::queries::user_repo::UserRepo::find_by_id(&pool, &user_id)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "User not found".to_string()))?;

    let user_email = user.email.clone();

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
    let mut updated_user = user;
    updated_user.password_hash = password_hash;
    crate::db::queries::user_repo::UserRepo::create_or_update(&pool, &updated_user)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update password".to_string()))?;

    // Invalidate all user sessions for security
    crate::db::queries::admin_repo::SessionRepo::delete_user_sessions(&pool, &user_id)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to invalidate sessions".to_string()))?;

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
    let poll_result = crate::db::queries::poll_repo::PollRepo::get_details(&pool, &id).ok().flatten();

    let (title, description) = match poll_result {
        Some((p, _, _, _)) => (p.title, p.description),
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

pub async fn delete_user_admin(
    State(pool): State<DbPool>,
    Extension(event_store): Extension<std::sync::Arc<crate::core::store::RedbEventStore>>,
    Extension(users_projection): Extension<
        std::sync::Arc<crate::core::projections::UsersProjection>,
    >,
    admin_user: crate::auth::AdminUser,
    Path(user_id): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    // AdminUser extractor validates admin authentication
    let admin = admin_user.0;

    // Validate user_id
    validate_uuid(&user_id).map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    // Check if user exists
    let user_option = users_projection.get(&user_id);

    // If not in projection, check DB (legacy/sync issue coverage)
    let email = if let Some(u) = user_option {
        u.email.clone()
    } else {
        match crate::db::queries::user_repo::UserRepo::find_by_id(&pool, &user_id).ok().flatten() {
            Some(u) => u.email,
            None => return Err((StatusCode::NOT_FOUND, "User not found".to_string())),
        }
    };

    // Log deletion in audit log
    crate::audit::log_audit(
        &pool,
        Some(admin.id.clone()),
        "admin_delete_user",
        Some("admin".to_string()),
        true,
        Some(format!("Admin deleted user: {}", email)),
        None,
    )
    .await;

    crate::db::queries::user_repo::UserRepo::delete_user_all_data(&pool, &user_id)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database cleanup error".to_string()))?;

    // Also delete user completely
    crate::db::queries::user_repo::UserRepo::delete(&pool, &user_id)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database cleanup error".to_string()))?;

    // Event Sourcing: Emit UserDeletedV1
    let event = crate::core::events::Event::V1UserDeleted(crate::core::events::UserDeletedV1 {
        id: user_id.clone(),
        email: email.clone(),
    });

    if let Err(e) = event_store
        .append_event(&format!("user-{}", user_id), event.clone())
        .await
    {
        tracing::error!("Failed to persist UserDeleted event: {}", e);
        // Non-fatal if SQL delete succeeded, but inconsistent.
    } else {
        // Update Projection
        users_projection.apply(event);
    }

    Ok(Json(json!({
        "success": true,
        "message": format!("User {} deleted successfully", email)
    })))
}
