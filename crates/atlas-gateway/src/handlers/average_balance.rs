//! Average Balance Processing Handlers
//!
//! Oracle Fusion: Financials > General Ledger > Average Balances

use crate::handlers::auth::Claims;
use crate::handlers::{created_json, to_json};
use crate::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension, Json,
};
use serde::Deserialize;
use std::sync::Arc;
use tracing::error;
use uuid::Uuid;

// ============================================================================
// Books
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateBookRequest {
    pub book_code: String,
    pub book_name: String,
    pub description: Option<String>,
    pub period_type: Option<String>,
    pub averaging_window_days: Option<i32>,
    pub is_primary: Option<bool>,
    pub currency_code: Option<String>,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
}

pub async fn create_book(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateBookRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state
        .financials
        .average_balance_engine
        .create_book(
            org_id,
            &payload.book_code,
            &payload.book_name,
            payload.description.as_deref(),
            payload.period_type.as_deref().unwrap_or("daily"),
            payload.averaging_window_days.unwrap_or(30),
            payload.is_primary.unwrap_or(false),
            payload.currency_code.as_deref().unwrap_or("USD"),
            payload
                .effective_from
                .unwrap_or_else(|| chrono::Utc::now().date_naive()),
            payload.effective_to,
            Some(user_id),
        )
        .await
    {
        Ok(book) => Ok(created_json(book)),
        Err(e) => {
            error!("Failed to create average balance book: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListBooksQuery {
    pub status: Option<String>,
}

pub async fn list_books(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListBooksQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state
        .financials
        .average_balance_engine
        .list_books(org_id, query.status.as_deref())
        .await
    {
        Ok(books) => Ok(Json(serde_json::json!({ "data": books }))),
        Err(e) => {
            error!("Failed to list average balance books: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn get_book(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.financials.average_balance_engine.get_book(id).await {
        Ok(Some(b)) => Ok(to_json(b)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get book: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn activate_book(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state
        .financials
        .average_balance_engine
        .activate_book(id)
        .await
    {
        Ok(b) => Ok(to_json(b)),
        Err(e) => {
            error!("Failed to activate book: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn deactivate_book(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state
        .financials
        .average_balance_engine
        .deactivate_book(id)
        .await
    {
        Ok(b) => Ok(to_json(b)),
        Err(e) => {
            error!("Failed to deactivate book: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn delete_book(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    match state
        .financials
        .average_balance_engine
        .delete_book(id)
        .await
    {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            error!("Failed to delete book: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// ============================================================================
// Book Accounts
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct AddAccountRequest {
    pub gl_account: String,
    pub gl_account_name: Option<String>,
    pub account_type: Option<String>,
    pub track_negative: Option<bool>,
}

pub async fn add_account(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(book_id): Path<Uuid>,
    Json(payload): Json<AddAccountRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state
        .financials
        .average_balance_engine
        .add_account(
            org_id,
            book_id,
            &payload.gl_account,
            payload.gl_account_name.as_deref(),
            payload.account_type.as_deref(),
            payload.track_negative.unwrap_or(false),
        )
        .await
    {
        Ok(account) => Ok(created_json(account)),
        Err(e) => {
            error!("Failed to add account: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_)
                | atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn list_accounts(
    State(state): State<Arc<AppState>>,
    Path(book_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state
        .financials
        .average_balance_engine
        .list_accounts(book_id)
        .await
    {
        Ok(accounts) => Ok(Json(serde_json::json!({ "data": accounts }))),
        Err(e) => {
            error!("Failed to list accounts: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn remove_account(
    State(state): State<Arc<AppState>>,
    Path(account_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    match state
        .financials
        .average_balance_engine
        .remove_account(account_id)
        .await
    {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            error!("Failed to remove account: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// ============================================================================
// Daily Balances
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct UpsertDailyBalanceRequest {
    pub account_id: Uuid,
    pub balance_date: chrono::NaiveDate,
    pub closing_balance: String,
    pub opening_balance: String,
    pub total_debits: String,
    pub total_credits: String,
    pub transaction_count: Option<i32>,
    pub negative_balance: Option<String>,
}

pub async fn upsert_daily_balance(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(book_id): Path<Uuid>,
    Json(payload): Json<UpsertDailyBalanceRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state
        .financials
        .average_balance_engine
        .upsert_daily_balance(
            org_id,
            book_id,
            payload.account_id,
            payload.balance_date,
            &payload.closing_balance,
            &payload.opening_balance,
            &payload.total_debits,
            &payload.total_credits,
            payload.transaction_count.unwrap_or(0),
            payload.negative_balance.as_deref().unwrap_or("0.00"),
        )
        .await
    {
        Ok(db) => Ok(to_json(db)),
        Err(e) => {
            error!("Failed to upsert daily balance: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_)
                | atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListDailyBalancesQuery {
    pub account_id: Uuid,
    pub from_date: Option<chrono::NaiveDate>,
    pub to_date: Option<chrono::NaiveDate>,
}

pub async fn list_daily_balances(
    State(state): State<Arc<AppState>>,
    Path(book_id): Path<Uuid>,
    Query(query): Query<ListDailyBalancesQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state
        .financials
        .average_balance_engine
        .list_daily_balances(book_id, query.account_id, query.from_date, query.to_date)
        .await
    {
        Ok(balances) => Ok(Json(serde_json::json!({ "data": balances }))),
        Err(e) => {
            error!("Failed to list daily balances: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// ============================================================================
// Calculations
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CalculateRequest {
    pub account_id: Uuid,
    pub period_start_date: chrono::NaiveDate,
    pub period_end_date: chrono::NaiveDate,
    pub calculation_type: Option<String>,
}

pub async fn calculate_average_balance(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(book_id): Path<Uuid>,
    Json(payload): Json<CalculateRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state
        .financials
        .average_balance_engine
        .calculate_average_balance(
            org_id,
            book_id,
            payload.account_id,
            payload.period_start_date,
            payload.period_end_date,
            payload.calculation_type.as_deref().unwrap_or("daily"),
            Some(user_id),
        )
        .await
    {
        Ok(calc) => Ok(to_json(calc)),
        Err(e) => {
            error!("Failed to calculate average balance: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn get_calculation(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state
        .financials
        .average_balance_engine
        .get_calculation(id)
        .await
    {
        Ok(Some(c)) => Ok(to_json(c)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get calculation: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListCalculationsQuery {
    pub account_id: Option<Uuid>,
    pub calculation_type: Option<String>,
    pub status: Option<String>,
}

pub async fn list_calculations(
    State(state): State<Arc<AppState>>,
    Path(book_id): Path<Uuid>,
    Query(query): Query<ListCalculationsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state
        .financials
        .average_balance_engine
        .list_calculations(
            book_id,
            query.account_id,
            query.calculation_type.as_deref(),
            query.status.as_deref(),
        )
        .await
    {
        Ok(calcs) => Ok(Json(serde_json::json!({ "data": calcs }))),
        Err(e) => {
            error!("Failed to list calculations: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn approve_calculation(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state
        .financials
        .average_balance_engine
        .approve_calculation(id, Some(user_id))
        .await
    {
        Ok(c) => Ok(to_json(c)),
        Err(e) => {
            error!("Failed to approve calculation: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn post_calculation(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state
        .financials
        .average_balance_engine
        .post_calculation(id)
        .await
    {
        Ok(c) => Ok(to_json(c)),
        Err(e) => {
            error!("Failed to post calculation: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
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
    match state
        .financials
        .average_balance_engine
        .get_dashboard(org_id)
        .await
    {
        Ok(dashboard) => Ok(to_json(dashboard)),
        Err(e) => {
            error!("Failed to get dashboard: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
