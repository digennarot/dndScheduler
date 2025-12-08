use crate::{db::DbPool, models::*};
use axum::{
    extract::{FromRef, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::Utc;
use serde::Serialize;
use uuid::Uuid;

// Error response for JSON
#[derive(Serialize)]
pub struct ErrorResponse {
    error: String,
}

// Helper to create JSON error responses
fn json_error(status: StatusCode, message: impl Into<String>) -> Response {
    (
        status,
        Json(ErrorResponse {
            error: message.into(),
        }),
    )
        .into_response()
}

// Security constants
const MAX_EMAIL_LENGTH: usize = 254;
const MAX_NAME_LENGTH: usize = 100;
const MAX_PASSWORD_LENGTH: usize = 128;
const SESSION_DURATION_HOURS: i64 = 24 * 7; // 7 days

// Validation helpers
fn validate_email(email: &str) -> Result<(), String> {
    if email.is_empty() || email.len() > MAX_EMAIL_LENGTH {
        return Err("Invalid email length".to_string());
    }

    if !email.contains('@') || !email.contains('.') {
        return Err("Invalid email format".to_string());
    }

    if email.contains(['<', '>', '"', '\'', '\\', '\0']) {
        return Err("Invalid characters in email".to_string());
    }

    Ok(())
}

fn validate_password(password: &str) -> Result<(), String> {
    // Minimum length check (OWASP recommends 12+)
    if password.len() < 12 {
        return Err("Password must be at least 12 characters long".to_string());
    }

    if password.len() > MAX_PASSWORD_LENGTH {
        return Err(format!(
            "Password must be less than {} characters",
            MAX_PASSWORD_LENGTH
        ));
    }

    // Complexity requirements
    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_numeric());
    let has_special = password.chars().any(|c| !c.is_alphanumeric());

    if !has_uppercase {
        return Err("Password must contain at least one uppercase letter".to_string());
    }
    if !has_lowercase {
        return Err("Password must contain at least one lowercase letter".to_string());
    }
    if !has_digit {
        return Err("Password must contain at least one number".to_string());
    }
    if !has_special {
        return Err("Password must contain at least one special character".to_string());
    }

    // Entropy check using zxcvbn
    // Score 0-2: Weak, 3: Fair, 4: Strong -> We require 3+
    let entropy = zxcvbn::zxcvbn(password, &[]);
    if (entropy.score() as u8) < 3 {
        return Err(
            "Password is too weak (common pattern or simple). Please choose a stronger password."
                .to_string(),
        );
    }

    Ok(())
}

fn validate_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Name cannot be empty".to_string());
    }

    if name.len() > MAX_NAME_LENGTH {
        return Err(format!(
            "Name must be less than {} characters",
            MAX_NAME_LENGTH
        ));
    }

    Ok(())
}

fn sanitize_string(s: &str) -> String {
    s.chars()
        .filter(|c| !c.is_control() || *c == '\n' || *c == '\r' || *c == '\t')
        .collect()
}

// ============================================================================
// REGISTRATION
// ============================================================================

