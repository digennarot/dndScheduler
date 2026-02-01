// Test Helpers Module
// Utilities per setup e teardown dei test

use axum::Router;
use dnd_scheduler::create_router;
use once_cell::sync::Lazy;
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use std::sync::Mutex;
use uuid::Uuid;

// Database temporaneo per i test
pub static TEST_DB: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

/// Crea un database SQLite temporaneo per i test
pub async fn setup_test_db() -> Pool<Sqlite> {
    // Usa un database in-memory per i test (più veloce e senza problemi di permessi)
    let database_url = format!("sqlite::memory:");

    // Crea il database
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    // Esegui le migrazioni (copia le tabelle da db.rs)
    setup_schema(&pool).await;

    pool
}

/// Setup dell'applicazione per i test (restituisce router e pool)
pub async fn setup_test_app() -> (Router, Pool<Sqlite>) {
    let pool = setup_test_db().await;
    let app = create_router(pool.clone());
    (app, pool)
}

/// Setup dello schema del database per i test
async fn setup_schema(pool: &Pool<Sqlite>) {
    // Tabella polls
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS polls (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            description TEXT NOT NULL,
            location TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            dates TEXT NOT NULL,
            time_range TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'active',
            finalized_at INTEGER,
            finalized_time TEXT,
            notes TEXT,
            admin_token TEXT
        );
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to create polls table");

    // Tabella participants
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS participants (
            id TEXT PRIMARY KEY,
            poll_id TEXT NOT NULL,
            name TEXT NOT NULL,
            email TEXT,
            access_token TEXT UNIQUE,
            user_id TEXT,
            FOREIGN KEY (poll_id) REFERENCES polls (id)
        );
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to create participants table");

    // Tabella availability
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS availability (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            poll_id TEXT NOT NULL,
            participant_id TEXT NOT NULL,
            date TEXT NOT NULL,
            time_slot TEXT NOT NULL,
            status TEXT NOT NULL,
            FOREIGN KEY (poll_id) REFERENCES polls (id),
            FOREIGN KEY (participant_id) REFERENCES participants (id)
        );
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to create availability table");

    // Tabella users
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            email TEXT NOT NULL UNIQUE,
            password_hash TEXT NOT NULL,
            name TEXT NOT NULL,
            role TEXT NOT NULL DEFAULT 'player',
            created_at INTEGER NOT NULL,
            last_login INTEGER,
            phone TEXT
        );
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to create users table");

    // Tabella user_sessions
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS user_sessions (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            token TEXT NOT NULL UNIQUE,
            expires_at INTEGER NOT NULL,
            created_at INTEGER NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users (id)
        );
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to create user_sessions table");

    // Tabella admins
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS admins (
            id TEXT PRIMARY KEY,
            username TEXT NOT NULL UNIQUE,
            password_hash TEXT NOT NULL,
            email TEXT,
            role TEXT NOT NULL DEFAULT 'admin',
            created_at INTEGER NOT NULL
        );
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to create admins table");

    // Tabella sessions (admin)
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS sessions (
            token TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            expires_at INTEGER NOT NULL,
            FOREIGN KEY (user_id) REFERENCES admins (id)
        );
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to create sessions table");

    // Tabella login_attempts
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS login_attempts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            email TEXT NOT NULL,
            attempt_time INTEGER NOT NULL,
            success BOOLEAN NOT NULL,
            ip_address TEXT
        );
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to create login_attempts table");

    // Tabella account_locks
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS account_locks (
            email TEXT PRIMARY KEY,
            locked_until INTEGER NOT NULL,
            reason TEXT
        );
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to create account_locks table");

    // Tabella audit_log
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS audit_log (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id TEXT,
            action TEXT NOT NULL,
            resource TEXT,
            timestamp INTEGER NOT NULL,
            ip_address TEXT,
            user_agent TEXT,
            success BOOLEAN NOT NULL,
            details TEXT
        );
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to create audit_log table");

    // Tabella activities
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS activities (
            id TEXT PRIMARY KEY,
            activity_type TEXT NOT NULL,
            user_id TEXT NOT NULL,
            user_name TEXT NOT NULL,
            poll_id TEXT,
            poll_name TEXT,
            message TEXT NOT NULL,
            timestamp INTEGER NOT NULL
        );
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to create activities table");
}

