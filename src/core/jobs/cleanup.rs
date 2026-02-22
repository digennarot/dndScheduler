use crate::db::DbPool;
use chrono::{Duration, Utc};
use tokio::time;

const CLEANUP_INTERVAL: u64 = 24 * 60 * 60; // 24 hours
const ARCHIVE_AFTER_DAYS: i64 = 30;

pub async fn run_cron(pool: DbPool) {
    let mut interval = time::interval(time::Duration::from_secs(CLEANUP_INTERVAL));

    loop {
        interval.tick().await;

        tracing::info!("Running daily cleanup job...");

        match cleanup_old_polls(&pool).await {
            Ok(count) => {
                if count > 0 {
                    tracing::info!("Archived/Deleted {} old polls", count);
                }
            }
            Err(e) => tracing::error!("Cleanup job failed: {}", e),
        }
    }
}

async fn cleanup_old_polls(pool: &DbPool) -> Result<u64, anyhow::Error> {
    let cutoff_timestamp = (Utc::now() - Duration::days(ARCHIVE_AFTER_DAYS)).timestamp();
    let old_created_timestamp = (Utc::now() - Duration::days(60)).timestamp();

    let write_txn = pool.begin_write()?;
    let mut count = 0;
    {
        use crate::core::models::Poll;
        use crate::db::tables;
        use redb::ReadableTable;

        let mut polls_to_update = Vec::new();
        if let Ok(table) = write_txn.open_table(tables::POLLS_TABLE) {
            for result in table.iter()? {
                let (k, v) = result?;
                let mut poll: Poll = bincode::deserialize(v.value())?;
                if poll.status != "archived" {
                    let mut should_archive = false;
                    if let Some(finalized_at) = poll.finalized_at {
                        if finalized_at < cutoff_timestamp {
                            should_archive = true;
                        }
                    } else if poll.created_at < old_created_timestamp && poll.status == "active" {
                        should_archive = true;
                    }

                    if should_archive {
                        poll.status = "archived".to_string();
                        polls_to_update.push((k.value().to_string(), poll));
                    }
                }
            }
        }

        if let Ok(mut table) = write_txn.open_table(tables::POLLS_TABLE) {
            for (k, poll) in polls_to_update {
                let bytes = bincode::serialize(&poll)?;
                table.insert(k.as_str(), bytes.as_slice())?;
                count += 1;
            }
        }
    }
    write_txn.commit()?;

    Ok(count)
}
