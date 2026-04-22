//! Multi-Book Accounting Handlers
//!
//! Oracle Fusion Cloud ERP: General Ledger > Multi-Book Accounting
//!
//! API endpoints for managing accounting books, account mappings,
//! book journal entries, journal propagation, and multi-book dashboard.

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
// Request Types
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateAccountingBookRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub book_type: String,
    pub chart_of_accounts_code: String,
    pub calendar_code: String,
    #[serde(default = "default_usd")]
    pub currency_code: String,
    #[serde(default)]
    pub auto_propagation_enabled: bool,
    #[serde(default = "default_journal")]
    pub mapping_level: String,
}

fn default_usd() -> String { "USD".to_string() }
fn default_journal() -> String { "journal".to_string() }

#[derive(Debug, Deserialize)]
pub struct CreateAccountMappingRequest {
    pub source_book_id: Uuid,
    pub target_book_id: Uuid,
    pub source_account_code: String,
    pub target_account_code: String,
    #[serde(default = "default_segment_mappings")]
    pub segment_mappings: serde_json::Value,
    #[serde(default = "default_priority")]
    pub priority: i32,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
}

fn default_segment_mappings() -> serde_json::Value { serde_json::json!({}) }
fn default_priority() -> i32 { 10 }

#[derive(Debug, Deserialize)]
pub struct CreateBookJournalEntryRequest {
    pub book_id: Uuid,
    pub header_description: Option<String>,
    pub external_reference: Option<String>,
    pub accounting_date: chrono::NaiveDate,
    pub period_name: Option<String>,
    #[serde(default = "default_usd")]
    pub currency_code: String,
    pub lines: Vec<JournalLineRequest>,
}

#[derive(Debug, Deserialize)]
pub struct JournalLineRequest {
    pub account_code: String,
    pub account_name: Option<String>,
    pub debit_amount: String,
    pub credit_amount: String,
    pub description: Option<String>,
    pub tax_code: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PropagateEntryRequest {
    pub target_book_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct UpdateBookStatusRequest {
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct BookFilterParams {
    pub source_book_id: Option<Uuid>,
    pub target_book_id: Option<Uuid>,
    pub status: Option<String>,
}

// Helper to extract org_id and user_id from claims
fn extract_ids(claims: &Claims) -> Result<(Uuid, Uuid), (StatusCode, Json<serde_json::Value>)> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Invalid org_id in token"}))))?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Invalid user_id in token"}))))?;
    Ok((org_id, user_id))
}

fn err_response(e: atlas_shared::AtlasError) -> (StatusCode, Json<serde_json::Value>) {
    (StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::BAD_REQUEST),
     Json(serde_json::json!({"error": e.to_string()})))
}

// ============================================================================
// Accounting Book Handlers
// ============================================================================

pub async fn create_accounting_book(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(req): Json<CreateAccountingBookRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let (org_id, user_id) = extract_ids(&claims)?;

    match state.multi_book_engine.create_book(
        org_id,
        &req.code,
        &req.name,
        req.description.as_deref(),
        &req.book_type,
        &req.chart_of_accounts_code,
        &req.calendar_code,
        &req.currency_code,
        req.auto_propagation_enabled,
        &req.mapping_level,
        Some(user_id),
    ).await {
        Ok(book) => {
            info!("Created accounting book '{}' for org {}", req.code, org_id);
            Ok((StatusCode::CREATED, Json(serde_json::to_value(book).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))))
        }
        Err(e) => {
            error!("Failed to create accounting book: {}", e);
            Err(err_response(e))
        }
    }
}

pub async fn get_accounting_book(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(code): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Invalid org_id"}))))?;

    match state.multi_book_engine.get_book(org_id, &code).await {
        Ok(Some(book)) => Ok(Json(serde_json::to_value(book).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Ok(None) => Err((StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": format!("Accounting book '{}' not found", code)})))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})))),
    }
}

pub async fn list_accounting_books(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Invalid org_id"}))))?;

    match state.multi_book_engine.list_books(org_id).await {
        Ok(books) => Ok(Json(serde_json::json!({"data": books}))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})))),
    }
}

