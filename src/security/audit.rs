use crate::core::models::AuditLog;
use crate::db::DbPool;
use crate::db::tables;
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
    
    let log_entry = AuditLog {
        id: uuid::Uuid::new_v4().to_string(),
        user_id: user_id.clone(),
        action: action.to_string(),
        resource: resource.clone(),
        timestamp: now,
        ip_address: ip_address.clone(),
        user_agent: Some(user_agent.to_string()),
        success,
        details: details.clone(),
    };

    if let Ok(write_txn) = pool.begin_write() {
        if let Ok(mut table) = write_txn.open_table(tables::AUDIT_LOG_TABLE) {
            let key = format!("{}:{}", now, log_entry.id);
            if let Ok(bytes) = bincode::serialize(&log_entry) {
                let _ = table.insert(key.as_str(), bytes.as_slice());
            }
        }
        let _ = write_txn.commit();
    }
}
