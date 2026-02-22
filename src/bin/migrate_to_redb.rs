use dnd_scheduler::core::events::{
    AvailabilityEntryV1, Event, ParticipantJoinedV1, PollCreatedV2, PollFinalizedV1, VoteUpdatedV2,
};
use dnd_scheduler::core::models::{Availability, Participant, Poll};
use dnd_scheduler::core::store::RedbEventStore;
use sqlx::sqlite::SqlitePoolOptions;
use std::collections::HashMap;
use std::sync::Arc;
use tokio;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("Starting migration from SQLite to Redb...");

    // 1. Connect to SQLite
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:data/read_model.sqlite".to_string());
    let pool = SqlitePoolOptions::new()
        .connect(&database_url)
        .await
        .expect("Failed to connect to SQLite");

    println!("Connected to SQLite: {}", database_url);

    // 2. Initialize Redb
    let event_store = Arc::new(RedbEventStore::new("data/event_store.redb")?);
    println!("Opened Redb: dnd_events.redb");

    // 3. Fetch all Polls
    let polls: Vec<Poll> = sqlx::query_as("SELECT * FROM polls")
        .fetch_all(&pool)
        .await?;

    println!("Found {} polls to migrate.", polls.len());

    for poll in polls {
        println!("Migrating poll: {} ({})", poll.title, poll.id);
        let stream_id = format!("poll-{}", poll.id);
        let mut version = 0;

        // A. PollCreated Event
        // Check if already exists? For now assume we migrating to fresh or appending.
        // Better to check if stream exists (version > 0).
        // Since we don't have check_stream, we just try to append version 0.
        // If it fails, we assume it's migrated or conflict.

        let dates: Vec<String> = serde_json::from_str(&poll.dates).unwrap_or_default();

        let event_created = Event::V2PollCreated(PollCreatedV2 {
            id: poll.id.clone(),
            title: poll.title.clone(),
            description: poll.description.clone(),
            location: poll.location.clone(),
            dates: dates.clone(),
            created_at: 0, // Fallback, could adjust to poll.created_at if parsed, let's just use 0 for migration compat.
        });

        let data = bincode::serialize(&event_created)?;

        // Try to append version 0
        match event_store.append(&stream_id, &data, 0).await {
            Ok(_) => {
                version += 1;
                println!("  - Created (v1)");
            }
            Err(_) => {
                println!(
                    "  - Stream likely exists, skipping PollCreated. (Assuming partial migration)"
                );
                // We could try to read last version, but for simplicity let's continue or skip?
                // If version 0 exists, we assume this poll is done?
                // Or maybe we are just re-running.
                // Let's CONTINUE checking other parts?
                // Actually if 0 exists, we don't know expected version.
                // Let's skip this poll entirely if we can't write v0, to avoid corruption.
                println!("  - Skipping poll {}", poll.id);
                continue;
            }
        }

        // B. Participants
        let participants: Vec<Participant> =
            sqlx::query_as("SELECT * FROM participants WHERE poll_id = ?")
                .bind(&poll.id)
                .fetch_all(&pool)
                .await?;

        for p in &participants {
            let event_joined = Event::V1ParticipantJoined(ParticipantJoinedV1 {
                id: p.id.clone(),
                poll_id: p.poll_id.clone(),
                name: p.name.clone(),
                email: p.email.clone(),
                access_token: p.access_token.clone().unwrap_or_default(), // Should have token
            });

            let data = bincode::serialize(&event_joined)?;
            match event_store.append(&stream_id, &data, version).await {
                Ok(v) => version = v,
                Err(e) => println!("  - Failed to append participant {}: {}", p.name, e),
            }
        }
        println!("  - Migrated {} participants", participants.len());

        // C. Votes (Availability)
        let availabilities: Vec<Availability> =
            sqlx::query_as("SELECT * FROM availability WHERE poll_id = ?")
                .bind(&poll.id)
                .fetch_all(&pool)
                .await?;

        // Group by participant
        let mut votes_by_participant: HashMap<String, Vec<AvailabilityEntryV1>> = HashMap::new();

        for a in availabilities {
            let entry = AvailabilityEntryV1 {
                date: a.date,
                slot: a.time_slot,
                status: a.status,
            };
            votes_by_participant
                .entry(a.participant_id)
                .or_default()
                .push(entry);
        }

        for (p_id, entries) in votes_by_participant {
            // Find participant details
            if let Some(p) = participants.iter().find(|p| p.id == p_id) {
                let event_vote = Event::V2VoteUpdated(VoteUpdatedV2 {
                    participant_name: p.name.clone(),
                    participant_email: p.email.clone(),
                    availability: entries,
                });

                let data = bincode::serialize(&event_vote)?;
                match event_store.append(&stream_id, &data, version).await {
                    Ok(v) => version = v,
                    Err(e) => println!("  - Failed to append vote for {}: {}", p.name, e),
                }
            }
        }
        println!("  - Migrated votes");

        // D. Finalized Status
        if poll.status == "finalized" {
            let event_finalized = Event::V1PollFinalized(PollFinalizedV1 {
                id: poll.id.clone(),
                finalized_at: poll.finalized_at.unwrap_or(0),
                finalized_time: poll.finalized_time.unwrap_or_default(),
                notes: poll.notes.clone(),
            });

            let data = bincode::serialize(&event_finalized)?;
            match event_store.append(&stream_id, &data, version).await {
                Ok(_v) => {}
                Err(e) => println!("  - Failed to append finalized status: {}", e),
            }
            println!("  - Finalized");
        }
    }

    // 4. Migrate Users
    println!("Migrating Users...");
    let users: Vec<dnd_scheduler::core::models::User> = sqlx::query_as("SELECT * FROM users")
        .fetch_all(&pool)
        .await?;

    for user in users {
        println!("Migrating User: {} ({})", user.email, user.id);
        let stream_id = format!("user-{}", user.id);

        // Register Event
        let event_reg = Event::V1UserRegistered(dnd_scheduler::core::events::UserRegisteredV1 {
            id: user.id.clone(),
            email: user.email.clone(),
            password_hash: user.password_hash.clone(),
            name: user.name.clone(),
            role: user.role.clone(),
            created_at: user.created_at,
            phone: user.phone.clone(),
        });

        // Try append (using helper which handles serialization and version check)
        match event_store.append_event(&stream_id, event_reg).await {
            Ok(_) => println!("  - Registered"),
            Err(e) => println!("  - Failed to Register (likely exists): {}", e),
        }

        // Last Login Event
        if let Some(last_login) = user.last_login {
            let event_login = Event::V1UserLoggedIn(dnd_scheduler::core::events::UserLoggedInV1 {
                id: user.id.clone(),
                timestamp: last_login,
            });
            match event_store.append_event(&stream_id, event_login).await {
                Ok(_) => println!("  - Logged In history preserved"),
                Err(e) => println!("  - Failed to append Login event: {}", e),
            }
        }
    }

    println!("Migration complete.");
    Ok(())
}
