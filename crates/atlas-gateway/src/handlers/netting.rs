//! Netting Handlers
//!
//! Oracle Fusion Cloud ERP: Financials > Netting
//!
//! API endpoints for managing netting agreements, netting batches,
//! and netting settlements.

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

#[derive(Debug, Deserialize)]
pub struct CreateNettingAgreementRequest {
    pub agreement_number: String,
    pub name: String,
    pub description: Option<String>,
    pub partner_id: Uuid,
    pub partner_number: Option<String>,
    pub partner_name: Option<String>,
    pub currency_code: String,
    pub netting_direction: String,
    pub settlement_method: String,
    pub minimum_netting_amount: String,
    pub maximum_netting_amount: Option<String>,
    #[serde(default)]
    pub auto_select_transactions: bool,
    #[serde(default)]
    pub selection_criteria: serde_json::Value,
    pub netting_clearing_account: Option<String>,
    pub ap_clearing_account: Option<String>,
    pub ar_clearing_account: Option<String>,
    #[serde(default)]
    pub approval_required: bool,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
}

/// Create a netting agreement
pub async fn create_netting_agreement(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateNettingAgreementRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    info!("Creating netting agreement '{}' for org {}", payload.agreement_number, org_id);

    match state.netting_engine.create_agreement(
        org_id,
        &payload.agreement_number,
        &payload.name,
        payload.description.as_deref(),
        payload.partner_id,
        payload.partner_number.as_deref(),
        payload.partner_name.as_deref(),
        &payload.currency_code,
        &payload.netting_direction,
        &payload.settlement_method,
        &payload.minimum_netting_amount,
        payload.maximum_netting_amount.as_deref(),
        payload.auto_select_transactions,
        payload.selection_criteria.clone(),
        payload.netting_clearing_account.as_deref(),
        payload.ap_clearing_account.as_deref(),
        payload.ar_clearing_account.as_deref(),
        payload.approval_required,
        payload.effective_from,
        payload.effective_to,
        Some(user_id),
    ).await {
        Ok(agreement) => Ok((StatusCode::CREATED, Json(serde_json::to_value(agreement).unwrap()))),
        Err(e) => {
            error!("Failed to create netting agreement: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListAgreementsQuery {
    pub status: Option<String>,
}

/// List netting agreements
pub async fn list_netting_agreements(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListAgreementsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.netting_engine.list_agreements(org_id, query.status.as_deref()).await {
        Ok(agreements) => Ok(Json(serde_json::json!({ "data": agreements }))),
        Err(e) => {
            error!("Failed to list netting agreements: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get a netting agreement by ID
pub async fn get_netting_agreement(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.netting_engine.get_agreement(id).await {
        Ok(Some(agreement)) => Ok(Json(serde_json::to_value(agreement).unwrap())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get netting agreement: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Activate a netting agreement
pub async fn activate_netting_agreement(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.netting_engine.activate_agreement(id).await {
        Ok(agreement) => Ok(Json(serde_json::to_value(agreement).unwrap())),
        Err(e) => {
            error!("Failed to activate netting agreement: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateNettingBatchRequest {
    pub agreement_id: Uuid,
    pub settlement_date: chrono::NaiveDate,
    pub description: Option<String>,
}

/// Create a netting batch
pub async fn create_netting_batch(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateNettingBatchRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.netting_engine.create_batch(
        org_id,
        payload.agreement_id,
        payload.settlement_date,
        None,
        Some(user_id),
    ).await {
        Ok(batch) => Ok((StatusCode::CREATED, Json(serde_json::to_value(batch).unwrap()))),
        Err(e) => {
            error!("Failed to create netting batch: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

/// Submit a netting batch
pub async fn submit_netting_batch(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.netting_engine.submit_batch(id, Some(user_id)).await {
        Ok(batch) => Ok(Json(serde_json::to_value(batch).unwrap())),
        Err(e) => {
            error!("Failed to submit netting batch: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

/// Approve a netting batch
pub async fn approve_netting_batch(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.netting_engine.approve_batch(id, Some(user_id)).await {
        Ok(batch) => Ok(Json(serde_json::to_value(batch).unwrap())),
        Err(e) => {
            error!("Failed to approve netting batch: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

/// Settle a netting batch
pub async fn settle_netting_batch(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.netting_engine.settle_batch(id).await {
        Ok(batch) => Ok(Json(serde_json::to_value(batch).unwrap())),
        Err(e) => {
            error!("Failed to settle netting batch: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

/// Get netting dashboard
pub async fn get_netting_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.netting_engine.get_dashboard(org_id).await {
        Ok(dashboard) => Ok(Json(serde_json::to_value(dashboard).unwrap())),
        Err(e) => {
            error!("Failed to get netting dashboard: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
