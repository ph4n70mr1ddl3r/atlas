//! Inventory Management Handlers
//!
//! Oracle Fusion Cloud ERP: SCM > Inventory Management
//! API endpoints for inventory organizations, items, subinventories,
//! locators, on-hand balances, transactions, cycle counts, and dashboard.

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

fn default_true() -> bool { true }
fn default_false() -> bool { false }
fn default_warehouse() -> String { "warehouse".to_string() }
fn default_inventory() -> String { "inventory".to_string() }
fn default_ea() -> String { "EA".to_string() }
fn default_storage() -> String { "storage".to_string() }
fn default_manual() -> String { "manual".to_string() }
fn default_usd() -> String { "USD".to_string() }
#[allow(dead_code)]
fn default_full() -> String { "full".to_string() }
fn default_zero() -> String { "0".to_string() }
#[allow(dead_code)]
fn default_zero_tolerance() -> String { "0.0".to_string() }

// ============================================================================
// Inventory Organizations
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateInventoryOrgRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(default = "default_warehouse")]
    pub org_type: String,
    pub location_code: Option<String>,
    pub address: Option<serde_json::Value>,
    pub default_subinventory_code: Option<String>,
    #[serde(default = "default_usd")]
    pub default_currency_code: String,
    #[serde(default = "default_false")]
    pub requires_approval_for_issues: bool,
    #[serde(default = "default_true")]
    pub requires_approval_for_transfers: bool,
    #[serde(default)]
    pub enable_lot_control: bool,
    #[serde(default)]
    pub enable_serial_control: bool,
    #[serde(default)]
    pub enable_revision_control: bool,
}

