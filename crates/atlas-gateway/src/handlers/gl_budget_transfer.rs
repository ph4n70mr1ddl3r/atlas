//! GL Budget Transfer Handlers
//!
//! Oracle Fusion: General Ledger > Budget Transfers

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
pub struct CreateBudgetTransferRequest {
    pub description: Option<String>,
    pub transfer_date: chrono::NaiveDate,
    pub effective_date: chrono::NaiveDate,
    pub budget_name: Option<String>,
    pub transfer_type: String,
    pub from_account_combination: Option<String>,
    pub from_department: Option<String>,
    pub from_period: Option<String>,
    pub to_account_combination: Option<String>,
    pub to_department: Option<String>,
    pub to_period: Option<String>,
    pub transfer_amount: String,
    pub currency_code: String,
    pub reason: Option<String>,
}

pub async fn create_budget_transfer(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateBudgetTransferRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.gl_budget_transfer_engine.create(
        org_id, payload.description.as_deref(),
        payload.transfer_date, payload.effective_date,
        payload.budget_name.as_deref(), &payload.transfer_type,
        payload.from_account_combination.as_deref(), payload.from_department.as_deref(),
        payload.from_period.as_deref(),
        payload.to_account_combination.as_deref(), payload.to_department.as_deref(),
        payload.to_period.as_deref(),
        &payload.transfer_amount, &payload.currency_code,
        payload.reason.as_deref(), Some(user_id),
    ).await {
        Ok(t) => Ok((StatusCode::CREATED, Json(serde_json::to_value(t).unwrap()))),
        Err(e) => {
            error!("Failed to create budget transfer: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn get_budget_transfer(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.gl_budget_transfer_engine.get(id).await {
        Ok(Some(t)) => Ok(Json(serde_json::to_value(t).unwrap())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Failed to get budget transfer: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListBudgetTransfersQuery {
    pub status: Option<String>,
    pub transfer_type: Option<String>,
}

pub async fn list_budget_transfers(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListBudgetTransfersQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.gl_budget_transfer_engine.list(org_id, query.status.as_deref(), query.transfer_type.as_deref()).await {
        Ok(items) => Ok(Json(serde_json::json!({ "data": items }))),
        Err(e) => { error!("Failed to list budget transfers: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn submit_budget_transfer(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.gl_budget_transfer_engine.submit(id).await {
        Ok(t) => Ok(Json(serde_json::to_value(t).unwrap())),
        Err(e) => {
            error!("Failed to submit budget transfer: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) | atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn approve_budget_transfer(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.gl_budget_transfer_engine.approve(id, user_id).await {
        Ok(t) => Ok(Json(serde_json::to_value(t).unwrap())),
        Err(e) => {
            error!("Failed to approve budget transfer: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) | atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn complete_budget_transfer(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.gl_budget_transfer_engine.complete(id).await {
        Ok(t) => Ok(Json(serde_json::to_value(t).unwrap())),
        Err(e) => {
            error!("Failed to complete budget transfer: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) | atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn get_budget_transfer_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.gl_budget_transfer_engine.get_dashboard(org_id).await {
        Ok(dash) => Ok(Json(serde_json::to_value(dash).unwrap())),
        Err(e) => { error!("Failed to get budget transfer dashboard: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}
