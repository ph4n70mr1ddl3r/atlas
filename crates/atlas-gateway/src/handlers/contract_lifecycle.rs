//! Contract Lifecycle Management Handlers
//!
//! Oracle Fusion Enterprise Contracts API endpoints:
//! - Contract type CRUD
//! - Clause library management
//! - Contract template management
//! - Contract lifecycle (create, transition, parties, clauses, milestones, deliverables)
//! - Amendments
//! - Risk assessments
//! - Dashboard

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

fn err_status(e: &atlas_shared::AtlasError) -> StatusCode {
    match e {
        atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
        atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
        atlas_shared::AtlasError::InvalidStateTransition(_, _) => StatusCode::BAD_REQUEST,
        atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

fn parse_uuid(s: &str, label: &str) -> Result<Uuid, StatusCode> {
    s.parse().map_err(|_| { tracing::error!("Invalid {}: {}", label, s); StatusCode::BAD_REQUEST })
}

fn parse_date(s: Option<&String>) -> Option<chrono::NaiveDate> {
    s.as_deref().and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
}

// ═══════════════════════════════════════════════════════════════════════
// Contract Types
// ═══════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateContractTypeRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub contract_category: Option<String>,
    pub default_duration_days: Option<i32>,
    pub requires_approval: Option<bool>,
    pub is_auto_renew: Option<bool>,
    pub risk_scoring_enabled: Option<bool>,
}

pub async fn create_contract_type(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateContractTypeRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = parse_uuid(&claims.org_id, "org_id")?;
    let user_id = parse_uuid(&claims.sub, "user_id")?;
    let ct = state.clm_engine.create_contract_type(
        org_id, &payload.code, &payload.name, payload.description.as_deref(),
        payload.contract_category.as_deref().unwrap_or("general"),
        payload.default_duration_days,
        payload.requires_approval.unwrap_or(true),
        payload.is_auto_renew.unwrap_or(false),
        payload.risk_scoring_enabled.unwrap_or(false),
        Some(user_id),
    ).await.map_err(|e| { tracing::error!("Create contract type: {}", e); err_status(&e) })?;
    Ok((StatusCode::CREATED, Json(serde_json::to_value(ct).unwrap_or_default())))
}

pub async fn get_contract_type(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let ct = state.clm_engine.get_contract_type(id).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match ct {
        Some(c) => Ok(Json(serde_json::to_value(c).unwrap_or_default())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn list_contract_types(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = parse_uuid(&claims.org_id, "org_id")?;
    let types = state.clm_engine.list_contract_types(org_id).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({"data": types, "meta": {"total": types.len()}})))
}

pub async fn delete_contract_type(
    State(state): State<Arc<AppState>>,
    Path(code): Path<String>,
    claims: Extension<Claims>,
) -> Result<StatusCode, StatusCode> {
    let org_id = parse_uuid(&claims.org_id, "org_id")?;
    state.clm_engine.delete_contract_type(org_id, &code).await.map_err(|e| err_status(&e))?;
    Ok(StatusCode::NO_CONTENT)
}

// ═══════════════════════════════════════════════════════════════════════
// Clauses
// ═══════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateClauseRequest {
    pub code: String,
    pub title: String,
    pub body: String,
    pub clause_type: Option<String>,
    pub clause_category: Option<String>,
    pub applicability: Option<String>,
    pub is_locked: Option<bool>,
}

pub async fn create_clause(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateClauseRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = parse_uuid(&claims.org_id, "org_id")?;
    let user_id = parse_uuid(&claims.sub, "user_id")?;
    let c = state.clm_engine.create_clause(
        org_id, &payload.code, &payload.title, &payload.body,
        payload.clause_type.as_deref().unwrap_or("standard"),
        payload.clause_category.as_deref().unwrap_or("general"),
        payload.applicability.as_deref().unwrap_or("all"),
        payload.is_locked.unwrap_or(false),
        Some(user_id),
    ).await.map_err(|e| { tracing::error!("Create clause: {}", e); err_status(&e) })?;
    Ok((StatusCode::CREATED, Json(serde_json::to_value(c).unwrap_or_default())))
}

pub async fn list_clauses(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<ClauseListParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = parse_uuid(&claims.org_id, "org_id")?;
    let clauses = state.clm_engine.list_clauses(org_id, params.category.as_deref()).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({"data": clauses, "meta": {"total": clauses.len()}})))
}

#[derive(Debug, Deserialize)]
pub struct ClauseListParams { pub category: Option<String> }

pub async fn delete_clause(
    State(state): State<Arc<AppState>>,
    Path(code): Path<String>,
    claims: Extension<Claims>,
) -> Result<StatusCode, StatusCode> {
    let org_id = parse_uuid(&claims.org_id, "org_id")?;
    state.clm_engine.delete_clause(org_id, &code).await.map_err(|e| err_status(&e))?;
    Ok(StatusCode::NO_CONTENT)
}

