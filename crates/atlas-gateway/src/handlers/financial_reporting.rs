//! Financial Reporting API Handlers
//!
//! REST endpoints for Oracle Fusion-inspired Financial Reporting Center.

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
// Request Types
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTemplateRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(default = "default_report_type")]
    pub report_type: String,
    #[serde(default = "default_currency")]
    pub currency_code: String,
    #[serde(default = "default_display_order")]
    pub row_display_order: String,
    #[serde(default = "default_display_order")]
    pub column_display_order: String,
    #[serde(default = "default_rounding")]
    pub rounding_option: String,
    #[serde(default)]
    pub show_zero_amounts: bool,
    #[serde(default)]
    pub segment_filter: serde_json::Value,
}

fn default_report_type() -> String { "custom".to_string() }
fn default_currency() -> String { "USD".to_string() }
fn default_display_order() -> String { "sequential".to_string() }
fn default_rounding() -> String { "none".to_string() }

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRowRequest {
    pub row_number: i32,
    #[serde(default = "default_line_type")]
    pub line_type: String,
    pub label: String,
    #[serde(default)]
    pub indent_level: i32,
    pub account_range_from: Option<String>,
    pub account_range_to: Option<String>,
    #[serde(default)]
    pub account_filter: serde_json::Value,
    pub compute_action: Option<String>,
    #[serde(default)]
    pub compute_source_rows: serde_json::Value,
    #[serde(default = "default_true")]
    pub show_line: bool,
    #[serde(default)]
    pub bold: bool,
    #[serde(default)]
    pub underline: bool,
    #[serde(default)]
    pub double_underline: bool,
    #[serde(default)]
    pub page_break_before: bool,
    pub scaling_factor: Option<String>,
    pub parent_row_id: Option<Uuid>,
}

fn default_line_type() -> String { "data".to_string() }
fn default_true() -> bool { true }

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateColumnRequest {
    pub column_number: i32,
    #[serde(default = "default_column_type")]
    pub column_type: String,
    pub header_label: String,
    pub sub_header_label: Option<String>,
    #[serde(default)]
    pub period_offset: i32,
    #[serde(default = "default_period_type")]
    pub period_type: String,
    pub compute_action: Option<String>,
    #[serde(default)]
    pub compute_source_columns: serde_json::Value,
    #[serde(default = "default_true")]
    pub show_column: bool,
    pub column_width: Option<i32>,
    pub format_override: Option<String>,
}

fn default_column_type() -> String { "actuals".to_string() }
fn default_period_type() -> String { "period".to_string() }

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateReportRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub as_of_date: Option<chrono::NaiveDate>,
    pub period_from: Option<chrono::NaiveDate>,
    pub period_to: Option<chrono::NaiveDate>,
    pub currency_code: Option<String>,
    #[serde(default)]
    pub segment_filter: serde_json::Value,
    #[serde(default)]
    pub include_unposted: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateQuickTemplateRequest {
    pub code: String,
    pub name: String,
    #[serde(default = "default_currency")]
    pub currency_code: String,
}

