//! Accounting Hub Handlers
//!
//! Oracle Fusion: Financials > Accounting Hub

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
pub struct CreateMappingRuleRequest {
    pub external_system_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub event_type: String,
    pub event_class: String,
    pub priority: Option<i32>,
    pub conditions: serde_json::Value,
    pub field_mappings: serde_json::Value,
    pub accounting_method_id: Option<Uuid>,
    pub stop_on_match: Option<bool>,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
}

pub async fn create_mapping_rule(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateMappingRuleRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.accounting_hub_engine.create_mapping_rule(
        org_id, payload.external_system_id, &payload.code, &payload.name,
        payload.description.as_deref(), &payload.event_type, &payload.event_class,
        payload.priority.unwrap_or(1), payload.conditions.clone(),
        payload.field_mappings.clone(), payload.accounting_method_id,
        payload.stop_on_match.unwrap_or(false),
        payload.effective_from, payload.effective_to, Some(user_id),
    ).await {
        Ok(r) => Ok((StatusCode::CREATED, Json(serde_json::to_value(r).unwrap()))),
        Err(e) => {
            error!("Failed to create mapping rule: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn list_mapping_rules(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.accounting_hub_engine.list_mapping_rules(org_id, None).await {
        Ok(rules) => Ok(Json(serde_json::json!({ "data": rules }))),
        Err(e) => { error!("Failed to list rules: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn delete_mapping_rule(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(code): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.accounting_hub_engine.delete_mapping_rule(org_id, &code).await {
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

pub async fn get_accounting_hub_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.accounting_hub_engine.get_dashboard_summary(org_id).await {
        Ok(dashboard) => Ok(Json(serde_json::to_value(dashboard).unwrap())),
        Err(e) => { error!("Failed to get dashboard: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}
