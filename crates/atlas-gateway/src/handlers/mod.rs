//! API Handlers
//! 
//! Request handlers for all API endpoints.

mod schema;
mod records;
pub mod auth;
mod admin;
mod reports;
pub mod fusion;
pub mod advanced;

pub use schema::*;
pub use records::*;
pub use auth::*;
pub use admin::*;
pub use reports::*;
pub use fusion::*;
pub use advanced::*;

use axum::{
    Router,
    routing::{get, post, put, delete},
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
        
        // Reports (requires auth)
        .route("/reports/dashboard", get(dashboard_report))
        .route("/reports/:entity", get(generate_entity_report))
        
        // Import/Export (requires auth)
        .route("/import", post(import_data))
        .route("/export/:entity", get(export_data))
        
        // ═══════════════════════════════════════════════════════
        // Oracle Fusion-inspired features
        // ═══════════════════════════════════════════════════════
        
        // Notifications (bell icon)
        .route("/notifications", get(list_notifications))
        .route("/notifications/unread-count", get(get_unread_count))
        .route("/notifications/:id/read", put(mark_notification_read))
        .route("/notifications/read-all", put(mark_all_notifications_read))
        .route("/notifications/:id/dismiss", put(dismiss_notification))
        
        // Saved Searches (personalized views)
        .route("/saved-searches", get(list_saved_searches))
        .route("/saved-searches", post(create_saved_search))
        .route("/saved-searches/:id", delete(delete_saved_search))
        
        // Approval Chains
        .route("/approval-chains", get(list_approval_chains))
        .route("/approval-chains", post(create_approval_chain))
        .route("/approvals/pending", get(get_pending_approvals))
        .route("/approvals/:step_id/approve", post(approve_approval_step))
        .route("/approvals/:step_id/reject", post(reject_approval_step))
        .route("/approvals/:step_id/delegate", post(delegate_approval_step))
        
        // Duplicate Detection
        .route("/duplicates/check", post(check_duplicates))
        
        // ═══════════════════════════════════════════════════════
        // Advanced Oracle Fusion features (Phase 2)
        // ═══════════════════════════════════════════════════════
        
        // Structured Filtering (advanced list endpoint)
        .route("/:entity/filtered", get(list_records_advanced))
        
        // Bulk Operations
        .route("/bulk", post(execute_bulk_operation))
        
        // Comments / Notes on Records
        .route("/:entity/:id/comments", get(list_comments))
        .route("/:entity/:id/comments", post(create_comment))
        .route("/:entity/:id/comments/:comment_id", delete(delete_comment))
        .route("/:entity/:id/comments/:comment_id/pin", put(toggle_pin_comment))
        
        // Favorites / Bookmarks
        .route("/favorites", get(list_favorites))
        .route("/:entity/:id/favorite", post(add_favorite))
        .route("/:entity/:id/favorite", delete(remove_favorite))
        .route("/:entity/:id/favorite", get(check_favorite))
        
        // CSV Export
        .route("/export/:entity/csv", get(export_csv))
        
        // CSV Import
        .route("/import/csv", post(import_csv))
        
        // Related Records
        .route("/:entity/:id/related/:related_entity", get(get_related_records))
        
        // Effective Dating
        .route("/:entity/:id/effective", get(get_effective_record))
        .route("/:entity/:id/effective", post(create_effective_version))
        
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
        
        // Oracle Fusion: Duplicate detection rules
        .route("/duplicate-rules", post(create_duplicate_rule))
        
        .route("/cache/clear", post(clear_cache))
        .route("/cache/invalidate/:entity", post(invalidate_entity_cache))
        .layer(middleware::from_fn(auth_middleware))
}
