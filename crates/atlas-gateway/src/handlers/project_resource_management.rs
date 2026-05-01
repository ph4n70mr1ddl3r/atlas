//! Project Resource Management Handlers
//!
//! Oracle Fusion Cloud: Project Management > Resource Management
//! Provides HTTP endpoints for:
//! - Resource profile CRUD and availability management
//! - Resource request lifecycle (draft -> submitted -> fulfilled -> cancelled)
//! - Resource assignment management with planned/actual hours
//! - Utilization entry recording and approval workflow
//! - Resource management dashboard

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
// Profiles
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProfileRequest {
    pub resource_number: String,
    pub name: String,
    pub email: Option<String>,
    pub resource_type: Option<String>,
    pub department: Option<String>,
    pub job_title: Option<String>,
    pub skills: Option<String>,
    pub certifications: Option<String>,
    pub availability_status: Option<String>,
    pub available_hours_per_week: Option<f64>,
    pub cost_rate: Option<f64>,
    pub cost_rate_currency: Option<String>,
    pub bill_rate: Option<f64>,
    pub bill_rate_currency: Option<String>,
    pub location: Option<String>,
    pub manager_id: Option<Uuid>,
    pub manager_name: Option<String>,
    pub hire_date: Option<chrono::NaiveDate>,
    pub notes: Option<String>,
}

pub async fn create_profile(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateProfileRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let profile = state.project_resource_engine
        .create_profile(
            org_id,
            &payload.resource_number,
            &payload.name,
            payload.email.as_deref().unwrap_or(""),
            payload.resource_type.as_deref().unwrap_or("employee"),
            payload.department.as_deref().unwrap_or(""),
            payload.job_title.as_deref().unwrap_or(""),
            payload.skills.as_deref(),
            payload.certifications.as_deref(),
            payload.availability_status.as_deref(),
            payload.available_hours_per_week,
            payload.cost_rate,
            payload.cost_rate_currency.as_deref(),
            payload.bill_rate,
            payload.bill_rate_currency.as_deref(),
            payload.location.as_deref(),
            payload.manager_id,
            payload.manager_name.as_deref(),
            payload.hire_date,
            payload.notes.as_deref(),
            Some(user_id),
        )
        .await
        .map_err(|e| {
            tracing::error!("Create resource profile error: {}", e);
            match e {
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(profile).unwrap())))
}

