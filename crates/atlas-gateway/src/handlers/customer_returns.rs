//! Customer Returns Management Handlers
//!
//! Oracle Fusion Cloud ERP: Order Management > Returns > Return Material Authorization
//! API endpoints for return reasons, RMAs, return lines, credit memos, and dashboard.

use axum::{
    extract::{State, Path, Query},
    Json,
    http::StatusCode,
    Extension,
};
use serde::Deserialize;
use crate::AppState;
use crate::handlers::auth::{Claims, parse_uuid};
use std::sync::Arc;
use uuid::Uuid;
use tracing::error;

fn default_standard_return() -> String { "standard_return".to_string() }
#[allow(dead_code)]
fn default_return_to_stock() -> String { "return_to_stock".to_string() }

// ============================================================================
// Return Reasons
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateReturnReasonRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(default = "default_standard_return")]
    pub return_type: String,
    pub default_disposition: Option<String>,
    #[serde(default)]
    pub requires_approval: bool,
    #[serde(default = "default_true")]
    pub credit_issued_automatically: bool,
}

fn default_true() -> bool { true }

pub async fn create_return_reason(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreateReturnReasonRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    match state.customer_returns_engine.create_return_reason(
        parse_uuid(&claims.org_id)?,
        &req.code, &req.name, req.description.as_deref(),
        &req.return_type, req.default_disposition.as_deref(),
        req.requires_approval, req.credit_issued_automatically, None,
    ).await {
        Ok(reason) => Ok((StatusCode::CREATED, Json(serde_json::to_value(reason).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => {
            error!("Failed to create return reason: {}", e);
            Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

pub async fn get_return_reason(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(code): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.customer_returns_engine.get_return_reason(
        parse_uuid(&claims.org_id)?, &code,
    ).await {
        Ok(Some(reason)) => Ok(Json(serde_json::to_value(reason).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Not found"})))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()})))),
    }
}

#[derive(Debug, Deserialize)]
pub struct ListReturnReasonsQuery {
    pub return_type: Option<String>,
}

pub async fn list_return_reasons(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<ListReturnReasonsQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.customer_returns_engine.list_return_reasons(
        parse_uuid(&claims.org_id)?, query.return_type.as_deref(),
    ).await {
        Ok(reasons) => Ok(Json(serde_json::json!({"data": reasons}))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()})))),
    }
}

pub async fn delete_return_reason(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(code): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    match state.customer_returns_engine.delete_return_reason(
        parse_uuid(&claims.org_id)?, &code,
    ).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Return Authorizations (RMAs)
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateRmaRequest {
    pub customer_id: Uuid,
    pub customer_number: Option<String>,
    pub customer_name: Option<String>,
    #[serde(default = "default_standard_return")]
    pub return_type: String,
    pub reason_code: Option<String>,
    pub original_order_number: Option<String>,
    pub original_order_id: Option<Uuid>,
    pub customer_contact: Option<String>,
    pub customer_email: Option<String>,
    pub customer_phone: Option<String>,
    pub return_date: Option<chrono::NaiveDate>,
    pub expected_receipt_date: Option<chrono::NaiveDate>,
    #[serde(default = "default_usd")]
    pub currency_code: String,
    pub notes: Option<String>,
}

fn default_usd() -> String { "USD".to_string() }

pub async fn create_rma(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreateRmaRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let return_date = req.return_date.unwrap_or_else(|| chrono::Utc::now().date_naive());
    match state.customer_returns_engine.create_rma(
        parse_uuid(&claims.org_id)?,
        req.customer_id, req.customer_number.as_deref(), req.customer_name.as_deref(),
        &req.return_type, req.reason_code.as_deref(),
        req.original_order_number.as_deref(), req.original_order_id,
        req.customer_contact.as_deref(), req.customer_email.as_deref(), req.customer_phone.as_deref(),
        return_date, req.expected_receipt_date,
        &req.currency_code, req.notes.as_deref(), None,
    ).await {
        Ok(rma) => Ok((StatusCode::CREATED, Json(serde_json::to_value(rma).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => {
            error!("Failed to create RMA: {}", e);
            Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

pub async fn get_rma(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.customer_returns_engine.get_rma(id).await {
        Ok(Some(rma)) => Ok(Json(serde_json::to_value(rma).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Not found"})))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()})))),
    }
}

#[derive(Debug, Deserialize)]
pub struct ListRmasQuery {
    pub status: Option<String>,
    pub customer_id: Option<Uuid>,
    pub return_type: Option<String>,
}

pub async fn list_rmas(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<ListRmasQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.customer_returns_engine.list_rmas(
        parse_uuid(&claims.org_id)?,
        query.status.as_deref(), query.customer_id, query.return_type.as_deref(),
    ).await {
        Ok(rmas) => Ok(Json(serde_json::json!({"data": rmas}))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()})))),
    }
}

pub async fn submit_rma(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.customer_returns_engine.submit_rma(id).await {
        Ok(rma) => Ok(Json(serde_json::to_value(rma).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()})))),
    }
}

#[derive(Debug, Deserialize)]
pub struct ApproveRejectRmaRequest {
    pub reason: Option<String>,
}

pub async fn approve_rma(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = parse_uuid(&claims.sub)?;
    match state.customer_returns_engine.approve_rma(id, user_id).await {
        Ok(rma) => Ok(Json(serde_json::to_value(rma).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()})))),
    }
}

pub async fn reject_rma(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<ApproveRejectRmaRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let reason = req.reason.as_deref().unwrap_or("");
    match state.customer_returns_engine.reject_rma(id, reason).await {
        Ok(rma) => Ok(Json(serde_json::to_value(rma).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()})))),
    }
}

