//! Financial Statements Handlers
//!
//! Oracle Fusion Cloud ERP: Financials > Financial Statements
//!
//! API endpoints for generating financial statements:
//! - Balance Sheet
//! - Income Statement
//! - Cash Flow Statement

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

#[derive(Debug, Deserialize)]
pub struct GenerateStatementRequest {
    pub report_type: Option<String>,
    pub as_of_date: chrono::NaiveDate,
    pub period_name: Option<String>,
    pub fiscal_year: Option<i32>,
    pub currency_code: Option<String>,
    pub include_comparative: Option<bool>,
}

/// Generate a financial statement
pub async fn generate_financial_statement(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<GenerateStatementRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let report_type = payload.report_type.as_deref().unwrap_or("balance_sheet");
    info!("Generating {} for org {} as of {}", report_type, org_id, payload.as_of_date);

    let request = atlas_shared::FinancialStatementRequest {
        definition_id: None,
        report_type: payload.report_type,
        as_of_date: payload.as_of_date,
        period_name: payload.period_name,
        fiscal_year: payload.fiscal_year,
        currency_code: payload.currency_code,
        include_comparative: payload.include_comparative,
        row_definitions: None,
        column_definitions: None,
    };

    match state.financial_statements_engine.generate_statement(
        org_id,
        request,
        Some(user_id),
    ).await {
        Ok(statement) => Ok((StatusCode::CREATED, Json(serde_json::to_value(statement).unwrap()))),
        Err(e) => {
            error!("Failed to generate financial statement: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListStatementsQuery {
    pub report_type: Option<String>,
}

/// List generated financial statements
pub async fn list_financial_statements(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListStatementsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.financial_statements_engine.list_statements(
        org_id,
        query.report_type.as_deref(),
    ).await {
        Ok(statements) => Ok(Json(serde_json::json!({ "data": statements }))),
        Err(e) => {
            error!("Failed to list financial statements: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get a financial statement by ID
pub async fn get_financial_statement(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.financial_statements_engine.get_statement(id).await {
        Ok(Some(statement)) => Ok(Json(serde_json::to_value(statement).unwrap())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get financial statement: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
