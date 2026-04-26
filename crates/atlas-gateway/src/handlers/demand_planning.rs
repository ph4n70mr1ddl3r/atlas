//! Demand Planning / Demand Management Handlers
//!
//! Oracle Fusion SCM: Demand Management
//!
//! API endpoints for managing demand forecast methods, demand schedules,
//! schedule lines, demand history, forecast consumption, accuracy measurement,
//! and demand planning analytics.

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
// Forecast Method Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateMethodRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub method_type: String,
    pub parameters: Option<serde_json::Value>,
}

pub async fn create_method(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateMethodRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).ok();
    match state.demand_planning_engine.create_method(
        org_id, &payload.code, &payload.name, payload.description.as_deref(),
        &payload.method_type, payload.parameters.unwrap_or(serde_json::json!({})), user_id,
    ).await {
        Ok(method) => Ok((StatusCode::CREATED, Json(serde_json::to_value(method).unwrap_or_default()))),
        Err(e) => {
            error!("Failed to create forecast method: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 409 => StatusCode::CONFLICT, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

pub async fn list_methods(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.demand_planning_engine.list_methods(org_id).await {
        Ok(methods) => Ok(Json(serde_json::json!({"data": methods}))),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn get_method(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let _ = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.demand_planning_engine.get_method(id).await {
        Ok(Some(method)) => Ok(Json(serde_json::to_value(method).unwrap_or_default())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn delete_method(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(code): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.demand_planning_engine.delete_method(org_id, &code).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// ============================================================================
// Demand Schedule Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateScheduleRequest {
    pub schedule_number: String,
    pub name: String,
    pub description: Option<String>,
    pub method_id: Option<Uuid>,
    pub schedule_type: String,
    pub start_date: chrono::NaiveDate,
    pub end_date: chrono::NaiveDate,
    pub currency_code: Option<String>,
    pub confidence_level: Option<String>,
    pub owner_id: Option<Uuid>,
    pub owner_name: Option<String>,
}

pub async fn create_schedule(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateScheduleRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).ok();
    match state.demand_planning_engine.create_schedule(
        org_id, &payload.schedule_number, &payload.name, payload.description.as_deref(),
        payload.method_id, &payload.schedule_type,
        payload.start_date, payload.end_date,
        payload.currency_code.as_deref().unwrap_or("USD"),
        payload.confidence_level.as_deref().unwrap_or("medium"),
        payload.owner_id, payload.owner_name.as_deref(), user_id,
    ).await {
        Ok(schedule) => Ok((StatusCode::CREATED, Json(serde_json::to_value(schedule).unwrap_or_default()))),
        Err(e) => {
            error!("Failed to create schedule: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 409 => StatusCode::CONFLICT, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListSchedulesQuery {
    pub status: Option<String>,
}

pub async fn list_schedules(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListSchedulesQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.demand_planning_engine.list_schedules(org_id, query.status.as_deref()).await {
        Ok(schedules) => Ok(Json(serde_json::json!({"data": schedules}))),
        Err(e) => {
            error!("Error: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

pub async fn get_schedule(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let _ = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.demand_planning_engine.get_schedule(id).await {
        Ok(Some(schedule)) => Ok(Json(serde_json::to_value(schedule).unwrap_or_default())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn submit_schedule(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.demand_planning_engine.submit_schedule(id).await {
        Ok(schedule) => Ok(Json(serde_json::to_value(schedule).unwrap_or_default())),
        Err(e) => {
            error!("Error: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 404 => StatusCode::NOT_FOUND, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

pub async fn approve_schedule(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).ok();
    match state.demand_planning_engine.approve_schedule(id, user_id).await {
        Ok(schedule) => Ok(Json(serde_json::to_value(schedule).unwrap_or_default())),
        Err(e) => {
            error!("Error: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 404 => StatusCode::NOT_FOUND, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

pub async fn activate_schedule(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.demand_planning_engine.activate_schedule(id).await {
        Ok(schedule) => Ok(Json(serde_json::to_value(schedule).unwrap_or_default())),
        Err(e) => {
            error!("Error: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 404 => StatusCode::NOT_FOUND, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

pub async fn close_schedule(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.demand_planning_engine.close_schedule(id).await {
        Ok(schedule) => Ok(Json(serde_json::to_value(schedule).unwrap_or_default())),
        Err(e) => {
            error!("Error: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 404 => StatusCode::NOT_FOUND, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

pub async fn cancel_schedule(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.demand_planning_engine.cancel_schedule(id).await {
        Ok(schedule) => Ok(Json(serde_json::to_value(schedule).unwrap_or_default())),
        Err(e) => {
            error!("Error: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 404 => StatusCode::NOT_FOUND, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

pub async fn delete_schedule(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(schedule_number): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.demand_planning_engine.delete_schedule(org_id, &schedule_number).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// ============================================================================
// Schedule Line Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct AddScheduleLineRequest {
    pub item_code: String,
    pub item_name: Option<String>,
    pub item_category: Option<String>,
    pub warehouse_code: Option<String>,
    pub region: Option<String>,
    pub customer_group: Option<String>,
    pub period_start: chrono::NaiveDate,
    pub period_end: chrono::NaiveDate,
    pub forecast_quantity: String,
    pub unit_price: Option<String>,
    pub confidence_pct: Option<String>,
    pub notes: Option<String>,
}

pub async fn add_schedule_line(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(schedule_id): Path<Uuid>,
    Json(payload): Json<AddScheduleLineRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.demand_planning_engine.add_schedule_line(
        org_id, schedule_id, &payload.item_code, payload.item_name.as_deref(),
        payload.item_category.as_deref(), payload.warehouse_code.as_deref(),
        payload.region.as_deref(), payload.customer_group.as_deref(),
        payload.period_start, payload.period_end,
        &payload.forecast_quantity,
        payload.unit_price.as_deref().unwrap_or("0"),
        payload.confidence_pct.as_deref().unwrap_or("0"),
        payload.notes.as_deref(),
    ).await {
        Ok(line) => Ok((StatusCode::CREATED, Json(serde_json::to_value(line).unwrap_or_default()))),
        Err(e) => {
            error!("Failed to add schedule line: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 404 => StatusCode::NOT_FOUND, 409 => StatusCode::CONFLICT, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

pub async fn list_schedule_lines(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(schedule_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let _ = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.demand_planning_engine.list_schedule_lines(schedule_id).await {
        Ok(lines) => Ok(Json(serde_json::json!({"data": lines}))),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn delete_schedule_line(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    match state.demand_planning_engine.delete_schedule_line(id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            error!("Error: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 404 => StatusCode::NOT_FOUND, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

// ============================================================================
// Demand History Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateHistoryRequest {
    pub item_code: String,
    pub item_name: Option<String>,
    pub warehouse_code: Option<String>,
    pub region: Option<String>,
    pub customer_group: Option<String>,
    pub actual_date: chrono::NaiveDate,
    pub actual_quantity: String,
    pub actual_value: Option<String>,
    pub source_type: Option<String>,
    pub source_id: Option<Uuid>,
    pub source_line_id: Option<Uuid>,
}

pub async fn create_history(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateHistoryRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.demand_planning_engine.create_history(
        org_id, &payload.item_code, payload.item_name.as_deref(),
        payload.warehouse_code.as_deref(), payload.region.as_deref(),
        payload.customer_group.as_deref(), payload.actual_date,
        &payload.actual_quantity,
        payload.actual_value.as_deref().unwrap_or("0"),
        payload.source_type.as_deref().unwrap_or("manual"),
        payload.source_id, payload.source_line_id,
    ).await {
        Ok(history) => Ok((StatusCode::CREATED, Json(serde_json::to_value(history).unwrap_or_default()))),
        Err(e) => {
            error!("Failed to create history: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListHistoryQuery {
    pub item_code: Option<String>,
    pub start_date: Option<chrono::NaiveDate>,
    pub end_date: Option<chrono::NaiveDate>,
}

pub async fn list_history(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListHistoryQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.demand_planning_engine.list_history(
        org_id, query.item_code.as_deref(), query.start_date, query.end_date,
    ).await {
        Ok(history) => Ok(Json(serde_json::json!({"data": history}))),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn delete_history(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    match state.demand_planning_engine.delete_history(id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// ============================================================================
// Forecast Consumption Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ConsumeForecastRequest {
    pub schedule_line_id: Uuid,
    pub history_id: Option<Uuid>,
    pub consumed_quantity: String,
    pub consumed_date: chrono::NaiveDate,
    pub source_type: Option<String>,
    pub notes: Option<String>,
}

pub async fn consume_forecast(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<ConsumeForecastRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).ok();
    match state.demand_planning_engine.consume_forecast(
        org_id, payload.schedule_line_id, payload.history_id,
        &payload.consumed_quantity, payload.consumed_date,
        payload.source_type.as_deref().unwrap_or("manual"),
        payload.notes.as_deref(), user_id,
    ).await {
        Ok(consumption) => Ok((StatusCode::CREATED, Json(serde_json::to_value(consumption).unwrap_or_default()))),
        Err(e) => {
            error!("Failed to consume forecast: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 404 => StatusCode::NOT_FOUND, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

pub async fn list_consumption(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(schedule_line_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let _ = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.demand_planning_engine.list_consumption(schedule_line_id).await {
        Ok(consumption) => Ok(Json(serde_json::json!({"data": consumption}))),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn delete_consumption(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    match state.demand_planning_engine.delete_consumption(id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// ============================================================================
// Accuracy Measurement Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct MeasureAccuracyRequest {
    pub schedule_line_id: Uuid,
    pub actual_quantity: String,
    pub measurement_date: Option<chrono::NaiveDate>,
}

pub async fn measure_accuracy(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<MeasureAccuracyRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.demand_planning_engine.measure_accuracy(
        org_id, payload.schedule_line_id, &payload.actual_quantity, payload.measurement_date,
    ).await {
        Ok(accuracy) => Ok((StatusCode::CREATED, Json(serde_json::to_value(accuracy).unwrap_or_default()))),
        Err(e) => {
            error!("Failed to measure accuracy: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 404 => StatusCode::NOT_FOUND, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

pub async fn list_accuracy(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(schedule_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let _ = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.demand_planning_engine.list_accuracy(schedule_id).await {
        Ok(accuracy) => Ok(Json(serde_json::json!({"data": accuracy}))),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// ============================================================================
// Dashboard Handler
// ============================================================================

pub async fn get_demand_planning_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.demand_planning_engine.get_dashboard(org_id).await {
        Ok(dashboard) => Ok(Json(serde_json::to_value(dashboard).unwrap_or_default())),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}
