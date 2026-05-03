//! Receipt Write-Off Handlers
//!
//! Oracle Fusion: Receivables > Receipt Write-Off

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
pub struct CreateReasonRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub write_off_type: String,
    pub requires_approval: Option<bool>,
    pub max_auto_approve_amount: Option<String>,
    pub gl_account_code: Option<String>,
}

pub async fn create_reason(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateReasonRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.receipt_write_off_engine.create_reason(
        org_id, &payload.code, &payload.name,
        payload.description.as_deref(),
        payload.gl_account_code.as_deref(),
        payload.requires_approval.unwrap_or(true),
        payload.max_auto_approve_amount.as_deref(), Some(user_id),
    ).await {
        Ok(r) => Ok((StatusCode::CREATED, Json(serde_json::to_value(r).unwrap()))),
        Err(e) => {
            error!("Failed to create write-off reason: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn list_reasons(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.receipt_write_off_engine.list_reasons(org_id).await {
        Ok(items) => Ok(Json(serde_json::json!({ "data": items }))),
        Err(e) => { error!("Failed to list write-off reasons: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateWriteOffRequestReq {
    pub receipt_id: Uuid,
    pub receipt_number: String,
    pub customer_id: Option<Uuid>,
    pub customer_number: Option<String>,
    pub write_off_amount: String,
    pub currency_code: String,
    pub reason_id: Uuid,
    pub comments: Option<String>,
}

pub async fn create_write_off_request(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateWriteOffRequestReq>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.receipt_write_off_engine.create_request(
        org_id, payload.receipt_id, &payload.receipt_number,
        payload.customer_id, payload.customer_number.as_deref(),
        &payload.write_off_amount, &payload.currency_code,
        payload.reason_id, payload.comments.as_deref(), Some(user_id),
    ).await {
        Ok(r) => Ok((StatusCode::CREATED, Json(serde_json::to_value(r).unwrap()))),
        Err(e) => {
            error!("Failed to create write-off request: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListWriteOffRequestsQuery {
    pub status: Option<String>,
}

pub async fn list_write_off_requests(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListWriteOffRequestsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.receipt_write_off_engine.list_requests(org_id, query.status.as_deref(), None).await {
        Ok(items) => Ok(Json(serde_json::json!({ "data": items }))),
        Err(e) => { error!("Failed to list write-off requests: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn approve_write_off_request(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.receipt_write_off_engine.approve_request(id, user_id).await {
        Ok(r) => Ok(Json(serde_json::to_value(r).unwrap())),
        Err(e) => {
            error!("Failed to approve write-off request: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) | atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn post_write_off_request(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.receipt_write_off_engine.post_request(id, user_id, None).await {
        Ok(r) => Ok(Json(serde_json::to_value(r).unwrap())),
        Err(e) => {
            error!("Failed to post write-off request: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) | atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn get_write_off_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.receipt_write_off_engine.get_dashboard(org_id).await {
        Ok(dash) => Ok(Json(serde_json::to_value(dash).unwrap())),
        Err(e) => { error!("Failed to get write-off dashboard: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}
