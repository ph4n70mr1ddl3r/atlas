//! Scheduled Processes API Handlers
//!
//! Oracle Fusion Cloud ERP: Navigator > Tools > Scheduled Processes
//!
//! Endpoints for managing process templates, submitting processes,
//! scheduling recurring jobs, monitoring execution, and viewing logs.

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
pub struct ListTemplatesQuery {
    pub process_type: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ListProcessesQuery {
    pub status: Option<String>,
    pub submitted_by: Option<String>,
    pub process_type: Option<String>,
    pub limit: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct ListRecurrencesQuery {
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ListLogsQuery {
    pub log_level: Option<String>,
    pub limit: Option<i32>,
}

// ============================================================================
// Template Management
// ============================================================================

/// Create a process template
pub async fn create_template(
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
    let process_type = body["process_type"].as_str().unwrap_or("report").to_string();
    let executor_type = body["executor_type"].as_str().unwrap_or("built_in").to_string();
    let executor_config = body.get("executor_config").cloned().unwrap_or(json!({}));
    let parameters = body.get("parameters").cloned().unwrap_or(json!([]));
    let default_parameters = body.get("default_parameters").cloned().unwrap_or(json!({}));
    let timeout_minutes = body["timeout_minutes"].as_i64().unwrap_or(60) as i32;
    let max_retries = body["max_retries"].as_i64().unwrap_or(0) as i32;
    let retry_delay_minutes = body["retry_delay_minutes"].as_i64().unwrap_or(5) as i32;
    let requires_approval = body["requires_approval"].as_bool().unwrap_or(false);
    let approval_chain_id = body["approval_chain_id"].as_str().and_then(|s| Uuid::parse_str(s).ok());
    let effective_from = body["effective_from"].as_str()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let effective_to = body["effective_to"].as_str()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());

    match state.scheduled_process_engine.create_template(
        org_id, &code, &name, description,
        &process_type, &executor_type, executor_config,
        parameters, default_parameters,
        timeout_minutes, max_retries, retry_delay_minutes,
        requires_approval, approval_chain_id,
        effective_from, effective_to,
        Some(claims.sub.parse().unwrap_or(Uuid::nil())),
    ).await {
        Ok(template) => Ok((StatusCode::CREATED, Json(serde_json::to_value(template).unwrap()))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Get a template by code
pub async fn get_template(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(code): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    match state.scheduled_process_engine.get_template(org_id, &code).await {
        Ok(Some(template)) => Ok(Json(serde_json::to_value(template).unwrap())),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(json!({"error": "Template not found"})))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// List templates
pub async fn list_templates(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<ListTemplatesQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    match state.scheduled_process_engine.list_templates(
        org_id,
        params.process_type.as_deref(),
        params.is_active,
    ).await {
        Ok(templates) => Ok(Json(json!({"data": templates, "total": templates.len()}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Activate a template
pub async fn activate_template(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.scheduled_process_engine.activate_template(id).await {
        Ok(template) => Ok(Json(serde_json::to_value(template).unwrap())),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Deactivate a template
pub async fn deactivate_template(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.scheduled_process_engine.deactivate_template(id).await {
        Ok(template) => Ok(Json(serde_json::to_value(template).unwrap())),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Delete a template
pub async fn delete_template(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(code): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    match state.scheduled_process_engine.delete_template(org_id, &code).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Process Submission & Management
// ============================================================================

/// Submit a new process
pub async fn submit_process(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    let template_code = body["template_code"].as_str();
    let process_name = body["process_name"].as_str().unwrap_or("").to_string();
    let process_type = body["process_type"].as_str().unwrap_or("report").to_string();
    let description = body["description"].as_str();
    let priority = body["priority"].as_str().unwrap_or("normal").to_string();
    let scheduled_start_at = body["scheduled_start_at"].as_str()
        .and_then(|d| chrono::DateTime::parse_from_rfc3339(d).ok())
        .map(|d| d.to_utc());
    let parameters = body.get("parameters").cloned().unwrap_or(json!({}));
    let submitted_by = Uuid::parse_str(&claims.sub).unwrap_or(Uuid::nil());

    match state.scheduled_process_engine.submit_process(
        org_id, template_code,
        &process_name, &process_type, description,
        &priority, scheduled_start_at,
        parameters, submitted_by,
    ).await {
        Ok(process) => Ok((StatusCode::CREATED, Json(serde_json::to_value(process).unwrap()))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Get a process by ID
pub async fn get_process(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.scheduled_process_engine.get_process(id).await {
        Ok(Some(process)) => Ok(Json(serde_json::to_value(process).unwrap())),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(json!({"error": "Process not found"})))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// List processes
pub async fn list_processes(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<ListProcessesQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    let submitted_by = params.submitted_by.as_ref()
        .and_then(|s| Uuid::parse_str(s).ok());

    match state.scheduled_process_engine.list_processes(
        org_id,
        params.status.as_deref(),
        submitted_by,
        params.process_type.as_deref(),
        params.limit,
    ).await {
        Ok(processes) => Ok(Json(json!({"data": processes, "total": processes.len()}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Start a process (transition to running)
pub async fn start_process(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.scheduled_process_engine.start_process(id).await {
        Ok(process) => Ok(Json(serde_json::to_value(process).unwrap())),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Complete a process
pub async fn complete_process(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let result_summary = body["result_summary"].as_str();
    let output_file_url = body["output_file_url"].as_str();
    let log_output = body["log_output"].as_str();

    match state.scheduled_process_engine.complete_process(
        id, result_summary, output_file_url, log_output,
    ).await {
        Ok(process) => Ok(Json(serde_json::to_value(process).unwrap())),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Cancel a process
pub async fn cancel_process(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let cancelled_by = Uuid::parse_str(&claims.sub).unwrap_or(Uuid::nil());
    let reason = body["reason"].as_str();

    match state.scheduled_process_engine.cancel_process(id, cancelled_by, reason).await {
        Ok(process) => Ok(Json(serde_json::to_value(process).unwrap())),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Update process progress
pub async fn update_progress(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let progress_percent = body["progress_percent"].as_i64().unwrap_or(0) as i32;

    match state.scheduled_process_engine.update_progress(id, progress_percent).await {
        Ok(process) => Ok(Json(serde_json::to_value(process).unwrap())),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Approve a waiting process
pub async fn approve_process(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.scheduled_process_engine.approve_process(id).await {
        Ok(process) => Ok(Json(serde_json::to_value(process).unwrap())),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Recurrence Management
// ============================================================================

/// Create a recurrence schedule
pub async fn create_recurrence(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    let name = body["name"].as_str().unwrap_or("").to_string();
    let description = body["description"].as_str();
    let template_code = body["template_code"].as_str().unwrap_or("").to_string();
    let parameters = body.get("parameters").cloned().unwrap_or(json!({}));
    let recurrence_type = body["recurrence_type"].as_str().unwrap_or("daily").to_string();
    let recurrence_config = body.get("recurrence_config").cloned().unwrap_or(json!({}));
    let start_date = body["start_date"].as_str()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .unwrap_or(chrono::Utc::now().date_naive());
    let end_date = body["end_date"].as_str()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let max_runs = body["max_runs"].as_i64().map(|m| m as i32);

    match state.scheduled_process_engine.create_recurrence(
        org_id, &name, description, &template_code,
        parameters, &recurrence_type, recurrence_config,
        start_date, end_date, max_runs,
        Some(claims.sub.parse().unwrap_or(Uuid::nil())),
    ).await {
        Ok(recurrence) => Ok((StatusCode::CREATED, Json(serde_json::to_value(recurrence).unwrap()))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Get a recurrence by ID
pub async fn get_recurrence(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.scheduled_process_engine.get_recurrence(id).await {
        Ok(Some(recurrence)) => Ok(Json(serde_json::to_value(recurrence).unwrap())),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(json!({"error": "Recurrence not found"})))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// List recurrences
pub async fn list_recurrences(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<ListRecurrencesQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    match state.scheduled_process_engine.list_recurrences(org_id, params.is_active).await {
        Ok(recurrences) => Ok(Json(json!({"data": recurrences, "total": recurrences.len()}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Deactivate a recurrence
pub async fn deactivate_recurrence(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.scheduled_process_engine.deactivate_recurrence(id).await {
        Ok(recurrence) => Ok(Json(serde_json::to_value(recurrence).unwrap())),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Delete a recurrence
pub async fn delete_recurrence(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<Value>)> {
    match state.scheduled_process_engine.delete_recurrence(id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Process due recurrences (cron-like trigger)
pub async fn process_due_recurrences(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.scheduled_process_engine.process_due_recurrences().await {
        Ok(spawned) => Ok(Json(json!({
            "spawned_count": spawned.len(),
            "spawned_process_ids": spawned,
        }))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Process Logs
// ============================================================================

/// List logs for a process
pub async fn list_process_logs(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Query(params): Query<ListLogsQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.scheduled_process_engine.list_logs(
        id, params.log_level.as_deref(), params.limit,
    ).await {
        Ok(logs) => Ok(Json(json!({"data": logs, "total": logs.len()}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Add a log entry to a process
pub async fn add_process_log(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    let log_level = body["log_level"].as_str().unwrap_or("info").to_string();
    let message = body["message"].as_str().unwrap_or("").to_string();
    let details = body.get("details").cloned();
    let step_name = body["step_name"].as_str();
    let duration_ms = body["duration_ms"].as_i64().map(|m| m as i32);

    match state.scheduled_process_engine.add_log(
        org_id, id, &log_level, &message,
        details, step_name, duration_ms,
    ).await {
        Ok(log) => Ok((StatusCode::CREATED, Json(serde_json::to_value(log).unwrap()))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Dashboard
// ============================================================================

/// Get scheduled processes dashboard summary
pub async fn get_scheduled_process_dashboard(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    match state.scheduled_process_engine.get_dashboard_summary(org_id).await {
        Ok(summary) => Ok(Json(serde_json::to_value(summary).unwrap())),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}
