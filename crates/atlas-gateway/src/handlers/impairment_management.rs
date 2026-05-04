//! Impairment Management Handlers
//!
//! Oracle Fusion Cloud ERP: Financials > Fixed Assets > Impairment Management

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
pub struct CreateIndicatorRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub indicator_type: String,
    pub severity: String,
}

pub async fn create_impairment_indicator(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateIndicatorRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.impairment_management_engine.create_indicator(
        org_id, &payload.code, &payload.name, payload.description.as_deref(),
        &payload.indicator_type, &payload.severity, Some(user_id),
    ).await {
        Ok(indicator) => Ok((StatusCode::CREATED, Json(serde_json::to_value(indicator).unwrap()))),
        Err(e) => {
            error!("Failed to create indicator: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn list_impairment_indicators(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.impairment_management_engine.list_indicators(org_id, false).await {
        Ok(indicators) => Ok(Json(serde_json::json!({ "data": indicators }))),
        Err(e) => { error!("Failed to list indicators: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateImpairmentTestRequest {
    pub name: String,
    pub description: Option<String>,
    pub test_type: String,
    pub test_method: String,
    pub test_date: chrono::NaiveDate,
    pub carrying_amount: String,
    pub indicator_id: Option<Uuid>,
    pub asset_id: Option<Uuid>,
    pub discount_rate: Option<String>,
}

pub async fn create_impairment_test(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateImpairmentTestRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.impairment_management_engine.create_test(
        org_id, &payload.name, payload.description.as_deref(),
        &payload.test_type, &payload.test_method, payload.test_date,
        None, payload.indicator_id, &payload.carrying_amount,
        None, None, payload.asset_id, None,
        payload.discount_rate.as_deref(), None, Some(user_id),
    ).await {
        Ok(test) => Ok((StatusCode::CREATED, Json(serde_json::to_value(test).unwrap()))),
        Err(e) => { error!("Failed to create test: {}", e); Err(StatusCode::BAD_REQUEST) }
    }
}

pub async fn list_impairment_tests(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.impairment_management_engine.list_tests(org_id, None).await {
        Ok(tests) => Ok(Json(serde_json::json!({ "data": tests }))),
        Err(e) => { error!("Failed to list tests: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn submit_impairment_test(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.impairment_management_engine.submit_test(id, Some(user_id)).await {
        Ok(test) => Ok(Json(serde_json::to_value(test).unwrap())),
        Err(e) => { error!("Failed to submit test: {}", e); Err(StatusCode::BAD_REQUEST) }
    }
}

pub async fn approve_impairment_test(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.impairment_management_engine.approve_test(id, Some(user_id)).await {
        Ok(test) => Ok(Json(serde_json::to_value(test).unwrap())),
        Err(e) => { error!("Failed to approve test: {}", e); Err(StatusCode::BAD_REQUEST) }
    }
}

pub async fn get_impairment_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.impairment_management_engine.get_dashboard(org_id).await {
        Ok(dashboard) => Ok(Json(serde_json::to_value(dashboard).unwrap())),
        Err(e) => { error!("Failed to get dashboard: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}
