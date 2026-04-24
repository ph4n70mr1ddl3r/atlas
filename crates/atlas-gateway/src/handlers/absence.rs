//! Absence Management API Handlers
//!
//! Oracle Fusion Cloud HCM: Absence Management > Types, Plans, Entries
//!
//! Endpoints for managing absence types, absence plans with accrual rules,
//! employee absence entries with approval workflows, balances, and dashboard.

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
use crate::handlers::auth::Claims;

// ============================================================================
// Query Parameters
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ListAbsenceTypesQuery {
    pub category: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListAbsencePlansQuery {
    pub absence_type_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct ListAbsenceEntriesQuery {
    pub employee_id: Option<Uuid>,
    pub absence_type_id: Option<Uuid>,
    pub status: Option<String>,
}

// ============================================================================
// Absence Type Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAbsenceTypeRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub category: String,
    pub plan_type: String,
    #[serde(default = "default_true")]
    pub requires_approval: bool,
    #[serde(default)]
    pub requires_documentation: bool,
    #[serde(default)]
    pub auto_approve_below_days: f64,
    #[serde(default)]
    pub allow_negative_balance: bool,
    #[serde(default = "default_true")]
    pub allow_half_day: bool,
}

fn default_true() -> bool { true }

/// Create or update an absence type
pub async fn create_absence_type(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateAbsenceTypeRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.absence_engine.create_absence_type(
        org_id,
        &payload.code,
        &payload.name,
        payload.description.as_deref(),
        &payload.category,
        &payload.plan_type,
        payload.requires_approval,
        payload.requires_documentation,
        payload.auto_approve_below_days,
        payload.allow_negative_balance,
        payload.allow_half_day,
        Some(user_id),
    ).await {
        Ok(at) => Ok((StatusCode::CREATED, Json(serde_json::to_value(at).unwrap_or_default()))),
        Err(e) => {
            error!("Failed to create absence type: {}", e);
            Err(map_error(e))
        }
    }
}

/// Get an absence type by code
pub async fn get_absence_type(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(code): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.absence_engine.get_absence_type(org_id, &code).await {
        Ok(Some(at)) => Ok(Json(serde_json::to_value(at).unwrap_or_default())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get absence type: {}", e);
            Err(map_error(e))
        }
    }
}

/// List absence types
pub async fn list_absence_types(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<ListAbsenceTypesQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.absence_engine.list_absence_types(org_id, params.category.as_deref()).await {
        Ok(types) => Ok(Json(serde_json::json!({ "data": types }))),
        Err(e) => {
            error!("Failed to list absence types: {}", e);
            Err(map_error(e))
        }
    }
}

/// Delete (deactivate) an absence type
pub async fn delete_absence_type(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(code): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.absence_engine.delete_absence_type(org_id, &code).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            error!("Failed to delete absence type: {}", e);
            Err(map_error(e))
        }
    }
}

// ============================================================================
// Absence Plan Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAbsencePlanRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub absence_type_code: String,
    pub accrual_frequency: String,
    pub accrual_rate: f64,
    #[serde(default = "default_days")]
    pub accrual_unit: String,
    pub carry_over_max: Option<f64>,
    pub carry_over_expiry_months: Option<i32>,
    pub max_balance: Option<f64>,
    #[serde(default)]
    pub probation_period_days: i32,
    #[serde(default)]
    pub prorate_first_year: bool,
}

fn default_days() -> String { "days".to_string() }

/// Create or update an absence plan
pub async fn create_absence_plan(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateAbsencePlanRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.absence_engine.create_absence_plan(
        org_id,
        &payload.code,
        &payload.name,
        payload.description.as_deref(),
        &payload.absence_type_code,
        &payload.accrual_frequency,
        payload.accrual_rate,
        &payload.accrual_unit,
        payload.carry_over_max,
        payload.carry_over_expiry_months,
        payload.max_balance,
        payload.probation_period_days,
        payload.prorate_first_year,
        Some(user_id),
    ).await {
        Ok(plan) => Ok((StatusCode::CREATED, Json(serde_json::to_value(plan).unwrap_or_default()))),
        Err(e) => {
            error!("Failed to create absence plan: {}", e);
            Err(map_error(e))
        }
    }
}

/// Get an absence plan by code
pub async fn get_absence_plan(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(code): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.absence_engine.get_absence_plan(org_id, &code).await {
        Ok(Some(plan)) => Ok(Json(serde_json::to_value(plan).unwrap_or_default())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get absence plan: {}", e);
            Err(map_error(e))
        }
    }
}

/// List absence plans
pub async fn list_absence_plans(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<ListAbsencePlansQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.absence_engine.list_absence_plans(org_id, params.absence_type_id).await {
        Ok(plans) => Ok(Json(serde_json::json!({ "data": plans }))),
        Err(e) => {
            error!("Failed to list absence plans: {}", e);
            Err(map_error(e))
        }
    }
}

/// Delete (deactivate) an absence plan
pub async fn delete_absence_plan(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(code): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.absence_engine.delete_absence_plan(org_id, &code).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            error!("Failed to delete absence plan: {}", e);
            Err(map_error(e))
        }
    }
}

// ============================================================================
// Absence Entry Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAbsenceEntryRequest {
    pub employee_id: Uuid,
    pub employee_name: Option<String>,
    pub absence_type_code: String,
    pub plan_code: Option<String>,
    pub start_date: chrono::NaiveDate,
    pub end_date: chrono::NaiveDate,
    pub duration_days: f64,
    pub duration_hours: Option<f64>,
    #[serde(default)]
    pub is_half_day: bool,
    pub half_day_period: Option<String>,
    pub reason: Option<String>,
    pub comments: Option<String>,
    #[serde(default)]
    pub documentation_provided: bool,
}

