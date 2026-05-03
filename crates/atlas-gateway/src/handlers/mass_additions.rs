//! Mass Additions Handlers
//!
//! Oracle Fusion: Fixed Assets > Mass Additions

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
pub struct CreateMassAdditionRequest {
    pub invoice_number: Option<String>,
    pub description: Option<String>,
    pub cost: String,
    pub supplier_number: Option<String>,
    pub category_code: Option<String>,
    pub book_code: Option<String>,
    pub asset_type: Option<String>,
}

pub async fn create_mass_addition(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateMassAdditionRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.mass_addition_engine.create_from_invoice(
        org_id, None, payload.invoice_number.as_deref(),
        None, None,
        payload.description.as_deref(), &payload.cost,
        None, payload.supplier_number.as_deref(), None,
        payload.category_code.as_deref(), payload.book_code.as_deref(),
        payload.asset_type.as_deref(), Some(user_id),
    ).await {
        Ok(ma) => Ok((StatusCode::CREATED, Json(serde_json::to_value(ma).unwrap()))),
        Err(e) => {
            error!("Failed to create mass addition: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn get_mass_addition(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.mass_addition_engine.get(id).await {
        Ok(Some(ma)) => Ok(Json(serde_json::to_value(ma).unwrap())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Failed to get mass addition: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListMassAdditionsQuery {
    pub status: Option<String>,
    pub category_code: Option<String>,
}

pub async fn list_mass_additions(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListMassAdditionsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.mass_addition_engine.list(org_id, query.status.as_deref(), query.category_code.as_deref()).await {
        Ok(items) => Ok(Json(serde_json::json!({ "data": items }))),
        Err(e) => {
            error!("Failed to list mass additions: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn hold_mass_addition(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.mass_addition_engine.hold(id).await {
        Ok(ma) => Ok(Json(serde_json::to_value(ma).unwrap())),
        Err(e) => {
            error!("Failed to hold mass addition: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) | atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn release_mass_addition(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.mass_addition_engine.release(id).await {
        Ok(ma) => Ok(Json(serde_json::to_value(ma).unwrap())),
        Err(e) => {
            error!("Failed to release mass addition: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) | atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct RejectRequest { pub reason: String }

pub async fn reject_mass_addition(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<RejectRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.mass_addition_engine.reject(id, &payload.reason).await {
        Ok(ma) => Ok(Json(serde_json::to_value(ma).unwrap())),
        Err(e) => {
            error!("Failed to reject mass addition: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) | atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct MergeRequest { pub target_id: Uuid }

pub async fn merge_mass_addition(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<MergeRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.mass_addition_engine.merge(id, payload.target_id).await {
        Ok(ma) => Ok(Json(serde_json::to_value(ma).unwrap())),
        Err(e) => {
            error!("Failed to merge mass addition: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) | atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn convert_mass_addition(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.mass_addition_engine.convert(id).await {
        Ok(ma) => Ok(Json(serde_json::to_value(ma).unwrap())),
        Err(e) => {
            error!("Failed to convert mass addition: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) | atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn get_mass_addition_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.mass_addition_engine.get_dashboard(org_id).await {
        Ok(dash) => Ok(Json(serde_json::to_value(dash).unwrap())),
        Err(e) => { error!("Failed to get mass additions dashboard: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}
