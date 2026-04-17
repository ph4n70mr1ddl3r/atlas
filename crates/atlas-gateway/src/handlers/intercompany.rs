//! Intercompany Transaction Handlers
//!
//! Oracle Fusion Cloud ERP: Intercompany > Intercompany Transactions
//!
//! API endpoints for managing intercompany batches, transactions,
//! settlements, and balance monitoring between legal entities.

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
// Batch Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateIntercompanyBatchRequest {
    pub batch_number: String,
    pub description: Option<String>,
    pub from_entity_id: Uuid,
    pub from_entity_name: String,
    pub to_entity_id: Uuid,
    pub to_entity_name: String,
    #[serde(default = "default_currency")]
    pub currency_code: String,
    pub accounting_date: Option<chrono::NaiveDate>,
}

fn default_currency() -> String { "USD".to_string() }

/// Create a new intercompany batch
pub async fn create_intercompany_batch(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateIntercompanyBatchRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    info!("Creating intercompany batch {} for org {}", payload.batch_number, org_id);

    match state.intercompany_engine.create_batch(
        org_id,
        &payload.batch_number,
        payload.description.as_deref(),
        payload.from_entity_id,
        &payload.from_entity_name,
        payload.to_entity_id,
        &payload.to_entity_name,
        &payload.currency_code,
        payload.accounting_date,
        Some(user_id),
    ).await {
        Ok(batch) => Ok((StatusCode::CREATED, Json(serde_json::to_value(batch).unwrap()))),
        Err(e) => {
            error!("Failed to create intercompany batch: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

/// List intercompany batches
pub async fn list_intercompany_batches(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<ListBatchesQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.intercompany_engine.list_batches(org_id, params.status.as_deref()).await {
        Ok(batches) => Ok(Json(serde_json::json!({ "data": batches }))),
        Err(e) => {
            error!("Failed to list intercompany batches: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListBatchesQuery {
    pub status: Option<String>,
}

/// Get a specific intercompany batch
pub async fn get_intercompany_batch(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(batch_number): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.intercompany_engine.get_batch(org_id, &batch_number).await {
        Ok(Some(batch)) => Ok(Json(serde_json::to_value(batch).unwrap())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get intercompany batch: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Submit a batch for approval
pub async fn submit_intercompany_batch(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(batch_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.intercompany_engine.submit_batch(batch_id, Some(user_id)).await {
        Ok(batch) => Ok(Json(serde_json::to_value(batch).unwrap())),
        Err(e) => {
            error!("Failed to submit intercompany batch: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

/// Approve a batch
pub async fn approve_intercompany_batch(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(batch_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.intercompany_engine.approve_batch(batch_id, user_id).await {
        Ok(batch) => Ok(Json(serde_json::to_value(batch).unwrap())),
        Err(e) => {
            error!("Failed to approve intercompany batch: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

/// Post a batch (creates journal entries)
pub async fn post_intercompany_batch(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(batch_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.intercompany_engine.post_batch(batch_id).await {
        Ok(batch) => Ok(Json(serde_json::to_value(batch).unwrap())),
        Err(e) => {
            error!("Failed to post intercompany batch: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

/// Reject a batch
pub async fn reject_intercompany_batch(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(batch_id): Path<Uuid>,
    Json(payload): Json<RejectBatchRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.intercompany_engine.reject_batch(batch_id, &payload.reason).await {
        Ok(batch) => Ok(Json(serde_json::to_value(batch).unwrap())),
        Err(e) => {
            error!("Failed to reject intercompany batch: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct RejectBatchRequest {
    pub reason: String,
}

// ============================================================================
// Transaction Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateIntercompanyTransactionRequest {
    pub batch_number: String,
    pub transaction_number: String,
    #[serde(default = "default_txn_type")]
    pub transaction_type: String,
    pub description: Option<String>,
    pub from_entity_id: Uuid,
    pub from_entity_name: String,
    pub to_entity_id: Uuid,
    pub to_entity_name: String,
    pub amount: String,
    #[serde(default = "default_currency")]
    pub currency_code: String,
    pub exchange_rate: Option<String>,
    pub from_debit_account: Option<String>,
    pub from_credit_account: Option<String>,
    pub to_debit_account: Option<String>,
    pub to_credit_account: Option<String>,
    pub from_ic_account: Option<String>,
    pub to_ic_account: Option<String>,
    pub transaction_date: Option<chrono::NaiveDate>,
    pub due_date: Option<chrono::NaiveDate>,
    pub source_entity_type: Option<String>,
    pub source_entity_id: Option<Uuid>,
}

fn default_txn_type() -> String { "invoice".to_string() }

/// Create an intercompany transaction
pub async fn create_intercompany_transaction(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateIntercompanyTransactionRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let txn_date = payload.transaction_date.unwrap_or_else(|| chrono::Utc::now().date_naive());

    match state.intercompany_engine.create_transaction(
        org_id,
        &payload.batch_number,
        &payload.transaction_number,
        &payload.transaction_type,
        payload.description.as_deref(),
        payload.from_entity_id,
        &payload.from_entity_name,
        payload.to_entity_id,
        &payload.to_entity_name,
        &payload.amount,
        &payload.currency_code,
        payload.exchange_rate.as_deref(),
        payload.from_debit_account.as_deref(),
        payload.from_credit_account.as_deref(),
        payload.to_debit_account.as_deref(),
        payload.to_credit_account.as_deref(),
        payload.from_ic_account.as_deref(),
        payload.to_ic_account.as_deref(),
        txn_date,
        payload.due_date,
        payload.source_entity_type.as_deref(),
        payload.source_entity_id,
        Some(user_id),
    ).await {
        Ok(txn) => Ok((StatusCode::CREATED, Json(serde_json::to_value(txn).unwrap()))),
        Err(e) => {
            error!("Failed to create intercompany transaction: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

/// List transactions in a batch
pub async fn list_intercompany_transactions(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(batch_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.intercompany_engine.list_transactions_by_batch(batch_id).await {
        Ok(transactions) => Ok(Json(serde_json::json!({ "data": transactions }))),
        Err(e) => {
            error!("Failed to list intercompany transactions: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// List transactions for an entity
pub async fn list_entity_transactions(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(entity_id): Path<Uuid>,
    Query(params): Query<ListEntityTransactionsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.intercompany_engine.list_transactions_by_entity(
        org_id, entity_id, params.status.as_deref(),
    ).await {
        Ok(transactions) => Ok(Json(serde_json::json!({ "data": transactions }))),
        Err(e) => {
            error!("Failed to list entity transactions: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListEntityTransactionsQuery {
    pub status: Option<String>,
}

// ============================================================================
// Settlement Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateIntercompanySettlementRequest {
    pub settlement_number: String,
    #[serde(default = "default_settlement_method")]
    pub settlement_method: String,
    pub from_entity_id: Uuid,
    pub to_entity_id: Uuid,
    pub settled_amount: String,
    #[serde(default = "default_currency")]
    pub currency_code: String,
    pub payment_reference: Option<String>,
    pub transaction_ids: Option<Vec<Uuid>>,
}

fn default_settlement_method() -> String { "cash".to_string() }

/// Create an intercompany settlement
pub async fn create_intercompany_settlement(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateIntercompanySettlementRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let txn_ids = payload.transaction_ids.unwrap_or_default();

    match state.intercompany_engine.create_settlement(
        org_id,
        &payload.settlement_number,
        &payload.settlement_method,
        payload.from_entity_id,
        payload.to_entity_id,
        &payload.settled_amount,
        &payload.currency_code,
        payload.payment_reference.as_deref(),
        &txn_ids,
        Some(user_id),
    ).await {
        Ok(settlement) => Ok((StatusCode::CREATED, Json(serde_json::to_value(settlement).unwrap()))),
        Err(e) => {
            error!("Failed to create intercompany settlement: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

/// List intercompany settlements
pub async fn list_intercompany_settlements(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<ListSettlementsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.intercompany_engine.list_settlements(
        org_id, params.entity_id,
    ).await {
        Ok(settlements) => Ok(Json(serde_json::json!({ "data": settlements }))),
        Err(e) => {
            error!("Failed to list intercompany settlements: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListSettlementsQuery {
    pub entity_id: Option<Uuid>,
}

// ============================================================================
// Balance Handlers
// ============================================================================

/// Get intercompany balance summary (dashboard)
pub async fn get_intercompany_balance_summary(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.intercompany_engine.get_balance_summary(org_id).await {
        Ok(summary) => Ok(Json(serde_json::to_value(summary).unwrap())),
        Err(e) => {
            error!("Failed to get intercompany balance summary: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get balance for a specific entity pair
pub async fn get_intercompany_balance(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path((from_entity_id, to_entity_id)): Path<(Uuid, Uuid)>,
    Query(params): Query<GetBalanceQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let currency = params.currency_code.unwrap_or_else(|| "USD".to_string());

    match state.intercompany_engine.get_balance(
        org_id, from_entity_id, to_entity_id, &currency,
    ).await {
        Ok(Some(balance)) => Ok(Json(serde_json::to_value(balance).unwrap())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get intercompany balance: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct GetBalanceQuery {
    pub currency_code: Option<String>,
}
