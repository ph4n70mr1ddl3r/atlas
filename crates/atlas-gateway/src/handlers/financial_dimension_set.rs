//! Financial Dimension Set Handlers
//!
//! Oracle Fusion: General Ledger > Financial Dimension Sets

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
pub struct CreateDimensionSetRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
}

pub async fn create_dimension_set(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateDimensionSetRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.financial_dimension_set_engine.create(
        org_id, &payload.code, &payload.name,
        payload.description.as_deref(), Some(user_id),
    ).await {
        Ok(ds) => Ok((StatusCode::CREATED, Json(serde_json::to_value(ds).unwrap()))),
        Err(e) => {
            error!("Failed to create dimension set: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn get_dimension_set(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.financial_dimension_set_engine.get(id).await {
        Ok(Some(ds)) => Ok(Json(serde_json::to_value(ds).unwrap())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Failed to get dimension set: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListDimensionSetsQuery { pub is_active: Option<bool> }

pub async fn list_dimension_sets(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListDimensionSetsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.financial_dimension_set_engine.list(org_id, query.is_active).await {
        Ok(items) => Ok(Json(serde_json::json!({ "data": items }))),
        Err(e) => { error!("Failed to list dimension sets: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn deactivate_dimension_set(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.financial_dimension_set_engine.deactivate(id).await {
        Ok(ds) => Ok(Json(serde_json::to_value(ds).unwrap())),
        Err(e) => {
            error!("Failed to deactivate dimension set: {}", e);
            Err(match e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn get_dimension_set_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.financial_dimension_set_engine.get_dashboard(org_id).await {
        Ok(dash) => Ok(Json(serde_json::to_value(dash).unwrap())),
        Err(e) => { error!("Failed to get dimension set dashboard: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}
