//! GL Allocation API Handlers
//!
//! Oracle Fusion Cloud ERP: General Ledger > Allocations
//!
//! Endpoints for managing allocation pools, bases, rules,
//! executing allocation runs, and dashboard.

use axum::{
    extract::{Path, Query, State, Extension},
    Json,
    http::StatusCode,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;
use uuid::Uuid;

use crate::AppState;
use crate::handlers::auth::{Claims, parse_uuid};

// ============================================================================
// Query Parameters
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ListPoolsQuery {
    pub active_only: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ListBasesQuery {
    pub active_only: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ListRulesQuery {
    pub active_only: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ListBasisDetailsQuery {
    pub period_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListRunsQuery {
    pub status: Option<String>,
}

// ============================================================================
// Pool Management
// ============================================================================

/// Create a new allocation pool
pub async fn create_allocation_pool(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    let code = body["code"].as_str().unwrap_or("").to_string();
    let name = body["name"].as_str().unwrap_or("").to_string();
    let description = body["description"].as_str();
    let pool_type = body["pool_type"].as_str().unwrap_or("cost_center").to_string();
    let source_account_code = body["source_account_code"].as_str();
    let source_account_range_from = body["source_account_range_from"].as_str();
    let source_account_range_to = body["source_account_range_to"].as_str();
    let source_department_id = body["source_department_id"].as_str().and_then(|s| Uuid::parse_str(s).ok());
    let source_project_id = body["source_project_id"].as_str().and_then(|s| Uuid::parse_str(s).ok());
    let currency_code = body["currency_code"].as_str().unwrap_or("USD").to_string();
    let effective_from = body["effective_from"].as_str().and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let effective_to = body["effective_to"].as_str().and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());

    match state.allocation_engine.create_pool(
        org_id, &code, &name, description,
        &pool_type, source_account_code,
        source_account_range_from, source_account_range_to,
        source_department_id, source_project_id,
        &currency_code, effective_from, effective_to,
        parse_uuid(&claims.sub).ok(),
    ).await {
        Ok(pool) => Ok((StatusCode::CREATED, Json(serde_json::to_value(pool).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Get a pool by code
pub async fn get_allocation_pool(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(code): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    match state.allocation_engine.get_pool(org_id, &code).await {
        Ok(Some(pool)) => Ok(Json(serde_json::to_value(pool).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(json!({"error": "Pool not found"})))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))),
    }
}

/// List all pools
pub async fn list_allocation_pools(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<ListPoolsQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    let active_only = query.active_only.unwrap_or(false);
    match state.allocation_engine.list_pools(org_id, active_only).await {
        Ok(pools) => Ok(Json(json!({"data": pools}))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))),
    }
}

/// Activate a pool
pub async fn activate_allocation_pool(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.allocation_engine.activate_pool(id).await {
        Ok(pool) => Ok(Json(serde_json::to_value(pool).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Deactivate a pool
pub async fn deactivate_allocation_pool(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.allocation_engine.deactivate_pool(id).await {
        Ok(pool) => Ok(Json(serde_json::to_value(pool).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Delete a pool
pub async fn delete_allocation_pool(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(code): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    match state.allocation_engine.delete_pool(org_id, &code).await {
        Ok(()) => Ok(Json(json!({"message": "Pool deleted"}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Basis Management
// ============================================================================

/// Create a new allocation basis
pub async fn create_allocation_basis(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    let code = body["code"].as_str().unwrap_or("").to_string();
    let name = body["name"].as_str().unwrap_or("").to_string();
    let description = body["description"].as_str();
    let basis_type = body["basis_type"].as_str().unwrap_or("statistical").to_string();
    let unit_of_measure = body["unit_of_measure"].as_str();
    let is_manual = body["is_manual"].as_bool().unwrap_or(true);
    let source_account_code = body["source_account_code"].as_str();
    let effective_from = body["effective_from"].as_str().and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let effective_to = body["effective_to"].as_str().and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());

    match state.allocation_engine.create_basis(
        org_id, &code, &name, description, &basis_type,
        unit_of_measure, is_manual, source_account_code,
        effective_from, effective_to,
        parse_uuid(&claims.sub).ok(),
    ).await {
        Ok(basis) => Ok((StatusCode::CREATED, Json(serde_json::to_value(basis).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Get a basis by code
pub async fn get_allocation_basis(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(code): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    match state.allocation_engine.get_basis(org_id, &code).await {
        Ok(Some(basis)) => Ok(Json(serde_json::to_value(basis).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(json!({"error": "Basis not found"})))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))),
    }
}

/// List all bases
pub async fn list_allocation_bases(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<ListBasesQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    let active_only = query.active_only.unwrap_or(false);
    match state.allocation_engine.list_bases(org_id, active_only).await {
        Ok(bases) => Ok(Json(json!({"data": bases}))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))),
    }
}

/// Activate a basis
pub async fn activate_allocation_basis(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.allocation_engine.activate_basis(id).await {
        Ok(basis) => Ok(Json(serde_json::to_value(basis).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Deactivate a basis
pub async fn deactivate_allocation_basis(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.allocation_engine.deactivate_basis(id).await {
        Ok(basis) => Ok(Json(serde_json::to_value(basis).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Delete a basis
pub async fn delete_allocation_basis(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(code): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    match state.allocation_engine.delete_basis(org_id, &code).await {
        Ok(()) => Ok(Json(json!({"message": "Basis deleted"}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Basis Detail Management
// ============================================================================

/// Add a basis detail
pub async fn add_allocation_basis_detail(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(basis_code): Path<String>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    let request = atlas_shared::GlAllocationBasisDetailRequest {
        target_department_id: body["target_department_id"].as_str().and_then(|s| Uuid::parse_str(s).ok()),
        target_department_name: body["target_department_name"].as_str().map(String::from),
        target_cost_center: body["target_cost_center"].as_str().map(String::from),
        target_project_id: body["target_project_id"].as_str().and_then(|s| Uuid::parse_str(s).ok()),
        target_project_name: body["target_project_name"].as_str().map(String::from),
        target_account_code: body["target_account_code"].as_str().map(String::from),
        basis_amount: body["basis_amount"].as_str().unwrap_or("0").to_string(),
        period_name: body["period_name"].as_str().map(String::from),
        period_start_date: body["period_start_date"].as_str().and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok()),
        period_end_date: body["period_end_date"].as_str().and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok()),
    };

    match state.allocation_engine.add_basis_detail(
        org_id, &basis_code, &request,
        parse_uuid(&claims.sub).ok(),
    ).await {
        Ok(detail) => Ok((StatusCode::CREATED, Json(serde_json::to_value(detail).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// List basis details
pub async fn list_allocation_basis_details(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(basis_code): Path<String>,
    Query(query): Query<ListBasisDetailsQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    match state.allocation_engine.list_basis_details(
        org_id, &basis_code, query.period_name.as_deref(),
    ).await {
        Ok(details) => Ok(Json(json!({"data": details}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Recalculate basis percentages
pub async fn recalculate_basis_percentages(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(basis_code): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    match state.allocation_engine.recalculate_basis_percentages(org_id, &basis_code).await {
        Ok(details) => Ok(Json(json!({"data": details}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Rule Management
// ============================================================================

/// Create a new allocation rule
pub async fn create_allocation_rule(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    let mut request = atlas_shared::GlAllocationRuleRequest {
        code: body["code"].as_str().unwrap_or("").to_string(),
        name: body["name"].as_str().unwrap_or("").to_string(),
        description: body["description"].as_str().map(String::from),
        pool_code: body["pool_code"].as_str().unwrap_or("").to_string(),
        basis_code: body["basis_code"].as_str().unwrap_or("").to_string(),
        allocation_method: body["allocation_method"].as_str().unwrap_or("proportional").to_string(),
        offset_method: body["offset_method"].as_str().unwrap_or("same_account").to_string(),
        offset_account_code: body["offset_account_code"].as_str().map(String::from),
        journal_batch_prefix: body["journal_batch_prefix"].as_str().map(String::from),
        round_to_largest: body["round_to_largest"].as_bool().unwrap_or(false),
        minimum_threshold: body["minimum_threshold"].as_str().map(String::from),
        effective_from: body["effective_from"].as_str().and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok()),
        effective_to: body["effective_to"].as_str().and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok()),
        target_lines: None,
    };

    // Parse target lines if provided
    if let Some(lines_array) = body["target_lines"].as_array() {
        let mut lines = Vec::new();
        for line_val in lines_array {
            lines.push(atlas_shared::GlAllocationTargetLineRequest {
                target_department_id: line_val["target_department_id"].as_str().and_then(|s| Uuid::parse_str(s).ok()),
                target_department_name: line_val["target_department_name"].as_str().map(String::from),
                target_cost_center: line_val["target_cost_center"].as_str().map(String::from),
                target_project_id: line_val["target_project_id"].as_str().and_then(|s| Uuid::parse_str(s).ok()),
                target_project_name: line_val["target_project_name"].as_str().map(String::from),
                target_account_code: line_val["target_account_code"].as_str().unwrap_or("").to_string(),
                target_account_name: line_val["target_account_name"].as_str().map(String::from),
                fixed_percentage: line_val["fixed_percentage"].as_str().map(String::from),
                is_active: line_val["is_active"].as_bool(),
            });
        }
        request.target_lines = Some(lines);
    }

    match state.allocation_engine.create_rule(
        org_id, &request,
        parse_uuid(&claims.sub).ok(),
    ).await {
        Ok(rule) => Ok((StatusCode::CREATED, Json(serde_json::to_value(rule).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Get a rule by code
pub async fn get_allocation_rule(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(code): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    match state.allocation_engine.get_rule(org_id, &code).await {
        Ok(Some(rule)) => Ok(Json(serde_json::to_value(rule).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(json!({"error": "Rule not found"})))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))),
    }
}

/// List all rules
pub async fn list_allocation_rules(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<ListRulesQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    let active_only = query.active_only.unwrap_or(false);
    match state.allocation_engine.list_rules(org_id, active_only).await {
        Ok(rules) => Ok(Json(json!({"data": rules}))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))),
    }
}

/// Activate a rule
pub async fn activate_allocation_rule(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.allocation_engine.activate_rule(id).await {
        Ok(rule) => Ok(Json(serde_json::to_value(rule).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Deactivate a rule
pub async fn deactivate_allocation_rule(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.allocation_engine.deactivate_rule(id).await {
        Ok(rule) => Ok(Json(serde_json::to_value(rule).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Delete a rule
pub async fn delete_allocation_rule(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(code): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    match state.allocation_engine.delete_rule(org_id, &code).await {
        Ok(()) => Ok(Json(json!({"message": "Rule deleted"}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Allocation Run Management
// ============================================================================

/// Execute an allocation (create a run)
pub async fn execute_allocation(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    let request = atlas_shared::GlAllocationRunRequest {
        rule_code: body["rule_code"].as_str().unwrap_or("").to_string(),
        period_name: body["period_name"].as_str().unwrap_or("").to_string(),
        period_start_date: body["period_start_date"].as_str()
            .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
            .unwrap_or(chrono::Utc::now().date_naive()),
        period_end_date: body["period_end_date"].as_str()
            .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
            .unwrap_or(chrono::Utc::now().date_naive()),
        run_date: body["run_date"].as_str().and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok()),
        pool_amount_override: body["pool_amount_override"].as_str().map(String::from),
    };

    match state.allocation_engine.execute_allocation(
        org_id, &request,
        parse_uuid(&claims.sub).ok(),
    ).await {
        Ok(run) => Ok((StatusCode::CREATED, Json(serde_json::to_value(run).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Get an allocation run
pub async fn get_allocation_run(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.allocation_engine.get_run(id).await {
        Ok(Some(run)) => Ok(Json(serde_json::to_value(run).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(json!({"error": "Run not found"})))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))),
    }
}

/// List allocation runs
pub async fn list_allocation_runs(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<ListRunsQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    match state.allocation_engine.list_runs(org_id, query.status.as_deref()).await {
        Ok(runs) => Ok(Json(json!({"data": runs}))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))),
    }
}

/// Post an allocation run
pub async fn post_allocation_run(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let posted_by = parse_uuid(&claims.sub)?;
    match state.allocation_engine.post_run(id, Some(posted_by)).await {
        Ok(run) => Ok(Json(serde_json::to_value(run).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Reverse an allocation run
pub async fn reverse_allocation_run(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let reversed_by = parse_uuid(&claims.sub)?;
    match state.allocation_engine.reverse_run(id, Some(reversed_by)).await {
        Ok(run) => Ok(Json(serde_json::to_value(run).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Cancel an allocation run
pub async fn cancel_allocation_run(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.allocation_engine.cancel_run(id).await {
        Ok(run) => Ok(Json(serde_json::to_value(run).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Dashboard
// ============================================================================

/// Get allocation dashboard summary
pub async fn get_allocation_dashboard(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    match state.allocation_engine.get_dashboard(org_id).await {
        Ok(summary) => Ok(Json(serde_json::to_value(summary).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))),
    }
}