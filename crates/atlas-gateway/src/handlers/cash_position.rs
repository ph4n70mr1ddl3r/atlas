//! Cash Position Handlers
//!
//! Oracle Fusion: Financials > Treasury > Cash Position

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

#[derive(Debug, Deserialize)]
pub struct RecordPositionRequest {
    pub bank_account_id: Uuid,
    pub bank_account_number: Option<String>,
    pub bank_account_name: Option<String>,
    pub currency_code: String,
    pub opening_balance: String,
    pub total_inflows: String,
    pub total_outflows: String,
    pub closing_balance: String,
    pub ledger_balance: String,
    pub available_balance: String,
    pub hold_amount: String,
    pub position_date: chrono::NaiveDate,
    pub source_breakdown: Option<serde_json::Value>,
}

pub async fn record_position(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<RecordPositionRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.cash_position_engine.record_position(
        org_id, payload.bank_account_id,
        payload.bank_account_number.as_deref(), payload.bank_account_name.as_deref(),
        &payload.currency_code, &payload.opening_balance, &payload.total_inflows,
        &payload.total_outflows, &payload.closing_balance, &payload.ledger_balance,
        &payload.available_balance, &payload.hold_amount, payload.position_date,
        payload.source_breakdown.clone().unwrap_or(serde_json::json!({})),
    ).await {
        Ok(p) => Ok((StatusCode::CREATED, Json(serde_json::to_value(p).unwrap()))),
        Err(e) => {
            error!("Failed to record position: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListPositionsQuery {
    pub position_date: Option<chrono::NaiveDate>,
    pub currency_code: Option<String>,
}

pub async fn list_positions(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListPositionsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.cash_position_engine.list_positions(org_id, query.position_date, query.currency_code.as_deref()).await {
        Ok(positions) => Ok(Json(serde_json::json!({ "data": positions }))),
        Err(e) => { error!("Failed to list positions: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn get_cash_position_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.cash_position_engine.get_dashboard(org_id).await {
        Ok(dashboard) => Ok(Json(serde_json::to_value(dashboard).unwrap())),
        Err(e) => { error!("Failed to get dashboard: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}
