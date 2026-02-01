use dnd_scheduler::core::events::{Event, UserDeletedV1};
use dnd_scheduler::core::store::RedbEventStore;
use sqlx::sqlite::SqlitePoolOptions;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    // Target email to delete
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("Usage: cargo run --bin delete_user <email>");
        return Ok(());
    }
    let email = &args[1];

    println!("Attempting to delete user: {}", email);

    // 1. Connect to SQLite to find ID
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:data/read_model.sqlite".to_string());
    let pool = SqlitePoolOptions::new()
        .connect(&database_url)
        .await
        .expect("Failed to connect to SQLite");

    let user_opt: Option<(String,)> = sqlx::query_as("SELECT id FROM users WHERE email = ?")
        .bind(email)
        .fetch_optional(&pool)
        .await?;

    if let Some((id,)) = user_opt {
        println!("Found User ID: {}", id);

        // 2. Initialize Redb
        let event_store_url = std::env::var("EVENT_STORE_URL")
            .unwrap_or_else(|_| "data/event_store.redb".to_string());
        let event_store = Arc::new(RedbEventStore::new(&event_store_url)?);

        // 3. Append UserDeleted Event
        let event = Event::V1UserDeleted(UserDeletedV1 {
            id: id.clone(),
            email: email.to_string(),
        });

        let stream_id = format!("user-{}", id);
        match event_store.append_event(&stream_id, event).await {
            Ok(_) => println!("UserDeleted event appended to Event Store."),
            Err(e) => println!("Failed to append event: {}", e),
        }

        // 4. Delete dependent records first (FK constraints)
        sqlx::query("DELETE FROM user_sessions WHERE user_id = ?")
            .bind(&id)
            .execute(&pool)
            .await?;
        println!("User sessions deleted.");

        // 5. Delete from SQLite users
        sqlx::query("DELETE FROM users WHERE id = ?")
            .bind(&id)
            .execute(&pool)
            .await?;
        println!("User deleted from SQLite users table.");

        println!("SUCCESS: User {} deleted completely.", email);
    } else {
        println!(
            "User {} not found in SQLite. Checking Redb directly is harder without ID.",
            email
        );
    }

    Ok(())
}
