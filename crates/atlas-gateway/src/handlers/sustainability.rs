//! Sustainability & ESG Management Handlers
//!
//! Oracle Fusion Cloud: Sustainability / Environmental Accounting and Reporting
//! Provides HTTP endpoints for:
//! - Facility tracking for environmental footprint
//! - GHG emission factors (Scope 1, 2, 3)
//! - Environmental activity / emissions logging
//! - ESG metric definitions and readings
//! - Sustainability goals with progress tracking
//! - Carbon offset management and retirement
//! - Sustainability dashboard

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension, Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::AppState;
use crate::handlers::auth::Claims;

// ============================================================================
// Facilities
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateFacilityRequest {
    pub facility_code: String,
    pub name: String,
    pub description: Option<String>,
    pub country_code: Option<String>,
    pub region: Option<String>,
    pub city: Option<String>,
    pub address: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub facility_type: String,
    pub industry_sector: Option<String>,
    pub total_area_sqm: Option<f64>,
    pub employee_count: Option<i32>,
    pub operating_hours_per_year: Option<i32>,
}

pub async fn create_facility(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateFacilityRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let facility = state.sustainability_engine
        .create_facility(
            org_id,
            &payload.facility_code,
            &payload.name,
            payload.description.as_deref(),
            payload.country_code.as_deref(),
            payload.region.as_deref(),
            payload.city.as_deref(),
            payload.address.as_deref(),
            payload.latitude,
            payload.longitude,
            &payload.facility_type,
            payload.industry_sector.as_deref(),
            payload.total_area_sqm,
            payload.employee_count,
            payload.operating_hours_per_year,
            Some(user_id),
        )
        .await
        .map_err(|e| {
            tracing::error!("Create facility error: {}", e);
            match e {
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(facility).unwrap_or_default()),
    ))
}

