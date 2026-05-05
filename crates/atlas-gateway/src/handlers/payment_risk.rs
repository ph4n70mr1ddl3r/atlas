//! Payment Risk & Fraud Detection API Handlers
//!
//! Oracle Fusion Cloud ERP: Financials > Payables > Payment Risk Management
//!
//! Endpoints for managing risk profiles, fraud alerts, sanctions screening,
//! and supplier risk assessments.

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
pub struct RiskProfileListQuery {
    pub profile_type: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct FraudAlertListQuery {
    pub status: Option<String>,
    pub alert_type: Option<String>,
    pub severity: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ScreeningListQuery {
    pub supplier_id: Option<String>,
    pub match_status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AssessmentListQuery {
    pub supplier_id: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct StatusTransitionBody {
    pub status: String,
    pub resolution_notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AssignBody {
    pub assigned_to: Option<String>,
    pub assigned_team: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ReviewBody {
    pub reviewed_by: String,
    pub review_notes: Option<String>,
    pub action_taken: String,
}

// ============================================================================
// Risk Profiles
// ============================================================================

/// Create a risk profile
pub async fn create_risk_profile(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = claims.org_uuid_json()?;

    let code = body["code"].as_str().unwrap_or("").to_string();
    let name = body["name"].as_str().unwrap_or("").to_string();
    if code.is_empty() || name.is_empty() {
        return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "code and name are required"}))));
    }

    match state.payment_risk_engine.create_risk_profile(
        org_id, &code, &name,
        body["description"].as_str(),
        body["profile_type"].as_str().unwrap_or("global"),
        body["default_risk_level"].as_str().unwrap_or("medium"),
        body["duplicate_amount_tolerance_pct"].as_str(),
        body["duplicate_date_tolerance_days"].as_str(),
        body["velocity_daily_limit"].as_str(),
        body["velocity_weekly_limit"].as_str(),
        body["amount_anomaly_std_dev"].as_str(),
        body["enable_sanctions_screening"].as_bool().unwrap_or(true),
        body["enable_duplicate_detection"].as_bool().unwrap_or(true),
        body["enable_velocity_checks"].as_bool().unwrap_or(true),
        body["enable_amount_anomaly"].as_bool().unwrap_or(true),
        body["enable_behavioral_analysis"].as_bool().unwrap_or(false),
        body["auto_block_critical"].as_bool().unwrap_or(true),
        body["auto_block_high"].as_bool().unwrap_or(false),
        body["effective_from"].as_str().and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok()),
        body["effective_to"].as_str().and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok()),
        claims.user_uuid_json().ok(),
    ).await {
        Ok(profile) => Ok((StatusCode::CREATED, Json(serde_json::to_value(profile).unwrap_or(Value::Null)))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Get a risk profile by code
pub async fn get_risk_profile(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(code): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = claims.org_uuid_json()?;

    match state.payment_risk_engine.get_risk_profile(org_id, &code).await {
        Ok(Some(profile)) => Ok(Json(serde_json::to_value(profile).unwrap_or(Value::Null))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(json!({"error": "Risk profile not found"})))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// List risk profiles
pub async fn list_risk_profiles(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<RiskProfileListQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = claims.org_uuid_json()?;

    match state.payment_risk_engine.list_risk_profiles(
        org_id, params.profile_type.as_deref(), params.is_active,
    ).await {
        Ok(profiles) => Ok(Json(json!({"data": profiles}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Activate or deactivate a risk profile
pub async fn set_risk_profile_active(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let is_active = body["is_active"].as_bool().unwrap_or(true);
    match state.payment_risk_engine.set_risk_profile_active(id, is_active).await {
        Ok(profile) => Ok(Json(serde_json::to_value(profile).unwrap_or(Value::Null))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Delete a risk profile
pub async fn delete_risk_profile(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(code): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<Value>)> {
    let org_id = claims.org_uuid_json()?;

    match state.payment_risk_engine.delete_risk_profile(org_id, &code).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Fraud Alerts
// ============================================================================

/// Create a fraud alert
pub async fn create_fraud_alert(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = claims.org_uuid_json()?;

    let alert_type = body["alert_type"].as_str().unwrap_or("").to_string();
    let severity = body["severity"].as_str().unwrap_or("").to_string();
    if alert_type.is_empty() || severity.is_empty() {
        return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "alert_type and severity are required"}))));
    }

    let payment_id = body["payment_id"].as_str().and_then(|s| Uuid::parse_str(s).ok());
    let invoice_id = body["invoice_id"].as_str().and_then(|s| Uuid::parse_str(s).ok());
    let supplier_id = body["supplier_id"].as_str().and_then(|s| Uuid::parse_str(s).ok());

    match state.payment_risk_engine.create_fraud_alert(
        org_id, &alert_type, &severity,
        payment_id, invoice_id, supplier_id,
        body["supplier_number"].as_str(),
        body["supplier_name"].as_str(),
        body["amount"].as_str(),
        body["currency_code"].as_str(),
        body["risk_score"].as_str(),
        body["detection_rule"].as_str(),
        body["description"].as_str(),
        body["evidence"].as_str(),
        body["assigned_to"].as_str(),
        body["assigned_team"].as_str(),
        body["related_alert_ids"].as_str(),
        claims.user_uuid_json().ok(),
    ).await {
        Ok(alert) => Ok((StatusCode::CREATED, Json(serde_json::to_value(alert).unwrap_or(Value::Null)))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Get a fraud alert by number
pub async fn get_fraud_alert(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(alert_number): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = claims.org_uuid_json()?;

    match state.payment_risk_engine.get_fraud_alert(org_id, &alert_number).await {
        Ok(Some(alert)) => Ok(Json(serde_json::to_value(alert).unwrap_or(Value::Null))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(json!({"error": "Fraud alert not found"})))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// List fraud alerts
pub async fn list_fraud_alerts(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<FraudAlertListQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = claims.org_uuid_json()?;

    match state.payment_risk_engine.list_fraud_alerts(
        org_id, params.status.as_deref(), params.alert_type.as_deref(), params.severity.as_deref(),
    ).await {
        Ok(alerts) => Ok(Json(json!({"data": alerts}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Transition a fraud alert (workflow action)
pub async fn transition_fraud_alert(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(body): Json<StatusTransitionBody>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let resolved_by = claims.user_uuid_json().ok();
    match state.payment_risk_engine.transition_fraud_alert(
        id, &body.status, body.resolution_notes.as_deref(), resolved_by,
    ).await {
        Ok(alert) => Ok(Json(serde_json::to_value(alert).unwrap_or(Value::Null))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Assign a fraud alert
pub async fn assign_fraud_alert(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(body): Json<AssignBody>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.payment_risk_engine.assign_fraud_alert(
        id, body.assigned_to.as_deref(), body.assigned_team.as_deref(),
    ).await {
        Ok(alert) => Ok(Json(serde_json::to_value(alert).unwrap_or(Value::Null))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Sanctions Screening
// ============================================================================

/// Create a sanctions screening result
pub async fn create_screening_result(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = claims.org_uuid_json()?;

    let screening_type = body["screening_type"].as_str().unwrap_or("").to_string();
    let screened_list = body["screened_list"].as_str().unwrap_or("").to_string();
    if screening_type.is_empty() || screened_list.is_empty() {
        return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "screening_type and screened_list are required"}))));
    }

    let supplier_id = body["supplier_id"].as_str().and_then(|s| Uuid::parse_str(s).ok());
    let payment_id = body["payment_id"].as_str().and_then(|s| Uuid::parse_str(s).ok());

    match state.payment_risk_engine.create_screening_result(
        org_id, &screening_type, supplier_id,
        body["supplier_name"].as_str(), payment_id,
        &screened_list,
        body["match_name"].as_str(),
        body["match_type"].as_str().unwrap_or("none"),
        body["match_score"].as_str(),
        body["match_status"].as_str().unwrap_or("no_match"),
        body["sanctions_list_entry"].as_str(),
        body["sanctions_list_program"].as_str(),
        body["match_details"].as_str(),
        body["action_taken"].as_str(),
        claims.user_uuid_json().ok(),
    ).await {
        Ok(result) => Ok((StatusCode::CREATED, Json(serde_json::to_value(result).unwrap_or(Value::Null)))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Get a screening result by ID
pub async fn get_screening_result(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(screening_id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = claims.org_uuid_json()?;

    match state.payment_risk_engine.get_screening_result(org_id, &screening_id).await {
        Ok(Some(result)) => Ok(Json(serde_json::to_value(result).unwrap_or(Value::Null))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(json!({"error": "Screening result not found"})))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// List screening results
pub async fn list_screening_results(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<ScreeningListQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = claims.org_uuid_json()?;

    let supplier_id = params.supplier_id.as_deref()
        .and_then(|s| Uuid::parse_str(s).ok());

    match state.payment_risk_engine.list_screening_results(
        org_id, supplier_id, params.match_status.as_deref(),
    ).await {
        Ok(results) => Ok(Json(json!({"data": results}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Review a screening result
pub async fn review_screening_result(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(body): Json<ReviewBody>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.payment_risk_engine.review_screening_result(
        id, &body.reviewed_by, body.review_notes.as_deref(), &body.action_taken,
    ).await {
        Ok(result) => Ok(Json(serde_json::to_value(result).unwrap_or(Value::Null))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Supplier Risk Assessments
// ============================================================================

/// Create a supplier risk assessment
pub async fn create_assessment(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = claims.org_uuid_json()?;

    let supplier_id = match body["supplier_id"].as_str().and_then(|s| Uuid::parse_str(s).ok()) {
        Some(id) => id,
        None => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "supplier_id is required"})))),
    };
    let supplier_name = body["supplier_name"].as_str().unwrap_or("").to_string();
    if supplier_name.is_empty() {
        return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "supplier_name is required"}))));
    }

    match state.payment_risk_engine.create_assessment(
        org_id, supplier_id, &supplier_name,
        body["assessment_type"].as_str().unwrap_or("periodic"),
        body["financial_risk_score"].as_str(),
        body["operational_risk_score"].as_str(),
        body["compliance_risk_score"].as_str(),
        body["payment_history_score"].as_str(),
        body["years_in_business"].as_i64().map(|v| v as i32),
        body["has_financial_statements"].as_bool().unwrap_or(false),
        body["has_audit_reports"].as_bool().unwrap_or(false),
        body["has_insurance"].as_bool().unwrap_or(false),
        body["is_sanctions_clear"].as_bool().unwrap_or(false),
        body["is_aml_clear"].as_bool().unwrap_or(false),
        body["is_pep_clear"].as_bool().unwrap_or(false),
        body["total_historical_payments"].as_i64().map(|v| v as i32),
        body["total_historical_amount"].as_str(),
        body["fraud_alerts_count"].as_i64().map(|v| v as i32),
        body["duplicate_payments_count"].as_i64().map(|v| v as i32),
        body["assessed_by"].as_str(),
        body["findings"].as_str(),
        body["recommendations"].as_str(),
        claims.user_uuid_json().ok(),
    ).await {
        Ok(assessment) => Ok((StatusCode::CREATED, Json(serde_json::to_value(assessment).unwrap_or(Value::Null)))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Get an assessment by number
pub async fn get_assessment(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(assessment_number): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = claims.org_uuid_json()?;

    match state.payment_risk_engine.get_assessment(org_id, &assessment_number).await {
        Ok(Some(assessment)) => Ok(Json(serde_json::to_value(assessment).unwrap_or(Value::Null))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(json!({"error": "Assessment not found"})))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// List assessments
pub async fn list_assessments(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<AssessmentListQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = claims.org_uuid_json()?;

    let supplier_id = params.supplier_id.as_deref()
        .and_then(|s| Uuid::parse_str(s).ok());

    match state.payment_risk_engine.list_assessments(
        org_id, supplier_id, params.status.as_deref(),
    ).await {
        Ok(assessments) => Ok(Json(json!({"data": assessments}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Transition an assessment (workflow action)
pub async fn transition_assessment(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(body): Json<StatusTransitionBody>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.payment_risk_engine.transition_assessment(id, &body.status).await {
        Ok(assessment) => Ok(Json(serde_json::to_value(assessment).unwrap_or(Value::Null))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Delete an assessment
pub async fn delete_assessment(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(assessment_number): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<Value>)> {
    let org_id = claims.org_uuid_json()?;

    match state.payment_risk_engine.delete_assessment(org_id, &assessment_number).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}
