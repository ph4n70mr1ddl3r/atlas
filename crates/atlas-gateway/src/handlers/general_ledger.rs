//! General Ledger Handlers
//!
//! Oracle Fusion Cloud ERP: General Ledger > Chart of Accounts, Journal Entries, Trial Balance
//!
//! API endpoints for managing GL accounts, journal entries, journal lines,
//! and generating trial balance reports.

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
// Chart of Accounts Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateGlAccountRequest {
    pub account_code: String,
    pub account_name: String,
    pub description: Option<String>,
    pub account_type: String,
    pub subtype: Option<String>,
    pub parent_account_id: Option<Uuid>,
    pub natural_balance: String,
}

/// Create a new GL account
pub async fn create_gl_account(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateGlAccountRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    info!("Creating GL account '{}' for org {}", payload.account_code, org_id);

    match state.general_ledger_engine.create_account(
        org_id,
        &payload.account_code,
        &payload.account_name,
        payload.description.as_deref(),
        &payload.account_type,
        payload.subtype.as_deref(),
        payload.parent_account_id,
        &payload.natural_balance,
        Some(user_id),
    ).await {
        Ok(account) => Ok((StatusCode::CREATED, Json(serde_json::to_value(account).unwrap()))),
        Err(e) => {
            error!("Failed to create GL account: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListAccountsQuery {
    pub account_type: Option<String>,
}

/// List GL accounts
pub async fn list_gl_accounts(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListAccountsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.general_ledger_engine.list_accounts(
        org_id,
        query.account_type.as_deref(),
    ).await {
        Ok(accounts) => Ok(Json(serde_json::json!({ "data": accounts }))),
        Err(e) => {
            error!("Failed to list GL accounts: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get a GL account by ID
pub async fn get_gl_account(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.general_ledger_engine.get_account(id).await {
        Ok(Some(account)) => Ok(Json(serde_json::to_value(account).unwrap())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get GL account: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// ============================================================================
// Journal Entry Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateJournalEntryRequest {
    pub entry_date: chrono::NaiveDate,
    pub gl_date: chrono::NaiveDate,
    #[serde(default = "default_entry_type")]
    pub entry_type: String,
    pub description: Option<String>,
    #[serde(default = "default_currency_usd")]
    pub currency_code: String,
    pub source_type: Option<String>,
    pub source_id: Option<Uuid>,
}

fn default_entry_type() -> String { "standard".to_string() }
fn default_currency_usd() -> String { "USD".to_string() }

/// Create a new journal entry
pub async fn create_journal_entry(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateJournalEntryRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    info!("Creating journal entry for org {}", org_id);

    match state.general_ledger_engine.create_journal_entry(
        org_id,
        payload.entry_date,
        payload.gl_date,
        &payload.entry_type,
        payload.description.as_deref(),
        &payload.currency_code,
        payload.source_type.as_deref(),
        payload.source_id,
        Some(user_id),
    ).await {
        Ok(entry) => Ok((StatusCode::CREATED, Json(serde_json::to_value(entry).unwrap()))),
        Err(e) => {
            error!("Failed to create journal entry: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListJournalEntriesQuery {
    pub status: Option<String>,
    pub entry_type: Option<String>,
}

/// List journal entries
pub async fn list_journal_entries(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListJournalEntriesQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.general_ledger_engine.list_journal_entries(
        org_id,
        query.status.as_deref(),
        query.entry_type.as_deref(),
    ).await {
        Ok(entries) => Ok(Json(serde_json::json!({ "data": entries }))),
        Err(e) => {
            error!("Failed to list journal entries: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get a journal entry by ID
pub async fn get_journal_entry(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.general_ledger_engine.get_journal_entry(id).await {
        Ok(Some(entry)) => Ok(Json(serde_json::to_value(entry).unwrap())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get journal entry: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// ============================================================================
// Journal Line Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct AddJournalLineRequest {
    pub line_type: String,
    pub account_code: String,
    pub description: Option<String>,
    pub entered_dr: String,
    pub entered_cr: String,
}

/// Add a line to a journal entry
pub async fn add_journal_line(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(entry_id): Path<Uuid>,
    Json(payload): Json<AddJournalLineRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    info!("Adding journal line to entry {}", entry_id);

    match state.general_ledger_engine.add_journal_line(
        org_id,
        entry_id,
        &payload.line_type,
        &payload.account_code,
        payload.description.as_deref(),
        &payload.entered_dr,
        &payload.entered_cr,
        Some(user_id),
    ).await {
        Ok(line) => Ok((StatusCode::CREATED, Json(serde_json::to_value(line).unwrap()))),
        Err(e) => {
            error!("Failed to add journal line: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::CONFLICT,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::BAD_REQUEST,
            })
        }
    }
}

/// List journal lines for an entry
pub async fn list_journal_lines(
    State(state): State<Arc<AppState>>,
    Path(entry_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.general_ledger_engine.list_journal_lines(entry_id).await {
        Ok(lines) => Ok(Json(serde_json::json!({ "data": lines }))),
        Err(e) => {
            error!("Failed to list journal lines: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// ============================================================================
// Journal Entry Workflow Handlers
// ============================================================================

/// Post a journal entry
pub async fn post_journal_entry(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.general_ledger_engine.post_journal_entry(id, Some(user_id)).await {
        Ok(entry) => Ok(Json(serde_json::to_value(entry).unwrap())),
        Err(e) => {
            error!("Failed to post journal entry: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::CONFLICT,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::BAD_REQUEST,
            })
        }
    }
}

/// Reverse a posted journal entry
pub async fn reverse_journal_entry(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.general_ledger_engine.reverse_journal_entry(id).await {
        Ok(entry) => Ok(Json(serde_json::to_value(entry).unwrap())),
        Err(e) => {
            error!("Failed to reverse journal entry: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::CONFLICT,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::BAD_REQUEST,
            })
        }
    }
}

// ============================================================================
// Trial Balance Handler
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct TrialBalanceQuery {
    pub as_of_date: chrono::NaiveDate,
}

/// Generate trial balance
pub async fn generate_trial_balance(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<TrialBalanceQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.general_ledger_engine.generate_trial_balance(org_id, query.as_of_date).await {
        Ok(tb) => Ok(Json(serde_json::to_value(tb).unwrap())),
        Err(e) => {
            error!("Failed to generate trial balance: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
