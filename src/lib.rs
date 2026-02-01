use axum::{
    handler::Handler,
    routing::{delete, get, post, post_service, put},
    Router,
};
use tower::Layer;
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

pub fn create_router(pool: DbPool) -> Router {
    // Verify static assets directory exists
    let static_dir = match std::env::var("STATIC_DIR") {
        Ok(v) => v,
        Err(_) => "static".to_string(),
    };

    // Setup CORS with security restrictions
    let cors = security_headers::get_cors_layer();

    // Rate Limiting Configuration - Separate limits for auth and general API

    // Auth routes: More lenient limits (authentication is less frequent but critical)
    let auth_governor_conf = std::sync::Arc::new(
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
    let creation_governor_conf = std::sync::Arc::new(
        GovernorConfigBuilder::default()
            .period(std::time::Duration::from_secs(2))
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
    let voting_governor_conf = std::sync::Arc::new(
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
        )
        .layer(GovernorLayer {
            config: auth_governor_conf,
        });

    // General API Routes (with standard rate limiting)
    // Note: We use post_service for create_poll to apply strict limits specifically to it
    let general_routes = Router::new()
        // Poll Routes
        .route(
            "/polls",
            get(handlers::list_polls).post_service(
                GovernorLayer {
                    config: creation_governor_conf,
                }
                .layer(handlers::create_poll.with_state(pool.clone())),
            ),
        )
        // Story 1.6: Serve dynamic poll page with OG metadata (Short link)
        .route("/p/:id", get(handlers::serve_poll_page))
        .route("/polls/:id", get(handlers::get_poll))
        .route("/polls/:id/join", post(handlers::join_poll))
        .route(
            "/polls/:id/participants/:participant_id/availability",
            post_service(
                GovernorLayer {
                    config: voting_governor_conf,
                }
                .layer(handlers::update_availability.with_state(pool.clone())),
            ),
        )
        .route("/polls/:id", put(handlers::update_poll))
        .route("/polls/:id", delete(handlers::delete_poll))
        .route("/participants/:id", delete(handlers::delete_participant))
        .route("/polls/:id/finalize", put(handlers::finalize_poll))
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
        .route("/admin/stats", get(admin_stats::get_admin_stats))
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
        .route("/gdpr/delete", post(gdpr::delete_account_confirmed))
        .layer(GovernorLayer {
            config: general_governor_conf,
        });

    // Combine auth and general routes
    let api_routes = Router::new().merge(auth_routes).merge(general_routes);

    // Build main router
    Router::new()
        .nest("/api", api_routes)
        // Static files
        .nest_service("/", ServeDir::new(&static_dir))
        // Global Middleware (Outer layers wrap inner layers)
        // Handle 429 JSON first (so it catches 429s from anywhere)
        .layer(axum::middleware::from_fn(security_headers::handle_429_json))
        // Security middleware (applied to all routes)
        .layer(axum::middleware::from_fn(
            security_headers::security_headers,
        ))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(pool)
}
