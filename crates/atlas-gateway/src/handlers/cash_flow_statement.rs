//! Cash Flow Statement Handlers
//!
//! Oracle Fusion: Financials > General Ledger > Financial Reports > Cash Flow Statements
//! Generates cash flow statements using direct or indirect methods.

use crate::handlers::auth::Claims;
use crate::handlers::{created_json, to_json};
use crate::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension, Json,
};
use serde::Deserialize;
use std::sync::Arc;
use tracing::error;
use uuid::Uuid;

// ============================================================================
// Create Cash Flow Statement
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateCashFlowStatementRequest {
    pub statement_number: String,
    pub method: String,
    pub period_type: String,
    pub period_start: chrono::NaiveDate,
    pub period_end: chrono::NaiveDate,
}

pub async fn create_cash_flow_statement(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateCashFlowStatementRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state
        .financials
        .cash_flow_statement_engine
        .create_statement(
            org_id,
            &payload.statement_number,
            &payload.method,
            &payload.period_type,
            payload.period_start,
            payload.period_end,
            Some(user_id),
        )
        .await
    {
        Ok(stmt) => Ok(created_json(stmt)),
        Err(e) => {
            error!("Failed to create cash flow statement: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// ============================================================================
// List Cash Flow Statements
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ListCashFlowStatementsQuery {
    pub status: Option<String>,
    pub method: Option<String>,
}

pub async fn list_cash_flow_statements(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListCashFlowStatementsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state
        .financials
        .cash_flow_statement_engine
        .list_statements(org_id, query.status.as_deref(), query.method.as_deref())
        .await
    {
        Ok(stmts) => Ok(Json(serde_json::json!({ "data": stmts }))),
        Err(e) => {
            error!("Failed to list cash flow statements: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// ============================================================================
// Get Cash Flow Statement
// ============================================================================

pub async fn get_cash_flow_statement(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state
        .financials
        .cash_flow_statement_engine
        .get_statement(id)
        .await
    {
        Ok(Some(stmt)) => Ok(to_json(stmt)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get cash flow statement: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// ============================================================================
// Calculate Cash Flow Statement
// ============================================================================

pub async fn calculate_cash_flow_statement(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state
        .financials
        .cash_flow_statement_engine
        .calculate(id)
        .await
    {
        Ok(stmt) => Ok(to_json(stmt)),
        Err(e) => {
            error!("Failed to calculate cash flow statement: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_)
                | atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// ============================================================================
// Review Cash Flow Statement
// ============================================================================

pub async fn review_cash_flow_statement(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state
        .financials
        .cash_flow_statement_engine
        .review(id, user_id)
        .await
    {
        Ok(stmt) => Ok(to_json(stmt)),
        Err(e) => {
            error!("Failed to review cash flow statement: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_)
                | atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// ============================================================================
// Publish Cash Flow Statement
// ============================================================================

pub async fn publish_cash_flow_statement(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state
        .financials
        .cash_flow_statement_engine
        .publish(id)
        .await
    {
        Ok(stmt) => Ok(to_json(stmt)),
        Err(e) => {
            error!("Failed to publish cash flow statement: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_)
                | atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// ============================================================================
// Archive Cash Flow Statement
// ============================================================================

pub async fn archive_cash_flow_statement(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state
        .financials
        .cash_flow_statement_engine
        .archive(id)
        .await
    {
        Ok(stmt) => Ok(to_json(stmt)),
        Err(e) => {
            error!("Failed to archive cash flow statement: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_)
                | atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// ============================================================================
// Add Cash Flow Statement Line
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct AddCashFlowLineRequest {
    pub line_number: i32,
    pub category: String,
    pub description: Option<String>,
    pub line_type: String,
    pub amount: String,
    pub account_range_from: Option<String>,
    pub account_range_to: Option<String>,
    pub is_non_cash: Option<bool>,
    pub display_order: Option<i32>,
}

pub async fn add_cash_flow_line(
    State(state): State<Arc<AppState>>,
    Path(statement_id): Path<Uuid>,
    Json(payload): Json<AddCashFlowLineRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    match state
        .financials
        .cash_flow_statement_engine
        .add_line(
            statement_id,
            payload.line_number,
            &payload.category,
            payload.description.as_deref(),
            &payload.line_type,
            &payload.amount,
            payload.account_range_from.as_deref(),
            payload.account_range_to.as_deref(),
            payload.is_non_cash.unwrap_or(false),
            payload.display_order.unwrap_or(payload.line_number),
        )
        .await
    {
        Ok(line) => Ok(created_json(line)),
        Err(e) => {
            error!("Failed to add cash flow line: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_)
                | atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// ============================================================================
// List Cash Flow Statement Lines
// ============================================================================

pub async fn list_cash_flow_lines(
    State(state): State<Arc<AppState>>,
    Path(statement_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state
        .financials
        .cash_flow_statement_engine
        .list_lines(statement_id)
        .await
    {
        Ok(lines) => Ok(Json(serde_json::json!({ "data": lines }))),
        Err(e) => {
            error!("Failed to list cash flow lines: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// ============================================================================
// Remove Cash Flow Statement Line
// ============================================================================

pub async fn remove_cash_flow_line(
    State(state): State<Arc<AppState>>,
    Path(line_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    match state
        .financials
        .cash_flow_statement_engine
        .remove_line(line_id)
        .await
    {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            error!("Failed to remove cash flow line: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// ============================================================================
// Dashboard
// ============================================================================

pub async fn get_cash_flow_statement_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state
        .financials
        .cash_flow_statement_engine
        .get_dashboard(org_id)
        .await
    {
        Ok(dashboard) => Ok(to_json(dashboard)),
        Err(e) => {
            error!("Failed to get cash flow dashboard: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
