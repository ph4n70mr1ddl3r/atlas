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
        // Try JSONB first (works for JSONB columns), then concrete PG types
        let value = row.try_get::<serde_json::Value, _>(i)
            .or_else(|_| row.try_get::<String, _>(i).map(serde_json::Value::String))
            .or_else(|_| row.try_get::<bool, _>(i).map(|b| serde_json::json!(b)))
            .or_else(|_| row.try_get::<i64, _>(i).map(|n| serde_json::json!(n)))
            .or_else(|_| row.try_get::<i32, _>(i).map(|n| serde_json::json!(n)))
            .or_else(|_| row.try_get::<f64, _>(i).map(|n| serde_json::json!(n)))
            .or_else(|_| row.try_get::<chrono::DateTime<chrono::Utc>, _>(i).map(|d| serde_json::json!(d.to_rfc3339())))
            .or_else(|_| row.try_get::<chrono::NaiveDate, _>(i).map(|d| serde_json::json!(d.to_string())))
            .or_else(|_| row.try_get::<uuid::Uuid, _>(i).map(|u| serde_json::json!(u.to_string())))
            .unwrap_or(serde_json::Value::Null);
        obj.insert(name.to_string(), value);
    }
    serde_json::Value::Object(obj)
}


/// Convert a JSON value into an `Option<String>` suitable for binding as
/// `::text` in a parameterised PostgreSQL query.
pub fn json_to_text(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::Null => None,
        serde_json::Value::Bool(b) => Some(if *b { "true".into() } else { "false".into() }),
        serde_json::Value::Number(n) => Some(n.to_string()),
        serde_json::Value::String(s) => Some(s.clone()),
        other => Some(other.to_string()),
    }
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
    claims: Extension<Claims>,
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
    
    // Build optional search clause using parameterized pattern.
    // In the main query ($1=limit, $2=offset, $3=org_id) the search placeholder is $4.
    // In the count query ($1=org_id) the search placeholder is $2.
    let (search_clause, search_clause_count, search_pattern) = match &params.search {
        Some(search) if !search.is_empty() => {
            // Escape ILIKE special characters in user input
            let escaped = search.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_");
            let pattern = format!("%{}%", escaped);
            let fields: Vec<String> = entity_def.fields.iter()
                .filter(|f| f.is_searchable && matches!(
                    f.field_type,
                    atlas_shared::FieldType::String { .. } | atlas_shared::FieldType::Email | atlas_shared::FieldType::Phone
                ))
                .map(|f| format!("\"{}\"::text ILIKE ", f.name))
                .collect();
            if fields.is_empty() {
                (String::new(), String::new(), pattern)
            } else {
                // $4 for the main query, $2 for the count query
                (
                    format!(" AND ({})", fields.join(" OR ").replace(" ILIKE ", " ILIKE $4")),
                    format!(" AND ({})", fields.join(" OR ").replace(" ILIKE ", " ILIKE $2")),
                    pattern,
                )
            }
        }
        _ => (String::new(), String::new(), String::new()),
    };

    // Soft-delete filter + organization_id multi-tenancy filter.
    // `org_param_idx` is the $N placeholder for the organization_id value.
    // It's $3 in the main query ($1=limit, $2=offset) and $1 in the count query.
    let (base_filter_main, base_filter_count, org_id_param) = {
        let soft_delete = if entity_def.is_soft_delete {
            "deleted_at IS NULL"
        } else {
            "1=1"
        };
        let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        (
            format!(" WHERE {} AND organization_id = $3", soft_delete),
            format!(" WHERE {} AND organization_id = $1", soft_delete),
            org_id,
        )
    };

    // Build ORDER BY from sort/order params (sanitized)
    let order_clause = build_order_clause(&params.sort, &params.order);
    
    let sql = format!(
        "SELECT * FROM \"{}\"{}{}{} LIMIT $1 OFFSET $2",
        table_name, base_filter_main, search_clause, order_clause
    );
    
    let rows = if search_pattern.is_empty() {
        sqlx::query(&sql)
            .bind(limit)
            .bind(offset)
            .bind(org_id_param)
            .fetch_all(&state.db_pool)
            .await
    } else {
        sqlx::query(&sql)
            .bind(limit)
            .bind(offset)
            .bind(org_id_param)
            .bind(&search_pattern)
            .fetch_all(&state.db_pool)
            .await
    }
    .map_err(|e| {
        error!("Query error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    let records: Vec<serde_json::Value> = rows.iter().map(row_to_json).collect();
    
    // Get total count for pagination
    // Count query: $1=org_id, $2=search_pattern (or just $1=org_id if no search)
    let count_sql = format!(
        "SELECT COUNT(*) FROM \"{}\"{}{}",
        table_name, base_filter_count, search_clause_count
    );
    let total: i64 = if search_pattern.is_empty() {
        sqlx::query_scalar(&count_sql)
            .bind(org_id_param)
    } else {
        sqlx::query_scalar(&count_sql)
            .bind(org_id_param)
            .bind(&search_pattern)
    }
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
///
/// Scoped to the caller's organization via JWT claims to prevent
/// cross-tenant data access.
pub async fn get_record(
    State(state): State<Arc<AppState>>,
    Path((entity, id)): Path<(String, Uuid)>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    debug!("Getting record {} for entity: {}", id, entity);
    
    let entity_def = match state.schema_engine.get_entity(&entity) {
        Some(def) => def,
        None => return Err(StatusCode::NOT_FOUND),
    };
    
    let table_name = entity_def.table_name.as_deref().unwrap_or(&entity);
    
    // Sanitize table name to prevent SQL injection
    let table_name = sanitize_identifier(table_name)?;

    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let row = sqlx::query(
        format!(
            "SELECT * FROM \"{}\" WHERE id = $1 AND organization_id = $2{}",
            table_name,
            if entity_def.is_soft_delete { " AND deleted_at IS NULL" } else { "" }
        ).as_str()
    )
    .bind(id)
    .bind(org_id)
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
    
    // Inject organization_id from JWT claims for multi-tenancy
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // Validate and sanitize field names, filtering out null values
    let non_null_fields: Vec<(String, &serde_json::Value)> = payload.values.as_object().unwrap()
        .iter()
        .filter(|(_, v)| !v.is_null())
        .map(|(k, v)| sanitize_identifier(k).map(|safe_k| (safe_k, v)))
        .collect::<Result<Vec<_>, _>>()?;
    
    let fields: Vec<&str> = non_null_fields.iter().map(|(k, _)| k.as_str()).collect();
    let values: Vec<&serde_json::Value> = non_null_fields.iter().map(|(_, v)| *v).collect();
    
    // Placeholders: ($1::text, $2::text, ..., $N::text, $(N+1)::uuid)
    // We bind all user values as text (sqlx sends Option<String>),
    // then PostgreSQL auto-casts text to most column types.
    // organization_id is bound separately as UUID.
    let field_count = fields.len();
    let placeholders: Vec<String> = (1..=field_count)
        .map(|i| format!("${}::text", i))
        .collect();
    let org_placeholder = format!("${}::uuid", field_count + 1);
    
    let query = format!(
        "INSERT INTO \"{}\" ({}, \"organization_id\") VALUES ({}, {}) RETURNING *",
        table_name,
        fields.iter().map(|f| format!("\"{}\"", f)).collect::<Vec<_>>().join(", "),
        placeholders.join(", "),
        org_placeholder
    );
    let mut db_query = sqlx::query(&query);
    for value in &values {
        db_query = db_query.bind(json_to_text(value));
    }
    db_query = db_query.bind(org_id);
    
    let row = db_query
        .fetch_one(&state.db_pool)
        .await
        .map_err(|e| {
            error!("Create error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    let record = row_to_json(&row);
    let record_id = row.try_get::<Uuid, _>("id").ok();
    
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
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Fetch the old record for audit (scoped to organization)
    let soft_delete_filter = if entity_def.is_soft_delete {
        " AND deleted_at IS NULL"
    } else {
        ""
    };
    let old_row = sqlx::query(
        format!(
            "SELECT * FROM \"{}\" WHERE id = $1 AND organization_id = $2{}",
            table_name, soft_delete_filter
        ).as_str()
    )
    .bind(id)
    .bind(org_id)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Query error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    let old_record = old_row.map(|r| row_to_json(&r));
    
    // Validate and sanitize field names, filter null values
    let non_null: Vec<(String, &serde_json::Value)> = payload.values.as_object().unwrap()
        .iter()
        .filter(|(_, v)| !v.is_null())
        .map(|(k, v)| sanitize_identifier(k).map(|safe_k| (safe_k, v)))
        .collect::<Result<Vec<_>, _>>()?;
    
    let set_clauses: Vec<String> = non_null.iter()
        .enumerate()
        .map(|(i, (k, _))| format!("\"{}\" = ${}::text", k, i + 1))
        .collect();
    
    let soft_delete_update = if entity_def.is_soft_delete {
        " AND deleted_at IS NULL"
    } else {
        ""
    };
    let query = format!(
        "UPDATE \"{}\" SET {}, updated_at = now() WHERE id = ${} AND organization_id = ${}{} RETURNING *",
        table_name,
        set_clauses.join(", "),
        non_null.len() + 1,
        non_null.len() + 2,
        soft_delete_update
    );
    
    let mut db_query = sqlx::query(&query);
    for (_, value) in &non_null {
        db_query = db_query.bind(json_to_text(value));
    }
    db_query = db_query.bind(id);
    db_query = db_query.bind(org_id);
    
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
    
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Fetch the old record for audit (scoped to organization)
    let old_row = sqlx::query(
        format!(
            "SELECT * FROM \"{}\" WHERE id = $1 AND organization_id = $2 AND deleted_at IS NULL",
            table_name
        ).as_str()
    )
    .bind(id)
    .bind(org_id)
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
        format!("UPDATE \"{}\" SET deleted_at = NOW() WHERE id = $1 AND organization_id = $2 AND deleted_at IS NULL", table_name)
    } else {
        format!("DELETE FROM \"{}\" WHERE id = $1 AND organization_id = $2", table_name)
    };
    
    let result = sqlx::query(&query)
        .bind(id)
        .bind(org_id)
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
///
/// Scoped to the caller's organization to prevent cross-tenant access.
pub async fn get_transitions(
    State(state): State<Arc<AppState>>,
    Path((entity, id)): Path<(String, Uuid)>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    debug!("Getting transitions for record {} of entity {}", id, entity);

    let entity_def = state.schema_engine.get_entity(&entity)
        .ok_or(StatusCode::NOT_FOUND)?;

    let workflow = match &entity_def.workflow {
        Some(w) => w,
        None => return Ok(Json(serde_json::json!({"transitions": []}))),
    };

    // Look up current workflow state from DB, scoped to organization
    let table_name = entity_def.table_name.as_deref().unwrap_or(&entity);
    let table_name = sanitize_identifier(table_name)?;
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let row = sqlx::query(
        format!(
            "SELECT workflow_state FROM \"{}\" WHERE id = $1 AND organization_id = $2 AND deleted_at IS NULL",
            table_name
        ).as_str()
    )
    .bind(id)
    .bind(org_id)
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
///
/// Scoped to the caller's organization to prevent cross-tenant access.
pub async fn execute_action(
    State(state): State<Arc<AppState>>,
    Path((entity, id, action)): Path<(String, Uuid, String)>,
    claims: Extension<Claims>,
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

    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Fetch the current record, scoped to organization
    let row = sqlx::query(
        format!(
            "SELECT * FROM \"{}\" WHERE id = $1 AND organization_id = $2 AND deleted_at IS NULL",
            table_name
        ).as_str()
    )
    .bind(id)
    .bind(org_id)
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
            // Sanitize field names while preserving value lookup with original keys
            let original_keys: Vec<String> = obj.keys().cloned().collect();
            let sanitized_fields: Vec<String> = original_keys.iter()
                .map(|k| sanitize_identifier(k))
                .collect::<Result<Vec<_>, _>>()?;

            let set_clauses: Vec<String> = sanitized_fields.iter()
                .enumerate()
                .map(|(i, k)| format!("\"{}\" = ${}::text", k, i + 1))
                .collect();

            if !set_clauses.is_empty() {
                let update_query = format!(
                    "UPDATE \"{}\" SET {}, updated_at = now() WHERE id = ${}",
                    table_name,
                    set_clauses.join(", "),
                    set_clauses.len() + 1
                );
                let mut q = sqlx::query(&update_query);
                for key in &original_keys {
                    let value = obj.get(key).unwrap_or(&serde_json::Value::Null);
                    q = q.bind(json_to_text(value));
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
///
/// Scoped to the caller's organization to prevent cross-tenant access.
/// The audit entries are filtered by matching the entity_id against
/// records that belong to the caller's organization.
pub async fn get_record_history(
    State(state): State<Arc<AppState>>,
    Path((entity, id)): Path<(String, Uuid)>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    debug!("Getting history for record {} of entity {}", id, entity);

    // Verify the record belongs to the caller's organization before
    // returning audit data.  If the entity doesn't exist in the schema
    // or the record doesn't exist / is not in the caller's org, return 404.
    if let Some(entity_def) = state.schema_engine.get_entity(&entity) {
        let table_name = entity_def.table_name.as_deref().unwrap_or(&entity);
        if let Ok(safe_table) = sanitize_identifier(table_name) {
            let org_id = Uuid::parse_str(&claims.org_id)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let row = sqlx::query(
                format!(
                    "SELECT id FROM \"{}\" WHERE id = $1 AND organization_id = $2{}",
                    safe_table,
                    if entity_def.is_soft_delete { " AND deleted_at IS NULL" } else { "" }
                ).as_str()
            )
            .bind(id)
            .bind(org_id)
            .fetch_optional(&state.db_pool)
            .await
            .map_err(|e| {
                error!("Query error: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

            if row.is_none() {
                return Err(StatusCode::NOT_FOUND);
            }
        }
    }

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
