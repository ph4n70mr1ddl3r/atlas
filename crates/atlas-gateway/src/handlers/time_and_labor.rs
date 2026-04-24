//! Time and Labor Management API Handlers
//!
//! Oracle Fusion Cloud HCM: Time and Labor > Schedules, Overtime Rules, Time Cards, Entries
//!
//! Endpoints for managing work schedules, overtime rules, time cards with entries,
//! labor distributions, and approval workflows.

use axum::{
    extract::{Path, Query, State, Extension},
    Json,
    http::StatusCode,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;
use tracing::error;

use crate::AppState;

// ============================================================================
// Query Parameters
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ListTimeCardsQuery {
    pub employee_id: Option<Uuid>,
    pub status: Option<String>,
}

// ============================================================================
// Work Schedule Handlers
// ========================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateWorkScheduleRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(default = "default_fixed")]
    pub schedule_type: String,
    #[serde(default = "default_eight")]
    pub standard_hours_per_day: f64,
    #[serde(default = "default_forty")]
    pub standard_hours_per_week: f64,
    #[serde(default = "default_five")]
    pub work_days_per_week: i32,
    pub start_time: Option<chrono::NaiveTime>,
    pub end_time: Option<chrono::NaiveTime>,
    #[serde(default = "default_sixty")]
    pub break_duration_minutes: i32,
}

fn default_fixed() -> String { "fixed".to_string() }
fn default_eight() -> f64 { 8.0 }
fn default_forty() -> f64 { 40.0 }
fn default_five() -> i32 { 5 }
fn default_sixty() -> i32 { 60 }

/// Create or update a work schedule
pub async fn create_work_schedule(
    State(state): State<Arc<AppState>>,
    claims: Extension<crate::handlers::auth::Claims>,
    Json(payload): Json<CreateWorkScheduleRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.time_and_labor_engine.create_work_schedule(
        org_id, &payload.code, &payload.name, payload.description.as_deref(),
        &payload.schedule_type, payload.standard_hours_per_day, payload.standard_hours_per_week,
        payload.work_days_per_week, payload.start_time, payload.end_time,
        payload.break_duration_minutes, Some(user_id),
    ).await {
        Ok(s) => Ok((StatusCode::CREATED, Json(serde_json::to_value(s).unwrap()))),
        Err(e) => { error!("Failed to create work schedule: {}", e); Err(map_error(e)) }
    }
}

/// Get a work schedule by code
pub async fn get_work_schedule(
    State(state): State<Arc<AppState>>,
    claims: Extension<crate::handlers::auth::Claims>,
    Path(code): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.time_and_labor_engine.get_work_schedule(org_id, &code).await {
        Ok(Some(s)) => Ok(Json(serde_json::to_value(s).unwrap())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Failed to get work schedule: {}", e); Err(map_error(e)) }
    }
}

/// List work schedules
pub async fn list_work_schedules(
    State(state): State<Arc<AppState>>,
    claims: Extension<crate::handlers::auth::Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.time_and_labor_engine.list_work_schedules(org_id).await {
        Ok(schedules) => Ok(Json(serde_json::json!({ "data": schedules }))),
        Err(e) => { error!("Failed to list work schedules: {}", e); Err(map_error(e)) }
    }
}

/// Delete (deactivate) a work schedule
pub async fn delete_work_schedule(
    State(state): State<Arc<AppState>>,
    claims: Extension<crate::handlers::auth::Claims>,
    Path(code): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.time_and_labor_engine.delete_work_schedule(org_id, &code).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => { error!("Failed to delete work schedule: {}", e); Err(map_error(e)) }
    }
}

// ============================================================================
// Overtime Rule Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOvertimeRuleRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(default = "default_weekly_threshold")]
    pub threshold_type: String,
    #[serde(default = "default_eight")]
    pub daily_threshold_hours: f64,
    #[serde(default = "default_forty")]
    pub weekly_threshold_hours: f64,
    #[serde(default = "default_one_point_five")]
    pub overtime_multiplier: f64,
    pub double_time_threshold_hours: Option<f64>,
    #[serde(default = "default_two")]
    pub double_time_multiplier: f64,
    #[serde(default)]
    pub include_holidays: bool,
    #[serde(default)]
    pub include_weekends: bool,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
}

