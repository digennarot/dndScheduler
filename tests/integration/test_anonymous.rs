use crate::helpers;
use axum::http::StatusCode;

use serde_json::json;

#[tokio::test]
async fn test_anonymous_join_flow() {
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_test_writer()
        .try_init();

    let (app, pool) = helpers::setup_test_app().await;

    // 1. Create a poll (authenticated)
    let poll_id = helpers::create_test_poll_db(&pool).await;

    // 2. Join as Anonymous (No Email)
    let client = axum_test::TestServer::new(app).unwrap();
    let join_payload = json!({
        "name": "Anonymous Player",
        "email": null
    });

    let response = client
        .post(&format!("/api/polls/{}/join", poll_id))
        .add_header("X-Forwarded-For", "127.0.0.1")
        .json(&join_payload)
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let json: serde_json::Value = response.json();
    assert!(
        json.get("access_token").is_some(),
        "Should return access token"
    );
    let participant_id_1 = json.get("id").unwrap().as_str().unwrap();

    // 3. Join as Another Anonymous (No Email)
    let join_payload_2 = json!({
        "name": "Anonymous Player 2",
        "email": null
    });

    let response_2 = client
        .post(&format!("/api/polls/{}/join", poll_id))
        .add_header("X-Forwarded-For", "127.0.0.1")
        .json(&join_payload_2)
        .await;

    assert_eq!(response_2.status_code(), StatusCode::OK);
    let json_2: serde_json::Value = response_2.json();
    let participant_id_2 = json_2.get("id").unwrap().as_str().unwrap();

    // Verify IDs are different
    assert_ne!(participant_id_1, participant_id_2);

    // 4. Verify in DB
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM participants WHERE poll_id = ?")
        .bind(&poll_id)
        .fetch_one(&pool)
        .await
        .unwrap();

    // 2 anonymous + 0 organizers (organizer is not automatically a participant unless they join)
    assert_eq!(count, 2);
}
