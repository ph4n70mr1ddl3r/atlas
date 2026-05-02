//! Revenue Management (ASC 606 / IFRS 15) Handlers
//!
//! Oracle Fusion Cloud ERP: Financials > Revenue Management
//!
//! API endpoints for revenue contracts, performance obligations,
//! standalone selling prices, and revenue recognition events.

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
use tracing::{info, error};

// ============================================================================
// Contracts
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateContractRequest {
    pub contract_number: String,
    pub customer_id: Uuid,
    pub customer_name: String,
    pub description: Option<String>,
    pub transaction_price: String,
    pub currency_code: String,
    pub contract_start_date: chrono::NaiveDate,
    pub contract_end_date: Option<chrono::NaiveDate>,
}

pub async fn create_contract(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateContractRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.revenue_management_engine.create_contract(
        org_id, &payload.contract_number, payload.customer_id, &payload.customer_name,
        payload.description.as_deref(), &payload.transaction_price, &payload.currency_code,
        payload.contract_start_date, payload.contract_end_date, Some(user_id),
    ).await {
        Ok(c) => Ok((StatusCode::CREATED, Json(serde_json::to_value(c).unwrap()))),
        Err(e) => {
            error!("Failed to create revenue contract: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListContractsQuery {
    pub status: Option<String>,
}

pub async fn list_contracts(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListContractsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.revenue_management_engine.list_contracts(org_id, query.status.as_deref()).await {
        Ok(contracts) => Ok(Json(serde_json::json!({ "data": contracts }))),
        Err(e) => { error!("Failed to list contracts: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn get_contract(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(number): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.revenue_management_engine.get_contract(org_id, &number).await {
        Ok(Some(c)) => Ok(Json(serde_json::to_value(c).unwrap())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Failed to get contract: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn activate_contract(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.revenue_management_engine.activate_contract(id).await {
        Ok(c) => Ok(Json(serde_json::to_value(c).unwrap())),
        Err(e) => {
            error!("Failed to activate contract: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn cancel_contract(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.revenue_management_engine.cancel_contract(id).await {
        Ok(c) => Ok(Json(serde_json::to_value(c).unwrap())),
        Err(e) => {
            error!("Failed to cancel contract: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// ============================================================================
// Performance Obligations
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateObligationRequest {
    pub contract_id: Uuid,
    pub obligation_number: String,
    pub description: String,
    pub obligation_type: String,
    pub satisfaction_method: String,
    pub recognition_pattern: String,
    pub standalone_selling_price: String,
    pub recognition_start_date: Option<chrono::NaiveDate>,
    pub recognition_end_date: Option<chrono::NaiveDate>,
}

pub async fn create_obligation(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateObligationRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.revenue_management_engine.create_obligation(
        org_id, payload.contract_id, &payload.obligation_number,
        &payload.description, &payload.obligation_type,
        &payload.satisfaction_method, &payload.recognition_pattern,
        &payload.standalone_selling_price,
        payload.recognition_start_date, payload.recognition_end_date,
    ).await {
        Ok(o) => Ok((StatusCode::CREATED, Json(serde_json::to_value(o).unwrap()))),
        Err(e) => {
            error!("Failed to create obligation: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn list_obligations(
    State(state): State<Arc<AppState>>,
    Path(contract_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.revenue_management_engine.list_obligations(contract_id).await {
        Ok(obs) => Ok(Json(serde_json::json!({ "data": obs }))),
        Err(e) => { error!("Failed to list obligations: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// ============================================================================
// Transaction Price Allocation
// ============================================================================

pub async fn allocate_transaction_price(
    State(state): State<Arc<AppState>>,
    Path(contract_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.revenue_management_engine.allocate_transaction_price(contract_id).await {
        Ok(obs) => Ok(Json(serde_json::json!({ "data": obs }))),
        Err(e) => {
            error!("Failed to allocate: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) | atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// ============================================================================
// Standalone Selling Prices
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateSspRequest {
    pub item_code: String,
    pub item_name: String,
    pub estimation_method: String,
    pub price: String,
    pub currency_code: String,
    pub effective_from: chrono::NaiveDate,
    pub effective_to: Option<chrono::NaiveDate>,
}

pub async fn create_ssp(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateSspRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.revenue_management_engine.create_ssp(
        org_id, &payload.item_code, &payload.item_name, &payload.estimation_method,
        &payload.price, &payload.currency_code, payload.effective_from,
        payload.effective_to, Some(user_id),
    ).await {
        Ok(s) => Ok((StatusCode::CREATED, Json(serde_json::to_value(s).unwrap()))),
        Err(e) => {
            error!("Failed to create SSP: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn list_ssps(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.revenue_management_engine.list_ssps(org_id).await {
        Ok(ssps) => Ok(Json(serde_json::json!({ "data": ssps }))),
        Err(e) => { error!("Failed to list SSPs: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// ============================================================================
// Revenue Recognition Events
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct SatisfyObligationRequest {
    pub amount: String,
    pub recognition_date: chrono::NaiveDate,
    pub gl_account_code: Option<String>,
}

pub async fn satisfy_obligation(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(obligation_id): Path<Uuid>,
    Json(payload): Json<SatisfyObligationRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.revenue_management_engine.satisfy_obligation(
        obligation_id, &payload.amount, payload.recognition_date,
        payload.gl_account_code.as_deref(), Some(user_id),
    ).await {
        Ok(e) => Ok(Json(serde_json::to_value(e).unwrap())),
        Err(e) => {
            error!("Failed to satisfy obligation: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn list_recognition_events(
    State(state): State<Arc<AppState>>,
    Path(contract_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.revenue_management_engine.list_recognition_events(contract_id).await {
        Ok(events) => Ok(Json(serde_json::json!({ "data": events }))),
        Err(e) => { error!("Failed to list events: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// ============================================================================
// Dashboard
// ============================================================================

pub async fn get_revenue_management_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.revenue_management_engine.get_dashboard(org_id).await {
        Ok(dashboard) => Ok(Json(serde_json::to_value(dashboard).unwrap())),
        Err(e) => { error!("Failed to get revenue dashboard: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}
