use reqwest::Client;
use std::env;

/// Telegram configuration
#[derive(Debug, Clone)]
pub struct TelegramConfig {
    pub bot_token: String,
}

impl TelegramConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, String> {
        let bot_token = env::var("TELEGRAM_BOT_TOKEN").map_err(|_| "TELEGRAM_BOT_TOKEN not set")?;
        Ok(Self { bot_token })
    }
}

/// Send a Telegram message via Bot API
pub async fn send_telegram(chat_id: &str, message: &str) -> Result<(), String> {
    let config = TelegramConfig::from_env()?;

    let client = Client::new();
    let url = format!(
        "https://api.telegram.org/bot{}/sendMessage",
        config.bot_token
    );

    let params = serde_json::json!({
        "chat_id": chat_id,
        "text": message,
        "parse_mode": "HTML" // Support basic formatting
    });

    let response = client
        .post(&url)
        .json(&params)
        .send()
        .await
        .map_err(|e| format!("Failed to send Telegram message: {}", e))?;

    if response.status().is_success() {
        Ok(())
    } else {
        let status = response.status();
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        Err(format!("Telegram API error ({}): {}", status, body))
    }
}

/// Send a session reminder via Telegram
pub async fn send_reminder_telegram(
    chat_id: &str,
    session_name: &str,
    message: &str,
) -> Result<(), String> {
    let body = format!(
        "<b>🎲 Promemoria Sessione: {}</b>\n\n{}\n\n<i>Che i dadi siano sempre a tuo favore!</i>",
        session_name, message
    );
    send_telegram(chat_id, &body).await
}

/// Send a welcome Telegram message
pub async fn send_welcome_telegram(chat_id: &str, name: &str) -> Result<(), String> {
    let body = format!(
        "<b>🎲 Benvenuto in D&D Scheduler, {}!</b>\n\nGrazie per esserti registrato. Ora puoi partecipare alle sessioni di gioco.\n\n<i>Che i dadi siano sempre a tuo favore!</i>",
        name
    );
    send_telegram(chat_id, &body).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_from_env() {
        // Only run if env var is set, or mock it
        if std::env::var("TELEGRAM_BOT_TOKEN").is_ok() {
            let config = TelegramConfig::from_env();
            assert!(config.is_ok());
        } else {
            // Mock for test stability if not running in a conditioned env
            unsafe {
                std::env::set_var("TELEGRAM_BOT_TOKEN", "test_token");
            }
            let config = TelegramConfig::from_env();
            assert!(config.is_ok());
            assert_eq!(config.unwrap().bot_token, "test_token");
            unsafe {
                std::env::remove_var("TELEGRAM_BOT_TOKEN");
            }
        }
    }
}
