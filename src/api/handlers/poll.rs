use crate::core::models::CreatePollRequest;
use crate::core::services::PollService;

use crate::db::DbPool;
use axum::{
    extract::{State},
    http::StatusCode,
    Json,
};
use serde_json::Value;

use crate::security::auth::MaybeAuthUser;

pub async fn create_poll(
    State(state): State<crate::api::handlers::event_handlers::EventAppState>,
    MaybeAuthUser(user): MaybeAuthUser,
    Json(payload): Json<CreatePollRequest>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let pool = &state.pool;
    let _event_store = &state.event_store;
    let projection = &state.projection;
    let organizer_id = user.map(|u| u.id);

    let result = PollService::create_poll(
        &pool,
        &_event_store,
        &projection,
        organizer_id,
        payload
    ).await.map_err(|(s, e)| {
        tracing::error!("create_poll failed with: {} - {}", s, e);
        (s, serde_json::to_string(&serde_json::json!({ "error": e })).unwrap_or(e))
    })?;

    Ok(Json(result))
}

use axum::extract::Path;
use axum_extra::extract::cookie::{SignedCookieJar};
use crate::core::models::{PollResponse, VoteRequest, FinalizePollRequest};
use crate::db::queries::poll_repo::PollRepo;

pub async fn get_poll(
    State(pool): State<DbPool>,
    Path(id): Path<String>,
    jar: SignedCookieJar,
) -> Result<Json<PollResponse>, (StatusCode, String)> {
    let details = PollRepo::get_details(&pool, &id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let Some((poll, participants, votes, instances)) = details else {
        return Err((StatusCode::NOT_FOUND, "Poll not found".to_string()));
    };


    // Determine current user context from session cookie
    let my_session_id = jar.get("dnd_session").map(|c| c.value().to_string());
    
    let my_vote = if let Some(session_id) = my_session_id {
        // Filter votes for this session
        let user_votes: Vec<_> = votes.iter()
            .filter(|v| v.participant_id == session_id)
            .map(|v| crate::core::models::AvailabilityEntry {
                date: v.date.clone(),
                time_slot: v.time_slot.clone(),
                status: v.status.clone(),
            })
            .collect();
        
        if user_votes.is_empty() {
             None
        } else {
             Some(user_votes)
        }
    } else {
        None
    };
    
    let response = PollResponse {
        poll,
        participants,
        votes,
        instances,
        my_vote,
    };

    Ok(Json(response))
}

pub async fn submit_vote(
    State(pool): State<DbPool>,
    axum::extract::Extension(event_store): axum::extract::Extension<std::sync::Arc<crate::core::store::RedbEventStore>>,
    axum::extract::Extension(projection): axum::extract::Extension<std::sync::Arc<crate::core::projections::PollsProjection>>,
    Path(id): Path<String>,
    jar: SignedCookieJar,
    Json(payload): Json<VoteRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    // 1. Validate Session
    let session_cookie = jar.get("dnd_session").ok_or((
        StatusCode::UNAUTHORIZED,
        "No session cookie found".to_string(),
    ))?;
    let session_id = session_cookie.value();

    let availability_clone = payload.availability.clone();

    // 2. Upsert Vote
    PollRepo::upsert_vote(&pool, &id, session_id, &payload.name, payload.availability)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // 2.5 Emit Events
    let participant_exists = projection.get(&id)
        .map(|v| v.participants.iter().any(|p| p.id == session_id))
        .unwrap_or(false);

    let stream_id = format!("poll-{}", id);

    if !participant_exists {
        let event1 = crate::core::events::Event::V1ParticipantJoined(crate::core::events::ParticipantJoinedV1 {
            id: session_id.to_string(),
            poll_id: id.clone(),
            name: payload.name.clone(),
            email: None,
            access_token: session_id.to_string(),
        });
        
        if let Err(e) = event_store.append_event(&stream_id, &event1).await {
            tracing::error!("Failed to append ParticipantJoinedV1 event: {}", e);
        } else {
            projection.apply(event1);
        }
    }

    let event = crate::core::events::Event::V2VoteUpdated(crate::core::events::VoteUpdatedV2 {
        participant_name: payload.name.clone(),
        participant_email: None,
        availability: availability_clone.into_iter().map(|a| crate::core::events::AvailabilityEntryV1 {
            date: a.date,
            slot: a.time_slot,
            status: a.status,
        }).collect(),
    });

    if let Err(e) = event_store.append_event(&stream_id, &event).await {
        tracing::error!("Failed to append VoteUpdatedV2 event: {}", e);
    } else {
        projection.apply_with_id(&id, event);
    }

    // 3. Return Success
    Ok(Json(serde_json::json!({ "status": "success" })))
}

use axum::response::IntoResponse;

pub async fn finalize_poll(
    State(pool): State<DbPool>,
    Path(id): Path<String>,
    jar: SignedCookieJar,
    Json(payload): Json<FinalizePollRequest>,
) -> impl IntoResponse {
    // Use verify session logic
    if jar.get("dnd_session").is_none() {
         return (StatusCode::UNAUTHORIZED, "No session cookie found".to_string()).into_response();
    }

    // 2. Finalize Poll
    let success = match PollRepo::finalize_poll(
        &pool,
        &id,
        &payload.finalized_time,
        payload.notes.as_deref(),
    ) {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("Failed to finalize poll: {:?}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to finalize poll".to_string(),
            ).into_response();
        }
    };

    if !success {
        return (StatusCode::NOT_FOUND, "Poll not found".to_string()).into_response();
    }

    Json(serde_json::json!({ "status": "success", "finalized": true })).into_response()
}
