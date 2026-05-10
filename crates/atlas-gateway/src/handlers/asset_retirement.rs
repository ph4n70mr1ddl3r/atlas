//! Asset Retirement Handlers
//!
//! Oracle Fusion Cloud ERP: Financials > Fixed Assets > Asset Retirements
//!
//! API endpoints for fixed asset retirement/disposal management:
//! - Create retirement requests (sale, scrap, donation, transfer, etc.)
//! - Approve, complete, reverse, and cancel retirements
//! - List and get retirements with filtering
//! - Dashboard summary statistics

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

// ============================================================================
// Create Retirement
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRetirementRequest {
    pub asset_id: Uuid,
    pub asset_number: Option<String>,
    pub asset_description: Option<String>,
    pub retirement_type: String,
    pub retirement_date: chrono::NaiveDate,
    pub cost: String,
    pub accumulated_depreciation: String,
    pub proceeds: String,
    pub removal_cost: String,
    pub gain_loss_account: Option<String>,
    pub asset_account: Option<String>,
    pub depreciation_account: Option<String>,
    pub proceeds_account: Option<String>,
    pub removal_cost_account: Option<String>,
    pub buyer_name: Option<String>,
    pub reason: Option<String>,
}

pub async fn create_retirement(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateRetirementRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.asset_retirement_engine.create(
        org_id,
        payload.asset_id,
        payload.asset_number.as_deref(),
        payload.asset_description.as_deref(),
        &payload.retirement_type,
        payload.retirement_date,
        &payload.cost,
        &payload.accumulated_depreciation,
        &payload.proceeds,
        &payload.removal_cost,
        payload.gain_loss_account.as_deref(),
        payload.asset_account.as_deref(),
        payload.depreciation_account.as_deref(),
        payload.proceeds_account.as_deref(),
        payload.removal_cost_account.as_deref(),
        payload.buyer_name.as_deref(),
        payload.reason.as_deref(),
        Some(user_id),
    ).await {
        Ok(ret) => Ok((StatusCode::CREATED, Json(serde_json::to_value(ret).unwrap_or_default()))),
        Err(e) => {
            error!("Failed to create asset retirement: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(msg) => {
                    Ok((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": msg}))))
                }
                atlas_shared::AtlasError::Conflict(msg) => {
                    Ok((StatusCode::CONFLICT, Json(serde_json::json!({"error": msg}))))
                }
                _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
    }
}

// ============================================================================
// Get / List
// ============================================================================

pub async fn get_retirement(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.asset_retirement_engine.get(id).await {
        Ok(Some(ret)) => Ok(Json(serde_json::to_value(ret).unwrap_or_default())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get asset retirement: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListRetirementsQuery {
    pub status: Option<String>,
    pub retirement_type: Option<String>,
}

pub async fn list_retirements(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListRetirementsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.asset_retirement_engine.list(
        org_id,
        query.status.as_deref(),
        query.retirement_type.as_deref(),
    ).await {
        Ok(retirements) => Ok(Json(serde_json::json!({ "data": retirements }))),
        Err(e) => {
            error!("Failed to list asset retirements: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(msg) => {
                    Ok(Json(serde_json::json!({"error": msg})))
                }
                _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
    }
}

// ============================================================================
// Workflow Actions
// ============================================================================

pub async fn approve_retirement(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.asset_retirement_engine.approve(id, user_id).await {
        Ok(ret) => Ok((StatusCode::OK, Json(serde_json::to_value(ret).unwrap_or_default()))),
        Err(e) => {
            error!("Failed to approve asset retirement: {}", e);
            match e {
                atlas_shared::AtlasError::EntityNotFound(_) => Err(StatusCode::NOT_FOUND),
                atlas_shared::AtlasError::WorkflowError(msg) => {
                    Ok((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": msg}))))
                }
                _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompleteRetirementRequest {
    pub gl_batch_id: Option<Uuid>,
}

pub async fn complete_retirement(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<CompleteRetirementRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    match state.asset_retirement_engine.complete(id, payload.gl_batch_id).await {
        Ok(ret) => Ok((StatusCode::OK, Json(serde_json::to_value(ret).unwrap_or_default()))),
        Err(e) => {
            error!("Failed to complete asset retirement: {}", e);
            match e {
                atlas_shared::AtlasError::EntityNotFound(_) => Err(StatusCode::NOT_FOUND),
                atlas_shared::AtlasError::WorkflowError(msg) => {
                    Ok((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": msg}))))
                }
                _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
    }
}

pub async fn reverse_retirement(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    match state.asset_retirement_engine.reverse(id).await {
        Ok(ret) => Ok((StatusCode::OK, Json(serde_json::to_value(ret).unwrap_or_default()))),
        Err(e) => {
            error!("Failed to reverse asset retirement: {}", e);
            match e {
                atlas_shared::AtlasError::EntityNotFound(_) => Err(StatusCode::NOT_FOUND),
                atlas_shared::AtlasError::WorkflowError(msg) => {
                    Ok((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": msg}))))
                }
                _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
    }
}

pub async fn cancel_retirement(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    match state.asset_retirement_engine.cancel(id).await {
        Ok(ret) => Ok((StatusCode::OK, Json(serde_json::to_value(ret).unwrap_or_default()))),
        Err(e) => {
            error!("Failed to cancel asset retirement: {}", e);
            match e {
                atlas_shared::AtlasError::EntityNotFound(_) => Err(StatusCode::NOT_FOUND),
                atlas_shared::AtlasError::WorkflowError(msg) => {
                    Ok((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": msg}))))
                }
                _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
    }
}

// ============================================================================
// Dashboard
// ============================================================================

pub async fn get_retirement_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.asset_retirement_engine.get_dashboard(org_id).await {
        Ok(dashboard) => Ok(Json(serde_json::to_value(dashboard).unwrap_or_default())),
        Err(e) => {
            error!("Failed to get retirement dashboard: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
