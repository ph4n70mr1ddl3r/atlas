//! Payment Terms Handlers
//!
//! Oracle Fusion: Financials > Payment Terms Management

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
pub struct CreateTermRequest {
    pub term_code: String,
    pub name: String,
    pub description: Option<String>,
    pub base_due_days: i32,
    pub due_date_cutoff_day: Option<i32>,
    pub term_type: String,
    pub default_discount_percent: String,
}

pub async fn create_term(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateTermRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.payment_terms_engine.create_term(
        org_id, &payload.term_code, &payload.name, payload.description.as_deref(),
        payload.base_due_days, payload.due_date_cutoff_day, &payload.term_type,
        &payload.default_discount_percent, Some(user_id),
    ).await {
        Ok(t) => Ok((StatusCode::CREATED, Json(serde_json::to_value(t).unwrap()))),
        Err(e) => {
            error!("Failed to create payment term: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListTermsQuery { pub status: Option<String> }

pub async fn list_terms(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListTermsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.payment_terms_engine.list_terms(org_id, query.status.as_deref()).await {
        Ok(terms) => Ok(Json(serde_json::json!({ "data": terms }))),
        Err(e) => { error!("Failed to list payment terms: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn get_term(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.payment_terms_engine.get_term_by_id(id).await {
        Ok(Some(t)) => Ok(Json(serde_json::to_value(t).unwrap())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Failed to get payment term: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn activate_term(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.payment_terms_engine.activate_term(id).await {
        Ok(t) => Ok(Json(serde_json::to_value(t).unwrap())),
        Err(e) => {
            error!("Failed to activate payment term: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn deactivate_term(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.payment_terms_engine.deactivate_term(id).await {
        Ok(t) => Ok(Json(serde_json::to_value(t).unwrap())),
        Err(e) => {
            error!("Failed to deactivate payment term: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn delete_term(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    match state.payment_terms_engine.delete_term(id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            error!("Failed to delete payment term: {}", e);
            Err(match e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// Discount Schedules
#[derive(Debug, Deserialize)]
pub struct CreateDiscountScheduleRequest {
    pub discount_percent: String,
    pub discount_days: i32,
    pub discount_day_of_month: Option<i32>,
    pub discount_basis: String,
    pub display_order: Option<i32>,
}

pub async fn create_discount_schedule(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(term_id): Path<Uuid>,
    Json(payload): Json<CreateDiscountScheduleRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.payment_terms_engine.create_discount_schedule(
        org_id, term_id, &payload.discount_percent, payload.discount_days,
        payload.discount_day_of_month, &payload.discount_basis,
        payload.display_order.unwrap_or(0),
    ).await {
        Ok(ds) => Ok((StatusCode::CREATED, Json(serde_json::to_value(ds).unwrap()))),
        Err(e) => {
            error!("Failed to create discount schedule: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn list_discount_schedules(
    State(state): State<Arc<AppState>>,
    Path(term_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.payment_terms_engine.list_discount_schedules(term_id).await {
        Ok(schedules) => Ok(Json(serde_json::json!({ "data": schedules }))),
        Err(e) => { error!("Failed to list discount schedules: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn delete_discount_schedule(
    State(state): State<Arc<AppState>>,
    Path((_term_id, schedule_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, StatusCode> {
    match state.payment_terms_engine.delete_discount_schedule(schedule_id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => { error!("Failed to delete discount schedule: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// Installments
#[derive(Debug, Deserialize)]
pub struct CreateInstallmentRequest {
    pub installment_number: i32,
    pub due_days_offset: i32,
    pub percentage: String,
    pub discount_percent: String,
    pub discount_days: i32,
}

pub async fn create_installment(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(term_id): Path<Uuid>,
    Json(payload): Json<CreateInstallmentRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.payment_terms_engine.create_installment(
        org_id, term_id, payload.installment_number, payload.due_days_offset,
        &payload.percentage, &payload.discount_percent, payload.discount_days,
    ).await {
        Ok(inst) => Ok((StatusCode::CREATED, Json(serde_json::to_value(inst).unwrap()))),
        Err(e) => {
            error!("Failed to create installment: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn list_installments(
    State(state): State<Arc<AppState>>,
    Path(term_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.payment_terms_engine.list_installments(term_id).await {
        Ok(installments) => Ok(Json(serde_json::json!({ "data": installments }))),
        Err(e) => { error!("Failed to list installments: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn delete_installment(
    State(state): State<Arc<AppState>>,
    Path((_term_id, installment_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, StatusCode> {
    match state.payment_terms_engine.delete_installment(installment_id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => { error!("Failed to delete installment: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// Dashboard
pub async fn get_payment_terms_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.payment_terms_engine.get_dashboard(org_id).await {
        Ok(dashboard) => Ok(Json(serde_json::to_value(dashboard).unwrap())),
        Err(e) => { error!("Failed to get dashboard: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}
