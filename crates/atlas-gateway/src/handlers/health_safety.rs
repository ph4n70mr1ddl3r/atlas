//! Workplace Health & Safety (EHS) Handlers
//!
//! Oracle Fusion Cloud: Environment, Health, and Safety
//! Provides HTTP endpoints for:
//! - Safety incident tracking (injuries, near-misses, property damage)
//! - Hazard identification and risk assessment
//! - Safety inspections and audits
//! - Corrective and Preventive Actions (CAPA)
//! - OSHA compliance reporting
//! - Health & Safety dashboard

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
// Incidents
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateIncidentRequest {
    pub incident_number: String,
    pub title: String,
    pub description: Option<String>,
    pub incident_type: String,
    pub severity: String,
    pub priority: String,
    pub incident_date: chrono::NaiveDate,
    pub incident_time: Option<String>,
    pub location: Option<String>,
    pub facility_id: Option<Uuid>,
    pub department_id: Option<Uuid>,
    pub reported_by_name: Option<String>,
    pub assigned_to_id: Option<Uuid>,
    pub assigned_to_name: Option<String>,
    pub osha_recordable: Option<bool>,
    pub osha_classification: Option<String>,
    pub body_part: Option<String>,
    pub injury_source: Option<String>,
    pub event_type: Option<String>,
    pub environment_factor: Option<String>,
}

pub async fn create_incident(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateIncidentRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let incident = state.health_safety_engine
        .create_incident(
            org_id,
            &payload.incident_number,
            &payload.title,
            payload.description.as_deref(),
            &payload.incident_type,
            &payload.severity,
            &payload.priority,
            payload.incident_date,
            payload.incident_time.as_deref(),
            payload.location.as_deref(),
            payload.facility_id,
            payload.department_id,
            Some(user_id),
            None,
            payload.assigned_to_id,
            payload.assigned_to_name.as_deref(),
            payload.osha_recordable.unwrap_or(false),
            payload.osha_classification.as_deref(),
            payload.body_part.as_deref(),
            payload.injury_source.as_deref(),
            payload.event_type.as_deref(),
            payload.environment_factor.as_deref(),
            Some(user_id),
        )
        .await
        .map_err(|e| {
            tracing::error!("Create incident error: {}", e);
            match e {
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(incident).unwrap())))
}

#[derive(Debug, Deserialize)]
pub struct ListIncidentsQuery {
    pub status: Option<String>,
    pub severity: Option<String>,
    pub incident_type: Option<String>,
    pub facility_id: Option<Uuid>,
}

pub async fn list_incidents(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListIncidentsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let incidents = state.health_safety_engine
        .list_incidents(
            org_id,
            query.status.as_deref(),
            query.severity.as_deref(),
            query.incident_type.as_deref(),
            query.facility_id.as_ref(),
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({ "data": incidents })))
}

