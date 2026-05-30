//! Cash Receipt Management Handlers
//!
//! Oracle Fusion: Financials > Receivables > Receipts

use crate::handlers::{to_json, created_json};
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

// ============================================================================
// Batches
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateBatchRequest {
    pub batch_number: String,
    pub batch_name: String,
    pub description: Option<String>,
    pub receipt_method: Option<String>,
    pub bank_account_id: Option<Uuid>,
    pub currency_code: Option<String>,
}

pub async fn create_batch(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateBatchRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.financials.cash_receipt_engine.create_batch(
        org_id, &payload.batch_number, &payload.batch_name,
        payload.description.as_deref(),
        payload.receipt_method.as_deref().unwrap_or("bank"),
        payload.bank_account_id,
        payload.currency_code.as_deref().unwrap_or("USD"),
        Some(user_id),
    ).await {
        Ok(batch) => Ok(created_json(batch)),
        Err(e) => {
            error!("Failed to create receipt batch: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListBatchesQuery {
    pub status: Option<String>,
}

pub async fn list_batches(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListBatchesQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.financials.cash_receipt_engine.list_batches(org_id, query.status.as_deref()).await {
        Ok(batches) => Ok(Json(serde_json::json!({ "data": batches }))),
        Err(e) => {
            error!("Failed to list receipt batches: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn get_batch(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.financials.cash_receipt_engine.get_batch(id).await {
        Ok(Some(b)) => Ok(to_json(b)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Failed to get batch: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn confirm_batch(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.financials.cash_receipt_engine.confirm_batch(id).await {
        Ok(b) => Ok(to_json(b)),
        Err(e) => {
            error!("Failed to confirm batch: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn close_batch(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.financials.cash_receipt_engine.close_batch(id).await {
        Ok(b) => Ok(to_json(b)),
        Err(e) => {
            error!("Failed to close batch: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn cancel_batch(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.financials.cash_receipt_engine.cancel_batch(id).await {
        Ok(b) => Ok(to_json(b)),
        Err(e) => {
            error!("Failed to cancel batch: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn delete_batch(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    match state.financials.cash_receipt_engine.delete_batch(id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            error!("Failed to delete batch: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// ============================================================================
// Receipts
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateReceiptRequest {
    pub batch_id: Option<Uuid>,
    pub receipt_number: String,
    pub customer_id: Uuid,
    pub customer_name: Option<String>,
    pub customer_account_number: Option<String>,
    pub payment_method: Option<String>,
    pub amount: String,
    pub currency_code: Option<String>,
    pub exchange_rate: Option<String>,
    pub receipt_date: Option<chrono::NaiveDate>,
    pub maturity_date: Option<chrono::NaiveDate>,
    pub reference_number: Option<String>,
    pub bank_name: Option<String>,
    pub bank_branch: Option<String>,
    pub deposit_date: Option<chrono::NaiveDate>,
    pub notes: Option<String>,
}

pub async fn create_receipt(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateReceiptRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.financials.cash_receipt_engine.create_receipt(
        org_id, payload.batch_id, &payload.receipt_number,
        payload.customer_id, payload.customer_name.as_deref(),
        payload.customer_account_number.as_deref(),
        payload.payment_method.as_deref().unwrap_or("cash"),
        &payload.amount,
        payload.currency_code.as_deref().unwrap_or("USD"),
        payload.exchange_rate.as_deref(),
        payload.receipt_date.unwrap_or_else(|| chrono::Utc::now().date_naive()),
        payload.maturity_date, payload.reference_number.as_deref(),
        payload.bank_name.as_deref(), payload.bank_branch.as_deref(),
        payload.deposit_date, payload.notes.as_deref(),
        Some(user_id),
    ).await {
        Ok(receipt) => Ok(created_json(receipt)),
        Err(e) => {
            error!("Failed to create receipt: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) | atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn get_receipt(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.financials.cash_receipt_engine.get_receipt(id).await {
        Ok(Some(r)) => Ok(to_json(r)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Failed to get receipt: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListReceiptsQuery {
    pub batch_id: Option<Uuid>,
    pub customer_id: Option<Uuid>,
    pub status: Option<String>,
}

pub async fn list_receipts(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListReceiptsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.financials.cash_receipt_engine.list_receipts(
        org_id, query.batch_id, query.customer_id, query.status.as_deref(),
    ).await {
        Ok(receipts) => Ok(Json(serde_json::json!({ "data": receipts }))),
        Err(e) => {
            error!("Failed to list receipts: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn identify_receipt(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.financials.cash_receipt_engine.identify_receipt(id).await {
        Ok(r) => Ok(to_json(r)),
        Err(e) => {
            error!("Failed to identify receipt: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// ============================================================================
// Applications
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ApplyReceiptRequest {
    pub receipt_id: Uuid,
    pub invoice_id: Uuid,
    pub invoice_number: Option<String>,
    pub applied_amount: String,
    pub discount_taken: Option<String>,
    pub application_date: Option<chrono::NaiveDate>,
}

pub async fn apply_receipt(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<ApplyReceiptRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.financials.cash_receipt_engine.apply_receipt(
        org_id, payload.receipt_id, payload.invoice_id,
        payload.invoice_number.as_deref(),
        &payload.applied_amount,
        payload.discount_taken.as_deref().unwrap_or("0.00"),
        payload.application_date.unwrap_or_else(|| chrono::Utc::now().date_naive()),
        Some(user_id),
    ).await {
        Ok(app) => Ok(created_json(app)),
        Err(e) => {
            error!("Failed to apply receipt: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) | atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn unapply_receipt(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.financials.cash_receipt_engine.unapply_receipt(id, Some(user_id)).await {
        Ok(app) => Ok(to_json(app)),
        Err(e) => {
            error!("Failed to unapply receipt: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListApplicationsQuery {
    pub status: Option<String>,
}

pub async fn list_applications(
    State(state): State<Arc<AppState>>,
    Path(receipt_id): Path<Uuid>,
    Query(query): Query<ListApplicationsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.financials.cash_receipt_engine.list_applications(
        receipt_id, query.status.as_deref(),
    ).await {
        Ok(apps) => Ok(Json(serde_json::json!({ "data": apps }))),
        Err(e) => {
            error!("Failed to list applications: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// ============================================================================
// Reversal
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ReverseReceiptRequest {
    pub reason: String,
}

pub async fn reverse_receipt(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<ReverseReceiptRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.financials.cash_receipt_engine.reverse_receipt(id, &payload.reason).await {
        Ok(r) => Ok(to_json(r)),
        Err(e) => {
            error!("Failed to reverse receipt: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) | atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// ============================================================================
// Dashboard
// ============================================================================

pub async fn get_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.financials.cash_receipt_engine.get_dashboard(org_id).await {
        Ok(dashboard) => Ok(to_json(dashboard)),
        Err(e) => { error!("Failed to get dashboard: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}
