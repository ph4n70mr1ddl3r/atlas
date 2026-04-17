//! Notification, Saved Search, Approval, and Duplicate Detection handlers
//!
//! Oracle Fusion-inspired features: bell-icon notifications, personalized
//! saved searches, multi-level approval chains, duplicate detection, and
//! enhanced import with CSV support.

use axum::{
    extract::{State, Path, Query},
    Json,
    http::StatusCode,
    Extension,
};
use serde::Deserialize;
use crate::AppState;
use crate::handlers::auth::Claims;
use crate::handlers::records::{sanitize_identifier, row_to_json};
use std::sync::Arc;
use uuid::Uuid;
use tracing::{debug, error};

// ============================================================================
// Notifications
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct NotificationListParams {
    pub include_read: Option<bool>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Get unread notification count
pub async fn get_unread_count(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let count = state.notification_engine
        .unread_count(org_id, user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({ "count": count })))
}

/// List notifications for the current user
pub async fn list_notifications(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<NotificationListParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let include_read = params.include_read.unwrap_or(false);
    let limit = params.limit.unwrap_or(50).clamp(1, 200);
    let offset = params.offset.unwrap_or(0).max(0);

    let notifications = state.notification_engine
        .list(org_id, user_id, include_read, limit, offset)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({
        "data": notifications,
        "meta": {
            "include_read": include_read,
            "limit": limit,
            "offset": offset,
        }
    })))
}

/// Mark a notification as read
pub async fn mark_notification_read(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    state.notification_engine
        .mark_read(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::NO_CONTENT)
}

/// Mark all notifications as read
pub async fn mark_all_notifications_read(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let count = state.notification_engine
        .mark_all_read(org_id, user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({ "marked_read": count })))
}