pub async fn get_incident(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let incident = state.health_safety_engine
        .get_incident(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match incident {
        Some(inc) => Ok(Json(serde_json::to_value(inc).unwrap())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateStatusRequest {
    pub status: String,
}

pub async fn update_incident_status(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateStatusRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let incident = state.health_safety_engine
        .update_incident_status(id, &payload.status)
        .await
        .map_err(|e| {
            tracing::error!("Update incident status error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(Json(serde_json::to_value(incident).unwrap()))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInvestigationRequest {
    pub root_cause: Option<String>,
    pub immediate_action: Option<String>,
    pub assigned_to_id: Option<Uuid>,
    pub assigned_to_name: Option<String>,
    pub days_away_from_work: Option<i32>,
    pub days_restricted: Option<i32>,
}

pub async fn update_incident_investigation(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateInvestigationRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let incident = state.health_safety_engine
        .update_incident_investigation(
            id,
            payload.root_cause.as_deref(),
            payload.immediate_action.as_deref(),
            payload.assigned_to_id,
            payload.assigned_to_name.as_deref(),
            payload.days_away_from_work,
            payload.days_restricted,
        )
        .await
        .map_err(|e| {
            tracing::error!("Update investigation error: {}", e);
            match e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(Json(serde_json::to_value(incident).unwrap()))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CloseIncidentRequest {
    pub closed_by: Option<Uuid>,
}

pub async fn close_incident(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(_payload): Json<CloseIncidentRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let incident = state.health_safety_engine
        .close_incident(id, Some(user_id))
        .await
        .map_err(|e| {
            tracing::error!("Close incident error: {}", e);
            match e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(Json(serde_json::to_value(incident).unwrap()))
}

pub async fn delete_incident(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(incident_number): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    state.health_safety_engine
        .delete_incident(org_id, &incident_number)
        .await
        .map_err(|e| {
            tracing::error!("Delete incident error: {}", e);
            match e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Hazards
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateHazardRequest {
    pub hazard_code: String,
    pub title: String,
    pub description: Option<String>,
    pub hazard_category: String,
    pub likelihood: String,
    pub consequence: String,
    pub location: Option<String>,
    pub facility_id: Option<Uuid>,
    pub department_id: Option<Uuid>,
    pub identified_by_name: Option<String>,
    pub identified_date: chrono::NaiveDate,
    pub mitigation_measures: Option<serde_json::Value>,
    pub review_date: Option<chrono::NaiveDate>,
    pub owner_id: Option<Uuid>,
    pub owner_name: Option<String>,
}

pub async fn create_hazard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateHazardRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let hazard = state.health_safety_engine
        .create_hazard(
            org_id,
            &payload.hazard_code,
            &payload.title,
            payload.description.as_deref(),
            &payload.hazard_category,
            &payload.likelihood,
            &payload.consequence,
            payload.location.as_deref(),
            payload.facility_id,
            payload.department_id,
            Some(user_id),
            payload.identified_by_name.as_deref(),
            payload.identified_date,
            payload.mitigation_measures,
            payload.review_date,
            payload.owner_id,
            payload.owner_name.as_deref(),
            Some(user_id),
        )
        .await
        .map_err(|e| {
            tracing::error!("Create hazard error: {}", e);
            match e {
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(hazard).unwrap())))
}

#[derive(Debug, Deserialize)]
pub struct ListHazardsQuery {
    pub status: Option<String>,
    pub risk_level: Option<String>,
    pub hazard_category: Option<String>,
    pub facility_id: Option<Uuid>,
}

pub async fn list_hazards(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListHazardsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let hazards = state.health_safety_engine
        .list_hazards(
            org_id,
            query.status.as_deref(),
            query.risk_level.as_deref(),
            query.hazard_category.as_deref(),
            query.facility_id.as_ref(),
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({ "data": hazards })))
}

pub async fn get_hazard(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let hazard = state.health_safety_engine
        .get_hazard(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match hazard {
        Some(h) => Ok(Json(serde_json::to_value(h).unwrap())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn update_hazard_status(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateStatusRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let hazard = state.health_safety_engine
        .update_hazard_status(id, &payload.status)
        .await
        .map_err(|e| {
            tracing::error!("Update hazard status error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(Json(serde_json::to_value(hazard).unwrap()))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssessResidualRiskRequest {
    pub residual_likelihood: String,
    pub residual_consequence: String,
}

pub async fn assess_hazard_residual_risk(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<AssessResidualRiskRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let hazard = state.health_safety_engine
        .assess_residual_risk(id, &payload.residual_likelihood, &payload.residual_consequence)
        .await
        .map_err(|e| {
            tracing::error!("Assess residual risk error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(Json(serde_json::to_value(hazard).unwrap()))
}

pub async fn delete_hazard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(hazard_code): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    state.health_safety_engine
        .delete_hazard(org_id, &hazard_code)
        .await
        .map_err(|e| {
            tracing::error!("Delete hazard error: {}", e);
            match e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Inspections
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateInspectionRequest {
    pub inspection_number: String,
    pub title: String,
    pub description: Option<String>,
    pub inspection_type: String,
    pub priority: String,
    pub scheduled_date: chrono::NaiveDate,
    pub location: Option<String>,
    pub facility_id: Option<Uuid>,
    pub department_id: Option<Uuid>,
    pub inspector_name: Option<String>,
}

pub async fn create_inspection(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateInspectionRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let inspection = state.health_safety_engine
        .create_inspection(
            org_id,
            &payload.inspection_number,
            &payload.title,
            payload.description.as_deref(),
            &payload.inspection_type,
            &payload.priority,
            payload.scheduled_date,
            payload.location.as_deref(),
            payload.facility_id,
            payload.department_id,
            Some(user_id),
            payload.inspector_name.as_deref(),
            Some(user_id),
        )
        .await
        .map_err(|e| {
            tracing::error!("Create inspection error: {}", e);
            match e {
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(inspection).unwrap())))
}

#[derive(Debug, Deserialize)]
pub struct ListInspectionsQuery {
    pub status: Option<String>,
    pub inspection_type: Option<String>,
    pub facility_id: Option<Uuid>,
}

pub async fn list_inspections(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListInspectionsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let inspections = state.health_safety_engine
        .list_inspections(
            org_id,
            query.status.as_deref(),
            query.inspection_type.as_deref(),
            query.facility_id.as_ref(),
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({ "data": inspections })))
}

pub async fn get_inspection(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let inspection = state.health_safety_engine
        .get_inspection(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match inspection {
        Some(ins) => Ok(Json(serde_json::to_value(ins).unwrap())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompleteInspectionRequest {
    pub findings_summary: Option<String>,
    pub findings: serde_json::Value,
    pub critical_findings: i32,
    pub non_conformities: i32,
    pub observations: i32,
    pub score: Option<f64>,
    pub max_score: Option<f64>,
}

pub async fn complete_inspection(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<CompleteInspectionRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let inspection = state.health_safety_engine
        .complete_inspection(
            id,
            payload.findings_summary.as_deref(),
            payload.findings,
            payload.critical_findings,
            payload.non_conformities,
            payload.observations,
            payload.score,
            payload.max_score,
        )
        .await
        .map_err(|e| {
            tracing::error!("Complete inspection error: {}", e);
            match e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(Json(serde_json::to_value(inspection).unwrap()))
}

pub async fn update_inspection_status(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateStatusRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let inspection = state.health_safety_engine
        .update_inspection_status(id, &payload.status)
        .await
        .map_err(|e| {
            tracing::error!("Update inspection status error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(Json(serde_json::to_value(inspection).unwrap()))
}

pub async fn delete_inspection(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(inspection_number): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    state.health_safety_engine
        .delete_inspection(org_id, &inspection_number)
        .await
        .map_err(|e| {
            tracing::error!("Delete inspection error: {}", e);
            match e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Corrective Actions (CAPA)
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCorrectiveActionRequest {
    pub action_number: String,
    pub title: String,
    pub description: Option<String>,
    pub action_type: String,
    pub priority: String,
    pub source_type: Option<String>,
    pub source_id: Option<Uuid>,
    pub source_number: Option<String>,
    pub root_cause: Option<String>,
    pub corrective_action_plan: Option<String>,
    pub preventive_action_plan: Option<String>,
    pub assigned_to_id: Option<Uuid>,
    pub assigned_to_name: Option<String>,
    pub due_date: Option<chrono::NaiveDate>,
    pub facility_id: Option<Uuid>,
    pub department_id: Option<Uuid>,
    pub estimated_cost: Option<f64>,
    pub currency_code: Option<String>,
}

pub async fn create_corrective_action(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateCorrectiveActionRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let action = state.health_safety_engine
        .create_corrective_action(
            org_id,
            &payload.action_number,
            &payload.title,
            payload.description.as_deref(),
            &payload.action_type,
            &payload.priority,
            payload.source_type.as_deref(),
            payload.source_id,
            payload.source_number.as_deref(),
            payload.root_cause.as_deref(),
            payload.corrective_action_plan.as_deref(),
            payload.preventive_action_plan.as_deref(),
            payload.assigned_to_id,
            payload.assigned_to_name.as_deref(),
            payload.due_date,
            payload.facility_id,
            payload.department_id,
            payload.estimated_cost,
            payload.currency_code.as_deref(),
            Some(user_id),
        )
        .await
        .map_err(|e| {
            tracing::error!("Create corrective action error: {}", e);
            match e {
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(action).unwrap())))
}

#[derive(Debug, Deserialize)]
pub struct ListCorrectiveActionsQuery {
    pub status: Option<String>,
    pub action_type: Option<String>,
    pub source_type: Option<String>,
}

pub async fn list_corrective_actions(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListCorrectiveActionsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let actions = state.health_safety_engine
        .list_corrective_actions(
            org_id,
            query.status.as_deref(),
            query.action_type.as_deref(),
            query.source_type.as_deref(),
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({ "data": actions })))
}

pub async fn get_corrective_action(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let action = state.health_safety_engine
        .get_corrective_action(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match action {
        Some(a) => Ok(Json(serde_json::to_value(a).unwrap())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn update_corrective_action_status(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateStatusRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let action = state.health_safety_engine
        .update_corrective_action_status(id, &payload.status)
        .await
        .map_err(|e| {
            tracing::error!("Update CAPA status error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(Json(serde_json::to_value(action).unwrap()))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompleteCorrectiveActionRequest {
    pub effectiveness: String,
    pub actual_cost: Option<f64>,
}

pub async fn complete_corrective_action(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<CompleteCorrectiveActionRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let action = state.health_safety_engine
        .complete_corrective_action(id, &payload.effectiveness, payload.actual_cost, Some(user_id))
        .await
        .map_err(|e| {
            tracing::error!("Complete CAPA error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(Json(serde_json::to_value(action).unwrap()))
}

pub async fn delete_corrective_action(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(action_number): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    state.health_safety_engine
        .delete_corrective_action(org_id, &action_number)
        .await
        .map_err(|e| {
            tracing::error!("Delete CAPA error: {}", e);
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

pub async fn get_health_safety_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let dashboard = state.health_safety_engine
        .get_dashboard(org_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::to_value(dashboard).unwrap()))
}
