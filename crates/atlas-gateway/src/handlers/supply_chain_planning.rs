//! Supply Chain Planning Handlers (MRP)
//!
//! Oracle Fusion Cloud: Supply Chain Management > Supply Chain Planning.
//! HTTP API for planning scenarios, parameters, supply/demand,
//! planned orders, exceptions, and dashboard.

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

fn scp_map_err(e: atlas_shared::AtlasError) -> StatusCode {
    match e.status_code() {
        400 => StatusCode::BAD_REQUEST,
        404 => StatusCode::NOT_FOUND,
        409 => StatusCode::CONFLICT,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

// ============================================================================
// Scenarios
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateScenarioRequest {
    pub name: String,
    pub description: Option<String>,
    pub scenario_type: Option<String>,
    pub planning_horizon_days: Option<i32>,
    pub planning_start_date: Option<chrono::NaiveDate>,
    pub include_existing_supply: Option<bool>,
    pub include_on_hand: Option<bool>,
    pub include_work_in_progress: Option<bool>,
    pub auto_firm: Option<bool>,
    pub auto_firm_days: Option<i32>,
    pub net_shortages_only: Option<bool>,
}

pub async fn create_scenario(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateScenarioRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).ok();
    match state.planning_engine.create_scenario(
        org_id, &payload.name, payload.description.as_deref(),
        payload.scenario_type.as_deref().unwrap_or("mrp"),
        payload.planning_horizon_days.unwrap_or(90),
        payload.planning_start_date,
        payload.include_existing_supply.unwrap_or(true),
        payload.include_on_hand.unwrap_or(true),
        payload.include_work_in_progress.unwrap_or(true),
        payload.auto_firm.unwrap_or(false),
        payload.auto_firm_days,
        payload.net_shortages_only.unwrap_or(false),
        user_id,
    ).await {
        Ok(s) => Ok((StatusCode::CREATED, Json(serde_json::to_value(s).unwrap_or_default()))),
        Err(e) => { error!("Failed to create planning scenario: {}", e); Err(scp_map_err(e)) }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListScenariosParams {
    pub scenario_type: Option<String>,
    pub status: Option<String>,
}

pub async fn list_scenarios(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<ListScenariosParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.planning_engine.list_scenarios(
        org_id, params.scenario_type.as_deref(), params.status.as_deref(),
    ).await {
        Ok(list) => Ok(Json(serde_json::to_value(list).unwrap_or_default())),
        Err(e) => { error!("Failed to list scenarios: {}", e); Err(scp_map_err(e)) }
    }
}

pub async fn get_scenario(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.planning_engine.get_scenario(id).await {
        Ok(Some(s)) => Ok(Json(serde_json::to_value(s).unwrap_or_default())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Failed to get scenario: {}", e); Err(scp_map_err(e)) }
    }
}

pub async fn run_mrp(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.planning_engine.run_mrp(id).await {
        Ok(s) => Ok(Json(serde_json::to_value(s).unwrap_or_default())),
        Err(e) => { error!("Failed to run MRP: {}", e); Err(scp_map_err(e)) }
    }
}

pub async fn cancel_scenario(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.planning_engine.cancel_scenario(id).await {
        Ok(s) => Ok(Json(serde_json::to_value(s).unwrap_or_default())),
        Err(e) => { error!("Failed to cancel scenario: {}", e); Err(scp_map_err(e)) }
    }
}

// ============================================================================
// Planning Parameters
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct UpsertParameterRequest {
    pub item_id: Uuid,
    pub item_name: Option<String>,
    pub item_number: Option<String>,
    pub planner_code: Option<String>,
    pub planning_method: Option<String>,
    pub make_buy: Option<String>,
    pub lead_time_days: Option<i32>,
    pub safety_stock_quantity: Option<String>,
    pub min_order_quantity: Option<String>,
    pub max_order_quantity: Option<String>,
    pub fixed_order_quantity: Option<String>,
    pub lot_size_policy: Option<String>,
    pub order_multiple: Option<String>,
    pub default_supplier_id: Option<Uuid>,
    pub default_supplier_name: Option<String>,
}

pub async fn upsert_parameter(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<UpsertParameterRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).ok();
    match state.planning_engine.upsert_planning_parameter(
        org_id, payload.item_id, payload.item_name.as_deref(),
        payload.item_number.as_deref(), payload.planner_code.as_deref(),
        payload.planning_method.as_deref().unwrap_or("mrp"),
        payload.make_buy.as_deref().unwrap_or("buy"),
        payload.lead_time_days.unwrap_or(0),
        payload.safety_stock_quantity.as_deref().unwrap_or("0"),
        payload.min_order_quantity.as_deref().unwrap_or("0"),
        payload.max_order_quantity.as_deref(),
        payload.fixed_order_quantity.as_deref(),
        payload.lot_size_policy.as_deref().unwrap_or("lot_for_lot"),
        payload.order_multiple.as_deref(),
        payload.default_supplier_id,
        payload.default_supplier_name.as_deref(),
        user_id,
    ).await {
        Ok(p) => Ok((StatusCode::CREATED, Json(serde_json::to_value(p).unwrap_or_default()))),
        Err(e) => { error!("Failed to upsert parameter: {}", e); Err(scp_map_err(e)) }
    }
}

pub async fn list_parameters(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.planning_engine.list_planning_parameters(org_id).await {
        Ok(list) => Ok(Json(serde_json::to_value(list).unwrap_or_default())),
        Err(e) => { error!("Failed to list parameters: {}", e); Err(scp_map_err(e)) }
    }
}

pub async fn delete_parameter(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(item_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.planning_engine.delete_planning_parameter(org_id, item_id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => { error!("Failed to delete parameter: {}", e); Err(scp_map_err(e)) }
    }
}

// ============================================================================
// Supply/Demand Entries
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateSupplyDemandRequest {
    pub scenario_id: Option<Uuid>,
    pub item_id: Uuid,
    pub item_name: Option<String>,
    pub item_number: Option<String>,
    pub entry_type: String,
    pub source_type: String,
    pub source_id: Option<Uuid>,
    pub source_number: Option<String>,
    pub quantity: String,
    pub due_date: chrono::NaiveDate,
    pub priority: Option<i32>,
}

pub async fn create_supply_demand(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateSupplyDemandRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.planning_engine.create_supply_demand_entry(
        org_id, payload.scenario_id, payload.item_id,
        payload.item_name.as_deref(), payload.item_number.as_deref(),
        &payload.entry_type, &payload.source_type,
        payload.source_id, payload.source_number.as_deref(),
        &payload.quantity, payload.due_date, payload.priority,
    ).await {
        Ok(e) => Ok((StatusCode::CREATED, Json(serde_json::to_value(e).unwrap_or_default()))),
        Err(e) => { error!("Failed to create supply/demand: {}", e); Err(scp_map_err(e)) }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListSupplyDemandParams {
    pub entry_type: Option<String>,
}

pub async fn list_supply_demand(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(scenario_id): Path<Uuid>,
    Query(params): Query<ListSupplyDemandParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.planning_engine.list_supply_demand(
        scenario_id, params.entry_type.as_deref(),
    ).await {
        Ok(list) => Ok(Json(serde_json::to_value(list).unwrap_or_default())),
        Err(e) => { error!("Failed to list supply/demand: {}", e); Err(scp_map_err(e)) }
    }
}

// ============================================================================
// Planned Orders
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ListPlannedOrdersParams {
    pub status: Option<String>,
    pub order_type: Option<String>,
}

pub async fn list_planned_orders(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(scenario_id): Path<Uuid>,
    Query(params): Query<ListPlannedOrdersParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.planning_engine.list_planned_orders(
        scenario_id, params.status.as_deref(), params.order_type.as_deref(),
    ).await {
        Ok(list) => Ok(Json(serde_json::to_value(list).unwrap_or_default())),
        Err(e) => { error!("Failed to list planned orders: {}", e); Err(scp_map_err(e)) }
    }
}

pub async fn get_planned_order(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.planning_engine.get_planned_order(id).await {
        Ok(Some(o)) => Ok(Json(serde_json::to_value(o).unwrap_or_default())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Failed to get planned order: {}", e); Err(scp_map_err(e)) }
    }
}

pub async fn firm_planned_order(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.planning_engine.firm_planned_order(id).await {
        Ok(o) => Ok(Json(serde_json::to_value(o).unwrap_or_default())),
        Err(e) => { error!("Failed to firm order: {}", e); Err(scp_map_err(e)) }
    }
}

pub async fn cancel_planned_order(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.planning_engine.cancel_planned_order(id).await {
        Ok(o) => Ok(Json(serde_json::to_value(o).unwrap_or_default())),
        Err(e) => { error!("Failed to cancel order: {}", e); Err(scp_map_err(e)) }
    }
}

// ============================================================================
// Exceptions
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ListExceptionsParams {
    pub severity: Option<String>,
    pub resolution_status: Option<String>,
}

pub async fn list_exceptions(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(scenario_id): Path<Uuid>,
    Query(params): Query<ListExceptionsParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.planning_engine.list_exceptions(
        scenario_id, params.severity.as_deref(), params.resolution_status.as_deref(),
    ).await {
        Ok(list) => Ok(Json(serde_json::to_value(list).unwrap_or_default())),
        Err(e) => { error!("Failed to list exceptions: {}", e); Err(scp_map_err(e)) }
    }
}

#[derive(Debug, Deserialize)]
pub struct ResolveExceptionRequest {
    pub resolution: String,
}

pub async fn resolve_exception(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<ResolveExceptionRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).ok();
    match state.planning_engine.resolve_exception(
        id, &payload.resolution, user_id,
    ).await {
        Ok(ex) => Ok(Json(serde_json::to_value(ex).unwrap_or_default())),
        Err(e) => { error!("Failed to resolve exception: {}", e); Err(scp_map_err(e)) }
    }
}

#[derive(Debug, Deserialize)]
pub struct DismissExceptionRequest {
    pub reason: Option<String>,
}

pub async fn dismiss_exception(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<DismissExceptionRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).ok();
    match state.planning_engine.dismiss_exception(
        id, payload.reason.as_deref(), user_id,
    ).await {
        Ok(ex) => Ok(Json(serde_json::to_value(ex).unwrap_or_default())),
        Err(e) => { error!("Failed to dismiss exception: {}", e); Err(scp_map_err(e)) }
    }
}

// ============================================================================
// Dashboard
// ============================================================================

pub async fn get_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.planning_engine.get_dashboard(org_id).await {
        Ok(d) => Ok(Json(serde_json::to_value(d).unwrap_or_default())),
        Err(e) => { error!("Failed to get planning dashboard: {}", e); Err(scp_map_err(e)) }
    }
}
