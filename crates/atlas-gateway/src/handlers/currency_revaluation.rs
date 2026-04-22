//! Currency Revaluation API Handlers
//!
//! Oracle Fusion Cloud ERP: General Ledger > Currency Revaluation
//!
//! Endpoints for managing revaluation definitions, accounts,
//! executing revaluation runs, and dashboard.

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
use crate::handlers::auth::Claims;

// ============================================================================
// Query Parameters
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ListDefinitionsQuery {
    pub active_only: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ListRunsQuery {
    pub status: Option<String>,
}

// ============================================================================
// Definition Management
// ============================================================================

/// Create a new revaluation definition
pub async fn create_revaluation_definition(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    let request = atlas_shared::CurrencyRevaluationDefinitionRequest {
        code: body["code"].as_str().unwrap_or("").to_string(),
        name: body["name"].as_str().unwrap_or("").to_string(),
        description: body["description"].as_str().map(String::from),
        revaluation_type: body["revaluation_type"].as_str().unwrap_or("period_end").to_string(),
        currency_code: body["currency_code"].as_str().unwrap_or("USD").to_string(),
        rate_type: body["rate_type"].as_str().unwrap_or("period_end").to_string(),
        gain_account_code: body["gain_account_code"].as_str().unwrap_or("").to_string(),
        loss_account_code: body["loss_account_code"].as_str().unwrap_or("").to_string(),
        unrealized_gain_account_code: body["unrealized_gain_account_code"].as_str().map(String::from),
        unrealized_loss_account_code: body["unrealized_loss_account_code"].as_str().map(String::from),
        account_range_from: body["account_range_from"].as_str().map(String::from),
        account_range_to: body["account_range_to"].as_str().map(String::from),
        include_subledger: body["include_subledger"].as_bool().unwrap_or(false),
        auto_reverse: body["auto_reverse"].as_bool().unwrap_or(true),
        reversal_period_offset: body["reversal_period_offset"].as_i64().unwrap_or(1) as i32,
        effective_from: body["effective_from"].as_str().and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok()),
        effective_to: body["effective_to"].as_str().and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok()),
        accounts: None, // Processed separately via add_account endpoint
    };

    match state.currency_revaluation_engine.create_definition(org_id, &request, None).await {
        Ok(def) => Ok((StatusCode::CREATED, Json(serde_json::to_value(def).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => {
            let status = match &e {
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status, Json(json!({"error": e.to_string()}))))
        }
    }
}

/// Get a revaluation definition by code
pub async fn get_revaluation_definition(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(code): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    match state.currency_revaluation_engine.get_definition(org_id, &code).await {
        Ok(Some(def)) => Ok(Json(serde_json::to_value(def).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(json!({"error": "Definition not found"})))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))),
    }
}

/// List all revaluation definitions
pub async fn list_revaluation_definitions(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<ListDefinitionsQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    let active_only = query.active_only.unwrap_or(false);

    match state.currency_revaluation_engine.list_definitions(org_id, active_only).await {
        Ok(definitions) => Ok(Json(json!({"data": definitions}))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))),
    }
}

