//! Inflation Adjustment Handlers
//!
//! Oracle Fusion Cloud ERP: Financials > General Ledger > Inflation Adjustment

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
pub struct CreateIndexRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub country_code: String,
    pub currency_code: String,
    pub index_type: String,
    pub is_hyperinflationary: bool,
    pub hyperinflationary_start_date: Option<chrono::NaiveDate>,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
}

pub async fn create_inflation_index(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateIndexRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.inflation_adjustment_engine.create_index(
        org_id, &payload.code, &payload.name, payload.description.as_deref(),
        &payload.country_code, &payload.currency_code, &payload.index_type,
        payload.is_hyperinflationary, payload.hyperinflationary_start_date,
        payload.effective_from, payload.effective_to, Some(user_id),
    ).await {
        Ok(index) => Ok((StatusCode::CREATED, Json(serde_json::to_value(index).unwrap()))),
        Err(e) => {
            error!("Failed to create inflation index: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn list_inflation_indices(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.inflation_adjustment_engine.list_indices(org_id, None).await {
        Ok(indices) => Ok(Json(serde_json::json!({ "data": indices }))),
        Err(e) => { error!("Failed to list indices: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn get_inflation_index(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.inflation_adjustment_engine.get_index(id).await {
        Ok(Some(index)) => Ok(Json(serde_json::to_value(index).unwrap())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Failed to get index: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateIndexRateRequest {
    pub index_id: Uuid,
    pub period_start: chrono::NaiveDate,
    pub period_end: chrono::NaiveDate,
    pub index_value: String,
    pub cumulative_factor: String,
    pub period_factor: String,
    pub source: Option<String>,
}

pub async fn add_index_rate(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateIndexRateRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.inflation_adjustment_engine.add_index_rate(
        org_id, payload.index_id, payload.period_start, payload.period_end,
        &payload.index_value, &payload.cumulative_factor, &payload.period_factor,
        payload.source.as_deref(), Some(user_id),
    ).await {
        Ok(rate) => Ok((StatusCode::CREATED, Json(serde_json::to_value(rate).unwrap()))),
        Err(e) => { error!("Failed to add index rate: {}", e); Err(StatusCode::BAD_REQUEST) }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateAdjustmentRunRequest {
    pub index_id: Uuid,
    pub name: Option<String>,
    pub description: Option<String>,
    pub from_period: chrono::NaiveDate,
    pub to_period: chrono::NaiveDate,
    pub adjustment_method: String,
}

pub async fn create_adjustment_run(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateAdjustmentRunRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.inflation_adjustment_engine.create_run(
        org_id, payload.name.as_deref(), payload.description.as_deref(),
        payload.index_id, None, payload.from_period, payload.to_period,
        &payload.adjustment_method, Some(user_id),
    ).await {
        Ok(run) => Ok((StatusCode::CREATED, Json(serde_json::to_value(run).unwrap()))),
        Err(e) => { error!("Failed to create run: {}", e); Err(StatusCode::BAD_REQUEST) }
    }
}

pub async fn submit_adjustment_run(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.inflation_adjustment_engine.submit_run(id, Some(user_id)).await {
        Ok(run) => Ok(Json(serde_json::to_value(run).unwrap())),
        Err(e) => { error!("Failed to submit run: {}", e); Err(StatusCode::BAD_REQUEST) }
    }
}

pub async fn approve_adjustment_run(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.inflation_adjustment_engine.approve_run(id, Some(user_id)).await {
        Ok(run) => Ok(Json(serde_json::to_value(run).unwrap())),
        Err(e) => { error!("Failed to approve run: {}", e); Err(StatusCode::BAD_REQUEST) }
    }
}

pub async fn get_inflation_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.inflation_adjustment_engine.get_dashboard(org_id).await {
        Ok(dashboard) => Ok(Json(serde_json::to_value(dashboard).unwrap())),
        Err(e) => { error!("Failed to get dashboard: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}
