//! Rebate Management Handlers
//!
//! Oracle Fusion Cloud: Trade Management > Rebates
//! Provides HTTP endpoints for:
//! - Rebate agreement CRUD and lifecycle (activate, hold, terminate)
//! - Rebate tier management for tiered agreements
//! - Rebate transaction recording and status management
//! - Rebate accrual creation, posting, and reversal
//! - Rebate settlement approval and payment
//! - Rebate management dashboard

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension, Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::AppState;
use crate::handlers::auth::Claims;

// ============================================================================
// Agreements
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAgreementRequest {
    pub agreement_number: String,
    pub name: String,
    pub description: Option<String>,
    pub rebate_type: String,
    pub direction: String,
    pub partner_type: String,
    pub partner_id: Option<Uuid>,
    pub partner_name: Option<String>,
    pub partner_number: Option<String>,
    pub product_category: Option<String>,
    pub product_id: Option<Uuid>,
    pub product_name: Option<String>,
    pub uom: Option<String>,
    pub currency_code: Option<String>,
    pub start_date: chrono::NaiveDate,
    pub end_date: chrono::NaiveDate,
    pub calculation_method: Option<String>,
    pub accrual_account: Option<String>,
    pub liability_account: Option<String>,
    pub expense_account: Option<String>,
    pub payment_terms: Option<String>,
    pub settlement_frequency: Option<String>,
    pub minimum_amount: Option<f64>,
    pub maximum_amount: Option<f64>,
    pub auto_accrue: Option<bool>,
    pub requires_approval: Option<bool>,
    pub notes: Option<String>,
}

pub async fn create_agreement(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateAgreementRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let agreement = state.rebate_management_engine
        .create_agreement(
            org_id,
            &payload.agreement_number,
            &payload.name,
            payload.description.as_deref(),
            &payload.rebate_type,
            &payload.direction,
            &payload.partner_type,
            payload.partner_id,
            payload.partner_name.as_deref(),
            payload.partner_number.as_deref(),
            payload.product_category.as_deref(),
            payload.product_id,
            payload.product_name.as_deref(),
            payload.uom.as_deref(),
            payload.currency_code.as_deref().unwrap_or("USD"),
            payload.start_date,
            payload.end_date,
            payload.calculation_method.as_deref().unwrap_or("tiered"),
            payload.accrual_account.as_deref(),
            payload.liability_account.as_deref(),
            payload.expense_account.as_deref(),
            payload.payment_terms.as_deref(),
            payload.settlement_frequency.as_deref(),
            payload.minimum_amount,
            payload.maximum_amount,
            payload.auto_accrue,
            payload.requires_approval,
            payload.notes.as_deref(),
            Some(user_id),
        )
        .await
        .map_err(|e| {
            tracing::error!("Create rebate agreement error: {}", e);
            match e {
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(agreement).unwrap())))
}

