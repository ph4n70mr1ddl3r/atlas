//! Bank Account Transfer Handlers
//!
//! Oracle Fusion Cloud ERP: Financials > Cash Management > Bank Account Transfers

use axum::{
    extract::{State, Path},
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
pub struct CreateTransferTypeRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub settlement_method: String,
    pub requires_approval: Option<bool>,
    pub approval_threshold: Option<String>,
}

pub async fn create_bank_transfer_type(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateTransferTypeRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.bank_transfer_engine.create_transfer_type(
        org_id, &payload.code, &payload.name, payload.description.as_deref(),
        &payload.settlement_method, payload.requires_approval.unwrap_or(true),
        payload.approval_threshold.as_deref(), Some(user_id),
    ).await {
        Ok(tt) => Ok((StatusCode::CREATED, Json(serde_json::to_value(tt).unwrap()))),
        Err(e) => {
            error!("Failed to create transfer type: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateBankTransferRequest {
    pub transfer_type_id: Option<Uuid>,
    pub from_bank_account_id: Uuid,
    pub from_bank_account_number: Option<String>,
    pub from_bank_name: Option<String>,
    pub to_bank_account_id: Uuid,
    pub to_bank_account_number: Option<String>,
    pub to_bank_name: Option<String>,
    pub amount: String,
    pub currency_code: String,
    pub exchange_rate: Option<String>,
    pub from_currency: Option<String>,
    pub to_currency: Option<String>,
    pub transfer_date: chrono::NaiveDate,
    pub value_date: Option<chrono::NaiveDate>,
    pub reference_number: Option<String>,
    pub description: Option<String>,
    pub purpose: Option<String>,
    pub priority: Option<String>,
}

pub async fn create_bank_transfer(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateBankTransferRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.bank_transfer_engine.create_transfer(
        org_id, payload.transfer_type_id,
        payload.from_bank_account_id, payload.from_bank_account_number.as_deref(),
        payload.from_bank_name.as_deref(),
        payload.to_bank_account_id, payload.to_bank_account_number.as_deref(),
        payload.to_bank_name.as_deref(),
        &payload.amount, &payload.currency_code, payload.exchange_rate.as_deref(),
        payload.from_currency.as_deref(), payload.to_currency.as_deref(),
        payload.transfer_date, payload.value_date,
        payload.reference_number.as_deref(), payload.description.as_deref(),
        payload.purpose.as_deref(), payload.priority.as_deref().unwrap_or("normal"),
        Some(user_id),
    ).await {
        Ok(transfer) => Ok((StatusCode::CREATED, Json(serde_json::to_value(transfer).unwrap()))),
        Err(e) => { error!("Failed to create transfer: {}", e); Err(StatusCode::BAD_REQUEST) }
    }
}

pub async fn list_bank_transfers(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.bank_transfer_engine.list_transfers(org_id, None).await {
        Ok(transfers) => Ok(Json(serde_json::json!({ "data": transfers }))),
        Err(e) => { error!("Failed to list transfers: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn get_bank_transfer(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.bank_transfer_engine.get_transfer(id).await {
        Ok(Some(t)) => Ok(Json(serde_json::to_value(t).unwrap())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Failed to get transfer: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn submit_bank_transfer(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.bank_transfer_engine.submit_transfer(id, Some(user_id)).await {
        Ok(t) => Ok(Json(serde_json::to_value(t).unwrap())),
        Err(e) => { error!("Failed to submit transfer: {}", e); Err(StatusCode::BAD_REQUEST) }
    }
}

pub async fn approve_bank_transfer(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.bank_transfer_engine.approve_transfer(id, Some(user_id)).await {
        Ok(t) => Ok(Json(serde_json::to_value(t).unwrap())),
        Err(e) => { error!("Failed to approve transfer: {}", e); Err(StatusCode::BAD_REQUEST) }
    }
}

pub async fn complete_bank_transfer(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.bank_transfer_engine.complete_transfer(id, Some(user_id)).await {
        Ok(t) => Ok(Json(serde_json::to_value(t).unwrap())),
        Err(e) => { error!("Failed to complete transfer: {}", e); Err(StatusCode::BAD_REQUEST) }
    }
}

pub async fn get_bank_transfer_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.bank_transfer_engine.get_dashboard(org_id).await {
        Ok(dashboard) => Ok(Json(serde_json::to_value(dashboard).unwrap())),
        Err(e) => { error!("Failed to get dashboard: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}
