// Test Helpers Module
// Utilities per setup e teardown dei test

use dnd_scheduler::db::DbPool;
use std::sync::Arc;
use tracing_subscriber;
use uuid::Uuid;

pub struct TestContext {
    pub app: axum::Router,
    pub pool: DbPool,
    pub _db_dir: tempfile::TempDir,
    pub _ev_dir: tempfile::TempDir,
}

pub async fn setup_test_app() -> TestContext {
    let _ = tracing_subscriber::fmt::try_init();
    std::env::set_var("DND_DISABLE_RATE_LIMIT", "1");

    let db_dir = tempfile::tempdir().expect("Failed to create temp db dir");
    let db_path = db_dir.path().join("test_db.redb");
    
    let ev_dir = tempfile::tempdir().expect("Failed to create temp ev dir");
    let ev_path = ev_dir.path().join("test_ev.redb");

    let db = redb::Database::create(&db_path).expect("Failed to create test redb db");
    let pool = Arc::new(db);
    dnd_scheduler::db::setup_redb_schema(&pool).expect("Failed to setup schema");

    let app = dnd_scheduler::create_router(pool.clone(), Some(ev_path.to_str().unwrap().to_string())).await;
    
    TestContext {
        app,
        pool,
        _db_dir: db_dir,
        _ev_dir: ev_dir,
    }
}

pub async fn setup_test_db() -> (DbPool, tempfile::TempDir) {
    let dir = tempfile::tempdir().expect("Failed to create temp dir");
    let path = dir.path().join("test.redb");
    let db = redb::Database::create(&path).expect("Failed to create test db");
    let pool = Arc::new(db);
    dnd_scheduler::db::setup_redb_schema(&pool).expect("Failed to setup schema");
    (pool, dir)
}

pub async fn create_test_user_with_session(
    pool: &DbPool,
    email: &str,
    password: &str,
    role: &str,
) -> (String, String) {
    let user_id = Uuid::new_v4().to_string();
    let password_hash = bcrypt::hash(password, bcrypt::DEFAULT_COST).unwrap();
    
    let user = dnd_scheduler::core::models::User {
        id: user_id.clone(),
        email: email.to_string(),
        password_hash,
        name: "Test User".to_string(),
        role: role.to_string(),
        created_at: chrono::Utc::now().timestamp(),
        last_login: None,
        phone: None,
        consent_marketing: false,
        consent_analytics: false,
        privacy_policy_accepted_at: None,
    };
    
    dnd_scheduler::db::queries::user_repo::UserRepo::create_or_update(pool, &user).expect("Failed to create user");
    
    let session_token = Uuid::new_v4().to_string();
    let expires_at = chrono::Utc::now().timestamp() + 3600;
    
    let session = dnd_scheduler::core::models::UserSession {
        id: Uuid::new_v4().to_string(),
        user_id: user_id.clone(),
        token: session_token.clone(),
        expires_at,
        created_at: chrono::Utc::now().timestamp(),
    };
    
    dnd_scheduler::db::queries::admin_repo::SessionRepo::create_user_session(pool, &session)
        .expect("Failed to create session");
        
    (user_id, session_token)
}

pub async fn create_test_poll(pool: &DbPool, organizer_id: Option<&str>) -> String {
    let future_date = (chrono::Utc::now().date_naive() + chrono::Duration::days(7))
        .format("%Y-%m-%d")
        .to_string();
    let dates_json = serde_json::to_string(&[&future_date]).unwrap();
    let poll_id = Uuid::new_v4().to_string();
    let write_txn = pool.begin_write().unwrap();
    dnd_scheduler::db::queries::poll_repo::PollRepo::create(
        &write_txn,
        &poll_id,
        "Test Poll",
        "Description",
        "Location",
        &dates_json,
        "20:00-22:00",
        "admin-token",
        organizer_id,
        None,
    ).unwrap();
    write_txn.commit().unwrap();
    poll_id
}

