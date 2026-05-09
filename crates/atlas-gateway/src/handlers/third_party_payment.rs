//! Third-Party Payment Handlers
//!
//! Oracle Fusion: Financials > Payables > Third-Party Payments

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
pub struct CreatePaymentRequest {
    pub payment_number: String,
    pub payment_type: String,
    pub source_entity_type: String,
    pub source_entity_id: Uuid,
    pub source_entity_name: String,
    pub payee_name: String,
    pub payee_tax_id: Option<String>,
    pub payee_address: Option<String>,
    pub payee_bank_account: Option<String>,
    pub amount: String,
    pub currency_code: Option<String>,
    pub payment_method: Option<String>,
    pub payment_date: Option<chrono::NaiveDate>,
    pub due_date: Option<chrono::NaiveDate>,
    pub reference_document: Option<String>,
    pub reference_document_id: Option<Uuid>,
    pub description: Option<String>,
    pub case_number: Option<String>,
    pub court_jurisdiction: Option<String>,
    pub is_recurring: Option<bool>,
    pub recurrence_frequency: Option<String>,
    pub recurrence_start_date: Option<chrono::NaiveDate>,
    pub recurrence_end_date: Option<chrono::NaiveDate>,
}

pub async fn create_payment(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreatePaymentRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.third_party_payment_engine.create_payment(
        org_id, &payload.payment_number, &payload.payment_type,
        &payload.source_entity_type, payload.source_entity_id, &payload.source_entity_name,
        &payload.payee_name, payload.payee_tax_id.as_deref(),
        payload.payee_address.as_deref(), payload.payee_bank_account.as_deref(),
        &payload.amount, payload.currency_code.as_deref().unwrap_or("USD"),
        payload.payment_method.as_deref(), payload.payment_date, payload.due_date,
        payload.reference_document.as_deref(), payload.reference_document_id,
        payload.description.as_deref(), payload.case_number.as_deref(),
        payload.court_jurisdiction.as_deref(),
        payload.is_recurring.unwrap_or(false),
        payload.recurrence_frequency.as_deref(),
        payload.recurrence_start_date, payload.recurrence_end_date,
        Some(user_id),
    ).await {
        Ok(p) => Ok((StatusCode::CREATED, Json(serde_json::to_value(p).unwrap()))),
        Err(e) => {
            error!("Failed to create third-party payment: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListPaymentsQuery {
    pub status: Option<String>,
    pub payment_type: Option<String>,
    pub source_entity_id: Option<Uuid>,
}

pub async fn list_payments(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListPaymentsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.third_party_payment_engine.list_payments(
        org_id, query.status.as_deref(), query.payment_type.as_deref(), query.source_entity_id,
    ).await {
        Ok(payments) => Ok(Json(serde_json::json!({ "data": payments }))),
        Err(e) => {
            error!("Failed to list third-party payments: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn get_payment(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.third_party_payment_engine.get_payment(id).await {
        Ok(Some(p)) => Ok(Json(serde_json::to_value(p).unwrap())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Failed to get third-party payment: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn submit_payment(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.third_party_payment_engine.submit_payment(id).await {
        Ok(p) => Ok(Json(serde_json::to_value(p).unwrap())),
        Err(e) => {
            error!("Failed to submit third-party payment: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn approve_payment(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.third_party_payment_engine.approve_payment(id, Some(user_id)).await {
        Ok(p) => Ok(Json(serde_json::to_value(p).unwrap())),
        Err(e) => {
            error!("Failed to approve third-party payment: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct RejectPaymentRequest { pub reason: String }

pub async fn reject_payment(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<RejectPaymentRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.third_party_payment_engine.reject_payment(id, &payload.reason, Some(user_id)).await {
        Ok(p) => Ok(Json(serde_json::to_value(p).unwrap())),
        Err(e) => {
            error!("Failed to reject third-party payment: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) | atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct HoldPaymentRequest { pub reason: Option<String> }

pub async fn place_on_hold(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<HoldPaymentRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.third_party_payment_engine.place_on_hold(id, payload.reason.as_deref(), Some(user_id)).await {
        Ok(p) => Ok(Json(serde_json::to_value(p).unwrap())),
        Err(e) => {
            error!("Failed to hold third-party payment: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn release_hold(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.third_party_payment_engine.release_hold(id).await {
        Ok(p) => Ok(Json(serde_json::to_value(p).unwrap())),
        Err(e) => {
            error!("Failed to release hold: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct RecordPaymentRequest { pub payment_reference: String }

pub async fn record_payment(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<RecordPaymentRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.third_party_payment_engine.record_payment(id, &payload.payment_reference, Some(user_id)).await {
        Ok(p) => Ok(Json(serde_json::to_value(p).unwrap())),
        Err(e) => {
            error!("Failed to record third-party payment: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) | atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CancelPaymentRequest { pub reason: Option<String> }

pub async fn cancel_payment(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<CancelPaymentRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.third_party_payment_engine.cancel_payment(id, payload.reason.as_deref(), Some(user_id)).await {
        Ok(p) => Ok(Json(serde_json::to_value(p).unwrap())),
        Err(e) => {
            error!("Failed to cancel third-party payment: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// ============================================================================
// Lines
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct AddLineRequest {
    pub payment_id: Uuid,
    pub line_number: i32,
    pub line_type: String,
    pub description: Option<String>,
    pub amount: String,
    pub gl_account: Option<String>,
    pub cost_center: Option<String>,
    pub tax_code: Option<String>,
}

pub async fn add_line(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<AddLineRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.third_party_payment_engine.add_line(
        org_id, payload.payment_id, payload.line_number,
        &payload.line_type, payload.description.as_deref(),
        &payload.amount, payload.gl_account.as_deref(),
        payload.cost_center.as_deref(), payload.tax_code.as_deref(),
    ).await {
        Ok(line) => Ok((StatusCode::CREATED, Json(serde_json::to_value(line).unwrap()))),
        Err(e) => {
            error!("Failed to add third-party payment line: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) | atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn list_lines(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.third_party_payment_engine.list_lines(id).await {
        Ok(lines) => Ok(Json(serde_json::json!({ "data": lines }))),
        Err(e) => { error!("Failed to list lines: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn remove_line(
    State(state): State<Arc<AppState>>,
    Path(line_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    match state.third_party_payment_engine.remove_line(line_id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => { error!("Failed to remove line: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn get_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.third_party_payment_engine.get_dashboard(org_id).await {
        Ok(dashboard) => Ok(Json(serde_json::to_value(dashboard).unwrap())),
        Err(e) => { error!("Failed to get dashboard: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}
