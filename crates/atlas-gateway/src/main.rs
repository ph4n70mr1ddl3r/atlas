//! Atlas Gateway Service
//! 
//! Main API entry point handling authentication, routing, and request processing.

use atlas_gateway::{handlers, AppState};

use axum::Router;
use tower_http::cors::{CorsLayer, Any};
use tower_http::trace::TraceLayer;
use tower_http::timeout::TimeoutLayer;
use std::sync::Arc;
use std::time::Duration;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use anyhow::Context;

use atlas_gateway::middleware::get_login_rate_limiter;

/// Request timeout duration
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info,atlas_gateway=debug".into())
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    info!("Starting Atlas Gateway");
    
    // Validate required environment variables
    validate_environment()?;
    
    // Initialize application state
    let state = AppState::new().await
        .context("Failed to initialize application state")?;
    let state = Arc::new(state);
    
    // Initialize rate limiter for login
    let _rate_limiter = get_login_rate_limiter();
    
    // Build CORS layer with configurable origins
    let cors_layer = build_cors_layer();
    
    // Build router with middleware applied directly
    let app = Router::new()
        .nest("/api/v1", handlers::api_routes())
        .nest("/api/admin", handlers::admin_routes())
        .route("/health", axum::routing::get(handlers::health_check))
        .route("/metrics", axum::routing::get(handlers::metrics))
        .route(
            "/api/v1/auth/login",
            axum::routing::post(handlers::login)
        )
        .layer(cors_layer)
        .layer(TraceLayer::new_for_http())
        .layer(TimeoutLayer::new(REQUEST_TIMEOUT))
        .with_state(state);
    
    // Start server with graceful shutdown
    let addr = std::env::var("BIND_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    
    info!("Listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .context("Failed to bind to address")?;
    
    // Enable graceful shutdown
    let graceful = axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal());
    
    if let Err(e) = graceful.await {
        tracing::error!("Server error: {}", e);
    }
    
    info!("Server shutdown complete");
    Ok(())
}

/// Build CORS layer with configurable allowed origins
fn build_cors_layer() -> CorsLayer {
    let cors_layer = CorsLayer::new()
        .allow_methods(Any)
        .allow_headers(Any);
    
    if let Ok(origins) = std::env::var("CORS_ORIGINS") {
        let origins: Vec<_> = origins
            .split(',')
            .filter_map(|o| {
                let o = o.trim();
                o.parse::<axum::http::HeaderValue>().ok()
            })
            .collect();
        
        if !origins.is_empty() {
            return cors_layer.allow_origin(origins);
        }
    }
    
    // Development: allow localhost
    cors_layer.allow_origin(Any)
}

/// Validate required environment variables
fn validate_environment() -> anyhow::Result<()> {
    // Check for JWT_SECRET in production
    if std::env::var("RUST_ENV").unwrap_or_default() == "production" {
        let jwt_secret = std::env::var("JWT_SECRET")
            .context("JWT_SECRET must be set in production")?;
        
        if jwt_secret.len() < 32 {
            anyhow::bail!("JWT_SECRET must be at least 32 characters");
        }
        
        if jwt_secret.contains("change") || jwt_secret.contains("secret") {
            anyhow::bail!("JWT_SECRET appears to be a placeholder value");
        }
    }
    
    Ok(())
}

/// Signal handler for graceful shutdown
async fn shutdown_signal() {
    use tokio::signal;
    
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install CTRL+C handler");
    };
    
    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };
    
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    
    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    
    tracing::info!("Shutdown signal received");
}
