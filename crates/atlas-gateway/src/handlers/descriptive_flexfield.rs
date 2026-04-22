//! Descriptive Flexfield API Handlers
//!
//! REST API for managing Descriptive Flexfields (DFF).
//! Oracle Fusion equivalent: Application Extensions > Flexfields > Descriptive

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension,
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;
use uuid::Uuid;
use tracing::{info, error};
use crate::AppState;
use crate::handlers::auth::Claims;

// ═══════════════════════════════════════════════════════════════════════════════
// Value Sets
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Deserialize)]
pub struct CreateValueSetRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub validation_type: String,
    pub data_type: String,
    pub max_length: Option<i32>,
    pub min_length: Option<i32>,
    pub format_mask: Option<String>,
    pub table_validation: Option<Value>,
    pub independent_values: Option<Value>,
    pub parent_value_set_code: Option<String>,
}

pub async fn create_value_set(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(body): Json<CreateValueSetRequest>,
) -> Result<(StatusCode, Json<Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).ok();

    match state.dff_engine.create_value_set(
        org_id,
        &body.code,
        &body.name,
        body.description.as_deref(),
        &body.validation_type,
        &body.data_type,
        body.max_length.unwrap_or(240),
        body.min_length.unwrap_or(0),
        body.format_mask.as_deref(),
        body.table_validation.clone(),
        body.independent_values.clone(),
        body.parent_value_set_code.as_deref(),
        user_id,
    ).await {
        Ok(vs) => {
            info!("Created value set '{}' for org {}", vs.code, org_id);
            Ok((StatusCode::CREATED, Json(serde_json::to_value(vs).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))))
        }
        Err(e) => {
            error!("Failed to create value set: {}", e);
            Err(map_error_status(&e))
        }
    }
}

pub async fn list_value_sets(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.dff_engine.list_value_sets(org_id).await {
        Ok(list) => Ok(Json(json!(list))),
        Err(e) => {
            error!("Failed to list value sets: {}", e);
            Err(map_error_status(&e))
        }
    }
}

pub async fn get_value_set(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(code): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.dff_engine.get_value_set(org_id, &code).await {
        Ok(Some(vs)) => Ok(Json(serde_json::to_value(vs).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => Err(map_error_status(&e)),
    }
}

pub async fn delete_value_set(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(code): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.dff_engine.delete_value_set(org_id, &code).await {
        Ok(()) => {
            info!("Deleted value set '{}'", code);
            Ok(StatusCode::NO_CONTENT)
        }
        Err(e) => Err(map_error_status(&e)),
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Value Set Entries
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Deserialize)]
pub struct CreateValueSetEntryRequest {
    pub value: String,
    pub meaning: Option<String>,
    pub description: Option<String>,
    pub parent_value: Option<String>,
    pub is_enabled: Option<bool>,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub sort_order: Option<i32>,
}

pub async fn create_value_set_entry(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(code): Path<String>,
    Json(body): Json<CreateValueSetEntryRequest>,
) -> Result<(StatusCode, Json<Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).ok();

    match state.dff_engine.create_value_set_entry(
        org_id,
        &code,
        &body.value,
        body.meaning.as_deref(),
        body.description.as_deref(),
        body.parent_value.as_deref(),
        body.is_enabled.unwrap_or(true),
        body.effective_from,
        body.effective_to,
        body.sort_order.unwrap_or(0),
        user_id,
    ).await {
        Ok(entry) => {
            info!("Created value set entry '{}' in '{}'", entry.value, code);
            Ok((StatusCode::CREATED, Json(serde_json::to_value(entry).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))))
        }
        Err(e) => {
            error!("Failed to create value set entry: {}", e);
            Err(map_error_status(&e))
        }
    }
}

#[derive(Deserialize)]
pub struct ListEntriesQuery {
    pub parent_value: Option<String>,
}

pub async fn list_value_set_entries(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(code): Path<String>,
    Query(query): Query<ListEntriesQuery>,
) -> Result<Json<Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.dff_engine.list_value_set_entries(
        org_id, &code, query.parent_value.as_deref(),
    ).await {
        Ok(list) => Ok(Json(json!(list))),
        Err(e) => Err(map_error_status(&e)),
    }
}

