use super::helpers;
use axum::http::StatusCode;
use serde_json::json;
use axum_test::TestServer;

#[tokio::test]
async fn test_rbac_user_management_by_admin() {
    let ctx = helpers::setup_test_app().await;
    let pool = ctx.pool.clone();
    let server = TestServer::new(ctx.app).unwrap();

    // 1. Create a normal user and an admin
    let (user_id, _) = helpers::create_test_user_with_session(&pool, "player@test.com", "pass", "player").await;
    let (_, admin_token) = helpers::create_test_user_with_session(&pool, "admin@test.com", "pass", "admin").await;

    // 2. Verify initial role is player
    let saved = dnd_scheduler::db::queries::user_repo::UserRepo::find_by_id(&pool, &user_id).unwrap().unwrap();
    assert_eq!(saved.role, "player");

    // 3. Admin promotes user to dm
    let promote_res = server
        .put(&format!("/api/admin/users/{}/role", user_id))
        .add_header("Authorization", format!("Bearer {}", admin_token))
        .json(&json!({ "role": "dm" }))
        .await;
    
    assert_eq!(promote_res.status_code(), StatusCode::OK);

    // 4. Verify DB state
    let updated = dnd_scheduler::db::queries::user_repo::UserRepo::find_by_id(&pool, &user_id).unwrap().unwrap();
    assert_eq!(updated.role, "dm");
}

#[tokio::test]
async fn test_rbac_promotion_denied_to_non_admin() {
    let ctx = helpers::setup_test_app().await;
    let pool = ctx.pool.clone();
    let server = TestServer::new(ctx.app).unwrap();

    // 1. Create two normal users
    let (target_user_id, _) = helpers::create_test_user_with_session(&pool, "target@test.com", "pass", "player").await;
    let (_, player_token) = helpers::create_test_user_with_session(&pool, "hacker@test.com", "pass", "player").await;

    // 2. Player tries to promote target to admin (should fail)
    let promote_res = server
        .put(&format!("/api/admin/users/{}/role", target_user_id))
        .add_header("Authorization", format!("Bearer {}", player_token))
        .json(&json!({ "role": "admin" }))
        .await;
    
    // AdminUser extractor should return Forbidden for non-admin tokens
    assert_eq!(promote_res.status_code(), StatusCode::FORBIDDEN);

    // 3. Verify role hasn't changed
    let unchanged = dnd_scheduler::db::queries::user_repo::UserRepo::find_by_id(&pool, &target_user_id).unwrap().unwrap();
    assert_eq!(unchanged.role, "player");
}
