use crate::db::DbPool;

use axum::{
    extract::{Path, State},
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
    _admin: crate::security::auth::AdminUser,
) -> Result<Json<AdminStatsResponse>, Response> {
    let now = chrono::Utc::now().timestamp();
    let six_months_ago = now - (180 * 24 * 60 * 60);

    let (total_users, online_users, active_campaigns, scheduled_sessions) = pool.begin_read()
        .map_err(|e| {
            tracing::error!("Failed to begin read tx: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string()).into_response()
        }).and_then(|read_txn| {
            use redb::ReadableTable;
            use crate::db::tables;
            
            let mut t_users = 0;
            if let Ok(table) = read_txn.open_table(tables::USERS_TABLE) {
                t_users = table.iter().map(|i| i.count()).unwrap_or(0) as i64;
            }

            let mut o_users = 0;
            if let Ok(table) = read_txn.open_table(tables::USER_SESSIONS_TABLE) {
                let mut distinct_users = std::collections::HashSet::new();
                if let Ok(iter) = table.iter() {
                    for result in iter {
                        if let Ok((_, v)) = result {
                            let sess: crate::core::models::UserSession = bincode::deserialize(v.value()).unwrap();
                            if sess.expires_at > now {
                                distinct_users.insert(sess.user_id);
                            }
                        }
                    }
                }
                o_users = distinct_users.len() as i64;
            }

            let mut active_c = 0;
            let mut scheduled_s = 0;
            if let Ok(table) = read_txn.open_table(tables::POLLS_TABLE) {
                if let Ok(iter) = table.iter() {
                    for result in iter {
                        if let Ok((_, v)) = result {
                            let poll: crate::core::models::Poll = bincode::deserialize(v.value()).unwrap();
                            if poll.status == "active" && poll.created_at > six_months_ago {
                                active_c += 1;
                            }
                            if poll.status == "finalized" {
                                scheduled_s += 1;
                            }
                        }
                    }
                }
            }

            Ok((t_users, o_users, active_c, scheduled_s))
        })?;

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

    let admin_exists = crate::db::queries::user_repo::UserRepo::find_by_id(&pool, admin_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if admin_exists.is_none() {
        let now = chrono::Utc::now().timestamp();
        // Insert dummy admin user
        let new_user = crate::core::models::User {
            id: admin_id.to_string(),
            email: admin_email.to_string(),
            password_hash: "ADMIN_TOKEN_AUTH".to_string(),
            name: "System Admin".to_string(),
            role: "admin".to_string(),
            created_at: now,
            last_login: Some(now),
            phone: None,
            consent_marketing: false,
            consent_analytics: false,
            privacy_policy_accepted_at: None,
        };
        crate::db::queries::user_repo::UserRepo::create_or_update(&pool, &new_user)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create admin user: {}", e)))?;
    }

    // 3. Create Session
    let session_token = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp();
    let expires_at = now + (24 * 60 * 60); // 24 hours

    let session = crate::core::models::UserSession {
        id: uuid::Uuid::new_v4().to_string(),
        user_id: admin_id.to_string(),
        token: session_token.clone(),
        expires_at,
        created_at: now,
    };
    
    crate::db::queries::admin_repo::SessionRepo::create_user_session(&pool, &session)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

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

// Story 3.3: Delete APIs
pub async fn delete_poll(
    State(pool): State<DbPool>,
    axum::extract::Extension(event_store): axum::extract::Extension<std::sync::Arc<crate::core::store::RedbEventStore>>,
    axum::extract::Extension(projection): axum::extract::Extension<std::sync::Arc<crate::core::projections::PollsProjection>>,
    // Ensure admin auth
    _admin: crate::security::auth::AdminUser,
    Path(poll_id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    crate::db::queries::poll_repo::PollRepo::delete_poll(&pool, &poll_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let event = crate::core::events::Event::V1PollDeleted(crate::core::events::PollDeletedV1 {
        id: poll_id.clone(),
    });

    let stream_id = format!("poll-{}", poll_id);
    if let Err(e) = event_store.append_event(&stream_id, &event).await {
        tracing::error!("Failed to append PollDeletedV1 event: {}", e);
    } else {
        projection.apply(event);
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_vote(
    State(pool): State<DbPool>,
    axum::extract::Extension(event_store): axum::extract::Extension<std::sync::Arc<crate::core::store::RedbEventStore>>,
    axum::extract::Extension(projection): axum::extract::Extension<std::sync::Arc<crate::core::projections::PollsProjection>>,
    // Ensure admin auth
    _admin: crate::security::auth::AdminUser,
    Path((poll_id, participant_id)): Path<(String, String)>,
) -> Result<StatusCode, (StatusCode, String)> {
    crate::db::queries::poll_repo::PollRepo::delete_vote(&pool, &poll_id, &participant_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let event = crate::core::events::Event::V1ParticipantRemoved(crate::core::events::ParticipantRemovedV1 {
        poll_id: poll_id.clone(),
        participant_id,
    });

    let stream_id = format!("poll-{}", poll_id);
    if let Err(e) = event_store.append_event(&stream_id, &event).await {
        tracing::error!("Failed to append ParticipantRemovedV1 event: {}", e);
    } else {
        projection.apply(event);
    }

    Ok(StatusCode::NO_CONTENT)
}