// ═══════════════════════════════════════════════════════════════════════
// Templates
// ═══════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTemplateRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub contract_type_id: Option<Uuid>,
    pub default_currency: Option<String>,
    pub default_duration_days: Option<i32>,
    pub terms_and_conditions: Option<String>,
}

pub async fn create_template(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateTemplateRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = parse_uuid(&claims.org_id, "org_id")?;
    let user_id = parse_uuid(&claims.sub, "user_id")?;
    let t = state.clm_engine.create_template(
        org_id, &payload.code, &payload.name, payload.description.as_deref(),
        payload.contract_type_id, payload.default_currency.as_deref().unwrap_or("USD"),
        payload.default_duration_days, payload.terms_and_conditions.as_deref(), Some(user_id),
    ).await.map_err(|e| { tracing::error!("Create template: {}", e); err_status(&e) })?;
    Ok((StatusCode::CREATED, Json(serde_json::to_value(t).unwrap_or_default())))
}

pub async fn list_templates(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = parse_uuid(&claims.org_id, "org_id")?;
    let templates = state.clm_engine.list_templates(org_id).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({"data": templates, "meta": {"total": templates.len()}})))
}

pub async fn delete_template(
    State(state): State<Arc<AppState>>,
    Path(code): Path<String>,
    claims: Extension<Claims>,
) -> Result<StatusCode, StatusCode> {
    let org_id = parse_uuid(&claims.org_id, "org_id")?;
    state.clm_engine.delete_template(org_id, &code).await.map_err(|e| err_status(&e))?;
    Ok(StatusCode::NO_CONTENT)
}

// ═══════════════════════════════════════════════════════════════════════
// Contracts
// ═══════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateContractRequest {
    pub contract_number: String,
    pub title: String,
    pub description: Option<String>,
    pub contract_type_id: Option<Uuid>,
    pub template_id: Option<Uuid>,
    pub contract_category: Option<String>,
    pub currency: Option<String>,
    pub total_value: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub priority: Option<String>,
    pub renewal_type: Option<String>,
    pub auto_renew_months: Option<i32>,
    pub renewal_notice_days: Option<i32>,
}

pub async fn create_contract(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateContractRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = parse_uuid(&claims.org_id, "org_id")?;
    let user_id = parse_uuid(&claims.sub, "user_id")?;
    let c = state.clm_engine.create_contract(
        org_id, &payload.contract_number, &payload.title, payload.description.as_deref(),
        payload.contract_type_id, payload.template_id,
        payload.contract_category.as_deref().unwrap_or("general"),
        payload.currency.as_deref().unwrap_or("USD"),
        payload.total_value.as_deref().unwrap_or("0"),
        parse_date(payload.start_date.as_ref()),
        parse_date(payload.end_date.as_ref()),
        payload.priority.as_deref().unwrap_or("normal"),
        payload.renewal_type.as_deref().unwrap_or("none"),
        payload.auto_renew_months,
        payload.renewal_notice_days.unwrap_or(30),
        Some(user_id),
    ).await.map_err(|e| { tracing::error!("Create contract: {}", e); err_status(&e) })?;
    Ok((StatusCode::CREATED, Json(serde_json::to_value(c).unwrap_or_default())))
}

