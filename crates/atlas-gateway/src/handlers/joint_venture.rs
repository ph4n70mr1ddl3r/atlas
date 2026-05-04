//! Joint Venture Management Handlers
//!
//! Oracle Fusion Cloud ERP: Financials > Joint Venture Management
//!
//! API endpoints for joint venture agreements, partner ownership,
//! AFEs (Authorizations for Expenditure), cost/revenue distributions,
//! and Joint Interest Billing (JIB).

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

// ============================================================================
// Joint Venture CRUD
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateVentureRequest {
    pub venture_number: String,
    pub name: String,
    pub description: Option<String>,
    pub operator_name: Option<String>,
    pub currency_code: String,
    pub start_date: Option<chrono::NaiveDate>,
    pub end_date: Option<chrono::NaiveDate>,
    pub accounting_method: String,
    pub billing_cycle: String,
}

pub async fn create_venture(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateVentureRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.joint_venture_engine.create_venture(
        org_id, &payload.venture_number, &payload.name,
        payload.description.as_deref(), None,
        payload.operator_name.as_deref(), &payload.currency_code,
        payload.start_date, payload.end_date,
        &payload.accounting_method, &payload.billing_cycle,
        None, None, None, None, None, Some(user_id),
    ).await {
        Ok(venture) => Ok((StatusCode::CREATED, Json(serde_json::to_value(venture).unwrap()))),
        Err(e) => {
            error!("Failed to create joint venture: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListVenturesQuery {
    pub status: Option<String>,
}

pub async fn list_ventures(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListVenturesQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.joint_venture_engine.list_ventures(org_id, query.status.as_deref()).await {
        Ok(ventures) => Ok(Json(serde_json::json!({ "data": ventures }))),
        Err(e) => { error!("Failed to list ventures: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn get_venture(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.joint_venture_engine.get_venture(id).await {
        Ok(Some(v)) => Ok(Json(serde_json::to_value(v).unwrap())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Failed to get venture: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn activate_venture(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.joint_venture_engine.activate_venture(id).await {
        Ok(v) => Ok(Json(serde_json::to_value(v).unwrap())),
        Err(e) => {
            error!("Failed to activate venture: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn close_venture(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.joint_venture_engine.close_venture(id).await {
        Ok(v) => Ok(Json(serde_json::to_value(v).unwrap())),
        Err(e) => {
            error!("Failed to close venture: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// ============================================================================
// Partner Management
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct AddPartnerRequest {
    pub partner_id: Uuid,
    pub partner_name: String,
    pub partner_type: String,
    pub ownership_percentage: String,
    pub revenue_interest_pct: Option<String>,
    pub cost_bearing_pct: Option<String>,
    pub role: String,
    pub billing_contact: Option<String>,
    pub billing_email: Option<String>,
    pub billing_address: Option<String>,
    pub effective_from: chrono::NaiveDate,
    pub effective_to: Option<chrono::NaiveDate>,
}

pub async fn add_partner(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(venture_id): Path<Uuid>,
    Json(payload): Json<AddPartnerRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.joint_venture_engine.add_partner(
        org_id, venture_id, payload.partner_id, &payload.partner_name,
        &payload.partner_type, &payload.ownership_percentage,
        payload.revenue_interest_pct.as_deref(), payload.cost_bearing_pct.as_deref(),
        &payload.role, payload.billing_contact.as_deref(),
        payload.billing_email.as_deref(), payload.billing_address.as_deref(),
        payload.effective_from, payload.effective_to, Some(user_id),
    ).await {
        Ok(partner) => Ok((StatusCode::CREATED, Json(serde_json::to_value(partner).unwrap()))),
        Err(e) => {
            error!("Failed to add partner: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn list_partners(
    State(state): State<Arc<AppState>>,
    Path(venture_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.joint_venture_engine.list_partners(venture_id).await {
        Ok(partners) => Ok(Json(serde_json::json!({ "data": partners }))),
        Err(e) => { error!("Failed to list partners: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// ============================================================================
// AFE Management
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateAfeRequest {
    pub afe_number: String,
    pub title: String,
    pub description: Option<String>,
    pub estimated_cost: String,
    pub currency_code: String,
    pub cost_center: Option<String>,
    pub work_area: Option<String>,
    pub well_name: Option<String>,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
}

pub async fn create_afe(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(venture_id): Path<Uuid>,
    Json(payload): Json<CreateAfeRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.joint_venture_engine.create_afe(
        org_id, venture_id, &payload.afe_number, &payload.title,
        payload.description.as_deref(), &payload.estimated_cost,
        &payload.currency_code, payload.cost_center.as_deref(),
        payload.work_area.as_deref(), payload.well_name.as_deref(),
        payload.effective_from, payload.effective_to, Some(user_id),
    ).await {
        Ok(afe) => Ok((StatusCode::CREATED, Json(serde_json::to_value(afe).unwrap()))),
        Err(e) => {
            error!("Failed to create AFE: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn submit_afe(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.joint_venture_engine.submit_afe(id).await {
        Ok(a) => Ok(Json(serde_json::to_value(a).unwrap())),
        Err(e) => {
            error!("Failed to submit AFE: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn approve_afe(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.joint_venture_engine.approve_afe(id, user_id).await {
        Ok(a) => Ok(Json(serde_json::to_value(a).unwrap())),
        Err(e) => {
            error!("Failed to approve AFE: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn list_afes(
    State(state): State<Arc<AppState>>,
    Path(venture_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.joint_venture_engine.list_afes(venture_id, None).await {
        Ok(afes) => Ok(Json(serde_json::json!({ "data": afes }))),
        Err(e) => { error!("Failed to list AFEs: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// ============================================================================
// Cost Distribution
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateCostDistributionRequest {
    pub distribution_number: String,
    pub afe_id: Option<Uuid>,
    pub description: Option<String>,
    pub total_amount: String,
    pub currency_code: String,
    pub cost_type: String,
    pub distribution_date: chrono::NaiveDate,
    pub source_type: Option<String>,
    pub source_id: Option<Uuid>,
    pub source_number: Option<String>,
}

pub async fn create_cost_distribution(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(venture_id): Path<Uuid>,
    Json(payload): Json<CreateCostDistributionRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.joint_venture_engine.create_cost_distribution(
        org_id, venture_id, &payload.distribution_number,
        payload.afe_id, payload.description.as_deref(),
        &payload.total_amount, &payload.currency_code,
        &payload.cost_type, payload.distribution_date,
        payload.source_type.as_deref(), payload.source_id,
        payload.source_number.as_deref(), Some(user_id),
    ).await {
        Ok((dist, _lines)) => Ok((StatusCode::CREATED, Json(serde_json::to_value(dist).unwrap()))),
        Err(e) => {
            let msg = e.to_string();
            error!("Failed to create cost distribution: {}", msg);
            let _body = Json(serde_json::json!({"error": msg}));
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn list_cost_distributions(
    State(state): State<Arc<AppState>>,
    Path(venture_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.joint_venture_engine.list_cost_distributions(venture_id, None).await {
        Ok(dists) => Ok(Json(serde_json::json!({ "data": dists }))),
        Err(e) => { error!("Failed to list cost distributions: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn post_cost_distribution(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.joint_venture_engine.post_cost_distribution(id).await {
        Ok(d) => Ok(Json(serde_json::to_value(d).unwrap())),
        Err(e) => {
            error!("Failed to post cost distribution: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// ============================================================================
// Dashboard
// ============================================================================

pub async fn get_joint_venture_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.joint_venture_engine.get_dashboard(org_id).await {
        Ok(dashboard) => Ok(Json(serde_json::to_value(dashboard).unwrap())),
        Err(e) => { error!("Failed to get JV dashboard: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}
