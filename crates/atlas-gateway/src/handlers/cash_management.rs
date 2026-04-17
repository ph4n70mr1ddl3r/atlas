//! Cash Position & Cash Forecasting Handlers
//!
//! Oracle Fusion Cloud ERP: Treasury > Cash Management
//!
//! API endpoints for managing cash positions, forecast templates,
//! forecast sources, cash forecasts, and forecast lines.

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
// Request Types
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct UpsertCashPositionRequest {
    pub bank_account_id: Uuid,
    pub account_number: String,
    pub account_name: String,
    #[serde(default = "default_usd")]
    pub currency_code: String,
    pub book_balance: String,
    pub available_balance: String,
    #[serde(default = "default_zero")]
    pub float_amount: String,
    #[serde(default = "default_zero")]
    pub one_day_float: String,
    #[serde(default = "default_zero")]
    pub two_day_float: String,
    pub position_date: chrono::NaiveDate,
    pub average_balance: Option<String>,
    pub prior_day_balance: Option<String>,
    #[serde(default = "default_zero")]
    pub projected_inflows: String,
    #[serde(default = "default_zero")]
    pub projected_outflows: String,
    #[serde(default = "default_zero")]
    pub projected_net: String,
    #[serde(default)]
    pub is_reconciled: bool,
}

fn default_usd() -> String { "USD".to_string() }
fn default_zero() -> String { "0".to_string() }

#[derive(Debug, Deserialize)]
pub struct CreateForecastTemplateRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(default = "default_monthly")]
    pub bucket_type: String,
    #[serde(default = "default_12")]
    pub number_of_periods: i32,
    #[serde(default)]
    pub start_offset_days: i32,
    #[serde(default)]
    pub is_default: bool,
    #[serde(default = "default_columns")]
    pub columns: serde_json::Value,
}

fn default_monthly() -> String { "monthly".to_string() }
fn default_12() -> i32 { 12 }
fn default_columns() -> serde_json::Value { serde_json::json!([]) }

#[derive(Debug, Deserialize)]
pub struct CreateForecastSourceRequest {
    pub template_code: String,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub source_type: String,
    pub cash_flow_direction: String,
    #[serde(default)]
    pub is_actual: bool,
    #[serde(default = "default_10")]
    pub display_order: i32,
    #[serde(default)]
    pub lead_time_days: i32,
    pub payment_terms_reference: Option<String>,
    pub account_code_filter: Option<String>,
}

fn default_10() -> i32 { 10 }

