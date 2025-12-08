use reqwest::Client;
use std::env;

/// Twilio WhatsApp configuration
#[derive(Debug, Clone)]
pub struct TwilioConfig {
    pub account_sid: String,
    pub auth_token: String,
    pub whatsapp_number: String,
}

impl TwilioConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, String> {
        let account_sid =
            env::var("TWILIO_ACCOUNT_SID").map_err(|_| "TWILIO_ACCOUNT_SID not set")?;
        let auth_token = env::var("TWILIO_AUTH_TOKEN").map_err(|_| "TWILIO_AUTH_TOKEN not set")?;
        let whatsapp_number = env::var("TWILIO_WHATSAPP_NUMBER")
            .unwrap_or_else(|_| "whatsapp:+14155238886".to_string());

        Ok(Self {
            account_sid,
            auth_token,
            whatsapp_number,
        })
    }
}

/// Validates a phone number is in E.164 format
pub fn validate_phone_number(phone: &str) -> bool {
    if phone.is_empty() {
        return false;
    }

    let cleaned = phone.trim();

    // Must start with + and contain only digits after
    if !cleaned.starts_with('+') {
        return false;
    }

    let digits = &cleaned[1..];

    // Check all remaining chars are digits
    if !digits.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }

    // E.164 requires 7-15 digits (including country code)
    let digit_count = digits.len();
    digit_count >= 7 && digit_count <= 15
}

/// Formats a phone number for WhatsApp (adds whatsapp: prefix)
pub fn format_whatsapp_number(phone: &str) -> String {
    let cleaned = phone
        .trim()
        .replace(" ", "")
        .replace("-", "")
        .replace("(", "")
        .replace(")", "");

    if cleaned.starts_with("whatsapp:") {
        cleaned
    } else if cleaned.starts_with('+') {
        format!("whatsapp:{}", cleaned)
    } else {
        // Assume Italian number if no country code
        format!("whatsapp:+39{}", cleaned)
    }
}

/// Builds reminder message body
pub fn build_reminder_message(session_name: &str, message: &str) -> String {
    format!(
        "🎲 Promemoria D&D Scheduler\n\n📅 Sessione: {}\n\n📝 {}\n\nChe i dadi siano sempre a tuo favore!",
        session_name, message
    )
}

/// Send a WhatsApp message via Twilio API
pub async fn send_whatsapp(to: &str, body: &str) -> Result<(), String> {
    let config = TwilioConfig::from_env()?;

    let formatted_to = format_whatsapp_number(to);

    if !validate_phone_number(&formatted_to.replace("whatsapp:", "")) {
        return Err(format!("Invalid phone number: {}", to));
    }

    let client = Client::new();

    let url = format!(
        "https://api.twilio.com/2010-04-01/Accounts/{}/Messages.json",
        config.account_sid
    );

    let params = [
        ("From", config.whatsapp_number.as_str()),
        ("To", formatted_to.as_str()),
        ("Body", body),
    ];

    let response = client
        .post(&url)
        .basic_auth(&config.account_sid, Some(&config.auth_token))
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Failed to send WhatsApp message: {}", e))?;

    if response.status().is_success() {
        Ok(())
    } else {
        let status = response.status();
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        Err(format!("Twilio API error ({}): {}", status, body))
    }
}

/// Send a session reminder via WhatsApp
pub async fn send_reminder_whatsapp(
    phone: &str,
    session_name: &str,
    message: &str,
) -> Result<(), String> {
    let body = build_reminder_message(session_name, message);
    send_whatsapp(phone, &body).await
}

/// Send a welcome WhatsApp message
pub async fn send_welcome_whatsapp(phone: &str, name: &str) -> Result<(), String> {
    let body = format!(
        "🎲 Benvenuto in D&D Scheduler, {}!\n\nGrazie per esserti registrato. Ora puoi partecipare alle sessioni di gioco.\n\nChe i dadi siano sempre a tuo favore!",
        name
    );
    send_whatsapp(phone, &body).await
}
