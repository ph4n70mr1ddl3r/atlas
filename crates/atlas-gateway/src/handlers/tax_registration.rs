//! Tax Registration Management API Handlers
//!
//! REST endpoints for Oracle Fusion-inspired Tax Registration Management.
//! Manages taxpayer identification numbers (TIN, VAT, GST, EIN, etc.)
//! across jurisdictions with validation, status lifecycle, and compliance.
//!
//! Oracle Fusion equivalent: Financials > Tax > Tax Registrations

use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use tracing::error;
use uuid::Uuid;

use crate::handlers::auth::{parse_uuid, Claims};
use crate::AppState;

// ============================================================================
// Request / Response Types
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTaxRegistrationRequest {
    pub registration_number: String,
    pub registration_type: String,
    pub tax_purpose: Option<String>,
    pub party_type: String,
    pub party_id: Option<Uuid>,
    pub party_name: Option<String>,
    pub jurisdiction_code: String,
    pub country_code: String,
    pub state_code: Option<String>,
    pub effective_from: String,
    pub effective_to: Option<String>,
    pub is_default: Option<bool>,
    pub reporting_name: Option<String>,
    pub legal_entity_id: Option<Uuid>,
    pub source: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListTaxRegistrationsQuery {
    pub party_type: Option<String>,
    pub status: Option<String>,
    pub jurisdiction_code: Option<String>,
    pub country_code: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeregisterRequest {
    pub deregistration_date: Option<String>,
    pub reason: Option<String>,
}

// ============================================================================
// CRUD Handlers
// ============================================================================

pub async fn create_registration(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreateTaxRegistrationRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    let user_id = parse_uuid(&claims.sub).ok();

    let effective_from = chrono::NaiveDate::parse_from_str(&req.effective_from, "%Y-%m-%d")
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": format!("Invalid effective_from date: {}", e)
                })),
            )
        })?;

    let effective_to = match req.effective_to.as_deref() {
        Some(d) => Some(
            chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").map_err(|e| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "error": format!("Invalid effective_to date: {}", e)
                    })),
                )
            })?,
        ),
        None => None,
    };

    match state
        .financials
        .tax_registration_engine
        .create_registration(
            org_id,
            &req.registration_number,
            &req.registration_type,
            req.tax_purpose.as_deref().unwrap_or("both"),
            &req.party_type,
            req.party_id,
            req.party_name.as_deref(),
            &req.jurisdiction_code,
            &req.country_code,
            req.state_code.as_deref(),
            effective_from,
            effective_to,
            req.is_default.unwrap_or(false),
            req.reporting_name.as_deref(),
            req.legal_entity_id,
            req.source.as_deref().unwrap_or("manual"),
            user_id,
        )
        .await
    {
        Ok(reg) => Ok((
            StatusCode::CREATED,
            Json(serde_json::to_value(reg).unwrap_or(serde_json::Value::Null)),
        )),
        Err(e) => {
            error!("Failed to create tax registration: {}", e);
            Err((
                StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(serde_json::json!({"error": e.to_string()})),
            ))
        }
    }
}

pub async fn list_registrations(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<ListTaxRegistrationsQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;

    match state
        .financials
        .tax_registration_engine
        .list_registrations(
            org_id,
            query.party_type.as_deref(),
            query.status.as_deref(),
            query.jurisdiction_code.as_deref(),
            query.country_code.as_deref(),
        )
        .await
    {
        Ok(regs) => Ok(Json(serde_json::json!({"data": regs}))),
        Err(e) => {
            error!("Failed to list tax registrations: {}", e);
            Err((
                StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(serde_json::json!({"error": e.to_string()})),
            ))
        }
    }
}

pub async fn get_registration(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let _org_id = parse_uuid(&claims.org_id)?;

    match state
        .financials
        .tax_registration_engine
        .get_registration(id)
        .await
    {
        Ok(Some(reg)) => Ok(Json(
            serde_json::to_value(reg).unwrap_or(serde_json::Value::Null),
        )),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Tax registration not found"})),
        )),
        Err(e) => {
            error!("Failed to get tax registration: {}", e);
            Err((
                StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(serde_json::json!({"error": e.to_string()})),
            ))
        }
    }
}

