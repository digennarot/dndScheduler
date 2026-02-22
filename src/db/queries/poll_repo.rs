use crate::core::models::{Availability, AvailabilityEntry, Participant, Poll, PollInstance};
use crate::db::tables;
use crate::db::DbPool;
use anyhow::Result;
use chrono::Utc;
use redb::{ReadableTable, WriteTransaction};

pub struct PollRepo;

impl PollRepo {
    pub fn create(
        tx: &WriteTransaction,
        poll_id: &str,
        title: &str,
        description: &str,
        location: &str,
        dates_json: &str,
        time_range_value: &str,
        _admin_token: &str,
        organizer_id: Option<&str>,
        recurrence_rule: Option<&str>,
    ) -> Result<()> {
        let created_at = Utc::now().timestamp();

        let poll = Poll {
            id: poll_id.to_string(),
            title: title.to_string(),
            description: description.to_string(),
            location: location.to_string(),
            created_at,
            dates: dates_json.to_string(),
            time_range: time_range_value.to_string(),
            status: "active".to_string(),
            finalized_at: None,
            finalized_time: None,
            notes: None,
            organizer_id: organizer_id.map(|s| s.to_string()),
            recurrence_rule: recurrence_rule.map(|s| s.to_string()),
        };

        let poll_bytes = bincode::serialize(&poll)?;

        let mut table = tx.open_table(tables::POLLS_TABLE)?;
        table.insert(poll_id, poll_bytes.as_slice())?;

        Ok(())
    }

    pub fn add_participant(
        tx: &WriteTransaction,
        participant_id: &str,
        poll_id: &str,
        name: &str,
        email: Option<&str>,
        access_token: Option<&str>,
    ) -> Result<()> {
        let participant = Participant {
            id: participant_id.to_string(),
            poll_id: poll_id.to_string(),
            name: name.to_string(),
            email: email.map(|s| s.to_string()),
            access_token: access_token.map(|s| s.to_string()),
            user_id: None,
        };

        let pt_bytes = bincode::serialize(&participant)?;
        let key = format!("{}:{}", poll_id, participant_id);

        let mut table = tx.open_table(tables::PARTICIPANTS_TABLE)?;
        table.insert(key.as_str(), pt_bytes.as_slice())?;

        Ok(())
    }

    pub fn create_instance(
        tx: &WriteTransaction,
        instance_id: &str,
        poll_id: &str,
        date: &str,
        start_time: &str,
        end_time: &str,
    ) -> Result<()> {
        let instance = PollInstance {
            id: instance_id.to_string(),
            poll_id: poll_id.to_string(),
            date: date.to_string(),
            start_time: start_time.to_string(),
            end_time: end_time.to_string(),
            status: "active".to_string(),
        };

        let bytes = bincode::serialize(&instance)?;

        let mut table = tx.open_table(tables::POLL_INSTANCES_TABLE)?;
        table.insert(instance_id, bytes.as_slice())?;

        Ok(())
    }

