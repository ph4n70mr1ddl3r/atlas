//! Prepayment Application Handlers
//!
//! Oracle Fusion: Payables > Prepayment Application

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
pub struct ApplyPrepaymentRequest {
    pub prepayment_invoice_id: Uuid,
    pub prepayment_invoice_number: Option<String>,
    pub standard_invoice_id: Uuid,
    pub standard_invoice_number: Option<String>,
    pub supplier_id: Uuid,
    pub supplier_number: Option<String>,
    pub applied_amount: String,
    pub remaining_prepayment_amount: String,
    pub currency_code: String,
    pub application_date: chrono::NaiveDate,
    pub gl_date: chrono::NaiveDate,
    pub reason: Option<String>,
    pub notes: Option<String>,
}

pub async fn apply_prepayment(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<ApplyPrepaymentRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.prepayment_application_engine.apply(
        org_id, payload.prepayment_invoice_id,
        payload.prepayment_invoice_number.as_deref(),
        payload.standard_invoice_id,
        payload.standard_invoice_number.as_deref(),
        payload.supplier_id, payload.supplier_number.as_deref(),
        &payload.applied_amount, &payload.remaining_prepayment_amount,
        &payload.currency_code, payload.application_date, payload.gl_date,
        payload.reason.as_deref(), payload.notes.as_deref(),
        Some(user_id),
    ).await {
        Ok(app) => Ok((StatusCode::CREATED, Json(serde_json::to_value(app).unwrap()))),
        Err(e) => {
            error!("Failed to apply prepayment: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn get_prepayment(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.prepayment_application_engine.get(id).await {
        Ok(Some(app)) => Ok(Json(serde_json::to_value(app).unwrap())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Failed to get prepayment: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListPrepaymentsQuery {
    pub status: Option<String>,
    pub supplier_id: Option<Uuid>,
}

pub async fn list_prepayments(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListPrepaymentsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.prepayment_application_engine.list(org_id, query.status.as_deref(), query.supplier_id).await {
        Ok(items) => Ok(Json(serde_json::json!({ "data": items }))),
        Err(e) => { error!("Failed to list prepayments: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn confirm_prepayment(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.prepayment_application_engine.confirm(id).await {
        Ok(app) => Ok(Json(serde_json::to_value(app).unwrap())),
        Err(e) => {
            error!("Failed to confirm prepayment: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) | atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn cancel_prepayment(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.prepayment_application_engine.cancel(id).await {
        Ok(app) => Ok(Json(serde_json::to_value(app).unwrap())),
        Err(e) => {
            error!("Failed to cancel prepayment: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) | atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn get_prepayment_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.prepayment_application_engine.get_dashboard(org_id).await {
        Ok(dash) => Ok(Json(serde_json::to_value(dash).unwrap())),
        Err(e) => { error!("Failed to get prepayment dashboard: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}
