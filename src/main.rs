use dnd_scheduler::core;
use dnd_scheduler::db;
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            match std::env::var("RUST_LOG") {
                Ok(v) => v,
                Err(_) => "dnd_scheduler=debug,tower_http=debug".into(),
            },
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load environment variables
    if let Err(e) = dotenvy::dotenv() {
        tracing::warn!("Failed to load .env file: {}", e);
    } else {
        tracing::info!(".env file loaded successfully");
    }

    if std::env::var("GOOGLE_CLIENT_ID").is_ok() {
        tracing::info!("GOOGLE_CLIENT_ID is set");
    } else {
        tracing::warn!("GOOGLE_CLIENT_ID is NOT set - Google Login will be disabled");
    }

    // Initialize database
    let pool = match db::init_db().await {
        Ok(p) => p,
        Err(e) => {
            tracing::error!("Failed to initialize database: {}", e);
            std::process::exit(1);
        }
    };

    // Verify static assets directory exists
    let static_dir = match std::env::var("STATIC_DIR") {
        Ok(v) => v,
        Err(_) => "static".to_string(),
    };
    if !std::path::Path::new(&static_dir).exists() {
        tracing::error!("FATAL: '{}' directory not found in current working directory. Frontend assets cannot be served. Please run the server from the project root.", static_dir);
        std::process::exit(1);
    }

    // Story 1.7: Start background cleanup job
    let cleanup_pool = pool.clone();
    tokio::spawn(async move {
        core::jobs::cleanup::run_cron(cleanup_pool).await;
    });

    // Create App Router using library function
    let app = dnd_scheduler::create_router(pool);

    // Run server
    let port_str = match std::env::var("PORT") {
        Ok(v) => v,
        Err(_) => "3000".to_string(),
    };
    let port_num = port_str.parse::<u16>().unwrap_or(3000);
    let addr = SocketAddr::from(([0, 0, 0, 0], port_num));
    tracing::info!("listening on {}", addr);

    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            tracing::error!(
                "Failed to bind to port {}: {}. Is the port already in use?",
                port_num,
                e
            );
            std::process::exit(1);
        }
    };

    match axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    {
        Ok(_) => {}
        Err(e) => {
            tracing::error!("Server error: {}", e);
            std::process::exit(1);
        }
    }
}