#[derive(Debug, Deserialize)]
pub struct GenerateForecastRequest {
    pub template_code: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListCashPositionsQuery {
    pub position_date: Option<chrono::NaiveDate>,
}

#[derive(Debug, Deserialize)]
pub struct ListForecastsQuery {
    pub template_id: Option<Uuid>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CashPositionSummaryQuery {
    pub position_date: Option<chrono::NaiveDate>,
}

#[derive(Debug, Deserialize)]
pub struct ForecastSummaryQuery {
    pub template_code: String,
}

// ============================================================================
// Cash Position Handlers
// ============================================================================

/// Create or update a cash position
pub async fn upsert_cash_position(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<UpsertCashPositionRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.cash_management_engine.upsert_cash_position(
        org_id, payload.bank_account_id, &payload.account_number, &payload.account_name,
        &payload.currency_code, &payload.book_balance, &payload.available_balance,
        &payload.float_amount, &payload.one_day_float, &payload.two_day_float,
        payload.position_date, payload.average_balance.as_deref(), payload.prior_day_balance.as_deref(),
        &payload.projected_inflows, &payload.projected_outflows, &payload.projected_net,
        payload.is_reconciled, Some(user_id),
    ).await {
        Ok(pos) => Ok((StatusCode::CREATED, Json(serde_json::to_value(pos).unwrap()))),
        Err(e) => {
            error!("Failed to upsert cash position: {}", e);
            Err(map_error(e))
        }
    }
}

/// Get cash position for a specific bank account and date
pub async fn get_cash_position(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(bank_account_id): Path<Uuid>,
    Query(params): Query<crate::handlers::cash_management::CashPositionSummaryQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let date = params.position_date.unwrap_or_else(|| chrono::Utc::now().date_naive());
    match state.cash_management_engine.get_cash_position(org_id, bank_account_id, date).await {
        Ok(Some(pos)) => Ok(Json(serde_json::to_value(pos).unwrap())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

/// List cash positions
pub async fn list_cash_positions(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListCashPositionsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.cash_management_engine.list_cash_positions(org_id, query.position_date).await {
        Ok(positions) => Ok(Json(serde_json::to_value(positions).unwrap())),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

/// Get cash position summary (aggregated across all accounts)
pub async fn get_cash_position_summary(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<CashPositionSummaryQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let date = query.position_date.unwrap_or_else(|| chrono::Utc::now().date_naive());
    match state.cash_management_engine.get_cash_position_summary(org_id, date).await {
        Ok(summary) => Ok(Json(serde_json::to_value(summary).unwrap())),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// ============================================================================
// Forecast Template Handlers
// ============================================================================

/// Create or update a forecast template
pub async fn create_forecast_template(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateForecastTemplateRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.cash_management_engine.create_forecast_template(
        org_id, &payload.code, &payload.name, payload.description.as_deref(),
        &payload.bucket_type, payload.number_of_periods, payload.start_offset_days,
        payload.is_default, payload.columns, Some(user_id),
    ).await {
        Ok(t) => Ok((StatusCode::CREATED, Json(serde_json::to_value(t).unwrap()))),
        Err(e) => {
            error!("Failed to create forecast template: {}", e);
            Err(map_error(e))
        }
    }
}

/// Get a forecast template by code
pub async fn get_forecast_template(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(code): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.cash_management_engine.get_forecast_template(org_id, &code).await {
        Ok(Some(t)) => Ok(Json(serde_json::to_value(t).unwrap())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

/// List all forecast templates
pub async fn list_forecast_templates(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.cash_management_engine.list_forecast_templates(org_id).await {
        Ok(templates) => Ok(Json(serde_json::to_value(templates).unwrap())),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

/// Delete a forecast template
pub async fn delete_forecast_template(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(code): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.cash_management_engine.delete_forecast_template(org_id, &code).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err(map_error(e))
    }
}

// ============================================================================
// Forecast Source Handlers
// ============================================================================

/// Create or update a forecast source
pub async fn create_forecast_source(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateForecastSourceRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Resolve template code to ID
    let template = state.cash_management_engine.get_forecast_template(org_id, &payload.template_code).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    match state.cash_management_engine.create_forecast_source(
        org_id, template.id, &payload.code, &payload.name, payload.description.as_deref(),
        &payload.source_type, &payload.cash_flow_direction, payload.is_actual,
        payload.display_order, payload.lead_time_days,
        payload.payment_terms_reference.as_deref(), payload.account_code_filter.as_deref(),
        Some(user_id),
    ).await {
        Ok(s) => Ok((StatusCode::CREATED, Json(serde_json::to_value(s).unwrap()))),
        Err(e) => {
            error!("Failed to create forecast source: {}", e);
            Err(map_error(e))
        }
    }
}

/// List forecast sources for a template
pub async fn list_forecast_sources(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(template_code): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let template = state.cash_management_engine.get_forecast_template(org_id, &template_code).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    match state.cash_management_engine.list_forecast_sources(template.id).await {
        Ok(sources) => Ok(Json(serde_json::to_value(sources).unwrap())),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

/// Delete a forecast source
pub async fn delete_forecast_source(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path((template_code, code)): Path<(String, String)>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let template = state.cash_management_engine.get_forecast_template(org_id, &template_code).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    match state.cash_management_engine.delete_forecast_source(org_id, template.id, &code).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err(map_error(e))
    }
}

// ============================================================================
// Cash Forecast Handlers
// ============================================================================

/// Generate a new cash forecast
pub async fn generate_forecast(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<GenerateForecastRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.cash_management_engine.generate_forecast(
        org_id, &payload.template_code, &payload.name, payload.description.as_deref(),
        Some(user_id),
    ).await {
        Ok(f) => Ok((StatusCode::CREATED, Json(serde_json::to_value(f).unwrap()))),
        Err(e) => {
            error!("Failed to generate forecast: {}", e);
            Err(map_error(e))
        }
    }
}

/// Get a cash forecast by ID
pub async fn get_cash_forecast(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.cash_management_engine.get_forecast(id).await {
        Ok(Some(f)) => Ok(Json(serde_json::to_value(f).unwrap())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

/// List cash forecasts with optional filters
pub async fn list_cash_forecasts(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListForecastsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.cash_management_engine.list_forecasts(
        org_id, query.template_id, query.status.as_deref(),
    ).await {
        Ok(forecasts) => Ok(Json(serde_json::to_value(forecasts).unwrap())),
        Err(e) => { error!("Error: {}", e); Err(map_error(e)) }
    }
}

/// Approve a cash forecast
pub async fn approve_cash_forecast(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.cash_management_engine.approve_forecast(id, user_id).await {
        Ok(f) => Ok(Json(serde_json::to_value(f).unwrap())),
        Err(e) => Err(map_error(e))
    }
}

/// List forecast lines for a forecast
pub async fn list_forecast_lines(
    State(state): State<Arc<AppState>>,
    Path(forecast_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.cash_management_engine.list_forecast_lines(forecast_id).await {
        Ok(lines) => Ok(Json(serde_json::to_value(lines).unwrap())),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

/// Get forecast summary for dashboard
pub async fn get_forecast_summary(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ForecastSummaryQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.cash_management_engine.get_forecast_summary(org_id, &query.template_code).await {
        Ok(summary) => Ok(Json(serde_json::to_value(summary).unwrap())),
        Err(e) => Err(map_error(e))
    }
}

// ============================================================================
// Error Mapping
// ============================================================================

fn map_error(e: atlas_shared::AtlasError) -> StatusCode {
    match e {
        atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
        atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
        atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
        atlas_shared::AtlasError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
        atlas_shared::AtlasError::Forbidden(_) => StatusCode::FORBIDDEN,
        atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
