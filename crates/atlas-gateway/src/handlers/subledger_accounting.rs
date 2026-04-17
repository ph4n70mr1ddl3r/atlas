//! Subledger Accounting Handlers
//!
//! Oracle Fusion Cloud ERP: Financials > General Ledger > Subledger Accounting
//!
//! API endpoints for managing accounting methods, derivation rules,
//! journal entries, journal lines, posting, reversal, transfer to GL,
//! and SLA dashboard.

use axum::{
    extract::{State, Path, Query},
    Json,
    http::StatusCode,
    Extension,
};
use serde::{Deserialize, Serialize};
use crate::AppState;
use crate::handlers::auth::Claims;
use std::sync::Arc;
use uuid::Uuid;
use tracing::{info, error};

// ============================================================================
// Accounting Methods
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateAccountingMethodRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub application: String,
    pub transaction_type: String,
    pub event_class: Option<String>,
    pub auto_accounting: Option<bool>,
    pub allow_manual_entries: Option<bool>,
    pub apply_rounding: Option<bool>,
    pub rounding_account_code: Option<String>,
    pub rounding_threshold: Option<String>,
    pub require_balancing: Option<bool>,
    pub intercompany_balancing_account: Option<String>,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
}

/// Create or update an accounting method
pub async fn create_accounting_method(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateAccountingMethodRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    info!("Creating accounting method {} for org {}", payload.code, org_id);

    match state.sla_engine.create_accounting_method(
        org_id, &payload.code, &payload.name, payload.description.as_deref(),
        &payload.application, &payload.transaction_type,
        payload.event_class.as_deref(),
        payload.auto_accounting, payload.allow_manual_entries,
        payload.apply_rounding, payload.rounding_account_code.as_deref(),
        payload.rounding_threshold.as_deref(),
        payload.require_balancing, payload.intercompany_balancing_account.as_deref(),
        payload.effective_from, payload.effective_to,
        Some(user_id),
    ).await {
        Ok(method) => Ok((StatusCode::CREATED, Json(serde_json::to_value(method).unwrap_or_default()))),
        Err(e) => {
            error!("Failed to create accounting method: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

/// Get an accounting method by code
pub async fn get_accounting_method(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(code): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.sla_engine.get_accounting_method(org_id, &code).await {
        Ok(Some(method)) => Ok(Json(serde_json::to_value(method).unwrap_or_default())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListAccountingMethodsParams {
    pub application: Option<String>,
}

/// List accounting methods
pub async fn list_accounting_methods(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<ListAccountingMethodsParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.sla_engine.list_accounting_methods(org_id, params.application.as_deref()).await {
        Ok(methods) => Ok(Json(serde_json::json!({"data": methods}))),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

/// Delete an accounting method
pub async fn delete_accounting_method(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(code): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.sla_engine.delete_accounting_method(org_id, &code).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::NOT_FOUND) }
    }
}

// ============================================================================
// Accounting Derivation Rules
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateDerivationRuleRequest {
    pub accounting_method_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub line_type: String,
    pub priority: Option<i32>,
    pub conditions: serde_json::Value,
    pub source_field: Option<String>,
    pub derivation_type: String,
    pub fixed_account_code: Option<String>,
    pub account_derivation_lookup: Option<serde_json::Value>,
    pub formula_expression: Option<String>,
    pub sequence: Option<i32>,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
}

/// Create a derivation rule
pub async fn create_derivation_rule(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateDerivationRuleRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.sla_engine.create_derivation_rule(
        org_id, payload.accounting_method_id,
        &payload.code, &payload.name, payload.description.as_deref(),
        &payload.line_type, payload.priority.unwrap_or(10),
        payload.conditions.clone(), payload.source_field.as_deref(),
        &payload.derivation_type, payload.fixed_account_code.as_deref(),
        payload.account_derivation_lookup.clone().unwrap_or(serde_json::json!({})),
        payload.formula_expression.as_deref(),
        payload.sequence.unwrap_or(10),
        payload.effective_from, payload.effective_to,
        Some(user_id),
    ).await {
        Ok(rule) => Ok((StatusCode::CREATED, Json(serde_json::to_value(rule).unwrap_or_default()))),
        Err(e) => {
            error!("Failed to create derivation rule: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListDerivationRulesParams {
    pub line_type: Option<String>,
}

/// List derivation rules for an accounting method
pub async fn list_derivation_rules(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(method_id): Path<Uuid>,
    Query(params): Query<ListDerivationRulesParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let rules = if let Some(lt) = params.line_type.as_deref() {
        state.sla_engine.list_active_derivation_rules(org_id, method_id, lt).await
    } else {
        state.sla_engine.list_derivation_rules(org_id, method_id).await
    };

    match rules {
        Ok(rules) => Ok(Json(serde_json::json!({"data": rules}))),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

/// Delete a derivation rule
pub async fn delete_derivation_rule(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path((method_id, code)): Path<(Uuid, String)>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.sla_engine.delete_derivation_rule(org_id, method_id, &code).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::NOT_FOUND) }
    }
}

// ============================================================================
// Journal Entries
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateJournalEntryRequest {
    pub source_application: String,
    pub source_transaction_type: String,
    pub source_transaction_id: Uuid,
    pub source_transaction_number: Option<String>,
    pub accounting_method_id: Option<Uuid>,
    pub description: Option<String>,
    pub reference_number: Option<String>,
    pub accounting_date: chrono::NaiveDate,
    pub period_name: Option<String>,
    pub currency_code: Option<String>,
    pub entered_currency_code: Option<String>,
    pub currency_conversion_date: Option<chrono::NaiveDate>,
    pub currency_conversion_type: Option<String>,
    pub currency_conversion_rate: Option<String>,
}

/// Create a subledger journal entry
pub async fn create_journal_entry(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateJournalEntryRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.sla_engine.create_journal_entry(
        org_id, &payload.source_application, &payload.source_transaction_type,
        payload.source_transaction_id, payload.source_transaction_number.as_deref(),
        payload.accounting_method_id, payload.description.as_deref(),
        payload.reference_number.as_deref(), payload.accounting_date,
        payload.period_name.as_deref(),
        payload.currency_code.as_deref().unwrap_or("USD"),
        payload.entered_currency_code.as_deref().unwrap_or("USD"),
        payload.currency_conversion_date, payload.currency_conversion_type.as_deref(),
        payload.currency_conversion_rate.as_deref(),
        Some(user_id),
    ).await {
        Ok(entry) => Ok((StatusCode::CREATED, Json(serde_json::to_value(entry).unwrap_or_default()))),
        Err(e) => {
            error!("Failed to create journal entry: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

/// Get a journal entry
pub async fn get_journal_entry(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.sla_engine.get_journal_entry(id).await {
        Ok(Some(entry)) => Ok(Json(serde_json::to_value(entry).unwrap_or_default())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListJournalEntriesParams {
    pub status: Option<String>,
    pub source_application: Option<String>,
    pub source_transaction_type: Option<String>,
    pub accounting_date_from: Option<chrono::NaiveDate>,
    pub accounting_date_to: Option<chrono::NaiveDate>,
}

/// List journal entries
pub async fn list_journal_entries(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<ListJournalEntriesParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.sla_engine.list_journal_entries(
        org_id, params.status.as_deref(), params.source_application.as_deref(),
        params.source_transaction_type.as_deref(),
        params.accounting_date_from, params.accounting_date_to,
    ).await {
        Ok(entries) => Ok(Json(serde_json::json!({"data": entries}))),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

/// Get journal lines for an entry
pub async fn list_journal_lines(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(entry_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.sla_engine.list_journal_lines(entry_id).await {
        Ok(lines) => Ok(Json(serde_json::json!({"data": lines}))),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

#[derive(Debug, Deserialize)]
pub struct AddJournalLineRequest {
    pub line_type: String,
    pub account_code: String,
    pub account_description: Option<String>,
    pub derivation_rule_id: Option<Uuid>,
    pub entered_amount: String,
    pub accounted_amount: String,
    pub currency_code: Option<String>,
    pub conversion_date: Option<chrono::NaiveDate>,
    pub conversion_rate: Option<String>,
    pub attribute_category: Option<String>,
    pub attribute1: Option<String>,
    pub attribute2: Option<String>,
    pub attribute3: Option<String>,
    pub attribute4: Option<String>,
    pub attribute5: Option<String>,
    pub source_line_id: Option<Uuid>,
    pub source_line_type: Option<String>,
    pub tax_code: Option<String>,
    pub tax_rate: Option<String>,
    pub tax_amount: Option<String>,
}

/// Add a journal line to an entry
pub async fn add_journal_line(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(entry_id): Path<Uuid>,
    Json(payload): Json<AddJournalLineRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.sla_engine.add_journal_line(
        org_id, entry_id,
        &payload.line_type, &payload.account_code,
        payload.account_description.as_deref(), payload.derivation_rule_id,
        &payload.entered_amount, &payload.accounted_amount,
        payload.currency_code.as_deref().unwrap_or("USD"),
        payload.conversion_date, payload.conversion_rate.as_deref(),
        payload.attribute_category.as_deref(),
        payload.attribute1.as_deref(), payload.attribute2.as_deref(),
        payload.attribute3.as_deref(), payload.attribute4.as_deref(),
        payload.attribute5.as_deref(),
        payload.source_line_id, payload.source_line_type.as_deref(),
        payload.tax_code.as_deref(), payload.tax_rate.as_deref(),
        payload.tax_amount.as_deref(),
    ).await {
        Ok(line) => Ok((StatusCode::CREATED, Json(serde_json::to_value(line).unwrap_or_default()))),
        Err(e) => {
            error!("Failed to add journal line: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

// ============================================================================
// Posting / Status Transitions
// ============================================================================

/// Account a draft entry (draft → accounted)
pub async fn account_journal_entry(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.sla_engine.account_entry(id, Some(user_id)).await {
        Ok(entry) => Ok(Json(serde_json::to_value(entry).unwrap_or_default())),
        Err(e) => { error!("Failed to account entry: {}", e); Err(StatusCode::BAD_REQUEST) }
    }
}

/// Post an accounted entry (accounted → posted)
pub async fn post_journal_entry(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.sla_engine.post_entry(id, Some(user_id)).await {
        Ok(entry) => Ok(Json(serde_json::to_value(entry).unwrap_or_default())),
        Err(e) => { error!("Failed to post entry: {}", e); Err(StatusCode::BAD_REQUEST) }
    }
}

#[derive(Debug, Deserialize)]
pub struct ReverseEntryRequest {
    pub reason: String,
}

/// Reverse a posted entry (posted → reversed)
pub async fn reverse_journal_entry(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<ReverseEntryRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.sla_engine.reverse_entry(id, &payload.reason, Some(user_id)).await {
        Ok(entry) => Ok(Json(serde_json::to_value(entry).unwrap_or_default())),
        Err(e) => { error!("Failed to reverse entry: {}", e); Err(StatusCode::BAD_REQUEST) }
    }
}

// ============================================================================
// Auto-Accounting (Generate Lines from Derivation Rules)
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct GenerateLinesRequest {
    pub transaction_attributes: serde_json::Value,
}

/// Auto-generate journal lines using derivation rules
pub async fn generate_journal_lines(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(entry_id): Path<Uuid>,
    Json(payload): Json<GenerateLinesRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.sla_engine.generate_journal_lines(
        org_id, entry_id, &payload.transaction_attributes,
    ).await {
        Ok(lines) => Ok(Json(serde_json::json!({"data": lines}))),
        Err(e) => { error!("Failed to generate lines: {}", e); Err(StatusCode::BAD_REQUEST) }
    }
}

// ============================================================================
// Transfer to GL
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct TransferToGlRequest {
    pub from_period: Option<String>,
    pub source_applications: Option<Vec<String>>,
}

/// Transfer posted entries to the General Ledger
pub async fn transfer_to_gl(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<TransferToGlRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.sla_engine.transfer_to_gl(
        org_id, payload.from_period.as_deref(),
        payload.source_applications, Some(user_id),
    ).await {
        Ok(log) => Ok((StatusCode::CREATED, Json(serde_json::to_value(log).unwrap_or_default()))),
        Err(e) => { error!("Failed to transfer to GL: {}", e); Err(StatusCode::BAD_REQUEST) }
    }
}

/// Get a transfer log
pub async fn get_transfer_log(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.sla_engine.get_transfer_log(id).await {
        Ok(Some(log)) => Ok(Json(serde_json::to_value(log).unwrap_or_default())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListTransferLogsParams {
    pub status: Option<String>,
}

/// List transfer logs
pub async fn list_transfer_logs(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<ListTransferLogsParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.sla_engine.list_transfer_logs(org_id, params.status.as_deref()).await {
        Ok(logs) => Ok(Json(serde_json::json!({"data": logs}))),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// ============================================================================
// SLA Events
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ListSlaEventsParams {
    pub source_application: Option<String>,
    pub event_type: Option<String>,
}

/// List SLA events
pub async fn list_sla_events(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<ListSlaEventsParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.sla_engine.list_sla_events(
        org_id, params.source_application.as_deref(), params.event_type.as_deref(),
    ).await {
        Ok(events) => Ok(Json(serde_json::json!({"data": events}))),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// ============================================================================
// Dashboard
// ============================================================================

/// Get SLA dashboard summary
pub async fn get_sla_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.sla_engine.get_dashboard_summary(org_id).await {
        Ok(summary) => Ok(Json(serde_json::to_value(summary).unwrap_or_default())),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

/// Resolve account code via derivation rules (utility endpoint)
#[derive(Debug, Deserialize)]
pub struct ResolveAccountCodeRequest {
    pub accounting_method_id: Uuid,
    pub line_type: String,
    pub transaction_attributes: serde_json::Value,
}

pub async fn resolve_account_code(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<ResolveAccountCodeRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let rules = state.sla_engine.list_active_derivation_rules(
        org_id, payload.accounting_method_id, &payload.line_type,
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let account_code = state.sla_engine.resolve_account_code(
        &rules, &payload.line_type, &payload.transaction_attributes,
    );

    Ok(Json(serde_json::json!({
        "account_code": account_code,
        "line_type": payload.line_type,
        "rules_evaluated": rules.len(),
    })))
}