pub async fn get_profile(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let profile = state.project_resource_engine
        .get_profile(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match profile {
        Some(p) => Ok(Json(serde_json::to_value(p).unwrap())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

#[derive(Debug, Deserialize)]
pub struct ListProfilesQuery {
    pub availability_status: Option<String>,
    pub resource_type: Option<String>,
    pub department: Option<String>,
}

pub async fn list_profiles(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListProfilesQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let profiles = state.project_resource_engine
        .list_profiles(
            org_id,
            query.availability_status.as_deref(),
            query.resource_type.as_deref(),
            query.department.as_deref(),
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({ "data": profiles })))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAvailabilityRequest {
    pub availability_status: String,
}

pub async fn update_availability(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateAvailabilityRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let profile = state.project_resource_engine
        .update_availability(id, &payload.availability_status)
        .await
        .map_err(|e| {
            tracing::error!("Update availability error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;
    Ok(Json(serde_json::to_value(profile).unwrap()))
}

pub async fn delete_profile(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(number): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    state.project_resource_engine
        .delete_profile(org_id, &number)
        .await
        .map_err(|e| {
            match e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Requests
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateResourceRequestRequest {
    pub request_number: String,
    pub project_id: Option<Uuid>,
    pub project_name: Option<String>,
    pub project_number: Option<String>,
    pub requested_role: String,
    pub required_skills: Option<String>,
    pub priority: String,
    pub start_date: chrono::NaiveDate,
    pub end_date: chrono::NaiveDate,
    pub hours_per_week: Option<f64>,
    pub total_planned_hours: Option<f64>,
    pub max_cost_rate: Option<f64>,
    pub currency_code: Option<String>,
    pub resource_type_preference: Option<String>,
    pub location_requirement: Option<String>,
    pub notes: Option<String>,
}

pub async fn create_request(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateResourceRequestRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let request = state.project_resource_engine
        .create_request(
            org_id,
            &payload.request_number,
            payload.project_id,
            payload.project_name.as_deref(),
            payload.project_number.as_deref(),
            &payload.requested_role,
            payload.required_skills.as_deref(),
            &payload.priority,
            payload.start_date,
            payload.end_date,
            payload.hours_per_week.unwrap_or(40.0),
            payload.total_planned_hours.unwrap_or(0.0),
            payload.max_cost_rate,
            payload.currency_code.as_deref(),
            payload.resource_type_preference.as_deref(),
            payload.location_requirement.as_deref(),
            payload.notes.as_deref(),
            Some(user_id),
        )
        .await
        .map_err(|e| {
            tracing::error!("Create resource request error: {}", e);
            match e {
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(request).unwrap())))
}

pub async fn get_request(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let request = state.project_resource_engine
        .get_request(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match request {
        Some(r) => Ok(Json(serde_json::to_value(r).unwrap())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

#[derive(Debug, Deserialize)]
pub struct ListRequestsQuery {
    pub status: Option<String>,
    pub priority: Option<String>,
    pub project_id: Option<Uuid>,
}

pub async fn list_requests(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListRequestsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let requests = state.project_resource_engine
        .list_requests(
            org_id,
            query.status.as_deref(),
            query.priority.as_deref(),
            query.project_id,
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({ "data": requests })))
}

pub async fn submit_request(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let request = state.project_resource_engine
        .submit_request(id)
        .await
        .map_err(|e| {
            tracing::error!("Submit request error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(Json(serde_json::to_value(request).unwrap()))
}

pub async fn fulfill_request(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let request = state.project_resource_engine
        .fulfill_request(id, user_id)
        .await
        .map_err(|e| {
            tracing::error!("Fulfill request error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(Json(serde_json::to_value(request).unwrap()))
}

pub async fn cancel_request(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let request = state.project_resource_engine
        .cancel_request(id)
        .await
        .map_err(|e| {
            tracing::error!("Cancel request error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(Json(serde_json::to_value(request).unwrap()))
}

pub async fn delete_request(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(number): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    state.project_resource_engine
        .delete_request(org_id, &number)
        .await
        .map_err(|e| match e {
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        })?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Assignments
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAssignmentRequest {
    pub assignment_number: String,
    pub resource_id: Uuid,
    pub resource_name: Option<String>,
    pub resource_email: Option<String>,
    pub project_id: Option<Uuid>,
    pub project_name: Option<String>,
    pub project_number: Option<String>,
    pub request_id: Option<Uuid>,
    pub role: String,
    pub start_date: chrono::NaiveDate,
    pub end_date: chrono::NaiveDate,
    pub planned_hours: f64,
    pub cost_rate: Option<f64>,
    pub bill_rate: Option<f64>,
    pub currency_code: Option<String>,
    pub notes: Option<String>,
}

pub async fn create_assignment(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateAssignmentRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let assignment = state.project_resource_engine
        .create_assignment(
            org_id,
            &payload.assignment_number,
            payload.resource_id,
            payload.resource_name.as_deref(),
            payload.resource_email.as_deref(),
            payload.project_id,
            payload.project_name.as_deref(),
            payload.project_number.as_deref(),
            payload.request_id,
            &payload.role,
            payload.start_date,
            payload.end_date,
            payload.planned_hours,
            payload.cost_rate,
            payload.bill_rate,
            payload.currency_code.as_deref(),
            payload.notes.as_deref(),
            Some(user_id),
        )
        .await
        .map_err(|e| {
            tracing::error!("Create assignment error: {}", e);
            match e {
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(assignment).unwrap())))
}

pub async fn get_assignment(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let assignment = state.project_resource_engine
        .get_assignment(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match assignment {
        Some(a) => Ok(Json(serde_json::to_value(a).unwrap())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

#[derive(Debug, Deserialize)]
pub struct ListAssignmentsQuery {
    pub status: Option<String>,
    pub resource_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
}

pub async fn list_assignments(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListAssignmentsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let assignments = state.project_resource_engine
        .list_assignments(
            org_id,
            query.status.as_deref(),
            query.resource_id,
            query.project_id,
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({ "data": assignments })))
}

pub async fn activate_assignment(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let assignment = state.project_resource_engine
        .activate_assignment(id)
        .await
        .map_err(|e| {
            tracing::error!("Activate assignment error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(Json(serde_json::to_value(assignment).unwrap()))
}

pub async fn complete_assignment(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let assignment = state.project_resource_engine
        .complete_assignment(id)
        .await
        .map_err(|e| {
            tracing::error!("Complete assignment error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(Json(serde_json::to_value(assignment).unwrap()))
}

pub async fn cancel_assignment(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let assignment = state.project_resource_engine
        .cancel_assignment(id)
        .await
        .map_err(|e| {
            tracing::error!("Cancel assignment error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(Json(serde_json::to_value(assignment).unwrap()))
}

pub async fn delete_assignment(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(number): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    state.project_resource_engine
        .delete_assignment(org_id, &number)
        .await
        .map_err(|e| match e {
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        })?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Utilization Entries
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateUtilizationRequest {
    pub assignment_id: Uuid,
    pub resource_id: Uuid,
    pub entry_date: chrono::NaiveDate,
    pub hours_worked: f64,
    pub description: Option<String>,
    pub billable: Option<bool>,
    pub notes: Option<String>,
}

pub async fn create_utilization_entry(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateUtilizationRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let entry = state.project_resource_engine
        .create_utilization_entry(
            org_id,
            payload.assignment_id,
            payload.resource_id,
            payload.entry_date,
            payload.hours_worked,
            payload.description.as_deref(),
            payload.billable,
            payload.notes.as_deref(),
            Some(user_id),
        )
        .await
        .map_err(|e| {
            tracing::error!("Create utilization entry error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(entry).unwrap())))
}

#[derive(Debug, Deserialize)]
pub struct ListUtilizationQuery {
    pub assignment_id: Option<Uuid>,
    pub resource_id: Option<Uuid>,
    pub status: Option<String>,
}

pub async fn list_utilization_entries(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListUtilizationQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let entries = state.project_resource_engine
        .list_utilization_entries(
            org_id,
            query.assignment_id,
            query.resource_id,
            query.status.as_deref(),
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({ "data": entries })))
}

pub async fn approve_utilization_entry(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let entry = state.project_resource_engine
        .approve_utilization_entry(id, user_id)
        .await
        .map_err(|e| match e {
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        })?;
    Ok(Json(serde_json::to_value(entry).unwrap()))
}

pub async fn reject_utilization_entry(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let entry = state.project_resource_engine
        .reject_utilization_entry(id)
        .await
        .map_err(|e| match e {
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        })?;
    Ok(Json(serde_json::to_value(entry).unwrap()))
}

pub async fn delete_utilization_entry(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    state.project_resource_engine
        .delete_utilization_entry(id)
        .await
        .map_err(|e| match e {
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        })?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Dashboard
// ============================================================================

pub async fn get_resource_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let dashboard = state.project_resource_engine
        .get_dashboard(org_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::to_value(dashboard).unwrap()))
}