pub async fn register(
    State(pool): State<DbPool>,
    Json(payload): Json<UserRegisterRequest>,
) -> Result<Response, Response> {
    // Validate inputs
    if let Err(e) = validate_email(&payload.email) {
        return Err(json_error(StatusCode::BAD_REQUEST, e));
    }
    if let Err(e) = validate_password(&payload.password) {
        return Err(json_error(StatusCode::BAD_REQUEST, e));
    }
    if let Err(e) = validate_name(&payload.name) {
        return Err(json_error(StatusCode::BAD_REQUEST, e));
    }

    // Check if user already exists
    let existing_user: Option<(i64,)> = match sqlx::query_as("SELECT 1 FROM users WHERE email = ?")
        .bind(&payload.email)
        .fetch_optional(&pool)
        .await
    {
        Ok(user) => user,
        Err(_) => {
            return Err(json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error",
            ))
        }
    };

    if existing_user.is_some() {
        crate::audit::log_audit(
            &pool,
            None,
            "register_failed",
            Some("user".to_string()),
            false,
            Some(format!("Email already exists: {}", payload.email)),
            None,
        )
        .await;
        return Err(json_error(StatusCode::CONFLICT, "Email already registered"));
    }

    // Hash password
    let password_hash = match hash(&payload.password, DEFAULT_COST) {
        Ok(hash) => hash,
        Err(_) => {
            return Err(json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to hash password",
            ))
        }
    };

    // Create user
    let user_id = Uuid::new_v4().to_string();
    let now = Utc::now().timestamp();
    let sanitized_name = sanitize_string(&payload.name);
    let default_role = "player";

    if let Err(_) = sqlx::query(
        "INSERT INTO users (id, email, password_hash, name, role, created_at, last_login) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&user_id)
    .bind(&payload.email)
    .bind(&password_hash)
    .bind(&sanitized_name)
    .bind(default_role)
    .bind(now)
    .bind(now)
    .execute(&pool)
    .await
    {
        return Err(json_error(StatusCode::INTERNAL_SERVER_ERROR, "Failed to create user"));
    }

    // Audit log success
    crate::audit::log_audit(
        &pool,
        Some(user_id.clone()),
        "register_success",
        Some("user".to_string()),
        true,
        Some("User registered successfully".to_string()),
        None,
    )
    .await;

    // Create session
    let session_id = Uuid::new_v4().to_string();
    let session_token = Uuid::new_v4().to_string();
    let expires_at =
        match Utc::now().checked_add_signed(chrono::Duration::hours(SESSION_DURATION_HOURS)) {
            Some(time) => time.timestamp(),
            None => {
                return Err(json_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to calculate expiration",
                ));
            }
        };

    sqlx::query(
        "INSERT INTO user_sessions (id, user_id, token, expires_at, created_at) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&session_id)
    .bind(&user_id)
    .bind(&session_token)
    .bind(expires_at)
    .bind(now)
    .execute(&pool)
    .await
    .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "Failed to create session"))?;

    // Send welcome email (fire and forget)
    // CLONE BEFORE MOVE: Clone data needed for the async task before constructing the response which moves them.
    let email_for_task = payload.email.clone();
    let name_for_task = sanitized_name.clone();

    tokio::spawn(async move {
        if let Err(e) =
            crate::email_service::send_welcome_email(&email_for_task, &name_for_task).await
        {
            tracing::error!("Failed to send welcome email to {}: {}", email_for_task, e);
        }
    });

    // Return response
    // Re-construct UserPublic from payload since proper fields were moved
    let user_response = UserPublic {
        id: user_id,
        email: payload.email,
        name: sanitized_name,
        role: default_role.to_string(),
        created_at: now,
    };

    Ok((
        StatusCode::CREATED,
        Json(UserAuthResponse {
            token: session_token,
            user: user_response,
        }),
    )
        .into_response())
}

// ============================================================================
// LOGIN
// ============================================================================

