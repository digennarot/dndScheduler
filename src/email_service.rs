use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::Message;
use std::env;

/// Check if email mocking is enabled
fn is_mock_mode() -> bool {
    env::var("MOCK_EMAIL")
        .unwrap_or_else(|_| "false".to_string())
        .to_lowercase() == "true"
}

/// Invia email generica
pub async fn send_email(to: &str, subject: &str, body: &str) -> Result<(), String> {
    // If mock mode is enabled, just log and return success
    if is_mock_mode() {
        println!("[MOCK EMAIL] To: {} | Subject: {} | Body: {}", to, subject, body);
        return Ok(());
    }

    // Wrapper for async implementation
    send_email_async(to, subject, body).await
}

async fn send_email_async(to: &str, subject: &str, body: &str) -> Result<(), String> {
    use lettre::AsyncSmtpTransport;
    use lettre::AsyncTransport;
    use lettre::Tokio1Executor; // Import trait for .send()

    // Carica configurazione
    let smtp_host = env::var("SMTP_HOST").map_err(|_| "SMTP_HOST not set".to_string())?;
    let smtp_port = env::var("SMTP_PORT")
        .unwrap_or_else(|_| "587".to_string())
        .parse::<u16>()
        .map_err(|_| "Invalid SMTP_PORT".to_string())?;
    let username = env::var("SMTP_USERNAME").map_err(|_| "SMTP_USERNAME not set".to_string())?;
    let password = env::var("SMTP_PASSWORD").map_err(|_| "SMTP_PASSWORD not set".to_string())?;
    let from_email =
        env::var("SMTP_FROM_EMAIL").unwrap_or_else(|_| "admin@cronachednd.it".to_string());

    let email = Message::builder()
        .from(
            from_email
                .parse()
                .map_err(|_| "Invalid FROM address".to_string())?,
        )
        .to(to.parse().map_err(|_| "Invalid TO address".to_string())?)
        .subject(subject)
        .header(ContentType::TEXT_HTML)
        .body(body.to_string())
        .map_err(|e| format!("Failed to build email: {}", e))?;

    let creds = Credentials::new(username, password);
    let mailer: AsyncSmtpTransport<Tokio1Executor> =
        AsyncSmtpTransport::<Tokio1Executor>::relay(&smtp_host)
            .map_err(|e| format!("Failed to create mailer: {}", e))?
            .port(smtp_port)
            .credentials(creds)
            .build();

    mailer
        .send(email)
        .await
        .map_err(|e| format!("Failed to send email: {}", e))?;

    Ok(())
}

/// Invia email di benvenuto
pub async fn send_welcome_email(email: &str, name: &str) -> Result<(), String> {
    let subject = "Benvenuto in D&D Scheduler!";
    let body = format!(
        r#"
        <h1>Benvenuto, {}!</h1>
        <p>Grazie per esserti registrato a D&D Scheduler.</p>
        <p>Ora puoi creare e partecipare alle campagne per organizzare le tue sessioni.</p>
        <br>
        <p>Che i dadi siano sempre a tuo favore!</p>
        "#,
        name
    );

    send_email(email, subject, &body).await
}

/// Invia email di promemoria
pub async fn send_reminder_email(
    email: &str,
    session_name: &str,
    message: &str,
) -> Result<(), String> {
    let subject = format!("Promemoria Sessione: {}", session_name);
    let body = format!(
        r#"
        <h2>Promemoria Sessione: {}</h2>
        <p>Ciao,</p>
        <p>Questo è un promemoria per la tua prossima sessione.</p>
        <p><strong>Messaggio:</strong> {}</p>
        <br>
        <p>A presto!</p>
        "#,
        session_name, message
    );

    send_email(email, &subject, &body).await
}
