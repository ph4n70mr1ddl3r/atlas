//! Cost Allocation API Handlers
//!
//! REST endpoints for Oracle Fusion-inspired Cost Allocation Management.

use axum::{
    extract::{Path, Query, State, Extension},
    Json,
};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

use crate::AppState;
use crate::handlers::auth::Claims;

// ============================================================================
// Request Types
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePoolRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(default = "default_pool_type")]
    pub pool_type: String,
    pub source_account_codes: serde_json::Value,
    pub source_department_id: Option<Uuid>,
    pub source_cost_center: Option<String>,
}

fn default_pool_type() -> String { "cost_center".to_string() }

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateBaseRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub base_type: String,
    pub financial_account_code: Option<String>,
    pub unit_of_measure: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetBaseValueRequest {
    pub base_code: String,
    pub department_id: Option<Uuid>,
    pub department_name: Option<String>,
    pub cost_center: Option<String>,
    pub project_id: Option<Uuid>,
    pub value: String,
    pub effective_date: chrono::NaiveDate,
    #[serde(default = "default_source")]
    pub source: String,
}

fn default_source() -> String { "manual".to_string() }

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRuleRequest {
    pub name: String,
    pub description: Option<String>,
    pub pool_code: String,
    pub base_code: String,
    pub allocation_method: String,
    pub journal_description: Option<String>,
    pub offset_account_code: Option<String>,
    #[serde(default = "default_currency")]
    pub currency_code: String,
    #[serde(default)]
    pub is_reversing: bool,
}