/// Dismiss a notification
pub async fn dismiss_notification(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    state.notification_engine
        .dismiss(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Saved Searches (Oracle Fusion personalized views)
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct SavedSearchListParams {
    entity: Option<String>,
}

/// List saved searches for the current user
pub async fn list_saved_searches(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<SavedSearchListParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut where_clauses = vec![
        "organization_id = $1".to_string(),
        "(user_id = $2 OR is_shared = true)".to_string(),
        "deleted_at IS NULL".to_string(),
    ];
    let bind_count = 3;

    if let Some(ref _entity) = params.entity {
        where_clauses.push(format!("entity_type = ${}", bind_count));
    }

    let where_sql = where_clauses.join(" AND ");
    let query_str = format!(
        "SELECT * FROM _atlas.saved_searches WHERE {} ORDER BY created_at DESC",
        where_sql
    );

    let mut query = sqlx::query(&query_str).bind(org_id).bind(user_id);
    if let Some(ref entity) = params.entity {
        query = query.bind(entity);
    }

    let rows = query
        .fetch_all(&state.db_pool)
        .await
        .map_err(|e| { error!("Saved search query error: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;

    let searches: Vec<serde_json::Value> = rows.iter().map(row_to_json).collect();

    Ok(Json(serde_json::json!({ "data": searches })))
}

#[derive(Debug, Deserialize)]
pub struct CreateSavedSearchRequest {
    pub name: String,
    pub entity_type: String,
    pub description: Option<String>,
    pub filters: Option<serde_json::Value>,
    pub sort_by: Option<String>,
    pub sort_direction: Option<String>,
    pub columns: Option<serde_json::Value>,
    pub columns_widths: Option<serde_json::Value>,
    pub page_size: Option<i32>,
    pub is_shared: Option<bool>,
    pub is_default: Option<bool>,
    pub color: Option<String>,
    pub icon: Option<String>,
}

/// Create a saved search
pub async fn create_saved_search(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateSavedSearchRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let row = sqlx::query(
        r#"
        INSERT INTO _atlas.saved_searches 
            (organization_id, user_id, name, description, entity_type, 
             filters, sort_by, sort_direction, columns, columns_widths,
             page_size, is_shared, is_default, color, icon, metadata)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, '{}'::jsonb)
        RETURNING *
        "#
    )
    .bind(org_id)
    .bind(user_id)
    .bind(&payload.name)
    .bind(&payload.description)
    .bind(&payload.entity_type)
    .bind(payload.filters.unwrap_or(serde_json::json!([])))
    .bind(payload.sort_by.unwrap_or_else(|| "created_at".to_string()))
    .bind(payload.sort_direction.unwrap_or_else(|| "desc".to_string()))
    .bind(payload.columns.unwrap_or(serde_json::json!([])))
    .bind(payload.columns_widths.unwrap_or(serde_json::json!({})))
    .bind(payload.page_size.unwrap_or(20))
    .bind(payload.is_shared.unwrap_or(false))
    .bind(payload.is_default.unwrap_or(false))
    .bind(&payload.color)
    .bind(&payload.icon)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| { error!("Create saved search error: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;

    Ok((StatusCode::CREATED, Json(row_to_json(&row))))
}

/// Delete a saved search
pub async fn delete_saved_search(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    claims: Extension<Claims>,
) -> Result<StatusCode, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    sqlx::query("UPDATE _atlas.saved_searches SET deleted_at = now() WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user_id)
        .execute(&state.db_pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Approval Chains
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateApprovalChainRequest {
    pub name: String,
    pub description: Option<String>,
    pub entity_type: String,
    pub condition_expression: Option<String>,
    pub chain_definition: serde_json::Value,
    pub escalation_enabled: Option<bool>,
    pub escalation_hours: Option<i32>,
    pub escalation_to_roles: Option<serde_json::Value>,
    pub allow_delegation: Option<bool>,
}

/// Create an approval chain
pub async fn create_approval_chain(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateApprovalChainRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let row = sqlx::query(
        r#"
        INSERT INTO _atlas.approval_chains 
            (organization_id, name, description, entity_type, condition_expression,
             chain_definition, escalation_enabled, escalation_hours, escalation_to_roles, allow_delegation)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING *
        "#
    )
    .bind(org_id)
    .bind(&payload.name)
    .bind(&payload.description)
    .bind(&payload.entity_type)
    .bind(&payload.condition_expression)
    .bind(&payload.chain_definition)
    .bind(payload.escalation_enabled.unwrap_or(true))
    .bind(payload.escalation_hours.unwrap_or(48))
    .bind(payload.escalation_to_roles.unwrap_or(serde_json::json!([])))
    .bind(payload.allow_delegation.unwrap_or(true))
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| { error!("Create approval chain error: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;

    Ok((StatusCode::CREATED, Json(row_to_json(&row))))
}

/// List approval chains for an entity type
pub async fn list_approval_chains(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<ApprovalChainListParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let rows = if let Some(ref entity_type) = params.entity_type {
        sqlx::query(
            "SELECT * FROM _atlas.approval_chains WHERE organization_id = $1 AND entity_type = $2 AND is_active = true"
        )
        .bind(org_id).bind(entity_type)
        .fetch_all(&state.db_pool).await
    } else {
        sqlx::query(
            "SELECT * FROM _atlas.approval_chains WHERE organization_id = $1 AND is_active = true"
        )
        .bind(org_id)
        .fetch_all(&state.db_pool).await
    }.map_err(|e| { error!("List approval chains error: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;

    let chains: Vec<serde_json::Value> = rows.iter().map(row_to_json).collect();
    Ok(Json(serde_json::json!({ "data": chains })))
}

#[derive(Debug, Deserialize)]
pub struct ApprovalChainListParams {
    entity_type: Option<String>,
}

/// Get pending approvals for the current user
pub async fn get_pending_approvals(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Get steps assigned to this user
    let user_steps = state.approval_engine
        .get_pending_for_user(org_id, user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Get steps assigned to user's roles
    let mut role_steps = Vec::new();
    for role in &claims.roles {
        if let Ok(steps) = state.approval_engine.get_pending_for_role(org_id, role).await {
            role_steps.extend(steps);
        }
    }

    // Combine and deduplicate
    let mut all_step_ids = std::collections::HashSet::new();
    let mut all_steps: Vec<serde_json::Value> = Vec::new();

    for step in user_steps.iter().chain(role_steps.iter()) {
        if all_step_ids.insert(step.id) {
            all_steps.push(serde_json::to_value(step).unwrap_or_default());
        }
    }

    Ok(Json(serde_json::json!({
        "data": all_steps,
        "meta": {
            "user_steps": user_steps.len(),
            "role_steps": role_steps.len(),
        }
    })))
}

/// Approve an approval step
#[derive(Debug, Deserialize)]
pub struct ApproveStepRequest {
    pub comment: Option<String>,
}

pub async fn approve_approval_step(
    State(state): State<Arc<AppState>>,
    Path(step_id): Path<Uuid>,
    claims: Extension<Claims>,
    Json(payload): Json<ApproveStepRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let request = state.approval_engine
        .approve_step(org_id, step_id, user_id, payload.comment.as_deref())
        .await
        .map_err(|e| {
            error!("Approve step error: {}", e);
            StatusCode::BAD_REQUEST
        })?;

    Ok(Json(serde_json::to_value(request).unwrap_or_default()))
}

/// Reject an approval step
pub async fn reject_approval_step(
    State(state): State<Arc<AppState>>,
    Path(step_id): Path<Uuid>,
    claims: Extension<Claims>,
    Json(payload): Json<ApproveStepRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let request = state.approval_engine
        .reject_step(org_id, step_id, user_id, payload.comment.as_deref())
        .await
        .map_err(|e| {
            error!("Reject step error: {}", e);
            StatusCode::BAD_REQUEST
        })?;

    Ok(Json(serde_json::to_value(request).unwrap_or_default()))
}

/// Delegate an approval step to another user
#[derive(Debug, Deserialize)]
pub struct DelegateStepRequest {
    pub delegated_to: Uuid,
}

pub async fn delegate_approval_step(
    State(state): State<Arc<AppState>>,
    Path(step_id): Path<Uuid>,
    claims: Extension<Claims>,
    Json(payload): Json<DelegateStepRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let step = state.approval_engine
        .delegate_step(org_id, step_id, user_id, payload.delegated_to)
        .await
        .map_err(|e| {
            error!("Delegate step error: {}", e);
            StatusCode::BAD_REQUEST
        })?;

    Ok(Json(serde_json::to_value(step).unwrap_or_default()))
}

// ============================================================================
// Duplicate Detection
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CheckDuplicatesRequest {
    pub entity_type: String,
    pub data: serde_json::Value,
}

/// Check for potential duplicate records before creating
pub async fn check_duplicates(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CheckDuplicatesRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Find active duplicate rules for this entity type
    let rules = sqlx::query(
        "SELECT * FROM _atlas.duplicate_rules WHERE organization_id = $1 AND entity_type = $2 AND is_active = true"
    )
    .bind(org_id)
    .bind(&payload.entity_type)
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| { error!("Duplicate rules query error: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;

    let mut duplicates = Vec::new();

    for rule_row in &rules {
        use sqlx::Row;
        let rule_id: Uuid = rule_row.try_get("id").unwrap_or_default();
        let rule_name: String = rule_row.try_get("name").unwrap_or_default();
        let match_criteria_val: serde_json::Value = rule_row.try_get("match_criteria").unwrap_or(serde_json::json!([]));
        let on_duplicate: String = rule_row.try_get("on_duplicate").unwrap_or_else(|_| "warn".to_string());

        // Parse match criteria
        let criteria_items: Vec<serde_json::Value> = match match_criteria_val.as_array() {
            Some(arr) => arr.clone(),
            None => continue,
        };

        // Build WHERE clause from match criteria
        let mut conditions = Vec::new();
        let mut bind_values: Vec<String> = Vec::new();
        let entity_def = state.schema_engine.get_entity(&payload.entity_type);
        let table_name = match entity_def {
            Some(ref def) => def.table_name.as_deref().unwrap_or(&payload.entity_type),
            None => &payload.entity_type,
        };
        
        let safe_table = sanitize_identifier(table_name).map_err(|_| StatusCode::BAD_REQUEST)?;

        let mut bind_idx = 1;
        for criterion in &criteria_items {
            let field = criterion.get("field").and_then(|f| f.as_str()).unwrap_or("");
            let safe_field = sanitize_identifier(field).map_err(|_| StatusCode::BAD_REQUEST)?;
            let match_type = criterion.get("match_type").and_then(|m| m.as_str()).unwrap_or("exact");

            if let Some(value) = payload.data.get(field) {
                let text_value = match value {
                    serde_json::Value::String(s) => s.clone(),
                    serde_json::Value::Null => String::new(),
                    other => other.to_string(),
                };

                match match_type {
                    "exact" => {
                        conditions.push(format!("\"{}\" = ${}::text", safe_field, bind_idx));
                        bind_values.push(text_value);
                        bind_idx += 1;
                    }
                    "case_insensitive" => {
                        conditions.push(format!("LOWER(\"{}\") = LOWER(${}::text)", safe_field, bind_idx));
                        bind_values.push(text_value);
                        bind_idx += 1;
                    }
                    _ => {
                        conditions.push(format!("\"{}\" = ${}::text", safe_field, bind_idx));
                        bind_values.push(text_value);
                        bind_idx += 1;
                    }
                }
            }
        }

        if conditions.is_empty() {
            continue;
        }

        let where_sql = format!(
            "organization_id = ${} AND ({}) AND deleted_at IS NULL",
            bind_idx,
            conditions.join(" OR ")
        );

        let query_str = format!("SELECT id, * FROM \"{}\" WHERE {} LIMIT 5", safe_table, where_sql);

        let mut query = sqlx::query(&query_str).bind(org_id);
        for val in &bind_values {
            query = query.bind(val);
        }

        match query.fetch_all(&state.db_pool).await {
            Ok(rows) => {
                for row in rows {
                    let existing_id: Uuid = row.try_get("id").unwrap_or_default();
                    let existing_data = row_to_json(&row);
                    
                    // Find which fields matched
                    let match_fields: Vec<String> = criteria_items.iter()
                        .filter_map(|c| {
                            let f = c.get("field").and_then(|f| f.as_str()).unwrap_or("");
                            let new_val = payload.data.get(f);
                            let existing_val = existing_data.get(f);
                            match (new_val, existing_val) {
                                (Some(n), Some(e)) if n == e => Some(f.to_string()),
                                _ => None,
                            }
                        })
                        .collect();

                    duplicates.push(serde_json::json!({
                        "rule_id": rule_id,
                        "rule_name": rule_name,
                        "existing_record_id": existing_id,
                        "match_fields": match_fields,
                        "action": on_duplicate,
                        "existing_data": existing_data,
                    }));
                }
            }
            Err(e) => {
                debug!("Duplicate check query failed for rule {}: {}", rule_name, e);
            }
        }
    }

    Ok(Json(serde_json::json!({
        "has_duplicates": !duplicates.is_empty(),
        "duplicates": duplicates,
    })))
}

// ============================================================================
// Enhanced Import with validation
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateDuplicateRuleRequest {
    pub name: String,
    pub entity_type: String,
    pub description: Option<String>,
    pub match_criteria: serde_json::Value,
    pub filter_condition: Option<serde_json::Value>,
    pub on_duplicate: Option<String>,
}

/// Create a duplicate detection rule
pub async fn create_duplicate_rule(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateDuplicateRuleRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let row = sqlx::query(
        r#"
        INSERT INTO _atlas.duplicate_rules
            (organization_id, name, entity_type, description, match_criteria,
             filter_condition, on_duplicate, is_active)
        VALUES ($1, $2, $3, $4, $5, $6, $7, true)
        RETURNING *
        "#
    )
    .bind(org_id)
    .bind(&payload.name)
    .bind(&payload.entity_type)
    .bind(&payload.description)
    .bind(&payload.match_criteria)
    .bind(payload.filter_condition.unwrap_or(serde_json::json!({})))
    .bind(payload.on_duplicate.unwrap_or_else(|| "warn".to_string()))
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| { error!("Create duplicate rule error: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;

    Ok((StatusCode::CREATED, Json(row_to_json(&row))))
}