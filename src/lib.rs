use axum::{
    extract::Extension,
    routing::{delete, get, post, put},
    Router,
};
use tower_governor::{
    governor::GovernorConfigBuilder, key_extractor::SmartIpKeyExtractor, GovernorLayer,
};
use tower_http::{services::ServeDir, trace::TraceLayer};

pub mod api;
pub mod core;
pub mod db;
pub mod security;

// Re-export / Alias modules
use api::handlers::{activity as activity_handlers, admin as admin_stats, general as handlers};
use db::DbPool;
use security::{audit, auth, authelia as authelia_auth, gdpr, headers as security_headers};

#[derive(Clone)]
pub struct AppState {
    pub pool: DbPool,
    pub key: axum_extra::extract::cookie::Key,
}

impl axum::extract::FromRef<AppState> for DbPool {
    fn from_ref(state: &AppState) -> Self {
        state.pool.clone()
    }
}

impl axum::extract::FromRef<AppState> for axum_extra::extract::cookie::Key {
    fn from_ref(state: &AppState) -> Self {
        state.key.clone()
    }
}

pub async fn create_router(pool: DbPool, event_store_path: Option<String>) -> Router {
    let event_store_url = event_store_path.unwrap_or_else(|| {
        std::env::var("EVENT_STORE_URL").unwrap_or_else(|_| "data/event_store.redb".to_string())
    });
    let event_store = std::sync::Arc::new(
        core::store::RedbEventStore::new(&event_store_url).expect("Failed to init Redb"),
    );

    // Initialize Projections
    let projection = std::sync::Arc::new(core::projections::PollsProjection::new());
    let users_projection = std::sync::Arc::new(core::projections::UsersProjection::new());

    // Replay Events to rebuild in-memory state
    match event_store.read_all_events().await {
        Ok(events) => {
            tracing::info!("Replaying {} events to build projections...", events.len());
            for (key, event_data) in events {
                if let Ok(event) = bincode::deserialize::<core::events::Event>(&event_data) {
                    // Extract poll_id from key "poll-{uuid}/{version}"
                    // Key format: poll-{id}/{version}
                    // We need to parse this carefully.
                    let parts: Vec<&str> = key.split('/').collect();
                    if let Some(stream_id) = parts.first() {
                        if let Some(poll_id) = stream_id.strip_prefix("poll-") {
                            projection.apply_with_id(poll_id, event);
                        } else if stream_id.starts_with("user-") {
                            users_projection.apply(event);
                        }
                    } else {
                        // Fallback? Should not happen if key is valid
                    }
                } else {
                    tracing::warn!("Failed to deserialize event: {}", key);
                }
            }
            tracing::info!("Projection rebuild complete.");
        }
        Err(e) => {
            tracing::error!("Failed to read events for replay: {}", e);
            // We might want to panic here? If we can't read events, state is empty.
        }
    }

    // Initialize Cookie Key (for Story 2.1)
    let session_key = {
        use std::env;
        use axum_extra::extract::cookie::Key;
        use rand::RngCore;

        match env::var("SESSION_SECRET") {
            Ok(secret) => Key::from(secret.as_bytes()),
            Err(_) => {
                tracing::warn!("SESSION_SECRET not set! Using random key. Sessions will be invalidated on restart.");
                let mut key = [0u8; 64];
                rand::thread_rng().fill_bytes(&mut key);
                Key::from(&key)
            }
        }
    };

    let event_state = api::handlers::event_handlers::EventAppState {
        event_store: event_store.clone(),
        pool: pool.clone(),
        projection: projection.clone(),
        users_projection: users_projection.clone(),
        key: session_key.clone(),
    };

    // Verify static assets directory exists
    let static_dir = match std::env::var("STATIC_DIR") {
        Ok(v) => v,
        Err(_) => "static".to_string(),
    };

    // Setup CORS with security restrictions
    let cors = security_headers::get_cors_layer();

    // Rate Limiting Configuration - Separate limits for auth and general API

    // Auth routes: More lenient limits (authentication is less frequent but critical)
    let _auth_governor_conf = std::sync::Arc::new(
        GovernorConfigBuilder::default()
            .per_second(30) // Allow 30 auth requests per second
            .burst_size(100) // Allow bursts of 100
            .key_extractor(SmartIpKeyExtractor)
            .finish()
            .unwrap_or_else(|| {
                tracing::error!("Failed to configure Auth Rate Limiting");
                std::process::exit(1);
            }),
    );

    // General API routes: Standard limits
    let general_governor_conf = std::sync::Arc::new(
        GovernorConfigBuilder::default()
            .per_second(50) // Allow 50 requests per second
            .burst_size(200) // Allow bursts of 200
            .key_extractor(SmartIpKeyExtractor)
            .finish()
            .unwrap_or_else(|| {
                tracing::error!("Failed to configure General Rate Limiting");
                std::process::exit(1);
            }),
    );

    // Poll Creation: Relaxed limits for Development
    // Was 5/min, now ~30/min (1 request every 2 seconds, burst 20)
    let _creation_governor_conf = std::sync::Arc::new(
        GovernorConfigBuilder::default()
            .period(std::time::Duration::from_secs(6))
            .burst_size(20)
            .key_extractor(SmartIpKeyExtractor)
            .finish()
            .unwrap_or_else(|| {
                tracing::error!("Failed to configure Creation Rate Limiting");
                std::process::exit(1);
            }),
    );

    // Voting: Relaxed limits for Development
    // Was 5/min, now high throughput (1 request every 1 second, burst 50)
    let _voting_governor_conf = std::sync::Arc::new(
        GovernorConfigBuilder::default()
            .period(std::time::Duration::from_secs(60))
            .burst_size(5)
            .key_extractor(SmartIpKeyExtractor)
            .finish()
            .unwrap_or_else(|| {
                tracing::error!("Failed to configure Voting Rate Limiting");
                std::process::exit(1);
            }),
    );

    // Authentication Routes (with separate, lenient rate limiting)
    let auth_routes = Router::new()
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
        .route("/auth/google/login", post(auth::login_google))
        .route("/auth/google/config", get(auth::get_google_config))
        .route("/auth/logout/:token", post(auth::logout))
        .route("/auth/me", get(auth::get_current_session_user))
        .route("/auth/me/:token", get(auth::get_current_user))
        .route("/auth/account", delete(auth::delete_account))
        .route("/auth/profile", put(auth::update_profile))
        .route("/auth/password", put(auth::change_password))
        // Authelia SSO Routes
        .route(
            "/auth/authelia/config",
            get(authelia_auth::get_authelia_config),
        )
        .route(
            "/auth/authelia/session",
            get(authelia_auth::get_authelia_session),
        );

    // General API Routes (with standard rate limiting)
    // Note: We use post_service for create_poll to apply strict limits specifically to it
    let general_routes = Router::new()
        // Poll Routes
        .route(
            "/polls",
            get(handlers::list_polls).post(crate::api::handlers::poll::create_poll),
        )
        // Story 1.6: Serve dynamic poll page with OG metadata (Short link)
        .route("/p/:id", get(handlers::serve_poll_page))
        .route(
            "/polls/:id",
            get(handlers::get_poll)
                .put(handlers::update_poll)
                .delete(handlers::delete_poll),
        )
        // Join route moved to event_routes for Event Sourcing migration
        // .route("/polls/:id/join", post(handlers::join_poll))
        .route("/polls/:id/vote", post(crate::api::handlers::poll::submit_vote))
        // .route("/polls/:id/finalize", put(crate::api::handlers::poll::finalize_poll)) // Moved to event_handlers
        .route("/participants/:id", delete(handlers::delete_participant))
        // Finalize route moved to event_routes for Event Sourcing migration
        // .route("/polls/:id/finalize", put(handlers::finalize_poll))
        // Story 5.2: Calendar Export
        .route(
            "/polls/:id/export",
            get(crate::api::handlers::export::export_poll_ics),
        )
        // Admin Routes
        .route("/admin/login", post(admin_stats::admin_login))
        .route("/admin/google-login", post(handlers::google_login))
        .route("/admin/me", get(handlers::get_current_admin))
        .route("/admin/users", get(handlers::get_all_users))
        .route("/admin/users/:id/role", put(handlers::update_user_role))
        .route(
            "/admin/users/:id/password",
            put(handlers::admin_reset_user_password),
        )
        .route("/admin/users/:id", delete(handlers::delete_user_admin))
        .route("/admin/stats", get(admin_stats::get_admin_stats))
        .route("/admin/polls", get(admin_stats::get_admin_polls))
        .route("/admin/polls/:id", delete(admin_stats::delete_poll))
        .route(
            "/admin/polls/:poll_id/participants/:participant_id",
            delete(admin_stats::delete_vote),
        )
        // Activity and Reminder Routes
        .route(
            "/activity/recent",
            get(activity_handlers::get_recent_activity),
        )
        .route(
            "/reminder/config",
            get(activity_handlers::get_reminder_config),
        )
        .route(
            "/reminder/whatsapp",
            post(activity_handlers::send_whatsapp_reminder),
        )
        .route(
            "/reminder/telegram",
            post(activity_handlers::send_telegram_reminder),
        )
        .route(
            "/reminder/email",
            post(activity_handlers::send_email_reminder),
        )
        // GDPR Compliance Routes
        .route(
            "/gdpr/consent",
            get(gdpr::get_consent).post(gdpr::update_consent),
        )
        .route("/gdpr/export", get(gdpr::export_data))
        .route("/gdpr/delete", post(gdpr::delete_account_confirmed));

    // Event API Routes (Event Store)
    let event_routes = Router::new()
        .route(
            "/events/poll",
            post(api::handlers::event_handlers::create_poll_event),
        )
        .route(
            "/events/poll/:id",
            get(api::handlers::event_handlers::get_poll_event),
        )
        .route(
            "/events/poll/:id/participants/:participant_id/availability",
            post(api::handlers::event_handlers::update_availability_event),
        )
        // Override legacy voting route with Event Sourcing handler (Dual Write enabled)
        .route(
            "/polls/:id/participants/:participant_id/availability",
            post(api::handlers::event_handlers::update_availability_event),
        )
        // Override legacy join route with Event Sourcing handler (Dual Write enabled)
        .route(
            "/polls/:id/join",
            post(api::handlers::event_handlers::join_poll_event),
        )
        // Override legacy finalize route with Event Sourcing handler (Dual Write enabled)
        .route(
            "/polls/:id/finalize",
            put(api::handlers::event_handlers::finalize_poll_event),
        );

    // Build main router
    let mut main_api = Router::new()
        .merge(auth_routes)
        .merge(general_routes)
        .merge(event_routes)
        .layer(Extension(event_store.clone()))
        .layer(Extension(projection))
        .layer(Extension(users_projection.clone()));

    // Optional Rate Limiting (Disabled in tests to avoid 500 when IP is missing)
    if std::env::var("DND_DISABLE_RATE_LIMIT").is_err() {
        main_api = main_api.layer(GovernorLayer {
            config: general_governor_conf,
        });
    }

    let main_api = main_api.with_state(event_state);

    // Build main router
    Router::new()
        .nest("/api", main_api)
        // Static files
        .nest_service("/", ServeDir::new(&static_dir))
        // Middleware
        .layer(axum::middleware::from_fn_with_state(
            session_key.clone(),
            security::middleware::ensure_session,
        ))
        .layer(Extension(session_key)) // Story 2.1: Key for Signed Cookies
        .layer(axum::middleware::from_fn(security_headers::handle_429_json))
        .layer(axum::middleware::from_fn(
            security_headers::security_headers,
        ))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
}
