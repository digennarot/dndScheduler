use crate::db::DbPool;

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
// use sqlx::Row;

#[derive(Serialize)]
pub struct AdminStatsResponse {
    pub total_users: i64,
    pub online_users: i64,
    pub active_campaigns: i64,
    pub scheduled_sessions: i64,
}

pub async fn get_admin_stats(
    State(pool): State<DbPool>,
) -> Result<Json<AdminStatsResponse>, Response> {
    // 1. Total Users
    let total_users: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch total users: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error".to_string(),
            )
                .into_response()
        })?;

    // 2. Online Users (Active sessions not expired)
    // We count distinct user_ids from valid sessions
    let now = chrono::Utc::now().timestamp();
    let online_users: i64 = sqlx::query_scalar(
        "SELECT COUNT(DISTINCT user_id) FROM user_sessions WHERE expires_at > ?",
    )
    .bind(now)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch online users: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Database error".to_string(),
        )
            .into_response()
    })?;

    // 3. Active Campaigns (Polls that are active and recent - within last 6 months)
    let six_months_ago = now - (180 * 24 * 60 * 60);
    let active_campaigns: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM polls WHERE status = 'active' AND created_at > ?")
            .bind(six_months_ago)
            .fetch_one(&pool)
            .await
            .map_err(|e| {
                tracing::error!("Failed to fetch active campaigns: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Database error".to_string(),
                )
                    .into_response()
            })?;

    // 4. Scheduled Sessions (Polls that are finalized)
    let scheduled_sessions: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM polls WHERE status = 'finalized'")
            .fetch_one(&pool)
            .await
            .unwrap_or(0);

    Ok(Json(AdminStatsResponse {
        total_users,
        online_users,
        active_campaigns,
        scheduled_sessions,
    }))
}

// Story 3.5: Admin Poll Management
#[derive(Serialize)]
pub struct AdminPollSummary {
    #[serde(flatten)]
    pub poll: crate::core::models::Poll,
    pub organizer_name: String,
    pub participant_count: usize,
    pub instance_count: usize,
    pub last_activity: Option<i64>,
}

pub async fn get_admin_polls(
    State(_pool): State<DbPool>,
    axum::extract::Extension(polls_projection): axum::extract::Extension<
        std::sync::Arc<crate::core::projections::PollsProjection>,
    >,
    axum::extract::Extension(users_projection): axum::extract::Extension<
        std::sync::Arc<crate::core::projections::UsersProjection>,
    >,
    // Ensure admin auth
    _admin: crate::security::auth::AdminUser,
) -> Result<Json<Vec<AdminPollSummary>>, Response> {
    let polls = polls_projection.get_all();
    let mut summaries = Vec::new();

    for poll in polls {
        // Enrich with organizer name
        let organizer_name = if let Some(org_id) = &poll.organizer_id {
            users_projection
                .get(org_id)
                .map(|u| u.name)
                .unwrap_or_else(|| "Sconosciuto".to_string())
        } else {
            "Anonimo".to_string()
        };

        // Get counts from projection view
        // We need to fetch the full view for this, get_all only returned Poll structs
        // Efficient way: modify get_all or just fetch view here?
        // polls_projection.get_all() returns `Vec<Poll>`.
        // We should probably rely on `get` for details or just iterate the map if we had access.
        // But `PollsProjection` implementation of `get_all` only gives `Poll`.
        // Let's rely on `get` for now, even if N+1 (in-memory, so fast).

        let (participant_count, instance_count) = if let Some(view) = polls_projection.get(&poll.id)
        {
            (view.participants.len(), view.instances.len())
        } else {
            (0, 0)
        };

        summaries.push(AdminPollSummary {
            poll,
            organizer_name,
            participant_count,
            instance_count,
            last_activity: None, // TODO: Implement activity tracking link
        });
    }

    // Sort by creation date desc
    summaries.sort_by(|a, b| b.poll.created_at.cmp(&a.poll.created_at));

    Ok(Json(summaries))
}

// Story 3.1: Admin Token Exchange
#[derive(serde::Deserialize)]
pub struct AdminLoginRequest {
    pub token: String,
}

pub async fn admin_login(
    State(pool): State<DbPool>,
    Json(payload): Json<AdminLoginRequest>,
) -> Result<Response, (StatusCode, String)> {
    // 1. Validate Token against Env Var
    let expected_token = std::env::var("ADMIN_TOKEN").map_err(|_| {
        tracing::error!("ADMIN_TOKEN not set in environment");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Server configuration error".to_string(),
        )
    })?;

    if payload.token != expected_token {
        // Slow down brute force attacks slightly
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        return Err((StatusCode::UNAUTHORIZED, "Invalid admin token".to_string()));
    }

    // 2. JIT Provision 'admin' user if needed
    // We need a user in the DB to satisfy the FK constraint on user_sessions
    let admin_email = "admin@system.local";
    let admin_id = "00000000-0000-0000-0000-000000000000"; // Fixed UUID for admin

    let admin_exists: Option<(i64,)> = sqlx::query_as("SELECT 1 FROM users WHERE id = ?")
        .bind(admin_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if admin_exists.is_none() {
        let now = chrono::Utc::now().timestamp();
        // Insert dummy admin user
        sqlx::query(
            "INSERT INTO users (id, email, password_hash, name, role, created_at, last_login) VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(admin_id)
        .bind(admin_email)
        .bind("ADMIN_TOKEN_AUTH") // Sentinel hash
        .bind("System Admin")
        .bind("admin")
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create admin user: {}", e)))?;
    }

    // 3. Create Session
    let session_id = uuid::Uuid::new_v4().to_string();
    let session_token = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp();
    let expires_at = now + (24 * 60 * 60); // 24 hours

    sqlx::query(
        "INSERT INTO user_sessions (id, user_id, token, expires_at, created_at) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&session_id)
    .bind(admin_id) // Bind to the fixed admin user
    .bind(&session_token)
    .bind(expires_at)
    .bind(now)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create session: {}", e)))?;

    // 4. Return Cookie
    // We construct the Set-Cookie header manually
    let cookie_value = format!(
        "admin_session={}; HttpOnly; SameSite=Lax; Path=/; Max-Age=86400",
        session_token
    );

    let mut response = Json(serde_json::json!({
        "message": "Admin login successful",
        "role": "admin"
    }))
    .into_response();

    response.headers_mut().insert(
        axum::http::header::SET_COOKIE,
        cookie_value.parse().unwrap(),
    );

    Ok(response)
}
