//! Suspense Account Processing Handlers
//!
//! Oracle Fusion: General Ledger > Suspense Accounts

use axum::{
    extract::{State, Path, Query},
    Json,
    http::StatusCode,
    Extension,
};
use serde::Deserialize;
use crate::AppState;
use crate::handlers::auth::Claims;
use std::sync::Arc;
use uuid::Uuid;
use tracing::error;

// ── Definitions ──────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateDefinitionRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub balancing_segment: String,
    pub suspense_account: String,
}

pub async fn create_definition(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateDefinitionRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.suspense_account_engine.create_definition(
        org_id, &payload.code, &payload.name, payload.description.as_deref(),
        &payload.balancing_segment, &payload.suspense_account, Some(user_id),
    ).await {
        Ok(d) => Ok((StatusCode::CREATED, Json(serde_json::to_value(d).unwrap()))),
        Err(e) => {
            error!("Failed to create suspense definition: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn get_definition(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.suspense_account_engine.get_definition(id).await {
        Ok(Some(d)) => Ok(Json(serde_json::to_value(d).unwrap())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Failed to get definition: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn list_definitions(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.suspense_account_engine.list_definitions(org_id).await {
        Ok(defs) => Ok(Json(serde_json::json!({ "data": defs }))),
        Err(e) => { error!("Failed to list definitions: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn activate_definition(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.suspense_account_engine.activate_definition(id).await {
        Ok(d) => Ok(Json(serde_json::to_value(d).unwrap())),
        Err(e) => {
            error!("Failed to activate definition: {}", e);
            Err(match e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn deactivate_definition(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.suspense_account_engine.deactivate_definition(id).await {
        Ok(d) => Ok(Json(serde_json::to_value(d).unwrap())),
        Err(e) => {
            error!("Failed to deactivate definition: {}", e);
            Err(match e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn delete_definition(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    match state.suspense_account_engine.delete_definition(id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            error!("Failed to delete definition: {}", e);
            Err(match e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// ── Entries ──────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateEntryRequest {
    pub definition_id: Uuid,
    pub journal_entry_id: Option<Uuid>,
    pub journal_batch_id: Option<Uuid>,
    pub balancing_segment_value: String,
    pub suspense_amount: String,
    pub original_amount: Option<String>,
    pub entry_type: String,
    pub entry_date: chrono::NaiveDate,
    pub currency_code: String,
}

pub async fn create_entry(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateEntryRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.suspense_account_engine.create_entry(
        org_id, payload.definition_id, payload.journal_entry_id, payload.journal_batch_id,
        &payload.balancing_segment_value, &payload.suspense_amount,
        payload.original_amount.as_deref(), &payload.entry_type, payload.entry_date,
        &payload.currency_code, Some(user_id),
    ).await {
        Ok(e) => Ok((StatusCode::CREATED, Json(serde_json::to_value(e).unwrap()))),
        Err(e) => {
            error!("Failed to create suspense entry: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn get_entry(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.suspense_account_engine.get_entry(id).await {
        Ok(Some(e)) => Ok(Json(serde_json::to_value(e).unwrap())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Failed to get entry: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListEntriesQuery { pub status: Option<String> }

pub async fn list_entries(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListEntriesQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.suspense_account_engine.list_entries(org_id, query.status.as_deref()).await {
        Ok(entries) => Ok(Json(serde_json::json!({ "data": entries }))),
        Err(e) => {
            error!("Failed to list entries: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn list_entries_by_definition(
    State(state): State<Arc<AppState>>,
    Path(def_id): Path<Uuid>,
    Query(query): Query<ListEntriesQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.suspense_account_engine.list_entries_by_definition(def_id, query.status.as_deref()).await {
        Ok(entries) => Ok(Json(serde_json::json!({ "data": entries }))),
        Err(e) => {
            error!("Failed to list entries by definition: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ReverseEntryRequest { pub resolution_notes: Option<String> }

pub async fn reverse_entry(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<ReverseEntryRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.suspense_account_engine.reverse_entry(id, payload.resolution_notes.as_deref()).await {
        Ok(e) => Ok(Json(serde_json::to_value(e).unwrap())),
        Err(e) => {
            error!("Failed to reverse entry: {}", e);
            Err(match e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct WriteOffEntryRequest { pub resolution_notes: Option<String> }

pub async fn write_off_entry(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<WriteOffEntryRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.suspense_account_engine.write_off_entry(id, payload.resolution_notes.as_deref()).await {
        Ok(e) => Ok(Json(serde_json::to_value(e).unwrap())),
        Err(e) => {
            error!("Failed to write off entry: {}", e);
            Err(match e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// ── Clearing Batches ─────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateClearingBatchRequest {
    pub batch_number: String,
    pub description: Option<String>,
    pub clearing_date: chrono::NaiveDate,
}

pub async fn create_clearing_batch(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateClearingBatchRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.suspense_account_engine.create_clearing_batch(
        org_id, &payload.batch_number, payload.description.as_deref(),
        payload.clearing_date, Some(user_id),
    ).await {
        Ok(b) => Ok((StatusCode::CREATED, Json(serde_json::to_value(b).unwrap()))),
        Err(e) => {
            error!("Failed to create clearing batch: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn get_clearing_batch(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.suspense_account_engine.get_clearing_batch(id).await {
        Ok(Some(b)) => Ok(Json(serde_json::to_value(b).unwrap())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Failed to get clearing batch: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListClearingBatchesQuery { pub status: Option<String> }

pub async fn list_clearing_batches(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListClearingBatchesQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.suspense_account_engine.list_clearing_batches(org_id, query.status.as_deref()).await {
        Ok(batches) => Ok(Json(serde_json::json!({ "data": batches }))),
        Err(e) => {
            error!("Failed to list clearing batches: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct AddClearingLineRequest {
    pub entry_id: Uuid,
    pub clearing_account: String,
    pub cleared_amount: String,
    pub resolution_notes: Option<String>,
}

pub async fn add_clearing_line(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(batch_id): Path<Uuid>,
    Json(payload): Json<AddClearingLineRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.suspense_account_engine.add_clearing_line(
        org_id, batch_id, payload.entry_id, &payload.clearing_account,
        &payload.cleared_amount, payload.resolution_notes.as_deref(),
    ).await {
        Ok(l) => Ok((StatusCode::CREATED, Json(serde_json::to_value(l).unwrap()))),
        Err(e) => {
            error!("Failed to add clearing line: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn list_clearing_lines(
    State(state): State<Arc<AppState>>,
    Path(batch_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.suspense_account_engine.list_clearing_lines(batch_id).await {
        Ok(lines) => Ok(Json(serde_json::json!({ "data": lines }))),
        Err(e) => { error!("Failed to list clearing lines: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn submit_clearing_batch(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.suspense_account_engine.submit_clearing_batch(id).await {
        Ok(b) => Ok(Json(serde_json::to_value(b).unwrap())),
        Err(e) => {
            error!("Failed to submit clearing batch: {}", e);
            Err(match e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn approve_clearing_batch(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.suspense_account_engine.approve_clearing_batch(id).await {
        Ok(b) => Ok(Json(serde_json::to_value(b).unwrap())),
        Err(e) => {
            error!("Failed to approve clearing batch: {}", e);
            Err(match e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn post_clearing_batch(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.suspense_account_engine.post_clearing_batch(id).await {
        Ok(b) => Ok(Json(serde_json::to_value(b).unwrap())),
        Err(e) => {
            error!("Failed to post clearing batch: {}", e);
            Err(match e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// ── Aging & Dashboard ────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateAgingSnapshotRequest { pub snapshot_date: chrono::NaiveDate }

pub async fn create_aging_snapshot(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateAgingSnapshotRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.suspense_account_engine.create_aging_snapshot(org_id, payload.snapshot_date).await {
        Ok(s) => Ok((StatusCode::CREATED, Json(serde_json::to_value(s).unwrap()))),
        Err(e) => { error!("Failed to create aging snapshot: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn get_suspense_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.suspense_account_engine.get_dashboard(org_id).await {
        Ok(d) => Ok(Json(serde_json::to_value(d).unwrap())),
        Err(e) => { error!("Failed to get dashboard: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}
