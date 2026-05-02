//! Financial Controls Handlers
//!
//! Oracle Fusion: Financials > Financial Controls

use axum::{
    extract::{State, Path},
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
pub struct CreateControlRuleRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub category: String,
    pub risk_level: String,
    pub control_type: String,
    pub conditions: serde_json::Value,
    pub threshold_value: Option<String>,
    pub target_entity: String,
    pub target_fields: serde_json::Value,
    pub actions: serde_json::Value,
    pub auto_resolve: Option<bool>,
    pub check_schedule: String,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
}

pub async fn create_control_rule(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateControlRuleRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.financial_controls_engine.create_rule(
        org_id, &payload.code, &payload.name, payload.description.as_deref(),
        &payload.category, &payload.risk_level, &payload.control_type,
        payload.conditions.clone(), payload.threshold_value.as_deref(),
        &payload.target_entity, payload.target_fields.clone(),
        payload.actions.clone(), payload.auto_resolve.unwrap_or(false),
        &payload.check_schedule, payload.effective_from, payload.effective_to,
        Some(user_id),
    ).await {
        Ok(r) => Ok((StatusCode::CREATED, Json(serde_json::to_value(r).unwrap()))),
        Err(e) => {
            error!("Failed to create control rule: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn list_control_rules(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.financial_controls_engine.list_rules(org_id, None, None).await {
        Ok(rules) => Ok(Json(serde_json::json!({ "data": rules }))),
        Err(e) => { error!("Failed to list rules: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn delete_control_rule(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(code): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.financial_controls_engine.delete_rule(org_id, &code).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            error!("Failed to delete rule: {}", e);
            Err(match e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn get_financial_controls_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.financial_controls_engine.get_dashboard_summary(org_id).await {
        Ok(dashboard) => Ok(Json(serde_json::to_value(dashboard).unwrap())),
        Err(e) => { error!("Failed to get dashboard: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}
