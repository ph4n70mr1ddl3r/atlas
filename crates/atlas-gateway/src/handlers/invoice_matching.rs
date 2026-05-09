//! Invoice Matching Handlers
//!
//! Oracle Fusion: Financials > Payables > Invoice Matching

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

#[derive(Debug, Deserialize)]
pub struct CreateMatchRequest {
    pub match_number: String,
    pub invoice_id: Uuid,
    pub invoice_number: Option<String>,
    pub purchase_order_id: Uuid,
    pub po_number: Option<String>,
    pub supplier_id: Uuid,
    pub supplier_name: String,
    pub match_type: String,
    pub invoice_amount: String,
    pub po_amount: String,
    pub receipt_amount: Option<String>,
    pub inspection_amount: Option<String>,
    pub price_tolerance_pct: Option<String>,
    pub quantity_tolerance_pct: Option<String>,
    pub amount_tolerance: Option<String>,
    pub receipt_id: Option<Uuid>,
    pub receipt_number: Option<String>,
    pub inspection_id: Option<Uuid>,
    pub inspection_status: Option<String>,
}

pub async fn create_match(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateMatchRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.invoice_matching_engine.create_match(
        org_id, &payload.match_number, payload.invoice_id,
        payload.invoice_number.as_deref(), payload.purchase_order_id,
        payload.po_number.as_deref(), payload.supplier_id, &payload.supplier_name,
        &payload.match_type, &payload.invoice_amount, &payload.po_amount,
        payload.receipt_amount.as_deref(), payload.inspection_amount.as_deref(),
        payload.price_tolerance_pct.as_deref().unwrap_or("2.00"),
        payload.quantity_tolerance_pct.as_deref().unwrap_or("5.00"),
        payload.amount_tolerance.as_deref().unwrap_or("100.00"),
        payload.receipt_id, payload.receipt_number.as_deref(),
        payload.inspection_id, payload.inspection_status.as_deref(),
        Some(user_id),
    ).await {
        Ok(m) => Ok((StatusCode::CREATED, Json(serde_json::to_value(m).unwrap()))),
        Err(e) => {
            error!("Failed to create match: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListMatchesQuery {
    pub status: Option<String>,
    pub match_type: Option<String>,
    pub supplier_id: Option<Uuid>,
}

pub async fn list_matches(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListMatchesQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.invoice_matching_engine.list_matches(
        org_id, query.status.as_deref(), query.match_type.as_deref(), query.supplier_id,
    ).await {
        Ok(matches) => Ok(Json(serde_json::json!({ "data": matches }))),
        Err(e) => { error!("Failed to list matches: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn get_match(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.invoice_matching_engine.get_match(id).await {
        Ok(Some(m)) => Ok(Json(serde_json::to_value(m).unwrap())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Failed to get match: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

#[derive(Debug, Deserialize)]
pub struct HoldMatchRequest { pub reason: Option<String> }

pub async fn hold_match(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<HoldMatchRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.invoice_matching_engine.hold_match(id, payload.reason.as_deref(), Some(user_id)).await {
        Ok(m) => Ok(Json(serde_json::to_value(m).unwrap())),
        Err(e) => {
            error!("Failed to hold match: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct OverrideMatchRequest { pub reason: String }

pub async fn override_match(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<OverrideMatchRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.invoice_matching_engine.override_match(id, &payload.reason, Some(user_id)).await {
        Ok(m) => Ok(Json(serde_json::to_value(m).unwrap())),
        Err(e) => {
            error!("Failed to override match: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) | atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ConfirmMatchRequest {}

pub async fn confirm_match(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.invoice_matching_engine.confirm_match(id, Some(user_id)).await {
        Ok(m) => Ok(Json(serde_json::to_value(m).unwrap())),
        Err(e) => {
            error!("Failed to confirm match: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CancelMatchRequest { pub reason: Option<String> }

pub async fn cancel_match(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<CancelMatchRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.invoice_matching_engine.cancel_match(id, payload.reason.as_deref()).await {
        Ok(m) => Ok(Json(serde_json::to_value(m).unwrap())),
        Err(e) => {
            error!("Failed to cancel match: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// Match Lines
#[derive(Debug, Deserialize)]
pub struct AddMatchLineRequest {
    pub match_id: Uuid,
    pub invoice_line_id: Option<Uuid>,
    pub po_line_id: Option<Uuid>,
    pub receipt_line_id: Option<Uuid>,
    pub inspection_line_id: Option<Uuid>,
    pub line_number: i32,
    pub item_description: Option<String>,
    pub invoice_quantity: String,
    pub po_quantity: String,
    pub receipt_quantity: Option<String>,
    pub inspected_quantity: Option<String>,
    pub invoice_unit_price: String,
    pub po_unit_price: String,
    pub invoice_line_amount: String,
    pub po_line_amount: String,
    pub notes: Option<String>,
}

pub async fn add_match_line(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<AddMatchLineRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.invoice_matching_engine.add_match_line(
        org_id, payload.match_id,
        payload.invoice_line_id, payload.po_line_id,
        payload.receipt_line_id, payload.inspection_line_id,
        payload.line_number, payload.item_description.as_deref(),
        &payload.invoice_quantity, &payload.po_quantity,
        payload.receipt_quantity.as_deref(), payload.inspected_quantity.as_deref(),
        &payload.invoice_unit_price, &payload.po_unit_price,
        &payload.invoice_line_amount, &payload.po_line_amount,
        payload.notes.as_deref(),
    ).await {
        Ok(line) => Ok((StatusCode::CREATED, Json(serde_json::to_value(line).unwrap()))),
        Err(e) => {
            error!("Failed to add match line: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) | atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn list_match_lines(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.invoice_matching_engine.list_match_lines(id).await {
        Ok(lines) => Ok(Json(serde_json::json!({ "data": lines }))),
        Err(e) => { error!("Failed to list match lines: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

#[derive(Debug, Deserialize)]
pub struct OverrideLineRequest {}

pub async fn override_match_line(
    State(state): State<Arc<AppState>>,
    Path(line_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.invoice_matching_engine.override_match_line(line_id).await {
        Ok(line) => Ok(Json(serde_json::to_value(line).unwrap())),
        Err(e) => {
            error!("Failed to override match line: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn get_invoice_matching_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.invoice_matching_engine.get_dashboard(org_id).await {
        Ok(dashboard) => Ok(Json(serde_json::to_value(dashboard).unwrap())),
        Err(e) => { error!("Failed to get dashboard: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}
