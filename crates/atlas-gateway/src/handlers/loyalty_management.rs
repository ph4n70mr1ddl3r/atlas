//! Loyalty Management Handlers
//!
//! Oracle Fusion Cloud: CX > Loyalty Management

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

fn map_err(e: atlas_shared::AtlasError) -> StatusCode {
    match e {
        atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
        atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
        atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

fn parse_org(claims: &Claims) -> Result<Uuid, StatusCode> {
    Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

fn parse_user(claims: &Claims) -> Result<Uuid, StatusCode> {
    Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

// ========================================================================
// Programs
// ========================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProgramRequest {
    pub program_number: String,
    pub name: String,
    pub description: Option<String>,
    pub program_type: Option<String>,
    pub currency_code: Option<String>,
    pub points_name: Option<String>,
    pub enrollment_type: Option<String>,
    pub start_date: chrono::NaiveDate,
    pub end_date: Option<chrono::NaiveDate>,
    pub accrual_rate: Option<f64>,
    pub accrual_basis: Option<String>,
    pub minimum_accrual_amount: Option<f64>,
    pub rounding_method: Option<String>,
    pub points_expiry_days: Option<i32>,
    pub tier_qualification_period: Option<String>,
    pub auto_upgrade: Option<bool>,
    pub auto_downgrade: Option<bool>,
    pub max_points_per_member: Option<f64>,
    pub allow_point_transfer: Option<bool>,
    pub allow_redemption: Option<bool>,
    pub notes: Option<String>,
}

pub async fn create_program(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateProgramRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = parse_org(&claims)?;
    let user_id = parse_user(&claims)?;

    let program = state.loyalty_engine
        .create_program(
            org_id, &payload.program_number, &payload.name, payload.description.as_deref(),
            payload.program_type.as_deref().unwrap_or("points"),
            payload.currency_code.as_deref(), payload.points_name.as_deref(),
            payload.enrollment_type.as_deref(), payload.start_date, payload.end_date,
            payload.accrual_rate, payload.accrual_basis.as_deref(),
            payload.minimum_accrual_amount, payload.rounding_method.as_deref(),
            payload.points_expiry_days, payload.tier_qualification_period.as_deref(),
            payload.auto_upgrade, payload.auto_downgrade, payload.max_points_per_member,
            payload.allow_point_transfer, payload.allow_redemption,
            payload.notes.as_deref(), Some(user_id),
        )
        .await
        .map_err(map_err)?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(program).unwrap())))
}

pub async fn get_program(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let program = state.loyalty_engine.get_program(id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match program {
        Some(p) => Ok(Json(serde_json::to_value(p).unwrap())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListProgramsQuery {
    pub status: Option<String>,
    pub program_type: Option<String>,
}

pub async fn list_programs(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<ListProgramsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = parse_org(&claims)?;
    let programs = state.loyalty_engine.list_programs(
        org_id, params.status.as_deref(), params.program_type.as_deref(),
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({ "data": programs })))
}

pub async fn activate_program(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let program = state.loyalty_engine.activate_program(id).await.map_err(map_err)?;
    Ok(Json(serde_json::to_value(program).unwrap()))
}

pub async fn suspend_program(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let program = state.loyalty_engine.suspend_program(id).await.map_err(map_err)?;
    Ok(Json(serde_json::to_value(program).unwrap()))
}

pub async fn close_program(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let program = state.loyalty_engine.close_program(id).await.map_err(map_err)?;
    Ok(Json(serde_json::to_value(program).unwrap()))
}

pub async fn delete_program(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(program_number): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = parse_org(&claims)?;
    state.loyalty_engine.delete_program(org_id, &program_number).await.map_err(map_err)?;
    Ok(StatusCode::NO_CONTENT)
}

// ========================================================================
// Tiers
// ========================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTierRequest {
    pub tier_code: String,
    pub tier_name: String,
    pub tier_level: Option<i32>,
    pub minimum_points: Option<f64>,
    pub maximum_points: Option<f64>,
    pub accrual_bonus_percentage: Option<f64>,
    pub benefits: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub is_default: Option<bool>,
}

pub async fn create_tier(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(program_id): Path<Uuid>,
    Json(payload): Json<CreateTierRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = parse_org(&claims)?;

    let tier = state.loyalty_engine.create_tier(
        org_id, program_id, &payload.tier_code, &payload.tier_name,
        payload.tier_level.unwrap_or(0), payload.minimum_points.unwrap_or(0.0),
        payload.maximum_points, payload.accrual_bonus_percentage,
        payload.benefits.as_deref(), payload.color.as_deref(), payload.icon.as_deref(),
        payload.is_default,
    ).await.map_err(map_err)?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(tier).unwrap())))
}

