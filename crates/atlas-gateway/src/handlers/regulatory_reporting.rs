//! Regulatory Reporting Handlers
//!
//! Oracle Fusion: Financials > Regulatory Reporting

use axum::{
    extract::{State, Path, Query},
    Json,
    http::StatusCode,
    Extension,
};
use serde::Deserialize;
use crate::AppState;
use crate::handlers::auth::Claims;
use std::sync::Arc;
use uuid::Uuid;
use tracing::error;

// Templates
#[derive(Debug, Deserialize)]
pub struct CreateRegTemplateRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub authority: String,
    pub report_category: String,
    pub filing_frequency: String,
    pub output_format: String,
    pub row_definitions: Option<serde_json::Value>,
    pub column_definitions: Option<serde_json::Value>,
    pub validation_rules: Option<serde_json::Value>,
}

pub async fn create_reg_template(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateRegTemplateRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.regulatory_reporting_engine.create_template(
        org_id, &payload.code, &payload.name, payload.description.as_deref(),
        &payload.authority, &payload.report_category, &payload.filing_frequency,
        &payload.output_format,
        payload.row_definitions.clone().unwrap_or(serde_json::json!([])),
        payload.column_definitions.clone().unwrap_or(serde_json::json!([])),
        payload.validation_rules.clone().unwrap_or(serde_json::json!([])),
        Some(user_id),
    ).await {
        Ok(t) => Ok((StatusCode::CREATED, Json(serde_json::to_value(t).unwrap()))),
        Err(e) => {
            error!("Failed to create regulatory template: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn list_reg_templates(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.regulatory_reporting_engine.list_templates(org_id, None, None).await {
        Ok(templates) => Ok(Json(serde_json::json!({ "data": templates }))),
        Err(e) => { error!("Failed to list templates: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn delete_reg_template(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(code): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.regulatory_reporting_engine.delete_template(org_id, &code).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            error!("Failed to delete template: {}", e);
            Err(match e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// Reports
#[derive(Debug, Deserialize)]
pub struct CreateRegReportRequest {
    pub template_code: String,
    pub report_number: String,
    pub name: String,
    pub period_start: chrono::NaiveDate,
    pub period_end: chrono::NaiveDate,
}

pub async fn create_reg_report(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateRegReportRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.regulatory_reporting_engine.create_report(
        org_id, &payload.template_code, &payload.report_number, &payload.name,
        payload.period_start, payload.period_end, Some(user_id),
    ).await {
        Ok(r) => Ok((StatusCode::CREATED, Json(serde_json::to_value(r).unwrap()))),
        Err(e) => {
            error!("Failed to create report: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListRegReportsQuery { pub status: Option<String>, pub authority: Option<String> }

pub async fn list_reg_reports(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListRegReportsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.regulatory_reporting_engine.list_reports(org_id, query.status.as_deref(), query.authority.as_deref()).await {
        Ok(reports) => Ok(Json(serde_json::json!({ "data": reports }))),
        Err(e) => { error!("Failed to list reports: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn submit_for_review(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.regulatory_reporting_engine.submit_for_review(id, user_id).await {
        Ok(r) => Ok(Json(serde_json::to_value(r).unwrap())),
        Err(e) => {
            error!("Failed to submit for review: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn approve_reg_report(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.regulatory_reporting_engine.approve_report(id, user_id).await {
        Ok(r) => Ok(Json(serde_json::to_value(r).unwrap())),
        Err(e) => {
            error!("Failed to approve report: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct RejectRegReportRequest { pub reason: String }

pub async fn reject_reg_report(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<RejectRegReportRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.regulatory_reporting_engine.reject_report(id, &payload.reason).await {
        Ok(r) => Ok(Json(serde_json::to_value(r).unwrap())),
        Err(e) => {
            error!("Failed to reject report: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) | atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// Filings
#[derive(Debug, Deserialize)]
pub struct CreateFilingRequest {
    pub template_code: Option<String>,
    pub authority: String,
    pub report_name: String,
    pub filing_frequency: String,
    pub period_start: chrono::NaiveDate,
    pub period_end: chrono::NaiveDate,
    pub due_date: chrono::NaiveDate,
}

pub async fn create_filing(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateFilingRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.regulatory_reporting_engine.create_filing(
        org_id, payload.template_code.as_deref(), &payload.authority,
        &payload.report_name, &payload.filing_frequency,
        payload.period_start, payload.period_end, payload.due_date, Some(user_id),
    ).await {
        Ok(f) => Ok((StatusCode::CREATED, Json(serde_json::to_value(f).unwrap()))),
        Err(e) => {
            error!("Failed to create filing: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn list_filings(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListFilingsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.regulatory_reporting_engine.list_filings(org_id, query.status.as_deref()).await {
        Ok(filings) => Ok(Json(serde_json::json!({ "data": filings }))),
        Err(e) => { error!("Failed to list filings: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListFilingsQuery { pub status: Option<String> }

pub async fn get_regulatory_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.regulatory_reporting_engine.get_dashboard(org_id).await {
        Ok(dashboard) => Ok(Json(serde_json::to_value(dashboard).unwrap())),
        Err(e) => { error!("Failed to get dashboard: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}
