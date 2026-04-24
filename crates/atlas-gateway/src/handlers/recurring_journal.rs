//! Recurring Journals API Handlers
//!
//! Oracle Fusion Cloud ERP: General Ledger > Journals > Recurring Journals
//!
//! Endpoints for managing recurring journal schedules, template lines,
//! journal generation, and generation history.

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
pub struct ListQuery {
    pub status: Option<String>,
}

// ============================================================================
// Schedules
// ============================================================================

/// Create a recurring journal schedule
pub async fn create_schedule(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    let schedule_number = body["schedule_number"].as_str().unwrap_or("").to_string();
    let name = body["name"].as_str().unwrap_or("").to_string();
    let description = body["description"].as_str();
    let recurrence_type = body["recurrence_type"].as_str().unwrap_or("monthly").to_string();
    let journal_type = body["journal_type"].as_str().unwrap_or("standard").to_string();
    let currency_code = body["currency_code"].as_str().unwrap_or("USD").to_string();
    let effective_from = body["effective_from"].as_str()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let effective_to = body["effective_to"].as_str()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let incremental_percent = body["incremental_percent"].as_str();
    let auto_post = body["auto_post"].as_bool().unwrap_or(false);
    let reversal_method = body["reversal_method"].as_str();
    let ledger_id = body["ledger_id"].as_str().and_then(|s| Uuid::parse_str(s).ok());
    let journal_category = body["journal_category"].as_str();
    let reference_template = body["reference_template"].as_str();

    match state.recurring_journal_engine.create_schedule(
        org_id, &schedule_number, &name, description, &recurrence_type,
        &journal_type, &currency_code, effective_from, effective_to,
        incremental_percent, auto_post, reversal_method, ledger_id,
        journal_category, reference_template, parse_uuid(&claims.sub).ok(),
    ).await {
        Ok(schedule) => Ok((StatusCode::CREATED, Json(serde_json::to_value(schedule).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Get a schedule by number
pub async fn get_schedule(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(schedule_number): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    match state.recurring_journal_engine.get_schedule(org_id, &schedule_number).await {
        Ok(Some(schedule)) => Ok(Json(serde_json::to_value(schedule).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(json!({"error": "Schedule not found"})))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// List schedules
pub async fn list_schedules(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<ListQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    match state.recurring_journal_engine.list_schedules(org_id, params.status.as_deref()).await {
        Ok(schedules) => Ok(Json(json!({"data": schedules}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Activate a schedule
pub async fn activate_schedule(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let approved_by: Uuid = parse_uuid(&claims.sub)?;
    match state.recurring_journal_engine.activate_schedule(id, Some(approved_by)).await {
        Ok(schedule) => Ok(Json(serde_json::to_value(schedule).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Deactivate a schedule
pub async fn deactivate_schedule(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.recurring_journal_engine.deactivate_schedule(id).await {
        Ok(schedule) => Ok(Json(serde_json::to_value(schedule).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Delete a schedule
pub async fn delete_schedule(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(schedule_number): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    match state.recurring_journal_engine.delete_schedule(org_id, &schedule_number).await {
        Ok(()) => Ok(Json(json!({"message": "Schedule deleted"}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Schedule Lines
// ============================================================================

/// Add a template line to a schedule
pub async fn add_schedule_line(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(schedule_id): Path<Uuid>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    let line_type = body["line_type"].as_str().unwrap_or("debit").to_string();
    let account_code = body["account_code"].as_str().unwrap_or("").to_string();
    let account_name = body["account_name"].as_str();
    let description = body["description"].as_str();
    let amount = body["amount"].as_str().unwrap_or("0").to_string();
    let currency_code = body["currency_code"].as_str().unwrap_or("USD").to_string();
    let tax_code = body["tax_code"].as_str();
    let cost_center = body["cost_center"].as_str();
    let department_id = body["department_id"].as_str().and_then(|s| Uuid::parse_str(s).ok());
    let project_id = body["project_id"].as_str().and_then(|s| Uuid::parse_str(s).ok());

    match state.recurring_journal_engine.add_schedule_line(
        org_id, schedule_id, &line_type, &account_code, account_name,
        description, &amount, &currency_code, tax_code, cost_center,
        department_id, project_id,
    ).await {
        Ok(line) => Ok((StatusCode::CREATED, Json(serde_json::to_value(line).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// List schedule lines
pub async fn list_schedule_lines(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(schedule_id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.recurring_journal_engine.list_schedule_lines(schedule_id).await {
        Ok(lines) => Ok(Json(json!({"data": lines}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Delete a schedule line
pub async fn delete_schedule_line(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(line_id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.recurring_journal_engine.delete_schedule_line(line_id).await {
        Ok(()) => Ok(Json(json!({"message": "Line deleted"}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Generation
// ============================================================================

/// Generate a journal from a recurring schedule
pub async fn generate_journal(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(schedule_id): Path<Uuid>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let generation_date = body["generation_date"].as_str()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .unwrap_or_else(|| chrono::Utc::now().date_naive());

    let override_amounts: Option<Vec<(i32, String)>> = body.get("override_amounts")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter().filter_map(|item| {
                let num = item.get("line_number")?.as_i64()? as i32;
                let amt = item.get("amount")?.as_str()?.to_string();
                Some((num, amt))
            }).collect()
        });

    let generated_by: Uuid = parse_uuid(&claims.sub)?;

    match state.recurring_journal_engine.generate_journal(
        schedule_id, generation_date, override_amounts, Some(generated_by),
    ).await {
        Ok(gen) => Ok((StatusCode::CREATED, Json(serde_json::to_value(gen).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Get a generation by ID
pub async fn get_generation(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.recurring_journal_engine.get_generation(id).await {
        Ok(Some(gen)) => Ok(Json(serde_json::to_value(gen).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(json!({"error": "Generation not found"})))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// List generations for a schedule
pub async fn list_generations(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(schedule_id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.recurring_journal_engine.list_generations(schedule_id).await {
        Ok(gens) => Ok(Json(json!({"data": gens}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Post a generation
pub async fn post_generation(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.recurring_journal_engine.post_generation(id).await {
        Ok(gen) => Ok(Json(serde_json::to_value(gen).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Reverse a generation
pub async fn reverse_generation(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.recurring_journal_engine.reverse_generation(id).await {
        Ok(gen) => Ok(Json(serde_json::to_value(gen).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Cancel a generation
pub async fn cancel_generation(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.recurring_journal_engine.cancel_generation(id).await {
        Ok(gen) => Ok(Json(serde_json::to_value(gen).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// List generation lines
pub async fn list_generation_lines(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(generation_id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.recurring_journal_engine.list_generation_lines(generation_id).await {
        Ok(lines) => Ok(Json(json!({"data": lines}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Dashboard
// ============================================================================

/// Get recurring journals dashboard
pub async fn get_dashboard(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    match state.recurring_journal_engine.get_dashboard_summary(org_id).await {
        Ok(summary) => Ok(Json(serde_json::to_value(summary).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}
