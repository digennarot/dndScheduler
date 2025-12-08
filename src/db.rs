use chrono::Utc;
use sqlx::{migrate::MigrateDatabase, sqlite::SqlitePoolOptions, Pool, Sqlite};
use std::error::Error;
use uuid::Uuid;

pub type DbPool = Pool<Sqlite>;

pub async fn init_db() -> Result<DbPool, Box<dyn Error>> {
    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:dnd_scheduler.db".to_string());

    // Create database file if it doesn't exist
    if !sqlx::Sqlite::database_exists(&database_url)
        .await
        .unwrap_or(false)
    {
        sqlx::Sqlite::create_database(&database_url).await?;
    }

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS polls (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            description TEXT NOT NULL,
            location TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            dates TEXT NOT NULL,
            time_range TEXT NOT NULL
        );
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS participants (
            id TEXT PRIMARY KEY,
            poll_id TEXT NOT NULL,
            name TEXT NOT NULL,
            email TEXT,
            access_token TEXT UNIQUE,
            FOREIGN KEY (poll_id) REFERENCES polls (id)
        );
        "#,
    )
    .execute(&pool)
    .await?;

    // Migration: Add access_token column if it doesn't exist
    sqlx::query(
        r#"
        ALTER TABLE participants ADD COLUMN access_token TEXT;
        "#,
    )
    .execute(&pool)
    .await
    .ok(); // Ignore error if column already exists

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
    .execute(&pool)
    .await?;

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
    .execute(&pool)
    .await?;

    // Check if default admin exists, if not create one
    // Get admin credentials from environment or use defaults
    let default_admin_email =
        std::env::var("DEFAULT_ADMIN_EMAIL").unwrap_or_else(|_| "admin@example.com".to_string());
    let default_admin_password =
        std::env::var("DEFAULT_ADMIN_PASSWORD").unwrap_or_else(|_| "password123".to_string());

    // Check if admin exists by either email OR username to avoid UNIQUE constraint violations
    let default_admin_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM admins WHERE email = ? OR username = 'admin')",
    )
    .bind(&default_admin_email)
    .fetch_one(&pool)
    .await?;

    if !default_admin_exists {
        let password_hash = bcrypt::hash(&default_admin_password, bcrypt::DEFAULT_COST).unwrap();
        let admin_id = Uuid::new_v4().to_string();
        let now = Utc::now().timestamp();

        // Use INSERT OR IGNORE to gracefully handle race conditions
        sqlx::query(
            "INSERT OR IGNORE INTO admins (id, username, password_hash, email, role, created_at) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(admin_id)
        .bind("admin")
        .bind(password_hash)
        .bind(&default_admin_email)
        .bind("superadmin")
        .bind(now)
        .execute(&pool)
        .await?;

        println!(
            "Default admin created: {} / {}",
            default_admin_email, default_admin_password
        );
    }

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
    .execute(&pool)
    .await?;

    // Create users table for participant authentication
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            email TEXT NOT NULL UNIQUE,
            password_hash TEXT NOT NULL,
            name TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            last_login INTEGER
        );
        "#,
    )
    .execute(&pool)
    .await?;

    // Migration: Add role column to users table if it doesn't exist
    sqlx::query(
        r#"
        ALTER TABLE users ADD COLUMN role TEXT NOT NULL DEFAULT 'player';
        "#,
    )
    .execute(&pool)
    .await
    .ok(); // Ignore error if column already exists

    // Create user_sessions table
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
    .execute(&pool)
    .await?;

    // Migration: Add user_id to participants table
    sqlx::query(
        r#"
        ALTER TABLE participants ADD COLUMN user_id TEXT;
        "#,
    )
    .execute(&pool)
    .await
    .ok(); // Ignore error if column already exists

    // Create activities table for activity feed
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
    .execute(&pool)
    .await?;

    // Create index for faster queries
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_activities_timestamp ON activities(timestamp DESC);",
    )
    .execute(&pool)
    .await?;

    // OWASP: Account Lockout Tables
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
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS account_locks (
            email TEXT PRIMARY KEY,
            locked_until INTEGER NOT NULL,
            reason TEXT
        );
        "#,
    )
    .execute(&pool)
    .await?;

    // OWASP: Audit Logging Table
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
    .execute(&pool)
    .await?;

    // GDPR: Consent Records Table for audit trail
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS consent_records (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id TEXT NOT NULL,
            consent_type TEXT NOT NULL,
            consented BOOLEAN NOT NULL,
            ip_address TEXT,
            user_agent TEXT,
            timestamp INTEGER NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users (id)
        );
        "#,
    )
    .execute(&pool)
    .await?;

    // GDPR: Data Export Requests Table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS data_export_requests (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            status TEXT NOT NULL,
            requested_at INTEGER NOT NULL,
            completed_at INTEGER,
            FOREIGN KEY (user_id) REFERENCES users (id)
        );
        "#,
    )
    .execute(&pool)
    .await?;

    // GDPR: Add consent columns to users table
    sqlx::query(
        r#"
        ALTER TABLE users ADD COLUMN consent_marketing BOOLEAN NOT NULL DEFAULT 0;
        "#,
    )
    .execute(&pool)
    .await
    .ok(); // Ignore error if column already exists

    sqlx::query(
        r#"
        ALTER TABLE users ADD COLUMN consent_analytics BOOLEAN NOT NULL DEFAULT 0;
        "#,
    )
    .execute(&pool)
    .await
    .ok(); // Ignore error if column already exists

    sqlx::query(
        r#"
        ALTER TABLE users ADD COLUMN privacy_policy_accepted_at INTEGER;
        "#,
    )
    .execute(&pool)
    .await
    .ok(); // Ignore error if column already exists

    Ok(pool)
}
