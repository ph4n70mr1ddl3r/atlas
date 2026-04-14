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
use atlas_core::EventBus;
use crate::AppState;
use std::sync::Arc;
use uuid::Uuid;
use tracing::{info, debug, error, warn};
use sqlx::{Row, Column};
use std::sync::OnceLock;
use axum::Extension;
use crate::handlers::auth::Claims;

static IDENTIFIER_RE: OnceLock<regex::Regex> = OnceLock::new();

fn identifier_regex() -> &'static regex::Regex {
    IDENTIFIER_RE.get_or_init(|| regex::Regex::new(r"^[a-z_][a-z0-9_]*$").unwrap())
}

/// Validates that an identifier is safe to use in SQL
/// Only allows lowercase alphanumeric and underscores
pub fn is_valid_identifier(identifier: &str) -> bool {
    identifier_regex().is_match(identifier)
}

/// Validates and sanitizes a table or column name
pub fn sanitize_identifier(name: &str) -> Result<String, StatusCode> {
    if name.is_empty() || name.len() > 64 {
        return Err(StatusCode::BAD_REQUEST);
    }
    if !is_valid_identifier(name) {
        return Err(StatusCode::BAD_REQUEST);
    }
    Ok(name.to_lowercase())
}

/// Convert a database row into a JSON object.
///
/// Eliminates the repeated column-iteration boilerplate that was previously
/// copy-pasted across every handler.
pub fn row_to_json(row: &sqlx::postgres::PgRow) -> serde_json::Value {
    let mut obj = serde_json::Map::new();
    for i in 0..row.columns().len() {
        let name = row.columns()[i].name();
        let value = row.try_get::<serde_json::Value, _>(i).unwrap_or(serde_json::Value::Null);
        obj.insert(name.to_string(), value);
    }
    serde_json::Value::Object(obj)
}

#[derive(Debug, Deserialize)]
pub struct ListParams {
    pub search: Option<String>,
    pub offset: Option<i64>,
    pub limit: Option<i64>,
    /// Sort field
    pub sort: Option<String>,
    /// Sort direction: "asc" or "desc" (defaults to "desc")
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
    
    // Build optional search clause using parameterized pattern ($3)
    let (search_clause, search_pattern) = match &params.search {
        Some(search) if !search.is_empty() => {
            // Escape ILIKE special characters in user input
            let escaped = search.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_");
            let pattern = format!("%{}%", escaped);
            let fields: Vec<String> = entity_def.fields.iter()
                .filter(|f| f.is_searchable && matches!(
                    f.field_type,
                    atlas_shared::FieldType::String { .. } | atlas_shared::FieldType::Email | atlas_shared::FieldType::Phone
                ))
                .map(|f| format!("\"{}\"::text ILIKE $3", f.name))
                .collect();
            if fields.is_empty() {
                (String::new(), pattern)
            } else {
                (format!(" AND ({})", fields.join(" OR ")), pattern)
            }
        }
        _ => (String::new(), String::new()),
    };

    // Build ORDER BY from sort/order params (sanitized)
    let order_clause = build_order_clause(&params.sort, &params.order);
    
    let sql = format!(
        "SELECT * FROM \"{}\" WHERE deleted_at IS NULL{} {} LIMIT $1 OFFSET $2",
        table_name, search_clause, order_clause
    );
    
