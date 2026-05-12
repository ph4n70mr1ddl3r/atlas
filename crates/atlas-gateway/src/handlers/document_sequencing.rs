//! Document Sequencing Handlers
//!
//! Oracle Fusion Cloud ERP: General Ledger > Setup > Document Sequencing
//!
//! API endpoints for document sequence management:
//! - Sequence CRUD and lifecycle (create, list, get, activate, deactivate, delete)
//! - Sequence assignments (map sequences to document categories)
//! - Number generation (automatic, direct)
//! - Audit trail
//! - Dashboard summary

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
// Sequence Management
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateSequenceRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub sequence_type: Option<String>,
    pub document_type: Option<String>,
    pub initial_value: Option<i64>,
    pub increment_by: Option<i32>,
    pub max_value: Option<i64>,
    pub cycle_flag: Option<bool>,
    pub prefix: Option<String>,
    pub suffix: Option<String>,
    pub pad_length: Option<i32>,
    pub pad_character: Option<String>,
    pub reset_frequency: Option<String>,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
}

pub async fn create_sequence(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateSequenceRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.document_sequencing_engine.create_sequence(
        org_id,
        &payload.code,
        &payload.name,
        payload.description.as_deref(),
        payload.sequence_type.as_deref().unwrap_or("gap_permitted"),
        payload.document_type.as_deref().unwrap_or("custom"),
        payload.initial_value.unwrap_or(1),
        payload.increment_by.unwrap_or(1),
        payload.max_value,
        payload.cycle_flag.unwrap_or(false),
        payload.prefix.as_deref(),
        payload.suffix.as_deref(),
        payload.pad_length.unwrap_or(0),
        payload.pad_character.as_deref().unwrap_or("0"),
        payload.reset_frequency.as_deref(),
        payload.effective_from,
        payload.effective_to,
        Some(user_id),
    ).await {
        Ok(seq) => Ok((StatusCode::CREATED, Json(crate::handlers::records::to_json_or_null(seq)))),
        Err(e) => {
            error!("Failed to create document sequence: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(msg) => {
                    return Ok((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": msg}))));
                }
                atlas_shared::AtlasError::Conflict(msg) => {
                    return Ok((StatusCode::CONFLICT, Json(serde_json::json!({"error": msg}))));
                }
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListSequencesQuery {
    pub status: Option<String>,
    pub document_type: Option<String>,
}

pub async fn list_sequences(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListSequencesQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.document_sequencing_engine.list_sequences(
        org_id,
        query.status.as_deref(),
        query.document_type.as_deref(),
    ).await {
        Ok(sequences) => Ok(Json(serde_json::json!({ "data": sequences }))),
        Err(e) => {
            error!("Failed to list document sequences: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_sequence(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.document_sequencing_engine.get_sequence_by_id(id).await {
        Ok(Some(seq)) => Ok(Json(crate::handlers::records::to_json_or_null(seq))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get document sequence: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_sequence_by_code(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(code): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.document_sequencing_engine.get_sequence(org_id, &code).await {
        Ok(Some(seq)) => Ok(Json(crate::handlers::records::to_json_or_null(seq))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get document sequence by code: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn activate_sequence(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.document_sequencing_engine.activate_sequence(id).await {
        Ok(seq) => Ok(Json(crate::handlers::records::to_json_or_null(seq))),
        Err(e) => {
            error!("Failed to activate document sequence: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) | atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn deactivate_sequence(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.document_sequencing_engine.deactivate_sequence(id).await {
        Ok(seq) => Ok(Json(crate::handlers::records::to_json_or_null(seq))),
        Err(e) => {
            error!("Failed to deactivate document sequence: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) | atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn delete_sequence(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(code): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.document_sequencing_engine.delete_sequence(org_id, &code).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            error!("Failed to delete document sequence: {}", e);
            Err(match e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// ============================================================================
// Number Generation
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct GenerateNumberRequest {
    pub document_category: String,
    pub business_unit_id: Option<Uuid>,
    pub ledger_id: Option<Uuid>,
    pub document_id: Option<Uuid>,
    pub document_number: Option<String>,
}

pub async fn generate_number(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<GenerateNumberRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.document_sequencing_engine.generate_number(
        org_id,
        &payload.document_category,
        payload.business_unit_id,
        payload.ledger_id,
        payload.document_id,
        payload.document_number.as_deref(),
        Some(user_id),
    ).await {
        Ok(audit) => Ok((StatusCode::CREATED, Json(crate::handlers::records::to_json_or_null(audit)))),
        Err(e) => {
            error!("Failed to generate document number: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) | atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct GenerateNumberDirectRequest {
    pub sequence_code: String,
    pub document_category: String,
    pub document_id: Option<Uuid>,
    pub document_number: Option<String>,
    pub business_unit_id: Option<Uuid>,
}

pub async fn generate_number_direct(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<GenerateNumberDirectRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.document_sequencing_engine.generate_number_direct(
        org_id,
        &payload.sequence_code,
        &payload.document_category,
        payload.document_id,
        payload.document_number.as_deref(),
        payload.business_unit_id,
        Some(user_id),
    ).await {
        Ok(audit) => Ok((StatusCode::CREATED, Json(crate::handlers::records::to_json_or_null(audit)))),
        Err(e) => {
            error!("Failed to generate direct document number: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) | atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// ============================================================================
// Assignment Management
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateAssignmentRequest {
    pub sequence_code: String,
    pub document_category: String,
    pub business_unit_id: Option<Uuid>,
    pub ledger_id: Option<Uuid>,
    pub method: Option<String>,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub priority: Option<i32>,
}

pub async fn create_assignment(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateAssignmentRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.document_sequencing_engine.create_assignment(
        org_id,
        &payload.sequence_code,
        &payload.document_category,
        payload.business_unit_id,
        payload.ledger_id,
        payload.method.as_deref().unwrap_or("automatic"),
        payload.effective_from,
        payload.effective_to,
        payload.priority.unwrap_or(0),
        Some(user_id),
    ).await {
        Ok(assignment) => Ok((StatusCode::CREATED, Json(crate::handlers::records::to_json_or_null(assignment)))),
        Err(e) => {
            error!("Failed to create sequence assignment: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn get_assignment(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.document_sequencing_engine.get_assignment(id).await {
        Ok(Some(assignment)) => Ok(Json(crate::handlers::records::to_json_or_null(assignment))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get sequence assignment: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListAssignmentsQuery {
    pub sequence_id: Option<Uuid>,
}

pub async fn list_assignments(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListAssignmentsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.document_sequencing_engine.list_assignments(org_id, query.sequence_id).await {
        Ok(assignments) => Ok(Json(serde_json::json!({ "data": assignments }))),
        Err(e) => {
            error!("Failed to list sequence assignments: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn deactivate_assignment(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.document_sequencing_engine.deactivate_assignment(id).await {
        Ok(assignment) => Ok(Json(crate::handlers::records::to_json_or_null(assignment))),
        Err(e) => {
            error!("Failed to deactivate sequence assignment: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn delete_assignment(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    match state.document_sequencing_engine.delete_assignment(id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            error!("Failed to delete sequence assignment: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// ============================================================================
// Audit Trail
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ListAuditEntriesQuery {
    pub sequence_id: Option<Uuid>,
    pub limit: Option<i32>,
}

pub async fn list_audit_entries(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListAuditEntriesQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.document_sequencing_engine.list_audit_entries(
        org_id,
        query.sequence_id,
        query.limit,
    ).await {
        Ok(entries) => Ok(Json(serde_json::json!({ "data": entries }))),
        Err(e) => {
            error!("Failed to list audit entries: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_audit_by_document(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(document_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.document_sequencing_engine.get_audit_by_document(document_id).await {
        Ok(Some(entry)) => Ok(Json(crate::handlers::records::to_json_or_null(entry))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get audit by document: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// ============================================================================
// Dashboard
// ============================================================================

pub async fn get_document_sequencing_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.document_sequencing_engine.get_dashboard_summary(org_id).await {
        Ok(summary) => Ok(Json(crate::handlers::records::to_json_or_null(summary))),
        Err(e) => {
            error!("Failed to get document sequencing dashboard: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
