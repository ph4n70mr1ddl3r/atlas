//! Interest Invoice Management API Handlers
//!
//! Oracle Fusion Cloud ERP: Receivables > Late Charges
//!
//! Endpoints for managing interest rate schedules, overdue invoices,
//! interest calculation, and interest invoice generation.

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

// ============================================================================
// Interest Rate Schedules
// ============================================================================

/// Create an interest rate schedule
pub async fn create_schedule(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    let schedule_code = body["schedule_code"].as_str().unwrap_or("").to_string();
    let name = body["name"].as_str().unwrap_or("").to_string();
    let description = body["description"].as_str();
    let annual_rate = body["annual_rate"].as_str().unwrap_or("0").to_string();
    let compounding_frequency = body["compounding_frequency"].as_str().unwrap_or("daily").to_string();
    let charge_type = body["charge_type"].as_str().unwrap_or("interest").to_string();
    let grace_period_days = body["grace_period_days"].as_i64().unwrap_or(0) as i32;
    let minimum_charge = body["minimum_charge"].as_str().unwrap_or("0").to_string();
    let maximum_charge = body["maximum_charge"].as_str();
    let currency_code = body["currency_code"].as_str().unwrap_or("USD").to_string();
    let effective_from = body["effective_from"].as_str()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .unwrap_or_else(|| chrono::Utc::now().date_naive());
    let effective_to = body["effective_to"].as_str()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());

    match state.interest_invoice_engine.create_schedule(
        org_id, &schedule_code, &name, description, &annual_rate,
        &compounding_frequency, &charge_type, grace_period_days,
        &minimum_charge, maximum_charge, &currency_code,
        effective_from, effective_to, parse_uuid(&claims.sub).ok(),
    ).await {
        Ok(schedule) => Ok((StatusCode::CREATED, Json(serde_json::to_value(&schedule).map_err(|e| {
            tracing::error!("Serialization error: {}", e);
            e
        }).unwrap_or(Value::Null)))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Get a schedule by code
pub async fn get_schedule(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(schedule_code): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    match state.interest_invoice_engine.get_schedule(org_id, &schedule_code).await {
        Ok(Some(schedule)) => Ok(Json(serde_json::to_value(schedule).unwrap_or(Value::Null))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(json!({"error": "Schedule not found"})))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// List schedules
pub async fn list_schedules(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<StatusQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    match state.interest_invoice_engine.list_schedules(org_id, params.status.as_deref()).await {
        Ok(schedules) => Ok(Json(json!({"data": schedules}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Activate a schedule
pub async fn activate_schedule(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.interest_invoice_engine.activate_schedule(id).await {
        Ok(schedule) => Ok(Json(serde_json::to_value(schedule).unwrap_or(Value::Null))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Deactivate a schedule
pub async fn deactivate_schedule(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.interest_invoice_engine.deactivate_schedule(id).await {
        Ok(schedule) => Ok(Json(serde_json::to_value(schedule).unwrap_or(Value::Null))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Delete a schedule
pub async fn delete_schedule(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(schedule_code): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    match state.interest_invoice_engine.delete_schedule(org_id, &schedule_code).await {
        Ok(()) => Ok(Json(json!({"message": "Schedule deleted"}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Overdue Invoices
// ============================================================================

/// Register an overdue invoice
pub async fn register_overdue_invoice(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    let invoice_number = body["invoice_number"].as_str().unwrap_or("").to_string();
    let customer_id = body["customer_id"].as_str()
        .and_then(|s| Uuid::parse_str(s).ok())
        .unwrap_or_else(Uuid::new_v4);
    let customer_name = body["customer_name"].as_str();
    let original_amount = body["original_amount"].as_str().unwrap_or("0").to_string();
    let outstanding_amount = body["outstanding_amount"].as_str().unwrap_or("0").to_string();
    let due_date = body["due_date"].as_str()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .unwrap_or_else(|| chrono::Utc::now().date_naive());
    let overdue_days = body["overdue_days"].as_i64().unwrap_or(0) as i32;
    let currency_code = body["currency_code"].as_str().unwrap_or("USD").to_string();

    match state.interest_invoice_engine.register_overdue_invoice(
        org_id, &invoice_number, customer_id, customer_name,
        &original_amount, &outstanding_amount, due_date, overdue_days, &currency_code,
    ).await {
        Ok(inv) => Ok((StatusCode::CREATED, Json(serde_json::to_value(inv).unwrap_or(Value::Null)))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// List overdue invoices
pub async fn list_overdue_invoices(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<StatusQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    match state.interest_invoice_engine.list_overdue_invoices(org_id, params.status.as_deref()).await {
        Ok(invoices) => Ok(Json(json!({"data": invoices}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Close an overdue invoice
pub async fn close_overdue_invoice(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.interest_invoice_engine.close_overdue_invoice(id).await {
        Ok(inv) => Ok(Json(serde_json::to_value(inv).unwrap_or(Value::Null))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Interest Calculation
// ============================================================================

/// Calculate interest on overdue invoices
pub async fn calculate_interest(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    let description = body["description"].as_str();
    let schedule_id = body["schedule_id"].as_str().and_then(|s| Uuid::parse_str(s).ok());
    let calculation_date = body["calculation_date"].as_str()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .unwrap_or_else(|| chrono::Utc::now().date_naive());

    match state.interest_invoice_engine.calculate_interest(
        org_id, description, schedule_id, calculation_date, parse_uuid(&claims.sub).ok(),
    ).await {
        Ok(run) => Ok((StatusCode::CREATED, Json(serde_json::to_value(run).unwrap_or(Value::Null)))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Get a calculation run
pub async fn get_calculation_run(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.interest_invoice_engine.get_calculation_run(id).await {
        Ok(Some(run)) => Ok(Json(serde_json::to_value(run).unwrap_or(Value::Null))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(json!({"error": "Calculation run not found"})))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// List calculation runs
pub async fn list_calculation_runs(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    match state.interest_invoice_engine.list_calculation_runs(org_id).await {
        Ok(runs) => Ok(Json(json!({"data": runs}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// List calculation lines for a run
pub async fn list_calculation_lines(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(run_id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.interest_invoice_engine.list_calculation_lines(run_id).await {
        Ok(lines) => Ok(Json(json!({"data": lines}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Cancel a calculation run
pub async fn cancel_calculation_run(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.interest_invoice_engine.cancel_calculation_run(id).await {
        Ok(run) => Ok(Json(serde_json::to_value(run).unwrap_or(Value::Null))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Interest Invoice Generation
// ============================================================================

/// Generate interest invoices from a calculation run
pub async fn generate_interest_invoices(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(run_id): Path<Uuid>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    let invoice_date = body["invoice_date"].as_str()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .unwrap_or_else(|| chrono::Utc::now().date_naive());
    let due_date = body["due_date"].as_str()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let gl_account_code = body["gl_account_code"].as_str();

    match state.interest_invoice_engine.generate_interest_invoices(
        org_id, run_id, invoice_date, due_date,
        gl_account_code, parse_uuid(&claims.sub).ok(),
    ).await {
        Ok(invoices) => Ok((StatusCode::CREATED, Json(json!({"data": invoices})))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Get an interest invoice by number
pub async fn get_interest_invoice(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(invoice_number): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    match state.interest_invoice_engine.get_interest_invoice(org_id, &invoice_number).await {
        Ok(Some(inv)) => Ok(Json(serde_json::to_value(inv).unwrap_or(Value::Null))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(json!({"error": "Interest invoice not found"})))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// List interest invoices
pub async fn list_interest_invoices(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<StatusQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    match state.interest_invoice_engine.list_interest_invoices(org_id, params.status.as_deref()).await {
        Ok(invoices) => Ok(Json(json!({"data": invoices}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Post an interest invoice
pub async fn post_interest_invoice(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.interest_invoice_engine.post_interest_invoice(id).await {
        Ok(inv) => Ok(Json(serde_json::to_value(inv).unwrap_or(Value::Null))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Reverse an interest invoice
pub async fn reverse_interest_invoice(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.interest_invoice_engine.reverse_interest_invoice(id).await {
        Ok(inv) => Ok(Json(serde_json::to_value(inv).unwrap_or(Value::Null))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Cancel an interest invoice
pub async fn cancel_interest_invoice(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.interest_invoice_engine.cancel_interest_invoice(id).await {
        Ok(inv) => Ok(Json(serde_json::to_value(inv).unwrap_or(Value::Null))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// List interest invoice lines
pub async fn list_interest_invoice_lines(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(invoice_id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.interest_invoice_engine.list_interest_invoice_lines(invoice_id).await {
        Ok(lines) => Ok(Json(json!({"data": lines}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Dashboard
// ============================================================================

/// Get interest invoice dashboard
pub async fn get_interest_invoice_dashboard(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    match state.interest_invoice_engine.get_dashboard_summary(org_id).await {
        Ok(summary) => Ok(Json(serde_json::to_value(summary).unwrap_or(Value::Null))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}
