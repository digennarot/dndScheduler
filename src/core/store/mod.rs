use anyhow::{anyhow, Result};
use redb::{Database, ReadableTable, TableDefinition};
use serde::Serialize;
use std::sync::Arc;

// Table: Key = "stream_id/version_formatted", Value = Event Payload (Bincode/Bytes)
// We use a helper function to generate keys.
const EVENTS_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("events");

pub struct RedbEventStore {
    db: Arc<Database>,
}

impl RedbEventStore {
    pub fn new(path: &str) -> Result<Self> {
        let db = Database::builder().create(path)?;
        let write_txn = db.begin_write()?;
        {
            // Ensure table exists
            write_txn.open_table(EVENTS_TABLE)?;
        }
        write_txn.commit()?;

        Ok(Self { db: Arc::new(db) })
    }

    fn make_key(stream_id: &str, version: u64) -> String {
        // Zero-padding ensures lexicographical order matches numeric order
        format!("{}/{:020}", stream_id, version)
    }

    pub async fn append(
        &self,
        stream_id: &str,
        event_data: &[u8],
        expected_version: u64,
    ) -> Result<u64> {
        // Redb is synchronous/blocking, so strict async usage might need `spawn_blocking`
        // if high throughput, but for now direct call in async function is acceptable
        // if latency is low. For "100% Reliability", correctness is key.
        // We clone Arc DB to move into blocking task if needed, but redb handles are cheap.

        // Note: For true async-friendliness we should wrap this in spawn_blocking,
        // but let's keep it simple first.

        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(EVENTS_TABLE)?;

            // 1. Concurrency Check: Find current version
            // We scan the stream to find "last" key.
            // Range scan: "stream_id/" to "stream_id0" (next char)
            let start_key = format!("{}/", stream_id);
            let end_key = format!("{}0", stream_id); // '0' is after '/' in ASCII? No.
                                                     // '/' is 47. '0' is 48. So "stream_id/" to "stream_id0" covers all "stream_id/..."

            let mut last_version = 0;
            // ...
            let range = table.range(start_key.as_str()..end_key.as_str())?;
            if let Some(Ok((k, _))) = range.last() {
                let value = k.value();
                // Parse version from key "stream_id/000...123"
                let parts: Vec<&str> = value.split('/').collect();
                if let Some(ver_str) = parts.last() {
                    last_version = ver_str.parse::<u64>().unwrap_or(0);
                }
            }

            if last_version != expected_version {
                return Err(anyhow!(
                    "Concurrency Error: Expected version {}, found {}",
                    expected_version,
                    last_version
                ));
            }

            // 2. Append
            let new_version = last_version + 1;
            let key = Self::make_key(stream_id, new_version);
            table.insert(key.as_str(), event_data)?;
        }
        write_txn.commit()?;

        Ok(expected_version + 1)
    }

    pub async fn append_unchecked(&self, stream_id: &str, event_data: &[u8]) -> Result<u64> {
        let write_txn = self.db.begin_write()?;
        let next_version;
        {
            let mut table = write_txn.open_table(EVENTS_TABLE)?;

            // 1. Find current version
            let start_key = format!("{}/", stream_id);
            let end_key = format!("{}0", stream_id);

            let mut last_version = 0;
            let range = table.range(start_key.as_str()..end_key.as_str())?;
            if let Some(Ok((k, _))) = range.last() {
                let value = k.value();
                let parts: Vec<&str> = value.split('/').collect();
                if let Some(ver_str) = parts.last() {
                    last_version = ver_str.parse::<u64>().unwrap_or(0);
                }
            }

            // 2. Append without check
            next_version = last_version + 1;
            let key = Self::make_key(stream_id, next_version);
            table.insert(key.as_str(), event_data)?;
        }
        write_txn.commit()?;

        Ok(next_version)
    }

    pub async fn read_stream(&self, stream_id: &str) -> Result<Vec<Vec<u8>>> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(EVENTS_TABLE)?;

        let start_key = format!("{}/", stream_id);
        let end_key = format!("{}0", stream_id); // Assumes stream_id doesn't contain older chars than '/'

        let mut events = Vec::new();
        let range = table.range(start_key.as_str()..end_key.as_str())?;

        for result in range {
            let (_k, v) = result?;
            events.push(v.value().to_vec());
        }

        Ok(events)
    }

    pub async fn read_all_events(&self) -> Result<Vec<(String, Vec<u8>)>> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(EVENTS_TABLE)?;

        let mut events = Vec::new();
        // Iterate over all entries in the events table
        let range = table.range::<&str>(..)?;

        for result in range {
            let (k, v) = result?;
            events.push((k.value().to_string(), v.value().to_vec()));
        }

        Ok(events)
    }

    pub async fn append_event<T: Serialize>(&self, stream_id: &str, event: T) -> Result<u64> {
        let event_data =
            bincode::serialize(&event).map_err(|e| anyhow!("Serialization error: {}", e))?;

        // 1. Get current version (Naive check-then-act, acceptable for low concurrency user streams)
        // Ideally this should be inside the transaction in append, but append takes &self and opens its own txn.
        // We can optimize strictness later.
        let events = self.read_stream(stream_id).await?;
        let current_version = events.len() as u64;

        self.append(stream_id, &event_data, current_version).await
    }

    pub async fn append_event_unchecked<T: Serialize>(
        &self,
        stream_id: &str,
        event: T,
    ) -> Result<u64> {
        let event_data =
            bincode::serialize(&event).map_err(|e| anyhow!("Serialization error: {}", e))?;
        self.append_unchecked(stream_id, &event_data).await
    }
}
