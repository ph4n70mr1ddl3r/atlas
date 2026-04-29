//! Quality Management Handlers
//!
//! Oracle Fusion Cloud: Quality Management > Inspections > Non-Conformance > CAPA
//! Provides HTTP endpoints for:
//! - Inspection plan CRUD and lifecycle
//! - Plan criteria management
//! - Quality inspections with verdict workflow
//! - Inspection result recording
//! - Non-conformance reports (NCRs) with status workflow
//! - Corrective & preventive actions (CAPA) with verification
//! - Quality holds with release workflow
//! - Quality dashboard summary

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
// Inspection Plans
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePlanRequest {
    pub plan_code: String,
    pub name: String,
    pub description: Option<String>,
    pub plan_type: String,
    pub item_id: Option<Uuid>,
    pub item_code: Option<String>,
    pub supplier_id: Option<Uuid>,
    pub supplier_name: Option<String>,
    pub inspection_trigger: String,
    pub sampling_method: String,
    pub sample_size_percent: Option<String>,
    pub accept_number: Option<i32>,
    pub reject_number: Option<i32>,
    pub frequency: String,
    pub effective_from: Option<String>,
    pub effective_to: Option<String>,
}

pub async fn create_plan(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreatePlanRequest>,
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

    let plan = state
        .quality_engine
        .create_plan(
            org_id,
            &payload.plan_code,
            &payload.name,
            payload.description.as_deref(),
            &payload.plan_type,
            payload.item_id,
            payload.item_code.as_deref(),
            payload.supplier_id,
            payload.supplier_name.as_deref(),
            &payload.inspection_trigger,
            &payload.sampling_method,
            payload.sample_size_percent.as_deref(),
            payload.accept_number,
            payload.reject_number,
            &payload.frequency,
            effective_from,
            effective_to,
            Some(user_id),
        )
        .await
        .map_err(|e| {
            tracing::error!("Create plan error: {}", e);
            match e {
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(plan).unwrap_or_default()),
    ))
}

