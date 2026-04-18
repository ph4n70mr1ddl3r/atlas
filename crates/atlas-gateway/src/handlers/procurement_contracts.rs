//! Procurement Contracts Handlers
//!
//! Oracle Fusion Cloud ERP: SCM > Procurement > Contracts
//! API endpoints for contract types, procurement contracts,
//! contract lines, milestones, renewals, spend tracking,
//! and dashboard.

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
// Contract Types
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateContractTypeRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(default = "default_classification")]
    pub contract_classification: String,
    #[serde(default = "default_true")]
    pub requires_approval: bool,
    pub default_duration_days: Option<i32>,
    #[serde(default = "default_true")]
    pub allow_amount_commitment: bool,
    #[serde(default = "default_true")]
    pub allow_quantity_commitment: bool,
    #[serde(default = "default_true")]
    pub allow_line_additions: bool,
    #[serde(default = "default_false")]
    pub allow_price_adjustment: bool,
    #[serde(default = "default_true")]
    pub allow_renewal: bool,
    #[serde(default = "default_true")]
    pub allow_termination: bool,
    pub max_renewals: Option<i32>,
    pub default_payment_terms_code: Option<String>,
    pub default_currency_code: Option<String>,
}

fn default_classification() -> String { "blanket".to_string() }
fn default_true() -> bool { true }
fn default_false() -> bool { false }