pub async fn create_inventory_org(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreateInventoryOrgRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    match state.inventory_engine.create_inventory_org(
        Uuid::parse_str(&claims.org_id).unwrap_or_default(), &req.code, &req.name, req.description.as_deref(),
        &req.org_type, req.location_code.as_deref(), req.address.clone(),
        req.default_subinventory_code.as_deref(), &req.default_currency_code,
        req.requires_approval_for_issues, req.requires_approval_for_transfers,
        req.enable_lot_control, req.enable_serial_control, req.enable_revision_control,
        None,
    ).await {
        Ok(org) => Ok((StatusCode::CREATED, Json(serde_json::to_value(org).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => {
            error!("Failed to create inventory org: {}", e);
            Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

pub async fn get_inventory_org(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(code): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.inventory_engine.get_inventory_org(Uuid::parse_str(&claims.org_id).unwrap_or_default(), &code).await {
        Ok(Some(org)) => Ok(Json(serde_json::to_value(org).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Not found"})))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()})))),
    }
}

pub async fn list_inventory_orgs(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.inventory_engine.list_inventory_orgs(Uuid::parse_str(&claims.org_id).unwrap_or_default()).await {
        Ok(orgs) => Ok(Json(serde_json::json!({"data": orgs}))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()})))),
    }
}

pub async fn delete_inventory_org(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(code): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    match state.inventory_engine.delete_inventory_org(Uuid::parse_str(&claims.org_id).unwrap_or_default(), &code).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Items
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateItemRequest {
    pub item_code: String,
    pub name: String,
    pub description: Option<String>,
    pub long_description: Option<String>,
    pub category_id: Option<Uuid>,
    pub category_code: Option<String>,
    #[serde(default = "default_inventory")]
    pub item_type: String,
    #[serde(default = "default_ea")]
    pub uom: String,
    pub secondary_uom: Option<String>,
    pub weight: Option<String>,
    pub weight_uom: Option<String>,
    pub volume: Option<String>,
    pub volume_uom: Option<String>,
    #[serde(default = "default_zero")]
    pub list_price: String,
    #[serde(default = "default_zero")]
    pub standard_cost: String,
    pub min_order_quantity: Option<String>,
    pub max_order_quantity: Option<String>,
    #[serde(default)]
    pub lead_time_days: i32,
    pub shelf_life_days: Option<i32>,
    #[serde(default)]
    pub is_lot_controlled: bool,
    #[serde(default)]
    pub is_serial_controlled: bool,
    #[serde(default)]
    pub is_revision_controlled: bool,
    #[serde(default)]
    pub is_perishable: bool,
    #[serde(default)]
    pub is_hazardous: bool,
    #[serde(default = "default_true")]
    pub is_purchasable: bool,
    #[serde(default = "default_true")]
    pub is_sellable: bool,
    #[serde(default = "default_true")]
    pub is_stockable: bool,
    pub inventory_asset_account_code: Option<String>,
    pub expense_account_code: Option<String>,
    pub cost_of_goods_sold_account: Option<String>,
    pub revenue_account_code: Option<String>,
    pub barcode: Option<String>,
}

pub async fn create_item(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreateItemRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let org_id = Uuid::parse_str(&claims.org_id).unwrap_or_default();
    match state.inventory_engine.create_item(
        org_id, &req.item_code, &req.name, req.description.as_deref(),
        req.long_description.as_deref(), req.category_id, req.category_code.as_deref(),
        &req.item_type, &req.uom, req.secondary_uom.as_deref(),
        req.weight.as_deref(), req.weight_uom.as_deref(),
        req.volume.as_deref(), req.volume_uom.as_deref(),
        &req.list_price, &req.standard_cost,
        req.min_order_quantity.as_deref(), req.max_order_quantity.as_deref(),
        req.lead_time_days, req.shelf_life_days,
        req.is_lot_controlled, req.is_serial_controlled, req.is_revision_controlled,
        req.is_perishable, req.is_hazardous,
        req.is_purchasable, req.is_sellable, req.is_stockable,
        req.inventory_asset_account_code.as_deref(), req.expense_account_code.as_deref(),
        req.cost_of_goods_sold_account.as_deref(), req.revenue_account_code.as_deref(),
        req.barcode.as_deref(),
        None,
    ).await {
        Ok(item) => Ok((StatusCode::CREATED, Json(serde_json::to_value(item).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => {
            error!("Failed to create item: {}", e);
            Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

pub async fn get_item(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.inventory_engine.get_item(id).await {
        Ok(Some(item)) => Ok(Json(serde_json::to_value(item).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Not found"})))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()})))),
    }
}

#[derive(Debug, Deserialize)]
pub struct ListItemQuery {
    pub category_code: Option<String>,
    pub item_type: Option<String>,
}

pub async fn list_items(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<ListItemQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.inventory_engine.list_items(
        Uuid::parse_str(&claims.org_id).unwrap_or_default(), query.category_code.as_deref(), query.item_type.as_deref(),
    ).await {
        Ok(items) => Ok(Json(serde_json::json!({"data": items}))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// On-Hand Balances
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct OnHandQuery {
    pub item_id: Option<Uuid>,
    pub inventory_org_id: Option<Uuid>,
}

pub async fn list_on_hand_balances(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<OnHandQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.inventory_engine.list_on_hand_balances(
        Uuid::parse_str(&claims.org_id).unwrap_or_default(), query.item_id, query.inventory_org_id,
    ).await {
        Ok(balances) => Ok(Json(serde_json::json!({"data": balances}))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Inventory Transactions
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ReceiveItemRequest {
    pub item_id: Uuid,
    pub item_code: Option<String>,
    pub item_description: Option<String>,
    pub to_inventory_org_id: Uuid,
    pub to_subinventory_id: Uuid,
    pub to_locator_id: Option<Uuid>,
    pub quantity: String,
    #[serde(default = "default_ea")]
    pub uom: String,
    #[serde(default = "default_zero")]
    pub unit_cost: String,
    pub lot_number: Option<String>,
    pub serial_number: Option<String>,
    pub revision: Option<String>,
    #[serde(default = "default_manual")]
    pub source_type: String,
    pub source_id: Option<Uuid>,
    pub source_number: Option<String>,
    pub reason_id: Option<Uuid>,
    pub reason_name: Option<String>,
    pub notes: Option<String>,
}

pub async fn receive_item(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<ReceiveItemRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    match state.inventory_engine.receive_item(
        Uuid::parse_str(&claims.org_id).unwrap_or_default(), req.item_id, req.item_code.as_deref(), req.item_description.as_deref(),
        req.to_inventory_org_id, req.to_subinventory_id, req.to_locator_id,
        &req.quantity, &req.uom, &req.unit_cost,
        req.lot_number.as_deref(), req.serial_number.as_deref(), req.revision.as_deref(),
        &req.source_type, req.source_id, req.source_number.as_deref(),
        req.reason_id, req.reason_name.as_deref(), req.notes.as_deref(),
        None,
    ).await {
        Ok(txn) => Ok((StatusCode::CREATED, Json(serde_json::to_value(txn).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => {
            error!("Failed to receive item: {}", e);
            Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct IssueItemRequest {
    pub item_id: Uuid,
    pub item_code: Option<String>,
    pub item_description: Option<String>,
    pub from_inventory_org_id: Uuid,
    pub from_subinventory_id: Uuid,
    pub from_locator_id: Option<Uuid>,
    pub quantity: String,
    #[serde(default = "default_ea")]
    pub uom: String,
    #[serde(default = "default_zero")]
    pub unit_cost: String,
    pub lot_number: Option<String>,
    pub serial_number: Option<String>,
    pub revision: Option<String>,
    #[serde(default = "default_manual")]
    pub source_type: String,
    pub source_id: Option<Uuid>,
    pub source_number: Option<String>,
    pub reason_id: Option<Uuid>,
    pub reason_name: Option<String>,
    pub notes: Option<String>,
}

pub async fn issue_item(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<IssueItemRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    match state.inventory_engine.issue_item(
        Uuid::parse_str(&claims.org_id).unwrap_or_default(), req.item_id, req.item_code.as_deref(), req.item_description.as_deref(),
        req.from_inventory_org_id, req.from_subinventory_id, req.from_locator_id,
        &req.quantity, &req.uom, &req.unit_cost,
        req.lot_number.as_deref(), req.serial_number.as_deref(), req.revision.as_deref(),
        &req.source_type, req.source_id, req.source_number.as_deref(),
        req.reason_id, req.reason_name.as_deref(), req.notes.as_deref(),
        None,
    ).await {
        Ok(txn) => Ok((StatusCode::CREATED, Json(serde_json::to_value(txn).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => {
            error!("Failed to issue item: {}", e);
            Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct TransferItemRequest {
    pub item_id: Uuid,
    pub item_code: Option<String>,
    pub item_description: Option<String>,
    pub from_inventory_org_id: Uuid,
    pub from_subinventory_id: Uuid,
    pub from_locator_id: Option<Uuid>,
    pub to_inventory_org_id: Uuid,
    pub to_subinventory_id: Uuid,
    pub to_locator_id: Option<Uuid>,
    pub quantity: String,
    #[serde(default = "default_ea")]
    pub uom: String,
    #[serde(default = "default_zero")]
    pub unit_cost: String,
    pub lot_number: Option<String>,
    pub serial_number: Option<String>,
    pub revision: Option<String>,
    pub reason_id: Option<Uuid>,
    pub reason_name: Option<String>,
    pub notes: Option<String>,
}

pub async fn transfer_item(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<TransferItemRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    match state.inventory_engine.transfer_item(
        Uuid::parse_str(&claims.org_id).unwrap_or_default(), req.item_id, req.item_code.as_deref(), req.item_description.as_deref(),
        req.from_inventory_org_id, req.from_subinventory_id, req.from_locator_id,
        req.to_inventory_org_id, req.to_subinventory_id, req.to_locator_id,
        &req.quantity, &req.uom, &req.unit_cost,
        req.lot_number.as_deref(), req.serial_number.as_deref(), req.revision.as_deref(),
        req.reason_id, req.reason_name.as_deref(), req.notes.as_deref(),
        None,
    ).await {
        Ok(txn) => Ok((StatusCode::CREATED, Json(serde_json::to_value(txn).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => {
            error!("Failed to transfer item: {}", e);
            Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct AdjustItemRequest {
    pub item_id: Uuid,
    pub item_code: Option<String>,
    pub item_description: Option<String>,
    pub inventory_org_id: Uuid,
    pub subinventory_id: Uuid,
    pub locator_id: Option<Uuid>,
    pub quantity_delta: String,
    #[serde(default = "default_ea")]
    pub uom: String,
    #[serde(default = "default_zero")]
    pub unit_cost: String,
    pub lot_number: Option<String>,
    pub serial_number: Option<String>,
    pub revision: Option<String>,
    pub reason_id: Option<Uuid>,
    pub reason_name: Option<String>,
    pub notes: Option<String>,
}

pub async fn adjust_item(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<AdjustItemRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    match state.inventory_engine.adjust_item(
        Uuid::parse_str(&claims.org_id).unwrap_or_default(), req.item_id, req.item_code.as_deref(), req.item_description.as_deref(),
        req.inventory_org_id, req.subinventory_id, req.locator_id,
        &req.quantity_delta, &req.uom, &req.unit_cost,
        req.lot_number.as_deref(), req.serial_number.as_deref(), req.revision.as_deref(),
        req.reason_id, req.reason_name.as_deref(), req.notes.as_deref(),
        None,
    ).await {
        Ok(txn) => Ok((StatusCode::CREATED, Json(serde_json::to_value(txn).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => {
            error!("Failed to adjust item: {}", e);
            Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListTransactionsQuery {
    pub item_id: Option<Uuid>,
    pub transaction_action: Option<String>,
    pub status: Option<String>,
}

pub async fn list_transactions(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<ListTransactionsQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.inventory_engine.list_transactions(
        Uuid::parse_str(&claims.org_id).unwrap_or_default(), query.item_id, query.transaction_action.as_deref(), query.status.as_deref(),
    ).await {
        Ok(txns) => Ok(Json(serde_json::json!({"data": txns}))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Subinventories
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateSubinventoryRequest {
    pub inventory_org_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(default = "default_storage")]
    pub subinventory_type: String,
    #[serde(default = "default_true")]
    pub asset_subinventory: bool,
    #[serde(default = "default_true")]
    pub quantity_tracked: bool,
    pub location_code: Option<String>,
}

pub async fn create_subinventory(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreateSubinventoryRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    match state.inventory_engine.create_subinventory(
        Uuid::parse_str(&claims.org_id).unwrap_or_default(), req.inventory_org_id, &req.code, &req.name,
        req.description.as_deref(), &req.subinventory_type,
        req.asset_subinventory, req.quantity_tracked,
        req.location_code.as_deref(), None,
    ).await {
        Ok(sub) => Ok((StatusCode::CREATED, Json(serde_json::to_value(sub).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()})))),
    }
}

pub async fn list_subinventories(
    State(state): State<Arc<AppState>>,
    Path(inventory_org_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.inventory_engine.list_subinventories(inventory_org_id).await {
        Ok(subs) => Ok(Json(serde_json::json!({"data": subs}))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Dashboard
// ============================================================================

pub async fn get_inventory_dashboard(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.inventory_engine.get_dashboard_summary(Uuid::parse_str(&claims.org_id).unwrap_or_default()).await {
        Ok(summary) => Ok(Json(serde_json::to_value(summary).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()})))),
    }
}
