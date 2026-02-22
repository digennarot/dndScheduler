// Activity and Reminder Handlers for Axum
use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use crate::db::DbPool;

use crate::core::models::*;

// ============================================================================
// ACTIVITY HANDLERS
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ActivityQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// GET /api/activity/recent
pub async fn get_recent_activity(
    State(pool): State<DbPool>,
    Query(query): Query<ActivityQuery>,
) -> Result<Json<Vec<Activity>>, StatusCode> {
    let limit = query.limit.unwrap_or(10);
    let offset = query.offset.unwrap_or(0);

    let activities = crate::db::queries::activity_repo::ActivityRepo::get_recent_activity(&pool, limit, offset)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(activities))
}

/// Helper: Log activity
pub async fn log_activity(
    pool: &DbPool,
    activity_type: &str,
    user_id: String,
    user_name: String,
    poll_id: Option<String>,
    poll_name: Option<String>,
) -> Result<(), anyhow::Error> {
    let activity = Activity::new(activity_type, user_id, user_name, poll_id, poll_name);
    crate::db::queries::activity_repo::ActivityRepo::log_activity(pool, &activity)?;

    Ok(())
}

// ============================================================================
// REMINDER HANDLERS
// ============================================================================

/// GET /api/reminder/config
pub async fn get_reminder_config() -> Json<ReminderConfig> {
    Json(ReminderConfig {
        whatsapp_enabled: std::env::var("TWILIO_ACCOUNT_SID").is_ok(),
        telegram_enabled: std::env::var("TELEGRAM_BOT_TOKEN").is_ok(),
        email_enabled: true,
    })
}

/// POST /api/reminder/whatsapp
pub async fn send_whatsapp_reminder(
    Json(req): Json<WhatsAppReminderRequest>,
) -> Result<Json<ReminderResponse>, StatusCode> {
    // Use the dedicated WhatsApp service module
    match crate::core::services::whatsapp::send_reminder_whatsapp(
        &req.phone,
        "Sessione D&D", // Could be enhanced to pass actual session name
        &req.message,
    )
    .await
    {
        Ok(()) => Ok(Json(ReminderResponse {
            success: true,
            message: "Promemoria WhatsApp inviato".to_string(),
        })),
        Err(e) => {
            tracing::error!("Failed to send WhatsApp reminder: {}", e);
            Ok(Json(ReminderResponse {
                success: false,
                message: format!("Errore invio WhatsApp: {}", e),
            }))
        }
    }
}

/// POST /api/reminder/telegram
pub async fn send_telegram_reminder(
    Json(req): Json<TelegramReminderRequest>,
) -> Result<Json<ReminderResponse>, StatusCode> {
    // Use the dedicated Telegram service module
    match crate::core::services::telegram::send_reminder_telegram(
        &req.chat_id,
        "Sessione D&D", // Context could be improved
        &req.message,
    )
    .await
    {
        Ok(()) => Ok(Json(ReminderResponse {
            success: true,
            message: "Promemoria Telegram inviato".to_string(),
        })),
        Err(e) => {
            tracing::error!("Failed to send Telegram reminder: {}", e);
            Ok(Json(ReminderResponse {
                success: false,
                message: format!("Errore invio Telegram: {}", e),
            }))
        }
    }
}

/// POST /api/reminder/email
pub async fn send_email_reminder(
    State(pool): State<DbPool>,
    Json(req): Json<EmailReminderRequest>,
) -> Result<Json<ReminderResponse>, StatusCode> {
    // 1. Fetch user email and name
    let user = crate::db::queries::user_repo::UserRepo::find_by_id(&pool, &req.user_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    
    let email = user.email;
    let name = user.name;

    // 2. Fetch session/poll name for context (optional but good)
    let poll_title = crate::db::queries::poll_repo::PollRepo::get_details(&pool, &req.session_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map(|(p, _, _, _)| p.title)
        .unwrap_or("Sessione D&D".to_string());

    // 3. Send email
    if let Err(e) =
        crate::core::services::email::send_reminder_email(&email, &poll_title, &req.message).await
    {
        tracing::error!("Failed to send reminder email to {}: {}", email, e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    // 4. Log activity
    crate::activity_handlers::log_activity(
        &pool,
        "reminder_sent",
        req.user_id,
        name,
        Some(req.session_id),
        Some(poll_title),
    )
    .await
    .ok();

    Ok(Json(ReminderResponse {
        success: true,
        message: "Email inviata con successo".to_string(),
    }))
}
