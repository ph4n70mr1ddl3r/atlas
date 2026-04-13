//! Record handlers

use axum::{
    extract::{State, Path, Query},
    Json,
    http::StatusCode,
};
use serde::Deserialize;
use atlas_shared::{
    CreateRequest, UpdateRequest, WorkflowActionRequest,
};
use crate::AppState;
use std::sync::Arc;
use uuid::Uuid;
use tracing::{info, debug, warn, error};
use sqlx::{Row, Column};
use regex::Regex;

/// Validates that an identifier is safe to use in SQL
/// Only allows lowercase alphanumeric and underscores
fn is_valid_identifier(identifier: &str) -> bool {
    let re = Regex::new(r"^[a-z_][a-z0-9_]*$").unwrap();
    re.is_match(identifier)
}

/// Validates and sanitizes a table or column name
fn sanitize_identifier(name: &str) -> Result<String, StatusCode> {
    if name.is_empty() || name.len() > 64 {
        return Err(StatusCode::BAD_REQUEST);
    }
    if !is_valid_identifier(name) {
        return Err(StatusCode::BAD_REQUEST);
    }
    Ok(name.to_lowercase())
}

#[derive(Debug, Deserialize)]
pub struct ListParams {
    pub search: Option<String>,
    pub offset: Option<i64>,
    pub limit: Option<i64>,
    pub sort: Option<String>,
    pub order: Option<String>,
}

