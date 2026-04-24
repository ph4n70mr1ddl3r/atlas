//! Data Archiving and Retention Management API Handlers
//!
//! Oracle Fusion Cloud: Tools > Information Lifecycle Management (ILM)
//!
//! Endpoints for managing retention policies, legal holds, archival,
//! purging, and restoration of enterprise data.

use axum::{
    extract::{Path, Query, State, Extension},
    Json,
    http::StatusCode,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;
use tracing::error;

use crate::AppState;
use crate::handlers::auth::Claims;
use atlas_shared::{CreateRetentionPolicyRequest, CreateLegalHoldRequest};

// ============================================================================
// Query Parameters
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ListPoliciesQuery {
    pub status: Option<String>,
    pub entity_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListHoldsQuery {
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListArchivedQuery {
    pub entity_type: Option<String>,
    pub status: Option<String>,
    pub limit: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct ListBatchesQuery {
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListAuditQuery {
    pub operation: Option<String>,
    pub entity_type: Option<String>,
    pub limit: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddHoldItemsRequest {
    pub items: Vec<HoldItemEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HoldItemEntry {
    pub entity_type: String,
    pub record_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteArchiveRequest {
    pub policy_id: String,
    pub batch_number: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseHoldRequest {
    pub reason: Option<String>,
}

// ============================================================================
// Retention Policy Handlers
// ============================================================================

pub async fn create_retention_policy(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateRetentionPolicyRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.data_archiving_engine.create_policy(org_id, payload, Some(user_id)).await {
        Ok(policy) => Ok((StatusCode::CREATED, Json(serde_json::to_value(policy).unwrap_or_else(|e| {
            error!("Serialization error: {}", e); serde_json::Value::Null
        })))),
        Err(e) => {
            error!("Failed to create retention policy: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                409 => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn get_retention_policy(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let id = Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;

    match state.data_archiving_engine.get_policy(id).await {
        Ok(Some(policy)) => Ok(Json(serde_json::to_value(policy).unwrap_or_else(|e| {
            error!("Serialization error: {}", e); serde_json::Value::Null
        }))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get retention policy: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn list_retention_policies(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<ListPoliciesQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.data_archiving_engine.list_policies(
        org_id,
        query.status.as_deref(),
        query.entity_type.as_deref(),
    ).await {
        Ok(policies) => Ok(Json(serde_json::to_value(policies).unwrap_or_else(|e| {
            error!("Serialization error: {}", e); serde_json::Value::Null
        }))),
        Err(e) => {
            error!("Failed to list retention policies: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn activate_retention_policy(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let id = Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;

    match state.data_archiving_engine.activate_policy(id).await {
        Ok(policy) => Ok(Json(serde_json::to_value(policy).unwrap_or_else(|e| {
            error!("Serialization error: {}", e); serde_json::Value::Null
        }))),
        Err(e) => {
            error!("Failed to activate retention policy: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn deactivate_retention_policy(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let id = Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;

    match state.data_archiving_engine.deactivate_policy(id).await {
        Ok(policy) => Ok(Json(serde_json::to_value(policy).unwrap_or_else(|e| {
            error!("Serialization error: {}", e); serde_json::Value::Null
        }))),
        Err(e) => {
            error!("Failed to deactivate retention policy: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn delete_retention_policy(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let id = Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;

    match state.data_archiving_engine.delete_policy(id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            error!("Failed to delete retention policy: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// ============================================================================
// Legal Hold Handlers
// ============================================================================

pub async fn create_legal_hold(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateLegalHoldRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.data_archiving_engine.create_legal_hold(org_id, payload, Some(user_id)).await {
        Ok(hold) => Ok((StatusCode::CREATED, Json(serde_json::to_value(hold).unwrap_or_else(|e| {
            error!("Serialization error: {}", e); serde_json::Value::Null
        })))),
        Err(e) => {
            error!("Failed to create legal hold: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                409 => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn get_legal_hold(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let id = Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;

    match state.data_archiving_engine.get_legal_hold(id).await {
        Ok(Some(hold)) => Ok(Json(serde_json::to_value(hold).unwrap_or_else(|e| {
            error!("Serialization error: {}", e); serde_json::Value::Null
        }))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get legal hold: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn list_legal_holds(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<ListHoldsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.data_archiving_engine.list_legal_holds(
        org_id, query.status.as_deref(),
    ).await {
        Ok(holds) => Ok(Json(serde_json::to_value(holds).unwrap_or_else(|e| {
            error!("Serialization error: {}", e); serde_json::Value::Null
        }))),
        Err(e) => {
            error!("Failed to list legal holds: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn release_legal_hold(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<String>,
    Json(payload): Json<ReleaseHoldRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let id = Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.data_archiving_engine.release_legal_hold(id, user_id, payload.reason.as_deref()).await {
        Ok(hold) => Ok(Json(serde_json::to_value(hold).unwrap_or_else(|e| {
            error!("Serialization error: {}", e); serde_json::Value::Null
        }))),
        Err(e) => {
            error!("Failed to release legal hold: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn delete_legal_hold(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let id = Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;

    match state.data_archiving_engine.delete_legal_hold(id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            error!("Failed to delete legal hold: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// ============================================================================
// Legal Hold Items
// ============================================================================

pub async fn add_legal_hold_items(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<String>,
    Json(payload): Json<AddHoldItemsRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let legal_hold_id = Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;

    let items: Vec<(String, Uuid)> = payload.items.into_iter()
        .filter_map(|item| {
            let record_id = Uuid::parse_str(&item.record_id).ok()?;
            Some((item.entity_type, record_id))
        })
        .collect();

    match state.data_archiving_engine.add_legal_hold_items(org_id, legal_hold_id, items).await {
        Ok(items) => Ok(Json(serde_json::to_value(items).unwrap_or_else(|e| {
            error!("Serialization error: {}", e); serde_json::Value::Null
        }))),
        Err(e) => {
            error!("Failed to add legal hold items: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn list_legal_hold_items(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let legal_hold_id = Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;

    match state.data_archiving_engine.list_legal_hold_items(legal_hold_id).await {
        Ok(items) => Ok(Json(serde_json::to_value(items).unwrap_or_else(|e| {
            error!("Serialization error: {}", e); serde_json::Value::Null
        }))),
        Err(e) => {
            error!("Failed to list legal hold items: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn remove_legal_hold_item(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let id = Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;

    match state.data_archiving_engine.remove_legal_hold_item(id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            error!("Failed to remove legal hold item: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn check_legal_hold(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<CheckHoldQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let record_id = Uuid::parse_str(&query.record_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    match state.data_archiving_engine.is_record_under_hold(org_id, &query.entity_type, record_id).await {
        Ok(is_held) => Ok(Json(serde_json::json!({
            "entityType": query.entity_type,
            "recordId": query.record_id,
            "isUnderHold": is_held,
        }))),
        Err(e) => {
            error!("Failed to check legal hold: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckHoldQuery {
    pub entity_type: String,
    pub record_id: String,
}

// ============================================================================
// Archive Operations
// ============================================================================

pub async fn execute_archive(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<ExecuteArchiveRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let policy_id = Uuid::parse_str(&payload.policy_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    match state.data_archiving_engine.execute_archive(
        org_id, policy_id, &payload.batch_number, Some(user_id),
    ).await {
        Ok(batch) => Ok((StatusCode::CREATED, Json(serde_json::to_value(batch).unwrap_or_else(|e| {
            error!("Serialization error: {}", e); serde_json::Value::Null
        })))),
        Err(e) => {
            error!("Failed to execute archive: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn get_archived_record(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let id = Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;

    match state.data_archiving_engine.get_archived_record(id).await {
        Ok(Some(record)) => Ok(Json(serde_json::to_value(record).unwrap_or_else(|e| {
            error!("Serialization error: {}", e); serde_json::Value::Null
        }))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get archived record: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn list_archived_records(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<ListArchivedQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.data_archiving_engine.list_archived_records(
        org_id, query.entity_type.as_deref(), query.status.as_deref(), query.limit,
    ).await {
        Ok(records) => Ok(Json(serde_json::to_value(records).unwrap_or_else(|e| {
            error!("Serialization error: {}", e); serde_json::Value::Null
        }))),
        Err(e) => {
            error!("Failed to list archived records: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn restore_archived_record(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let archived_id = Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.data_archiving_engine.restore_archived_record(org_id, archived_id, Some(user_id)).await {
        Ok(record) => Ok(Json(serde_json::to_value(record).unwrap_or_else(|e| {
            error!("Serialization error: {}", e); serde_json::Value::Null
        }))),
        Err(e) => {
            error!("Failed to restore archived record: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn purge_archived_record(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let archived_id = Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.data_archiving_engine.purge_archived_record(org_id, archived_id, Some(user_id)).await {
        Ok(record) => Ok(Json(serde_json::to_value(record).unwrap_or_else(|e| {
            error!("Serialization error: {}", e); serde_json::Value::Null
        }))),
        Err(e) => {
            error!("Failed to purge archived record: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// ============================================================================
// Archive Batches
// ============================================================================

pub async fn list_archive_batches(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<ListBatchesQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.data_archiving_engine.list_archive_batches(
        org_id, query.status.as_deref(),
    ).await {
        Ok(batches) => Ok(Json(serde_json::to_value(batches).unwrap_or_else(|e| {
            error!("Serialization error: {}", e); serde_json::Value::Null
        }))),
        Err(e) => {
            error!("Failed to list archive batches: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_archive_batch(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let id = Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;

    match state.data_archiving_engine.get_archive_batch(id).await {
        Ok(Some(batch)) => Ok(Json(serde_json::to_value(batch).unwrap_or_else(|e| {
            error!("Serialization error: {}", e); serde_json::Value::Null
        }))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get archive batch: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// ============================================================================
// Audit Trail
// ============================================================================

pub async fn list_archive_audit(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<ListAuditQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.data_archiving_engine.list_audit_entries(
        org_id, query.operation.as_deref(), query.entity_type.as_deref(), query.limit,
    ).await {
        Ok(entries) => Ok(Json(serde_json::to_value(entries).unwrap_or_else(|e| {
            error!("Serialization error: {}", e); serde_json::Value::Null
        }))),
        Err(e) => {
            error!("Failed to list archive audit: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// ============================================================================
// Dashboard
// ============================================================================

pub async fn get_data_archiving_dashboard(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.data_archiving_engine.get_dashboard(org_id).await {
        Ok(dashboard) => Ok(Json(serde_json::to_value(dashboard).unwrap_or_else(|e| {
            error!("Serialization error: {}", e); serde_json::Value::Null
        }))),
        Err(e) => {
            error!("Failed to get archiving dashboard: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