pub async fn get_registration_by_number(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(number): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;

    match state
        .financials
        .tax_registration_engine
        .get_registration_by_number(org_id, &number)
        .await
    {
        Ok(Some(reg)) => Ok(Json(
            serde_json::to_value(reg).unwrap_or(serde_json::Value::Null),
        )),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Tax registration not found"})),
        )),
        Err(e) => {
            error!("Failed to get tax registration by number: {}", e);
            Err((
                StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(serde_json::json!({"error": e.to_string()})),
            ))
        }
    }
}

// ============================================================================
// Status Lifecycle Handlers
// ============================================================================

pub async fn activate_registration(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let _org_id = parse_uuid(&claims.org_id)?;

    match state
        .financials
        .tax_registration_engine
        .activate_registration(id)
        .await
    {
        Ok(reg) => Ok(Json(
            serde_json::to_value(reg).unwrap_or(serde_json::Value::Null),
        )),
        Err(e) => {
            error!("Failed to activate tax registration: {}", e);
            Err((
                StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(serde_json::json!({"error": e.to_string()})),
            ))
        }
    }
}

pub async fn suspend_registration(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let _org_id = parse_uuid(&claims.org_id)?;

    match state
        .financials
        .tax_registration_engine
        .suspend_registration(id)
        .await
    {
        Ok(reg) => Ok(Json(
            serde_json::to_value(reg).unwrap_or(serde_json::Value::Null),
        )),
        Err(e) => {
            error!("Failed to suspend tax registration: {}", e);
            Err((
                StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(serde_json::json!({"error": e.to_string()})),
            ))
        }
    }
}

pub async fn reactivate_registration(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let _org_id = parse_uuid(&claims.org_id)?;

    match state
        .financials
        .tax_registration_engine
        .reactivate_registration(id)
        .await
    {
        Ok(reg) => Ok(Json(
            serde_json::to_value(reg).unwrap_or(serde_json::Value::Null),
        )),
        Err(e) => {
            error!("Failed to reactivate tax registration: {}", e);
            Err((
                StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(serde_json::json!({"error": e.to_string()})),
            ))
        }
    }
}

pub async fn deregister(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(req): Json<DeregisterRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let _org_id = parse_uuid(&claims.org_id)?;

    let dereg_date = match req.deregistration_date.as_deref() {
        Some(d) => chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": format!("Invalid deregistration_date: {}", e)
                })),
            )
        })?,
        None => chrono::Utc::now().date_naive(),
    };

    match state
        .financials
        .tax_registration_engine
        .deregister(id, dereg_date)
        .await
    {
        Ok(reg) => Ok(Json(
            serde_json::to_value(reg).unwrap_or(serde_json::Value::Null),
        )),
        Err(e) => {
            error!("Failed to deregister tax registration: {}", e);
            Err((
                StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(serde_json::json!({"error": e.to_string()})),
            ))
        }
    }
}

// ============================================================================
// Validation & Dashboard
// ============================================================================

pub async fn validate_registration(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let _org_id = parse_uuid(&claims.org_id)?;

    match state
        .financials
        .tax_registration_engine
        .validate_registration(id)
        .await
    {
        Ok(reg) => Ok(Json(
            serde_json::to_value(reg).unwrap_or(serde_json::Value::Null),
        )),
        Err(e) => {
            error!("Failed to validate tax registration: {}", e);
            Err((
                StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(serde_json::json!({"error": e.to_string()})),
            ))
        }
    }
}

pub async fn get_summary(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;

    match state
        .financials
        .tax_registration_engine
        .get_summary(org_id)
        .await
    {
        Ok(summary) => Ok(Json(
            serde_json::to_value(summary).unwrap_or(serde_json::Value::Null),
        )),
        Err(e) => {
            error!("Failed to get tax registration summary: {}", e);
            Err((
                StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(serde_json::json!({"error": e.to_string()})),
            ))
        }
    }
}