    pub fn get_details(
        pool: &DbPool,
        poll_id: &str,
    ) -> Result<Option<(Poll, Vec<Participant>, Vec<Availability>, Vec<PollInstance>)>> {
        let read_txn = pool.begin_read()?;

        // 1. Get Poll
        let polls_table = read_txn.open_table(tables::POLLS_TABLE)?;
        let Some(poll_record) = polls_table.get(poll_id)? else {
            return Ok(None);
        };
        let poll: Poll = bincode::deserialize(poll_record.value())?;

        // 2. Get Participants for this poll
        let mut participants = Vec::new();
        if let Ok(pt_table) = read_txn.open_table(tables::PARTICIPANTS_TABLE) {
            let start_key = format!("{}:", poll_id);
            let end_key = format!("{}~", poll_id); // '0'-'9', 'a'-'z', '~' is safely after ':'
            let range = pt_table.range(start_key.as_str()..end_key.as_str())?;
            for result in range {
                let (_, v) = result?;
                let pt: Participant = bincode::deserialize(v.value())?;
                participants.push(pt);
            }
        }

        // 3. Get Availability for this poll
        let mut availability = Vec::new();
        if let Ok(av_table) = read_txn.open_table(tables::AVAILABILITY_TABLE) {
            let start_key = format!("{}:", poll_id);
            let end_key = format!("{}~", poll_id);
            let range = av_table.range(start_key.as_str()..end_key.as_str())?;
            for result in range {
                let (_, v) = result?;
                let av: Availability = bincode::deserialize(v.value())?;
                availability.push(av);
            }
        }

        // 4. Get Instances for this poll
        let mut instances = Vec::new();
        if let Ok(inst_table) = read_txn.open_table(tables::POLL_INSTANCES_TABLE) {
            // Poll instances don't have a prefixed key strategy in the current implementation,
            // they seem to be stored with their own ID. We might need to iterate or filter.
            // Looking at create_instance, it takes tx, instance_id, poll_id...
            // It seems instances are stored by instance_id.
            // Let's check if we have a range strategy or if we need to iterate.
            for result in inst_table.iter()? {
                let (_, v) = result?;
                let inst: PollInstance = bincode::deserialize(v.value())?;
                if inst.poll_id == poll_id {
                    instances.push(inst);
                }
            }
        }

        Ok(Some((poll, participants, availability, instances)))
    }

    pub fn upsert_vote(
        pool: &DbPool,
        poll_id: &str,
        participant_id: &str, // This is the session_id for anonymous users
        name: &str,
        availability: Vec<AvailabilityEntry>,
    ) -> Result<()> {
        let write_txn = pool.begin_write()?;

        {
            // 1. Upsert Participant (Ensure they exist)
            let mut pt_table = write_txn.open_table(tables::PARTICIPANTS_TABLE)?;
            let pt_key = format!("{}:{}", poll_id, participant_id);

            let pt = if let Some(existing) = pt_table.get(pt_key.as_str())? {
                let mut pt: Participant = bincode::deserialize(existing.value())?;
                pt.name = name.to_string(); // Update name if changed
                pt
            } else {
                Participant {
                    id: participant_id.to_string(),
                    poll_id: poll_id.to_string(),
                    name: name.to_string(),
                    email: None,
                    access_token: None,
                    user_id: None,
                }
            };
            
            let pt_bytes = bincode::serialize(&pt)?;
            pt_table.insert(pt_key.as_str(), pt_bytes.as_slice())?;

            // 2. Clear existing votes for this participant
            let mut av_table = write_txn.open_table(tables::AVAILABILITY_TABLE)?;
            let av_start_key = format!("{}:{}:", poll_id, participant_id);
            let av_end_key = format!("{}:{}~", poll_id, participant_id);

            // Since Redb doesn't support deleting multiple items directly during iteration safely in all cases,
            // we first collect keys to delete.
            let mut keys_to_delete = Vec::new();
            {
                let range = av_table.range(av_start_key.as_str()..av_end_key.as_str())?;
                for result in range {
                    let (k, _) = result?;
                    keys_to_delete.push(k.value().to_string());
                }
            }
            for k in keys_to_delete {
                av_table.remove(k.as_str())?;
            }

            // 3. Insert new availability
            for entry in availability {
                let av_key = format!("{}:{}:{}:{}", poll_id, participant_id, entry.date, entry.time_slot);
                let av = Availability {
                    id: None,
                    poll_id: poll_id.to_string(),
                    participant_id: participant_id.to_string(),
                    date: entry.date,
                    time_slot: entry.time_slot,
                    status: entry.status,
                };
                let av_bytes = bincode::serialize(&av)?;
                av_table.insert(av_key.as_str(), av_bytes.as_slice())?;
            }
        }

        write_txn.commit()?;
        Ok(())
    }

