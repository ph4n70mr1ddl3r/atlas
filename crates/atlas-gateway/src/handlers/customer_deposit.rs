//! Customer Deposit Handlers
//!
//! Oracle Fusion: Financials > Accounts Receivable > Customer Deposits

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
pub struct CreateDepositRequest {
    pub deposit_number: String,
    pub customer_id: Uuid,
    pub customer_name: String,
    pub customer_site_id: Option<Uuid>,
    pub description: Option<String>,
    pub currency_code: String,
    pub deposit_amount: String,
    pub exchange_rate: Option<String>,
    pub deposit_account_code: Option<String>,
    pub receivable_account_code: Option<String>,
    pub deposit_date: chrono::NaiveDate,
    pub expiration_date: Option<chrono::NaiveDate>,
}

pub async fn create_deposit(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateDepositRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.customer_deposit_engine.create_deposit(
        org_id, &payload.deposit_number, payload.customer_id, &payload.customer_name,
        payload.customer_site_id, payload.description.as_deref(), &payload.currency_code,
        &payload.deposit_amount, payload.exchange_rate.as_deref(),
        payload.deposit_account_code.as_deref(), payload.receivable_account_code.as_deref(),
        payload.deposit_date, payload.expiration_date, Some(user_id),
    ).await {
        Ok(d) => Ok((StatusCode::CREATED, Json(serde_json::to_value(d).unwrap()))),
        Err(e) => {
            error!("Failed to create deposit: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListDepositsQuery { pub status: Option<String>, pub customer_id: Option<Uuid> }

pub async fn list_deposits(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListDepositsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.customer_deposit_engine.list_deposits(org_id, query.status.as_deref(), query.customer_id).await {
        Ok(deposits) => Ok(Json(serde_json::json!({ "data": deposits }))),
        Err(e) => { error!("Failed to list deposits: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn get_deposit(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.customer_deposit_engine.get_deposit(id).await {
        Ok(Some(d)) => Ok(Json(serde_json::to_value(d).unwrap())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Failed to get deposit: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

#[derive(Debug, Deserialize)]
pub struct ReceiveDepositRequest { pub receipt_reference: Option<String> }

pub async fn receive_deposit(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<ReceiveDepositRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.customer_deposit_engine.receive_deposit(id, payload.receipt_reference.as_deref(), Some(user_id)).await {
        Ok(d) => Ok(Json(serde_json::to_value(d).unwrap())),
        Err(e) => {
            error!("Failed to receive deposit: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct RefundDepositRequest { pub refund_reference: Option<String> }

pub async fn refund_deposit(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<RefundDepositRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.customer_deposit_engine.refund_deposit(id, payload.refund_reference.as_deref(), Some(user_id)).await {
        Ok(d) => Ok(Json(serde_json::to_value(d).unwrap())),
        Err(e) => {
            error!("Failed to refund deposit: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) | atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CancelDepositRequest { pub reason: Option<String> }

pub async fn cancel_deposit(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<CancelDepositRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.customer_deposit_engine.cancel_deposit(id, payload.reason.as_deref()).await {
        Ok(d) => Ok(Json(serde_json::to_value(d).unwrap())),
        Err(e) => {
            error!("Failed to cancel deposit: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// Applications
#[derive(Debug, Deserialize)]
pub struct ApplyDepositRequest {
    pub deposit_id: Uuid,
    pub invoice_id: Uuid,
    pub invoice_number: Option<String>,
    pub applied_amount: String,
    pub application_date: chrono::NaiveDate,
    pub gl_account_code: Option<String>,
}

pub async fn apply_deposit_to_invoice(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<ApplyDepositRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.customer_deposit_engine.apply_to_invoice(
        org_id, payload.deposit_id, payload.invoice_id,
        payload.invoice_number.as_deref(), &payload.applied_amount,
        payload.application_date, payload.gl_account_code.as_deref(), Some(user_id),
    ).await {
        Ok(app) => Ok((StatusCode::CREATED, Json(serde_json::to_value(app).unwrap()))),
        Err(e) => {
            error!("Failed to apply deposit: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) | atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn get_deposit_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.customer_deposit_engine.get_dashboard(org_id).await {
        Ok(dashboard) => Ok(Json(serde_json::to_value(dashboard).unwrap())),
        Err(e) => { error!("Failed to get dashboard: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}
