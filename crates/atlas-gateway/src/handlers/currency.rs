//! Currency Management Handlers
//!
//! Oracle Fusion Cloud ERP: General Ledger > Currency Rates Manager
//!
//! API endpoints for managing currencies, exchange rates, and performing
//! currency conversions with gain/loss tracking.

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
// Currency Definition Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateCurrencyRequest {
    pub code: String,
    pub name: String,
    pub symbol: Option<String>,
    #[serde(default = "default_precision")]
    pub precision: i32,
    #[serde(default)]
    pub is_base_currency: bool,
}

fn default_precision() -> i32 { 2 }

/// Create or update a currency definition
pub async fn create_currency(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateCurrencyRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    info!("Creating currency {} for org {} by user {}", payload.code, org_id, user_id);

    let currency = state.currency_engine
        .create_currency(
            org_id,
            &payload.code,
            &payload.name,
            payload.symbol.as_deref(),
            payload.precision,
            payload.is_base_currency,
        )
        .await
        .map_err(|e| {
            error!("Create currency error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(currency).unwrap_or_default())))
}

/// List all currencies for the organization
pub async fn list_currencies(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let currencies = state.currency_engine
        .list_currencies(org_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({ "data": currencies })))
}

/// Get the base currency for the organization
pub async fn get_base_currency(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.currency_engine.get_base_currency(org_id).await {
        Ok(currency) => Ok(Json(serde_json::to_value(currency).unwrap_or_default())),
        Err(atlas_shared::AtlasError::ConfigError(_)) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Delete (deactivate) a currency
pub async fn delete_currency(
    State(state): State<Arc<AppState>>,
    Path(code): Path<String>,
    claims: Extension<Claims>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    state.currency_engine
        .delete_currency(org_id, &code)
        .await
        .map_err(|e| match e {
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        })?;

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Exchange Rate Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct SetExchangeRateRequest {
    pub from_currency: String,
    pub to_currency: String,
    #[serde(default = "default_rate_type")]
    pub rate_type: String,
    pub rate: String,
    pub effective_date: chrono::NaiveDate,
    pub source: Option<String>,
}

fn default_rate_type() -> String { "daily".to_string() }

/// Create or update an exchange rate
pub async fn set_exchange_rate(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<SetExchangeRateRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    info!("Setting exchange rate {} -> {} = {} by user {}",
        payload.from_currency, payload.to_currency, payload.rate, user_id);

    let rate = state.currency_engine
        .set_exchange_rate(
            org_id,
            &payload.from_currency,
            &payload.to_currency,
            &payload.rate_type,
            &payload.rate,
            payload.effective_date,
            payload.source.as_deref(),
            Some(user_id),
        )
        .await
        .map_err(|e| {
            error!("Set exchange rate error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(rate).unwrap_or_default())))
}

#[derive(Debug, Deserialize)]
pub struct ListRatesParams {
    pub from_currency: Option<String>,
    pub to_currency: Option<String>,
    pub rate_type: Option<String>,
    pub effective_date: Option<chrono::NaiveDate>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// List exchange rates with optional filters
pub async fn list_exchange_rates(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<ListRatesParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let limit = params.limit.unwrap_or(50).clamp(1, 200);
    let offset = params.offset.unwrap_or(0).max(0);

    let rates = state.currency_engine
        .list_rates(
            org_id,
            params.from_currency.as_deref(),
            params.to_currency.as_deref(),
            params.rate_type.as_deref(),
            params.effective_date,
            limit,
            offset,
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({
        "data": rates,
        "meta": { "limit": limit, "offset": offset }
    })))
}

/// Get a specific exchange rate
pub async fn get_exchange_rate(
    State(state): State<Arc<AppState>>,
    Path((from, to)): Path<(String, String)>,
    claims: Extension<Claims>,
    Query(params): Query<GetRateParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let rate_type = params.rate_type.unwrap_or_else(|| "daily".to_string());
    let date = params.effective_date
        .unwrap_or_else(|| chrono::Utc::now().date_naive());

    let rate = state.currency_engine
        .get_exchange_rate(org_id, &from, &to, &rate_type, date)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match rate {
        Some(r) => Ok(Json(serde_json::to_value(r).unwrap_or_default())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

#[derive(Debug, Deserialize)]
pub struct GetRateParams {
    pub rate_type: Option<String>,
    pub effective_date: Option<chrono::NaiveDate>,
}

/// Delete an exchange rate
pub async fn delete_exchange_rate(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    claims: Extension<Claims>,
) -> Result<StatusCode, StatusCode> {
    let _org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    state.currency_engine
        .delete_exchange_rate(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Currency Conversion
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ConvertRequest {
    pub from_currency: String,
    pub to_currency: String,
    pub amount: String,
    #[serde(default = "default_rate_type")]
    pub rate_type: String,
    pub effective_date: chrono::NaiveDate,
    /// Optional entity reference for tracking
    pub entity_type: Option<String>,
    pub entity_id: Option<Uuid>,
}

/// Convert an amount between currencies
pub async fn convert_currency(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<ConvertRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    info!("Converting {} {} -> {} by user {}",
        payload.amount, payload.from_currency, payload.to_currency, user_id);

    let result = state.currency_engine
        .convert(
            org_id,
            &payload.from_currency,
            &payload.to_currency,
            &payload.amount,
            &payload.rate_type,
            payload.effective_date,
            payload.entity_type.as_deref(),
            payload.entity_id,
            Some(user_id),
        )
        .await
        .map_err(|e| {
            error!("Currency conversion error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(Json(serde_json::to_value(result).unwrap_or_default()))
}

// ============================================================================
// Unrealized Gain/Loss
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct GainLossRequest {
    pub currency: String,
    pub original_amount: String,
    pub original_rate: String,
    pub revaluation_date: chrono::NaiveDate,
    #[serde(default = "default_rate_type")]
    pub rate_type: String,
}

/// Calculate unrealized gain/loss on a foreign currency balance
pub async fn calculate_gain_loss(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<GainLossRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let result = state.currency_engine
        .calculate_unrealized_gain_loss(
            org_id,
            &payload.currency,
            &payload.original_amount,
            &payload.original_rate,
            payload.revaluation_date,
            &payload.rate_type,
        )
        .await
        .map_err(|e| {
            error!("Gain/loss calculation error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(Json(serde_json::to_value(result).unwrap_or_default()))
}

// ============================================================================
// Bulk Rate Import
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ImportRatesRequest {
    pub rates: Vec<atlas_core::currency::engine::ExchangeRateImport>,
}

/// Import multiple exchange rates
pub async fn import_rates(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<ImportRatesRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let result = state.currency_engine
        .import_rates(org_id, payload.rates, Some(user_id))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let _status = if result.failed > 0 { StatusCode::PARTIAL_CONTENT } else { StatusCode::OK };
    Ok(Json(serde_json::to_value(result).unwrap_or_default()))
}
