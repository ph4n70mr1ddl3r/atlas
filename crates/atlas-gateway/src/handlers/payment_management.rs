//! Payment Management Handlers
//!
//! Oracle Fusion Cloud ERP: Financials > Payments
//!
//! API endpoints for managing payments, payment batches,
//! and payment reversals.

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
pub struct CreatePaymentRequest {
    pub supplier_id: Uuid,
    pub supplier_number: Option<String>,
    pub supplier_name: Option<String>,
    pub supplier_site: Option<String>,
    pub payment_date: chrono::NaiveDate,
    pub payment_method: String,
    pub currency_code: String,
    pub payment_amount: String,
    #[serde(default = "default_zero")]
    pub discount_taken: String,
    pub bank_account_id: Option<Uuid>,
    pub bank_account_name: Option<String>,
    pub cash_account_code: Option<String>,
    pub ap_account_code: Option<String>,
    pub discount_account_code: Option<String>,
    pub check_number: Option<String>,
    pub batch_id: Option<Uuid>,
}

fn default_zero() -> String { "0.00".to_string() }

/// Create a new payment
pub async fn create_payment(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreatePaymentRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    info!("Creating payment for org {} supplier {}", org_id, payload.supplier_id);

    match state.payment_engine.create_payment(
        org_id,
        payload.batch_id,
        payload.supplier_id,
        payload.supplier_number.as_deref(),
        payload.supplier_name.as_deref(),
        payload.supplier_site.as_deref(),
        payload.payment_date,
        &payload.payment_method,
        &payload.currency_code,
        &payload.payment_amount,
        &payload.discount_taken,
        payload.bank_account_id,
        payload.bank_account_name.as_deref(),
        payload.cash_account_code.as_deref(),
        payload.ap_account_code.as_deref(),
        payload.discount_account_code.as_deref(),
        payload.check_number.as_deref(),
        Some(user_id),
    ).await {
        Ok(payment) => Ok((StatusCode::CREATED, Json(serde_json::to_value(payment).unwrap()))),
        Err(e) => {
            error!("Failed to create payment: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListPaymentsQuery {
    pub status: Option<String>,
    pub supplier_id: Option<Uuid>,
}

/// List payments
pub async fn list_payments(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListPaymentsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.payment_engine.list_payments(
        org_id,
        query.status.as_deref(),
        query.supplier_id,
        None,
    ).await {
        Ok(payments) => Ok(Json(serde_json::json!({ "data": payments }))),
        Err(e) => {
            error!("Failed to list payments: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get a payment by ID
pub async fn get_payment(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.payment_engine.get_payment(id).await {
        Ok(Some(payment)) => Ok(Json(serde_json::to_value(payment).unwrap())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get payment: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Issue (confirm) a payment
pub async fn issue_payment(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.payment_engine.issue_payment(id).await {
        Ok(payment) => Ok(Json(serde_json::to_value(payment).unwrap())),
        Err(e) => {
            error!("Failed to issue payment: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

/// Clear a payment
pub async fn clear_payment(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.payment_engine.clear_payment(id, Some(user_id)).await {
        Ok(payment) => Ok(Json(serde_json::to_value(payment).unwrap())),
        Err(e) => {
            error!("Failed to clear payment: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

/// Void a payment
pub async fn void_payment(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let reason = body["reason"].as_str().unwrap_or("Voided");
    match state.payment_engine.void_payment(id, user_id, reason).await {
        Ok(payment) => Ok(Json(serde_json::to_value(payment).unwrap())),
        Err(e) => {
            error!("Failed to void payment: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}
