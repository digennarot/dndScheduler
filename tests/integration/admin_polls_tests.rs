use crate::helpers;
use axum::http::StatusCode;
use serde_json::json;

#[tokio::test]
async fn test_admin_can_list_polls_enriched() {
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_test_writer()
        .try_init();

    let (app, pool) = helpers::setup_test_app().await;
    let server = axum_test::TestServer::new(app).unwrap();

    // 1. Create Organizer & Poll
    let (_org_id, org_token) =
        helpers::create_test_user_with_session(&pool, "org@test.com", "pass", "player").await;

    let future_date = (chrono::Utc::now() + chrono::Duration::days(30))
        .format("%Y-%m-%d")
        .to_string();

    let create_payload = json!({
        "title": "Admin Test Poll",
        "description": "Desc",
        "location": "Loc",
        "dates": [future_date],
        "participants": []
    });

    let res = server
        .post("/api/polls")
        .add_header("X-Forwarded-For", "127.0.0.1")
        .add_header("Authorization", format!("Bearer {}", org_token))
        .json(&create_payload)
        .await;
    assert_eq!(res.status_code(), StatusCode::OK);
    let poll_json: serde_json::Value = res.json();
    let poll_id = poll_json["id"].as_str().unwrap().to_string();

    // 2. Create Admin & Session
    let admin_id = uuid::Uuid::new_v4().to_string();
    let admin_token = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp();

    sqlx::query("INSERT INTO users (id, email, password_hash, name, role, created_at) VALUES (?, ?, 'hash', 'Admin', 'admin', ?)")
        .bind(&admin_id)
        .bind("admin@test.com")
        .bind(now)
        .execute(&pool).await.unwrap();

    sqlx::query("INSERT INTO user_sessions (id, user_id, token, expires_at, created_at) VALUES (?, ?, ?, ?, ?)")
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(&admin_id)
        .bind(&admin_token)
        .bind(now + 3600)
        .bind(now)
        .execute(&pool).await.unwrap();

    // 3. Fetch Admin Polls
    let response = server
        .get("/api/admin/polls")
        .add_header("X-Forwarded-For", "127.0.0.1")
        .add_header("Cookie", format!("admin_session={}", admin_token.clone()))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let polls: serde_json::Value = response.json();
    let polls_array = polls.as_array().expect("Expected array");

    let poll_entry = polls_array
        .iter()
        .find(|p| p["id"] == poll_id)
        .expect("Poll not found");

    assert_eq!(poll_entry["title"], "Admin Test Poll");
}

#[tokio::test]
async fn test_admin_can_finalize_poll() {
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_test_writer()
        .try_init();

    let (app, pool) = helpers::setup_test_app().await;
    let server = axum_test::TestServer::new(app).unwrap();

    // 1. Create Poll via API
    let (_org_id, org_token) =
        helpers::create_test_user_with_session(&pool, "org2@test.com", "pass", "player").await;
    let future_date = (chrono::Utc::now() + chrono::Duration::days(30))
        .format("%Y-%m-%d")
        .to_string();

    let create_payload = json!({
        "title": "To Be Finalized",
        "description": "D",
        "location": "L",
        "dates": [future_date],
        "participants": []
    });
    let res = server
        .post("/api/polls")
        .add_header("X-Forwarded-For", "127.0.0.1")
        .add_header("Authorization", format!("Bearer {}", org_token))
        .json(&create_payload)
        .await;
    let poll_id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // 2. Create Admin & Session
    let admin_id = uuid::Uuid::new_v4().to_string();
    let admin_token = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp();
    sqlx::query("INSERT INTO users (id, email, password_hash, name, role, created_at) VALUES (?, ?, 'hash', 'Admin', 'admin', ?)")
        .bind(&admin_id).bind("admin2@test.com").bind(now).execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO user_sessions (id, user_id, token, expires_at, created_at) VALUES (?, ?, ?, ?, ?)")
        .bind(uuid::Uuid::new_v4().to_string()).bind(&admin_id).bind(&admin_token).bind(now + 3600).bind(now).execute(&pool).await.unwrap();

    // 3. Finalize
    let response = server
        .put(&format!("/api/polls/{}/finalize", poll_id))
        .add_header("X-Forwarded-For", "127.0.0.1")
        .add_header("Cookie", format!("admin_session={}", admin_token.clone()))
        .json(&json!({
            "finalized_time": "Now",
            "notes": "Admin Closed"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);

    // 4. Verify status in Admin List
    let list_res = server
        .get("/api/admin/polls")
        .add_header("X-Forwarded-For", "127.0.0.1")
        .add_header("Cookie", format!("admin_session={}", admin_token))
        .await;

    let polls: serde_json::Value = list_res.json();
    let entry = polls
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["id"] == poll_id)
        .unwrap();
    assert_eq!(entry["status"], "finalized");
}

#[tokio::test]
async fn test_admin_can_delete_poll() {
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_test_writer()
        .try_init();

    let (app, pool) = helpers::setup_test_app().await;
    let server = axum_test::TestServer::new(app).unwrap();

    // 1. Create Poll via API
    let (_org_id, org_token) =
        helpers::create_test_user_with_session(&pool, "org3@test.com", "pass", "player").await;

    let future_date = (chrono::Utc::now() + chrono::Duration::days(30))
        .format("%Y-%m-%d")
        .to_string();

    let create_payload = json!({
        "title": "To Be Deleted",
        "description": "D",
        "location": "L",
        "dates": [future_date],
        "participants": []
    });
    let res = server
        .post("/api/polls")
        .add_header("X-Forwarded-For", "127.0.0.1")
        .add_header("Authorization", format!("Bearer {}", org_token))
        .json(&create_payload)
        .await;

    let poll_id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // 2. Create Admin & Session
    let admin_id = uuid::Uuid::new_v4().to_string();
    let admin_token = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp();
    sqlx::query("INSERT INTO users (id, email, password_hash, name, role, created_at) VALUES (?, ?, 'hash', 'Admin', 'admin', ?)")
        .bind(&admin_id).bind("admin3@test.com").bind(now).execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO user_sessions (id, user_id, token, expires_at, created_at) VALUES (?, ?, ?, ?, ?)")
        .bind(uuid::Uuid::new_v4().to_string()).bind(&admin_id).bind(&admin_token).bind(now + 3600).bind(now).execute(&pool).await.unwrap();

    // 3. Delete Poll
    let response = server
        .delete(&format!("/api/polls/{}", poll_id))
        .add_header("X-Forwarded-For", "127.0.0.1")
        .add_header("Cookie", format!("admin_session={}", admin_token.clone()))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);

    // 4. Verify Gone
    let list_res = server
        .get("/api/admin/polls")
        .add_header("X-Forwarded-For", "127.0.0.1")
        .add_header("Cookie", format!("admin_session={}", admin_token))
        .await;

    let polls: serde_json::Value = list_res.json();
    let entry = polls
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["id"] == poll_id);

    assert!(entry.is_none());
}
