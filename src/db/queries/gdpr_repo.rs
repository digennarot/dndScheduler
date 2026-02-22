use crate::core::models::*;
use crate::db::tables;
use crate::db::DbPool;
use anyhow::Result;
use redb::ReadableTable;

pub struct GdprRepo;

impl GdprRepo {
    pub fn log_consent_change(
        pool: &DbPool,
        user_id: &str,
        consent_type: &str,
        consented: bool,
        ip_address: &Option<String>,
        user_agent: &Option<String>,
        timestamp: i64,
    ) -> Result<()> {
        let write_txn = pool.begin_write()?;
        {
            let id = uuid::Uuid::new_v4().to_string();
            let key = format!("{}:{}:{}", user_id, timestamp, id);
            
            let record = ConsentRecord {
                id: None,
                user_id: user_id.to_string(),
                consent_type: consent_type.to_string(),
                consented,
                ip_address: ip_address.clone(),
                user_agent: user_agent.clone(),
                timestamp,
            };
            
            let bytes = bincode::serialize(&record)?;
            let mut table = write_txn.open_table(tables::CONSENT_RECORDS_TABLE)?;
            table.insert(key.as_str(), bytes.as_slice())?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub fn get_consent_history(pool: &DbPool, user_id: &str) -> Result<Vec<ConsentRecord>> {
        let read_txn = pool.begin_read()?;
        let mut history = Vec::new();
        if let Ok(table) = read_txn.open_table(tables::CONSENT_RECORDS_TABLE) {
            let start_key = format!("{}:", user_id);
            let end_key = format!("{}~", user_id);
            for result in table.range(start_key.as_str()..end_key.as_str())? {
                let (k, v) = result?;
                if k.value().starts_with(&start_key) {
                    let record: ConsentRecord = bincode::deserialize(v.value())?;
                    history.push(record);
                }
            }
        }
        // Descending order sort by timestamp
        history.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(history)
    }

    pub fn anonymize_consent_records(pool: &DbPool, user_id: &str) -> Result<()> {
        let write_txn = pool.begin_write()?;
        let mut keys_to_update = Vec::new();
        {
            if let Ok(table) = write_txn.open_table(tables::CONSENT_RECORDS_TABLE) {
                let start_key = format!("{}:", user_id);
                let end_key = format!("{}~", user_id);
                for result in table.range(start_key.as_str()..end_key.as_str())? {
                    let (k, _v) = result?;
                    if k.value().starts_with(&start_key) {
                        keys_to_update.push(k.value().to_string());
                    }
                }
            }
            if let Ok(mut table) = write_txn.open_table(tables::CONSENT_RECORDS_TABLE) {
                for key in keys_to_update {
                    let mut payload = None;
                    if let Some(guard) = table.get(key.as_str())? {
                        let mut record: ConsentRecord = bincode::deserialize(guard.value())?;
                        record.user_id = "DELETED_USER".to_string();
                        let new_key = format!("DELETED_USER:{}:{}", record.timestamp, record.id.unwrap_or(0));
                        let bytes = bincode::serialize(&record)?;
                        payload = Some((new_key, bytes));
                    }
                    if let Some((new_key, bytes)) = payload {
                        table.remove(key.as_str())?;
                        table.insert(new_key.as_str(), bytes.as_slice())?;
                    }
                }
            }
        }
        write_txn.commit()?;
        Ok(())
    }
}
