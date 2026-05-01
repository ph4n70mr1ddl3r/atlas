//! Accounts Receivable Handlers
//!
//! Oracle Fusion Cloud ERP: Receivables > Transactions, Receipts, Credit Memos, Adjustments
//!
//! API endpoints for managing AR transactions, transaction lines, receipts,
//! credit memos, adjustments, and AR aging analysis.

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
// Transaction Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateArTransactionRequest {
    pub transaction_type: String,
    pub transaction_date: chrono::NaiveDate,
    pub customer_id: Uuid,
    pub customer_number: Option<String>,
    pub customer_name: Option<String>,
    #[serde(default = "default_currency_usd")]
    pub currency_code: String,
    pub entered_amount: String,
    #[serde(default = "default_zero")]
    pub tax_amount: String,
    pub payment_terms: Option<String>,
    pub due_date: Option<chrono::NaiveDate>,
    pub gl_date: Option<chrono::NaiveDate>,
    pub reference_number: Option<String>,
    pub purchase_order: Option<String>,
    pub sales_rep: Option<String>,
    pub notes: Option<String>,
}

fn default_currency_usd() -> String { "USD".to_string() }
fn default_zero() -> String { "0.00".to_string() }

/// Create a new AR transaction
pub async fn create_ar_transaction(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateArTransactionRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    info!("Creating AR transaction for org {} customer {}", org_id, payload.customer_id);

    match state.accounts_receivable_engine.create_transaction(
        org_id,
        &payload.transaction_type,
        payload.transaction_date,
        payload.customer_id,
        payload.customer_number.as_deref(),
        payload.customer_name.as_deref(),
        &payload.currency_code,
        &payload.entered_amount,
        &payload.tax_amount,
        payload.payment_terms.as_deref(),
        payload.due_date,
        payload.gl_date,
        payload.reference_number.as_deref(),
        payload.purchase_order.as_deref(),
        payload.sales_rep.as_deref(),
        payload.notes.as_deref(),
        Some(user_id),
    ).await {
        Ok(txn) => Ok((StatusCode::CREATED, Json(serde_json::to_value(txn).unwrap()))),
        Err(e) => {
            error!("Failed to create AR transaction: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListTransactionsQuery {
    pub status: Option<String>,
    pub customer_id: Option<Uuid>,
    pub transaction_type: Option<String>,
}

/// List AR transactions
pub async fn list_ar_transactions(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListTransactionsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.accounts_receivable_engine.list_transactions(
        org_id,
        query.status.as_deref(),
        query.customer_id,
        query.transaction_type.as_deref(),
    ).await {
        Ok(transactions) => Ok(Json(serde_json::json!({ "data": transactions }))),
        Err(e) => {
            error!("Failed to list AR transactions: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get an AR transaction by ID
pub async fn get_ar_transaction(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.accounts_receivable_engine.get_transaction(id).await {
        Ok(Some(txn)) => Ok(Json(serde_json::to_value(txn).unwrap())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get AR transaction: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Complete a draft AR transaction
pub async fn complete_ar_transaction(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.accounts_receivable_engine.complete_transaction(id).await {
        Ok(txn) => Ok(Json(serde_json::to_value(txn).unwrap())),
        Err(e) => {
            error!("Failed to complete AR transaction: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::CONFLICT,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::BAD_REQUEST,
            })
        }
    }
}

/// Post a completed AR transaction
pub async fn post_ar_transaction(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.accounts_receivable_engine.post_transaction(id).await {
        Ok(txn) => Ok(Json(serde_json::to_value(txn).unwrap())),
        Err(e) => {
            error!("Failed to post AR transaction: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::CONFLICT,
                _ => StatusCode::BAD_REQUEST,
            })
        }
    }
}

/// Cancel an AR transaction
pub async fn cancel_ar_transaction(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.accounts_receivable_engine.cancel_transaction(id, None).await {
        Ok(txn) => Ok(Json(serde_json::to_value(txn).unwrap())),
        Err(e) => {
            error!("Failed to cancel AR transaction: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::CONFLICT,
                _ => StatusCode::BAD_REQUEST,
            })
        }
    }
}

// ============================================================================
// Transaction Line Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct AddTransactionLineRequest {
    pub line_type: String,
    pub description: Option<String>,
    pub item_code: Option<String>,
    pub item_description: Option<String>,
    pub unit_of_measure: Option<String>,
    pub quantity: Option<String>,
    pub unit_price: Option<String>,
    pub line_amount: String,
    #[serde(default = "default_zero")]
    pub tax_amount: String,
    pub tax_code: Option<String>,
    pub revenue_account: Option<String>,
}

/// Add a line to an AR transaction
pub async fn add_transaction_line(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(transaction_id): Path<Uuid>,
    Json(payload): Json<AddTransactionLineRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.accounts_receivable_engine.create_transaction_line(
        org_id,
        transaction_id,
        &payload.line_type,
        payload.description.as_deref(),
        payload.item_code.as_deref(),
        payload.item_description.as_deref(),
        payload.unit_of_measure.as_deref(),
        payload.quantity.as_deref(),
        payload.unit_price.as_deref(),
        &payload.line_amount,
        &payload.tax_amount,
        payload.tax_code.as_deref(),
        payload.revenue_account.as_deref(),
        Some(user_id),
    ).await {
        Ok(line) => Ok((StatusCode::CREATED, Json(serde_json::to_value(line).unwrap()))),
        Err(e) => {
            error!("Failed to add AR transaction line: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

/// List transaction lines
pub async fn list_transaction_lines(
    State(state): State<Arc<AppState>>,
    Path(transaction_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.accounts_receivable_engine.list_transaction_lines(transaction_id).await {
        Ok(lines) => Ok(Json(serde_json::json!({ "data": lines }))),
        Err(e) => {
            error!("Failed to list transaction lines: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// ============================================================================
// Receipt Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateReceiptRequest {
    pub receipt_date: chrono::NaiveDate,
    pub receipt_type: String,
    pub receipt_method: String,
    pub amount: String,
    #[serde(default = "default_currency_usd")]
    pub currency_code: String,
    pub customer_id: Option<Uuid>,
    pub customer_number: Option<String>,
    pub customer_name: Option<String>,
    pub reference_number: Option<String>,
    pub bank_account_name: Option<String>,
    pub check_number: Option<String>,
    pub notes: Option<String>,
}

/// Create a new receipt
pub async fn create_receipt(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateReceiptRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.accounts_receivable_engine.create_receipt(
        org_id,
        payload.receipt_date,
        &payload.receipt_type,
        &payload.receipt_method,
        &payload.amount,
        &payload.currency_code,
        payload.customer_id,
        payload.customer_number.as_deref(),
        payload.customer_name.as_deref(),
        payload.reference_number.as_deref(),
        payload.bank_account_name.as_deref(),
        payload.check_number.as_deref(),
        payload.notes.as_deref(),
        Some(user_id),
    ).await {
        Ok(receipt) => Ok((StatusCode::CREATED, Json(serde_json::to_value(receipt).unwrap()))),
        Err(e) => {
            error!("Failed to create receipt: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListReceiptsQuery {
    pub status: Option<String>,
    pub customer_id: Option<Uuid>,
}

/// List receipts
pub async fn list_receipts(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListReceiptsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.accounts_receivable_engine.list_receipts(
        org_id,
        query.status.as_deref(),
        query.customer_id,
    ).await {
        Ok(receipts) => Ok(Json(serde_json::json!({ "data": receipts }))),
        Err(e) => {
            error!("Failed to list receipts: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Confirm a receipt
pub async fn confirm_receipt(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.accounts_receivable_engine.confirm_receipt(id).await {
        Ok(receipt) => Ok(Json(serde_json::to_value(receipt).unwrap())),
        Err(e) => {
            error!("Failed to confirm receipt: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

/// Apply a receipt to a transaction
pub async fn apply_receipt(
    State(state): State<Arc<AppState>>,
    Path((receipt_id, transaction_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.accounts_receivable_engine.apply_receipt(receipt_id, transaction_id).await {
        Ok(receipt) => Ok(Json(serde_json::to_value(receipt).unwrap())),
        Err(e) => {
            error!("Failed to apply receipt: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

/// Reverse a receipt
pub async fn reverse_receipt(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.accounts_receivable_engine.reverse_receipt(id).await {
        Ok(receipt) => Ok(Json(serde_json::to_value(receipt).unwrap())),
        Err(e) => {
            error!("Failed to reverse receipt: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

// ============================================================================
// Credit Memo Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateCreditMemoRequest {
    pub customer_id: Uuid,
    pub customer_number: Option<String>,
    pub customer_name: Option<String>,
    pub transaction_id: Option<Uuid>,
    pub transaction_number: Option<String>,
    pub credit_memo_date: chrono::NaiveDate,
    pub reason_code: String,
    pub reason_description: Option<String>,
    pub amount: String,
    #[serde(default = "default_zero")]
    pub tax_amount: String,
    pub notes: Option<String>,
}

/// Create a credit memo
pub async fn create_credit_memo(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateCreditMemoRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.accounts_receivable_engine.create_credit_memo(
        org_id,
        payload.customer_id,
        payload.customer_number.as_deref(),
        payload.customer_name.as_deref(),
        payload.transaction_id,
        payload.transaction_number.as_deref(),
        payload.credit_memo_date,
        &payload.reason_code,
        payload.reason_description.as_deref(),
        &payload.amount,
        &payload.tax_amount,
        payload.notes.as_deref(),
        Some(user_id),
    ).await {
        Ok(memo) => Ok((StatusCode::CREATED, Json(serde_json::to_value(memo).unwrap()))),
        Err(e) => {
            error!("Failed to create credit memo: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

/// Approve a credit memo
pub async fn approve_credit_memo(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.accounts_receivable_engine.approve_credit_memo(id).await {
        Ok(memo) => Ok(Json(serde_json::to_value(memo).unwrap())),
        Err(e) => {
            error!("Failed to approve credit memo: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

/// Apply a credit memo to a transaction
pub async fn apply_credit_memo(
    State(state): State<Arc<AppState>>,
    Path((memo_id, transaction_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.accounts_receivable_engine.apply_credit_memo(memo_id, transaction_id).await {
        Ok(memo) => Ok(Json(serde_json::to_value(memo).unwrap())),
        Err(e) => {
            error!("Failed to apply credit memo: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

// ============================================================================
// AR Aging Handler
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ArAgingQuery {
    pub as_of_date: chrono::NaiveDate,
}

/// Get AR aging summary
pub async fn get_ar_aging(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ArAgingQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.accounts_receivable_engine.get_aging_summary(org_id, query.as_of_date).await {
        Ok(summary) => Ok(Json(serde_json::to_value(summary).unwrap())),
        Err(e) => {
            error!("Failed to get AR aging: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