pub async fn get_facility(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let facility = state
        .sustainability_engine
        .get_facility(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match facility {
        Some(f) => Ok(Json(serde_json::to_value(f).unwrap_or_default())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

#[derive(Debug, Deserialize)]
pub struct ListFacilitiesParams {
    pub status: Option<String>,
    pub facility_type: Option<String>,
}

pub async fn list_facilities(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<ListFacilitiesParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let facilities = state
        .sustainability_engine
        .list_facilities(org_id, params.status.as_deref(), params.facility_type.as_deref())
        .await
        .map_err(|e| {
            tracing::error!("List facilities error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(serde_json::json!({
        "data": facilities,
        "meta": { "total": facilities.len() }
    })))
}

pub async fn update_facility_status(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    claims: Extension<Claims>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let _claims = claims;
    let status = payload["status"]
        .as_str()
        .ok_or(StatusCode::BAD_REQUEST)?;

    let facility = state
        .sustainability_engine
        .update_facility_status(id, status)
        .await
        .map_err(|e| {
            tracing::error!("Update facility status error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(Json(serde_json::to_value(facility).unwrap_or_default()))
}

pub async fn delete_facility(
    State(state): State<Arc<AppState>>,
    Path(facility_code): Path<String>,
    claims: Extension<Claims>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    state
        .sustainability_engine
        .delete_facility(org_id, &facility_code)
        .await
        .map_err(|e| {
            tracing::error!("Delete facility error: {}", e);
            match e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Emission Factors
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateEmissionFactorRequest {
    pub factor_code: String,
    pub name: String,
    pub description: Option<String>,
    pub scope: String,
    pub category: String,
    pub activity_type: String,
    pub factor_value: f64,
    pub unit_of_measure: String,
    pub gas_type: String,
    pub factor_source: Option<String>,
    pub effective_from: String,
    pub effective_to: Option<String>,
    pub region_code: Option<String>,
}

pub async fn create_emission_factor(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateEmissionFactorRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let effective_from = chrono::NaiveDate::parse_from_str(&payload.effective_from, "%Y-%m-%d")
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let effective_to = payload
        .effective_to
        .as_deref()
        .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());

    let ef = state
        .sustainability_engine
        .create_emission_factor(
            org_id,
            &payload.factor_code,
            &payload.name,
            payload.description.as_deref(),
            &payload.scope,
            &payload.category,
            &payload.activity_type,
            payload.factor_value,
            &payload.unit_of_measure,
            &payload.gas_type,
            payload.factor_source.as_deref(),
            effective_from,
            effective_to,
            payload.region_code.as_deref(),
            Some(user_id),
        )
        .await
        .map_err(|e| {
            tracing::error!("Create emission factor error: {}", e);
            match e {
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(ef).unwrap_or_default()),
    ))
}

pub async fn get_emission_factor(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let ef = state
        .sustainability_engine
        .get_emission_factor(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match ef {
        Some(f) => Ok(Json(serde_json::to_value(f).unwrap_or_default())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

#[derive(Debug, Deserialize)]
pub struct ListEmissionFactorsParams {
    pub scope: Option<String>,
    pub category: Option<String>,
    pub activity_type: Option<String>,
}

pub async fn list_emission_factors(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<ListEmissionFactorsParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let factors = state
        .sustainability_engine
        .list_emission_factors(
            org_id,
            params.scope.as_deref(),
            params.category.as_deref(),
            params.activity_type.as_deref(),
        )
        .await
        .map_err(|e| {
            tracing::error!("List emission factors error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(serde_json::json!({
        "data": factors,
        "meta": { "total": factors.len() }
    })))
}

pub async fn delete_emission_factor(
    State(state): State<Arc<AppState>>,
    Path(factor_code): Path<String>,
    claims: Extension<Claims>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    state
        .sustainability_engine
        .delete_emission_factor(org_id, &factor_code)
        .await
        .map_err(|e| {
            tracing::error!("Delete emission factor error: {}", e);
            match e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Environmental Activities
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateActivityRequest {
    pub activity_number: String,
    pub facility_id: Option<Uuid>,
    pub facility_code: Option<String>,
    pub activity_type: String,
    pub scope: String,
    pub category: Option<String>,
    pub quantity: f64,
    pub unit_of_measure: String,
    pub emission_factor_id: Option<Uuid>,
    pub co2e_kg: f64,
    pub co2_kg: Option<f64>,
    pub ch4_kg: Option<f64>,
    pub n2o_kg: Option<f64>,
    pub cost_amount: Option<f64>,
    pub cost_currency: Option<String>,
    pub activity_date: String,
    pub reporting_period: Option<String>,
    pub source_type: Option<String>,
    pub source_reference: Option<String>,
    pub department_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
}

pub async fn create_activity(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateActivityRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let activity_date = chrono::NaiveDate::parse_from_str(&payload.activity_date, "%Y-%m-%d")
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let activity = state
        .sustainability_engine
        .create_activity(
            org_id,
            &payload.activity_number,
            payload.facility_id,
            payload.facility_code.as_deref(),
            &payload.activity_type,
            &payload.scope,
            payload.category.as_deref(),
            payload.quantity,
            &payload.unit_of_measure,
            payload.emission_factor_id,
            payload.co2e_kg,
            payload.co2_kg,
            payload.ch4_kg,
            payload.n2o_kg,
            payload.cost_amount,
            payload.cost_currency.as_deref(),
            activity_date,
            payload.reporting_period.as_deref(),
            payload.source_type.as_deref(),
            payload.source_reference.as_deref(),
            payload.department_id,
            payload.project_id,
            Some(user_id),
        )
        .await
        .map_err(|e| {
            tracing::error!("Create activity error: {}", e);
            match e {
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(activity).unwrap_or_default()),
    ))
}

pub async fn get_activity(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let activity = state
        .sustainability_engine
        .get_activity(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match activity {
        Some(a) => Ok(Json(serde_json::to_value(a).unwrap_or_default())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

#[derive(Debug, Deserialize)]
pub struct ListActivitiesParams {
    pub scope: Option<String>,
    pub facility_id: Option<Uuid>,
    pub activity_type: Option<String>,
    pub reporting_period: Option<String>,
}

pub async fn list_activities(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<ListActivitiesParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let activities = state
        .sustainability_engine
        .list_activities(
            org_id,
            params.scope.as_deref(),
            params.facility_id.as_ref(),
            params.activity_type.as_deref(),
            params.reporting_period.as_deref(),
        )
        .await
        .map_err(|e| {
            tracing::error!("List activities error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(serde_json::json!({
        "data": activities,
        "meta": { "total": activities.len() }
    })))
}

pub async fn update_activity_status(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    claims: Extension<Claims>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let _claims = claims;
    let status = payload["status"]
        .as_str()
        .ok_or(StatusCode::BAD_REQUEST)?;

    let activity = state
        .sustainability_engine
        .update_activity_status(id, status)
        .await
        .map_err(|e| {
            tracing::error!("Update activity status error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(Json(serde_json::to_value(activity).unwrap_or_default()))
}

pub async fn delete_activity(
    State(state): State<Arc<AppState>>,
    Path(activity_number): Path<String>,
    claims: Extension<Claims>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    state
        .sustainability_engine
        .delete_activity(org_id, &activity_number)
        .await
        .map_err(|e| {
            tracing::error!("Delete activity error: {}", e);
            match e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// ESG Metrics
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMetricRequest {
    pub metric_code: String,
    pub name: String,
    pub description: Option<String>,
    pub pillar: String,
    pub category: String,
    pub unit_of_measure: String,
    pub gri_standard: Option<String>,
    pub sasb_standard: Option<String>,
    pub tcfd_category: Option<String>,
    pub eu_taxonomy_code: Option<String>,
    pub target_value: Option<f64>,
    pub warning_threshold: Option<f64>,
    pub direction: String,
}

pub async fn create_metric(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateMetricRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let metric = state
        .sustainability_engine
        .create_metric(
            org_id,
            &payload.metric_code,
            &payload.name,
            payload.description.as_deref(),
            &payload.pillar,
            &payload.category,
            &payload.unit_of_measure,
            payload.gri_standard.as_deref(),
            payload.sasb_standard.as_deref(),
            payload.tcfd_category.as_deref(),
            payload.eu_taxonomy_code.as_deref(),
            payload.target_value,
            payload.warning_threshold,
            &payload.direction,
            Some(user_id),
        )
        .await
        .map_err(|e| {
            tracing::error!("Create metric error: {}", e);
            match e {
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(metric).unwrap_or_default()),
    ))
}

pub async fn get_metric(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let metric = state
        .sustainability_engine
        .get_metric(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match metric {
        Some(m) => Ok(Json(serde_json::to_value(m).unwrap_or_default())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

#[derive(Debug, Deserialize)]
pub struct ListMetricsParams {
    pub pillar: Option<String>,
    pub category: Option<String>,
}

pub async fn list_metrics(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<ListMetricsParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let metrics = state
        .sustainability_engine
        .list_metrics(org_id, params.pillar.as_deref(), params.category.as_deref())
        .await
        .map_err(|e| {
            tracing::error!("List metrics error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(serde_json::json!({
        "data": metrics,
        "meta": { "total": metrics.len() }
    })))
}

pub async fn delete_metric(
    State(state): State<Arc<AppState>>,
    Path(metric_code): Path<String>,
    claims: Extension<Claims>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    state
        .sustainability_engine
        .delete_metric(org_id, &metric_code)
        .await
        .map_err(|e| {
            tracing::error!("Delete metric error: {}", e);
            match e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// ESG Metric Readings
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMetricReadingRequest {
    pub metric_id: Uuid,
    pub metric_value: f64,
    pub reading_date: String,
    pub reporting_period: Option<String>,
    pub facility_id: Option<Uuid>,
    pub notes: Option<String>,
    pub source: Option<String>,
}

pub async fn create_metric_reading(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateMetricReadingRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let reading_date = chrono::NaiveDate::parse_from_str(&payload.reading_date, "%Y-%m-%d")
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let reading = state
        .sustainability_engine
        .create_metric_reading(
            org_id,
            payload.metric_id,
            payload.metric_value,
            reading_date,
            payload.reporting_period.as_deref(),
            payload.facility_id,
            payload.notes.as_deref(),
            payload.source.as_deref(),
            Some(user_id),
        )
        .await
        .map_err(|e| {
            tracing::error!("Create metric reading error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(reading).unwrap_or_default()),
    ))
}

pub async fn list_metric_readings(
    State(state): State<Arc<AppState>>,
    Path(metric_id): Path<Uuid>,
    Query(params): Query<ListMetricReadingsParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let from_date = params
        .from_date
        .as_deref()
        .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());
    let to_date = params
        .to_date
        .as_deref()
        .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());

    let readings = state
        .sustainability_engine
        .list_metric_readings(metric_id, from_date, to_date)
        .await
        .map_err(|e| {
            tracing::error!("List metric readings error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(serde_json::json!({
        "data": readings,
        "meta": { "total": readings.len() }
    })))
}

#[derive(Debug, Deserialize)]
pub struct ListMetricReadingsParams {
    pub from_date: Option<String>,
    pub to_date: Option<String>,
}

pub async fn delete_metric_reading(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    state
        .sustainability_engine
        .delete_metric_reading(id)
        .await
        .map_err(|e| {
            tracing::error!("Delete metric reading error: {}", e);
            match e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Sustainability Goals
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateGoalRequest {
    pub goal_code: String,
    pub name: String,
    pub description: Option<String>,
    pub goal_type: String,
    pub scope: Option<String>,
    pub baseline_value: f64,
    pub baseline_year: i32,
    pub baseline_unit: String,
    pub target_value: f64,
    pub target_year: i32,
    pub target_unit: String,
    pub target_reduction_pct: Option<f64>,
    pub milestones: Option<serde_json::Value>,
    pub facility_id: Option<Uuid>,
    pub owner_id: Option<Uuid>,
    pub owner_name: Option<String>,
    pub framework: Option<String>,
    pub framework_reference: Option<String>,
    pub effective_from: Option<String>,
    pub effective_to: Option<String>,
}

pub async fn create_goal(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateGoalRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let effective_from = payload
        .effective_from
        .as_deref()
        .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());
    let effective_to = payload
        .effective_to
        .as_deref()
        .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());

    let goal = state
        .sustainability_engine
        .create_goal(
            org_id,
            &payload.goal_code,
            &payload.name,
            payload.description.as_deref(),
            &payload.goal_type,
            payload.scope.as_deref(),
            payload.baseline_value,
            payload.baseline_year,
            &payload.baseline_unit,
            payload.target_value,
            payload.target_year,
            &payload.target_unit,
            payload.target_reduction_pct,
            payload.milestones,
            payload.facility_id,
            payload.owner_id,
            payload.owner_name.as_deref(),
            payload.framework.as_deref(),
            payload.framework_reference.as_deref(),
            effective_from,
            effective_to,
            Some(user_id),
        )
        .await
        .map_err(|e| {
            tracing::error!("Create goal error: {}", e);
            match e {
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(goal).unwrap_or_default()),
    ))
}

pub async fn get_goal(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let goal = state
        .sustainability_engine
        .get_goal(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match goal {
        Some(g) => Ok(Json(serde_json::to_value(g).unwrap_or_default())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

#[derive(Debug, Deserialize)]
pub struct ListGoalsParams {
    pub goal_type: Option<String>,
    pub status: Option<String>,
}

pub async fn list_goals(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<ListGoalsParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let goals = state
        .sustainability_engine
        .list_goals(org_id, params.goal_type.as_deref(), params.status.as_deref())
        .await
        .map_err(|e| {
            tracing::error!("List goals error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(serde_json::json!({
        "data": goals,
        "meta": { "total": goals.len() }
    })))
}

pub async fn update_goal_progress(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    claims: Extension<Claims>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let _claims = claims;
    let current_value = payload["currentValue"]
        .as_f64()
        .ok_or(StatusCode::BAD_REQUEST)?;

    let goal = state
        .sustainability_engine
        .update_goal_progress(id, current_value)
        .await
        .map_err(|e| {
            tracing::error!("Update goal progress error: {}", e);
            match e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(Json(serde_json::to_value(goal).unwrap_or_default()))
}

pub async fn update_goal_status(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    claims: Extension<Claims>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let _claims = claims;
    let status = payload["status"]
        .as_str()
        .ok_or(StatusCode::BAD_REQUEST)?;

    let goal = state
        .sustainability_engine
        .update_goal_status(id, status)
        .await
        .map_err(|e| {
            tracing::error!("Update goal status error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(Json(serde_json::to_value(goal).unwrap_or_default()))
}

pub async fn delete_goal(
    State(state): State<Arc<AppState>>,
    Path(goal_code): Path<String>,
    claims: Extension<Claims>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    state
        .sustainability_engine
        .delete_goal(org_id, &goal_code)
        .await
        .map_err(|e| {
            tracing::error!("Delete goal error: {}", e);
            match e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Carbon Offsets
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCarbonOffsetRequest {
    pub offset_number: String,
    pub name: String,
    pub description: Option<String>,
    pub project_name: String,
    pub project_type: String,
    pub project_location: Option<String>,
    pub registry: Option<String>,
    pub registry_id: Option<String>,
    pub certification_standard: Option<String>,
    pub quantity_tonnes: f64,
    pub unit_price: Option<f64>,
    pub total_cost: Option<f64>,
    pub currency_code: Option<String>,
    pub vintage_year: i32,
    pub effective_from: String,
    pub effective_to: Option<String>,
    pub supplier_name: Option<String>,
    pub supplier_id: Option<Uuid>,
    pub notes: Option<String>,
}

pub async fn create_carbon_offset(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateCarbonOffsetRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let effective_from = chrono::NaiveDate::parse_from_str(&payload.effective_from, "%Y-%m-%d")
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let effective_to = payload
        .effective_to
        .as_deref()
        .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());

    let offset = state
        .sustainability_engine
        .create_carbon_offset(
            org_id,
            &payload.offset_number,
            &payload.name,
            payload.description.as_deref(),
            &payload.project_name,
            &payload.project_type,
            payload.project_location.as_deref(),
            payload.registry.as_deref(),
            payload.registry_id.as_deref(),
            payload.certification_standard.as_deref(),
            payload.quantity_tonnes,
            payload.unit_price,
            payload.total_cost,
            payload.currency_code.as_deref(),
            payload.vintage_year,
            effective_from,
            effective_to,
            payload.supplier_name.as_deref(),
            payload.supplier_id,
            payload.notes.as_deref(),
            Some(user_id),
        )
        .await
        .map_err(|e| {
            tracing::error!("Create carbon offset error: {}", e);
            match e {
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(offset).unwrap_or_default()),
    ))
}

pub async fn get_carbon_offset(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let offset = state
        .sustainability_engine
        .get_carbon_offset(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match offset {
        Some(o) => Ok(Json(serde_json::to_value(o).unwrap_or_default())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

#[derive(Debug, Deserialize)]
pub struct ListCarbonOffsetsParams {
    pub status: Option<String>,
    pub project_type: Option<String>,
}

pub async fn list_carbon_offsets(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<ListCarbonOffsetsParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let offsets = state
        .sustainability_engine
        .list_carbon_offsets(org_id, params.status.as_deref(), params.project_type.as_deref())
        .await
        .map_err(|e| {
            tracing::error!("List carbon offsets error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(serde_json::json!({
        "data": offsets,
        "meta": { "total": offsets.len() }
    })))
}

pub async fn retire_carbon_offset(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    claims: Extension<Claims>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let _claims = claims;
    let retire_quantity = payload["retireQuantity"]
        .as_f64()
        .ok_or(StatusCode::BAD_REQUEST)?;

    let offset = state
        .sustainability_engine
        .retire_carbon_offset(id, retire_quantity)
        .await
        .map_err(|e| {
            tracing::error!("Retire carbon offset error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(Json(serde_json::to_value(offset).unwrap_or_default()))
}

pub async fn delete_carbon_offset(
    State(state): State<Arc<AppState>>,
    Path(offset_number): Path<String>,
    claims: Extension<Claims>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    state
        .sustainability_engine
        .delete_carbon_offset(org_id, &offset_number)
        .await
        .map_err(|e| {
            tracing::error!("Delete carbon offset error: {}", e);
            match e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Dashboard
// ============================================================================

pub async fn get_sustainability_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let dashboard = state
        .sustainability_engine
        .get_dashboard(org_id)
        .await
        .map_err(|e| {
            tracing::error!("Sustainability dashboard error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(serde_json::to_value(dashboard).unwrap_or_default()))
}
