use super::helpers;

use dnd_scheduler::core::models::User;
use dnd_scheduler::db::queries::user_repo::UserRepo;
use axum_test::TestServer;
use serde_json::json;

async fn create_authelia_user(pool: &dnd_scheduler::db::DbPool, email: &str, name: &str, role: &str) -> String {
    let user_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp();
    let placeholder_hash = "$authelia_sso$";

    let user = User {
        id: user_id.clone(),
        email: email.to_string(),
        password_hash: placeholder_hash.to_string(),
        name: name.to_string(),
        role: role.to_string(),
        created_at: now,
        last_login: Some(now),
        phone: None,
        consent_marketing: false,
        consent_analytics: false,
        privacy_policy_accepted_at: None,
    };
    UserRepo::create_or_update(pool, &user).unwrap();
    user_id
}

#[tokio::test]
async fn test_authelia_user_auto_creation() {
    let (pool, _dir) = helpers::setup_test_db().await;
    let email = "nuovo@cronachednd.it";

    let existing = UserRepo::find_by_email(&pool, email).unwrap();
    assert!(existing.is_none());

    let _user_id = create_authelia_user(&pool, email, "Nuovo Utente", "player").await;

    let saved = UserRepo::find_by_email(&pool, email).unwrap();
    assert!(saved.is_some());
}

#[tokio::test]
async fn test_authelia_user_cannot_login_with_password() {
    let ctx = helpers::setup_test_app().await;
    let pool = ctx.pool.clone();
    let server = TestServer::new(ctx.app).unwrap();

    let email = "sso@cronachednd.it";
    create_authelia_user(&pool, email, "SSO User", "player").await;

    // Attempt login with any password
    let res = server.post("/api/auth/login")
        .json(&json!({ "email": email, "password": "password" }))
        .await;
    
    // Should fail because "$authelia_sso$" is not a valid bcrypt hash
    assert_eq!(res.status_code(), StatusCode::UNAUTHORIZED);
}

use axum::http::StatusCode;

#[tokio::test]
async fn test_fallback_to_bearer_token() {
    let ctx = helpers::setup_test_app().await;
    let pool = ctx.pool.clone();
    let server = TestServer::new(ctx.app).unwrap();

    let (_user_id, token) = helpers::create_test_user_with_session(&pool, "bearer@cronachednd.it", "pass", "player").await;

    let res = server.get("/api/auth/me")
        .add_header("Authorization", format!("Bearer {}", token))
        .await;
    
    assert_eq!(res.status_code(), StatusCode::OK);
    assert_eq!(res.json::<serde_json::Value>()["email"], "bearer@cronachednd.it");
}
