//! Compensation Management Handlers
//!
//! Oracle Fusion Cloud HCM: Compensation Workbench
//!
//! API endpoints for managing compensation plans, cycles, budget pools,
//! manager worksheets, and employee compensation statements.

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
// Plan Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreatePlanRequest {
    pub plan_code: String,
    pub plan_name: String,
    pub description: Option<String>,
    pub plan_type: String,
    pub effective_start_date: Option<chrono::NaiveDate>,
    pub effective_end_date: Option<chrono::NaiveDate>,
    pub eligibility_criteria: Option<serde_json::Value>,
}

pub async fn create_plan(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreatePlanRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.compensation_engine.create_plan(
        org_id, &payload.plan_code, &payload.plan_name,
        payload.description.as_deref(), &payload.plan_type,
        payload.effective_start_date, payload.effective_end_date,
        payload.eligibility_criteria.unwrap_or(serde_json::json!({})),
        Some(user_id),
    ).await {
        Ok(plan) => Ok((StatusCode::CREATED, Json(serde_json::to_value(plan).unwrap_or_default()))),
        Err(e) => {
            error!("Failed to create compensation plan: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 409 => StatusCode::CONFLICT, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

pub async fn get_plan(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(code): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.compensation_engine.get_plan_by_code(org_id, &code).await {
        Ok(Some(p)) => Ok(Json(serde_json::to_value(p).unwrap_or_default())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn list_plans(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.compensation_engine.list_plans(org_id).await {
        Ok(plans) => Ok(Json(serde_json::json!({"data": plans}))),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn delete_plan(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(code): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.compensation_engine.delete_plan(org_id, &code).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// ============================================================================
// Component Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateComponentRequest {
    pub component_name: String,
    pub component_type: String,
    pub description: Option<String>,
    pub is_recurring: Option<bool>,
    pub frequency: Option<String>,
}

pub async fn create_component(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(plan_code): Path<String>,
    Json(payload): Json<CreateComponentRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let plan = state.compensation_engine.get_plan_by_code(org_id, &plan_code).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    match state.compensation_engine.create_component(
        org_id, plan.id, &payload.component_name, &payload.component_type,
        payload.description.as_deref(),
        payload.is_recurring.unwrap_or(true),
        payload.frequency.as_deref(),
    ).await {
        Ok(comp) => Ok((StatusCode::CREATED, Json(serde_json::to_value(comp).unwrap_or_default()))),
        Err(e) => {
            error!("Failed to create component: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

pub async fn list_components(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(plan_code): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let plan = state.compensation_engine.get_plan_by_code(org_id, &plan_code).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    match state.compensation_engine.list_components(plan.id).await {
        Ok(comps) => Ok(Json(serde_json::json!({"data": comps}))),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// ============================================================================
// Cycle Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateCycleRequest {
    pub cycle_name: String,
    pub description: Option<String>,
    pub cycle_type: String,
    pub start_date: chrono::NaiveDate,
    pub end_date: chrono::NaiveDate,
    pub total_budget: String,
    pub currency_code: Option<String>,
}

pub async fn create_cycle(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateCycleRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.compensation_engine.create_cycle(
        org_id, &payload.cycle_name, payload.description.as_deref(),
        &payload.cycle_type, payload.start_date, payload.end_date,
        &payload.total_budget, payload.currency_code.as_deref().unwrap_or("USD"),
        Some(user_id),
    ).await {
        Ok(cycle) => Ok((StatusCode::CREATED, Json(serde_json::to_value(cycle).unwrap_or_default()))),
        Err(e) => {
            error!("Failed to create cycle: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

pub async fn get_cycle(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.compensation_engine.get_cycle(id).await {
        Ok(Some(c)) => Ok(Json(serde_json::to_value(c).unwrap_or_default())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListCyclesQuery {
    pub status: Option<String>,
}

pub async fn list_cycles(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListCyclesQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.compensation_engine.list_cycles(org_id, query.status.as_deref()).await {
        Ok(cycles) => Ok(Json(serde_json::json!({"data": cycles}))),
        Err(e) => {
            error!("Error: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct TransitionCycleRequest {
    pub status: String,
}

pub async fn transition_cycle(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<TransitionCycleRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.compensation_engine.transition_cycle(id, &payload.status).await {
        Ok(cycle) => Ok(Json(serde_json::to_value(cycle).unwrap_or_default())),
        Err(e) => {
            error!("Failed to transition cycle: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

pub async fn list_budget_pools(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(cycle_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.compensation_engine.list_budget_pools(cycle_id).await {
        Ok(pools) => Ok(Json(serde_json::json!({"data": pools}))),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// ============================================================================
// Budget Pool Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateBudgetPoolRequest {
    pub pool_name: String,
    pub pool_type: String,
    pub manager_id: Option<Uuid>,
    pub manager_name: Option<String>,
    pub department_id: Option<Uuid>,
    pub department_name: Option<String>,
    pub total_budget: String,
    pub currency_code: Option<String>,
}

pub async fn create_budget_pool(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(cycle_id): Path<Uuid>,
    Json(payload): Json<CreateBudgetPoolRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.compensation_engine.create_budget_pool(
        org_id, cycle_id, &payload.pool_name, &payload.pool_type,
        payload.manager_id, payload.manager_name.as_deref(),
        payload.department_id, payload.department_name.as_deref(),
        &payload.total_budget, payload.currency_code.as_deref().unwrap_or("USD"),
        Some(user_id),
    ).await {
        Ok(pool) => Ok((StatusCode::CREATED, Json(serde_json::to_value(pool).unwrap_or_default()))),
        Err(e) => {
            error!("Failed to create budget pool: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 404 => StatusCode::NOT_FOUND, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

pub async fn delete_cycle(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    match state.compensation_engine.delete_cycle(id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            error!("Error: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 404 => StatusCode::NOT_FOUND, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

// ============================================================================
// Worksheet Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateWorksheetRequest {
    pub pool_id: Option<Uuid>,
    pub manager_id: Uuid,
    pub manager_name: Option<String>,
}

pub async fn create_worksheet(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(cycle_id): Path<Uuid>,
    Json(payload): Json<CreateWorksheetRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.compensation_engine.create_worksheet(
        org_id, cycle_id, payload.pool_id, payload.manager_id,
        payload.manager_name.as_deref(), Some(user_id),
    ).await {
        Ok(ws) => Ok((StatusCode::CREATED, Json(serde_json::to_value(ws).unwrap_or_default()))),
        Err(e) => {
            error!("Failed to create worksheet: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 404 => StatusCode::NOT_FOUND, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

pub async fn get_worksheet(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.compensation_engine.get_worksheet(id).await {
        Ok(Some(ws)) => Ok(Json(serde_json::to_value(ws).unwrap_or_default())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListWorksheetsQuery {
    pub status: Option<String>,
}

pub async fn list_worksheets(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(cycle_id): Path<Uuid>,
    Query(query): Query<ListWorksheetsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.compensation_engine.list_worksheets(cycle_id, query.status.as_deref()).await {
        Ok(wss) => Ok(Json(serde_json::json!({"data": wss}))),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn submit_worksheet(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.compensation_engine.submit_worksheet(id).await {
        Ok(ws) => Ok(Json(serde_json::to_value(ws).unwrap_or_default())),
        Err(e) => {
            error!("Failed to submit worksheet: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 404 => StatusCode::NOT_FOUND, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

pub async fn approve_worksheet(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.compensation_engine.approve_worksheet(id).await {
        Ok(ws) => Ok(Json(serde_json::to_value(ws).unwrap_or_default())),
        Err(e) => {
            error!("Failed to approve worksheet: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 404 => StatusCode::NOT_FOUND, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

pub async fn reject_worksheet(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.compensation_engine.reject_worksheet(id).await {
        Ok(ws) => Ok(Json(serde_json::to_value(ws).unwrap_or_default())),
        Err(e) => {
            error!("Failed to reject worksheet: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 404 => StatusCode::NOT_FOUND, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

// ============================================================================
// Worksheet Line Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct AddWorksheetLineRequest {
    pub employee_id: Uuid,
    pub employee_name: Option<String>,
    pub job_title: Option<String>,
    pub department_name: Option<String>,
    pub current_base_salary: String,
    pub proposed_base_salary: String,
    pub merit_amount: Option<String>,
    pub bonus_amount: Option<String>,
    pub equity_amount: Option<String>,
    pub performance_rating: Option<String>,
    pub manager_comments: Option<String>,
}

pub async fn add_worksheet_line(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(worksheet_id): Path<Uuid>,
    Json(payload): Json<AddWorksheetLineRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.compensation_engine.add_worksheet_line(
        org_id, worksheet_id, payload.employee_id,
        payload.employee_name.as_deref(), payload.job_title.as_deref(),
        payload.department_name.as_deref(),
        &payload.current_base_salary, &payload.proposed_base_salary,
        payload.merit_amount.as_deref().unwrap_or("0"),
        payload.bonus_amount.as_deref().unwrap_or("0"),
        payload.equity_amount.as_deref().unwrap_or("0"),
        payload.performance_rating.as_deref(),
        payload.manager_comments.as_deref(),
        Some(user_id),
    ).await {
        Ok(line) => Ok((StatusCode::CREATED, Json(serde_json::to_value(line).unwrap_or_default()))),
        Err(e) => {
            error!("Failed to add worksheet line: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 404 => StatusCode::NOT_FOUND, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateWorksheetLineRequest {
    pub proposed_base_salary: String,
    pub merit_amount: Option<String>,
    pub bonus_amount: Option<String>,
    pub equity_amount: Option<String>,
    pub manager_comments: Option<String>,
}

pub async fn update_worksheet_line(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(line_id): Path<Uuid>,
    Json(payload): Json<UpdateWorksheetLineRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.compensation_engine.update_worksheet_line(
        line_id,
        &payload.proposed_base_salary,
        payload.merit_amount.as_deref().unwrap_or("0"),
        payload.bonus_amount.as_deref().unwrap_or("0"),
        payload.equity_amount.as_deref().unwrap_or("0"),
        payload.manager_comments.as_deref(),
    ).await {
        Ok(line) => Ok(Json(serde_json::to_value(line).unwrap_or_default())),
        Err(e) => {
            error!("Failed to update worksheet line: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 404 => StatusCode::NOT_FOUND, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

pub async fn list_worksheet_lines(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(worksheet_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.compensation_engine.list_worksheet_lines(worksheet_id).await {
        Ok(lines) => Ok(Json(serde_json::json!({"data": lines}))),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn delete_worksheet_line(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(line_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    match state.compensation_engine.delete_worksheet_line(line_id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            error!("Error: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 404 => StatusCode::NOT_FOUND, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

// ============================================================================
// Statement Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct GenerateStatementRequest {
    pub employee_id: Uuid,
    pub employee_name: Option<String>,
    pub base_salary: String,
    pub merit_increase: Option<String>,
    pub bonus: Option<String>,
    pub equity: Option<String>,
    pub benefits_value: Option<String>,
    pub currency_code: Option<String>,
    pub components: Option<serde_json::Value>,
}

pub async fn generate_statement(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(cycle_id): Path<Uuid>,
    Json(payload): Json<GenerateStatementRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.compensation_engine.generate_statement(
        org_id, cycle_id, payload.employee_id,
        payload.employee_name.as_deref(),
        &payload.base_salary,
        payload.merit_increase.as_deref().unwrap_or("0"),
        payload.bonus.as_deref().unwrap_or("0"),
        payload.equity.as_deref().unwrap_or("0"),
        payload.benefits_value.as_deref().unwrap_or("0"),
        payload.currency_code.as_deref().unwrap_or("USD"),
        payload.components.unwrap_or(serde_json::json!([])),
    ).await {
        Ok(stmt) => Ok((StatusCode::CREATED, Json(serde_json::to_value(stmt).unwrap_or_default()))),
        Err(e) => {
            error!("Failed to generate statement: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 404 => StatusCode::NOT_FOUND, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

pub async fn get_statement(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.compensation_engine.get_statement(id).await {
        Ok(Some(s)) => Ok(Json(serde_json::to_value(s).unwrap_or_default())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn list_statements(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(cycle_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.compensation_engine.list_statements(cycle_id).await {
        Ok(stmts) => Ok(Json(serde_json::json!({"data": stmts}))),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn publish_statement(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.compensation_engine.publish_statement(id).await {
        Ok(s) => Ok(Json(serde_json::to_value(s).unwrap_or_default())),
        Err(e) => {
            error!("Failed to publish statement: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 404 => StatusCode::NOT_FOUND, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

// ============================================================================
// Dashboard Handler
// ============================================================================

pub async fn get_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.compensation_engine.get_dashboard(org_id).await {
        Ok(d) => Ok(Json(serde_json::to_value(d).unwrap_or_default())),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}
