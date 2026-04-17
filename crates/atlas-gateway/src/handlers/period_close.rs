//! Period Close Management Handlers
//!
//! Oracle Fusion Cloud ERP: General Ledger Period Close
//! API endpoints for managing accounting calendars, periods,
//! subledger close status, and the period close checklist.

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
// Calendar Management
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateCalendarRequest {
    pub name: String,
    pub description: Option<String>,
    #[serde(default = "default_monthly")]
    pub calendar_type: String,
    #[serde(default = "default_one")]
    pub fiscal_year_start_month: i32,
    #[serde(default = "default_twelve")]
    pub periods_per_year: i32,
    #[serde(default)]
    pub has_adjusting_period: bool,
    pub current_fiscal_year: Option<i32>,
}

fn default_monthly() -> String { "monthly".to_string() }
fn default_one() -> i32 { 1 }
fn default_twelve() -> i32 { 12 }

#[derive(Debug, Deserialize)]
pub struct GeneratePeriodsRequest {
    pub fiscal_year: i32,
}

/// Create an accounting calendar
pub async fn create_calendar(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateCalendarRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let calendar = state.period_close_engine
        .create_calendar(
            org_id,
            &payload.name,
            payload.description.as_deref(),
            &payload.calendar_type,
            payload.fiscal_year_start_month,
            payload.periods_per_year,
            payload.has_adjusting_period,
            payload.current_fiscal_year,
            Some(user_id),
        )
        .await
        .map_err(|e| {
            error!("Create calendar error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(calendar).unwrap_or_default())))
}

/// List accounting calendars
pub async fn list_calendars(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let calendars = state.period_close_engine
        .list_calendars(org_id)
        .await
        .map_err(|e| { error!("List calendars error: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;

    Ok(Json(serde_json::json!({ "data": calendars })))
}

/// Get a specific calendar
pub async fn get_calendar(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let _org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let calendar = state.period_close_engine
        .get_calendar(id)
        .await
        .map_err(|e| { error!("Get calendar error: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(serde_json::to_value(calendar).unwrap_or_default()))
}

/// Delete a calendar (soft delete)
pub async fn delete_calendar(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    claims: Extension<Claims>,
) -> Result<StatusCode, StatusCode> {
    let _org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    state.period_close_engine
        .delete_calendar(id)
        .await
        .map_err(|e| { error!("Delete calendar error: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Period Management
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ListPeriodsParams {
    pub fiscal_year: Option<i32>,
}

/// Generate periods for a fiscal year
pub async fn generate_periods(
    State(state): State<Arc<AppState>>,
    Path(calendar_id): Path<Uuid>,
    claims: Extension<Claims>,
    Json(payload): Json<GeneratePeriodsRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let periods = state.period_close_engine
        .generate_periods(org_id, calendar_id, payload.fiscal_year)
        .await
        .map_err(|e| {
            error!("Generate periods error: {}", e);
            match e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::Forbidden(_) => StatusCode::FORBIDDEN,
                atlas_shared::AtlasError::DatabaseError(msg) if msg.contains("duplicate") || msg.contains("unique") => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok((StatusCode::CREATED, Json(serde_json::json!({ "data": periods }))))
}

/// List periods for a calendar
pub async fn list_periods(
    State(state): State<Arc<AppState>>,
    Path(calendar_id): Path<Uuid>,
    claims: Extension<Claims>,
    Query(params): Query<ListPeriodsParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let periods = state.period_close_engine
        .list_periods(org_id, calendar_id, params.fiscal_year)
        .await
        .map_err(|e| { error!("List periods error: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;

    Ok(Json(serde_json::json!({ "data": periods })))
}

/// Get a specific period
pub async fn get_period(
    State(state): State<Arc<AppState>>,
    Path(period_id): Path<Uuid>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let _org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let period = state.period_close_engine
        .get_period(period_id)
        .await
        .map_err(|e| { error!("Get period error: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(serde_json::to_value(period).unwrap_or_default()))
}

// ============================================================================
// Period Status Changes
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct PeriodStatusChangeRequest {
    /// Optional reason/comment for the status change
    #[serde(default)]
    pub comment: Option<String>,
    /// Force close even if subledgers aren't all closed
    #[serde(default)]
    pub force: bool,
}

/// Open a period
pub async fn open_period(
    State(state): State<Arc<AppState>>,
    Path(period_id): Path<Uuid>,
    claims: Extension<Claims>,
    Json(_payload): Json<PeriodStatusChangeRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let period = state.period_close_engine
        .open_period(period_id, Some(user_id))
        .await
        .map_err(|e| {
            error!("Open period error: {}", e);
            match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(Json(serde_json::to_value(period).unwrap_or_default()))
}

/// Set period to pending close
pub async fn pending_close_period(
    State(state): State<Arc<AppState>>,
    Path(period_id): Path<Uuid>,
    claims: Extension<Claims>,
    Json(_payload): Json<PeriodStatusChangeRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let period = state.period_close_engine
        .pending_close_period(period_id, Some(user_id))
        .await
        .map_err(|e| {
            error!("Pending close period error: {}", e);
            match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(Json(serde_json::to_value(period).unwrap_or_default()))
}

/// Close a period
pub async fn close_period(
    State(state): State<Arc<AppState>>,
    Path(period_id): Path<Uuid>,
    claims: Extension<Claims>,
    Json(payload): Json<PeriodStatusChangeRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let period = state.period_close_engine
        .close_period(period_id, Some(user_id), payload.force)
        .await
        .map_err(|e| {
            error!("Close period error: {}", e);
            match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(Json(serde_json::to_value(period).unwrap_or_default()))
}

/// Permanently close a period (irreversible)
pub async fn permanently_close_period(
    State(state): State<Arc<AppState>>,
    Path(period_id): Path<Uuid>,
    claims: Extension<Claims>,
    Json(_payload): Json<PeriodStatusChangeRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let period = state.period_close_engine
        .permanently_close_period(period_id, Some(user_id))
        .await
        .map_err(|e| {
            error!("Permanently close period error: {}", e);
            match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(Json(serde_json::to_value(period).unwrap_or_default()))
}

/// Reopen a closed period
pub async fn reopen_period(
    State(state): State<Arc<AppState>>,
    Path(period_id): Path<Uuid>,
    claims: Extension<Claims>,
    Json(_payload): Json<PeriodStatusChangeRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let period = state.period_close_engine
        .reopen_period(period_id, Some(user_id))
        .await
        .map_err(|e| {
            error!("Reopen period error: {}", e);
            match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(Json(serde_json::to_value(period).unwrap_or_default()))
}

// ============================================================================
// Subledger Status
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct UpdateSubledgerRequest {
    pub subledger: String,
    pub status: String,
}

/// Update subledger close status for a period
pub async fn update_subledger_status(
    State(state): State<Arc<AppState>>,
    Path(period_id): Path<Uuid>,
    claims: Extension<Claims>,
    Json(payload): Json<UpdateSubledgerRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let _org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let period = state.period_close_engine
        .update_subledger_status(period_id, &payload.subledger, &payload.status)
        .await
        .map_err(|e| {
            error!("Update subledger error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(Json(serde_json::to_value(period).unwrap_or_default()))
}

// ============================================================================
// Period Close Checklist
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateChecklistItemRequest {
    pub task_name: String,
    pub task_description: Option<String>,
    pub task_order: Option<i32>,
    pub category: Option<String>,
    pub subledger: Option<String>,
    pub assigned_to: Option<Uuid>,
    pub due_date: Option<String>,
    pub depends_on: Option<Uuid>,
    pub notes: Option<String>,
}

/// Add a checklist item to a period
pub async fn create_checklist_item(
    State(state): State<Arc<AppState>>,
    Path(period_id): Path<Uuid>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateChecklistItemRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let due_date = payload.due_date
        .as_deref()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());

    let item = state.period_close_engine
        .add_checklist_item(
            org_id,
            period_id,
            &payload.task_name,
            payload.task_description.as_deref(),
            payload.task_order,
            payload.category.as_deref(),
            payload.subledger.as_deref(),
            payload.assigned_to,
            due_date,
            payload.depends_on,
        )
        .await
        .map_err(|e| {
            error!("Create checklist item error: {}", e);
            match e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::Forbidden(_) => StatusCode::FORBIDDEN,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(item).unwrap_or_default())))
}

/// List checklist items for a period
pub async fn list_checklist_items(
    State(state): State<Arc<AppState>>,
    Path(period_id): Path<Uuid>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let _org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let items = state.period_close_engine
        .list_checklist_items(period_id)
        .await
        .map_err(|e| { error!("List checklist items error: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;

    Ok(Json(serde_json::json!({ "data": items })))
}

#[derive(Debug, Deserialize)]
pub struct UpdateChecklistItemRequest {
    pub status: String,
}

/// Update a checklist item status
pub async fn update_checklist_item(
    State(state): State<Arc<AppState>>,
    Path((_period_id, item_id)): Path<(Uuid, Uuid)>,
    claims: Extension<Claims>,
    Json(payload): Json<UpdateChecklistItemRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let completed_by = if payload.status == "completed" { Some(user_id) } else { None };

    let item = state.period_close_engine
        .update_checklist_item(item_id, &payload.status, completed_by)
        .await
        .map_err(|e| {
            error!("Update checklist item error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(Json(serde_json::to_value(item).unwrap_or_default()))
}

/// Delete a checklist item
pub async fn delete_checklist_item(
    State(state): State<Arc<AppState>>,
    Path((_period_id, item_id)): Path<(Uuid, Uuid)>,
    claims: Extension<Claims>,
) -> Result<StatusCode, StatusCode> {
    let _org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    state.period_close_engine
        .delete_checklist_item(item_id)
        .await
        .map_err(|e| { error!("Delete checklist item error: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Period Exceptions
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct GrantExceptionRequest {
    pub user_id: Uuid,
    pub allowed_actions: Option<Vec<String>>,
    pub reason: Option<String>,
    pub valid_until: Option<String>,
}

/// Grant a user exception to post to a period
pub async fn grant_period_exception(
    State(state): State<Arc<AppState>>,
    Path(period_id): Path<Uuid>,
    claims: Extension<Claims>,
    Json(payload): Json<GrantExceptionRequest>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let granted_by = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let valid_until = payload.valid_until
        .as_deref()
        .and_then(|d| chrono::DateTime::parse_from_rfc3339(d).ok())
        .map(|dt| dt.to_utc());

    state.period_close_engine
        .grant_exception(
            org_id,
            period_id,
            payload.user_id,
            payload.allowed_actions.unwrap_or_else(|| vec!["post".to_string()]),
            payload.reason.as_deref(),
            Some(granted_by),
            valid_until,
        )
        .await
        .map_err(|e| {
            error!("Grant period exception error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(StatusCode::NO_CONTENT)
}

/// Revoke a period exception
pub async fn revoke_period_exception(
    State(state): State<Arc<AppState>>,
    Path((period_id, user_id)): Path<(Uuid, Uuid)>,
    claims: Extension<Claims>,
) -> Result<StatusCode, StatusCode> {
    let _org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    state.period_close_engine
        .revoke_exception(period_id, user_id)
        .await
        .map_err(|e| {
            error!("Revoke period exception error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Period Close Dashboard
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CloseSummaryParams {
    pub fiscal_year: Option<i32>,
}

/// Get period close dashboard summary
pub async fn get_close_summary(
    State(state): State<Arc<AppState>>,
    Path(calendar_id): Path<Uuid>,
    claims: Extension<Claims>,
    Query(params): Query<CloseSummaryParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let summary = state.period_close_engine
        .get_close_summary(org_id, calendar_id, params.fiscal_year)
        .await
        .map_err(|e| {
            error!("Get close summary error: {}", e);
            match e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::Forbidden(_) => StatusCode::FORBIDDEN,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(Json(serde_json::to_value(summary).unwrap_or_default()))
}

// ============================================================================
// Posting Validation
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CheckPostingParams {
    pub date: String,
}

/// Check if posting is allowed for a given date
pub async fn check_posting_allowed(
    State(state): State<Arc<AppState>>,
    Path(calendar_id): Path<Uuid>,
    claims: Extension<Claims>,
    Query(params): Query<CheckPostingParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let date = chrono::NaiveDate::parse_from_str(&params.date, "%Y-%m-%d")
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    match state.period_close_engine
        .check_posting_allowed(org_id, calendar_id, date, Some(user_id))
        .await
    {
        Ok(period) => Ok(Json(serde_json::json!({
            "allowed": true,
            "period": period,
        }))),
        Err(atlas_shared::AtlasError::WorkflowError(msg)) => Ok(Json(serde_json::json!({
            "allowed": false,
            "reason": msg,
        }))),
        Err(atlas_shared::AtlasError::ValidationFailed(msg)) => Ok(Json(serde_json::json!({
            "allowed": false,
            "reason": msg,
        }))),
        Err(e) => {
            error!("Check posting error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
