// GDPR Compliance Handlers
// Gestione conformità GDPR per diritti degli utenti

use crate::{auth::AuthUser, db::DbPool, models::*};
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};
use chrono::Utc;
use serde_json::json;

// ============================================================================
// GET CONSENT PREFERENCES
// ============================================================================

pub async fn get_consent(
    State(pool): State<DbPool>,
    auth_user: AuthUser,
) -> Result<Json<ConsentPreferences>, (StatusCode, String)> {
    let user = auth_user.0;

    // Query user consent preferences
    let result: Option<(bool, bool, Option<i64>)> = sqlx::query_as(
        "SELECT consent_marketing, consent_analytics, privacy_policy_accepted_at FROM users WHERE id = ?",
    )
    .bind(&user.id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    match result {
        Some((marketing, analytics, privacy_accepted)) => Ok(Json(ConsentPreferences {
            consent_marketing: marketing,
            consent_analytics: analytics,
            privacy_policy_accepted: privacy_accepted.is_some(),
        })),
        None => Err((StatusCode::NOT_FOUND, "User not found".to_string())),
    }
}

// ============================================================================
// UPDATE CONSENT PREFERENCES
// ============================================================================

pub async fn update_consent(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    auth_user: AuthUser,
    Json(payload): Json<UpdateConsentRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user = auth_user.0;
    let now = Utc::now().timestamp();

    // Extract IP and User-Agent for audit
    let ip_address = headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .or_else(|| headers.get("x-real-ip").and_then(|v| v.to_str().ok()))
        .map(|s| s.to_string());

    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    // Update marketing consent if provided
    if let Some(marketing) = payload.consent_marketing {
        sqlx::query("UPDATE users SET consent_marketing = ? WHERE id = ?")
            .bind(marketing)
            .bind(&user.id)
            .execute(&pool)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to update marketing consent: {}", e),
                )
            })?;

        // Log consent change
        log_consent_change(
            &pool,
            &user.id,
            "marketing",
            marketing,
            &ip_address,
            &user_agent,
        )
        .await?;
    }

    // Update analytics consent if provided
    if let Some(analytics) = payload.consent_analytics {
        sqlx::query("UPDATE users SET consent_analytics = ? WHERE id = ?")
            .bind(analytics)
            .bind(&user.id)
            .execute(&pool)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to update analytics consent: {}", e),
                )
            })?;

        // Log consent change
        log_consent_change(
            &pool,
            &user.id,
            "analytics",
            analytics,
            &ip_address,
            &user_agent,
        )
        .await?;
    }

    // Update privacy policy acceptance if provided
    if let Some(accept) = payload.accept_privacy_policy {
        if accept {
            sqlx::query("UPDATE users SET privacy_policy_accepted_at = ? WHERE id = ?")
                .bind(now)
                .bind(&user.id)
                .execute(&pool)
                .await
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Failed to update privacy policy acceptance: {}", e),
                    )
                })?;

            // Log consent change
            log_consent_change(
                &pool,
                &user.id,
                "privacy_policy",
                true,
                &ip_address,
                &user_agent,
            )
            .await?;
        }
    }

    Ok(Json(json!({
        "success": true,
        "message": "Preferenze di consenso aggiornate"
    })))
}

// Helper to log consent changes
async fn log_consent_change(
    pool: &DbPool,
    user_id: &str,
    consent_type: &str,
    consented: bool,
    ip_address: &Option<String>,
    user_agent: &Option<String>,
) -> Result<(), (StatusCode, String)> {
    let now = Utc::now().timestamp();

    sqlx::query(
        "INSERT INTO consent_records (user_id, consent_type, consented, ip_address, user_agent, timestamp) VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(user_id)
    .bind(consent_type)
    .bind(consented)
    .bind(ip_address)
    .bind(user_agent)
    .bind(now)
    .execute(pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to log consent change: {}", e),
        )
    })?;

    Ok(())
}

// ============================================================================
// EXPORT USER DATA
// ============================================================================

