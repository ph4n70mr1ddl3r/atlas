//! Cross-Validation Rules API Handlers
//!
//! REST API for managing Cross-Validation Rules (CVR).
//! Oracle Fusion equivalent: General Ledger > Setup > Chart of Accounts >
//!   Cross-Validation Rules

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension,
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;
use uuid::Uuid;
use tracing::{info, error};
use crate::AppState;
use crate::handlers::auth::Claims;

// ═══════════════════════════════════════════════════════════════════════════════
// Rules
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Deserialize)]
pub struct CreateRuleRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub rule_type: String,
    pub error_message: String,
    pub priority: Option<i32>,
    pub segment_names: Vec<String>,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
}

#[derive(Deserialize)]
pub struct ListRulesQuery {
    pub enabled_only: Option<bool>,
}

pub async fn create_rule(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(body): Json<CreateRuleRequest>,
) -> Result<(StatusCode, Json<Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).ok();

    match state.cvr_engine.create_rule(
        org_id,
        &body.code,
        &body.name,
        body.description.as_deref(),
        &body.rule_type,
        &body.error_message,
        body.priority.unwrap_or(10),
        body.segment_names,
        body.effective_from,
        body.effective_to,
        user_id,
    ).await {
        Ok(rule) => {
            info!("Created cross-validation rule '{}' for org {}", rule.code, org_id);
            Ok((StatusCode::CREATED, Json(serde_json::to_value(rule).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))))
        }
        Err(e) => {
            error!("Failed to create cross-validation rule: {}", e);
            Err(map_error_status(&e))
        }
    }
}

pub async fn list_rules(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListRulesQuery>,
) -> Result<Json<Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.cvr_engine.list_rules(org_id, query.enabled_only.unwrap_or(false)).await {
        Ok(list) => Ok(Json(json!(list))),
        Err(e) => {
            error!("Failed to list cross-validation rules: {}", e);
            Err(map_error_status(&e))
        }
    }
}

pub async fn get_rule(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(code): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.cvr_engine.get_rule(org_id, &code).await {
        Ok(Some(rule)) => Ok(Json(serde_json::to_value(rule).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => Err(map_error_status(&e)),
    }
}

pub async fn enable_rule(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, StatusCode> {
    match state.cvr_engine.enable_rule(id).await {
        Ok(rule) => {
            info!("Enabled cross-validation rule '{}'", rule.code);
            Ok(Json(serde_json::to_value(rule).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))
        }
        Err(e) => Err(map_error_status(&e)),
    }
}

pub async fn disable_rule(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, StatusCode> {
    match state.cvr_engine.disable_rule(id).await {
        Ok(rule) => {
            info!("Disabled cross-validation rule '{}'", rule.code);
            Ok(Json(serde_json::to_value(rule).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))
        }
        Err(e) => Err(map_error_status(&e)),
    }
}

pub async fn delete_rule(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(code): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.cvr_engine.delete_rule(org_id, &code).await {
        Ok(()) => {
            info!("Deleted cross-validation rule '{}'", code);
            Ok(StatusCode::NO_CONTENT)
        }
        Err(e) => Err(map_error_status(&e)),
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Rule Lines
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Deserialize)]
pub struct CreateRuleLineRequest {
    pub line_type: String,
    pub patterns: Vec<String>,
    pub display_order: Option<i32>,
}

pub async fn create_rule_line(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(rule_code): Path<String>,
    Json(body): Json<CreateRuleLineRequest>,
) -> Result<(StatusCode, Json<Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.cvr_engine.create_rule_line(
        org_id,
        &rule_code,
        &body.line_type,
        body.patterns,
        body.display_order.unwrap_or(1),
    ).await {
        Ok(line) => {
            info!("Created {} line for rule {}", line.line_type, rule_code);
            Ok((StatusCode::CREATED, Json(serde_json::to_value(line).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))))
        }
        Err(e) => {
            error!("Failed to create rule line: {}", e);
            Err(map_error_status(&e))
        }
    }
}

pub async fn list_rule_lines(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(rule_code): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let rule = state.cvr_engine.get_rule(org_id, &rule_code).await
        .map_err(|e| map_error_status(&e))?
        .ok_or(StatusCode::NOT_FOUND)?;

    match state.cvr_engine.list_rule_lines(rule.id).await {
        Ok(lines) => Ok(Json(json!(lines))),
        Err(e) => Err(map_error_status(&e)),
    }
}

pub async fn delete_rule_line(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    match state.cvr_engine.delete_rule_line(id).await {
        Ok(()) => {
            info!("Deleted cross-validation rule line {}", id);
            Ok(StatusCode::NO_CONTENT)
        }
        Err(e) => Err(map_error_status(&e)),
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Validation
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Deserialize)]
pub struct ValidateCombinationRequest {
    pub segment_values: Vec<String>,
}

pub async fn validate_combination(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(body): Json<ValidateCombinationRequest>,
) -> Result<Json<Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.cvr_engine.validate_combination(org_id, &body.segment_values).await {
        Ok(result) => Ok(Json(serde_json::to_value(result).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => {
            error!("Failed to validate combination: {}", e);
            Err(map_error_status(&e))
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Dashboard
// ═══════════════════════════════════════════════════════════════════════════════

pub async fn get_cvr_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.cvr_engine.get_dashboard_summary(org_id).await {
        Ok(summary) => Ok(Json(serde_json::to_value(summary).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err(map_error_status(&e)),
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════════════════════

fn map_error_status(e: &atlas_shared::AtlasError) -> StatusCode {
    use atlas_shared::AtlasError;
    match e {
        AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
        AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
        AtlasError::Conflict(_) => StatusCode::CONFLICT,
        AtlasError::WorkflowError(_) => StatusCode::CONFLICT,
        AtlasError::ConfigError(_) => StatusCode::BAD_REQUEST,
        AtlasError::DatabaseError(msg) => {
            error!("Database error: {}", msg);
            StatusCode::INTERNAL_SERVER_ERROR
        }
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
