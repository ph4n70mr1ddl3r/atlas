//! KPI & Embedded Analytics Handlers
//!
//! Oracle Fusion OTBI-inspired analytics endpoints:
//! - KPI definition CRUD
//! - KPI data point recording and retrieval
//! - Dashboard management
//! - Dashboard widget management
//! - KPI dashboard summary

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
// KPI Definitions
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateKpiRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub category: String,
    pub unit_of_measure: Option<String>,
    pub direction: Option<String>,
    pub target_value: String,
    pub warning_threshold: Option<String>,
    pub critical_threshold: Option<String>,
    pub data_source_query: Option<String>,
    pub evaluation_frequency: Option<String>,
}

/// Create a KPI definition
pub async fn create_kpi(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateKpiRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let kpi = state.kpi_engine.create_kpi(
        org_id,
        &payload.code,
        &payload.name,
        payload.description.as_deref(),
        &payload.category,
        payload.unit_of_measure.as_deref().unwrap_or("number"),
        payload.direction.as_deref().unwrap_or("higher_is_better"),
        &payload.target_value,
        payload.warning_threshold.as_deref(),
        payload.critical_threshold.as_deref(),
        payload.data_source_query.as_deref(),
        payload.evaluation_frequency.as_deref().unwrap_or("manual"),
        Some(user_id),
    ).await.map_err(|e| {
        tracing::error!("Create KPI error: {}", e);
        match e {
            atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(kpi).unwrap_or_default())))
}

