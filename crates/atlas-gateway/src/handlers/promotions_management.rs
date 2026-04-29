//! Promotions Management API Handlers
//!
//! Oracle Fusion Trade Management > Trade Promotion
//!
//! Endpoints for managing trade promotions, offers, fund allocations,
//! claims processing, and ROI analytics.

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
use crate::handlers::auth::Claims;

// ============================================================================
// Query Parameters
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ListPromotionsQuery {
    pub promotion_type: Option<String>,
    pub status: Option<String>,
    pub include_inactive: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListClaimsQuery {
    pub status: Option<String>,
}

// ============================================================================
// Promotion CRUD
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePromotionRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub promotion_type: String,
    pub start_date: chrono::NaiveDate,
    pub end_date: chrono::NaiveDate,
    pub customer_id: Option<String>,
    pub customer_name: Option<String>,
    pub territory_id: Option<String>,
    pub product_id: Option<String>,
    pub product_name: Option<String>,
    pub budget_amount: String,
    pub currency_code: Option<String>,
    pub owner_id: Option<String>,
    pub owner_name: Option<String>,
}

pub async fn create_promotion(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreatePromotionRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let customer_id = payload.customer_id
        .map(|s| Uuid::parse_str(&s))
        .transpose()
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let territory_id = payload.territory_id
        .map(|s| Uuid::parse_str(&s))
        .transpose()
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let product_id = payload.product_id
        .map(|s| Uuid::parse_str(&s))
        .transpose()
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let owner_id = payload.owner_id
        .map(|s| Uuid::parse_str(&s))
        .transpose()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    match state.promotions_engine.create_promotion(
        org_id,
        &payload.code,
        &payload.name,
        payload.description.as_deref(),
        &payload.promotion_type,
        payload.start_date,
        payload.end_date,
        customer_id,
        payload.customer_name.as_deref(),
        territory_id,
        product_id,
        payload.product_name.as_deref(),
        &payload.budget_amount,
        payload.currency_code.as_deref().unwrap_or("USD"),
        owner_id,
        payload.owner_name.as_deref(),
        None,
    ).await {
        Ok(promotion) => Ok((StatusCode::CREATED, Json(serde_json::to_value(promotion).unwrap_or_else(|e| {
            error!("Serialization error: {}", e); serde_json::Value::Null
        })))),
        Err(e) => {
            error!("Failed to create promotion: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                409 => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn get_promotion(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let id = Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;
    match state.promotions_engine.get_promotion(id).await {
        Ok(Some(p)) => Ok(Json(serde_json::to_value(p).unwrap_or(serde_json::Value::Null))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Failed to get promotion: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn list_promotions(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListPromotionsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let include_inactive = query.include_inactive
        .map(|s| s == "true" || s == "1")
        .unwrap_or(false);

    match state.promotions_engine.list_promotions(
        org_id,
        query.promotion_type.as_deref(),
        query.status.as_deref(),
        include_inactive,
    ).await {
        Ok(promotions) => Ok(Json(serde_json::to_value(promotions).unwrap_or(serde_json::Value::Null))),
        Err(e) => { error!("Failed to list promotions: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePromotionRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub start_date: Option<chrono::NaiveDate>,
    pub end_date: Option<chrono::NaiveDate>,
    pub budget_amount: Option<String>,
    pub owner_id: Option<String>,
    pub owner_name: Option<String>,
}

pub async fn update_promotion(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<String>,
    Json(payload): Json<UpdatePromotionRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let id = Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;
    let owner_id = payload.owner_id
        .map(|s| Uuid::parse_str(&s))
        .transpose()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    match state.promotions_engine.update_promotion(
        id,
        payload.name.as_deref(),
        payload.description.as_deref(),
        payload.start_date,
        payload.end_date,
        payload.budget_amount.as_deref(),
        owner_id,
        payload.owner_name.as_deref(),
    ).await {
        Ok(p) => Ok(Json(serde_json::to_value(p).unwrap_or(serde_json::Value::Null))),
        Err(e) => {
            error!("Failed to update promotion: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn activate_promotion(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let id = Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;
    match state.promotions_engine.activate_promotion(id).await {
        Ok(p) => Ok(Json(serde_json::to_value(p).unwrap_or(serde_json::Value::Null))),
        Err(e) => {
            error!("Failed to activate promotion: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 404 => StatusCode::NOT_FOUND, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

pub async fn hold_promotion(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let id = Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;
    match state.promotions_engine.hold_promotion(id).await {
        Ok(p) => Ok(Json(serde_json::to_value(p).unwrap_or(serde_json::Value::Null))),
        Err(e) => {
            error!("Failed to hold promotion: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 404 => StatusCode::NOT_FOUND, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

pub async fn complete_promotion(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let id = Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;
    match state.promotions_engine.complete_promotion(id).await {
        Ok(p) => Ok(Json(serde_json::to_value(p).unwrap_or(serde_json::Value::Null))),
        Err(e) => {
            error!("Failed to complete promotion: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 404 => StatusCode::NOT_FOUND, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

pub async fn cancel_promotion(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let id = Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;
    match state.promotions_engine.cancel_promotion(id).await {
        Ok(p) => Ok(Json(serde_json::to_value(p).unwrap_or(serde_json::Value::Null))),
        Err(e) => {
            error!("Failed to cancel promotion: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 404 => StatusCode::NOT_FOUND, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

pub async fn delete_promotion(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let id = Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;
    match state.promotions_engine.delete_promotion(id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            error!("Failed to delete promotion: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 404 => StatusCode::NOT_FOUND, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

// ============================================================================
// Offers
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOfferRequest {
    pub offer_type: String,
    pub description: Option<String>,
    pub discount_type: String,
    pub discount_value: String,
    pub buy_quantity: Option<i32>,
    pub get_quantity: Option<i32>,
    pub minimum_purchase: Option<String>,
    pub maximum_discount: Option<String>,
}

pub async fn create_offer(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(promotion_id): Path<String>,
    Json(payload): Json<CreateOfferRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let promotion_id = Uuid::parse_str(&promotion_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    match state.promotions_engine.create_offer(
        org_id,
        promotion_id,
        &payload.offer_type,
        payload.description.as_deref(),
        &payload.discount_type,
        &payload.discount_value,
        payload.buy_quantity,
        payload.get_quantity,
        payload.minimum_purchase.as_deref(),
        payload.maximum_discount.as_deref(),
        None,
    ).await {
        Ok(o) => Ok((StatusCode::CREATED, Json(serde_json::to_value(o).unwrap_or(serde_json::Value::Null)))),
        Err(e) => {
            error!("Failed to create offer: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 404 => StatusCode::NOT_FOUND, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

pub async fn list_offers(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(promotion_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let promotion_id = Uuid::parse_str(&promotion_id).map_err(|_| StatusCode::BAD_REQUEST)?;
    match state.promotions_engine.list_offers(promotion_id).await {
        Ok(offers) => Ok(Json(serde_json::to_value(offers).unwrap_or(serde_json::Value::Null))),
        Err(e) => { error!("Failed to list offers: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn delete_offer(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(offer_id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let offer_id = Uuid::parse_str(&offer_id).map_err(|_| StatusCode::BAD_REQUEST)?;
    match state.promotions_engine.delete_offer(offer_id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => { error!("Failed to delete offer: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// ============================================================================
// Funds
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateFundRequest {
    pub fund_type: String,
    pub allocated_amount: String,
    pub currency_code: Option<String>,
}

pub async fn create_fund(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(promotion_id): Path<String>,
    Json(payload): Json<CreateFundRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let promotion_id = Uuid::parse_str(&promotion_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    match state.promotions_engine.create_fund(
        org_id,
        promotion_id,
        &payload.fund_type,
        &payload.allocated_amount,
        payload.currency_code.as_deref().unwrap_or("USD"),
        None,
    ).await {
        Ok(f) => Ok((StatusCode::CREATED, Json(serde_json::to_value(f).unwrap_or(serde_json::Value::Null)))),
        Err(e) => {
            error!("Failed to create fund: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 404 => StatusCode::NOT_FOUND, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

pub async fn list_funds(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(promotion_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let promotion_id = Uuid::parse_str(&promotion_id).map_err(|_| StatusCode::BAD_REQUEST)?;
    match state.promotions_engine.list_funds(promotion_id).await {
        Ok(funds) => Ok(Json(serde_json::to_value(funds).unwrap_or(serde_json::Value::Null))),
        Err(e) => { error!("Failed to list funds: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateFundAmountRequest {
    pub amount: String,
}

pub async fn update_fund_committed(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(fund_id): Path<String>,
    Json(payload): Json<UpdateFundAmountRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let fund_id = Uuid::parse_str(&fund_id).map_err(|_| StatusCode::BAD_REQUEST)?;
    match state.promotions_engine.update_fund_committed(fund_id, &payload.amount).await {
        Ok(f) => Ok(Json(serde_json::to_value(f).unwrap_or(serde_json::Value::Null))),
        Err(e) => {
            error!("Failed to update fund committed: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

pub async fn update_fund_spent(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(fund_id): Path<String>,
    Json(payload): Json<UpdateFundAmountRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let fund_id = Uuid::parse_str(&fund_id).map_err(|_| StatusCode::BAD_REQUEST)?;
    match state.promotions_engine.update_fund_spent(fund_id, &payload.amount).await {
        Ok(f) => Ok(Json(serde_json::to_value(f).unwrap_or(serde_json::Value::Null))),
        Err(e) => {
            error!("Failed to update fund spent: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

pub async fn delete_fund(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(fund_id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let fund_id = Uuid::parse_str(&fund_id).map_err(|_| StatusCode::BAD_REQUEST)?;
    match state.promotions_engine.delete_fund(fund_id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => { error!("Failed to delete fund: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// ============================================================================
// Claims
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateClaimRequest {
    pub claim_type: String,
    pub amount: String,
    pub currency_code: Option<String>,
    pub claim_date: chrono::NaiveDate,
    pub customer_id: Option<String>,
    pub customer_name: Option<String>,
    pub description: Option<String>,
}

pub async fn create_claim(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(promotion_id): Path<String>,
    Json(payload): Json<CreateClaimRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let promotion_id = Uuid::parse_str(&promotion_id).map_err(|_| StatusCode::BAD_REQUEST)?;
    let customer_id = payload.customer_id
        .map(|s| Uuid::parse_str(&s))
        .transpose()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    match state.promotions_engine.create_claim(
        org_id,
        promotion_id,
        &payload.claim_type,
        &payload.amount,
        payload.currency_code.as_deref().unwrap_or("USD"),
        payload.claim_date,
        customer_id,
        payload.customer_name.as_deref(),
        payload.description.as_deref(),
        None,
    ).await {
        Ok(c) => Ok((StatusCode::CREATED, Json(serde_json::to_value(c).unwrap_or(serde_json::Value::Null)))),
        Err(e) => {
            error!("Failed to create claim: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 404 => StatusCode::NOT_FOUND, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

pub async fn get_claim(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(claim_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let claim_id = Uuid::parse_str(&claim_id).map_err(|_| StatusCode::BAD_REQUEST)?;
    match state.promotions_engine.get_claim(claim_id).await {
        Ok(Some(c)) => Ok(Json(serde_json::to_value(c).unwrap_or(serde_json::Value::Null))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Failed to get claim: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn list_claims(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(promotion_id): Path<String>,
    Query(query): Query<ListClaimsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let promotion_id = Uuid::parse_str(&promotion_id).map_err(|_| StatusCode::BAD_REQUEST)?;
    match state.promotions_engine.list_claims(promotion_id, query.status.as_deref()).await {
        Ok(claims) => Ok(Json(serde_json::to_value(claims).unwrap_or(serde_json::Value::Null))),
        Err(e) => { error!("Failed to list claims: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn review_claim(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(claim_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let claim_id = Uuid::parse_str(&claim_id).map_err(|_| StatusCode::BAD_REQUEST)?;
    match state.promotions_engine.review_claim(claim_id).await {
        Ok(c) => Ok(Json(serde_json::to_value(c).unwrap_or(serde_json::Value::Null))),
        Err(e) => {
            error!("Failed to review claim: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 404 => StatusCode::NOT_FOUND, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApproveClaimRequest {
    pub approved_amount: Option<String>,
}

pub async fn approve_claim(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(claim_id): Path<String>,
    Json(payload): Json<ApproveClaimRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let claim_id = Uuid::parse_str(&claim_id).map_err(|_| StatusCode::BAD_REQUEST)?;
    match state.promotions_engine.approve_claim(claim_id, payload.approved_amount.as_deref()).await {
        Ok(c) => Ok(Json(serde_json::to_value(c).unwrap_or(serde_json::Value::Null))),
        Err(e) => {
            error!("Failed to approve claim: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 404 => StatusCode::NOT_FOUND, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RejectClaimRequest {
    pub reason: String,
}

pub async fn reject_claim(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(claim_id): Path<String>,
    Json(payload): Json<RejectClaimRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let claim_id = Uuid::parse_str(&claim_id).map_err(|_| StatusCode::BAD_REQUEST)?;
    match state.promotions_engine.reject_claim(claim_id, &payload.reason).await {
        Ok(c) => Ok(Json(serde_json::to_value(c).unwrap_or(serde_json::Value::Null))),
        Err(e) => {
            error!("Failed to reject claim: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 404 => StatusCode::NOT_FOUND, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettleClaimRequest {
    pub paid_amount: String,
}

pub async fn settle_claim(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(claim_id): Path<String>,
    Json(payload): Json<SettleClaimRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let claim_id = Uuid::parse_str(&claim_id).map_err(|_| StatusCode::BAD_REQUEST)?;
    match state.promotions_engine.settle_claim(claim_id, &payload.paid_amount).await {
        Ok(c) => Ok(Json(serde_json::to_value(c).unwrap_or(serde_json::Value::Null))),
        Err(e) => {
            error!("Failed to settle claim: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 404 => StatusCode::NOT_FOUND, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

pub async fn delete_claim(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(claim_id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let claim_id = Uuid::parse_str(&claim_id).map_err(|_| StatusCode::BAD_REQUEST)?;
    match state.promotions_engine.delete_claim(claim_id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            error!("Failed to delete claim: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 404 => StatusCode::NOT_FOUND, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

// ============================================================================
// Dashboard
// ============================================================================

pub async fn get_promotions_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.promotions_engine.get_dashboard(org_id).await {
        Ok(dashboard) => Ok(Json(serde_json::to_value(dashboard).unwrap_or(serde_json::Value::Null))),
        Err(e) => { error!("Failed to get promotions dashboard: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}
