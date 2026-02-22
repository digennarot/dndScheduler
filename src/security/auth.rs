use crate::core::models::*;
use crate::db::DbPool;
use axum::{
    extract::{Extension, FromRef, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::Utc;
use serde::Serialize;
use uuid::Uuid;

// Error response for JSON
#[derive(serde::Serialize, serde::Deserialize)]
pub struct ErrorResponse {
    pub error: String,
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

// Google Token Verification
pub async fn verify_google_token(token: &str) -> Result<GoogleClaims, String> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://oauth2.googleapis.com/tokeninfo")
        .query(&[("id_token", token)])
        .send()
        .await
        .map_err(|e| format!("Failed to contact Google: {}", e))?;

    if !response.status().is_success() {
        return Err("Invalid Google token".to_string());
    }

    let response_text = response
        .text()
        .await
        .map_err(|e| format!("Failed to read Google response text: {}", e))?;

    tracing::info!("Google Token Response: {}", response_text);

    let claims: GoogleClaims = serde_json::from_str(&response_text).map_err(|e| {
        format!(
            "Failed to parse Google response: {} Body: {}",
            e, response_text
        )
    })?;

    // Verify Audience
    if let Ok(expected_client_id) = std::env::var("GOOGLE_CLIENT_ID") {
        if claims.aud != expected_client_id.trim() {
            tracing::error!(
                "Token audience mismatch! Received: '{}', Expected: '{}'",
                claims.aud,
                expected_client_id
            );
            return Err(format!(
                "Token audience mismatch: got {}, want {}",
                claims.aud, expected_client_id
            ));
        }
    } else {
        tracing::warn!("GOOGLE_CLIENT_ID not set! Security warning.");
    }

    // Verify Email Verified
    if !claims.email_verified {
        return Err("Google email not verified".to_string());
    }

    Ok(claims)
}

// ============================================================================
// REGISTRATION
// ============================================================================

pub async fn register(
    State(pool): State<DbPool>,
    Extension(event_store): Extension<std::sync::Arc<crate::core::store::RedbEventStore>>,
    Extension(users_projection): Extension<
        std::sync::Arc<crate::core::projections::UsersProjection>,
    >,
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

    // Check if user already exists (using Projection)
    if users_projection.get_by_email(&payload.email).is_some() {
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

    // Create user ID & Event
    let user_id = Uuid::new_v4().to_string();
    let now = Utc::now().timestamp();
    let sanitized_name = sanitize_string(&payload.name);
    let default_role = "player".to_string();

    let event =
        crate::core::events::Event::V1UserRegistered(crate::core::events::UserRegisteredV1 {
            id: user_id.clone(),
            email: payload.email.clone(),
            password_hash: password_hash.clone(),
            name: sanitized_name.clone(),
            role: default_role.clone(),
            created_at: now,
            phone: payload.phone.clone(),
        });

    // Persist to Redb
    // Stream ID: user-{id}
    if let Err(e) = event_store
        .append_event(&format!("user-{}", user_id), event.clone())
        .await
    {
        tracing::error!("Failed to persist user registration event: {}", e);
        return Err(json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to create user",
        ));
    }

    // Apply to Projection (Read-Your-Writes)
    users_projection.apply(event);

    // PROJECTION (Sync Dual Write): Update Read Model for FK constraints
    let new_user = User {
        id: user_id.clone(),
        email: payload.email.clone(),
        password_hash: password_hash.clone(),
        name: sanitized_name.clone(),
        role: default_role.clone(),
        created_at: now,
        last_login: Some(now), // Last login = created_at
        phone: payload.phone.clone(),
        consent_marketing: false,
        consent_analytics: false,
        privacy_policy_accepted_at: None,
    };

    if let Err(e) = crate::db::queries::user_repo::UserRepo::create_or_update(&pool, &new_user) {
        tracing::error!("Failed to update users table projection: {}", e);
        return Err(json_error(StatusCode::INTERNAL_SERVER_ERROR, "Failed to create user projection"));
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
    let session_token = Uuid::new_v4().to_string();
    let expires_at = Utc::now()
        .checked_add_signed(chrono::Duration::hours(SESSION_DURATION_HOURS))
        .ok_or(json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to calculate expiration",
        ))?
        .timestamp();

    let session = crate::core::models::UserSession {
        id: Uuid::new_v4().to_string(),
        user_id: user_id.clone(),
        token: session_token.clone(),
        expires_at,
        created_at: now,
    };

    crate::db::queries::admin_repo::SessionRepo::create_user_session(&pool, &session)
        .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "Failed to create session"))?;

    // Send welcome email (fire and forget)
    let email_for_task = payload.email.clone();
    let name_for_task = sanitized_name.clone();

    tokio::spawn(async move {
        if let Err(e) =
            crate::core::services::email::send_welcome_email(&email_for_task, &name_for_task).await
        {
            tracing::error!("Failed to send welcome email to {}: {}", email_for_task, e);
        }
    });

    // Handle WhatsApp welcome message if phone is provided
    if let Some(phone) = &payload.phone {
        if crate::core::services::whatsapp::validate_phone_number(phone) {
            let phone_for_task = phone.clone();
            let name_for_wa = sanitized_name.clone();
            tokio::spawn(async move {
                if let Err(e) = crate::core::services::whatsapp::send_welcome_whatsapp(
                    &phone_for_task,
                    &name_for_wa,
                )
                .await
                {
                    tracing::error!(
                        "Failed to send welcome whatsapp to {}: {}",
                        phone_for_task,
                        e
                    );
                }
            });
        }
    }

    // Return response
    let user_response = UserPublic {
        id: user_id,
        email: payload.email,
        name: sanitized_name,
        role: default_role,
        phone: payload.phone,
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
    Extension(event_store): Extension<std::sync::Arc<crate::core::store::RedbEventStore>>,
    Extension(users_projection): Extension<
        std::sync::Arc<crate::core::projections::UsersProjection>,
    >,
    Json(payload): Json<UserLoginRequest>,
) -> Result<Json<UserAuthResponse>, Response> {
    // Validate inputs
    validate_email(&payload.email).map_err(|e| json_error(StatusCode::BAD_REQUEST, e))?;

    // CHECK ACCOUNT LOCK STATUS
    let now = Utc::now().timestamp();
    let lock_status = crate::db::queries::auth_repo::AuthRepo::get_account_lock(&pool, &payload.email)
        .unwrap_or(None);

    if let Some(locked_until) = lock_status {
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
            let _ = crate::db::queries::auth_repo::AuthRepo::delete_account_lock(&pool, &payload.email);
        }
    }

    // Find user (Using Projection)
    let user_view = users_projection.get_by_email(&payload.email);

    // Verify password if user exists
    let password_valid = if let Some(ref user) = user_view {
        verify(&payload.password, &user.password_hash).unwrap_or(false)
    } else {
        // Fake verification to prevent timing attacks
        let _ = hash("dummy", DEFAULT_COST);
        false
    };

    // Handle Login Attempt
    let ip_address = "unknown"; // In a real app, extract from request headers
    let _ = crate::db::queries::auth_repo::AuthRepo::record_login_attempt(
        &pool,
        &payload.email,
        now,
        password_valid,
        ip_address,
    );

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
        let failures = crate::db::queries::auth_repo::AuthRepo::count_failed_attempts(
            &pool,
            &payload.email,
            window_start,
        ).unwrap_or(0);

        if failures >= 10 {
            let lock_duration = 5 * 60; // 5 minutes
            let locked_until = now + lock_duration;
            let _ = crate::db::queries::auth_repo::AuthRepo::lock_account(
                &pool,
                &payload.email,
                locked_until,
                "Too many failed login attempts",
            );

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

    let user = user_view.unwrap(); // Safe because we checked password_valid which requires user to exist

    // Create Event for Last Login
    let event = crate::core::events::Event::V1UserLoggedIn(crate::core::events::UserLoggedInV1 {
        id: user.id.clone(),
        timestamp: now,
    });

    // Persist Login Event (Fire and forget or wait? Wait is safer for consistency)
    if let Err(e) = event_store
        .append_event(&format!("user-{}", user.id), event.clone())
        .await
    {
        tracing::error!("Failed to persist login event: {}", e);
        // Non-fatal, proceed with login but log error
    } else {
        // Apply to projection
        users_projection.apply(event);
    }

    // Heal Read Model if needed (Self-Healing)
    let existing = crate::db::queries::user_repo::UserRepo::find_by_id(&pool, &user.id).unwrap_or(None);
    if existing.is_none() {
        tracing::warn!(
            "Healing Read Model: User {} exists in Event Store but not in Redb. Restoring...",
            user.email
        );
        let healed_user = User {
            id: user.id.clone(),
            email: user.email.clone(),
            password_hash: user.password_hash.clone(),
            name: user.name.clone(),
            role: user.role.clone(),
            created_at: user.created_at,
            last_login: user.last_login.or(Some(user.created_at)),
            phone: user.phone.clone(),
            consent_marketing: user.consent_marketing,
            consent_analytics: user.consent_analytics,
            privacy_policy_accepted_at: user.privacy_policy_accepted_at,
        };
        let _ = crate::db::queries::user_repo::UserRepo::create_or_update(&pool, &healed_user);
    }

    // Create session
    let session_token = Uuid::new_v4().to_string();
    let expires_at = Utc::now()
        .checked_add_signed(chrono::Duration::hours(SESSION_DURATION_HOURS))
        .ok_or(json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to calculate expiration",
        ))?
        .timestamp();

    let session = crate::core::models::UserSession {
        id: Uuid::new_v4().to_string(),
        user_id: user.id.clone(),
        token: session_token.clone(),
        expires_at,
        created_at: now,
    };

    crate::db::queries::admin_repo::SessionRepo::create_user_session(&pool, &session)
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
        user: UserPublic {
            id: user.id,
            email: user.email,
            name: user.name,
            role: user.role,
            created_at: user.created_at,
            phone: user.phone,
        },
    }))
}