fn default_weekly_threshold() -> String { "weekly".to_string() }
fn default_one_point_five() -> f64 { 1.5 }
fn default_two() -> f64 { 2.0 }

/// Create or update an overtime rule
pub async fn create_overtime_rule(
    State(state): State<Arc<AppState>>,
    claims: Extension<crate::handlers::auth::Claims>,
    Json(payload): Json<CreateOvertimeRuleRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.time_and_labor_engine.create_overtime_rule(
        org_id, &payload.code, &payload.name, payload.description.as_deref(),
        &payload.threshold_type, payload.daily_threshold_hours, payload.weekly_threshold_hours,
        payload.overtime_multiplier, payload.double_time_threshold_hours,
        payload.double_time_multiplier, payload.include_holidays, payload.include_weekends,
        payload.effective_from, payload.effective_to, Some(user_id),
    ).await {
        Ok(r) => Ok((StatusCode::CREATED, Json(serde_json::to_value(r).unwrap()))),
        Err(e) => { error!("Failed to create overtime rule: {}", e); Err(map_error(e)) }
    }
}

/// Get an overtime rule by code
pub async fn get_overtime_rule(
    State(state): State<Arc<AppState>>,
    claims: Extension<crate::handlers::auth::Claims>,
    Path(code): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.time_and_labor_engine.get_overtime_rule(org_id, &code).await {
        Ok(Some(r)) => Ok(Json(serde_json::to_value(r).unwrap())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Failed to get overtime rule: {}", e); Err(map_error(e)) }
    }
}

/// List overtime rules
pub async fn list_overtime_rules(
    State(state): State<Arc<AppState>>,
    claims: Extension<crate::handlers::auth::Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.time_and_labor_engine.list_overtime_rules(org_id).await {
        Ok(rules) => Ok(Json(serde_json::json!({ "data": rules }))),
        Err(e) => { error!("Failed to list overtime rules: {}", e); Err(map_error(e)) }
    }
}

/// Delete (deactivate) an overtime rule
pub async fn delete_overtime_rule(
    State(state): State<Arc<AppState>>,
    claims: Extension<crate::handlers::auth::Claims>,
    Path(code): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.time_and_labor_engine.delete_overtime_rule(org_id, &code).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => { error!("Failed to delete overtime rule: {}", e); Err(map_error(e)) }
    }
}

// ============================================================================
// Time Card Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTimeCardRequest {
    pub employee_id: Uuid,
    pub employee_name: Option<String>,
    pub period_start: chrono::NaiveDate,
    pub period_end: chrono::NaiveDate,
    pub schedule_code: Option<String>,
    pub overtime_rule_code: Option<String>,
}

/// Create a time card
pub async fn create_time_card(
    State(state): State<Arc<AppState>>,
    claims: Extension<crate::handlers::auth::Claims>,
    Json(payload): Json<CreateTimeCardRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.time_and_labor_engine.create_time_card(
        org_id, payload.employee_id, payload.employee_name.as_deref(),
        payload.period_start, payload.period_end,
        payload.schedule_code.as_deref(), payload.overtime_rule_code.as_deref(),
        Some(user_id),
    ).await {
        Ok(c) => Ok((StatusCode::CREATED, Json(serde_json::to_value(c).unwrap()))),
        Err(e) => { error!("Failed to create time card: {}", e); Err(map_error(e)) }
    }
}