pub async fn list_tiers(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(program_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let tiers = state.loyalty_engine.list_tiers(program_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({ "data": tiers })))
}

pub async fn delete_tier(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(tier_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    state.loyalty_engine.delete_tier(tier_id).await.map_err(map_err)?;
    Ok(StatusCode::NO_CONTENT)
}

// ========================================================================
// Members
// ========================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnrollMemberRequest {
    pub member_number: String,
    pub customer_id: Option<Uuid>,
    pub customer_name: String,
    pub customer_email: Option<String>,
    pub notes: Option<String>,
}

pub async fn enroll_member(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(program_id): Path<Uuid>,
    Json(payload): Json<EnrollMemberRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = parse_org(&claims)?;
    let user_id = parse_user(&claims)?;

    let member = state.loyalty_engine.enroll_member(
        org_id, program_id, &payload.member_number, payload.customer_id,
        &payload.customer_name, payload.customer_email.as_deref(),
        payload.notes.as_deref(), Some(user_id),
    ).await.map_err(map_err)?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(member).unwrap())))
}

pub async fn get_member(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.loyalty_engine.get_member(id).await {
        Ok(Some(m)) => Ok(Json(serde_json::to_value(m).unwrap())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListMembersQuery { pub status: Option<String> }

pub async fn list_members(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(program_id): Path<Uuid>,
    Query(params): Query<ListMembersQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let members = state.loyalty_engine.list_members(program_id, params.status.as_deref())
        .await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({ "data": members })))
}

pub async fn suspend_member(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let member = state.loyalty_engine.suspend_member(id).await.map_err(map_err)?;
    Ok(Json(serde_json::to_value(member).unwrap()))
}

pub async fn reactivate_member(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let member = state.loyalty_engine.reactivate_member(id).await.map_err(map_err)?;
    Ok(Json(serde_json::to_value(member).unwrap()))
}

pub async fn delete_member(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(member_number): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = parse_org(&claims)?;
    state.loyalty_engine.delete_member(org_id, &member_number).await.map_err(map_err)?;
    Ok(StatusCode::NO_CONTENT)
}

// ========================================================================
// Point Transactions
// ========================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccruePointsRequest {
    pub member_id: Uuid,
    pub transaction_number: String,
    pub source_type: Option<String>,
    pub source_id: Option<Uuid>,
    pub source_number: Option<String>,
    pub reference_amount: Option<f64>,
    pub reference_currency: Option<String>,
    pub description: Option<String>,
}

pub async fn accrue_points(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(program_id): Path<Uuid>,
    Json(payload): Json<AccruePointsRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = parse_org(&claims)?;
    let user_id = parse_user(&claims)?;

    let txn = state.loyalty_engine.accrue_points(
        org_id, program_id, payload.member_id, &payload.transaction_number,
        payload.source_type.as_deref(), payload.source_id, payload.source_number.as_deref(),
        payload.reference_amount, payload.reference_currency.as_deref(),
        payload.description.as_deref(), Some(user_id),
    ).await.map_err(map_err)?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(txn).unwrap())))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdjustPointsRequest {
    pub member_id: Uuid,
    pub transaction_number: String,
    pub points: f64,
    pub description: Option<String>,
}

pub async fn adjust_points(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(program_id): Path<Uuid>,
    Json(payload): Json<AdjustPointsRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = parse_org(&claims)?;
    let user_id = parse_user(&claims)?;

    let txn = state.loyalty_engine.adjust_points(
        org_id, program_id, payload.member_id, &payload.transaction_number,
        payload.points, payload.description.as_deref(), Some(user_id),
    ).await.map_err(map_err)?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(txn).unwrap())))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReverseTransactionRequest { pub reason: String }

pub async fn reverse_transaction(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<ReverseTransactionRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let txn = state.loyalty_engine.reverse_transaction(id, &payload.reason).await.map_err(map_err)?;
    Ok(Json(serde_json::to_value(txn).unwrap()))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListTransactionsQuery { pub txn_type: Option<String> }

pub async fn list_transactions(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(member_id): Path<Uuid>,
    Query(params): Query<ListTransactionsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let txns = state.loyalty_engine.list_transactions(member_id, params.txn_type.as_deref())
        .await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({ "data": txns })))
}

