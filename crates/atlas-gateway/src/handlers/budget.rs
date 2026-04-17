//! Budget Management Handlers
//!
//! Oracle Fusion Cloud ERP: General Ledger > Budgets
//!
//! API endpoints for managing budget definitions, budget versions with
//! approval workflow, budget lines, budget transfers, and variance reporting.

use axum::{
    extract::{State, Path},
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
// Budget Definition Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateBudgetDefinitionRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub calendar_id: Option<Uuid>,
    pub fiscal_year: Option<i32>,
    #[serde(default = "default_budget_type")]
    pub budget_type: String,
    #[serde(default = "default_control_level")]
    pub control_level: String,
    #[serde(default)]
    pub allow_carry_forward: bool,
    #[serde(default = "default_true_fn")]
    pub allow_transfers: bool,
    #[serde(default = "default_usd")]
    pub currency_code: String,
}

fn default_budget_type() -> String { "operating".to_string() }
fn default_control_level() -> String { "none".to_string() }
fn default_true_fn() -> bool { true }
fn default_usd() -> String { "USD".to_string() }

/// Create or update a budget definition
pub async fn create_budget_definition(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateBudgetDefinitionRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.budget_engine.create_definition(
        org_id,
        &payload.code,
        &payload.name,
        payload.description.as_deref(),
        payload.calendar_id,
        payload.fiscal_year,
        &payload.budget_type,
        &payload.control_level,
        payload.allow_carry_forward,
        payload.allow_transfers,
        &payload.currency_code,
        Some(user_id),
    ).await {
        Ok(def) => {
            info!("Created budget definition '{}' for org {}", def.code, org_id);
            Ok((StatusCode::CREATED, Json(serde_json::to_value(def).unwrap())))
        }
        Err(e) => {
            error!("Failed to create budget definition: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

/// Get a budget definition by code
pub async fn get_budget_definition(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(code): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.budget_engine.get_definition(org_id, &code).await {
        Ok(Some(def)) => Ok(Json(serde_json::to_value(def).unwrap())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get budget definition: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// List all budget definitions
pub async fn list_budget_definitions(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.budget_engine.list_definitions(org_id).await {
        Ok(defs) => Ok(Json(serde_json::json!({ "data": defs }))),
        Err(e) => {
            error!("Failed to list budget definitions: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Delete (deactivate) a budget definition
pub async fn delete_budget_definition(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(code): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.budget_engine.delete_definition(org_id, &code).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            error!("Failed to delete budget definition: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// ============================================================================
// Budget Version Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateBudgetVersionRequest {
    pub label: Option<String>,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub notes: Option<String>,
}

/// Create a new budget version
pub async fn create_budget_version(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(budget_code): Path<String>,
    Json(payload): Json<CreateBudgetVersionRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.budget_engine.create_version(
        org_id,
        &budget_code,
        payload.label.as_deref(),
        payload.effective_from,
        payload.effective_to,
        payload.notes.as_deref(),
        Some(user_id),
    ).await {
        Ok(version) => {
            info!("Created budget version for '{}' (v{})", budget_code, version.version_number);
            Ok((StatusCode::CREATED, Json(serde_json::to_value(version).unwrap())))
        }
        Err(e) => {
            error!("Failed to create budget version: {}", e);
            Err(match e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

/// List budget versions for a definition
pub async fn list_budget_versions(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(budget_code): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let definition = state.budget_engine.get_definition(org_id, &budget_code).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    match state.budget_engine.list_versions(definition.id).await {
        Ok(versions) => Ok(Json(serde_json::json!({ "data": versions }))),
        Err(e) => {
            error!("Failed to list budget versions: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get a budget version by ID
pub async fn get_budget_version(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(version_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.budget_engine.get_version(version_id).await {
        Ok(Some(version)) => Ok(Json(serde_json::to_value(version).unwrap())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get budget version: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// ============================================================================
// Budget Version Workflow Handlers
// ============================================================================

/// Submit a budget version for approval
pub async fn submit_budget_version(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(version_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.budget_engine.submit_version(version_id, user_id).await {
        Ok(version) => Ok(Json(serde_json::to_value(version).unwrap())),
        Err(e) => {
            error!("Failed to submit budget version: {}", e);
            Err(match e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::CONFLICT,
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

/// Approve a budget version
pub async fn approve_budget_version(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(version_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.budget_engine.approve_version(version_id, user_id).await {
        Ok(version) => Ok(Json(serde_json::to_value(version).unwrap())),
        Err(e) => {
            error!("Failed to approve budget version: {}", e);
            Err(match e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

/// Activate an approved budget version
pub async fn activate_budget_version(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(version_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.budget_engine.activate_version(version_id).await {
        Ok(version) => Ok(Json(serde_json::to_value(version).unwrap())),
        Err(e) => {
            error!("Failed to activate budget version: {}", e);
            Err(match e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

/// Reject a budget version
#[derive(Debug, Deserialize)]
pub struct RejectBudgetRequest {
    pub reason: Option<String>,
}

pub async fn reject_budget_version(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(version_id): Path<Uuid>,
    Json(payload): Json<RejectBudgetRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.budget_engine.reject_version(version_id, payload.reason.as_deref()).await {
        Ok(version) => Ok(Json(serde_json::to_value(version).unwrap())),
        Err(e) => {
            error!("Failed to reject budget version: {}", e);
            Err(match e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

/// Close an active budget version
pub async fn close_budget_version(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(version_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.budget_engine.close_version(version_id).await {
        Ok(version) => Ok(Json(serde_json::to_value(version).unwrap())),
        Err(e) => {
            error!("Failed to close budget version: {}", e);
            Err(match e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// ============================================================================
// Budget Line Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateBudgetLineRequest {
    pub account_code: String,
    pub account_name: Option<String>,
    pub period_name: Option<String>,
    pub period_start_date: Option<chrono::NaiveDate>,
    pub period_end_date: Option<chrono::NaiveDate>,
    pub fiscal_year: Option<i32>,
    pub quarter: Option<i32>,
    pub department_id: Option<Uuid>,
    pub department_name: Option<String>,
    pub project_id: Option<Uuid>,
    pub project_name: Option<String>,
    pub cost_center: Option<String>,
    pub budget_amount: String,
    pub description: Option<String>,
}

/// Add a budget line to a version
pub async fn add_budget_line(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(version_id): Path<Uuid>,
    Json(payload): Json<CreateBudgetLineRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.budget_engine.add_line(
        org_id,
        version_id,
        &payload.account_code,
        payload.account_name.as_deref(),
        payload.period_name.as_deref(),
        payload.period_start_date,
        payload.period_end_date,
        payload.fiscal_year,
        payload.quarter,
        payload.department_id,
        payload.department_name.as_deref(),
        payload.project_id,
        payload.project_name.as_deref(),
        payload.cost_center.as_deref(),
        &payload.budget_amount,
        payload.description.as_deref(),
        Some(user_id),
    ).await {
        Ok(line) => {
            info!("Added budget line to version {} for account {}", version_id, payload.account_code);
            Ok((StatusCode::CREATED, Json(serde_json::to_value(line).unwrap())))
        }
        Err(e) => {
            error!("Failed to add budget line: {}", e);
            Err(match e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::CONFLICT,
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

/// List budget lines for a version
pub async fn list_budget_lines(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(version_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.budget_engine.list_lines(version_id).await {
        Ok(lines) => Ok(Json(serde_json::json!({ "data": lines }))),
        Err(e) => {
            error!("Failed to list budget lines: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Delete a budget line
pub async fn delete_budget_line(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path((version_id, line_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, StatusCode> {
    match state.budget_engine.delete_line(version_id, line_id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            error!("Failed to delete budget line: {}", e);
            Err(match e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// ============================================================================
// Budget Transfer Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateBudgetTransferRequest {
    pub from_account_code: String,
    pub from_period_name: Option<String>,
    pub from_department_id: Option<Uuid>,
    pub from_cost_center: Option<String>,
    pub to_account_code: String,
    pub to_period_name: Option<String>,
    pub to_department_id: Option<Uuid>,
    pub to_cost_center: Option<String>,
    pub amount: String,
    pub description: Option<String>,
}

/// Create a budget transfer
pub async fn create_budget_transfer(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(version_id): Path<Uuid>,
    Json(payload): Json<CreateBudgetTransferRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let transfer_number = format!("BTR-{}", uuid::Uuid::new_v4().to_string()[..8].to_uppercase());

    match state.budget_engine.create_transfer(
        org_id,
        version_id,
        &transfer_number,
        payload.description.as_deref(),
        &payload.from_account_code,
        payload.from_period_name.as_deref(),
        payload.from_department_id,
        payload.from_cost_center.as_deref(),
        &payload.to_account_code,
        payload.to_period_name.as_deref(),
        payload.to_department_id,
        payload.to_cost_center.as_deref(),
        &payload.amount,
        Some(user_id),
    ).await {
        Ok(transfer) => {
            info!("Created budget transfer {} for version {}", transfer_number, version_id);
            Ok((StatusCode::CREATED, Json(serde_json::to_value(transfer).unwrap())))
        }
        Err(e) => {
            error!("Failed to create budget transfer: {}", e);
            Err(match e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::CONFLICT,
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

/// Approve a budget transfer
pub async fn approve_budget_transfer(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(transfer_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.budget_engine.approve_transfer(transfer_id, user_id).await {
        Ok(transfer) => Ok(Json(serde_json::to_value(transfer).unwrap())),
        Err(e) => {
            error!("Failed to approve budget transfer: {}", e);
            Err(match e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

/// Reject a budget transfer
pub async fn reject_budget_transfer(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(transfer_id): Path<Uuid>,
    Json(payload): Json<RejectBudgetRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.budget_engine.reject_transfer(transfer_id, payload.reason.as_deref()).await {
        Ok(transfer) => Ok(Json(serde_json::to_value(transfer).unwrap())),
        Err(e) => {
            error!("Failed to reject budget transfer: {}", e);
            Err(match e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

/// List budget transfers for a version
pub async fn list_budget_transfers(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(version_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.budget_engine.list_transfers(version_id).await {
        Ok(transfers) => Ok(Json(serde_json::json!({ "data": transfers }))),
        Err(e) => {
            error!("Failed to list budget transfers: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// ============================================================================
// Budget Variance Report Handler
// ============================================================================

/// Get budget vs actuals variance report
pub async fn get_budget_variance(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(version_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.budget_engine.get_variance_report(version_id).await {
        Ok(report) => Ok(Json(serde_json::to_value(report).unwrap())),
        Err(e) => {
            error!("Failed to generate budget variance report: {}", e);
            Err(match e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// ============================================================================
// Budget Control Check Handler
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CheckBudgetControlRequest {
    pub account_code: String,
    pub period_name: Option<String>,
    pub department_id: Option<Uuid>,
    pub cost_center: Option<String>,
    pub proposed_amount: f64,
}

/// Check if a proposed amount is within budget
pub async fn check_budget_control(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(budget_code): Path<String>,
    Json(payload): Json<CheckBudgetControlRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.budget_engine.check_budget_control(
        org_id,
        &budget_code,
        &payload.account_code,
        payload.period_name.as_deref(),
        payload.department_id.as_ref(),
        payload.cost_center.as_deref(),
        payload.proposed_amount,
    ).await {
        Ok(result) => Ok(Json(serde_json::to_value(result).unwrap())),
        Err(e) => {
            error!("Failed to check budget control: {}", e);
            Err(match e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}