    let rows = sqlx::query(&sql)
        .bind(limit)
        .bind(offset)
        .bind(&search_pattern)
        .fetch_all(&state.db_pool)
        .await
        .map_err(|e| {
            error!("Query error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    let records: Vec<serde_json::Value> = rows.iter().map(row_to_json).collect();
    
    // Get total count for pagination
    let count_sql = format!(
        "SELECT COUNT(*) FROM \"{}\" WHERE deleted_at IS NULL{}",
        table_name, search_clause
    );
    let total: i64 = sqlx::query_scalar(&count_sql)
        .bind(&search_pattern)
        .fetch_one(&state.db_pool)
        .await
        .map_err(|e| {
            error!("Count query error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    Ok(Json(serde_json::json!({
        "data": records,
        "meta": {
            "total": total,
            "offset": offset,
            "limit": limit,
        }
    })))
}

/// Build a safe ORDER BY clause from user-supplied sort/order params.
///
/// Field names are validated against the identifier regex; the order
/// direction is whitelisted to ASC/DESC.
fn build_order_clause(sort: &Option<String>, order: &Option<String>) -> String {
    match sort {
        Some(field) if !field.is_empty() => {
            if !is_valid_identifier(field) {
                // Fall back to default if the field name looks suspicious
                return "ORDER BY created_at DESC".to_string();
            }
            let dir = match order.as_deref() {
                Some("asc") | Some("ASC") => "ASC",
                Some("desc") | Some("DESC") => "DESC",
                _ => "DESC",
            };
            format!("ORDER BY \"{}\" {}", field.to_lowercase(), dir)
        }
        _ => "ORDER BY created_at DESC".to_string(),
    }
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
        Some(row) => Ok(Json(row_to_json(&row))),
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// Create a new record
pub async fn create_record(
    State(state): State<Arc<AppState>>,
    Path(entity): Path<String>,
    claims: Extension<Claims>,
    Json(mut payload): Json<CreateRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    info!("Creating record for entity: {}", entity);
    
    let entity_def = match state.schema_engine.get_entity(&entity) {
        Some(def) => def,
        None => return Err(StatusCode::NOT_FOUND),
    };
    
    let table_name = entity_def.table_name.as_deref().unwrap_or(&entity);
    
    // Sanitize table name to prevent SQL injection
    let table_name = sanitize_identifier(table_name)?;
    
    // Inject organization_id from JWT claims for multi-tenancy
    if let Some(obj) = payload.values.as_object_mut() {
        obj.insert("organization_id".to_string(), serde_json::json!(claims.org_id));
    }
    
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
    
    let record = row_to_json(&row);
    let record_id = record.get("id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok());
    
    // Log audit entry for create
    if let Some(id) = record_id {
        if let Err(e) = state.audit_engine.log_create(
            &entity,
            id,
            &record,
            claims.0.sub.parse().ok(),
        ).await {
            warn!("Failed to log audit for create: {}", e);
        }
    }
    
    // Publish event
    if let Some(id) = record_id {
        let event = atlas_core::eventbus::EventFactory::record_created(
            "atlas-gateway",
            &entity,
            id,
            record.clone(),
            claims.0.sub.parse().ok(),
        );
        let _ = state.event_bus.publish(event).await;
    }
    
    Ok((StatusCode::CREATED, Json(record)))
}

/// Update a record
pub async fn update_record(
    State(state): State<Arc<AppState>>,
    Path((entity, id)): Path<(String, Uuid)>,
    claims: Extension<Claims>,
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
    
    // Fetch the old record for audit
    let old_row = sqlx::query(
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
    
    let old_record = old_row.map(|r| row_to_json(&r));
    
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
            let new_record = row_to_json(&row);
            
            // Log audit entry
            if let Some(ref old) = old_record {
                if let Err(e) = state.audit_engine.log_update(
                    &entity,
                    id,
                    old,
                    &new_record,
                    claims.0.sub.parse().ok(),
                ).await {
                    warn!("Failed to log audit for update: {}", e);
                }
            }
            
            // Publish event
            let event = atlas_core::eventbus::EventFactory::record_updated(
                "atlas-gateway",
                &entity,
                id,
                new_record.clone(),
                claims.0.sub.parse().ok(),
            );
            let _ = state.event_bus.publish(event).await;
            
            Ok(Json(new_record))
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// Delete a record (soft delete)
pub async fn delete_record(
    State(state): State<Arc<AppState>>,
    Path((entity, id)): Path<(String, Uuid)>,
    claims: Extension<Claims>,
) -> Result<StatusCode, StatusCode> {
    info!("Deleting record {} for entity: {}", id, entity);
    
    let entity_def = match state.schema_engine.get_entity(&entity) {
        Some(def) => def,
        None => return Err(StatusCode::NOT_FOUND),
    };
    
    let table_name = entity_def.table_name.as_deref().unwrap_or(&entity);
    
    // Sanitize table name to prevent SQL injection
    let table_name = sanitize_identifier(table_name)?;
    
    // Fetch the old record for audit
    let old_row = sqlx::query(
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
    
    let old_record = match old_row {
        Some(r) => row_to_json(&r),
        None => return Err(StatusCode::NOT_FOUND),
    };
    
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
    
    // Log audit entry
    if let Err(e) = state.audit_engine.log_delete(
        &entity,
        id,
        &old_record,
        claims.0.sub.parse().ok(),
    ).await {
        warn!("Failed to log audit for delete: {}", e);
    }
    
    // Publish event
    let event = atlas_core::eventbus::EventFactory::record_deleted(
        "atlas-gateway",
        &entity,
        id,
        claims.0.sub.parse().ok(),
    );
    let _ = state.event_bus.publish(event).await;
    
    Ok(StatusCode::NO_CONTENT)
}

/// Get available workflow transitions
pub async fn get_transitions(
    State(state): State<Arc<AppState>>,
    Path((entity, id)): Path<(String, Uuid)>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    debug!("Getting transitions for record {} of entity {}", id, entity);

    let entity_def = state.schema_engine.get_entity(&entity)
        .ok_or(StatusCode::NOT_FOUND)?;

    let workflow = match &entity_def.workflow {
        Some(w) => w,
        None => return Ok(Json(serde_json::json!({"transitions": []}))),
    };

    // Look up current workflow state from DB
    let table_name = entity_def.table_name.as_deref().unwrap_or(&entity);
    let table_name = sanitize_identifier(table_name)?;

    let row = sqlx::query(
        format!("SELECT workflow_state FROM \"{}\" WHERE id = $1 AND deleted_at IS NULL", table_name).as_str()
    )
    .bind(id)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Query error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let current_state = match row {
        Some(r) => r.try_get::<String, _>(0).unwrap_or_else(|_| workflow.initial_state.clone()),
        None => return Err(StatusCode::NOT_FOUND),
    };

    // Get available transitions from the workflow engine
    let available = state.workflow_engine
        .get_available_transitions(&workflow.name, &current_state, None)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({
        "current_state": current_state,
        "transitions": available.transitions
    })))
}

/// Execute a workflow action
pub async fn execute_action(
    State(state): State<Arc<AppState>>,
    Path((entity, id, action)): Path<(String, Uuid, String)>,
    Json(payload): Json<WorkflowActionRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Executing action {} on {}:{}", action, entity, id);

    let entity_def = state.schema_engine.get_entity(&entity)
        .ok_or(StatusCode::NOT_FOUND)?;

    let workflow = entity_def.workflow.as_ref()
        .ok_or_else(|| {
            error!("No workflow defined for entity {}", entity);
            StatusCode::BAD_REQUEST
        })?;

    let table_name = entity_def.table_name.as_deref().unwrap_or(&entity);
    let table_name = sanitize_identifier(table_name)?;

    // Fetch the current record
    let row = sqlx::query(
        format!("SELECT * FROM \"{}\" WHERE id = $1 AND deleted_at IS NULL", table_name).as_str()
    )
    .bind(id)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Query error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let record = match row {
        Some(r) => row_to_json(&r),
        None => return Err(StatusCode::NOT_FOUND),
    };

    let current_state = record.get("workflow_state")
        .and_then(|v| v.as_str())
        .unwrap_or(&workflow.initial_state)
        .to_string();

    // Execute the transition via workflow engine
    let result = state.workflow_engine
        .execute_transition(
            &workflow.name,
            id,
            &current_state,
            &action,
            None,
            &record,
            payload.comment.clone(),
        )
        .await
        .map_err(|e| {
            error!("Workflow transition error: {}", e);
            StatusCode::BAD_REQUEST
        })?;

    if !result.success {
        return Ok(Json(serde_json::json!({
            "success": false,
            "error": result.error,
            "from_state": result.from_state,
            "to_state": result.to_state,
        })));
    }

    // Update the record's workflow_state in the database
    let update_query = format!(
        "UPDATE \"{}\" SET workflow_state = $1, updated_at = now() WHERE id = $2",
        table_name
    );
    sqlx::query(&update_query)
        .bind(&result.to_state)
        .bind(id)
        .execute(&state.db_pool)
        .await
        .map_err(|e| {
            error!("Failed to update workflow state: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Merge any values from the payload
    if let Some(values) = payload.values {
        if let Some(obj) = values.as_object() {
            let set_clauses: Vec<String> = obj.keys()
                .map(|k| sanitize_identifier(k))
                .collect::<Result<Vec<_>, _>>()?
                .iter()
                .enumerate()
                .map(|(i, k)| format!("\"{}\" = ${}", k, i + 1))
                .collect();

            if !set_clauses.is_empty() {
                let update_query = format!(
                    "UPDATE \"{}\" SET {}, updated_at = now() WHERE id = ${}",
                    table_name,
                    set_clauses.join(", "),
                    set_clauses.len() + 1
                );
                let mut q = sqlx::query(&update_query);
                for (_, value) in obj {
                    q = q.bind(value);
                }
                q = q.bind(id);
                q.execute(&state.db_pool).await.map_err(|e| {
                    error!("Failed to update record: {}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;
            }
        }
    }

    // Publish workflow transition event
    let event = atlas_core::eventbus::EventFactory::workflow_transition(
        "atlas-gateway",
        &entity,
        id,
        &workflow.name,
        &result.from_state,
        &result.to_state,
        &action,
        None,
    );
    let _ = state.event_bus.publish(event).await;

    // Log to audit
    if let Err(e) = state.audit_engine.log(
        &entity,
        id,
        atlas_shared::AuditAction::ExecuteAction,
        Some(&record),
        Some(&serde_json::json!({"action": action, "from_state": result.from_state, "to_state": result.to_state})),
        None,
        None,
        None,
        None,
    ).await {
        warn!("Failed to log audit: {}", e);
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "from_state": result.from_state,
        "to_state": result.to_state,
        "action": action,
        "executed_actions": result.executed_actions,
    })))
}

/// Get record audit history
pub async fn get_record_history(
    State(state): State<Arc<AppState>>,
    Path((entity, id)): Path<(String, Uuid)>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    debug!("Getting history for record {} of entity {}", id, entity);

    let entries = state.audit_engine
        .get_entity_history(&entity, id)
        .await
        .map_err(|e| {
            error!("Audit query error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(serde_json::json!({
        "entity": entity,
        "id": id,
        "history": entries
    })))
}
