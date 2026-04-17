//! Tax Management Handlers
//!
//! Oracle Fusion Cloud ERP: Tax > Tax Configuration and Calculation
//!
//! API endpoints for managing tax regimes, jurisdictions, rates,
//! determination rules, and performing tax calculations.

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
// Tax Regime Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateTaxRegimeRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(default = "default_tax_type")]
    pub tax_type: String,
    #[serde(default)]
    pub default_inclusive: bool,
    #[serde(default)]
    pub allows_recovery: bool,
    #[serde(default = "default_rounding_rule")]
    pub rounding_rule: String,
    #[serde(default = "default_rounding_precision")]
    pub rounding_precision: i32,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
}

fn default_tax_type() -> String { "vat".to_string() }
fn default_rounding_rule() -> String { "nearest".to_string() }
fn default_rounding_precision() -> i32 { 2 }

/// Create or update a tax regime
pub async fn create_tax_regime(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateTaxRegimeRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    info!("Creating tax regime {} for org {} by user {}", payload.code, org_id, user_id);

    let regime = state.tax_engine
        .create_regime(
            org_id,
            &payload.code,
            &payload.name,
            payload.description.as_deref(),
            &payload.tax_type,
            payload.default_inclusive,
            payload.allows_recovery,
            &payload.rounding_rule,
            payload.rounding_precision,
            payload.effective_from,
            payload.effective_to,
            Some(user_id),
        )
        .await
        .map_err(|e| {
            error!("Create tax regime error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(regime).unwrap_or_default())))
}

/// List all tax regimes for the organization
pub async fn list_tax_regimes(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let regimes = state.tax_engine
        .list_regimes(org_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({ "data": regimes })))
}

/// Get a specific tax regime
pub async fn get_tax_regime(
    State(state): State<Arc<AppState>>,
    Path(code): Path<String>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let regime = state.tax_engine
        .get_regime(org_id, &code)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match regime {
        Some(r) => Ok(Json(serde_json::to_value(r).unwrap_or_default())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// Delete (deactivate) a tax regime
pub async fn delete_tax_regime(
    State(state): State<Arc<AppState>>,
    Path(code): Path<String>,
    claims: Extension<Claims>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    state.tax_engine
        .delete_regime(org_id, &code)
        .await
        .map_err(|e| match e {
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        })?;

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Tax Jurisdiction Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateTaxJurisdictionRequest {
    pub regime_code: String,
    pub code: String,
    pub name: String,
    #[serde(default = "default_geo_level")]
    pub geographic_level: String,
    pub country_code: Option<String>,
    pub state_code: Option<String>,
    pub county: Option<String>,
    pub city: Option<String>,
    pub postal_code_pattern: Option<String>,
}

fn default_geo_level() -> String { "country".to_string() }

/// Create a tax jurisdiction
pub async fn create_tax_jurisdiction(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateTaxJurisdictionRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    info!("Creating tax jurisdiction {} for org {} by user {}", payload.code, org_id, user_id);

    let jurisdiction = state.tax_engine
        .create_jurisdiction(
            org_id,
            &payload.regime_code,
            &payload.code,
            &payload.name,
            &payload.geographic_level,
            payload.country_code.as_deref(),
            payload.state_code.as_deref(),
            payload.county.as_deref(),
            payload.city.as_deref(),
            payload.postal_code_pattern.as_deref(),
            Some(user_id),
        )
        .await
        .map_err(|e| {
            error!("Create tax jurisdiction error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(jurisdiction).unwrap_or_default())))
}

/// List tax jurisdictions
pub async fn list_tax_jurisdictions(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<JurisdictionListParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let jurisdictions = state.tax_engine
        .list_jurisdictions(org_id, params.regime_code.as_deref())
        .await
        .map_err(|e| {
            error!("List jurisdictions error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(serde_json::json!({ "data": jurisdictions })))
}

#[derive(Debug, Deserialize)]
pub struct JurisdictionListParams {
    pub regime_code: Option<String>,
}

/// Delete (deactivate) a tax jurisdiction
pub async fn delete_tax_jurisdiction(
    State(state): State<Arc<AppState>>,
    Path((regime_code, code)): Path<(String, String)>,
    claims: Extension<Claims>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    state.tax_engine
        .delete_jurisdiction(org_id, &regime_code, &code)
        .await
        .map_err(|e| match e {
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        })?;

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Tax Rate Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateTaxRateRequest {
    pub regime_code: String,
    pub jurisdiction_code: Option<String>,
    pub code: String,
    pub name: String,
    pub rate_percentage: String,
    #[serde(default = "default_rate_type")]
    pub rate_type: String,
    pub tax_account_code: Option<String>,
    #[serde(default)]
    pub recoverable: bool,
    pub recovery_percentage: Option<String>,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
}

fn default_rate_type() -> String { "standard".to_string() }

/// Create or update a tax rate
pub async fn create_tax_rate(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateTaxRateRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    info!("Creating tax rate {} for org {} by user {}", payload.code, org_id, user_id);

    let effective_from = payload.effective_from
        .unwrap_or_else(|| chrono::Utc::now().date_naive());

    let rate = state.tax_engine
        .create_tax_rate(
            org_id,
            &payload.regime_code,
            payload.jurisdiction_code.as_deref(),
            &payload.code,
            &payload.name,
            &payload.rate_percentage,
            &payload.rate_type,
            payload.tax_account_code.as_deref(),
            payload.recoverable,
            payload.recovery_percentage.as_deref(),
            effective_from,
            payload.effective_to,
            Some(user_id),
        )
        .await
        .map_err(|e| {
            error!("Create tax rate error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(rate).unwrap_or_default())))
}

/// List tax rates for a regime
pub async fn list_tax_rates(
    State(state): State<Arc<AppState>>,
    Path(regime_code): Path<String>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let rates = state.tax_engine
        .list_tax_rates(org_id, &regime_code)
        .await
        .map_err(|e| match e {
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        })?;

    Ok(Json(serde_json::json!({ "data": rates })))
}

/// Delete (deactivate) a tax rate
pub async fn delete_tax_rate(
    State(state): State<Arc<AppState>>,
    Path((regime_code, code)): Path<(String, String)>,
    claims: Extension<Claims>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    state.tax_engine
        .delete_tax_rate(org_id, &regime_code, &code)
        .await
        .map_err(|e| match e {
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        })?;

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Tax Determination Rules
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateDeterminationRuleRequest {
    pub regime_code: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(default = "default_priority")]
    pub priority: i32,
    pub condition: serde_json::Value,
    pub action: serde_json::Value,
    #[serde(default = "default_true_val")]
    pub stop_on_match: bool,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
}

fn default_priority() -> i32 { 100 }
fn default_true_val() -> bool { true }

/// Create a tax determination rule
pub async fn create_determination_rule(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateDeterminationRuleRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    info!("Creating tax determination rule '{}' for org {} by user {}", payload.name, org_id, user_id);

    let rule = state.tax_engine
        .create_determination_rule(
            org_id,
            &payload.regime_code,
            &payload.name,
            payload.description.as_deref(),
            payload.priority,
            payload.condition,
            payload.action,
            payload.stop_on_match,
            payload.effective_from,
            payload.effective_to,
            Some(user_id),
        )
        .await
        .map_err(|e| {
            error!("Create determination rule error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(rule).unwrap_or_default())))
}

/// List determination rules for a regime
pub async fn list_determination_rules(
    State(state): State<Arc<AppState>>,
    Path(regime_code): Path<String>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let rules = state.tax_engine
        .list_determination_rules(org_id, &regime_code)
        .await
        .map_err(|e| match e {
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        })?;

    Ok(Json(serde_json::json!({ "data": rules })))
}

// ============================================================================
// Tax Calculation
// ============================================================================

/// Calculate taxes for a transaction
pub async fn calculate_tax(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<atlas_shared::TaxCalculationRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    info!("Calculating tax for {} lines, org {} by user {}",
        payload.lines.len(), org_id, user_id);

    let result = state.tax_engine
        .calculate_tax(org_id, payload, Some(user_id))
        .await
        .map_err(|e| {
            error!("Tax calculation error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(Json(serde_json::to_value(result).unwrap_or_default()))
}

// ============================================================================
// Tax Lines (for a specific transaction)
// ============================================================================

/// Get tax lines for a transaction
pub async fn get_tax_lines(
    State(state): State<Arc<AppState>>,
    Path((entity_type, entity_id)): Path<(String, Uuid)>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let _org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let lines = state.tax_engine
        .get_tax_lines(&entity_type, entity_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({ "data": lines })))
}

// ============================================================================
// Tax Reporting
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct GenerateTaxReportRequest {
    pub regime_code: String,
    pub jurisdiction_code: Option<String>,
    pub period_start: chrono::NaiveDate,
    pub period_end: chrono::NaiveDate,
}

/// Generate a tax report for a period
pub async fn generate_tax_report(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<GenerateTaxReportRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    info!("Generating tax report for regime {} org {} by user {}",
        payload.regime_code, org_id, user_id);

    let report = state.tax_engine
        .generate_tax_report(
            org_id,
            &payload.regime_code,
            payload.jurisdiction_code.as_deref(),
            payload.period_start,
            payload.period_end,
            Some(user_id),
        )
        .await
        .map_err(|e| {
            error!("Tax report generation error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(Json(serde_json::to_value(report).unwrap_or_default()))
}

/// List tax reports
pub async fn list_tax_reports(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<TaxReportListParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let reports = state.tax_engine
        .list_tax_reports(org_id, params.regime_code.as_deref())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({ "data": reports })))
}

#[derive(Debug, Deserialize)]
pub struct TaxReportListParams {
    pub regime_code: Option<String>,
}