pub async fn update_accounting_book_status(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(code): Path<String>,
    Json(req): Json<UpdateBookStatusRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Invalid org_id"}))))?;

    let book = state.multi_book_engine.get_book(org_id, &code).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()}))))?
        .ok_or_else(|| (StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": format!("Book '{}' not found", code)}))))?;

    match state.multi_book_engine.update_book_status(book.id, &req.status).await {
        Ok(updated) => Ok(Json(serde_json::to_value(updated).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err(err_response(e)),
    }
}

pub async fn delete_accounting_book(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(code): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Invalid org_id"}))))?;

    match state.multi_book_engine.delete_book(org_id, &code).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err(err_response(e)),
    }
}

// ============================================================================
// Account Mapping Handlers
// ============================================================================

pub async fn create_account_mapping(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(req): Json<CreateAccountMappingRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let (org_id, user_id) = extract_ids(&claims)?;

    match state.multi_book_engine.create_account_mapping(
        org_id,
        req.source_book_id,
        req.target_book_id,
        &req.source_account_code,
        &req.target_account_code,
        req.segment_mappings,
        req.priority,
        req.effective_from,
        req.effective_to,
        Some(user_id),
    ).await {
        Ok(mapping) => {
            info!("Created account mapping for org {}", org_id);
            Ok((StatusCode::CREATED, Json(serde_json::to_value(mapping).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))))
        }
        Err(e) => {
            error!("Failed to create account mapping: {}", e);
            Err(err_response(e))
        }
    }
}

pub async fn list_account_mappings(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<BookFilterParams>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Invalid org_id"}))))?;

    match state.multi_book_engine.list_account_mappings(
        org_id,
        params.source_book_id,
        params.target_book_id,
    ).await {
        Ok(mappings) => Ok(Json(serde_json::json!({"data": mappings}))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})))),
    }
}

pub async fn delete_account_mapping(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    match state.multi_book_engine.delete_account_mapping(id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err(err_response(e)),
    }
}

// ============================================================================
// Journal Entry Handlers
// ============================================================================

pub async fn create_book_journal_entry(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(req): Json<CreateBookJournalEntryRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let (org_id, user_id) = extract_ids(&claims)?;

    let lines: Vec<atlas_core::multi_book::engine::JournalLineData> = req.lines.iter()
        .map(|l| atlas_core::multi_book::engine::JournalLineData {
            account_code: l.account_code.clone(),
            account_name: l.account_name.clone(),
            debit_amount: l.debit_amount.clone(),
            credit_amount: l.credit_amount.clone(),
            description: l.description.clone(),
            tax_code: l.tax_code.clone(),
        })
        .collect();

    match state.multi_book_engine.create_journal_entry(
        org_id,
        req.book_id,
        req.header_description.as_deref(),
        req.external_reference.as_deref(),
        req.accounting_date,
        req.period_name.as_deref(),
        &req.currency_code,
        &lines,
        Some(user_id),
    ).await {
        Ok(entry) => {
            info!("Created journal entry {} for org {}", entry.entry_number, org_id);
            Ok((StatusCode::CREATED, Json(serde_json::to_value(entry).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))))
        }
        Err(e) => {
            error!("Failed to create journal entry: {}", e);
            Err(err_response(e))
        }
    }
}

pub async fn get_book_journal_entry(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.multi_book_engine.get_journal_entry(id).await {
        Ok(Some(entry)) => Ok(Json(serde_json::to_value(entry).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Ok(None) => Err((StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Journal entry not found"})))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})))),
    }
}

pub async fn list_book_journal_entries(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(book_id): Path<Uuid>,
    Query(params): Query<BookFilterParams>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Invalid org_id"}))))?;

    match state.multi_book_engine.list_journal_entries(
        org_id,
        book_id,
        params.status.as_deref(),
    ).await {
        Ok(entries) => Ok(Json(serde_json::json!({"data": entries}))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})))),
    }
}

pub async fn get_book_journal_lines(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(entry_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.multi_book_engine.get_journal_lines(entry_id).await {
        Ok(lines) => Ok(Json(serde_json::json!({"data": lines}))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})))),
    }
}

pub async fn post_book_journal_entry(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Invalid user_id"}))))?;

    match state.multi_book_engine.post_journal_entry(id, Some(user_id)).await {
        Ok(entry) => Ok(Json(serde_json::to_value(entry).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err(err_response(e)),
    }
}

pub async fn reverse_book_journal_entry(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Invalid user_id"}))))?;

    match state.multi_book_engine.reverse_journal_entry(id, Some(user_id)).await {
        Ok(entry) => Ok(Json(serde_json::to_value(entry).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err(err_response(e)),
    }
}

// ============================================================================
// Propagation Handlers
// ============================================================================

pub async fn propagate_entry(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(req): Json<PropagateEntryRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.multi_book_engine.propagate_entry(id, req.target_book_id).await {
        Ok(log) => Ok(Json(serde_json::to_value(log).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err(err_response(e)),
    }
}

pub async fn list_propagation_logs(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<BookFilterParams>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Invalid org_id"}))))?;

    match state.multi_book_engine.list_propagation_logs(
        org_id,
        params.source_book_id,
        params.target_book_id,
    ).await {
        Ok(logs) => Ok(Json(serde_json::json!({"data": logs}))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Dashboard
// ============================================================================

pub async fn get_multi_book_summary(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Invalid org_id"}))))?;

    match state.multi_book_engine.get_summary(org_id).await {
        Ok(summary) => Ok(Json(serde_json::to_value(summary).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})))),
    }
}
