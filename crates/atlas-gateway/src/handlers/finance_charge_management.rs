//! Finance Charge Management API Handlers
//!
//! REST endpoints for Oracle Fusion-inspired Finance Charge Management.
//! Manages late payment charge assessment on overdue customer invoices.
//!
//! Oracle Fusion equivalent: Financials > Receivables > Finance Charges

use axum::{
    extract::{Path, Query, State, Extension},
    Json,
    http::StatusCode,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;
use tracing::error;

use crate::AppState;
use crate::handlers::auth::{Claims, parse_uuid};

// ============================================================================
// Term CRUD Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTermRequest {
    pub term_code: String,
    pub term_name: String,
    pub description: Option<String>,
    pub charge_type: Option<String>,
    pub charge_rate: Option<f64>,
    pub minimum_charge: Option<f64>,
    pub maximum_charge: Option<f64>,
    pub grace_period_days: Option<i32>,
    pub currency_code: Option<String>,
    pub calculation_basis: Option<String>,
    pub include_tax: Option<bool>,
    pub compound_charges: Option<bool>,
    pub effective_from: Option<String>,
    pub effective_to: Option<String>,
    pub auto_assess: Option<bool>,
    pub revenue_account_code: Option<String>,
    pub receivable_account_code: Option<String>,
}

