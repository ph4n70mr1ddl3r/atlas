//! Treasury Management API Handlers
//!
//! REST endpoints for Oracle Fusion-inspired Treasury Management.
//! Manages counterparties, treasury deals (investments, borrowings, FX),
//! deal lifecycle, settlements, and dashboard.

use axum::{
    extract::{Path, Query, State, Extension},
    Json,
    http::StatusCode,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;
use tracing::error;

use crate::AppState;
use crate::handlers::auth::{Claims, parse_uuid};

// ============================================================================
// Counterparty Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCounterpartyRequest {
    pub counterparty_code: String,
    pub name: String,
    pub counterparty_type: Option<String>,
    pub country_code: Option<String>,
    pub credit_rating: Option<String>,
    pub credit_limit: Option<String>,
    pub settlement_currency: Option<String>,
    pub contact_name: Option<String>,
    pub contact_email: Option<String>,
    pub contact_phone: Option<String>,
}

pub async fn create_counterparty(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreateCounterpartyRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    let cp_type = req.counterparty_type.as_deref().unwrap_or("bank");
    match state.treasury_engine.create_counterparty(
        org_id, &req.counterparty_code, &req.name, cp_type,
        req.country_code.as_deref(), req.credit_rating.as_deref(),
        req.credit_limit.as_deref(), req.settlement_currency.as_deref(),
        req.contact_name.as_deref(), req.contact_email.as_deref(),
        req.contact_phone.as_deref(), None,
    ).await {
        Ok(cp) => Ok((StatusCode::CREATED, Json(serde_json::to_value(cp).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => {
            error!("Failed to create counterparty: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListCounterpartiesQuery {
    pub active_only: Option<bool>,
}

pub async fn list_counterparties(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<ListCounterpartiesQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    let active_only = query.active_only.unwrap_or(true);
    match state.treasury_engine.list_counterparties(org_id, active_only).await {
        Ok(cps) => Ok(Json(serde_json::json!({"data": cps}))),
        Err(e) => {
            error!("Failed to list counterparties: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

pub async fn get_counterparty(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(code): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    match state.treasury_engine.get_counterparty(org_id, &code).await {
        Ok(Some(cp)) => Ok(Json(serde_json::to_value(cp).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Counterparty not found"})))),
        Err(e) => {
            error!("Failed to get counterparty: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

pub async fn delete_counterparty(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(code): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    match state.treasury_engine.delete_counterparty(org_id, &code).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            error!("Failed to delete counterparty: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

// ============================================================================
// Deal Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDealRequest {
    pub deal_type: String,
    pub description: Option<String>,
    pub counterparty_id: Uuid,
    pub counterparty_name: Option<String>,
    pub currency_code: Option<String>,
    pub principal_amount: String,
    pub interest_rate: Option<String>,
    pub interest_basis: Option<String>,
    pub start_date: chrono::NaiveDate,
    pub maturity_date: chrono::NaiveDate,
    // FX-specific
    pub fx_buy_currency: Option<String>,
    pub fx_buy_amount: Option<String>,
    pub fx_sell_currency: Option<String>,
    pub fx_sell_amount: Option<String>,
    pub fx_rate: Option<String>,
    pub gl_account_code: Option<String>,
}

pub async fn create_deal(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreateDealRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    let currency = req.currency_code.as_deref().unwrap_or("USD");
    match state.treasury_engine.create_deal(
        org_id, &req.deal_type, req.description.as_deref(),
        req.counterparty_id, req.counterparty_name.as_deref(),
        currency, &req.principal_amount,
        req.interest_rate.as_deref(), req.interest_basis.as_deref(),
        req.start_date, req.maturity_date,
        req.fx_buy_currency.as_deref(), req.fx_buy_amount.as_deref(),
        req.fx_sell_currency.as_deref(), req.fx_sell_amount.as_deref(),
        req.fx_rate.as_deref(),
        req.gl_account_code.as_deref(), None,
    ).await {
        Ok(deal) => Ok((StatusCode::CREATED, Json(serde_json::to_value(deal).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => {
            error!("Failed to create deal: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListDealsQuery {
    pub deal_type: Option<String>,
    pub status: Option<String>,
}

pub async fn list_deals(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<ListDealsQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    match state.treasury_engine.list_deals(
        org_id, query.deal_type.as_deref(), query.status.as_deref(),
    ).await {
        Ok(deals) => Ok(Json(serde_json::json!({"data": deals}))),
        Err(e) => {
            error!("Failed to list deals: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

pub async fn get_deal(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.treasury_engine.get_deal(id).await {
        Ok(Some(deal)) => Ok(Json(serde_json::to_value(deal).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Deal not found"})))),
        Err(e) => {
            error!("Failed to get deal: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

// ============================================================================
// Deal Lifecycle Handlers
// ============================================================================

pub async fn authorize_deal(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = parse_uuid(&claims.sub).ok();
    match state.treasury_engine.authorize_deal(id, user_id).await {
        Ok(deal) => Ok(Json(serde_json::to_value(deal).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => {
            error!("Failed to authorize deal: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettleDealRequest {
    pub settlement_type: Option<String>,
    pub payment_reference: Option<String>,
}

pub async fn settle_deal(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(req): Json<SettleDealRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let user_id = parse_uuid(&claims.sub).ok();
    let settlement_type = req.settlement_type.as_deref().unwrap_or("full");
    match state.treasury_engine.settle_deal(
        id, settlement_type, req.payment_reference.as_deref(), user_id,
    ).await {
        Ok(settlement) => Ok((StatusCode::CREATED, Json(serde_json::to_value(settlement).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => {
            error!("Failed to settle deal: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

pub async fn mature_deal(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.treasury_engine.mature_deal(id).await {
        Ok(deal) => Ok(Json(serde_json::to_value(deal).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => {
            error!("Failed to mature deal: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

pub async fn cancel_deal(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.treasury_engine.cancel_deal(id).await {
        Ok(deal) => Ok(Json(serde_json::to_value(deal).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => {
            error!("Failed to cancel deal: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

// ============================================================================
// Settlement Handlers
// ============================================================================

pub async fn list_deal_settlements(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.treasury_engine.list_settlements(id).await {
        Ok(settlements) => Ok(Json(serde_json::json!({"data": settlements}))),
        Err(e) => {
            error!("Failed to list settlements: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

// ============================================================================
// Dashboard Handler
// ============================================================================

pub async fn get_treasury_dashboard(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    match state.treasury_engine.get_dashboard_summary(org_id).await {
        Ok(summary) => Ok(Json(serde_json::to_value(summary).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => {
            error!("Failed to get treasury dashboard: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}