pub async fn cancel_rma(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.customer_returns_engine.cancel_rma(id).await {
        Ok(rma) => Ok(Json(serde_json::to_value(rma).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Return Lines
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct AddReturnLineRequest {
    pub item_id: Option<Uuid>,
    pub item_code: Option<String>,
    pub item_description: Option<String>,
    pub original_line_id: Option<Uuid>,
    pub original_quantity: String,
    pub return_quantity: String,
    pub unit_price: String,
    pub reason_code: Option<String>,
    pub disposition: Option<String>,
    pub lot_number: Option<String>,
    pub serial_number: Option<String>,
    pub condition: Option<String>,
    pub notes: Option<String>,
}

pub async fn add_return_line(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(rma_id): Path<Uuid>,
    Json(req): Json<AddReturnLineRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    match state.customer_returns_engine.add_return_line(
        parse_uuid(&claims.org_id)?, rma_id,
        req.item_id, req.item_code.as_deref(), req.item_description.as_deref(),
        req.original_line_id, &req.original_quantity, &req.return_quantity,
        &req.unit_price, req.reason_code.as_deref(), req.disposition.as_deref(),
        req.lot_number.as_deref(), req.serial_number.as_deref(),
        req.condition.as_deref(), req.notes.as_deref(), None,
    ).await {
        Ok(line) => Ok((StatusCode::CREATED, Json(serde_json::to_value(line).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => {
            error!("Failed to add return line: {}", e);
            Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

pub async fn list_return_lines(
    State(state): State<Arc<AppState>>,
    Path(rma_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.customer_returns_engine.list_return_lines(rma_id).await {
        Ok(lines) => Ok(Json(serde_json::json!({"data": lines}))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()})))),
    }
}

#[derive(Debug, Deserialize)]
pub struct ReceiveReturnLineRequest {
    pub received_quantity: String,
}

pub async fn receive_return_line(
    State(state): State<Arc<AppState>>,
    Path(line_id): Path<Uuid>,
    Json(req): Json<ReceiveReturnLineRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.customer_returns_engine.receive_return_line(line_id, &req.received_quantity).await {
        Ok(line) => Ok(Json(serde_json::to_value(line).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()})))),
    }
}

#[derive(Debug, Deserialize)]
pub struct InspectReturnLineRequest {
    pub inspection_status: String,
    pub inspection_notes: Option<String>,
    pub disposition: Option<String>,
}

pub async fn inspect_return_line(
    State(state): State<Arc<AppState>>,
    Path(line_id): Path<Uuid>,
    Json(req): Json<InspectReturnLineRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.customer_returns_engine.inspect_return_line(
        line_id, &req.inspection_status, req.inspection_notes.as_deref(), req.disposition.as_deref(),
    ).await {
        Ok(line) => Ok(Json(serde_json::to_value(line).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Credit Memos
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct GenerateCreditMemoRequest {
    pub gl_account_code: Option<String>,
}

pub async fn generate_credit_memo(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(rma_id): Path<Uuid>,
    Json(req): Json<GenerateCreditMemoRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    match state.customer_returns_engine.generate_credit_memo(
        rma_id, req.gl_account_code.as_deref(), None,
    ).await {
        Ok(memo) => Ok((StatusCode::CREATED, Json(serde_json::to_value(memo).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => {
            error!("Failed to generate credit memo: {}", e);
            Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

pub async fn get_credit_memo(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.customer_returns_engine.get_credit_memo(id).await {
        Ok(Some(memo)) => Ok(Json(serde_json::to_value(memo).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Not found"})))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()})))),
    }
}

#[derive(Debug, Deserialize)]
pub struct ListCreditMemosQuery {
    pub customer_id: Option<Uuid>,
    pub status: Option<String>,
}

pub async fn list_credit_memos(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<ListCreditMemosQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.customer_returns_engine.list_credit_memos(
        parse_uuid(&claims.org_id)?,
        query.customer_id, query.status.as_deref(),
    ).await {
        Ok(memos) => Ok(Json(serde_json::json!({"data": memos}))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()})))),
    }
}

pub async fn issue_credit_memo(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.customer_returns_engine.issue_credit_memo(id).await {
        Ok(memo) => Ok(Json(serde_json::to_value(memo).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()})))),
    }
}

pub async fn cancel_credit_memo(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.customer_returns_engine.cancel_credit_memo(id).await {
        Ok(memo) => Ok(Json(serde_json::to_value(memo).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Dashboard
// ============================================================================

pub async fn get_returns_dashboard(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.customer_returns_engine.get_dashboard_summary(
        parse_uuid(&claims.org_id)?,
    ).await {
        Ok(summary) => Ok(Json(serde_json::to_value(summary).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()})))),
    }
}