pub async fn delete_transaction(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(transaction_number): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = parse_org(&claims)?;
    state.loyalty_engine.delete_transaction(org_id, &transaction_number).await.map_err(map_err)?;
    Ok(StatusCode::NO_CONTENT)
}

// ========================================================================
// Rewards
// ========================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRewardRequest {
    pub reward_code: String,
    pub name: String,
    pub description: Option<String>,
    pub reward_type: Option<String>,
    pub points_required: f64,
    pub cash_value: Option<f64>,
    pub currency_code: Option<String>,
    pub tier_restriction: Option<String>,
    pub quantity_available: Option<i32>,
    pub max_per_member: Option<i32>,
    pub image_url: Option<String>,
    pub start_date: Option<chrono::NaiveDate>,
    pub end_date: Option<chrono::NaiveDate>,
    pub notes: Option<String>,
}

pub async fn create_reward(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(program_id): Path<Uuid>,
    Json(payload): Json<CreateRewardRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = parse_org(&claims)?;
    let user_id = parse_user(&claims)?;

    let reward = state.loyalty_engine.create_reward(
        org_id, program_id, &payload.reward_code, &payload.name,
        payload.description.as_deref(), payload.reward_type.as_deref().unwrap_or("merchandise"),
        payload.points_required, payload.cash_value, payload.currency_code.as_deref(),
        payload.tier_restriction.as_deref(), payload.quantity_available, payload.max_per_member,
        payload.image_url.as_deref(), payload.start_date, payload.end_date,
        payload.notes.as_deref(), Some(user_id),
    ).await.map_err(map_err)?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(reward).unwrap())))
}

#[derive(Debug, Deserialize)]
pub struct ListRewardsQuery { pub reward_type: Option<String> }

pub async fn list_rewards(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(program_id): Path<Uuid>,
    Query(params): Query<ListRewardsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let rewards = state.loyalty_engine.list_rewards(program_id, params.reward_type.as_deref())
        .await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({ "data": rewards })))
}

pub async fn deactivate_reward(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let reward = state.loyalty_engine.deactivate_reward(id).await.map_err(map_err)?;
    Ok(Json(serde_json::to_value(reward).unwrap()))
}

pub async fn delete_reward(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(reward_code): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = parse_org(&claims)?;
    state.loyalty_engine.delete_reward(org_id, &reward_code).await.map_err(map_err)?;
    Ok(StatusCode::NO_CONTENT)
}

// ========================================================================
// Redemptions
// ========================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RedeemRewardRequest {
    pub member_id: Uuid,
    pub reward_id: Uuid,
    pub redemption_number: String,
    pub quantity: Option<i32>,
    pub notes: Option<String>,
}

pub async fn redeem_reward(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(program_id): Path<Uuid>,
    Json(payload): Json<RedeemRewardRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = parse_org(&claims)?;
    let user_id = parse_user(&claims)?;

    let redemption = state.loyalty_engine.redeem_reward(
        org_id, program_id, payload.member_id, payload.reward_id,
        &payload.redemption_number, payload.quantity, payload.notes.as_deref(), Some(user_id),
    ).await.map_err(map_err)?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(redemption).unwrap())))
}

pub async fn fulfill_redemption(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let redemption = state.loyalty_engine.fulfill_redemption(id).await.map_err(map_err)?;
    Ok(Json(serde_json::to_value(redemption).unwrap()))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelRedemptionRequest { pub reason: String }

pub async fn cancel_redemption(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<CancelRedemptionRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let redemption = state.loyalty_engine.cancel_redemption(id, &payload.reason).await.map_err(map_err)?;
    Ok(Json(serde_json::to_value(redemption).unwrap()))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListRedemptionsQuery { pub status: Option<String> }

pub async fn list_redemptions(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(member_id): Path<Uuid>,
    Query(params): Query<ListRedemptionsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let redemptions = state.loyalty_engine.list_redemptions(member_id, params.status.as_deref())
        .await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({ "data": redemptions })))
}

// ========================================================================
// Dashboard
// ========================================================================

pub async fn get_loyalty_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = parse_org(&claims)?;
    let dashboard = state.loyalty_engine.get_dashboard(org_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::to_value(dashboard).unwrap()))
}
