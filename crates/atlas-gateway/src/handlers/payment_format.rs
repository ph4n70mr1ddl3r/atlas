//! Payment Format Handlers
//!
//! Oracle Fusion: Payables > Payment Formats

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
pub struct CreatePaymentFormatRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub format_type: String,
    pub payment_method: String,
    pub file_template: Option<String>,
    pub requires_bank_details: Option<bool>,
    pub supports_remittance: Option<bool>,
    pub supports_void: Option<bool>,
    pub max_payments_per_file: Option<i32>,
    pub currency_code: Option<String>,
}

pub async fn create_payment_format(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreatePaymentFormatRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.payment_format_engine.create(
        org_id, &payload.code, &payload.name,
        payload.description.as_deref(), &payload.format_type,
        &payload.payment_method, payload.file_template.as_deref(),
        payload.requires_bank_details.unwrap_or(false),
        payload.supports_remittance.unwrap_or(true),
        payload.supports_void.unwrap_or(true),
        payload.max_payments_per_file,
        payload.currency_code.as_deref().unwrap_or("USD"),
        Some(user_id),
    ).await {
        Ok(f) => Ok((StatusCode::CREATED, Json(serde_json::to_value(f).unwrap()))),
        Err(e) => {
            error!("Failed to create payment format: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn get_payment_format(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.payment_format_engine.get(id).await {
        Ok(Some(f)) => Ok(Json(serde_json::to_value(f).unwrap())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Failed to get payment format: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListPaymentFormatsQuery {
    pub format_type: Option<String>,
    pub payment_method: Option<String>,
    pub is_active: Option<bool>,
}

pub async fn list_payment_formats(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListPaymentFormatsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.payment_format_engine.list(
        org_id, query.format_type.as_deref(),
        query.payment_method.as_deref(), query.is_active,
    ).await {
        Ok(items) => Ok(Json(serde_json::json!({ "data": items }))),
        Err(e) => { error!("Failed to list payment formats: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn deactivate_payment_format(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.payment_format_engine.deactivate(id).await {
        Ok(f) => Ok(Json(serde_json::to_value(f).unwrap())),
        Err(e) => {
            error!("Failed to deactivate payment format: {}", e);
            Err(match e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn activate_payment_format(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.payment_format_engine.activate(id).await {
        Ok(f) => Ok(Json(serde_json::to_value(f).unwrap())),
        Err(e) => {
            error!("Failed to activate payment format: {}", e);
            Err(match e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn get_payment_format_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.payment_format_engine.get_dashboard(org_id).await {
        Ok(dash) => Ok(Json(serde_json::to_value(dash).unwrap())),
        Err(e) => { error!("Failed to get payment format dashboard: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}