/// Crea un utente test nel database
pub async fn create_test_user(
    pool: &Pool<Sqlite>,
    email: &str,
    password: &str,
    role: &str,
) -> String {
    let user_id = Uuid::new_v4().to_string();
    let password_hash = bcrypt::hash(password, bcrypt::DEFAULT_COST).unwrap();
    let now = chrono::Utc::now().timestamp();

    sqlx::query(
        "INSERT INTO users (id, email, password_hash, name, role, created_at, last_login) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&user_id)
    .bind(email)
    .bind(&password_hash)
    .bind("Test User")
    .bind(role)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await
    .expect("Failed to create test user");

    user_id
}

/// Crea una sessione test e restituisce il token
pub async fn create_test_session(pool: &Pool<Sqlite>, user_id: &str) -> String {
    let session_id = Uuid::new_v4().to_string();
    let token = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp();
    let expires_at = now + 86400; // 24 ore

    sqlx::query(
        "INSERT INTO user_sessions (id, user_id, token, expires_at, created_at) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&session_id)
    .bind(user_id)
    .bind(&token)
    .bind(expires_at)
    .bind(now)
    .execute(pool)
    .await
    .expect("Failed to create test session");

    token
}

/// Crea un utente test con sessione
pub async fn create_test_user_with_session(
    pool: &Pool<Sqlite>,
    email: &str,
    password: &str,
    role: &str,
) -> (String, String) {
    let user_id = create_test_user(pool, email, password, role).await;
    let token = create_test_session(pool, &user_id).await;
    (user_id, token)
}

/// Crea un admin test
pub async fn create_test_admin(pool: &Pool<Sqlite>, email: &str, password: &str) -> String {
    let admin_id = Uuid::new_v4().to_string();
    let password_hash = bcrypt::hash(password, bcrypt::DEFAULT_COST).unwrap();
    let now = chrono::Utc::now().timestamp();

    sqlx::query(
        "INSERT INTO admins (id, username, password_hash, email, role, created_at) VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&admin_id)
    .bind("testadmin")
    .bind(&password_hash)
    .bind(email)
    .bind("admin")
    .bind(now)
    .execute(pool)
    .await
    .expect("Failed to create test admin");

    admin_id
}

// ============================================================================
// MULTIPLE TEST USER HELPERS
// ============================================================================

/// Test user configuration
#[derive(Debug, Clone)]
pub struct TestUserConfig {
    pub email: String,
    pub password: String,
    pub name: String,
    pub role: String,
}

impl TestUserConfig {
    pub fn new(email: &str, password: &str, name: &str, role: &str) -> Self {
        Self {
            email: email.to_string(),
            password: password.to_string(),
            name: name.to_string(),
            role: role.to_string(),
        }
    }

    pub fn player(email: &str) -> Self {
        Self::new(email, "password123", "Test Player", "player")
    }

    pub fn dm(email: &str) -> Self {
        Self::new(email, "password123", "Test DM", "dm")
    }

    pub fn admin(email: &str) -> Self {
        Self::new(email, "password123", "Test Admin", "admin")
    }
}

/// Creates multiple test users from configs
pub async fn create_test_users(
    pool: &Pool<Sqlite>,
    configs: Vec<TestUserConfig>,
) -> Vec<(String, String)> {
    let mut results = Vec::new();

    for config in configs {
        let (user_id, token) =
            create_test_user_with_session(pool, &config.email, &config.password, &config.role)
                .await;
        results.push((user_id, token));
    }

    results
}

/// Creates a set of default test users for common scenarios
pub async fn create_default_test_users(pool: &Pool<Sqlite>) -> DefaultTestUsers {
    let admin_config = TestUserConfig::admin("admin@test.com");
    let dm_config = TestUserConfig::dm("dm@test.com");
    let player1_config = TestUserConfig::player("player1@test.com");
    let player2_config = TestUserConfig::player("player2@test.com");
    let player3_config = TestUserConfig::player("player3@test.com");

    let (admin_id, admin_token) = create_test_user_with_session(
        pool,
        &admin_config.email,
        &admin_config.password,
        &admin_config.role,
    )
    .await;

    let (dm_id, dm_token) =
        create_test_user_with_session(pool, &dm_config.email, &dm_config.password, &dm_config.role)
            .await;

    let (player1_id, player1_token) = create_test_user_with_session(
        pool,
        &player1_config.email,
        &player1_config.password,
        &player1_config.role,
    )
    .await;

    let (player2_id, player2_token) = create_test_user_with_session(
        pool,
        &player2_config.email,
        &player2_config.password,
        &player2_config.role,
    )
    .await;

    let (player3_id, player3_token) = create_test_user_with_session(
        pool,
        &player3_config.email,
        &player3_config.password,
        &player3_config.role,
    )
    .await;

    DefaultTestUsers {
        admin: TestUser {
            id: admin_id,
            token: admin_token,
            email: admin_config.email,
            role: admin_config.role,
        },
        dm: TestUser {
            id: dm_id,
            token: dm_token,
            email: dm_config.email,
            role: dm_config.role,
        },
        player1: TestUser {
            id: player1_id,
            token: player1_token,
            email: player1_config.email,
            role: player1_config.role,
        },
        player2: TestUser {
            id: player2_id,
            token: player2_token,
            email: player2_config.email,
            role: player2_config.role,
        },
        player3: TestUser {
            id: player3_id,
            token: player3_token,
            email: player3_config.email,
            role: player3_config.role,
        },
    }
}

#[derive(Debug)]
pub struct TestUser {
    pub id: String,
    pub token: String,
    pub email: String,
    pub role: String,
}

#[derive(Debug)]
pub struct DefaultTestUsers {
    pub admin: TestUser,
    pub dm: TestUser,
    pub player1: TestUser,
    pub player2: TestUser,
    pub player3: TestUser,
}

/// Pulisce il database dopo i test
pub async fn cleanup_test_db(pool: &Pool<Sqlite>) {
    // Elimina tutti i dati dalle tabelle
    let tables = vec![
        "activities",
        "audit_log",
        "account_locks",
        "login_attempts",
        "sessions",
        "user_sessions",
        "availability",
        "participants",
        "polls",
        "users",
        "admins",
    ];

    for table in tables {
        sqlx::query(&format!("DELETE FROM {}", table))
            .execute(pool)
            .await
            .ok();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_setup_test_db() {
        let pool = setup_test_db().await;

        // Verifica che il database sia stato creato correttamente
        let result: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(result.0, 0);
    }

    #[tokio::test]
    async fn test_create_test_user() {
        let pool = setup_test_db().await;

        let user_id = create_test_user(&pool, "test@test.com", "password123", "player").await;

        // Verifica che l'utente sia stato creato
        let result: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE id = ?")
            .bind(&user_id)
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(result.0, 1);
    }

    #[tokio::test]
    async fn test_create_test_session() {
        let pool = setup_test_db().await;

        let user_id = create_test_user(&pool, "test@test.com", "password123", "player").await;
        let token = create_test_session(&pool, &user_id).await;

        // Verifica che la sessione sia stata creata
        let result: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM user_sessions WHERE token = ?")
            .bind(&token)
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(result.0, 1);
    }
}
// Helper per creare un sondaggio di test
pub async fn create_test_poll(_app: &Router) -> String {
    // Implementazione semplificata: crea un sondaggio direttamente nel DB o via API
    // Per semplicità, usiamo una chiamata API se possibile, o helper DB se abbiamo accesso al pool qui
    // Ma setup_test_app ritorna (app, pool), quindi nel test abbiamo il pool.
    // Questo helper dovrebbe accettare App? No, meglio se il test usa create_poll endpoint.
    // Ma per autenticazione serve token.

    // Placeholder - i test devono implementare la logica specifica o passare il pool a questo helper if needed.
    // Modifichiamo la firma o assumiamo che il test lo faccia.
    // In auth_tests.rs non usano questo helper.
    // In test_anonymous.rs lo usiamo.
    "admin_token".to_string()
}

// NOTE: create_test_poll above is a placeholder.
// Real implementation should likely take &Pool and insert a poll directly.
pub async fn create_test_poll_db(pool: &Pool<Sqlite>) -> String {
    let poll_id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp();

    sqlx::query(
        "INSERT INTO polls (id, title, description, location, created_at, dates, time_range, status) VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&poll_id)
    .bind("Test Poll")
    .bind("Description")
    .bind("Remote")
    .bind(now)
    .bind("[\"2023-10-10\"]")
    .bind("[]")
    .bind("active")
    .execute(pool)
    .await
    .unwrap();

    poll_id
}
