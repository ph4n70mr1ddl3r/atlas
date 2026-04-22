//! Encumbrance Management Handlers
//!
//! Oracle Fusion Cloud ERP: General Ledger > Encumbrance Management
//!
//! API endpoints for managing encumbrance types, entries, lines,
//! liquidations, year-end carry-forward, and budgetary control summaries.

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
// Request Types
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateEncumbranceTypeRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(default = "default_category")]
    pub category: String,
    #[serde(default = "default_true")]
    pub allow_manual_entry: bool,
    pub default_encumbrance_account_code: Option<String>,
    #[serde(default = "default_true")]
    pub allow_carry_forward: bool,
    #[serde(default = "default_priority")]
    pub priority: i32,
}

fn default_category() -> String { "commitment".to_string() }
fn default_true() -> bool { true }
fn default_priority() -> i32 { 10 }
fn default_usd() -> String { "USD".to_string() }

#[derive(Debug, Deserialize)]
pub struct CreateEncumbranceEntryRequest {
    pub encumbrance_type_code: String,
    pub source_type: Option<String>,
    pub source_id: Option<Uuid>,
    pub source_number: Option<String>,
    pub description: Option<String>,
    pub encumbrance_date: chrono::NaiveDate,
    pub amount: String,
    #[serde(default = "default_usd")]
    pub currency_code: String,
    pub fiscal_year: Option<i32>,
    pub period_name: Option<String>,
    pub expiry_date: Option<chrono::NaiveDate>,
    pub budget_line_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct AddEncumbranceLineRequest {
    pub account_code: String,
    pub account_description: Option<String>,
    pub department_id: Option<Uuid>,
    pub department_name: Option<String>,
    pub project_id: Option<Uuid>,
    pub project_name: Option<String>,
    pub cost_center: Option<String>,
    pub amount: String,
    pub encumbrance_account_code: Option<String>,
    pub source_line_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct CreateLiquidationRequest {
    pub encumbrance_entry_id: Uuid,
    pub encumbrance_line_id: Option<Uuid>,
    #[serde(default = "default_liq_type")]
    pub liquidation_type: String,
    pub liquidation_amount: String,
    pub source_type: Option<String>,
    pub source_id: Option<Uuid>,
    pub source_number: Option<String>,
    pub description: Option<String>,
    pub liquidation_date: chrono::NaiveDate,
}

fn default_liq_type() -> String { "partial".to_string() }

#[derive(Debug, Deserialize)]
pub struct ReverseLiquidationRequest {
    pub reason: String,
}

#[derive(Debug, Deserialize)]
pub struct CancelEntryRequest {
    pub reason: String,
}

#[derive(Debug, Deserialize)]
pub struct ProcessCarryForwardRequest {
    pub from_fiscal_year: i32,
    pub to_fiscal_year: i32,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListEntriesQuery {
    pub status: Option<String>,
    pub encumbrance_type_code: Option<String>,
    pub source_type: Option<String>,
    pub fiscal_year: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct ListLiquidationsQuery {
    pub entry_id: Option<Uuid>,
    pub status: Option<String>,
}

// ============================================================================
// Encumbrance Type Handlers
// ============================================================================

/// Create or update an encumbrance type
pub async fn create_encumbrance_type(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateEncumbranceTypeRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.encumbrance_engine.create_encumbrance_type(
        org_id, &payload.code, &payload.name, payload.description.as_deref(),
        &payload.category, payload.allow_manual_entry,
        payload.default_encumbrance_account_code.as_deref(),
        payload.allow_carry_forward, payload.priority, Some(user_id),
    ).await {
        Ok(t) => Ok((StatusCode::CREATED, Json(serde_json::to_value(t).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => {
            error!("Failed to create encumbrance type: {}", e);
            Err(map_error(e))
        }
    }
}

/// Get an encumbrance type by code
pub async fn get_encumbrance_type(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(code): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.encumbrance_engine.get_encumbrance_type(org_id, &code).await {
        Ok(Some(t)) => Ok(Json(serde_json::to_value(t).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

/// List all encumbrance types
pub async fn list_encumbrance_types(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.encumbrance_engine.list_encumbrance_types(org_id).await {
        Ok(types) => Ok(Json(serde_json::to_value(types).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

/// Delete an encumbrance type
pub async fn delete_encumbrance_type(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(code): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.encumbrance_engine.delete_encumbrance_type(org_id, &code).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err(map_error(e))
    }
}

// ============================================================================
// Encumbrance Entry Handlers
// ============================================================================

/// Create a new encumbrance entry
pub async fn create_encumbrance_entry(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateEncumbranceEntryRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.encumbrance_engine.create_entry(
        org_id, &payload.encumbrance_type_code,
        payload.source_type.as_deref(), payload.source_id,
        payload.source_number.as_deref(), payload.description.as_deref(),
        payload.encumbrance_date, &payload.amount,
        &payload.currency_code, payload.fiscal_year,
        payload.period_name.as_deref(), payload.expiry_date,
        payload.budget_line_id, Some(user_id),
    ).await {
        Ok(entry) => Ok((StatusCode::CREATED, Json(serde_json::to_value(entry).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => {
            error!("Failed to create encumbrance entry: {}", e);
            Err(map_error(e))
        }
    }
}

/// Get an encumbrance entry by ID
pub async fn get_encumbrance_entry(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.encumbrance_engine.get_entry(id).await {
        Ok(Some(e)) => Ok(Json(serde_json::to_value(e).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

/// List encumbrance entries with optional filters
pub async fn list_encumbrance_entries(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListEntriesQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.encumbrance_engine.list_entries(
        org_id, query.status.as_deref(), query.encumbrance_type_code.as_deref(),
        query.source_type.as_deref(), query.fiscal_year,
    ).await {
        Ok(entries) => Ok(Json(serde_json::to_value(entries).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => { error!("Error: {}", e); Err(map_error(e)) }
    }
}

/// Activate (approve) a draft encumbrance entry
pub async fn activate_encumbrance_entry(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.encumbrance_engine.activate_entry(id, Some(user_id)).await {
        Ok(entry) => Ok(Json(serde_json::to_value(entry).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err(map_error(e))
    }
}

/// Cancel an encumbrance entry
pub async fn cancel_encumbrance_entry(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<CancelEntryRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.encumbrance_engine.cancel_entry(id, user_id, &payload.reason).await {
        Ok(entry) => Ok(Json(serde_json::to_value(entry).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err(map_error(e))
    }
}

// ============================================================================
// Encumbrance Line Handlers
// ============================================================================

/// Add a line to an encumbrance entry
pub async fn add_encumbrance_line(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(entry_id): Path<Uuid>,
    Json(payload): Json<AddEncumbranceLineRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.encumbrance_engine.add_line(
        org_id, entry_id, &payload.account_code, payload.account_description.as_deref(),
        payload.department_id, payload.department_name.as_deref(),
        payload.project_id, payload.project_name.as_deref(),
        payload.cost_center.as_deref(), &payload.amount,
        payload.encumbrance_account_code.as_deref(), payload.source_line_id,
        Some(user_id),
    ).await {
        Ok(line) => Ok((StatusCode::CREATED, Json(serde_json::to_value(line).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => { error!("Failed to add encumbrance line: {}", e); Err(map_error(e)) }
    }
}

/// List lines for an encumbrance entry
pub async fn list_encumbrance_lines(
    State(state): State<Arc<AppState>>,
    Path(entry_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.encumbrance_engine.list_lines(entry_id).await {
        Ok(lines) => Ok(Json(serde_json::to_value(lines).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

/// Delete an encumbrance line
pub async fn delete_encumbrance_line(
    State(state): State<Arc<AppState>>,
    Path(line_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    match state.encumbrance_engine.delete_line(line_id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err(map_error(e))
    }
}

// ============================================================================
// Liquidation Handlers
// ============================================================================

/// Liquidate (reduce) an encumbrance
pub async fn create_liquidation(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateLiquidationRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.encumbrance_engine.liquidate(
        org_id, payload.encumbrance_entry_id, payload.encumbrance_line_id,
        &payload.liquidation_type, &payload.liquidation_amount,
        payload.source_type.as_deref(), payload.source_id,
        payload.source_number.as_deref(), payload.description.as_deref(),
        payload.liquidation_date, Some(user_id),
    ).await {
        Ok(liq) => Ok((StatusCode::CREATED, Json(serde_json::to_value(liq).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => { error!("Failed to create liquidation: {}", e); Err(map_error(e)) }
    }
}

/// List liquidations
pub async fn list_liquidations(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListLiquidationsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.encumbrance_engine.list_liquidations(
        org_id, query.entry_id, query.status.as_deref(),
    ).await {
        Ok(liqs) => Ok(Json(serde_json::to_value(liqs).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => { error!("Error: {}", e); Err(map_error(e)) }
    }
}

/// Reverse a liquidation
pub async fn reverse_liquidation(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<ReverseLiquidationRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.encumbrance_engine.reverse_liquidation(id, &payload.reason).await {
        Ok(liq) => Ok(Json(serde_json::to_value(liq).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err(map_error(e))
    }
}

// ============================================================================
// Carry-Forward Handlers
// ============================================================================

/// Process year-end carry-forward
pub async fn process_carry_forward(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<ProcessCarryForwardRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.encumbrance_engine.process_carry_forward(
        org_id, payload.from_fiscal_year, payload.to_fiscal_year,
        payload.description.as_deref(), Some(user_id),
    ).await {
        Ok(cf) => Ok((StatusCode::CREATED, Json(serde_json::to_value(cf).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => { error!("Failed to process carry-forward: {}", e); Err(map_error(e)) }
    }
}

/// List carry-forward batches
pub async fn list_carry_forwards(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.encumbrance_engine.list_carry_forwards(org_id).await {
        Ok(cfs) => Ok(Json(serde_json::to_value(cfs).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

/// Get carry-forward dashboard summary
pub async fn get_encumbrance_summary(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.encumbrance_engine.get_summary(org_id).await {
        Ok(summary) => Ok(Json(serde_json::to_value(summary).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// ============================================================================
// Error Mapping
// ============================================================================

fn map_error(e: atlas_shared::AtlasError) -> StatusCode {
    match e {
        atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
        atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
        atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
        atlas_shared::AtlasError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
        atlas_shared::AtlasError::Forbidden(_) => StatusCode::FORBIDDEN,
        atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
