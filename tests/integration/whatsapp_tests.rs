// WhatsApp Service Integration Tests
// Test di integrazione per il servizio WhatsApp con TDD

#[cfg(test)]
mod whatsapp_tests {
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

    // ============================================================================
    // WELCOME MESSAGE TESTS
    // ============================================================================

    #[test]
    fn test_welcome_message_contains_name() {
        let message = build_welcome_message("Tiziano");

        assert!(message.contains("Tiziano"));
        assert!(message.contains("Benvenuto"));
    }

    #[test]
    fn test_welcome_message_has_branding() {
        let message = build_welcome_message("Test");

        assert!(message.contains("D&D Scheduler"));
    }

    // ============================================================================
    // HELPER FUNCTIONS AND STRUCTS (for testing)
    // ============================================================================

    #[allow(dead_code)]
    #[derive(Debug, Clone)]
    struct TwilioConfig {
        account_sid: String,
        auth_token: String,
        whatsapp_number: String,
    }

    fn validate_phone_number(phone: &str) -> bool {
        if phone.trim().is_empty() {
            return false;
        }

        let cleaned = phone.trim();

        // Must start with +
        if !cleaned.starts_with('+') {
            return false;
        }

        let digits = &cleaned[1..];

        // Check all remaining chars are digits
        if !digits.chars().all(|c| c.is_ascii_digit()) {
            return false;
        }

        // E.164 requires 7-15 digits
        let digit_count = digits.len();
        digit_count >= 7 && digit_count <= 15
    }

    fn format_whatsapp_number(phone: &str) -> String {
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

    fn build_reminder_message(session_name: &str, message: &str) -> String {
        format!(
            "🎲 Promemoria D&D Scheduler\n\n📅 Sessione: {}\n\n📝 {}\n\nChe i dadi siano sempre a tuo favore!",
            session_name, message
        )
    }

    fn build_welcome_message(name: &str) -> String {
        format!(
            "🎲 Benvenuto in D&D Scheduler, {}!\n\nGrazie per esserti registrato. Ora puoi partecipare alle sessioni di gioco.\n\nChe i dadi siano sempre a tuo favore!",
            name
        )
    }
}