/// Create a contract type
pub async fn create_contract_type(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateContractTypeRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let ct = state.procurement_contract_engine
        .create_contract_type(
            org_id, &payload.code, &payload.name,
            payload.description.as_deref(),
            &payload.contract_classification,
            payload.requires_approval,
            payload.default_duration_days,
            payload.allow_amount_commitment,
            payload.allow_quantity_commitment,
            payload.allow_line_additions,
            payload.allow_price_adjustment,
            payload.allow_renewal,
            payload.allow_termination,
            payload.max_renewals,
            payload.default_payment_terms_code.as_deref(),
            payload.default_currency_code.as_deref(),
            Some(user_id),
        )
        .await
        .map_err(|e| {
            error!("Create contract type error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(ct).unwrap_or_default())))
}

/// Get a contract type by code
pub async fn get_contract_type(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(code): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let ct = state.procurement_contract_engine
        .get_contract_type(org_id, &code)
        .await
        .map_err(|e| {
            error!("Get contract type error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(serde_json::to_value(ct).unwrap_or_default()))
}

/// List contract types
pub async fn list_contract_types(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let types = state.procurement_contract_engine
        .list_contract_types(org_id)
        .await
        .map_err(|e| {
            error!("List contract types error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(serde_json::json!({"data": types})))
}

/// Delete a contract type
pub async fn delete_contract_type(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(code): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    state.procurement_contract_engine
        .delete_contract_type(org_id, &code)
        .await
        .map_err(|e| {
            error!("Delete contract type error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Contracts
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateContractRequest {
    pub title: String,
    pub description: Option<String>,
    pub contract_type_code: Option<String>,
    #[serde(default = "default_classification")]
    pub contract_classification: String,
    pub supplier_id: Uuid,
    pub supplier_number: Option<String>,
    pub supplier_name: Option<String>,
    pub supplier_contact: Option<String>,
    pub buyer_id: Option<Uuid>,
    pub buyer_name: Option<String>,
    pub start_date: Option<chrono::NaiveDate>,
    pub end_date: Option<chrono::NaiveDate>,
    #[serde(default = "default_zero")]
    pub total_committed_amount: String,
    #[serde(default = "default_usd")]
    pub currency_code: String,
    pub payment_terms_code: Option<String>,
    #[serde(default = "default_fixed")]
    pub price_type: String,
    pub max_renewals: Option<i32>,
    pub notes: Option<String>,
}

fn default_zero() -> String { "0".to_string() }
fn default_usd() -> String { "USD".to_string() }
fn default_fixed() -> String { "fixed".to_string() }

/// Create a procurement contract
pub async fn create_contract(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateContractRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let contract = state.procurement_contract_engine
        .create_contract(
            org_id, &payload.title, payload.description.as_deref(),
            payload.contract_type_code.as_deref(),
            &payload.contract_classification,
            payload.supplier_id,
            payload.supplier_number.as_deref(),
            payload.supplier_name.as_deref(),
            payload.supplier_contact.as_deref(),
            payload.buyer_id, payload.buyer_name.as_deref(),
            payload.start_date, payload.end_date,
            &payload.total_committed_amount,
            &payload.currency_code,
            payload.payment_terms_code.as_deref(),
            &payload.price_type,
            payload.max_renewals,
            payload.notes.as_deref(),
            Some(user_id),
        )
        .await
        .map_err(|e| {
            error!("Create contract error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(contract).unwrap_or_default())))
}

/// Get a contract by ID
pub async fn get_contract(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let contract = state.procurement_contract_engine
        .get_contract(id)
        .await
        .map_err(|e| {
            error!("Get contract error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(serde_json::to_value(contract).unwrap_or_default()))
}

#[derive(Debug, Deserialize)]
pub struct ListContractsQuery {
    pub status: Option<String>,
    pub supplier_id: Option<Uuid>,
}

/// List contracts
pub async fn list_contracts(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<ListContractsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let contracts = state.procurement_contract_engine
        .list_contracts(org_id, params.status.as_deref(), params.supplier_id)
        .await
        .map_err(|e| {
            error!("List contracts error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(Json(serde_json::json!({"data": contracts})))
}

/// Submit a contract for approval
pub async fn submit_contract(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let contract = state.procurement_contract_engine
        .submit_contract(id)
        .await
        .map_err(|e| {
            error!("Submit contract error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(Json(serde_json::to_value(contract).unwrap_or_default()))
}

#[derive(Debug, Deserialize)]
pub struct ApproveContractRequest {
    pub approved_by: Option<Uuid>,
}

/// Approve a contract
pub async fn approve_contract(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(_payload): Json<ApproveContractRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let contract = state.procurement_contract_engine
        .approve_contract(id, user_id)
        .await
        .map_err(|e| {
            error!("Approve contract error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(Json(serde_json::to_value(contract).unwrap_or_default()))
}

#[derive(Debug, Deserialize)]
pub struct RejectContractRequest {
    pub reason: String,
}

/// Reject a contract
pub async fn reject_contract(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<RejectContractRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let contract = state.procurement_contract_engine
        .reject_contract(id, &payload.reason)
        .await
        .map_err(|e| {
            error!("Reject contract error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(Json(serde_json::to_value(contract).unwrap_or_default()))
}

#[derive(Debug, Deserialize)]
pub struct TerminateContractRequest {
    pub reason: String,
}

/// Terminate a contract
pub async fn terminate_contract(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<TerminateContractRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let contract = state.procurement_contract_engine
        .terminate_contract(id, user_id, &payload.reason)
        .await
        .map_err(|e| {
            error!("Terminate contract error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(Json(serde_json::to_value(contract).unwrap_or_default()))
}

/// Close a contract
pub async fn close_contract(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let contract = state.procurement_contract_engine
        .close_contract(id)
        .await
        .map_err(|e| {
            error!("Close contract error: {}", e);
            match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(Json(serde_json::to_value(contract).unwrap_or_default()))
}

// ============================================================================
// Contract Lines
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct AddContractLineRequest {
    pub item_description: String,
    pub item_code: Option<String>,
    pub category: Option<String>,
    pub uom: Option<String>,
    pub quantity_committed: Option<String>,
    pub unit_price: String,
    pub delivery_date: Option<chrono::NaiveDate>,
    pub supplier_part_number: Option<String>,
    pub account_code: Option<String>,
    pub cost_center: Option<String>,
    pub project_id: Option<Uuid>,
    pub notes: Option<String>,
}

/// Add a line to a contract
pub async fn add_contract_line(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(contract_id): Path<Uuid>,
    Json(payload): Json<AddContractLineRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let line = state.procurement_contract_engine
        .add_contract_line(
            org_id, contract_id,
            &payload.item_description,
            payload.item_code.as_deref(),
            payload.category.as_deref(),
            payload.uom.as_deref(),
            payload.quantity_committed.as_deref(),
            &payload.unit_price,
            payload.delivery_date,
            payload.supplier_part_number.as_deref(),
            payload.account_code.as_deref(),
            payload.cost_center.as_deref(),
            payload.project_id,
            payload.notes.as_deref(),
            Some(user_id),
        )
        .await
        .map_err(|e| {
            error!("Add contract line error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(line).unwrap_or_default())))
}

/// List contract lines
pub async fn list_contract_lines(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(contract_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let lines = state.procurement_contract_engine
        .list_contract_lines(contract_id)
        .await
        .map_err(|e| {
            error!("List contract lines error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(serde_json::json!({"data": lines})))
}

/// Delete a contract line
pub async fn delete_contract_line(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(line_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    state.procurement_contract_engine
        .delete_contract_line(line_id)
        .await
        .map_err(|e| {
            error!("Delete contract line error: {}", e);
            match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Milestones
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct AddMilestoneRequest {
    pub contract_line_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    #[serde(default = "default_milestone_type")]
    pub milestone_type: String,
    pub target_date: chrono::NaiveDate,
    #[serde(default = "default_zero")]
    pub amount: String,
    #[serde(default = "default_zero")]
    pub percent_of_total: String,
    pub deliverable: Option<String>,
    #[serde(default = "default_false")]
    pub is_billable: bool,
}

fn default_milestone_type() -> String { "delivery".to_string() }

/// Add a milestone
pub async fn add_milestone(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(contract_id): Path<Uuid>,
    Json(payload): Json<AddMilestoneRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let milestone = state.procurement_contract_engine
        .add_milestone(
            org_id, contract_id,
            payload.contract_line_id,
            &payload.name,
            payload.description.as_deref(),
            &payload.milestone_type,
            payload.target_date,
            &payload.amount,
            &payload.percent_of_total,
            payload.deliverable.as_deref(),
            payload.is_billable,
            Some(user_id),
        )
        .await
        .map_err(|e| {
            error!("Add milestone error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(milestone).unwrap_or_default())))
}

/// List milestones
pub async fn list_milestones(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(contract_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let milestones = state.procurement_contract_engine
        .list_milestones(contract_id)
        .await
        .map_err(|e| {
            error!("List milestones error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(serde_json::json!({"data": milestones})))
}

#[derive(Debug, Deserialize)]
pub struct UpdateMilestoneRequest {
    pub status: String,
    pub actual_date: Option<chrono::NaiveDate>,
}

/// Update milestone status
pub async fn update_milestone(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(milestone_id): Path<Uuid>,
    Json(payload): Json<UpdateMilestoneRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let milestone = state.procurement_contract_engine
        .update_milestone_status(milestone_id, &payload.status, payload.actual_date)
        .await
        .map_err(|e| {
            error!("Update milestone error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(Json(serde_json::to_value(milestone).unwrap_or_default()))
}

// ============================================================================
// Renewals
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct RenewContractRequest {
    pub new_end_date: chrono::NaiveDate,
    #[serde(default = "default_renewal_type")]
    pub renewal_type: String,
    pub terms_changed: Option<String>,
    pub notes: Option<String>,
}

fn default_renewal_type() -> String { "manual".to_string() }

/// Renew a contract
pub async fn renew_contract(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(contract_id): Path<Uuid>,
    Json(payload): Json<RenewContractRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let renewal = state.procurement_contract_engine
        .renew_contract(
            contract_id,
            payload.new_end_date,
            &payload.renewal_type,
            payload.terms_changed.as_deref(),
            Some(user_id),
            payload.notes.as_deref(),
        )
        .await
        .map_err(|e| {
            error!("Renew contract error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(renewal).unwrap_or_default())))
}

/// List renewals
pub async fn list_renewals(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(contract_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let renewals = state.procurement_contract_engine
        .list_renewals(contract_id)
        .await
        .map_err(|e| {
            error!("List renewals error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(serde_json::json!({"data": renewals})))
}

// ============================================================================
// Spend Tracking
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct RecordSpendRequest {
    pub contract_line_id: Option<Uuid>,
    pub source_type: String,
    pub source_id: Option<Uuid>,
    pub source_number: Option<String>,
    pub transaction_date: chrono::NaiveDate,
    pub amount: String,
    pub quantity: Option<String>,
    pub description: Option<String>,
}

/// Record spend against a contract
pub async fn record_spend(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(contract_id): Path<Uuid>,
    Json(payload): Json<RecordSpendRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let spend = state.procurement_contract_engine
        .record_spend(
            org_id, contract_id,
            payload.contract_line_id,
            &payload.source_type,
            payload.source_id,
            payload.source_number.as_deref(),
            payload.transaction_date,
            &payload.amount,
            payload.quantity.as_deref(),
            payload.description.as_deref(),
            Some(user_id),
        )
        .await
        .map_err(|e| {
            error!("Record spend error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(spend).unwrap_or_default())))
}

/// List spend entries
pub async fn list_spend_entries(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(contract_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let entries = state.procurement_contract_engine
        .list_spend_entries(contract_id)
        .await
        .map_err(|e| {
            error!("List spend entries error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(serde_json::json!({"data": entries})))
}

// ============================================================================
// Dashboard
// ============================================================================

/// Get procurement contracts dashboard summary
pub async fn get_dashboard_summary(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let summary = state.procurement_contract_engine
        .get_dashboard_summary(org_id)
        .await
        .map_err(|e| {
            error!("Get contract dashboard error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(serde_json::to_value(summary).unwrap_or_default()))
}
