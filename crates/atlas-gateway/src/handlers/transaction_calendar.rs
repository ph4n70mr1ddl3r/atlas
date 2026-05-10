//! Transaction Calendar Handlers
//!
//! Oracle Fusion Cloud ERP: General Ledger > Setup > Transaction Calendars
//!
//! API endpoints for transaction calendar management:
//! - Calendar CRUD and lifecycle (create, list, get, activate, deactivate, delete)
//! - Exception management (holidays, non-working days, special working days)
//! - Business day calculations (next/previous business day, add business days, is business day)
//! - Dashboard summary

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
// Calendar Management
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCalendarRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub working_days: Option<serde_json::Value>,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
}

pub async fn create_calendar(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateCalendarRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.transaction_calendar_engine.create_calendar(
        org_id,
        &payload.code,
        &payload.name,
        payload.description.as_deref(),
        payload.working_days.as_ref(),
        payload.effective_from,
        payload.effective_to,
        Some(user_id),
    ).await {
        Ok(cal) => Ok((StatusCode::CREATED, Json(serde_json::to_value(cal).unwrap_or_default()))),
        Err(e) => {
            error!("Failed to create transaction calendar: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(msg) => {
                    return Ok((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": msg}))));
                }
                atlas_shared::AtlasError::Conflict(msg) => {
                    return Ok((StatusCode::CONFLICT, Json(serde_json::json!({"error": msg}))));
                }
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListCalendarsQuery {
    pub status: Option<String>,
}

pub async fn list_calendars(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListCalendarsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.transaction_calendar_engine.list_calendars(
        org_id,
        query.status.as_deref(),
    ).await {
        Ok(calendars) => Ok(Json(serde_json::json!({ "data": calendars }))),
        Err(e) => {
            error!("Failed to list transaction calendars: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_calendar(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.transaction_calendar_engine.get_calendar_by_id(id).await {
        Ok(Some(cal)) => Ok(Json(serde_json::to_value(cal).unwrap_or_default())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get transaction calendar: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_calendar_by_code(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(code): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.transaction_calendar_engine.get_calendar(org_id, &code).await {
        Ok(Some(cal)) => Ok(Json(serde_json::to_value(cal).unwrap_or_default())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get transaction calendar by code: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn activate_calendar(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.transaction_calendar_engine.activate_calendar(id).await {
        Ok(cal) => Ok(Json(serde_json::to_value(cal).unwrap_or_default())),
        Err(e) => {
            error!("Failed to activate transaction calendar: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) | atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn deactivate_calendar(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.transaction_calendar_engine.deactivate_calendar(id).await {
        Ok(cal) => Ok(Json(serde_json::to_value(cal).unwrap_or_default())),
        Err(e) => {
            error!("Failed to deactivate transaction calendar: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) | atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn delete_calendar(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(code): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.transaction_calendar_engine.delete_calendar(org_id, &code).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            error!("Failed to delete transaction calendar: {}", e);
            Err(match e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// ============================================================================
// Exception Management
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateExceptionRequest {
    pub exception_date: chrono::NaiveDate,
    pub exception_type: String,
    pub name: String,
    pub description: Option<String>,
}

pub async fn create_exception(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(calendar_id): Path<Uuid>,
    Json(payload): Json<CreateExceptionRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.transaction_calendar_engine.create_exception(
        org_id,
        calendar_id,
        payload.exception_date,
        &payload.exception_type,
        &payload.name,
        payload.description.as_deref(),
        Some(user_id),
    ).await {
        Ok(exc) => Ok((StatusCode::CREATED, Json(serde_json::to_value(exc).unwrap_or_default()))),
        Err(e) => {
            error!("Failed to create calendar exception: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(msg) => {
                    return Ok((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": msg}))));
                }
                atlas_shared::AtlasError::Conflict(msg) => {
                    return Ok((StatusCode::CONFLICT, Json(serde_json::json!({"error": msg}))));
                }
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn list_exceptions(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(calendar_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.transaction_calendar_engine.list_exceptions(calendar_id).await {
        Ok(exceptions) => Ok(Json(serde_json::json!({ "data": exceptions }))),
        Err(e) => {
            error!("Failed to list calendar exceptions: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListExceptionsRangeQuery {
    pub from_date: chrono::NaiveDate,
    pub to_date: chrono::NaiveDate,
}

pub async fn list_exceptions_range(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(calendar_id): Path<Uuid>,
    Query(query): Query<ListExceptionsRangeQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.transaction_calendar_engine.list_exceptions_range(
        calendar_id, query.from_date, query.to_date,
    ).await {
        Ok(exceptions) => Ok(Json(serde_json::json!({ "data": exceptions }))),
        Err(e) => {
            error!("Failed to list calendar exceptions range: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn delete_exception(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    match state.transaction_calendar_engine.delete_exception(id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            error!("Failed to delete calendar exception: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// ============================================================================
// Business Day Calculations
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IsBusinessDayRequest {
    pub date: chrono::NaiveDate,
    pub reference_type: Option<String>,
    pub reference_id: Option<Uuid>,
}

pub async fn is_business_day(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(calendar_id): Path<Uuid>,
    Json(payload): Json<IsBusinessDayRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.transaction_calendar_engine.is_business_day(
        calendar_id,
        payload.date,
        payload.reference_type.as_deref(),
        payload.reference_id,
        Some(user_id),
    ).await {
        Ok(result) => Ok(Json(serde_json::json!({
            "date": payload.date,
            "isBusinessDay": result,
            "calendarId": calendar_id,
        }))),
        Err(e) => {
            error!("Failed to check business day: {}", e);
            Err(match e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NextBusinessDayRequest {
    pub date: chrono::NaiveDate,
    pub reference_type: Option<String>,
    pub reference_id: Option<Uuid>,
}

pub async fn next_business_day(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(calendar_id): Path<Uuid>,
    Json(payload): Json<NextBusinessDayRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.transaction_calendar_engine.next_business_day(
        calendar_id,
        payload.date,
        payload.reference_type.as_deref(),
        payload.reference_id,
        Some(user_id),
    ).await {
        Ok(result) => Ok(Json(serde_json::json!({
            "inputDate": payload.date,
            "nextBusinessDay": result,
            "calendarId": calendar_id,
        }))),
        Err(e) => {
            error!("Failed to calculate next business day: {}", e);
            Err(match e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviousBusinessDayRequest {
    pub date: chrono::NaiveDate,
    pub reference_type: Option<String>,
    pub reference_id: Option<Uuid>,
}

pub async fn previous_business_day(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(calendar_id): Path<Uuid>,
    Json(payload): Json<PreviousBusinessDayRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.transaction_calendar_engine.previous_business_day(
        calendar_id,
        payload.date,
        payload.reference_type.as_deref(),
        payload.reference_id,
        Some(user_id),
    ).await {
        Ok(result) => Ok(Json(serde_json::json!({
            "inputDate": payload.date,
            "previousBusinessDay": result,
            "calendarId": calendar_id,
        }))),
        Err(e) => {
            error!("Failed to calculate previous business day: {}", e);
            Err(match e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddBusinessDaysRequest {
    pub start_date: chrono::NaiveDate,
    pub days: i32,
    pub reference_type: Option<String>,
    pub reference_id: Option<Uuid>,
}

pub async fn add_business_days(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(calendar_id): Path<Uuid>,
    Json(payload): Json<AddBusinessDaysRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.transaction_calendar_engine.add_business_days(
        calendar_id,
        payload.start_date,
        payload.days,
        payload.reference_type.as_deref(),
        payload.reference_id,
        Some(user_id),
    ).await {
        Ok(result) => Ok(Json(serde_json::json!({
            "startDate": payload.start_date,
            "businessDaysToAdd": payload.days,
            "resultDate": result,
            "calendarId": calendar_id,
        }))),
        Err(e) => {
            error!("Failed to add business days: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// ============================================================================
// Audit Trail
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListCalculationsQuery {
    pub calendar_id: Option<Uuid>,
    pub limit: Option<i32>,
}

pub async fn list_calculations(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListCalculationsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.transaction_calendar_engine.list_calculations(
        org_id,
        query.calendar_id,
        query.limit,
    ).await {
        Ok(entries) => Ok(Json(serde_json::json!({ "data": entries }))),
        Err(e) => {
            error!("Failed to list calendar calculations: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// ============================================================================
// Dashboard
// ============================================================================

pub async fn get_transaction_calendar_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.transaction_calendar_engine.get_dashboard(org_id).await {
        Ok(summary) => Ok(Json(serde_json::to_value(summary).unwrap_or_default())),
        Err(e) => {
            error!("Failed to get transaction calendar dashboard: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
