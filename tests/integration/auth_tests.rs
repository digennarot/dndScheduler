use super::helpers;
use axum::http::StatusCode;
use axum_test::TestServer;

#[tokio::test]
async fn test_user_registration_and_login() {
    let ctx = helpers::setup_test_app().await;
    let server = TestServer::new(ctx.app).unwrap();

    let email = "newuser@test.com";
    let password = "SecurePass123!@#";
    let name = "Test User";

    // 1. Register via API (important for projection)
    let reg_response = server
        .post("/api/auth/register")
        .json(&serde_json::json!({
            "email": email,
            "password": password,
            "name": name,
            "phone": "+1234567890"
        }))
        .await;
    
    assert_eq!(reg_response.status_code(), StatusCode::CREATED);
    let reg_data = reg_response.json::<serde_json::Value>();
    assert_eq!(reg_data["user"]["email"], email);

    // 2. Login
    let login_response = server
        .post("/api/auth/login")
        .json(&serde_json::json!({
            "email": email,
            "password": password
        }))
        .await;

    assert_eq!(login_response.status_code(), StatusCode::OK);
    let auth_data = login_response.json::<serde_json::Value>();
    assert!(auth_data.get("token").is_some());
    assert_eq!(auth_data["user"]["email"], email);

    // 3. Get Current User with token
    let token = auth_data["token"].as_str().unwrap();
    let get_user_res = server
        .get("/api/auth/me")
        .add_header("Authorization", format!("Bearer {}", token))
        .await;
    
    assert_eq!(get_user_res.status_code(), StatusCode::OK);
    assert_eq!(get_user_res.json::<serde_json::Value>()["email"], email);
}

#[tokio::test]
async fn test_login_invalid_credentials() {
    let ctx = helpers::setup_test_app().await;
    let server = TestServer::new(ctx.app).unwrap();

    let email = "wrong@test.com";
    let password = "SecurePass123!@#";
    
    // Create user via registration
    server
        .post("/api/auth/register")
        .json(&serde_json::json!({
            "email": email,
            "password": password,
            "name": "Test User"
        }))
        .await;

    // Wrong password
    let res = server
        .post("/api/auth/login")
        .json(&serde_json::json!({
            "email": email,
            "password": "WrongPassword123!"
        }))
        .await;
    assert_eq!(res.status_code(), StatusCode::UNAUTHORIZED);

    // Missing email
    let res = server
        .post("/api/auth/login")
        .json(&serde_json::json!({
            "email": "nonexistent@test.com",
            "password": password
        }))
        .await;
    assert_eq!(res.status_code(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_logout() {
    let ctx = helpers::setup_test_app().await;
    let pool = ctx.pool.clone();
    let server = TestServer::new(ctx.app).unwrap();

    let (_user_id, token) = helpers::create_test_user_with_session(&pool, "user@test.com", "pass", "player").await;

    // Verify session exists in DB
    let session = dnd_scheduler::db::queries::admin_repo::SessionRepo::get_user_session(&pool, &token).unwrap();
    assert!(session.is_some());

    // Logout
    let logout_res = server
        .post(&format!("/api/auth/logout/{}", token)) // Handler seems to take token in path based on src/security/auth.rs:760
        .await;
        
    assert_eq!(logout_res.status_code(), StatusCode::OK);

    // Verify session gone
    let session_after = dnd_scheduler::db::queries::admin_repo::SessionRepo::get_user_session(&pool, &token).unwrap();
    assert!(session_after.is_none());
}