/// Activate a definition
pub async fn activate_revaluation_definition(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    match state.currency_revaluation_engine.activate_definition(id).await {
        Ok(def) => Ok((StatusCode::OK, Json(serde_json::to_value(def).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => {
            let status = match &e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status, Json(json!({"error": e.to_string()}))))
        }
    }
}

/// Deactivate a definition
pub async fn deactivate_revaluation_definition(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    match state.currency_revaluation_engine.deactivate_definition(id).await {
        Ok(def) => Ok((StatusCode::OK, Json(serde_json::to_value(def).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => {
            let status = match &e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status, Json(json!({"error": e.to_string()}))))
        }
    }
}

/// Delete a definition
pub async fn delete_revaluation_definition(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(code): Path<String>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    match state.currency_revaluation_engine.delete_definition(org_id, &code).await {
        Ok(()) => Ok((StatusCode::OK, Json(json!({"message": "Deleted"})))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Account Management
// ============================================================================

/// Add an account to a definition
pub async fn add_revaluation_account(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(code): Path<String>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    let request = atlas_shared::CurrencyRevaluationAccountRequest {
        account_code: body["account_code"].as_str().unwrap_or("").to_string(),
        account_name: body["account_name"].as_str().map(String::from),
        account_type: body["account_type"].as_str().unwrap_or("asset").to_string(),
        is_included: body["is_included"].as_bool().unwrap_or(true),
    };

    match state.currency_revaluation_engine.add_account(org_id, &code, &request).await {
        Ok(acct) => Ok((StatusCode::CREATED, Json(serde_json::to_value(acct).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => {
            let status = match &e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status, Json(json!({"error": e.to_string()}))))
        }
    }
}

/// List accounts for a definition
pub async fn list_revaluation_accounts(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(code): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    match state.currency_revaluation_engine.list_accounts(org_id, &code).await {
        Ok(accounts) => Ok(Json(json!({"data": accounts}))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))),
    }
}

/// Remove an account
pub async fn remove_revaluation_account(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    match state.currency_revaluation_engine.remove_account(id).await {
        Ok(()) => Ok((StatusCode::OK, Json(json!({"message": "Removed"})))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Revaluation Run Management
// ============================================================================

/// Execute a revaluation run
pub async fn execute_revaluation(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    let balances = match body["balances"].as_array() {
        Some(arr) => arr.iter().filter_map(|b| {
            Some(atlas_shared::CurrencyRevaluationBalanceRequest {
                account_code: b["account_code"].as_str()?.to_string(),
                account_name: b["account_name"].as_str().map(String::from),
                account_type: b["account_type"].as_str().unwrap_or("asset").to_string(),
                original_amount: b["original_amount"].as_str().unwrap_or("0").to_string(),
                original_currency: b["original_currency"].as_str().unwrap_or("USD").to_string(),
                original_exchange_rate: b["original_exchange_rate"].as_str().unwrap_or("1").to_string(),
                original_base_amount: b["original_base_amount"].as_str().unwrap_or("0").to_string(),
            })
        }).collect(),
        None => Vec::new(),
    };

    let request = atlas_shared::CurrencyRevaluationRunRequest {
        definition_code: body["definition_code"].as_str().unwrap_or("").to_string(),
        period_name: body["period_name"].as_str().unwrap_or("").to_string(),
        period_start_date: body["period_start_date"].as_str()
            .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
            .unwrap_or(chrono::Utc::now().date_naive()),
        period_end_date: body["period_end_date"].as_str()
            .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
            .unwrap_or(chrono::Utc::now().date_naive()),
        revaluation_date: body["revaluation_date"].as_str()
            .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok()),
        rate_type_override: body["rate_type_override"].as_str().map(String::from),
        balances,
    };

    match state.currency_revaluation_engine.execute_revaluation(org_id, &request, None).await {
        Ok(run) => Ok((StatusCode::CREATED, Json(serde_json::to_value(run).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => {
            let status = match &e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status, Json(json!({"error": e.to_string()}))))
        }
    }
}

/// Get a revaluation run
pub async fn get_revaluation_run(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.currency_revaluation_engine.get_run(id).await {
        Ok(Some(run)) => Ok(Json(serde_json::to_value(run).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(json!({"error": "Run not found"})))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))),
    }
}

/// List revaluation runs
pub async fn list_revaluation_runs(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<ListRunsQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    match state.currency_revaluation_engine.list_runs(org_id, query.status.as_deref()).await {
        Ok(runs) => Ok(Json(json!({"data": runs}))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))),
    }
}

/// Post a revaluation run
pub async fn post_revaluation_run(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let posted_by = Uuid::parse_str(&claims.sub).ok();

    match state.currency_revaluation_engine.post_run(id, posted_by).await {
        Ok(run) => Ok((StatusCode::OK, Json(serde_json::to_value(run).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => {
            let status = match &e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status, Json(json!({"error": e.to_string()}))))
        }
    }
}

/// Reverse a revaluation run
pub async fn reverse_revaluation_run(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let reversed_by = Uuid::parse_str(&claims.sub).ok();

    match state.currency_revaluation_engine.reverse_run(id, reversed_by).await {
        Ok(run) => Ok((StatusCode::OK, Json(serde_json::to_value(run).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => {
            let status = match &e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status, Json(json!({"error": e.to_string()}))))
        }
    }
}

/// Cancel a revaluation run
pub async fn cancel_revaluation_run(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    match state.currency_revaluation_engine.cancel_run(id).await {
        Ok(run) => Ok((StatusCode::OK, Json(serde_json::to_value(run).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => {
            let status = match &e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status, Json(json!({"error": e.to_string()}))))
        }
    }
}

// ============================================================================
// Dashboard
// ============================================================================

/// Get revaluation dashboard summary
pub async fn get_revaluation_dashboard(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    match state.currency_revaluation_engine.get_dashboard(org_id).await {
        Ok(summary) => Ok(Json(serde_json::to_value(summary).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))),
    }
}