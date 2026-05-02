//! Lockbox Processing Handlers
//!
//! Oracle Fusion: AR > Lockbox

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

#[derive(Debug, Deserialize)]
pub struct CreateBatchRequest {
    pub batch_number: String,
    pub lockbox_number: String,
    pub bank_name: Option<String>,
    pub deposit_date: chrono::NaiveDate,
    pub currency_code: String,
    pub source_file_name: Option<String>,
}

pub async fn create_batch(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateBatchRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.lockbox_engine.create_batch(
        org_id, &payload.batch_number, &payload.lockbox_number,
        payload.bank_name.as_deref(), payload.deposit_date,
        &payload.currency_code, payload.source_file_name.as_deref(), Some(user_id),
    ).await {
        Ok(b) => Ok((StatusCode::CREATED, Json(serde_json::to_value(b).unwrap()))),
        Err(e) => {
            error!("Failed to create lockbox batch: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn get_batch(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.lockbox_engine.get_batch_by_id(id).await {
        Ok(Some(b)) => Ok(Json(serde_json::to_value(b).unwrap())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Failed to get batch: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListBatchesQuery { pub status: Option<String> }

pub async fn list_batches(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListBatchesQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.lockbox_engine.list_batches(org_id, query.status.as_deref()).await {
        Ok(batches) => Ok(Json(serde_json::json!({ "data": batches }))),
        Err(e) => { error!("Failed to list batches: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn validate_batch(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.lockbox_engine.validate_batch(id).await {
        Ok(b) => Ok(Json(serde_json::to_value(b).unwrap())),
        Err(e) => {
            error!("Failed to validate batch: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn apply_batch(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.lockbox_engine.apply_batch(id).await {
        Ok(b) => Ok(Json(serde_json::to_value(b).unwrap())),
        Err(e) => {
            error!("Failed to apply batch: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// Receipts
#[derive(Debug, Deserialize)]
pub struct CreateReceiptRequest {
    pub receipt_number: String,
    pub customer_number: Option<String>,
    pub customer_id: Option<Uuid>,
    pub receipt_date: chrono::NaiveDate,
    pub receipt_amount: String,
    pub remittance_reference: Option<String>,
}

pub async fn create_receipt(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(batch_id): Path<Uuid>,
    Json(payload): Json<CreateReceiptRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.lockbox_engine.create_receipt(
        org_id, batch_id, &payload.receipt_number, payload.customer_number.as_deref(),
        payload.customer_id, payload.receipt_date, &payload.receipt_amount,
        payload.remittance_reference.as_deref(),
    ).await {
        Ok(r) => Ok((StatusCode::CREATED, Json(serde_json::to_value(r).unwrap()))),
        Err(e) => {
            error!("Failed to create receipt: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn list_receipts(
    State(state): State<Arc<AppState>>,
    Path(batch_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.lockbox_engine.list_receipts(batch_id).await {
        Ok(receipts) => Ok(Json(serde_json::json!({ "data": receipts }))),
        Err(e) => { error!("Failed to list receipts: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// Manual Application
#[derive(Debug, Deserialize)]
pub struct ManualApplyRequest {
    pub invoice_number: String,
    pub applied_amount: String,
}

pub async fn manual_apply_receipt(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(receipt_id): Path<Uuid>,
    Json(payload): Json<ManualApplyRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.lockbox_engine.manual_apply_receipt(
        receipt_id, &payload.invoice_number, &payload.applied_amount, Some(user_id),
    ).await {
        Ok(app) => Ok((StatusCode::CREATED, Json(serde_json::to_value(app).unwrap()))),
        Err(e) => {
            error!("Failed to apply receipt: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn list_applications(
    State(state): State<Arc<AppState>>,
    Path(receipt_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.lockbox_engine.list_applications(receipt_id).await {
        Ok(apps) => Ok(Json(serde_json::json!({ "data": apps }))),
        Err(e) => { error!("Failed to list applications: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// Transmission Formats
#[derive(Debug, Deserialize)]
pub struct CreateFormatRequest {
    pub format_code: String,
    pub name: String,
    pub description: Option<String>,
    pub format_type: String,
    pub field_delimiter: Option<String>,
    pub record_delimiter: Option<String>,
    pub header_identifier: Option<String>,
    pub detail_identifier: Option<String>,
    pub trailer_identifier: Option<String>,
}

pub async fn create_format(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateFormatRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.lockbox_engine.create_format(
        org_id, &payload.format_code, &payload.name, payload.description.as_deref(),
        &payload.format_type, payload.field_delimiter.as_deref(), payload.record_delimiter.as_deref(),
        payload.header_identifier.as_deref(), payload.detail_identifier.as_deref(),
        payload.trailer_identifier.as_deref(), Some(user_id),
    ).await {
        Ok(f) => Ok((StatusCode::CREATED, Json(serde_json::to_value(f).unwrap()))),
        Err(e) => {
            error!("Failed to create format: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn list_formats(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.lockbox_engine.list_formats(org_id).await {
        Ok(formats) => Ok(Json(serde_json::json!({ "data": formats }))),
        Err(e) => { error!("Failed to list formats: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// Dashboard
pub async fn get_lockbox_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.lockbox_engine.get_dashboard(org_id).await {
        Ok(dashboard) => Ok(Json(serde_json::to_value(dashboard).unwrap())),
        Err(e) => { error!("Failed to get dashboard: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}
