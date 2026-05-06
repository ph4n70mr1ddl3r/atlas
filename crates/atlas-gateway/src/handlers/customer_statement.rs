//! Customer Statement / Balance Forward Billing API Handlers
//!
//! Oracle Fusion Cloud ERP: Receivables > Billing > Balance Forward Billing
//!
//! Endpoints for managing customer account statements, statement lines,
//! delivery tracking, and statement generation.

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
pub struct StatementListQuery {
    pub customer_id: Option<String>,
    pub status: Option<String>,
    pub billing_cycle: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct StatementLineListQuery {}

// ============================================================================
// Statements
// ============================================================================

/// Create a customer statement
pub async fn create_statement(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    let customer_id = body["customer_id"].as_str()
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| (StatusCode::BAD_REQUEST, Json(json!({"error": "customer_id is required"}))))?;
    let customer_number = body["customer_number"].as_str();
    let customer_name = body["customer_name"].as_str();
    let statement_date = body["statement_date"].as_str()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .ok_or_else(|| (StatusCode::BAD_REQUEST, Json(json!({"error": "statement_date is required (YYYY-MM-DD)"}))))?;
    let billing_period_from = body["billing_period_from"].as_str()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .ok_or_else(|| (StatusCode::BAD_REQUEST, Json(json!({"error": "billing_period_from is required (YYYY-MM-DD)"}))))?;
    let billing_period_to = body["billing_period_to"].as_str()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .ok_or_else(|| (StatusCode::BAD_REQUEST, Json(json!({"error": "billing_period_to is required (YYYY-MM-DD)"}))))?;
    let billing_cycle = body["billing_cycle"].as_str().unwrap_or("monthly");
    let opening_balance = body["opening_balance"].as_str().unwrap_or("0.00");
    let total_charges = body["total_charges"].as_str().unwrap_or("0.00");
    let total_payments = body["total_payments"].as_str().unwrap_or("0.00");
    let total_credits = body["total_credits"].as_str().unwrap_or("0.00");
    let total_adjustments = body["total_adjustments"].as_str().unwrap_or("0.00");
    let aging_current = body["aging_current"].as_str().unwrap_or("0.00");
    let aging_1_30 = body["aging_1_30"].as_str().unwrap_or("0.00");
    let aging_31_60 = body["aging_31_60"].as_str().unwrap_or("0.00");
    let aging_61_90 = body["aging_61_90"].as_str().unwrap_or("0.00");
    let aging_91_120 = body["aging_91_120"].as_str().unwrap_or("0.00");
    let aging_121_plus = body["aging_121_plus"].as_str().unwrap_or("0.00");
    let currency_code = body["currency_code"].as_str().unwrap_or("USD");
    let delivery_method = body["delivery_method"].as_str();
    let delivery_email = body["delivery_email"].as_str();
    let previous_statement_id = body["previous_statement_id"].as_str().and_then(|s| Uuid::parse_str(s).ok());
    let notes = body["notes"].as_str();

