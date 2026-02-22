use crate::core::models::Activity;
use crate::db::tables;
use crate::db::DbPool;
use anyhow::Result;
use redb::ReadableTable;

pub struct ActivityRepo;

impl ActivityRepo {
    pub fn get_recent_activity(
        pool: &DbPool,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Activity>> {
        let read_txn = pool.begin_read()?;
        let mut activities = Vec::new();

        if let Ok(table) = read_txn.open_table(tables::ACTIVITIES_TABLE) {
            for result in table.iter()? {
                let (_, v) = result?;
                let act: Activity = bincode::deserialize(v.value())?;
                activities.push(act);
            }
        }

        // Sort descending by timestamp
        activities.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        // Apply pagination
        let start = offset as usize;
        let end = (offset + limit) as usize;
        let paginated = activities.into_iter().skip(start).take(end - start).collect();

        Ok(paginated)
    }

    pub fn get_user_activities(
        pool: &DbPool,
        user_id: &str,
    ) -> Result<Vec<Activity>> {
        let read_txn = pool.begin_read()?;
        let mut activities = Vec::new();

        if let Ok(table) = read_txn.open_table(tables::ACTIVITIES_TABLE) {
            for result in table.iter()? {
                let (_, v) = result?;
                let act: Activity = bincode::deserialize(v.value())?;
                if act.user_id == user_id {
                    activities.push(act);
                }
            }
        }

        activities.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(activities)
    }

    pub fn log_activity(
        pool: &DbPool,
        activity: &Activity,
    ) -> Result<()> {
        let write_txn = pool.begin_write()?;
        {
            let mut table = write_txn.open_table(tables::ACTIVITIES_TABLE)?;
            let bytes = bincode::serialize(activity)?;
            let key = format!("{}:{}", activity.timestamp, activity.id); // Lexicographical sort friendly
            table.insert(key.as_str(), bytes.as_slice())?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub fn delete_user_activities(
        pool: &DbPool,
        user_id: &str,
    ) -> Result<()> {
        let write_txn = pool.begin_write()?;
        let mut keys_to_delete = Vec::new();
        {
            if let Ok(table) = write_txn.open_table(tables::ACTIVITIES_TABLE) {
                for result in table.iter()? {
                    let (k, v) = result?;
                    let act: Activity = bincode::deserialize(v.value())?;
                    if act.user_id == user_id {
                        keys_to_delete.push(k.value().to_string());
                    }
                }
            }
            if let Ok(mut table) = write_txn.open_table(tables::ACTIVITIES_TABLE) {
                for key in keys_to_delete {
                    table.remove(key.as_str())?;
                }
            }
        }
        write_txn.commit()?;
        Ok(())
    }
}
