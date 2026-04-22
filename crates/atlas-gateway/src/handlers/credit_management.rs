//! Credit Management Handlers
//!
//! Oracle Fusion Cloud: Receivables > Credit Management
//!
//! API endpoints for managing credit scoring models, credit profiles,
//! credit limits, credit check rules, credit exposure, credit holds,
//! credit reviews, and the credit management dashboard.

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
// Scoring Model Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateScoringModelRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub model_type: String,
    pub scoring_criteria: serde_json::Value,
    pub score_ranges: serde_json::Value,
}

pub async fn create_scoring_model(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateScoringModelRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.credit_management_engine.create_scoring_model(
        org_id, &payload.code, &payload.name, payload.description.as_deref(),
        &payload.model_type, payload.scoring_criteria, payload.score_ranges,
        Some(user_id),
    ).await {
        Ok(model) => Ok((StatusCode::CREATED, Json(serde_json::to_value(model).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => {
            error!("Failed to create scoring model: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                409 => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn get_scoring_model(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(code): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.credit_management_engine.get_scoring_model_by_code(org_id, &code).await {
        Ok(Some(m)) => Ok(Json(serde_json::to_value(m).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn list_scoring_models(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.credit_management_engine.list_scoring_models(org_id).await {
        Ok(models) => Ok(Json(serde_json::json!({"data": models}))),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn delete_scoring_model(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(code): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.credit_management_engine.delete_scoring_model(org_id, &code).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// ============================================================================
// Credit Profile Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateProfileRequest {
    pub profile_number: String,
    pub profile_name: String,
    pub description: Option<String>,
    pub profile_type: String,
    pub customer_id: Option<Uuid>,
    pub customer_name: Option<String>,
    pub customer_group_id: Option<Uuid>,
    pub customer_group_name: Option<String>,
    pub scoring_model_id: Option<Uuid>,
    #[serde(default = "default_ninety")]
    pub review_frequency_days: i32,
}

fn default_ninety() -> i32 { 90 }

pub async fn create_profile(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateProfileRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.credit_management_engine.create_profile(
        org_id, &payload.profile_number, &payload.profile_name,
        payload.description.as_deref(), &payload.profile_type,
        payload.customer_id, payload.customer_name.as_deref(),
        payload.customer_group_id, payload.customer_group_name.as_deref(),
        payload.scoring_model_id, payload.review_frequency_days,
        Some(user_id),
    ).await {
        Ok(profile) => Ok((StatusCode::CREATED, Json(serde_json::to_value(profile).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => {
            error!("Failed to create credit profile: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                409 => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn get_profile(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.credit_management_engine.get_profile(id).await {
        Ok(Some(p)) => Ok(Json(serde_json::to_value(p).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListProfilesQuery {
    pub status: Option<String>,
}

pub async fn list_profiles(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<ListProfilesQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.credit_management_engine.list_profiles(org_id, params.status.as_deref()).await {
        Ok(profiles) => Ok(Json(serde_json::json!({"data": profiles}))),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateProfileStatusRequest {
    pub status: String,
}

pub async fn update_profile_status(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateProfileStatusRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.credit_management_engine.update_profile_status(id, &payload.status).await {
        Ok(p) => Ok(Json(serde_json::to_value(p).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => {
            error!("Error: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateProfileScoreRequest {
    pub credit_score: String,
    pub credit_rating: String,
    pub risk_level: String,
}

pub async fn update_profile_score(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateProfileScoreRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.credit_management_engine.update_profile_score(
        id, &payload.credit_score, &payload.credit_rating, &payload.risk_level,
    ).await {
        Ok(p) => Ok(Json(serde_json::to_value(p).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => {
            error!("Error: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn delete_profile(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    // Get the profile to find its number for deletion
    match state.credit_management_engine.get_profile(id).await {
        Ok(Some(profile)) => {
            let org_id = Uuid::parse_str(&_claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            match state.credit_management_engine.delete_profile(org_id, &profile.profile_number).await {
                Ok(()) => Ok(StatusCode::NO_CONTENT),
                Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
            }
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// ============================================================================
// Credit Limit Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateCreditLimitRequest {
    pub profile_id: Uuid,
    pub limit_type: String,
    pub currency_code: Option<String>,
    pub credit_limit: String,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
}

pub async fn create_credit_limit(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateCreditLimitRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.credit_management_engine.create_credit_limit(
        org_id, payload.profile_id, &payload.limit_type,
        payload.currency_code.as_deref(), &payload.credit_limit,
        payload.effective_from, payload.effective_to, Some(user_id),
    ).await {
        Ok(limit) => Ok((StatusCode::CREATED, Json(serde_json::to_value(limit).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => {
            error!("Failed to create credit limit: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn list_credit_limits(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(profile_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.credit_management_engine.list_credit_limits(profile_id).await {
        Ok(limits) => Ok(Json(serde_json::json!({"data": limits}))),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateCreditLimitRequest {
    pub credit_limit: String,
}

pub async fn update_credit_limit(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateCreditLimitRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.credit_management_engine.update_credit_limit_amount(id, &payload.credit_limit).await {
        Ok(l) => Ok(Json(serde_json::to_value(l).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => {
            error!("Error: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SetTempLimitRequest {
    pub temp_limit_increase: String,
    pub temp_limit_expiry: chrono::NaiveDate,
}

pub async fn set_temp_limit(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<SetTempLimitRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.credit_management_engine.set_temp_limit(
        id, &payload.temp_limit_increase, Some(payload.temp_limit_expiry),
    ).await {
        Ok(l) => Ok(Json(serde_json::to_value(l).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => {
            error!("Error: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn delete_credit_limit(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    match state.credit_management_engine.delete_credit_limit(id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// ============================================================================
// Credit Check Rule Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateCheckRuleRequest {
    pub name: String,
    pub description: Option<String>,
    pub check_point: String,
    pub check_type: String,
    pub condition: serde_json::Value,
    pub action_on_failure: String,
    #[serde(default = "default_ten")]
    pub priority: i32,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
}

fn default_ten() -> i32 { 10 }

pub async fn create_check_rule(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateCheckRuleRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.credit_management_engine.create_check_rule(
        org_id, &payload.name, payload.description.as_deref(),
        &payload.check_point, &payload.check_type, payload.condition,
        &payload.action_on_failure, payload.priority,
        payload.effective_from, payload.effective_to, Some(user_id),
    ).await {
        Ok(rule) => Ok((StatusCode::CREATED, Json(serde_json::to_value(rule).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => {
            error!("Failed to create check rule: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                409 => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn list_check_rules(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.credit_management_engine.list_check_rules(org_id).await {
        Ok(rules) => Ok(Json(serde_json::json!({"data": rules}))),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn delete_check_rule(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    match state.credit_management_engine.delete_check_rule(id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// ============================================================================
// Credit Exposure Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CalculateExposureRequest {
    pub profile_id: Uuid,
    pub currency_code: String,
    pub open_receivables: String,
    pub open_orders: String,
    pub open_shipments: String,
    pub open_invoices: String,
    pub unapplied_cash: String,
    pub on_hold_amount: String,
}

pub async fn calculate_exposure(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CalculateExposureRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.credit_management_engine.calculate_exposure(
        org_id, payload.profile_id, &payload.currency_code,
        &payload.open_receivables, &payload.open_orders,
        &payload.open_shipments, &payload.open_invoices,
        &payload.unapplied_cash, &payload.on_hold_amount,
    ).await {
        Ok(exposure) => Ok(Json(serde_json::to_value(exposure).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => {
            error!("Error: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn get_latest_exposure(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(profile_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.credit_management_engine.get_latest_exposure(profile_id).await {
        Ok(Some(e)) => Ok(Json(serde_json::to_value(e).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// ============================================================================
// Credit Check (perform)
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct PerformCreditCheckRequest {
    pub profile_id: Uuid,
    pub requested_amount: String,
    pub check_point: String,
}

pub async fn perform_credit_check(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<PerformCreditCheckRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.credit_management_engine.perform_credit_check(
        org_id, payload.profile_id, &payload.requested_amount, &payload.check_point,
    ).await {
        Ok(result) => {
            Ok(Json(serde_json::json!({
                "passed": result.passed,
                "reason": result.reason,
                "exposure": result.exposure,
            })))
        }
        Err(e) => {
            error!("Error: {}", e);
            Err(match e.status_code() {
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// ============================================================================
// Credit Hold Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateHoldRequest {
    pub profile_id: Uuid,
    pub hold_type: String,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub entity_number: Option<String>,
    pub hold_amount: Option<String>,
    pub reason: Option<String>,
}

pub async fn create_hold(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateHoldRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.credit_management_engine.create_hold(
        org_id, payload.profile_id, &payload.hold_type,
        &payload.entity_type, payload.entity_id,
        payload.entity_number.as_deref(), payload.hold_amount.as_deref(),
        payload.reason.as_deref(), Some(user_id),
    ).await {
        Ok(hold) => Ok((StatusCode::CREATED, Json(serde_json::to_value(hold).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => {
            error!("Failed to create hold: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListHoldsQuery {
    pub status: Option<String>,
    pub profile_id: Option<Uuid>,
}

pub async fn list_holds(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<ListHoldsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.credit_management_engine.list_holds(org_id, params.status.as_deref(), params.profile_id).await {
        Ok(holds) => Ok(Json(serde_json::json!({"data": holds}))),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

#[derive(Debug, Deserialize)]
pub struct ReleaseHoldRequest {
    pub release_reason: Option<String>,
}

pub async fn release_hold(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<ReleaseHoldRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.credit_management_engine.release_hold(id, Some(user_id), payload.release_reason.as_deref()).await {
        Ok(h) => Ok(Json(serde_json::to_value(h).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => {
            error!("Error: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct OverrideHoldRequest {
    pub override_reason: String,
}

pub async fn override_hold(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<OverrideHoldRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.credit_management_engine.override_hold(id, Some(user_id), Some(&payload.override_reason)).await {
        Ok(h) => Ok(Json(serde_json::to_value(h).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => {
            error!("Error: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// ============================================================================
// Credit Review Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateReviewRequest {
    pub profile_id: Uuid,
    pub review_type: String,
    pub recommended_credit_limit: Option<String>,
    pub due_date: Option<chrono::NaiveDate>,
}

pub async fn create_review(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateReviewRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.credit_management_engine.create_review(
        org_id, payload.profile_id, &payload.review_type,
        None, payload.recommended_credit_limit.as_deref(),
        None, None, payload.due_date, Some(user_id),
    ).await {
        Ok(review) => Ok((StatusCode::CREATED, Json(serde_json::to_value(review).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => {
            error!("Failed to create review: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListReviewsQuery {
    pub status: Option<String>,
    pub profile_id: Option<Uuid>,
}

pub async fn list_reviews(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<ListReviewsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.credit_management_engine.list_reviews(org_id, params.status.as_deref(), params.profile_id).await {
        Ok(reviews) => Ok(Json(serde_json::json!({"data": reviews}))),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn start_review(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.credit_management_engine.start_review(id).await {
        Ok(r) => Ok(Json(serde_json::to_value(r).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => {
            error!("Error: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CompleteReviewRequest {
    pub new_score: Option<String>,
    pub new_rating: Option<String>,
    pub approved_credit_limit: Option<String>,
    pub findings: Option<String>,
    pub recommendations: Option<String>,
    pub reviewer_name: Option<String>,
}

pub async fn complete_review(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<CompleteReviewRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).ok();

    match state.credit_management_engine.complete_review(
        id, payload.new_score.as_deref(), payload.new_rating.as_deref(),
        payload.approved_credit_limit.as_deref(), payload.findings.as_deref(),
        payload.recommendations.as_deref(), user_id, payload.reviewer_name.as_deref(),
    ).await {
        Ok(r) => Ok(Json(serde_json::to_value(r).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => {
            error!("Error: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn approve_review(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).ok();

    match state.credit_management_engine.approve_review(id, user_id, None).await {
        Ok(r) => Ok(Json(serde_json::to_value(r).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => {
            error!("Error: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct RejectReviewRequest {
    pub reason: String,
}

pub async fn reject_review(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<RejectReviewRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.credit_management_engine.reject_review(id, Some(&payload.reason)).await {
        Ok(r) => Ok(Json(serde_json::to_value(r).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => {
            error!("Error: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn cancel_review(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.credit_management_engine.cancel_review(id).await {
        Ok(r) => Ok(Json(serde_json::to_value(r).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => {
            error!("Error: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// ============================================================================
// Dashboard
// ============================================================================

pub async fn get_credit_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.credit_management_engine.get_dashboard(org_id).await {
        Ok(dashboard) => Ok(Json(serde_json::to_value(dashboard).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}
