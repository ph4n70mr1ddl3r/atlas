//! Payment Process Request API Handlers
//!
//! REST endpoints for Oracle Fusion-inspired Payment Process Requests (PPR).
//! Manages automated batch payment processing with full lifecycle:
//! draft → submitted → `selection_complete` → formatted → confirmed → cancelled

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
// Request CRUD Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePprRequest {
    pub request_name: String,
    pub description: Option<String>,
    pub payment_date: String,
    pub gl_date: String,
    pub payment_method: Option<String>,
    pub currency_code: Option<String>,
    pub exchange_rate_type: Option<String>,
    pub exchange_rate: Option<f64>,
    pub selection_criteria: Option<String>,
    pub due_date_from: Option<String>,
    pub due_date_to: Option<String>,
    pub supplier_id: Option<String>,
    pub supplier_name: Option<String>,
    pub pay_group: Option<String>,
    pub minimum_amount: Option<f64>,
    pub maximum_amount: Option<f64>,
    pub include_on_hold: Option<bool>,
    pub take_discount: Option<bool>,
    pub pay_only_due: Option<bool>,
    pub bank_account_id: Option<String>,
    pub bank_account_name: Option<String>,
    pub payment_document: Option<String>,
}

pub async fn create_request(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreatePprRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    let user_id = parse_uuid(&claims.sub).ok();
    let payment_date = chrono::NaiveDate::parse_from_str(&req.payment_date, "%Y-%m-%d")
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": format!("Invalid payment_date: {}", e)}))))?;
    let gl_date = chrono::NaiveDate::parse_from_str(&req.gl_date, "%Y-%m-%d")
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": format!("Invalid gl_date: {}", e)}))))?;

    let due_from = req.due_date_from.as_deref()
        .map(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d"))
        .transpose()
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": format!("Invalid due_date_from: {}", e)}))))?;
    let due_to = req.due_date_to.as_deref()
        .map(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d"))
        .transpose()
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": format!("Invalid due_date_to: {}", e)}))))?;

    let supplier_id = req.supplier_id.as_deref().and_then(|s| s.parse::<Uuid>().ok());
    let bank_account_id = req.bank_account_id.as_deref().and_then(|s| s.parse::<Uuid>().ok());

    match state.financials.payment_process_request_engine.create_request(
        org_id,
        &req.request_name,
        req.description.as_deref(),
        payment_date,
        gl_date,
        req.payment_method.as_deref().unwrap_or("electronic"),
        req.currency_code.as_deref().unwrap_or("USD"),
        req.exchange_rate_type.as_deref(),
        req.exchange_rate,
        req.selection_criteria.as_deref().unwrap_or("all_open"),
        due_from,
        due_to,
        supplier_id,
        req.supplier_name.as_deref(),
        req.pay_group.as_deref(),
        req.minimum_amount,
        req.maximum_amount,
        req.include_on_hold.unwrap_or(false),
        req.take_discount.unwrap_or(true),
        req.pay_only_due.unwrap_or(false),
        bank_account_id,
        req.bank_account_name.as_deref(),
        req.payment_document.as_deref(),
        user_id,
    ).await {
        Ok(ppr) => Ok((StatusCode::CREATED, Json(serde_json::to_value(ppr).unwrap_or(serde_json::Value::Null)))),
        Err(e) => {
            error!("Failed to create PPR: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListPprQuery {
    pub status: Option<String>,
}

pub async fn list_requests(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<ListPprQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;

    match state.financials.payment_process_request_engine.list_requests(
        org_id,
        query.status.as_deref(),
    ).await {
        Ok(requests) => Ok(Json(serde_json::json!({"data": requests}))),
        Err(e) => {
            error!("Failed to list PPRs: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

pub async fn get_request(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;

    match state.financials.payment_process_request_engine.get_request(org_id, id).await {
        Ok(ppr) => Ok(Json(serde_json::to_value(ppr).unwrap_or(serde_json::Value::Null))),
        Err(e) => {
            error!("Failed to get PPR: {}", e);
            let status = if matches!(e, atlas_shared::AtlasError::EntityNotFound(_)) {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
            };
            Err((status, Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

pub async fn get_request_by_number(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(number): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;

    match state.financials.payment_process_request_engine.get_request_by_number(org_id, &number).await {
        Ok(ppr) => Ok(Json(serde_json::to_value(ppr).unwrap_or(serde_json::Value::Null))),
        Err(e) => {
            error!("Failed to get PPR by number: {}", e);
            let status = if matches!(e, atlas_shared::AtlasError::EntityNotFound(_)) {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
            };
            Err((status, Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

pub async fn delete_request(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(number): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;

    match state.financials.payment_process_request_engine.delete_request(org_id, &number).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            error!("Failed to delete PPR: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

// ============================================================================
// Lifecycle Handlers
// ============================================================================

macro_rules! lifecycle_handler {
    ($name:ident, $method:ident, $action:expr) => {
        pub async fn $name(
            State(state): State<Arc<AppState>>,
            Extension(claims): Extension<Claims>,
            Path(id): Path<Uuid>,
        ) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
            let org_id = match parse_uuid(&claims.org_id) {
                Ok(id) => id,
                Err(e) => return Err(e),
            };
            let user_id = match parse_uuid(&claims.sub) {
                Ok(id) => id,
                Err(e) => return Err(e),
            };

            match state.financials.payment_process_request_engine.$method(org_id, id, user_id).await {
                Ok(ppr) => Ok(Json(serde_json::to_value(ppr).unwrap_or(serde_json::Value::Null))),
                Err(e) => {
                    error!("Failed to {} PPR: {}", $action, e);
                    Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                         Json(serde_json::json!({"error": e.to_string()}))))
                }
            }
        }
    };
}

lifecycle_handler!(submit_request, submit_request, "submit");
lifecycle_handler!(complete_selection, complete_selection, "complete selection for");
lifecycle_handler!(format_payments, format_payments, "format");
lifecycle_handler!(confirm_payments, confirm_payments, "confirm");

#[derive(Debug, Deserialize)]
pub struct CancelRequest {
    pub reason: Option<String>,
}

pub async fn cancel_request(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(req): Json<CancelRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    let user_id = parse_uuid(&claims.sub)?;

    match state.financials.payment_process_request_engine.cancel_request(
        org_id, id, user_id, req.reason.as_deref()
    ).await {
        Ok(ppr) => Ok(Json(serde_json::to_value(ppr).unwrap_or(serde_json::Value::Null))),
        Err(e) => {
            error!("Failed to cancel PPR: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

// ============================================================================
// Document Management Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddDocumentRequest {
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
    pub amount_to_pay: f64,
    pub discount_available: Option<f64>,
    pub discount_taken: Option<f64>,
    pub discount_date: Option<String>,
    pub currency_code: Option<String>,
    pub liability_account: Option<String>,
    pub discount_account: Option<String>,
    pub cash_account: Option<String>,
}

pub async fn add_document(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(ppr_id): Path<Uuid>,
    Json(req): Json<AddDocumentRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    let invoice_id = match parse_uuid(&req.invoice_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "Invalid invoice_id"})))),
    };
    let supplier_id = req.supplier_id.as_deref().and_then(|s| s.parse::<Uuid>().ok());
    let invoice_date = req.invoice_date.as_deref()
        .map(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d"))
        .transpose()
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": format!("Invalid invoice_date: {}", e)}))))?;
    let discount_date = req.discount_date.as_deref()
        .map(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d"))
        .transpose()
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": format!("Invalid discount_date: {}", e)}))))?;

    match state.financials.payment_process_request_engine.add_document(
        org_id, ppr_id,
        invoice_id, req.invoice_number.as_deref(), invoice_date, req.invoice_amount,
        supplier_id, req.supplier_number.as_deref(), req.supplier_name.as_deref(),
        req.supplier_site.as_deref(),
        req.original_amount, req.amount_due, req.amount_to_pay,
        req.discount_available.unwrap_or(0.0), req.discount_taken.unwrap_or(0.0), discount_date,
        req.currency_code.as_deref(),
        req.liability_account.as_deref(), req.discount_account.as_deref(), req.cash_account.as_deref(),
    ).await {
        Ok(doc) => Ok((StatusCode::CREATED, Json(serde_json::to_value(doc).unwrap_or(serde_json::Value::Null)))),
        Err(e) => {
            error!("Failed to add document to PPR: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

pub async fn list_documents(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(ppr_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.financials.payment_process_request_engine.list_documents(ppr_id).await {
        Ok(docs) => Ok(Json(serde_json::json!({"data": docs}))),
        Err(e) => {
            error!("Failed to list PPR documents: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

pub async fn remove_document(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path((ppr_id, doc_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;

    match state.financials.payment_process_request_engine.remove_document(org_id, ppr_id, doc_id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            error!("Failed to remove document from PPR: {}", e);
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
    Path(ppr_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.financials.payment_process_request_engine.list_activities(ppr_id).await {
        Ok(activities) => Ok(Json(serde_json::json!({"data": activities}))),
        Err(e) => {
            error!("Failed to list PPR activities: {}", e);
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

    match state.financials.payment_process_request_engine.get_dashboard(org_id).await {
        Ok(dashboard) => Ok(Json(serde_json::to_value(dashboard).unwrap_or(serde_json::Value::Null))),
        Err(e) => {
            error!("Failed to get PPR dashboard: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}
