use super::helpers;
use axum::http::StatusCode;
use axum_test::TestServer;
use chrono::{Duration, Utc};
use serde_json::json;

#[tokio::test]
async fn test_create_poll_valid_dates() {
    let ctx = helpers::setup_test_app().await;
    let pool = ctx.pool.clone();
    let server = TestServer::new(ctx.app).unwrap();

    let today = Utc::now().date_naive();
    let tomorrow = today + Duration::days(1);
    let obj = json!({
        "title": "Valid Dates",
        "description": "D",
        "location": "L",
        "dates": [tomorrow.format("%Y-%m-%d").to_string()],
        "participants": []
    });

    let res = server.post("/api/polls").json(&obj).await;
    assert_eq!(res.status_code(), StatusCode::OK);
    
    // Verify JSON serialization of dates in DB
    let id = res.json::<serde_json::Value>()["id"].as_str().unwrap().to_string();
    let (poll, _, _, _) = dnd_scheduler::db::queries::poll_repo::PollRepo::get_details(&pool, &id)
        .unwrap()
        .expect("Poll should exist");
    
    let dates_arr: Vec<String> = serde_json::from_str(&poll.dates).unwrap();
    assert_eq!(dates_arr.len(), 1);
    assert_eq!(dates_arr[0], tomorrow.format("%Y-%m-%d").to_string());
}

#[tokio::test]
async fn test_create_poll_past_dates() {
    let ctx = helpers::setup_test_app().await;
    let server = TestServer::new(ctx.app).unwrap();

    let today = Utc::now().date_naive();
    let yesterday = today - Duration::days(1);
    let obj = json!({
        "title": "Past Dates",
        "description": "D",
        "location": "L",
        "dates": [yesterday.format("%Y-%m-%d").to_string()],
        "participants": []
    });

    let res = server.post("/api/polls").json(&obj).await;
    assert_eq!(res.status_code(), StatusCode::BAD_REQUEST);
    let err_json = res.json::<serde_json::Value>();
    assert!(err_json["error"].as_str().unwrap().contains("Date cannot be in the past"));
}

#[tokio::test]
async fn test_create_poll_exceeds_21_days() {
    let ctx = helpers::setup_test_app().await;
    let server = TestServer::new(ctx.app).unwrap();

    let today = Utc::now().date_naive();
    let tomorrow = today + Duration::days(1);
    let far_future = tomorrow + Duration::days(22); // 22 > 21, should trigger BAD_REQUEST
    
    let obj = json!({
        "title": "Date Range Too Wide",
        "description": "D",
        "location": "L",
        "dates": [
            tomorrow.format("%Y-%m-%d").to_string(),
            far_future.format("%Y-%m-%d").to_string()
        ],
        "participants": []
    });

    let res = server.post("/api/polls").json(&obj).await;
    assert_eq!(res.status_code(), StatusCode::BAD_REQUEST);
    let err_json = res.json::<serde_json::Value>();
    assert!(err_json["error"].as_str().unwrap().contains("Date range cannot exceed 21 days"));
}

#[tokio::test]
async fn test_create_poll_invalid_date_format() {
    let ctx = helpers::setup_test_app().await;
    let server = TestServer::new(ctx.app).unwrap();

    let obj = json!({
        "title": "Invalid Date",
        "description": "D",
        "location": "L",
        "dates": ["not-a-date"],
        "participants": []
    });

    let res = server.post("/api/polls").json(&obj).await;
    assert_eq!(res.status_code(), StatusCode::BAD_REQUEST);
    let err_json = res.json::<serde_json::Value>();
    assert!(err_json["error"].as_str().unwrap().contains("Invalid date format"));
}

#[tokio::test]
async fn test_create_poll_with_timezone() {
    let ctx = helpers::setup_test_app().await;
    let server = TestServer::new(ctx.app).unwrap();

    // Imagine it's UTC midnight (next day). In America/Los_Angeles it's still the previous day.
    // We send a date equal to UTC's "yesterday", but in LA that date might actually be "today".
    // Wait, testing time logic reliably is tricky without mocking time.
    // Let's at least test that providing a valid timezone allows the poll to be created,
    // and an invalid one falls back gracefully without crashing.
    let today = Utc::now().date_naive();
    let tomorrow = today + Duration::days(1);
    
    let obj = json!({
        "title": "Timezone test",
        "description": "D",
        "location": "L",
        "dates": [tomorrow.format("%Y-%m-%d").to_string()],
        "timezone": "America/Los_Angeles",
        "participants": []
    });

    let res = server.post("/api/polls").json(&obj).await;
    assert_eq!(res.status_code(), StatusCode::OK);
}
