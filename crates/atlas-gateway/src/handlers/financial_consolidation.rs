//! Financial Consolidation Handlers
//!
//! Oracle Fusion Cloud ERP: General Ledger > Financial Consolidation
//!
//! API endpoints for multi-entity financial consolidation including:
//! - Consolidation ledgers
//! - Consolidation entities
//! - Consolidation scenarios (execute, approve, post, reverse)
//! - Elimination rules
//! - Consolidation adjustments
//! - Currency translation rates
//! - Consolidated trial balance

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
// Ledger Management
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateLedgerRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub base_currency_code: String,
    pub translation_method: String,
    pub equity_elimination_method: String,
}

pub async fn create_ledger(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateLedgerRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.financial_consolidation_engine.create_ledger(
        org_id, &payload.code, &payload.name, payload.description.as_deref(),
        &payload.base_currency_code, &payload.translation_method,
        &payload.equity_elimination_method, Some(user_id),
    ).await {
        Ok(ledger) => Ok((StatusCode::CREATED, Json(serde_json::to_value(ledger).unwrap()))),
        Err(e) => {
            error!("Failed to create consolidation ledger: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn list_ledgers(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.financial_consolidation_engine.list_ledgers(org_id, false).await {
        Ok(ledgers) => Ok(Json(serde_json::json!({ "data": ledgers }))),
        Err(e) => { error!("Failed to list ledgers: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn get_ledger(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(code): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.financial_consolidation_engine.get_ledger(org_id, &code).await {
        Ok(Some(l)) => Ok(Json(serde_json::to_value(l).unwrap())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Failed to get ledger: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// ============================================================================
// Entity Management
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct AddEntityRequest {
    pub entity_id: Uuid,
    pub entity_name: String,
    pub entity_code: String,
    pub local_currency_code: String,
    pub ownership_percentage: String,
    pub consolidation_method: String,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
}

pub async fn add_entity(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(ledger_id): Path<Uuid>,
    Json(payload): Json<AddEntityRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.financial_consolidation_engine.add_entity(
        org_id, ledger_id, payload.entity_id, &payload.entity_name,
        &payload.entity_code, &payload.local_currency_code,
        &payload.ownership_percentage, &payload.consolidation_method,
        payload.effective_from, payload.effective_to, Some(user_id),
    ).await {
        Ok(entity) => Ok((StatusCode::CREATED, Json(serde_json::to_value(entity).unwrap()))),
        Err(e) => {
            error!("Failed to add consolidation entity: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn list_entities(
    State(state): State<Arc<AppState>>,
    Path(ledger_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.financial_consolidation_engine.list_entities(ledger_id, false).await {
        Ok(entities) => Ok(Json(serde_json::json!({ "data": entities }))),
        Err(e) => { error!("Failed to list entities: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// ============================================================================
// Scenario Management
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateScenarioRequest {
    pub ledger_id: Uuid,
    pub scenario_number: String,
    pub name: String,
    pub description: Option<String>,
    pub fiscal_year: i32,
    pub period_name: String,
    pub period_start: chrono::NaiveDate,
    pub period_end: chrono::NaiveDate,
    pub translation_rate_type: Option<String>,
}

pub async fn create_scenario(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateScenarioRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.financial_consolidation_engine.create_scenario(
        org_id, payload.ledger_id, &payload.scenario_number, &payload.name,
        payload.description.as_deref(), payload.fiscal_year, &payload.period_name,
        payload.period_start, payload.period_end,
        payload.translation_rate_type.as_deref(), Some(user_id),
    ).await {
        Ok(scenario) => Ok((StatusCode::CREATED, Json(serde_json::to_value(scenario).unwrap()))),
        Err(e) => {
            error!("Failed to create scenario: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn list_scenarios(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.financial_consolidation_engine.list_scenarios(org_id, None, None).await {
        Ok(scenarios) => Ok(Json(serde_json::json!({ "data": scenarios }))),
        Err(e) => { error!("Failed to list scenarios: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn execute_consolidation(
    State(state): State<Arc<AppState>>,
    Path(scenario_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.financial_consolidation_engine.execute_consolidation(scenario_id).await {
        Ok(s) => Ok(Json(serde_json::to_value(s).unwrap())),
        Err(e) => {
            error!("Failed to execute consolidation: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn approve_scenario(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(scenario_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.financial_consolidation_engine.approve_scenario(scenario_id, user_id).await {
        Ok(s) => Ok(Json(serde_json::to_value(s).unwrap())),
        Err(e) => {
            error!("Failed to approve scenario: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn post_scenario(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(scenario_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.financial_consolidation_engine.post_scenario(scenario_id, user_id).await {
        Ok(s) => Ok(Json(serde_json::to_value(s).unwrap())),
        Err(e) => {
            error!("Failed to post scenario: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn reverse_scenario(
    State(state): State<Arc<AppState>>,
    Path(scenario_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.financial_consolidation_engine.reverse_scenario(scenario_id).await {
        Ok(s) => Ok(Json(serde_json::to_value(s).unwrap())),
        Err(e) => {
            error!("Failed to reverse scenario: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// ============================================================================
// Elimination Rules
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateEliminationRuleRequest {
    pub ledger_id: Uuid,
    pub rule_code: String,
    pub name: String,
    pub description: Option<String>,
    pub elimination_type: String,
    pub offset_account_code: String,
}

pub async fn create_elimination_rule(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateEliminationRuleRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.financial_consolidation_engine.create_elimination_rule(
        org_id, payload.ledger_id, &payload.rule_code, &payload.name,
        payload.description.as_deref(), &payload.elimination_type,
        None, None, None, None,
        &payload.offset_account_code, 10, Some(user_id),
    ).await {
        Ok(rule) => Ok((StatusCode::CREATED, Json(serde_json::to_value(rule).unwrap()))),
        Err(e) => {
            error!("Failed to create elimination rule: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn list_elimination_rules(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let _org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    // List rules from the first available ledger (requires ledger_id in real usage)
    // For now, return empty as the user should specify a ledger
    Ok(Json(serde_json::json!({ "data": [] })))
}

// ============================================================================
// Dashboard
// ============================================================================

pub async fn get_consolidation_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.financial_consolidation_engine.get_dashboard_summary(org_id).await {
        Ok(dashboard) => Ok(Json(serde_json::to_value(dashboard).unwrap())),
        Err(e) => { error!("Failed to get consolidation dashboard: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}
