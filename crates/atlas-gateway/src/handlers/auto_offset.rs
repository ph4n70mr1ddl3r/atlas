//! Automatic Offsets (Intercompany Balancing) Handlers
//!
//! Oracle Fusion Cloud ERP: General Ledger > Automatic Offsets
//!
//! API endpoints for managing automatic intercompany offset entries:
//! - Offset templates (CRUD, activate/deactivate)
//! - Template lines (add/remove per-segment account mappings)
//! - Offset generation (generate, post, reverse, cancel)
//! - Offset lines (list generated entries)
//! - Activities (audit trail)
//! - Dashboard

use crate::handlers::{to_json, created_json};
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
// Template CRUD
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateOffsetTemplateRequest {
    pub template_code: String,
    pub template_name: String,
    pub description: Option<String>,
    pub balancing_segment: String,
    pub intercompany_segment: Option<String>,
    pub generation_method: String,
    pub default_offset_account: String,
    pub default_offset_account_description: Option<String>,
    pub enable_intra_entity: Option<bool>,
    pub intra_entity_account: Option<String>,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
}

pub async fn create_offset_template(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateOffsetTemplateRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.financials.auto_offset_engine.create_template(
        org_id, &payload.template_code, &payload.template_name,
        payload.description.as_deref(), &payload.balancing_segment,
        payload.intercompany_segment.as_deref(), &payload.generation_method,
        &payload.default_offset_account, payload.default_offset_account_description.as_deref(),
        payload.enable_intra_entity.unwrap_or(false),
        payload.intra_entity_account.as_deref(),
        payload.effective_from, payload.effective_to,
        Some(user_id),
    ).await {
        Ok(tmpl) => Ok(created_json(tmpl)),
        Err(e) => {
            error!("Failed to create offset template: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListTemplatesQuery {
    pub is_active: Option<bool>,
}

pub async fn list_offset_templates(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListTemplatesQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.financials.auto_offset_engine.list_templates(org_id, query.is_active).await {
        Ok(templates) => Ok(Json(serde_json::json!({ "data": templates }))),
        Err(e) => {
            error!("Failed to list offset templates: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_offset_template(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let _org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.financials.auto_offset_engine.get_template(id).await {
        Ok(Some(tmpl)) => Ok(to_json(tmpl)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get offset template: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn activate_offset_template(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let _org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.financials.auto_offset_engine.activate_template(id).await {
        Ok(tmpl) => Ok(to_json(tmpl)),
        Err(e) => {
            error!("Failed to activate offset template: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn deactivate_offset_template(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let _org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.financials.auto_offset_engine.deactivate_template(id).await {
        Ok(tmpl) => Ok(to_json(tmpl)),
        Err(e) => {
            error!("Failed to deactivate offset template: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn delete_offset_template(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    claims: Extension<Claims>,
) -> Result<StatusCode, StatusCode> {
    let _org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.financials.auto_offset_engine.delete_template(id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            error!("Failed to delete offset template: {}", e);
            Err(match e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// ============================================================================
// Template Lines
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct AddTemplateLineRequest {
    pub balancing_segment_value: String,
    pub due_to_account: String,
    pub due_to_account_description: Option<String>,
    pub due_from_account: String,
    pub due_from_account_description: Option<String>,
    pub clearing_account: Option<String>,
    pub priority: Option<i32>,
}

pub async fn add_offset_template_line(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(template_id): Path<Uuid>,
    Json(payload): Json<AddTemplateLineRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Get current line count for line_number
    let existing_lines = state.financials.auto_offset_engine.list_template_lines(template_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let line_number = (existing_lines.len() + 1) as i32;

    match state.financials.auto_offset_engine.add_template_line(
        org_id, template_id, line_number,
        &payload.balancing_segment_value,
        &payload.due_to_account, payload.due_to_account_description.as_deref(),
        &payload.due_from_account, payload.due_from_account_description.as_deref(),
        payload.clearing_account.as_deref(),
        payload.priority.unwrap_or(100),
    ).await {
        Ok(line) => Ok(created_json(line)),
        Err(e) => {
            error!("Failed to add template line: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn list_offset_template_lines(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(template_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let _org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.financials.auto_offset_engine.list_template_lines(template_id).await {
        Ok(lines) => Ok(Json(serde_json::json!({ "data": lines }))),
        Err(e) => {
            error!("Failed to list template lines: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn delete_offset_template_line(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(line_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let _org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.financials.auto_offset_engine.delete_template_line(line_id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            error!("Failed to delete template line: {}", e);
            Err(match e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// ============================================================================
// Offset Generation
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct GenerateOffsetsRequest {
    pub template_id: Uuid,
    pub source_type: String,
    pub source_id: Option<Uuid>,
    pub source_number: Option<String>,
    pub fiscal_year: i32,
    pub period_name: String,
    pub generation_date: chrono::NaiveDate,
    pub currency_code: Option<String>,
    pub journal_lines: Vec<JournalLineInput>,
}

#[derive(Debug, Deserialize)]
pub struct JournalLineInput {
    pub balancing_segment_value: String,
    pub account_code: String,
    pub debit: f64,
    pub credit: f64,
}

pub async fn generate_offsets(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<GenerateOffsetsRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let journal_lines: Vec<atlas_core::auto_offset::JournalLine> = payload.journal_lines
        .iter()
        .map(|l| atlas_core::auto_offset::JournalLine {
            balancing_segment_value: l.balancing_segment_value.clone(),
            account_code: l.account_code.clone(),
            debit: l.debit,
            credit: l.credit,
        })
        .collect();

    match state.financials.auto_offset_engine.generate_offsets(
        org_id, payload.template_id,
        &payload.source_type, payload.source_id, payload.source_number.as_deref(),
        payload.fiscal_year, &payload.period_name,
        payload.generation_date,
        payload.currency_code.as_deref().unwrap_or("USD"),
        &journal_lines,
        Some(user_id),
    ).await {
        Ok(gen) => Ok(created_json(gen)),
        Err(e) => {
            error!("Failed to generate offsets: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn get_offset_generation(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let _org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.financials.auto_offset_engine.get_generation(id).await {
        Ok(Some(gen)) => Ok(to_json(gen)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get offset generation: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListGenerationsQuery {
    pub status: Option<String>,
    pub source_type: Option<String>,
}

pub async fn list_offset_generations(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListGenerationsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.financials.auto_offset_engine.list_generations(org_id, query.status.as_deref(), query.source_type.as_deref()).await {
        Ok(generations) => Ok(Json(serde_json::json!({ "data": generations }))),
        Err(e) => {
            error!("Failed to list offset generations: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn post_offset_generation(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.financials.auto_offset_engine.post_generation(id, user_id).await {
        Ok(gen) => Ok(to_json(gen)),
        Err(e) => {
            error!("Failed to post offset generation: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn reverse_offset_generation(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.financials.auto_offset_engine.reverse_generation(id, user_id).await {
        Ok(gen) => Ok(to_json(gen)),
        Err(e) => {
            error!("Failed to reverse offset generation: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn cancel_offset_generation(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.financials.auto_offset_engine.cancel_generation(id, user_id).await {
        Ok(gen) => Ok(to_json(gen)),
        Err(e) => {
            error!("Failed to cancel offset generation: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// ============================================================================
// Offset Lines & Activities
// ============================================================================

pub async fn list_offset_lines(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(generation_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let _org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.financials.auto_offset_engine.list_offset_lines(generation_id).await {
        Ok(lines) => Ok(Json(serde_json::json!({ "data": lines }))),
        Err(e) => {
            error!("Failed to list offset lines: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn list_offset_activities(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(generation_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let _org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.financials.auto_offset_engine.list_activities(generation_id).await {
        Ok(activities) => Ok(Json(serde_json::json!({ "data": activities }))),
        Err(e) => {
            error!("Failed to list offset activities: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// ============================================================================
// Dashboard
// ============================================================================

pub async fn get_auto_offset_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.financials.auto_offset_engine.get_dashboard(org_id).await {
        Ok(dashboard) => Ok(to_json(dashboard)),
        Err(e) => {
            error!("Failed to get auto offset dashboard: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
