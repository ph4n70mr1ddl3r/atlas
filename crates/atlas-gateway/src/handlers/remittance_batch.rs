//! Remittance Batch API Handlers
//!
//! Oracle Fusion Cloud ERP: Receivables > Receipts > Automatic Receipts > Remittance Batches
//!
//! Endpoints for managing remittance batches, batch receipts, lifecycle transitions,
//! and remittance advice.

use axum::{
    extract::{Path, Query, State, Extension},
    Json,
    http::StatusCode,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;
use uuid::Uuid;

use crate::AppState;
use crate::handlers::auth::{Claims, parse_uuid};

// ============================================================================
// Query Parameters
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct BatchListQuery {
    pub status: Option<String>,
    pub currency_code: Option<String>,
    pub remittance_method: Option<String>,
}

// ============================================================================
// Batches
// ============================================================================

/// Create a remittance batch
pub async fn create_batch(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    let batch_name = body["batch_name"].as_str();
    let bank_account_id = body["bank_account_id"].as_str().and_then(|s| Uuid::parse_str(s).ok());
    let bank_account_name = body["bank_account_name"].as_str();
    let bank_name = body["bank_name"].as_str();
    let remittance_method = body["remittance_method"].as_str().unwrap_or("standard");
    let currency_code = body["currency_code"].as_str().unwrap_or("USD");
    let batch_date = body["batch_date"].as_str()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .ok_or_else(|| (StatusCode::BAD_REQUEST, Json(json!({"error": "batch_date is required (YYYY-MM-DD)"}))))?;
    let gl_date = body["gl_date"].as_str().and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let receipt_currency_code = body["receipt_currency_code"].as_str();
    let exchange_rate_type = body["exchange_rate_type"].as_str();
    let format_program = body["format_program"].as_str();
    let notes = body["notes"].as_str();

    match state.remittance_batch_engine.create_batch(
        org_id, batch_name,
        bank_account_id, bank_account_name, bank_name,
        remittance_method, currency_code, batch_date, gl_date,
        receipt_currency_code, exchange_rate_type,
        format_program, notes, parse_uuid(&claims.sub).ok(),
    ).await {
        Ok(batch) => Ok((StatusCode::CREATED, Json(serde_json::to_value(batch).unwrap_or(Value::Null)))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Get a batch by ID
pub async fn get_batch(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.remittance_batch_engine.get_batch(id).await {
        Ok(Some(batch)) => Ok(Json(serde_json::to_value(batch).unwrap_or(Value::Null))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(json!({"error": "Remittance batch not found"})))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Get a batch by number
pub async fn get_batch_by_number(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(batch_number): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    match state.remittance_batch_engine.get_batch_by_number(org_id, &batch_number).await {
        Ok(Some(batch)) => Ok(Json(serde_json::to_value(batch).unwrap_or(Value::Null))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(json!({"error": "Remittance batch not found"})))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// List batches
pub async fn list_batches(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<BatchListQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    match state.remittance_batch_engine.list_batches(
        org_id, params.status.as_deref(), params.currency_code.as_deref(), params.remittance_method.as_deref(),
    ).await {
        Ok(batches) => Ok(Json(json!({"data": batches}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Lifecycle Actions
// ============================================================================

/// Approve a batch
pub async fn approve_batch(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.remittance_batch_engine.approve_batch(id).await {
        Ok(batch) => Ok(Json(serde_json::to_value(batch).unwrap_or(Value::Null))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Format a batch
pub async fn format_batch(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.remittance_batch_engine.format_batch(id).await {
        Ok(batch) => Ok(Json(serde_json::to_value(batch).unwrap_or(Value::Null))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Transmit a batch
pub async fn transmit_batch(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let reference_number = body["reference_number"].as_str();
    match state.remittance_batch_engine.transmit_batch(id, reference_number).await {
        Ok(batch) => Ok(Json(serde_json::to_value(batch).unwrap_or(Value::Null))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Confirm a batch
pub async fn confirm_batch(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.remittance_batch_engine.confirm_batch(id).await {
        Ok(batch) => Ok(Json(serde_json::to_value(batch).unwrap_or(Value::Null))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Settle a batch
pub async fn settle_batch(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.remittance_batch_engine.settle_batch(id).await {
        Ok(batch) => Ok(Json(serde_json::to_value(batch).unwrap_or(Value::Null))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Reverse a batch
pub async fn reverse_batch(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let reason = body["reason"].as_str();
    match state.remittance_batch_engine.reverse_batch(id, reason).await {
        Ok(batch) => Ok(Json(serde_json::to_value(batch).unwrap_or(Value::Null))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Cancel a batch
pub async fn cancel_batch(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let reason = body["reason"].as_str();
    match state.remittance_batch_engine.cancel_batch(id, reason).await {
        Ok(batch) => Ok(Json(serde_json::to_value(batch).unwrap_or(Value::Null))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Batch Receipts
// ============================================================================

/// Add a receipt to a batch
pub async fn add_receipt(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(batch_id): Path<Uuid>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    let receipt_id = body["receipt_id"].as_str()
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| (StatusCode::BAD_REQUEST, Json(json!({"error": "receipt_id is required"}))))?;
    let receipt_number = body["receipt_number"].as_str();
    let customer_id = body["customer_id"].as_str().and_then(|s| Uuid::parse_str(s).ok());
    let customer_number = body["customer_number"].as_str();
    let customer_name = body["customer_name"].as_str();
    let receipt_date = body["receipt_date"].as_str().and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let receipt_amount = body["receipt_amount"].as_str().unwrap_or("0.00");
    let applied_amount = body["applied_amount"].as_str().unwrap_or("0.00");
    let receipt_method = body["receipt_method"].as_str();
    let currency_code = body["currency_code"].as_str().unwrap_or("USD");
    let exchange_rate = body["exchange_rate"].as_str();
    let metadata = body.get("metadata").cloned().unwrap_or(serde_json::json!({}));

    match state.remittance_batch_engine.add_receipt(
        org_id, batch_id, receipt_id, receipt_number,
        customer_id, customer_number, customer_name,
        receipt_date, receipt_amount, applied_amount,
        receipt_method, currency_code, exchange_rate, metadata,
    ).await {
        Ok(receipt) => Ok((StatusCode::CREATED, Json(serde_json::to_value(receipt).unwrap_or(Value::Null)))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// List batch receipts
pub async fn list_batch_receipts(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(batch_id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.remittance_batch_engine.list_batch_receipts(batch_id).await {
        Ok(receipts) => Ok(Json(json!({"data": receipts}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Remove a receipt from a batch
pub async fn remove_receipt(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path((batch_id, receipt_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.remittance_batch_engine.remove_receipt(batch_id, receipt_id).await {
        Ok(()) => Ok(Json(json!({"message": "Receipt removed from batch"}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Remittance Advice
// ============================================================================

/// Mark remittance advice as sent
pub async fn mark_advice_sent(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.remittance_batch_engine.mark_advice_sent(id).await {
        Ok(batch) => Ok(Json(serde_json::to_value(batch).unwrap_or(Value::Null))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Dashboard
// ============================================================================

/// Get remittance batch summary dashboard
pub async fn get_batch_summary(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    match state.remittance_batch_engine.get_batch_summary(org_id).await {
        Ok(summary) => Ok(Json(serde_json::to_value(summary).unwrap_or(Value::Null))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}