/// Get a time card by ID
pub async fn get_time_card(
    State(state): State<Arc<AppState>>,
    _claims: Extension<crate::handlers::auth::Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.time_and_labor_engine.get_time_card(id).await {
        Ok(Some(c)) => Ok(Json(serde_json::to_value(c).unwrap())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Failed to get time card: {}", e); Err(map_error(e)) }
    }
}

/// List time cards
pub async fn list_time_cards(
    State(state): State<Arc<AppState>>,
    claims: Extension<crate::handlers::auth::Claims>,
    Query(params): Query<ListTimeCardsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.time_and_labor_engine.list_time_cards(
        org_id, params.employee_id, params.status.as_deref(),
    ).await {
        Ok(cards) => Ok(Json(serde_json::json!({ "data": cards }))),
        Err(e) => { error!("Failed to list time cards: {}", e); Err(map_error(e)) }
    }
}

/// Submit a time card for approval
pub async fn submit_time_card(
    State(state): State<Arc<AppState>>,
    claims: Extension<crate::handlers::auth::Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.time_and_labor_engine.submit_time_card(id, Some(user_id)).await {
        Ok(c) => Ok(Json(serde_json::to_value(c).unwrap())),
        Err(e) => { error!("Failed to submit time card: {}", e); Err(map_error(e)) }
    }
}

/// Approve a submitted time card
pub async fn approve_time_card(
    State(state): State<Arc<AppState>>,
    claims: Extension<crate::handlers::auth::Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.time_and_labor_engine.approve_time_card(id, user_id).await {
        Ok(c) => Ok(Json(serde_json::to_value(c).unwrap())),
        Err(e) => { error!("Failed to approve time card: {}", e); Err(map_error(e)) }
    }
}

/// Reject a submitted time card
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RejectTimeCardRequest {
    pub reason: Option<String>,
}

pub async fn reject_time_card(
    State(state): State<Arc<AppState>>,
    claims: Extension<crate::handlers::auth::Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<RejectTimeCardRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.time_and_labor_engine.reject_time_card(id, user_id, payload.reason.as_deref()).await {
        Ok(c) => Ok(Json(serde_json::to_value(c).unwrap())),
        Err(e) => { error!("Failed to reject time card: {}", e); Err(map_error(e)) }
    }
}

/// Cancel a time card
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelTimeCardRequest {
    pub reason: Option<String>,
}

pub async fn cancel_time_card(
    State(state): State<Arc<AppState>>,
    _claims: Extension<crate::handlers::auth::Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<CancelTimeCardRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.time_and_labor_engine.cancel_time_card(id, payload.reason.as_deref()).await {
        Ok(c) => Ok(Json(serde_json::to_value(c).unwrap())),
        Err(e) => { error!("Failed to cancel time card: {}", e); Err(map_error(e)) }
    }
}

// ============================================================================
// Time Entry Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTimeEntryRequest {
    pub time_card_id: Uuid,
    pub entry_date: chrono::NaiveDate,
    #[serde(default = "default_regular")]
    pub entry_type: String,
    pub start_time: Option<chrono::NaiveTime>,
    pub end_time: Option<chrono::NaiveTime>,
    pub duration_hours: f64,
    pub project_id: Option<Uuid>,
    pub project_name: Option<String>,
    pub department_id: Option<Uuid>,
    pub department_name: Option<String>,
    pub task_name: Option<String>,
    pub location: Option<String>,
    pub cost_center: Option<String>,
    pub labor_category: Option<String>,
    pub comments: Option<String>,
}

fn default_regular() -> String { "regular".to_string() }

/// Add a time entry to a time card
pub async fn create_time_entry(
    State(state): State<Arc<AppState>>,
    claims: Extension<crate::handlers::auth::Claims>,
    Json(payload): Json<CreateTimeEntryRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.time_and_labor_engine.create_time_entry(
        org_id, payload.time_card_id, payload.entry_date,
        &payload.entry_type, payload.start_time, payload.end_time,
        payload.duration_hours, payload.project_id, payload.project_name.as_deref(),
        payload.department_id, payload.department_name.as_deref(),
        payload.task_name.as_deref(), payload.location.as_deref(),
        payload.cost_center.as_deref(), payload.labor_category.as_deref(),
        payload.comments.as_deref(), Some(user_id),
    ).await {
        Ok(e) => Ok((StatusCode::CREATED, Json(serde_json::to_value(e).unwrap()))),
        Err(e2) => { error!("Failed to create time entry: {}", e2); Err(map_error(e2)) }
    }
}

