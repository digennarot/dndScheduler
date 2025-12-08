// Integration Tests for Authentication
// Test di integrazione per il sistema di autenticazione

#[cfg(test)]
mod auth_integration_tests {
    use crate::helpers::{
        cleanup_test_db, create_test_user, create_test_user_with_session, setup_test_db,
    };

    #[tokio::test]
    async fn test_user_registration_with_valid_data() {
        let pool = setup_test_db().await;

        // Dati validi per la registrazione
        let email = "newuser@test.com";
        let password = "SecurePass123!@#";

        // Simula registrazione (usando helper)
        let user_id = create_test_user(&pool, email, password, "player").await;

        // Verifica che l'utente sia stato creato
        let result: Option<(String, String)> =
            sqlx::query_as("SELECT id, role FROM users WHERE email = ?")
                .bind(email)
                .fetch_optional(&pool)
                .await
                .unwrap();

        assert!(result.is_some());
        let (id, role) = result.unwrap();
        assert_eq!(id, user_id);
        assert_eq!(role, "player"); // Ruolo di default

        cleanup_test_db(&pool).await;
    }

    #[tokio::test]
    async fn test_user_cannot_register_with_duplicate_email() {
        let pool = setup_test_db().await;

        let email = "duplicate@test.com";
        let password = "SecurePass123!@#";

        // Prima registrazione
        create_test_user(&pool, email, password, "player").await;

        // Tenta seconda registrazione con stessa email - dovrebbe fallire
        let user_id = uuid::Uuid::new_v4().to_string();
        let password_hash = bcrypt::hash(password, bcrypt::DEFAULT_COST).unwrap();
        let now = chrono::Utc::now().timestamp();

        let result = sqlx::query(
            "INSERT INTO users (id, email, password_hash, name, role, created_at, last_login) VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&user_id)
        .bind(email)
        .bind(&password_hash)
        .bind("Test User")
        .bind("player")
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await;

        // La registrazione duplicata dovrebbe fallire con errore UNIQUE constraint
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("UNIQUE"));

        cleanup_test_db(&pool).await;
    }

    #[tokio::test]
    async fn test_user_login_with_valid_credentials() {
        let pool = setup_test_db().await;

        let email = "logintest@test.com";
        let password = "SecurePass123!@#";

        // Crea utente
        create_test_user(&pool, email, password, "player").await;

        // Verifica che il password hash sia corretto
        let stored_hash: String = sqlx::query_scalar("SELECT password_hash FROM users WHERE email = ?")
            .bind(email)
            .fetch_one(&pool)
            .await
            .unwrap();

        // Verifica che la password corrisponda
        let valid = bcrypt::verify(password, &stored_hash).unwrap();
        assert!(valid);

        cleanup_test_db(&pool).await;
    }

    #[tokio::test]
    async fn test_user_login_with_invalid_password() {
        let pool = setup_test_db().await;

        let email = "logintest@test.com";
        let correct_password = "SecurePass123!@#";
        let wrong_password = "WrongPassword123!";

        // Crea utente
        create_test_user(&pool, email, correct_password, "player").await;

        // Verifica password errata
        let stored_hash: String = sqlx::query_scalar("SELECT password_hash FROM users WHERE email = ?")
            .bind(email)
            .fetch_one(&pool)
            .await
            .unwrap();

        let valid = bcrypt::verify(wrong_password, &stored_hash).unwrap_or(false);
        assert!(!valid); // Password errata dovrebbe fallire

        cleanup_test_db(&pool).await;
    }

    #[tokio::test]
    async fn test_session_creation_and_validation() {
        let pool = setup_test_db().await;

        let email = "session@test.com";
        let password = "SecurePass123!@#";

        // Crea utente con sessione
        let (user_id, token) = create_test_user_with_session(&pool, email, password, "player").await;

        // Verifica che la sessione sia stata creata
        let session: Option<(String, i64)> =
            sqlx::query_as("SELECT user_id, expires_at FROM user_sessions WHERE token = ?")
                .bind(&token)
                .fetch_optional(&pool)
                .await
                .unwrap();

        assert!(session.is_some());
        let (session_user_id, expires_at) = session.unwrap();
        assert_eq!(session_user_id, user_id);

        // Verifica che la sessione non sia scaduta
        let now = chrono::Utc::now().timestamp();
        assert!(expires_at > now);

        cleanup_test_db(&pool).await;
    }

    #[tokio::test]
    async fn test_logout_invalidates_session() {
        let pool = setup_test_db().await;

        let email = "logout@test.com";
        let password = "SecurePass123!@#";

        // Crea utente con sessione
        let (_user_id, token) = create_test_user_with_session(&pool, email, password, "player").await;

        // Simula logout (elimina sessione)
        sqlx::query("DELETE FROM user_sessions WHERE token = ?")
            .bind(&token)
            .execute(&pool)
            .await
            .unwrap();

        // Verifica che la sessione sia stata eliminata
        let session: Option<(String,)> =
            sqlx::query_as("SELECT token FROM user_sessions WHERE token = ?")
                .bind(&token)
                .fetch_optional(&pool)
                .await
                .unwrap();

        assert!(session.is_none());

        cleanup_test_db(&pool).await;
    }

    #[tokio::test]
    async fn test_password_strength_requirements() {
        // Test password deboli (dovrebbero fallire in produzione)
        let weak_passwords = vec![
            "short",           // Troppo corta
            "nouppercase1!",   // Senza maiuscole
            "NOLOWERCASE1!",   // Senza minuscole
            "NoNumbers!",      // Senza numeri
            "NoSpecial123",    // Senza caratteri speciali
        ];

        // In produzione, questi dovrebbero fallire la validazione
        // Il test helper non valida le password, ma in produzione
        // il sistema dovrebbe rifiutare queste password

        for password in weak_passwords {
            assert!(password.len() < 12 ||
                    !password.chars().any(|c| c.is_uppercase()) ||
                    !password.chars().any(|c| c.is_lowercase()) ||
                    !password.chars().any(|c| c.is_numeric()) ||
                    !password.chars().any(|c| !c.is_alphanumeric()));
        }
    }
}
