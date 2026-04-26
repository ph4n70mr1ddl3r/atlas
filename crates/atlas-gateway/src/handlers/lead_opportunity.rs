//! Lead and Opportunity Management Handlers
//!
//! Oracle Fusion Cloud: CX Sales > Leads & Opportunities
//!
//! API endpoints for managing sales leads, opportunity pipeline,
//! sales activities, lead scoring, and pipeline analytics.

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
// Lead Source Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateLeadSourceRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
}

pub async fn create_lead_source(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateLeadSourceRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).ok();
    match state.lead_opportunity_engine.create_lead_source(
        org_id, &payload.code, &payload.name, payload.description.as_deref(), user_id,
    ).await {
        Ok(src) => Ok((StatusCode::CREATED, Json(serde_json::to_value(src).unwrap_or_default()))),
        Err(e) => {
            error!("Failed to create lead source: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 409 => StatusCode::CONFLICT, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

pub async fn list_lead_sources(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.lead_opportunity_engine.list_lead_sources(org_id).await {
        Ok(sources) => Ok(Json(serde_json::json!({"data": sources}))),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn delete_lead_source(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(code): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.lead_opportunity_engine.delete_lead_source(org_id, &code).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// ============================================================================
// Lead Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateLeadRequest {
    pub lead_number: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub company: Option<String>,
    pub title: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub website: Option<String>,
    pub industry: Option<String>,
    pub lead_source_id: Option<Uuid>,
    pub lead_source_name: Option<String>,
    pub lead_rating_model_id: Option<Uuid>,
    #[serde(default = "default_zero")]
    pub estimated_value: String,
    #[serde(default = "default_usd")]
    pub currency_code: String,
    pub owner_id: Option<Uuid>,
    pub owner_name: Option<String>,
    pub notes: Option<String>,
}

fn default_zero() -> String { "0".to_string() }
fn default_usd() -> String { "USD".to_string() }

pub async fn create_lead(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateLeadRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).ok();
    match state.lead_opportunity_engine.create_lead(
        org_id, &payload.lead_number,
        payload.first_name.as_deref(), payload.last_name.as_deref(),
        payload.company.as_deref(), payload.title.as_deref(),
        payload.email.as_deref(), payload.phone.as_deref(),
        payload.website.as_deref(), payload.industry.as_deref(),
        payload.lead_source_id, payload.lead_source_name.as_deref(),
        payload.lead_rating_model_id, &payload.estimated_value,
        &payload.currency_code, payload.owner_id, payload.owner_name.as_deref(),
        payload.notes.as_deref(), user_id,
    ).await {
        Ok(lead) => Ok((StatusCode::CREATED, Json(serde_json::to_value(lead).unwrap_or_default()))),
        Err(e) => {
            error!("Failed to create lead: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 409 => StatusCode::CONFLICT, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

pub async fn get_lead(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.lead_opportunity_engine.get_lead(id).await {
        Ok(Some(l)) => Ok(Json(serde_json::to_value(l).unwrap_or_default())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListLeadsQuery {
    pub status: Option<String>,
    pub owner_id: Option<Uuid>,
}

pub async fn list_leads(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<ListLeadsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.lead_opportunity_engine.list_leads(org_id, params.status.as_deref(), params.owner_id).await {
        Ok(leads) => Ok(Json(serde_json::json!({"data": leads}))),
        Err(e) => {
            error!("Error: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateLeadStatusRequest {
    pub status: String,
}

pub async fn update_lead_status(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateLeadStatusRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.lead_opportunity_engine.update_lead_status(id, &payload.status).await {
        Ok(l) => Ok(Json(serde_json::to_value(l).unwrap_or_default())),
        Err(e) => {
            error!("Error: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 404 => StatusCode::NOT_FOUND, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateLeadScoreRequest {
    pub score: String,
    pub rating: String,
}

pub async fn update_lead_score(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateLeadScoreRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.lead_opportunity_engine.update_lead_score(id, &payload.score, &payload.rating).await {
        Ok(l) => Ok(Json(serde_json::to_value(l).unwrap_or_default())),
        Err(e) => {
            error!("Error: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 404 => StatusCode::NOT_FOUND, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

pub async fn convert_lead(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).ok();
    match state.lead_opportunity_engine.convert_lead(id, None, user_id).await {
        Ok((lead, opp)) => Ok(Json(serde_json::json!({"lead": lead, "opportunity": opp}))),
        Err(e) => {
            error!("Error converting lead: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 404 => StatusCode::NOT_FOUND, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

pub async fn delete_lead(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.lead_opportunity_engine.get_lead(id).await {
        Ok(Some(lead)) => {
            match state.lead_opportunity_engine.delete_lead(org_id, &lead.lead_number).await {
                Ok(()) => Ok(StatusCode::NO_CONTENT),
                Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
            }
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// ============================================================================
// Opportunity Stage Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateStageRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(default = "default_zero")]
    pub probability: String,
    #[serde(default)]
    pub display_order: i32,
    #[serde(default)]
    pub is_won: bool,
    #[serde(default)]
    pub is_lost: bool,
}

pub async fn create_opportunity_stage(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateStageRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).ok();
    match state.lead_opportunity_engine.create_opportunity_stage(
        org_id, &payload.code, &payload.name, payload.description.as_deref(),
        &payload.probability, payload.display_order, payload.is_won, payload.is_lost, user_id,
    ).await {
        Ok(stage) => Ok((StatusCode::CREATED, Json(serde_json::to_value(stage).unwrap_or_default()))),
        Err(e) => {
            error!("Failed to create stage: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 409 => StatusCode::CONFLICT, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

pub async fn list_opportunity_stages(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.lead_opportunity_engine.list_opportunity_stages(org_id).await {
        Ok(stages) => Ok(Json(serde_json::json!({"data": stages}))),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn delete_opportunity_stage(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(code): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.lead_opportunity_engine.delete_opportunity_stage(org_id, &code).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// ============================================================================
// Opportunity Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateOpportunityRequest {
    pub opportunity_number: String,
    pub name: String,
    pub description: Option<String>,
    pub customer_id: Option<Uuid>,
    pub customer_name: Option<String>,
    pub lead_id: Option<Uuid>,
    pub stage_id: Option<Uuid>,
    #[serde(default = "default_zero")]
    pub amount: String,
    #[serde(default = "default_usd")]
    pub currency_code: String,
    #[serde(default = "default_twenty_five")]
    pub probability: String,
    pub expected_close_date: Option<chrono::NaiveDate>,
    pub owner_id: Option<Uuid>,
    pub owner_name: Option<String>,
    pub contact_id: Option<Uuid>,
    pub contact_name: Option<String>,
}

fn default_twenty_five() -> String { "25".to_string() }

pub async fn create_opportunity(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateOpportunityRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).ok();
    match state.lead_opportunity_engine.create_opportunity(
        org_id, &payload.opportunity_number, &payload.name, payload.description.as_deref(),
        payload.customer_id, payload.customer_name.as_deref(),
        payload.lead_id, payload.stage_id,
        &payload.amount, &payload.currency_code, &payload.probability,
        payload.expected_close_date, payload.owner_id, payload.owner_name.as_deref(),
        payload.contact_id, payload.contact_name.as_deref(), user_id,
    ).await {
        Ok(opp) => Ok((StatusCode::CREATED, Json(serde_json::to_value(opp).unwrap_or_default()))),
        Err(e) => {
            error!("Failed to create opportunity: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 409 => StatusCode::CONFLICT, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

pub async fn get_opportunity(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.lead_opportunity_engine.get_opportunity(id).await {
        Ok(Some(o)) => Ok(Json(serde_json::to_value(o).unwrap_or_default())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListOpportunitiesQuery {
    pub status: Option<String>,
    pub owner_id: Option<Uuid>,
    pub stage_id: Option<Uuid>,
}

pub async fn list_opportunities(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<ListOpportunitiesQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.lead_opportunity_engine.list_opportunities(
        org_id, params.status.as_deref(), params.owner_id, params.stage_id,
    ).await {
        Ok(opps) => Ok(Json(serde_json::json!({"data": opps}))),
        Err(e) => {
            error!("Error: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateOpportunityStageRequest {
    pub stage_id: Option<Uuid>,
}

pub async fn update_opportunity_stage(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateOpportunityStageRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).ok();
    let user_name: Option<String> = None;
    match state.lead_opportunity_engine.update_opportunity_stage(
        id, payload.stage_id, user_id, user_name.as_deref(), None,
    ).await {
        Ok(o) => Ok(Json(serde_json::to_value(o).unwrap_or_default())),
        Err(e) => {
            error!("Error: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 404 => StatusCode::NOT_FOUND, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

pub async fn close_opportunity_won(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.lead_opportunity_engine.close_opportunity_won(id).await {
        Ok(o) => Ok(Json(serde_json::to_value(o).unwrap_or_default())),
        Err(e) => {
            error!("Error: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 404 => StatusCode::NOT_FOUND, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CloseOpportunityLostRequest {
    pub lost_reason: Option<String>,
}

pub async fn close_opportunity_lost(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<CloseOpportunityLostRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.lead_opportunity_engine.close_opportunity_lost(id, payload.lost_reason.as_deref()).await {
        Ok(o) => Ok(Json(serde_json::to_value(o).unwrap_or_default())),
        Err(e) => {
            error!("Error: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 404 => StatusCode::NOT_FOUND, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

pub async fn list_stage_history(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.lead_opportunity_engine.list_stage_history(id).await {
        Ok(history) => Ok(Json(serde_json::json!({"data": history}))),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn delete_opportunity(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.lead_opportunity_engine.get_opportunity(id).await {
        Ok(Some(opp)) => {
            match state.lead_opportunity_engine.delete_opportunity(org_id, &opp.opportunity_number).await {
                Ok(()) => Ok(StatusCode::NO_CONTENT),
                Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
            }
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// ============================================================================
// Opportunity Line Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct AddOpportunityLineRequest {
    pub product_name: String,
    pub product_code: Option<String>,
    pub description: Option<String>,
    #[serde(default = "default_one")]
    pub quantity: String,
    #[serde(default = "default_zero")]
    pub unit_price: String,
    #[serde(default = "default_zero")]
    pub discount_percent: String,
}

fn default_one() -> String { "1".to_string() }

pub async fn add_opportunity_line(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(opportunity_id): Path<Uuid>,
    Json(payload): Json<AddOpportunityLineRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.lead_opportunity_engine.add_opportunity_line(
        org_id, opportunity_id, &payload.product_name, payload.product_code.as_deref(),
        payload.description.as_deref(), &payload.quantity, &payload.unit_price, &payload.discount_percent,
    ).await {
        Ok(line) => Ok((StatusCode::CREATED, Json(serde_json::to_value(line).unwrap_or_default()))),
        Err(e) => {
            error!("Failed to add opportunity line: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 404 => StatusCode::NOT_FOUND, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

pub async fn list_opportunity_lines(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(opportunity_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.lead_opportunity_engine.list_opportunity_lines(opportunity_id).await {
        Ok(lines) => Ok(Json(serde_json::json!({"data": lines}))),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn delete_opportunity_line(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    match state.lead_opportunity_engine.delete_opportunity_line(id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// ============================================================================
// Sales Activity Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateActivityRequest {
    pub subject: String,
    pub description: Option<String>,
    pub activity_type: String,
    #[serde(default = "default_medium")]
    pub priority: String,
    pub lead_id: Option<Uuid>,
    pub opportunity_id: Option<Uuid>,
    pub contact_id: Option<Uuid>,
    pub contact_name: Option<String>,
    pub owner_id: Option<Uuid>,
    pub owner_name: Option<String>,
    pub start_at: Option<chrono::DateTime<chrono::Utc>>,
    pub end_at: Option<chrono::DateTime<chrono::Utc>>,
}

fn default_medium() -> String { "medium".to_string() }

pub async fn create_activity(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateActivityRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).ok();
    match state.lead_opportunity_engine.create_activity(
        org_id, &payload.subject, payload.description.as_deref(),
        &payload.activity_type, &payload.priority,
        payload.lead_id, payload.opportunity_id,
        payload.contact_id, payload.contact_name.as_deref(),
        payload.owner_id, payload.owner_name.as_deref(),
        payload.start_at, payload.end_at, user_id,
    ).await {
        Ok(act) => Ok((StatusCode::CREATED, Json(serde_json::to_value(act).unwrap_or_default()))),
        Err(e) => {
            error!("Failed to create activity: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListActivitiesQuery {
    pub lead_id: Option<Uuid>,
    pub opportunity_id: Option<Uuid>,
}

pub async fn list_activities(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<ListActivitiesQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.lead_opportunity_engine.list_activities(org_id, params.lead_id, params.opportunity_id).await {
        Ok(acts) => Ok(Json(serde_json::json!({"data": acts}))),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

#[derive(Debug, Deserialize)]
pub struct CompleteActivityRequest {
    pub outcome: Option<String>,
}

pub async fn complete_activity(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<CompleteActivityRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.lead_opportunity_engine.complete_activity(id, payload.outcome.as_deref()).await {
        Ok(a) => Ok(Json(serde_json::to_value(a).unwrap_or_default())),
        Err(e) => {
            error!("Error: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 404 => StatusCode::NOT_FOUND, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

pub async fn cancel_activity(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.lead_opportunity_engine.cancel_activity(id).await {
        Ok(a) => Ok(Json(serde_json::to_value(a).unwrap_or_default())),
        Err(e) => {
            error!("Error: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 404 => StatusCode::NOT_FOUND, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

pub async fn delete_activity(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    match state.lead_opportunity_engine.delete_activity(id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// ============================================================================
// Dashboard
// ============================================================================

pub async fn get_sales_pipeline_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.lead_opportunity_engine.get_dashboard(org_id).await {
        Ok(dashboard) => Ok(Json(serde_json::to_value(dashboard).unwrap_or_default())),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}
