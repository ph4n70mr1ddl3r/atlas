//! Payment Settlement API Handlers
//!
//! REST endpoints for Oracle Fusion-inspired Payment Settlement & Clearing.
//! Manages settlement batches with full lifecycle:
//! draft → submitted → approved → settled → cancelled

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
use crate::handlers::auth::{Claims, parse_uuid};

// ============================================================================
// Batch CRUD Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateBatchRequest {
    pub batch_name: String,
    pub description: Option<String>,
    pub bank_account_id: Option<String>,
    pub bank_account_name: Option<String>,
    pub currency_code: Option<String>,
    pub exchange_rate_type: Option<String>,
    pub exchange_rate: Option<f64>,
    pub settlement_date: String,
    pub gl_date: String,
    pub settlement_method: Option<String>,
    pub settlement_type: Option<String>,
}

pub async fn create_batch(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreateBatchRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    let settlement_date = chrono::NaiveDate::parse_from_str(&req.settlement_date, "%Y-%m-%d")
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": format!("Invalid settlement_date: {}", e)}))))?;
    let gl_date = chrono::NaiveDate::parse_from_str(&req.gl_date, "%Y-%m-%d")
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": format!("Invalid gl_date: {}", e)}))))?;
    let bank_account_id = req.bank_account_id.as_deref().and_then(|s| s.parse::<Uuid>().ok());

    match state.financials.payment_settlement_engine.create_batch(
        org_id,
        &req.batch_name,
        req.description.as_deref(),
        bank_account_id,
        req.bank_account_name.as_deref(),
        req.currency_code.as_deref().unwrap_or("USD"),
        req.exchange_rate_type.as_deref(),
        req.exchange_rate,
        settlement_date,
        gl_date,
        req.settlement_method.as_deref().unwrap_or("electronic"),
        req.settlement_type.as_deref().unwrap_or("full"),
        None,
    ).await {
        Ok(batch) => Ok((StatusCode::CREATED, Json(serde_json::to_value(batch).unwrap_or(serde_json::Value::Null)))),
        Err(e) => {
            error!("Failed to create settlement batch: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListBatchesQuery {
    pub status: Option<String>,
}

pub async fn list_batches(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<ListBatchesQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;

    match state.financials.payment_settlement_engine.list_batches(
        org_id,
        query.status.as_deref(),
    ).await {
        Ok(batches) => Ok(Json(serde_json::json!({"data": batches}))),
        Err(e) => {
            error!("Failed to list settlement batches: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

pub async fn get_batch(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.financials.payment_settlement_engine.get_batch(id).await {
        Ok(Some(batch)) => Ok(Json(serde_json::to_value(batch).unwrap_or(serde_json::Value::Null))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Settlement batch not found"})))),
        Err(e) => {
            error!("Failed to get settlement batch: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

pub async fn get_batch_by_number(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(number): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    match state.financials.payment_settlement_engine.get_batch_by_number(org_id, &number).await {
        Ok(Some(batch)) => Ok(Json(serde_json::to_value(batch).unwrap_or(serde_json::Value::Null))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Settlement batch not found"})))),
        Err(e) => {
            error!("Failed to get settlement batch: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

pub async fn delete_batch(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(number): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    match state.financials.payment_settlement_engine.delete_batch(org_id, &number).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            error!("Failed to delete settlement batch: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

// ============================================================================
// Lifecycle Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubmitRequest {
    pub submitted_by: Option<String>,
}

pub async fn submit_batch(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(_req): Json<SubmitRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.financials.payment_settlement_engine.submit_batch(id, None).await {
        Ok(batch) => Ok(Json(serde_json::to_value(batch).unwrap_or(serde_json::Value::Null))),
        Err(e) => {
            error!("Failed to submit settlement batch: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApproveRequest {
    pub approved_by: Option<String>,
}

pub async fn approve_batch(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(_req): Json<ApproveRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.financials.payment_settlement_engine.approve_batch(id, None).await {
        Ok(batch) => Ok(Json(serde_json::to_value(batch).unwrap_or(serde_json::Value::Null))),
        Err(e) => {
            error!("Failed to approve settlement batch: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettleRequest {
    pub settled_by: Option<String>,
}

pub async fn settle_batch(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(_req): Json<SettleRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.financials.payment_settlement_engine.settle_batch(id, None).await {
        Ok(batch) => Ok(Json(serde_json::to_value(batch).unwrap_or(serde_json::Value::Null))),
        Err(e) => {
            error!("Failed to settle batch: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelRequest {
    pub reason: Option<String>,
}

pub async fn cancel_batch(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(req): Json<CancelRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.financials.payment_settlement_engine.cancel_batch(id, None, req.reason.as_deref()).await {
        Ok(batch) => Ok(Json(serde_json::to_value(batch).unwrap_or(serde_json::Value::Null))),
        Err(e) => {
            error!("Failed to cancel settlement batch: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

// ============================================================================
// Settlement Line Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddLineRequest {
    pub invoice_id: String,
    pub invoice_number: Option<String>,
    pub invoice_date: Option<String>,
    pub invoice_amount: f64,
    pub supplier_id: Option<String>,
    pub supplier_number: Option<String>,
    pub supplier_name: Option<String>,
    pub supplier_site: Option<String>,
    pub original_amount: f64,
    pub amount_due: f64,
    pub amount_paid: f64,
    pub discount_available: Option<f64>,
    pub discount_taken: Option<f64>,
    pub discount_date: Option<String>,
    pub bank_charges: Option<f64>,
    pub adjustment_amount: Option<f64>,
    pub adjustment_reason: Option<String>,
    pub settlement_type: Option<String>,
    pub liability_account: Option<String>,
    pub discount_account: Option<String>,
    pub charges_account: Option<String>,
}

pub async fn add_line(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(batch_id): Path<Uuid>,
    Json(req): Json<AddLineRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    let invoice_id = req.invoice_id.parse::<Uuid>()
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": format!("Invalid invoice_id: {}", e)}))))?;
    let invoice_date = req.invoice_date.as_deref()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let supplier_id = req.supplier_id.as_deref().and_then(|s| s.parse::<Uuid>().ok());
    let discount_date = req.discount_date.as_deref()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());

    match state.financials.payment_settlement_engine.add_line(
        org_id,
        batch_id,
        invoice_id,
        req.invoice_number.as_deref(),
        invoice_date,
        req.invoice_amount,
        supplier_id,
        req.supplier_number.as_deref(),
        req.supplier_name.as_deref(),
        req.supplier_site.as_deref(),
        req.original_amount,
        req.amount_due,
        req.amount_paid,
        req.discount_available.unwrap_or(0.0),
        req.discount_taken.unwrap_or(0.0),
        discount_date,
        req.bank_charges.unwrap_or(0.0),
        req.adjustment_amount.unwrap_or(0.0),
        req.adjustment_reason.as_deref(),
        req.settlement_type.as_deref().unwrap_or("full"),
        req.liability_account.as_deref(),
        req.discount_account.as_deref(),
        req.charges_account.as_deref(),
    ).await {
        Ok(line) => Ok((StatusCode::CREATED, Json(serde_json::to_value(line).unwrap_or(serde_json::Value::Null)))),
        Err(e) => {
            error!("Failed to add settlement line: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

pub async fn list_lines(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(batch_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.financials.payment_settlement_engine.list_lines(batch_id).await {
        Ok(lines) => Ok(Json(serde_json::json!({"data": lines}))),
        Err(e) => {
            error!("Failed to list settlement lines: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

pub async fn remove_line(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path((batch_id, line_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    match state.financials.payment_settlement_engine.remove_line(batch_id, line_id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            error!("Failed to remove settlement line: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

// ============================================================================
// Activity & Dashboard Handlers
// ============================================================================

pub async fn list_activities(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(batch_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.financials.payment_settlement_engine.list_activities(batch_id).await {
        Ok(activities) => Ok(Json(serde_json::json!({"data": activities}))),
        Err(e) => {
            error!("Failed to list settlement activities: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

pub async fn get_dashboard(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    match state.financials.payment_settlement_engine.get_dashboard(org_id).await {
        Ok(summary) => Ok(Json(serde_json::to_value(summary).unwrap_or(serde_json::Value::Null))),
        Err(e) => {
            error!("Failed to get settlement dashboard: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}
