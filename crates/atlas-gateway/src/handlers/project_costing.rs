//! Project Costing API Handlers
//!
//! REST endpoints for Oracle Fusion-inspired Project Costing.

use axum::{
    extract::{Path, Query, State, Extension},
    Json,
};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;
use tracing::error;

use crate::AppState;
use crate::handlers::auth::Claims;

// ============================================================================
// Request Types
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCostTransactionRequest {
    pub project_id: Uuid,
    pub project_number: Option<String>,
    pub task_id: Option<Uuid>,
    pub task_number: Option<String>,
    pub cost_type: String,
    pub raw_cost_amount: String,
    pub currency_code: Option<String>,
    pub transaction_date: chrono::NaiveDate,
    pub gl_date: Option<chrono::NaiveDate>,
    pub description: Option<String>,
    pub supplier_id: Option<Uuid>,
    pub supplier_name: Option<String>,
    pub employee_id: Option<Uuid>,
    pub employee_name: Option<String>,
    pub expenditure_category: Option<String>,
    pub quantity: Option<String>,
    pub unit_of_measure: Option<String>,
    pub unit_rate: Option<String>,
    pub is_billable: Option<bool>,
    pub is_capitalizable: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateBurdenScheduleRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub effective_from: chrono::NaiveDate,
    pub effective_to: Option<chrono::NaiveDate>,
    pub is_default: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddBurdenLineRequest {
    pub cost_type: String,
    pub expenditure_category: Option<String>,
    pub burden_rate_percent: String,
    pub burden_account_code: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCostAdjustmentRequest {
    pub original_transaction_id: Uuid,
    pub adjustment_type: String,
    pub adjustment_amount: String,
    pub reason: String,
    pub description: Option<String>,
    pub effective_date: chrono::NaiveDate,
    pub transfer_to_project_id: Option<Uuid>,
    pub transfer_to_task_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DistributeCostRequest {
    pub raw_cost_account: String,
    pub burden_account: String,
    pub ap_ar_account: String,
}

#[derive(Debug, Deserialize)]
pub struct CostTransactionFilters {
    pub project_id: Option<Uuid>,
    pub cost_type: Option<String>,
    pub status: Option<String>,
}

fn error_response(e: atlas_shared::AtlasError) -> (axum::http::StatusCode, Json<serde_json::Value>) {
    let status = axum::http::StatusCode::from_u16(e.status_code()).unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
    (status, Json(json!({"error": e.to_string()})))
}

// ============================================================================
// Cost Transaction Handlers
// ============================================================================

pub async fn create_cost_transaction(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreateCostTransactionRequest>,
) -> Result<(axum::http::StatusCode, Json<serde_json::Value>), (axum::http::StatusCode, Json<serde_json::Value>)> {
    let org_id = Uuid::parse_str(&claims.org_id).unwrap_or_default();
    let created_by = Uuid::parse_str(&claims.sub).ok();

    match state.project_costing_engine.create_cost_transaction(
        org_id,
        req.project_id,
        req.project_number.as_deref(),
        req.task_id,
        req.task_number.as_deref(),
        &req.cost_type,
        &req.raw_cost_amount,
        req.currency_code.as_deref().unwrap_or("USD"),
        req.transaction_date,
        req.gl_date,
        req.description.as_deref(),
        req.supplier_id,
        req.supplier_name.as_deref(),
        req.employee_id,
        req.employee_name.as_deref(),
        req.expenditure_category.as_deref(),
        req.quantity.as_deref(),
        req.unit_of_measure.as_deref(),
        req.unit_rate.as_deref(),
        req.is_billable.unwrap_or(false),
        req.is_capitalizable.unwrap_or(false),
        created_by,
    ).await {
        Ok(txn) => Ok((axum::http::StatusCode::CREATED, Json(serde_json::to_value(txn).unwrap()))),
        Err(e) => {
            error!("Failed to create cost transaction: {}", e);
            Err(error_response(e))
        }
    }
}

pub async fn list_cost_transactions(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(filters): Query<CostTransactionFilters>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let org_id = Uuid::parse_str(&claims.org_id).unwrap_or_default();

    match state.project_costing_engine.list_cost_transactions(
        org_id,
        filters.project_id,
        filters.cost_type.as_deref(),
        filters.status.as_deref(),
    ).await {
        Ok(transactions) => Ok(Json(json!({"data": transactions}))),
        Err(e) => Err(error_response(e)),
    }
}

pub async fn get_cost_transaction(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    match state.project_costing_engine.get_cost_transaction(id).await {
        Ok(Some(txn)) => Ok(Json(serde_json::to_value(txn).unwrap())),
        Ok(None) => Err((axum::http::StatusCode::NOT_FOUND, Json(json!({"error": "Cost transaction not found"})))),
        Err(e) => Err(error_response(e)),
    }
}

pub async fn approve_cost_transaction(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let approved_by = Uuid::parse_str(&claims.sub).ok();

    match state.project_costing_engine.approve_cost_transaction(id, approved_by).await {
        Ok(txn) => Ok(Json(serde_json::to_value(txn).unwrap())),
        Err(e) => {
            error!("Failed to approve cost transaction: {}", e);
            Err(error_response(e))
        }
    }
}

pub async fn reverse_cost_transaction(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let org_id = Uuid::parse_str(&claims.org_id).unwrap_or_default();
    let created_by = Uuid::parse_str(&claims.sub).ok();

    let reason = body.get("reason").and_then(|v| v.as_str());
    match state.project_costing_engine.reverse_cost_transaction(org_id, id, reason, created_by).await {
        Ok(txn) => Ok(Json(serde_json::to_value(txn).unwrap())),
        Err(e) => {
            error!("Failed to reverse cost transaction: {}", e);
            Err(error_response(e))
        }
    }
}

// ============================================================================
// Burden Schedule Handlers
// ============================================================================

pub async fn create_burden_schedule(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreateBurdenScheduleRequest>,
) -> Result<(axum::http::StatusCode, Json<serde_json::Value>), (axum::http::StatusCode, Json<serde_json::Value>)> {
    let org_id = Uuid::parse_str(&claims.org_id).unwrap_or_default();
    let created_by = Uuid::parse_str(&claims.sub).ok();

    match state.project_costing_engine.create_burden_schedule(
        org_id, &req.code, &req.name, req.description.as_deref(),
        req.effective_from, req.effective_to,
        req.is_default.unwrap_or(false), created_by,
    ).await {
        Ok(schedule) => Ok((axum::http::StatusCode::CREATED, Json(serde_json::to_value(schedule).unwrap()))),
        Err(e) => {
            error!("Failed to create burden schedule: {}", e);
            Err(error_response(e))
        }
    }
}

pub async fn list_burden_schedules(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let org_id = Uuid::parse_str(&claims.org_id).unwrap_or_default();

    match state.project_costing_engine.list_burden_schedules(org_id).await {
        Ok(schedules) => Ok(Json(json!({"data": schedules}))),
        Err(e) => Err(error_response(e)),
    }
}

pub async fn get_burden_schedule(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(code): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let org_id = Uuid::parse_str(&claims.org_id).unwrap_or_default();

    match state.project_costing_engine.get_burden_schedule(org_id, &code).await {
        Ok(Some(schedule)) => Ok(Json(serde_json::to_value(schedule).unwrap())),
        Ok(None) => Err((axum::http::StatusCode::NOT_FOUND, Json(json!({"error": "Burden schedule not found"})))),
        Err(e) => Err(error_response(e)),
    }
}

pub async fn activate_burden_schedule(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    match state.project_costing_engine.activate_burden_schedule(id).await {
        Ok(schedule) => Ok(Json(serde_json::to_value(schedule).unwrap())),
        Err(e) => Err(error_response(e)),
    }
}

pub async fn add_burden_schedule_line(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(schedule_id): Path<Uuid>,
    Json(req): Json<AddBurdenLineRequest>,
) -> Result<(axum::http::StatusCode, Json<serde_json::Value>), (axum::http::StatusCode, Json<serde_json::Value>)> {
    let org_id = Uuid::parse_str(&claims.org_id).unwrap_or_default();

    match state.project_costing_engine.add_burden_schedule_line(
        org_id, schedule_id, &req.cost_type,
        req.expenditure_category.as_deref(), &req.burden_rate_percent,
        req.burden_account_code.as_deref(),
    ).await {
        Ok(line) => Ok((axum::http::StatusCode::CREATED, Json(serde_json::to_value(line).unwrap()))),
        Err(e) => {
            error!("Failed to add burden line: {}", e);
            Err(error_response(e))
        }
    }
}

pub async fn list_burden_schedule_lines(
    State(state): State<Arc<AppState>>,
    Path(schedule_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    match state.project_costing_engine.list_burden_schedule_lines(schedule_id).await {
        Ok(lines) => Ok(Json(json!({"data": lines}))),
        Err(e) => Err(error_response(e)),
    }
}

// ============================================================================
// Cost Adjustment Handlers
// ============================================================================

pub async fn create_cost_adjustment(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreateCostAdjustmentRequest>,
) -> Result<(axum::http::StatusCode, Json<serde_json::Value>), (axum::http::StatusCode, Json<serde_json::Value>)> {
    let org_id = Uuid::parse_str(&claims.org_id).unwrap_or_default();
    let created_by = Uuid::parse_str(&claims.sub).ok();

    match state.project_costing_engine.create_cost_adjustment(
        org_id, req.original_transaction_id, &req.adjustment_type,
        &req.adjustment_amount, &req.reason, req.description.as_deref(),
        req.effective_date, req.transfer_to_project_id, req.transfer_to_task_id,
        created_by,
    ).await {
        Ok(adj) => Ok((axum::http::StatusCode::CREATED, Json(serde_json::to_value(adj).unwrap()))),
        Err(e) => {
            error!("Failed to create cost adjustment: {}", e);
            Err(error_response(e))
        }
    }
}

pub async fn list_cost_adjustments(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let org_id = Uuid::parse_str(&claims.org_id).unwrap_or_default();

    let status = params.get("status").map(|s| s.as_str());
    match state.project_costing_engine.list_cost_adjustments(org_id, status).await {
        Ok(adjustments) => Ok(Json(json!({"data": adjustments}))),
        Err(e) => Err(error_response(e)),
    }
}

pub async fn approve_cost_adjustment(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let approved_by = Uuid::parse_str(&claims.sub).ok();

    match state.project_costing_engine.approve_cost_adjustment(id, approved_by).await {
        Ok(adj) => Ok(Json(serde_json::to_value(adj).unwrap())),
        Err(e) => {
            error!("Failed to approve cost adjustment: {}", e);
            Err(error_response(e))
        }
    }
}

// ============================================================================
// Cost Distribution Handlers
// ============================================================================

pub async fn distribute_cost_transaction(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(req): Json<DistributeCostRequest>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let org_id = Uuid::parse_str(&claims.org_id).unwrap_or_default();

    match state.project_costing_engine.distribute_cost_transaction(
        org_id, id, &req.raw_cost_account, &req.burden_account, &req.ap_ar_account,
    ).await {
        Ok(distributions) => Ok(Json(json!({"data": distributions}))),
        Err(e) => {
            error!("Failed to distribute cost: {}", e);
            Err(error_response(e))
        }
    }
}

pub async fn list_cost_distributions(
    State(state): State<Arc<AppState>>,
    Path(transaction_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    match state.project_costing_engine.list_cost_distributions(transaction_id).await {
        Ok(distributions) => Ok(Json(json!({"data": distributions}))),
        Err(e) => Err(error_response(e)),
    }
}

pub async fn post_distributions(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let org_id = Uuid::parse_str(&claims.org_id).unwrap_or_default();

    let gl_batch_id = body.get("gl_batch_id").and_then(|v| v.as_str()).and_then(|s| s.parse::<Uuid>().ok());

    match state.project_costing_engine.post_distributions(org_id, gl_batch_id).await {
        Ok(count) => Ok(Json(json!({"posted_count": count}))),
        Err(e) => Err(error_response(e)),
    }
}

// ============================================================================
// Dashboard
// ============================================================================

pub async fn get_costing_summary(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let org_id = Uuid::parse_str(&claims.org_id).unwrap_or_default();

    match state.project_costing_engine.get_costing_summary(org_id).await {
        Ok(summary) => Ok(Json(serde_json::to_value(summary).unwrap())),
        Err(e) => Err(error_response(e)),
    }
}
