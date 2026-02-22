use crate::core::models::CreatePollRequest;
use crate::core::store::RedbEventStore;
use crate::db::queries::poll_repo::PollRepo;
use crate::db::DbPool;
use axum::http::StatusCode;
use chrono::Utc;
use nanoid::nanoid;
use serde_json::{json, Value};
use std::sync::Arc;
use uuid::Uuid;

// Security constants
const MAX_TITLE_LENGTH: usize = 100;
const MAX_DESCRIPTION_LENGTH: usize = 500;
const MAX_LOCATION_LENGTH: usize = 100;
const MAX_EMAIL_LENGTH: usize = 254; // RFC 5321
const MAX_PARTICIPANTS: usize = 100;
const MAX_DATES: usize = 365;

// Helper validations (copied from general.rs to keep logic here)
fn validate_email(email: &str) -> Result<(), String> {
    if email.is_empty() || email.len() > MAX_EMAIL_LENGTH {
        return Err("Invalid email length".to_string());
    }
    if !email.contains('@') || !email.contains('.') {
        return Err("Invalid email format".to_string());
    }
    if email.contains(['<', '>', '"', '\'', '\\', '\0']) {
        return Err("Invalid characters in email".to_string());
    }
    Ok(())
}

fn validate_string_length(s: &str, max_len: usize, field_name: &str) -> Result<(), String> {
    if s.is_empty() {
        return Err(format!("{} cannot be empty", field_name));
    }
    if s.len() > max_len {
        return Err(format!("{} exceeds maximum length of {}", field_name, max_len));
    }
    Ok(())
}

fn sanitize_string(s: &str) -> String {
    let mut escaped = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '&' => escaped.push_str("&amp;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#x27;"),
            _ => {
                if !c.is_control() || c == '\n' || c == '\r' || c == '\t' {
                    escaped.push(c);
                }
            }
        }
    }
    escaped
}

pub struct PollService;

impl PollService {
    pub async fn create_poll(
        pool: &DbPool,
        event_store: &Arc<RedbEventStore>,
        projection: &Arc<crate::core::projections::PollsProjection>,
        organizer_id: Option<String>,
        payload: CreatePollRequest,
    ) -> Result<Value, (StatusCode, String)> {
        // 1. Validation
        validate_string_length(&payload.title, MAX_TITLE_LENGTH, "Title")
            .map_err(|e| (StatusCode::BAD_REQUEST, e))?;
        validate_string_length(&payload.description, MAX_DESCRIPTION_LENGTH, "Description")
            .map_err(|e| (StatusCode::BAD_REQUEST, e))?;
        validate_string_length(&payload.location, MAX_LOCATION_LENGTH, "Location")
            .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

        if payload.dates.is_empty() {
            return Err((StatusCode::BAD_REQUEST, "At least one date is required".to_string()));
        }
        if payload.dates.len() > MAX_DATES {
            return Err((StatusCode::BAD_REQUEST, format!("Too many dates (max: {})", MAX_DATES)));
        }

        // Date Logic (Story 1.3)
        let today = if let Some(tz_str) = &payload.timezone {
            use std::str::FromStr;
            if let Ok(tz) = chrono_tz::Tz::from_str(tz_str) {
                chrono::Utc::now().with_timezone(&tz).date_naive()
            } else {
                chrono::Utc::now().date_naive()
            }
        } else {
            chrono::Utc::now().date_naive()
        };

        let mut parsed_dates = Vec::new();
        let mut min_date: Option<chrono::NaiveDate> = None;
        let mut max_date: Option<chrono::NaiveDate> = None;

        for date_str in &payload.dates {
            let date = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d").map_err(|_| {
                (StatusCode::BAD_REQUEST, format!("Invalid date format: {}", date_str))
            })?;

            if date < today {
                return Err((StatusCode::BAD_REQUEST, format!("Date cannot be in the past: {}", date_str)));
            }
            parsed_dates.push(date);