pub async fn get_contract(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let c = state.clm_engine.get_contract(id).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match c {
        Some(ct) => Ok(Json(serde_json::to_value(ct).unwrap_or_default())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

#[derive(Debug, Deserialize)]
pub struct ListContractsParams {
    pub status: Option<String>,
    pub category: Option<String>,
}

pub async fn list_contracts(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<ListContractsParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = parse_uuid(&claims.org_id, "org_id")?;
    let contracts = state.clm_engine.list_contracts(org_id, params.status.as_deref(), params.category.as_deref()).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({"data": contracts, "meta": {"total": contracts.len()}})))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransitionRequest {
    pub status: String,
}

pub async fn transition_contract(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    claims: Extension<Claims>,
    Json(payload): Json<TransitionRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = parse_uuid(&claims.sub, "user_id")?;
    let c = state.clm_engine.transition_contract(id, &payload.status, Some(user_id)).await.map_err(|e| { tracing::error!("Transition: {}", e); err_status(&e) })?;
    Ok(Json(serde_json::to_value(c).unwrap_or_default()))
}

pub async fn delete_contract(
    State(state): State<Arc<AppState>>,
    Path(number): Path<String>,
    claims: Extension<Claims>,
) -> Result<StatusCode, StatusCode> {
    let org_id = parse_uuid(&claims.org_id, "org_id")?;
    state.clm_engine.delete_contract(org_id, &number).await.map_err(|e| err_status(&e))?;
    Ok(StatusCode::NO_CONTENT)
}

// ═══════════════════════════════════════════════════════════════════════
// Contract Parties
// ═══════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddPartyRequest {
    pub party_type: Option<String>,
    pub party_role: Option<String>,
    pub party_name: String,
    pub contact_name: Option<String>,
    pub contact_email: Option<String>,
    pub contact_phone: Option<String>,
    pub entity_reference: Option<String>,
    pub is_primary: Option<bool>,
}

pub async fn add_contract_party(
    State(state): State<Arc<AppState>>,
    Path(contract_id): Path<Uuid>,
    claims: Extension<Claims>,
    Json(payload): Json<AddPartyRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = parse_uuid(&claims.org_id, "org_id")?;
    let p = state.clm_engine.add_contract_party(
        org_id, contract_id,
        payload.party_type.as_deref().unwrap_or("external"),
        payload.party_role.as_deref().unwrap_or("counterparty"),
        &payload.party_name, payload.contact_name.as_deref(),
        payload.contact_email.as_deref(), payload.contact_phone.as_deref(),
        payload.entity_reference.as_deref(), payload.is_primary.unwrap_or(false),
    ).await.map_err(|e| { tracing::error!("Add party: {}", e); err_status(&e) })?;
    Ok((StatusCode::CREATED, Json(serde_json::to_value(p).unwrap_or_default())))
}

pub async fn list_contract_parties(
    State(state): State<Arc<AppState>>,
    Path(contract_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let parties = state.clm_engine.list_contract_parties(contract_id).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({"data": parties, "meta": {"total": parties.len()}})))
}

pub async fn remove_contract_party(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    state.clm_engine.remove_contract_party(id).await.map_err(|e| err_status(&e))?;
    Ok(StatusCode::NO_CONTENT)
}

// ═══════════════════════════════════════════════════════════════════════
// Contract Milestones
// ═══════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMilestoneRequest {
    pub name: String,
    pub description: Option<String>,
    pub milestone_type: Option<String>,
    pub due_date: Option<String>,
    pub amount: Option<String>,
    pub currency: Option<String>,
}

pub async fn create_milestone(
    State(state): State<Arc<AppState>>,
    Path(contract_id): Path<Uuid>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateMilestoneRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = parse_uuid(&claims.org_id, "org_id")?;
    let m = state.clm_engine.create_milestone(
        org_id, contract_id, &payload.name, payload.description.as_deref(),
        payload.milestone_type.as_deref().unwrap_or("event"),
        parse_date(payload.due_date.as_ref()),
        payload.amount.as_deref(), payload.currency.as_deref().unwrap_or("USD"),
    ).await.map_err(|e| { tracing::error!("Create milestone: {}", e); err_status(&e) })?;
    Ok((StatusCode::CREATED, Json(serde_json::to_value(m).unwrap_or_default())))
}

pub async fn list_milestones(
    State(state): State<Arc<AppState>>,
    Path(contract_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let ms = state.clm_engine.list_milestones(contract_id).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({"data": ms, "meta": {"total": ms.len()}})))
}

pub async fn complete_milestone(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let m = state.clm_engine.complete_milestone(id).await.map_err(|e| err_status(&e))?;
    Ok(Json(serde_json::to_value(m).unwrap_or_default()))
}

pub async fn delete_milestone(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    state.clm_engine.delete_milestone(id).await.map_err(|e| err_status(&e))?;
    Ok(StatusCode::NO_CONTENT)
}

// ═══════════════════════════════════════════════════════════════════════
// Deliverables
// ═══════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDeliverableRequest {
    pub name: String,
    pub description: Option<String>,
    pub deliverable_type: Option<String>,
    pub quantity: Option<String>,
    pub unit_of_measure: Option<String>,
    pub due_date: Option<String>,
    pub amount: Option<String>,
    pub currency: Option<String>,
    pub milestone_id: Option<Uuid>,
}

pub async fn create_deliverable(
    State(state): State<Arc<AppState>>,
    Path(contract_id): Path<Uuid>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateDeliverableRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = parse_uuid(&claims.org_id, "org_id")?;
    let d = state.clm_engine.create_deliverable(
        org_id, contract_id, payload.milestone_id, &payload.name,
        payload.description.as_deref(), payload.deliverable_type.as_deref().unwrap_or("document"),
        payload.quantity.as_deref().unwrap_or("1"), payload.unit_of_measure.as_deref().unwrap_or("each"),
        parse_date(payload.due_date.as_ref()), payload.amount.as_deref(), payload.currency.as_deref().unwrap_or("USD"),
    ).await.map_err(|e| { tracing::error!("Create deliverable: {}", e); err_status(&e) })?;
    Ok((StatusCode::CREATED, Json(serde_json::to_value(d).unwrap_or_default())))
}

