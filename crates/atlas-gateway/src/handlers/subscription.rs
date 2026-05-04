//! Subscription Management Handlers
//!
//! Oracle Fusion Cloud ERP: Financials > Subscription Management

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
// Product Catalog
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateProductRequest {
    pub product_code: String,
    pub name: String,
    pub description: Option<String>,
    pub product_type: String,
    pub billing_frequency: String,
    #[serde(default = "default_duration")]
    pub default_duration_months: i32,
    #[serde(default)]
    pub is_auto_renew: bool,
    #[serde(default = "default_cancellation_notice")]
    pub cancellation_notice_days: i32,
    #[serde(default = "default_zero")]
    pub setup_fee: String,
    #[serde(default = "default_tier_type")]
    pub tier_type: String,
}

fn default_duration() -> i32 { 12 }
fn default_cancellation_notice() -> i32 { 30 }
fn default_zero() -> String { "0".to_string() }
fn default_tier_type() -> String { "flat".to_string() }

pub async fn create_product(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateProductRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.subscription_engine.create_product(
        org_id, &payload.product_code, &payload.name, payload.description.as_deref(),
        &payload.product_type, &payload.billing_frequency, payload.default_duration_months,
        payload.is_auto_renew, payload.cancellation_notice_days, &payload.setup_fee,
        &payload.tier_type, Some(user_id),
    ).await {
        Ok(product) => Ok((StatusCode::CREATED, Json(serde_json::to_value(product).unwrap()))),
        Err(e) => { error!("Failed to create subscription product: {}", e); Err(StatusCode::BAD_REQUEST) }
    }
}

