//! Goal Management Handlers
//!
//! Oracle Fusion HCM > Goal Management endpoints:
//! - Library category and template CRUD
//! - Goal plan management (draft → active → closed)
//! - Goal CRUD with cascading hierarchy
//! - Goal progress updates
//! - Goal alignment management
//! - Goal notes (comments, feedback, check-ins)
//! - Goal management dashboard summary

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
// Library Categories
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCategoryRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub display_order: Option<i32>,
}

pub async fn create_library_category(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateCategoryRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let cat = state.goal_management_engine.create_library_category(
        org_id, &payload.code, &payload.name, payload.description.as_deref(),
        payload.display_order.unwrap_or(0), Some(user_id),
    ).await.map_err(|e| {
        error!("Create category error: {}", e);
        match e {
            atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(cat).unwrap_or_default())))
}

pub async fn list_library_categories(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let cats = state.goal_management_engine.list_library_categories(org_id).await
        .map_err(|e| { error!("List categories error: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;
    Ok(Json(serde_json::json!({ "data": cats, "meta": { "total": cats.len() } })))
}

pub async fn delete_library_category(
    State(state): State<Arc<AppState>>,
    Path(code): Path<String>,
    claims: Extension<Claims>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    state.goal_management_engine.delete_library_category(org_id, &code).await.map_err(|e| {
        error!("Delete category error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Library Templates
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTemplateRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub category_id: Option<Uuid>,
    pub goal_type: Option<String>,
    pub success_criteria: Option<String>,
    pub target_metric: Option<String>,
    pub target_value: Option<String>,
    pub uom: Option<String>,
    pub suggested_weight: Option<String>,
    pub estimated_duration_days: Option<i32>,
}

pub async fn create_library_template(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateTemplateRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let tmpl = state.goal_management_engine.create_library_template(
        org_id, payload.category_id, &payload.code, &payload.name,
        payload.description.as_deref(),
        payload.goal_type.as_deref().unwrap_or("individual"),
        payload.success_criteria.as_deref(), payload.target_metric.as_deref(),
        payload.target_value.as_deref(), payload.uom.as_deref(),
        payload.suggested_weight.as_deref(), payload.estimated_duration_days,
        Some(user_id),
    ).await.map_err(|e| {
        error!("Create template error: {}", e);
        match e {
            atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(tmpl).unwrap_or_default())))
}

#[derive(Debug, Deserialize)]
pub struct ListTemplatesParams {
    pub category_id: Option<Uuid>,
    pub goal_type: Option<String>,
}

pub async fn list_library_templates(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<ListTemplatesParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let tmpls = state.goal_management_engine.list_library_templates(
        org_id, params.category_id, params.goal_type.as_deref(),
    ).await.map_err(|e| { error!("List templates error: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;
    Ok(Json(serde_json::json!({ "data": tmpls, "meta": { "total": tmpls.len() } })))
}

pub async fn delete_library_template(
    State(state): State<Arc<AppState>>,
    Path(code): Path<String>,
    claims: Extension<Claims>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    state.goal_management_engine.delete_library_template(org_id, &code).await.map_err(|e| {
        error!("Delete template error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Goal Plans
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateGoalPlanRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub plan_type: Option<String>,
    pub review_period_start: String,
    pub review_period_end: String,
    pub goal_creation_deadline: Option<String>,
    pub allow_self_goals: Option<bool>,
    pub allow_team_goals: Option<bool>,
    pub max_weight_sum: Option<String>,
}

pub async fn create_goal_plan(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateGoalPlanRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let start = chrono::NaiveDate::parse_from_str(&payload.review_period_start, "%Y-%m-%d")
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let end = chrono::NaiveDate::parse_from_str(&payload.review_period_end, "%Y-%m-%d")
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let deadline = payload.goal_creation_deadline.as_deref()
        .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());

    let plan = state.goal_management_engine.create_goal_plan(
        org_id, &payload.code, &payload.name, payload.description.as_deref(),
        payload.plan_type.as_deref().unwrap_or("performance"),
        start, end, deadline,
        payload.allow_self_goals.unwrap_or(true),
        payload.allow_team_goals.unwrap_or(true),
        payload.max_weight_sum.as_deref(),
        Some(user_id),
    ).await.map_err(|e| {
        error!("Create plan error: {}", e);
        match e {
            atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(plan).unwrap_or_default())))
}

pub async fn get_goal_plan(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let plan = state.goal_management_engine.get_goal_plan(id).await
        .map_err(|e| { error!("Get plan error: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;
    match plan {
        Some(p) => Ok(Json(serde_json::to_value(p).unwrap_or_default())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

#[derive(Debug, Deserialize)]
pub struct ListPlansParams {
    pub status: Option<String>,
}

pub async fn list_goal_plans(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<ListPlansParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let plans = state.goal_management_engine.list_goal_plans(
        org_id, params.status.as_deref(),
    ).await.map_err(|e| { error!("List plans error: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;
    Ok(Json(serde_json::json!({ "data": plans, "meta": { "total": plans.len() } })))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePlanStatusRequest {
    pub status: String,
}

pub async fn update_goal_plan_status(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    claims: Extension<Claims>,
    Json(payload): Json<UpdatePlanStatusRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let _org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let plan = state.goal_management_engine.update_goal_plan_status(
        id, &payload.status,
    ).await.map_err(|e| {
        error!("Update plan status error: {}", e);
        match e {
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;
    Ok(Json(serde_json::to_value(plan).unwrap_or_default()))
}

pub async fn delete_goal_plan(
    State(state): State<Arc<AppState>>,
    Path(code): Path<String>,
    claims: Extension<Claims>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    state.goal_management_engine.delete_goal_plan(org_id, &code).await.map_err(|e| {
        error!("Delete plan error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Goals
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateGoalRequest {
    pub plan_id: Option<Uuid>,
    pub parent_goal_id: Option<Uuid>,
    pub library_template_id: Option<Uuid>,
    pub code: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub goal_type: Option<String>,
    pub category: Option<String>,
    pub owner_id: Option<Uuid>,
    pub owner_type: Option<String>,
    pub success_criteria: Option<String>,
    pub target_metric: Option<String>,
    pub target_value: Option<String>,
    pub uom: Option<String>,
    pub weight: Option<String>,
    pub priority: Option<String>,
    pub start_date: Option<String>,
    pub target_date: Option<String>,
}

pub async fn create_goal(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateGoalRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let start_date = payload.start_date.as_deref()
        .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());
    let target_date = payload.target_date.as_deref()
        .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());

    let owner_id = payload.owner_id.unwrap_or(user_id);

    let goal = state.goal_management_engine.create_goal(
        org_id, payload.plan_id, payload.parent_goal_id,
        payload.library_template_id, payload.code.as_deref(),
        &payload.name, payload.description.as_deref(),
        payload.goal_type.as_deref().unwrap_or("individual"),
        payload.category.as_deref(),
        owner_id,
        payload.owner_type.as_deref().unwrap_or("employee"),
        Some(user_id),
        payload.success_criteria.as_deref(), payload.target_metric.as_deref(),
        payload.target_value.as_deref(), payload.uom.as_deref(),
        payload.weight.as_deref(),
        payload.priority.as_deref().unwrap_or("medium"),
        start_date, target_date, Some(user_id),
    ).await.map_err(|e| {
        error!("Create goal error: {}", e);
        match e {
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(goal).unwrap_or_default())))
}

pub async fn get_goal(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let goal = state.goal_management_engine.get_goal(id).await
        .map_err(|e| { error!("Get goal error: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;
    match goal {
        Some(g) => Ok(Json(serde_json::to_value(g).unwrap_or_default())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

#[derive(Debug, Deserialize)]
pub struct ListGoalsParams {
    pub plan_id: Option<Uuid>,
    pub owner_id: Option<Uuid>,
    pub goal_type: Option<String>,
    pub status: Option<String>,
    pub parent_goal_id: Option<Uuid>,
}

pub async fn list_goals(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<ListGoalsParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let goals = state.goal_management_engine.list_goals(
        org_id, params.plan_id, params.owner_id,
        params.goal_type.as_deref(), params.status.as_deref(),
        params.parent_goal_id,
    ).await.map_err(|e| { error!("List goals error: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;
    Ok(Json(serde_json::json!({ "data": goals, "meta": { "total": goals.len() } })))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateGoalProgressRequest {
    pub actual_value: Option<String>,
    pub progress_pct: Option<String>,
    pub status: Option<String>,
}

pub async fn update_goal_progress(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    claims: Extension<Claims>,
    Json(payload): Json<UpdateGoalProgressRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let _org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let goal = state.goal_management_engine.update_goal_progress(
        id, payload.actual_value.as_deref(),
        payload.progress_pct.as_deref(), payload.status.as_deref(),
    ).await.map_err(|e| {
        error!("Update goal progress error: {}", e);
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
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    state.goal_management_engine.delete_goal(id).await.map_err(|e| {
        error!("Delete goal error: {}", e);
        match e {
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Goal Alignments
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAlignmentRequest {
    pub source_goal_id: Uuid,
    pub aligned_to_goal_id: Uuid,
    pub alignment_type: Option<String>,
    pub description: Option<String>,
}

pub async fn create_goal_alignment(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateAlignmentRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let alignment = state.goal_management_engine.create_goal_alignment(
        org_id, payload.source_goal_id, payload.aligned_to_goal_id,
        payload.alignment_type.as_deref().unwrap_or("supports"),
        payload.description.as_deref(),
    ).await.map_err(|e| {
        error!("Create alignment error: {}", e);
        match e {
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(alignment).unwrap_or_default())))
}

pub async fn list_goal_alignments(
    State(state): State<Arc<AppState>>,
    Path(goal_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let alignments = state.goal_management_engine.list_goal_alignments(goal_id).await
        .map_err(|e| { error!("List alignments error: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;
    Ok(Json(serde_json::json!({ "data": alignments, "meta": { "total": alignments.len() } })))
}

pub async fn delete_goal_alignment(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    state.goal_management_engine.delete_goal_alignment(id).await.map_err(|e| {
        error!("Delete alignment error: {}", e);
        match e {
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Goal Notes
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateGoalNoteRequest {
    pub note_type: Option<String>,
    pub content: String,
    pub visibility: Option<String>,
}

pub async fn create_goal_note(
    State(state): State<Arc<AppState>>,
    Path(goal_id): Path<Uuid>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateGoalNoteRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let note = state.goal_management_engine.create_goal_note(
        org_id, goal_id, user_id,
        payload.note_type.as_deref().unwrap_or("comment"),
        &payload.content,
        payload.visibility.as_deref().unwrap_or("private"),
    ).await.map_err(|e| {
        error!("Create note error: {}", e);
        match e {
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(note).unwrap_or_default())))
}

pub async fn list_goal_notes(
    State(state): State<Arc<AppState>>,
    Path(goal_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let notes = state.goal_management_engine.list_goal_notes(goal_id).await
        .map_err(|e| { error!("List notes error: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;
    Ok(Json(serde_json::json!({ "data": notes, "meta": { "total": notes.len() } })))
}

pub async fn delete_goal_note(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    state.goal_management_engine.delete_goal_note(id).await.map_err(|e| {
        error!("Delete note error: {}", e);
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

pub async fn get_goal_management_summary(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let summary = state.goal_management_engine.get_summary(org_id).await
        .map_err(|e| { error!("Goal summary error: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;
    Ok(Json(serde_json::to_value(summary).unwrap_or_default()))
}