/// Get a KPI definition by ID
pub async fn get_kpi(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let kpi = state.kpi_engine.get_kpi(id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match kpi {
        Some(k) => Ok(Json(serde_json::to_value(k).unwrap_or_default())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

#[derive(Debug, Deserialize)]
pub struct ListKpisParams {
    pub category: Option<String>,
}

/// List KPIs for the organization
pub async fn list_kpis(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<ListKpisParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let kpis = state.kpi_engine.list_kpis(org_id, params.category.as_deref()).await
        .map_err(|e| { error!("List KPIs error: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;

    Ok(Json(serde_json::json!({
        "data": kpis,
        "meta": { "total": kpis.len() }
    })))
}

/// Delete a KPI by code
pub async fn delete_kpi(
    State(state): State<Arc<AppState>>,
    Path(code): Path<String>,
    claims: Extension<Claims>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    state.kpi_engine.delete_kpi(org_id, &code).await.map_err(|e| {
        error!("Delete KPI error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// KPI Data Points
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordDataPointRequest {
    pub value: String,
    pub period_start: Option<String>,
    pub period_end: Option<String>,
    pub notes: Option<String>,
}

/// Record a data point for a KPI
pub async fn record_data_point(
    State(state): State<Arc<AppState>>,
    Path(kpi_id): Path<Uuid>,
    claims: Extension<Claims>,
    Json(payload): Json<RecordDataPointRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let period_start = payload.period_start
        .as_deref()
        .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());
    let period_end = payload.period_end
        .as_deref()
        .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());

    let dp = state.kpi_engine.record_data_point(
        org_id, kpi_id, &payload.value, period_start, period_end,
        payload.notes.as_deref(), Some(user_id),
    ).await.map_err(|e| {
        error!("Record data point error: {}", e);
        match e {
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(dp).unwrap_or_default())))
}

/// Get the latest data point for a KPI
pub async fn get_latest_data_point(
    State(state): State<Arc<AppState>>,
    Path(kpi_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let dp = state.kpi_engine.get_latest_data_point(kpi_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match dp {
        Some(d) => Ok(Json(serde_json::to_value(d).unwrap_or_default())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

#[derive(Debug, Deserialize)]
pub struct ListDataPointsParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// List data points for a KPI
pub async fn list_data_points(
    State(state): State<Arc<AppState>>,
    Path(kpi_id): Path<Uuid>,
    Query(params): Query<ListDataPointsParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let limit = params.limit.unwrap_or(50).clamp(1, 200);
    let offset = params.offset.unwrap_or(0).max(0);

    let dps = state.kpi_engine.list_data_points(kpi_id, limit, offset).await
        .map_err(|e| { error!("List data points error: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;

    Ok(Json(serde_json::json!({
        "data": dps,
        "meta": { "kpi_id": kpi_id, "limit": limit, "offset": offset }
    })))
}

/// Delete a data point
pub async fn delete_data_point(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    state.kpi_engine.delete_data_point(id).await.map_err(|e| {
        error!("Delete data point error: {}", e);
        match e {
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Dashboards
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDashboardRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub is_shared: Option<bool>,
    pub is_default: Option<bool>,
    pub layout_config: Option<serde_json::Value>,
}

/// Create a dashboard
pub async fn create_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateDashboardRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let dashboard = state.kpi_engine.create_dashboard(
        org_id,
        &payload.code,
        &payload.name,
        payload.description.as_deref(),
        Some(user_id),
        payload.is_shared.unwrap_or(false),
        payload.is_default.unwrap_or(false),
        payload.layout_config.clone().unwrap_or(serde_json::json!({})),
        Some(user_id),
    ).await.map_err(|e| {
        error!("Create dashboard error: {}", e);
        match e {
            atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(dashboard).unwrap_or_default())))
}

/// Get a dashboard by ID
pub async fn get_dashboard(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let dashboard = state.kpi_engine.get_dashboard(id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match dashboard {
        Some(d) => Ok(Json(serde_json::to_value(d).unwrap_or_default())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// List dashboards
pub async fn list_dashboards(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let dashboards = state.kpi_engine.list_dashboards(org_id, None).await
        .map_err(|e| { error!("List dashboards error: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;

    Ok(Json(serde_json::json!({
        "data": dashboards,
        "meta": { "total": dashboards.len() }
    })))
}

/// Delete a dashboard by code
pub async fn delete_dashboard(
    State(state): State<Arc<AppState>>,
    Path(code): Path<String>,
    claims: Extension<Claims>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    state.kpi_engine.delete_dashboard(org_id, &code).await.map_err(|e| {
        error!("Delete dashboard error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Dashboard Widgets
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddWidgetRequest {
    pub kpi_id: Option<Uuid>,
    pub widget_type: String,
    pub title: String,
    pub position_row: Option<i32>,
    pub position_col: Option<i32>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub display_config: Option<serde_json::Value>,
}

/// Add a widget to a dashboard
pub async fn add_widget(
    State(state): State<Arc<AppState>>,
    Path(dashboard_id): Path<Uuid>,
    claims: Extension<Claims>,
    Json(payload): Json<AddWidgetRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let _org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let widget = state.kpi_engine.add_widget(
        dashboard_id,
        payload.kpi_id,
        &payload.widget_type,
        &payload.title,
        payload.position_row.unwrap_or(0),
        payload.position_col.unwrap_or(0),
        payload.width.unwrap_or(1),
        payload.height.unwrap_or(1),
        payload.display_config.clone().unwrap_or(serde_json::json!({})),
    ).await.map_err(|e| {
        error!("Add widget error: {}", e);
        match e {
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(widget).unwrap_or_default())))
}

/// List widgets for a dashboard
pub async fn list_widgets(
    State(state): State<Arc<AppState>>,
    Path(dashboard_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let widgets = state.kpi_engine.list_widgets(dashboard_id).await
        .map_err(|e| { error!("List widgets error: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;

    Ok(Json(serde_json::json!({
        "data": widgets,
        "meta": { "dashboard_id": dashboard_id, "total": widgets.len() }
    })))
}

/// Delete a widget
pub async fn delete_widget(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    state.kpi_engine.delete_widget(id).await.map_err(|e| {
        error!("Delete widget error: {}", e);
        match e {
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Dashboard Summary
// ============================================================================

/// Get the KPI analytics dashboard summary
pub async fn get_kpi_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let summary = state.kpi_engine.get_dashboard_summary(org_id).await
        .map_err(|e| { error!("KPI dashboard error: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;

    Ok(Json(serde_json::to_value(summary).unwrap_or_default()))
}