/// Create a new absence entry
pub async fn create_entry(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateAbsenceEntryRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.absence_engine.create_entry(
        org_id,
        payload.employee_id,
        payload.employee_name.as_deref(),
        &payload.absence_type_code,
        payload.plan_code.as_deref(),
        payload.start_date,
        payload.end_date,
        payload.duration_days,
        payload.duration_hours,
        payload.is_half_day,
        payload.half_day_period.as_deref(),
        payload.reason.as_deref(),
        payload.comments.as_deref(),
        payload.documentation_provided,
        Some(user_id),
    ).await {
        Ok(entry) => Ok((StatusCode::CREATED, Json(serde_json::to_value(entry).unwrap_or_default()))),
        Err(e) => {
            error!("Failed to create absence entry: {}", e);
            Err(map_error(e))
        }
    }
}

/// Get an absence entry by ID
pub async fn get_entry(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.absence_engine.get_entry(org_id, id).await {
        Ok(Some(entry)) => Ok(Json(serde_json::to_value(entry).unwrap_or_default())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get absence entry: {}", e);
            Err(map_error(e))
        }
    }
}

/// List absence entries
pub async fn list_entries(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<ListAbsenceEntriesQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.absence_engine.list_entries(
        org_id, params.employee_id, params.absence_type_id, params.status.as_deref(),
    ).await {
        Ok(entries) => Ok(Json(serde_json::json!({ "data": entries }))),
        Err(e) => {
            error!("Failed to list absence entries: {}", e);
            Err(map_error(e))
        }
    }
}

/// Submit a draft entry for approval
pub async fn submit_entry(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.absence_engine.submit_entry(org_id, id, Some(user_id)).await {
        Ok(entry) => Ok(Json(serde_json::to_value(entry).unwrap_or_default())),
        Err(e) => {
            error!("Failed to submit absence entry: {}", e);
            Err(map_error(e))
        }
    }
}

/// Approve a submitted entry
pub async fn approve_entry(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.absence_engine.approve_entry(org_id, id, user_id).await {
        Ok(entry) => Ok(Json(serde_json::to_value(entry).unwrap_or_default())),
        Err(e) => {
            error!("Failed to approve absence entry: {}", e);
            Err(map_error(e))
        }
    }
}

/// Reject a submitted entry
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RejectEntryRequest {
    pub reason: Option<String>,
}

pub async fn reject_entry(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<RejectEntryRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.absence_engine.reject_entry(org_id, id, user_id, payload.reason.as_deref()).await {
        Ok(entry) => Ok(Json(serde_json::to_value(entry).unwrap_or_default())),
        Err(e) => {
            error!("Failed to reject absence entry: {}", e);
            Err(map_error(e))
        }
    }
}

/// Cancel an entry
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelEntryRequest {
    pub reason: Option<String>,
}

pub async fn cancel_entry(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<CancelEntryRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.absence_engine.cancel_entry(org_id, id, payload.reason.as_deref()).await {
        Ok(entry) => Ok(Json(serde_json::to_value(entry).unwrap_or_default())),
        Err(e) => {
            error!("Failed to cancel absence entry: {}", e);
            Err(map_error(e))
        }
    }
}

// ============================================================================
// Absence Balance Handlers
// ============================================================================

/// Get or create balance for an employee/plan
#[derive(Debug, Deserialize)]
pub struct GetBalanceRequest {
    pub employee_id: Uuid,
    pub plan_code: String,
}

pub async fn get_balance(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<GetBalanceRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Look up the plan
    let plan = state.absence_engine.get_absence_plan(org_id, &params.plan_code).await
        .map_err(|e| {
            error!("Failed to get absence plan: {}", e);
            map_error(e)
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Calculate current period based on frequency
    let (period_start, period_end) = state.absence_engine.calculate_current_period(&plan.accrual_frequency);

    match state.absence_engine.get_or_create_balance(
        org_id, params.employee_id, plan.id, period_start, period_end,
    ).await {
        Ok(balance) => Ok(Json(serde_json::to_value(balance).unwrap_or_default())),
        Err(e) => {
            error!("Failed to get balance: {}", e);
            Err(map_error(e))
        }
    }
}

/// List balances for an employee
#[derive(Debug, Deserialize)]
pub struct ListBalancesQuery {
    pub employee_id: Uuid,
}

pub async fn list_balances(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<ListBalancesQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.absence_engine.list_balances(org_id, params.employee_id).await {
        Ok(balances) => Ok(Json(serde_json::json!({ "data": balances }))),
        Err(e) => {
            error!("Failed to list balances: {}", e);
            Err(map_error(e))
        }
    }
}

// ============================================================================
// Entry History
// ============================================================================

/// Get history for an absence entry
pub async fn get_entry_history(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.absence_engine.get_entry_history(org_id, id).await {
        Ok(history) => Ok(Json(serde_json::json!({ "data": history }))),
        Err(e) => {
            error!("Failed to get entry history: {}", e);
            Err(map_error(e))
        }
    }
}

// ============================================================================
// Dashboard
// ============================================================================

/// Get absence management dashboard
pub async fn get_absence_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.absence_engine.get_dashboard(org_id).await {
        Ok(dashboard) => Ok(Json(serde_json::to_value(dashboard).unwrap_or_default())),
        Err(e) => {
            error!("Failed to get absence dashboard: {}", e);
            Err(map_error(e))
        }
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