/// List time entries for a time card
pub async fn list_time_entries(
    State(state): State<Arc<AppState>>,
    _claims: Extension<crate::handlers::auth::Claims>,
    Path(time_card_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.time_and_labor_engine.list_time_entries(time_card_id).await {
        Ok(entries) => Ok(Json(serde_json::json!({ "data": entries }))),
        Err(e) => { error!("Failed to list time entries: {}", e); Err(map_error(e)) }
    }
}

/// Delete a time entry
pub async fn delete_time_entry(
    State(state): State<Arc<AppState>>,
    _claims: Extension<crate::handlers::auth::Claims>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    match state.time_and_labor_engine.delete_time_entry(id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => { error!("Failed to delete time entry: {}", e); Err(map_error(e)) }
    }
}

// ============================================================================
// Time Card History
// ============================================================================

/// Get history for a time card
pub async fn get_time_card_history(
    State(state): State<Arc<AppState>>,
    _claims: Extension<crate::handlers::auth::Claims>,
    Path(time_card_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.time_and_labor_engine.get_time_card_history(time_card_id).await {
        Ok(history) => Ok(Json(serde_json::json!({ "data": history }))),
        Err(e) => { error!("Failed to get time card history: {}", e); Err(map_error(e)) }
    }
}

// ============================================================================
// Labor Distribution Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateLaborDistributionRequest {
    pub time_entry_id: Uuid,
    pub distribution_percent: f64,
    pub cost_center: Option<String>,
    pub project_id: Option<Uuid>,
    pub project_name: Option<String>,
    pub department_id: Option<Uuid>,
    pub department_name: Option<String>,
    pub gl_account_code: Option<String>,
}

/// Create a labor distribution for a time entry
pub async fn create_labor_distribution(
    State(state): State<Arc<AppState>>,
    claims: Extension<crate::handlers::auth::Claims>,
    Json(payload): Json<CreateLaborDistributionRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.time_and_labor_engine.create_labor_distribution(
        org_id, payload.time_entry_id, payload.distribution_percent,
        payload.cost_center.as_deref(), payload.project_id, payload.project_name.as_deref(),
        payload.department_id, payload.department_name.as_deref(), payload.gl_account_code.as_deref(),
    ).await {
        Ok(d) => Ok((StatusCode::CREATED, Json(serde_json::to_value(d).unwrap()))),
        Err(e) => { error!("Failed to create labor distribution: {}", e); Err(map_error(e)) }
    }
}

/// List labor distributions for a time entry
pub async fn list_labor_distributions(
    State(state): State<Arc<AppState>>,
    _claims: Extension<crate::handlers::auth::Claims>,
    Path(time_entry_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.time_and_labor_engine.list_labor_distributions(time_entry_id).await {
        Ok(dists) => Ok(Json(serde_json::json!({ "data": dists }))),
        Err(e) => { error!("Failed to list labor distributions: {}", e); Err(map_error(e)) }
    }
}

/// Delete a labor distribution
pub async fn delete_labor_distribution(
    State(state): State<Arc<AppState>>,
    _claims: Extension<crate::handlers::auth::Claims>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    match state.time_and_labor_engine.delete_labor_distribution(id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => { error!("Failed to delete labor distribution: {}", e); Err(map_error(e)) }
    }
}

// ============================================================================
// Dashboard
// ============================================================================

/// Get Time and Labor dashboard
pub async fn get_time_and_labor_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<crate::handlers::auth::Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.time_and_labor_engine.get_dashboard(org_id).await {
        Ok(dashboard) => Ok(Json(serde_json::to_value(dashboard).unwrap())),
        Err(e) => { error!("Failed to get time and labor dashboard: {}", e); Err(map_error(e)) }
    }
}

// ============================================================================
// Helpers
// ============================================================================

fn map_error(e: atlas_shared::AtlasError) -> StatusCode {
    match e.status_code().try_into() {
        Ok(code) => code,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
