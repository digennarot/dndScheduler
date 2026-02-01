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
            FOREIGN KEY (poll_id) REFERENCES polls (id),
            UNIQUE(poll_id, email)
        );
        "#,
    )
    .execute(&pool)
    .await?;

    // Migration: Add access_token column if it doesn't exist
    if let Err(e) = sqlx::query(
        r#"
        ALTER TABLE participants ADD COLUMN access_token TEXT;
        "#,
    )
    .execute(&pool)
    .await
    {
        if !e.to_string().contains("duplicate column") {
            tracing::warn!("Migration failed (add access_token): {}", e);
        }
    }

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
    let default_admin_email = match std::env::var("DEFAULT_ADMIN_EMAIL") {
        Ok(v) => v,
        Err(_) => "admin@example.com".to_string(),
    };
    let default_admin_password = match std::env::var("DEFAULT_ADMIN_PASSWORD") {
        Ok(v) => v,
        Err(_) => {
            // Generate a random strong password if not set
            use rand::Rng;
            let mut rng = rand::thread_rng();
            let mut pwd = String::new();
            // Ensure policy compliance: Upper, Lower, Digit, Special
            pwd.push(rng.gen_range(b'A'..=b'Z') as char);
            pwd.push(rng.gen_range(b'a'..=b'z') as char);
            pwd.push(rng.gen_range(b'0'..=b'9') as char);
            let specials = "!@#$%^&*";
            pwd.push(
                specials
                    .chars()
                    .nth(rng.gen_range(0..specials.len()))
                    .unwrap_or('!'),
            );

            // Fill rest to 16 chars
            for _ in 0..12 {
                if rng.gen_bool(0.5) {
                    pwd.push(rng.gen_range(b'a'..=b'z') as char);
                } else {
                    pwd.push(rng.gen_range(b'0'..=b'9') as char);
                }
            }

            tracing::warn!(
                "DEFAULT_ADMIN_PASSWORD not set. Generated temporary password: '{}'",
                pwd
            );
            pwd
        }
    };

    // Check if admin exists by either email OR username to avoid UNIQUE constraint violations
    let default_admin_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM admins WHERE email = ? OR username = 'admin')",
    )
    .bind(&default_admin_email)
    .fetch_one(&pool)
    .await?;

    if !default_admin_exists {
        let password_hash = bcrypt::hash(&default_admin_password, bcrypt::DEFAULT_COST)
            .map_err(|e| format!("Failed to hash default admin password: {}", e))?;
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
    // Migration: Add role column to users table if it doesn't exist
    if let Err(e) = sqlx::query(
        r#"
        ALTER TABLE users ADD COLUMN role TEXT NOT NULL DEFAULT 'player';
        "#,
    )
    .execute(&pool)
    .await
    {
        if !e.to_string().contains("duplicate column") {
            tracing::warn!("Migration failed (add role): {}", e);
        }
    }

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
    // Migration: Add user_id to participants table
    if let Err(e) = sqlx::query(
        r#"
        ALTER TABLE participants ADD COLUMN user_id TEXT;
        "#,
    )
    .execute(&pool)
    .await
    {
        if !e.to_string().contains("duplicate column") {
            tracing::warn!("Migration failed (add user_id): {}", e);
        }
    }

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
    // GDPR: Add consent columns to users table
    if let Err(e) = sqlx::query(
        r#"
        ALTER TABLE users ADD COLUMN consent_marketing BOOLEAN NOT NULL DEFAULT 0;
        "#,
    )
    .execute(&pool)
    .await
    {
        if !e.to_string().contains("duplicate column") {
            tracing::warn!("Migration failed (add consent_marketing): {}", e);
        }
    }

    if let Err(e) = sqlx::query(
        r#"
        ALTER TABLE users ADD COLUMN consent_analytics BOOLEAN NOT NULL DEFAULT 0;
        "#,
    )
    .execute(&pool)
    .await
    {
        if !e.to_string().contains("duplicate column") {
            tracing::warn!("Migration failed (add consent_analytics): {}", e);
        }
    }

    if let Err(e) = sqlx::query(
        r#"
        ALTER TABLE users ADD COLUMN privacy_policy_accepted_at INTEGER;
        "#,
    )
    .execute(&pool)
    .await
    {
        if !e.to_string().contains("duplicate column") {
            tracing::warn!("Migration failed (add privacy_policy_accepted_at): {}", e);
        }
    }

    if let Err(e) = sqlx::query(
        r#"
        ALTER TABLE users ADD COLUMN phone TEXT;
        "#,
    )
    .execute(&pool)
    .await
    {
        if !e.to_string().contains("duplicate column") {
            tracing::warn!("Migration failed (add phone): {}", e);
        }
    }

    // Migration: Add status and finalization columns to polls
    if let Err(e) =
        sqlx::query("ALTER TABLE polls ADD COLUMN status TEXT NOT NULL DEFAULT 'active'")
            .execute(&pool)
            .await
    {
        if !e.to_string().contains("duplicate column") {
            tracing::warn!("Migration failed (add status): {}", e);
        }
    }

    if let Err(e) = sqlx::query("ALTER TABLE polls ADD COLUMN finalized_at INTEGER")
        .execute(&pool)
        .await
    {
        if !e.to_string().contains("duplicate column") {
            tracing::warn!("Migration failed (add finalized_at): {}", e);
        }
    }

    if let Err(e) = sqlx::query("ALTER TABLE polls ADD COLUMN finalized_time TEXT")
        .execute(&pool)
        .await
    {
        if !e.to_string().contains("duplicate column") {
            tracing::warn!("Migration failed (add finalized_time): {}", e);
        }
    }

    if let Err(e) = sqlx::query("ALTER TABLE polls ADD COLUMN notes TEXT")
        .execute(&pool)
        .await
    {
        if !e.to_string().contains("duplicate column") {
            tracing::warn!("Migration failed (add notes): {}", e);
        }
    }

    if let Err(e) = sqlx::query("ALTER TABLE polls ADD COLUMN admin_token TEXT")
        .execute(&pool)
        .await
    {
        if !e.to_string().contains("duplicate column") {
            tracing::warn!("Migration failed (add admin_token): {}", e);
        }
    }

    if let Err(e) = sqlx::query("ALTER TABLE polls ADD COLUMN organizer_id TEXT")
        .execute(&pool)
        .await
    {
        if !e.to_string().contains("duplicate column") {
            tracing::warn!("Migration failed (add organizer_id): {}", e);
        }
    }

    Ok(pool)
}
