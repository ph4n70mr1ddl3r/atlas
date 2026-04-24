//! Manual Journal Entries API Handlers
//!
//! Oracle Fusion Cloud ERP: General Ledger > Journals > New Journal
//!
//! Endpoints for managing journal batches, journal entries, and journal lines
//! with full lifecycle support: Draft → Submitted → Approved → Posted → Reversed.

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
pub struct ListQuery {
    pub status: Option<String>,
}

// ============================================================================
// Batches
// ============================================================================

/// Create a journal batch
pub async fn create_batch(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    let batch_number = body["batch_number"].as_str().unwrap_or("").to_string();
    let name = body["name"].as_str().unwrap_or("").to_string();
    let description = body["description"].as_str();
    let ledger_id = body["ledger_id"].as_str().and_then(|s| Uuid::parse_str(s).ok());
    let currency_code = body["currency_code"].as_str().unwrap_or("USD").to_string();
    let accounting_date = body["accounting_date"].as_str()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let period_name = body["period_name"].as_str();
    let source = body["source"].as_str();
    let is_automatic_post = body["is_automatic_post"].as_bool().unwrap_or(false);

    match state.manual_journal_engine.create_batch(
        org_id, &batch_number, &name, description, ledger_id,
        &currency_code, accounting_date, period_name, source,
        is_automatic_post, parse_uuid(&claims.sub).ok(),
    ).await {
        Ok(batch) => Ok((StatusCode::CREATED, Json(serde_json::to_value(batch).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Get a batch by number
pub async fn get_batch(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(batch_number): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    match state.manual_journal_engine.get_batch(org_id, &batch_number).await {
        Ok(Some(batch)) => Ok(Json(serde_json::to_value(batch).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(json!({"error": "Batch not found"})))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// List batches
pub async fn list_batches(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<ListQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    match state.manual_journal_engine.list_batches(org_id, params.status.as_deref()).await {
        Ok(batches) => Ok(Json(json!({"data": batches}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Delete a batch
pub async fn delete_batch(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(batch_number): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    match state.manual_journal_engine.delete_batch(org_id, &batch_number).await {
        Ok(()) => Ok(Json(json!({"message": "Batch deleted"}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Submit a batch for approval
pub async fn submit_batch(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let submitted_by: Uuid = parse_uuid(&claims.sub)?;
    match state.manual_journal_engine.submit_batch(id, Some(submitted_by)).await {
        Ok(batch) => Ok(Json(serde_json::to_value(batch).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Approve a batch
pub async fn approve_batch(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let approved_by: Uuid = parse_uuid(&claims.sub)?;
    match state.manual_journal_engine.approve_batch(id, Some(approved_by)).await {
        Ok(batch) => Ok(Json(serde_json::to_value(batch).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Reject a batch
pub async fn reject_batch(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let reason = body["reason"].as_str();
    match state.manual_journal_engine.reject_batch(id, reason).await {
        Ok(batch) => Ok(Json(serde_json::to_value(batch).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Post a batch to the General Ledger
pub async fn post_batch(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let posted_by: Uuid = parse_uuid(&claims.sub)?;
    match state.manual_journal_engine.post_batch(id, Some(posted_by)).await {
        Ok(batch) => Ok(Json(serde_json::to_value(batch).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Reverse a posted batch
pub async fn reverse_batch(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let reversed_by: Uuid = parse_uuid(&claims.sub)?;
    match state.manual_journal_engine.reverse_batch(id, Some(reversed_by)).await {
        Ok(batch) => Ok(Json(serde_json::to_value(batch).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Entries
// ============================================================================

/// Create a journal entry
pub async fn create_entry(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(batch_id): Path<Uuid>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    let entry_number = body["entry_number"].as_str().unwrap_or("").to_string();
    let name = body["name"].as_str();
    let description = body["description"].as_str();
    let ledger_id = body["ledger_id"].as_str().and_then(|s| Uuid::parse_str(s).ok());
    let currency_code = body["currency_code"].as_str().unwrap_or("USD").to_string();
    let accounting_date = body["accounting_date"].as_str()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let period_name = body["period_name"].as_str();
    let journal_category = body["journal_category"].as_str();
    let reference_number = body["reference_number"].as_str();
    let external_reference = body["external_reference"].as_str();
    let statistical_entry = body["statistical_entry"].as_bool().unwrap_or(false);

    match state.manual_journal_engine.create_entry(
        org_id, batch_id, &entry_number, name, description,
        ledger_id, &currency_code, accounting_date, period_name,
        journal_category, reference_number, external_reference,
        statistical_entry, parse_uuid(&claims.sub).ok(),
    ).await {
        Ok(entry) => Ok((StatusCode::CREATED, Json(serde_json::to_value(entry).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Get an entry by ID
pub async fn get_entry(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.manual_journal_engine.get_entry(id).await {
        Ok(Some(entry)) => Ok(Json(serde_json::to_value(entry).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(json!({"error": "Entry not found"})))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// List entries in a batch
pub async fn list_entries_by_batch(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(batch_id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.manual_journal_engine.list_entries_by_batch(batch_id).await {
        Ok(entries) => Ok(Json(json!({"data": entries}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// List all entries
pub async fn list_entries(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<ListQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    match state.manual_journal_engine.list_entries(org_id, params.status.as_deref()).await {
        Ok(entries) => Ok(Json(json!({"data": entries}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Delete an entry
pub async fn delete_entry(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.manual_journal_engine.delete_entry(id).await {
        Ok(()) => Ok(Json(json!({"message": "Entry deleted"}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Lines
// ============================================================================

/// Add a line to a journal entry
pub async fn add_line(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(entry_id): Path<Uuid>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    let line_type = body["line_type"].as_str().unwrap_or("debit").to_string();
    let account_code = body["account_code"].as_str().unwrap_or("").to_string();
    let account_name = body["account_name"].as_str();
    let description = body["description"].as_str();
    let amount = body["amount"].as_str().unwrap_or("0").to_string();
    let entered_amount = body["entered_amount"].as_str();
    let entered_currency_code = body["entered_currency_code"].as_str();
    let exchange_rate = body["exchange_rate"].as_str();
    let tax_code = body["tax_code"].as_str();
    let cost_center = body["cost_center"].as_str();
    let department_id = body["department_id"].as_str().and_then(|s| Uuid::parse_str(s).ok());
    let project_id = body["project_id"].as_str().and_then(|s| Uuid::parse_str(s).ok());
    let intercompany_entity_id = body["intercompany_entity_id"].as_str().and_then(|s| Uuid::parse_str(s).ok());
    let statistical_amount = body["statistical_amount"].as_str();

    match state.manual_journal_engine.add_line(
        org_id, entry_id, &line_type, &account_code, account_name,
        description, &amount, entered_amount, entered_currency_code,
        exchange_rate, tax_code, cost_center, department_id,
        project_id, intercompany_entity_id, statistical_amount,
    ).await {
        Ok(line) => Ok((StatusCode::CREATED, Json(serde_json::to_value(line).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// List lines for an entry
pub async fn list_lines(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(entry_id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.manual_journal_engine.list_lines(entry_id).await {
        Ok(lines) => Ok(Json(json!({"data": lines}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Dashboard
// ============================================================================

/// Get manual journal dashboard
pub async fn get_dashboard(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    match state.manual_journal_engine.get_dashboard_summary(org_id).await {
        Ok(summary) => Ok(Json(serde_json::to_value(summary).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}
