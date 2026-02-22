use super::helpers;
use axum::http::StatusCode;
use serde_json::json;
use axum_test::TestServer;

#[tokio::test]
async fn test_anonymous_join_flow() {
    let ctx = helpers::setup_test_app().await;
    let pool = ctx.pool.clone();
    let server = TestServer::new_with_config(ctx.app, axum_test::TestServerConfig {
        save_cookies: true,
        ..Default::default()
    }).unwrap();

    // 1. Create Poll (authenticated)
    let (_org_id, org_token) = helpers::create_test_user_with_session(&pool, "org_anon@test.com", "pass", "player").await;
    
    let create_response = server
        .post("/api/polls")
        .add_header("Authorization", format!("Bearer {}", org_token))
        .add_header("host", "localhost")
        .json(&json!({
            "title": "Anon Test",
            "description": "D",
            "location": "L",
            "dates": ["2027-01-01"],
            "participants": []
        }))
        .await;
    
    assert_eq!(create_response.status_code(), StatusCode::OK);
    let poll_id = create_response.json::<serde_json::Value>()["id"].as_str().unwrap().to_string();

    // 2. Join as Anonymous
    let join_response = server
        .post(&format!("/api/polls/{}/join", poll_id))
        .json(&json!({
            "name": "Anonymous Player",
            "email": null
        }))
        .await;

    assert_eq!(join_response.status_code(), StatusCode::OK);
    let join_data = join_response.json::<serde_json::Value>();
    assert!(join_data.get("access_token").is_some());
    let participant_id_1 = join_data["id"].as_str().unwrap().to_string();

    // 3. Join as Another Anonymous
    let join_response_2 = server
        .post(&format!("/api/polls/{}/join", poll_id))
        .json(&json!({
            "name": "Anonymous Player 2",
            "email": null
        }))
        .await;

    assert_eq!(join_response_2.status_code(), StatusCode::OK);
    let participant_id_2 = join_response_2.json::<serde_json::Value>()["id"].as_str().unwrap().to_string();

    assert_ne!(participant_id_1, participant_id_2);

    // Verify DB state
    let (_, participants, _, _) = dnd_scheduler::db::queries::poll_repo::PollRepo::get_details(&pool, &poll_id)
        .unwrap()
        .expect("Poll should exist");
    
    assert_eq!(participants.len(), 2);
}

#[tokio::test]
async fn test_anonymous_vote_flow() {
    let ctx = helpers::setup_test_app().await;
    let pool = ctx.pool.clone();
    let server = TestServer::new(ctx.app).unwrap();

    // 1. Create Poll
    let (_org_id, org_token) = helpers::create_test_user_with_session(&pool, "org_anon2@test.com", "pass", "player").await;
    let create_res = server.post("/api/polls")
        .add_header("Authorization", format!("Bearer {}", org_token))
        .add_header("host", "localhost")
        .json(&json!({"title": "Vote Test", "description": "D", "location": "L", "dates": ["2027-02-20"], "participants": []}))
        .await;
    let poll_id = create_res.json::<serde_json::Value>()["id"].as_str().unwrap().to_string();

    // 2. Fetch Poll initially (middleware should set session cookie if it's missing, but here we test the vote flow)
    // For anonymous voting, we usually get a session cookie from the first GET or JOIN.
    let res = server.get(&format!("/api/polls/{}", poll_id))
        .add_header("host", "localhost")
        .await;
    assert_eq!(res.status_code(), StatusCode::OK);
    let cookie = res.cookie("dnd_session");
    
    // 3. Vote
    let vote_res = server
        .post(&format!("/api/polls/{}/vote", poll_id))
        .add_header("host", "localhost")
        .add_cookie(cookie.clone())
        .json(&json!({
            "name": "Voter 1",
            "availability": [
                { "date": "2027-02-20", "timeSlot": "18:00", "status": "available" }
            ]
        }))
        .await;

    assert_eq!(vote_res.status_code(), StatusCode::OK);

    // 4. Verify Vote in GET response
    let res_2 = server.get(&format!("/api/polls/{}", poll_id))
        .add_header("host", "localhost")
        .add_cookie(cookie)
        .await;
    assert_eq!(res_2.status_code(), StatusCode::OK);
    let json_2 = res_2.json::<serde_json::Value>();
    
    let my_vote = json_2["my_vote"].as_array().expect("my_vote should be an array");
    assert_eq!(my_vote.len(), 1);
    assert_eq!(my_vote[0]["time_slot"], "18:00");

    // 5. Verify DB state
    let (_, _, availability, _) = dnd_scheduler::db::queries::poll_repo::PollRepo::get_details(&pool, &poll_id)
        .unwrap()
        .expect("Poll should exist");
    assert_eq!(availability.len(), 1);
}
