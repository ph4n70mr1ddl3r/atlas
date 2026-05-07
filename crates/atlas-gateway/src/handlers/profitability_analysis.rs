//! Profitability Analysis Handlers
//!
//! Oracle Fusion: Financials > Profitability Analysis
//! API endpoints for profitability segments, analysis runs, templates, and dashboards.

use axum::{
    extract::{Path, Query, State, Extension},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
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
    pub segment_type: Option<String>,
    pub is_active: Option<bool>,
}

// ============================================================================
// Segment Handlers
// ============================================================================

/// Create a profitability segment
pub async fn create_segment(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let org_id = parse_uuid(&claims.org_id).unwrap_or_default();
    let user_id = claims.sub.parse().ok();

    let code = body["segmentCode"].as_str().unwrap_or("").to_string();
    let name = body["segmentName"].as_str().unwrap_or("").to_string();
    let segment_type = body["segmentType"].as_str().unwrap_or("product").to_string();
    let description = body["description"].as_str().map(|s| s.to_string());
    let parent_id = body["parentSegmentId"].as_str().and_then(|s| s.parse().ok());
    let sort_order = body["sortOrder"].as_i64().map(|v| v as i32);

    match state.profitability_engine.create_segment(
        org_id, &code, &name, &segment_type,
        description.as_deref(), parent_id, sort_order, user_id,
    ).await {
        Ok(seg) => (StatusCode::CREATED, Json(serde_json::to_value(seg).unwrap())).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// List profitability segments
pub async fn list_segments(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<ListQuery>,
) -> impl IntoResponse {
    let org_id = parse_uuid(&claims.org_id).unwrap_or_default();
    match state.profitability_engine.list_segments(
        org_id,
        query.segment_type.as_deref(),
        query.is_active,
    ).await {
        Ok(segments) => Json(serde_json::json!({"data": segments})).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// Get a segment by ID
pub async fn get_segment(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.profitability_engine.get_segment(id).await {
        Ok(Some(seg)) => Json(serde_json::to_value(seg).unwrap()).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Segment not found"}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// Delete a segment by code
pub async fn delete_segment(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(code): Path<String>,
) -> impl IntoResponse {
    let org_id = parse_uuid(&claims.org_id).unwrap_or_default();
    match state.profitability_engine.delete_segment(org_id, &code).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

// ============================================================================
// Run Handlers
// ============================================================================

/// Create an analysis run
pub async fn create_run(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let org_id = parse_uuid(&claims.org_id).unwrap_or_default();
    let user_id = claims.sub.parse().ok();

    let run_number = body["runNumber"].as_str().unwrap_or("").to_string();
    let run_name = body["runName"].as_str().unwrap_or("").to_string();
    let analysis_type = body["analysisType"].as_str().unwrap_or("standard").to_string();
    let period_from = body["periodFrom"].as_str()
        .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
        .unwrap_or(chrono::Utc::now().naive_utc().date());
    let period_to = body["periodTo"].as_str()
        .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
        .unwrap_or(chrono::Utc::now().naive_utc().date());
    let currency_code = body["currencyCode"].as_str().unwrap_or("USD").to_string();
    let comparison_run_id = body["comparisonRunId"].as_str().and_then(|s| s.parse().ok());
    let notes = body["notes"].as_str().map(|s| s.to_string());

    match state.profitability_engine.create_run(
        org_id, &run_number, &run_name, &analysis_type,
        period_from, period_to, &currency_code,
        comparison_run_id, notes.as_deref(), user_id,
    ).await {
        Ok(run) => (StatusCode::CREATED, Json(serde_json::to_value(run).unwrap())).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// List analysis runs
pub async fn list_runs(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<ListQuery>,
) -> impl IntoResponse {
    let org_id = parse_uuid(&claims.org_id).unwrap_or_default();
    match state.profitability_engine.list_runs(org_id, query.status.as_deref()).await {
        Ok(runs) => Json(serde_json::json!({"data": runs})).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// Get a run by ID
pub async fn get_run(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.profitability_engine.get_run(id).await {
        Ok(Some(run)) => Json(serde_json::to_value(run).unwrap()).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Run not found"}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// Transition run status
pub async fn transition_run(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let new_status = body["status"].as_str().unwrap_or("");
    match state.profitability_engine.transition_run(id, new_status).await {
        Ok(run) => Json(serde_json::to_value(run).unwrap()).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// Delete a draft run by number
pub async fn delete_run(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(run_number): Path<String>,
) -> impl IntoResponse {
    let org_id = parse_uuid(&claims.org_id).unwrap_or_default();
    match state.profitability_engine.delete_run(org_id, &run_number).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

// ============================================================================
// Run Line Handlers
// ============================================================================

/// Add a line to a run
pub async fn add_run_line(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(run_id): Path<Uuid>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let org_id = parse_uuid(&claims.org_id).unwrap_or_default();

    let segment_id = body["segmentId"].as_str().and_then(|s| s.parse().ok());
    let segment_code = body["segmentCode"].as_str().map(|s| s.to_string());
    let segment_name = body["segmentName"].as_str().map(|s| s.to_string());
    let segment_type = body["segmentType"].as_str().map(|s| s.to_string());
    let line_number = body["lineNumber"].as_i64().unwrap_or(1) as i32;
    let revenue = body["revenue"].as_f64().unwrap_or(0.0);
    let cogs = body["costOfGoodsSold"].as_f64().unwrap_or(0.0);
    let opex = body["operatingExpenses"].as_f64().unwrap_or(0.0);
    let other_income = body["otherIncome"].as_f64().unwrap_or(0.0);
    let other_expense = body["otherExpense"].as_f64().unwrap_or(0.0);

    match state.profitability_engine.add_run_line(
        org_id, run_id,
        segment_id, segment_code.as_deref(), segment_name.as_deref(), segment_type.as_deref(),
        line_number, revenue, cogs, opex, other_income, other_expense,
    ).await {
        Ok(line) => (StatusCode::CREATED, Json(serde_json::to_value(line).unwrap())).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// List lines for a run
pub async fn list_run_lines(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(run_id): Path<Uuid>,
) -> impl IntoResponse {
    match state.profitability_engine.list_run_lines(run_id).await {
        Ok(lines) => Json(serde_json::json!({"data": lines})).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// Remove a line from a run
pub async fn remove_run_line(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path((run_id, line_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    match state.profitability_engine.remove_run_line(run_id, line_id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

// ============================================================================
// Template Handlers
// ============================================================================

/// Create a template
pub async fn create_template(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let org_id = parse_uuid(&claims.org_id).unwrap_or_default();
    let user_id = claims.sub.parse().ok();

    let code = body["templateCode"].as_str().unwrap_or("").to_string();
    let name = body["templateName"].as_str().unwrap_or("").to_string();
    let description = body["description"].as_str().map(|s| s.to_string());
    let segment_type = body["segmentType"].as_str().unwrap_or("product").to_string();
    let includes_cogs = body["includesCogs"].as_bool();
    let includes_operating = body["includesOperating"].as_bool();
    let includes_other = body["includesOther"].as_bool();
    let auto_calculate = body["autoCalculate"].as_bool();

    match state.profitability_engine.create_template(
        org_id, &code, &name, description.as_deref(), &segment_type,
        includes_cogs, includes_operating, includes_other, auto_calculate, user_id,
    ).await {
        Ok(tmpl) => (StatusCode::CREATED, Json(serde_json::to_value(tmpl).unwrap())).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// List templates
pub async fn list_templates(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<ListQuery>,
) -> impl IntoResponse {
    let org_id = parse_uuid(&claims.org_id).unwrap_or_default();
    match state.profitability_engine.list_templates(org_id, query.is_active).await {
        Ok(templates) => Json(serde_json::json!({"data": templates})).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// Delete a template by code
pub async fn delete_template(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(code): Path<String>,
) -> impl IntoResponse {
    let org_id = parse_uuid(&claims.org_id).unwrap_or_default();
    match state.profitability_engine.delete_template(org_id, &code).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

// ============================================================================
// Dashboard Handler
// ============================================================================

/// Get profitability dashboard
pub async fn get_dashboard(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> impl IntoResponse {
    let org_id = parse_uuid(&claims.org_id).unwrap_or_default();
    match state.profitability_engine.get_dashboard(org_id).await {
        Ok(dashboard) => Json(serde_json::to_value(dashboard).unwrap()).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}
