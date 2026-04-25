//! Revenue Recognition Handlers (ASC 606 / IFRS 15)
//!
//! Oracle Fusion Cloud ERP: Financials > Revenue Management.
//! Implements the five-step revenue recognition model via HTTP API.

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

fn rev_map_err(e: atlas_shared::AtlasError) -> StatusCode {
    match e.status_code() {
        400 => StatusCode::BAD_REQUEST,
        404 => StatusCode::NOT_FOUND,
        409 => StatusCode::CONFLICT,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

// ============================================================================
// Revenue Policies
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreatePolicyRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub recognition_method: Option<String>,
    pub over_time_method: Option<String>,
    pub allocation_basis: Option<String>,
    pub default_selling_price: Option<String>,
    pub constrain_variable_consideration: Option<bool>,
    pub constraint_threshold_percent: Option<String>,
    pub revenue_account_code: Option<String>,
    pub deferred_revenue_account_code: Option<String>,
    pub contra_revenue_account_code: Option<String>,
}

pub async fn create_policy(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreatePolicyRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).ok();
    match state.revenue_engine.create_policy(
        org_id, &payload.code, &payload.name, payload.description.as_deref(),
        payload.recognition_method.as_deref().unwrap_or("point_in_time"),
        payload.over_time_method.as_deref(),
        payload.allocation_basis.as_deref().unwrap_or("standalone_selling_price"),
        payload.default_selling_price.as_deref(),
        payload.constrain_variable_consideration.unwrap_or(false),
        payload.constraint_threshold_percent.as_deref(),
        payload.revenue_account_code.as_deref(),
        payload.deferred_revenue_account_code.as_deref(),
        payload.contra_revenue_account_code.as_deref(),
        user_id,
    ).await {
        Ok(p) => Ok((StatusCode::CREATED, Json(serde_json::to_value(p).unwrap_or_default()))),
        Err(e) => { error!("Failed to create revenue policy: {}", e); Err(rev_map_err(e)) }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListPoliciesParams {}

pub async fn list_policies(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.revenue_engine.list_policies(org_id).await {
        Ok(policies) => Ok(Json(serde_json::json!({ "data": policies }))),
        Err(e) => { error!("Failed to list revenue policies: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn get_policy(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(code): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.revenue_engine.get_policy(org_id, &code).await {
        Ok(Some(p)) => Ok(Json(serde_json::to_value(p).unwrap_or_default())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Failed to get revenue policy: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn delete_policy(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(code): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.revenue_engine.delete_policy(org_id, &code).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => { error!("Failed to delete revenue policy: {}", e); Err(rev_map_err(e)) }
    }
}

// ============================================================================
// Revenue Contracts
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateContractRequest {
    pub customer_id: Uuid,
    pub customer_number: Option<String>,
    pub customer_name: Option<String>,
    pub source_type: Option<String>,
    pub source_id: Option<Uuid>,
    pub source_number: Option<String>,
    pub contract_date: Option<chrono::NaiveDate>,
    pub start_date: Option<chrono::NaiveDate>,
    pub end_date: Option<chrono::NaiveDate>,
    pub total_transaction_price: String,
    pub currency_code: Option<String>,
    pub notes: Option<String>,
}

pub async fn create_contract(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateContractRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).ok();
    match state.revenue_engine.create_contract(
        org_id,
        payload.source_type.as_deref(),
        payload.source_id,
        payload.source_number.as_deref(),
        payload.customer_id,
        payload.customer_number.as_deref(),
        payload.customer_name.as_deref(),
        payload.contract_date,
        payload.start_date,
        payload.end_date,
        &payload.total_transaction_price,
        payload.currency_code.as_deref().unwrap_or("USD"),
        payload.notes.as_deref(),
        user_id,
    ).await {
        Ok(c) => Ok((StatusCode::CREATED, Json(serde_json::to_value(c).unwrap_or_default()))),
        Err(e) => { error!("Failed to create revenue contract: {}", e); Err(rev_map_err(e)) }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListContractsParams {
    status: Option<String>,
    customer_id: Option<Uuid>,
}

pub async fn list_contracts(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<ListContractsParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.revenue_engine.list_contracts(org_id, params.status.as_deref(), params.customer_id).await {
        Ok(contracts) => Ok(Json(serde_json::json!({ "data": contracts }))),
        Err(e) => { error!("Failed to list revenue contracts: {}", e); Err(rev_map_err(e)) }
    }
}

pub async fn get_contract(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.revenue_engine.get_contract(id).await {
        Ok(Some(c)) => Ok(Json(serde_json::to_value(c).unwrap_or_default())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Failed to get revenue contract: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn activate_contract(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.revenue_engine.activate_contract(id).await {
        Ok(c) => Ok(Json(serde_json::to_value(c).unwrap_or_default())),
        Err(e) => { error!("Failed to activate revenue contract: {}", e); Err(rev_map_err(e)) }
    }
}

#[derive(Debug, Deserialize)]
pub struct CancelContractRequest {
    pub reason: Option<String>,
}

pub async fn cancel_contract(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<CancelContractRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.revenue_engine.cancel_contract(id, payload.reason.as_deref()).await {
        Ok(c) => Ok(Json(serde_json::to_value(c).unwrap_or_default())),
        Err(e) => { error!("Failed to cancel revenue contract: {}", e); Err(rev_map_err(e)) }
    }
}

// ============================================================================
// Performance Obligations
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateObligationRequest {
    pub description: Option<String>,
    pub product_id: Option<Uuid>,
    pub product_name: Option<String>,
    pub source_line_id: Option<Uuid>,
    pub revenue_policy_id: Option<Uuid>,
    pub recognition_method: Option<String>,
    pub over_time_method: Option<String>,
    pub standalone_selling_price: String,
    pub satisfaction_method: Option<String>,
    pub recognition_start_date: Option<chrono::NaiveDate>,
    pub recognition_end_date: Option<chrono::NaiveDate>,
    pub revenue_account_code: Option<String>,
    pub deferred_revenue_account_code: Option<String>,
}

pub async fn create_obligation(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(contract_id): Path<Uuid>,
    Json(payload): Json<CreateObligationRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).ok();
    match state.revenue_engine.create_obligation(
        org_id,
        contract_id,
        payload.description.as_deref(),
        payload.product_id,
        payload.product_name.as_deref(),
        payload.source_line_id,
        payload.revenue_policy_id,
        payload.recognition_method.as_deref(),
        payload.over_time_method.as_deref(),
        &payload.standalone_selling_price,
        payload.satisfaction_method.as_deref().unwrap_or("point_in_time"),
        payload.recognition_start_date,
        payload.recognition_end_date,
        payload.revenue_account_code.as_deref(),
        payload.deferred_revenue_account_code.as_deref(),
        user_id,
    ).await {
        Ok(o) => Ok((StatusCode::CREATED, Json(serde_json::to_value(o).unwrap_or_default()))),
        Err(e) => { error!("Failed to create obligation: {}", e); Err(rev_map_err(e)) }
    }
}

pub async fn list_obligations(
    State(state): State<Arc<AppState>>,
    Path(contract_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.revenue_engine.list_obligations(contract_id).await {
        Ok(obligations) => Ok(Json(serde_json::json!({ "data": obligations }))),
        Err(e) => { error!("Failed to list obligations: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn get_obligation(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.revenue_engine.get_obligation(id).await {
        Ok(Some(o)) => Ok(Json(serde_json::to_value(o).unwrap_or_default())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Failed to get obligation: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// ============================================================================
// Transaction Price Allocation
// ============================================================================

pub async fn allocate_transaction_price(
    State(state): State<Arc<AppState>>,
    Path(contract_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.revenue_engine.allocate_transaction_price(contract_id).await {
        Ok(obligations) => Ok(Json(serde_json::json!({ "data": obligations }))),
        Err(e) => { error!("Failed to allocate transaction price: {}", e); Err(rev_map_err(e)) }
    }
}

// ============================================================================
// Revenue Recognition Scheduling
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct GenerateScheduleRequest {
    pub start_date: chrono::NaiveDate,
    pub end_date: chrono::NaiveDate,
}

pub async fn generate_straight_line_schedule(
    State(state): State<Arc<AppState>>,
    Path(obligation_id): Path<Uuid>,
    Json(payload): Json<GenerateScheduleRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.revenue_engine.generate_straight_line_schedule(
        obligation_id, payload.start_date, payload.end_date,
    ).await {
        Ok(lines) => Ok(Json(serde_json::json!({ "data": lines }))),
        Err(e) => { error!("Failed to generate schedule: {}", e); Err(rev_map_err(e)) }
    }
}

#[derive(Debug, Deserialize)]
pub struct SchedulePointInTimeRequest {
    pub recognition_date: chrono::NaiveDate,
}

pub async fn schedule_point_in_time(
    State(state): State<Arc<AppState>>,
    Path(obligation_id): Path<Uuid>,
    Json(payload): Json<SchedulePointInTimeRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    match state.revenue_engine.schedule_point_in_time(
        obligation_id, payload.recognition_date,
    ).await {
        Ok(line) => Ok((StatusCode::CREATED, Json(serde_json::to_value(line).unwrap_or_default()))),
        Err(e) => { error!("Failed to schedule point-in-time: {}", e); Err(rev_map_err(e)) }
    }
}

// ============================================================================
// Revenue Recognition Execution
// ============================================================================

pub async fn recognize_revenue(
    State(state): State<Arc<AppState>>,
    Path(line_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.revenue_engine.recognize_revenue(line_id).await {
        Ok(line) => Ok(Json(serde_json::to_value(line).unwrap_or_default())),
        Err(e) => { error!("Failed to recognize revenue: {}", e); Err(rev_map_err(e)) }
    }
}

#[derive(Debug, Deserialize)]
pub struct ReverseRecognitionRequest {
    pub reason: String,
}

pub async fn reverse_recognition(
    State(state): State<Arc<AppState>>,
    Path(line_id): Path<Uuid>,
    Json(payload): Json<ReverseRecognitionRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.revenue_engine.reverse_recognition(line_id, &payload.reason).await {
        Ok(line) => Ok(Json(serde_json::to_value(line).unwrap_or_default())),
        Err(e) => { error!("Failed to reverse recognition: {}", e); Err(rev_map_err(e)) }
    }
}

pub async fn list_schedule_lines(
    State(state): State<Arc<AppState>>,
    Path(obligation_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.revenue_engine.list_schedule_lines(obligation_id).await {
        Ok(lines) => Ok(Json(serde_json::json!({ "data": lines }))),
        Err(e) => { error!("Failed to list schedule lines: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn list_contract_schedule_lines(
    State(state): State<Arc<AppState>>,
    Path(contract_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.revenue_engine.list_contract_schedule_lines(contract_id).await {
        Ok(lines) => Ok(Json(serde_json::json!({ "data": lines }))),
        Err(e) => { error!("Failed to list contract schedule lines: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// ============================================================================
// Contract Modifications
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateModificationRequest {
    pub modification_type: String,
    pub description: Option<String>,
    pub previous_transaction_price: String,
    pub new_transaction_price: String,
    pub previous_end_date: Option<chrono::NaiveDate>,
    pub new_end_date: Option<chrono::NaiveDate>,
    pub effective_date: chrono::NaiveDate,
}

pub async fn create_modification(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(contract_id): Path<Uuid>,
    Json(payload): Json<CreateModificationRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).ok();
    match state.revenue_engine.create_modification(
        org_id,
        contract_id,
        &payload.modification_type,
        payload.description.as_deref(),
        &payload.previous_transaction_price,
        &payload.new_transaction_price,
        payload.previous_end_date,
        payload.new_end_date,
        payload.effective_date,
        user_id,
    ).await {
        Ok(m) => Ok((StatusCode::CREATED, Json(serde_json::to_value(m).unwrap_or_default()))),
        Err(e) => { error!("Failed to create modification: {}", e); Err(rev_map_err(e)) }
    }
}

pub async fn list_modifications(
    State(state): State<Arc<AppState>>,
    Path(contract_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.revenue_engine.list_modifications(contract_id).await {
        Ok(modifications) => Ok(Json(serde_json::json!({ "data": modifications }))),
        Err(e) => { error!("Failed to list modifications: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}
