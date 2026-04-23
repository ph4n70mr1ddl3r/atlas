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

use crate::AppState;
use crate::handlers::auth::Claims;

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
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.warehouse_management_engine.create_warehouse(
        org_id, &payload.code, &payload.name,
        payload.description.as_deref(), payload.location_code.as_deref(),
        Some(user_id),
    ).await {
        Ok(wh) => Ok((StatusCode::CREATED, Json(serde_json::to_value(wh).unwrap_or_default()))),
        Err(e) => {
            error!("Failed to create warehouse: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                409 => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn get_warehouse(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.warehouse_management_engine.get_warehouse(id).await {
        Ok(Some(wh)) => Ok(Json(serde_json::to_value(wh).unwrap_or_default())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Failed to get warehouse: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn list_warehouses(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<ListWarehousesQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.warehouse_management_engine.list_warehouses(org_id, params.active_only.unwrap_or(false)).await {
        Ok(warehouses) => Ok(Json(serde_json::to_value(warehouses).unwrap_or_default())),
        Err(e) => { error!("Failed to list warehouses: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn delete_warehouse(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    match state.warehouse_management_engine.delete_warehouse(id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            error!("Failed to delete warehouse: {}", e);
            Err(match e.status_code() {
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
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
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.warehouse_management_engine.create_zone(
        org_id, warehouse_id, &payload.code, &payload.name, &payload.zone_type,
        payload.description.as_deref(), payload.aisle_count,
    ).await {
        Ok(zone) => Ok((StatusCode::CREATED, Json(serde_json::to_value(zone).unwrap_or_default()))),
        Err(e) => {
            error!("Failed to create zone: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                409 => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn list_zones(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(warehouse_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.warehouse_management_engine.list_zones(warehouse_id).await {
        Ok(zones) => Ok(Json(serde_json::to_value(zones).unwrap_or_default())),
        Err(e) => { error!("Failed to list zones: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn delete_zone(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    match state.warehouse_management_engine.delete_zone(id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            error!("Failed to delete zone: {}", e);
            Err(match e.status_code() {
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
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
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.warehouse_management_engine.create_put_away_rule(
        org_id, warehouse_id, &payload.rule_name, payload.description.as_deref(),
        payload.priority.unwrap_or(10), payload.item_category.as_deref(),
        &payload.target_zone_type, &payload.strategy,
    ).await {
        Ok(rule) => Ok((StatusCode::CREATED, Json(serde_json::to_value(rule).unwrap_or_default()))),
        Err(e) => {
            error!("Failed to create put-away rule: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn list_put_away_rules(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(warehouse_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.warehouse_management_engine.list_put_away_rules(warehouse_id).await {
        Ok(rules) => Ok(Json(serde_json::to_value(rules).unwrap_or_default())),
        Err(e) => { error!("Failed to list put-away rules: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn delete_put_away_rule(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    match state.warehouse_management_engine.delete_put_away_rule(id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            error!("Failed to delete put-away rule: {}", e);
            Err(match e.status_code() {
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
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
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Extract warehouse_id from payload or source
    // For standalone tasks, warehouse_id comes from the zone or wave
    let _warehouse_id = payload.wave_id
        .or(payload.to_zone_id)
        .or(payload.from_zone_id)
        .ok_or(StatusCode::BAD_REQUEST)?;

    // Try to resolve warehouse_id from wave or zone
    let warehouse_id = if let Some(wave_id) = payload.wave_id {
        if let Ok(Some(wave)) = state.warehouse_management_engine.get_wave(wave_id).await {
            wave.warehouse_id
        } else {
            return Err(StatusCode::BAD_REQUEST);
        }
    } else if let Some(zone_id) = payload.to_zone_id.or(payload.from_zone_id) {
        if let Ok(Some(zone)) = state.warehouse_management_engine.get_zone(zone_id).await {
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
        Err(e) => {
            error!("Failed to create task: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                409 => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTaskForWarehouseRequest {
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

pub async fn create_task_for_warehouse(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(warehouse_id): Path<Uuid>,
    Json(payload): Json<CreateTaskForWarehouseRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

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
        Err(e) => {
            error!("Failed to create task: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                409 => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn get_task(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.warehouse_management_engine.get_task(id).await {
        Ok(Some(task)) => Ok(Json(serde_json::to_value(task).unwrap_or_default())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Failed to get task: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn list_tasks(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<ListTasksQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.warehouse_management_engine.list_tasks(
        org_id, params.warehouse_id, params.status.as_deref(), params.task_type.as_deref(),
    ).await {
        Ok(tasks) => Ok(Json(serde_json::to_value(tasks).unwrap_or_default())),
        Err(e) => { error!("Failed to list tasks: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartTaskRequest {
    pub assigned_to: Option<Uuid>,
}

pub async fn start_task(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.warehouse_management_engine.start_task(id, None).await {
        Ok(task) => Ok(Json(serde_json::to_value(task).unwrap_or_default())),
        Err(e) => {
            error!("Failed to start task: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn complete_task(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.warehouse_management_engine.complete_task(id).await {
        Ok(task) => Ok(Json(serde_json::to_value(task).unwrap_or_default())),
        Err(e) => {
            error!("Failed to complete task: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn cancel_task(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.warehouse_management_engine.cancel_task(id).await {
        Ok(task) => Ok(Json(serde_json::to_value(task).unwrap_or_default())),
        Err(e) => {
            error!("Failed to cancel task: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn delete_task(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    match state.warehouse_management_engine.delete_task(id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            error!("Failed to delete task: {}", e);
            Err(match e.status_code() {
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
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
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.warehouse_management_engine.create_wave(
        org_id, warehouse_id, &payload.wave_number,
        payload.priority.as_deref().unwrap_or("medium"),
        payload.cut_off_date, payload.shipping_method.as_deref(), Some(user_id),
    ).await {
        Ok(wave) => Ok((StatusCode::CREATED, Json(serde_json::to_value(wave).unwrap_or_default()))),
        Err(e) => {
            error!("Failed to create wave: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                409 => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn get_wave(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.warehouse_management_engine.get_wave(id).await {
        Ok(Some(wave)) => Ok(Json(serde_json::to_value(wave).unwrap_or_default())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Failed to get wave: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn list_waves(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<ListWavesQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.warehouse_management_engine.list_waves(
        org_id, params.warehouse_id, params.status.as_deref(),
    ).await {
        Ok(waves) => Ok(Json(serde_json::to_value(waves).unwrap_or_default())),
        Err(e) => { error!("Failed to list waves: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn release_wave(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.warehouse_management_engine.release_wave(id).await {
        Ok(wave) => Ok(Json(serde_json::to_value(wave).unwrap_or_default())),
        Err(e) => {
            error!("Failed to release wave: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn complete_wave(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.warehouse_management_engine.complete_wave(id).await {
        Ok(wave) => Ok(Json(serde_json::to_value(wave).unwrap_or_default())),
        Err(e) => {
            error!("Failed to complete wave: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn cancel_wave(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.warehouse_management_engine.cancel_wave(id).await {
        Ok(wave) => Ok(Json(serde_json::to_value(wave).unwrap_or_default())),
        Err(e) => {
            error!("Failed to cancel wave: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn delete_wave(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    match state.warehouse_management_engine.delete_wave(id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            error!("Failed to delete wave: {}", e);
            Err(match e.status_code() {
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// ============================================================================
// Dashboard Handler
// ============================================================================

pub async fn get_warehouse_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.warehouse_management_engine.get_dashboard(org_id).await {
        Ok(dashboard) => Ok(Json(serde_json::to_value(dashboard).unwrap_or_default())),
        Err(e) => {
            error!("Failed to get warehouse dashboard: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
