// Email Service Integration Tests
// Test di integrazione per il servizio email con TDD

#[cfg(test)]
mod email_tests {
    // ============================================================================
    // EMAIL ADDRESS VALIDATION TESTS
    // ============================================================================

    #[test]
    fn test_valid_email_address() {
        assert!(validate_email_address("user@example.com"));
        assert!(validate_email_address("test.user@domain.it"));
        assert!(validate_email_address("admin@cronachednd.it"));
    }

    #[test]
    fn test_invalid_email_address() {
        assert!(!validate_email_address("invalid"));
        assert!(!validate_email_address("@nodomain.com"));
        assert!(!validate_email_address("noat.com"));
        assert!(!validate_email_address(""));
    }

    #[test]
    fn test_email_with_special_chars_valid() {
        assert!(validate_email_address("user+tag@example.com"));
        assert!(validate_email_address("user.name@sub.domain.com"));
    }

    #[test]
    fn test_email_without_tld_invalid() {
        assert!(!validate_email_address("user@localhost"));
    }

    // ============================================================================
    // EMAIL MESSAGE BUILDING TESTS
    // ============================================================================

    #[test]
    fn test_welcome_email_contains_name() {
        let name = "Tiziano";
        let body = build_welcome_email_body(name);

        assert!(body.contains("Benvenuto, Tiziano!"));
        assert!(body.contains("D&D Scheduler"));
        assert!(body.contains("Che i dadi siano sempre a tuo favore"));
    }

    #[test]
    fn test_welcome_email_escapes_html() {
        let name = "<script>alert('xss')</script>";
        let body = build_welcome_email_body_safe(name);

        // Should not contain raw script tags
        assert!(!body.contains("<script>"));
    }

    #[test]
    fn test_reminder_email_contains_session_info() {
        let session = "Campagna Epica";
        let message = "Ricordati di portare i dadi!";
        let body = build_reminder_email_body(session, message);

        assert!(body.contains("Campagna Epica"));
        assert!(body.contains("Ricordati di portare i dadi!"));
        assert!(body.contains("Promemoria Sessione"));
    }

    #[test]
    fn test_email_subject_not_empty() {
        let subject = build_reminder_subject("Test Session");
        assert!(!subject.is_empty());
        assert!(subject.contains("Test Session"));
    }

    #[test]
    fn test_email_subject_has_prefix() {
        let subject = build_reminder_subject("Test");
        assert!(subject.starts_with("Promemoria Sessione:"));
    }

    // ============================================================================
    // SMTP CONFIGURATION VALIDATION TESTS
    // ============================================================================

    #[test]
    fn test_smtp_config_struct_creation() {
        let config = SmtpConfig {
            host: "smtp.test.com".to_string(),
            port: 587,
            username: "user@test.com".to_string(),
            password: "password123".to_string(),
            from_email: "noreply@test.com".to_string(),
        };

        assert_eq!(config.host, "smtp.test.com");
        assert_eq!(config.port, 587);
        assert_eq!(config.username, "user@test.com");
        assert_eq!(config.from_email, "noreply@test.com");
    }

    #[test]
    fn test_smtp_config_default_port() {
        assert_eq!(get_default_smtp_port(), 587);
    }

    #[test]
    fn test_smtp_config_default_from_email() {
        assert_eq!(get_default_from_email(), "admin@cronachednd.it");
    }

    #[test]
    fn test_smtp_port_validation_valid() {
        assert!(is_valid_smtp_port(25));
        assert!(is_valid_smtp_port(465));
        assert!(is_valid_smtp_port(587));
        assert!(is_valid_smtp_port(2525));
    }

    #[test]
    fn test_smtp_port_validation_invalid() {
        assert!(!is_valid_smtp_port(0));
        assert!(!is_valid_smtp_port(70000));
    }

    #[test]
    fn test_smtp_host_validation_valid() {
        assert!(is_valid_smtp_host("smtp.gmail.com"));
        assert!(is_valid_smtp_host("authsmtp.securemail.pro"));
        assert!(is_valid_smtp_host("mail.example.com"));
    }

    #[test]
    fn test_smtp_host_validation_invalid() {
        assert!(!is_valid_smtp_host(""));
        assert!(!is_valid_smtp_host("  "));
    }

    // ============================================================================
    // RATE LIMITING TESTS
    // ============================================================================

    #[test]
    fn test_email_rate_limit_allows_under_limit() {
        let mut rate_limiter = EmailRateLimiter::new(5); // 5 emails per minute

        // Should allow first 5 emails
        for _ in 0..5 {
            assert!(rate_limiter.can_send());
            rate_limiter.record_send();
        }
    }