pub async fn login(
    State(pool): State<DbPool>,
    Json(payload): Json<UserLoginRequest>,
) -> Result<Json<UserAuthResponse>, Response> {
    // Validate inputs
    validate_email(&payload.email).map_err(|e| json_error(StatusCode::BAD_REQUEST, e))?;

    // CHECK ACCOUNT LOCK STATUS
    let now = Utc::now().timestamp();
    let lock_status: Option<(i64,)> =
        sqlx::query_as("SELECT locked_until FROM account_locks WHERE email = ?")
            .bind(&payload.email)
            .fetch_optional(&pool)
            .await
            .unwrap_or(None);

    if let Some((locked_until,)) = lock_status {
        if locked_until > now {
            let wait_seconds = locked_until - now;
            crate::audit::log_audit(
                &pool,
                None,
                "login_locked",
                Some("auth".to_string()),
                false,
                Some(format!("Attempt on locked account: {}", payload.email)),
                None,
            )
            .await;
            return Err(json_error(
                StatusCode::TOO_MANY_REQUESTS,
                format!("Account locked. Try again in {} seconds.", wait_seconds),
            ));
        } else {
            // Lock expired, remove it
            sqlx::query("DELETE FROM account_locks WHERE email = ?")
                .bind(&payload.email)
                .execute(&pool)
                .await
                .ok();
        }
    }

    // Find user
    let user_result: Option<User> = sqlx::query_as("SELECT * FROM users WHERE email = ?")
        .bind(&payload.email)
        .fetch_optional(&pool)
        .await
        .unwrap_or(None);

    // Verify password if user exists
    let password_valid = if let Some(ref user) = user_result {
        verify(&payload.password, &user.password_hash).unwrap_or(false)
    } else {
        // Fake verification to prevent timing attacks
        let _ = hash("dummy", DEFAULT_COST);
        false
    };

    // Handle Login Attempt
    let ip_address = "unknown"; // In a real app, extract from request headers
    sqlx::query(
        "INSERT INTO login_attempts (email, attempt_time, success, ip_address) VALUES (?, ?, ?, ?)",
    )
    .bind(&payload.email)
    .bind(now)
    .bind(password_valid)
    .bind(ip_address)
    .execute(&pool)
    .await
    .ok();

    if !password_valid {
        crate::audit::log_audit(
            &pool,
            None,
            "login_failed",
            Some("auth".to_string()),
            false,
            Some(format!("Failed login for: {}", payload.email)),
            None,
        )
        .await;

        // Check for too many failures in last 5 minutes
        let window_start = now - (5 * 60);
        let failures: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM login_attempts WHERE email = ? AND success = FALSE AND attempt_time > ?")
            .bind(&payload.email)
            .bind(window_start)
            .fetch_one(&pool)
            .await
            .unwrap_or(0);

        if failures >= 10 {
            let lock_duration = 5 * 60; // 5 minutes
            let locked_until = now + lock_duration;
            sqlx::query("INSERT OR REPLACE INTO account_locks (email, locked_until, reason) VALUES (?, ?, ?)")
                .bind(&payload.email)
                .bind(locked_until)
                .bind("Too many failed login attempts")
                .execute(&pool)
                .await
                .ok();

            crate::audit::log_audit(
                &pool,
                None,
                "account_locked",
                Some("auth".to_string()),
                true,
                Some(format!("Account locked: {}", payload.email)),
                None,
            )
            .await;

            return Err(json_error(
                StatusCode::TOO_MANY_REQUESTS,
                "Account locked due to too many failed attempts.".to_string(),
            ));
        }

        return Err(json_error(StatusCode::UNAUTHORIZED, "Invalid credentials"));
    }

    let user = user_result.unwrap(); // Safe because we checked password_valid which requires user to exist

    // Update last login
    let now = Utc::now().timestamp();
    sqlx::query("UPDATE users SET last_login = ? WHERE id = ?")
        .bind(now)
        .bind(&user.id)
        .execute(&pool)
        .await
        .ok(); // Ignore error, not critical

    // Create session
    let session_id = Uuid::new_v4().to_string();
    let session_token = Uuid::new_v4().to_string();
    let expires_at = Utc::now()
        .checked_add_signed(chrono::Duration::hours(SESSION_DURATION_HOURS))
        .ok_or(json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to calculate expiration",
        ))?
        .timestamp();

    sqlx::query(
        "INSERT INTO user_sessions (id, user_id, token, expires_at, created_at) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&session_id)
    .bind(&user.id)
    .bind(&session_token)
    .bind(expires_at)
    .bind(now)
    .execute(&pool)
    .await
    .map_err(|_| {
        json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to create session",
        )
    })?;

    // Audit log success
    crate::audit::log_audit(
        &pool,
        Some(user.id.clone()),
        "login_success",
        Some("auth".to_string()),
        true,
        Some("User logged in".to_string()),
        None,
    )
    .await;

    // Return response
    Ok(Json(UserAuthResponse {
        token: session_token,
        user: user.into(),
    }))
}

// ============================================================================
// LOGOUT
// ============================================================================