    pub fn delete_poll(pool: &DbPool, poll_id: &str) -> Result<()> {
        let write_txn = pool.begin_write()?;

        {
            // Delete availability
            if let Ok(mut av_table) = write_txn.open_table(tables::AVAILABILITY_TABLE) {
                let start_key = format!("{}:", poll_id);
                let end_key = format!("{}~", poll_id);
                let mut keys = Vec::new();
                for result in av_table.range(start_key.as_str()..end_key.as_str())? {
                    keys.push(result?.0.value().to_string());
                }
                for k in keys {
                    av_table.remove(k.as_str())?;
                }
            }

            // Delete participants
            if let Ok(mut pt_table) = write_txn.open_table(tables::PARTICIPANTS_TABLE) {
                let start_key = format!("{}:", poll_id);
                let end_key = format!("{}~", poll_id);
                let mut keys = Vec::new();
                for result in pt_table.range(start_key.as_str()..end_key.as_str())? {
                    keys.push(result?.0.value().to_string());
                }
                for k in keys {
                    pt_table.remove(k.as_str())?;
                }
            }

            // Delete instances
            if let Ok(mut inst_table) = write_txn.open_table(tables::POLL_INSTANCES_TABLE) {
                let mut keys_to_delete = Vec::new();
                for result in inst_table.iter()? {
                    let (k, v) = result?;
                    let instance: PollInstance = bincode::deserialize(v.value())?;
                    if instance.poll_id == poll_id {
                        keys_to_delete.push(k.value().to_string());
                    }
                }
                for k in keys_to_delete {
                    inst_table.remove(k.as_str())?;
                }
            }

            // Delete poll
            let mut polls_table = write_txn.open_table(tables::POLLS_TABLE)?;
            polls_table.remove(poll_id)?;
        }

        write_txn.commit()?;
        Ok(())
    }

    pub fn delete_vote(pool: &DbPool, poll_id: &str, participant_id: &str) -> Result<()> {
        let write_txn = pool.begin_write()?;

        {
            // Delete availability
            if let Ok(mut av_table) = write_txn.open_table(tables::AVAILABILITY_TABLE) {
                let start_key = format!("{}:{}:", poll_id, participant_id);
                let end_key = format!("{}:{}~", poll_id, participant_id);
                let mut keys = Vec::new();
                for result in av_table.range(start_key.as_str()..end_key.as_str())? {
                    keys.push(result?.0.value().to_string());
                }
                for k in keys {
                    av_table.remove(k.as_str())?;
                }
            }

            // Delete participant
            if let Ok(mut pt_table) = write_txn.open_table(tables::PARTICIPANTS_TABLE) {
                let pt_key = format!("{}:{}", poll_id, participant_id);
                pt_table.remove(pt_key.as_str())?;
            }
        }

        write_txn.commit()?;
        Ok(())
    }

    pub fn finalize_poll(
        pool: &DbPool,
        poll_id: &str,
        finalized_time: &str,
        notes: Option<&str>,
    ) -> Result<bool> {
        let now = Utc::now().timestamp();
        let write_txn = pool.begin_write()?;

        let mut payload = None;
        let mut polls_table = write_txn.open_table(tables::POLLS_TABLE)?;
        if let Some(existing) = polls_table.get(poll_id)? {
            let mut poll: Poll = bincode::deserialize(existing.value())?;
            
            poll.status = "finalized".to_string();
            poll.finalized_at = Some(now);
            poll.finalized_time = Some(finalized_time.to_string());
            poll.notes = notes.map(|s| s.to_string());

            payload = Some(bincode::serialize(&poll)?);
        }
        
        let updated = if let Some(bytes) = payload {
            polls_table.insert(poll_id, bytes.as_slice())?;
            true
        } else {
            false
        };

        drop(polls_table);
        write_txn.commit()?;
        Ok(updated)
    }

    pub fn update(
        pool: &DbPool,
        poll_id: &str,
        title: &str,
        description: &str,
        location: &str,
        dates_json: &str,
        time_range_value: &str,
    ) -> Result<bool> {
        let write_txn = pool.begin_write()?;
        let mut payload = None;
        let mut polls_table = write_txn.open_table(tables::POLLS_TABLE)?;
        
        if let Some(existing) = polls_table.get(poll_id)? {
            let mut poll: Poll = bincode::deserialize(existing.value())?;
            poll.title = title.to_string();
            poll.description = description.to_string();
            poll.location = location.to_string();
            poll.dates = dates_json.to_string();
            poll.time_range = time_range_value.to_string();
            
            payload = Some(bincode::serialize(&poll)?);
        }
        
        let updated = if let Some(bytes) = payload {
            polls_table.insert(poll_id, bytes.as_slice())?;
            true
        } else {
            false
        };
        
        drop(polls_table);
        write_txn.commit()?;
        Ok(updated)
    }

