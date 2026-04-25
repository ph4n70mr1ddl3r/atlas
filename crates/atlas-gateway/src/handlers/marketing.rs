//! Marketing Campaign Management Handlers
//!
//! Oracle Fusion Cloud: CX Marketing > Campaigns
//!
//! API endpoints for managing marketing campaigns, campaign types,
//! campaign members, responses, and ROI analytics.

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
// Campaign Type Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateCampaignTypeRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(default = "default_email")]
    pub channel: String,
}

fn default_email() -> String { "email".to_string() }
fn default_zero() -> String { "0".to_string() }
fn default_usd() -> String { "USD".to_string() }

pub async fn create_campaign_type(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateCampaignTypeRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).ok();
    match state.marketing_engine.create_campaign_type(
        org_id, &payload.code, &payload.name, payload.description.as_deref(),
        &payload.channel, user_id,
    ).await {
        Ok(ct) => Ok((StatusCode::CREATED, Json(serde_json::to_value(ct).unwrap_or_default()))),
        Err(e) => {
            error!("Failed to create campaign type: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 409 => StatusCode::CONFLICT, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

pub async fn list_campaign_types(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.marketing_engine.list_campaign_types(org_id).await {
        Ok(types) => Ok(Json(serde_json::json!({"data": types}))),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn delete_campaign_type(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(code): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.marketing_engine.delete_campaign_type(org_id, &code).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// ============================================================================
// Campaign Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateCampaignRequest {
    pub campaign_number: String,
    pub name: String,
    pub description: Option<String>,
    pub campaign_type_id: Option<Uuid>,
    pub campaign_type_name: Option<String>,
    #[serde(default = "default_email")]
    pub channel: String,
    #[serde(default = "default_zero")]
    pub budget: String,
    #[serde(default = "default_usd")]
    pub currency_code: String,
    pub start_date: Option<chrono::NaiveDate>,
    pub end_date: Option<chrono::NaiveDate>,
    pub owner_id: Option<Uuid>,
    pub owner_name: Option<String>,
    #[serde(default)]
    pub expected_responses: i32,
    #[serde(default = "default_zero")]
    pub expected_revenue: String,
    pub parent_campaign_id: Option<Uuid>,
    pub parent_campaign_name: Option<String>,
    #[serde(default = "default_empty_array")]
    pub tags: serde_json::Value,
    pub notes: Option<String>,
}

fn default_empty_array() -> serde_json::Value { serde_json::json!([]) }

pub async fn create_campaign(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateCampaignRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).ok();
    match state.marketing_engine.create_campaign(
        org_id, &payload.campaign_number, &payload.name, payload.description.as_deref(),
        payload.campaign_type_id, payload.campaign_type_name.as_deref(),
        &payload.channel, &payload.budget, &payload.currency_code,
        payload.start_date, payload.end_date,
        payload.owner_id, payload.owner_name.as_deref(),
        payload.expected_responses, &payload.expected_revenue,
        payload.parent_campaign_id, payload.parent_campaign_name.as_deref(),
        payload.tags, payload.notes.as_deref(), user_id,
    ).await {
        Ok(campaign) => Ok((StatusCode::CREATED, Json(serde_json::to_value(campaign).unwrap_or_default()))),
        Err(e) => {
            error!("Failed to create campaign: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 409 => StatusCode::CONFLICT, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

pub async fn get_campaign(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.marketing_engine.get_campaign(id).await {
        Ok(Some(c)) => Ok(Json(serde_json::to_value(c).unwrap_or_default())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListCampaignsQuery {
    pub status: Option<String>,
    pub channel: Option<String>,
    pub owner_id: Option<Uuid>,
}

pub async fn list_campaigns(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<ListCampaignsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.marketing_engine.list_campaigns(
        org_id, params.status.as_deref(), params.channel.as_deref(), params.owner_id,
    ).await {
        Ok(campaigns) => Ok(Json(serde_json::json!({"data": campaigns}))),
        Err(e) => {
            error!("Error: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

pub async fn activate_campaign(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.marketing_engine.activate_campaign(id).await {
        Ok(c) => Ok(Json(serde_json::to_value(c).unwrap_or_default())),
        Err(e) => {
            error!("Error: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 404 => StatusCode::NOT_FOUND, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

pub async fn pause_campaign(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.marketing_engine.pause_campaign(id).await {
        Ok(c) => Ok(Json(serde_json::to_value(c).unwrap_or_default())),
        Err(e) => {
            error!("Error: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 404 => StatusCode::NOT_FOUND, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

pub async fn complete_campaign(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.marketing_engine.complete_campaign(id).await {
        Ok(c) => Ok(Json(serde_json::to_value(c).unwrap_or_default())),
        Err(e) => {
            error!("Error: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 404 => StatusCode::NOT_FOUND, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

pub async fn cancel_campaign(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.marketing_engine.cancel_campaign(id).await {
        Ok(c) => Ok(Json(serde_json::to_value(c).unwrap_or_default())),
        Err(e) => {
            error!("Error: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 404 => StatusCode::NOT_FOUND, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

pub async fn delete_campaign(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.marketing_engine.get_campaign(id).await {
        Ok(Some(campaign)) => {
            match state.marketing_engine.delete_campaign(org_id, &campaign.campaign_number).await {
                Ok(()) => Ok(StatusCode::NO_CONTENT),
                Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
            }
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// ============================================================================
// Campaign Member Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct AddCampaignMemberRequest {
    pub contact_id: Option<Uuid>,
    pub contact_name: Option<String>,
    pub contact_email: Option<String>,
    pub lead_id: Option<Uuid>,
    pub lead_number: Option<String>,
}

pub async fn add_campaign_member(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(campaign_id): Path<Uuid>,
    Json(payload): Json<AddCampaignMemberRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).ok();
    match state.marketing_engine.add_campaign_member(
        org_id, campaign_id,
        payload.contact_id, payload.contact_name.as_deref(), payload.contact_email.as_deref(),
        payload.lead_id, payload.lead_number.as_deref(), user_id,
    ).await {
        Ok(member) => Ok((StatusCode::CREATED, Json(serde_json::to_value(member).unwrap_or_default()))),
        Err(e) => {
            error!("Failed to add campaign member: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 404 => StatusCode::NOT_FOUND, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListCampaignMembersQuery {
    pub status: Option<String>,
}

pub async fn list_campaign_members(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(campaign_id): Path<Uuid>,
    Query(params): Query<ListCampaignMembersQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.marketing_engine.list_campaign_members(campaign_id, params.status.as_deref()).await {
        Ok(members) => Ok(Json(serde_json::json!({"data": members}))),
        Err(e) => {
            error!("Error: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateMemberStatusRequest {
    pub status: String,
    pub response: Option<String>,
}

pub async fn update_member_status(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateMemberStatusRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.marketing_engine.update_member_status(id, &payload.status, payload.response.as_deref()).await {
        Ok(m) => Ok(Json(serde_json::to_value(m).unwrap_or_default())),
        Err(e) => {
            error!("Error: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 404 => StatusCode::NOT_FOUND, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

pub async fn delete_campaign_member(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    match state.marketing_engine.delete_campaign_member(id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// ============================================================================
// Campaign Response Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateCampaignResponseRequest {
    pub member_id: Option<Uuid>,
    pub response_type: String,
    pub contact_id: Option<Uuid>,
    pub contact_name: Option<String>,
    pub contact_email: Option<String>,
    pub lead_id: Option<Uuid>,
    pub description: Option<String>,
    #[serde(default = "default_zero")]
    pub value: String,
    #[serde(default = "default_usd")]
    pub currency_code: String,
    pub source_url: Option<String>,
}

pub async fn create_campaign_response(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(campaign_id): Path<Uuid>,
    Json(payload): Json<CreateCampaignResponseRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).ok();
    match state.marketing_engine.create_response(
        org_id, campaign_id, payload.member_id, &payload.response_type,
        payload.contact_id, payload.contact_name.as_deref(), payload.contact_email.as_deref(),
        payload.lead_id, payload.description.as_deref(),
        &payload.value, &payload.currency_code, payload.source_url.as_deref(), user_id,
    ).await {
        Ok(resp) => Ok((StatusCode::CREATED, Json(serde_json::to_value(resp).unwrap_or_default()))),
        Err(e) => {
            error!("Failed to create campaign response: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 404 => StatusCode::NOT_FOUND, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListCampaignResponsesQuery {
    pub response_type: Option<String>,
}

pub async fn list_campaign_responses(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(campaign_id): Path<Uuid>,
    Query(params): Query<ListCampaignResponsesQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.marketing_engine.list_responses(campaign_id, params.response_type.as_deref()).await {
        Ok(responses) => Ok(Json(serde_json::json!({"data": responses}))),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn delete_campaign_response(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    match state.marketing_engine.delete_response(id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// ============================================================================
// Dashboard
// ============================================================================

pub async fn get_marketing_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.marketing_engine.get_dashboard(org_id).await {
        Ok(dashboard) => Ok(Json(serde_json::to_value(dashboard).unwrap_or_default())),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}
