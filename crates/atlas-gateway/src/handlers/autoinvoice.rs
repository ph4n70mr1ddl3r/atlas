//! AutoInvoice Handlers
//!
//! Oracle Fusion Cloud Receivables: AutoInvoice
//!
//! API endpoints for automated invoice creation from imported transaction data
//! with configurable grouping rules, validation rules, batch processing,
//! and invoice lifecycle management.

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
use tracing::error;

// ============================================================================
// Grouping Rule Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateGroupingRuleRequest {
    pub name: String,
    pub description: Option<String>,
    pub transaction_types: Option<serde_json::Value>,
    pub group_by_fields: Option<serde_json::Value>,
    pub line_order_by: Option<serde_json::Value>,
    pub is_default: Option<bool>,
    pub priority: Option<i32>,
}

pub async fn create_grouping_rule(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateGroupingRuleRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).ok();

    match state.autoinvoice_engine.create_grouping_rule(
        org_id,
        &payload.name,
        payload.description.as_deref(),
        payload.transaction_types.unwrap_or(serde_json::json!(["invoice", "credit_memo"])),
        payload.group_by_fields.unwrap_or(serde_json::json!(["bill_to_customer_id", "currency_code", "transaction_type"])),
        payload.line_order_by.unwrap_or(serde_json::json!(["line_number"])),
        payload.is_default.unwrap_or(false),
        payload.priority.unwrap_or(10),
        user_id,
    ).await {
        Ok(rule) => Ok((StatusCode::CREATED, Json(serde_json::to_value(rule).unwrap_or_default()))),
        Err(e) => {
            error!("Failed to create grouping rule: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 409 => StatusCode::CONFLICT, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

pub async fn list_grouping_rules(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.autoinvoice_engine.list_grouping_rules(org_id).await {
        Ok(rules) => Ok(Json(serde_json::json!({"data": rules}))),
        Err(e) => { error!("Error listing grouping rules: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn get_grouping_rule(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.autoinvoice_engine.get_grouping_rule(id).await {
        Ok(Some(rule)) => Ok(Json(serde_json::to_value(rule).unwrap_or_default())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn delete_grouping_rule(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    match state.autoinvoice_engine.delete_grouping_rule(id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// ============================================================================
// Validation Rule Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateValidationRuleRequest {
    pub name: String,
    pub description: Option<String>,
    pub field_name: String,
    pub validation_type: String,
    pub validation_expression: Option<String>,
    pub error_message: String,
    pub is_fatal: Option<bool>,
    pub transaction_types: Option<serde_json::Value>,
    pub priority: Option<i32>,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
}

pub async fn create_validation_rule(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateValidationRuleRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).ok();

    match state.autoinvoice_engine.create_validation_rule(
        org_id,
        &payload.name,
        payload.description.as_deref(),
        &payload.field_name,
        &payload.validation_type,
        payload.validation_expression.as_deref(),
        &payload.error_message,
        payload.is_fatal.unwrap_or(true),
        payload.transaction_types.unwrap_or(serde_json::json!(["invoice", "credit_memo", "debit_memo"])),
        payload.priority.unwrap_or(10),
        payload.effective_from,
        payload.effective_to,
        user_id,
    ).await {
        Ok(rule) => Ok((StatusCode::CREATED, Json(serde_json::to_value(rule).unwrap_or_default()))),
        Err(e) => {
            error!("Failed to create validation rule: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

pub async fn list_validation_rules(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.autoinvoice_engine.list_validation_rules(org_id).await {
        Ok(rules) => Ok(Json(serde_json::json!({"data": rules}))),
        Err(e) => { error!("Error listing validation rules: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn delete_validation_rule(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    match state.autoinvoice_engine.delete_validation_rule(id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// ============================================================================
// Batch Import & Processing Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ImportBatchRequest {
    pub batch_source: String,
    pub description: Option<String>,
    pub grouping_rule_id: Option<Uuid>,
    pub lines: Vec<ImportLineRequest>,
}

#[derive(Debug, Deserialize)]
pub struct ImportLineRequest {
    pub source_line_id: Option<String>,
    pub transaction_type: Option<String>,
    pub customer_id: Option<Uuid>,
    pub customer_number: Option<String>,
    pub customer_name: Option<String>,
    pub bill_to_customer_id: Option<Uuid>,
    pub bill_to_site_id: Option<Uuid>,
    pub ship_to_customer_id: Option<Uuid>,
    pub ship_to_site_id: Option<Uuid>,
    pub item_code: Option<String>,
    pub item_description: Option<String>,
    pub quantity: Option<String>,
    pub unit_of_measure: Option<String>,
    pub unit_price: Option<String>,
    pub line_amount: Option<String>,
    pub currency_code: String,
    pub exchange_rate: Option<String>,
    pub transaction_date: Option<chrono::NaiveDate>,
    pub gl_date: Option<chrono::NaiveDate>,
    pub due_date: Option<chrono::NaiveDate>,
    pub revenue_account_code: Option<String>,
    pub receivable_account_code: Option<String>,
    pub tax_code: Option<String>,
    pub tax_amount: Option<String>,
    pub sales_rep_id: Option<Uuid>,
    pub sales_rep_name: Option<String>,
    pub memo_line: Option<String>,
    pub reference_number: Option<String>,
    pub sales_order_number: Option<String>,
    pub sales_order_line: Option<String>,
}

pub async fn import_batch(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<ImportBatchRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).ok();

    let import_request = atlas_shared::AutoInvoiceImportRequest {
        batch_source: payload.batch_source,
        description: payload.description,
        grouping_rule_id: payload.grouping_rule_id,
        lines: payload.lines.into_iter().map(|l| atlas_shared::AutoInvoiceLineRequest {
            source_line_id: l.source_line_id,
            transaction_type: l.transaction_type,
            customer_id: l.customer_id,
            customer_number: l.customer_number,
            customer_name: l.customer_name,
            bill_to_customer_id: l.bill_to_customer_id,
            bill_to_site_id: l.bill_to_site_id,
            ship_to_customer_id: l.ship_to_customer_id,
            ship_to_site_id: l.ship_to_site_id,
            item_code: l.item_code,
            item_description: l.item_description,
            quantity: l.quantity,
            unit_of_measure: l.unit_of_measure,
            unit_price: l.unit_price,
            line_amount: l.line_amount,
            currency_code: l.currency_code,
            exchange_rate: l.exchange_rate,
            transaction_date: l.transaction_date,
            gl_date: l.gl_date,
            due_date: l.due_date,
            revenue_account_code: l.revenue_account_code,
            receivable_account_code: l.receivable_account_code,
            tax_code: l.tax_code,
            tax_amount: l.tax_amount,
            sales_rep_id: l.sales_rep_id,
            sales_rep_name: l.sales_rep_name,
            memo_line: l.memo_line,
            reference_number: l.reference_number,
            sales_order_number: l.sales_order_number,
            sales_order_line: l.sales_order_line,
        }).collect(),
    };

    match state.autoinvoice_engine.import_batch(org_id, &import_request, user_id).await {
        Ok(batch) => Ok((StatusCode::CREATED, Json(serde_json::to_value(batch).unwrap_or_default()))),
        Err(e) => {
            error!("Failed to import batch: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

pub async fn list_batches(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<ListBatchesQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.autoinvoice_engine.list_batches(org_id, params.status.as_deref()).await {
        Ok(batches) => Ok(Json(serde_json::json!({"data": batches}))),
        Err(e) => { error!("Error listing batches: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListBatchesQuery {
    pub status: Option<String>,
}

pub async fn get_batch(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.autoinvoice_engine.get_batch(id).await {
        Ok(Some(batch)) => Ok(Json(serde_json::to_value(batch).unwrap_or_default())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn validate_batch(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.autoinvoice_engine.validate_batch(id).await {
        Ok(batch) => Ok(Json(serde_json::to_value(batch).unwrap_or_default())),
        Err(e) => {
            error!("Failed to validate batch: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

pub async fn process_batch(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.autoinvoice_engine.process_batch(id).await {
        Ok(batch) => Ok(Json(serde_json::to_value(batch).unwrap_or_default())),
        Err(e) => {
            error!("Failed to process batch: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

pub async fn import_and_process(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<ImportBatchRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).ok();

    let import_request = atlas_shared::AutoInvoiceImportRequest {
        batch_source: payload.batch_source,
        description: payload.description,
        grouping_rule_id: payload.grouping_rule_id,
        lines: payload.lines.into_iter().map(|l| atlas_shared::AutoInvoiceLineRequest {
            source_line_id: l.source_line_id,
            transaction_type: l.transaction_type,
            customer_id: l.customer_id,
            customer_number: l.customer_number,
            customer_name: l.customer_name,
            bill_to_customer_id: l.bill_to_customer_id,
            bill_to_site_id: l.bill_to_site_id,
            ship_to_customer_id: l.ship_to_customer_id,
            ship_to_site_id: l.ship_to_site_id,
            item_code: l.item_code,
            item_description: l.item_description,
            quantity: l.quantity,
            unit_of_measure: l.unit_of_measure,
            unit_price: l.unit_price,
            line_amount: l.line_amount,
            currency_code: l.currency_code,
            exchange_rate: l.exchange_rate,
            transaction_date: l.transaction_date,
            gl_date: l.gl_date,
            due_date: l.due_date,
            revenue_account_code: l.revenue_account_code,
            receivable_account_code: l.receivable_account_code,
            tax_code: l.tax_code,
            tax_amount: l.tax_amount,
            sales_rep_id: l.sales_rep_id,
            sales_rep_name: l.sales_rep_name,
            memo_line: l.memo_line,
            reference_number: l.reference_number,
            sales_order_number: l.sales_order_number,
            sales_order_line: l.sales_order_line,
        }).collect(),
    };

    match state.autoinvoice_engine.import_and_process(org_id, &import_request, user_id).await {
        Ok(batch) => Ok((StatusCode::CREATED, Json(serde_json::to_value(batch).unwrap_or_default()))),
        Err(e) => {
            error!("Failed to import and process: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

// ============================================================================
// Batch Lines & Results Handlers
// ============================================================================

pub async fn get_batch_lines(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.autoinvoice_engine.get_batch_lines(id).await {
        Ok(lines) => Ok(Json(serde_json::json!({"data": lines}))),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn get_batch_results(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.autoinvoice_engine.get_batch_results(id).await {
        Ok(results) => Ok(Json(serde_json::json!({"data": results}))),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn get_invoice(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.autoinvoice_engine.get_invoice_lines(id).await {
        Ok(lines) => Ok(Json(serde_json::json!({"data": lines}))),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateInvoiceStatusRequest {
    pub status: String,
}

pub async fn update_invoice_status(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateInvoiceStatusRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.autoinvoice_engine.update_invoice_status(id, &payload.status).await {
        Ok(invoice) => Ok(Json(serde_json::to_value(invoice).unwrap_or_default())),
        Err(e) => {
            error!("Failed to update invoice status: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 404 => StatusCode::NOT_FOUND, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

// ============================================================================
// Dashboard
// ============================================================================

pub async fn get_autoinvoice_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.autoinvoice_engine.get_summary(org_id).await {
        Ok(summary) => Ok(Json(serde_json::to_value(summary).unwrap_or_default())),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}
