use dnd_scheduler::core::events::{Event, UserRoleUpdatedV1};
use dnd_scheduler::core::store::RedbEventStore;
use sqlx::sqlite::SqlitePoolOptions;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: promote_admin <email>");
        std::process::exit(1);
    }

    let email = &args[1];
    println!("Promoting user to ADMIN: {}", email);

    // 1. Connect to SQLite to find User ID
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = SqlitePoolOptions::new().connect(&database_url).await?;

    let user_id: Option<String> = sqlx::query_scalar("SELECT id FROM users WHERE email = ?")
        .bind(email)
        .fetch_optional(&pool)
        .await?;

    let user_id = match user_id {
        Some(id) => id,
        None => {
            eprintln!("User not found in SQLite: {}", email);
            std::process::exit(1);
        }
    };
    println!("Found User ID: {}", user_id);

    // 2. Update SQLite (Read Model)
    sqlx::query("UPDATE users SET role = 'admin' WHERE id = ?")
        .bind(&user_id)
        .execute(&pool)
        .await?;
    println!("Updated SQLite Read Model (role='admin').");

    // 3. Update Event Store (Write Model)
    // IMPORTANT: Server must be stopped to acquire lock!
    let event_store = RedbEventStore::new("data/event_store.redb")?;
    let event = Event::V1UserRoleUpdated(UserRoleUpdatedV1 {
        id: user_id.clone(),
        role: "admin".to_string(),
    });

    let stream_id = format!("user-{}", user_id);
    match event_store.append_event_unchecked(&stream_id, event).await {
        Ok(ver) => println!("Appended UserRoleUpdated event to Redb (Version {}).", ver),
        Err(e) => eprintln!("Failed to append event to Redb: {}", e),
    }

    Ok(())
}
