//! Accounts Payable Handlers
//!
//! Oracle Fusion Cloud ERP: Payables > Invoices, Payments, Holds
//!
//! API endpoints for managing supplier invoices, invoice lines, distributions,
//! holds, payments, and AP aging reporting.

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
use tracing::{info, error};

// ============================================================================
// Invoice Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateApInvoiceRequest {
    pub invoice_number: String,
    pub invoice_date: chrono::NaiveDate,
    #[serde(default = "default_invoice_type")]
    pub invoice_type: String,
    pub description: Option<String>,
    pub supplier_id: Uuid,
    pub supplier_number: Option<String>,
    pub supplier_name: Option<String>,
    pub supplier_site: Option<String>,
    #[serde(default = "default_currency_usd")]
    pub invoice_currency_code: String,
    #[serde(default = "default_currency_usd")]
    pub payment_currency_code: String,
    pub exchange_rate: Option<String>,
    pub exchange_rate_type: Option<String>,
    pub exchange_date: Option<chrono::NaiveDate>,
    pub invoice_amount: String,
    #[serde(default = "default_zero")]
    pub tax_amount: String,
    pub payment_terms: Option<String>,
    pub payment_method: Option<String>,
    pub payment_due_date: Option<chrono::NaiveDate>,
    pub discount_date: Option<chrono::NaiveDate>,
    pub gl_date: Option<chrono::NaiveDate>,
    pub po_number: Option<String>,
    pub receipt_number: Option<String>,
    pub source: Option<String>,
}

fn default_invoice_type() -> String { "standard".to_string() }
fn default_currency_usd() -> String { "USD".to_string() }
fn default_zero() -> String { "0.00".to_string() }

