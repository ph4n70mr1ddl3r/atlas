//! Approval Delegation API Handlers
//!
//! Oracle Fusion Cloud BPM: Worklist > Rules > Configure Delegation
//!
//! Endpoints for managing approval delegation rules where users
//! proactively delegate their approval authority to another user
//! for a specified date range.

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
use atlas_shared::CreateDelegationRuleRequest;

// ============================================================================
// Query Parameters
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ListRulesQuery {
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListHistoryQuery {
    pub limit: Option<i64>,
}

// ============================================================================
// Delegation Rule CRUD Handlers
// ============================================================================

pub async fn create_delegation_rule(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateDelegationRuleRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let delegator_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.approval_delegation_engine.create_rule(org_id, delegator_id, payload).await {
        Ok(rule) => Ok((StatusCode::CREATED, Json(serde_json::to_value(rule).unwrap()))),
        Err(e) => {
            error!("Failed to create delegation rule: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                409 => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn get_delegation_rule(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let id = Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;

    match state.approval_delegation_engine.get_rule(id).await {
        Ok(Some(rule)) => Ok(Json(serde_json::to_value(rule).unwrap())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get delegation rule {}: {}", id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn list_delegation_rules(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<ListRulesQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.approval_delegation_engine.list_rules(
        org_id,
        params.status.as_deref(),
    ).await {
        Ok(rules) => Ok(Json(serde_json::to_value(rules).unwrap())),
        Err(e) => {
            error!("Failed to list delegation rules: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn list_my_delegation_rules(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<ListRulesQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let delegator_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.approval_delegation_engine.list_rules_for_delegator(
        org_id,
        delegator_id,
        params.status.as_deref(),
    ).await {
        Ok(rules) => Ok(Json(serde_json::to_value(rules).unwrap())),
        Err(e) => {
            error!("Failed to list my delegation rules: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// ============================================================================
// Delegation Rule Lifecycle Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CancelRuleRequest {
    pub reason: Option<String>,
}

pub async fn cancel_delegation_rule(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<String>,
    Json(payload): Json<CancelRuleRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let id = Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Authorization: only the delegator or an admin can cancel
    let rule = state.approval_delegation_engine.get_rule(id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    let is_admin = claims.roles.contains(&"admin".to_string());
    if rule.delegator_id != user_id && !is_admin {
        return Err(StatusCode::FORBIDDEN);
    }

    match state.approval_delegation_engine.cancel_rule(id, user_id, payload.reason.as_deref()).await {
        Ok(rule) => Ok(Json(serde_json::to_value(rule).unwrap())),
        Err(e) => {
            error!("Failed to cancel delegation rule {}: {}", id, e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn activate_delegation_rule(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let id = Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Authorization: only the delegator or an admin can activate
    let rule = state.approval_delegation_engine.get_rule(id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    let is_admin = claims.roles.contains(&"admin".to_string());
    if rule.delegator_id != user_id && !is_admin {
        return Err(StatusCode::FORBIDDEN);
    }

    match state.approval_delegation_engine.activate_rule(id).await {
        Ok(rule) => Ok(Json(serde_json::to_value(rule).unwrap())),
        Err(e) => {
            error!("Failed to activate delegation rule {}: {}", id, e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn delete_delegation_rule(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let id = Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Authorization: only the delegator or an admin can delete
    let rule = state.approval_delegation_engine.get_rule(id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    let is_admin = claims.roles.contains(&"admin".to_string());
    if rule.delegator_id != user_id && !is_admin {
        return Err(StatusCode::FORBIDDEN);
    }

    match state.approval_delegation_engine.delete_rule(id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            error!("Failed to delete delegation rule {}: {}", id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// ============================================================================
// Process Scheduled Rules (Admin/Cron)
// ============================================================================

pub async fn process_scheduled_delegations(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.approval_delegation_engine.process_scheduled_rules().await {
        Ok((activated, expired)) => Ok(Json(serde_json::json!({
            "activated": activated.len(),
            "expired": expired.len(),
            "activated_rule_ids": activated,
            "expired_rule_ids": expired,
        }))),
        Err(e) => {
            error!("Failed to process scheduled delegations: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// ============================================================================
// Delegation History
// ============================================================================

pub async fn list_delegation_history(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<ListHistoryQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.approval_delegation_engine.list_delegation_history(
        org_id, user_id, params.limit,
    ).await {
        Ok(history) => Ok(Json(serde_json::to_value(history).unwrap())),
        Err(e) => {
            error!("Failed to list delegation history: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// ============================================================================
// Delegation Dashboard
// ============================================================================

pub async fn get_delegation_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.approval_delegation_engine.get_dashboard(org_id).await {
        Ok(dashboard) => Ok(Json(serde_json::to_value(dashboard).unwrap())),
        Err(e) => {
            error!("Failed to get delegation dashboard: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
