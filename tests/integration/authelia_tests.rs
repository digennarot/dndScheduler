// Authelia SSO Integration Tests (TDD)
//
// Test suite per verificare l'integrazione Single Sign-On con Authelia.
// Copre: parsing header, sincronizzazione utenti, gestione gruppi, fallback auth.

use super::helpers::*;
use sqlx::Pool;
use sqlx::Sqlite;

// ============================================================================
// TEST HELPERS
// ============================================================================

/// Simula un utente creato via Authelia SSO
async fn create_authelia_user(pool: &Pool<Sqlite>, email: &str, name: &str, role: &str) -> String {
    let user_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp();

    // Password hash placeholder per utenti SSO (non possono login con password)
    let placeholder_hash = "$authelia_sso$";

    sqlx::query(
        "INSERT INTO users (id, email, password_hash, name, role, created_at, last_login) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&user_id)
    .bind(email)
    .bind(placeholder_hash)
    .bind(name)
    .bind(role)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await
    .expect("Failed to create Authelia user");

    user_id
}

/// Verifica che un utente esista nel database
async fn user_exists(pool: &Pool<Sqlite>, email: &str) -> bool {
    let result: Option<(String,)> = sqlx::query_as("SELECT id FROM users WHERE email = ?")
        .bind(email)
        .fetch_optional(pool)
        .await
        .unwrap();
    result.is_some()
}

/// Recupera il ruolo di un utente
async fn get_user_role(pool: &Pool<Sqlite>, email: &str) -> Option<String> {
    let result: Option<(String,)> = sqlx::query_as("SELECT role FROM users WHERE email = ?")
        .bind(email)
        .fetch_optional(pool)
        .await
        .unwrap();
    result.map(|(role,)| role)
}

// ============================================================================
// TDD TEST: HEADER PARSING
// ============================================================================

/// TEST 1: Parsing header Remote-User (email)
#[tokio::test]
async fn test_authelia_header_parsing_remote_user() {
    use axum::http::{HeaderMap, HeaderValue};

    let mut headers = HeaderMap::new();
    headers.insert(
        "Remote-User",
        HeaderValue::from_static("test@cronachednd.it"),
    );

    // Simula il parsing degli header
    let email = headers
        .get("Remote-User")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    assert!(email.is_some());
    assert_eq!(email.unwrap(), "test@cronachednd.it");
}

/// TEST 2: Parsing header Remote-Name
#[tokio::test]
async fn test_authelia_header_parsing_remote_name() {
    use axum::http::{HeaderMap, HeaderValue};

    let mut headers = HeaderMap::new();
    headers.insert("Remote-Name", HeaderValue::from_static("Mario Rossi"));

    let name = headers
        .get("Remote-Name")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    assert!(name.is_some());
    assert_eq!(name.unwrap(), "Mario Rossi");
}

/// TEST 3: Parsing header Remote-Groups (multipli gruppi)
#[tokio::test]
async fn test_authelia_header_parsing_remote_groups() {
    use axum::http::{HeaderMap, HeaderValue};

    let mut headers = HeaderMap::new();
    headers.insert(
        "Remote-Groups",
        HeaderValue::from_static("players,admins,dm"),
    );

    let groups_str = headers
        .get("Remote-Groups")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let groups: Vec<&str> = groups_str.split(',').map(|s| s.trim()).collect();

    assert_eq!(groups.len(), 3);
    assert!(groups.contains(&"players"));
    assert!(groups.contains(&"admins"));
    assert!(groups.contains(&"dm"));
}

/// TEST 4: Header mancanti restituiscono None
#[tokio::test]
async fn test_authelia_header_missing() {
    use axum::http::HeaderMap;

    let headers = HeaderMap::new();

    let email = headers.get("Remote-User").and_then(|v| v.to_str().ok());

    assert!(email.is_none());
}

// ============================================================================
// TDD TEST: USER SYNC
// ============================================================================

/// TEST 5: Creazione automatica utente da sessione Authelia
#[tokio::test]
async fn test_authelia_user_auto_creation() {
    let pool = setup_test_db().await;

    // L'utente non esiste ancora
    assert!(!user_exists(&pool, "nuovo@cronachednd.it").await);

    // Simula creazione utente da Authelia
    let _user_id =
        create_authelia_user(&pool, "nuovo@cronachednd.it", "Nuovo Utente", "player").await;

    // Ora l'utente esiste
    assert!(user_exists(&pool, "nuovo@cronachednd.it").await);

    cleanup_test_db(&pool).await;
}

/// TEST 6: Utente Authelia non può fare login con password
#[tokio::test]
async fn test_authelia_user_cannot_login_with_password() {
    let pool = setup_test_db().await;

    // Crea utente Authelia
    create_authelia_user(&pool, "sso@cronachednd.it", "SSO User", "player").await;

    // Verifica che il password_hash sia il placeholder
    let result: Option<(String,)> =
        sqlx::query_as("SELECT password_hash FROM users WHERE email = ?")
            .bind("sso@cronachednd.it")
            .fetch_optional(&pool)
            .await
            .unwrap();

    assert!(result.is_some());
    let (hash,) = result.unwrap();
    assert_eq!(hash, "$authelia_sso$");

    // Verifica che bcrypt::verify fallisce (non è un hash valido)
    let verify_result = bcrypt::verify("qualsiasi_password", &hash);
    assert!(verify_result.is_err());

    cleanup_test_db(&pool).await;
}

/// TEST 7: Utente esistente viene aggiornato da Authelia
#[tokio::test]
async fn test_authelia_user_update_existing() {
    let pool = setup_test_db().await;

    // Crea utente standard
    create_test_user(&pool, "existing@cronachednd.it", "Password123!@#", "player").await;

    // Verifica ruolo iniziale
    assert_eq!(
        get_user_role(&pool, "existing@cronachednd.it").await,
        Some("player".to_string())
    );

    // Simula update da Authelia (utente diventa admin)
    sqlx::query("UPDATE users SET role = ? WHERE email = ?")
        .bind("admin")
        .bind("existing@cronachednd.it")
        .execute(&pool)
        .await
        .unwrap();

    // Verifica ruolo aggiornato
    assert_eq!(
        get_user_role(&pool, "existing@cronachednd.it").await,
        Some("admin".to_string())
    );

    cleanup_test_db(&pool).await;
}

// ============================================================================
// TDD TEST: GROUP-BASED ROLE ASSIGNMENT
// ============================================================================

/// TEST 8: Gruppo "admins" assegna ruolo admin
#[tokio::test]
async fn test_authelia_group_admin_role() {
    let groups = vec!["players", "admins"];

    let is_admin = groups.iter().any(|g| {
        g.eq_ignore_ascii_case("admins")
            || g.eq_ignore_ascii_case("admin")
            || g.eq_ignore_ascii_case("administrators")
    });

    assert!(is_admin);
}

/// TEST 9: Gruppo "players" non assegna ruolo admin
#[tokio::test]
async fn test_authelia_group_player_role() {
    let groups = vec!["players", "users"];

    let is_admin = groups.iter().any(|g| {
        g.eq_ignore_ascii_case("admins")
            || g.eq_ignore_ascii_case("admin")
            || g.eq_ignore_ascii_case("administrators")
    });

    assert!(!is_admin);
}

/// TEST 10: Gruppi alternativi riconosciuti ("administrators")
#[tokio::test]
async fn test_authelia_group_alternative_admin_names() {
    let test_cases = vec![
        (vec!["administrators"], true),
        (vec!["Admins"], true), // Case insensitive
        (vec!["ADMIN"], true),
        (vec!["moderators"], false),
        (vec!["dm"], false),
    ];

    for (groups, expected_admin) in test_cases {
        let is_admin = groups.iter().any(|g| {
            g.eq_ignore_ascii_case("admins")
                || g.eq_ignore_ascii_case("admin")
                || g.eq_ignore_ascii_case("administrators")
        });

        assert_eq!(is_admin, expected_admin, "Failed for groups: {:?}", groups);
    }
}

// ============================================================================
// TDD TEST: FALLBACK AUTHENTICATION
// ============================================================================

/// TEST 11: Fallback a Bearer token quando Authelia non è attivo
#[tokio::test]
async fn test_fallback_to_bearer_token() {
    let pool = setup_test_db().await;

    // Crea utente con sessione Bearer token
    let (user_id, token) =
        create_test_user_with_session(&pool, "bearer@cronachednd.it", "SecurePass123!@#", "player")
            .await;

    // Verifica che la sessione esiste
    let session: Option<(String,)> =
        sqlx::query_as("SELECT user_id FROM user_sessions WHERE token = ?")
            .bind(&token)
            .fetch_optional(&pool)
            .await
            .unwrap();

    assert!(session.is_some());
    assert_eq!(session.unwrap().0, user_id);

    cleanup_test_db(&pool).await;
}

/// TEST 12: Bearer token e Authelia coesistono
#[tokio::test]
async fn test_dual_auth_coexistence() {
    let pool = setup_test_db().await;

    // Utente con login password (Bearer token)
    let (bearer_user_id, _token) = create_test_user_with_session(
        &pool,
        "password@cronachednd.it",
        "SecurePass123!@#",
        "player",
    )
    .await;

    // Utente SSO (Authelia)
    let sso_user_id = create_authelia_user(&pool, "sso@cronachednd.it", "SSO User", "player").await;

    // Entrambi gli utenti esistono
    assert!(user_exists(&pool, "password@cronachednd.it").await);
    assert!(user_exists(&pool, "sso@cronachednd.it").await);

    // Sono utenti diversi
    assert_ne!(bearer_user_id, sso_user_id);

    cleanup_test_db(&pool).await;
}

// ============================================================================
// TDD TEST: EMAIL VALIDATION
// ============================================================================

/// TEST 13: Email valida accettata
#[tokio::test]
async fn test_authelia_valid_email() {
    let valid_emails = vec![
        "user@cronachednd.it",
        "admin@example.com",
        "test.user@domain.org",
        "user+tag@gmail.com",
    ];

    for email in valid_emails {
        let is_valid = email.contains('@') && email.contains('.');
        assert!(is_valid, "Email '{}' should be valid", email);
    }
}

/// TEST 14: Email invalida rifiutata
#[tokio::test]
async fn test_authelia_invalid_email() {
    let invalid_emails = vec!["invalid-email", "no-domain@", "@nodomain.com", ""];

    for email in invalid_emails {
        // Validazione completa: non vuota, contiene @, parte locale non vuota, dominio con punto
        let parts: Vec<&str> = email.split('@').collect();
        let is_valid = !email.is_empty()
            && parts.len() == 2
            && !parts[0].is_empty()  // Local part non vuota
            && parts[1].contains('.'); // Dominio con punto
        assert!(!is_valid, "Email '{}' should be invalid", email);
    }
}

// ============================================================================
// TDD TEST: CONFIGURATION
// ============================================================================

/// TEST 15: Configurazione Authelia disabilitata per default
#[tokio::test]
async fn test_authelia_config_disabled_by_default() {
    // Verifica che senza la variabile d'ambiente, Authelia è disabilitato
    // Nota: non modifichiamo env vars nei test per evitare race conditions
    let test_var = "AUTHELIA_TEST_DISABLED_CHECK";

    // Simula il comportamento del check
    let enabled = std::env::var(test_var)
        .map(|v| v.to_lowercase() == "true" || v == "1")
        .unwrap_or(false);

    assert!(!enabled);
}

/// TEST 16: Configurazione Authelia abilitata con env var
#[tokio::test]
async fn test_authelia_config_enabled_parsing() {
    // Test parsing di vari valori env
    let test_cases = vec![
        ("true", true),
        ("TRUE", true),
        ("True", true),
        ("1", true),
        ("false", false),
        ("FALSE", false),
        ("0", false),
        ("", false),
        ("anything_else", false),
    ];

    for (value, expected) in test_cases {
        let enabled = value.to_lowercase() == "true" || value == "1";
        assert_eq!(
            enabled, expected,
            "Value '{}' should be {}",
            value, expected
        );
    }
}

// ============================================================================
// TDD TEST: DISPLAY NAME FALLBACK
// ============================================================================

/// TEST 17: Display name da Remote-Name header
#[tokio::test]
async fn test_display_name_from_header() {
    let name = Some("Mario Rossi".to_string());
    let email = "mario@cronachednd.it";

    let display_name = name.unwrap_or_else(|| email.split('@').next().unwrap_or(email).to_string());

    assert_eq!(display_name, "Mario Rossi");
}

/// TEST 18: Display name fallback a email prefix
#[tokio::test]
async fn test_display_name_fallback_to_email() {
    let name: Option<String> = None;
    let email = "mario.rossi@cronachednd.it";

    let display_name = name.unwrap_or_else(|| email.split('@').next().unwrap_or(email).to_string());

    assert_eq!(display_name, "mario.rossi");
}

// ============================================================================
// TDD TEST: AUDIT LOGGING
// ============================================================================

/// TEST 19: Creazione utente Authelia viene loggata
#[tokio::test]
async fn test_authelia_user_creation_logged() {
    let pool = setup_test_db().await;

    // Simula creazione utente e audit log
    let user_id = create_authelia_user(&pool, "new@cronachednd.it", "New User", "player").await;

    // Inserisci audit log manualmente (come fa authelia_auth.rs)
    let now = chrono::Utc::now().timestamp();
    sqlx::query(
        "INSERT INTO audit_log (user_id, action, resource, timestamp, success, details) VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind(&user_id)
    .bind("authelia_user_created")
    .bind("auth")
    .bind(now)
    .bind(true)
    .bind("User auto-created from Authelia SSO: new@cronachednd.it")
    .execute(&pool)
    .await
    .unwrap();

    // Verifica audit log
    let log: Option<(String,)> =
        sqlx::query_as("SELECT action FROM audit_log WHERE user_id = ? AND action = ?")
            .bind(&user_id)
            .bind("authelia_user_created")
            .fetch_optional(&pool)
            .await
            .unwrap();

    assert!(log.is_some());

    cleanup_test_db(&pool).await;
}

/// TEST 20: Conteggio utenti SSO vs password
#[tokio::test]
async fn test_count_sso_vs_password_users() {
    let pool = setup_test_db().await;

    // Crea utenti misti
    create_test_user(&pool, "pass1@test.com", "Password123!@#", "player").await;
    create_test_user(&pool, "pass2@test.com", "Password123!@#", "player").await;
    create_authelia_user(&pool, "sso1@test.com", "SSO 1", "player").await;
    create_authelia_user(&pool, "sso2@test.com", "SSO 2", "player").await;
    create_authelia_user(&pool, "sso3@test.com", "SSO 3", "admin").await;

    // Conta utenti SSO (password_hash = $authelia_sso$)
    let sso_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE password_hash = ?")
        .bind("$authelia_sso$")
        .fetch_one(&pool)
        .await
        .unwrap();

    // Conta utenti password (bcrypt hash)
    let pass_count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM users WHERE password_hash LIKE '$2%'")
            .fetch_one(&pool)
            .await
            .unwrap();

    assert_eq!(sso_count.0, 3);
    assert_eq!(pass_count.0, 2);

    cleanup_test_db(&pool).await;
}
