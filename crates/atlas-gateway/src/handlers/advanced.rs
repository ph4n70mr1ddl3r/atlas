//! Advanced Oracle Fusion-inspired feature handlers
//!
//! Structured filtering, bulk operations, comments, favorites,
//! CSV export, effective dating, and related records navigation.

use axum::{
    extract::{State, Path, Query},
    Json,
    http::StatusCode,
    Extension,
};
use serde::{Deserialize, Serialize};
use crate::AppState;
use crate::handlers::auth::Claims;
use crate::handlers::records::{sanitize_identifier, row_to_json, json_to_text};
use std::sync::Arc;
use uuid::Uuid;
use tracing::{info, debug, error};
use sqlx::Row;

// ============================================================================
// Structured / Advanced Filtering
// Oracle Fusion: Faceted search with AND/OR filter groups
// ============================================================================

/// A structured filter expression (supports nested AND/OR)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FilterExpression {
    /// A single field condition
    Condition {
        field: String,
        operator: String, // eq, ne, gt, gte, lt, lte, contains, starts_with, ends_with, is_null, is_not_null, in, not_in, between
        value: serde_json::Value,
    },
    /// All children must match
    And {
        conditions: Vec<FilterExpression>,
    },
    /// Any child must match
    Or {
        conditions: Vec<FilterExpression>,
    },
}

