//! Cost Accounting Handlers
//!
//! Oracle Fusion Cloud: Cost Management > Cost Accounting
//! Provides HTTP endpoints for:
//! - Cost Books CRUD (standard, average, FIFO, LIFO)
//! - Cost Elements (material, labor, overhead, subcontracting, expense)
//! - Cost Profiles (item-level costing configuration)
//! - Standard Costs per item/element/book
//! - Cost Adjustments with full lifecycle (draft → submitted → approved → posted)
//! - Cost Adjustment Lines
//! - Variance Analysis (purchase price, routing, overhead, rate, usage, mix)
//! - Cost Accounting Dashboard

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension, Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::AppState;
use crate::handlers::auth::Claims;

// ============================================================================
// Cost Books
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCostBookRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub costing_method: String,
    pub currency_code: Option<String>,
    pub effective_from: Option<String>,
    pub effective_to: Option<String>,
}

pub async fn create_cost_book(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateCostBookRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let effective_from = payload
        .effective_from
        .as_deref()
        .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());
    let effective_to = payload
        .effective_to
        .as_deref()
        .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());

    let book = state
        .cost_accounting_engine
        .create_cost_book(
            org_id,
            &payload.code,
            &payload.name,
            payload.description.as_deref(),
            &payload.costing_method,
            payload.currency_code.as_deref().unwrap_or("USD"),
            effective_from,
            effective_to,
            Some(user_id),
        )
        .await
        .map_err(|e| {
            tracing::error!("Create cost book error: {}", e);
            match e {
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(book).unwrap_or_default()),
    ))
}

