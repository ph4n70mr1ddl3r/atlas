//! Cash Concentration / Pooling API Handlers
//!
//! Oracle Fusion Cloud ERP: Treasury > Cash Pooling
//!
//! Endpoints for managing cash pools, participants, sweep rules, and sweep execution.

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
pub struct PoolListQuery {
    pub status: Option<String>,
    pub pool_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ParticipantListQuery {
    pub status: Option<String>,
}

// ============================================================================
// Cash Pools
// ============================================================================

/// Create a cash pool
pub async fn create_pool(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    let pool_code = body["pool_code"].as_str().unwrap_or("").to_string();
    let pool_name = body["pool_name"].as_str().unwrap_or("").to_string();
    let pool_type = body["pool_type"].as_str().unwrap_or("physical").to_string();
    let concentration_account_id = body["concentration_account_id"].as_str().and_then(|s| Uuid::parse_str(s).ok());
    let concentration_account_name = body["concentration_account_name"].as_str();
    let currency_code = body["currency_code"].as_str().unwrap_or("USD").to_string();
    let sweep_frequency = body["sweep_frequency"].as_str();
    let sweep_time = body["sweep_time"].as_str();
    let minimum_transfer_amount = body["minimum_transfer_amount"].as_str();
    let maximum_transfer_amount = body["maximum_transfer_amount"].as_str();
    let target_balance = body["target_balance"].as_str();
    let interest_allocation_method = body["interest_allocation_method"].as_str();
    let interest_rate = body["interest_rate"].as_str();
    let effective_date = body["effective_date"].as_str().and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let termination_date = body["termination_date"].as_str().and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let description = body["description"].as_str();
    let notes = body["notes"].as_str();

    match state.cash_concentration_engine.create_pool(
        org_id, &pool_code, &pool_name, &pool_type,
        concentration_account_id, concentration_account_name,
        &currency_code, sweep_frequency, sweep_time,
        minimum_transfer_amount, maximum_transfer_amount,
        target_balance, interest_allocation_method, interest_rate,
        effective_date, termination_date, description, notes,
        parse_uuid(&claims.sub).ok(),
    ).await {
        Ok(pool) => Ok((StatusCode::CREATED, Json(serde_json::to_value(pool).unwrap_or(Value::Null)))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Get a cash pool by code
pub async fn get_pool(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(pool_code): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    match state.cash_concentration_engine.get_pool(org_id, &pool_code).await {
        Ok(Some(pool)) => Ok(Json(serde_json::to_value(pool).unwrap_or(Value::Null))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(json!({"error": "Cash pool not found"})))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// List cash pools
pub async fn list_pools(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<PoolListQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    match state.cash_concentration_engine.list_pools(
        org_id, params.status.as_deref(), params.pool_type.as_deref(),
    ).await {
        Ok(pools) => Ok(Json(json!({"data": pools}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Activate a cash pool
pub async fn activate_pool(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.cash_concentration_engine.activate_pool(id).await {
        Ok(pool) => Ok(Json(serde_json::to_value(pool).unwrap_or(Value::Null))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Suspend a cash pool
pub async fn suspend_pool(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.cash_concentration_engine.suspend_pool(id).await {
        Ok(pool) => Ok(Json(serde_json::to_value(pool).unwrap_or(Value::Null))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Close a cash pool
pub async fn close_pool(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.cash_concentration_engine.close_pool(id).await {
        Ok(pool) => Ok(Json(serde_json::to_value(pool).unwrap_or(Value::Null))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Delete a cash pool
pub async fn delete_pool(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(pool_code): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    match state.cash_concentration_engine.delete_pool(org_id, &pool_code).await {
        Ok(()) => Ok(Json(json!({"message": "Cash pool deleted"}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Participants
// ============================================================================

/// Add a participant to a pool
pub async fn add_participant(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(pool_id): Path<Uuid>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    let participant_code = body["participant_code"].as_str().unwrap_or("").to_string();
    let bank_account_id = body["bank_account_id"].as_str().and_then(|s| Uuid::parse_str(s).ok());
    let bank_account_name = body["bank_account_name"].as_str();
    let bank_name = body["bank_name"].as_str();
    let account_number = body["account_number"].as_str();
    let participant_type = body["participant_type"].as_str().unwrap_or("source").to_string();
    let sweep_direction = body["sweep_direction"].as_str().unwrap_or("to_concentration").to_string();
    let priority = body["priority"].as_i64().map(|p| p as i32);
    let minimum_balance = body["minimum_balance"].as_str();
    let maximum_balance = body["maximum_balance"].as_str();
    let threshold_amount = body["threshold_amount"].as_str();
    let current_balance = body["current_balance"].as_str();
    let entity_id = body["entity_id"].as_str().and_then(|s| Uuid::parse_str(s).ok());
    let entity_name = body["entity_name"].as_str();
    let effective_date = body["effective_date"].as_str().and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let description = body["description"].as_str();

    match state.cash_concentration_engine.add_participant(
        org_id, pool_id, &participant_code,
        bank_account_id, bank_account_name, bank_name, account_number,
        &participant_type, &sweep_direction, priority,
        minimum_balance, maximum_balance, threshold_amount,
        current_balance, entity_id, entity_name,
        effective_date, description, parse_uuid(&claims.sub).ok(),
    ).await {
        Ok(participant) => Ok((StatusCode::CREATED, Json(serde_json::to_value(participant).unwrap_or(Value::Null)))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// List participants
pub async fn list_participants(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(pool_id): Path<Uuid>,
    Query(params): Query<ParticipantListQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.cash_concentration_engine.list_participants(pool_id, params.status.as_deref()).await {
        Ok(participants) => Ok(Json(json!({"data": participants}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Remove a participant
pub async fn remove_participant(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path((pool_id, participant_code)): Path<(Uuid, String)>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.cash_concentration_engine.remove_participant(pool_id, &participant_code).await {
        Ok(()) => Ok(Json(json!({"message": "Participant removed"}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Sweep Rules
// ============================================================================

/// Create a sweep rule
pub async fn create_sweep_rule(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(pool_id): Path<Uuid>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    let rule_code = body["rule_code"].as_str().unwrap_or("").to_string();
    let rule_name = body["rule_name"].as_str().unwrap_or("").to_string();
    let sweep_type = body["sweep_type"].as_str().unwrap_or("zero_balance").to_string();
    let participant_id = body["participant_id"].as_str().and_then(|s| Uuid::parse_str(s).ok());
    let direction = body["direction"].as_str().unwrap_or("to_concentration").to_string();
    let trigger_condition = body["trigger_condition"].as_str();
    let threshold_amount = body["threshold_amount"].as_str();
    let target_balance = body["target_balance"].as_str();
    let minimum_transfer = body["minimum_transfer"].as_str();
    let maximum_transfer = body["maximum_transfer"].as_str();
    let priority = body["priority"].as_i64().map(|p| p as i32);
    let effective_date = body["effective_date"].as_str().and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let description = body["description"].as_str();

    match state.cash_concentration_engine.create_sweep_rule(
        org_id, pool_id, &rule_code, &rule_name, &sweep_type,
        participant_id, &direction, trigger_condition,
        threshold_amount, target_balance, minimum_transfer,
        maximum_transfer, priority, effective_date, description,
        parse_uuid(&claims.sub).ok(),
    ).await {
        Ok(rule) => Ok((StatusCode::CREATED, Json(serde_json::to_value(rule).unwrap_or(Value::Null)))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// List sweep rules
pub async fn list_sweep_rules(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(pool_id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.cash_concentration_engine.list_sweep_rules(pool_id).await {
        Ok(rules) => Ok(Json(json!({"data": rules}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Delete a sweep rule
pub async fn delete_sweep_rule(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path((pool_id, rule_code)): Path<(Uuid, String)>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.cash_concentration_engine.delete_sweep_rule(pool_id, &rule_code).await {
        Ok(()) => Ok(Json(json!({"message": "Sweep rule deleted"}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Sweep Execution
// ============================================================================

/// Execute a sweep run
pub async fn execute_sweep(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(pool_id): Path<Uuid>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    let run_type = body["run_type"].as_str().unwrap_or("manual").to_string();
    let run_date = body["run_date"].as_str()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .unwrap_or_else(|| chrono::Utc::now().date_naive());
    let notes = body["notes"].as_str();

    match state.cash_concentration_engine.execute_sweep(
        org_id, pool_id, &run_type, run_date, notes, parse_uuid(&claims.sub).ok(),
    ).await {
        Ok(run) => Ok((StatusCode::CREATED, Json(serde_json::to_value(run).unwrap_or(Value::Null)))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Get a sweep run
pub async fn get_sweep_run(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.cash_concentration_engine.get_sweep_run(id).await {
        Ok(Some(run)) => Ok(Json(serde_json::to_value(run).unwrap_or(Value::Null))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(json!({"error": "Sweep run not found"})))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// List sweep runs for a pool
pub async fn list_sweep_runs(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(pool_id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.cash_concentration_engine.list_sweep_runs(pool_id).await {
        Ok(runs) => Ok(Json(json!({"data": runs}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// List sweep run lines
pub async fn list_sweep_run_lines(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(sweep_run_id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.cash_concentration_engine.list_sweep_run_lines(sweep_run_id).await {
        Ok(lines) => Ok(Json(json!({"data": lines}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Dashboard
// ============================================================================

/// Get cash concentration dashboard
pub async fn get_cash_pooling_dashboard(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    match state.cash_concentration_engine.get_dashboard(org_id).await {
        Ok(summary) => Ok(Json(serde_json::to_value(summary).unwrap_or(Value::Null))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}