pub async fn logout(
    State(pool): State<DbPool>,
    Path(token): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    // Get user_id before deleting session for logging
    let user_id: Option<String> =
        sqlx::query_scalar("SELECT user_id FROM user_sessions WHERE token = ?")
            .bind(&token)
            .fetch_optional(&pool)
            .await
            .unwrap_or(None);

    sqlx::query("DELETE FROM user_sessions WHERE token = ?")
        .bind(&token)
        .execute(&pool)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to logout".to_string(),
            )
        })?;

    if let Some(uid) = user_id {
        crate::audit::log_audit(
            &pool,
            Some(uid),
            "logout",
            Some("auth".to_string()),
            true,
            Some("User logged out".to_string()),
            None,
        )
        .await;
    }

    Ok(StatusCode::OK)
}

// ============================================================================
// GET CURRENT USER
// ============================================================================

pub async fn get_current_user(
    State(pool): State<DbPool>,
    Path(token): Path<String>,
) -> Result<Json<UserPublic>, (StatusCode, String)> {
    // Validate session
    let session: UserSession = sqlx::query_as("SELECT * FROM user_sessions WHERE token = ?")
        .bind(&token)
        .fetch_optional(&pool)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error".to_string(),
            )
        })?
        .ok_or((StatusCode::UNAUTHORIZED, "Invalid session".to_string()))?;

    // Check if session expired
    let now = Utc::now().timestamp();
    if session.expires_at < now {
        // Delete expired session
        sqlx::query("DELETE FROM user_sessions WHERE id = ?")
            .bind(&session.id)
            .execute(&pool)
            .await
            .ok();

        return Err((StatusCode::UNAUTHORIZED, "Session expired".to_string()));
    }

    // Get user
    let user: User = sqlx::query_as("SELECT * FROM users WHERE id = ?")
        .bind(&session.user_id)
        .fetch_optional(&pool)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error".to_string(),
            )
        })?
        .ok_or((StatusCode::NOT_FOUND, "User not found".to_string()))?;

    Ok(Json(user.into()))
}

// ============================================================================
// DELETE ACCOUNT
// ============================================================================

pub async fn delete_account(
    State(pool): State<DbPool>,
    auth_user: AuthUser,
) -> Result<StatusCode, (StatusCode, String)> {
    let user = auth_user.0;

    // Delete user's sessions first (foreign key)
    sqlx::query("DELETE FROM user_sessions WHERE user_id = ?")
        .bind(&user.id)
        .execute(&pool)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to delete sessions".to_string(),
            )
        })?;

    // Delete user's availability entries
    sqlx::query("DELETE FROM availability WHERE participant_id IN (SELECT id FROM participants WHERE user_id = ?)")
        .bind(&user.id)
        .execute(&pool)
        .await
        .ok(); // Ignore if no entries

    // Delete user's participant entries
    sqlx::query("DELETE FROM participants WHERE user_id = ?")
        .bind(&user.id)
        .execute(&pool)
        .await
        .ok();

    // Delete user's activities
    sqlx::query("DELETE FROM activities WHERE user_id = ?")
        .bind(&user.id)
        .execute(&pool)
        .await
        .ok();

    // Finally, delete the user
    let result = sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(&user.id)
        .execute(&pool)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to delete account".to_string(),
            )
        })?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "User not found".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// UPDATE PROFILE
// ============================================================================

#[derive(serde::Deserialize)]
pub struct UpdateProfileRequest {
    pub name: Option<String>,
    pub email: Option<String>,
    pub role: Option<String>,
}

