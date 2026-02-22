use crate::core::models::{Admin, UserSession};
use crate::db::tables;
use crate::db::DbPool;
use anyhow::Result;
use redb::ReadableTable;

pub struct AdminRepo;

impl AdminRepo {
    pub fn find_by_email(pool: &DbPool, email: &str) -> Result<Option<Admin>> {
        let read_txn = pool.begin_read()?;
        let id_opt = {
            if let Ok(table) = read_txn.open_table(tables::ADMINS_BY_EMAIL_TABLE) {
                if let Some(guard) = table.get(email)? {
                    Some(guard.value().to_string())
                } else {
                    None
                }
            } else {
                None
            }
        };

        if let Some(id) = id_opt {
            let table = read_txn.open_table(tables::ADMINS_TABLE)?;
            if let Some(guard) = table.get(id.as_str())? {
                let admin: Admin = bincode::deserialize(guard.value())?;
                return Ok(Some(admin));
            }
        }
        Ok(None)
    }

    pub fn find_by_username(pool: &DbPool, username: &str) -> Result<Option<Admin>> {
        let read_txn = pool.begin_read()?;
        let id_opt = {
            if let Ok(table) = read_txn.open_table(tables::ADMINS_BY_USERNAME_TABLE) {
                if let Some(guard) = table.get(username)? {
                    Some(guard.value().to_string())
                } else {
                    None
                }
            } else {
                None
            }
        };

        if let Some(id) = id_opt {
            let table = read_txn.open_table(tables::ADMINS_TABLE)?;
            if let Some(guard) = table.get(id.as_str())? {
                let admin: Admin = bincode::deserialize(guard.value())?;
                return Ok(Some(admin));
            }
        }
        Ok(None)
    }

    pub fn create(pool: &DbPool, admin: &Admin) -> Result<()> {
        let write_txn = pool.begin_write()?;
        {
            let admin_bytes = bincode::serialize(admin)?;
            let mut admins_table = write_txn.open_table(tables::ADMINS_TABLE)?;
            admins_table.insert(admin.id.as_str(), admin_bytes.as_slice())?;

            if let Some(email) = &admin.email {
                let mut by_email = write_txn.open_table(tables::ADMINS_BY_EMAIL_TABLE)?;
                by_email.insert(email.as_str(), admin.id.as_str())?;
            }

            let mut by_username = write_txn.open_table(tables::ADMINS_BY_USERNAME_TABLE)?;
            by_username.insert(admin.username.as_str(), admin.id.as_str())?;
        }
        write_txn.commit()?;
        Ok(())
    }
}

pub struct SessionRepo;

impl SessionRepo {
    // Admin sessions
    pub fn create_admin_session(pool: &DbPool, token: &str, user_id: &str, expires_at: i64) -> Result<()> {
        let write_txn = pool.begin_write()?;
        {
            let mut table = write_txn.open_table(tables::ADMIN_SESSIONS_TABLE)?;
            let session_data = serde_json::json!({
                "user_id": user_id,
                "expires_at": expires_at
            }).to_string();
            table.insert(token, session_data.as_bytes())?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub fn get_admin_session(pool: &DbPool, token: &str) -> Result<Option<(String, i64)>> {
        let read_txn = pool.begin_read()?;
        if let Ok(table) = read_txn.open_table(tables::ADMIN_SESSIONS_TABLE) {
            if let Some(guard) = table.get(token)? {
                let json_str = std::str::from_utf8(guard.value())?;
                let v: serde_json::Value = serde_json::from_str(json_str)?;
                let uid = v["user_id"].as_str().unwrap_or("").to_string();
                let exp = v["expires_at"].as_i64().unwrap_or(0);
                return Ok(Some((uid, exp)));
            }
        }
        Ok(None)
    }

    pub fn delete_admin_session(pool: &DbPool, token: &str) -> Result<()> {
        let write_txn = pool.begin_write()?;
        if let Ok(mut table) = write_txn.open_table(tables::ADMIN_SESSIONS_TABLE) {
            table.remove(token)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    // User sessions
    pub fn delete_user_sessions(pool: &DbPool, user_id: &str) -> Result<()> {
        let write_txn = pool.begin_write()?;
        
        let mut session_tokens_to_delete = Vec::new();
        {
            if let Ok(_by_user_table) = write_txn.open_table(tables::USER_SESSIONS_BY_USER_ID_TABLE) {
                // By User ID index isn't properly a 1:N mapping in Redb unless we use composite keys
                // If it's 1:1, we can just look up. Wait, a user can have multiple sessions.
                // The table definition shows Key: user_id -> Value: token. Which implies 1 token per user.
                // Let's iterate all USER_SESSIONS_TABLE instead to be safe if multiple sessions exist.
            }

            if let Ok(mut sessions_table) = write_txn.open_table(tables::USER_SESSIONS_TABLE) {
                let range = sessions_table.iter()?;
                for result in range {
                    let (k, v) = result?;
                    let sess: UserSession = bincode::deserialize(v.value())?;
                    if sess.user_id == user_id {
                        session_tokens_to_delete.push(k.value().to_string());
                    }
                }
                for tok in &session_tokens_to_delete {
                    sessions_table.remove(tok.as_str())?;
                }
            }
        }
        write_txn.commit()?;
        Ok(())
    }

    pub fn get_user_session(pool: &DbPool, token: &str) -> Result<Option<crate::core::models::UserSession>> {
        let read_txn = pool.begin_read()?;
        if let Ok(table) = read_txn.open_table(tables::USER_SESSIONS_TABLE) {
            if let Some(guard) = table.get(token)? {
                let sess: crate::core::models::UserSession = bincode::deserialize(guard.value())?;
                return Ok(Some(sess));
            }
        }
        Ok(None)
    }

    pub fn delete_user_session_by_token(pool: &DbPool, token: &str) -> Result<()> {
        let write_txn = pool.begin_write()?;
        if let Ok(mut table) = write_txn.open_table(tables::USER_SESSIONS_TABLE) {
            table.remove(token)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub fn create_user_session(pool: &DbPool, session: &crate::core::models::UserSession) -> Result<()> {
        let write_txn = pool.begin_write()?;
        {
            let mut table = write_txn.open_table(tables::USER_SESSIONS_TABLE)?;
            let bytes = bincode::serialize(session)?;
            table.insert(session.token.as_str(), bytes.as_slice())?;
            
            let mut by_user_table = write_txn.open_table(tables::USER_SESSIONS_BY_USER_ID_TABLE)?;
            by_user_table.insert(session.user_id.as_str(), session.token.as_str())?;
        }
        write_txn.commit()?;
        Ok(())
    }
}
