//! Warehouse Management API Handlers
//!
//! Oracle Fusion Cloud Warehouse Management
//!
//! Endpoints for managing warehouse operations including:
//! - Warehouses (create, get, list, delete)
//! - Warehouse zones (create, get, list, delete)
//! - Put-away rules (create, get, list, delete)
//! - Warehouse tasks (create, get, list, start, complete, cancel, delete)
//! - Pick waves (create, get, list, release, complete, cancel, delete)
//! - Dashboard

use axum::{
    extract::{Path, Query, State, Extension},
    Json,
    http::StatusCode,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;
use tracing::error;

use atlas_shared::AtlasError;
use crate::AppState;
use crate::handlers::auth::Claims;

// Common task-creation payload shared by both `create_task` (resolve warehouse
// from wave/zone) and `create_task_for_warehouse` (warehouse in URL) endpoints.
type TaskPayload = CreateTaskRequest;

// ============================================================================
// Query Parameters
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ListWarehousesQuery {
    pub active_only: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ListTasksQuery {
    pub warehouse_id: Option<Uuid>,
    pub status: Option<String>,
    pub task_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListWavesQuery {
    pub warehouse_id: Option<Uuid>,
    pub status: Option<String>,
}

// ============================================================================
// Warehouse Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateWarehouseRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub location_code: Option<String>,
}

pub async fn create_warehouse(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateWarehouseRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = claims.org_uuid()?;
    let user_id = claims.user_uuid()?;

    match state.warehouse_management_engine.create_warehouse(
        org_id, &payload.code, &payload.name,
        payload.description.as_deref(), payload.location_code.as_deref(),
        Some(user_id),
    ).await {
        Ok(wh) => Ok((StatusCode::CREATED, Json(serde_json::to_value(wh).unwrap_or_default()))),
        Err(e) => { error!("Failed to create warehouse: {}", e); Err(map_error(e)) }
    }
}

pub async fn get_warehouse(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = claims.org_uuid()?;

    match state.warehouse_management_engine.get_warehouse(org_id, id).await {
        Ok(Some(wh)) => Ok(Json(serde_json::to_value(wh).unwrap_or_default())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Failed to get warehouse: {}", e); Err(map_error(e)) }
    }
}

pub async fn list_warehouses(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<ListWarehousesQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = claims.org_uuid()?;

    match state.warehouse_management_engine.list_warehouses(org_id, params.active_only.unwrap_or(false)).await {
        Ok(warehouses) => Ok(Json(serde_json::json!({ "data": warehouses }))),
        Err(e) => { error!("Failed to list warehouses: {}", e); Err(map_error(e)) }
    }
}

pub async fn delete_warehouse(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let org_id = claims.org_uuid()?;

    match state.warehouse_management_engine.delete_warehouse(org_id, id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => { error!("Failed to delete warehouse: {}", e); Err(map_error(e)) }
    }
}

// ============================================================================
// Zone Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateZoneRequest {
    pub code: String,
    pub name: String,
    pub zone_type: String,
    pub description: Option<String>,
    pub aisle_count: Option<i32>,
}

pub async fn create_zone(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(warehouse_id): Path<Uuid>,
    Json(payload): Json<CreateZoneRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = claims.org_uuid()?;

    match state.warehouse_management_engine.create_zone(
        org_id, warehouse_id, &payload.code, &payload.name, &payload.zone_type,
        payload.description.as_deref(), payload.aisle_count,
    ).await {
        Ok(zone) => Ok((StatusCode::CREATED, Json(serde_json::to_value(zone).unwrap_or_default()))),
        Err(e) => { error!("Failed to create zone: {}", e); Err(map_error(e)) }
    }
}

pub async fn list_zones(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(warehouse_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = claims.org_uuid()?;

    match state.warehouse_management_engine.list_zones(org_id, warehouse_id).await {
        Ok(zones) => Ok(Json(serde_json::json!({ "data": zones }))),
        Err(e) => { error!("Failed to list zones: {}", e); Err(map_error(e)) }
    }
}

pub async fn delete_zone(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let org_id = claims.org_uuid()?;

    match state.warehouse_management_engine.delete_zone(org_id, id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => { error!("Failed to delete zone: {}", e); Err(map_error(e)) }
    }
}

// ============================================================================
// Put-Away Rule Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePutAwayRuleRequest {
    pub rule_name: String,
    pub description: Option<String>,
    pub priority: Option<i32>,
    pub item_category: Option<String>,
    pub target_zone_type: String,
    pub strategy: String,
}

pub async fn create_put_away_rule(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(warehouse_id): Path<Uuid>,
    Json(payload): Json<CreatePutAwayRuleRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = claims.org_uuid()?;

    match state.warehouse_management_engine.create_put_away_rule(
        org_id, warehouse_id, &payload.rule_name, payload.description.as_deref(),
        payload.priority.unwrap_or(10), payload.item_category.as_deref(),
        &payload.target_zone_type, &payload.strategy,
    ).await {
        Ok(rule) => Ok((StatusCode::CREATED, Json(serde_json::to_value(rule).unwrap_or_default()))),
        Err(e) => { error!("Failed to create put-away rule: {}", e); Err(map_error(e)) }
    }
}

pub async fn list_put_away_rules(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(warehouse_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = claims.org_uuid()?;

    match state.warehouse_management_engine.list_put_away_rules(org_id, warehouse_id).await {
        Ok(rules) => Ok(Json(serde_json::json!({ "data": rules }))),
        Err(e) => { error!("Failed to list put-away rules: {}", e); Err(map_error(e)) }
    }
}

pub async fn delete_put_away_rule(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let org_id = claims.org_uuid()?;

    match state.warehouse_management_engine.delete_put_away_rule(org_id, id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => { error!("Failed to delete put-away rule: {}", e); Err(map_error(e)) }
    }
}

// ============================================================================
// Warehouse Task Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTaskRequest {
    pub task_number: String,
    pub task_type: String,
    pub priority: Option<String>,
    pub item_id: Option<Uuid>,
    pub item_description: Option<String>,
    pub from_zone_id: Option<Uuid>,
    pub to_zone_id: Option<Uuid>,
    pub from_location: Option<String>,
    pub to_location: Option<String>,
    pub quantity: Option<String>,
    pub uom: Option<String>,
    pub source_document: Option<String>,
    pub source_document_id: Option<Uuid>,
    pub source_line_id: Option<Uuid>,
    pub wave_id: Option<Uuid>,
}

pub async fn create_task(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateTaskRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = claims.org_uuid()?;
    let user_id = claims.user_uuid()?;

    // Resolve warehouse_id from wave or zone
    let warehouse_id = if let Some(wave_id) = payload.wave_id {
        if let Ok(Some(wave)) = state.warehouse_management_engine.get_wave(org_id, wave_id).await {
            wave.warehouse_id
        } else {
            return Err(StatusCode::BAD_REQUEST);
        }
    } else if let Some(zone_id) = payload.to_zone_id.or(payload.from_zone_id) {
        if let Ok(Some(zone)) = state.warehouse_management_engine.get_zone(org_id, zone_id).await {
            zone.warehouse_id
        } else {
            return Err(StatusCode::BAD_REQUEST);
        }
    } else {
        return Err(StatusCode::BAD_REQUEST);
    };

    match state.warehouse_management_engine.create_task(
        org_id, warehouse_id, &payload.task_number, &payload.task_type,
        payload.priority.as_deref().unwrap_or("medium"),
        payload.item_id, payload.item_description.as_deref(),
        payload.from_zone_id, payload.to_zone_id,
        payload.from_location.as_deref(), payload.to_location.as_deref(),
        payload.quantity, payload.uom.as_deref(),
        payload.source_document.as_deref(), payload.source_document_id,
        payload.source_line_id, payload.wave_id, Some(user_id),
    ).await {
        Ok(task) => Ok((StatusCode::CREATED, Json(serde_json::to_value(task).unwrap_or_default()))),
        Err(e) => { error!("Failed to create task: {}", e); Err(map_error(e)) }
    }
}

pub async fn create_task_for_warehouse(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(warehouse_id): Path<Uuid>,
    Json(payload): Json<TaskPayload>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = claims.org_uuid()?;
    let user_id = claims.user_uuid()?;

    match state.warehouse_management_engine.create_task(
        org_id, warehouse_id, &payload.task_number, &payload.task_type,
        payload.priority.as_deref().unwrap_or("medium"),
        payload.item_id, payload.item_description.as_deref(),
        payload.from_zone_id, payload.to_zone_id,
        payload.from_location.as_deref(), payload.to_location.as_deref(),
        payload.quantity, payload.uom.as_deref(),
        payload.source_document.as_deref(), payload.source_document_id,
        payload.source_line_id, payload.wave_id, Some(user_id),
    ).await {
        Ok(task) => Ok((StatusCode::CREATED, Json(serde_json::to_value(task).unwrap_or_default()))),
        Err(e) => { error!("Failed to create task: {}", e); Err(map_error(e)) }
    }
}

pub async fn get_task(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = claims.org_uuid()?;

    match state.warehouse_management_engine.get_task(org_id, id).await {
        Ok(Some(task)) => Ok(Json(serde_json::to_value(task).unwrap_or_default())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Failed to get task: {}", e); Err(map_error(e)) }
    }
}

pub async fn list_tasks(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<ListTasksQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = claims.org_uuid()?;

    match state.warehouse_management_engine.list_tasks(
        org_id, params.warehouse_id, params.status.as_deref(), params.task_type.as_deref(),
    ).await {
        Ok(tasks) => Ok(Json(serde_json::json!({ "data": tasks }))),
        Err(e) => { error!("Failed to list tasks: {}", e); Err(map_error(e)) }
    }
}

pub async fn start_task(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = claims.org_uuid()?;

    match state.warehouse_management_engine.start_task(org_id, id, None).await {
        Ok(task) => Ok(Json(serde_json::to_value(task).unwrap_or_default())),
        Err(e) => { error!("Failed to start task: {}", e); Err(map_error(e)) }
    }
}

pub async fn complete_task(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = claims.org_uuid()?;

    match state.warehouse_management_engine.complete_task(org_id, id).await {
        Ok(task) => Ok(Json(serde_json::to_value(task).unwrap_or_default())),
        Err(e) => { error!("Failed to complete task: {}", e); Err(map_error(e)) }
    }
}

pub async fn cancel_task(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = claims.org_uuid()?;

    match state.warehouse_management_engine.cancel_task(org_id, id).await {
        Ok(task) => Ok(Json(serde_json::to_value(task).unwrap_or_default())),
        Err(e) => { error!("Failed to cancel task: {}", e); Err(map_error(e)) }
    }
}

pub async fn delete_task(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let org_id = claims.org_uuid()?;

    match state.warehouse_management_engine.delete_task(org_id, id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => { error!("Failed to delete task: {}", e); Err(map_error(e)) }
    }
}

// ============================================================================
// Pick Wave Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateWaveRequest {
    pub wave_number: String,
    pub priority: Option<String>,
    pub cut_off_date: Option<chrono::NaiveDate>,
    pub shipping_method: Option<String>,
}

pub async fn create_wave(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(warehouse_id): Path<Uuid>,
    Json(payload): Json<CreateWaveRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = claims.org_uuid()?;
    let user_id = claims.user_uuid()?;

    match state.warehouse_management_engine.create_wave(
        org_id, warehouse_id, &payload.wave_number,
        payload.priority.as_deref().unwrap_or("medium"),
        payload.cut_off_date, payload.shipping_method.as_deref(), Some(user_id),
    ).await {
        Ok(wave) => Ok((StatusCode::CREATED, Json(serde_json::to_value(wave).unwrap_or_default()))),
        Err(e) => { error!("Failed to create wave: {}", e); Err(map_error(e)) }
    }
}

pub async fn get_wave(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = claims.org_uuid()?;

    match state.warehouse_management_engine.get_wave(org_id, id).await {
        Ok(Some(wave)) => Ok(Json(serde_json::to_value(wave).unwrap_or_default())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Failed to get wave: {}", e); Err(map_error(e)) }
    }
}

pub async fn list_waves(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<ListWavesQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = claims.org_uuid()?;

    match state.warehouse_management_engine.list_waves(
        org_id, params.warehouse_id, params.status.as_deref(),
    ).await {
        Ok(waves) => Ok(Json(serde_json::json!({ "data": waves }))),
        Err(e) => { error!("Failed to list waves: {}", e); Err(map_error(e)) }
    }
}

pub async fn release_wave(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = claims.org_uuid()?;

    match state.warehouse_management_engine.release_wave(org_id, id).await {
        Ok(wave) => Ok(Json(serde_json::to_value(wave).unwrap_or_default())),
        Err(e) => { error!("Failed to release wave: {}", e); Err(map_error(e)) }
    }
}

pub async fn complete_wave(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = claims.org_uuid()?;

    match state.warehouse_management_engine.complete_wave(org_id, id).await {
        Ok(wave) => Ok(Json(serde_json::to_value(wave).unwrap_or_default())),
        Err(e) => { error!("Failed to complete wave: {}", e); Err(map_error(e)) }
    }
}

pub async fn cancel_wave(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = claims.org_uuid()?;

    match state.warehouse_management_engine.cancel_wave(org_id, id).await {
        Ok(wave) => Ok(Json(serde_json::to_value(wave).unwrap_or_default())),
        Err(e) => { error!("Failed to cancel wave: {}", e); Err(map_error(e)) }
    }
}

pub async fn delete_wave(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let org_id = claims.org_uuid()?;

    match state.warehouse_management_engine.delete_wave(org_id, id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => { error!("Failed to delete wave: {}", e); Err(map_error(e)) }
    }
}

// ============================================================================
// Dashboard Handler
// ============================================================================

pub async fn get_warehouse_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = claims.org_uuid()?;

    match state.warehouse_management_engine.get_dashboard(org_id).await {
        Ok(dashboard) => Ok(Json(serde_json::to_value(dashboard).unwrap_or_default())),
        Err(e) => { error!("Failed to get warehouse dashboard: {}", e); Err(map_error(e)) }
    }
}

// ============================================================================
// Helpers
// ============================================================================

fn map_error(e: AtlasError) -> StatusCode {
    match e.status_code().try_into() {
        Ok(code) => code,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
