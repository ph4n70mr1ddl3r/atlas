//! Asset Reclassification Handlers
//!
//! Oracle Fusion: Fixed Assets > Asset Reclassification

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
pub struct CreateReclassificationRequest {
    pub asset_id: Uuid,
    pub asset_number: Option<String>,
    pub asset_name: Option<String>,
    pub reclassification_type: String,
    pub reason: Option<String>,
    pub from_category_code: Option<String>,
    pub from_asset_type: Option<String>,
    pub to_category_code: Option<String>,
    pub to_asset_type: Option<String>,
    pub to_depreciation_method: Option<String>,
    pub to_useful_life_months: Option<i32>,
    pub to_asset_account_code: Option<String>,
    pub effective_date: chrono::NaiveDate,
    pub amortization_adjustment: Option<String>,
    pub notes: Option<String>,
}

pub async fn create_reclassification(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateReclassificationRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.asset_reclassification_engine.create(
        org_id, payload.asset_id,
        payload.asset_number.as_deref(), payload.asset_name.as_deref(),
        &payload.reclassification_type, payload.reason.as_deref(),
        payload.from_category_code.as_deref(), payload.from_asset_type.as_deref(),
        None, None, None,
        payload.to_category_code.as_deref(), payload.to_asset_type.as_deref(),
        payload.to_depreciation_method.as_deref(), payload.to_useful_life_months,
        payload.to_asset_account_code.as_deref(),
        payload.effective_date, payload.amortization_adjustment.as_deref(),
        payload.notes.as_deref(), Some(user_id),
    ).await {
        Ok(r) => Ok((StatusCode::CREATED, Json(serde_json::to_value(r).unwrap()))),
        Err(e) => {
            error!("Failed to create reclassification: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn get_reclassification(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.asset_reclassification_engine.get(id).await {
        Ok(Some(r)) => Ok(Json(serde_json::to_value(r).unwrap())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Failed to get reclassification: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListReclassificationsQuery {
    pub status: Option<String>,
    pub asset_id: Option<Uuid>,
}

pub async fn list_reclassifications(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListReclassificationsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.asset_reclassification_engine.list(org_id, query.status.as_deref(), query.asset_id).await {
        Ok(items) => Ok(Json(serde_json::json!({ "data": items }))),
        Err(e) => { error!("Failed to list reclassifications: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn approve_reclassification(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.asset_reclassification_engine.approve(id, user_id).await {
        Ok(r) => Ok(Json(serde_json::to_value(r).unwrap())),
        Err(e) => {
            error!("Failed to approve reclassification: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) | atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn complete_reclassification(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.asset_reclassification_engine.complete(id).await {
        Ok(r) => Ok(Json(serde_json::to_value(r).unwrap())),
        Err(e) => {
            error!("Failed to complete reclassification: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) | atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn get_reclassification_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.asset_reclassification_engine.get_dashboard(org_id).await {
        Ok(dash) => Ok(Json(serde_json::to_value(dash).unwrap())),
        Err(e) => { error!("Failed to get reclassification dashboard: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}