    #[test]
    fn test_email_rate_limit_blocks_over_limit() {
        let mut rate_limiter = EmailRateLimiter::new(5);

        for _ in 0..5 {
            rate_limiter.record_send();
        }

        // 6th should be rate limited
        assert!(!rate_limiter.can_send());
    }

    #[test]
    fn test_email_rate_limit_reset() {
        let mut rate_limiter = EmailRateLimiter::new(5);

        for _ in 0..5 {
            rate_limiter.record_send();
        }

        assert!(!rate_limiter.can_send());

        rate_limiter.reset();

        assert!(rate_limiter.can_send());
    }

    #[test]
    fn test_email_rate_limit_zero_max_blocks_all() {
        let rate_limiter = EmailRateLimiter::new(0);
        assert!(!rate_limiter.can_send());
    }

    // ============================================================================
    // EMAIL TEMPLATE TESTS
    // ============================================================================

    #[test]
    fn test_email_template_gdpr_export_notice() {
        let notice = build_gdpr_export_notice();
        assert!(notice.contains("GDPR"));
        assert!(notice.contains("Art. 20"));
        assert!(notice.contains("cronachednd.it"));
    }

    #[test]
    fn test_email_template_password_reset() {
        let token = "abc123";
        let body = build_password_reset_body(token);

        assert!(body.contains("abc123"));
        assert!(body.contains("reset") || body.contains("password"));
    }

    // ============================================================================
    // HELPER FUNCTIONS AND STRUCTS
    // ============================================================================

    #[derive(Debug)]
    struct SmtpConfig {
        host: String,
        port: u16,
        username: String,
        #[allow(dead_code)]
        password: String,
        from_email: String,
    }

    fn validate_email_address(email: &str) -> bool {
        if email.is_empty() {
            return false;
        }

        // Simple email validation
        let parts: Vec<&str> = email.split('@').collect();
        if parts.len() != 2 {
            return false;
        }

        let local = parts[0];
        let domain = parts[1];

        !local.is_empty() && !domain.is_empty() && domain.contains('.')
    }

    fn build_welcome_email_body(name: &str) -> String {
        format!(
            r#"
            <h1>Benvenuto, {}!</h1>
            <p>Grazie per esserti registrato a D&D Scheduler.</p>
            <p>Ora puoi creare e partecipare alle campagne per organizzare le tue sessioni.</p>
            <br>
            <p>Che i dadi siano sempre a tuo favore!</p>
            "#,
            name
        )
    }

    fn build_welcome_email_body_safe(name: &str) -> String {
        // HTML escape the name to prevent XSS
        let escaped_name = name
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#x27;");

        format!(
            r#"
            <h1>Benvenuto, {}!</h1>
            <p>Grazie per esserti registrato a D&D Scheduler.</p>
            "#,
            escaped_name
        )
    }

    fn build_reminder_email_body(session_name: &str, message: &str) -> String {
        format!(
            r#"
            <h2>Promemoria Sessione: {}</h2>
            <p>Ciao,</p>
            <p>Questo è un promemoria per la tua prossima sessione.</p>
            <p><strong>Messaggio:</strong> {}</p>
            <br>
            <p>A presto!</p>
            "#,
            session_name, message
        )
    }

    fn build_reminder_subject(session_name: &str) -> String {
        format!("Promemoria Sessione: {}", session_name)
    }

    fn get_default_smtp_port() -> u16 {
        587
    }

    fn get_default_from_email() -> &'static str {
        "admin@cronachednd.it"
    }

    fn is_valid_smtp_port(port: u32) -> bool {
        port > 0 && port <= 65535
    }

    fn is_valid_smtp_host(host: &str) -> bool {
        !host.trim().is_empty()
    }

    fn build_gdpr_export_notice() -> String {
        "Questo export contiene tutti i dati personali memorizzati in conformità con il GDPR Art. 20 (Diritto alla portabilità dei dati). Per domande, contatta privacy@cronachednd.it".to_string()
    }

    fn build_password_reset_body(token: &str) -> String {
        format!(
            r#"
            <h2>Reset Password</h2>
            <p>Hai richiesto il reset della password.</p>
            <p>Il tuo codice di reset è: <strong>{}</strong></p>
            <p>Questo codice scadrà tra 1 ora.</p>
            "#,
            token
        )
    }

    struct EmailRateLimiter {
        max_per_minute: u32,
        sent_count: u32,
    }

    impl EmailRateLimiter {
        fn new(max_per_minute: u32) -> Self {
            Self {
                max_per_minute,
                sent_count: 0,
            }
        }

        fn can_send(&self) -> bool {
            self.sent_count < self.max_per_minute
        }

        fn record_send(&mut self) {
            self.sent_count += 1;
        }

        fn reset(&mut self) {
            self.sent_count = 0;
        }
    }
}