#[derive(Debug, Deserialize)]
pub struct TemplateListQuery {
    pub report_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RunListQuery {
    pub template_id: Option<Uuid>,
    pub status: Option<String>,
}

// ============================================================================
// Handlers
// ============================================================================

/// Create a new report template
pub async fn create_financial_template(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<CreateTemplateRequest>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_org_id(&claims)?;
    let created_by = parse_user_id(&claims);

    match state.financial_reporting_engine.create_template(
        org_id,
        &body.code,
        &body.name,
        body.description.as_deref(),
        &body.report_type,
        &body.currency_code,
        &body.row_display_order,
        &body.column_display_order,
        &body.rounding_option,
        body.show_zero_amounts,
        body.segment_filter,
        created_by,
    ).await {
        Ok(t) => Ok(Json(serde_json::to_value(t).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err(map_error(e)),
    }
}

/// Get a template by code
pub async fn get_financial_template(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(code): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_org_id(&claims)?;
    match state.financial_reporting_engine.get_template(org_id, &code).await {
        Ok(Some(t)) => Ok(Json(serde_json::to_value(t).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Ok(None) => Err((axum::http::StatusCode::NOT_FOUND, Json(json!({"error": "Template not found"})))),
        Err(e) => Err(map_error(e)),
    }
}

/// List templates
pub async fn list_financial_templates(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<TemplateListQuery>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_org_id(&claims)?;
    match state.financial_reporting_engine.list_templates(org_id, query.report_type.as_deref()).await {
        Ok(templates) => Ok(Json(json!(templates))),
        Err(e) => Err(map_error(e)),
    }
}

/// Delete a template
pub async fn delete_financial_template(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(code): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_org_id(&claims)?;
    match state.financial_reporting_engine.delete_template(org_id, &code).await {
        Ok(()) => Ok(Json(json!({"message": "Template deleted"}))),
        Err(e) => Err(map_error(e)),
    }
}

/// Add a row to a template
pub async fn create_financial_row(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(template_id): Path<Uuid>,
    Json(body): Json<CreateRowRequest>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_org_id(&claims)?;
    match state.financial_reporting_engine.create_row(
        org_id,
        template_id,
        body.row_number,
        &body.line_type,
        &body.label,
        body.indent_level,
        body.account_range_from.as_deref(),
        body.account_range_to.as_deref(),
        body.account_filter,
        body.compute_action.as_deref(),
        body.compute_source_rows,
        body.show_line,
        body.bold,
        body.underline,
        body.double_underline,
        body.page_break_before,
        body.scaling_factor.as_deref(),
        body.parent_row_id,
    ).await {
        Ok(row) => Ok(Json(serde_json::to_value(row).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err(map_error(e)),
    }
}

/// List rows for a template
pub async fn list_financial_rows(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(template_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    match state.financial_reporting_engine.list_rows(template_id).await {
        Ok(rows) => Ok(Json(json!(rows))),
        Err(e) => Err(map_error(e)),
    }
}

/// Delete a row
pub async fn delete_financial_row(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    match state.financial_reporting_engine.delete_row(id).await {
        Ok(()) => Ok(Json(json!({"message": "Row deleted"}))),
        Err(e) => Err(map_error(e)),
    }
}

/// Add a column to a template
pub async fn create_financial_column(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(template_id): Path<Uuid>,
    Json(body): Json<CreateColumnRequest>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_org_id(&claims)?;
    match state.financial_reporting_engine.create_column(
        org_id,
        template_id,
        body.column_number,
        &body.column_type,
        &body.header_label,
        body.sub_header_label.as_deref(),
        body.period_offset,
        &body.period_type,
        body.compute_action.as_deref(),
        body.compute_source_columns,
        body.show_column,
        body.column_width,
        body.format_override.as_deref(),
    ).await {
        Ok(col) => Ok(Json(serde_json::to_value(col).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err(map_error(e)),
    }
}

/// List columns for a template
pub async fn list_financial_columns(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(template_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    match state.financial_reporting_engine.list_columns(template_id).await {
        Ok(cols) => Ok(Json(json!(cols))),
        Err(e) => Err(map_error(e)),
    }
}

/// Delete a column
pub async fn delete_financial_column(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    match state.financial_reporting_engine.delete_column(id).await {
        Ok(()) => Ok(Json(json!({"message": "Column deleted"}))),
        Err(e) => Err(map_error(e)),
    }
}

/// Generate a report
pub async fn generate_financial_report(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(template_code): Path<String>,
    Json(body): Json<GenerateReportRequest>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_org_id(&claims)?;
    let user_id = parse_user_id(&claims);

    match state.financial_reporting_engine.generate_report(
        org_id,
        &template_code,
        body.name.as_deref(),
        body.description.as_deref(),
        body.as_of_date,
        body.period_from,
        body.period_to,
        body.currency_code.as_deref(),
        body.segment_filter,
        body.include_unposted,
        user_id,
    ).await {
        Ok(run) => Ok(Json(serde_json::to_value(run).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err(map_error(e)),
    }
}

/// Get a report run
pub async fn get_financial_run(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    match state.financial_reporting_engine.get_run(id).await {
        Ok(Some(run)) => Ok(Json(serde_json::to_value(run).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Ok(None) => Err((axum::http::StatusCode::NOT_FOUND, Json(json!({"error": "Report run not found"})))),
        Err(e) => Err(map_error(e)),
    }
}

/// List report runs
pub async fn list_financial_runs(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<RunListQuery>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_org_id(&claims)?;
    match state.financial_reporting_engine.list_runs(org_id, query.template_id, query.status.as_deref()).await {
        Ok(runs) => Ok(Json(json!(runs))),
        Err(e) => Err(map_error(e)),
    }
}

/// Get run results
pub async fn get_financial_run_results(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(run_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    match state.financial_reporting_engine.get_run_results(run_id).await {
        Ok(results) => Ok(Json(json!(results))),
        Err(e) => Err(map_error(e)),
    }
}

/// Approve a report run
pub async fn approve_financial_report(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let user_id = parse_user_id(&claims).ok_or_else(|| {
        (axum::http::StatusCode::UNAUTHORIZED, Json(json!({"error": "Invalid user"})))
    })?;

    match state.financial_reporting_engine.approve_report(id, user_id).await {
        Ok(run) => Ok(Json(serde_json::to_value(run).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err(map_error(e)),
    }
}

/// Publish a report run
pub async fn publish_financial_report(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let user_id = parse_user_id(&claims).ok_or_else(|| {
        (axum::http::StatusCode::UNAUTHORIZED, Json(json!({"error": "Invalid user"})))
    })?;

    match state.financial_reporting_engine.publish_report(id, user_id).await {
        Ok(run) => Ok(Json(serde_json::to_value(run).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err(map_error(e)),
    }
}

/// Archive a report run
pub async fn archive_financial_report(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    match state.financial_reporting_engine.archive_report(id).await {
        Ok(run) => Ok(Json(serde_json::to_value(run).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err(map_error(e)),
    }
}

/// Create a quick Trial Balance template
pub async fn create_financial_trial_balance(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<CreateQuickTemplateRequest>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_org_id(&claims)?;
    let created_by = parse_user_id(&claims);
    match state.financial_reporting_engine.create_trial_balance_template(
        org_id, &body.code, &body.name, &body.currency_code, created_by,
    ).await {
        Ok(t) => Ok(Json(serde_json::to_value(t).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err(map_error(e)),
    }
}

/// Create a quick Income Statement template
pub async fn create_financial_income_statement(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<CreateQuickTemplateRequest>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_org_id(&claims)?;
    let created_by = parse_user_id(&claims);
    match state.financial_reporting_engine.create_income_statement_template(
        org_id, &body.code, &body.name, &body.currency_code, created_by,
    ).await {
        Ok(t) => Ok(Json(serde_json::to_value(t).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err(map_error(e)),
    }
}

/// Create a quick Balance Sheet template
pub async fn create_financial_balance_sheet(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<CreateQuickTemplateRequest>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_org_id(&claims)?;
    let created_by = parse_user_id(&claims);
    match state.financial_reporting_engine.create_balance_sheet_template(
        org_id, &body.code, &body.name, &body.currency_code, created_by,
    ).await {
        Ok(t) => Ok(Json(serde_json::to_value(t).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err(map_error(e)),
    }
}

/// Add a template to favourites
pub async fn add_financial_favourite(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(template_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_org_id(&claims)?;
    let user_id = parse_user_id(&claims).ok_or_else(|| {
        (axum::http::StatusCode::UNAUTHORIZED, Json(json!({"error": "Invalid user"})))
    })?;

    match state.financial_reporting_engine.add_favourite(org_id, user_id, template_id, None).await {
        Ok(fav) => Ok(Json(serde_json::to_value(fav).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err(map_error(e)),
    }
}

/// List user favourites
pub async fn list_financial_favourites(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_org_id(&claims)?;
    let user_id = parse_user_id(&claims).ok_or_else(|| {
        (axum::http::StatusCode::UNAUTHORIZED, Json(json!({"error": "Invalid user"})))
    })?;

    match state.financial_reporting_engine.list_favourites(org_id, user_id).await {
        Ok(favs) => Ok(Json(json!(favs))),
        Err(e) => Err(map_error(e)),
    }
}

/// Remove from favourites
pub async fn remove_financial_favourite(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(template_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_org_id(&claims)?;
    let user_id = parse_user_id(&claims).ok_or_else(|| {
        (axum::http::StatusCode::UNAUTHORIZED, Json(json!({"error": "Invalid user"})))
    })?;

    match state.financial_reporting_engine.remove_favourite(org_id, user_id, template_id).await {
        Ok(()) => Ok(Json(json!({"message": "Favourite removed"}))),
        Err(e) => Err(map_error(e)),
    }
}

/// Get financial reporting dashboard summary
pub async fn get_financial_dashboard(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_org_id(&claims)?;
    match state.financial_reporting_engine.get_dashboard_summary(org_id).await {
        Ok(summary) => Ok(Json(serde_json::to_value(summary).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err(map_error(e)),
    }
}

// ============================================================================
// Helpers
// ============================================================================

fn parse_org_id(claims: &Claims) -> Result<Uuid, (StatusCode, Json<Value>)> {
    parse_uuid(&claims.org_id)
}

fn parse_user_id(claims: &Claims) -> Option<Uuid> {
    claims.sub.parse().ok()
}

fn map_error(e: atlas_shared::AtlasError) -> (axum::http::StatusCode, Json<serde_json::Value>) {
    let (status, message) = match e {
        atlas_shared::AtlasError::ValidationFailed(msg) => (axum::http::StatusCode::BAD_REQUEST, msg),
        atlas_shared::AtlasError::EntityNotFound(msg) => (axum::http::StatusCode::NOT_FOUND, msg),
        atlas_shared::AtlasError::WorkflowError(msg) => (axum::http::StatusCode::CONFLICT, msg),
        atlas_shared::AtlasError::Forbidden(msg) => (axum::http::StatusCode::FORBIDDEN, msg),
        _ => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("{:?}", e)),
    };
    (status, Json(json!({"error": message})))
}
