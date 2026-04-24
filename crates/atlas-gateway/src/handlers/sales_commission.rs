//! Sales Commission Management Handlers
//!
//! Oracle Fusion Cloud ERP: Incentive Compensation
//! API endpoints for sales reps, commission plans, rate tiers, assignments,
//! quotas, commission transactions, payouts, and dashboard.

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

fn default_usd() -> String { "USD".to_string() }
fn default_revenue() -> String { "revenue".to_string() }
fn default_percentage() -> String { "percentage".to_string() }


/// Parse a UUID from a claim string, returning a JSON error on failure.
///
/// Unlike `unwrap_or_default()`, this does NOT silently fall back to the nil
/// UUID — which would be an auth-scoping bypass.
fn parse_uuid(s: &str) -> Result<Uuid, (axum::http::StatusCode, Json<serde_json::Value>)> {
    Uuid::parse_str(s).map_err(|_| {
        (axum::http::StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "Invalid auth token"})))
    })
}
// ============================================================================
// Sales Representatives
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateRepRequest {
    pub rep_code: String,
    pub employee_id: Option<Uuid>,
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub territory_code: Option<String>,
    pub territory_name: Option<String>,
    pub manager_id: Option<Uuid>,
    pub manager_name: Option<String>,
    pub hire_date: Option<chrono::NaiveDate>,
}