            match min_date {
                Some(min) => if date < min { min_date = Some(date) },
                None => min_date = Some(date),
            }
            match max_date {
                Some(max) => if date > max { max_date = Some(date) },
                None => max_date = Some(date),
            }
        }

        if let (Some(min), Some(max)) = (min_date, max_date) {
            let duration = max.signed_duration_since(min);
            if duration.num_days() >= 14 {
                return Err((StatusCode::BAD_REQUEST, "Date range cannot exceed 14 days".to_string()));
            }
        }

        if payload.participants.len() > MAX_PARTICIPANTS {
             return Err((StatusCode::BAD_REQUEST, format!("Too many participants (max: {})", MAX_PARTICIPANTS)));
        }
        for email in &payload.participants {
            validate_email(email).map_err(|e| (StatusCode::BAD_REQUEST, e))?;
        }

        // 2. ID Generation (NanoID)
        let poll_id = nanoid!(12); // URL-safe, 12 chars
        
        // Admin token MUST be secure (UUID is fine, or longer NanoID)
        let admin_token = Uuid::new_v4().to_string(); 

        let title = sanitize_string(&payload.title);
        let description = sanitize_string(&payload.description);
        let location = sanitize_string(&payload.location);
    
        let dates_json = serde_json::to_string(&payload.dates).map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to serialize dates: {}", e))
        })?;

        let time_range_value = if let Some(time_prefs) = &payload.time_preferences {
            serde_json::to_string(time_prefs).map_err(|e| {
                (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to serialize time preferences: {}", e))
            })?
        } else if let Some(legacy_time_range) = &payload.time_range {
            legacy_time_range.clone()
        } else {
            "{}".to_string()
        };

        // 3. Persist to DB via Repo
        tracing::info!("DEBUG: Starting write tx for poll {}", poll_id);
        let mut tx = pool.begin_write().map_err(|e| {
            tracing::error!("DEBUG: Failed to begin write tx: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;

        PollRepo::create(
            &mut tx,
            &poll_id,
            &title,
            &description,
            &location,
            &dates_json,
            &time_range_value,
            &admin_token,
            organizer_id.as_deref(),
            payload.recurrence_rule.as_deref()
        ).map_err(|e| {
            tracing::error!("Failed to create poll: {}", e);
             (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create poll".to_string())
        })?;

        // Recurrence implementation (simplified copy from general.rs)
        if let Some(rrule_str) = &payload.recurrence_rule {
             use rrule::RRuleSet;
             use std::str::FromStr;
             let start_date = parsed_dates.first().copied().unwrap_or(today);
             let dt_start = format!("DTSTART:{}T120000Z", start_date.format("%Y%m%d"));
             let full_rrule = format!("{}\nRRULE:{}", dt_start, rrule_str);

              if let Ok(rset) = RRuleSet::from_str(&full_rrule) {
                  let limit_date = Utc::now() + chrono::Duration::days(90);
                  let instances = rset.into_iter().take(50).collect::<Vec<_>>();
                  for instance in instances {
                        let instance_date = instance.date_naive();
                        if instance_date > limit_date.date_naive() { break; }
                        if instance_date < today { continue; }

                        let instance_id = Uuid::new_v4().to_string(); // Internal ID, UUID is fine
                        let date_str = instance_date.format("%Y-%m-%d").to_string();
                         PollRepo::create_instance(
                             &mut tx, 
                             &instance_id, 
                             &poll_id, 
                             &date_str, 
                             "19:00", 
                             "23:00"
                        ).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
                  }
              }
        }

        // Participants
        for email in &payload.participants {
            let participant_id = Uuid::new_v4().to_string(); // Internal
            let access_token = Uuid::new_v4().to_string();
            let name = email.split('@').next().unwrap_or("Player").to_string();
            let sanitized_name = sanitize_string(&name);
            
            PollRepo::add_participant(
                &tx, // Wait, tx is initialized through pool.begin_write(), passing &tx allows write ops
                &participant_id,
                &poll_id,
                &sanitized_name,
                Some(email.as_str()),
                Some(access_token.as_str())
            ).map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to add participant".to_string()))?;
        }

        tx.commit().map_err(|e| {
            tracing::error!("DEBUG: Failed to commit write tx: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;
        tracing::info!("DEBUG: Committed write tx for poll {}", poll_id);

        // 4. Log Activity
        crate::api::handlers::activity::log_activity(
            pool,
            "poll_created",
            "anonymous".to_string(),
            "Organizzatore".to_string(),
             Some(poll_id.clone()),
             Some(title.clone()),
        ).await.unwrap_or_else(|e| tracing::error!("Activity log error: {}", e));

        // 5. Event Sourcing
        let event_v2 = crate::core::events::PollCreatedV2 {
            id: poll_id.clone(),
            title: title.clone(),
            description: description.clone(),
            location: location.clone(),
            dates: payload.dates.clone(),
            created_at: chrono::Utc::now().timestamp(),
        };
        let event_enum = crate::core::events::Event::V2PollCreated(event_v2);

        if let Ok(event_data) = bincode::serialize(&event_enum) {
            let stream_id = format!("poll-{}", poll_id);
            println!("DEBUG: Appending event to stream {}", stream_id);
             if let Err(e) = event_store.append(&stream_id, &event_data, 0).await {
                println!("DEBUG: Append failed: {}", e);
                tracing::error!("Failed to append PollCreatedV2 event: {}", e);
                return Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to persist event: {}", e)));
             } else {
                 projection.apply(event_enum);
             }
        }

        Ok(json!({
            "id": poll_id,
            "adminToken": admin_token
        }))
    }
}
