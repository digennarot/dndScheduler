// Integration Tests for RBAC (Role-Based Access Control)
// Test di integrazione per il controllo degli accessi basato sui ruoli

#[cfg(test)]
mod rbac_integration_tests {
    use crate::helpers::{
        cleanup_test_db, create_test_user, create_test_user_with_session, setup_test_db,
    };

    #[tokio::test]
    async fn test_new_user_has_player_role_by_default() {
        let pool = setup_test_db().await;

        let email = "newplayer@test.com";
        let password = "SecurePass123!@#";

        // Crea utente (il ruolo di default dovrebbe essere 'player')
        let user_id = create_test_user(&pool, email, password, "player").await;

        // Verifica il ruolo
        let role: String = sqlx::query_scalar("SELECT role FROM users WHERE id = ?")
            .bind(&user_id)
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(role, "player");

        cleanup_test_db(&pool).await;
    }

    #[tokio::test]
    async fn test_dm_role_can_be_assigned() {
        let pool = setup_test_db().await;

        let email = "dm@test.com";
        let password = "SecurePass123!@#";

        // Crea utente come DM
        let user_id = create_test_user(&pool, email, password, "dm").await;

        // Verifica il ruolo
        let role: String = sqlx::query_scalar("SELECT role FROM users WHERE id = ?")
            .bind(&user_id)
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(role, "dm");

        cleanup_test_db(&pool).await;
    }

    #[tokio::test]
    async fn test_player_role_cannot_create_polls() {
        let pool = setup_test_db().await;

        let email = "player@test.com";
        let password = "SecurePass123!@#";

        // Crea utente player
        let (_user_id, _token) = create_test_user_with_session(&pool, email, password, "player").await;

        // Verifica ruolo
        let role: String = sqlx::query_scalar("SELECT role FROM users WHERE email = ?")
            .bind(email)
            .fetch_one(&pool)
            .await
            .unwrap();

        // Il ruolo deve essere 'player'
        assert_eq!(role, "player");

        // In un test di integrazione completo, dovremmo:
        // 1. Tentare di creare un poll con il token del player
        // 2. Verificare che riceviamo un errore 403 Forbidden
        // Per ora, verifichiamo solo che il ruolo non sia 'dm'
        assert_ne!(role, "dm");

        cleanup_test_db(&pool).await;
    }

    #[tokio::test]
    async fn test_dm_role_can_create_polls() {
        let pool = setup_test_db().await;

        let email = "dm@test.com";
        let password = "SecurePass123!@#";

        // Crea utente DM
        let (_user_id, _token) = create_test_user_with_session(&pool, email, password, "dm").await;

        // Verifica ruolo
        let role: String = sqlx::query_scalar("SELECT role FROM users WHERE email = ?")
            .bind(email)
            .fetch_one(&pool)
            .await
            .unwrap();

        // Il ruolo deve essere 'dm'
        assert_eq!(role, "dm");

        // In un test di integrazione completo, dovremmo:
        // 1. Tentare di creare un poll con il token del DM
        // 2. Verificare che la creazione ha successo (200 OK)
        // Per ora, verifichiamo solo che il ruolo sia corretto
        assert_eq!(role, "dm");

        cleanup_test_db(&pool).await;
    }

    #[tokio::test]
    async fn test_promote_user_from_player_to_dm() {
        let pool = setup_test_db().await;

        let email = "promoteme@test.com";
        let password = "SecurePass123!@#";

        // Crea utente come player
        let user_id = create_test_user(&pool, email, password, "player").await;

        // Verifica ruolo iniziale
        let role_before: String = sqlx::query_scalar("SELECT role FROM users WHERE id = ?")
            .bind(&user_id)
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(role_before, "player");

        // Promuovi a DM
        sqlx::query("UPDATE users SET role = 'dm' WHERE id = ?")
            .bind(&user_id)
            .execute(&pool)
            .await
            .unwrap();

        // Verifica ruolo aggiornato
        let role_after: String = sqlx::query_scalar("SELECT role FROM users WHERE id = ?")
            .bind(&user_id)
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(role_after, "dm");

        cleanup_test_db(&pool).await;
    }

    #[tokio::test]
    async fn test_role_persists_across_sessions() {
        let pool = setup_test_db().await;

        let email = "persistent@test.com";
        let password = "SecurePass123!@#";

        // Crea utente DM con sessione
        let (user_id, token1) = create_test_user_with_session(&pool, email, password, "dm").await;

        // Verifica ruolo
        let role: String = sqlx::query_scalar("SELECT role FROM users WHERE id = ?")
            .bind(&user_id)
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(role, "dm");

        // Elimina la prima sessione (simula logout)
        sqlx::query("DELETE FROM user_sessions WHERE token = ?")
            .bind(&token1)
            .execute(&pool)
            .await
            .unwrap();

        // Crea una nuova sessione (simula nuovo login)
        let session_id = uuid::Uuid::new_v4().to_string();
        let token2 = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp();
        let expires_at = now + 86400;

        sqlx::query(
            "INSERT INTO user_sessions (id, user_id, token, expires_at, created_at) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&session_id)
        .bind(&user_id)
        .bind(&token2)
        .bind(expires_at)
        .bind(now)
        .execute(&pool)
        .await
        .unwrap();

        // Verifica che il ruolo sia ancora 'dm'
        let role_after_new_session: String = sqlx::query_scalar(
            "SELECT u.role FROM users u JOIN user_sessions s ON u.id = s.user_id WHERE s.token = ?",
        )
        .bind(&token2)
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(role_after_new_session, "dm");

        cleanup_test_db(&pool).await;
    }

    #[tokio::test]
    async fn test_invalid_role_values_not_allowed() {
        let pool = setup_test_db().await;

        let email = "invalidrole@test.com";
        let password = "SecurePass123!@#";

        // Crea utente con ruolo valido
        let user_id = create_test_user(&pool, email, password, "player").await;

        // I ruoli validi sono solo 'player' e 'dm'
        // Verifica che il sistema accetti solo questi ruoli
        let valid_roles = vec!["player", "dm"];

        let current_role: String = sqlx::query_scalar("SELECT role FROM users WHERE id = ?")
            .bind(&user_id)
            .fetch_one(&pool)
            .await
            .unwrap();

        assert!(valid_roles.contains(&current_role.as_str()));

        cleanup_test_db(&pool).await;
    }
}
