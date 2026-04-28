//! Risk Management & Internal Controls Handlers
//!
//! Oracle Fusion Cloud GRC-inspired endpoints:
//! - Risk category CRUD
//! - Risk register CRUD & assessment
//! - Control registry CRUD & effectiveness
//! - Risk-control mappings
//! - Control testing & certification
//! - Issue/remediation tracking
//! - Risk dashboard summary

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
// Risk Categories
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCategoryRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub parent_category_id: Option<Uuid>,
    pub sort_order: Option<i32>,
}

pub async fn create_category(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateCategoryRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let cat = state.risk_management_engine.create_category(
        org_id, &payload.code, &payload.name, payload.description.as_deref(),
        payload.parent_category_id, payload.sort_order, Some(user_id),
    ).await.map_err(|e| {
        tracing::error!("Create category error: {}", e);
        match e {
            atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(cat).unwrap_or_default())))
}

pub async fn list_categories(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let cats = state.risk_management_engine.list_categories(org_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({ "data": cats })))
}

pub async fn get_category(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let cat = state.risk_management_engine.get_category(id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match cat {
        Some(c) => Ok(Json(serde_json::to_value(c).unwrap_or_default())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn delete_category(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(code): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    state.risk_management_engine.delete_category(org_id, &code).await.map_err(|e| {
        match e {
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Risk Register
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRiskRequest {
    pub risk_number: String,
    pub title: String,
    pub description: Option<String>,
    pub category_id: Option<Uuid>,
    pub risk_source: Option<String>,
    pub likelihood: Option<i32>,
    pub impact: Option<i32>,
    pub owner_id: Option<Uuid>,
    pub owner_name: Option<String>,
    pub response_strategy: Option<String>,
    pub business_units: Option<serde_json::Value>,
    pub related_entity_type: Option<String>,
    pub related_entity_id: Option<Uuid>,
}

pub async fn create_risk(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateRiskRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let risk = state.risk_management_engine.create_risk(
        org_id, &payload.risk_number, &payload.title, payload.description.as_deref(),
        payload.category_id, payload.risk_source.as_deref().unwrap_or("operational"),
        payload.likelihood.unwrap_or(3), payload.impact.unwrap_or(3),
        payload.owner_id, payload.owner_name.as_deref(),
        payload.response_strategy.as_deref(), payload.business_units,
        payload.related_entity_type.as_deref(), payload.related_entity_id,
        Some(user_id),
    ).await.map_err(|e| {
        tracing::error!("Create risk error: {}", e);
        match e {
            atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(risk).unwrap_or_default())))
}

pub async fn get_risk(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let risk = state.risk_management_engine.get_risk(id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match risk {
        Some(r) => Ok(Json(serde_json::to_value(r).unwrap_or_default())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

#[derive(Debug, Deserialize)]
pub struct ListRisksParams {
    pub status: Option<String>,
    pub risk_level: Option<String>,
    pub risk_source: Option<String>,
}

pub async fn list_risks(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<ListRisksParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let risks = state.risk_management_engine.list_risks(
        org_id, params.status.as_deref(), params.risk_level.as_deref(), params.risk_source.as_deref(),
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({ "data": risks })))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRiskStatusRequest {
    pub status: String,
}

pub async fn update_risk_status(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateRiskStatusRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let risk = state.risk_management_engine.update_risk_status(id, &payload.status).await.map_err(|e| {
        match e {
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;
    Ok(Json(serde_json::to_value(risk).unwrap_or_default()))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssessRiskRequest {
    pub likelihood: i32,
    pub impact: i32,
    pub residual_likelihood: Option<i32>,
    pub residual_impact: Option<i32>,
}

pub async fn assess_risk(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<AssessRiskRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let risk = state.risk_management_engine.assess_risk(
        id, payload.likelihood, payload.impact,
        payload.residual_likelihood, payload.residual_impact,
    ).await.map_err(|e| {
        match e {
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;
    Ok(Json(serde_json::to_value(risk).unwrap_or_default()))
}

pub async fn delete_risk(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(risk_number): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    state.risk_management_engine.delete_risk(org_id, &risk_number).await.map_err(|e| {
        match e {
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Control Registry
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateControlRequest {
    pub control_number: String,
    pub title: String,
    pub description: Option<String>,
    pub control_type: Option<String>,
    pub control_nature: Option<String>,
    pub frequency: Option<String>,
    pub objective: Option<String>,
    pub test_procedures: Option<String>,
    pub owner_id: Option<Uuid>,
    pub owner_name: Option<String>,
    pub is_key_control: Option<bool>,
    pub business_processes: Option<serde_json::Value>,
    pub regulatory_frameworks: Option<serde_json::Value>,
}

pub async fn create_control(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateControlRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let ctrl = state.risk_management_engine.create_control(
        org_id, &payload.control_number, &payload.title, payload.description.as_deref(),
        payload.control_type.as_deref().unwrap_or("preventive"),
        payload.control_nature.as_deref().unwrap_or("manual"),
        payload.frequency.as_deref().unwrap_or("monthly"),
        payload.objective.as_deref(), payload.test_procedures.as_deref(),
        payload.owner_id, payload.owner_name.as_deref(),
        payload.is_key_control.unwrap_or(false),
        payload.business_processes, payload.regulatory_frameworks,
        Some(user_id),
    ).await.map_err(|e| {
        tracing::error!("Create control error: {}", e);
        match e {
            atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(ctrl).unwrap_or_default())))
}

pub async fn get_control(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let ctrl = state.risk_management_engine.get_control(id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match ctrl {
        Some(c) => Ok(Json(serde_json::to_value(c).unwrap_or_default())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

#[derive(Debug, Deserialize)]
pub struct ListControlsParams {
    pub status: Option<String>,
    pub control_type: Option<String>,
}

pub async fn list_controls(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<ListControlsParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let ctrls = state.risk_management_engine.list_controls(
        org_id, params.status.as_deref(), params.control_type.as_deref(),
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({ "data": ctrls })))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateControlStatusRequest {
    pub status: String,
}

pub async fn update_control_status(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateControlStatusRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let ctrl = state.risk_management_engine.update_control_status(id, &payload.status).await.map_err(|e| {
        match e {
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;
    Ok(Json(serde_json::to_value(ctrl).unwrap_or_default()))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateEffectivenessRequest {
    pub effectiveness: String,
}

pub async fn update_control_effectiveness(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateEffectivenessRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let ctrl = state.risk_management_engine.update_control_effectiveness(id, &payload.effectiveness).await.map_err(|e| {
        match e {
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;
    Ok(Json(serde_json::to_value(ctrl).unwrap_or_default()))
}

pub async fn delete_control(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(control_number): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    state.risk_management_engine.delete_control(org_id, &control_number).await.map_err(|e| {
        match e {
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Risk-Control Mappings
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMappingRequest {
    pub risk_id: Uuid,
    pub control_id: Uuid,
    pub mitigation_effectiveness: Option<String>,
    pub description: Option<String>,
}

pub async fn create_mapping(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateMappingRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mapping = state.risk_management_engine.create_risk_control_mapping(
        org_id, payload.risk_id, payload.control_id,
        payload.mitigation_effectiveness.as_deref().unwrap_or("partial"),
        payload.description.as_deref(), Some(user_id),
    ).await.map_err(|e| {
        tracing::error!("Create mapping error: {}", e);
        match e {
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(mapping).unwrap_or_default())))
}

pub async fn list_risk_mappings(
    State(state): State<Arc<AppState>>,
    Path(risk_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mappings = state.risk_management_engine.list_risk_mappings(risk_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({ "data": mappings })))
}

pub async fn list_control_mappings(
    State(state): State<Arc<AppState>>,
    Path(control_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mappings = state.risk_management_engine.list_control_mappings(control_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({ "data": mappings })))
}

pub async fn delete_mapping(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    state.risk_management_engine.delete_mapping(id).await.map_err(|e| {
        match e {
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Control Tests
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateControlTestRequest {
    pub control_id: Uuid,
    pub test_number: String,
    pub test_plan: String,
    pub test_period_start: chrono::NaiveDate,
    pub test_period_end: chrono::NaiveDate,
    pub tester_id: Option<Uuid>,
    pub tester_name: Option<String>,
}

pub async fn create_control_test(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateControlTestRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let test = state.risk_management_engine.create_control_test(
        org_id, payload.control_id, &payload.test_number, &payload.test_plan,
        payload.test_period_start, payload.test_period_end,
        payload.tester_id, payload.tester_name.as_deref(), Some(user_id),
    ).await.map_err(|e| {
        tracing::error!("Create control test error: {}", e);
        match e {
            atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(test).unwrap_or_default())))
}

pub async fn get_control_test(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let test = state.risk_management_engine.get_control_test(id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match test {
        Some(t) => Ok(Json(serde_json::to_value(t).unwrap_or_default())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn list_control_tests(
    State(state): State<Arc<AppState>>,
    Path(control_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let tests = state.risk_management_engine.list_control_tests(control_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({ "data": tests })))
}

pub async fn start_control_test(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let test = state.risk_management_engine.start_control_test(id).await.map_err(|e| {
        match e {
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;
    Ok(Json(serde_json::to_value(test).unwrap_or_default()))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompleteControlTestRequest {
    pub result: String,
    pub findings: Option<String>,
    pub deficiency_severity: Option<String>,
    pub sample_size: Option<i32>,
    pub sample_exceptions: Option<i32>,
}

pub async fn complete_control_test(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<CompleteControlTestRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let test = state.risk_management_engine.complete_control_test(
        id, &payload.result, payload.findings.as_deref(),
        payload.deficiency_severity.as_deref(), payload.sample_size, payload.sample_exceptions,
    ).await.map_err(|e| {
        match e {
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;
    Ok(Json(serde_json::to_value(test).unwrap_or_default()))
}

pub async fn delete_control_test(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(test_number): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    state.risk_management_engine.delete_control_test(org_id, &test_number).await.map_err(|e| {
        match e {
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Issues & Remediations
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateIssueRequest {
    pub issue_number: String,
    pub title: String,
    pub description: String,
    pub source: Option<String>,
    pub risk_id: Option<Uuid>,
    pub control_id: Option<Uuid>,
    pub control_test_id: Option<Uuid>,
    pub severity: Option<String>,
    pub priority: Option<String>,
    pub owner_id: Option<Uuid>,
    pub owner_name: Option<String>,
    pub remediation_plan: Option<String>,
    pub remediation_due_date: Option<chrono::NaiveDate>,
}

pub async fn create_issue(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateIssueRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let issue = state.risk_management_engine.create_issue(
        org_id, &payload.issue_number, &payload.title, &payload.description,
        payload.source.as_deref().unwrap_or("self_identified"),
        payload.risk_id, payload.control_id, payload.control_test_id,
        payload.severity.as_deref().unwrap_or("medium"),
        payload.priority.as_deref().unwrap_or("normal"),
        payload.owner_id, payload.owner_name.as_deref(),
        payload.remediation_plan.as_deref(), payload.remediation_due_date,
        Some(user_id),
    ).await.map_err(|e| {
        tracing::error!("Create issue error: {}", e);
        match e {
            atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(issue).unwrap_or_default())))
}

pub async fn get_issue(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let issue = state.risk_management_engine.get_issue(id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match issue {
        Some(i) => Ok(Json(serde_json::to_value(i).unwrap_or_default())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

#[derive(Debug, Deserialize)]
pub struct ListIssuesParams {
    pub status: Option<String>,
    pub severity: Option<String>,
}

pub async fn list_issues(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<ListIssuesParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let issues = state.risk_management_engine.list_issues(
        org_id, params.status.as_deref(), params.severity.as_deref(),
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({ "data": issues })))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateIssueStatusRequest {
    pub status: String,
}

pub async fn update_issue_status(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateIssueStatusRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let issue = state.risk_management_engine.update_issue_status(id, &payload.status).await.map_err(|e| {
        match e {
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;
    Ok(Json(serde_json::to_value(issue).unwrap_or_default()))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolveIssueRequest {
    pub root_cause: Option<String>,
    pub corrective_actions: Option<String>,
}

pub async fn resolve_issue(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<ResolveIssueRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let issue = state.risk_management_engine.resolve_issue(
        id, payload.root_cause.as_deref(), payload.corrective_actions.as_deref(),
    ).await.map_err(|e| {
        match e {
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;
    Ok(Json(serde_json::to_value(issue).unwrap_or_default()))
}

pub async fn delete_issue(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(issue_number): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    state.risk_management_engine.delete_issue(org_id, &issue_number).await.map_err(|e| {
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

pub async fn get_risk_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let dashboard = state.risk_management_engine.get_dashboard(org_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::to_value(dashboard).unwrap_or_default()))
}