pub async fn get_agreement(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let _org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let agreement = state.rebate_management_engine
        .get_agreement(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match agreement {
        Some(a) => Ok(Json(serde_json::to_value(a).unwrap())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

#[derive(Debug, Deserialize)]
pub struct ListAgreementsQuery {
    pub status: Option<String>,
    pub rebate_type: Option<String>,
    pub partner_type: Option<String>,
}

pub async fn list_agreements(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListAgreementsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let agreements = state.rebate_management_engine
        .list_agreements(
            org_id,
            query.status.as_deref(),
            query.rebate_type.as_deref(),
            query.partner_type.as_deref(),
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({ "data": agreements })))
}

pub async fn activate_agreement(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let _org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let agreement = state.rebate_management_engine
        .activate_agreement(id)
        .await
        .map_err(|e| {
            tracing::error!("Activate agreement error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(Json(serde_json::to_value(agreement).unwrap()))
}

pub async fn hold_agreement(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let _org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let agreement = state.rebate_management_engine
        .hold_agreement(id)
        .await
        .map_err(|e| {
            tracing::error!("Hold agreement error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(Json(serde_json::to_value(agreement).unwrap()))
}

pub async fn terminate_agreement(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let _org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let agreement = state.rebate_management_engine
        .terminate_agreement(id)
        .await
        .map_err(|e| {
            tracing::error!("Terminate agreement error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(Json(serde_json::to_value(agreement).unwrap()))
}

pub async fn delete_agreement(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(number): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    state.rebate_management_engine
        .delete_agreement(org_id, &number)
        .await
        .map_err(|e| {
            tracing::error!("Delete agreement error: {}", e);
            match e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Tiers
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTierRequest {
    pub tier_number: i32,
    pub from_value: f64,
    pub to_value: Option<f64>,
    pub rebate_rate: f64,
    pub rate_type: String,
    pub description: Option<String>,
}

pub async fn create_tier(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(agreement_id): Path<Uuid>,
    Json(payload): Json<CreateTierRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let tier = state.rebate_management_engine
        .create_tier(
            org_id, agreement_id,
            payload.tier_number, payload.from_value, payload.to_value,
            payload.rebate_rate, &payload.rate_type, payload.description.as_deref(),
        )
        .await
        .map_err(|e| {
            tracing::error!("Create tier error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(tier).unwrap())))
}

pub async fn list_tiers(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(agreement_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let tiers = state.rebate_management_engine
        .list_tiers(agreement_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({ "data": tiers })))
}

pub async fn delete_tier(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    state.rebate_management_engine
        .delete_tier(id)
        .await
        .map_err(|e| {
            match e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Transactions
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTransactionRequest {
    pub transaction_number: String,
    pub source_type: Option<String>,
    pub source_id: Option<Uuid>,
    pub source_number: Option<String>,
    pub transaction_date: chrono::NaiveDate,
    pub product_id: Option<Uuid>,
    pub product_name: Option<String>,
    pub quantity: Option<f64>,
    pub unit_price: Option<f64>,
    pub transaction_amount: f64,
    pub currency_code: Option<String>,
}

pub async fn create_transaction(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(agreement_id): Path<Uuid>,
    Json(payload): Json<CreateTransactionRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let txn = state.rebate_management_engine
        .create_transaction(
            org_id, agreement_id,
            &payload.transaction_number,
            payload.source_type.as_deref(),
            payload.source_id,
            payload.source_number.as_deref(),
            payload.transaction_date,
            payload.product_id,
            payload.product_name.as_deref(),
            payload.quantity.unwrap_or(0.0),
            payload.unit_price.unwrap_or(0.0),
            payload.transaction_amount,
            payload.currency_code.as_deref(),
            Some(user_id),
        )
        .await
        .map_err(|e| {
            tracing::error!("Create rebate transaction error: {}", e);
            match e {
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(txn).unwrap())))
}

pub async fn get_transaction(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let txn = state.rebate_management_engine
        .get_transaction(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match txn {
        Some(t) => Ok(Json(serde_json::to_value(t).unwrap())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

#[derive(Debug, Deserialize)]
pub struct ListTransactionsQuery {
    pub status: Option<String>,
}

pub async fn list_transactions(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(agreement_id): Path<Uuid>,
    Query(query): Query<ListTransactionsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let txns = state.rebate_management_engine
        .list_transactions(agreement_id, query.status.as_deref())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({ "data": txns })))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTransactionStatusRequest {
    pub status: String,
    pub reason: Option<String>,
}

pub async fn update_transaction_status(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateTransactionStatusRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let txn = state.rebate_management_engine
        .update_transaction_status(id, &payload.status, payload.reason.as_deref())
        .await
        .map_err(|e| {
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;
    Ok(Json(serde_json::to_value(txn).unwrap()))
}

pub async fn delete_transaction(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(number): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    state.rebate_management_engine
        .delete_transaction(org_id, &number)
        .await
        .map_err(|e| match e {
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        })?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Accruals
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAccrualRequest {
    pub accrual_number: String,
    pub accrual_date: chrono::NaiveDate,
    pub accrual_period: Option<String>,
    pub notes: Option<String>,
}

pub async fn create_accrual(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(agreement_id): Path<Uuid>,
    Json(payload): Json<CreateAccrualRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let accrual = state.rebate_management_engine
        .create_accrual(
            org_id, agreement_id,
            &payload.accrual_number,
            payload.accrual_date,
            payload.accrual_period.as_deref(),
            payload.notes.as_deref(),
            Some(user_id),
        )
        .await
        .map_err(|e| {
            tracing::error!("Create rebate accrual error: {}", e);
            match e {
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(accrual).unwrap())))
}

pub async fn get_accrual(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let accrual = state.rebate_management_engine
        .get_accrual(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match accrual {
        Some(a) => Ok(Json(serde_json::to_value(a).unwrap())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

#[derive(Debug, Deserialize)]
pub struct ListAccrualsQuery {
    pub status: Option<String>,
}

pub async fn list_accruals(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(agreement_id): Path<Uuid>,
    Query(query): Query<ListAccrualsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let accruals = state.rebate_management_engine
        .list_accruals(agreement_id, query.status.as_deref())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({ "data": accruals })))
}

pub async fn post_accrual(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let accrual = state.rebate_management_engine
        .post_accrual(id)
        .await
        .map_err(|e| match e {
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        })?;
    Ok(Json(serde_json::to_value(accrual).unwrap()))
}

pub async fn reverse_accrual(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let accrual = state.rebate_management_engine
        .reverse_accrual(id)
        .await
        .map_err(|e| match e {
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        })?;
    Ok(Json(serde_json::to_value(accrual).unwrap()))
}

pub async fn delete_accrual(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(number): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    state.rebate_management_engine
        .delete_accrual(org_id, &number)
        .await
        .map_err(|e| match e {
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        })?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Settlements
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSettlementRequest {
    pub settlement_number: String,
    pub settlement_date: chrono::NaiveDate,
    pub settlement_period_from: Option<chrono::NaiveDate>,
    pub settlement_period_to: Option<chrono::NaiveDate>,
    pub settlement_type: Option<String>,
    pub payment_method: Option<String>,
    pub notes: Option<String>,
}

pub async fn create_settlement(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(agreement_id): Path<Uuid>,
    Json(payload): Json<CreateSettlementRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let settlement = state.rebate_management_engine
        .create_settlement(
            org_id, agreement_id,
            &payload.settlement_number,
            payload.settlement_date,
            payload.settlement_period_from,
            payload.settlement_period_to,
            payload.settlement_type.as_deref().unwrap_or("payment"),
            payload.payment_method.as_deref(),
            payload.notes.as_deref(),
            Some(user_id),
        )
        .await
        .map_err(|e| {
            tracing::error!("Create rebate settlement error: {}", e);
            match e {
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(settlement).unwrap())))
}

pub async fn get_settlement(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let settlement = state.rebate_management_engine
        .get_settlement(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match settlement {
        Some(s) => Ok(Json(serde_json::to_value(s).unwrap())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

#[derive(Debug, Deserialize)]
pub struct ListSettlementsQuery {
    pub status: Option<String>,
}

pub async fn list_settlements(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(agreement_id): Path<Uuid>,
    Query(query): Query<ListSettlementsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let settlements = state.rebate_management_engine
        .list_settlements(agreement_id, query.status.as_deref())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({ "data": settlements })))
}

pub async fn approve_settlement(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let settlement = state.rebate_management_engine
        .approve_settlement(id, user_id)
        .await
        .map_err(|e| match e {
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        })?;
    Ok(Json(serde_json::to_value(settlement).unwrap()))
}

pub async fn pay_settlement(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let settlement = state.rebate_management_engine
        .pay_settlement(id)
        .await
        .map_err(|e| match e {
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        })?;
    Ok(Json(serde_json::to_value(settlement).unwrap()))
}

pub async fn cancel_settlement(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let settlement = state.rebate_management_engine
        .cancel_settlement(id)
        .await
        .map_err(|e| match e {
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        })?;
    Ok(Json(serde_json::to_value(settlement).unwrap()))
}

pub async fn delete_settlement(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(number): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    state.rebate_management_engine
        .delete_settlement(org_id, &number)
        .await
        .map_err(|e| match e {
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        })?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_settlement_lines(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(settlement_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let lines = state.rebate_management_engine
        .list_settlement_lines(settlement_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({ "data": lines })))
}

// ============================================================================
// Dashboard
// ============================================================================

pub async fn get_rebate_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let dashboard = state.rebate_management_engine
        .get_dashboard(org_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::to_value(dashboard).unwrap()))
}
