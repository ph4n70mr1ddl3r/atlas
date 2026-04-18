//! Procurement Sourcing Handlers
//!
//! Oracle Fusion Cloud ERP: Procurement > Sourcing > Negotiations
//!
//! API endpoints for managing sourcing events (RFQ/RFP/RFI),
//! supplier responses, scoring & evaluation, and awards.

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
use tracing::{info, error};

// ============================================================================
// Request Types
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateSourcingEventRequest {
    pub title: String,
    pub description: Option<String>,
    #[serde(default = "default_rfq")]
    pub event_type: String,
    #[serde(default = "default_sealed")]
    pub style: String,
    pub response_deadline: chrono::NaiveDate,
    #[serde(default = "default_usd")]
    pub currency_code: String,
    #[serde(default = "default_weighted")]
    pub scoring_method: String,
    pub template_code: Option<String>,
    pub evaluation_lead_id: Option<Uuid>,
    pub evaluation_lead_name: Option<String>,
    pub contact_person_id: Option<Uuid>,
    pub contact_person_name: Option<String>,
    #[serde(default)]
    pub are_bids_visible: bool,
    #[serde(default)]
    pub allow_supplier_rank_visibility: bool,
    pub terms_and_conditions: Option<String>,
}

fn default_rfq() -> String { "rfq".to_string() }
fn default_sealed() -> String { "sealed".to_string() }
fn default_weighted() -> String { "weighted".to_string() }
fn default_usd() -> String { "USD".to_string() }

#[derive(Debug, Deserialize)]
pub struct AddEventLineRequest {
    pub description: String,
    pub item_number: Option<String>,
    pub category: Option<String>,
    pub quantity: String,
    #[serde(default = "default_ea")]
    pub uom: String,
    pub target_price: Option<String>,
    pub target_total: Option<String>,
    pub need_by_date: Option<chrono::NaiveDate>,
    pub ship_to: Option<String>,
    pub specifications: Option<serde_json::Value>,
    #[serde(default)]
    pub allow_partial_quantity: bool,
    pub min_award_quantity: Option<String>,
}

fn default_ea() -> String { "EA".to_string() }

