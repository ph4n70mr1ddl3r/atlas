//! Service Request Management Handlers
//!
//! Oracle Fusion CX Service-inspired API endpoints for managing
//! service request categories, service requests, communications,
//! assignments, resolution, and SLA tracking.
//!
//! Oracle Fusion equivalent: CX Service > Service Requests

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

// ============================================================================
// Category Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateCategoryRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub parent_category_id: Option<Uuid>,
    pub default_priority: Option<String>,
    pub default_sla_hours: Option<i32>,
}

/// Create a service category
pub async fn create_category(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateCategoryRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let created_by = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let category = state.service_request_engine
        .create_category(
            org_id, &payload.code, &payload.name, payload.description.as_deref(),
            payload.parent_category_id, payload.default_priority.as_deref(),
            payload.default_sla_hours, Some(created_by),
        )
        .await
        .map_err(|e| {
            tracing::error!("Create category error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(category).unwrap_or_default())))
}

/// Get a service category by code
pub async fn get_category(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(code): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let category = state.service_request_engine
        .get_category(org_id, &code)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match category {
        Some(c) => Ok(Json(serde_json::to_value(c).unwrap_or_default())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// List service categories
pub async fn list_categories(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let categories = state.service_request_engine
        .list_categories(org_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({ "data": categories })))
}

/// Delete a service category
pub async fn delete_category(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(code): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    state.service_request_engine
        .delete_category(org_id, &code)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Service Request Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateServiceRequestPayload {
    pub request_number: String,
    pub title: String,
    pub description: Option<String>,
    pub category_id: Option<Uuid>,
    pub priority: String,
    pub request_type: String,
    pub channel: String,
    pub customer_id: Option<Uuid>,
    pub customer_name: Option<String>,
    pub contact_id: Option<Uuid>,
    pub contact_name: Option<String>,
    pub assigned_to: Option<Uuid>,
    pub assigned_to_name: Option<String>,
    pub assigned_group: Option<String>,
    pub product_id: Option<Uuid>,
    pub product_name: Option<String>,
    pub serial_number: Option<String>,
    pub parent_request_id: Option<Uuid>,
    pub related_object_type: Option<String>,
    pub related_object_id: Option<Uuid>,
}

/// Create a service request
pub async fn create_request(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateServiceRequestPayload>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let created_by = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let request = state.service_request_engine
        .create_request(
            org_id, &payload.request_number, &payload.title,
            payload.description.as_deref(), payload.category_id,
            &payload.priority, &payload.request_type, &payload.channel,
            payload.customer_id, payload.customer_name.as_deref(),
            payload.contact_id, payload.contact_name.as_deref(),
            payload.assigned_to, payload.assigned_to_name.as_deref(),
            payload.assigned_group.as_deref(),
            payload.product_id, payload.product_name.as_deref(),
            payload.serial_number.as_deref(),
            payload.parent_request_id,
            payload.related_object_type.as_deref(),
            payload.related_object_id,
            Some(created_by),
        )
        .await
        .map_err(|e| {
            tracing::error!("Create service request error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(request).unwrap_or_default())))
}

/// Get a service request by ID
pub async fn get_request(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let request = state.service_request_engine
        .get_request(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match request {
        Some(r) => Ok(Json(serde_json::to_value(r).unwrap_or_default())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// Get a service request by number
pub async fn get_request_by_number(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(number): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let request = state.service_request_engine
        .get_request_by_number(org_id, &number)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match request {
        Some(r) => Ok(Json(serde_json::to_value(r).unwrap_or_default())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

#[derive(Debug, Deserialize)]
pub struct ListRequestsParams {
    pub status: Option<String>,
    pub priority: Option<String>,
    pub customer_id: Option<Uuid>,
    pub assigned_to: Option<Uuid>,
    pub category_id: Option<Uuid>,
}

/// List service requests with optional filters
pub async fn list_requests(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<ListRequestsParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let requests = state.service_request_engine
        .list_requests(
            org_id,
            params.status.as_deref(),
            params.priority.as_deref(),
            params.customer_id,
            params.assigned_to,
            params.category_id,
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({ "data": requests })))
}

// ============================================================================
// Status Transitions
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct UpdateStatusPayload {
    pub status: String,
}

/// Update service request status
pub async fn update_request_status(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateStatusPayload>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let request = state.service_request_engine
        .update_status(id, &payload.status)
        .await
        .map_err(|e| {
            tracing::error!("Update status error: {}", e);
            match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(Json(serde_json::to_value(request).unwrap_or_default()))
}

#[derive(Debug, Deserialize)]
pub struct ResolveRequestPayload {
    pub resolution: String,
    pub resolution_code: String,
}

/// Resolve a service request
pub async fn resolve_request(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<ResolveRequestPayload>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let request = state.service_request_engine
        .resolve_request(id, &payload.resolution, &payload.resolution_code)
        .await
        .map_err(|e| {
            tracing::error!("Resolve request error: {}", e);
            match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(Json(serde_json::to_value(request).unwrap_or_default()))
}

// ============================================================================
// Assignment Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct AssignRequestPayload {
    pub assigned_to: Option<Uuid>,
    pub assigned_to_name: Option<String>,
    pub assigned_group: Option<String>,
    pub assignment_type: Option<String>,
}

/// Assign a service request
pub async fn assign_request(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<AssignRequestPayload>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let assigned_by = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let assignment = state.service_request_engine
        .assign_request(
            org_id, id,
            payload.assigned_to, payload.assigned_to_name.as_deref(),
            payload.assigned_group.as_deref(),
            payload.assignment_type.as_deref().unwrap_or("initial"),
            Some(assigned_by), Some(&claims.email),
        )
        .await
        .map_err(|e| {
            tracing::error!("Assign request error: {}", e);
            match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(Json(serde_json::to_value(assignment).unwrap_or_default()))
}

/// List assignments for a request
pub async fn list_assignments(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let assignments = state.service_request_engine
        .list_assignments(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({ "data": assignments })))
}

// ============================================================================
// Update / Communication Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct AddUpdatePayload {
    pub update_type: String,
    pub subject: Option<String>,
    pub body: String,
    pub is_internal: Option<bool>,
}

/// Add a communication/update to a service request
pub async fn add_update(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<AddUpdatePayload>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let author_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let update = state.service_request_engine
        .add_update(
            org_id, id, &payload.update_type,
            Some(author_id), Some(&claims.email),
            payload.subject.as_deref(), &payload.body,
            payload.is_internal.unwrap_or(false),
        )
        .await
        .map_err(|e| {
            tracing::error!("Add update error: {}", e);
            match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(update).unwrap_or_default())))
}

#[derive(Debug, Deserialize)]
pub struct ListUpdatesParams {
    pub include_internal: Option<bool>,
}

/// List updates for a service request
pub async fn list_updates(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Query(params): Query<ListUpdatesParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let updates = state.service_request_engine
        .list_updates(id, params.include_internal.unwrap_or(false))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({ "data": updates })))
}

// ============================================================================
// Dashboard
// ============================================================================

/// Get service request dashboard
pub async fn get_service_request_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let dashboard = state.service_request_engine
        .get_dashboard(org_id)
        .await
        .map_err(|e| {
            tracing::error!("Service request dashboard error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(serde_json::to_value(dashboard).unwrap_or_default()))
}
