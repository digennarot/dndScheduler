use bcrypt::{hash, DEFAULT_COST};
use dnd_scheduler::core::events::{Event, UserPasswordChangedV1};
use dnd_scheduler::core::store::RedbEventStore;
use sqlx::sqlite::SqlitePoolOptions;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: reset_password <email> <new_password>");
        std::process::exit(1);
    }

    let email = &args[1];
    let new_password = &args[2];

    println!("Resetting password for user: {}", email);

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

    // 2. Hash New Password
    let password_hash = hash(new_password, DEFAULT_COST)?;

    // 3. Update SQLite (Read Model)
    sqlx::query("UPDATE users SET password_hash = ? WHERE id = ?")
        .bind(&password_hash)
        .bind(&user_id)
        .execute(&pool)
        .await?;
    println!("Updated SQLite Read Model.");

    // 4. Update Event Store (Write Model)
    // IMPORTANT: Server must be stopped to acquire lock!
    let event_store = RedbEventStore::new("data/event_store.redb")?;
    let event = Event::V1UserPasswordChanged(UserPasswordChangedV1 {
        id: user_id.clone(),
        password_hash,
    });

    let stream_id = format!("user-{}", user_id);
    // Use append_event_unchecked for simplicity in maintenance tool
    match event_store.append_event_unchecked(&stream_id, event).await {
        Ok(ver) => println!(
            "Appended UserPasswordChanged event to Redb (Version {}).",
            ver
        ),
        Err(e) => eprintln!("Failed to append event to Redb: {}", e),
    }

    Ok(())
}
