// Example Tests Using Multiple Test Users
// Demonstrates how to use the new test user helpers

#[cfg(test)]
mod test_users_example {
    use crate::helpers::*;

    #[tokio::test]
    async fn test_create_default_users() {
        let pool = setup_test_db().await;

        // Create a complete set of test users
        let users = create_default_test_users(&pool).await;

        println!("Admin: {} - {}", users.admin.email, users.admin.id);
        println!("DM: {} - {}", users.dm.email, users.dm.id);
        println!("Player 1: {} - {}", users.player1.email, users.player1.id);
        println!("Player 2: {} - {}", users.player2.email, users.player2.id);
        println!("Player 3: {} - {}", users.player3.email, users.player3.id);

        // Verify all users were created
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(count.0, 5);
    }

    #[tokio::test]
    async fn test_create_custom_users() {
        let pool = setup_test_db().await;

        // Create custom test users
        let configs = vec![
            TestUserConfig::player("gandalf@middleearth.com"),
            TestUserConfig::player("frodo@shire.com"),
            TestUserConfig::dm("sauron@mordor.com"),
        ];

        let users = create_test_users(&pool, configs).await;

        assert_eq!(users.len(), 3);
    }

    #[tokio::test]
    async fn test_custom_user_config() {
        let pool = setup_test_db().await;

        // Create a user with custom configuration
        let custom_config = TestUserConfig::new(
            "custom@test.com",
            "super_secure_password",
            "Custom User",
            "player",
        );

        let (user_id, token) = create_test_user_with_session(
            &pool,
            &custom_config.email,
            &custom_config.password,
            &custom_config.role,
        )
        .await;

        assert!(!user_id.is_empty());
        assert!(!token.is_empty());
    }

    #[tokio::test]
    async fn test_multiple_roles() {
        let pool = setup_test_db().await;

        let users = create_default_test_users(&pool).await;

        // Verify different roles
        assert_eq!(users.admin.role, "admin");
        assert_eq!(users.dm.role, "dm");
        assert_eq!(users.player1.role, "player");
        assert_eq!(users.player2.role, "player");
        assert_eq!(users.player3.role, "player");
    }

    #[tokio::test]
    async fn test_user_sessions() {
        let pool = setup_test_db().await;

        let users = create_default_test_users(&pool).await;

        // All users should have valid session tokens
        assert!(!users.admin.token.is_empty());
        assert!(!users.dm.token.is_empty());
        assert!(!users.player1.token.is_empty());
        assert!(!users.player2.token.is_empty());
        assert!(!users.player3.token.is_empty());

        // Verify sessions exist in database
        let session_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM user_sessions")
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(session_count.0, 5);
    }
}