/// List records with filtering and pagination
pub async fn list_records(
    State(state): State<Arc<AppState>>,
    Path(entity): Path<String>,
    Query(params): Query<ListParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    debug!("Listing records for entity: {}", entity);
    
    let entity_def = match state.schema_engine.get_entity(&entity) {
        Some(def) => def,
        None => return Err(StatusCode::NOT_FOUND),
    };
    
    let table_name = entity_def.table_name.as_deref().unwrap_or(&entity);
    
    // Sanitize table name to prevent SQL injection
    let table_name = sanitize_identifier(table_name)?;
    
    let offset = params.offset.unwrap_or(0).max(0);
    let limit = params.limit.unwrap_or(20).clamp(1, 100);
    
    // Use parameterized queries for LIMIT and OFFSET (integers only)
    let rows = sqlx::query(
        format!(
            "SELECT * FROM \"{}\" WHERE deleted_at IS NULL ORDER BY created_at DESC LIMIT $1 OFFSET $2",
            table_name
        ).as_str()
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Query error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    let records: Vec<serde_json::Value> = rows.iter().map(|row| {
        let mut obj = serde_json::Map::new();
        for i in 0..row.columns().len() {
            let name = row.columns()[i].name();
            let value = row.try_get::<serde_json::Value, _>(i).unwrap_or(serde_json::Value::Null);
            obj.insert(name.to_string(), value);
        }
        serde_json::Value::Object(obj)
    }).collect();
    
    Ok(Json(serde_json::json!({
        "data": records,
        "meta": {
            "offset": offset,
            "limit": limit,
        }
    })))
}

/// Get a single record
pub async fn get_record(
    State(state): State<Arc<AppState>>,
    Path((entity, id)): Path<(String, Uuid)>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    debug!("Getting record {} for entity: {}", id, entity);
    
    let entity_def = match state.schema_engine.get_entity(&entity) {
        Some(def) => def,
        None => return Err(StatusCode::NOT_FOUND),
    };
    
    let table_name = entity_def.table_name.as_deref().unwrap_or(&entity);
    
    // Sanitize table name to prevent SQL injection
    let table_name = sanitize_identifier(table_name)?;
    
    let row = sqlx::query(
        format!(
            "SELECT * FROM \"{}\" WHERE id = $1 AND deleted_at IS NULL",
            table_name
        ).as_str()
    )
    .bind(id)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Query error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    match row {
        Some(row) => {
            let mut obj = serde_json::Map::new();
            for i in 0..row.columns().len() {
                let name = row.columns()[i].name();
                let value = row.try_get::<serde_json::Value, _>(i).unwrap_or(serde_json::Value::Null);
                obj.insert(name.to_string(), value);
            }
            Ok(Json(serde_json::Value::Object(obj)))
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// Create a new record
pub async fn create_record(
    State(state): State<Arc<AppState>>,
    Path(entity): Path<String>,
    Json(payload): Json<CreateRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    info!("Creating record for entity: {}", entity);
    
    let entity_def = match state.schema_engine.get_entity(&entity) {
        Some(def) => def,
        None => return Err(StatusCode::NOT_FOUND),
    };
    
    let table_name = entity_def.table_name.as_deref().unwrap_or(&entity);
    
    // Sanitize table name to prevent SQL injection
    let table_name = sanitize_identifier(table_name)?;
    
    // Validate and sanitize field names
    let fields: Vec<String> = payload.values.as_object().unwrap().keys()
        .map(|k| sanitize_identifier(k))
        .collect::<Result<Vec<_>, _>>()?;
    
    let placeholders: Vec<String> = (1..=fields.len())
        .map(|i| format!("${}", i))
        .collect();
    
    let query = format!(
        "INSERT INTO \"{}\" ({}) VALUES ({}) RETURNING *",
        table_name,
        fields.iter().map(|f| format!("\"{}\"", f)).collect::<Vec<_>>().join(", "),
        placeholders.join(", ")
    );
    
    let mut db_query = sqlx::query(&query);
    for (_, value) in payload.values.as_object().unwrap() {
        db_query = db_query.bind(value);
    }
    
    let row = db_query
        .fetch_one(&state.db_pool)
        .await
        .map_err(|e| {
            error!("Create error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    let mut obj = serde_json::Map::new();
    for i in 0..row.columns().len() {
        let name = row.columns()[i].name();
        let value = row.try_get::<serde_json::Value, _>(i).unwrap_or(serde_json::Value::Null);
        obj.insert(name.to_string(), value);
    }
    
    Ok((StatusCode::CREATED, Json(serde_json::Value::Object(obj))))
}

/// Update a record
pub async fn update_record(
    State(state): State<Arc<AppState>>,
    Path((entity, id)): Path<(String, Uuid)>,
    Json(payload): Json<UpdateRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Updating record {} for entity: {}", id, entity);
    
    let entity_def = match state.schema_engine.get_entity(&entity) {
        Some(def) => def,
        None => return Err(StatusCode::NOT_FOUND),
    };
    
    let table_name = entity_def.table_name.as_deref().unwrap_or(&entity);
    
    // Sanitize table name to prevent SQL injection
    let table_name = sanitize_identifier(table_name)?;
    
    // Validate and sanitize field names
    let set_clauses: Vec<String> = payload.values.as_object().unwrap().keys()
        .map(|k| sanitize_identifier(k))
        .collect::<Result<Vec<_>, _>>()?
        .iter()
        .enumerate()
        .map(|(i, k)| format!("\"{}\" = ${}", k, i + 1))
        .collect();
    
    let query = format!(
        "UPDATE \"{}\" SET {} WHERE id = ${} RETURNING *",
        table_name,
        set_clauses.join(", "),
        set_clauses.len() + 1
    );
    
    let mut db_query = sqlx::query(&query);
    for (_, value) in payload.values.as_object().unwrap() {
        db_query = db_query.bind(value);
    }
    db_query = db_query.bind(id);
    
    let row = db_query
        .fetch_optional(&state.db_pool)
        .await
        .map_err(|e| {
            error!("Update error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    match row {
        Some(row) => {
            let mut obj = serde_json::Map::new();
            for i in 0..row.columns().len() {
                let name = row.columns()[i].name();
                let value = row.try_get::<serde_json::Value, _>(i).unwrap_or(serde_json::Value::Null);
                obj.insert(name.to_string(), value);
            }
            Ok(Json(serde_json::Value::Object(obj)))
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// Delete a record (soft delete)
pub async fn delete_record(
    State(state): State<Arc<AppState>>,
    Path((entity, id)): Path<(String, Uuid)>,
) -> Result<StatusCode, StatusCode> {
    info!("Deleting record {} for entity: {}", id, entity);
    
    let entity_def = match state.schema_engine.get_entity(&entity) {
        Some(def) => def,
        None => return Err(StatusCode::NOT_FOUND),
    };
    
    let table_name = entity_def.table_name.as_deref().unwrap_or(&entity);
    
    // Sanitize table name to prevent SQL injection
    let table_name = sanitize_identifier(table_name)?;
    
    let query = if entity_def.is_soft_delete {
        format!("UPDATE \"{}\" SET deleted_at = NOW() WHERE id = $1", table_name)
    } else {
        format!("DELETE FROM \"{}\" WHERE id = $1", table_name)
    };
    
    let result = sqlx::query(&query)
        .bind(id)
        .execute(&state.db_pool)
        .await
        .map_err(|e| {
            error!("Delete error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }
    
    Ok(StatusCode::NO_CONTENT)
}

/// Get available workflow transitions
pub async fn get_transitions(
    State(_state): State<Arc<AppState>>,
    Path((entity, _id)): Path<(String, Uuid)>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    debug!("Getting transitions for record of entity {}", entity);
    Ok(Json(serde_json::json!({
        "transitions": []
    })))
}

/// Execute a workflow action
pub async fn execute_action(
    State(_state): State<Arc<AppState>>,
    Path((entity, _id, action)): Path<(String, Uuid, String)>,
    Json(_payload): Json<WorkflowActionRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Executing action {} on entity {}", action, entity);
    Ok(Json(serde_json::json!({
        "success": false,
        "error": "Not implemented"
    })))
}

/// Get record audit history
pub async fn get_record_history(
    State(_state): State<Arc<AppState>>,
    Path((entity, id)): Path<(String, Uuid)>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    debug!("Getting history for record {} of entity {}", id, entity);
    Ok(Json(serde_json::json!({
        "history": []
    })))
}
