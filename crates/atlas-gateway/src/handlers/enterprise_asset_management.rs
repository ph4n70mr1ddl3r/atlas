//! Enterprise Asset Management (eAM) HTTP Handlers
//!
//! Oracle Fusion Cloud: Maintenance Management endpoints

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension, Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::handlers::auth::Claims;
use crate::AppState;

// ============================================================================
// Query Parameters
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ListAssetsQuery {
    pub status: Option<String>,
    pub asset_group: Option<String>,
    pub criticality: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListWorkOrdersQuery {
    pub status: Option<String>,
    pub work_order_type: Option<String>,
    pub priority: Option<String>,
    pub asset_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct ListPmSchedulesQuery {
    pub status: Option<String>,
    pub asset_id: Option<Uuid>,
}

// ============================================================================
// Location Handlers
// ============================================================================

pub async fn create_location(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(body): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let code = body["code"].as_str().unwrap_or("");
    let name = body["name"].as_str().unwrap_or("");
    let description = body["description"].as_str();
    let location_type = body["locationType"].as_str();
    let address = body["address"].as_str();

    let loc = state.eam_engine.create_location(
        org_id, code, name, description, None,
        location_type, address, Some(user_id),
    ).await.map_err(|e| {
        tracing::error!("Create location error: {}", e);
        match e {
            atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;

    Ok((StatusCode::CREATED, Json(loc)))
}

pub async fn list_locations(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let locs = state.eam_engine.list_locations(org_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({ "data": locs })))
}

pub async fn get_location(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let loc = state.eam_engine.get_location(id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match loc {
        Some(l) => Ok(Json(l)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn delete_location(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(code): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    state.eam_engine.delete_location(org_id, &code).await.map_err(|e| {
        match e {
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Asset Definition Handlers
// ============================================================================

pub async fn create_asset(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(body): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let asset_number = body["assetNumber"].as_str().unwrap_or("");
    let name = body["name"].as_str().unwrap_or("");
    let description = body["description"].as_str();
    let asset_group = body["assetGroup"].as_str().unwrap_or("general");
    let asset_criticality = body["assetCriticality"].as_str().unwrap_or("medium");
    let location_id = body["locationId"].as_str().and_then(|s| Uuid::parse_str(s).ok());
    let location_name = body["locationName"].as_str();
    let parent_asset_id = body["parentAssetId"].as_str().and_then(|s| Uuid::parse_str(s).ok());
    let serial_number = body["serialNumber"].as_str();
    let manufacturer = body["manufacturer"].as_str();
    let model = body["model"].as_str();
    let install_date = body["installDate"].as_str()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let warranty_expiry = body["warrantyExpiry"].as_str()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let meter_reading = body.get("meterReading").cloned().filter(|v| !v.is_null());

    let asset = state.eam_engine.create_asset(
        org_id, asset_number, name, description,
        asset_group, asset_criticality,
        location_id, location_name, parent_asset_id,
        serial_number, manufacturer, model,
        install_date, warranty_expiry, meter_reading, Some(user_id),
    ).await.map_err(|e| {
        tracing::error!("Create asset error: {}", e);
        match e {
            atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(asset).unwrap_or_default())))
}

pub async fn list_assets(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListAssetsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let assets = state.eam_engine.list_assets(
        org_id, query.status.as_deref(), query.asset_group.as_deref(), query.criticality.as_deref(),
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({ "data": assets })))
}

pub async fn get_asset(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let asset = state.eam_engine.get_asset(id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match asset {
        Some(a) => Ok(Json(serde_json::to_value(a).unwrap_or_default())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn update_asset_status(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let status = body["status"].as_str().unwrap_or("");
    let asset = state.eam_engine.update_asset_status(id, status).await.map_err(|e| {
        match e {
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;
    Ok(Json(serde_json::to_value(asset).unwrap_or_default()))
}

pub async fn update_asset_meter(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let meter_reading = body.get("meterReading").cloned().unwrap_or(serde_json::json!({}));
    let asset = state.eam_engine.update_asset_meter(id, meter_reading).await.map_err(|e| {
        match e {
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;
    Ok(Json(serde_json::to_value(asset).unwrap_or_default()))
}

pub async fn delete_asset(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(asset_number): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    state.eam_engine.delete_asset(org_id, &asset_number).await.map_err(|e| {
        match e {
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Work Order Handlers
// ============================================================================

pub async fn create_work_order(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(body): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let work_order_number = body["workOrderNumber"].as_str().unwrap_or("");
    let title = body["title"].as_str().unwrap_or("");
    let description = body["description"].as_str();
    let work_order_type = body["workOrderType"].as_str().unwrap_or("corrective");
    let priority = body["priority"].as_str().unwrap_or("normal");
    let asset_id: Uuid = serde_json::from_value(body["assetId"].clone())
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let assigned_to = body["assignedTo"].as_str().and_then(|s| Uuid::parse_str(s).ok());
    let assigned_to_name = body["assignedToName"].as_str();
    let scheduled_start = body["scheduledStart"].as_str()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let scheduled_end = body["scheduledEnd"].as_str()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let estimated_hours = body.get("estimatedHours").cloned().filter(|v| !v.is_null());
    let estimated_cost = body["estimatedCost"].as_str();
    let failure_code = body["failureCode"].as_str();
    let cause_code = body["causeCode"].as_str();

    let wo = state.eam_engine.create_work_order(
        org_id, work_order_number, title, description,
        work_order_type, priority, asset_id,
        assigned_to, assigned_to_name,
        scheduled_start, scheduled_end,
        estimated_hours, estimated_cost,
        failure_code, cause_code, Some(user_id),
    ).await.map_err(|e| {
        tracing::error!("Create work order error: {}", e);
        match e {
            atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(wo).unwrap_or_default())))
}

pub async fn list_work_orders(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListWorkOrdersQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let wos = state.eam_engine.list_work_orders(
        org_id, query.status.as_deref(), query.work_order_type.as_deref(),
        query.priority.as_deref(), query.asset_id,
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({ "data": wos })))
}

pub async fn get_work_order(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let wo = state.eam_engine.get_work_order(id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match wo {
        Some(w) => Ok(Json(serde_json::to_value(w).unwrap_or_default())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn update_work_order_status(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let status = body["status"].as_str().unwrap_or("");
    let wo = state.eam_engine.update_work_order_status(id, status).await.map_err(|e| {
        match e {
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;
    Ok(Json(serde_json::to_value(wo).unwrap_or_default()))
}

pub async fn complete_work_order(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let actual_cost = body["actualCost"].as_str();
    let actual_hours = body.get("actualHours").cloned().filter(|v| !v.is_null());
    let downtime_hours = body["downtimeHours"].as_f64();
    let resolution_code = body["resolutionCode"].as_str();
    let completion_notes = body["completionNotes"].as_str();
    let materials = body.get("materials").cloned().filter(|v| !v.is_null());
    let labor = body.get("labor").cloned().filter(|v| !v.is_null());

    let wo = state.eam_engine.complete_work_order(
        id, actual_cost, actual_hours, downtime_hours,
        resolution_code, completion_notes, materials, labor,
    ).await.map_err(|e| {
        match e {
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;
    Ok(Json(serde_json::to_value(wo).unwrap_or_default()))
}

pub async fn delete_work_order(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(wo_number): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    state.eam_engine.delete_work_order(org_id, &wo_number).await.map_err(|e| {
        match e {
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// PM Schedule Handlers
// ============================================================================

pub async fn create_pm_schedule(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(body): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let schedule_number = body["scheduleNumber"].as_str().unwrap_or("");
    let name = body["name"].as_str().unwrap_or("");
    let description = body["description"].as_str();
    let asset_id: Uuid = serde_json::from_value(body["assetId"].clone())
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let schedule_type = body["scheduleType"].as_str().unwrap_or("time_based");
    let frequency = body["frequency"].as_str();
    let interval_value = body["intervalValue"].as_i64().map(|v| v as i32);
    let interval_unit = body["intervalUnit"].as_str();
    let meter_type = body["meterType"].as_str();
    let meter_threshold = body.get("meterThreshold").cloned().filter(|v| !v.is_null());
    let work_order_template = body.get("workOrderTemplate").cloned().filter(|v| !v.is_null());
    let estimated_duration_hours = body["estimatedDurationHours"].as_f64();
    let estimated_cost = body["estimatedCost"].as_str();
    let auto_generate = body["autoGenerate"].as_bool().unwrap_or(false);
    let lead_time_days = body["leadTimeDays"].as_i64().map(|v| v as i32);
    let effective_start = body["effectiveStart"].as_str()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let effective_end = body["effectiveEnd"].as_str()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());

    let sched = state.eam_engine.create_pm_schedule(
        org_id, schedule_number, name, description,
        asset_id, schedule_type, frequency, interval_value, interval_unit,
        meter_type, meter_threshold, work_order_template,
        estimated_duration_hours, estimated_cost,
        auto_generate, lead_time_days,
        effective_start, effective_end, Some(user_id),
    ).await.map_err(|e| {
        tracing::error!("Create PM schedule error: {}", e);
        match e {
            atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(sched).unwrap_or_default())))
}

pub async fn list_pm_schedules(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListPmSchedulesQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let schedules = state.eam_engine.list_pm_schedules(
        org_id, query.status.as_deref(), query.asset_id,
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({ "data": schedules })))
}

pub async fn get_pm_schedule(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let sched = state.eam_engine.get_pm_schedule(id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match sched {
        Some(s) => Ok(Json(serde_json::to_value(s).unwrap_or_default())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn update_pm_schedule_status(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let status = body["status"].as_str().unwrap_or("");
    let sched = state.eam_engine.update_pm_schedule_status(id, status).await.map_err(|e| {
        match e {
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;
    Ok(Json(serde_json::to_value(sched).unwrap_or_default()))
}

pub async fn delete_pm_schedule(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(schedule_number): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    state.eam_engine.delete_pm_schedule(org_id, &schedule_number).await.map_err(|e| {
        match e {
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Dashboard Handler
// ============================================================================

pub async fn get_maintenance_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let dashboard = state.eam_engine.get_dashboard(org_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::to_value(dashboard).unwrap_or_default()))
}
