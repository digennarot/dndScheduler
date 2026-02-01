use crate::helpers::{create_test_poll_db, setup_test_app};
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use dnd_scheduler::core::models::{AvailabilityEntry, JoinPollRequest, UpdateAvailabilityRequest};
use serde_json::Value;
use tower::util::ServiceExt;

#[tokio::test]
async fn test_availability_flow_and_rate_limit() {
    let (app, pool) = setup_test_app().await;

    // 1. Create Poll in DB directly
    let poll_id = create_test_poll_db(&pool).await;

    // 2. Join Poll via API to get Participant ID and Token
    let join_payload = JoinPollRequest {
        name: "Test User".to_string(),
        email: None,
    };

    let join_req = Request::builder()
        .method("POST")
        .uri(format!("/api/polls/{}/join", poll_id))
        .header("Content-Type", "application/json")
        .header("X-Forwarded-For", "127.0.0.1")
        .body(Body::from(serde_json::to_string(&join_payload).unwrap()))
        .unwrap();

    let join_response = app.clone().oneshot(join_req).await.unwrap();

    let status = join_response.status();
    let body_bytes = axum::body::to_bytes(join_response.into_body(), usize::MAX)
        .await
        .unwrap();

    if status != StatusCode::OK {
        println!("Join failed: {:?}", String::from_utf8_lossy(&body_bytes));
        panic!("Join failed with status {:?}", status);
    }

    let body_json: Value = serde_json::from_slice(&body_bytes).unwrap();

    let participant_id = body_json.get("id").unwrap().as_str().unwrap().to_string();
    let access_token = body_json
        .get("access_token")
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();

    // 3. Update Availability (Valid)
    let update_payload = UpdateAvailabilityRequest {
        availability: vec![AvailabilityEntry {
            date: "2023-10-10".to_string(),
            time_slot: "18:00".to_string(),
            status: "available".to_string(),
        }],
        access_token: Some(access_token.clone()),
    };

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/polls/{}/participants/{}/availability",
                    poll_id, participant_id
                ))
                .header("Content-Type", "application/json")
                .header("X-Forwarded-For", "127.0.0.1") // Valid IP
                .body(Body::from(serde_json::to_string(&update_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // 4. Update Again (Upsert check)
    let update_payload_2 = UpdateAvailabilityRequest {
        availability: vec![AvailabilityEntry {
            date: "2023-10-10".to_string(),
            time_slot: "19:00".to_string(), // Changed time slot
            status: "tentative".to_string(),
        }],
        access_token: Some(access_token.clone()),
    };

    let response_2 = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/polls/{}/participants/{}/availability",
                    poll_id, participant_id
                ))
                .header("Content-Type", "application/json")
                .header("X-Forwarded-For", "127.0.0.1")
                .body(Body::from(
                    serde_json::to_string(&update_payload_2).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response_2.status(), StatusCode::OK);

    // Verify DB state (Should only have the latest entry)
    let entries: Vec<(String,)> =
        sqlx::query_as("SELECT status FROM availability WHERE participant_id = ?")
            .bind(&participant_id)
            .fetch_all(&pool)
            .await
            .unwrap();

    // update_availability clears previous entries for that user/poll tuple
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].0, "tentative");

    // 5. Rate Limit Check
    // We already did 2 requests. Limit is 5 per minute.
    // Let's send 4 more. The 4th (Total 6th) should fail.

    for i in 0..4 {
        let res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!(
                        "/api/polls/{}/participants/{}/availability",
                        poll_id, participant_id
                    ))
                    .header("Content-Type", "application/json")
                    .header("X-Forwarded-For", "127.0.0.1")
                    .body(Body::from(serde_json::to_string(&update_payload).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        if i == 3 {
            assert_eq!(
                res.status(),
                StatusCode::TOO_MANY_REQUESTS,
                "Should be rate limited on 6th request"
            );
        } else {
            assert_eq!(
                res.status(),
                StatusCode::OK,
                "Request {} should succeed",
                i + 3
            );
        }
    }
}
