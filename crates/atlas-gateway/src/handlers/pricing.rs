//! Advanced Pricing Management Handlers
//!
//! Oracle Fusion Cloud ERP: Order Management > Pricing > Advanced Pricing
//! API endpoints for price lists, price list lines, price tiers, discount rules,
//! charge definitions, pricing strategies, price calculation, and dashboard.

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

fn default_sale() -> String { "sale".to_string() }
fn default_fixed() -> String { "fixed".to_string() }
fn default_usd() -> String { "USD".to_string() }
fn default_percentage() -> String { "percentage".to_string() }
fn default_line() -> String { "line".to_string() }
fn default_exclusive() -> String { "exclusive".to_string() }
fn default_surcharge() -> String { "surcharge".to_string() }
fn default_handling() -> String { "handling".to_string() }
fn default_price_list_type() -> String { "price_list".to_string() }


/// Parse a UUID from a claim string, returning a JSON error on failure.
///
/// Unlike `unwrap_or_default()`, this does NOT silently fall back to the nil
/// UUID — which would be an auth-scoping bypass.
fn parse_uuid(s: &str) -> Result<Uuid, (axum::http::StatusCode, Json<serde_json::Value>)> {
    Uuid::parse_str(s).map_err(|_| {
        (axum::http::StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "Invalid auth token"})))
    })
}
// ============================================================================
// Price Lists
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreatePriceListRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(default = "default_usd")]
    pub currency_code: String,
    #[serde(default = "default_sale")]
    pub list_type: String,
    #[serde(default = "default_fixed")]
    pub pricing_basis: String,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
}

pub async fn create_price_list(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreatePriceListRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    match state.pricing_engine.create_price_list(
        parse_uuid(&claims.org_id)?,
        &req.code, &req.name, req.description.as_deref(),
        &req.currency_code, &req.list_type, &req.pricing_basis,
        req.effective_from, req.effective_to, None,
    ).await {
        Ok(pl) => Ok((StatusCode::CREATED, Json(serde_json::to_value(pl).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => {
            error!("Failed to create price list: {}", e);
            Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

pub async fn get_price_list(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(code): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.pricing_engine.get_price_list(
        parse_uuid(&claims.org_id)?, &code,
    ).await {
        Ok(Some(pl)) => Ok(Json(serde_json::to_value(pl).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Not found"})))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()})))),
    }
}

#[derive(Debug, Deserialize)]
pub struct ListPriceListsQuery {
    pub list_type: Option<String>,
    pub status: Option<String>,
}

pub async fn list_price_lists(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<ListPriceListsQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.pricing_engine.list_price_lists(
        parse_uuid(&claims.org_id)?,
        query.list_type.as_deref(), query.status.as_deref(),
    ).await {
        Ok(lists) => Ok(Json(serde_json::json!({"data": lists}))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()})))),
    }
}

pub async fn activate_price_list(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.pricing_engine.activate_price_list(id).await {
        Ok(pl) => Ok(Json(serde_json::to_value(pl).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()})))),
    }
}

pub async fn deactivate_price_list(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.pricing_engine.deactivate_price_list(id).await {
        Ok(pl) => Ok(Json(serde_json::to_value(pl).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()})))),
    }
}

