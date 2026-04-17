//! Bank Reconciliation Handlers
//!
//! Oracle Fusion Cloud ERP: Cash Management > Bank Statements and Reconciliation
//! API endpoints for bank accounts, statements, auto-matching,
//! manual matching, and reconciliation summaries.

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
// Bank Account Management
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateBankAccountRequest {
    pub account_number: String,
    pub account_name: String,
    pub bank_name: String,
    pub bank_code: Option<String>,
    pub branch_name: Option<String>,
    pub branch_code: Option<String>,
    pub gl_account_code: Option<String>,
    #[serde(default = "default_currency_usd")]
    pub currency_code: String,
    #[serde(default = "default_checking")]
    pub account_type: String,
}

fn default_currency_usd() -> String { "USD".to_string() }
fn default_checking() -> String { "checking".to_string() }

/// Create a bank account
pub async fn create_bank_account(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateBankAccountRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let account = state.reconciliation_engine
        .create_bank_account(
            org_id,
            &payload.account_number,
            &payload.account_name,
            &payload.bank_name,
            payload.bank_code.as_deref(),
            payload.branch_name.as_deref(),
            payload.branch_code.as_deref(),
            payload.gl_account_code.as_deref(),
            &payload.currency_code,
            &payload.account_type,
            Some(user_id),
        )
        .await
        .map_err(|e| {
            error!("Create bank account error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(account).unwrap_or_default())))
}

/// List bank accounts
pub async fn list_bank_accounts(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let accounts = state.reconciliation_engine
        .list_bank_accounts(org_id)
        .await
        .map_err(|e| { error!("List bank accounts error: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;

    Ok(Json(serde_json::json!({ "data": accounts })))
}

/// Get a bank account
pub async fn get_bank_account(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let _org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let account = state.reconciliation_engine
        .get_bank_account(id)
        .await
        .map_err(|e| { error!("Get bank account error: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(serde_json::to_value(account).unwrap_or_default()))
}

/// Delete a bank account
pub async fn delete_bank_account(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    claims: Extension<Claims>,
) -> Result<StatusCode, StatusCode> {
    let _org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    state.reconciliation_engine
        .delete_bank_account(id)
        .await
        .map_err(|e| { error!("Delete bank account error: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Bank Statement Management
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateBankStatementRequest {
    pub bank_account_id: Uuid,
    pub statement_number: String,
    pub statement_date: String,
    pub start_date: String,
    pub end_date: String,
    pub opening_balance: String,
    pub closing_balance: String,
    pub lines: Option<Vec<StatementLineInput>>,
}

#[derive(Debug, Deserialize)]
pub struct StatementLineInput {
    pub line_number: i32,
    pub transaction_date: String,
    pub transaction_type: String,
    pub amount: String,
    pub description: Option<String>,
    pub reference_number: Option<String>,
    pub check_number: Option<String>,
    pub counterparty_name: Option<String>,
    pub counterparty_account: Option<String>,
}

/// Create a bank statement (with optional lines)
pub async fn create_bank_statement(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateBankStatementRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let statement_date = chrono::NaiveDate::parse_from_str(&payload.statement_date, "%Y-%m-%d")
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let start_date = chrono::NaiveDate::parse_from_str(&payload.start_date, "%Y-%m-%d")
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let end_date = chrono::NaiveDate::parse_from_str(&payload.end_date, "%Y-%m-%d")
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let statement = state.reconciliation_engine
        .create_bank_statement(
            org_id,
            payload.bank_account_id,
            &payload.statement_number,
            statement_date,
            start_date,
            end_date,
            &payload.opening_balance,
            &payload.closing_balance,
            Some(user_id),
        )
        .await
        .map_err(|e| {
            error!("Create bank statement error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::Forbidden(_) => StatusCode::FORBIDDEN,
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    // Add statement lines if provided
    if let Some(lines) = payload.lines {
        for line in lines {
            let tx_date = chrono::NaiveDate::parse_from_str(&line.transaction_date, "%Y-%m-%d")
                .map_err(|_| StatusCode::BAD_REQUEST)?;

            state.reconciliation_engine
                .add_statement_line(
                    org_id,
                    statement.id,
                    line.line_number,
                    tx_date,
                    &line.transaction_type,
                    &line.amount,
                    line.description.as_deref(),
                    line.reference_number.as_deref(),
                    line.check_number.as_deref(),
                    line.counterparty_name.as_deref(),
                    line.counterparty_account.as_deref(),
                )
                .await
                .map_err(|e| {
                    error!("Add statement line error: {}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;
        }
    }

    Ok((StatusCode::CREATED, Json(serde_json::to_value(statement).unwrap_or_default())))
}

/// List bank statements for an account
pub async fn list_bank_statements(
    State(state): State<Arc<AppState>>,
    Path(bank_account_id): Path<Uuid>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let statements = state.reconciliation_engine
        .list_bank_statements(org_id, bank_account_id)
        .await
        .map_err(|e| { error!("List bank statements error: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;

    Ok(Json(serde_json::json!({ "data": statements })))
}

/// Get a bank statement with its lines
pub async fn get_bank_statement(
    State(state): State<Arc<AppState>>,
    Path(statement_id): Path<Uuid>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let _org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let statement = state.reconciliation_engine
        .get_bank_statement(statement_id)
        .await
        .map_err(|e| { error!("Get bank statement error: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(serde_json::to_value(statement).unwrap_or_default()))
}

/// List statement lines
pub async fn list_statement_lines(
    State(state): State<Arc<AppState>>,
    Path(statement_id): Path<Uuid>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let _org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let lines = state.reconciliation_engine
        .list_statement_lines(statement_id)
        .await
        .map_err(|e| { error!("List statement lines error: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;

    Ok(Json(serde_json::json!({ "data": lines })))
}

// ============================================================================
// System Transactions
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateSystemTransactionRequest {
    pub bank_account_id: Uuid,
    pub source_type: String,
    pub source_id: Uuid,
    pub source_number: Option<String>,
    pub transaction_date: String,
    pub amount: String,
    pub transaction_type: String,
    pub description: Option<String>,
    pub reference_number: Option<String>,
    pub check_number: Option<String>,
    pub counterparty_name: Option<String>,
}

/// Create a system transaction
pub async fn create_system_transaction(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateSystemTransactionRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let tx_date = chrono::NaiveDate::parse_from_str(&payload.transaction_date, "%Y-%m-%d")
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let txn = state.reconciliation_engine
        .create_system_transaction(
            org_id,
            payload.bank_account_id,
            &payload.source_type,
            payload.source_id,
            payload.source_number.as_deref(),
            tx_date,
            &payload.amount,
            &payload.transaction_type,
            payload.description.as_deref(),
            payload.reference_number.as_deref(),
            payload.check_number.as_deref(),
            payload.counterparty_name.as_deref(),
            Some(user_id),
        )
        .await
        .map_err(|e| {
            error!("Create system transaction error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(txn).unwrap_or_default())))
}

/// List unreconciled system transactions for a bank account
pub async fn list_unreconciled_transactions(
    State(state): State<Arc<AppState>>,
    Path(bank_account_id): Path<Uuid>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let transactions = state.reconciliation_engine
        .list_unreconciled_transactions(org_id, bank_account_id)
        .await
        .map_err(|e| { error!("List unreconciled transactions error: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;

    Ok(Json(serde_json::json!({ "data": transactions })))
}

// ============================================================================
// Auto-Matching
// ============================================================================

/// Run auto-matching for a bank statement
pub async fn auto_match_statement(
    State(state): State<Arc<AppState>>,
    Path(statement_id): Path<Uuid>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let result = state.reconciliation_engine
        .auto_match(org_id, statement_id, Some(user_id))
        .await
        .map_err(|e| {
            error!("Auto-match error: {}", e);
            match e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::Forbidden(_) => StatusCode::FORBIDDEN,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(Json(serde_json::to_value(result).unwrap_or_default()))
}

// ============================================================================
// Manual Matching
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ManualMatchRequest {
    pub statement_line_id: Uuid,
    pub system_transaction_id: Uuid,
}

/// Manually match a statement line to a system transaction
pub async fn manual_match(
    State(state): State<Arc<AppState>>,
    Path(statement_id): Path<Uuid>,
    claims: Extension<Claims>,
    Json(payload): Json<ManualMatchRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let match_record = state.reconciliation_engine
        .manual_match(
            org_id,
            statement_id,
            payload.statement_line_id,
            payload.system_transaction_id,
            Some(user_id),
        )
        .await
        .map_err(|e| {
            error!("Manual match error: {}", e);
            match e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(Json(serde_json::to_value(match_record).unwrap_or_default()))
}

/// Unmatch a previously matched pair
pub async fn unmatch(
    State(state): State<Arc<AppState>>,
    Path(match_id): Path<Uuid>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let match_record = state.reconciliation_engine
        .unmatch(match_id, Some(user_id))
        .await
        .map_err(|e| {
            error!("Unmatch error: {}", e);
            match e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(Json(serde_json::to_value(match_record).unwrap_or_default()))
}

/// List matches for a statement
pub async fn list_matches(
    State(state): State<Arc<AppState>>,
    Path(statement_id): Path<Uuid>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let _org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let matches = state.reconciliation_engine
        .list_matches(statement_id)
        .await
        .map_err(|e| { error!("List matches error: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;

    Ok(Json(serde_json::json!({ "data": matches })))
}

// ============================================================================
// Reconciliation Summary
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct GetSummaryParams {
    pub bank_account_id: Uuid,
    pub period_start: String,
    pub period_end: String,
}

/// Get reconciliation summary for an account + period
pub async fn get_reconciliation_summary(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<GetSummaryParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let period_start = chrono::NaiveDate::parse_from_str(&params.period_start, "%Y-%m-%d")
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let period_end = chrono::NaiveDate::parse_from_str(&params.period_end, "%Y-%m-%d")
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let summary = state.reconciliation_engine
        .get_reconciliation_summary(org_id, params.bank_account_id, period_start, period_end)
        .await
        .map_err(|e| {
            error!("Get reconciliation summary error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(serde_json::to_value(summary).unwrap_or_default()))
}

/// List reconciliation summaries
pub async fn list_reconciliation_summaries(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let summaries = state.reconciliation_engine
        .list_reconciliation_summaries(org_id)
        .await
        .map_err(|e| { error!("List reconciliation summaries error: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;

    Ok(Json(serde_json::json!({ "data": summaries })))
}

// ============================================================================
// Matching Rules
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateMatchingRuleRequest {
    pub name: String,
    pub description: Option<String>,
    pub bank_account_id: Option<Uuid>,
    #[serde(default = "default_priority")]
    pub priority: i32,
    pub criteria: serde_json::Value,
    #[serde(default = "default_true")]
    pub stop_on_match: bool,
}

fn default_priority() -> i32 { 100 }
fn default_true() -> bool { true }

/// Create a matching rule
pub async fn create_matching_rule(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateMatchingRuleRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let rule = state.reconciliation_engine
        .create_matching_rule(
            org_id,
            &payload.name,
            payload.description.as_deref(),
            payload.bank_account_id,
            payload.priority,
            payload.criteria,
            payload.stop_on_match,
            Some(user_id),
        )
        .await
        .map_err(|e| {
            error!("Create matching rule error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(rule).unwrap_or_default())))
}

/// List matching rules
pub async fn list_matching_rules(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let rules = state.reconciliation_engine
        .list_matching_rules(org_id)
        .await
        .map_err(|e| { error!("List matching rules error: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;

    Ok(Json(serde_json::json!({ "data": rules })))
}

/// Delete a matching rule
pub async fn delete_matching_rule(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    claims: Extension<Claims>,
) -> Result<StatusCode, StatusCode> {
    let _org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    state.reconciliation_engine
        .delete_matching_rule(id)
        .await
        .map_err(|e| { error!("Delete matching rule error: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;

    Ok(StatusCode::NO_CONTENT)
}