pub async fn login_google(
    State(pool): State<DbPool>,
    Extension(event_store): Extension<std::sync::Arc<crate::core::store::RedbEventStore>>,
    Extension(users_projection): Extension<
        std::sync::Arc<crate::core::projections::UsersProjection>,
    >,
    Json(payload): Json<GoogleLoginPayload>,
) -> Result<Json<UserAuthResponse>, Response> {
    // 1. Verify Token (Stateless)
    let claims = match verify_google_token(&payload.credential).await {
        Ok(claims) => claims,
        Err(e) => {
            tracing::error!("Google Token Verification Failed: {}", e);
            return Err(json_error(StatusCode::UNAUTHORIZED, e));
        }
    };

    let email = claims.email;
    let name = claims.name;

    // 2. Check if user exists (Use Projection)
    let existing_user_view = users_projection.get_by_email(&email);

    let user = if let Some(user_view) = existing_user_view {
        // User exists - Return them (Silent Merge)
        let now = Utc::now().timestamp();

        // Log Login Event
        let event =
            crate::core::events::Event::V1UserLoggedIn(crate::core::events::UserLoggedInV1 {
                id: user_view.id.clone(),
                timestamp: now,
            });

        if let Err(e) = event_store
            .append_event(&format!("user-{}", user_view.id), event.clone())
            .await
        {
            tracing::warn!("Failed to log google login event: {}", e);
        }
        users_projection.apply(event);

        // Healing Read Model if needed (Self-Healing)
        let existing = crate::db::queries::user_repo::UserRepo::find_by_id(&pool, &user_view.id).unwrap_or(None);
        if existing.is_none() {
            tracing::warn!("Healing Read Model (Google Login): User {} exists in Event Store but not in Redb. Restoring...", user_view.email);
            let healed_user = User {
                id: user_view.id.clone(),
                email: user_view.email.clone(),
                password_hash: user_view.password_hash.clone(),
                name: user_view.name.clone(),
                role: user_view.role.clone(),
                created_at: user_view.created_at,
                last_login: Some(now),
                phone: user_view.phone.clone(),
                consent_marketing: user_view.consent_marketing,
                consent_analytics: user_view.consent_analytics,
                privacy_policy_accepted_at: user_view.privacy_policy_accepted_at,
            };
            let _ = crate::db::queries::user_repo::UserRepo::create_or_update(&pool, &healed_user);
        } else {
            let mut u = existing.unwrap();
            u.last_login = Some(now);
            let _ = crate::db::queries::user_repo::UserRepo::create_or_update(&pool, &u);
        }

        // Convert to User struct for response (Simplified)
        User {
            id: user_view.id.clone(),
            email: user_view.email.clone(),
            password_hash: user_view.password_hash.clone(),
            name: user_view.name.clone(),
            role: user_view.role.clone(),
            created_at: user_view.created_at,
            last_login: Some(now),
            phone: user_view.phone.clone(),
            consent_marketing: user_view.consent_marketing,
            consent_analytics: user_view.consent_analytics,
            privacy_policy_accepted_at: user_view.privacy_policy_accepted_at,
        }
    } else {
        // User does not exist in Projection (Event Store)
        // Check if exists in Read Model (Redb)
        let legacy_user = crate::db::queries::user_repo::UserRepo::find_by_email(&pool, &email).unwrap_or(None);

        let (user_id, _is_new, password_hash, role, created_at, _phone) = if let Some(lu) = legacy_user {
            // User exists in Redb but not Event Store projection. Adopt them.
            tracing::info!(
                "JIT Migration: Adopting legacy user {} for Event Store.",
                email
            );
            (lu.id, false, lu.password_hash, lu.role, lu.created_at, lu.phone)
        } else {
            // Truly new user
            (
                Uuid::new_v4().to_string(),
                true,
                "GOOGLE_OAUTH_USER".to_string(),
                "player".to_string(),
                Utc::now().timestamp(),
                None,
            )
        };

        // Use existing timestamp if adapting, or now if new (though created_at covers it)
        let now = Utc::now().timestamp();
        let sanitized_name = sanitize_string(&name);

        // Create Register Event
        let event =
            crate::core::events::Event::V1UserRegistered(crate::core::events::UserRegisteredV1 {
                id: user_id.clone(),
                email: email.clone(),
                password_hash: password_hash.clone(),
                name: sanitized_name.clone(),
                role: role.clone(),
                created_at: created_at,
                phone: None,
            });

        // Append to Event Store
        if let Err(e) = event_store
            .append_event(&format!("user-{}", user_id), event.clone())
            .await
        {
            tracing::error!("Failed to append google user registration event: {}", e);
            return Err(json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to create user",
            ));
        }
        users_projection.apply(event.clone());

        // Dual Write to Redb
        let new_user = User {
            id: user_id.clone(),
            email: email.clone(),
            password_hash: password_hash.clone(),
            name: sanitized_name.clone(),
            role: role.clone(),
            created_at: created_at,
            last_login: Some(now),
            phone: None,
            consent_marketing: false,
            consent_analytics: false,
            privacy_policy_accepted_at: None,
        };
        if let Err(e) = crate::db::queries::user_repo::UserRepo::create_or_update(&pool, &new_user) {
            tracing::error!("Failed to update users table projection: {}", e);
            return Err(json_error(StatusCode::INTERNAL_SERVER_ERROR, "Failed to create user projection"));
        }

        // Audit Log
        crate::audit::log_audit(
            &pool,
            Some(user_id.clone()),
            "register_google",
            Some("auth".to_string()),
            true,
            Some(format!("JIT Provisioning for {}", email)),
            None,
        )
        .await;

        // Log Login Event immediately
        let login_event =
            crate::core::events::Event::V1UserLoggedIn(crate::core::events::UserLoggedInV1 {
                id: user_id.clone(),
                timestamp: now,
            });
        let _ = event_store
            .append_event(&format!("user-{}", user_id), login_event.clone())
            .await;
        users_projection.apply(login_event);

        User {
            id: user_id,
            email,
            password_hash: password_hash,
            name: sanitized_name,
            role: role,
            created_at: created_at,
            last_login: Some(now),
            phone: None,
            consent_marketing: false,
            consent_analytics: false,
            privacy_policy_accepted_at: None,
        }
    };

    // Return the SAME Google Token as the session token (Stateless)
    // The client will send this token back in headers
    Ok(Json(UserAuthResponse {
        token: payload.credential,
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
    let sess = crate::db::queries::admin_repo::SessionRepo::get_user_session(&pool, &token)
        .unwrap_or(None);
        
    let user_id = sess.map(|s| s.user_id);

    let _ = crate::db::queries::admin_repo::SessionRepo::delete_user_session_by_token(&pool, &token);

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
    // Refactored to use shared validation logic
    let user = validate_session(&pool, &token).await?;
    Ok(Json(UserPublic::from(user)))
}

pub async fn get_current_session_user(auth_user: AuthUser) -> Json<UserPublic> {
    Json(UserPublic::from(auth_user.0))
}

// ============================================================================
// DELETE ACCOUNT
// ============================================================================

pub async fn delete_account(
    State(pool): State<DbPool>,
    auth_user: AuthUser,
) -> Result<StatusCode, (StatusCode, String)> {
    let user = auth_user.0;

    crate::db::queries::user_repo::UserRepo::delete_user_all_data(&pool, &user.id).map_err(|_| (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Failed to clean up user data".to_string(),
    ))?;

    // Finally, delete the user
    crate::db::queries::user_repo::UserRepo::delete(&pool, &user.id).map_err(|_| (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Failed to delete account".to_string(),
    ))?;

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
    pub phone: Option<String>,
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
        if let Ok(Some(existing_user)) = crate::db::queries::user_repo::UserRepo::find_by_email(&pool, email) {
            if existing_user.id != user.id {
                return Err(json_error(
                    StatusCode::CONFLICT,
                    "Email already in use by another account",
                ));
            }
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
    let new_phone = payload.phone.or(user.phone.clone());
    let role_changed = new_role != user.role;

    let mut updated_user = user.clone();
    updated_user.name = new_name.clone();
    updated_user.email = new_email.clone();
    updated_user.role = new_role.clone();
    updated_user.phone = new_phone.clone();

    crate::db::queries::user_repo::UserRepo::create_or_update(&pool, &updated_user)
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
        phone: new_phone,
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
    let mut updated_user = user.clone();
    updated_user.password_hash = new_hash;
    crate::db::queries::user_repo::UserRepo::create_or_update(&pool, &updated_user)
        .map_err(|_| {
            json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to update password",
            )
        })?;

    // Invalidate all other sessions (security best practice)
    let _ = crate::db::queries::admin_repo::SessionRepo::delete_user_sessions(&pool, &user.id);

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
    // Strategy: Check if it's a UUID (DB Session) or JWT (Google)

    // Simple heuristic: UUIDs are 36 chars. JWTs are much longer.
    if token.len() > 50 {
        // Assume Google JWT
        let claims = verify_google_token(token)
            .await
            .map_err(|e| (StatusCode::UNAUTHORIZED, e))?;

        // Find user by email
        let user = crate::db::queries::user_repo::UserRepo::find_by_email(pool, &claims.email)
            .map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Database error".to_string(),
                )
            })?
            .ok_or((
                StatusCode::UNAUTHORIZED,
                "User not found (JIT required)".to_string(),
            ))?;

        Ok(user)
    } else {
        // Assume DB Session
        validate_db_session(pool, token).await
    }
}