/// Create a new AP invoice
pub async fn create_ap_invoice(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateApInvoiceRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    info!("Creating AP invoice '{}' for org {}", payload.invoice_number, org_id);

    match state.accounts_payable_engine.create_invoice(
        org_id,
        &payload.invoice_number,
        payload.invoice_date,
        &payload.invoice_type,
        payload.description.as_deref(),
        payload.supplier_id,
        payload.supplier_number.as_deref(),
        payload.supplier_name.as_deref(),
        payload.supplier_site.as_deref(),
        &payload.invoice_currency_code,
        &payload.payment_currency_code,
        payload.exchange_rate.as_deref(),
        payload.exchange_rate_type.as_deref(),
        payload.exchange_date,
        &payload.invoice_amount,
        &payload.tax_amount,
        payload.payment_terms.as_deref(),
        payload.payment_method.as_deref(),
        payload.payment_due_date,
        payload.discount_date,
        payload.gl_date,
        payload.po_number.as_deref(),
        payload.receipt_number.as_deref(),
        payload.source.as_deref(),
        Some(user_id),
    ).await {
        Ok(invoice) => Ok((StatusCode::CREATED, Json(serde_json::to_value(invoice).unwrap_or_default()))),
        Err(e) => {
            error!("Failed to create AP invoice: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                409 => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

/// Get an AP invoice by ID
pub async fn get_ap_invoice(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.accounts_payable_engine.get_invoice(id).await {
        Ok(Some(invoice)) => Ok(Json(serde_json::to_value(invoice).unwrap_or_default())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get AP invoice: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListApInvoicesQuery {
    pub supplier_id: Option<Uuid>,
    pub status: Option<String>,
    pub invoice_type: Option<String>,
}

/// List AP invoices with optional filters
pub async fn list_ap_invoices(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListApInvoicesQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.accounts_payable_engine.list_invoices(
        org_id, query.supplier_id, query.status.as_deref(), query.invoice_type.as_deref(),
    ).await {
        Ok(invoices) => Ok(Json(serde_json::json!({ "data": invoices }))),
        Err(e) => {
            error!("Failed to list AP invoices: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

/// Submit an AP invoice for approval
pub async fn submit_ap_invoice(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.accounts_payable_engine.submit_invoice(id).await {
        Ok(invoice) => Ok(Json(serde_json::to_value(invoice).unwrap_or_default())),
        Err(e) => {
            error!("Failed to submit AP invoice: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

/// Approve an AP invoice
pub async fn approve_ap_invoice(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.accounts_payable_engine.approve_invoice(id, user_id).await {
        Ok(invoice) => Ok(Json(serde_json::to_value(invoice).unwrap_or_default())),
        Err(e) => {
            error!("Failed to approve AP invoice: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CancelApInvoiceRequest {
    pub reason: Option<String>,
}

/// Cancel an AP invoice
pub async fn cancel_ap_invoice(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<CancelApInvoiceRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.accounts_payable_engine.cancel_invoice(id, user_id, payload.reason.as_deref()).await {
        Ok(invoice) => Ok(Json(serde_json::to_value(invoice).unwrap_or_default())),
        Err(e) => {
            error!("Failed to cancel AP invoice: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// ============================================================================
// Invoice Line Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct AddApInvoiceLineRequest {
    #[serde(default = "default_item_line_type")]
    pub line_type: String,
    pub description: Option<String>,
    pub amount: String,
    pub unit_price: Option<String>,
    pub quantity_invoiced: Option<String>,
    pub unit_of_measure: Option<String>,
    pub po_line_id: Option<Uuid>,
    pub po_line_number: Option<String>,
    pub product_code: Option<String>,
    pub tax_code: Option<String>,
    pub tax_amount: Option<String>,
}

fn default_item_line_type() -> String { "item".to_string() }

/// Add a line to an AP invoice
pub async fn add_ap_invoice_line(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(invoice_id): Path<Uuid>,
    Json(payload): Json<AddApInvoiceLineRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    info!("Adding line to AP invoice {}", invoice_id);

    match state.accounts_payable_engine.add_line(
        org_id, invoice_id, &payload.line_type,
        payload.description.as_deref(), &payload.amount,
        payload.unit_price.as_deref(), payload.quantity_invoiced.as_deref(),
        payload.unit_of_measure.as_deref(), payload.po_line_id,
        payload.po_line_number.as_deref(), payload.product_code.as_deref(),
        payload.tax_code.as_deref(), payload.tax_amount.as_deref(),
        Some(user_id),
    ).await {
        Ok(line) => Ok((StatusCode::CREATED, Json(serde_json::to_value(line).unwrap_or_default()))),
        Err(e) => {
            error!("Failed to add AP invoice line: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

/// List lines for an AP invoice
pub async fn list_ap_invoice_lines(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(invoice_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.accounts_payable_engine.list_lines(invoice_id).await {
        Ok(lines) => Ok(Json(serde_json::json!({ "data": lines }))),
        Err(e) => {
            error!("Failed to list AP invoice lines: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Delete an AP invoice line
pub async fn delete_ap_invoice_line(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path((invoice_id, line_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, StatusCode> {
    match state.accounts_payable_engine.delete_line(invoice_id, line_id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            error!("Failed to delete AP invoice line: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// ============================================================================
// Invoice Distribution Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct AddApDistributionRequest {
    pub invoice_line_id: Option<Uuid>,
    #[serde(default = "default_charge_dist")]
    pub distribution_type: String,
    pub account_combination: Option<String>,
    pub description: Option<String>,
    pub amount: String,
    #[serde(default = "default_currency_usd")]
    pub currency_code: String,
    pub exchange_rate: Option<String>,
    pub gl_account: Option<String>,
    pub cost_center: Option<String>,
    pub department: Option<String>,
    pub project_id: Option<Uuid>,
    pub task_id: Option<Uuid>,
    pub expenditure_type: Option<String>,
    pub tax_code: Option<String>,
    #[serde(default)]
    pub tax_recoverable: bool,
    pub tax_recoverable_amount: Option<String>,
    pub accounting_date: Option<chrono::NaiveDate>,
}

fn default_charge_dist() -> String { "charge".to_string() }

/// Add a distribution to an AP invoice
pub async fn add_ap_distribution(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(invoice_id): Path<Uuid>,
    Json(payload): Json<AddApDistributionRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    info!("Adding distribution to AP invoice {}", invoice_id);

    match state.accounts_payable_engine.add_distribution(
        org_id, invoice_id, payload.invoice_line_id,
        &payload.distribution_type, payload.account_combination.as_deref(),
        payload.description.as_deref(), &payload.amount,
        &payload.currency_code, payload.exchange_rate.as_deref(),
        payload.gl_account.as_deref(), payload.cost_center.as_deref(),
        payload.department.as_deref(), payload.project_id, payload.task_id,
        payload.expenditure_type.as_deref(), payload.tax_code.as_deref(),
        payload.tax_recoverable, payload.tax_recoverable_amount.as_deref(),
        payload.accounting_date, Some(user_id),
    ).await {
        Ok(dist) => Ok((StatusCode::CREATED, Json(serde_json::to_value(dist).unwrap_or_default()))),
        Err(e) => {
            error!("Failed to add AP distribution: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

/// List distributions for an AP invoice
pub async fn list_ap_distributions(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(invoice_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.accounts_payable_engine.list_distributions(invoice_id).await {
        Ok(distributions) => Ok(Json(serde_json::json!({ "data": distributions }))),
        Err(e) => {
            error!("Failed to list AP distributions: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// ============================================================================
// Hold Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ApplyApHoldRequest {
    pub hold_type: String,
    pub hold_reason: String,
}

/// Apply a hold to an AP invoice
pub async fn apply_ap_hold(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(invoice_id): Path<Uuid>,
    Json(payload): Json<ApplyApHoldRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.accounts_payable_engine.apply_hold(
        org_id, invoice_id, &payload.hold_type, &payload.hold_reason, Some(user_id),
    ).await {
        Ok(hold) => Ok((StatusCode::CREATED, Json(serde_json::to_value(hold).unwrap_or_default()))),
        Err(e) => {
            error!("Failed to apply AP hold: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ReleaseApHoldRequest {
    pub release_reason: Option<String>,
}

/// Release a hold
pub async fn release_ap_hold(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(hold_id): Path<Uuid>,
    Json(payload): Json<ReleaseApHoldRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.accounts_payable_engine.release_hold(
        hold_id, user_id, payload.release_reason.as_deref(),
    ).await {
        Ok(hold) => Ok(Json(serde_json::to_value(hold).unwrap_or_default())),
        Err(e) => {
            error!("Failed to release AP hold: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

/// List holds for an AP invoice
pub async fn list_ap_holds(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(invoice_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.accounts_payable_engine.list_holds(invoice_id).await {
        Ok(holds) => Ok(Json(serde_json::json!({ "data": holds }))),
        Err(e) => {
            error!("Failed to list AP holds: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// ============================================================================
// Payment Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateApPaymentRequest {
    pub payment_number: String,
    pub payment_date: chrono::NaiveDate,
    #[serde(default = "default_check_payment")]
    pub payment_method: String,
    #[serde(default = "default_currency_usd")]
    pub payment_currency_code: String,
    pub payment_amount: String,
    pub bank_account_id: Option<Uuid>,
    pub bank_account_name: Option<String>,
    pub payment_document: Option<String>,
    pub supplier_id: Uuid,
    pub supplier_number: Option<String>,
    pub supplier_name: Option<String>,
    pub invoice_ids: Vec<Uuid>,
}

fn default_check_payment() -> String { "check".to_string() }

/// Create a payment for AP invoices
pub async fn create_ap_payment(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateApPaymentRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    info!("Creating AP payment '{}' for supplier {}", payload.payment_number, payload.supplier_id);

    match state.accounts_payable_engine.create_payment(
        org_id, &payload.payment_number, payload.payment_date,
        &payload.payment_method, &payload.payment_currency_code,
        &payload.payment_amount, payload.bank_account_id,
        payload.bank_account_name.as_deref(), payload.payment_document.as_deref(),
        payload.supplier_id, payload.supplier_number.as_deref(),
        payload.supplier_name.as_deref(), &payload.invoice_ids,
        Some(user_id),
    ).await {
        Ok(payment) => Ok((StatusCode::CREATED, Json(serde_json::to_value(payment).unwrap_or_default()))),
        Err(e) => {
            error!("Failed to create AP payment: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

/// Get an AP payment by ID
pub async fn get_ap_payment(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.accounts_payable_engine.get_payment(id).await {
        Ok(Some(payment)) => Ok(Json(serde_json::to_value(payment).unwrap_or_default())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get AP payment: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListApPaymentsQuery {
    pub supplier_id: Option<Uuid>,
    pub status: Option<String>,
}

/// List AP payments
pub async fn list_ap_payments(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListApPaymentsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.accounts_payable_engine.list_payments(
        org_id, query.supplier_id, query.status.as_deref(),
    ).await {
        Ok(payments) => Ok(Json(serde_json::json!({ "data": payments }))),
        Err(e) => {
            error!("Failed to list AP payments: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Confirm an AP payment
pub async fn confirm_ap_payment(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.accounts_payable_engine.confirm_payment(id, user_id).await {
        Ok(payment) => Ok(Json(serde_json::to_value(payment).unwrap_or_default())),
        Err(e) => {
            error!("Failed to confirm AP payment: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// ============================================================================
// AP Aging
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ApAgingQuery {
    pub as_of_date: Option<chrono::NaiveDate>,
}

/// Get AP aging summary
pub async fn get_ap_aging(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ApAgingQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let as_of_date = query.as_of_date.unwrap_or_else(|| chrono::Utc::now().date_naive());

    match state.accounts_payable_engine.get_aging_summary(org_id, as_of_date).await {
        Ok(summary) => Ok(Json(serde_json::to_value(summary).unwrap_or_default())),
        Err(e) => {
            error!("Failed to get AP aging: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
