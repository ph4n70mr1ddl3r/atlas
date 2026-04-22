//! Segregation of Duties API Handlers
//!
//! Oracle Fusion Cloud ERP: Advanced Access Control > Segregation of Duties
//!
//! Endpoints for managing SoD rules, role assignments, conflict detection,
//! violations, mitigating controls, and compliance dashboard.

use axum::{
    extract::{Path, Query, State, Extension},
    Json,
    http::StatusCode,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;
use uuid::Uuid;

use crate::AppState;
use crate::handlers::auth::Claims;

// ============================================================================
// Query Parameters
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ListRulesQuery {
    pub active_only: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ListViolationsQuery {
    pub user_id: Option<String>,
    pub status: Option<String>,
    pub risk_level: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListAssignmentsQuery {
    pub user_id: Option<String>,
}

// ============================================================================
// SoD Rule Management
// ============================================================================

/// Create a new SoD rule
pub async fn create_sod_rule(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    let code = body["code"].as_str().unwrap_or("").to_string();
    let name = body["name"].as_str().unwrap_or("").to_string();
    let description = body["description"].as_str();
    let first_duties: Vec<String> = body["first_duties"].as_array()
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();
    let second_duties: Vec<String> = body["second_duties"].as_array()
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();
    let enforcement_mode = body["enforcement_mode"].as_str().unwrap_or("detective").to_string();
    let risk_level = body["risk_level"].as_str().unwrap_or("medium").to_string();
    let effective_from = body["effective_from"].as_str()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let effective_to = body["effective_to"].as_str()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());

    match state.sod_engine.create_rule(
        org_id, &code, &name, description,
        first_duties, second_duties,
        &enforcement_mode, &risk_level,
        effective_from, effective_to,
        Some(claims.sub.parse().unwrap_or(Uuid::nil())),
    ).await {
        Ok(rule) => Ok((StatusCode::CREATED, Json(serde_json::to_value(rule).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Get a rule by code
pub async fn get_sod_rule(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(code): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = Uuid::parse_str(&claims.org_id).unwrap_or(Uuid::nil());
    match state.sod_engine.get_rule(org_id, &code).await {
        Ok(Some(rule)) => Ok(Json(serde_json::to_value(rule).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(json!({"error": "Rule not found"})))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))),
    }
}

/// List all rules
pub async fn list_sod_rules(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<ListRulesQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = Uuid::parse_str(&claims.org_id).unwrap_or(Uuid::nil());
    let active_only = query.active_only.unwrap_or(false);
    match state.sod_engine.list_rules(org_id, active_only).await {
        Ok(rules) => Ok(Json(json!({"data": rules}))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))),
    }
}

/// Activate a rule
pub async fn activate_sod_rule(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.sod_engine.activate_rule(id).await {
        Ok(rule) => Ok(Json(serde_json::to_value(rule).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Deactivate a rule
pub async fn deactivate_sod_rule(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.sod_engine.deactivate_rule(id).await {
        Ok(rule) => Ok(Json(serde_json::to_value(rule).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Delete a rule
pub async fn delete_sod_rule(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(code): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = Uuid::parse_str(&claims.org_id).unwrap_or(Uuid::nil());
    match state.sod_engine.delete_rule(org_id, &code).await {
        Ok(()) => Ok(Json(json!({"message": "Rule deleted"}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Role Assignment Management
// ============================================================================

/// Assign a role/duty to a user
pub async fn assign_sod_role(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    let user_id = body["user_id"].as_str()
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| (StatusCode::BAD_REQUEST, Json(json!({"error": "user_id is required"}))))?;
    let role_name = body["role_name"].as_str().unwrap_or("").to_string();
    let duty_code = body["duty_code"].as_str().unwrap_or("").to_string();

    match state.sod_engine.assign_role(
        org_id, user_id, &role_name, &duty_code,
        Some(claims.sub.parse().unwrap_or(Uuid::nil())),
    ).await {
        Ok(assignment) => Ok((StatusCode::CREATED, Json(serde_json::to_value(assignment).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// List role assignments
pub async fn list_sod_assignments(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<ListAssignmentsQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = Uuid::parse_str(&claims.org_id).unwrap_or(Uuid::nil());
    let user_id = query.user_id.as_ref().and_then(|s| Uuid::parse_str(s).ok());
    match state.sod_engine.list_role_assignments(org_id, user_id).await {
        Ok(assignments) => Ok(Json(json!({"data": assignments}))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))),
    }
}

/// Remove a role assignment
pub async fn remove_sod_assignment(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.sod_engine.remove_role_assignment(id).await {
        Ok(assignment) => Ok(Json(serde_json::to_value(assignment).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Conflict Detection
// ============================================================================

/// Check if a proposed role assignment would create conflicts
pub async fn check_sod_conflict(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    let user_id = body["user_id"].as_str()
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| (StatusCode::BAD_REQUEST, Json(json!({"error": "user_id is required"}))))?;
    let duty_code = body["duty_code"].as_str().unwrap_or("").to_string();

    match state.sod_engine.check_conflicts_for_assignment(org_id, user_id, &duty_code).await {
        Ok(result) => Ok(Json(serde_json::to_value(result).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))),
    }
}

/// Run full violation detection across all users
pub async fn run_sod_detection(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = Uuid::parse_str(&claims.org_id).unwrap_or(Uuid::nil());
    match state.sod_engine.run_full_detection(org_id).await {
        Ok(count) => Ok(Json(json!({"new_violations_detected": count}))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Violation Management
// ============================================================================

/// List violations
pub async fn list_sod_violations(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<ListViolationsQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = Uuid::parse_str(&claims.org_id).unwrap_or(Uuid::nil());
    let user_id = query.user_id.as_ref().and_then(|s| Uuid::parse_str(s).ok());
    match state.sod_engine.list_violations(
        org_id, user_id, query.status.as_deref(), query.risk_level.as_deref(),
    ).await {
        Ok(violations) => Ok(Json(json!({"data": violations}))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))),
    }
}

/// Get a violation by ID
pub async fn get_sod_violation(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.sod_engine.get_violation(id).await {
        Ok(Some(v)) => Ok(Json(serde_json::to_value(v).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(json!({"error": "Violation not found"})))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))),
    }
}

/// Resolve a violation
pub async fn resolve_sod_violation(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let resolved_by = claims.sub.parse().unwrap_or(Uuid::nil());
    match state.sod_engine.resolve_violation(id, resolved_by).await {
        Ok(v) => Ok(Json(serde_json::to_value(v).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Accept a violation as an exception
pub async fn accept_sod_exception(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let accepted_by = claims.sub.parse().unwrap_or(Uuid::nil());
    match state.sod_engine.accept_exception(id, accepted_by).await {
        Ok(v) => Ok(Json(serde_json::to_value(v).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Mitigating Controls
// ============================================================================

/// Add a mitigating control to a violation
pub async fn create_sod_mitigation(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    let violation_id = body["violation_id"].as_str()
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| (StatusCode::BAD_REQUEST, Json(json!({"error": "violation_id is required"}))))?;
    let control_name = body["control_name"].as_str().unwrap_or("").to_string();
    let control_description = body["control_description"].as_str().unwrap_or("").to_string();
    let control_owner_id = body["control_owner_id"].as_str().and_then(|s| Uuid::parse_str(s).ok());
    let review_frequency = body["review_frequency"].as_str().unwrap_or("monthly").to_string();
    let effective_from = body["effective_from"].as_str()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let effective_to = body["effective_to"].as_str()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());

    match state.sod_engine.add_mitigating_control(
        org_id, violation_id,
        &control_name, &control_description,
        control_owner_id, &review_frequency,
        effective_from, effective_to,
        Some(claims.sub.parse().unwrap_or(Uuid::nil())),
    ).await {
        Ok(control) => Ok((StatusCode::CREATED, Json(serde_json::to_value(control).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// List mitigating controls for a violation
pub async fn list_sod_mitigations(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(violation_id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.sod_engine.get_mitigating_controls(violation_id).await {
        Ok(controls) => Ok(Json(json!({"data": controls}))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))),
    }
}

/// Approve a mitigating control
pub async fn approve_sod_mitigation(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let approved_by = claims.sub.parse().unwrap_or(Uuid::nil());
    match state.sod_engine.approve_mitigating_control(id, approved_by).await {
        Ok(control) => Ok(Json(serde_json::to_value(control).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Revoke a mitigating control
pub async fn revoke_sod_mitigation(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.sod_engine.revoke_mitigating_control(id).await {
        Ok(control) => Ok(Json(serde_json::to_value(control).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Dashboard
// ============================================================================

/// Get SoD compliance dashboard
pub async fn get_sod_dashboard(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = Uuid::parse_str(&claims.org_id).unwrap_or(Uuid::nil());
    match state.sod_engine.get_dashboard(org_id).await {
        Ok(summary) => Ok(Json(serde_json::to_value(summary).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))),
    }
}