async fn validate_db_session(pool: &DbPool, token: &str) -> Result<User, (StatusCode, String)> {
    // Get session
    let session = crate::db::queries::admin_repo::SessionRepo::get_user_session(pool, token)
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
        let _ = crate::db::queries::admin_repo::SessionRepo::delete_user_session_by_token(pool, token);

        return Err((StatusCode::UNAUTHORIZED, "Session expired".to_string()));
    }

    // Get user
    let user = crate::db::queries::user_repo::UserRepo::find_by_id(pool, &session.user_id)
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

pub struct MaybeAuthUser(pub Option<User>);

#[axum::async_trait]
impl<S> axum::extract::FromRequestParts<S> for MaybeAuthUser
where
    DbPool: axum::extract::FromRef<S>,
    S: Send + Sync,
{
    type Rejection = (StatusCode, Json<ErrorResponse>);

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts.headers.get(axum::http::header::AUTHORIZATION);

        let token = match auth_header {
            Some(header) => {
                let auth_str = header.to_str().unwrap_or("");
                if auth_str.starts_with("Bearer ") {
                    Some(&auth_str[7..])
                } else {
                    None
                }
            }
            None => None,
        };

        if let Some(token) = token {
            let pool = DbPool::from_ref(state);
            match validate_session(&pool, token).await {
                Ok(user) => Ok(MaybeAuthUser(Some(user))),
                Err(_) => Ok(MaybeAuthUser(None)), // Invalid token -> Treat as guest
            }
        } else {
            Ok(MaybeAuthUser(None))
        }
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
        let pool = DbPool::from_ref(state);

        // Try to get token from Cookie first
        let cookie_token = parts
            .headers
            .get(axum::http::header::COOKIE)
            .and_then(|h| h.to_str().ok())
            .and_then(|s| {
                s.split(';').find_map(|c| {
                    let c = c.trim();
                    if c.starts_with("admin_session=") {
                        Some(c["admin_session=".len()..].to_string())
                    } else {
                        None
                    }
                })
            });

        // Fallback to Bearer token
        let token = if let Some(t) = cookie_token {
            t
        } else {
            let auth_header = parts
                .headers
                .get(axum::http::header::AUTHORIZATION)
                .ok_or((
                    StatusCode::UNAUTHORIZED,
                    Json(ErrorResponse {
                        error: "Missing authentication".to_string(),
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
            auth_str[7..].to_string()
        };

        let admin = validate_admin_session(&pool, &token)
            .await
            .map_err(|(status, msg)| (status, Json(ErrorResponse { error: msg })))?;

        Ok(AdminUser(admin))
    }
}

pub async fn validate_admin_session(
    pool: &DbPool,
    token: &str,
) -> Result<Admin, (StatusCode, String)> {
    // 1. Check user_sessions table
    let session = crate::db::queries::admin_repo::SessionRepo::get_user_session(pool, token)
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

    // 2. Check expiration
    let now = Utc::now().timestamp();
    if session.expires_at < now {
        let _ = crate::db::queries::admin_repo::SessionRepo::delete_user_session_by_token(pool, token);
        return Err((StatusCode::UNAUTHORIZED, "Session expired".to_string()));
    }

    // 3. Get User and Verify Role
    let user = crate::db::queries::user_repo::UserRepo::find_by_id(pool, &session.user_id)
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error".to_string(),
            )
        })?
        .ok_or((StatusCode::UNAUTHORIZED, "User not found".to_string()))?;

    if user.role != "admin" {
        return Err((StatusCode::FORBIDDEN, "User is not an admin".to_string()));
    }

    // Construct Admin struct (mapping from User)
    Ok(Admin {
        id: user.id,
        username: user.name, // Map User.name to Admin.username
        password_hash: user.password_hash,
        email: Some(user.email),
        role: user.role,
        created_at: user.created_at,
    })
}
#[derive(Serialize)]
pub struct GoogleConfig {
    pub client_id: Option<String>,
}

pub async fn get_google_config() -> Json<GoogleConfig> {
    let client_id = std::env::var("GOOGLE_CLIENT_ID").ok();
    Json(GoogleConfig { client_id })
}
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_google_config() {
        // Case 1: Variable not set
        unsafe {
            std::env::remove_var("GOOGLE_CLIENT_ID");
        }
        let response = get_google_config().await;
        assert!(response.0.client_id.is_none());

        // Case 2: Variable set
        let test_id = "test-client-id-123";
        unsafe {
            std::env::set_var("GOOGLE_CLIENT_ID", test_id);
        }
        let response = get_google_config().await;
        assert_eq!(response.0.client_id, Some(test_id.to_string()));

        // Cleanup
        unsafe {
            std::env::remove_var("GOOGLE_CLIENT_ID");
        }
    }
}
