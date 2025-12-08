use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Poll {
    pub id: String,
    pub title: String,
    pub description: String,
    pub location: String,
    pub created_at: i64,    // Unix timestamp
    pub dates: String,      // JSON string of dates
    pub time_range: String, // JSON string or simple string
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Participant {
    pub id: String,
    pub poll_id: String,
    pub name: String,
    pub email: Option<String>,
    pub access_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Availability {
    pub id: Option<i64>,
    pub poll_id: String,
    pub participant_id: String,
    pub date: String,
    pub time_slot: String,
    pub status: String, // "available", "tentative", "busy"
}

// Request structs
#[derive(Debug, Deserialize)]
pub struct CreatePollRequest {
    pub title: String,
    pub description: String,
    pub location: String,
    pub dates: Vec<String>,
    #[serde(rename = "timeRange")]
    pub time_range: Option<String>, // Legacy: camelCase from JS, optional for backward compatibility
    #[serde(rename = "timePreferences")]
    pub time_preferences: Option<serde_json::Value>, // New: per-day time preferences as JSON object
    pub participants: Vec<String>, // List of emails
}

#[derive(Debug, Deserialize)]
pub struct JoinPollRequest {
    pub name: String,
    pub email: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Admin {
    pub id: String,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub email: Option<String>,
    pub role: String,
    pub created_at: i64,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct GoogleLoginRequest {
    #[allow(dead_code)]
    pub token: String,
    pub email: String,
    pub name: String,
    #[allow(dead_code)]
    pub picture: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: Admin,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAvailabilityRequest {
    pub availability: Vec<AvailabilityEntry>,
    pub access_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AvailabilityEntry {
    pub date: String,
    #[serde(rename = "timeSlot")]
    pub time_slot: String, // camelCase from JS
    pub status: String,
}

// ============================================================================
// USER AUTHENTICATION MODELS
// ============================================================================

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct User {
    pub id: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub name: String,
    pub role: String, // 'player' or 'dm'
    pub created_at: i64,
    pub last_login: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct UserSession {
    pub id: String,
    pub user_id: String,
    pub token: String,
    pub expires_at: i64,
    pub created_at: i64,
}

#[derive(Debug, Deserialize)]
pub struct UserRegisterRequest {
    pub email: String,
    pub password: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct UserLoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct UserAuthResponse {
    pub token: String,
    pub user: UserPublic,
}

#[derive(Debug, Serialize, Clone)]
pub struct UserPublic {
    pub id: String,
    pub email: String,
    pub name: String,
    pub role: String,
    pub created_at: i64,
}

impl From<User> for UserPublic {
    fn from(user: User) -> Self {
        UserPublic {
            id: user.id,
            email: user.email,
            name: user.name,
            role: user.role,
            created_at: user.created_at,
        }
    }
}
// ============================================================================
// ACTIVITY FEED MODELS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Activity {
    pub id: String,
    pub activity_type: String, // "poll_created", "response_submitted", ecc.
    pub user_id: String,
    pub user_name: String,
    pub poll_id: Option<String>,
    pub poll_name: Option<String>,
    pub message: String,
    pub timestamp: i64, // Unix timestamp
}

impl Activity {
    pub fn new(
        activity_type: &str,
        user_id: String,
        user_name: String,
        poll_id: Option<String>,
        poll_name: Option<String>,
    ) -> Self {
        let message = Self::generate_message(activity_type, &user_name, poll_name.as_deref());

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            activity_type: activity_type.to_string(),
            user_id,
            user_name,
            poll_id,
            poll_name,
            message,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    fn generate_message(activity_type: &str, user_name: &str, poll_name: Option<&str>) -> String {
        match activity_type {
            "poll_created" => {
                format!(
                    "{} ha creato la campagna {}",
                    user_name,
                    poll_name.unwrap_or("Senza nome")
                )
            }
            "response_submitted" => {
                format!(
                    "{} ha indicato la disponibilità per {}",
                    user_name,
                    poll_name.unwrap_or("una sessione")
                )
            }
            "poll_finalized" => {
                format!(
                    "La sessione {} è stata finalizzata",
                    poll_name.unwrap_or("Senza nome")
                )
            }
            "user_joined" => {
                format!("{} si è unito alla piattaforma", user_name)
            }
            "reminder_sent" => {
                format!(
                    "Promemoria inviato per {}",
                    poll_name.unwrap_or("una sessione")
                )
            }
            "poll_edited" => {
                format!(
                    "{} ha modificato la campagna {}",
                    user_name,
                    poll_name.unwrap_or("Senza nome")
                )
            }
            _ => format!("{} - {}", user_name, poll_name.unwrap_or("attività")),
        }
    }
}

// ============================================================================
// REMINDER MODELS
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct WhatsAppReminderRequest {
    pub phone: String,
    pub message: String,
    #[allow(dead_code)]
    pub session_id: String,
}

#[derive(Debug, Deserialize)]
pub struct TelegramReminderRequest {
    pub chat_id: String,
    pub message: String,
    #[allow(dead_code)]
    pub session_id: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct EmailReminderRequest {
    pub user_id: String,
    pub session_id: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct ReminderResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct ReminderConfig {
    pub whatsapp_enabled: bool,
    pub telegram_enabled: bool,
    pub email_enabled: bool,
}

// ============================================================================
// GDPR COMPLIANCE MODELS
// ============================================================================

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct ConsentRecord {
    pub id: Option<i64>,
    pub user_id: String,
    pub consent_type: String,
    pub consented: bool,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConsentPreferences {
    pub consent_marketing: bool,
    pub consent_analytics: bool,
    pub privacy_policy_accepted: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateConsentRequest {
    pub consent_marketing: Option<bool>,
    pub consent_analytics: Option<bool>,
    pub accept_privacy_policy: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct UserDataExport {
    pub user: UserPublicExport,
    pub consent_history: Vec<ConsentRecord>,
    pub activities: Vec<Activity>,
    pub poll_participation: Vec<PollParticipation>,
    pub availability_records: Vec<Availability>,
    pub export_date: String,
    pub gdpr_notice: String,
}

#[derive(Debug, Serialize)]
pub struct UserPublicExport {
    pub id: String,
    pub email: String,
    pub name: String,
    pub role: String,
    pub created_at: i64,
    pub consent_marketing: bool,
    pub consent_analytics: bool,
}

#[derive(Debug, Serialize, FromRow)]
pub struct PollParticipation {
    pub poll_id: String,
    pub poll_title: String,
    pub participant_name: String,
    pub joined_at: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct DeleteAccountRequest {
    pub password: String,
    pub confirmation: String, // Must be "ELIMINA" or "DELETE"
}