    match state.customer_statement_engine.create_statement(
        org_id, customer_id, customer_number, customer_name,
        statement_date, billing_period_from, billing_period_to, billing_cycle,
        opening_balance, total_charges, total_payments, total_credits, total_adjustments,
        aging_current, aging_1_30, aging_31_60, aging_61_90, aging_91_120, aging_121_plus,
        currency_code, delivery_method, delivery_email,
        previous_statement_id, notes, parse_uuid(&claims.sub).ok(),
    ).await {
        Ok(stmt) => Ok((StatusCode::CREATED, Json(serde_json::to_value(stmt).unwrap_or(Value::Null)))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Get a statement by ID
pub async fn get_statement(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.customer_statement_engine.get_statement(id).await {
        Ok(Some(stmt)) => Ok(Json(serde_json::to_value(stmt).unwrap_or(Value::Null))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(json!({"error": "Statement not found"})))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Get a statement by number
pub async fn get_statement_by_number(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(statement_number): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    match state.customer_statement_engine.get_statement_by_number(org_id, &statement_number).await {
        Ok(Some(stmt)) => Ok(Json(serde_json::to_value(stmt).unwrap_or(Value::Null))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(json!({"error": "Statement not found"})))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// List statements
pub async fn list_statements(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<StatementListQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    let customer_id = params.customer_id.as_deref().and_then(|s| Uuid::parse_str(s).ok());

    match state.customer_statement_engine.list_statements(
        org_id, customer_id, params.status.as_deref(), params.billing_cycle.as_deref(),
    ).await {
        Ok(stmts) => Ok(Json(json!({"data": stmts}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Generate (finalize) a statement
pub async fn generate_statement(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.customer_statement_engine.generate_statement(id).await {
        Ok(stmt) => Ok(Json(serde_json::to_value(stmt).unwrap_or(Value::Null))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Send a statement
pub async fn send_statement(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.customer_statement_engine.send_statement(id).await {
        Ok(stmt) => Ok(Json(serde_json::to_value(stmt).unwrap_or(Value::Null))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Mark statement as viewed
pub async fn mark_viewed(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.customer_statement_engine.mark_viewed(id).await {
        Ok(stmt) => Ok(Json(serde_json::to_value(stmt).unwrap_or(Value::Null))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Archive a statement
pub async fn archive_statement(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.customer_statement_engine.archive_statement(id).await {
        Ok(stmt) => Ok(Json(serde_json::to_value(stmt).unwrap_or(Value::Null))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Cancel a statement
pub async fn cancel_statement(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let reason = body["reason"].as_str();
    match state.customer_statement_engine.cancel_statement(id, reason).await {
        Ok(stmt) => Ok(Json(serde_json::to_value(stmt).unwrap_or(Value::Null))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Resend a statement
pub async fn resend_statement(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.customer_statement_engine.resend_statement(id).await {
        Ok(stmt) => Ok(Json(serde_json::to_value(stmt).unwrap_or(Value::Null))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Statement Lines
// ============================================================================

/// Add a line to a statement
pub async fn add_statement_line(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(statement_id): Path<Uuid>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    let line_type = body["line_type"].as_str()
        .ok_or_else(|| (StatusCode::BAD_REQUEST, Json(json!({"error": "line_type is required"}))))?;
    let transaction_id = body["transaction_id"].as_str().and_then(|s| Uuid::parse_str(s).ok());
    let transaction_number = body["transaction_number"].as_str();
    let transaction_date = body["transaction_date"].as_str().and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let due_date = body["due_date"].as_str().and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let original_amount = body["original_amount"].as_str();
    let amount = body["amount"].as_str().unwrap_or("0.00");
    let description = body["description"].as_str();
    let reference_type = body["reference_type"].as_str();
    let reference_id = body["reference_id"].as_str().and_then(|s| Uuid::parse_str(s).ok());
    let metadata = body.get("metadata").cloned().unwrap_or(serde_json::json!({}));

    match state.customer_statement_engine.add_statement_line(
        org_id, statement_id, line_type,
        transaction_id, transaction_number, transaction_date, due_date,
        original_amount, amount, description,
        reference_type, reference_id, metadata,
    ).await {
        Ok(line) => Ok((StatusCode::CREATED, Json(serde_json::to_value(line).unwrap_or(Value::Null)))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// List statement lines
pub async fn list_statement_lines(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(statement_id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.customer_statement_engine.list_statement_lines(statement_id).await {
        Ok(lines) => Ok(Json(json!({"data": lines}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Remove a statement line
pub async fn remove_statement_line(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path((statement_id, line_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.customer_statement_engine.remove_statement_line(statement_id, line_id).await {
        Ok(()) => Ok(Json(json!({"message": "Statement line removed"}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Dashboard
// ============================================================================

/// Get statement summary dashboard
pub async fn get_statement_summary(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    match state.customer_statement_engine.get_statement_summary(org_id).await {
        Ok(summary) => Ok(Json(serde_json::to_value(summary).unwrap_or(Value::Null))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}
