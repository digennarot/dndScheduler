//! Authelia SSO Integration Module
//!
//! This module provides authentication via Authelia ForwardAuth headers.
//! When Authelia is configured in front of the application (via Caddy/nginx),
//! it passes authenticated user information in HTTP headers.
//!
//! Headers used:
//! - `Remote-User`: User's email (primary identifier)
//! - `Remote-Name`: User's display name
//! - `Remote-Groups`: Comma-separated list of user groups
//! - `Remote-Email`: Alternative email header (fallback)

use crate::core::models::{User, UserPublic};
use crate::db::DbPool;
use axum::{
    extract::FromRef,
    http::{header::HeaderMap, StatusCode},
    Json,
};
use chrono::Utc;
use serde::Serialize;
use uuid::Uuid;

// ============================================================================
// CONFIGURATION
// ============================================================================

/// Check if Authelia integration is enabled via environment variable
pub fn is_authelia_enabled() -> bool {
    std::env::var("AUTHELIA_ENABLED")
        .map(|v| v.to_lowercase() == "true" || v == "1")
        .unwrap_or(false)
}

/// Get the Authelia login URL for redirects
pub fn get_authelia_login_url() -> Option<String> {
    std::env::var("AUTHELIA_LOGIN_URL").ok()
}

/// Get the Authelia logout URL
pub fn get_authelia_logout_url() -> Option<String> {
    std::env::var("AUTHELIA_LOGOUT_URL").ok()
}

// ============================================================================
// AUTHELIA USER INFO FROM HEADERS
// ============================================================================

/// User information extracted from Authelia headers
#[derive(Debug, Clone, Serialize)]
pub struct AutheliaUserInfo {
    /// User's email (from Remote-User or Remote-Email header)
    pub email: String,
    /// User's display name (from Remote-Name header)
    pub name: Option<String>,
    /// User's groups (from Remote-Groups header, comma-separated)
    pub groups: Vec<String>,
}

impl AutheliaUserInfo {
    /// Extract user info from Authelia headers
    pub fn from_headers(headers: &HeaderMap) -> Option<Self> {
        // Try Remote-User first (primary), then Remote-Email as fallback
        let email = headers
            .get("Remote-User")
            .or_else(|| headers.get("Remote-Email"))
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())?;

        // Validate email format
        if !email.contains('@') {
            return None;
        }

        let name = headers
            .get("Remote-Name")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let groups = headers
            .get("Remote-Groups")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.split(',').map(|g| g.trim().to_string()).collect())
            .unwrap_or_default();

        Some(Self {
            email,
            name,
            groups,
        })
    }

    /// Check if user is in a specific group
    pub fn is_in_group(&self, group: &str) -> bool {
        self.groups.iter().any(|g| g.eq_ignore_ascii_case(group))
    }

    /// Check if user is an admin (based on groups)
    pub fn is_admin(&self) -> bool {
        self.is_in_group("admins")
            || self.is_in_group("admin")
            || self.is_in_group("administrators")
    }

    /// Get display name (fallback to email prefix if no name)
    pub fn display_name(&self) -> String {
        match self.name.clone() {
            Some(n) => n,
            None => self
                .email
                .split('@')
                .next()
                .unwrap_or(&self.email)
                .to_string(),
        }
    }
}

// ============================================================================
// DATABASE SYNC - Create/Update User from Authelia Session
// ============================================================================

/// Create or update a user in the database from Authelia session info
pub async fn sync_authelia_user(pool: &DbPool, info: &AutheliaUserInfo) -> Result<User, String> {
    let now = Utc::now().timestamp();

    // Check if user exists
    let existing_user: Option<User> = sqlx::query_as("SELECT * FROM users WHERE email = ?")
        .bind(&info.email)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

    if let Some(mut user) = existing_user {
        // Update last login and name if changed
        let new_name = info.display_name();
        let role = if info.is_admin() { "admin" } else { &user.role };

        sqlx::query("UPDATE users SET last_login = ?, name = ?, role = ? WHERE id = ?")
            .bind(now)
            .bind(&new_name)
            .bind(role)
            .bind(&user.id)
            .execute(pool)
            .await
            .map_err(|e| format!("Failed to update user: {}", e))?;

        user.last_login = Some(now);
        user.name = new_name;
        if info.is_admin() {
            user.role = "admin".to_string();
        }

        Ok(user)
    } else {
        // Create new user
        let user_id = Uuid::new_v4().to_string();
        let name = info.display_name();
        let role = if info.is_admin() { "admin" } else { "player" };

        // Use a placeholder password hash since auth is handled by Authelia
        // This user won't be able to login with password (only via SSO)
        let placeholder_hash = "$authelia_sso$";

        sqlx::query(
            "INSERT INTO users (id, email, password_hash, name, role, created_at, last_login) VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&user_id)
        .bind(&info.email)
        .bind(placeholder_hash)
        .bind(&name)
        .bind(role)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to create user: {}", e))?;

        // Log the auto-creation
        crate::audit::log_audit(
            pool,
            Some(user_id.clone()),
            "authelia_user_created",
            Some("auth".to_string()),
            true,
            Some(format!(
                "User auto-created from Authelia SSO: {}",
                info.email
            )),
            None,
        )
        .await;

        Ok(User {
            id: user_id,
            email: info.email.clone(),
            password_hash: placeholder_hash.to_string(),
            name,
            role: role.to_string(),
            created_at: now,
            last_login: Some(now),
            phone: None,
        })
    }
}

// ============================================================================
// AUTHELIA USER EXTRACTOR
// ============================================================================

/// Error response for JSON
#[derive(Serialize)]
pub struct ErrorResponse {
    error: String,
}

/// Authenticated user from Authelia headers (with fallback to Bearer token)
///
/// This extractor first checks for Authelia headers (when behind ForwardAuth proxy),
/// then falls back to the existing Bearer token authentication.
#[allow(dead_code)]
pub struct AutheliaUser(pub User);

#[axum::async_trait]
impl<S> axum::extract::FromRequestParts<S> for AutheliaUser
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

        // First, try Authelia headers (if enabled)
        if is_authelia_enabled() {
            if let Some(info) = AutheliaUserInfo::from_headers(&parts.headers) {
                // Sync user with database and return
                match sync_authelia_user(&pool, &info).await {
                    Ok(user) => return Ok(AutheliaUser(user)),
                    Err(e) => {
                        tracing::error!("Failed to sync Authelia user: {}", e);
                        return Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(ErrorResponse {
                                error: "Failed to process Authelia session".to_string(),
                            }),
                        ));
                    }
                }
            }
        }

        // Fallback to existing Bearer token authentication
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

        // Validate session using existing auth logic
        let user = crate::auth::validate_session(&pool, token)
            .await
            .map_err(|(status, msg)| (status, Json(ErrorResponse { error: msg })))?;

        Ok(AutheliaUser(user))
    }
}