pub async fn update_profile(
    State(pool): State<DbPool>,
    auth_user: AuthUser,
    Json(payload): Json<UpdateProfileRequest>,
) -> Result<Json<UserPublic>, Response> {
    let user = auth_user.0;

    // Validate name if provided
    if let Some(ref name) = payload.name {
        validate_name(name).map_err(|e| json_error(StatusCode::BAD_REQUEST, e))?;
    }

    // Validate email if provided
    if let Some(ref email) = payload.email {
        validate_email(email).map_err(|e| json_error(StatusCode::BAD_REQUEST, e))?;

        // Check if email already exists (for a different user)
        let existing: Option<String> =
            sqlx::query_scalar("SELECT id FROM users WHERE email = ? AND id != ?")
                .bind(email)
                .bind(&user.id)
                .fetch_optional(&pool)
                .await
                .unwrap_or(None);

        if existing.is_some() {
            return Err(json_error(
                StatusCode::CONFLICT,
                "Email already in use by another account",
            ));
        }
    }

    // Validate role if provided (must be 'player' or 'dm')
    if let Some(ref role) = payload.role {
        if role != "player" && role != "dm" {
            return Err(json_error(
                StatusCode::BAD_REQUEST,
                "Role must be 'player' or 'dm'",
            ));
        }
    }

    // Build update values
    let new_name = payload.name.unwrap_or_else(|| user.name.clone());
    let new_email = payload.email.unwrap_or_else(|| user.email.clone());
    let new_role = payload.role.unwrap_or_else(|| user.role.clone());
    let role_changed = new_role != user.role;

    sqlx::query("UPDATE users SET name = ?, email = ?, role = ? WHERE id = ?")
        .bind(&new_name)
        .bind(&new_email)
        .bind(&new_role)
        .bind(&user.id)
        .execute(&pool)
        .await
        .map_err(|_| {
            json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to update profile",
            )
        })?;

    // Log audit - include role change if applicable
    let audit_details = if role_changed {
        format!(
            "Profile updated: name='{}', email='{}', role changed from '{}' to '{}'",
            new_name, new_email, user.role, new_role
        )
    } else {
        format!(
            "Profile updated: name='{}', email='{}'",
            new_name, new_email
        )
    };

    crate::audit::log_audit(
        &pool,
        Some(user.id.clone()),
        "profile_updated",
        Some("auth".to_string()),
        true,
        Some(audit_details),
        None,
    )
    .await;

    // Return updated user
    Ok(Json(UserPublic {
        id: user.id,
        email: new_email,
        name: new_name,
        role: new_role,
        created_at: user.created_at,
    }))
}

// ============================================================================
// CHANGE PASSWORD
// ============================================================================

#[derive(serde::Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

pub async fn change_password(
    State(pool): State<DbPool>,
    auth_user: AuthUser,
    Json(payload): Json<ChangePasswordRequest>,
) -> Result<StatusCode, Response> {
    let user = auth_user.0;

    // Verify current password
    if !verify(&payload.current_password, &user.password_hash).unwrap_or(false) {
        crate::audit::log_audit(
            &pool,
            Some(user.id.clone()),
            "password_change_failed",
            Some("auth".to_string()),
            false,
            Some("Invalid current password provided".to_string()),
            None,
        )
        .await;
        return Err(json_error(
            StatusCode::UNAUTHORIZED,
            "Current password is incorrect",
        ));
    }

    // Validate new password
    validate_password(&payload.new_password).map_err(|e| json_error(StatusCode::BAD_REQUEST, e))?;

    // Check that new password is different from current
    if verify(&payload.new_password, &user.password_hash).unwrap_or(false) {
        return Err(json_error(
            StatusCode::BAD_REQUEST,
            "New password must be different from current password",
        ));
    }

    // Hash new password
    let new_hash = hash(&payload.new_password, DEFAULT_COST)
        .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "Failed to hash password"))?;

    // Update password
    sqlx::query("UPDATE users SET password_hash = ? WHERE id = ?")
        .bind(&new_hash)
        .bind(&user.id)
        .execute(&pool)
        .await
        .map_err(|_| {
            json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to update password",
            )
        })?;

    // Invalidate all other sessions (security best practice)
    sqlx::query("DELETE FROM user_sessions WHERE user_id = ?")
        .bind(&user.id)
        .execute(&pool)
        .await
        .ok();

    // Log audit
    crate::audit::log_audit(
        &pool,
        Some(user.id),
        "password_changed",
        Some("auth".to_string()),
        true,
        Some("Password successfully changed".to_string()),
        None,
    )
    .await;

    Ok(StatusCode::OK)
}

// ============================================================================
// VALIDATE SESSION (Helper for middleware)
// ============================================================================

