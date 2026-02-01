// Activity and Reminder Handlers for Axum
use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use sqlx::SqlitePool;

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
    State(pool): State<SqlitePool>,
    Query(query): Query<ActivityQuery>,
) -> Result<Json<Vec<Activity>>, StatusCode> {
    let limit = query.limit.unwrap_or(10);
    let offset = query.offset.unwrap_or(0);

    let activities = sqlx::query_as::<_, Activity>(
        "SELECT * FROM activities ORDER BY timestamp DESC LIMIT ? OFFSET ?",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(activities))
}

/// Helper: Log activity
pub async fn log_activity(
    pool: &SqlitePool,
    activity_type: &str,
    user_id: String,
    user_name: String,
    poll_id: Option<String>,
    poll_name: Option<String>,
) -> Result<(), sqlx::Error> {
    let activity = Activity::new(activity_type, user_id, user_name, poll_id, poll_name);

    sqlx::query(
        "INSERT INTO activities (id, activity_type, user_id, user_name, poll_id, poll_name, message, timestamp)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&activity.id)
    .bind(&activity.activity_type)
    .bind(&activity.user_id)
    .bind(&activity.user_name)
    .bind(&activity.poll_id)
    .bind(&activity.poll_name)
    .bind(&activity.message)
    .bind(activity.timestamp)
    .execute(pool)
    .await?;

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
    // Verifica configurazione
    let bot_token =
        std::env::var("TELEGRAM_BOT_TOKEN").map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;

    // Invia tramite Telegram Bot API
    let client = reqwest::Client::new();
    let url = format!("https://api.telegram.org/bot{}/sendMessage", bot_token);

    let params = serde_json::json!({
        "chat_id": req.chat_id,
        "text": req.message,
        "parse_mode": "HTML"
    });

    let response = client
        .post(&url)
        .json(&params)
        .send()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if response.status().is_success() {
        Ok(Json(ReminderResponse {
            success: true,
            message: "Promemoria Telegram inviato".to_string(),
        }))
    } else {
        Ok(Json(ReminderResponse {
            success: false,
            message: "Errore invio Telegram".to_string(),
        }))
    }
}

/// POST /api/reminder/email
pub async fn send_email_reminder(
    State(pool): State<SqlitePool>,
    Json(req): Json<EmailReminderRequest>,
) -> Result<Json<ReminderResponse>, StatusCode> {
    // 1. Fetch user email and name
    let (email, name): (String, String) =
        sqlx::query_as("SELECT email, name FROM users WHERE id = ?")
            .bind(&req.user_id)
            .fetch_optional(&pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)?;

    // 2. Fetch session/poll name for context (optional but good)
    let poll_title: String = sqlx::query_scalar("SELECT title FROM polls WHERE id = ?")
        .bind(&req.session_id)
        .fetch_optional(&pool)
        .await
        .unwrap_or(Some("Sessione D&D".to_string()))
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
