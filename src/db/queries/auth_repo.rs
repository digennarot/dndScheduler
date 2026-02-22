use crate::db::tables;
use crate::db::DbPool;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct AccountLock {
    pub locked_until: i64,
    pub reason: String,
}

#[derive(Serialize, Deserialize)]
pub struct LoginAttempt {
    pub success: bool,
    // Add other fields if needed, but for now we only need success/time
}

pub struct AuthRepo;

impl AuthRepo {
    pub fn get_account_lock(pool: &DbPool, email: &str) -> Result<Option<i64>> {
        let read_txn = pool.begin_read()?;
        if let Ok(table) = read_txn.open_table(tables::ACCOUNT_LOCKS_TABLE) {
            if let Some(guard) = table.get(email)? {
                let lock: AccountLock = bincode::deserialize(guard.value())?;
                return Ok(Some(lock.locked_until));
            }
        }
        Ok(None)
    }

    pub fn delete_account_lock(pool: &DbPool, email: &str) -> Result<()> {
        let write_txn = pool.begin_write()?;
        if let Ok(mut table) = write_txn.open_table(tables::ACCOUNT_LOCKS_TABLE) {
            table.remove(email)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub fn record_login_attempt(pool: &DbPool, email: &str, attempt_time: i64, success: bool, _ip_address: &str) -> Result<()> {
        let write_txn = pool.begin_write()?;
        {
            let id = uuid::Uuid::new_v4().to_string();
            let key = format!("{}:{}:{}", email, attempt_time, id);
            let attempt = LoginAttempt { success };
            let bytes = bincode::serialize(&attempt)?;
            let mut table = write_txn.open_table(tables::LOGIN_ATTEMPTS_TABLE)?;
            table.insert(key.as_str(), bytes.as_slice())?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub fn count_failed_attempts(pool: &DbPool, email: &str, since_time: i64) -> Result<i64> {
        let read_txn = pool.begin_read()?;
        let mut count = 0;
        if let Ok(table) = read_txn.open_table(tables::LOGIN_ATTEMPTS_TABLE) {
            // Range scan from {email}:{since_time}: to {email}:~
            let start_key = format!("{}:{}:", email, since_time);
            let end_key = format!("{}~", email); // basically all subsequent times for this email
            
            for result in table.range(start_key.as_str()..end_key.as_str())? {
                let (k, v) = result?;
                // Ensure it matches the email exactly
                if k.value().starts_with(&format!("{}:", email)) {
                    let attempt: LoginAttempt = bincode::deserialize(v.value())?;
                    if !attempt.success {
                        count += 1;
                    }
                }
            }
        }
        Ok(count)
    }

    pub fn lock_account(pool: &DbPool, email: &str, locked_until: i64, reason: &str) -> Result<()> {
        let write_txn = pool.begin_write()?;
        {
            let lock = AccountLock {
                locked_until,
                reason: reason.to_string(),
            };
            let bytes = bincode::serialize(&lock)?;
            let mut table = write_txn.open_table(tables::ACCOUNT_LOCKS_TABLE)?;
            table.insert(email, bytes.as_slice())?;
        }
        write_txn.commit()?;
        Ok(())
    }
}
