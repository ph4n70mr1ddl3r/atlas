//! Project Billing API Handlers
//!
//! REST endpoints for Oracle Fusion-inspired Project Billing.
//!
//! Oracle Fusion Cloud equivalent: Project Management > Project Billing
//!
//! Provides:
//! - Bill Rate Schedule CRUD (draft → active / inactive)
//! - Bill Rate Lines per schedule, with date-effective lookups
//! - Project Billing Configurations per project
//! - Billing Events (milestone, progress, completion, retention_release)
//! - Project Invoices with full lifecycle (draft → submitted → approved → posted)
//! - Dashboard summary

use axum::{
    extract::{Path, Query, State, Extension},
    Json,
};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;
use tracing::error;

use crate::AppState;
use crate::handlers::auth::{Claims, parse_uuid};

// ============================================================================
// Helpers
// ============================================================================

fn error_response(e: atlas_shared::AtlasError) -> (axum::http::StatusCode, Json<serde_json::Value>) {
    let status = axum::http::StatusCode::from_u16(e.status_code())
        .unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
    (status, Json(json!({"error": e.to_string()})))
}

// ============================================================================
// Request Types
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateScheduleRequest {
    pub schedule_number: String,
    pub name: String,
    pub description: Option<String>,
    pub schedule_type: String,
    pub currency_code: String,
    pub effective_start: chrono::NaiveDate,
    pub effective_end: Option<chrono::NaiveDate>,
    pub default_markup_pct: Option<f64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddRateLineRequest {
    pub role_name: String,
    pub project_id: Option<Uuid>,
    pub bill_rate: f64,
    pub unit_of_measure: String,
    pub effective_start: chrono::NaiveDate,
    pub effective_end: Option<chrono::NaiveDate>,
    pub markup_pct: Option<f64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateBillingConfigRequest {
    pub project_id: Uuid,
    pub billing_method: String,
    pub bill_rate_schedule_id: Option<Uuid>,
    pub contract_amount: f64,
    pub currency_code: String,
    pub invoice_format: String,
    pub billing_cycle: String,
    pub payment_terms_days: i32,
    pub retention_pct: f64,
    pub retention_amount_cap: f64,
    pub customer_id: Option<Uuid>,
    pub customer_name: Option<String>,
    pub customer_po_number: Option<String>,
    pub contract_number: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateBillingEventRequest {
    pub project_id: Uuid,
    pub event_number: String,
    pub event_name: String,
    pub description: Option<String>,
    pub event_type: String,
    pub billing_amount: f64,
    pub currency_code: String,
    pub completion_pct: f64,
    pub planned_date: Option<chrono::NaiveDate>,
    pub task_id: Option<Uuid>,
    pub task_name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompleteBillingEventRequest {
    pub actual_date: chrono::NaiveDate,
    pub completion_pct: f64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InvoiceLineRequestDto {
    pub line_source: String,
    pub expenditure_item_id: Option<Uuid>,
    pub billing_event_id: Option<Uuid>,
    pub task_id: Option<Uuid>,
    pub task_number: Option<String>,
    pub task_name: Option<String>,
    pub description: Option<String>,
    pub employee_id: Option<Uuid>,
    pub employee_name: Option<String>,
    pub role_name: Option<String>,
    pub expenditure_type: Option<String>,
    pub quantity: f64,
    pub unit_of_measure: String,
    pub bill_rate: f64,
    pub raw_cost_amount: f64,
    pub bill_amount: f64,
    pub markup_amount: f64,
    pub tax_amount: f64,
    pub transaction_date: Option<chrono::NaiveDate>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateInvoiceRequest {
    pub invoice_number: String,
    pub project_id: Uuid,
    pub project_number: Option<String>,
    pub project_name: Option<String>,
    pub invoice_type: String,
    pub customer_id: Option<Uuid>,
    pub customer_name: Option<String>,
    pub billing_event_id: Option<Uuid>,
    pub lines: Vec<InvoiceLineRequestDto>,
    pub customer_po_number: Option<String>,
    pub contract_number: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RejectInvoiceRequest {
    pub reason: String,
}

#[derive(Debug, Deserialize)]
pub struct BillingFilters {
    pub project_id: Option<Uuid>,
    pub status: Option<String>,
}

// ============================================================================
// Bill Rate Schedule Handlers
// ============================================================================

pub async fn create_schedule(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreateScheduleRequest>,
) -> Result<(axum::http::StatusCode, Json<serde_json::Value>), (axum::http::StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    let created_by = parse_uuid(&claims.sub).ok();

    match state.project_billing_engine.create_schedule(
        org_id, &req.schedule_number, &req.name,
        req.description.as_deref(), &req.schedule_type, &req.currency_code,
        req.effective_start, req.effective_end,
        req.default_markup_pct.unwrap_or(0.0), created_by,
    ).await {
        Ok(schedule) => Ok((axum::http::StatusCode::CREATED, Json(serde_json::to_value(schedule).unwrap_or_default()))),
        Err(e) => {
            error!("Failed to create bill rate schedule: {}", e);
            Err(error_response(e))
        }
    }
}

pub async fn get_schedule(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    match state.project_billing_engine.get_schedule(id).await {
        Ok(Some(s)) => Ok(Json(serde_json::to_value(s).unwrap_or_default())),
        Ok(None) => Err((axum::http::StatusCode::NOT_FOUND, Json(json!({"error": "Schedule not found"})))),
        Err(e) => Err(error_response(e)),
    }
}

pub async fn list_schedules(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    let status = params.get("status").map(|s| s.as_str());

    match state.project_billing_engine.list_schedules(org_id, status).await {
        Ok(schedules) => Ok(Json(json!({"data": schedules}))),
        Err(e) => Err(error_response(e)),
    }
}

pub async fn activate_schedule(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    match state.project_billing_engine.activate_schedule(id).await {
        Ok(s) => Ok(Json(serde_json::to_value(s).unwrap_or_default())),
        Err(e) => Err(error_response(e)),
    }
}

pub async fn deactivate_schedule(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    match state.project_billing_engine.deactivate_schedule(id).await {
        Ok(s) => Ok(Json(serde_json::to_value(s).unwrap_or_default())),
        Err(e) => Err(error_response(e)),
    }
}

pub async fn delete_schedule(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(schedule_number): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;

    match state.project_billing_engine.delete_schedule(org_id, &schedule_number).await {
        Ok(()) => Ok(Json(json!({"message": "Schedule deleted"}))),
        Err(e) => Err(error_response(e)),
    }
}

// ============================================================================
// Bill Rate Line Handlers
// ============================================================================

pub async fn add_rate_line(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(schedule_id): Path<Uuid>,
    Json(req): Json<AddRateLineRequest>,
) -> Result<(axum::http::StatusCode, Json<serde_json::Value>), (axum::http::StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;

    match state.project_billing_engine.add_rate_line(
        org_id, schedule_id, &req.role_name, req.project_id,
        req.bill_rate, &req.unit_of_measure,
        req.effective_start, req.effective_end, req.markup_pct,
    ).await {
        Ok(line) => Ok((axum::http::StatusCode::CREATED, Json(serde_json::to_value(line).unwrap_or_default()))),
        Err(e) => {
            error!("Failed to add rate line: {}", e);
            Err(error_response(e))
        }
    }
}

pub async fn list_rate_lines(
    State(state): State<Arc<AppState>>,
    Path(schedule_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    match state.project_billing_engine.list_rate_lines(schedule_id).await {
        Ok(lines) => Ok(Json(json!({"data": lines}))),
        Err(e) => Err(error_response(e)),
    }
}

pub async fn find_rate_for_role(
    State(state): State<Arc<AppState>>,
    Path((schedule_id, role_name)): Path<(Uuid, String)>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let date_str = params.get("date").ok_or_else(|| {
        (axum::http::StatusCode::BAD_REQUEST, Json(json!({"error": "date query parameter is required"})))
    })?;
    let date = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d").map_err(|_| {
        (axum::http::StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid date format, use YYYY-MM-DD"})))
    })?;

    match state.project_billing_engine.find_rate_for_role(schedule_id, &role_name, date).await {
        Ok(Some(line)) => Ok(Json(serde_json::to_value(line).unwrap_or_default())),
        Ok(None) => Err((axum::http::StatusCode::NOT_FOUND, Json(json!({"error": "No rate found for role on given date"})))),
        Err(e) => Err(error_response(e)),
    }
}

pub async fn delete_rate_line(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    match state.project_billing_engine.delete_rate_line(id).await {
        Ok(()) => Ok(Json(json!({"message": "Rate line deleted"}))),
        Err(e) => Err(error_response(e)),
    }
}

// ============================================================================
// Project Billing Config Handlers
// ============================================================================

pub async fn create_billing_config(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreateBillingConfigRequest>,
) -> Result<(axum::http::StatusCode, Json<serde_json::Value>), (axum::http::StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    let created_by = parse_uuid(&claims.sub).ok();

    match state.project_billing_engine.create_billing_config(
        org_id, req.project_id, &req.billing_method,
        req.bill_rate_schedule_id, req.contract_amount, &req.currency_code,
        &req.invoice_format, &req.billing_cycle, req.payment_terms_days,
        req.retention_pct, req.retention_amount_cap,
        req.customer_id, req.customer_name.as_deref(),
        req.customer_po_number.as_deref(), req.contract_number.as_deref(),
        created_by,
    ).await {
        Ok(config) => Ok((axum::http::StatusCode::CREATED, Json(serde_json::to_value(config).unwrap_or_default()))),
        Err(e) => {
            error!("Failed to create billing config: {}", e);
            Err(error_response(e))
        }
    }
}

pub async fn get_billing_config(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    match state.project_billing_engine.get_billing_config(id).await {
        Ok(Some(c)) => Ok(Json(serde_json::to_value(c).unwrap_or_default())),
        Ok(None) => Err((axum::http::StatusCode::NOT_FOUND, Json(json!({"error": "Billing config not found"})))),
        Err(e) => Err(error_response(e)),
    }
}

pub async fn get_billing_config_by_project(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;

    match state.project_billing_engine.get_billing_config_by_project(org_id, project_id).await {
        Ok(Some(c)) => Ok(Json(serde_json::to_value(c).unwrap_or_default())),
        Ok(None) => Err((axum::http::StatusCode::NOT_FOUND, Json(json!({"error": "No billing config for this project"})))),
        Err(e) => Err(error_response(e)),
    }
}

pub async fn list_billing_configs(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    let status = params.get("status").map(|s| s.as_str());

    match state.project_billing_engine.list_billing_configs(org_id, status).await {
        Ok(configs) => Ok(Json(json!({"data": configs}))),
        Err(e) => Err(error_response(e)),
    }
}

pub async fn activate_billing_config(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    match state.project_billing_engine.activate_billing_config(id).await {
        Ok(c) => Ok(Json(serde_json::to_value(c).unwrap_or_default())),
        Err(e) => Err(error_response(e)),
    }
}

pub async fn cancel_billing_config(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    match state.project_billing_engine.cancel_billing_config(id).await {
        Ok(c) => Ok(Json(serde_json::to_value(c).unwrap_or_default())),
        Err(e) => Err(error_response(e)),
    }
}

// ============================================================================
// Billing Event Handlers
// ============================================================================

pub async fn create_billing_event(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreateBillingEventRequest>,
) -> Result<(axum::http::StatusCode, Json<serde_json::Value>), (axum::http::StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    let created_by = parse_uuid(&claims.sub).ok();

    match state.project_billing_engine.create_billing_event(
        org_id, req.project_id, &req.event_number, &req.event_name,
        req.description.as_deref(), &req.event_type, req.billing_amount,
        &req.currency_code, req.completion_pct, req.planned_date,
        req.task_id, req.task_name.as_deref(), created_by,
    ).await {
        Ok(event) => Ok((axum::http::StatusCode::CREATED, Json(serde_json::to_value(event).unwrap_or_default()))),
        Err(e) => {
            error!("Failed to create billing event: {}", e);
            Err(error_response(e))
        }
    }
}

pub async fn get_billing_event(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    match state.project_billing_engine.get_billing_event(id).await {
        Ok(Some(e)) => Ok(Json(serde_json::to_value(e).unwrap_or_default())),
        Ok(None) => Err((axum::http::StatusCode::NOT_FOUND, Json(json!({"error": "Billing event not found"})))),
        Err(e) => Err(error_response(e)),
    }
}

pub async fn list_billing_events(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(filters): Query<BillingFilters>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;

    match state.project_billing_engine.list_billing_events(
        org_id, filters.project_id, filters.status.as_deref(),
    ).await {
        Ok(events) => Ok(Json(json!({"data": events}))),
        Err(e) => Err(error_response(e)),
    }
}

pub async fn complete_billing_event(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<CompleteBillingEventRequest>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    match state.project_billing_engine.complete_billing_event(
        id, req.actual_date, req.completion_pct,
    ).await {
        Ok(e) => Ok(Json(serde_json::to_value(e).unwrap_or_default())),
        Err(e) => {
            error!("Failed to complete billing event: {}", e);
            Err(error_response(e))
        }
    }
}

pub async fn cancel_billing_event(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    match state.project_billing_engine.cancel_billing_event(id).await {
        Ok(e) => Ok(Json(serde_json::to_value(e).unwrap_or_default())),
        Err(e) => Err(error_response(e)),
    }
}

pub async fn delete_billing_event(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(event_number): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;

    match state.project_billing_engine.delete_billing_event(org_id, &event_number).await {
        Ok(()) => Ok(Json(json!({"message": "Billing event deleted"}))),
        Err(e) => Err(error_response(e)),
    }
}

// ============================================================================
// Project Invoice Handlers
// ============================================================================

pub async fn create_invoice(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreateInvoiceRequest>,
) -> Result<(axum::http::StatusCode, Json<serde_json::Value>), (axum::http::StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    let created_by = parse_uuid(&claims.sub).ok();

    let lines: Vec<atlas_core::project_billing::engine::InvoiceLineRequest> = req.lines.into_iter().map(|l| {
        atlas_core::project_billing::engine::InvoiceLineRequest {
            line_source: l.line_source,
            expenditure_item_id: l.expenditure_item_id,
            billing_event_id: l.billing_event_id,
            task_id: l.task_id,
            task_number: l.task_number,
            task_name: l.task_name,
            description: l.description,
            employee_id: l.employee_id,
            employee_name: l.employee_name,
            role_name: l.role_name,
            expenditure_type: l.expenditure_type,
            quantity: l.quantity,
            unit_of_measure: l.unit_of_measure,
            bill_rate: l.bill_rate,
            raw_cost_amount: l.raw_cost_amount,
            bill_amount: l.bill_amount,
            markup_amount: l.markup_amount,
            tax_amount: l.tax_amount,
            transaction_date: l.transaction_date,
        }
    }).collect();

    match state.project_billing_engine.create_invoice(
        org_id, &req.invoice_number, req.project_id,
        req.project_number.as_deref(), req.project_name.as_deref(),
        &req.invoice_type, req.customer_id, req.customer_name.as_deref(),
        req.billing_event_id, lines,
        req.customer_po_number.as_deref(), req.contract_number.as_deref(),
        req.notes.as_deref(), created_by,
    ).await {
        Ok(invoice) => Ok((axum::http::StatusCode::CREATED, Json(serde_json::to_value(invoice).unwrap_or_default()))),
        Err(e) => {
            error!("Failed to create invoice: {}", e);
            Err(error_response(e))
        }
    }
}

pub async fn get_invoice(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    match state.project_billing_engine.get_invoice(id).await {
        Ok(Some(i)) => Ok(Json(serde_json::to_value(i).unwrap_or_default())),
        Ok(None) => Err((axum::http::StatusCode::NOT_FOUND, Json(json!({"error": "Invoice not found"})))),
        Err(e) => Err(error_response(e)),
    }
}

pub async fn list_invoices(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(filters): Query<BillingFilters>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;

    match state.project_billing_engine.list_invoices(
        org_id, filters.project_id, filters.status.as_deref(),
    ).await {
        Ok(invoices) => Ok(Json(json!({"data": invoices}))),
        Err(e) => Err(error_response(e)),
    }
}

pub async fn get_invoice_lines(
    State(state): State<Arc<AppState>>,
    Path(invoice_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    match state.project_billing_engine.get_invoice_lines(invoice_id).await {
        Ok(lines) => Ok(Json(json!({"data": lines}))),
        Err(e) => Err(error_response(e)),
    }
}

pub async fn submit_invoice(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    match state.project_billing_engine.submit_invoice(id).await {
        Ok(i) => Ok(Json(serde_json::to_value(i).unwrap_or_default())),
        Err(e) => {
            error!("Failed to submit invoice: {}", e);
            Err(error_response(e))
        }
    }
}

pub async fn approve_invoice(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    match state.project_billing_engine.approve_invoice(id).await {
        Ok(i) => Ok(Json(serde_json::to_value(i).unwrap_or_default())),
        Err(e) => {
            error!("Failed to approve invoice: {}", e);
            Err(error_response(e))
        }
    }
}

pub async fn reject_invoice(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<RejectInvoiceRequest>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    match state.project_billing_engine.reject_invoice(id, &req.reason).await {
        Ok(i) => Ok(Json(serde_json::to_value(i).unwrap_or_default())),
        Err(e) => {
            error!("Failed to reject invoice: {}", e);
            Err(error_response(e))
        }
    }
}

pub async fn post_invoice(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    match state.project_billing_engine.post_invoice(id).await {
        Ok(i) => Ok(Json(serde_json::to_value(i).unwrap_or_default())),
        Err(e) => {
            error!("Failed to post invoice: {}", e);
            Err(error_response(e))
        }
    }
}

pub async fn cancel_invoice(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    match state.project_billing_engine.cancel_invoice(id).await {
        Ok(i) => Ok(Json(serde_json::to_value(i).unwrap_or_default())),
        Err(e) => {
            error!("Failed to cancel invoice: {}", e);
            Err(error_response(e))
        }
    }
}

// ============================================================================
// Dashboard
// ============================================================================

pub async fn get_project_billing_dashboard(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;

    match state.project_billing_engine.get_dashboard(org_id).await {
        Ok(dashboard) => Ok(Json(serde_json::to_value(dashboard).unwrap_or_default())),
        Err(e) => Err(error_response(e)),
    }
}
