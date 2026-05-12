//! Statistical Accounting Handlers
//!
//! Oracle Fusion: Financials > General Ledger > Statistical Accounting
//! Tracks non-monetary statistical data (headcount, sqft, machine hours, etc.)
//! alongside financial data for reporting and allocation purposes.

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
// Statistical Units
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateUnitRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub stat_type: Option<String>,
    pub unit_of_measure: Option<String>,
}

pub async fn create_unit(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateUnitRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.statistical_accounting_engine.create_unit(
        org_id,
        &payload.code,
        &payload.name,
        payload.description.as_deref(),
        payload.stat_type.as_deref().unwrap_or("custom"),
        payload.unit_of_measure.as_deref().unwrap_or("each"),
        Some(user_id),
    ).await {
        Ok(unit) => Ok((StatusCode::CREATED, Json(crate::handlers::records::to_json_or_null(unit)))),
        Err(e) => {
            error!("Failed to create statistical unit: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListUnitsQuery {
    pub stat_type: Option<String>,
    pub is_active: Option<bool>,
}

pub async fn list_units(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListUnitsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.statistical_accounting_engine.list_units(
        org_id,
        query.stat_type.as_deref(),
        query.is_active,
    ).await {
        Ok(units) => Ok(Json(serde_json::json!({ "data": units }))),
        Err(e) => {
            error!("Failed to list statistical units: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn get_unit(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.statistical_accounting_engine.get_unit(id).await {
        Ok(Some(u)) => Ok(Json(crate::handlers::records::to_json_or_null(u))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get statistical unit: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_unit_by_code(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(code): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.statistical_accounting_engine.get_unit_by_code(org_id, &code).await {
        Ok(Some(u)) => Ok(Json(crate::handlers::records::to_json_or_null(u))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get statistical unit by code: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn activate_unit(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.statistical_accounting_engine.activate_unit(id).await {
        Ok(u) => Ok(Json(crate::handlers::records::to_json_or_null(u))),
        Err(e) => {
            error!("Failed to activate statistical unit: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn deactivate_unit(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.statistical_accounting_engine.deactivate_unit(id).await {
        Ok(u) => Ok(Json(crate::handlers::records::to_json_or_null(u))),
        Err(e) => {
            error!("Failed to deactivate statistical unit: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// ============================================================================
// Statistical Entries
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateEntryRequest {
    pub statistical_unit_id: Uuid,
    pub account_code: Option<String>,
    pub dimension1: Option<String>,
    pub dimension2: Option<String>,
    pub dimension3: Option<String>,
    pub fiscal_year: i32,
    pub period_number: i32,
    pub quantity: String,
    pub unit_cost: Option<String>,
    pub source_type: Option<String>,
    pub source_id: Option<Uuid>,
    pub source_number: Option<String>,
    pub description: Option<String>,
}

pub async fn create_entry(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateEntryRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.statistical_accounting_engine.create_entry(
        org_id,
        payload.statistical_unit_id,
        payload.account_code.as_deref(),
        payload.dimension1.as_deref(),
        payload.dimension2.as_deref(),
        payload.dimension3.as_deref(),
        payload.fiscal_year,
        payload.period_number,
        &payload.quantity,
        payload.unit_cost.as_deref(),
        payload.source_type.as_deref(),
        payload.source_id,
        payload.source_number.as_deref(),
        payload.description.as_deref(),
        Some(user_id),
    ).await {
        Ok(entry) => Ok((StatusCode::CREATED, Json(crate::handlers::records::to_json_or_null(entry)))),
        Err(e) => {
            error!("Failed to create statistical entry: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListEntriesQuery {
    pub unit_id: Option<Uuid>,
    pub fiscal_year: Option<i32>,
    pub period: Option<i32>,
    pub status: Option<String>,
}

pub async fn list_entries(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListEntriesQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.statistical_accounting_engine.list_entries(
        org_id,
        query.unit_id,
        query.fiscal_year,
        query.period,
        query.status.as_deref(),
    ).await {
        Ok(entries) => Ok(Json(serde_json::json!({ "data": entries }))),
        Err(e) => {
            error!("Failed to list statistical entries: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn get_entry(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.statistical_accounting_engine.get_entry(id).await {
        Ok(Some(e)) => Ok(Json(crate::handlers::records::to_json_or_null(e))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get statistical entry: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn post_entry(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.statistical_accounting_engine.post_entry(id).await {
        Ok(e) => Ok(Json(crate::handlers::records::to_json_or_null(e))),
        Err(e) => {
            error!("Failed to post statistical entry: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn reverse_entry(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.statistical_accounting_engine.reverse_entry(id).await {
        Ok(e) => Ok(Json(crate::handlers::records::to_json_or_null(e))),
        Err(e) => {
            error!("Failed to reverse statistical entry: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// ============================================================================
// Balance
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct GetBalanceQuery {
    pub unit_id: Uuid,
    pub fiscal_year: i32,
    pub period: i32,
}

pub async fn get_balance(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<GetBalanceQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.statistical_accounting_engine.get_balance(
        org_id, query.unit_id, query.fiscal_year, query.period,
    ).await {
        Ok(Some(b)) => Ok(Json(crate::handlers::records::to_json_or_null(b))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get statistical balance: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// ============================================================================
// Dashboard
// ============================================================================

pub async fn get_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.statistical_accounting_engine.get_dashboard(org_id).await {
        Ok(dashboard) => Ok(Json(crate::handlers::records::to_json_or_null(dashboard))),
        Err(e) => {
            error!("Failed to get statistical accounting dashboard: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
