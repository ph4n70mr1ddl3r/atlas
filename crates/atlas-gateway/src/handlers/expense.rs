//! Expense Management Handlers
//!
//! Oracle Fusion Cloud ERP: Expenses > Expense Reports, Categories, Policies
//!
//! API endpoints for managing expense categories, policies, expense reports
//! with line items, and reimbursement workflow.

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
// Expense Category Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateExpenseCategoryRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub receipt_required: bool,
    pub receipt_threshold: Option<String>,
    #[serde(default)]
    pub is_per_diem: bool,
    pub default_per_diem_rate: Option<String>,
    #[serde(default)]
    pub is_mileage: bool,
    pub default_mileage_rate: Option<String>,
    pub expense_account_code: Option<String>,
}

/// Create or update an expense category
pub async fn create_expense_category(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateExpenseCategoryRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    info!("Creating expense category {} for org {} by user {}", payload.code, org_id, user_id);

    match state.expense_engine.create_category(
        org_id,
        &payload.code,
        &payload.name,
        payload.description.as_deref(),
        payload.receipt_required,
        payload.receipt_threshold.as_deref(),
        payload.is_per_diem,
        payload.default_per_diem_rate.as_deref(),
        payload.is_mileage,
        payload.default_mileage_rate.as_deref(),
        payload.expense_account_code.as_deref(),
        Some(user_id),
    ).await {
        Ok(category) => Ok((StatusCode::CREATED, Json(serde_json::to_value(category).unwrap()))),
        Err(e) => {
            error!("Failed to create expense category: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                409 => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

/// Get an expense category by code
pub async fn get_expense_category(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(code): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.expense_engine.get_category(org_id, &code).await {
        Ok(Some(category)) => Ok(Json(serde_json::to_value(category).unwrap())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get expense category: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// List all expense categories
pub async fn list_expense_categories(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.expense_engine.list_categories(org_id).await {
        Ok(categories) => Ok(Json(serde_json::json!({ "data": categories }))),
        Err(e) => {
            error!("Failed to list expense categories: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Delete (deactivate) an expense category
pub async fn delete_expense_category(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(code): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.expense_engine.delete_category(org_id, &code).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            error!("Failed to delete expense category: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// ============================================================================
// Expense Policy Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateExpensePolicyRequest {
    pub name: String,
    pub description: Option<String>,
    pub category_code: Option<String>,
    pub min_amount: Option<String>,
    pub max_amount: Option<String>,
    pub daily_limit: Option<String>,
    pub report_limit: Option<String>,
    #[serde(default)]
    pub requires_approval_on_violation: bool,
    #[serde(default = "default_violation_action")]
    pub violation_action: String,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
}

fn default_violation_action() -> String { "warn".to_string() }

/// Create an expense policy
pub async fn create_expense_policy(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateExpensePolicyRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    info!("Creating expense policy '{}' for org {}", payload.name, org_id);

    match state.expense_engine.create_policy(
        org_id,
        &payload.name,
        payload.description.as_deref(),
        payload.category_code.as_deref(),
        payload.min_amount.as_deref(),
        payload.max_amount.as_deref(),
        payload.daily_limit.as_deref(),
        payload.report_limit.as_deref(),
        payload.requires_approval_on_violation,
        &payload.violation_action,
        payload.effective_from,
        payload.effective_to,
        Some(user_id),
    ).await {
        Ok(policy) => Ok((StatusCode::CREATED, Json(serde_json::to_value(policy).unwrap()))),
        Err(e) => {
            error!("Failed to create expense policy: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListPoliciesQuery {
    pub category_code: Option<String>,
}

/// List expense policies
pub async fn list_expense_policies(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListPoliciesQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.expense_engine.list_policies(org_id, query.category_code.as_deref()).await {
        Ok(policies) => Ok(Json(serde_json::json!({ "data": policies }))),
        Err(e) => {
            error!("Failed to list expense policies: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Delete an expense policy
pub async fn delete_expense_policy(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    match state.expense_engine.delete_policy(id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            error!("Failed to delete expense policy: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// ============================================================================
// Expense Report Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateExpenseReportRequest {
    pub report_number: String,
    pub title: String,
    pub description: Option<String>,
    pub employee_id: Uuid,
    pub employee_name: Option<String>,
    pub department_id: Option<Uuid>,
    pub purpose: Option<String>,
    pub project_id: Option<Uuid>,
    #[serde(default = "default_currency_usd")]
    pub currency_code: String,
    pub trip_start_date: Option<chrono::NaiveDate>,
    pub trip_end_date: Option<chrono::NaiveDate>,
    pub cost_center: Option<String>,
}

fn default_currency_usd() -> String { "USD".to_string() }

/// Create a new expense report
pub async fn create_expense_report(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateExpenseReportRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    info!("Creating expense report '{}' for org {}", payload.report_number, org_id);

    match state.expense_engine.create_report(
        org_id,
        &payload.report_number,
        &payload.title,
        payload.description.as_deref(),
        payload.employee_id,
        payload.employee_name.as_deref(),
        payload.department_id,
        payload.purpose.as_deref(),
        payload.project_id,
        &payload.currency_code,
        payload.trip_start_date,
        payload.trip_end_date,
        payload.cost_center.as_deref(),
        Some(user_id),
    ).await {
        Ok(report) => Ok((StatusCode::CREATED, Json(serde_json::to_value(report).unwrap()))),
        Err(e) => {
            error!("Failed to create expense report: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

/// Get an expense report by ID
pub async fn get_expense_report(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.expense_engine.get_report(id).await {
        Ok(Some(report)) => Ok(Json(serde_json::to_value(report).unwrap())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get expense report: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListReportsQuery {
    pub employee_id: Option<Uuid>,
    pub status: Option<String>,
}

/// List expense reports with optional filters
pub async fn list_expense_reports(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListReportsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.expense_engine.list_reports(org_id, query.employee_id, query.status.as_deref()).await {
        Ok(reports) => Ok(Json(serde_json::json!({ "data": reports }))),
        Err(e) => {
            error!("Failed to list expense reports: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

/// Submit an expense report for approval
pub async fn submit_expense_report(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.expense_engine.submit_report(id).await {
        Ok(report) => Ok(Json(serde_json::to_value(report).unwrap())),
        Err(e) => {
            error!("Failed to submit expense report: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

/// Approve an expense report
pub async fn approve_expense_report(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.expense_engine.approve_report(id, user_id).await {
        Ok(report) => Ok(Json(serde_json::to_value(report).unwrap())),
        Err(e) => {
            error!("Failed to approve expense report: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct RejectExpenseReportRequest {
    pub reason: Option<String>,
}

/// Reject an expense report
pub async fn reject_expense_report(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<RejectExpenseReportRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.expense_engine.reject_report(id, user_id, payload.reason.as_deref()).await {
        Ok(report) => Ok(Json(serde_json::to_value(report).unwrap())),
        Err(e) => {
            error!("Failed to reject expense report: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

/// Mark expense report as reimbursed
pub async fn reimburse_expense_report(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.expense_engine.reimburse_report(id).await {
        Ok(report) => Ok(Json(serde_json::to_value(report).unwrap())),
        Err(e) => {
            error!("Failed to reimburse expense report: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// ============================================================================
// Expense Line Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct AddExpenseLineRequest {
    pub expense_type: String,
    pub expense_date: chrono::NaiveDate,
    pub amount: String,
    pub description: Option<String>,
    pub category_code: Option<String>,
    pub original_currency: Option<String>,
    pub original_amount: Option<String>,
    pub exchange_rate: Option<String>,
    pub is_reimbursable: Option<bool>,
    pub has_receipt: Option<bool>,
    pub receipt_reference: Option<String>,
    pub merchant_name: Option<String>,
    pub location: Option<String>,
    pub attendees: Option<serde_json::Value>,
    // Per-diem fields
    pub per_diem_days: Option<f64>,
    pub per_diem_rate: Option<String>,
    // Mileage fields
    pub mileage_distance: Option<f64>,
    pub mileage_rate: Option<String>,
    pub mileage_unit: Option<String>,
    pub mileage_from: Option<String>,
    pub mileage_to: Option<String>,
}

/// Add an expense line to a report
pub async fn add_expense_line(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(report_id): Path<Uuid>,
    Json(payload): Json<AddExpenseLineRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    info!("Adding expense line to report {}", report_id);

    match state.expense_engine.add_line(
        org_id,
        report_id,
        &payload.expense_type,
        payload.expense_date,
        &payload.amount,
        payload.description.as_deref(),
        payload.category_code.as_deref(),
        payload.original_currency.as_deref(),
        payload.original_amount.as_deref(),
        payload.exchange_rate.as_deref(),
        payload.is_reimbursable,
        payload.has_receipt,
        payload.receipt_reference.as_deref(),
        payload.merchant_name.as_deref(),
        payload.location.as_deref(),
        payload.attendees,
        payload.per_diem_days,
        payload.per_diem_rate.as_deref(),
        payload.mileage_distance,
        payload.mileage_rate.as_deref(),
        payload.mileage_unit.as_deref(),
        payload.mileage_from.as_deref(),
        payload.mileage_to.as_deref(),
        Some(user_id),
    ).await {
        Ok(line) => Ok((StatusCode::CREATED, Json(serde_json::to_value(line).unwrap()))),
        Err(e) => {
            error!("Failed to add expense line: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

/// List expense lines for a report
pub async fn list_expense_lines(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(report_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.expense_engine.list_lines(report_id).await {
        Ok(lines) => Ok(Json(serde_json::json!({ "data": lines }))),
        Err(e) => {
            error!("Failed to list expense lines: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Delete an expense line
pub async fn delete_expense_line(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path((report_id, line_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, StatusCode> {
    match state.expense_engine.delete_line(report_id, line_id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            error!("Failed to delete expense line: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}
