//! Lease Accounting API Handlers (ASC 842 / IFRS 16)
//!
//! REST endpoints for Oracle Fusion-inspired Lease Management.

use axum::{
    extract::{Path, Query, State, Extension},
    Json,
};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;
use tracing::info;

use crate::AppState;
use crate::handlers::auth::Claims;

// ============================================================================
// Request Types
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateLeaseRequest {
    pub title: String,
    pub description: Option<String>,
    pub classification: String,
    pub lessor_id: Option<Uuid>,
    pub lessor_name: Option<String>,
    pub asset_description: Option<String>,
    pub location: Option<String>,
    pub department_id: Option<Uuid>,
    pub department_name: Option<String>,
    pub commencement_date: chrono::NaiveDate,
    pub end_date: chrono::NaiveDate,
    pub lease_term_months: i32,
    pub purchase_option_exists: Option<bool>,
    pub purchase_option_likely: Option<bool>,
    pub renewal_option_exists: Option<bool>,
    pub renewal_option_months: Option<i32>,
    pub renewal_option_likely: Option<bool>,
    pub discount_rate: String,
    pub currency_code: Option<String>,
    pub payment_frequency: Option<String>,
    pub annual_payment_amount: String,
    pub escalation_rate: Option<String>,
    pub escalation_frequency_months: Option<i32>,
    pub residual_guarantee_amount: Option<String>,
    pub rou_asset_account_code: Option<String>,
    pub rou_depreciation_account_code: Option<String>,
    pub lease_liability_account_code: Option<String>,
    pub lease_expense_account_code: Option<String>,
    pub interest_expense_account_code: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessPaymentRequest {
    pub period_number: i32,
    pub payment_reference: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateModificationRequest {
    pub modification_type: String,
    pub description: Option<String>,
    pub effective_date: chrono::NaiveDate,
    pub new_term_months: Option<i32>,
    pub new_end_date: Option<chrono::NaiveDate>,
    pub new_discount_rate: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminateLeaseRequest {
    pub termination_type: String,
    pub termination_date: chrono::NaiveDate,
    pub termination_penalty: Option<String>,
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LeaseFilters {
    pub status: Option<String>,
    pub classification: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ImpairmentRequest {
    pub impairment_amount: String,
    pub impairment_date: chrono::NaiveDate,
}

fn error_response(e: atlas_shared::AtlasError) -> (axum::http::StatusCode, Json<serde_json::Value>) {
    let status = axum::http::StatusCode::from_u16(e.status_code()).unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
    (status, Json(json!({"error": e.to_string()})))
}

// ============================================================================
// Handlers
// ============================================================================

/// Create a new lease contract
pub async fn create_lease(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreateLeaseRequest>,
) -> Result<(axum::http::StatusCode, Json<serde_json::Value>), (axum::http::StatusCode, Json<serde_json::Value>)> {
    let org_id = Uuid::parse_str(&claims.org_id).unwrap_or_default();
    let created_by = Uuid::parse_str(&claims.sub).ok();

    let lease = state.lease_accounting_engine.create_lease(
        org_id,
        &req.title,
        req.description.as_deref(),
        &req.classification,
        req.lessor_id,
        req.lessor_name.as_deref(),
        req.asset_description.as_deref(),
        req.location.as_deref(),
        req.department_id,
        req.department_name.as_deref(),
        req.commencement_date,
        req.end_date,
        req.lease_term_months,
        req.purchase_option_exists.unwrap_or(false),
        req.purchase_option_likely.unwrap_or(false),
        req.renewal_option_exists.unwrap_or(false),
        req.renewal_option_months,
        req.renewal_option_likely.unwrap_or(false),
        &req.discount_rate,
        req.currency_code.as_deref().unwrap_or("USD"),
        req.payment_frequency.as_deref().unwrap_or("monthly"),
        &req.annual_payment_amount,
        req.escalation_rate.as_deref(),
        req.escalation_frequency_months,
        req.residual_guarantee_amount.as_deref(),
        req.rou_asset_account_code.as_deref(),
        req.rou_depreciation_account_code.as_deref(),
        req.lease_liability_account_code.as_deref(),
        req.lease_expense_account_code.as_deref(),
        req.interest_expense_account_code.as_deref(),
        created_by,
    ).await.map_err(error_response)?;

    info!("Created lease {} via API", lease.lease_number);
    Ok((axum::http::StatusCode::CREATED, Json(serde_json::to_value(lease).unwrap())))
}

/// Get a lease by ID
pub async fn get_lease(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let lease = state.lease_accounting_engine.get_lease(id).await.map_err(error_response)?;

    match lease {
        Some(l) => Ok(Json(serde_json::to_value(l).unwrap())),
        None => Err((axum::http::StatusCode::NOT_FOUND, Json(json!({"error": "Lease not found"})))),
    }
}

/// List leases with optional filters
pub async fn list_leases(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(filters): Query<LeaseFilters>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let org_id = Uuid::parse_str(&claims.org_id).unwrap_or_default();

    let leases = state.lease_accounting_engine.list_leases(
        org_id,
        filters.status.as_deref(),
        filters.classification.as_deref(),
    ).await.map_err(error_response)?;

    Ok(Json(json!({"data": leases})))
}

/// Activate a draft lease
pub async fn activate_lease(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let activated_by = Uuid::parse_str(&claims.sub).ok();
    let lease = state.lease_accounting_engine.activate_lease(id, activated_by).await.map_err(error_response)?;

    Ok(Json(serde_json::to_value(lease).unwrap()))
}

/// List payment schedule for a lease
pub async fn list_lease_payments(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let payments = state.lease_accounting_engine.list_payments(id).await.map_err(error_response)?;

    Ok(Json(json!({"data": payments})))
}

/// Process a lease payment
pub async fn process_lease_payment(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<ProcessPaymentRequest>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let payment = state.lease_accounting_engine.process_payment(
        id, req.period_number, req.payment_reference.as_deref(),
    ).await.map_err(error_response)?;

    Ok(Json(serde_json::to_value(payment).unwrap()))
}

/// Create a lease modification
pub async fn create_lease_modification(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(req): Json<CreateModificationRequest>,
) -> Result<(axum::http::StatusCode, Json<serde_json::Value>), (axum::http::StatusCode, Json<serde_json::Value>)> {
    let org_id = Uuid::parse_str(&claims.org_id).unwrap_or_default();
    let created_by = Uuid::parse_str(&claims.sub).ok();

    let modification = state.lease_accounting_engine.create_modification(
        org_id, id,
        &req.modification_type,
        req.description.as_deref(),
        req.effective_date,
        req.new_term_months,
        req.new_end_date,
        req.new_discount_rate.as_deref(),
        created_by,
    ).await.map_err(error_response)?;

    Ok((axum::http::StatusCode::CREATED, Json(serde_json::to_value(modification).unwrap())))
}

/// List modifications for a lease
pub async fn list_lease_modifications(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let modifications = state.lease_accounting_engine.list_modifications(id).await.map_err(error_response)?;

    Ok(Json(json!({"data": modifications})))
}

/// Record impairment on a lease
pub async fn record_lease_impairment(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<ImpairmentRequest>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let lease = state.lease_accounting_engine.record_impairment(
        id, &req.impairment_amount, req.impairment_date,
    ).await.map_err(error_response)?;

    Ok(Json(serde_json::to_value(lease).unwrap()))
}

/// Terminate a lease
pub async fn terminate_lease(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(req): Json<TerminateLeaseRequest>,
) -> Result<(axum::http::StatusCode, Json<serde_json::Value>), (axum::http::StatusCode, Json<serde_json::Value>)> {
    let org_id = Uuid::parse_str(&claims.org_id).unwrap_or_default();
    let created_by = Uuid::parse_str(&claims.sub).ok();

    let termination = state.lease_accounting_engine.terminate_lease(
        org_id, id,
        &req.termination_type,
        req.termination_date,
        req.termination_penalty.as_deref().unwrap_or("0"),
        req.reason.as_deref(),
        created_by,
    ).await.map_err(error_response)?;

    Ok((axum::http::StatusCode::CREATED, Json(serde_json::to_value(termination).unwrap())))
}

/// List terminations for a lease
pub async fn list_lease_terminations(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let terminations = state.lease_accounting_engine.list_terminations(id).await.map_err(error_response)?;

    Ok(Json(json!({"data": terminations})))
}

/// Get lease accounting dashboard summary
pub async fn get_lease_dashboard(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let org_id = Uuid::parse_str(&claims.org_id).unwrap_or_default();

    let summary = state.lease_accounting_engine.get_dashboard_summary(org_id).await.map_err(error_response)?;

    Ok(Json(serde_json::to_value(summary).unwrap()))
}
