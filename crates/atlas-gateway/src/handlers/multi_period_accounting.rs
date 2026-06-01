//! Multi-Period Accounting (MPA) Handlers
//!
//! Oracle Fusion: Financials > General Ledger > Multi-Period Accounting

use crate::handlers::auth::Claims;
use crate::handlers::{created_json, to_json};
use crate::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension, Json,
};
use serde::Deserialize;
use std::sync::Arc;
use tracing::error;
use uuid::Uuid;

// ============================================================================
// Templates
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateTemplateRequest {
    pub template_name: String,
    pub description: Option<String>,
    pub distribution_method: Option<String>,
    pub number_of_periods: Option<i32>,
    pub period_type: Option<String>,
    pub deferred_account_code: Option<String>,
    pub expense_account_code: Option<String>,
    pub currency_code: Option<String>,
}

pub async fn create_template(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateTemplateRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state
        .financials
        .mpa_engine
        .create_template(
            org_id,
            &payload.template_name,
            payload.description.as_deref(),
            payload.distribution_method.as_deref().unwrap_or("equal"),
            payload.number_of_periods.unwrap_or(1),
            payload.period_type.as_deref().unwrap_or("month"),
            payload.deferred_account_code.as_deref(),
            payload.expense_account_code.as_deref(),
            payload.currency_code.as_deref().unwrap_or("USD"),
            Some(user_id),
        )
        .await
    {
        Ok(t) => Ok(created_json(t)),
        Err(e) => {
            error!("Failed to create MPA template: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListTemplatesQuery {
    pub status: Option<String>,
}

pub async fn list_templates(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListTemplatesQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state
        .financials
        .mpa_engine
        .list_templates(org_id, query.status.as_deref())
        .await
    {
        Ok(templates) => Ok(Json(serde_json::json!({ "data": templates }))),
        Err(e) => {
            error!("Failed to list MPA templates: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn get_template(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.financials.mpa_engine.get_template(id).await {
        Ok(Some(t)) => Ok(to_json(t)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get MPA template: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn activate_template(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.financials.mpa_engine.activate_template(id).await {
        Ok(t) => Ok(to_json(t)),
        Err(e) => {
            error!("Failed to activate MPA template: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn deactivate_template(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.financials.mpa_engine.deactivate_template(id).await {
        Ok(t) => Ok(to_json(t)),
        Err(e) => {
            error!("Failed to deactivate MPA template: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct AddTemplateLineRequest {
    pub period_sequence: i32,
    pub percentage: String,
    pub offset_days: Option<i32>,
}

pub async fn add_template_line(
    State(state): State<Arc<AppState>>,
    Path(template_id): Path<Uuid>,
    Json(payload): Json<AddTemplateLineRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    match state
        .financials
        .mpa_engine
        .add_template_line(
            template_id,
            payload.period_sequence,
            &payload.percentage,
            payload.offset_days.unwrap_or(0),
        )
        .await
    {
        Ok(line) => Ok(created_json(line)),
        Err(e) => {
            error!("Failed to add template line: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_)
                | atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn list_template_lines(
    State(state): State<Arc<AppState>>,
    Path(template_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state
        .financials
        .mpa_engine
        .list_template_lines(template_id)
        .await
    {
        Ok(lines) => Ok(Json(serde_json::json!({ "data": lines }))),
        Err(e) => {
            error!("Failed to list template lines: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// ============================================================================
// Schedules
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateScheduleRequest {
    pub schedule_number: String,
    pub description: Option<String>,
    pub template_id: Option<Uuid>,
    pub source_journal_entry_id: Option<Uuid>,
    pub source_journal_line_id: Option<Uuid>,
    pub total_amount: String,
    pub start_date: chrono::NaiveDate,
    pub end_date: Option<chrono::NaiveDate>,
    pub currency_code: Option<String>,
    pub company_code: Option<String>,
    pub cost_center: Option<String>,
    pub account_segment: Option<String>,
}

pub async fn create_schedule(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateScheduleRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state
        .financials
        .mpa_engine
        .create_schedule(
            org_id,
            &payload.schedule_number,
            payload.description.as_deref(),
            payload.template_id,
            payload.source_journal_entry_id,
            payload.source_journal_line_id,
            &payload.total_amount,
            payload.start_date,
            payload.end_date,
            payload.currency_code.as_deref().unwrap_or("USD"),
            payload.company_code.as_deref(),
            payload.cost_center.as_deref(),
            payload.account_segment.as_deref(),
            Some(user_id),
        )
        .await
    {
        Ok(s) => Ok(created_json(s)),
        Err(e) => {
            error!("Failed to create MPA schedule: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListSchedulesQuery {
    pub status: Option<String>,
}

pub async fn list_schedules(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListSchedulesQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state
        .financials
        .mpa_engine
        .list_schedules(org_id, query.status.as_deref())
        .await
    {
        Ok(schedules) => Ok(Json(serde_json::json!({ "data": schedules }))),
        Err(e) => {
            error!("Failed to list MPA schedules: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn get_schedule(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.financials.mpa_engine.get_schedule(id).await {
        Ok(Some(s)) => Ok(to_json(s)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get MPA schedule: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn activate_schedule(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.financials.mpa_engine.activate_schedule(id).await {
        Ok(s) => Ok(to_json(s)),
        Err(e) => {
            error!("Failed to activate MPA schedule: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn hold_schedule(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.financials.mpa_engine.hold_schedule(id).await {
        Ok(s) => Ok(to_json(s)),
        Err(e) => {
            error!("Failed to hold MPA schedule: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn cancel_schedule(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.financials.mpa_engine.cancel_schedule(id).await {
        Ok(s) => Ok(to_json(s)),
        Err(e) => {
            error!("Failed to cancel MPA schedule: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// ============================================================================
// Schedule Lines
// ============================================================================

pub async fn list_schedule_lines(
    State(state): State<Arc<AppState>>,
    Path(schedule_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state
        .financials
        .mpa_engine
        .list_schedule_lines(schedule_id)
        .await
    {
        Ok(lines) => Ok(Json(serde_json::json!({ "data": lines }))),
        Err(e) => {
            error!("Failed to list schedule lines: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn recognize_line(
    State(state): State<Arc<AppState>>,
    Path(line_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.financials.mpa_engine.recognize_line(line_id).await {
        Ok(l) => Ok(to_json(l)),
        Err(e) => {
            error!("Failed to recognize MPA line: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn reverse_line(
    State(state): State<Arc<AppState>>,
    Path(line_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.financials.mpa_engine.reverse_line(line_id).await {
        Ok(l) => Ok(to_json(l)),
        Err(e) => {
            error!("Failed to reverse MPA line: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// ============================================================================
// Dashboard
// ============================================================================

pub async fn get_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.financials.mpa_engine.get_dashboard(org_id).await {
        Ok(dashboard) => Ok(to_json(dashboard)),
        Err(e) => {
            error!("Failed to get MPA dashboard: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
