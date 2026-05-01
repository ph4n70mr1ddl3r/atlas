//! Funds Reservation & Budgetary Control Handlers
//!
//! Oracle Fusion Cloud: Financials > Budgetary Control > Funds Reservation
//! Provides HTTP endpoints for:
//! - Fund reservation CRUD
//! - Fund consumption and release
//! - Fund availability checks
//! - Reservation lines management
//! - Budgetary control dashboard

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
// Fund Reservations
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateReservationRequest {
    pub reservation_number: String,
    pub budget_id: Uuid,
    pub budget_code: String,
    pub budget_version_id: Option<Uuid>,
    pub description: Option<String>,
    pub source_type: Option<String>,
    pub source_id: Option<Uuid>,
    pub source_number: Option<String>,
    pub reserved_amount: f64,
    pub currency_code: String,
    pub reservation_date: chrono::NaiveDate,
    pub expiry_date: Option<chrono::NaiveDate>,
    pub control_level: String,
    pub fiscal_year: Option<i32>,
    pub period_name: Option<String>,
    pub department_id: Option<Uuid>,
    pub department_name: Option<String>,
}

pub async fn create_reservation(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateReservationRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let reservation = state.funds_reservation_engine
        .create_reservation(
            org_id,
            &payload.reservation_number,
            payload.budget_id,
            &payload.budget_code,
            payload.budget_version_id,
            payload.description.as_deref(),
            payload.source_type.as_deref(),
            payload.source_id,
            payload.source_number.as_deref(),
            payload.reserved_amount,
            &payload.currency_code,
            payload.reservation_date,
            payload.expiry_date,
            &payload.control_level,
            payload.fiscal_year,
            payload.period_name.as_deref(),
            payload.department_id,
            payload.department_name.as_deref(),
            Some(user_id),
        )
        .await
        .map_err(|e| {
            tracing::error!("Create reservation error: {}", e);
            match e {
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(reservation).unwrap_or_default()),
    ))
}

pub async fn get_reservation(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let reservation = state.funds_reservation_engine
        .get_reservation(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match reservation {
        Some(r) => Ok(Json(serde_json::to_value(r).unwrap_or_default())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn get_reservation_by_number(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(number): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let reservation = state.funds_reservation_engine
        .get_reservation_by_number(org_id, &number)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match reservation {
        Some(r) => Ok(Json(serde_json::to_value(r).unwrap_or_default())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

#[derive(Debug, Deserialize)]
pub struct ListReservationsQuery {
    pub status: Option<String>,
    pub budget_id: Option<Uuid>,
    pub department_id: Option<Uuid>,
}

pub async fn list_reservations(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListReservationsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let reservations = state.funds_reservation_engine
        .list_reservations(
            org_id,
            query.status.as_deref(),
            query.budget_id.as_ref(),
            query.department_id.as_ref(),
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({ "data": reservations })))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsumeRequest {
    pub consume_amount: f64,
}

pub async fn consume_reservation(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<ConsumeRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let reservation = state.funds_reservation_engine
        .consume_reservation(id, payload.consume_amount)
        .await
        .map_err(|e| {
            tracing::error!("Consume reservation error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;
    Ok(Json(serde_json::to_value(reservation).unwrap_or_default()))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseRequest {
    pub release_amount: f64,
}

pub async fn release_reservation(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<ReleaseRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let reservation = state.funds_reservation_engine
        .release_reservation(id, payload.release_amount)
        .await
        .map_err(|e| {
            tracing::error!("Release reservation error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;
    Ok(Json(serde_json::to_value(reservation).unwrap_or_default()))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelRequest {
    pub reason: Option<String>,
}

pub async fn cancel_reservation(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<CancelRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let reservation = state.funds_reservation_engine
        .cancel_reservation(id, Some(user_id), payload.reason.as_deref())
        .await
        .map_err(|e| {
            tracing::error!("Cancel reservation error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;
    Ok(Json(serde_json::to_value(reservation).unwrap_or_default()))
}

pub async fn delete_reservation(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(number): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    state.funds_reservation_engine
        .delete_reservation(org_id, &number)
        .await
        .map_err(|e| {
            tracing::error!("Delete reservation error: {}", e);
            match e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Fund Reservation Lines
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateReservationLineRequest {
    pub line_number: i32,
    pub account_code: String,
    pub account_description: Option<String>,
    pub budget_line_id: Option<Uuid>,
    pub department_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub cost_center: Option<String>,
    pub reserved_amount: f64,
}

pub async fn create_reservation_line(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(reservation_id): Path<Uuid>,
    Json(payload): Json<CreateReservationLineRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let line = state.funds_reservation_engine
        .create_reservation_line(
            org_id,
            reservation_id,
            payload.line_number,
            &payload.account_code,
            payload.account_description.as_deref(),
            payload.budget_line_id,
            payload.department_id,
            payload.project_id,
            payload.cost_center.as_deref(),
            payload.reserved_amount,
        )
        .await
        .map_err(|e| {
            tracing::error!("Create reservation line error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(line).unwrap_or_default()),
    ))
}

pub async fn list_reservation_lines(
    State(state): State<Arc<AppState>>,
    Path(reservation_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let lines = state.funds_reservation_engine
        .list_reservation_lines(reservation_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({ "data": lines })))
}

// ============================================================================
// Fund Availability Check
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct FundAvailabilityQuery {
    pub budget_id: Uuid,
    pub account_code: String,
    pub as_of_date: chrono::NaiveDate,
    pub fiscal_year: Option<i32>,
    pub period_name: Option<String>,
}

pub async fn check_fund_availability(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<FundAvailabilityQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let availability = state.funds_reservation_engine
        .check_fund_availability(
            org_id,
            query.budget_id,
            &query.account_code,
            query.as_of_date,
            query.fiscal_year,
            query.period_name.as_deref(),
        )
        .await
        .map_err(|e| {
            tracing::error!("Check fund availability error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(Json(serde_json::to_value(availability).unwrap_or_default()))
}

// ============================================================================
// Dashboard
// ============================================================================

pub async fn get_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let dashboard = state.funds_reservation_engine
        .get_dashboard(org_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::to_value(dashboard).unwrap_or_default()))
}