pub async fn delete_value_set_entry(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    match state.dff_engine.delete_value_set_entry(id).await {
        Ok(()) => {
            info!("Deleted value set entry {}", id);
            Ok(StatusCode::NO_CONTENT)
        }
        Err(e) => Err(map_error_status(&e)),
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Flexfields
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Deserialize)]
pub struct CreateFlexfieldRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub entity_name: String,
    pub context_column: Option<String>,
    pub default_context_code: Option<String>,
}

pub async fn create_flexfield(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(body): Json<CreateFlexfieldRequest>,
) -> Result<(StatusCode, Json<Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).ok();

    match state.dff_engine.create_flexfield(
        org_id,
        &body.code,
        &body.name,
        body.description.as_deref(),
        &body.entity_name,
        body.context_column.as_deref(),
        body.default_context_code.as_deref(),
        user_id,
    ).await {
        Ok(ff) => {
            info!("Created flexfield '{}' for entity '{}'", ff.code, ff.entity_name);
            Ok((StatusCode::CREATED, Json(serde_json::to_value(ff).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))))
        }
        Err(e) => {
            error!("Failed to create flexfield: {}", e);
            Err(map_error_status(&e))
        }
    }
}

pub async fn list_flexfields(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.dff_engine.list_flexfields(org_id).await {
        Ok(list) => Ok(Json(json!(list))),
        Err(e) => Err(map_error_status(&e)),
    }
}