impl FilterExpression {
    /// Convert the filter expression tree into a SQL WHERE clause fragment.
    /// Returns (sql_fragment, bind_params_as_text).
    fn to_sql(&self, param_idx: &mut usize) -> (String, Vec<Option<String>>) {
        match self {
            FilterExpression::Condition { field, operator, value } => {
                let Ok(safe_field) = sanitize_identifier(field) else {
                    return ("1=0".to_string(), vec![]);
                };

                let op = operator.to_lowercase();
                match op.as_str() {
                    "eq" | "=" => {
                        *param_idx += 1;
                        (format!("\"{}\" = ${}::text", safe_field, *param_idx), vec![json_val_to_text(value)])
                    }
                    "ne" | "!=" => {
                        *param_idx += 1;
                        (format!("\"{}\" != ${}::text", safe_field, *param_idx), vec![json_val_to_text(value)])
                    }
                    "gt" | ">" => {
                        *param_idx += 1;
                        (format!("\"{}\" > ${}::text", safe_field, *param_idx), vec![json_val_to_text(value)])
                    }
                    "gte" | ">=" => {
                        *param_idx += 1;
                        (format!("\"{}\" >= ${}::text", safe_field, *param_idx), vec![json_val_to_text(value)])
                    }
                    "lt" | "<" => {
                        *param_idx += 1;
                        (format!("\"{}\" < ${}::text", safe_field, *param_idx), vec![json_val_to_text(value)])
                    }
                    "lte" | "<=" => {
                        *param_idx += 1;
                        (format!("\"{}\" <= ${}::text", safe_field, *param_idx), vec![json_val_to_text(value)])
                    }
                    "contains" | "like" => {
                        *param_idx += 1;
                        let pattern = value.as_str()
                            .map(|s| format!("%{}%", s.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_")))
                            .unwrap_or_default();
                        (format!("\"{}\"::text ILIKE ${}::text", safe_field, *param_idx), vec![Some(pattern)])
                    }
                    "starts_with" => {
                        *param_idx += 1;
                        let pattern = value.as_str()
                            .map(|s| format!("{}%", s.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_")))
                            .unwrap_or_default();
                        (format!("\"{}\"::text ILIKE ${}::text", safe_field, *param_idx), vec![Some(pattern)])
                    }
                    "ends_with" => {
                        *param_idx += 1;
                        let pattern = value.as_str()
                            .map(|s| format!("%{}", s.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_")))
                            .unwrap_or_default();
                        (format!("\"{}\"::text ILIKE ${}::text", safe_field, *param_idx), vec![Some(pattern)])
                    }
                    "is_null" => {
                        ("\"{}\" IS NULL".replace("{}", &safe_field), vec![])
                    }
                    "is_not_null" => {
                        ("\"{}\" IS NOT NULL".replace("{}", &safe_field), vec![])
                    }
                    "in" => {
                        if let Some(arr) = value.as_array() {
                            let mut parts = Vec::new();
                            let mut params = Vec::new();
                            for item in arr {
                                *param_idx += 1;
                                parts.push(format!("${}::text", *param_idx));
                                params.push(json_val_to_text(item));
                            }
                            if parts.is_empty() {
                                ("1=0".to_string(), vec![])
                            } else {
                                (format!("\"{}\" IN ({})", safe_field, parts.join(", ")), params)
                            }
                        } else {
                            ("1=0".to_string(), vec![])
                        }
                    }
                    "not_in" => {
                        if let Some(arr) = value.as_array() {
                            let mut parts = Vec::new();
                            let mut params = Vec::new();
                            for item in arr {
                                *param_idx += 1;
                                parts.push(format!("${}::text", *param_idx));
                                params.push(json_val_to_text(item));
                            }
                            if parts.is_empty() {
                                ("1=1".to_string(), vec![])
                            } else {
                                (format!("\"{}\" NOT IN ({})", safe_field, parts.join(", ")), params)
                            }
                        } else {
                            ("1=1".to_string(), vec![])
                        }
                    }
                    "between" => {
                        if let Some(arr) = value.as_array() {
                            if arr.len() >= 2 {
                                *param_idx += 1;
                                let p1 = *param_idx;
                                *param_idx += 1;
                                let p2 = *param_idx;
                                (format!("\"{}\" BETWEEN ${}::text AND ${}::text", safe_field, p1, p2),
                                 vec![json_val_to_text(&arr[0]), json_val_to_text(&arr[1])])
                            } else {
                                ("1=0".to_string(), vec![])
                            }
                        } else {
                            ("1=0".to_string(), vec![])
                        }
                    }
                    _ => ("1=1".to_string(), vec![]),
                }
            }
            FilterExpression::And { conditions } => {
                let mut parts = Vec::new();
                let mut all_params = Vec::new();
                for cond in conditions {
                    let (sql, params) = cond.to_sql(param_idx);
                    if !sql.is_empty() {
                        parts.push(sql);
                        all_params.extend(params);
                    }
                }
                if parts.is_empty() {
                    ("1=1".to_string(), vec![])
                } else if parts.len() == 1 {
                    (parts.into_iter().next().unwrap(), all_params)
                } else {
                    (format!("({})", parts.join(" AND ")), all_params)
                }
            }
            FilterExpression::Or { conditions } => {
                let mut parts = Vec::new();
                let mut all_params = Vec::new();
                for cond in conditions {
                    let (sql, params) = cond.to_sql(param_idx);
                    if !sql.is_empty() {
                        parts.push(sql);
                        all_params.extend(params);
                    }
                }
                if parts.is_empty() {
                    ("1=1".to_string(), vec![])
                } else if parts.len() == 1 {
                    (parts.into_iter().next().unwrap(), all_params)
                } else {
                    (format!("({})", parts.join(" OR ")), all_params)
                }
            }
        }
    }
}

fn json_val_to_text(v: &serde_json::Value) -> Option<String> {
    match v {
        serde_json::Value::Null => None,
        serde_json::Value::Bool(b) => Some(if *b { "true".into() } else { "false".into() }),
        serde_json::Value::Number(n) => Some(n.to_string()),
        serde_json::Value::String(s) => Some(s.clone()),
        other => Some(other.to_string()),
    }
}

#[derive(Debug, Deserialize)]
pub struct AdvancedListParams {
    pub search: Option<String>,
    pub offset: Option<i64>,
    pub limit: Option<i64>,
    pub sort: Option<String>,
    pub order: Option<String>,
    /// JSON-encoded FilterExpression tree
    pub filter: Option<String>,
}

/// List records with advanced structured filtering
pub async fn list_records_advanced(
    State(state): State<Arc<AppState>>,
    Path(entity): Path<String>,
    claims: Extension<Claims>,
    Query(params): Query<AdvancedListParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    debug!("Advanced listing records for entity: {}", entity);

    let entity_def = state.schema_engine.get_entity(&entity)
        .ok_or(StatusCode::NOT_FOUND)?;

    let table_name = sanitize_identifier(
        entity_def.table_name.as_deref().unwrap_or(&entity)
    )?;

    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let offset = params.offset.unwrap_or(0).max(0);
    let limit = params.limit.unwrap_or(20).clamp(1, 200);

    // Build base WHERE with org + soft delete
    let mut where_parts = vec![
        "organization_id = $1::uuid".to_string(),
    ];
    if entity_def.is_soft_delete {
        where_parts.push("deleted_at IS NULL".to_string());
    }

    let mut param_idx = 1; // $1 = org_id
    let mut bind_params: Vec<Option<String>> = vec![Some(org_id.to_string())];

    // Add text search if provided
    if let Some(ref search) = params.search {
        if !search.is_empty() {
            let escaped = search.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_");
            let pattern = format!("%{}%", escaped);
            let fields: Vec<String> = entity_def.fields.iter()
                .filter(|f| f.is_searchable && matches!(
                    f.field_type,
                    atlas_shared::FieldType::String { .. } | atlas_shared::FieldType::Email | atlas_shared::FieldType::Phone
                ))
                .map(|f| format!("\"{}\"::text ILIKE ", f.name))
                .collect();
            if !fields.is_empty() {
                param_idx += 1;
                where_parts.push(format!(
                    "({})",
                    fields.iter()
                        .map(|f| format!("{} ${}::text", f, param_idx))
                        .collect::<Vec<_>>()
                        .join(" OR ")
                ));
                bind_params.push(Some(pattern));
            }
        }
    }

    // Add structured filter if provided
    if let Some(ref filter_json) = params.filter {
        if let Ok(filter_expr) = serde_json::from_str::<FilterExpression>(filter_json) {
            let (sql, params) = filter_expr.to_sql(&mut param_idx);
            if sql != "1=1" {
                where_parts.push(format!("({})", sql));
                bind_params.extend(params);
            }
        }
    }

    let where_clause = where_parts.join(" AND ");

    // ORDER BY
    let order_clause = match &params.sort {
        Some(field) if !field.is_empty() => {
            let safe = sanitize_identifier(field).unwrap_or_else(|_| "created_at".to_string());
            let dir = match params.order.as_deref() {
                Some("asc") | Some("ASC") => "ASC",
                Some("desc") | Some("DESC") => "DESC",
                _ => "DESC",
            };
            format!("ORDER BY \"{}\" {}", safe, dir)
        }
        _ => "ORDER BY created_at DESC".to_string(),
    };

    let sql = format!(
        "SELECT * FROM \"{}\" WHERE {} {} LIMIT ${} OFFSET ${}",
        table_name, where_clause, order_clause, param_idx + 1, param_idx + 2
    );

    let mut query = sqlx::query(&sql);
    for param in &bind_params {
        query = query.bind(param.as_deref());
    }
    query = query.bind(limit);
    query = query.bind(offset);

    let rows = query.fetch_all(&state.db_pool).await
        .map_err(|e| { error!("Advanced query error: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;

    let records: Vec<serde_json::Value> = rows.iter().map(row_to_json).collect();

    // Count query
    let count_sql = format!("SELECT COUNT(*) FROM \"{}\" WHERE {}", table_name, where_clause);
    let mut count_query = sqlx::query_scalar::<_, i64>(&count_sql);
    for param in &bind_params {
        count_query = count_query.bind(param.as_deref());
    }
    let total = count_query.fetch_one(&state.db_pool).await
        .map_err(|e| { error!("Count error: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;

    Ok(Json(serde_json::json!({
        "data": records,
        "meta": {
            "total": total,
            "offset": offset,
            "limit": limit,
        }
    })))
}

// ============================================================================
// Bulk Operations
// Oracle Fusion: Mass update, delete, workflow action
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct BulkOperationRequest {
    pub entity_type: String,
    pub operation: String, // "update", "delete", "workflow_action"
    /// Optional filter expression to select records
    pub filter: Option<FilterExpression>,
    /// Explicit list of record IDs (alternative to filter)
    pub record_ids: Option<Vec<Uuid>>,
    /// Operation payload: {"values": {...}} for update, {"action": "..."} for workflow
    pub payload: serde_json::Value,
    /// If true, return a preview without executing
    #[serde(default)]
    pub dry_run: bool,
}

#[derive(Debug, Serialize)]
pub struct BulkOperationResponse {
    pub id: Uuid,
    pub operation: String,
    pub status: String,
    pub total_records: i32,
    pub succeeded: i32,
    pub failed: i32,
    pub errors: Vec<BulkError>,
    pub is_dry_run: bool,
}

#[derive(Debug, Serialize)]
pub struct BulkError {
    pub record_id: String,
    pub error: String,
}

/// Execute a bulk operation
pub async fn execute_bulk_operation(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<BulkOperationRequest>,
) -> Result<Json<BulkOperationResponse>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let entity_def = state.schema_engine.get_entity(&payload.entity_type)
        .ok_or(StatusCode::NOT_FOUND)?;

    let table_name = sanitize_identifier(
        entity_def.table_name.as_deref().unwrap_or(&payload.entity_type)
    )?;

    info!("Bulk {} on {} by {}", payload.operation, payload.entity_type, user_id);

    // Collect target record IDs
    let record_ids = if let Some(ref ids) = payload.record_ids {
        ids.clone()
    } else if let Some(ref filter) = payload.filter {
        // Use filter to find matching records
        let mut param_idx = 1;
        let mut bind_params: Vec<Option<String>> = vec![Some(org_id.to_string())];
        let mut where_parts = vec!["organization_id = $1::uuid".to_string()];
        if entity_def.is_soft_delete {
            where_parts.push("deleted_at IS NULL".to_string());
        }
        let (filter_sql, filter_params) = filter.to_sql(&mut param_idx);
        where_parts.push(format!("({})", filter_sql));
        bind_params.extend(filter_params);

        let sql = format!("SELECT id FROM \"{}\" WHERE {}", table_name, where_parts.join(" AND "));
        let mut query = sqlx::query(&sql);
        for param in &bind_params {
            query = query.bind(param.as_deref());
        }
        let rows = query.fetch_all(&state.db_pool).await
            .map_err(|e| { error!("Bulk query error: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;
        rows.iter()
            .filter_map(|r| r.try_get::<Uuid, _>("id").ok())
            .collect()
    } else {
        return Err(StatusCode::BAD_REQUEST);
    };

    let total = record_ids.len() as i32;

    if payload.dry_run {
        return Ok(Json(BulkOperationResponse {
            id: Uuid::new_v4(),
            operation: payload.operation.clone(),
            status: "preview".to_string(),
            total_records: total,
            succeeded: 0,
            failed: 0,
            errors: vec![],
            is_dry_run: true,
        }));
    }

    // Create job tracking record
    let job_row = sqlx::query(
        r#"INSERT INTO _atlas.bulk_operations
            (organization_id, user_id, entity_type, operation, filter, record_ids,
             payload, status, total_records, is_dry_run)
        VALUES ($1, $2, $3, $4, $5, $6, $7, 'running', $8, $9)
        RETURNING id"#
    )
    .bind(org_id)
    .bind(user_id)
    .bind(&payload.entity_type)
    .bind(&payload.operation)
    .bind(serde_json::to_value(&payload.filter).unwrap_or(serde_json::Value::Null))
    .bind(serde_json::to_value(&record_ids).unwrap_or(serde_json::json!([])))
    .bind(&payload.payload)
    .bind(total)
    .bind(payload.dry_run)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| { error!("Bulk job create error: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;

    let job_id: Uuid = job_row.try_get("id").unwrap_or_default();

    let mut succeeded = 0i32;
    let mut failed = 0i32;
    let mut errors = Vec::new();

    match payload.operation.as_str() {
        "delete" => {
            for rid in &record_ids {
                let q = if entity_def.is_soft_delete {
                    format!("UPDATE \"{}\" SET deleted_at = now() WHERE id = $1 AND organization_id = $2", table_name)
                } else {
                    format!("DELETE FROM \"{}\" WHERE id = $1 AND organization_id = $2", table_name)
                };
                match sqlx::query(&q).bind(rid).bind(org_id).execute(&state.db_pool).await {
                    Ok(r) if r.rows_affected() > 0 => succeeded += 1,
                    Ok(_) => { failed += 1; errors.push(BulkError { record_id: rid.to_string(), error: "Not found".into() }); }
                    Err(e) => { failed += 1; errors.push(BulkError { record_id: rid.to_string(), error: e.to_string() }); }
                }
            }
        }
        "update" => {
            let values = payload.payload.get("values")
                .and_then(|v| v.as_object())
                .cloned()
                .unwrap_or_default();

            let non_null: Vec<(String, &serde_json::Value)> = values.iter()
                .filter(|(_, v)| !v.is_null())
                .map(|(k, v)| sanitize_identifier(k).map(|sk| (sk, v)))
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| StatusCode::BAD_REQUEST)?;

            if non_null.is_empty() {
                return Err(StatusCode::BAD_REQUEST);
            }

            for rid in &record_ids {
                let set_clauses: Vec<String> = non_null.iter()
                    .enumerate()
                    .map(|(i, (k, _))| format!("\"{}\" = ${}::text", k, i + 1))
                    .collect();
                let q = format!(
                    "UPDATE \"{}\" SET {}, updated_at = now() WHERE id = ${} AND organization_id = ${}",
                    table_name,
                    set_clauses.join(", "),
                    non_null.len() + 1,
                    non_null.len() + 2
                );
                let mut query = sqlx::query(&q);
                for (_, v) in &non_null {
                    query = query.bind(json_to_text(v));
                }
                query = query.bind(rid).bind(org_id);
                match query.execute(&state.db_pool).await {
                    Ok(r) if r.rows_affected() > 0 => succeeded += 1,
                    Ok(_) => { failed += 1; errors.push(BulkError { record_id: rid.to_string(), error: "Not found".into() }); }
                    Err(e) => { failed += 1; errors.push(BulkError { record_id: rid.to_string(), error: e.to_string() }); }
                }
            }
        }
        "workflow_action" => {
            let action = payload.payload.get("action")
                .and_then(|a| a.as_str())
                .unwrap_or("")
                .to_string();
            for rid in &record_ids {
                let q = format!(
                    "UPDATE \"{}\" SET workflow_state = $1::text, updated_at = now() WHERE id = $2 AND organization_id = $3 AND deleted_at IS NULL",
                    table_name
                );
                match sqlx::query(&q).bind(&action).bind(rid).bind(org_id).execute(&state.db_pool).await {
                    Ok(r) if r.rows_affected() > 0 => succeeded += 1,
                    Ok(_) => { failed += 1; errors.push(BulkError { record_id: rid.to_string(), error: "Not found or no workflow".into() }); }
                    Err(e) => { failed += 1; errors.push(BulkError { record_id: rid.to_string(), error: e.to_string() }); }
                }
            }
        }
        _ => return Err(StatusCode::BAD_REQUEST),
    }

    // Update job
    let status = if failed == 0 { "completed" } else if succeeded == 0 { "failed" } else { "completed" };
    let _ = sqlx::query(
        "UPDATE _atlas.bulk_operations SET status = $1, processed_records = $2, succeeded_records = $3, failed_records = $4, completed_at = now() WHERE id = $5"
    )
    .bind(status)
    .bind(total)
    .bind(succeeded)
    .bind(failed)
    .bind(job_id)
    .execute(&state.db_pool)
    .await;

    Ok(Json(BulkOperationResponse {
        id: job_id,
        operation: payload.operation.clone(),
        status: status.to_string(),
        total_records: total,
        succeeded,
        failed,
        errors,
        is_dry_run: false,
    }))
}

// ============================================================================
// Comments / Notes
// Oracle Fusion: Conversation threads on business objects
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateCommentRequest {
    pub body: String,
    #[serde(default)]
    pub body_format: Option<String>,
    pub parent_id: Option<Uuid>,
    #[serde(default)]
    pub is_internal: bool,
    pub mentions: Option<Vec<Uuid>>,
}

/// List comments for a record
pub async fn list_comments(
    State(state): State<Arc<AppState>>,
    Path((entity, id)): Path<(String, Uuid)>,
    claims: Extension<Claims>,
    Query(params): Query<CommentListParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let limit = params.limit.unwrap_or(50).clamp(1, 200);
    let offset = params.offset.unwrap_or(0).max(0);

    let rows = sqlx::query(
        r#"SELECT c.*, u.name as author_name
           FROM _atlas.comments c
           LEFT JOIN _atlas.users u ON c.user_id = u.id
           WHERE c.organization_id = $1 AND c.entity_type = $2 AND c.entity_id = $3
             AND c.deleted_at IS NULL
           ORDER BY c.is_pinned DESC, c.created_at ASC
           LIMIT $4 OFFSET $5"#
    )
    .bind(org_id)
    .bind(&entity)
    .bind(id)
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| { error!("Comments query error: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;

    let comments: Vec<serde_json::Value> = rows.iter().map(row_to_json).collect();

    // Get count
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM _atlas.comments WHERE organization_id = $1 AND entity_type = $2 AND entity_id = $3 AND deleted_at IS NULL"
    )
    .bind(org_id).bind(&entity).bind(id)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| { error!("Comment count error: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;

    Ok(Json(serde_json::json!({
        "data": comments,
        "meta": { "total": count, "offset": offset, "limit": limit }
    })))
}

#[derive(Debug, Deserialize)]
pub struct CommentListParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Add a comment to a record
pub async fn create_comment(
    State(state): State<Arc<AppState>>,
    Path((entity, id)): Path<(String, Uuid)>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateCommentRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Verify the record exists
    let entity_def = state.schema_engine.get_entity(&entity)
        .ok_or(StatusCode::NOT_FOUND)?;
    let table_name = sanitize_identifier(
        entity_def.table_name.as_deref().unwrap_or(&entity)
    )?;
    let exists: bool = sqlx::query_scalar(
        &format!("SELECT EXISTS(SELECT 1 FROM \"{}\" WHERE id = $1 AND organization_id = $2{})",
            table_name,
            if entity_def.is_soft_delete { " AND deleted_at IS NULL" } else { "" }
        )
    )
    .bind(id).bind(org_id)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !exists {
        return Err(StatusCode::NOT_FOUND);
    }

    // Determine depth and thread root
    let (depth, thread_root_id) = if let Some(parent_id) = payload.parent_id {
        let parent: Option<(i32, Option<Uuid>)> = sqlx::query_as(
            "SELECT depth, COALESCE(thread_root_id, id) as thread_root_id FROM _atlas.comments WHERE id = $1 AND entity_type = $2 AND entity_id = $3 AND deleted_at IS NULL"
        )
        .bind(parent_id).bind(&entity).bind(id)
        .fetch_optional(&state.db_pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        match parent {
            Some((d, root)) => (d + 1, root),
            None => return Err(StatusCode::BAD_REQUEST),
        }
    } else {
        (0, None)
    };

    let row = sqlx::query(
        r#"INSERT INTO _atlas.comments
            (organization_id, entity_type, entity_id, parent_id, user_id, user_name,
             body, body_format, mentions, thread_root_id, depth, is_internal)
        VALUES ($1, $2, $3, $4, $5, 
                (SELECT name FROM _atlas.users WHERE id = $5),
                $6, $7, $8, $9, $10, $11)
        RETURNING *"#
    )
    .bind(org_id).bind(&entity).bind(id).bind(payload.parent_id)
    .bind(user_id)
    .bind(&payload.body)
    .bind(payload.body_format.unwrap_or_else(|| "plain".to_string()))
    .bind(serde_json::to_value(&payload.mentions).unwrap_or(serde_json::json!([])))
    .bind(thread_root_id)
    .bind(depth)
    .bind(payload.is_internal)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| { error!("Create comment error: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;

    Ok((StatusCode::CREATED, Json(row_to_json(&row))))
}

/// Delete a comment (soft delete)
pub async fn delete_comment(
    State(state): State<Arc<AppState>>,
    Path((_entity, _record_id, comment_id)): Path<(String, Uuid, Uuid)>,
    claims: Extension<Claims>,
) -> Result<StatusCode, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let result = sqlx::query(
        "UPDATE _atlas.comments SET deleted_at = now() WHERE id = $1 AND user_id = $2 AND deleted_at IS NULL"
    )
    .bind(comment_id).bind(user_id)
    .execute(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(StatusCode::NO_CONTENT)
}

/// Pin/unpin a comment
pub async fn toggle_pin_comment(
    State(state): State<Arc<AppState>>,
    Path((_entity, _record_id, comment_id)): Path<(String, Uuid, Uuid)>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let _org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let row = sqlx::query(
        "UPDATE _atlas.comments SET is_pinned = NOT is_pinned, updated_at = now() WHERE id = $1 RETURNING id, is_pinned"
    )
    .bind(comment_id)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match row {
        Some(r) => Ok(Json(row_to_json(&r))),
        None => Err(StatusCode::NOT_FOUND),
    }
}

// ============================================================================
// Favorites / Bookmarks
// Oracle Fusion: Quick access to important records
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateFavoriteRequest {
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

/// List current user's favorites
pub async fn list_favorites(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<FavoriteListParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut query_str = "SELECT * FROM _atlas.favorites WHERE organization_id = $1 AND user_id = $2".to_string();
    let mut bind_count = 2;

    if let Some(ref _entity_type) = params.entity_type {
        bind_count += 1;
        query_str.push_str(&format!(" AND entity_type = ${}", bind_count));
    }
    query_str.push_str(" ORDER BY display_order, created_at DESC");

    let mut query = sqlx::query(&query_str).bind(org_id).bind(user_id);
    if let Some(ref et) = params.entity_type {
        query = query.bind(et);
    }

    let rows = query.fetch_all(&state.db_pool).await
        .map_err(|e| { error!("Favorites query error: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;

    let favorites: Vec<serde_json::Value> = rows.iter().map(row_to_json).collect();

    Ok(Json(serde_json::json!({ "data": favorites })))
}

#[derive(Debug, Deserialize)]
pub struct FavoriteListParams {
    pub entity_type: Option<String>,
}

/// Add a record to favorites
pub async fn add_favorite(
    State(state): State<Arc<AppState>>,
    Path((entity, id)): Path<(String, Uuid)>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateFavoriteRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let row = sqlx::query(
        r#"INSERT INTO _atlas.favorites (organization_id, user_id, entity_type, entity_id, label, notes)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (user_id, entity_type, entity_id) DO UPDATE SET label = $5, notes = $6
        RETURNING *"#
    )
    .bind(org_id).bind(user_id).bind(&entity).bind(id)
    .bind(&payload.label).bind(&payload.notes)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| { error!("Add favorite error: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;

    Ok((StatusCode::CREATED, Json(row_to_json(&row))))
}

/// Remove a record from favorites
pub async fn remove_favorite(
    State(state): State<Arc<AppState>>,
    Path((entity, id)): Path<(String, Uuid)>,
    claims: Extension<Claims>,
) -> Result<StatusCode, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    sqlx::query("DELETE FROM _atlas.favorites WHERE user_id = $1 AND entity_type = $2 AND entity_id = $3")
        .bind(user_id).bind(&entity).bind(id)
        .execute(&state.db_pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

/// Check if a record is favorited by the current user
pub async fn check_favorite(
    State(state): State<Arc<AppState>>,
    Path((entity, id)): Path<(String, Uuid)>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let row = sqlx::query(
        "SELECT id, label, notes FROM _atlas.favorites WHERE user_id = $1 AND entity_type = $2 AND entity_id = $3"
    )
    .bind(user_id).bind(&entity).bind(id)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match row {
        Some(r) => Ok(Json(serde_json::json!({ "is_favorite": true, "favorite": row_to_json(&r) }))),
        None => Ok(Json(serde_json::json!({ "is_favorite": false }))),
    }
}

// ============================================================================
// CSV Export
// Oracle Fusion: Export data in multiple formats
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CsvExportParams {
    /// Optional filter expression
    pub filter: Option<String>,
    /// Comma-separated list of fields to include (defaults to all searchable)
    pub fields: Option<String>,
    pub delimiter: Option<String>,
}

/// Export entity data as CSV
pub async fn export_csv(
    State(state): State<Arc<AppState>>,
    Path(entity): Path<String>,
    claims: Extension<Claims>,
    Query(params): Query<CsvExportParams>,
) -> Result<axum::response::Response, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let entity_def = state.schema_engine.get_entity(&entity)
        .ok_or(StatusCode::NOT_FOUND)?;

    let table_name = sanitize_identifier(
        entity_def.table_name.as_deref().unwrap_or(&entity)
    )?;

    // Determine fields to export
    let fields: Vec<String> = if let Some(fields_str) = &params.fields {
        fields_str.split(',')
            .filter_map(|f| {
                let f = f.trim().to_string();
                if !f.is_empty() { Some(f) } else { None }
            })
            .collect()
    } else {
        entity_def.fields.iter()
            .filter(|f| f.is_searchable)
            .map(|f| f.name.clone())
            .collect()
    };

    if fields.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Build WHERE
    let mut where_parts = vec!["organization_id = $1::uuid".to_string()];
    if entity_def.is_soft_delete {
        where_parts.push("deleted_at IS NULL".to_string());
    }
    let mut param_idx = 1;
    let mut bind_params: Vec<Option<String>> = vec![Some(org_id.to_string())];

    if let Some(ref filter_json) = params.filter {
        if let Ok(filter_expr) = serde_json::from_str::<FilterExpression>(filter_json) {
            let (sql, params) = filter_expr.to_sql(&mut param_idx);
            if sql != "1=1" {
                where_parts.push(format!("({})", sql));
                bind_params.extend(params);
            }
        }
    }

    let where_clause = where_parts.join(" AND ");
    let select_fields = fields.iter()
        .map(|f| format!("\"{}\"", sanitize_identifier(f).unwrap_or_default()))
        .collect::<Vec<_>>()
        .join(", ");

    let sql = format!(
        "SELECT id, {} FROM \"{}\" WHERE {} ORDER BY created_at DESC",
        select_fields, table_name, where_clause
    );

    let mut query = sqlx::query(&sql);
    for param in &bind_params {
        query = query.bind(param.as_deref());
    }

    let rows = query.fetch_all(&state.db_pool).await
        .map_err(|e| { error!("CSV export query error: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;

    // Build CSV
    let delimiter = params.delimiter.as_deref().unwrap_or(",");
    let mut csv_lines = Vec::new();

    // Header
    let mut headers = vec!["id".to_string()];
    headers.extend(fields.iter().cloned());
    csv_lines.push(headers.iter()
        .map(|h| csv_escape(h))
        .collect::<Vec<_>>()
        .join(delimiter));

    // Data rows
    for row in &rows {
        let id: Uuid = row.try_get("id").unwrap_or_default();
        let mut values = vec![csv_escape(&id.to_string())];
        for field in &fields {
            let safe = sanitize_identifier(field).unwrap_or_default();
            let val = row.try_get::<serde_json::Value, _>(safe.as_str())
                .or_else(|_| row.try_get::<String, _>(safe.as_str()).map(|s| serde_json::json!(s)))
                .or_else(|_| row.try_get::<i64, _>(safe.as_str()).map(|n| serde_json::json!(n)))
                .or_else(|_| row.try_get::<bool, _>(safe.as_str()).map(|b| serde_json::json!(b)))
                .unwrap_or(serde_json::Value::Null);
            values.push(csv_escape(&json_to_csv_string(&val)));
        }
        csv_lines.push(values.join(delimiter));
    }

    let csv_body = csv_lines.join("\n");
    let filename = format!("{}_export_{}.csv", entity, chrono::Utc::now().format("%Y%m%d_%H%M%S"));

    use axum::response::IntoResponse;
    Ok((
        axum::http::StatusCode::OK,
        [
            ("content-type", "text/csv".to_string()),
            ("content-disposition", format!("attachment; filename=\"{}\"", filename)),
        ],
        csv_body,
    ).into_response())
}

fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') || s.contains('\r') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

fn json_to_csv_string(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::Null => String::new(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

// ============================================================================
// Related Records
// Oracle Fusion: Navigate relationships between entities
// ============================================================================

/// Get related records for a parent entity
/// e.g., GET /api/v1/purchase_orders/{id}/related/lines
pub async fn get_related_records(
    State(state): State<Arc<AppState>>,
    Path((entity, id, related_entity)): Path<(String, Uuid, String)>,
    claims: Extension<Claims>,
    Query(params): Query<RelatedRecordsParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let entity_def = state.schema_engine.get_entity(&entity)
        .ok_or(StatusCode::NOT_FOUND)?;

    // Find the relationship field
    let related_field = entity_def.fields.iter()
        .find(|f| {
            match &f.field_type {
                atlas_shared::FieldType::OneToMany { entity, foreign_key: _ } if entity == &related_entity => true,
                atlas_shared::FieldType::Reference { entity, field: _ } if entity == &related_entity => true,
                _ => false,
            }
        });

    let (related_table, foreign_key) = match related_field {
        Some(field) => match &field.field_type {
            atlas_shared::FieldType::OneToMany { entity: rel_entity, foreign_key } => {
                // Look up the related entity definition to get its table name
                if let Some(rel_def) = state.schema_engine.get_entity(rel_entity) {
                    let tbl = rel_def.table_name.as_deref().unwrap_or(rel_entity);
                    (sanitize_identifier(tbl)?, foreign_key.clone())
                } else {
                    (sanitize_identifier(rel_entity)?, foreign_key.clone())
                }
            }
            _ => return Err(StatusCode::BAD_REQUEST),
        },
        None => {
            // Try convention: {entity_singular}_id as foreign key
            let fk = format!("{}_id", entity.trim_end_matches('s'));
            if let Some(rel_def) = state.schema_engine.get_entity(&related_entity) {
                let tbl = rel_def.table_name.as_deref().unwrap_or(&related_entity);
                (sanitize_identifier(tbl)?, fk)
            } else {
                (sanitize_identifier(&related_entity)?, fk)
            }
        }
    };

    let limit = params.limit.unwrap_or(50).clamp(1, 200);
    let offset = params.offset.unwrap_or(0).max(0);

    let safe_fk = sanitize_identifier(&foreign_key).map_err(|_| StatusCode::BAD_REQUEST)?;

    let rows = sqlx::query(
        &format!(
            "SELECT * FROM \"{}\" WHERE \"{}\" = $1 AND organization_id = $2 AND deleted_at IS NULL ORDER BY created_at ASC LIMIT $3 OFFSET $4",
            related_table, safe_fk
        )
    )
    .bind(id).bind(org_id).bind(limit).bind(offset)
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| { error!("Related records query error: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;

    let records: Vec<serde_json::Value> = rows.iter().map(row_to_json).collect();

    // Count
    let total: i64 = sqlx::query_scalar(
        &format!(
            "SELECT COUNT(*) FROM \"{}\" WHERE \"{}\" = $1 AND organization_id = $2 AND deleted_at IS NULL",
            related_table, safe_fk
        )
    )
    .bind(id).bind(org_id)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({
        "data": records,
        "meta": {
            "parent_entity": entity,
            "parent_id": id,
            "related_entity": related_entity,
            "foreign_key": foreign_key,
            "total": total,
            "offset": offset,
            "limit": limit,
        }
    })))
}

#[derive(Debug, Deserialize)]
pub struct RelatedRecordsParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

// ============================================================================
// Effective Dating
// Oracle Fusion: Temporal data with effective date ranges
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct GetEffectiveRecordParams {
    /// The date to retrieve the record as-of (defaults to today)
    pub as_of_date: Option<String>,
    /// Include all versions (history)
    #[serde(default)]
    pub include_history: bool,
}

/// Get an effective-dated version of a record
pub async fn get_effective_record(
    State(state): State<Arc<AppState>>,
    Path((entity, id)): Path<(String, Uuid)>,
    claims: Extension<Claims>,
    Query(params): Query<GetEffectiveRecordParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if params.include_history {
        // Return all versions
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.effective_dated_records
            WHERE organization_id = $1 AND entity_type = $2 AND base_record_id = $3
            ORDER BY effective_from DESC"#
        )
        .bind(org_id).bind(&entity).bind(id)
        .fetch_all(&state.db_pool)
        .await
        .map_err(|e| { error!("Effective history error: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;

        let versions: Vec<serde_json::Value> = rows.iter().map(row_to_json).collect();
        return Ok(Json(serde_json::json!({
            "entity_type": entity,
            "base_record_id": id,
            "versions": versions,
        })));
    }

    // Get as-of date version
    let as_of_date = params.as_of_date
        .unwrap_or_else(|| chrono::Utc::now().format("%Y-%m-%d").to_string());

    let row = sqlx::query(
        r#"SELECT * FROM _atlas.effective_dated_records
        WHERE organization_id = $1 AND entity_type = $2 AND base_record_id = $3
          AND effective_from <= $4::date
          AND (effective_to IS NULL OR effective_to >= $4::date)
        ORDER BY effective_from DESC LIMIT 1"#
    )
    .bind(org_id).bind(&entity).bind(id).bind(&as_of_date)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| { error!("Effective record error: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;

    match row {
        Some(r) => Ok(Json(row_to_json(&r))),
        None => Err(StatusCode::NOT_FOUND),
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateEffectiveVersionRequest {
    pub effective_from: String,
    pub effective_to: Option<String>,
    pub data: serde_json::Value,
    pub change_reason: Option<String>,
}

/// Create a new effective-dated version of a record
pub async fn create_effective_version(
    State(state): State<Arc<AppState>>,
    Path((entity, id)): Path<(String, Uuid)>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateEffectiveVersionRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Get current max version
    let max_version: i32 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(version), 0) FROM _atlas.effective_dated_records WHERE entity_type = $1 AND base_record_id = $2"
    )
    .bind(&entity).bind(id)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let new_version = max_version + 1;

    // Close any currently-open records that overlap
    sqlx::query(
        r#"UPDATE _atlas.effective_dated_records
        SET effective_to = ($4::date - INTERVAL '1 day')::date, is_current = false, updated_at = now()
        WHERE entity_type = $1 AND base_record_id = $2 AND is_current = true
          AND effective_to IS NULL"#
    )
    .bind(&entity).bind(id).bind(org_id).bind(&payload.effective_from)
    .execute(&state.db_pool)
    .await
    .map_err(|e| { error!("Close effective version error: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;

    let row = sqlx::query(
        r#"INSERT INTO _atlas.effective_dated_records
            (organization_id, entity_type, base_record_id, effective_from, effective_to,
             data, change_reason, changed_by, version, is_current)
        VALUES ($1, $2, $3, $4::date, $5::date, $6, $7, $8, $9, true)
        RETURNING *"#
    )
    .bind(org_id).bind(&entity).bind(id)
    .bind(&payload.effective_from)
    .bind(payload.effective_to.as_deref().unwrap_or("9999-12-31"))
    .bind(&payload.data)
    .bind(&payload.change_reason)
    .bind(user_id)
    .bind(new_version)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| { error!("Create effective version error: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;

    info!("Created effective version {} for {} {}", new_version, entity, id);

    Ok((StatusCode::CREATED, Json(row_to_json(&row))))
}

// ============================================================================
// CSV Import with Field Mapping
// Oracle Fusion: Import CSV with column mapping, validation, preview
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CsvImportRequest {
    pub entity: String,
    /// Raw CSV content (first row = headers)
    pub csv_content: String,
    /// Column to field mapping: { "csv_column_name": "entity_field_name" }
    pub field_mapping: serde_json::Value,
    #[serde(default)]
    pub upsert_mode: bool,
    #[serde(default)]
    pub skip_validation: bool,
    #[serde(default)]
    pub stop_on_error: bool,
    /// Delimiter character
    #[serde(default = "default_delimiter")]
    pub delimiter: char,
}

fn default_delimiter() -> char { ',' }

#[derive(Debug, Serialize)]
pub struct CsvImportResponse {
    pub entity: String,
    pub total_rows: usize,
    pub imported: usize,
    pub failed: usize,
    pub skipped: usize,
    pub errors: Vec<CsvImportError>,
    pub preview: Option<CsvImportPreview>,
}

#[derive(Debug, Serialize)]
pub struct CsvImportError {
    pub row: usize,
    pub errors: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct CsvImportPreview {
    pub columns: Vec<String>,
    pub mapped_fields: Vec<(String, String)>,
    pub sample_rows: Vec<Vec<String>>,
    pub unmapped_columns: Vec<String>,
}

/// Import data from CSV content
pub async fn import_csv(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CsvImportRequest>,
) -> Result<Json<CsvImportResponse>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let entity_def = state.schema_engine.get_entity(&payload.entity)
        .ok_or(StatusCode::NOT_FOUND)?;

    let table_name = sanitize_identifier(
        entity_def.table_name.as_deref().unwrap_or(&payload.entity)
    )?;

    // Parse CSV
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(payload.delimiter as u8)
        .has_headers(true)
        .from_reader(payload.csv_content.as_bytes());

    let headers = reader.headers()
        .map(|h| h.iter().map(|s| s.to_string()).collect::<Vec<_>>())
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // Build field mapping
    let mapping: std::collections::HashMap<String, String> = if payload.field_mapping.is_object() {
        payload.field_mapping.as_object().unwrap()
            .iter()
            .map(|(k, v)| (k.clone(), v.as_str().unwrap_or(k).to_string()))
            .collect()
    } else {
        // Auto-map: csv header name = field name
        headers.iter()
            .map(|h| (h.clone(), h.clone()))
            .collect()
    };

    // Collect rows
    let mut all_rows: Vec<Vec<String>> = Vec::new();
    for result in reader.records() {
        match result {
            Ok(record) => {
                all_rows.push(record.iter().map(|s| s.to_string()).collect());
            }
            Err(e) => {
                return Ok(Json(CsvImportResponse {
                    entity: payload.entity.clone(),
                    total_rows: 0,
                    imported: 0,
                    failed: 0,
                    skipped: 0,
                    errors: vec![CsvImportError { row: 0, errors: vec![format!("CSV parse error: {}", e)] }],
                    preview: None,
                }));
            }
        }
    }

    let total_rows = all_rows.len();

    // Map csv columns to entity fields
    let mapped_fields: Vec<(String, String)> = headers.iter()
        .filter_map(|h| mapping.get(h).map(|f| (h.clone(), f.clone())))
        .collect();

    let unmapped: Vec<String> = headers.iter()
        .filter(|h| !mapping.contains_key(*h))
        .cloned()
        .collect();

    let preview = CsvImportPreview {
        columns: headers.clone(),
        mapped_fields: mapped_fields.clone(),
        sample_rows: all_rows.iter().take(3).cloned().collect(),
        unmapped_columns: unmapped,
    };

    // If preview only (empty data), return preview
    if total_rows == 0 {
        return Ok(Json(CsvImportResponse {
            entity: payload.entity,
            total_rows: 0,
            imported: 0,
            failed: 0,
            skipped: 0,
            errors: vec![],
            preview: Some(preview),
        }));
    }

    // Build SQL for insert
    let field_names: Vec<&str> = mapped_fields.iter().map(|(_, f)| f.as_str()).collect();
    let safe_fields: Vec<String> = field_names.iter()
        .map(|f| sanitize_identifier(f))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    if safe_fields.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let col_list = safe_fields.iter()
        .map(|f| format!("\"{}\"", f))
        .collect::<Vec<_>>()
        .join(", ");
    let col_list_with_org = format!("{}, \"organization_id\"", col_list);

    let mut imported = 0usize;
    let mut failed = 0usize;
    let mut skipped = 0usize;
    let mut errors = Vec::new();

    for (row_idx, row) in all_rows.iter().enumerate() {
        // Build values from row using mapping
        let mut values = Vec::new();
        let mut row_errors = Vec::new();

        for (csv_col, field_name) in &mapped_fields {
            let col_idx = headers.iter().position(|h| h == csv_col);
            let empty = String::new();
            let val = col_idx.and_then(|i| row.get(i)).unwrap_or(&empty);

            // Validate required fields
            if let Some(field_def) = entity_def.fields.iter().find(|f| &f.name == field_name) {
                if field_def.is_required && val.trim().is_empty() && !payload.skip_validation {
                    row_errors.push(format!("{} is required", field_def.label));
                }
            }

            values.push(if val.is_empty() { None } else { Some(val.clone()) });
        }

        if !row_errors.is_empty() {
            failed += 1;
            errors.push(CsvImportError { row: row_idx + 2, errors: row_errors }); // +2 for header + 1-based
            if payload.stop_on_error {
                break;
            }
            continue;
        }

        let placeholders: Vec<String> = (1..=values.len())
            .map(|i| format!("${}::text", i))
            .collect();
        let org_placeholder = format!("${}::uuid", values.len() + 1);

        let sql = if payload.upsert_mode {
            format!(
                "INSERT INTO \"{}\" ({}, id) VALUES ({}, {}, gen_random_uuid()) ON CONFLICT DO NOTHING",
                table_name, col_list_with_org, placeholders.join(", "), org_placeholder
            )
        } else {
            format!(
                "INSERT INTO \"{}\" ({}, id) VALUES ({}, {}, gen_random_uuid())",
                table_name, col_list_with_org, placeholders.join(", "), org_placeholder
            )
        };

        let mut query = sqlx::query(&sql);
        for val in &values {
            query = query.bind(val.as_deref());
        }
        query = query.bind(org_id);

        match query.execute(&state.db_pool).await {
            Ok(r) if r.rows_affected() > 0 => imported += 1,
            Ok(_) => skipped += 1,
            Err(e) => {
                failed += 1;
                errors.push(CsvImportError { row: row_idx + 2, errors: vec![e.to_string()] });
                if payload.stop_on_error {
                    break;
                }
            }
        }
    }

    Ok(Json(CsvImportResponse {
        entity: payload.entity,
        total_rows,
        imported,
        failed,
        skipped,
        errors,
        preview: Some(preview),
    }))
}