pub async fn create_rep(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreateRepRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    match state.sales_commission_engine.create_rep(
        org_id, &req.rep_code, req.employee_id,
        &req.first_name, &req.last_name,
        req.email.as_deref(), req.territory_code.as_deref(),
        req.territory_name.as_deref(), req.manager_id,
        req.manager_name.as_deref(), req.hire_date, None,
    ).await {
        Ok(rep) => Ok((StatusCode::CREATED, Json(serde_json::to_value(rep).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => {
            error!("Failed to create rep: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListRepsQuery {
    pub active_only: Option<bool>,
}

pub async fn list_reps(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<ListRepsQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    let active_only = query.active_only.unwrap_or(true);
    match state.sales_commission_engine.list_reps(org_id, active_only).await {
        Ok(reps) => Ok(Json(serde_json::json!({"data": reps}))),
        Err(e) => {
            error!("Failed to list reps: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

pub async fn get_rep(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(code): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    match state.sales_commission_engine.get_rep(org_id, &code).await {
        Ok(Some(rep)) => Ok(Json(serde_json::to_value(rep).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Rep not found"})))),
        Err(e) => {
            error!("Failed to get rep: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

pub async fn delete_rep(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(code): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    match state.sales_commission_engine.delete_rep(org_id, &code).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            error!("Failed to delete rep: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

// ============================================================================
// Commission Plans
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreatePlanRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(default = "default_revenue")]
    pub plan_type: String,
    #[serde(default = "default_revenue")]
    pub basis: String,
    #[serde(default = "default_percentage")]
    pub calculation_method: String,
    pub default_rate: String,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
}

pub async fn create_commission_plan(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreatePlanRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    match state.sales_commission_engine.create_plan(
        org_id, &req.code, &req.name, req.description.as_deref(),
        &req.plan_type, &req.basis, &req.calculation_method,
        &req.default_rate, req.effective_from, req.effective_to, None,
    ).await {
        Ok(plan) => Ok((StatusCode::CREATED, Json(serde_json::to_value(plan).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => {
            error!("Failed to create plan: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

pub async fn list_commission_plans(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    let status = params.get("status").map(|s| s.as_str());
    match state.sales_commission_engine.list_plans(org_id, status).await {
        Ok(plans) => Ok(Json(serde_json::json!({"data": plans}))),
        Err(e) => {
            error!("Failed to list plans: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

pub async fn get_commission_plan(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(code): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    match state.sales_commission_engine.get_plan(org_id, &code).await {
        Ok(Some(plan)) => Ok(Json(serde_json::to_value(plan).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Plan not found"})))),
        Err(e) => {
            error!("Failed to get plan: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

pub async fn activate_commission_plan(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.sales_commission_engine.activate_plan(id).await {
        Ok(plan) => Ok(Json(serde_json::to_value(plan).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => {
            error!("Failed to activate plan: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

pub async fn deactivate_commission_plan(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.sales_commission_engine.deactivate_plan(id).await {
        Ok(plan) => Ok(Json(serde_json::to_value(plan).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => {
            error!("Failed to deactivate plan: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

pub async fn delete_commission_plan(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(code): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    match state.sales_commission_engine.delete_plan(org_id, &code).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            error!("Failed to delete plan: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

// ============================================================================
// Commission Rate Tiers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct AddRateTierRequest {
    pub from_amount: String,
    pub to_amount: Option<String>,
    pub rate_percent: String,
    pub flat_amount: Option<String>,
}

pub async fn add_rate_tier(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(plan_id): Path<Uuid>,
    Json(req): Json<AddRateTierRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    match state.sales_commission_engine.add_rate_tier(
        org_id, plan_id,
        &req.from_amount, req.to_amount.as_deref(),
        &req.rate_percent, req.flat_amount.as_deref(),
    ).await {
        Ok(tier) => Ok((StatusCode::CREATED, Json(serde_json::to_value(tier).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => {
            error!("Failed to add rate tier: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

pub async fn list_rate_tiers(
    State(state): State<Arc<AppState>>,
    Path(plan_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.sales_commission_engine.list_rate_tiers(plan_id).await {
        Ok(tiers) => Ok(Json(serde_json::json!({"data": tiers}))),
        Err(e) => {
            error!("Failed to list rate tiers: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

// ============================================================================
// Plan Assignments
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct AssignPlanRequest {
    pub rep_id: Uuid,
    pub plan_id: Uuid,
    pub effective_from: chrono::NaiveDate,
    pub effective_to: Option<chrono::NaiveDate>,
}

pub async fn assign_plan(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<AssignPlanRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    match state.sales_commission_engine.assign_plan(
        org_id, req.rep_id, req.plan_id,
        req.effective_from, req.effective_to, None,
    ).await {
        Ok(assignment) => Ok((StatusCode::CREATED, Json(serde_json::to_value(assignment).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => {
            error!("Failed to assign plan: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListAssignmentsQuery {
    pub rep_id: Option<Uuid>,
}

pub async fn list_assignments(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<ListAssignmentsQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    match state.sales_commission_engine.list_assignments(org_id, query.rep_id).await {
        Ok(assignments) => Ok(Json(serde_json::json!({"data": assignments}))),
        Err(e) => {
            error!("Failed to list assignments: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

// ============================================================================
// Sales Quotas
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateQuotaRequest {
    pub rep_id: Uuid,
    pub plan_id: Option<Uuid>,
    pub period_name: String,
    pub period_start_date: chrono::NaiveDate,
    pub period_end_date: chrono::NaiveDate,
    #[serde(default = "default_revenue")]
    pub quota_type: String,
    pub target_amount: String,
}

pub async fn create_quota(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreateQuotaRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    let quota_number = format!("Q-{}", Uuid::new_v4().to_string()[..8].to_uppercase());
    match state.sales_commission_engine.create_quota(
        org_id, req.rep_id, req.plan_id, &quota_number,
        &req.period_name, req.period_start_date, req.period_end_date,
        &req.quota_type, &req.target_amount, None,
    ).await {
        Ok(quota) => Ok((StatusCode::CREATED, Json(serde_json::to_value(quota).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => {
            error!("Failed to create quota: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

pub async fn list_quotas(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    let rep_id = params.get("rep_id").and_then(|s| Uuid::parse_str(s).ok());
    let status = params.get("status").map(|s| s.as_str());
    match state.sales_commission_engine.list_quotas(org_id, rep_id, status).await {
        Ok(quotas) => Ok(Json(serde_json::json!({"data": quotas}))),
        Err(e) => {
            error!("Failed to list quotas: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

pub async fn get_quota(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.sales_commission_engine.get_quota(id).await {
        Ok(Some(quota)) => Ok(Json(serde_json::to_value(quota).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Quota not found"})))),
        Err(e) => {
            error!("Failed to get quota: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

// ============================================================================
// Commission Transactions
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreditTransactionRequest {
    pub rep_id: Uuid,
    pub plan_id: Option<Uuid>,
    pub quota_id: Option<Uuid>,
    pub source_type: String,
    pub source_id: Option<Uuid>,
    pub source_number: Option<String>,
    pub transaction_date: chrono::NaiveDate,
    pub sale_amount: String,
    #[serde(default = "default_usd")]
    pub currency_code: String,
}

pub async fn credit_transaction(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreditTransactionRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    match state.sales_commission_engine.credit_transaction(
        org_id, req.rep_id, req.plan_id, req.quota_id,
        &req.source_type, req.source_id, req.source_number.as_deref(),
        req.transaction_date, &req.sale_amount, &req.currency_code, None,
    ).await {
        Ok(tx) => Ok((StatusCode::CREATED, Json(serde_json::to_value(tx).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => {
            error!("Failed to credit transaction: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

pub async fn list_commission_transactions(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    let rep_id = params.get("rep_id").and_then(|s| Uuid::parse_str(s).ok());
    let status = params.get("status").map(|s| s.as_str());
    match state.sales_commission_engine.list_transactions(org_id, rep_id, status).await {
        Ok(txs) => Ok(Json(serde_json::json!({"data": txs}))),
        Err(e) => {
            error!("Failed to list transactions: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

pub async fn get_commission_transaction(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.sales_commission_engine.get_transaction(id).await {
        Ok(Some(tx)) => Ok(Json(serde_json::to_value(tx).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Transaction not found"})))),
        Err(e) => {
            error!("Failed to get transaction: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

// ============================================================================
// Payouts
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ProcessPayoutRequest {
    pub period_name: String,
    pub period_start_date: chrono::NaiveDate,
    pub period_end_date: chrono::NaiveDate,
    #[serde(default = "default_usd")]
    pub currency_code: String,
}

pub async fn process_payout(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<ProcessPayoutRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    match state.sales_commission_engine.process_payout(
        org_id, &req.period_name, req.period_start_date, req.period_end_date,
        &req.currency_code, None,
    ).await {
        Ok(payout) => Ok((StatusCode::CREATED, Json(serde_json::to_value(payout).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => {
            error!("Failed to process payout: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

pub async fn list_payouts(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    let status = params.get("status").map(|s| s.as_str());
    match state.sales_commission_engine.list_payouts(org_id, status).await {
        Ok(payouts) => Ok(Json(serde_json::json!({"data": payouts}))),
        Err(e) => {
            error!("Failed to list payouts: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

pub async fn get_payout(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.sales_commission_engine.get_payout(id).await {
        Ok(Some(payout)) => Ok(Json(serde_json::to_value(payout).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Payout not found"})))),
        Err(e) => {
            error!("Failed to get payout: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

pub async fn get_payout_lines(
    State(state): State<Arc<AppState>>,
    Path(payout_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.sales_commission_engine.list_payout_lines(payout_id).await {
        Ok(lines) => Ok(Json(serde_json::json!({"data": lines}))),
        Err(e) => {
            error!("Failed to list payout lines: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ApprovePayoutRequest {
    pub rejected_reason: Option<String>,
}

pub async fn approve_payout(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = parse_uuid(&claims.sub).ok();
    match state.sales_commission_engine.approve_payout(id, user_id).await {
        Ok(payout) => Ok(Json(serde_json::to_value(payout).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => {
            error!("Failed to approve payout: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

pub async fn reject_payout(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<ApprovePayoutRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.sales_commission_engine.reject_payout(id, req.rejected_reason.as_deref()).await {
        Ok(payout) => Ok(Json(serde_json::to_value(payout).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => {
            error!("Failed to reject payout: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

// ============================================================================
// Dashboard
// ============================================================================

pub async fn get_commission_dashboard(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    match state.sales_commission_engine.get_dashboard_summary(org_id).await {
        Ok(summary) => Ok(Json(serde_json::to_value(summary).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => {
            error!("Failed to get commission dashboard: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}
