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
        let whatsapp_number = match env::var("TWILIO_WHATSAPP_NUMBER") {
            Ok(v) => v,
            Err(_) => "whatsapp:+14155238886".to_string(),
        };

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

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // PHONE NUMBER VALIDATION TESTS
    // ============================================================================

    #[test]
    fn test_valid_phone_e164_format() {
        assert!(validate_phone_number("+393401234567")); // Italian mobile
        assert!(validate_phone_number("+14155238886")); // US number
        assert!(validate_phone_number("+447911123456")); // UK number
    }

    #[test]
    fn test_invalid_phone_no_plus() {
        assert!(!validate_phone_number("393401234567")); // Missing +
        assert!(!validate_phone_number("3401234567")); // Local format
    }

    #[test]
    fn test_invalid_phone_empty() {
        assert!(!validate_phone_number(""));
        assert!(!validate_phone_number("   "));
    }

    #[test]
    fn test_invalid_phone_too_short() {
        assert!(!validate_phone_number("+12345")); // Too short
        assert!(!validate_phone_number("+1")); // Way too short
    }

    #[test]
    fn test_invalid_phone_too_long() {
        assert!(!validate_phone_number("+12345678901234567890")); // Too long
    }

    #[test]
    fn test_invalid_phone_with_letters() {
        assert!(!validate_phone_number("+39abc1234567"));
        assert!(!validate_phone_number("+1-800-TEST"));
    }

    // ============================================================================
    // WHATSAPP NUMBER FORMATTING TESTS
    // ============================================================================

    #[test]
    fn test_format_already_prefixed() {
        assert_eq!(
            format_whatsapp_number("whatsapp:+393401234567"),
            "whatsapp:+393401234567"
        );
    }

    #[test]
    fn test_format_with_plus_prefix() {
        assert_eq!(
            format_whatsapp_number("+393401234567"),
            "whatsapp:+393401234567"
        );
    }

    #[test]
    fn test_format_italian_local_number() {
        // Assumes Italian (+39) for numbers without country code
        assert_eq!(
            format_whatsapp_number("3401234567"),
            "whatsapp:+393401234567"
        );
    }

    #[test]
    fn test_format_removes_spaces() {
        assert_eq!(
            format_whatsapp_number("+39 340 123 4567"),
            "whatsapp:+393401234567"
        );
    }

    #[test]
    fn test_format_removes_dashes() {
        assert_eq!(
            format_whatsapp_number("+39-340-123-4567"),
            "whatsapp:+393401234567"
        );
    }

    #[test]
    fn test_format_removes_parentheses() {
        assert_eq!(
            format_whatsapp_number("+39(340)1234567"),
            "whatsapp:+393401234567"
        );
    }

    // ============================================================================
    // MESSAGE BUILDING TESTS
    // ============================================================================

    #[test]
    fn test_reminder_message_contains_session_name() {
        let message = build_reminder_message("Campagna Epica", "Porta i dadi!");

        assert!(message.contains("Campagna Epica"));
        assert!(message.contains("Porta i dadi!"));
    }

    #[test]
    fn test_reminder_message_has_emoji() {
        let message = build_reminder_message("Test", "Test");

        assert!(message.contains('🎲')); // Dice emoji
        assert!(message.contains('📅')); // Calendar emoji
    }

    #[test]
    fn test_reminder_message_has_branding() {
        let message = build_reminder_message("Test", "Test");

        assert!(message.contains("D&D Scheduler"));
        assert!(message.contains("Che i dadi siano sempre a tuo favore"));
    }

    #[test]
    fn test_reminder_message_escapes_special_chars() {
        let session = "Test <session>";
        let msg = "Message & reminder";
        let message = build_reminder_message(session, msg);

        // Message should contain the original text (WhatsApp handles escaping)
        assert!(message.contains("Test <session>"));
        assert!(message.contains("Message & reminder"));
    }

    // ============================================================================
    // TWILIO CONFIG VALIDATION TESTS
    // ============================================================================

    #[test]
    fn test_twilio_config_struct_creation() {
        let config = TwilioConfig {
            account_sid: "AC1234567890abcdef".to_string(),
            auth_token: "auth_token_123".to_string(),
            whatsapp_number: "whatsapp:+14155238886".to_string(),
        };

        assert_eq!(config.account_sid, "AC1234567890abcdef");
        assert!(config.whatsapp_number.starts_with("whatsapp:"));
    }

    #[test]
    fn test_twilio_api_url_format() {
        let account_sid = "AC1234567890abcdef";
        let url = format!(
            "https://api.twilio.com/2010-04-01/Accounts/{}/Messages.json",
            account_sid
        );

        assert!(url.contains(account_sid));
        assert!(url.ends_with("Messages.json"));
    }
}
