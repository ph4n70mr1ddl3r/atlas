//! Expense Policy Compliance API Handlers
//!
//! Oracle Fusion Cloud ERP: Expenses > Policies > Expense Policy Compliance
//!
//! Endpoints for managing expense policy rules, compliance audits,
//! violation tracking, and compliance dashboard.

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
use crate::handlers::auth::{Claims, parse_uuid};

// ============================================================================
// Query Parameters
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct StatusQuery {
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RuleListQuery {
    pub status: Option<String>,
    pub rule_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AuditListQuery {
    pub status: Option<String>,
    pub risk_level: Option<String>,
}

// ============================================================================
// Policy Rule Endpoints
// ============================================================================

/// Create an expense policy rule
pub async fn create_rule(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    let rule_code = body["rule_code"].as_str().unwrap_or("").to_string();
    let name = body["name"].as_str().unwrap_or("").to_string();
    let description = body["description"].as_str();
    let rule_type = body["rule_type"].as_str().unwrap_or("amount_limit").to_string();
    let expense_category = body["expense_category"].as_str().unwrap_or("all").to_string();
    let severity = body["severity"].as_str().unwrap_or("warning").to_string();
    let evaluation_scope = body["evaluation_scope"].as_str().unwrap_or("per_line").to_string();
    let threshold_amount = body["threshold_amount"].as_str();
    let maximum_amount = body["maximum_amount"].as_str();
    let threshold_days = body["threshold_days"].as_i64().unwrap_or(0) as i32;
    let requires_receipt = body["requires_receipt"].as_bool().unwrap_or(false);
    let requires_justification = body["requires_justification"].as_bool().unwrap_or(false);
    let effective_from = body["effective_from"].as_str()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let effective_to = body["effective_to"].as_str()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let applies_to_department = body["applies_to_department"].as_str();
    let applies_to_cost_center = body["applies_to_cost_center"].as_str();

    match state.expense_policy_compliance_engine.create_rule(
        org_id, &rule_code, &name, description, &rule_type, &expense_category,
        &severity, &evaluation_scope, threshold_amount, maximum_amount,
        threshold_days, requires_receipt, requires_justification,
        effective_from, effective_to, applies_to_department,
        applies_to_cost_center, parse_uuid(&claims.sub).ok(),
    ).await {
        Ok(rule) => Ok((StatusCode::CREATED, Json(serde_json::to_value(rule).unwrap_or(Value::Null)))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Get a rule by code
pub async fn get_rule(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(rule_code): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    match state.expense_policy_compliance_engine.get_rule(org_id, &rule_code).await {
        Ok(Some(rule)) => Ok(Json(serde_json::to_value(rule).unwrap_or(Value::Null))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(json!({"error": "Rule not found"})))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// List rules
pub async fn list_rules(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<RuleListQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    match state.expense_policy_compliance_engine.list_rules(
        org_id, params.status.as_deref(), params.rule_type.as_deref()
    ).await {
        Ok(rules) => Ok(Json(json!({"data": rules}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Activate a rule
pub async fn activate_rule(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.expense_policy_compliance_engine.activate_rule(id).await {
        Ok(rule) => Ok(Json(serde_json::to_value(rule).unwrap_or(Value::Null))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Deactivate a rule
pub async fn deactivate_rule(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.expense_policy_compliance_engine.deactivate_rule(id).await {
        Ok(rule) => Ok(Json(serde_json::to_value(rule).unwrap_or(Value::Null))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Delete a rule
pub async fn delete_rule(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(rule_code): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    match state.expense_policy_compliance_engine.delete_rule(org_id, &rule_code).await {
        Ok(()) => Ok(Json(json!({"message": "Rule deleted"}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Compliance Audit Endpoints
// ============================================================================

/// Create a compliance audit
pub async fn create_audit(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    let report_id = body["report_id"].as_str()
        .and_then(|s| Uuid::parse_str(s).ok())
        .unwrap_or_else(Uuid::new_v4);
    let report_number = body["report_number"].as_str();
    let employee_id = body["employee_id"].as_str().and_then(|s| Uuid::parse_str(s).ok());
    let employee_name = body["employee_name"].as_str();
    let department_id = body["department_id"].as_str().and_then(|s| Uuid::parse_str(s).ok());
    let audit_trigger = body["audit_trigger"].as_str().unwrap_or("automatic").to_string();
    let audit_date = body["audit_date"].as_str()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .unwrap_or_else(|| chrono::Utc::now().date_naive());

    match state.expense_policy_compliance_engine.create_audit(
        org_id, report_id, report_number, employee_id, employee_name,
        department_id, &audit_trigger, audit_date,
    ).await {
        Ok(audit) => Ok((StatusCode::CREATED, Json(serde_json::to_value(audit).unwrap_or(Value::Null)))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Get an audit by ID
pub async fn get_audit(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.expense_policy_compliance_engine.get_audit(id).await {
        Ok(Some(audit)) => Ok(Json(serde_json::to_value(audit).unwrap_or(Value::Null))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(json!({"error": "Audit not found"})))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// List audits
pub async fn list_audits(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<AuditListQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    match state.expense_policy_compliance_engine.list_audits(
        org_id, params.status.as_deref(), params.risk_level.as_deref()
    ).await {
        Ok(audits) => Ok(Json(json!({"data": audits}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Evaluate compliance for an audit
pub async fn evaluate_compliance(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.expense_policy_compliance_engine.evaluate_compliance(id).await {
        Ok(audit) => Ok(Json(serde_json::to_value(audit).unwrap_or(Value::Null))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Complete audit review
pub async fn complete_audit_review(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let review_notes = body["review_notes"].as_str();

    match state.expense_policy_compliance_engine.complete_audit_review(
        id, parse_uuid(&claims.sub).ok(), review_notes,
    ).await {
        Ok(audit) => Ok(Json(serde_json::to_value(audit).unwrap_or(Value::Null))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Escalate audit
pub async fn escalate_audit(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let review_notes = body["review_notes"].as_str();

    match state.expense_policy_compliance_engine.escalate_audit(
        id, parse_uuid(&claims.sub).ok(), review_notes,
    ).await {
        Ok(audit) => Ok(Json(serde_json::to_value(audit).unwrap_or(Value::Null))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Violation Endpoints
// ============================================================================

/// List violations for an audit
pub async fn list_violations(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(audit_id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.expense_policy_compliance_engine.list_violations(audit_id).await {
        Ok(violations) => Ok(Json(json!({"data": violations}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Resolve a violation
pub async fn resolve_violation(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let resolution_status = body["resolution_status"].as_str().unwrap_or("justified").to_string();
    let justification = body["justification"].as_str();

    match state.expense_policy_compliance_engine.resolve_violation(
        id, &resolution_status, justification, parse_uuid(&claims.sub).ok(),
    ).await {
        Ok(violation) => Ok(Json(serde_json::to_value(violation).unwrap_or(Value::Null))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// List open violations
pub async fn list_open_violations(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    match state.expense_policy_compliance_engine.list_open_violations(org_id).await {
        Ok(violations) => Ok(Json(json!({"data": violations}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Dashboard
// ============================================================================

/// Get expense compliance dashboard
pub async fn get_dashboard(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    match state.expense_policy_compliance_engine.get_dashboard(org_id).await {
        Ok(dashboard) => Ok(Json(serde_json::to_value(dashboard).unwrap_or(Value::Null))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}