fn default_currency() -> String { "USD".to_string() }

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddRuleTargetRequest {
    pub department_id: Option<Uuid>,
    pub department_name: Option<String>,
    pub cost_center: Option<String>,
    pub project_id: Option<Uuid>,
    pub project_name: Option<String>,
    pub target_account_code: String,
    pub fixed_percent: Option<String>,
    pub fixed_amount: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteRuleRequest {
    pub source_amount: String,
    pub period_start: chrono::NaiveDate,
    pub period_end: chrono::NaiveDate,
}

#[derive(Debug, Deserialize)]
pub struct ReverseRunRequest {
    pub reason: String,
}

#[derive(Debug, Deserialize)]
pub struct RuleFilters {
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RunFilters {
    pub rule_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct BaseValueFilters {
    pub base_id: Option<Uuid>,
}

fn error_response(e: atlas_shared::AtlasError) -> (axum::http::StatusCode, Json<serde_json::Value>) {
    let status = axum::http::StatusCode::from_u16(e.status_code()).unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
    (status, Json(json!({"error": e.to_string()})))
}

// ============================================================================
// Pool Handlers
// ============================================================================

pub async fn create_allocation_pool(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreatePoolRequest>,
) -> Result<(axum::http::StatusCode, Json<serde_json::Value>), (axum::http::StatusCode, Json<serde_json::Value>)> {
    let user_id = uuid::Uuid::parse_str(&claims.sub).ok();
    let org_id = uuid::Uuid::parse_str(&claims.org_id).unwrap_or_default();
    match state.cost_allocation_engine.create_pool(
        org_id, &req.code, &req.name, req.description.as_deref(),
        &req.pool_type, req.source_account_codes,
        req.source_department_id, req.source_cost_center.as_deref(),
        user_id,
    ).await {
        Ok(pool) => Ok((axum::http::StatusCode::CREATED, Json(serde_json::to_value(pool).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => Err(error_response(e)),
    }
}

pub async fn list_allocation_pools(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let org_id = uuid::Uuid::parse_str(&claims.org_id).unwrap_or_default();
    match state.cost_allocation_engine.list_pools(org_id).await {
        Ok(pools) => Ok(Json(json!({"data": pools}))),
        Err(e) => Err(error_response(e)),
    }
}

pub async fn get_allocation_pool(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(code): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let org_id = uuid::Uuid::parse_str(&claims.org_id).unwrap_or_default();
    match state.cost_allocation_engine.get_pool(org_id, &code).await {
        Ok(Some(pool)) => Ok(Json(serde_json::to_value(pool).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Ok(None) => Err((axum::http::StatusCode::NOT_FOUND, Json(json!({"error": "Pool not found"})))),
        Err(e) => Err(error_response(e)),
    }
}

pub async fn delete_allocation_pool(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(code): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let org_id = uuid::Uuid::parse_str(&claims.org_id).unwrap_or_default();
    match state.cost_allocation_engine.delete_pool(org_id, &code).await {
        Ok(()) => Ok(Json(json!({"message": "Pool deleted"}))),
        Err(e) => Err(error_response(e)),
    }
}

// ============================================================================
// Base Handlers
// ============================================================================

pub async fn create_allocation_base(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreateBaseRequest>,
) -> Result<(axum::http::StatusCode, Json<serde_json::Value>), (axum::http::StatusCode, Json<serde_json::Value>)> {
    let user_id = uuid::Uuid::parse_str(&claims.sub).ok();
    let org_id = uuid::Uuid::parse_str(&claims.org_id).unwrap_or_default();
    match state.cost_allocation_engine.create_base(
        org_id, &req.code, &req.name, req.description.as_deref(),
        &req.base_type, req.financial_account_code.as_deref(),
        req.unit_of_measure.as_deref(), user_id,
    ).await {
        Ok(base) => Ok((axum::http::StatusCode::CREATED, Json(serde_json::to_value(base).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => Err(error_response(e)),
    }
}

pub async fn list_allocation_bases(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let org_id = uuid::Uuid::parse_str(&claims.org_id).unwrap_or_default();
    match state.cost_allocation_engine.list_bases(org_id).await {
        Ok(bases) => Ok(Json(json!({"data": bases}))),
        Err(e) => Err(error_response(e)),
    }
}

pub async fn get_allocation_base(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(code): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let org_id = uuid::Uuid::parse_str(&claims.org_id).unwrap_or_default();
    match state.cost_allocation_engine.get_base(org_id, &code).await {
        Ok(Some(base)) => Ok(Json(serde_json::to_value(base).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Ok(None) => Err((axum::http::StatusCode::NOT_FOUND, Json(json!({"error": "Base not found"})))),
        Err(e) => Err(error_response(e)),
    }
}

pub async fn delete_allocation_base(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(code): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let org_id = uuid::Uuid::parse_str(&claims.org_id).unwrap_or_default();
    match state.cost_allocation_engine.delete_base(org_id, &code).await {
        Ok(()) => Ok(Json(json!({"message": "Base deleted"}))),
        Err(e) => Err(error_response(e)),
    }
}

// ============================================================================
// Base Value Handlers
// ============================================================================

pub async fn set_base_value(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<SetBaseValueRequest>,
) -> Result<(axum::http::StatusCode, Json<serde_json::Value>), (axum::http::StatusCode, Json<serde_json::Value>)> {
    let user_id = uuid::Uuid::parse_str(&claims.sub).ok();
    let org_id = uuid::Uuid::parse_str(&claims.org_id).unwrap_or_default();
    match state.cost_allocation_engine.set_base_value(
        org_id, &req.base_code,
        req.department_id, req.department_name.as_deref(),
        req.cost_center.as_deref(), req.project_id,
        &req.value, req.effective_date, &req.source, user_id,
    ).await {
        Ok(val) => Ok((axum::http::StatusCode::CREATED, Json(serde_json::to_value(val).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => Err(error_response(e)),
    }
}

pub async fn list_base_values(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(filters): Query<BaseValueFilters>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let org_id = uuid::Uuid::parse_str(&claims.org_id).unwrap_or_default();
    match state.cost_allocation_engine.list_base_values(org_id, filters.base_id).await {
        Ok(values) => Ok(Json(json!({"data": values}))),
        Err(e) => Err(error_response(e)),
    }
}

// ============================================================================
// Rule Handlers
// ============================================================================

pub async fn create_allocation_rule(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreateRuleRequest>,
) -> Result<(axum::http::StatusCode, Json<serde_json::Value>), (axum::http::StatusCode, Json<serde_json::Value>)> {
    let user_id = uuid::Uuid::parse_str(&claims.sub).ok();
    let org_id = uuid::Uuid::parse_str(&claims.org_id).unwrap_or_default();
    match state.cost_allocation_engine.create_rule(
        org_id, &req.name, req.description.as_deref(),
        &req.pool_code, &req.base_code, &req.allocation_method,
        req.journal_description.as_deref(), req.offset_account_code.as_deref(),
        &req.currency_code, req.is_reversing, user_id,
    ).await {
        Ok(rule) => Ok((axum::http::StatusCode::CREATED, Json(serde_json::to_value(rule).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => Err(error_response(e)),
    }
}

pub async fn list_allocation_rules(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(filters): Query<RuleFilters>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let org_id = uuid::Uuid::parse_str(&claims.org_id).unwrap_or_default();
    match state.cost_allocation_engine.list_rules(org_id, filters.status.as_deref()).await {
        Ok(rules) => Ok(Json(json!({"data": rules}))),
        Err(e) => Err(error_response(e)),
    }
}

pub async fn get_allocation_rule(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    match state.cost_allocation_engine.get_rule(id).await {
        Ok(Some(rule)) => Ok(Json(serde_json::to_value(rule).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Ok(None) => Err((axum::http::StatusCode::NOT_FOUND, Json(json!({"error": "Rule not found"})))),
        Err(e) => Err(error_response(e)),
    }
}

pub async fn activate_allocation_rule(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    match state.cost_allocation_engine.activate_rule(id).await {
        Ok(rule) => Ok(Json(serde_json::to_value(rule).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err(error_response(e)),
    }
}

pub async fn deactivate_allocation_rule(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    match state.cost_allocation_engine.deactivate_rule(id).await {
        Ok(rule) => Ok(Json(serde_json::to_value(rule).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err(error_response(e)),
    }
}

// ============================================================================
// Rule Target Handlers
// ============================================================================

pub async fn add_rule_target(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(rule_id): Path<Uuid>,
    Json(req): Json<AddRuleTargetRequest>,
) -> Result<(axum::http::StatusCode, Json<serde_json::Value>), (axum::http::StatusCode, Json<serde_json::Value>)> {
    let org_id = uuid::Uuid::parse_str(&claims.org_id).unwrap_or_default();
    match state.cost_allocation_engine.add_rule_target(
        org_id, rule_id,
        req.department_id, req.department_name.as_deref(),
        req.cost_center.as_deref(), req.project_id, req.project_name.as_deref(),
        &req.target_account_code, req.fixed_percent.as_deref(), req.fixed_amount.as_deref(),
    ).await {
        Ok(target) => Ok((axum::http::StatusCode::CREATED, Json(serde_json::to_value(target).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => Err(error_response(e)),
    }
}

pub async fn list_rule_targets(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(rule_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    match state.cost_allocation_engine.list_rule_targets(rule_id).await {
        Ok(targets) => Ok(Json(json!({"data": targets}))),
        Err(e) => Err(error_response(e)),
    }
}

// ============================================================================
// Run Handlers
// ============================================================================

pub async fn execute_allocation_rule(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(rule_id): Path<Uuid>,
    Json(req): Json<ExecuteRuleRequest>,
) -> Result<(axum::http::StatusCode, Json<serde_json::Value>), (axum::http::StatusCode, Json<serde_json::Value>)> {
    let user_id = uuid::Uuid::parse_str(&claims.sub).ok();
    let org_id = uuid::Uuid::parse_str(&claims.org_id).unwrap_or_default();
    match state.cost_allocation_engine.execute_rule(
        org_id, rule_id, &req.source_amount,
        req.period_start, req.period_end, user_id,
    ).await {
        Ok(run) => Ok((axum::http::StatusCode::CREATED, Json(serde_json::to_value(run).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => Err(error_response(e)),
    }
}

pub async fn list_allocation_runs(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(filters): Query<RunFilters>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let org_id = uuid::Uuid::parse_str(&claims.org_id).unwrap_or_default();
    match state.cost_allocation_engine.list_runs(org_id, filters.rule_id).await {
        Ok(runs) => Ok(Json(json!({"data": runs}))),
        Err(e) => Err(error_response(e)),
    }
}

pub async fn get_allocation_run(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    match state.cost_allocation_engine.get_run(id).await {
        Ok(Some(run)) => Ok(Json(serde_json::to_value(run).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Ok(None) => Err((axum::http::StatusCode::NOT_FOUND, Json(json!({"error": "Run not found"})))),
        Err(e) => Err(error_response(e)),
    }
}

pub async fn post_allocation_run(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let user_id = uuid::Uuid::parse_str(&claims.sub).ok();
    match state.cost_allocation_engine.post_run(id, user_id).await {
        Ok(run) => Ok(Json(serde_json::to_value(run).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err(error_response(e)),
    }
}

pub async fn reverse_allocation_run(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(req): Json<ReverseRunRequest>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let user_id = uuid::Uuid::parse_str(&claims.sub).ok();
    match state.cost_allocation_engine.reverse_run(id, user_id, &req.reason).await {
        Ok(run) => Ok(Json(serde_json::to_value(run).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err(error_response(e)),
    }
}

pub async fn list_allocation_run_lines(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(run_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    match state.cost_allocation_engine.list_run_lines(run_id).await {
        Ok(lines) => Ok(Json(json!({"data": lines}))),
        Err(e) => Err(error_response(e)),
    }
}

// ============================================================================
// Dashboard
// ============================================================================

pub async fn get_allocation_summary(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let org_id = uuid::Uuid::parse_str(&claims.org_id).unwrap_or_default();
    match state.cost_allocation_engine.get_summary(org_id).await {
        Ok(summary) => Ok(Json(serde_json::to_value(summary).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err(error_response(e)),
    }
}
