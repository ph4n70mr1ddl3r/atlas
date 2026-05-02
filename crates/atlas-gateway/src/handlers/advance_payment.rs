//! Advance Payment Handlers
//!
//! Oracle Fusion: Financials > Accounts Payable > Advance Payments

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
pub struct CreateAdvanceRequest {
    pub advance_number: String,
    pub supplier_id: Uuid,
    pub supplier_name: String,
    pub supplier_site_id: Option<Uuid>,
    pub description: Option<String>,
    pub currency_code: String,
    pub advance_amount: String,
    pub exchange_rate: Option<String>,
    pub payment_method: Option<String>,
    pub prepayment_account_code: Option<String>,
    pub liability_account_code: Option<String>,
    pub advance_date: chrono::NaiveDate,
    pub due_date: Option<chrono::NaiveDate>,
    pub expiration_date: Option<chrono::NaiveDate>,
}

pub async fn create_advance(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateAdvanceRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.advance_payment_engine.create_advance(
        org_id, &payload.advance_number, payload.supplier_id, &payload.supplier_name,
        payload.supplier_site_id, payload.description.as_deref(), &payload.currency_code,
        &payload.advance_amount, payload.exchange_rate.as_deref(),
        payload.payment_method.as_deref(), payload.prepayment_account_code.as_deref(),
        payload.liability_account_code.as_deref(), payload.advance_date,
        payload.due_date, payload.expiration_date, Some(user_id),
    ).await {
        Ok(a) => Ok((StatusCode::CREATED, Json(serde_json::to_value(a).unwrap()))),
        Err(e) => {
            error!("Failed to create advance: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListAdvancesQuery { pub status: Option<String>, pub supplier_id: Option<Uuid> }

pub async fn list_advances(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListAdvancesQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.advance_payment_engine.list_advances(org_id, query.status.as_deref(), query.supplier_id).await {
        Ok(advances) => Ok(Json(serde_json::json!({ "data": advances }))),
        Err(e) => { error!("Failed to list advances: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn get_advance(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.advance_payment_engine.get_advance(id).await {
        Ok(Some(a)) => Ok(Json(serde_json::to_value(a).unwrap())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Failed to get advance: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn approve_advance(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.advance_payment_engine.approve_advance(id, user_id).await {
        Ok(a) => Ok(Json(serde_json::to_value(a).unwrap())),
        Err(e) => {
            error!("Failed to approve advance: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct PayAdvanceRequest { pub payment_reference: Option<String> }

pub async fn pay_advance(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<PayAdvanceRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.advance_payment_engine.pay_advance(id, payload.payment_reference.as_deref(), Some(user_id)).await {
        Ok(a) => Ok(Json(serde_json::to_value(a).unwrap())),
        Err(e) => {
            error!("Failed to pay advance: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CancelAdvanceRequest { pub reason: Option<String> }

pub async fn cancel_advance(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<CancelAdvanceRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.advance_payment_engine.cancel_advance(id, payload.reason.as_deref()).await {
        Ok(a) => Ok(Json(serde_json::to_value(a).unwrap())),
        Err(e) => {
            error!("Failed to cancel advance: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// Applications
#[derive(Debug, Deserialize)]
pub struct ApplyAdvanceRequest {
    pub advance_id: Uuid,
    pub invoice_id: Uuid,
    pub invoice_number: Option<String>,
    pub applied_amount: String,
    pub application_date: chrono::NaiveDate,
    pub gl_account_code: Option<String>,
}

pub async fn apply_to_invoice(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<ApplyAdvanceRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.advance_payment_engine.apply_to_invoice(
        org_id, payload.advance_id, payload.invoice_id,
        payload.invoice_number.as_deref(), &payload.applied_amount,
        payload.application_date, payload.gl_account_code.as_deref(), Some(user_id),
    ).await {
        Ok(app) => Ok((StatusCode::CREATED, Json(serde_json::to_value(app).unwrap()))),
        Err(e) => {
            error!("Failed to apply advance: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) | atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn get_advance_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.advance_payment_engine.get_dashboard(org_id).await {
        Ok(dashboard) => Ok(Json(serde_json::to_value(dashboard).unwrap())),
        Err(e) => { error!("Failed to get dashboard: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}
