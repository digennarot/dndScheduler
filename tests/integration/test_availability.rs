use super::helpers;
use axum::http::StatusCode;

use serde_json::json;
use axum_test::TestServer;

#[tokio::test]
async fn test_availability_flow() {
    let ctx = helpers::setup_test_app().await;
    let pool = ctx.pool.clone();
    let server = TestServer::new(ctx.app).unwrap();

    // 1. Create Poll via API (important for projection)
    let create_response = server
        .post("/api/polls")
        .json(&json!({
            "title": "Availability Flow Test",
            "description": "D",
            "location": "L",
            "dates": ["2027-11-01"],
            "participants": []
        }))
        .await;
    
    assert_eq!(create_response.status_code(), StatusCode::OK);
    let poll_id = create_response.json::<serde_json::Value>()["id"].as_str().unwrap().to_string();

    // 2. Join Poll via API to get Participant ID and Token
    let join_response = server
        .post(&format!("/api/polls/{}/join", poll_id))
        .json(&json!({
            "name": "Test User",
            "email": "test@example.com"
        }))
        .await;

    assert_eq!(join_response.status_code(), StatusCode::OK);
    let join_data = join_response.json::<serde_json::Value>();
    let participant_id = join_data["id"].as_str().unwrap().to_string();
    let access_token = join_data["access_token"].as_str().unwrap().to_string();

    // 3. Update Availability (Valid)
    let update_response = server
        .post(&format!("/api/polls/{}/participants/{}/availability", poll_id, participant_id))
        .json(&json!({
            "availability": [{
                "date": "2027-11-01",
                "timeSlot": "20:00",
                "status": "available"
            }],
            "access_token": access_token
        }))
        .await;

    assert_eq!(update_response.status_code(), StatusCode::OK);

    // 4. Update Again (Upsert check)
    let update_response_2 = server
        .post(&format!("/api/polls/{}/participants/{}/availability", poll_id, participant_id))
        .json(&json!({
            "availability": [{
                "date": "2027-11-01",
                "timeSlot": "21:00",
                "status": "tentative"
            }],
            "access_token": access_token
        }))
        .await;

    assert_eq!(update_response_2.status_code(), StatusCode::OK);

    // Verify DB state via Repo
    let (_, participants, availability, _) = dnd_scheduler::db::queries::poll_repo::PollRepo::get_details(&pool, &poll_id)
        .unwrap()
        .expect("Poll should exist");

    assert_eq!(participants.len(), 1);
    assert_eq!(availability.len(), 1);
    assert_eq!(availability[0].time_slot, "21:00");
    assert_eq!(availability[0].status, "tentative");
}

#[tokio::test]
async fn test_availability_rate_limiting() {
    // This test requires rate-limiting to be ENABLED. When DND_DISABLE_RATE_LIMIT=1
    // (set by setup_test_app), rate-limiting is always off and this test would
    // silently pass without testing anything. We skip explicitly in that case.
    if std::env::var("DND_DISABLE_RATE_LIMIT").is_ok() {
        eprintln!("SKIP: test_availability_rate_limiting requires rate-limiting enabled");
        return;
    }

    let ctx = helpers::setup_test_app().await;
    let server = TestServer::new(ctx.app).unwrap();
    
    let create_res = server.post("/api/polls")
        .json(&json!({"title": "RL Test", "description": "D", "location": "L", "dates": ["2027-02-20"], "participants": []}))
        .await;
    let poll_id = create_res.json::<serde_json::Value>()["id"].as_str().unwrap().to_string();

    let join_res = server.post(&format!("/api/polls/{}/join", poll_id))
        .json(&json!({"name": "Tester", "email": "tester@rl.com"}))
        .await;
    let join_data = join_res.json::<serde_json::Value>();
    let participant_id = join_data["id"].as_str().unwrap().to_string();
    let access_token = join_data["access_token"].as_str().unwrap().to_string();

    let url = format!("/api/polls/{}/participants/{}/availability", poll_id, participant_id);
    let payload = json!({
        "availability": [],
        "access_token": access_token
    });

    let mut rate_limit_hit = false;
    for _ in 0..20 {
        let res = server
            .post(&url)
            .add_header("X-Forwarded-For", "1.2.3.4")
            .json(&payload)
            .await;
            
        if res.status_code() == StatusCode::TOO_MANY_REQUESTS {
            rate_limit_hit = true;
            break;
        }
    }

    assert!(rate_limit_hit, "Rate limit should have been triggered after 20 requests to the same endpoint");
}
