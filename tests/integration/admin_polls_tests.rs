use super::helpers;
use axum::http::StatusCode;
use serde_json::json;
use axum_test::TestServer;

#[tokio::test]
async fn test_admin_login_flow() {
    // Use a unique token to avoid races when other tests read ADMIN_TOKEN
    let test_token = "test_admin_token_login_flow";
    // SAFETY: test binaries are single-threaded by default in tokio::test; we
    // document this limitation in the comment so future refactors are aware.
    // If parallel test execution of this module becomes an issue, use a mutex.
    unsafe { std::env::set_var("ADMIN_TOKEN", test_token); }

    let ctx = helpers::setup_test_app().await;
    let server = TestServer::new(ctx.app).unwrap();

    // Try with wrong token
    let res = server
        .post("/api/admin/login")
        .json(&json!({ "token": "wrong" }))
        .await;
    assert_eq!(res.status_code(), StatusCode::UNAUTHORIZED);

    // Try with correct token
    let res = server
        .post("/api/admin/login")
        .json(&json!({ "token": test_token }))
        .await;
    assert_eq!(res.status_code(), StatusCode::OK);
    
    // Verify Cookie
    let cookie = res.cookie("admin_session");
    assert_eq!(cookie.name(), "admin_session");
}

#[tokio::test]
async fn test_admin_can_delete_poll() {
    let ctx = helpers::setup_test_app().await;
    let pool = ctx.pool.clone();
    let server = TestServer::new(ctx.app).unwrap();

    // 1. Create Poll via API
    let (_org_id, org_token) = helpers::create_test_user_with_session(&pool, "org3@test.com", "pass", "player").await;
    let create_res = server
        .post("/api/polls")
        .add_header("Authorization", format!("Bearer {}", org_token))
        .json(&json!({
            "title": "To Be Deleted",
            "description": "D",
            "location": "L",
            "dates": ["2027-01-01"],
            "participants": []
        }))
        .await;
    assert_eq!(create_res.status_code(), StatusCode::OK);
    let poll_id = create_res.json::<serde_json::Value>()["id"].as_str().unwrap().to_string();

    // 2. Setup Admin Session directly in DB
    let (_admin_id, admin_token);
    {
        let (id, token) = helpers::create_test_user_with_session(&pool, "admin3@test.com", "pass", "admin").await;
        _admin_id = id;
        admin_token = token;
    }

    // 3. Delete Poll (Admin Endpoint)
    let response = server
        .delete(&format!("/api/admin/polls/{}", poll_id))
        .add_header("Cookie", format!("admin_session={}", admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::NO_CONTENT);

    // 4. Verify Gone
    let get_res = server.get(&format!("/api/polls/{}", poll_id)).await;
    assert_eq!(get_res.status_code(), StatusCode::NOT_FOUND); 
}

#[tokio::test]
async fn test_admin_can_delete_vote() {
    let ctx = helpers::setup_test_app().await;
    let pool = ctx.pool.clone();
    let server = TestServer::new_with_config(ctx.app, axum_test::TestServerConfig {
        save_cookies: true,
        ..Default::default()
    }).unwrap();

    // 1. Create Poll via API (important for projection)
    let (_org_id, org_token) = helpers::create_test_user_with_session(&pool, "org4@test.com", "pass", "player").await;
    let res = server.post("/api/polls")
        .add_header("Authorization", format!("Bearer {}", org_token))
        .json(&json!({
            "title": "Delete Vote Test",
            "description": "D",
            "location": "L",
            "dates": ["2027-01-01"],
            "participants": []
        }))
        .await;
    let poll_id = res.json::<serde_json::Value>()["id"].as_str().unwrap().to_string();

    // 2. Initialize session by fetching poll
    server.get(&format!("/api/polls/{}", poll_id)).await;

    // 3. Vote
    let vote_res = server.post(&format!("/api/polls/{}/vote", poll_id))
        .json(&json!({
            "name": "Troll User",
            "availability": [{"date": "2027-01-01", "timeSlot": "10:00", "status": "available"}]
        }))
        .await;
    assert_eq!(vote_res.status_code(), StatusCode::OK);
    
    // Get participant ID from details
    let poll_details = server.get(&format!("/api/polls/{}", poll_id)).await.json::<serde_json::Value>();
    let participants = poll_details["participants"].as_array().unwrap();
    let troll = participants.iter().find(|p| p["name"] == "Troll User").expect("Troll should exist");
    let participant_id = troll["id"].as_str().unwrap();

    // 4. Admin Delete Vote
    let (_, admin_token) = helpers::create_test_user_with_session(&pool, "admin4@test.com", "pass", "admin").await;

    let del_res = server
        .delete(&format!("/api/admin/polls/{}/participants/{}", poll_id, participant_id))
        .add_header("Authorization", format!("Bearer {}", admin_token))
        .await;
    
    assert_eq!(del_res.status_code(), StatusCode::NO_CONTENT);

    // 5. Verify Gone
    let poll_details_after = server.get(&format!("/api/polls/{}", poll_id)).await.json::<serde_json::Value>();
    let participants_after = poll_details_after["participants"].as_array().unwrap();
    assert!(participants_after.iter().find(|p| p["id"] == participant_id).is_none());
}
