//! AR Aging Analysis Handlers
//!
//! Oracle Fusion: AR > Aging Reports

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

// Definitions
#[derive(Debug, Deserialize)]
pub struct CreateDefinitionRequest {
    pub definition_code: String,
    pub name: String,
    pub description: Option<String>,
    pub aging_basis: String,
    pub num_buckets: i32,
}

pub async fn create_definition(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateDefinitionRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.ar_aging_engine.create_definition(
        org_id, &payload.definition_code, &payload.name, payload.description.as_deref(),
        &payload.aging_basis, payload.num_buckets, Some(user_id),
    ).await {
        Ok(d) => Ok((StatusCode::CREATED, Json(serde_json::to_value(d).unwrap()))),
        Err(e) => {
            error!("Failed to create aging definition: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn get_definition(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.ar_aging_engine.get_definition_by_id(id).await {
        Ok(Some(d)) => Ok(Json(serde_json::to_value(d).unwrap())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Failed to get definition: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListDefinitionsQuery { pub status: Option<String> }

pub async fn list_definitions(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListDefinitionsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.ar_aging_engine.list_definitions(org_id, query.status.as_deref()).await {
        Ok(defs) => Ok(Json(serde_json::json!({ "data": defs }))),
        Err(e) => { error!("Failed to list definitions: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn delete_definition(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    match state.ar_aging_engine.delete_definition(id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            error!("Failed to delete definition: {}", e);
            Err(match e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// Buckets
#[derive(Debug, Deserialize)]
pub struct CreateBucketRequest {
    pub bucket_number: i32,
    pub name: String,
    pub from_days: i32,
    pub to_days: Option<i32>,
    pub display_order: Option<i32>,
}

pub async fn create_bucket(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(def_id): Path<Uuid>,
    Json(payload): Json<CreateBucketRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.ar_aging_engine.create_bucket(
        org_id, def_id, payload.bucket_number, &payload.name,
        payload.from_days, payload.to_days, payload.display_order.unwrap_or(payload.bucket_number),
    ).await {
        Ok(b) => Ok((StatusCode::CREATED, Json(serde_json::to_value(b).unwrap()))),
        Err(e) => {
            error!("Failed to create bucket: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn list_buckets(
    State(state): State<Arc<AppState>>,
    Path(def_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.ar_aging_engine.list_buckets(def_id).await {
        Ok(buckets) => Ok(Json(serde_json::json!({ "data": buckets }))),
        Err(e) => { error!("Failed to list buckets: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// Snapshots
#[derive(Debug, Deserialize)]
pub struct CreateSnapshotRequest {
    pub definition_id: Uuid,
    pub as_of_date: chrono::NaiveDate,
    pub currency_code: String,
}

pub async fn create_snapshot(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateSnapshotRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.ar_aging_engine.create_snapshot(
        org_id, payload.definition_id, payload.as_of_date, &payload.currency_code, Some(user_id),
    ).await {
        Ok(s) => Ok((StatusCode::CREATED, Json(serde_json::to_value(s).unwrap()))),
        Err(e) => {
            error!("Failed to create snapshot: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn get_snapshot(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.ar_aging_engine.get_snapshot(id).await {
        Ok(Some(s)) => Ok(Json(serde_json::to_value(s).unwrap())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Failed to get snapshot: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn list_snapshots(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let def_id = params.get("definition_id").and_then(|s| Uuid::parse_str(s).ok());
    match state.ar_aging_engine.list_snapshots(org_id, def_id).await {
        Ok(snapshots) => Ok(Json(serde_json::json!({ "data": snapshots }))),
        Err(e) => { error!("Failed to list snapshots: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// Snapshot Lines
pub async fn list_snapshot_lines(
    State(state): State<Arc<AppState>>,
    Path(snapshot_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.ar_aging_engine.list_snapshot_lines(snapshot_id).await {
        Ok(lines) => Ok(Json(serde_json::json!({ "data": lines }))),
        Err(e) => { error!("Failed to list snapshot lines: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn get_aging_summary(
    State(state): State<Arc<AppState>>,
    Path(snapshot_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.ar_aging_engine.get_aging_summary(snapshot_id).await {
        Ok(summary) => Ok(Json(serde_json::json!({ "data": summary }))),
        Err(e) => { error!("Failed to get aging summary: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// Dashboard
pub async fn get_ar_aging_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.ar_aging_engine.get_dashboard(org_id).await {
        Ok(dashboard) => Ok(Json(serde_json::to_value(dashboard).unwrap())),
        Err(e) => { error!("Failed to get dashboard: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}
