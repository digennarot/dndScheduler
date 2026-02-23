use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::Message;
use std::env;

/// Check if email mocking is enabled
fn is_mock_mode() -> bool {
    match env::var("MOCK_EMAIL") {
        Ok(v) => v,
        Err(_) => "false".to_string(),
    }
    .to_lowercase()
        == "true"
}

/// Escape special HTML characters to prevent XSS when interpolating into HTML email bodies
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

/// Invia email generica
pub async fn send_email(to: &str, subject: &str, body: &str) -> Result<(), String> {
    // If mock mode is enabled, just log and return success
    if is_mock_mode() {
        println!(
            "[MOCK EMAIL] To: {} | Subject: {} | Body: {}",
            to, subject, body
        );
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
    let smtp_port = match env::var("SMTP_PORT") {
        Ok(v) => v,
        Err(_) => "587".to_string(),
    }
    .parse::<u16>()
    .map_err(|_| "Invalid SMTP_PORT".to_string())?;
    let username = env::var("SMTP_USERNAME").map_err(|_| "SMTP_USERNAME not set".to_string())?;
    let password = env::var("SMTP_PASSWORD").map_err(|_| "SMTP_PASSWORD not set".to_string())?;
    let from_email = match env::var("SMTP_FROM_EMAIL") {
        Ok(v) => v,
        Err(_) => "admin@cronachednd.it".to_string(),
    };

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

/// Invia email di invito a partecipare a un sondaggio
pub async fn send_invite_email(
    email: &str,
    player_name: &str,
    session_title: &str,
    organizer_name: &str,
    participate_url: &str,
) -> Result<(), String> {
    // Escape all interpolated values to prevent XSS in email HTML (M3)
    let player_name_h  = html_escape(player_name);
    let session_title_h = html_escape(session_title);
    let organizer_name_h = html_escape(organizer_name);
    // participate_url is URL-safe (built from NanoID + UUID) — still escape for safety
    let participate_url_h = html_escape(participate_url);

    let subject = format!("Sei stato invitato a: {}", session_title);
    let body = format!(
        r#"
        <div style="font-family: Arial, sans-serif; max-width: 600px; margin: 0 auto; padding: 20px;">
            <h1 style="color: #6b21a8;">⚔️ Invito alla Sessione D&amp;D</h1>
            <p>Ciao <strong>{player_name_h}</strong>!</p>
            <p><strong>{organizer_name_h}</strong> ti ha invitato a partecipare all'organizzazione della sessione:</p>
            <h2 style="color: #7c3aed;">{session_title_h}</h2>
            <p>Clicca sul link qui sotto per indicare la tua disponibilità:</p>
            <p style="text-align: center; margin: 30px 0;">
                <a href="{participate_url_h}"
                   style="background-color: #7c3aed; color: white; padding: 14px 28px;
                          text-decoration: none; border-radius: 8px; font-size: 16px; font-weight: bold;">
                    🎲 Indica la tua disponibilità
                </a>
            </p>
            <p style="color: #6b7280; font-size: 14px;">
                Oppure copia e incolla questo link nel browser:<br>
                <a href="{participate_url_h}">{participate_url_h}</a>
            </p>
            <hr style="border: none; border-top: 1px solid #e5e7eb; margin: 20px 0;">
            <p style="color: #9ca3af; font-size: 12px;">
                Che i dadi siano sempre a tuo favore! 🎲<br>
                D&amp;D Scheduler
            </p>
        </div>
        "#,
        player_name_h = player_name_h,
        organizer_name_h = organizer_name_h,
        session_title_h = session_title_h,
        participate_url_h = participate_url_h,
    );

    send_email(email, &subject, &body).await
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
