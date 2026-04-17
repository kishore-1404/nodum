mod api;
mod config;
mod db;
mod graph;
mod llm;
mod reader;

use std::sync::Arc;
use tower_http::cors::{CorsLayer, Any};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub struct AppState {
    pub pool: sqlx::PgPool,
    pub config: config::Config,
    pub upload_dir: String,
    pub jwt_secret: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "nodum=debug,tower_http=debug".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = config::Config::from_env()
        .expect("Failed to load configuration. Check environment variables.");

    tracing::info!("Starting Nodum server on {}:{}", config.host, config.port);

    // Create database connection pool
    let pool = db::create_pool(&config.database_url).await?;
    tracing::info!("Connected to PostgreSQL");

    // Run migrations
    db::run_migrations(&pool).await?;
    tracing::info!("Migrations complete");

    // Create upload directory
    tokio::fs::create_dir_all(&config.upload_dir).await?;

    let state = Arc::new(AppState {
        pool,
        upload_dir: config.upload_dir.clone(),
        jwt_secret: config.jwt_secret.clone(),
        config,
    });

    // Build router
    let app = api::routes()
        .layer(CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any))
        .layer(TraceLayer::new_for_http())
        .with_state(state.clone());

    // Serve static files in production (frontend build)
    let app = app.fallback_service(
        tower_http::services::ServeDir::new("../frontend/dist")
            .fallback(tower_http::services::ServeFile::new("../frontend/dist/index.html"))
    );

    let addr = format!("{}:{}", state.config.host, state.config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Nodum is running at http://{}", addr);

    axum::serve(listener, app).await?;
    Ok(())
}