// ============================================================================
// API ENDPOINTS
// ============================================================================

/// Get Authelia configuration for frontend
#[derive(Debug, Serialize)]
pub struct AutheliaConfig {
    pub enabled: bool,
    pub login_url: Option<String>,
    pub logout_url: Option<String>,
}

pub async fn get_authelia_config() -> Json<AutheliaConfig> {
    Json(AutheliaConfig {
        enabled: is_authelia_enabled(),
        login_url: get_authelia_login_url(),
        logout_url: get_authelia_logout_url(),
    })
}

/// Get current session from Authelia headers (for frontend session check)
pub async fn get_authelia_session(
    headers: HeaderMap,
    axum::extract::State(pool): axum::extract::State<DbPool>,
) -> Result<Json<UserPublic>, (StatusCode, Json<ErrorResponse>)> {
    if !is_authelia_enabled() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Authelia SSO not enabled".to_string(),
            }),
        ));
    }

    let info = AutheliaUserInfo::from_headers(&headers).ok_or((
        StatusCode::UNAUTHORIZED,
        Json(ErrorResponse {
            error: "Not authenticated via Authelia".to_string(),
        }),
    ))?;

    let user = sync_authelia_user(&pool, &info).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;

    Ok(Json(user.into()))
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn test_parse_authelia_headers() {
        let mut headers = HeaderMap::new();
        headers.insert("Remote-User", HeaderValue::from_static("test@example.com"));
        headers.insert("Remote-Name", HeaderValue::from_static("Test User"));
        headers.insert("Remote-Groups", HeaderValue::from_static("players,admins"));

        let info = AutheliaUserInfo::from_headers(&headers).unwrap();

        assert_eq!(info.email, "test@example.com");
        assert_eq!(info.name, Some("Test User".to_string()));
        assert_eq!(info.groups, vec!["players", "admins"]);
        assert!(info.is_admin());
        assert!(info.is_in_group("players"));
    }

    #[test]
    fn test_parse_authelia_headers_minimal() {
        let mut headers = HeaderMap::new();
        headers.insert("Remote-User", HeaderValue::from_static("user@test.it"));

        let info = AutheliaUserInfo::from_headers(&headers).unwrap();

        assert_eq!(info.email, "user@test.it");
        assert_eq!(info.name, None);
        assert!(info.groups.is_empty());
        assert!(!info.is_admin());
        assert_eq!(info.display_name(), "user");
    }

    #[test]
    fn test_parse_authelia_headers_invalid_email() {
        let mut headers = HeaderMap::new();
        headers.insert("Remote-User", HeaderValue::from_static("invalid-email"));

        let info = AutheliaUserInfo::from_headers(&headers);
        assert!(info.is_none());
    }

    #[test]
    fn test_parse_authelia_headers_missing() {
        let headers = HeaderMap::new();
        let info = AutheliaUserInfo::from_headers(&headers);
        assert!(info.is_none());
    }

    #[test]
    fn test_admin_detection() {
        let mut headers = HeaderMap::new();
        headers.insert("Remote-User", HeaderValue::from_static("admin@example.com"));
        headers.insert("Remote-Groups", HeaderValue::from_static("users,admins"));

        let info = AutheliaUserInfo::from_headers(&headers).unwrap();
        assert!(info.is_admin());

        // Test alternative admin group names
        headers.insert("Remote-Groups", HeaderValue::from_static("administrators"));
        let info = AutheliaUserInfo::from_headers(&headers).unwrap();
        assert!(info.is_admin());

        headers.insert("Remote-Groups", HeaderValue::from_static("players"));
        let info = AutheliaUserInfo::from_headers(&headers).unwrap();
        assert!(!info.is_admin());
    }

    #[test]
    fn test_email_fallback_header() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "Remote-Email",
            HeaderValue::from_static("fallback@example.com"),
        );

        let info = AutheliaUserInfo::from_headers(&headers).unwrap();
        assert_eq!(info.email, "fallback@example.com");
    }
}
