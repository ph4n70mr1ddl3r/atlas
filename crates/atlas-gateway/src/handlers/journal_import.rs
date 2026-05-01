//! Journal Import Handlers
//!
//! Oracle Fusion Cloud ERP: Financials > General Ledger > Import Journals
//!
//! API endpoints for managing journal import formats, column mappings,
//! import batches, data validation, and import processing.

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
use tracing::{info, error};

#[derive(Debug, Deserialize)]
pub struct CreateImportFormatRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub source_type: String,
    pub file_format: String,
    pub delimiter: Option<String>,
    #[serde(default)]
    pub header_row: bool,
    pub ledger_id: Option<Uuid>,
    pub currency_code: String,
    pub default_date: Option<chrono::NaiveDate>,
    pub default_journal_type: Option<String>,
    pub balancing_segment: Option<String>,
    #[serde(default = "default_true")]
    pub validation_enabled: bool,
    #[serde(default)]
    pub auto_post: bool,
    #[serde(default = "default_max_errors")]
    pub max_errors_allowed: i32,
    #[serde(default)]
    pub column_mappings: serde_json::Value,
}

fn default_true() -> bool { true }
fn default_max_errors() -> i32 { 100 }

/// Create a journal import format
pub async fn create_import_format(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateImportFormatRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    info!("Creating journal import format '{}' for org {}", payload.code, org_id);

    match state.journal_import_engine.create_format(
        org_id,
        &payload.code,
        &payload.name,
        payload.description.as_deref(),
        &payload.source_type,
        &payload.file_format,
        payload.delimiter.as_deref(),
        payload.header_row,
        payload.ledger_id,
        &payload.currency_code,
        payload.default_date,
        payload.default_journal_type.as_deref(),
        payload.balancing_segment.as_deref(),
        payload.validation_enabled,
        payload.auto_post,
        payload.max_errors_allowed,
        payload.column_mappings.clone(),
        Some(user_id),
    ).await {
        Ok(format) => Ok((StatusCode::CREATED, Json(serde_json::to_value(format).unwrap()))),
        Err(e) => {
            error!("Failed to create import format: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListFormatsQuery {
    pub status: Option<String>,
}

/// List journal import formats
pub async fn list_import_formats(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListFormatsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.journal_import_engine.list_formats(org_id, query.status.as_deref()).await {
        Ok(formats) => Ok(Json(serde_json::json!({ "data": formats }))),
        Err(e) => {
            error!("Failed to list import formats: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get an import format by ID
pub async fn get_import_format(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.journal_import_engine.get_format(id).await {
        Ok(Some(format)) => Ok(Json(serde_json::to_value(format).unwrap())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get import format: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Delete (deactivate) an import format
pub async fn delete_import_format(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(code): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.journal_import_engine.delete_format(org_id, &code).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            error!("Failed to delete import format: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct AddColumnMappingRequest {
    pub column_position: i32,
    pub source_column: String,
    pub target_field: String,
    pub data_type: String,
    #[serde(default)]
    pub is_required: bool,
    pub default_value: Option<String>,
    pub transformation: Option<String>,
    pub validation_rule: Option<String>,
}

/// Add a column mapping to a format
pub async fn add_column_mapping(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(format_id): Path<Uuid>,
    Json(payload): Json<AddColumnMappingRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.journal_import_engine.add_column_mapping(
        org_id,
        format_id,
        payload.column_position,
        &payload.source_column,
        &payload.target_field,
        &payload.data_type,
        payload.is_required,
        payload.default_value.as_deref(),
        payload.transformation.as_deref(),
        payload.validation_rule.as_deref(),
    ).await {
        Ok(mapping) => Ok((StatusCode::CREATED, Json(serde_json::to_value(mapping).unwrap()))),
        Err(e) => {
            error!("Failed to add column mapping: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

/// List column mappings for a format
pub async fn list_column_mappings(
    State(state): State<Arc<AppState>>,
    Path(format_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.journal_import_engine.list_column_mappings(format_id).await {
        Ok(mappings) => Ok(Json(serde_json::json!({ "data": mappings }))),
        Err(e) => {
            error!("Failed to list column mappings: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateImportBatchRequest {
    pub format_id: Uuid,
    pub name: Option<String>,
    pub description: Option<String>,
    pub source: String,
    pub source_file_name: Option<String>,
}

/// Create a journal import batch
pub async fn create_import_batch(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateImportBatchRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.journal_import_engine.create_batch(
        org_id,
        payload.format_id,
        payload.name.as_deref(),
        payload.description.as_deref(),
        &payload.source,
        payload.source_file_name.as_deref(),
        Some(user_id),
    ).await {
        Ok(batch) => Ok((StatusCode::CREATED, Json(serde_json::to_value(batch).unwrap()))),
        Err(e) => {
            error!("Failed to create import batch: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct AddImportRowRequest {
    pub account_code: Option<String>,
    pub account_name: Option<String>,
    pub description: Option<String>,
    pub entered_dr: String,
    pub entered_cr: String,
    pub currency_code: Option<String>,
    pub exchange_rate: Option<String>,
    pub gl_date: Option<chrono::NaiveDate>,
    pub reference: Option<String>,
    pub line_type: Option<String>,
    pub cost_center: Option<String>,
    pub department: Option<String>,
    pub project_code: Option<String>,
    #[serde(default)]
    pub raw_data: serde_json::Value,
}

/// Add a row to an import batch
pub async fn add_import_row(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(batch_id): Path<Uuid>,
    Json(payload): Json<AddImportRowRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.journal_import_engine.add_row(
        org_id,
        batch_id,
        payload.raw_data.clone(),
        payload.account_code.as_deref(),
        payload.account_name.as_deref(),
        payload.description.as_deref(),
        &payload.entered_dr,
        &payload.entered_cr,
        payload.currency_code.as_deref(),
        payload.exchange_rate.as_deref(),
        payload.gl_date,
        payload.reference.as_deref(),
        payload.line_type.as_deref(),
        payload.cost_center.as_deref(),
        payload.department.as_deref(),
        payload.project_code.as_deref(),
        Some(user_id),
    ).await {
        Ok(row) => Ok((StatusCode::CREATED, Json(serde_json::to_value(row).unwrap()))),
        Err(e) => {
            error!("Failed to add import row: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

/// List rows in a batch
pub async fn list_import_rows(
    State(state): State<Arc<AppState>>,
    Path(batch_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.journal_import_engine.list_batch_rows(batch_id).await {
        Ok(rows) => Ok(Json(serde_json::json!({ "data": rows }))),
        Err(e) => {
            error!("Failed to list import rows: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Validate an import batch
pub async fn validate_import_batch(
    State(state): State<Arc<AppState>>,
    Path(batch_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.journal_import_engine.validate_batch(batch_id).await {
        Ok(batch) => Ok(Json(serde_json::to_value(batch).unwrap())),
        Err(e) => {
            error!("Failed to validate import batch: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

/// Import (process) a validated batch
pub async fn import_batch(
    State(state): State<Arc<AppState>>,
    Path(batch_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.journal_import_engine.import_batch(batch_id).await {
        Ok(batch) => Ok(Json(serde_json::to_value(batch).unwrap())),
        Err(e) => {
            error!("Failed to import batch: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

/// Get an import batch by ID
pub async fn get_import_batch(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.journal_import_engine.get_batch(id).await {
        Ok(Some(batch)) => Ok(Json(serde_json::to_value(batch).unwrap())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get import batch: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListBatchesQuery {
    pub format_id: Option<Uuid>,
    pub status: Option<String>,
}

/// List import batches
pub async fn list_import_batches(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListBatchesQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.journal_import_engine.list_batches(
        org_id, query.format_id, query.status.as_deref(),
    ).await {
        Ok(batches) => Ok(Json(serde_json::json!({ "data": batches }))),
        Err(e) => {
            error!("Failed to list import batches: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Delete an import batch
pub async fn delete_import_batch(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    match state.journal_import_engine.delete_batch(id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            error!("Failed to delete import batch: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

/// Get journal import dashboard
pub async fn get_journal_import_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.journal_import_engine.get_dashboard(org_id).await {
        Ok(dashboard) => Ok(Json(serde_json::to_value(dashboard).unwrap())),
        Err(e) => {
            error!("Failed to get journal import dashboard: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