pub async fn get_cost_book(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let book = state
        .cost_accounting_engine
        .get_cost_book(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match book {
        Some(b) => Ok(Json(serde_json::to_value(b).unwrap_or_default())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

#[derive(Debug, Deserialize)]
pub struct ListCostBooksParams {
    pub costing_method: Option<String>,
    pub include_inactive: Option<String>,
}

pub async fn list_cost_books(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<ListCostBooksParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let include_inactive = params.include_inactive.as_deref() == Some("true");

    let books = state
        .cost_accounting_engine
        .list_cost_books(org_id, params.costing_method.as_deref(), include_inactive)
        .await
        .map_err(|e| {
            tracing::error!("List cost books error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(serde_json::json!({
        "data": books,
        "meta": { "total": books.len() }
    })))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCostBookRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub costing_method: Option<String>,
    pub effective_from: Option<String>,
    pub effective_to: Option<String>,
}

pub async fn update_cost_book(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateCostBookRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let effective_from = payload
        .effective_from
        .as_deref()
        .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());
    let effective_to = payload
        .effective_to
        .as_deref()
        .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());

    let book = state
        .cost_accounting_engine
        .update_cost_book(
            id,
            payload.name.as_deref(),
            payload.description.as_deref(),
            payload.costing_method.as_deref(),
            effective_from,
            effective_to,
        )
        .await
        .map_err(|e| {
            tracing::error!("Update cost book error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(Json(serde_json::to_value(book).unwrap_or_default()))
}

pub async fn deactivate_cost_book(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let book = state
        .cost_accounting_engine
        .deactivate_cost_book(id)
        .await
        .map_err(|e| {
            tracing::error!("Deactivate cost book error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(Json(serde_json::to_value(book).unwrap_or_default()))
}

pub async fn activate_cost_book(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let book = state
        .cost_accounting_engine
        .activate_cost_book(id)
        .await
        .map_err(|e| {
            tracing::error!("Activate cost book error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(Json(serde_json::to_value(book).unwrap_or_default()))
}

pub async fn delete_cost_book(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    state
        .cost_accounting_engine
        .delete_cost_book(id)
        .await
        .map_err(|e| {
            tracing::error!("Delete cost book error: {}", e);
            match e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Cost Elements
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCostElementRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub element_type: String,
    pub cost_book_id: Option<Uuid>,
    pub default_rate: String,
    pub rate_uom: Option<String>,
}

pub async fn create_cost_element(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateCostElementRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let element = state
        .cost_accounting_engine
        .create_cost_element(
            org_id,
            &payload.code,
            &payload.name,
            payload.description.as_deref(),
            &payload.element_type,
            payload.cost_book_id,
            &payload.default_rate,
            payload.rate_uom.as_deref(),
            Some(user_id),
        )
        .await
        .map_err(|e| {
            tracing::error!("Create cost element error: {}", e);
            match e {
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(element).unwrap_or_default()),
    ))
}

pub async fn get_cost_element(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let element = state
        .cost_accounting_engine
        .get_cost_element(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match element {
        Some(e) => Ok(Json(serde_json::to_value(e).unwrap_or_default())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

#[derive(Debug, Deserialize)]
pub struct ListCostElementsParams {
    pub element_type: Option<String>,
    pub cost_book_id: Option<String>,
}

pub async fn list_cost_elements(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<ListCostElementsParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let cost_book_id = params
        .cost_book_id
        .as_deref()
        .and_then(|s| Uuid::parse_str(s).ok());

    let elements = state
        .cost_accounting_engine
        .list_cost_elements(org_id, params.element_type.as_deref(), cost_book_id)
        .await
        .map_err(|e| {
            tracing::error!("List cost elements error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(serde_json::json!({
        "data": elements,
        "meta": { "total": elements.len() }
    })))
}

pub async fn delete_cost_element(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    state
        .cost_accounting_engine
        .delete_cost_element(id)
        .await
        .map_err(|e| {
            tracing::error!("Delete cost element error: {}", e);
            match e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Cost Profiles
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCostProfileRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub cost_book_id: Uuid,
    pub item_id: Option<Uuid>,
    pub item_name: Option<String>,
    pub cost_type: String,
    pub lot_level_costing: Option<bool>,
    pub include_landed_costs: Option<bool>,
    pub overhead_absorption_method: String,
}

pub async fn create_cost_profile(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateCostProfileRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let profile = state
        .cost_accounting_engine
        .create_cost_profile(
            org_id,
            &payload.code,
            &payload.name,
            payload.description.as_deref(),
            payload.cost_book_id,
            payload.item_id,
            payload.item_name.as_deref(),
            &payload.cost_type,
            payload.lot_level_costing.unwrap_or(false),
            payload.include_landed_costs.unwrap_or(true),
            &payload.overhead_absorption_method,
            Some(user_id),
        )
        .await
        .map_err(|e| {
            tracing::error!("Create cost profile error: {}", e);
            match e {
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(profile).unwrap_or_default()),
    ))
}

pub async fn get_cost_profile(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let profile = state
        .cost_accounting_engine
        .get_cost_profile(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match profile {
        Some(p) => Ok(Json(serde_json::to_value(p).unwrap_or_default())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

#[derive(Debug, Deserialize)]
pub struct ListCostProfilesParams {
    pub cost_book_id: Option<String>,
    pub item_id: Option<String>,
}

pub async fn list_cost_profiles(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<ListCostProfilesParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let cost_book_id = params
        .cost_book_id
        .as_deref()
        .and_then(|s| Uuid::parse_str(s).ok());
    let item_id = params
        .item_id
        .as_deref()
        .and_then(|s| Uuid::parse_str(s).ok());

    let profiles = state
        .cost_accounting_engine
        .list_cost_profiles(org_id, cost_book_id, item_id)
        .await
        .map_err(|e| {
            tracing::error!("List cost profiles error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(serde_json::json!({
        "data": profiles,
        "meta": { "total": profiles.len() }
    })))
}

pub async fn delete_cost_profile(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    state
        .cost_accounting_engine
        .delete_cost_profile(id)
        .await
        .map_err(|e| {
            tracing::error!("Delete cost profile error: {}", e);
            match e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Standard Costs
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateStandardCostRequest {
    pub cost_book_id: Uuid,
    pub cost_profile_id: Option<Uuid>,
    pub cost_element_id: Uuid,
    pub item_id: Uuid,
    pub item_name: Option<String>,
    pub standard_cost: String,
    pub currency_code: Option<String>,
    pub effective_date: String,
}

pub async fn create_standard_cost(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateStandardCostRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let effective_date = chrono::NaiveDate::parse_from_str(&payload.effective_date, "%Y-%m-%d")
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let cost = state
        .cost_accounting_engine
        .create_standard_cost(
            org_id,
            payload.cost_book_id,
            payload.cost_profile_id,
            payload.cost_element_id,
            payload.item_id,
            payload.item_name.as_deref(),
            &payload.standard_cost,
            payload.currency_code.as_deref().unwrap_or("USD"),
            effective_date,
            Some(user_id),
        )
        .await
        .map_err(|e| {
            tracing::error!("Create standard cost error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(cost).unwrap_or_default()),
    ))
}

pub async fn get_standard_cost(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let cost = state
        .cost_accounting_engine
        .get_standard_cost(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match cost {
        Some(c) => Ok(Json(serde_json::to_value(c).unwrap_or_default())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

#[derive(Debug, Deserialize)]
pub struct ListStandardCostsParams {
    pub cost_book_id: Option<String>,
    pub item_id: Option<String>,
}

pub async fn list_standard_costs(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<ListStandardCostsParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let cost_book_id = params
        .cost_book_id
        .as_deref()
        .and_then(|s| Uuid::parse_str(s).ok());
    let item_id = params
        .item_id
        .as_deref()
        .and_then(|s| Uuid::parse_str(s).ok());

    let costs = state
        .cost_accounting_engine
        .list_standard_costs(org_id, cost_book_id, item_id)
        .await
        .map_err(|e| {
            tracing::error!("List standard costs error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(serde_json::json!({
        "data": costs,
        "meta": { "total": costs.len() }
    })))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStandardCostRequest {
    pub standard_cost: String,
}

pub async fn update_standard_cost(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateStandardCostRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let cost = state
        .cost_accounting_engine
        .update_standard_cost(id, &payload.standard_cost)
        .await
        .map_err(|e| {
            tracing::error!("Update standard cost error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(Json(serde_json::to_value(cost).unwrap_or_default()))
}

pub async fn supersede_standard_cost(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let cost = state
        .cost_accounting_engine
        .supersede_standard_cost(id)
        .await
        .map_err(|e| {
            tracing::error!("Supersede standard cost error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(Json(serde_json::to_value(cost).unwrap_or_default()))
}

pub async fn delete_standard_cost(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    state
        .cost_accounting_engine
        .delete_standard_cost(id)
        .await
        .map_err(|e| {
            tracing::error!("Delete standard cost error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Cost Adjustments
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCostAdjustmentRequest {
    pub cost_book_id: Uuid,
    pub adjustment_type: String,
    pub description: Option<String>,
    pub reason: Option<String>,
    pub currency_code: Option<String>,
    pub effective_date: Option<String>,
}

pub async fn create_cost_adjustment(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateCostAdjustmentRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let effective_date = payload
        .effective_date
        .as_deref()
        .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());

    let adj = state
        .cost_accounting_engine
        .create_cost_adjustment(
            org_id,
            payload.cost_book_id,
            &payload.adjustment_type,
            payload.description.as_deref(),
            payload.reason.as_deref(),
            payload.currency_code.as_deref().unwrap_or("USD"),
            effective_date,
            Some(user_id),
        )
        .await
        .map_err(|e| {
            tracing::error!("Create cost adjustment error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(adj).unwrap_or_default()),
    ))
}

pub async fn get_cost_adjustment(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let adj = state
        .cost_accounting_engine
        .get_cost_adjustment(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match adj {
        Some(a) => Ok(Json(serde_json::to_value(a).unwrap_or_default())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

#[derive(Debug, Deserialize)]
pub struct ListCostAdjustmentsParams {
    pub status: Option<String>,
    pub adjustment_type: Option<String>,
}

pub async fn list_cost_adjustments(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<ListCostAdjustmentsParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let adjustments = state
        .cost_accounting_engine
        .list_cost_adjustments(org_id, params.status.as_deref(), params.adjustment_type.as_deref())
        .await
        .map_err(|e| {
            tracing::error!("List cost adjustments error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(serde_json::json!({
        "data": adjustments,
        "meta": { "total": adjustments.len() }
    })))
}

pub async fn submit_adjustment(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let adj = state
        .cost_accounting_engine
        .submit_adjustment(id)
        .await
        .map_err(|e| {
            tracing::error!("Submit adjustment error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(Json(serde_json::to_value(adj).unwrap_or_default()))
}

pub async fn approve_adjustment(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let adj = state
        .cost_accounting_engine
        .approve_adjustment(id, user_id)
        .await
        .map_err(|e| {
            tracing::error!("Approve adjustment error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(Json(serde_json::to_value(adj).unwrap_or_default()))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RejectAdjustmentRequest {
    pub reason: String,
}

pub async fn reject_adjustment(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    claims: Extension<Claims>,
    Json(payload): Json<RejectAdjustmentRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let adj = state
        .cost_accounting_engine
        .reject_adjustment(id, user_id, &payload.reason)
        .await
        .map_err(|e| {
            tracing::error!("Reject adjustment error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(Json(serde_json::to_value(adj).unwrap_or_default()))
}

pub async fn post_adjustment(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let adj = state
        .cost_accounting_engine
        .post_adjustment(id, user_id)
        .await
        .map_err(|e| {
            tracing::error!("Post adjustment error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(Json(serde_json::to_value(adj).unwrap_or_default()))
}

pub async fn delete_cost_adjustment(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    state
        .cost_accounting_engine
        .delete_cost_adjustment(id)
        .await
        .map_err(|e| {
            tracing::error!("Delete cost adjustment error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Cost Adjustment Lines
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddAdjustmentLineRequest {
    pub line_number: i32,
    pub item_id: Uuid,
    pub item_name: Option<String>,
    pub cost_element_id: Option<Uuid>,
    pub old_cost: String,
    pub new_cost: String,
    pub currency_code: Option<String>,
    pub effective_date: Option<String>,
}

pub async fn add_adjustment_line(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(adjustment_id): Path<Uuid>,
    Json(payload): Json<AddAdjustmentLineRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let effective_date = payload
        .effective_date
        .as_deref()
        .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());

    let line = state
        .cost_accounting_engine
        .add_adjustment_line(
            org_id,
            adjustment_id,
            payload.line_number,
            payload.item_id,
            payload.item_name.as_deref(),
            payload.cost_element_id,
            &payload.old_cost,
            &payload.new_cost,
            payload.currency_code.as_deref().unwrap_or("USD"),
            effective_date,
        )
        .await
        .map_err(|e| {
            tracing::error!("Add adjustment line error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(line).unwrap_or_default()),
    ))
}

pub async fn list_adjustment_lines(
    State(state): State<Arc<AppState>>,
    Path(adjustment_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let lines = state
        .cost_accounting_engine
        .list_adjustment_lines(adjustment_id)
        .await
        .map_err(|e| {
            tracing::error!("List adjustment lines error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(serde_json::json!({
        "data": lines,
        "meta": { "total": lines.len() }
    })))
}

pub async fn delete_adjustment_line(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    state
        .cost_accounting_engine
        .delete_adjustment_line(id)
        .await
        .map_err(|e| {
            tracing::error!("Delete adjustment line error: {}", e);
            match e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Cost Variances
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCostVarianceRequest {
    pub cost_book_id: Uuid,
    pub variance_type: String,
    pub variance_date: String,
    pub item_id: Uuid,
    pub item_name: Option<String>,
    pub cost_element_id: Option<Uuid>,
    pub source_type: Option<String>,
    pub source_id: Option<Uuid>,
    pub source_number: Option<String>,
    pub standard_cost: String,
    pub actual_cost: String,
    pub quantity: String,
    pub currency_code: Option<String>,
    pub accounting_period: Option<String>,
}

pub async fn create_cost_variance(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateCostVarianceRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let variance_date =
        chrono::NaiveDate::parse_from_str(&payload.variance_date, "%Y-%m-%d")
            .map_err(|_| StatusCode::BAD_REQUEST)?;

    let variance = state
        .cost_accounting_engine
        .create_cost_variance(
            org_id,
            payload.cost_book_id,
            &payload.variance_type,
            variance_date,
            payload.item_id,
            payload.item_name.as_deref(),
            payload.cost_element_id,
            payload.source_type.as_deref(),
            payload.source_id,
            payload.source_number.as_deref(),
            &payload.standard_cost,
            &payload.actual_cost,
            &payload.quantity,
            payload.currency_code.as_deref().unwrap_or("USD"),
            payload.accounting_period.as_deref(),
            Some(user_id),
        )
        .await
        .map_err(|e| {
            tracing::error!("Create cost variance error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(variance).unwrap_or_default()),
    ))
}

pub async fn get_cost_variance(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let variance = state
        .cost_accounting_engine
        .get_cost_variance(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match variance {
        Some(v) => Ok(Json(serde_json::to_value(v).unwrap_or_default())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

#[derive(Debug, Deserialize)]
pub struct ListCostVariancesParams {
    pub variance_type: Option<String>,
    pub item_id: Option<String>,
    pub cost_book_id: Option<String>,
}

pub async fn list_cost_variances(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<ListCostVariancesParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let item_id = params
        .item_id
        .as_deref()
        .and_then(|s| Uuid::parse_str(s).ok());
    let cost_book_id = params
        .cost_book_id
        .as_deref()
        .and_then(|s| Uuid::parse_str(s).ok());

    let variances = state
        .cost_accounting_engine
        .list_cost_variances(org_id, params.variance_type.as_deref(), item_id, cost_book_id)
        .await
        .map_err(|e| {
            tracing::error!("List cost variances error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(serde_json::json!({
        "data": variances,
        "meta": { "total": variances.len() }
    })))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyzeVarianceRequest {
    pub notes: String,
}

pub async fn analyze_variance(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<AnalyzeVarianceRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let variance = state
        .cost_accounting_engine
        .analyze_variance(id, &payload.notes)
        .await
        .map_err(|e| {
            tracing::error!("Analyze variance error: {}", e);
            match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(Json(serde_json::to_value(variance).unwrap_or_default()))
}

// ============================================================================
// Dashboard
// ============================================================================

pub async fn get_cost_accounting_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let dashboard = state
        .cost_accounting_engine
        .get_dashboard(org_id)
        .await
        .map_err(|e| {
            tracing::error!("Cost accounting dashboard error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(serde_json::to_value(dashboard).unwrap_or_default()))
}
