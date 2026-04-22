//! Fixed Assets Management Handlers
//!
//! Oracle Fusion Cloud ERP: Fixed Assets
//!
//! API endpoints for managing asset categories, asset books, fixed assets,
//! depreciation, asset transfers, and asset retirements.

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
// Asset Category Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateAssetCategoryRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(default = "default_depreciation_method")]
    pub default_depreciation_method: String,
    #[serde(default = "default_useful_life")]
    pub default_useful_life_months: i32,
    #[serde(default)]
    pub default_salvage_value_percent: String,
    pub default_asset_account_code: Option<String>,
    pub default_accum_depr_account_code: Option<String>,
    pub default_depr_expense_account_code: Option<String>,
    pub default_gain_loss_account_code: Option<String>,
}

fn default_depreciation_method() -> String { "straight_line".to_string() }
fn default_useful_life() -> i32 { 60 }

/// Create or update an asset category
pub async fn create_asset_category(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateAssetCategoryRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    info!("Creating asset category {} for org {}", payload.code, org_id);

    match state.fixed_asset_engine.create_category(
        org_id, &payload.code, &payload.name, payload.description.as_deref(),
        &payload.default_depreciation_method, payload.default_useful_life_months,
        &payload.default_salvage_value_percent,
        payload.default_asset_account_code.as_deref(),
        payload.default_accum_depr_account_code.as_deref(),
        payload.default_depr_expense_account_code.as_deref(),
        payload.default_gain_loss_account_code.as_deref(),
        Some(user_id),
    ).await {
        Ok(cat) => Ok((StatusCode::CREATED, Json(serde_json::to_value(cat).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => {
            error!("Failed to create asset category: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 409 => StatusCode::CONFLICT, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

/// Get an asset category by code
pub async fn get_asset_category(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(code): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.fixed_asset_engine.get_category(org_id, &code).await {
        Ok(Some(cat)) => Ok(Json(serde_json::to_value(cat).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Failed to get asset category: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

/// List all asset categories
pub async fn list_asset_categories(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.fixed_asset_engine.list_categories(org_id).await {
        Ok(cats) => Ok(Json(serde_json::json!({ "data": cats }))),
        Err(e) => { error!("Failed to list asset categories: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

/// Delete (deactivate) an asset category
pub async fn delete_asset_category(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(code): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.fixed_asset_engine.delete_category(org_id, &code).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => { error!("Failed to delete asset category: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// ============================================================================
// Asset Book Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateAssetBookRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(default = "default_book_type")]
    pub book_type: String,
    #[serde(default = "default_true")]
    pub auto_depreciation: bool,
    #[serde(default = "default_calendar")]
    pub depreciation_calendar: String,
}

fn default_book_type() -> String { "corporate".to_string() }
fn default_true() -> bool { true }
fn default_calendar() -> String { "monthly".to_string() }

/// Create or update an asset book
pub async fn create_asset_book(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateAssetBookRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    info!("Creating asset book {} for org {}", payload.code, org_id);

    match state.fixed_asset_engine.create_book(
        org_id, &payload.code, &payload.name, payload.description.as_deref(),
        &payload.book_type, payload.auto_depreciation, &payload.depreciation_calendar,
        Some(user_id),
    ).await {
        Ok(book) => Ok((StatusCode::CREATED, Json(serde_json::to_value(book).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => {
            error!("Failed to create asset book: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

/// List all asset books
pub async fn list_asset_books(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.fixed_asset_engine.list_books(org_id).await {
        Ok(books) => Ok(Json(serde_json::json!({ "data": books }))),
        Err(e) => { error!("Failed to list asset books: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// ============================================================================
// Fixed Asset Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateFixedAssetRequest {
    pub asset_number: String,
    pub asset_name: String,
    pub description: Option<String>,
    pub category_code: Option<String>,
    pub book_code: Option<String>,
    #[serde(default = "default_asset_type")]
    pub asset_type: String,
    pub original_cost: String,
    #[serde(default)]
    pub salvage_value: String,
    #[serde(default)]
    pub salvage_value_percent: String,
    pub depreciation_method: Option<String>,
    pub useful_life_months: Option<i32>,
    pub declining_balance_rate: Option<String>,
    pub acquisition_date: Option<chrono::NaiveDate>,
    pub location: Option<String>,
    pub department_id: Option<Uuid>,
    pub department_name: Option<String>,
    pub custodian_id: Option<Uuid>,
    pub custodian_name: Option<String>,
    pub serial_number: Option<String>,
    pub tag_number: Option<String>,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub warranty_expiry: Option<chrono::NaiveDate>,
    pub insurance_policy_number: Option<String>,
    pub insurance_expiry: Option<chrono::NaiveDate>,
    pub lease_number: Option<String>,
    pub lease_expiry: Option<chrono::NaiveDate>,
    pub asset_account_code: Option<String>,
    pub accum_depr_account_code: Option<String>,
    pub depr_expense_account_code: Option<String>,
    pub gain_loss_account_code: Option<String>,
}

fn default_asset_type() -> String { "tangible".to_string() }

/// Create a new fixed asset
pub async fn create_fixed_asset(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateFixedAssetRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    info!("Creating fixed asset {} for org {}", payload.asset_number, org_id);

    let salvage = if payload.salvage_value.is_empty() { "0" } else { &payload.salvage_value };
    let salvage_pct = if payload.salvage_value_percent.is_empty() { "0" } else { &payload.salvage_value_percent };

    match state.fixed_asset_engine.create_asset(
        org_id, &payload.asset_number, &payload.asset_name, payload.description.as_deref(),
        payload.category_code.as_deref(), payload.book_code.as_deref(),
        &payload.asset_type, &payload.original_cost, salvage, salvage_pct,
        payload.depreciation_method.as_deref(), payload.useful_life_months,
        payload.declining_balance_rate.as_deref(),
        payload.acquisition_date,
        payload.location.as_deref(), payload.department_id, payload.department_name.as_deref(),
        payload.custodian_id, payload.custodian_name.as_deref(),
        payload.serial_number.as_deref(), payload.tag_number.as_deref(),
        payload.manufacturer.as_deref(), payload.model.as_deref(),
        payload.warranty_expiry, payload.insurance_policy_number.as_deref(),
        payload.insurance_expiry, payload.lease_number.as_deref(), payload.lease_expiry,
        payload.asset_account_code.as_deref(), payload.accum_depr_account_code.as_deref(),
        payload.depr_expense_account_code.as_deref(), payload.gain_loss_account_code.as_deref(),
        Some(user_id),
    ).await {
        Ok(asset) => Ok((StatusCode::CREATED, Json(serde_json::to_value(asset).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => {
            error!("Failed to create fixed asset: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 404 => StatusCode::NOT_FOUND, 409 => StatusCode::CONFLICT, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

/// Get a fixed asset by ID
pub async fn get_fixed_asset(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.fixed_asset_engine.get_asset(id).await {
        Ok(Some(asset)) => Ok(Json(serde_json::to_value(asset).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Failed to get fixed asset: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListAssetsQuery {
    pub status: Option<String>,
    pub category_code: Option<String>,
    pub book_code: Option<String>,
}

/// List fixed assets with optional filters
pub async fn list_fixed_assets(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListAssetsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.fixed_asset_engine.list_assets(
        org_id, query.status.as_deref(), query.category_code.as_deref(), query.book_code.as_deref(),
    ).await {
        Ok(assets) => Ok(Json(serde_json::json!({ "data": assets }))),
        Err(e) => {
            error!("Failed to list fixed assets: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

// ============================================================================
// Asset Lifecycle Handlers
// ============================================================================

/// Acquire a draft asset
pub async fn acquire_fixed_asset(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.fixed_asset_engine.acquire_asset(id).await {
        Ok(asset) => Ok(Json(serde_json::to_value(asset).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => {
            error!("Failed to acquire asset: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 404 => StatusCode::NOT_FOUND, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

/// Place an asset in service
pub async fn place_asset_in_service(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.fixed_asset_engine.place_in_service(id, None).await {
        Ok(asset) => Ok(Json(serde_json::to_value(asset).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => {
            error!("Failed to place asset in service: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 404 => StatusCode::NOT_FOUND, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

// ============================================================================
// Depreciation Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CalculateDepreciationRequest {
    pub fiscal_year: i32,
    pub period_number: i32,
    pub period_name: Option<String>,
    pub depreciation_date: chrono::NaiveDate,
}

/// Calculate depreciation for a single asset
pub async fn calculate_depreciation(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<CalculateDepreciationRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.fixed_asset_engine.calculate_depreciation(
        id, payload.fiscal_year, payload.period_number,
        payload.period_name.as_deref(), payload.depreciation_date,
        Some(user_id),
    ).await {
        Ok((dep_amount, asset)) => Ok(Json(serde_json::json!({
            "depreciation_amount": format!("{:.2}", dep_amount),
            "asset": asset,
        }))),
        Err(e) => {
            error!("Failed to calculate depreciation: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 404 => StatusCode::NOT_FOUND, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

/// List depreciation history for an asset
pub async fn list_depreciation_history(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.fixed_asset_engine.list_depreciation_history(id).await {
        Ok(history) => Ok(Json(serde_json::json!({ "data": history }))),
        Err(e) => { error!("Failed to list depreciation history: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// ============================================================================
// Asset Transfer Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateAssetTransferRequest {
    pub asset_id: Uuid,
    pub to_department_id: Option<Uuid>,
    pub to_department_name: Option<String>,
    pub to_location: Option<String>,
    pub to_custodian_id: Option<Uuid>,
    pub to_custodian_name: Option<String>,
    pub transfer_date: chrono::NaiveDate,
    pub reason: Option<String>,
}

/// Create an asset transfer
pub async fn create_asset_transfer(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateAssetTransferRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.fixed_asset_engine.create_transfer(
        org_id, payload.asset_id,
        payload.to_department_id, payload.to_department_name.as_deref(),
        payload.to_location.as_deref(),
        payload.to_custodian_id, payload.to_custodian_name.as_deref(),
        payload.transfer_date, payload.reason.as_deref(),
        Some(user_id),
    ).await {
        Ok(transfer) => Ok((StatusCode::CREATED, Json(serde_json::to_value(transfer).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => {
            error!("Failed to create asset transfer: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 404 => StatusCode::NOT_FOUND, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

/// Approve an asset transfer
pub async fn approve_asset_transfer(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.fixed_asset_engine.approve_transfer(id, user_id).await {
        Ok(transfer) => Ok(Json(serde_json::to_value(transfer).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => {
            error!("Failed to approve asset transfer: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 404 => StatusCode::NOT_FOUND, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct RejectTransferRequest {
    pub reason: Option<String>,
}

/// Reject an asset transfer
pub async fn reject_asset_transfer(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<RejectTransferRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.fixed_asset_engine.reject_transfer(id, payload.reason.as_deref()).await {
        Ok(transfer) => Ok(Json(serde_json::to_value(transfer).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => {
            error!("Failed to reject asset transfer: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 404 => StatusCode::NOT_FOUND, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

/// List asset transfers
pub async fn list_asset_transfers(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListTransfersQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.fixed_asset_engine.list_transfers(org_id, query.asset_id).await {
        Ok(transfers) => Ok(Json(serde_json::json!({ "data": transfers }))),
        Err(e) => { error!("Failed to list asset transfers: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListTransfersQuery {
    pub asset_id: Option<Uuid>,
}

// ============================================================================
// Asset Retirement Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateAssetRetirementRequest {
    pub asset_id: Uuid,
    #[serde(default = "default_retirement_type")]
    pub retirement_type: String,
    pub retirement_date: chrono::NaiveDate,
    #[serde(default)]
    pub proceeds: String,
    #[serde(default)]
    pub removal_cost: String,
    pub reference_number: Option<String>,
    pub buyer_name: Option<String>,
    pub notes: Option<String>,
}

fn default_retirement_type() -> String { "sale".to_string() }

/// Create an asset retirement
pub async fn create_asset_retirement(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateAssetRetirementRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let proceeds = if payload.proceeds.is_empty() { "0" } else { &payload.proceeds };
    let removal = if payload.removal_cost.is_empty() { "0" } else { &payload.removal_cost };

    match state.fixed_asset_engine.create_retirement(
        org_id, payload.asset_id, &payload.retirement_type,
        payload.retirement_date, proceeds, removal,
        payload.reference_number.as_deref(), payload.buyer_name.as_deref(),
        payload.notes.as_deref(), Some(user_id),
    ).await {
        Ok(retirement) => Ok((StatusCode::CREATED, Json(serde_json::to_value(retirement).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => {
            error!("Failed to create asset retirement: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 404 => StatusCode::NOT_FOUND, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

/// Approve an asset retirement
pub async fn approve_asset_retirement(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.fixed_asset_engine.approve_retirement(id, user_id).await {
        Ok(retirement) => Ok(Json(serde_json::to_value(retirement).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => {
            error!("Failed to approve asset retirement: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 404 => StatusCode::NOT_FOUND, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

/// List asset retirements
pub async fn list_asset_retirements(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListRetirementsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.fixed_asset_engine.list_retirements(org_id, query.asset_id).await {
        Ok(retirements) => Ok(Json(serde_json::json!({ "data": retirements }))),
        Err(e) => { error!("Failed to list asset retirements: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListRetirementsQuery {
    pub asset_id: Option<Uuid>,
}
