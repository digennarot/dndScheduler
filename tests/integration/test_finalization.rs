use super::helpers;
use axum_test::TestServer;
use serde_json::json;


#[tokio::test]
async fn test_poll_finalization_flow() {
    let ctx = helpers::setup_test_app().await;
    let pool = ctx.pool.clone();
    let server = TestServer::new_with_config(ctx.app, axum_test::TestServerConfig {
        save_cookies: true,
        ..Default::default()
    }).unwrap();

    // 1. Create a Poll via API
    let create_response = server
        .post("/api/polls")
        .json(&json!({
            "title": "Finalization Test",
            "description": "Testing finalization",
            "location": "Test Loc",
            "dates": ["2027-11-01"],
            "participants": ["p1@example.com"]
        }))
        .await;
    
    assert_eq!(create_response.status_code(), 200);
    let poll_data: serde_json::Value = create_response.json();
    let poll_id = poll_data["id"].as_str().unwrap();

    // 2. Initialize session by fetching poll
    server.get(&format!("/api/polls/{}", poll_id)).await;

    // 3. Vote
    let vote_response = server
        .post(&format!("/api/polls/{}/vote", poll_id))
        .json(&json!({
            "name": "Voter 1",
            "availability": [{
                "date": "2027-11-01",
                "timeSlot": "20:00",
                "status": "available"
            }]
        }))
        .await;
    assert_eq!(vote_response.status_code(), 200);
    
    // 4. Finalize Poll (Admin authorization needed)
    let (_user_id, admin_token) = helpers::create_test_user_with_session(&pool, "admin_fin@test.com", "pass", "admin").await;

    let finalize_response = server
        .put(&format!("/api/polls/{}/finalize", poll_id))
        .add_header("Authorization", format!("Bearer {}", admin_token))
        .json(&json!({
            "finalized_time": "2027-11-01_20:00",
            "notes": "Locked in!"
        }))
        .await;
        
    assert_eq!(finalize_response.status_code(), 200);
    
    // 5. Verify Finalized State
    let get_response = server.get(&format!("/api/polls/{}", poll_id)).await;
    assert_eq!(get_response.status_code(), 200);
    let poll_details = get_response.json::<serde_json::Value>();
    assert_eq!(poll_details["poll"]["status"], "finalized");
    assert_eq!(poll_details["poll"]["finalized_time"], "2027-11-01_20:00");
    assert_eq!(poll_details["poll"]["notes"], "Locked in!");
    
    // Check votes
    let votes = poll_details["votes"].as_array().expect("votes should exist");
    assert!(!votes.is_empty());
}