pub async fn get_flexfield(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(code): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.dff_engine.get_flexfield(org_id, &code).await {
        Ok(Some(ff)) => Ok(Json(serde_json::to_value(ff).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => Err(map_error_status(&e)),
    }
}

pub async fn activate_flexfield(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, StatusCode> {
    match state.dff_engine.activate_flexfield(id).await {
        Ok(ff) => {
            info!("Activated flexfield '{}'", ff.code);
            Ok(Json(serde_json::to_value(ff).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))
        }
        Err(e) => Err(map_error_status(&e)),
    }
}

pub async fn deactivate_flexfield(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, StatusCode> {
    match state.dff_engine.deactivate_flexfield(id).await {
        Ok(ff) => {
            info!("Deactivated flexfield '{}'", ff.code);
            Ok(Json(serde_json::to_value(ff).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))
        }
        Err(e) => Err(map_error_status(&e)),
    }
}

pub async fn delete_flexfield(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(code): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.dff_engine.delete_flexfield(org_id, &code).await {
        Ok(()) => {
            info!("Deleted flexfield '{}'", code);
            Ok(StatusCode::NO_CONTENT)
        }
        Err(e) => Err(map_error_status(&e)),
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Contexts
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Deserialize)]
pub struct CreateContextRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub is_global: Option<bool>,
}

pub async fn create_context(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(flexfield_code): Path<String>,
    Json(body): Json<CreateContextRequest>,
) -> Result<(StatusCode, Json<Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).ok();

    match state.dff_engine.create_context(
        org_id,
        &flexfield_code,
        &body.code,
        &body.name,
        body.description.as_deref(),
        body.is_global.unwrap_or(false),
        user_id,
    ).await {
        Ok(ctx) => {
            info!("Created context '{}' in flexfield '{}'", ctx.code, flexfield_code);
            Ok((StatusCode::CREATED, Json(serde_json::to_value(ctx).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))))
        }
        Err(e) => {
            error!("Failed to create context: {}", e);
            Err(map_error_status(&e))
        }
    }
}

pub async fn list_contexts(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(flexfield_code): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.dff_engine.list_contexts(org_id, &flexfield_code).await {
        Ok(list) => Ok(Json(json!(list))),
        Err(e) => Err(map_error_status(&e)),
    }
}

pub async fn disable_context(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, StatusCode> {
    match state.dff_engine.disable_context(id).await {
        Ok(ctx) => Ok(Json(serde_json::to_value(ctx).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err(map_error_status(&e)),
    }
}

pub async fn enable_context(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, StatusCode> {
    match state.dff_engine.enable_context(id).await {
        Ok(ctx) => Ok(Json(serde_json::to_value(ctx).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err(map_error_status(&e)),
    }
}

pub async fn delete_context(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    match state.dff_engine.delete_context(id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err(map_error_status(&e)),
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Segments
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Deserialize)]
pub struct CreateSegmentRequest {
    pub segment_code: String,
    pub name: String,
    pub description: Option<String>,
    pub display_order: Option<i32>,
    pub column_name: String,
    pub data_type: String,
    pub is_required: Option<bool>,
    pub is_read_only: Option<bool>,
    pub is_visible: Option<bool>,
    pub default_value: Option<String>,
    pub value_set_code: Option<String>,
    pub help_text: Option<String>,
}

pub async fn create_segment(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path((flexfield_code, context_code)): Path<(String, String)>,
    Json(body): Json<CreateSegmentRequest>,
) -> Result<(StatusCode, Json<Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).ok();

    match state.dff_engine.create_segment(
        org_id,
        &flexfield_code,
        &context_code,
        &body.segment_code,
        &body.name,
        body.description.as_deref(),
        body.display_order.unwrap_or(1),
        &body.column_name,
        &body.data_type,
        body.is_required.unwrap_or(false),
        body.is_read_only.unwrap_or(false),
        body.is_visible.unwrap_or(true),
        body.default_value.as_deref(),
        body.value_set_code.as_deref(),
        body.help_text.as_deref(),
        user_id,
    ).await {
        Ok(seg) => {
            info!("Created segment '{}' in '{}/{}'", seg.segment_code, flexfield_code, context_code);
            Ok((StatusCode::CREATED, Json(serde_json::to_value(seg).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))))
        }
        Err(e) => {
            error!("Failed to create segment: {}", e);
            Err(map_error_status(&e))
        }
    }
}

pub async fn list_segments_by_context(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path((flexfield_code, context_code)): Path<(String, String)>,
) -> Result<Json<Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.dff_engine.list_segments_by_context(org_id, &flexfield_code, &context_code).await {
        Ok(list) => Ok(Json(json!(list))),
        Err(e) => Err(map_error_status(&e)),
    }
}

pub async fn list_segments_by_flexfield(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(flexfield_code): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.dff_engine.list_segments_by_flexfield(org_id, &flexfield_code).await {
        Ok(list) => Ok(Json(json!(list))),
        Err(e) => Err(map_error_status(&e)),
    }
}

pub async fn delete_segment(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    match state.dff_engine.delete_segment(id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err(map_error_status(&e)),
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Flexfield Data
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Deserialize)]
pub struct SetFlexfieldDataRequest {
    pub context_code: String,
    pub segment_values: Value,
}

pub async fn set_flexfield_data(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path((entity_name, entity_id)): Path<(String, Uuid)>,
    Json(body): Json<SetFlexfieldDataRequest>,
) -> Result<(StatusCode, Json<Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).ok();

    match state.dff_engine.set_data(
        org_id,
        &entity_name,
        entity_id,
        &body.context_code,
        body.segment_values,
        user_id,
    ).await {
        Ok(data) => {
            info!("Set flexfield data for {}/{}", entity_name, entity_id);
            Ok((StatusCode::OK, Json(serde_json::to_value(data).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))))
        }
        Err(e) => {
            error!("Failed to set flexfield data: {}", e);
            Err(map_error_status(&e))
        }
    }
}

#[derive(Deserialize)]
pub struct GetDataQuery {
    pub context_code: Option<String>,
}

pub async fn get_flexfield_data(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path((entity_name, entity_id)): Path<(String, Uuid)>,
    Query(query): Query<GetDataQuery>,
) -> Result<Json<Value>, StatusCode> {
    match state.dff_engine.get_data(
        &entity_name,
        entity_id,
        query.context_code.as_deref(),
    ).await {
        Ok(list) => Ok(Json(json!(list))),
        Err(e) => Err(map_error_status(&e)),
    }
}

pub async fn delete_flexfield_data(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path((entity_name, entity_id)): Path<(String, Uuid)>,
) -> Result<StatusCode, StatusCode> {
    match state.dff_engine.delete_data(&entity_name, entity_id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err(map_error_status(&e)),
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Dashboard
// ═══════════════════════════════════════════════════════════════════════════════

pub async fn get_flexfield_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.dff_engine.get_dashboard_summary(org_id).await {
        Ok(summary) => Ok(Json(serde_json::to_value(summary).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err(map_error_status(&e)),
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════════════════════

fn map_error_status(e: &atlas_shared::AtlasError) -> StatusCode {
    use atlas_shared::AtlasError;
    match e {
        AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
        AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
        AtlasError::Conflict(_) => StatusCode::CONFLICT,
        AtlasError::WorkflowError(_) => StatusCode::CONFLICT,
        AtlasError::ConfigError(_) => StatusCode::BAD_REQUEST,
        AtlasError::DatabaseError(msg) => {
            error!("Database error: {}", msg);
            StatusCode::INTERNAL_SERVER_ERROR
        }
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