    pub fn delete_participant_by_id(pool: &DbPool, participant_id: &str) -> Result<bool> {
        let write_txn = pool.begin_write()?;
        let mut target_key = None;
        let mut target_poll_id = String::new();

        {
            let pt_table = write_txn.open_table(tables::PARTICIPANTS_TABLE)?;
            for result in pt_table.iter()? {
                let (k, v) = result?;
                let pt: Participant = bincode::deserialize(v.value())?;
                if pt.id == participant_id {
                    target_key = Some(k.value().to_string());
                    target_poll_id = pt.poll_id;
                    break;
                }
            }
        }

        if let Some(key) = target_key {
            // Delete availability
            if let Ok(mut av_table) = write_txn.open_table(tables::AVAILABILITY_TABLE) {
                let start_key = format!("{}:{}:", target_poll_id, participant_id);
                let end_key = format!("{}:{}~", target_poll_id, participant_id);
                let mut av_keys = Vec::new();
                for result in av_table.range(start_key.as_str()..end_key.as_str())? {
                    av_keys.push(result?.0.value().to_string());
                }
                for k in av_keys {
                    av_table.remove(k.as_str())?;
                }
            }
            
            // Delete participant
            {
                let mut pt_table = write_txn.open_table(tables::PARTICIPANTS_TABLE)?;
                pt_table.remove(key.as_str())?;
            }
            write_txn.commit()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn get_user_participation(pool: &DbPool, user_id: &str) -> Result<Vec<crate::core::models::PollParticipation>> {
        let read_txn = pool.begin_read()?;
        let mut participations = Vec::new();

        if let Ok(pt_table) = read_txn.open_table(tables::PARTICIPANTS_TABLE) {
            for result in pt_table.iter()? {
                let (_, v) = result?;
                let pt: Participant = bincode::deserialize(v.value())?;
                if pt.user_id.as_deref() == Some(user_id) {
                    let mut poll_title = String::new();
                    if let Ok(polls_table) = read_txn.open_table(tables::POLLS_TABLE) {
                        if let Ok(Some(poll_record)) = polls_table.get(pt.poll_id.as_str()) {
                            let poll: Poll = bincode::deserialize(poll_record.value())?;
                            poll_title = poll.title;
                        }
                    }
                    participations.push(crate::core::models::PollParticipation {
                        poll_id: pt.poll_id,
                        poll_title,
                        participant_name: pt.name,
                        joined_at: None,
                    });
                }
            }
        }
        Ok(participations)
    }

    pub fn get_user_availability(pool: &DbPool, user_id: &str) -> Result<Vec<crate::core::models::Availability>> {
        let read_txn = pool.begin_read()?;
        let mut target_participant_ids = std::collections::HashSet::new();
        
        if let Ok(pt_table) = read_txn.open_table(tables::PARTICIPANTS_TABLE) {
            for result in pt_table.iter()? {
                let (_, v) = result?;
                let pt: Participant = bincode::deserialize(v.value())?;
                if pt.user_id.as_deref() == Some(user_id) {
                    target_participant_ids.insert(pt.id);
                }
            }
        }

        let mut availability = Vec::new();
        if let Ok(av_table) = read_txn.open_table(tables::AVAILABILITY_TABLE) {
            for result in av_table.iter()? {
                let (_, v) = result?;
                let av: Availability = bincode::deserialize(v.value())?;
                if target_participant_ids.contains(&av.participant_id) {
                    availability.push(av);
                }
            }
        }
        // Ideally we sort these depending on needs
        Ok(availability)
    }
}
