use crate::db::DbPool;
use chrono::Utc;

pub async fn log_audit(
    pool: &DbPool,
    user_id: Option<String>,
    action: &str,
    resource: Option<String>,
    success: bool,
    details: Option<String>,
    ip_address: Option<String>,
) {
    let now = Utc::now().timestamp();
    let user_agent = "unknown"; // Extract from request in future

    sqlx::query(
        "INSERT INTO audit_log (user_id, action, resource, timestamp, ip_address, user_agent, success, details) VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(user_id)
    .bind(action)
    .bind(resource)
    .bind(now)
    .bind(ip_address)
    .bind(user_agent)
    .bind(success)
    .bind(details)
    .execute(pool)
    .await
    .ok();
}
