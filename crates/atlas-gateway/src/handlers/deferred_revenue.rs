//! Deferred Revenue/Cost Management Handlers
//!
//! Oracle Fusion Cloud ERP: Financials > Revenue Management > Deferral Schedules
//!
//! API endpoints for deferral templates, recognition schedules,
//! and automated amortization of deferred revenue and costs.

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
use tracing::{info, error};

// ============================================================================
// Deferral Templates
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateTemplateRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub deferral_type: String,
    pub recognition_method: String,
    pub deferral_account_code: String,
    pub recognition_account_code: String,
    pub contra_account_code: Option<String>,
    pub default_periods: i32,
    pub period_type: String,
    pub start_date_basis: String,
    pub end_date_basis: String,
    #[serde(default)]
    pub prorate_partial_periods: bool,
    #[serde(default)]
    pub auto_generate_schedule: bool,
    #[serde(default)]
    pub auto_post: bool,
    pub rounding_threshold: Option<String>,
    pub currency_code: String,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
}

pub async fn create_template(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateTemplateRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.deferred_revenue_engine.create_template(
        org_id, &payload.code, &payload.name, payload.description.as_deref(),
        &payload.deferral_type, &payload.recognition_method,
        &payload.deferral_account_code, &payload.recognition_account_code,
        payload.contra_account_code.as_deref(), payload.default_periods,
        &payload.period_type, &payload.start_date_basis, &payload.end_date_basis,
        payload.prorate_partial_periods, payload.auto_generate_schedule,
        payload.auto_post, payload.rounding_threshold.as_deref(),
        &payload.currency_code, payload.effective_from, payload.effective_to,
        Some(user_id),
    ).await {
        Ok(template) => Ok((StatusCode::CREATED, Json(serde_json::to_value(template).unwrap()))),
        Err(e) => {
            error!("Failed to create deferral template: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn list_templates(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.deferred_revenue_engine.list_templates(org_id, None).await {
        Ok(templates) => Ok(Json(serde_json::json!({ "data": templates }))),
        Err(e) => { error!("Failed to list templates: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn get_template(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(code): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.deferred_revenue_engine.get_template(org_id, &code).await {
        Ok(Some(t)) => Ok(Json(serde_json::to_value(t).unwrap())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Failed to get template: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn delete_template(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(code): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.deferred_revenue_engine.delete_template(org_id, &code).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            error!("Failed to delete template: {}", e);
            Err(match e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// ============================================================================
// Deferral Schedules
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateScheduleRequest {
    pub schedule_number: String,
    pub template_id: Uuid,
    pub source_type: String,
    pub source_id: Option<Uuid>,
    pub source_number: Option<String>,
    pub source_line_id: Option<Uuid>,
    pub description: Option<String>,
    pub total_amount: String,
    pub currency_code: String,
    pub start_date: chrono::NaiveDate,
    pub end_date: chrono::NaiveDate,
    pub original_journal_entry_id: Option<Uuid>,
}

pub async fn create_schedule(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateScheduleRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.deferred_revenue_engine.create_schedule(
        org_id, payload.template_id, &payload.source_type,
        payload.source_id, payload.source_number.as_deref(),
        payload.source_line_id, payload.description.as_deref(),
        &payload.total_amount, &payload.currency_code,
        payload.start_date, payload.end_date,
        payload.original_journal_entry_id, Some(user_id),
    ).await {
        Ok(schedule) => Ok((StatusCode::CREATED, Json(serde_json::to_value(schedule).unwrap()))),
        Err(e) => {
            error!("Failed to create deferral schedule: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListSchedulesQuery {
    pub status: Option<String>,
    pub deferral_type: Option<String>,
}

pub async fn list_schedules(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListSchedulesQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.deferred_revenue_engine.list_schedules(
        org_id, query.status.as_deref(), query.deferral_type.as_deref(), None,
    ).await {
        Ok(schedules) => Ok(Json(serde_json::json!({ "data": schedules }))),
        Err(e) => { error!("Failed to list schedules: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn get_schedule(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.deferred_revenue_engine.get_schedule(id).await {
        Ok(Some(s)) => Ok(Json(serde_json::to_value(s).unwrap())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Failed to get schedule: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn list_schedule_lines(
    State(state): State<Arc<AppState>>,
    Path(schedule_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.deferred_revenue_engine.list_schedule_lines(schedule_id).await {
        Ok(lines) => Ok(Json(serde_json::json!({ "data": lines }))),
        Err(e) => { error!("Failed to list schedule lines: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// ============================================================================
// Schedule Lifecycle
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct RecognizePendingRequest {
    pub as_of_date: chrono::NaiveDate,
}

pub async fn recognize_pending(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<RecognizePendingRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.deferred_revenue_engine.recognize_pending(org_id, payload.as_of_date).await {
        Ok(lines) => Ok(Json(serde_json::json!({ "data": lines }))),
        Err(e) => { error!("Failed to recognize pending: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

#[derive(Debug, Deserialize)]
pub struct HoldScheduleRequest {
    pub reason: String,
}

pub async fn hold_schedule(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<HoldScheduleRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.deferred_revenue_engine.hold_schedule(id, &payload.reason).await {
        Ok(s) => Ok(Json(serde_json::to_value(s).unwrap())),
        Err(e) => {
            error!("Failed to hold schedule: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn resume_schedule(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.deferred_revenue_engine.resume_schedule(id).await {
        Ok(s) => Ok(Json(serde_json::to_value(s).unwrap())),
        Err(e) => {
            error!("Failed to resume schedule: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn cancel_schedule(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.deferred_revenue_engine.cancel_schedule(id).await {
        Ok(s) => Ok(Json(serde_json::to_value(s).unwrap())),
        Err(e) => {
            error!("Failed to cancel schedule: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// ============================================================================
// Dashboard
// ============================================================================

pub async fn get_deferred_revenue_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.deferred_revenue_engine.get_dashboard_summary(org_id).await {
        Ok(dashboard) => Ok(Json(serde_json::to_value(dashboard).unwrap())),
        Err(e) => { error!("Failed to get deferred revenue dashboard: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}
