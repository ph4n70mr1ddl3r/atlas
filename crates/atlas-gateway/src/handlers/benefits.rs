//! Benefits Administration Handlers
//!
//! Oracle Fusion Cloud HCM: Benefits > Benefits Plans, Enrollments, Coverage
//!
//! API endpoints for managing benefits plans, employee enrollments,
//! coverage tiers, and payroll deductions.

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
// Benefits Plan Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateBenefitsPlanRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub plan_type: String,
    pub coverage_tiers: serde_json::Value,
    pub provider_name: Option<String>,
    pub provider_plan_id: Option<String>,
    pub plan_year_start: Option<chrono::NaiveDate>,
    pub plan_year_end: Option<chrono::NaiveDate>,
    pub open_enrollment_start: Option<chrono::NaiveDate>,
    pub open_enrollment_end: Option<chrono::NaiveDate>,
    #[serde(default = "default_true")]
    pub allow_life_event_changes: bool,
    #[serde(default)]
    pub requires_eoi: bool,
    #[serde(default)]
    pub waiting_period_days: i32,
    pub max_dependents: Option<i32>,
}

fn default_true() -> bool { true }

/// Create or update a benefits plan
pub async fn create_benefits_plan(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateBenefitsPlanRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    info!("Creating benefits plan {} for org {} by user {}", payload.code, org_id, user_id);

    match state.benefits_engine.create_plan(
        org_id, &payload.code, &payload.name, payload.description.as_deref(),
        &payload.plan_type, payload.coverage_tiers,
        payload.provider_name.as_deref(), payload.provider_plan_id.as_deref(),
        payload.plan_year_start, payload.plan_year_end,
        payload.open_enrollment_start, payload.open_enrollment_end,
        payload.allow_life_event_changes, payload.requires_eoi,
        payload.waiting_period_days, payload.max_dependents,
        Some(user_id),
    ).await {
        Ok(plan) => Ok((StatusCode::CREATED, Json(serde_json::to_value(plan).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => {
            error!("Failed to create benefits plan: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                409 => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

/// Get a benefits plan by code
pub async fn get_benefits_plan(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(code): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.benefits_engine.get_plan(org_id, &code).await {
        Ok(Some(plan)) => Ok(Json(serde_json::to_value(plan).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get benefits plan: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListPlansQuery {
    pub plan_type: Option<String>,
}

/// List all benefits plans
pub async fn list_benefits_plans(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListPlansQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.benefits_engine.list_plans(org_id, query.plan_type.as_deref()).await {
        Ok(plans) => Ok(Json(serde_json::json!({"data": plans}))),
        Err(e) => {
            error!("Failed to list benefits plans: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Delete (deactivate) a benefits plan
pub async fn delete_benefits_plan(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(code): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.benefits_engine.delete_plan(org_id, &code).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            error!("Failed to delete benefits plan: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// ============================================================================
// Benefits Enrollment Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct EnrollRequest {
    pub employee_id: Uuid,
    pub employee_name: Option<String>,
    pub plan_code: String,
    pub coverage_tier: String,
    pub enrollment_type: String,
    pub effective_start_date: chrono::NaiveDate,
    pub effective_end_date: Option<chrono::NaiveDate>,
    #[serde(default = "default_deduction_frequency")]
    pub deduction_frequency: String,
    pub deduction_account_code: Option<String>,
    pub employer_contribution_account_code: Option<String>,
    pub dependents: Option<serde_json::Value>,
    pub life_event_reason: Option<String>,
    pub life_event_date: Option<chrono::NaiveDate>,
}

fn default_deduction_frequency() -> String { "per_pay_period".to_string() }

/// Enroll an employee in a benefits plan
pub async fn create_enrollment(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<EnrollRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    info!("Creating benefits enrollment for employee {} in plan {}", payload.employee_id, payload.plan_code);

    match state.benefits_engine.enroll(
        org_id, payload.employee_id, payload.employee_name.as_deref(),
        &payload.plan_code, &payload.coverage_tier, &payload.enrollment_type,
        payload.effective_start_date, payload.effective_end_date,
        &payload.deduction_frequency,
        payload.deduction_account_code.as_deref(),
        payload.employer_contribution_account_code.as_deref(),
        payload.dependents,
        payload.life_event_reason.as_deref(),
        payload.life_event_date,
        Some(user_id),
    ).await {
        Ok(enrollment) => Ok((StatusCode::CREATED, Json(serde_json::to_value(enrollment).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => {
            error!("Failed to create enrollment: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                409 => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

/// Get an enrollment by ID
pub async fn get_enrollment(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let _org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.benefits_engine.get_enrollment(id).await {
        Ok(Some(enrollment)) => Ok(Json(serde_json::to_value(enrollment).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get enrollment: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListEnrollmentsQuery {
    pub employee_id: Option<Uuid>,
    pub plan_id: Option<Uuid>,
    pub status: Option<String>,
}

/// List enrollments with optional filters
pub async fn list_enrollments(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListEnrollmentsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.benefits_engine.list_enrollments(
        org_id, query.employee_id, query.plan_id, query.status.as_deref(),
    ).await {
        Ok(enrollments) => Ok(Json(serde_json::json!({"data": enrollments}))),
        Err(e) => {
            error!("Failed to list enrollments: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Activate a pending enrollment
pub async fn activate_enrollment(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.benefits_engine.activate_enrollment(id, user_id).await {
        Ok(enrollment) => Ok(Json(serde_json::to_value(enrollment).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => {
            error!("Failed to activate enrollment: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

/// Waive a pending enrollment
pub async fn waive_enrollment(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.benefits_engine.waive_enrollment(id).await {
        Ok(enrollment) => Ok(Json(serde_json::to_value(enrollment).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => {
            error!("Failed to waive enrollment: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CancelEnrollmentRequest {
    pub cancellation_reason: Option<String>,
}

/// Cancel an enrollment
pub async fn cancel_enrollment(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<CancelEnrollmentRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.benefits_engine.cancel_enrollment(id, payload.cancellation_reason.as_deref()).await {
        Ok(enrollment) => Ok(Json(serde_json::to_value(enrollment).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => {
            error!("Failed to cancel enrollment: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

/// Suspend an active enrollment
pub async fn suspend_enrollment(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.benefits_engine.suspend_enrollment(id).await {
        Ok(enrollment) => Ok(Json(serde_json::to_value(enrollment).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => {
            error!("Failed to suspend enrollment: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

/// Reactivate a suspended enrollment
pub async fn reactivate_enrollment(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.benefits_engine.reactivate_enrollment(id).await {
        Ok(enrollment) => Ok(Json(serde_json::to_value(enrollment).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => {
            error!("Failed to reactivate enrollment: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// ============================================================================
// Benefits Deductions Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct GenerateDeductionsRequest {
    pub pay_period_start: chrono::NaiveDate,
    pub pay_period_end: chrono::NaiveDate,
}

/// Generate deductions for a pay period
pub async fn generate_deductions(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<GenerateDeductionsRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.benefits_engine.generate_deductions(
        org_id, payload.pay_period_start, payload.pay_period_end,
    ).await {
        Ok(deductions) => Ok(Json(serde_json::json!({"data": deductions, "count": deductions.len()}))),
        Err(e) => {
            error!("Failed to generate deductions: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListDeductionsQuery {
    pub employee_id: Option<Uuid>,
    pub enrollment_id: Option<Uuid>,
}

/// List deductions with optional filters
pub async fn list_deductions(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListDeductionsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.benefits_engine.list_deductions(org_id, query.employee_id, query.enrollment_id).await {
        Ok(deductions) => Ok(Json(serde_json::json!({"data": deductions}))),
        Err(e) => {
            error!("Failed to list deductions: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get benefits dashboard summary
pub async fn get_benefits_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.benefits_engine.get_summary(org_id).await {
        Ok(summary) => Ok(Json(serde_json::to_value(summary).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => {
            error!("Failed to get benefits summary: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
