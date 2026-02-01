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

async fn cleanup_old_polls(pool: &DbPool) -> Result<u64, sqlx::Error> {
    let cutoff_timestamp = (Utc::now() - Duration::days(ARCHIVE_AFTER_DAYS)).timestamp();
    let old_created_timestamp = (Utc::now() - Duration::days(60)).timestamp();

    // Strategy: Archive polls that have a 'finalized_at' older than 30 days
    // OR created_at is older than 60 days (if never finalized)
    let result = sqlx::query(
        "UPDATE polls SET status = 'archived' WHERE status != 'archived' AND (finalized_at < ? OR (created_at < ? AND status = 'active'))"
    )
    .bind(cutoff_timestamp)
    .bind(old_created_timestamp)
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn setup_test_db() -> DbPool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("Failed to connect to memory db");

        sqlx::query("CREATE TABLE polls (id TEXT PRIMARY KEY, title TEXT NOT NULL, description TEXT NOT NULL, location TEXT NOT NULL, created_at INTEGER NOT NULL, dates TEXT NOT NULL, time_range TEXT NOT NULL, status TEXT NOT NULL DEFAULT 'active', finalized_at INTEGER, finalized_time TEXT, notes TEXT, admin_token TEXT)")
            .execute(&pool).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn test_cleanup_logic() {
        let pool = setup_test_db().await;

        // 1. Create a recent active poll (Should NOT be archived)
        let recent_ts = Utc::now().timestamp();
        sqlx::query("INSERT INTO polls (id, title, description, location, created_at, dates, time_range, status) VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
            .bind("recent")
            .bind("Recent Poll")
            .bind("Desc")
            .bind("Loc")
            .bind(recent_ts)
            .bind("[]")
            .bind("[]")
            .bind("active")
            .execute(&pool).await.unwrap();

        // 2. Create an old finalized poll (Should BE archived)
        let old_finalized_ts = (Utc::now() - Duration::days(31)).timestamp();
        sqlx::query("INSERT INTO polls (id, title, description, location, created_at, dates, time_range, status, finalized_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)")
            .bind("old_finalized")
            .bind("Old Poll")
            .bind("Desc")
            .bind("Loc")
            .bind(recent_ts) // created recently
            .bind("[]")
            .bind("[]")
            .bind("finalized")
            .bind(old_finalized_ts) // finalized long ago
            .execute(&pool).await.unwrap();

        // 3. Create a very old active poll (abandoned) (Should BE archived)
        let old_created_ts = (Utc::now() - Duration::days(61)).timestamp();
        sqlx::query("INSERT INTO polls (id, title, description, location, created_at, dates, time_range, status) VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
            .bind("old_active")
            .bind("Abandoned Poll")
            .bind("Desc")
            .bind("Loc")
            .bind(old_created_ts)
            .bind("[]")
            .bind("[]")
            .bind("active")
            .execute(&pool).await.unwrap();

        // Run cleanup
        let count = cleanup_old_polls(&pool).await.unwrap();
        assert_eq!(count, 2, "Should archive exactly 2 polls");

        // Verify statuses
        let recent_status: String =
            sqlx::query_scalar("SELECT status FROM polls WHERE id = 'recent'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(recent_status, "active");

        let old_status: String =
            sqlx::query_scalar("SELECT status FROM polls WHERE id = 'old_finalized'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(old_status, "archived");

        let abandoned_status: String =
            sqlx::query_scalar("SELECT status FROM polls WHERE id = 'old_active'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(abandoned_status, "archived");
    }
}