pub async fn delete_price_list(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(code): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    match state.pricing_engine.delete_price_list(
        parse_uuid(&claims.org_id)?, &code,
    ).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Price List Lines
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct AddPriceListLineRequest {
    pub item_id: Option<Uuid>,
    pub item_code: Option<String>,
    pub item_description: Option<String>,
    #[serde(default = "default_ea")]
    pub pricing_unit_of_measure: String,
    pub list_price: String,
    pub unit_price: String,
    #[serde(default = "default_zero")]
    pub cost_price: String,
    #[serde(default = "default_zero")]
    pub margin_percent: String,
    #[serde(default = "default_one")]
    pub minimum_quantity: String,
    pub maximum_quantity: Option<String>,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
}

fn default_ea() -> String { "Ea".to_string() }
fn default_zero() -> String { "0".to_string() }
fn default_one() -> String { "1".to_string() }

pub async fn add_price_list_line(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(price_list_id): Path<Uuid>,
    Json(req): Json<AddPriceListLineRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    match state.pricing_engine.add_price_list_line(
        parse_uuid(&claims.org_id)?,
        price_list_id,
        req.item_id, req.item_code.as_deref(), req.item_description.as_deref(),
        &req.pricing_unit_of_measure,
        &req.list_price, &req.unit_price, &req.cost_price, &req.margin_percent,
        &req.minimum_quantity, req.maximum_quantity.as_deref(),
        req.effective_from, req.effective_to, None,
    ).await {
        Ok(line) => Ok((StatusCode::CREATED, Json(serde_json::to_value(line).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => {
            error!("Failed to add price list line: {}", e);
            Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

pub async fn list_price_list_lines(
    State(state): State<Arc<AppState>>,
    Path(price_list_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.pricing_engine.list_price_list_lines(price_list_id).await {
        Ok(lines) => Ok(Json(serde_json::json!({"data": lines}))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()})))),
    }
}

pub async fn delete_price_list_line(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    match state.pricing_engine.delete_price_list_line(id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Price Tiers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct AddPriceTierRequest {
    pub from_quantity: String,
    pub to_quantity: Option<String>,
    pub price: String,
    #[serde(default = "default_zero")]
    pub discount_percent: String,
    #[serde(default = "default_fixed")]
    pub price_type: String,
}

pub async fn add_price_tier(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(price_list_line_id): Path<Uuid>,
    Json(req): Json<AddPriceTierRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    match state.pricing_engine.add_price_tier(
        parse_uuid(&claims.org_id)?,
        price_list_line_id,
        &req.from_quantity, req.to_quantity.as_deref(),
        &req.price, &req.discount_percent, &req.price_type,
    ).await {
        Ok(tier) => Ok((StatusCode::CREATED, Json(serde_json::to_value(tier).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => {
            error!("Failed to add price tier: {}", e);
            Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

pub async fn list_price_tiers(
    State(state): State<Arc<AppState>>,
    Path(price_list_line_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.pricing_engine.list_price_tiers(price_list_line_id).await {
        Ok(tiers) => Ok(Json(serde_json::json!({"data": tiers}))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Discount Rules
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateDiscountRuleRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(default = "default_percentage")]
    pub discount_type: String,
    pub discount_value: String,
    #[serde(default = "default_line")]
    pub application_method: String,
    #[serde(default = "default_exclusive")]
    pub stacking_rule: String,
    #[serde(default = "default_priority")]
    pub priority: i32,
    #[serde(default)]
    pub condition: serde_json::Value,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub max_usage: Option<i32>,
}

fn default_priority() -> i32 { 10 }

pub async fn create_discount_rule(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreateDiscountRuleRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    match state.pricing_engine.create_discount_rule(
        parse_uuid(&claims.org_id)?,
        &req.code, &req.name, req.description.as_deref(),
        &req.discount_type, &req.discount_value,
        &req.application_method, &req.stacking_rule,
        req.priority, req.condition,
        req.effective_from, req.effective_to, req.max_usage, None,
    ).await {
        Ok(rule) => Ok((StatusCode::CREATED, Json(serde_json::to_value(rule).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => {
            error!("Failed to create discount rule: {}", e);
            Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

pub async fn get_discount_rule(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(code): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.pricing_engine.get_discount_rule(
        parse_uuid(&claims.org_id)?, &code,
    ).await {
        Ok(Some(rule)) => Ok(Json(serde_json::to_value(rule).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Not found"})))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()})))),
    }
}

#[derive(Debug, Deserialize)]
pub struct ListDiscountRulesQuery {
    pub status: Option<String>,
}

pub async fn list_discount_rules(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<ListDiscountRulesQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.pricing_engine.list_discount_rules(
        parse_uuid(&claims.org_id)?, query.status.as_deref(),
    ).await {
        Ok(rules) => Ok(Json(serde_json::json!({"data": rules}))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()})))),
    }
}

pub async fn delete_discount_rule(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(code): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    match state.pricing_engine.delete_discount_rule(
        parse_uuid(&claims.org_id)?, &code,
    ).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Charge Definitions
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateChargeDefinitionRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(default = "default_surcharge")]
    pub charge_type: String,
    #[serde(default = "default_handling")]
    pub charge_category: String,
    #[serde(default = "default_fixed")]
    pub calculation_method: String,
    #[serde(default = "default_zero")]
    pub charge_amount: String,
    #[serde(default = "default_zero")]
    pub charge_percent: String,
    #[serde(default = "default_zero")]
    pub minimum_charge: String,
    pub maximum_charge: Option<String>,
    #[serde(default)]
    pub taxable: bool,
    #[serde(default)]
    pub condition: serde_json::Value,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
}

pub async fn create_charge_definition(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreateChargeDefinitionRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    match state.pricing_engine.create_charge_definition(
        parse_uuid(&claims.org_id)?,
        &req.code, &req.name, req.description.as_deref(),
        &req.charge_type, &req.charge_category, &req.calculation_method,
        &req.charge_amount, &req.charge_percent, &req.minimum_charge,
        req.maximum_charge.as_deref(), req.taxable,
        req.condition, req.effective_from, req.effective_to, None,
    ).await {
        Ok(charge) => Ok((StatusCode::CREATED, Json(serde_json::to_value(charge).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => {
            error!("Failed to create charge definition: {}", e);
            Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

pub async fn get_charge_definition(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(code): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.pricing_engine.get_charge_definition(
        parse_uuid(&claims.org_id)?, &code,
    ).await {
        Ok(Some(charge)) => Ok(Json(serde_json::to_value(charge).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Not found"})))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()})))),
    }
}

#[derive(Debug, Deserialize)]
pub struct ListChargeDefinitionsQuery {
    pub charge_type: Option<String>,
}

pub async fn list_charge_definitions(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<ListChargeDefinitionsQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.pricing_engine.list_charge_definitions(
        parse_uuid(&claims.org_id)?, query.charge_type.as_deref(),
    ).await {
        Ok(charges) => Ok(Json(serde_json::json!({"data": charges}))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()})))),
    }
}

pub async fn delete_charge_definition(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(code): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    match state.pricing_engine.delete_charge_definition(
        parse_uuid(&claims.org_id)?, &code,
    ).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Pricing Strategies
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreatePricingStrategyRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(default = "default_price_list_type")]
    pub strategy_type: String,
    #[serde(default = "default_priority")]
    pub priority: i32,
    #[serde(default)]
    pub condition: serde_json::Value,
    pub price_list_id: Option<Uuid>,
    #[serde(default = "default_zero")]
    pub markup_percent: String,
    #[serde(default = "default_zero")]
    pub markdown_percent: String,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
}

pub async fn create_pricing_strategy(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreatePricingStrategyRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    match state.pricing_engine.create_pricing_strategy(
        parse_uuid(&claims.org_id)?,
        &req.code, &req.name, req.description.as_deref(),
        &req.strategy_type, req.priority, req.condition,
        req.price_list_id, &req.markup_percent, &req.markdown_percent,
        req.effective_from, req.effective_to, None,
    ).await {
        Ok(strategy) => Ok((StatusCode::CREATED, Json(serde_json::to_value(strategy).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => {
            error!("Failed to create pricing strategy: {}", e);
            Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

pub async fn list_pricing_strategies(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.pricing_engine.list_pricing_strategies(
        parse_uuid(&claims.org_id)?,
    ).await {
        Ok(strategies) => Ok(Json(serde_json::json!({"data": strategies}))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Price Calculation
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CalculatePriceRequest {
    pub item_code: String,
    pub quantity: String,
    #[serde(default = "default_usd")]
    pub currency_code: String,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub line_id: Option<Uuid>,
    pub customer_id: Option<Uuid>,
}

pub async fn calculate_price(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CalculatePriceRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.pricing_engine.calculate_price(
        parse_uuid(&claims.org_id)?,
        &req.item_code, &req.quantity, &req.currency_code,
        &req.entity_type, req.entity_id, req.line_id,
        req.customer_id, None,
    ).await {
        Ok(result) => Ok(Json(serde_json::to_value(result).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => {
            error!("Failed to calculate price: {}", e);
            Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

// ============================================================================
// Calculation Logs
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ListCalculationLogsQuery {
    pub entity_type: Option<String>,
    pub entity_id: Option<Uuid>,
}

pub async fn list_calculation_logs(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<ListCalculationLogsQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.pricing_engine.list_calculation_logs(
        parse_uuid(&claims.org_id)?,
        query.entity_type.as_deref(), query.entity_id,
    ).await {
        Ok(logs) => Ok(Json(serde_json::json!({"data": logs}))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Dashboard
// ============================================================================

pub async fn get_pricing_dashboard(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.pricing_engine.get_dashboard_summary(
        parse_uuid(&claims.org_id)?,
    ).await {
        Ok(summary) => Ok(Json(serde_json::to_value(summary).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()})))),
    }
}
