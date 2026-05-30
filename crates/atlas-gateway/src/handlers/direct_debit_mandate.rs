//! Direct Debit Mandate Management Handlers
//!
//! Oracle Fusion: Financials > Receivables > Direct Debit Mandates

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
// Mandates
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateMandateRequest {
    pub mandate_number: String,
    pub customer_id: Uuid,
    pub customer_name: Option<String>,
    pub customer_account_number: Option<String>,
    pub mandate_type: Option<String>,
    pub bank_account_holder: Option<String>,
    pub bank_account_number: String,
    pub bank_account_number_type: Option<String>,
    pub bank_code: String,
    pub bank_code_type: Option<String>,
    pub bank_name: Option<String>,
    pub bank_branch: Option<String>,
    pub creditor_scheme_id: Option<String>,
    pub creditor_name: Option<String>,
    pub mandate_reference: Option<String>,
    pub mandate_date: Option<chrono::NaiveDate>,
    pub expiry_date: Option<chrono::NaiveDate>,
    pub max_collection_amount: Option<String>,
    pub currency_code: Option<String>,
    pub authorization_reference: Option<String>,
    pub authorization_method: Option<String>,
    pub notes: Option<String>,
}

pub async fn create_mandate(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateMandateRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.financials.direct_debit_mandate_engine.create_mandate(
        org_id, &payload.mandate_number, payload.customer_id,
        payload.customer_name.as_deref(), payload.customer_account_number.as_deref(),
        payload.mandate_type.as_deref().unwrap_or("core"),
        payload.bank_account_holder.as_deref(),
        Some(&payload.bank_account_number),
        payload.bank_account_number_type.as_deref(),
        Some(&payload.bank_code),
        payload.bank_code_type.as_deref(),
        payload.bank_name.as_deref(), payload.bank_branch.as_deref(),
        payload.creditor_scheme_id.as_deref(), payload.creditor_name.as_deref(),
        payload.mandate_reference.as_deref(),
        payload.mandate_date.unwrap_or_else(|| chrono::Utc::now().date_naive()),
        payload.expiry_date, payload.max_collection_amount.as_deref(),
        payload.currency_code.as_deref().unwrap_or("USD"),
        payload.authorization_reference.as_deref(),
        payload.authorization_method.as_deref(),
        payload.notes.as_deref(), Some(user_id),
    ).await {
        Ok(mandate) => Ok(created_json(mandate)),
        Err(e) => {
            error!("Failed to create direct debit mandate: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListMandatesQuery {
    pub customer_id: Option<Uuid>,
    pub status: Option<String>,
}

pub async fn list_mandates(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListMandatesQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.financials.direct_debit_mandate_engine.list_mandates(org_id, query.customer_id, query.status.as_deref()).await {
        Ok(mandates) => Ok(Json(serde_json::json!({ "data": mandates }))),
        Err(e) => {
            error!("Failed to list mandates: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn get_mandate(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.financials.direct_debit_mandate_engine.get_mandate(id).await {
        Ok(Some(m)) => Ok(to_json(m)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Failed to get mandate: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn activate_mandate(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.financials.direct_debit_mandate_engine.activate_mandate(id).await {
        Ok(m) => Ok(to_json(m)),
        Err(e) => {
            error!("Failed to activate mandate: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CancelMandateRequest {
    pub reason: String,
}

pub async fn cancel_mandate(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<CancelMandateRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.financials.direct_debit_mandate_engine.cancel_mandate(id, &payload.reason).await {
        Ok(m) => Ok(to_json(m)),
        Err(e) => {
            error!("Failed to cancel mandate: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) | atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct RevokeMandateRequest {
    pub reason: String,
}

pub async fn revoke_mandate(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<RevokeMandateRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.financials.direct_debit_mandate_engine.revoke_mandate(id, &payload.reason).await {
        Ok(m) => Ok(to_json(m)),
        Err(e) => {
            error!("Failed to revoke mandate: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) | atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// ============================================================================
// Collections
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateCollectionRequest {
    pub mandate_id: Uuid,
    pub collection_number: String,
    pub collection_type: Option<String>,
    pub amount: String,
    pub currency_code: Option<String>,
    pub invoice_id: Option<Uuid>,
    pub invoice_number: Option<String>,
    pub receipt_id: Option<Uuid>,
    pub scheduled_date: Option<chrono::NaiveDate>,
    pub notes: Option<String>,
}

pub async fn create_collection(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateCollectionRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.financials.direct_debit_mandate_engine.create_collection(
        org_id, payload.mandate_id, &payload.collection_number,
        payload.collection_type.as_deref().unwrap_or("recurring"),
        &payload.amount,
        payload.currency_code.as_deref().unwrap_or("USD"),
        payload.invoice_id, payload.invoice_number.as_deref(),
        payload.receipt_id,
        payload.scheduled_date.unwrap_or_else(|| chrono::Utc::now().date_naive()),
        payload.notes.as_deref(), Some(user_id),
    ).await {
        Ok(collection) => Ok(created_json(collection)),
        Err(e) => {
            error!("Failed to create collection: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) | atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn get_collection(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.financials.direct_debit_mandate_engine.get_collection(id).await {
        Ok(Some(c)) => Ok(to_json(c)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Failed to get collection: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListCollectionsQuery {
    pub status: Option<String>,
}

pub async fn list_collections(
    State(state): State<Arc<AppState>>,
    Path(mandate_id): Path<Uuid>,
    Query(query): Query<ListCollectionsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.financials.direct_debit_mandate_engine.list_collections(
        mandate_id, query.status.as_deref(),
    ).await {
        Ok(collections) => Ok(Json(serde_json::json!({ "data": collections }))),
        Err(e) => {
            error!("Failed to list collections: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn submit_collection(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.financials.direct_debit_mandate_engine.submit_collection(id).await {
        Ok(c) => Ok(to_json(c)),
        Err(e) => {
            error!("Failed to submit collection: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CompleteCollectionRequest {
    pub bank_reference: Option<String>,
}

pub async fn complete_collection(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<CompleteCollectionRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.financials.direct_debit_mandate_engine.complete_collection(id, payload.bank_reference.as_deref()).await {
        Ok(c) => Ok(to_json(c)),
        Err(e) => {
            error!("Failed to complete collection: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct FailCollectionRequest {
    pub reason_code: String,
    pub reason_text: Option<String>,
}

pub async fn fail_collection(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<FailCollectionRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.financials.direct_debit_mandate_engine.fail_collection(id, &payload.reason_code, payload.reason_text.as_deref()).await {
        Ok(c) => Ok(to_json(c)),
        Err(e) => {
            error!("Failed to fail collection: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ReturnCollectionRequest {
    pub reason_code: String,
    pub reason_text: Option<String>,
}

pub async fn return_collection(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<ReturnCollectionRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.financials.direct_debit_mandate_engine.return_collection(id, &payload.reason_code, payload.reason_text.as_deref()).await {
        Ok(c) => Ok(to_json(c)),
        Err(e) => {
            error!("Failed to return collection: {}", e);
            Err(match e {
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ReverseCollectionRequest {
    pub reason_code: String,
    pub reason_text: Option<String>,
}

pub async fn reverse_collection(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<ReverseCollectionRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.financials.direct_debit_mandate_engine.reverse_collection(id, &payload.reason_code, payload.reason_text.as_deref()).await {
        Ok(c) => Ok(to_json(c)),
        Err(e) => {
            error!("Failed to reverse collection: {}", e);
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
    match state.financials.direct_debit_mandate_engine.get_dashboard(org_id).await {
        Ok(dashboard) => Ok(to_json(dashboard)),
        Err(e) => { error!("Failed to get dashboard: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}