#[derive(Debug, Deserialize)]
pub struct InviteSupplierRequest {
    pub supplier_id: Uuid,
    pub supplier_name: Option<String>,
    pub supplier_email: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SubmitResponseRequest {
    pub supplier_id: Uuid,
    pub supplier_name: Option<String>,
    pub cover_letter: Option<String>,
    pub valid_until: Option<chrono::NaiveDate>,
    pub payment_terms: Option<String>,
    pub lead_time_days: Option<i32>,
    pub warranty_months: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct AddResponseLineRequest {
    pub event_line_id: Uuid,
    pub unit_price: String,
    pub quantity: String,
    pub discount_percent: Option<String>,
    pub promised_delivery_date: Option<chrono::NaiveDate>,
    pub lead_time_days: Option<i32>,
    pub supplier_notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddScoringCriterionRequest {
    pub name: String,
    pub description: Option<String>,
    pub weight: String,
    pub max_score: String,
    #[serde(default = "default_custom")]
    pub criterion_type: String,
    #[serde(default = "default_10_order")]
    pub display_order: i32,
    #[serde(default)]
    pub is_mandatory: bool,
}

fn default_custom() -> String { "custom".to_string() }
fn default_10_order() -> i32 { 10 }

#[derive(Debug, Deserialize)]
pub struct ScoreResponseRequest {
    pub criterion_id: Uuid,
    pub score: String,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAwardRequest {
    #[serde(default = "default_single")]
    pub award_method: String,
    pub award_rationale: Option<String>,
    pub lines: Vec<CreateAwardLineRequest>,
}

fn default_single() -> String { "single".to_string() }

#[derive(Debug, Deserialize)]
pub struct CreateAwardLineRequest {
    pub event_line_id: Uuid,
    pub response_id: Uuid,
    pub supplier_id: Uuid,
    pub supplier_name: Option<String>,
    pub awarded_quantity: String,
    pub awarded_unit_price: String,
    pub awarded_amount: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateTemplateRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(default = "default_rfq")]
    pub default_event_type: String,
    #[serde(default = "default_sealed")]
    pub default_style: String,
    #[serde(default = "default_weighted")]
    pub default_scoring_method: String,
    #[serde(default = "default_deadline_days")]
    pub default_response_deadline_days: i32,
    #[serde(default = "default_usd")]
    pub currency_code: String,
    #[serde(default)]
    pub default_bids_visible: bool,
    pub default_terms: Option<String>,
    #[serde(default = "default_empty_array")]
    pub default_scoring_criteria: serde_json::Value,
    #[serde(default = "default_empty_array")]
    pub default_lines: serde_json::Value,
}

fn default_deadline_days() -> i32 { 14 }
fn default_empty_array() -> serde_json::Value { serde_json::json!([]) }

#[derive(Debug, Deserialize)]
pub struct ListEventsQuery {
    pub status: Option<String>,
    pub event_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListResponsesQuery {
    pub status: Option<String>,
}

// ============================================================================
// Sourcing Event Handlers
// ============================================================================

/// Create a new sourcing event
pub async fn create_sourcing_event(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateSourcingEventRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Resolve template code to ID if provided
    let template_id = if let Some(code) = &payload.template_code {
        state.sourcing_engine.get_template(org_id, code).await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .map(|t| t.id)
    } else {
        None
    };

    match state.sourcing_engine.create_event(
        org_id, &payload.title, payload.description.as_deref(),
        &payload.event_type, &payload.style, payload.response_deadline,
        &payload.currency_code, &payload.scoring_method,
        template_id, payload.evaluation_lead_id,
        payload.evaluation_lead_name.as_deref(),
        payload.contact_person_id, payload.contact_person_name.as_deref(),
        payload.are_bids_visible, payload.allow_supplier_rank_visibility,
        payload.terms_and_conditions.as_deref(), Some(user_id),
    ).await {
        Ok(event) => Ok((StatusCode::CREATED, Json(serde_json::to_value(event).unwrap()))),
        Err(e) => {
            error!("Failed to create sourcing event: {}", e);
            Err(map_error(e))
        }
    }
}

/// Get a sourcing event by ID
pub async fn get_sourcing_event(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.sourcing_engine.get_event(id).await {
        Ok(Some(event)) => Ok(Json(serde_json::to_value(event).unwrap())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

/// List sourcing events
pub async fn list_sourcing_events(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListEventsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.sourcing_engine.list_events(org_id, query.status.as_deref(), query.event_type.as_deref()).await {
        Ok(events) => Ok(Json(serde_json::to_value(events).unwrap())),
        Err(e) => Err(map_error(e))
    }
}

/// Publish a sourcing event
pub async fn publish_sourcing_event(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.sourcing_engine.publish_event(id, Some(user_id)).await {
        Ok(event) => Ok(Json(serde_json::to_value(event).unwrap())),
        Err(e) => Err(map_error(e))
    }
}

/// Close a sourcing event for responses
pub async fn close_sourcing_event(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.sourcing_engine.close_event(id, Some(user_id)).await {
        Ok(event) => Ok(Json(serde_json::to_value(event).unwrap())),
        Err(e) => Err(map_error(e))
    }
}

/// Cancel a sourcing event
pub async fn cancel_sourcing_event(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<CancelEventRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.sourcing_engine.cancel_event(id, Some(user_id), payload.reason.as_deref()).await {
        Ok(event) => Ok(Json(serde_json::to_value(event).unwrap())),
        Err(e) => Err(map_error(e))
    }
}

#[derive(Debug, Deserialize)]
pub struct CancelEventRequest {
    pub reason: Option<String>,
}

// ============================================================================
// Event Lines
// ============================================================================

/// Add a line to a sourcing event
pub async fn add_event_line(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(event_id): Path<Uuid>,
    Json(payload): Json<AddEventLineRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.sourcing_engine.add_event_line(
        org_id, event_id, &payload.description, payload.item_number.as_deref(),
        payload.category.as_deref(), &payload.quantity, &payload.uom,
        payload.target_price.as_deref(), payload.target_total.as_deref(),
        payload.need_by_date, payload.ship_to.as_deref(),
        payload.specifications, payload.allow_partial_quantity,
        payload.min_award_quantity.as_deref(),
    ).await {
        Ok(line) => Ok((StatusCode::CREATED, Json(serde_json::to_value(line).unwrap()))),
        Err(e) => {
            error!("Failed to add event line: {}", e);
            Err(map_error(e))
        }
    }
}

/// List lines for a sourcing event
pub async fn list_event_lines(
    State(state): State<Arc<AppState>>,
    Path(event_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.sourcing_engine.list_event_lines(event_id).await {
        Ok(lines) => Ok(Json(serde_json::to_value(lines).unwrap())),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// ============================================================================
// Supplier Invitations
// ============================================================================

/// Invite a supplier to an event
pub async fn invite_supplier(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(event_id): Path<Uuid>,
    Json(payload): Json<InviteSupplierRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.sourcing_engine.invite_supplier(
        org_id, event_id, payload.supplier_id,
        payload.supplier_name.as_deref(), payload.supplier_email.as_deref(),
    ).await {
        Ok(invite) => Ok((StatusCode::CREATED, Json(serde_json::to_value(invite).unwrap()))),
        Err(e) => {
            error!("Failed to invite supplier: {}", e);
            Err(map_error(e))
        }
    }
}

/// List invites for an event
pub async fn list_invites(
    State(state): State<Arc<AppState>>,
    Path(event_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.sourcing_engine.list_invites(event_id).await {
        Ok(invites) => Ok(Json(serde_json::to_value(invites).unwrap())),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// ============================================================================
// Supplier Responses
// ============================================================================

/// Submit a supplier response
pub async fn submit_response(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(event_id): Path<Uuid>,
    Json(payload): Json<SubmitResponseRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.sourcing_engine.submit_response(
        org_id, event_id, payload.supplier_id, payload.supplier_name.as_deref(),
        payload.cover_letter.as_deref(), payload.valid_until,
        payload.payment_terms.as_deref(), payload.lead_time_days,
        payload.warranty_months, Some(user_id),
    ).await {
        Ok(response) => Ok((StatusCode::CREATED, Json(serde_json::to_value(response).unwrap()))),
        Err(e) => {
            error!("Failed to submit response: {}", e);
            Err(map_error(e))
        }
    }
}

/// List responses for an event
pub async fn list_responses(
    State(state): State<Arc<AppState>>,
    Path(event_id): Path<Uuid>,
    Query(query): Query<ListResponsesQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.sourcing_engine.list_responses(event_id, query.status.as_deref()).await {
        Ok(responses) => Ok(Json(serde_json::to_value(responses).unwrap())),
        Err(e) => Err(map_error(e))
    }
}

/// Get a response by ID
pub async fn get_response(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.sourcing_engine.get_response(id).await {
        Ok(Some(response)) => Ok(Json(serde_json::to_value(response).unwrap())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

/// Add a line to a response
pub async fn add_response_line(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(response_id): Path<Uuid>,
    Json(payload): Json<AddResponseLineRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.sourcing_engine.add_response_line(
        org_id, response_id, payload.event_line_id,
        &payload.unit_price, &payload.quantity, payload.discount_percent.as_deref(),
        payload.promised_delivery_date, payload.lead_time_days,
        payload.supplier_notes.as_deref(),
    ).await {
        Ok(line) => Ok((StatusCode::CREATED, Json(serde_json::to_value(line).unwrap()))),
        Err(e) => {
            error!("Failed to add response line: {}", e);
            Err(map_error(e))
        }
    }
}

/// List lines for a response
pub async fn list_response_lines(
    State(state): State<Arc<AppState>>,
    Path(response_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.sourcing_engine.list_response_lines(response_id).await {
        Ok(lines) => Ok(Json(serde_json::to_value(lines).unwrap())),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// ============================================================================
// Scoring & Evaluation
// ============================================================================

/// Add a scoring criterion to an event
pub async fn add_scoring_criterion(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(event_id): Path<Uuid>,
    Json(payload): Json<AddScoringCriterionRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.sourcing_engine.add_scoring_criterion(
        org_id, event_id, &payload.name, payload.description.as_deref(),
        &payload.weight, &payload.max_score, &payload.criterion_type,
        payload.display_order, payload.is_mandatory, Some(user_id),
    ).await {
        Ok(criterion) => Ok((StatusCode::CREATED, Json(serde_json::to_value(criterion).unwrap()))),
        Err(e) => {
            error!("Failed to add scoring criterion: {}", e);
            Err(map_error(e))
        }
    }
}

/// List scoring criteria for an event
pub async fn list_scoring_criteria(
    State(state): State<Arc<AppState>>,
    Path(event_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.sourcing_engine.list_scoring_criteria(event_id).await {
        Ok(criteria) => Ok(Json(serde_json::to_value(criteria).unwrap())),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

/// Score a response
pub async fn score_response(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(response_id): Path<Uuid>,
    Json(payload): Json<ScoreResponseRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.sourcing_engine.score_response(
        org_id, response_id, payload.criterion_id, &payload.score,
        payload.notes.as_deref(), Some(user_id),
    ).await {
        Ok(score) => Ok(Json(serde_json::to_value(score).unwrap())),
        Err(e) => Err(map_error(e))
    }
}

/// Evaluate all responses for an event
pub async fn evaluate_responses(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(event_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.sourcing_engine.evaluate_responses(event_id, Some(user_id)).await {
        Ok(responses) => Ok(Json(serde_json::to_value(responses).unwrap())),
        Err(e) => Err(map_error(e))
    }
}

// ============================================================================
// Award Management
// ============================================================================

/// Create an award for an event
pub async fn create_award(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(event_id): Path<Uuid>,
    Json(payload): Json<CreateAwardRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let award_lines: Vec<atlas_core::sourcing::engine::SourcingAwardLineRequest> = payload.lines.into_iter().map(|l| {
        atlas_core::sourcing::engine::SourcingAwardLineRequest {
            event_line_id: l.event_line_id,
            response_id: l.response_id,
            supplier_id: l.supplier_id,
            supplier_name: l.supplier_name,
            awarded_quantity: l.awarded_quantity,
            awarded_unit_price: l.awarded_unit_price,
            awarded_amount: l.awarded_amount,
        }
    }).collect();

    match state.sourcing_engine.create_award(
        org_id, event_id, &payload.award_method, &award_lines,
        payload.award_rationale.as_deref(), Some(user_id),
    ).await {
        Ok(award) => Ok((StatusCode::CREATED, Json(serde_json::to_value(award).unwrap()))),
        Err(e) => {
            error!("Failed to create award: {}", e);
            Err(map_error(e))
        }
    }
}

/// Get an award by ID
pub async fn get_award(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.sourcing_engine.get_award(id).await {
        Ok(Some(award)) => Ok(Json(serde_json::to_value(award).unwrap())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

/// List awards for an event
pub async fn list_awards(
    State(state): State<Arc<AppState>>,
    Path(event_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.sourcing_engine.list_awards(event_id).await {
        Ok(awards) => Ok(Json(serde_json::to_value(awards).unwrap())),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

/// Approve an award
pub async fn approve_award(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.sourcing_engine.approve_award(id, Some(user_id)).await {
        Ok(award) => Ok(Json(serde_json::to_value(award).unwrap())),
        Err(e) => Err(map_error(e))
    }
}

/// Reject an award
pub async fn reject_award(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<RejectAwardRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.sourcing_engine.reject_award(id, payload.reason.as_deref()).await {
        Ok(award) => Ok(Json(serde_json::to_value(award).unwrap())),
        Err(e) => Err(map_error(e))
    }
}

#[derive(Debug, Deserialize)]
pub struct RejectAwardRequest {
    pub reason: Option<String>,
}

/// List award lines
pub async fn list_award_lines(
    State(state): State<Arc<AppState>>,
    Path(award_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.sourcing_engine.list_award_lines(award_id).await {
        Ok(lines) => Ok(Json(serde_json::to_value(lines).unwrap())),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// ============================================================================
// Templates
// ============================================================================

/// Create a sourcing template
pub async fn create_sourcing_template(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateTemplateRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.sourcing_engine.create_template(
        org_id, &payload.code, &payload.name, payload.description.as_deref(),
        &payload.default_event_type, &payload.default_style, &payload.default_scoring_method,
        payload.default_response_deadline_days, &payload.currency_code,
        payload.default_bids_visible, payload.default_terms.as_deref(),
        payload.default_scoring_criteria, payload.default_lines, Some(user_id),
    ).await {
        Ok(template) => Ok((StatusCode::CREATED, Json(serde_json::to_value(template).unwrap()))),
        Err(e) => {
            error!("Failed to create sourcing template: {}", e);
            Err(map_error(e))
        }
    }
}

/// List templates
pub async fn list_sourcing_templates(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.sourcing_engine.list_templates(org_id).await {
        Ok(templates) => Ok(Json(serde_json::to_value(templates).unwrap())),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

/// Get a template by code
pub async fn get_sourcing_template(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(code): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.sourcing_engine.get_template(org_id, &code).await {
        Ok(Some(template)) => Ok(Json(serde_json::to_value(template).unwrap())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

/// Delete a template
pub async fn delete_sourcing_template(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(code): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.sourcing_engine.delete_template(org_id, &code).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err(map_error(e))
    }
}

// ============================================================================
// Dashboard
// ============================================================================

/// Get sourcing dashboard summary
pub async fn get_sourcing_summary(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.sourcing_engine.get_summary(org_id).await {
        Ok(summary) => Ok(Json(serde_json::to_value(summary).unwrap())),
        Err(e) => Err(map_error(e))
    }
}

// ============================================================================
// Error Mapping
// ============================================================================

fn map_error(e: atlas_shared::AtlasError) -> StatusCode {
    match e {
        atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
        atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
        atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
        atlas_shared::AtlasError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
        atlas_shared::AtlasError::Forbidden(_) => StatusCode::FORBIDDEN,
        atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