pub async fn export_data(
    State(pool): State<DbPool>,
    auth_user: AuthUser,
) -> Result<Json<UserDataExport>, (StatusCode, String)> {
    let user = auth_user.0;
    let now = Utc::now();

    // Get user details with consent
    let user_details: Option<(String, String, String, String, i64, bool, bool)> = sqlx::query_as(
        "SELECT id, email, name, role, created_at, consent_marketing, consent_analytics FROM users WHERE id = ?",
    )
    .bind(&user.id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    let (id, email, name, role, created_at, marketing, analytics) =
        user_details.ok_or((StatusCode::NOT_FOUND, "User not found".to_string()))?;

    let user_export = UserPublicExport {
        id,
        email,
        name,
        role,
        created_at,
        consent_marketing: marketing,
        consent_analytics: analytics,
    };

    // Get consent history
    let consent_history: Vec<ConsentRecord> = sqlx::query_as(
        "SELECT id, user_id, consent_type, consented, ip_address, user_agent, timestamp FROM consent_records WHERE user_id = ? ORDER BY timestamp DESC",
    )
    .bind(&user.id)
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to fetch consent history: {}", e),
        )
    })?;

    // Get activities
    let activities: Vec<Activity> = sqlx::query_as(
        "SELECT id, activity_type, user_id, user_name, poll_id, poll_name, message, timestamp FROM activities WHERE user_id = ? ORDER BY timestamp DESC",
    )
    .bind(&user.id)
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to fetch activities: {}", e),
        )
    })?;

    // Get poll participation
    let poll_participation: Vec<PollParticipation> = sqlx::query_as(
        r#"
        SELECT p.poll_id, polls.title as poll_title, p.name as participant_name, NULL as joined_at
        FROM participants p
        JOIN polls ON p.poll_id = polls.id
        WHERE p.user_id = ?
        "#,
    )
    .bind(&user.id)
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to fetch poll participation: {}", e),
        )
    })?;

    // Get availability records
    let availability_records: Vec<Availability> = sqlx::query_as(
        r#"
        SELECT a.id, a.poll_id, a.participant_id, a.date, a.time_slot, a.status
        FROM availability a
        JOIN participants p ON a.participant_id = p.id
        WHERE p.user_id = ?
        ORDER BY a.date DESC
        "#,
    )
    .bind(&user.id)
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to fetch availability records: {}", e),
        )
    })?;

    // Log the export request in audit
    sqlx::query(
        "INSERT INTO audit_log (user_id, action, resource, timestamp, ip_address, success, details) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&user.id)
    .bind("data_export")
    .bind("user_data")
    .bind(now.timestamp())
    .bind::<Option<String>>(None)
    .bind(true)
    .bind("GDPR data export requested")
    .execute(&pool)
    .await
    .ok(); // Don't fail if audit log fails

    Ok(Json(UserDataExport {
        user: user_export,
        consent_history,
        activities,
        poll_participation,
        availability_records,
        export_date: now.to_rfc3339(),
        gdpr_notice: "Questo export contiene tutti i dati personali memorizzati in conformità con il GDPR Art. 20 (Diritto alla portabilità dei dati). Per domande, contatta privacy@cronachednd.it".to_string(),
    }))
}

// ============================================================================
// DELETE ACCOUNT WITH CONFIRMATION
// ============================================================================

pub async fn delete_account_confirmed(
    State(pool): State<DbPool>,
    auth_user: AuthUser,
    Json(payload): Json<DeleteAccountRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    let user = auth_user.0;

    // Verify confirmation text
    if payload.confirmation != "ELIMINA" && payload.confirmation != "DELETE" {
        return Err((
            StatusCode::BAD_REQUEST,
            "Conferma non valida. Scrivi 'ELIMINA' per confermare.".to_string(),
        ));
    }

    // Verify password
    let stored_hash: Option<String> =
        sqlx::query_scalar("SELECT password_hash FROM users WHERE id = ?")
            .bind(&user.id)
            .fetch_optional(&pool)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Database error: {}", e),
                )
            })?;

    let hash = stored_hash.ok_or((StatusCode::NOT_FOUND, "User not found".to_string()))?;

    let password_valid = bcrypt::verify(&payload.password, &hash).unwrap_or(false);
    if !password_valid {
        return Err((
            StatusCode::UNAUTHORIZED,
            "Password non corretta".to_string(),
        ));
    }

    // Log deletion in audit log before deleting
    let now = Utc::now().timestamp();
    sqlx::query(
        "INSERT INTO audit_log (user_id, action, resource, timestamp, success, details) VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&user.id)
    .bind("account_deletion")
    .bind("user")
    .bind(now)
    .bind(true)
    .bind(format!("User {} requested account deletion", user.email))
    .execute(&pool)
    .await
    .ok();

    // Delete user's sessions
    sqlx::query("DELETE FROM user_sessions WHERE user_id = ?")
        .bind(&user.id)
        .execute(&pool)
        .await
        .ok();

    // Delete user's availability entries
    sqlx::query("DELETE FROM availability WHERE participant_id IN (SELECT id FROM participants WHERE user_id = ?)")
        .bind(&user.id)
        .execute(&pool)
        .await
        .ok();

    // Delete user's participant entries
    sqlx::query("DELETE FROM participants WHERE user_id = ?")
        .bind(&user.id)
        .execute(&pool)
        .await
        .ok();

    // Delete user's activities
    sqlx::query("DELETE FROM activities WHERE user_id = ?")
        .bind(&user.id)
        .execute(&pool)
        .await
        .ok();

    // Delete consent records (keep for audit - anonymize instead)
    sqlx::query("UPDATE consent_records SET user_id = 'DELETED_USER' WHERE user_id = ?")
        .bind(&user.id)
        .execute(&pool)
        .await
        .ok();

    // Finally, delete the user
    let result = sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(&user.id)
        .execute(&pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to delete account: {}", e),
            )
        })?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "User not found".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}
