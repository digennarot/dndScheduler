use redb::Database;
use std::error::Error;
use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;

pub mod queries; // Expose queries module
pub mod tables;  // Expose tables module

pub type DbPool = Arc<Database>;

pub fn setup_redb_schema(db: &Database) -> Result<(), Box<dyn Error>> {
    let write_txn = db.begin_write()?;
    {
        // 1. Core scheduling tables
        write_txn.open_table(tables::POLLS_TABLE)?;
        write_txn.open_table(tables::POLL_INSTANCES_TABLE)?;
        write_txn.open_table(tables::PARTICIPANTS_TABLE)?;
        write_txn.open_table(tables::AVAILABILITY_TABLE)?;

        // 2. User & Admin auth tables
        write_txn.open_table(tables::USERS_TABLE)?;
        write_txn.open_table(tables::USERS_BY_EMAIL_TABLE)?;
        write_txn.open_table(tables::USER_SESSIONS_TABLE)?;
        write_txn.open_table(tables::USER_SESSIONS_BY_USER_ID_TABLE)?;

        write_txn.open_table(tables::ADMINS_TABLE)?;
        write_txn.open_table(tables::ADMINS_BY_EMAIL_TABLE)?;
        write_txn.open_table(tables::ADMINS_BY_USERNAME_TABLE)?;
        write_txn.open_table(tables::ADMIN_SESSIONS_TABLE)?;

        // 3. Application Data
        write_txn.open_table(tables::ACTIVITIES_TABLE)?;

        // 4. OWASP & GDPR Tables
        write_txn.open_table(tables::LOGIN_ATTEMPTS_TABLE)?;
        write_txn.open_table(tables::ACCOUNT_LOCKS_TABLE)?;
        write_txn.open_table(tables::AUDIT_LOG_TABLE)?;
        write_txn.open_table(tables::CONSENT_RECORDS_TABLE)?;
        write_txn.open_table(tables::DATA_EXPORT_REQUESTS_TABLE)?;
    }
    write_txn.commit()?;
    Ok(())
}

pub async fn init_db() -> Result<DbPool, Box<dyn Error>> {
    let database_path = std::env::var("DATABASE_URL")
        .map(|s| s.replace("sqlite:", ""))
        .unwrap_or_else(|_| "data/read_model.redb".to_string());

    let db = Database::create(&database_path)?;
    setup_redb_schema(&db)?;

    // The Redb `Database::create` and subsequent `open_table` calls already 
    // handle the creation and initialization of tables if they don't exist.
    // So we don't need any raw SQL queries here anymore.

    // Also handle default admin creation in Redb
    let default_admin_email = match std::env::var("DEFAULT_ADMIN_EMAIL") {
        Ok(v) => v,
        Err(_) => "admin@example.com".to_string(),
    };

    let default_admin_password = match std::env::var("DEFAULT_ADMIN_PASSWORD") {
        Ok(v) => v,
        Err(_) => {
            use rand::Rng;
            let mut rng = rand::thread_rng();
            let mut pwd = String::new();
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

    let mut admin_exists = false;
    let read_txn = db.begin_read()?;
    if let Ok(table) = read_txn.open_table(tables::ADMINS_BY_EMAIL_TABLE) {
        if table.get(default_admin_email.as_str())?.is_some() {
            admin_exists = true;
        }
    }
    
    if !admin_exists && read_txn.open_table(tables::ADMINS_BY_USERNAME_TABLE).is_ok() {
        let table = read_txn.open_table(tables::ADMINS_BY_USERNAME_TABLE)?;
        if table.get("admin")?.is_some() {
            admin_exists = true;
        }
    }

    if !admin_exists {
        let password_hash = bcrypt::hash(&default_admin_password, bcrypt::DEFAULT_COST)
            .map_err(|e| format!("Failed to hash default admin password: {}", e))?;
        let admin_id = Uuid::new_v4().to_string();
        let now = Utc::now().timestamp();

        let admin = crate::core::models::Admin {
            id: admin_id.clone(),
            username: "admin".to_string(),
            password_hash,
            email: Some(default_admin_email.clone()),
            role: "superadmin".to_string(),
            created_at: now,
        };

        let admin_bytes = bincode::serialize(&admin)
            .map_err(|e| format!("Failed to serialize default admin: {}", e))?;

        let write_txn = db.begin_write()?;
        {
            let mut admins_table = write_txn.open_table(tables::ADMINS_TABLE)?;
            admins_table.insert(admin_id.as_str(), admin_bytes.as_slice())?;

            let mut by_email_table = write_txn.open_table(tables::ADMINS_BY_EMAIL_TABLE)?;
            by_email_table.insert(default_admin_email.as_str(), admin_id.as_str())?;

            let mut by_username_table = write_txn.open_table(tables::ADMINS_BY_USERNAME_TABLE)?;
            by_username_table.insert("admin", admin_id.as_str())?;
        }
        write_txn.commit()?;

        println!(
            "Default admin created: {} / {}",
            default_admin_email, default_admin_password
        );
    }

    Ok(Arc::new(db))
}
