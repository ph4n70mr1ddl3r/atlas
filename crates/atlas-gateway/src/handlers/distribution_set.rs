//! Distribution Set Handlers
//!
//! Oracle Fusion: Financials > Payables > Distribution Sets
//! Reusable GL account distribution templates for AP invoices.

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
// Create Distribution Set
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateDistributionSetRequest {
    pub set_code: String,
    pub set_name: String,
    pub description: Option<String>,
    pub distribution_type: String,
    pub currency_code: Option<String>,
    pub is_default: Option<bool>,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
}

pub async fn create_distribution_set(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateDistributionSetRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.distribution_set_engine.create_set(
        org_id,
        &payload.set_code,
        &payload.set_name,
        payload.description.as_deref(),
        &payload.distribution_type,
        payload.currency_code.as_deref().unwrap_or("USD"),
        payload.is_default.unwrap_or(false),
        payload.effective_from,
        payload.effective_to,
        Some(user_id),
    ).await {
        Ok(set) => Ok(created_json(set)),
        Err(e) => {
            error!("Failed to create distribution set: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// ============================================================================
// List Distribution Sets
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ListDistributionSetsQuery {
    pub status: Option<String>,
    pub distribution_type: Option<String>,
}

pub async fn list_distribution_sets(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListDistributionSetsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.distribution_set_engine.list_sets(
        org_id, query.status.as_deref(), query.distribution_type.as_deref(),
    ).await {
        Ok(sets) => Ok(Json(serde_json::json!({ "data": sets }))),
        Err(e) => {
            error!("Failed to list distribution sets: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// ============================================================================
// Get Distribution Set
// ============================================================================

pub async fn get_distribution_set(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.distribution_set_engine.get_set(id).await {
        Ok(Some(set)) => Ok(to_json(set)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get distribution set: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// ============================================================================
// Activate Distribution Set
// ============================================================================

pub async fn activate_distribution_set(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.distribution_set_engine.activate_set(id).await {
        Ok(set) => Ok(to_json(set)),
        Err(e) => {
            error!("Failed to activate distribution set: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) | atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// ============================================================================
// Deactivate Distribution Set
// ============================================================================

pub async fn deactivate_distribution_set(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.distribution_set_engine.deactivate_set(id).await {
        Ok(set) => Ok(to_json(set)),
        Err(e) => {
            error!("Failed to deactivate distribution set: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) | atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// ============================================================================
// Delete Distribution Set
// ============================================================================

pub async fn delete_distribution_set(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    match state.distribution_set_engine.delete_set(id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            error!("Failed to delete distribution set: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// ============================================================================
// Add Distribution Set Line
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct AddDistributionLineRequest {
    pub account_combination: String,
    pub account_description: Option<String>,
    pub segment1: Option<String>,
    pub segment2: Option<String>,
    pub segment3: Option<String>,
    pub segment4: Option<String>,
    pub segment5: Option<String>,
    pub percentage: String,
    pub amount: Option<String>,
    pub description: Option<String>,
    pub cost_center: Option<String>,
    pub department: Option<String>,
    pub project_code: Option<String>,
}

pub async fn add_distribution_line(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(set_id): Path<Uuid>,
    Json(payload): Json<AddDistributionLineRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.distribution_set_engine.add_line(
        org_id,
        set_id,
        &payload.account_combination,
        payload.account_description.as_deref(),
        payload.segment1.as_deref(),
        payload.segment2.as_deref(),
        payload.segment3.as_deref(),
        payload.segment4.as_deref(),
        payload.segment5.as_deref(),
        &payload.percentage,
        payload.amount.as_deref(),
        payload.description.as_deref(),
        payload.cost_center.as_deref(),
        payload.department.as_deref(),
        payload.project_code.as_deref(),
    ).await {
        Ok(line) => Ok(created_json(line)),
        Err(e) => {
            error!("Failed to add distribution line: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) | atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// ============================================================================
// List Distribution Set Lines
// ============================================================================

pub async fn list_distribution_lines(
    State(state): State<Arc<AppState>>,
    Path(set_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.distribution_set_engine.list_lines(set_id).await {
        Ok(lines) => Ok(Json(serde_json::json!({ "data": lines }))),
        Err(e) => {
            error!("Failed to list distribution lines: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// ============================================================================
// Remove Distribution Set Line
// ============================================================================

pub async fn remove_distribution_line(
    State(state): State<Arc<AppState>>,
    Path(line_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    match state.distribution_set_engine.remove_line(line_id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            error!("Failed to remove distribution line: {}", e);
            Err(match e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// ============================================================================
// Apply Distribution Set to Invoice
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ApplyToInvoiceRequest {
    pub invoice_id: Uuid,
    pub invoice_number: Option<String>,
    pub invoice_amount: String,
}

pub async fn apply_to_invoice(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(set_id): Path<Uuid>,
    Json(payload): Json<ApplyToInvoiceRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.distribution_set_engine.apply_to_invoice(
        org_id,
        set_id,
        payload.invoice_id,
        payload.invoice_number.as_deref(),
        &payload.invoice_amount,
        Some(user_id),
    ).await {
        Ok(lines) => Ok(Json(serde_json::json!({
            "applied": true,
            "lineCount": lines.len(),
            "lines": lines,
        }))),
        Err(e) => {
            error!("Failed to apply distribution set: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) | atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// ============================================================================
// Usage History
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ListUsageQuery {
    pub distribution_set_id: Option<Uuid>,
}

pub async fn list_distribution_set_usage(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListUsageQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.distribution_set_engine.list_usage(
        org_id, query.distribution_set_id,
    ).await {
        Ok(usage) => Ok(Json(serde_json::json!({ "data": usage }))),
        Err(e) => {
            error!("Failed to list distribution set usage: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// ============================================================================
// Dashboard
// ============================================================================

pub async fn get_distribution_set_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.distribution_set_engine.get_dashboard(org_id).await {
        Ok(dashboard) => Ok(to_json(dashboard)),
        Err(e) => {
            error!("Failed to get distribution set dashboard: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
