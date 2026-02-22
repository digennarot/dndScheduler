// GDPR Compliance Handlers
// Gestione conformità GDPR per diritti degli utenti

use crate::{auth::AuthUser, core::models::*, db::DbPool};
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

    let db_user = crate::db::queries::user_repo::UserRepo::find_by_id(&pool, &user.id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))?
        .ok_or((StatusCode::NOT_FOUND, "User not found".to_string()))?;

    Ok(Json(ConsentPreferences {
        consent_marketing: db_user.consent_marketing,
        consent_analytics: db_user.consent_analytics,
        privacy_policy_accepted: db_user.privacy_policy_accepted_at.is_some(),
    }))
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

    let mut db_user = crate::db::queries::user_repo::UserRepo::find_by_id(&pool, &user.id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))?
        .ok_or((StatusCode::NOT_FOUND, "User not found".to_string()))?;
    let mut user_changed = false;

    // Update marketing consent if provided
    if let Some(marketing) = payload.consent_marketing {
        db_user.consent_marketing = marketing;
        user_changed = true;
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
        db_user.consent_analytics = analytics;
        user_changed = true;
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
            db_user.privacy_policy_accepted_at = Some(now);
            user_changed = true;
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

    if user_changed {
        crate::db::queries::user_repo::UserRepo::create_or_update(&pool, &db_user)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to update user: {}", e)))?;
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
    crate::db::queries::gdpr_repo::GdprRepo::log_consent_change(
        pool, user_id, consent_type, consented, ip_address, user_agent, now
    ).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to log consent change: {}", e)))?;

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
    let details = crate::db::queries::user_repo::UserRepo::find_by_id(&pool, &user.id)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?
        .ok_or((StatusCode::NOT_FOUND, "User not found".to_string()))?;

    let user_export = UserPublicExport {
        id: details.id,
        email: details.email,
        name: details.name,
        role: details.role,
        phone: details.phone,
        created_at: details.created_at,
        consent_marketing: details.consent_marketing,
        consent_analytics: details.consent_analytics,
    };

    // Get consent history
    let consent_history = crate::db::queries::gdpr_repo::GdprRepo::get_consent_history(&pool, &user.id)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to fetch consent history: {}", e),
            )
        })?;

    // Get activities
    let activities = crate::db::queries::activity_repo::ActivityRepo::get_user_activities(&pool, &user.id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to fetch activities: {}", e)))?;

    // Get poll participation
    let poll_participation = crate::db::queries::poll_repo::PollRepo::get_user_participation(&pool, &user.id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to fetch poll participation: {}", e)))?;

    // Get availability records
    let availability_records = crate::db::queries::poll_repo::PollRepo::get_user_availability(&pool, &user.id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to fetch availability records: {}", e)))?;

    // Log the export request in audit
    crate::security::audit::log_audit(
        &pool,
        Some(user.id.clone()),
        "data_export",
        Some("user_data".to_string()),
        true,
        Some("GDPR data export requested".to_string()),
        None,
    ).await;

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
    let db_user = crate::db::queries::user_repo::UserRepo::find_by_id(&pool, &user.id)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?
        .ok_or((StatusCode::NOT_FOUND, "User not found".to_string()))?;
    
    let hash = db_user.password_hash;

    let password_valid = bcrypt::verify(&payload.password, &hash).unwrap_or(false);
    if !password_valid {
        return Err((
            StatusCode::UNAUTHORIZED,
            "Password non corretta".to_string(),
        ));
    }

    // Log deletion in audit log before deleting
    crate::security::audit::log_audit(
        &pool,
        Some(user.id.clone()),
        "account_deletion",
        Some("user".to_string()),
        true,
        Some(format!("User {} requested account deletion", user.email)),
        None,
    ).await;

    // Delete user's activities
    let _ = crate::db::queries::activity_repo::ActivityRepo::delete_user_activities(&pool, &user.id);

    // Anonymize consent records (keep for audit)
    let _ = crate::db::queries::gdpr_repo::GdprRepo::anonymize_consent_records(&pool, &user.id);

    // Delete user data (sessions, etc.)
    let _ = crate::db::queries::user_repo::UserRepo::delete_user_all_data(&pool, &user.id);

    // Finally, delete the user
    crate::db::queries::user_repo::UserRepo::delete(&pool, &user.id).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to delete account: {}", e),
        )
    })?;

    Ok(StatusCode::NO_CONTENT)
}
