//! Cash Flow Forecasting Handlers
//!
//! Oracle Fusion: Treasury > Cash Forecasting

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
pub struct CreateForecastRequest {
    pub forecast_number: String,
    pub name: String,
    pub description: Option<String>,
    pub forecast_horizon: String,
    pub periods_out: i32,
    pub start_date: chrono::NaiveDate,
    pub end_date: chrono::NaiveDate,
    pub base_currency_code: String,
    pub opening_balance: String,
}

pub async fn create_forecast(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateForecastRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.cash_flow_forecast_engine.create_forecast(
        org_id, &payload.forecast_number, &payload.name, payload.description.as_deref(),
        &payload.forecast_horizon, payload.periods_out, payload.start_date, payload.end_date,
        &payload.base_currency_code, &payload.opening_balance, Some(user_id),
    ).await {
        Ok(f) => Ok((StatusCode::CREATED, Json(serde_json::to_value(f).unwrap()))),
        Err(e) => {
            error!("Failed to create forecast: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListForecastsQuery { pub status: Option<String> }

pub async fn list_forecasts(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListForecastsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.cash_flow_forecast_engine.list_forecasts(org_id, query.status.as_deref()).await {
        Ok(forecasts) => Ok(Json(serde_json::json!({ "data": forecasts }))),
        Err(e) => { error!("Failed to list forecasts: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn get_forecast(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.cash_flow_forecast_engine.get_forecast_by_id(id).await {
        Ok(Some(f)) => Ok(Json(serde_json::to_value(f).unwrap())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Failed to get forecast: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn activate_forecast(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.cash_flow_forecast_engine.activate_forecast(id).await {
        Ok(f) => Ok(Json(serde_json::to_value(f).unwrap())),
        Err(e) => {
            error!("Failed to activate forecast: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn approve_forecast(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.cash_flow_forecast_engine.approve_forecast(id, user_id).await {
        Ok(f) => Ok(Json(serde_json::to_value(f).unwrap())),
        Err(e) => {
            error!("Failed to approve forecast: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// Scenarios
#[derive(Debug, Deserialize)]
pub struct CreateScenarioRequest {
    pub forecast_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub scenario_type: String,
    pub adjustment_factor: String,
}

pub async fn create_scenario(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateScenarioRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let sn = format!("SC-{}", &Uuid::new_v4().to_string()[..8].to_uppercase());
    match state.cash_flow_forecast_engine.create_scenario(
        org_id, payload.forecast_id, &sn, &payload.name,
        payload.description.as_deref(), &payload.scenario_type, &payload.adjustment_factor,
    ).await {
        Ok(s) => Ok((StatusCode::CREATED, Json(serde_json::to_value(s).unwrap()))),
        Err(e) => {
            error!("Failed to create scenario: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn list_scenarios(
    State(state): State<Arc<AppState>>,
    Path(forecast_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.cash_flow_forecast_engine.list_scenarios(forecast_id).await {
        Ok(scenarios) => Ok(Json(serde_json::json!({ "data": scenarios }))),
        Err(e) => { error!("Failed to list scenarios: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// Entries
#[derive(Debug, Deserialize)]
pub struct CreateEntryRequest {
    pub forecast_id: Uuid,
    pub scenario_id: Option<Uuid>,
    pub period_name: String,
    pub period_start_date: chrono::NaiveDate,
    pub period_end_date: chrono::NaiveDate,
    pub source_category: String,
    pub flow_direction: String,
    pub amount: String,
    pub probability: String,
    pub is_manual: Option<bool>,
    pub description: Option<String>,
}

pub async fn create_entry(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateEntryRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.cash_flow_forecast_engine.create_entry(
        org_id, payload.forecast_id, payload.scenario_id,
        &payload.period_name, payload.period_start_date, payload.period_end_date,
        &payload.source_category, &payload.flow_direction,
        &payload.amount, &payload.probability,
        payload.is_manual.unwrap_or(true), payload.description.as_deref(),
    ).await {
        Ok(e) => Ok((StatusCode::CREATED, Json(serde_json::to_value(e).unwrap()))),
        Err(e) => {
            error!("Failed to create entry: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn list_entries(
    State(state): State<Arc<AppState>>,
    Path(forecast_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.cash_flow_forecast_engine.list_entries(forecast_id).await {
        Ok(entries) => Ok(Json(serde_json::json!({ "data": entries }))),
        Err(e) => { error!("Failed to list entries: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn get_cash_forecast_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.cash_flow_forecast_engine.get_dashboard(org_id).await {
        Ok(dashboard) => Ok(Json(serde_json::to_value(dashboard).unwrap())),
        Err(e) => { error!("Failed to get dashboard: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}
