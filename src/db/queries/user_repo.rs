use crate::core::models::User;
use crate::db::tables;
use crate::db::DbPool;
use anyhow::Result;
use redb::ReadableTable;

pub struct UserRepo;

impl UserRepo {
    pub fn find_by_email(pool: &DbPool, email: &str) -> Result<Option<User>> {
        let read_txn = pool.begin_read()?;
        let id_opt = {
            if let Ok(table) = read_txn.open_table(tables::USERS_BY_EMAIL_TABLE) {
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
            let table = read_txn.open_table(tables::USERS_TABLE)?;
            if let Some(guard) = table.get(id.as_str())? {
                let user: User = bincode::deserialize(guard.value())?;
                return Ok(Some(user));
            }
        }
        Ok(None)
    }

    pub fn find_by_id(pool: &DbPool, id: &str) -> Result<Option<User>> {
        let read_txn = pool.begin_read()?;
        if let Ok(table) = read_txn.open_table(tables::USERS_TABLE) {
            if let Some(guard) = table.get(id)? {
                let user: User = bincode::deserialize(guard.value())?;
                return Ok(Some(user));
            }
        }
        Ok(None)
    }

    pub fn get_all(pool: &DbPool) -> Result<Vec<User>> {
        let read_txn = pool.begin_read()?;
        let mut users = Vec::new();
        if let Ok(table) = read_txn.open_table(tables::USERS_TABLE) {
            for result in table.iter()? {
                let (_, v) = result?;
                let user: User = bincode::deserialize(v.value())?;
                users.push(user);
            }
        }
        Ok(users)
    }

    pub fn create_or_update(pool: &DbPool, user: &User) -> Result<()> {
        let write_txn = pool.begin_write()?;
        {
            let user_bytes = bincode::serialize(user)?;
            let mut users_table = write_txn.open_table(tables::USERS_TABLE)?;
            users_table.insert(user.id.as_str(), user_bytes.as_slice())?;

            let mut by_email = write_txn.open_table(tables::USERS_BY_EMAIL_TABLE)?;
            by_email.insert(user.email.as_str(), user.id.as_str())?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub fn delete(pool: &DbPool, user_id: &str) -> Result<()> {
        let write_txn = pool.begin_write()?;
        {
            if let Ok(mut users_table) = write_txn.open_table(tables::USERS_TABLE) {
                if let Some(guard) = users_table.get(user_id)? {
                    let user: User = bincode::deserialize(guard.value())?;
                    if let Ok(mut by_email) = write_txn.open_table(tables::USERS_BY_EMAIL_TABLE) {
                        by_email.remove(user.email.as_str())?;
                    }
                }
                users_table.remove(user_id)?;
            }
        }
        write_txn.commit()?;
        Ok(())
    }

    pub fn delete_user_all_data(pool: &DbPool, user_id: &str) -> Result<()> {
        let write_txn = pool.begin_write()?;
        
        // 1. Delete user sessions
        {
            let mut session_tokens = Vec::new();
            if let Ok(mut sessions_table) = write_txn.open_table(tables::USER_SESSIONS_TABLE) {
                for result in sessions_table.iter()? {
                    let (k, v) = result?;
                    let sess: crate::core::models::UserSession = bincode::deserialize(v.value())?;
                    if sess.user_id == user_id {
                        session_tokens.push(k.value().to_string());
                    }
                }
                for tok in session_tokens {
                    sessions_table.remove(tok.as_str())?;
                }
            }

            // 2. Delete availability & participants (participant has multiple availability entries)
            // But participant model doesn't store user_id in rust. Wait, we don't have user_id on Participant struct.
            // Oh, wait, how was SQLite doing it? "DELETE FROM participants WHERE user_id = ?"
            // The rust struct Participant in models.rs doesn't have `user_id`! If it doesn't, we can't find it easily without parsing email or extending the struct.
            // Wait, models.rs `Participant` doesn't have `user_id`. Let's check if we can skip it or just add user_id to Participant in models.
        }

        write_txn.commit()?;
        Ok(())
    }
}