pub async fn get_plan(
    State(state): State<Arc<AppState>>,
    Path(code): Path<String>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let plan = state
        .quality_engine
        .get_plan(org_id, &code)
        .await
        .map_err(|e| {
            tracing::error!("Get plan error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    match plan {
        Some(p) => Ok(Json(serde_json::to_value(p).unwrap_or_default())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

#[derive(Debug, Deserialize)]
pub struct ListPlansParams {
    pub active_only: Option<bool>,
}

pub async fn list_plans(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<ListPlansParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let plans = state
        .quality_engine
        .list_plans(org_id, params.active_only.unwrap_or(false))
        .await
        .map_err(|e| {
            tracing::error!("List plans error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(serde_json::json!({
        "data": plans,
        "meta": { "total": plans.len() }
    })))
}

pub async fn delete_plan(
    State(state): State<Arc<AppState>>,
    Path(code): Path<String>,
    claims: Extension<Claims>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    state
        .quality_engine
        .delete_plan(org_id, &code)
        .await
        .map_err(|e| {
            tracing::error!("Delete plan error: {}", e);
            match e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Plan Criteria
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCriterionRequest {
    pub criterion_number: i32,
    pub name: String,
    pub description: Option<String>,
    pub characteristic: String,
    pub measurement_type: String,
    pub target_value: Option<String>,
    pub lower_spec_limit: Option<String>,
    pub upper_spec_limit: Option<String>,
    pub unit_of_measure: Option<String>,
    pub is_mandatory: Option<bool>,
    pub weight: String,
    pub criticality: String,
}

pub async fn create_criterion(
    State(state): State<Arc<AppState>>,
    Path(plan_id): Path<Uuid>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateCriterionRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let criterion = state
        .quality_engine
        .create_criterion(
            org_id,
            plan_id,
            payload.criterion_number,
            &payload.name,
            payload.description.as_deref(),
            &payload.characteristic,
            &payload.measurement_type,
            payload.target_value.as_deref(),
            payload.lower_spec_limit.as_deref(),
            payload.upper_spec_limit.as_deref(),
            payload.unit_of_measure.as_deref(),
            payload.is_mandatory.unwrap_or(true),
            &payload.weight,
            &payload.criticality,
        )
        .await
        .map_err(|e| {
            tracing::error!("Create criterion error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(criterion).unwrap_or_default()),
    ))
}

pub async fn list_criteria(
    State(state): State<Arc<AppState>>,
    Path(plan_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let criteria = state
        .quality_engine
        .list_criteria(plan_id)
        .await
        .map_err(|e| {
            tracing::error!("List criteria error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(serde_json::json!({
        "data": criteria,
        "meta": { "total": criteria.len() }
    })))
}

pub async fn delete_criterion(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    state
        .quality_engine
        .delete_criterion(id)
        .await
        .map_err(|e| {
            tracing::error!("Delete criterion error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Inspections
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateInspectionRequest {
    pub plan_id: Uuid,
    pub source_type: String,
    pub source_id: Option<Uuid>,
    pub source_number: Option<String>,
    pub item_id: Option<Uuid>,
    pub item_code: Option<String>,
    pub item_description: Option<String>,
    pub lot_number: Option<String>,
    pub quantity_inspected: String,
    pub quantity_accepted: String,
    pub quantity_rejected: String,
    pub unit_of_measure: Option<String>,
    pub inspector_id: Option<Uuid>,
    pub inspector_name: Option<String>,
    pub inspection_date: String,
}

pub async fn create_inspection(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateInspectionRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let inspection_date = chrono::NaiveDate::parse_from_str(&payload.inspection_date, "%Y-%m-%d")
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let inspection = state
        .quality_engine
        .create_inspection(
            org_id,
            payload.plan_id,
            &payload.source_type,
            payload.source_id,
            payload.source_number.as_deref(),
            payload.item_id,
            payload.item_code.as_deref(),
            payload.item_description.as_deref(),
            payload.lot_number.as_deref(),
            &payload.quantity_inspected,
            &payload.quantity_accepted,
            &payload.quantity_rejected,
            payload.unit_of_measure.as_deref(),
            payload.inspector_id,
            payload.inspector_name.as_deref(),
            inspection_date,
            Some(user_id),
        )
        .await
        .map_err(|e| {
            tracing::error!("Create inspection error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(inspection).unwrap_or_default()),
    ))
}

pub async fn get_inspection(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let inspection = state
        .quality_engine
        .get_inspection(id)
        .await
        .map_err(|e| {
            tracing::error!("Get inspection error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    match inspection {
        Some(i) => Ok(Json(serde_json::to_value(i).unwrap_or_default())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

#[derive(Debug, Deserialize)]
pub struct ListInspectionsParams {
    pub status: Option<String>,
    pub plan_id: Option<Uuid>,
}

pub async fn list_inspections(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<ListInspectionsParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let inspections = state
        .quality_engine
        .list_inspections(org_id, params.status.as_deref(), params.plan_id)
        .await
        .map_err(|e| {
            tracing::error!("List inspections error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(serde_json::json!({
        "data": inspections,
        "meta": { "total": inspections.len() }
    })))
}

pub async fn start_inspection(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let inspection = state
        .quality_engine
        .start_inspection(id)
        .await
        .map_err(|e| {
            tracing::error!("Start inspection error: {}", e);
            match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::CONFLICT,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(Json(serde_json::to_value(inspection).unwrap_or_default()))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompleteInspectionRequest {
    pub verdict: String,
    pub notes: Option<String>,
}

pub async fn complete_inspection(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<CompleteInspectionRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let inspection = state
        .quality_engine
        .complete_inspection(id, &payload.verdict, payload.notes.as_deref())
        .await
        .map_err(|e| {
            tracing::error!("Complete inspection error: {}", e);
            match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::CONFLICT,
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(Json(serde_json::to_value(inspection).unwrap_or_default()))
}

pub async fn cancel_inspection(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let inspection = state
        .quality_engine
        .cancel_inspection(id)
        .await
        .map_err(|e| {
            tracing::error!("Cancel inspection error: {}", e);
            match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::CONFLICT,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(Json(serde_json::to_value(inspection).unwrap_or_default()))
}

// ============================================================================
// Inspection Results
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateResultRequest {
    pub criterion_id: Option<Uuid>,
    pub criterion_name: String,
    pub characteristic: String,
    pub measurement_type: String,
    pub observed_value: Option<String>,
    pub target_value: Option<String>,
    pub lower_spec_limit: Option<String>,
    pub upper_spec_limit: Option<String>,
    pub unit_of_measure: Option<String>,
    pub result_status: String,
    pub deviation: Option<String>,
    pub notes: Option<String>,
    pub evaluated_by: Option<Uuid>,
}

pub async fn create_result(
    State(state): State<Arc<AppState>>,
    Path(inspection_id): Path<Uuid>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateResultRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let result = state
        .quality_engine
        .create_result(
            org_id,
            inspection_id,
            payload.criterion_id,
            &payload.criterion_name,
            &payload.characteristic,
            &payload.measurement_type,
            payload.observed_value.as_deref(),
            payload.target_value.as_deref(),
            payload.lower_spec_limit.as_deref(),
            payload.upper_spec_limit.as_deref(),
            payload.unit_of_measure.as_deref(),
            &payload.result_status,
            payload.deviation.as_deref(),
            payload.notes.as_deref(),
            payload.evaluated_by,
        )
        .await
        .map_err(|e| {
            tracing::error!("Create result error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::CONFLICT,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(result).unwrap_or_default()),
    ))
}

pub async fn list_results(
    State(state): State<Arc<AppState>>,
    Path(inspection_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let results = state
        .quality_engine
        .list_results(inspection_id)
        .await
        .map_err(|e| {
            tracing::error!("List results error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(serde_json::json!({
        "data": results,
        "meta": { "total": results.len() }
    })))
}

// ============================================================================
// Non-Conformance Reports
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateNcrRequest {
    pub title: String,
    pub description: Option<String>,
    pub ncr_type: String,
    pub severity: String,
    pub origin: String,
    pub source_type: Option<String>,
    pub source_id: Option<Uuid>,
    pub source_number: Option<String>,
    pub item_id: Option<Uuid>,
    pub item_code: Option<String>,
    pub supplier_id: Option<Uuid>,
    pub supplier_name: Option<String>,
    pub detected_date: String,
    pub detected_by: Option<String>,
    pub responsible_party: Option<String>,
}

pub async fn create_ncr(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateNcrRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let detected_date = chrono::NaiveDate::parse_from_str(&payload.detected_date, "%Y-%m-%d")
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let ncr = state
        .quality_engine
        .create_ncr(
            org_id,
            &payload.title,
            payload.description.as_deref(),
            &payload.ncr_type,
            &payload.severity,
            &payload.origin,
            payload.source_type.as_deref(),
            payload.source_id,
            payload.source_number.as_deref(),
            payload.item_id,
            payload.item_code.as_deref(),
            payload.supplier_id,
            payload.supplier_name.as_deref(),
            detected_date,
            payload.detected_by.as_deref(),
            payload.responsible_party.as_deref(),
            Some(user_id),
        )
        .await
        .map_err(|e| {
            tracing::error!("Create NCR error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(ncr).unwrap_or_default()),
    ))
}

pub async fn get_ncr(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let ncr = state
        .quality_engine
        .get_ncr(id)
        .await
        .map_err(|e| {
            tracing::error!("Get NCR error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    match ncr {
        Some(n) => Ok(Json(serde_json::to_value(n).unwrap_or_default())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

#[derive(Debug, Deserialize)]
pub struct ListNcrsParams {
    pub status: Option<String>,
    pub severity: Option<String>,
}

pub async fn list_ncrs(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<ListNcrsParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let ncrs = state
        .quality_engine
        .list_ncrs(org_id, params.status.as_deref(), params.severity.as_deref())
        .await
        .map_err(|e| {
            tracing::error!("List NCRs error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(Json(serde_json::json!({
        "data": ncrs,
        "meta": { "total": ncrs.len() }
    })))
}

pub async fn investigate_ncr(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let ncr = state
        .quality_engine
        .investigate_ncr(id)
        .await
        .map_err(|e| {
            tracing::error!("Investigate NCR error: {}", e);
            match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::CONFLICT,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(Json(serde_json::to_value(ncr).unwrap_or_default()))
}

pub async fn start_ncr_corrective_action(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let ncr = state
        .quality_engine
        .start_corrective_action_phase(id)
        .await
        .map_err(|e| {
            tracing::error!("Start NCR corrective action error: {}", e);
            match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::CONFLICT,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(Json(serde_json::to_value(ncr).unwrap_or_default()))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolveNcrRequest {
    pub resolution_description: String,
    pub resolution_type: String,
    pub resolved_by: Option<String>,
}

pub async fn resolve_ncr(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<ResolveNcrRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let ncr = state
        .quality_engine
        .resolve_ncr(
            id,
            &payload.resolution_description,
            &payload.resolution_type,
            payload.resolved_by.as_deref(),
        )
        .await
        .map_err(|e| {
            tracing::error!("Resolve NCR error: {}", e);
            match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::CONFLICT,
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(Json(serde_json::to_value(ncr).unwrap_or_default()))
}

pub async fn close_ncr(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let ncr = state
        .quality_engine
        .close_ncr(id)
        .await
        .map_err(|e| {
            tracing::error!("Close NCR error: {}", e);
            match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::CONFLICT,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(Json(serde_json::to_value(ncr).unwrap_or_default()))
}

// ============================================================================
// Corrective & Preventive Actions
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCorrectiveActionRequest {
    pub action_type: String,
    pub title: String,
    pub description: Option<String>,
    pub root_cause: Option<String>,
    pub corrective_action_desc: Option<String>,
    pub preventive_action_desc: Option<String>,
    pub assigned_to: Option<String>,
    pub due_date: Option<String>,
    pub priority: String,
}

pub async fn create_corrective_action(
    State(state): State<Arc<AppState>>,
    Path(ncr_id): Path<Uuid>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateCorrectiveActionRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let due_date = payload
        .due_date
        .as_deref()
        .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());

    let action = state
        .quality_engine
        .create_corrective_action(
            org_id,
            ncr_id,
            &payload.action_type,
            &payload.title,
            payload.description.as_deref(),
            payload.root_cause.as_deref(),
            payload.corrective_action_desc.as_deref(),
            payload.preventive_action_desc.as_deref(),
            payload.assigned_to.as_deref(),
            due_date,
            &payload.priority,
            Some(user_id),
        )
        .await
        .map_err(|e| {
            tracing::error!("Create corrective action error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::CONFLICT,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(action).unwrap_or_default()),
    ))
}

pub async fn get_corrective_action(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let action = state
        .quality_engine
        .get_corrective_action(id)
        .await
        .map_err(|e| {
            tracing::error!("Get corrective action error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    match action {
        Some(a) => Ok(Json(serde_json::to_value(a).unwrap_or_default())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn list_corrective_actions(
    State(state): State<Arc<AppState>>,
    Path(ncr_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let actions = state
        .quality_engine
        .list_corrective_actions(ncr_id)
        .await
        .map_err(|e| {
            tracing::error!("List corrective actions error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(serde_json::json!({
        "data": actions,
        "meta": { "total": actions.len() }
    })))
}

pub async fn start_corrective_action(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let action = state
        .quality_engine
        .start_corrective_action(id)
        .await
        .map_err(|e| {
            tracing::error!("Start corrective action error: {}", e);
            match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::CONFLICT,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(Json(serde_json::to_value(action).unwrap_or_default()))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompleteCorrectiveActionRequest {
    pub effectiveness_rating: Option<i32>,
}

pub async fn complete_corrective_action(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<CompleteCorrectiveActionRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let action = state
        .quality_engine
        .complete_corrective_action(id, payload.effectiveness_rating)
        .await
        .map_err(|e| {
            tracing::error!("Complete corrective action error: {}", e);
            match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::CONFLICT,
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(Json(serde_json::to_value(action).unwrap_or_default()))
}

pub async fn verify_corrective_action(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let action = state
        .quality_engine
        .verify_corrective_action(id)
        .await
        .map_err(|e| {
            tracing::error!("Verify corrective action error: {}", e);
            match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::CONFLICT,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(Json(serde_json::to_value(action).unwrap_or_default()))
}

// ============================================================================
// Quality Holds
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateHoldRequest {
    pub reason: String,
    pub description: Option<String>,
    pub item_id: Option<Uuid>,
    pub item_code: Option<String>,
    pub lot_number: Option<String>,
    pub supplier_id: Option<Uuid>,
    pub supplier_name: Option<String>,
    pub source_type: Option<String>,
    pub source_id: Option<Uuid>,
    pub source_number: Option<String>,
    pub hold_type: String,
}

pub async fn create_hold(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateHoldRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let hold = state
        .quality_engine
        .create_hold(
            org_id,
            &payload.reason,
            payload.description.as_deref(),
            payload.item_id,
            payload.item_code.as_deref(),
            payload.lot_number.as_deref(),
            payload.supplier_id,
            payload.supplier_name.as_deref(),
            payload.source_type.as_deref(),
            payload.source_id,
            payload.source_number.as_deref(),
            &payload.hold_type,
            Some(user_id),
        )
        .await
        .map_err(|e| {
            tracing::error!("Create hold error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(hold).unwrap_or_default()),
    ))
}

pub async fn get_hold(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let hold = state
        .quality_engine
        .get_hold(id)
        .await
        .map_err(|e| {
            tracing::error!("Get hold error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    match hold {
        Some(h) => Ok(Json(serde_json::to_value(h).unwrap_or_default())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

#[derive(Debug, Deserialize)]
pub struct ListHoldsParams {
    pub status: Option<String>,
    pub item_id: Option<Uuid>,
}

pub async fn list_holds(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<ListHoldsParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let holds = state
        .quality_engine
        .list_holds(org_id, params.status.as_deref(), params.item_id)
        .await
        .map_err(|e| {
            tracing::error!("List holds error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(serde_json::json!({
        "data": holds,
        "meta": { "total": holds.len() }
    })))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseHoldRequest {
    pub release_notes: Option<String>,
}

pub async fn release_hold(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    claims: Extension<Claims>,
    Json(payload): Json<ReleaseHoldRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let hold = state
        .quality_engine
        .release_hold(id, Some(user_id), payload.release_notes.as_deref())
        .await
        .map_err(|e| {
            tracing::error!("Release hold error: {}", e);
            match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::CONFLICT,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(Json(serde_json::to_value(hold).unwrap_or_default()))
}

// ============================================================================
// Dashboard
// ============================================================================

pub async fn get_quality_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let dashboard = state
        .quality_engine
        .get_dashboard_summary(org_id)
        .await
        .map_err(|e| {
            tracing::error!("Quality dashboard error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(serde_json::to_value(dashboard).unwrap_or_default()))
}
