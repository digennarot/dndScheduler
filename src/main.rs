use axum::{
    routing::{delete, get, post, put},
    Router,
};
use std::net::SocketAddr;
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod activity_handlers;
mod audit;
mod auth;
mod authelia_auth;
mod db;
mod email_service;
mod gdpr;
mod handlers;
mod models;
mod security;
mod whatsapp_service;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "dnd_scheduler=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load environment variables from .env file (optional, for local dev)
    // If it fails, we assume env vars are set via other means (e.g. system env)
    dotenvy::dotenv().ok();

    // Initialize database
    let pool = db::init_db().await.expect("Failed to initialize database");

    // Setup CORS with security restrictions
    let cors = security::get_cors_layer();

    // Rate Limiting Configuration - Separate limits for auth and general API

    // Auth routes: More lenient limits (authentication is less frequent but critical)
    let auth_governor_conf = std::sync::Arc::new(
        GovernorConfigBuilder::default()
            .per_second(30) // Allow 30 auth requests per second
            .burst_size(100) // Allow bursts of 100
            .finish()
            .unwrap(),
    );

    // General API routes: Standard limits
    let general_governor_conf = std::sync::Arc::new(
        GovernorConfigBuilder::default()
            .per_second(50) // Allow 50 requests per second
            .burst_size(200) // Allow bursts of 200
            .finish()
            .unwrap(),
    );

    // Authentication Routes (with separate, lenient rate limiting)
    let auth_routes = Router::new()
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
        .route("/auth/logout/:token", post(auth::logout))
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
    let general_routes = Router::new()
        // Poll Routes
        .route(
            "/polls",
            post(handlers::create_poll).get(handlers::list_polls),
        )
        .route("/polls/:id", get(handlers::get_poll))
        .route("/polls/:id/join", post(handlers::join_poll))
        .route(
            "/polls/:id/participants/:participant_id/availability",
            post(handlers::update_availability),
        )
        .route("/polls/:id", put(handlers::update_poll))
        .route("/polls/:id", delete(handlers::delete_poll))
        .route("/participants/:id", delete(handlers::delete_participant))
        // Admin Routes
        .route("/admin/login", post(handlers::admin_login))
        .route("/admin/google-login", post(handlers::google_login))
        .route("/admin/me", get(handlers::get_current_admin))
        .route("/admin/users", get(handlers::get_all_users))
        .route(
            "/admin/users/:id/role",
            put(handlers::update_user_role),
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
        .route("/gdpr/delete", post(gdpr::delete_account_confirmed))
        .layer(GovernorLayer {
            config: general_governor_conf,
        });

    // Combine auth and general routes
    let api_routes = Router::new().merge(auth_routes).merge(general_routes);

    // Build main router
    let app = Router::new()
        .nest("/api", api_routes)
        // Static files
        .nest_service("/", ServeDir::new("static"))
        // Global Middleware (Outer layers wrap inner layers)
        // Handle 429 JSON first (so it catches 429s from anywhere)
        .layer(axum::middleware::from_fn(security::handle_429_json))
        // Security middleware (applied to all routes)
        .layer(axum::middleware::from_fn(security::security_headers))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(pool);

    // Run server
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to port 3000. Is the port already in use?");

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .expect("Server error occurred");
}