#[allow(dead_code)]
pub async fn validate_session(pool: &DbPool, token: &str) -> Result<User, (StatusCode, String)> {
    // Get session
    let session: UserSession = sqlx::query_as("SELECT * FROM user_sessions WHERE token = ?")
        .bind(token)
        .fetch_optional(pool)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error".to_string(),
            )
        })?
        .ok_or((StatusCode::UNAUTHORIZED, "Invalid session".to_string()))?;

    // Check expiration
    let now = Utc::now().timestamp();
    if session.expires_at < now {
        sqlx::query("DELETE FROM user_sessions WHERE id = ?")
            .bind(&session.id)
            .execute(pool)
            .await
            .ok();

        return Err((StatusCode::UNAUTHORIZED, "Session expired".to_string()));
    }

    // Get user
    let user: User = sqlx::query_as("SELECT * FROM users WHERE id = ?")
        .bind(&session.user_id)
        .fetch_optional(pool)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error".to_string(),
            )
        })?
        .ok_or((StatusCode::NOT_FOUND, "User not found".to_string()))?;

    Ok(user)
}

// ============================================================================
// AUTH EXTRACTOR
// ============================================================================

pub struct AuthUser(pub User);

#[axum::async_trait]
impl<S> axum::extract::FromRequestParts<S> for AuthUser
where
    DbPool: axum::extract::FromRef<S>,
    S: Send + Sync,
{
    type Rejection = (StatusCode, Json<ErrorResponse>);

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        // Extract Bearer token
        let auth_header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .ok_or((
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "Missing Authorization header".to_string(),
                }),
            ))?;

        let auth_str = auth_header.to_str().map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "Invalid Authorization header".to_string(),
                }),
            )
        })?;

        if !auth_str.starts_with("Bearer ") {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "Invalid token format".to_string(),
                }),
            ));
        }

        let token = &auth_str[7..];
        let pool = DbPool::from_ref(state);

        let user = validate_session(&pool, token)
            .await
            .map_err(|(status, msg)| (status, Json(ErrorResponse { error: msg })))?;

        Ok(AuthUser(user))
    }
}

pub struct AdminUser(pub Admin);

#[axum::async_trait]
impl<S> axum::extract::FromRequestParts<S> for AdminUser
where
    DbPool: axum::extract::FromRef<S>,
    S: Send + Sync,
{
    type Rejection = (StatusCode, Json<ErrorResponse>);

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .ok_or((
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "Missing Authorization header".to_string(),
                }),
            ))?;

        let auth_str = auth_header.to_str().map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "Invalid Authorization header".to_string(),
                }),
            )
        })?;

        if !auth_str.starts_with("Bearer ") {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "Invalid token format".to_string(),
                }),
            ));
        }

        let token = &auth_str[7..];
        let pool = DbPool::from_ref(state);

        let admin = validate_admin_session(&pool, token)
            .await
            .map_err(|(status, msg)| (status, Json(ErrorResponse { error: msg })))?;

        Ok(AdminUser(admin))
    }
}

pub async fn validate_admin_session(
    pool: &DbPool,
    token: &str,
) -> Result<Admin, (StatusCode, String)> {
    // Note: Admin sessions are in 'sessions' table, user sessions in 'user_sessions'
    #[derive(sqlx::FromRow)]
    struct AdminSession {
        user_id: String,
        expires_at: i64,
    }

    let session: AdminSession = sqlx::query_as("SELECT * FROM sessions WHERE token = ?")
        .bind(token)
        .fetch_optional(pool)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error".to_string(),
            )
        })?
        .ok_or((
            StatusCode::UNAUTHORIZED,
            "Invalid admin session".to_string(),
        ))?;

    let now = Utc::now().timestamp();
    if session.expires_at < now {
        sqlx::query("DELETE FROM sessions WHERE token = ?")
            .bind(token)
            .execute(pool)
            .await
            .ok();
        return Err((StatusCode::UNAUTHORIZED, "Session expired".to_string()));
    }

    let admin: Admin = sqlx::query_as("SELECT * FROM admins WHERE id = ?")
        .bind(&session.user_id)
        .fetch_optional(pool)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error".to_string(),
            )
        })?
        .ok_or((StatusCode::UNAUTHORIZED, "Admin not found".to_string()))?;

    Ok(admin)
}
