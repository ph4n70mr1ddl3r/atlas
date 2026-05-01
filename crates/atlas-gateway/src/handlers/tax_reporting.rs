//! Tax Reporting Handlers
//!
//! Oracle Fusion Cloud ERP: Financials > Tax > Tax Reporting

use axum::{
    extract::{State, Path},
    Json,
    http::StatusCode,
    Extension,
};
use serde::Deserialize;
use crate::AppState;
use crate::handlers::auth::Claims;
use std::sync::Arc;
use uuid::Uuid;
use tracing::{info, error};

#[derive(Debug, Deserialize)]
pub struct CreateTaxTemplateRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub tax_type: String,
    pub jurisdiction_code: Option<String>,
    pub filing_frequency: String,
    pub return_form_number: Option<String>,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
}

pub async fn create_tax_template(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateTaxTemplateRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.tax_reporting_engine.create_template(
        org_id, &payload.code, &payload.name, payload.description.as_deref(),
        &payload.tax_type, payload.jurisdiction_code.as_deref(),
        &payload.filing_frequency, payload.return_form_number.as_deref(),
        payload.effective_from, payload.effective_to, Some(user_id),
    ).await {
        Ok(template) => Ok((StatusCode::CREATED, Json(serde_json::to_value(template).unwrap()))),
        Err(e) => {
            error!("Failed to create tax template: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn list_tax_templates(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.tax_reporting_engine.list_templates(org_id).await {
        Ok(templates) => Ok(Json(serde_json::json!({ "data": templates }))),
        Err(e) => { error!("Failed to list templates: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateTaxReturnRequest {
    pub template_id: Uuid,
    pub filing_period_start: chrono::NaiveDate,
    pub filing_period_end: chrono::NaiveDate,
    pub filing_due_date: Option<chrono::NaiveDate>,
}

pub async fn create_tax_return(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateTaxReturnRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.tax_reporting_engine.create_return(
        org_id, payload.template_id,
        payload.filing_period_start, payload.filing_period_end,
        payload.filing_due_date, Some(user_id),
    ).await {
        Ok(tax_return) => Ok((StatusCode::CREATED, Json(serde_json::to_value(tax_return).unwrap()))),
        Err(e) => { error!("Failed to create tax return: {}", e); Err(StatusCode::BAD_REQUEST) }
    }
}

pub async fn list_tax_returns(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.tax_reporting_engine.list_returns(org_id, None).await {
        Ok(returns) => Ok(Json(serde_json::json!({ "data": returns }))),
        Err(e) => { error!("Failed to list returns: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn get_tax_return(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.tax_reporting_engine.get_return(id).await {
        Ok(Some(r)) => Ok(Json(serde_json::to_value(r).unwrap())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Failed to get return: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

#[derive(Debug, Deserialize)]
pub struct FileTaxReturnRequest {
    pub filing_method: String,
    pub filing_reference: Option<String>,
}

pub async fn file_tax_return(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<FileTaxReturnRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.tax_reporting_engine.file_return(
        id, &payload.filing_method, payload.filing_reference.as_deref(), Some(user_id),
    ).await {
        Ok(r) => Ok(Json(serde_json::to_value(r).unwrap())),
        Err(e) => { error!("Failed to file return: {}", e); Err(StatusCode::BAD_REQUEST) }
    }
}

#[derive(Debug, Deserialize)]
pub struct PayTaxReturnRequest {
    pub payment_amount: String,
    pub payment_reference: Option<String>,
}

pub async fn pay_tax_return(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<PayTaxReturnRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.tax_reporting_engine.mark_paid(
        id, &payload.payment_amount, payload.payment_reference.as_deref(),
    ).await {
        Ok(r) => Ok(Json(serde_json::to_value(r).unwrap())),
        Err(e) => { error!("Failed to mark paid: {}", e); Err(StatusCode::BAD_REQUEST) }
    }
}

pub async fn get_tax_reporting_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.tax_reporting_engine.get_dashboard(org_id).await {
        Ok(dashboard) => Ok(Json(serde_json::to_value(dashboard).unwrap())),
        Err(e) => { error!("Failed to get dashboard: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}