pub async fn get_product(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(code): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.subscription_engine.get_product(org_id, &code).await {
        Ok(Some(product)) => Ok(Json(serde_json::to_value(product).unwrap())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Failed to get product: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListProductsQuery {
    pub active_only: Option<bool>,
}

pub async fn list_products(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListProductsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let active_only = query.active_only.unwrap_or(true);
    match state.subscription_engine.list_products(org_id, active_only).await {
        Ok(products) => Ok(Json(serde_json::json!({ "data": products }))),
        Err(e) => { error!("Failed to list products: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn delete_product(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(code): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.subscription_engine.delete_product(org_id, &code).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => { error!("Failed to delete product: {}", e); Err(StatusCode::BAD_REQUEST) }
    }
}

// ============================================================================
// Price Tiers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreatePriceTierRequest {
    pub tier_name: Option<String>,
    pub min_quantity: String,
    pub max_quantity: Option<String>,
    pub unit_price: String,
    #[serde(default = "default_zero")]
    pub discount_percent: String,
    #[serde(default = "default_usd")]
    pub currency_code: String,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
}

fn default_usd() -> String { "USD".to_string() }

pub async fn create_price_tier(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(product_id): Path<Uuid>,
    Json(payload): Json<CreatePriceTierRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.subscription_engine.create_price_tier(
        org_id, product_id, payload.tier_name.as_deref(),
        &payload.min_quantity, payload.max_quantity.as_deref(),
        &payload.unit_price, &payload.discount_percent,
        &payload.currency_code, payload.effective_from, payload.effective_to,
    ).await {
        Ok(tier) => Ok((StatusCode::CREATED, Json(serde_json::to_value(tier).unwrap()))),
        Err(e) => { error!("Failed to create price tier: {}", e); Err(StatusCode::BAD_REQUEST) }
    }
}

// ============================================================================
// Subscription Lifecycle
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateSubscriptionRequest {
    pub customer_id: Uuid,
    pub customer_name: Option<String>,
    pub product_id: Uuid,
    pub description: Option<String>,
    pub start_date: chrono::NaiveDate,
    pub duration_months: i32,
    pub billing_frequency: Option<String>,
    pub billing_day_of_month: Option<i32>,
    pub billing_alignment: Option<String>,
    #[serde(default = "default_usd")]
    pub currency_code: String,
    #[serde(default = "default_one")]
    pub quantity: String,
    #[serde(default = "default_zero")]
    pub discount_percent: String,
    #[serde(default)]
    pub is_auto_renew: bool,
    pub sales_rep_id: Option<Uuid>,
    pub sales_rep_name: Option<String>,
    pub gl_revenue_account: Option<String>,
    pub gl_deferred_account: Option<String>,
}

fn default_one() -> String { "1".to_string() }

pub async fn create_subscription(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateSubscriptionRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.subscription_engine.create_subscription(
        org_id, payload.customer_id, payload.customer_name.as_deref(),
        payload.product_id, payload.description.as_deref(),
        payload.start_date, payload.duration_months,
        payload.billing_frequency.as_deref(), payload.billing_day_of_month,
        payload.billing_alignment.as_deref(), &payload.currency_code,
        &payload.quantity, &payload.discount_percent,
        payload.is_auto_renew, payload.sales_rep_id,
        payload.sales_rep_name.as_deref(), payload.gl_revenue_account.as_deref(),
        payload.gl_deferred_account.as_deref(), Some(user_id),
    ).await {
        Ok(sub) => Ok((StatusCode::CREATED, Json(serde_json::to_value(sub).unwrap()))),
        Err(e) => { error!("Failed to create subscription: {}", e); Err(StatusCode::BAD_REQUEST) }
    }
}

pub async fn get_subscription(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.subscription_engine.get_subscription(id).await {
        Ok(Some(sub)) => Ok(Json(serde_json::to_value(sub).unwrap())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Failed to get subscription: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListSubscriptionsQuery {
    pub status: Option<String>,
    pub customer_id: Option<String>,
}

pub async fn list_subscriptions(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListSubscriptionsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let customer_id = query.customer_id.as_ref()
        .and_then(|s| Uuid::parse_str(s).ok());

    match state.subscription_engine.list_subscriptions(
        org_id, query.status.as_deref(), customer_id,
    ).await {
        Ok(subs) => Ok(Json(serde_json::json!({ "data": subs }))),
        Err(e) => { error!("Failed to list subscriptions: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn activate_subscription(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.subscription_engine.activate_subscription(id).await {
        Ok(sub) => Ok(Json(serde_json::to_value(sub).unwrap())),
        Err(e) => { error!("Failed to activate subscription: {}", e); Err(StatusCode::BAD_REQUEST) }
    }
}

#[derive(Debug, Deserialize)]
pub struct SuspendSubscriptionRequest {
    pub reason: Option<String>,
}

pub async fn suspend_subscription(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<SuspendSubscriptionRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.subscription_engine.suspend_subscription(id, payload.reason.as_deref()).await {
        Ok(sub) => Ok(Json(serde_json::to_value(sub).unwrap())),
        Err(e) => { error!("Failed to suspend subscription: {}", e); Err(StatusCode::BAD_REQUEST) }
    }
}

pub async fn reactivate_subscription(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.subscription_engine.reactivate_subscription(id).await {
        Ok(sub) => Ok(Json(serde_json::to_value(sub).unwrap())),
        Err(e) => { error!("Failed to reactivate subscription: {}", e); Err(StatusCode::BAD_REQUEST) }
    }
}

#[derive(Debug, Deserialize)]
pub struct CancelSubscriptionRequest {
    pub cancellation_date: chrono::NaiveDate,
    pub reason: Option<String>,
}

pub async fn cancel_subscription(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<CancelSubscriptionRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.subscription_engine.cancel_subscription(
        id, payload.cancellation_date, payload.reason.as_deref(),
    ).await {
        Ok(sub) => Ok(Json(serde_json::to_value(sub).unwrap())),
        Err(e) => { error!("Failed to cancel subscription: {}", e); Err(StatusCode::BAD_REQUEST) }
    }
}

#[derive(Debug, Deserialize)]
pub struct RenewSubscriptionRequest {
    pub new_duration_months: Option<i32>,
}

pub async fn renew_subscription(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<RenewSubscriptionRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.subscription_engine.renew_subscription(
        id, payload.new_duration_months, Some(user_id),
    ).await {
        Ok(sub) => Ok(Json(serde_json::to_value(sub).unwrap())),
        Err(e) => { error!("Failed to renew subscription: {}", e); Err(StatusCode::BAD_REQUEST) }
    }
}

// ============================================================================
// Amendments
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateAmendmentRequest {
    pub amendment_type: String,
    pub description: Option<String>,
    pub new_quantity: Option<String>,
    pub new_unit_price: Option<String>,
    pub new_end_date: Option<chrono::NaiveDate>,
    pub effective_date: chrono::NaiveDate,
}

pub async fn create_amendment(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<CreateAmendmentRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.subscription_engine.create_amendment(
        id, &payload.amendment_type, payload.description.as_deref(),
        payload.new_quantity.as_deref(), payload.new_unit_price.as_deref(),
        payload.new_end_date, payload.effective_date, Some(user_id),
    ).await {
        Ok(amd) => Ok((StatusCode::CREATED, Json(serde_json::to_value(amd).unwrap()))),
        Err(e) => { error!("Failed to create amendment: {}", e); Err(StatusCode::BAD_REQUEST) }
    }
}

pub async fn apply_amendment(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.subscription_engine.apply_amendment(id, Some(user_id)).await {
        Ok(amd) => Ok(Json(serde_json::to_value(amd).unwrap())),
        Err(e) => { error!("Failed to apply amendment: {}", e); Err(StatusCode::BAD_REQUEST) }
    }
}

pub async fn cancel_amendment(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.subscription_engine.cancel_amendment(id).await {
        Ok(amd) => Ok(Json(serde_json::to_value(amd).unwrap())),
        Err(e) => { error!("Failed to cancel amendment: {}", e); Err(StatusCode::BAD_REQUEST) }
    }
}

pub async fn list_amendments(
    State(state): State<Arc<AppState>>,
    Path(subscription_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.subscription_engine.list_amendments(subscription_id).await {
        Ok(amendments) => Ok(Json(serde_json::json!({ "data": amendments }))),
        Err(e) => { error!("Failed to list amendments: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// ============================================================================
// Billing & Revenue Schedule
// ============================================================================

pub async fn list_billing_schedule(
    State(state): State<Arc<AppState>>,
    Path(subscription_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.subscription_engine.list_billing_schedule(subscription_id).await {
        Ok(lines) => Ok(Json(serde_json::json!({ "data": lines }))),
        Err(e) => { error!("Failed to list billing schedule: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn list_revenue_schedule(
    State(state): State<Arc<AppState>>,
    Path(subscription_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.subscription_engine.list_revenue_schedule(subscription_id).await {
        Ok(lines) => Ok(Json(serde_json::json!({ "data": lines }))),
        Err(e) => { error!("Failed to list revenue schedule: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn recognize_revenue(
    State(state): State<Arc<AppState>>,
    Path(line_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.subscription_engine.recognize_revenue(line_id).await {
        Ok(line) => Ok(Json(serde_json::to_value(line).unwrap())),
        Err(e) => { error!("Failed to recognize revenue: {}", e); Err(StatusCode::BAD_REQUEST) }
    }
}

// ============================================================================
// Dashboard
// ============================================================================

pub async fn get_subscription_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.subscription_engine.get_dashboard_summary(org_id).await {
        Ok(summary) => match serde_json::to_value(summary) {
            Ok(v) => Ok(Json(v)),
            Err(e) => { error!("Failed to serialize dashboard: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
        },
        Err(e) => { error!("Failed to get subscription dashboard: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}
