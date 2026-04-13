//! API Handlers
//! 
//! Request handlers for all API endpoints.

mod schema;
mod records;
pub mod auth;
mod admin;

pub use schema::*;
pub use records::*;
pub use auth::*;
pub use admin::*;

use axum::{
    Router,
    routing::{get, post, put, delete},
    extract::State,
    Json,
    middleware,
};
use serde_json::Value;
use crate::AppState;
use crate::middleware::auth_middleware;
use std::sync::Arc;

/// Health check endpoint
pub async fn health_check() -> &'static str {
    "OK"
}

/// Metrics endpoint (placeholder)
pub async fn metrics() -> Json<Value> {
    Json(serde_json::json!({
        "uptime": "N/A",
        "requests_total": 0,
        "active_connections": 0,
    }))
}

/// API routes (v1) - requires authentication
pub fn api_routes() -> Router<Arc<AppState>> {
    Router::new()
        // Schema introspection (requires auth)
        .route("/schema/:entity", get(get_entity_schema))
        .route("/schema/:entity/form", get(get_entity_form))
        .route("/schema/:entity/list", get(get_entity_list_view))
        
        // CRUD operations (requires auth)
        .route("/:entity", post(create_record))
        .route("/:entity", get(list_records))
        .route("/:entity/:id", get(get_record))
        .route("/:entity/:id", put(update_record))
        .route("/:entity/:id", delete(delete_record))
        
        // Workflow operations (requires auth)
        .route("/:entity/:id/transitions", get(get_transitions))
        .route("/:entity/:id/:action", post(execute_action))
        
        // Audit (requires auth)
        .route("/:entity/:id/history", get(get_record_history))
        .layer(middleware::from_fn(auth_middleware))
}

/// Admin routes - requires authentication
pub fn admin_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/schema", post(create_entity))
        .route("/schema/:entity", put(update_entity))
        .route("/schema/:entity", delete(delete_entity))
        
        .route("/workflows", post(create_workflow))
        .route("/workflows/:entity", put(update_workflow))
        
        .route("/config", get(get_config))
        .route("/config/:key", get(get_config_value))
        .route("/config/:key", put(set_config_value))
        
        .route("/cache/clear", post(clear_cache))
        .route("/cache/invalidate/:entity", post(invalidate_entity_cache))
        .layer(middleware::from_fn(auth_middleware))
}