pub async fn create_term(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreateTermRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    let charge_type = req.charge_type.as_deref().unwrap_or("percentage");
    let currency = req.currency_code.as_deref().unwrap_or("USD");
    let calc_basis = req.calculation_basis.as_deref().unwrap_or("monthly");
    let effective_from = req.effective_from.as_deref()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let effective_to = req.effective_to.as_deref()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());

    match state.finance_charge_engine.create_term(
        org_id,
        &req.term_code,
        &req.term_name,
        req.description.as_deref(),
        charge_type,
        req.charge_rate,
        req.minimum_charge,
        req.maximum_charge,
        req.grace_period_days.unwrap_or(0),
        currency,
        calc_basis,
        req.include_tax.unwrap_or(false),
        req.compound_charges.unwrap_or(false),
        effective_from,
        effective_to,
        req.auto_assess.unwrap_or(false),
        req.revenue_account_code.as_deref(),
        req.receivable_account_code.as_deref(),
        None,
    ).await {
        Ok(term) => Ok((StatusCode::CREATED, Json(serde_json::to_value(term).unwrap_or(serde_json::Value::Null)))),
        Err(e) => {
            error!("Failed to create finance charge term: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

pub async fn get_term(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.finance_charge_engine.get_term(id).await {
        Ok(Some(term)) => Ok(Json(serde_json::to_value(term).unwrap_or(serde_json::Value::Null))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Finance charge term not found"})))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                        Json(serde_json::json!({"error": e.to_string()})))),
    }
}

#[derive(Debug, Deserialize)]
pub struct ListTermsQuery {
    pub is_active: Option<bool>,
}

pub async fn list_terms(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<ListTermsQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    match state.finance_charge_engine.list_terms(org_id, query.is_active).await {
        Ok(terms) => Ok(Json(serde_json::json!({"data": terms}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                        Json(serde_json::json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Assessment Run Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRunRequest {
    pub run_date: String,
    pub gl_date: Option<String>,
    pub term_id: Option<String>,
    pub term_code: Option<String>,
    pub currency_code: Option<String>,
    pub notes: Option<String>,
}

pub async fn create_run(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreateRunRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    let run_date = chrono::NaiveDate::parse_from_str(&req.run_date, "%Y-%m-%d")
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": format!("Invalid run_date: {}", e)}))))?;
    let gl_date = req.gl_date.as_deref()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .unwrap_or(run_date);
    let term_id = req.term_id.as_deref().and_then(|s| s.parse::<Uuid>().ok());
    let currency = req.currency_code.as_deref().unwrap_or("USD");

    match state.finance_charge_engine.create_run(
        org_id, run_date, gl_date, term_id,
        req.term_code.as_deref(), currency,
        req.notes.as_deref(), None,
    ).await {
        Ok(run) => Ok((StatusCode::CREATED, Json(serde_json::to_value(run).unwrap_or(serde_json::Value::Null)))),
        Err(e) => {
            error!("Failed to create finance charge run: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

pub async fn get_run(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.finance_charge_engine.get_run(id).await {
        Ok(Some(run)) => Ok(Json(serde_json::to_value(run).unwrap_or(serde_json::Value::Null))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Finance charge run not found"})))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                        Json(serde_json::json!({"error": e.to_string()})))),
    }
}

pub async fn get_run_by_number(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(number): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    match state.finance_charge_engine.get_run_by_number(org_id, &number).await {
        Ok(Some(run)) => Ok(Json(serde_json::to_value(run).unwrap_or(serde_json::Value::Null))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Finance charge run not found"})))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                        Json(serde_json::json!({"error": e.to_string()})))),
    }
}

#[derive(Debug, Deserialize)]
pub struct ListRunsQuery {
    pub status: Option<String>,
}

pub async fn list_runs(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<ListRunsQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    match state.finance_charge_engine.list_runs(org_id, query.status.as_deref()).await {
        Ok(runs) => Ok(Json(serde_json::json!({"data": runs}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                        Json(serde_json::json!({"error": e.to_string()})))),
    }
}

pub async fn delete_run(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(number): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    match state.finance_charge_engine.delete_run(org_id, &number).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                        Json(serde_json::json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Run Transition Handler
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransitionRunRequest {
    pub status: String,
}

pub async fn transition_run(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(req): Json<TransitionRunRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.finance_charge_engine.transition_run(id, &req.status, None).await {
        Ok(run) => Ok(Json(serde_json::to_value(run).unwrap_or(serde_json::Value::Null))),
        Err(e) => {
            error!("Failed to transition finance charge run: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

// ============================================================================
// Charge Line Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddChargeLineRequest {
    pub customer_id: Option<String>,
    pub customer_number: Option<String>,
    pub customer_name: Option<String>,
    pub invoice_id: Option<String>,
    pub invoice_number: Option<String>,
    pub invoice_date: Option<String>,
    pub invoice_due_date: Option<String>,
    pub days_overdue: i32,
    pub invoice_amount: f64,
    pub outstanding_amount: f64,
    pub charge_type: Option<String>,
    pub charge_rate: Option<f64>,
    pub charge_amount: f64,
    pub currency_code: Option<String>,
    pub term_id: Option<String>,
    pub term_code: Option<String>,
}

pub async fn add_charge_line(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(run_id): Path<Uuid>,
    Json(req): Json<AddChargeLineRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    let customer_id = req.customer_id.as_deref().and_then(|s| s.parse::<Uuid>().ok());
    let invoice_id = req.invoice_id.as_deref().and_then(|s| s.parse::<Uuid>().ok());
    let term_id = req.term_id.as_deref().and_then(|s| s.parse::<Uuid>().ok());
    let invoice_date = req.invoice_date.as_deref()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let invoice_due_date = req.invoice_due_date.as_deref()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());

    match state.finance_charge_engine.add_charge_line(
        org_id, run_id,
        customer_id, req.customer_number.as_deref(), req.customer_name.as_deref(),
        invoice_id, req.invoice_number.as_deref(),
        invoice_date, invoice_due_date,
        req.days_overdue, req.invoice_amount, req.outstanding_amount,
        req.charge_type.as_deref().unwrap_or("percentage"),
        req.charge_rate.unwrap_or(0.0),
        req.charge_amount,
        req.currency_code.as_deref().unwrap_or("USD"),
        term_id, req.term_code.as_deref(),
    ).await {
        Ok(line) => Ok((StatusCode::CREATED, Json(serde_json::to_value(line).unwrap_or(serde_json::Value::Null)))),
        Err(e) => {
            error!("Failed to add charge line: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

pub async fn list_lines(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(run_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.finance_charge_engine.list_lines(run_id).await {
        Ok(lines) => Ok(Json(serde_json::json!({"data": lines}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                        Json(serde_json::json!({"error": e.to_string()})))),
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WaiveLineRequest {
    pub reason: String,
}

pub async fn waive_line(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(line_id): Path<Uuid>,
    Json(req): Json<WaiveLineRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.finance_charge_engine.waive_line(line_id, &req.reason).await {
        Ok(line) => Ok(Json(serde_json::to_value(line).unwrap_or(serde_json::Value::Null))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                        Json(serde_json::json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Invoice Generation Handler
// ============================================================================

pub async fn generate_invoices(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(run_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.finance_charge_engine.generate_invoices(run_id, None).await {
        Ok(invoices) => Ok(Json(serde_json::json!({"data": invoices}))),
        Err(e) => {
            error!("Failed to generate charge invoices: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

pub async fn list_invoices(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<ListInvoicesQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    let customer_id = params.customer_id.as_deref().and_then(|s| s.parse::<Uuid>().ok());
    match state.finance_charge_engine.list_invoices(org_id, params.status.as_deref(), customer_id).await {
        Ok(invoices) => Ok(Json(serde_json::json!({"data": invoices}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                        Json(serde_json::json!({"error": e.to_string()})))),
    }
}

#[derive(Debug, Deserialize)]
pub struct ListInvoicesQuery {
    pub status: Option<String>,
    pub customer_id: Option<String>,
}

pub async fn get_invoice(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.finance_charge_engine.get_invoice(id).await {
        Ok(Some(inv)) => Ok(Json(serde_json::to_value(inv).unwrap_or(serde_json::Value::Null))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Charge invoice not found"})))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                        Json(serde_json::json!({"error": e.to_string()})))),
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransitionInvoiceRequest {
    pub status: String,
}

pub async fn transition_invoice(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(req): Json<TransitionInvoiceRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.finance_charge_engine.transition_invoice(id, &req.status).await {
        Ok(inv) => Ok(Json(serde_json::to_value(inv).unwrap_or(serde_json::Value::Null))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                        Json(serde_json::json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Dashboard Handler
// ============================================================================

pub async fn get_dashboard(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    match state.finance_charge_engine.get_dashboard(org_id).await {
        Ok(summary) => Ok(Json(serde_json::to_value(summary).unwrap_or(serde_json::Value::Null))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                        Json(serde_json::json!({"error": e.to_string()})))),
    }
}
