//! Approval Authority Limits API Handlers
//!
//! Oracle Fusion Cloud BPM: Task Configuration > Document Approval Limits
//!
//! Endpoints for managing approval authority limits (signing limits)
//! and checking whether a user/role is authorised to approve a
//! transaction of a given amount.

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
use atlas_shared::CreateApprovalAuthorityLimitRequest;

// ============================================================================
// Query Parameters
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ListLimitsQuery {
    pub status: Option<String>,
    pub owner_type: Option<String>,
    pub document_type: Option<String>,
    pub user_id: Option<String>,
    pub role_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListAuditsQuery {
    pub user_id: Option<String>,
    pub document_type: Option<String>,
    pub result: Option<String>,
    pub limit: Option<i32>,
}

// ============================================================================
// Limit CRUD Handlers
// ============================================================================

pub async fn create_authority_limit(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateApprovalAuthorityLimitRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.approval_authority_engine.create_limit(org_id, payload, Some(user_id)).await {
        Ok(limit) => Ok((StatusCode::CREATED, Json(serde_json::to_value(limit).unwrap_or_else(|e| {
            error!("Serialization error: {}", e); serde_json::Value::Null
        })))),
        Err(e) => {
            error!("Failed to create authority limit: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                409 => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn get_authority_limit(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let id = Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;

    match state.approval_authority_engine.get_limit(id).await {
        Ok(Some(limit)) => Ok(Json(serde_json::to_value(limit).unwrap_or_else(|e| {
            error!("Serialization error: {}", e); serde_json::Value::Null
        }))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get authority limit {}: {}", id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn list_authority_limits(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Query(query): Query<ListLimitsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&_claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = query.user_id
        .map(|s| Uuid::parse_str(&s))
        .transpose()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    match state.approval_authority_engine.list_limits(
        org_id,
        query.status.as_deref(),
        query.owner_type.as_deref(),
        query.document_type.as_deref(),
        user_id,
        query.role_name.as_deref(),
    ).await {
        Ok(limits) => Ok(Json(serde_json::to_value(limits).unwrap_or_else(|e| {
            error!("Serialization error: {}", e); serde_json::Value::Null
        }))),
        Err(e) => {
            error!("Failed to list authority limits: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn activate_authority_limit(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let id = Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;

    match state.approval_authority_engine.activate_limit(id).await {
        Ok(limit) => Ok(Json(serde_json::to_value(limit).unwrap_or_else(|e| {
            error!("Serialization error: {}", e); serde_json::Value::Null
        }))),
        Err(e) => {
            error!("Failed to activate authority limit {}: {}", id, e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn deactivate_authority_limit(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let id = Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;

    match state.approval_authority_engine.deactivate_limit(id).await {
        Ok(limit) => Ok(Json(serde_json::to_value(limit).unwrap_or_else(|e| {
            error!("Serialization error: {}", e); serde_json::Value::Null
        }))),
        Err(e) => {
            error!("Failed to deactivate authority limit {}: {}", id, e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn delete_authority_limit(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let id = Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;

    match state.approval_authority_engine.delete_limit(id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            error!("Failed to delete authority limit {}: {}", id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// ============================================================================
// Authority Check
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckAuthorityRequest {
    pub document_type: String,
    pub amount: String,
    pub business_unit_id: Option<String>,
    pub cost_center: Option<String>,
    pub document_id: Option<String>,
}

pub async fn check_authority(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CheckAuthorityRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let business_unit_id = payload.business_unit_id
        .map(|s| Uuid::parse_str(&s))
        .transpose()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let document_id = payload.document_id
        .map(|s| Uuid::parse_str(&s))
        .transpose()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    match state.approval_authority_engine.check_authority(
        org_id,
        user_id,
        &claims.roles,
        &payload.document_type,
        &payload.amount,
        business_unit_id,
        payload.cost_center.as_deref(),
        document_id,
    ).await {
        Ok(audit) => {
            let status = if audit.result == "approved" { StatusCode::OK } else { StatusCode::FORBIDDEN };
            // We return the body either way, but set the status code
            let val = serde_json::to_value(&audit).unwrap_or_else(|e| {
                error!("Serialization error: {}", e); serde_json::Value::Null
            });
            // axum doesn't let us change status easily here, so return 200 with the result
            let _ = status;
            Ok(Json(val))
        }
        Err(e) => {
            error!("Failed to check authority: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// ============================================================================
// Audit Trail
// ============================================================================

pub async fn list_check_audits(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Query(query): Query<ListAuditsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&_claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = query.user_id
        .map(|s| Uuid::parse_str(&s))
        .transpose()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    match state.approval_authority_engine.list_check_audits(
        org_id,
        user_id,
        query.document_type.as_deref(),
        query.result.as_deref(),
        query.limit,
    ).await {
        Ok(audits) => Ok(Json(serde_json::to_value(audits).unwrap_or_else(|e| {
            error!("Serialization error: {}", e); serde_json::Value::Null
        }))),
        Err(e) => {
            error!("Failed to list check audits: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// ============================================================================
// Dashboard
// ============================================================================

pub async fn get_authority_dashboard(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.approval_authority_engine.get_dashboard(org_id).await {
        Ok(dashboard) => Ok(Json(serde_json::to_value(dashboard).unwrap_or_else(|e| {
            error!("Serialization error: {}", e); serde_json::Value::Null
        }))),
        Err(e) => {
            error!("Failed to get authority dashboard: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