pub async fn list_deliverables(
    State(state): State<Arc<AppState>>,
    Path(contract_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let ds = state.clm_engine.list_deliverables(contract_id).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({"data": ds, "meta": {"total": ds.len()}})))
}

pub async fn accept_deliverable(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = parse_uuid(&claims.sub, "user_id")?;
    let d = state.clm_engine.accept_deliverable(id, Some(user_id)).await.map_err(|e| err_status(&e))?;
    Ok(Json(serde_json::to_value(d).unwrap_or_default()))
}

pub async fn reject_deliverable(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let d = state.clm_engine.reject_deliverable(id).await.map_err(|e| err_status(&e))?;
    Ok(Json(serde_json::to_value(d).unwrap_or_default()))
}

pub async fn delete_deliverable(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    state.clm_engine.delete_deliverable(id).await.map_err(|e| err_status(&e))?;
    Ok(StatusCode::NO_CONTENT)
}

// ═══════════════════════════════════════════════════════════════════════
// Amendments
// ═══════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAmendmentRequest {
    pub amendment_number: String,
    pub title: String,
    pub description: Option<String>,
    pub amendment_type: Option<String>,
    pub previous_value: Option<String>,
    pub new_value: Option<String>,
    pub effective_date: Option<String>,
}

pub async fn create_amendment(
    State(state): State<Arc<AppState>>,
    Path(contract_id): Path<Uuid>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateAmendmentRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = parse_uuid(&claims.org_id, "org_id")?;
    let user_id = parse_uuid(&claims.sub, "user_id")?;
    let a = state.clm_engine.create_amendment(
        org_id, contract_id, &payload.amendment_number, &payload.title,
        payload.description.as_deref(), payload.amendment_type.as_deref().unwrap_or("modification"),
        payload.previous_value.as_deref(), payload.new_value.as_deref(),
        parse_date(payload.effective_date.as_ref()), Some(user_id),
    ).await.map_err(|e| { tracing::error!("Create amendment: {}", e); err_status(&e) })?;
    Ok((StatusCode::CREATED, Json(serde_json::to_value(a).unwrap_or_default())))
}

pub async fn list_amendments(
    State(state): State<Arc<AppState>>,
    Path(contract_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let as_ = state.clm_engine.list_amendments(contract_id).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({"data": as_, "meta": {"total": as_.len()}})))
}

pub async fn approve_amendment(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = parse_uuid(&claims.sub, "user_id")?;
    let a = state.clm_engine.approve_amendment(id, Some(user_id)).await.map_err(|e| err_status(&e))?;
    Ok(Json(serde_json::to_value(a).unwrap_or_default()))
}

pub async fn reject_amendment(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let a = state.clm_engine.reject_amendment(id).await.map_err(|e| err_status(&e))?;
    Ok(Json(serde_json::to_value(a).unwrap_or_default()))
}

// ═══════════════════════════════════════════════════════════════════════
// Risk Assessments
// ═══════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRiskRequest {
    pub risk_category: String,
    pub risk_description: String,
    pub probability: Option<String>,
    pub impact: Option<String>,
    pub mitigation_strategy: Option<String>,
}

pub async fn create_risk(
    State(state): State<Arc<AppState>>,
    Path(contract_id): Path<Uuid>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateRiskRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = parse_uuid(&claims.org_id, "org_id")?;
    let user_id = parse_uuid(&claims.sub, "user_id")?;
    let r = state.clm_engine.create_risk(
        org_id, contract_id, &payload.risk_category, &payload.risk_description,
        payload.probability.as_deref().unwrap_or("medium"),
        payload.impact.as_deref().unwrap_or("medium"),
        payload.mitigation_strategy.as_deref(), Some(user_id),
    ).await.map_err(|e| { tracing::error!("Create risk: {}", e); err_status(&e) })?;
    Ok((StatusCode::CREATED, Json(serde_json::to_value(r).unwrap_or_default())))
}

pub async fn list_risks(
    State(state): State<Arc<AppState>>,
    Path(contract_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let rs = state.clm_engine.list_risks(contract_id).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({"data": rs, "meta": {"total": rs.len()}})))
}

pub async fn delete_risk(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    state.clm_engine.delete_risk(id).await.map_err(|e| err_status(&e))?;
    Ok(StatusCode::NO_CONTENT)
}

// ═══════════════════════════════════════════════════════════════════════
// Dashboard
// ═══════════════════════════════════════════════════════════════════════

pub async fn get_clm_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = parse_uuid(&claims.org_id, "org_id")?;
    let summary = state.clm_engine.get_dashboard(org_id).await.map_err(|e| { tracing::error!("CLM dashboard: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;
    Ok(Json(serde_json::to_value(summary).unwrap_or_default()))
}
