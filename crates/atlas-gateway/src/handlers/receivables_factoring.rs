//! Receivables Factoring Handlers
//!
//! Oracle Fusion: Financials > Treasury > Receivables Factoring
//! Manages the sale of accounts receivable to third-party factors at a discount
//! for immediate cash. Supports recourse and non-recourse factoring.

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
// Factor Companies
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateFactorCompanyRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub contact_name: Option<String>,
    pub contact_email: Option<String>,
    pub contact_phone: Option<String>,
    pub bank_name: Option<String>,
    pub bank_account_number: Option<String>,
    pub advance_rate: Option<String>,
    pub fee_rate: Option<String>,
    pub recourse_type: Option<String>,
    pub minimum_invoice_amount: Option<String>,
    pub maximum_invoice_amount: Option<String>,
}

pub async fn create_factor_company(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateFactorCompanyRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.receivables_factoring_engine.create_factor_company(
        org_id, &payload.code, &payload.name, payload.description.as_deref(),
        payload.contact_name.as_deref(), payload.contact_email.as_deref(), payload.contact_phone.as_deref(),
        payload.bank_name.as_deref(), payload.bank_account_number.as_deref(),
        payload.advance_rate.as_deref().unwrap_or("0.8000"),
        payload.fee_rate.as_deref().unwrap_or("0.0150"),
        payload.recourse_type.as_deref().unwrap_or("recourse"),
        payload.minimum_invoice_amount.as_deref(),
        payload.maximum_invoice_amount.as_deref(),
        Some(user_id),
    ).await {
        Ok(fc) => Ok((StatusCode::CREATED, Json(crate::handlers::records::to_json_or_null(fc)))),
        Err(e) => {
            error!("Failed to create factor company: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(msg) => {
                    return Ok((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": msg}))));
                }
                atlas_shared::AtlasError::Conflict(msg) => {
                    return Ok((StatusCode::CONFLICT, Json(serde_json::json!({"error": msg}))));
                }
                atlas_shared::AtlasError::DatabaseError(msg) => {
                    return Ok((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": msg}))));
                }
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListFactorCompaniesQuery {
    pub is_active: Option<bool>,
}

pub async fn list_factor_companies(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListFactorCompaniesQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.receivables_factoring_engine.list_factor_companies(org_id, query.is_active).await {
        Ok(companies) => Ok(Json(serde_json::json!({ "data": companies }))),
        Err(e) => {
            error!("Failed to list factor companies: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_factor_company(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.receivables_factoring_engine.get_factor_company(id).await {
        Ok(Some(fc)) => Ok(Json(crate::handlers::records::to_json_or_null(fc))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get factor company: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_factor_company_by_code(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(code): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.receivables_factoring_engine.get_factor_company_by_code(org_id, &code).await {
        Ok(Some(fc)) => Ok(Json(crate::handlers::records::to_json_or_null(fc))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get factor company by code: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn deactivate_factor_company(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.receivables_factoring_engine.deactivate_factor_company(id).await {
        Ok(fc) => Ok(Json(crate::handlers::records::to_json_or_null(fc))),
        Err(e) => {
            error!("Failed to deactivate factor company: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn activate_factor_company(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.receivables_factoring_engine.activate_factor_company(id).await {
        Ok(fc) => Ok(Json(crate::handlers::records::to_json_or_null(fc))),
        Err(e) => {
            error!("Failed to activate factor company: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// ============================================================================
// Factoring Agreements
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateAgreementRequest {
    pub agreement_number: String,
    pub factor_company_id: Uuid,
    pub agreement_name: String,
    pub description: Option<String>,
    pub agreement_type: Option<String>,
    pub recourse_type: Option<String>,
    pub advance_rate: Option<String>,
    pub factoring_fee_rate: Option<String>,
    pub late_fee_rate: Option<String>,
    pub reserve_rate: Option<String>,
    pub minimum_fee: Option<String>,
    pub currency_code: Option<String>,
    pub start_date: chrono::NaiveDate,
    pub end_date: Option<chrono::NaiveDate>,
    pub credit_limit: Option<String>,
}

pub async fn create_agreement(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateAgreementRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.receivables_factoring_engine.create_agreement(
        org_id, &payload.agreement_number, payload.factor_company_id,
        &payload.agreement_name, payload.description.as_deref(),
        payload.agreement_type.as_deref().unwrap_or("spot"),
        payload.recourse_type.as_deref().unwrap_or("recourse"),
        payload.advance_rate.as_deref().unwrap_or("0.8000"),
        payload.factoring_fee_rate.as_deref().unwrap_or("0.0150"),
        payload.late_fee_rate.as_deref().unwrap_or("0.0050"),
        payload.reserve_rate.as_deref().unwrap_or("0.0500"),
        payload.minimum_fee.as_deref().unwrap_or("0.00"),
        payload.currency_code.as_deref().unwrap_or("USD"),
        payload.start_date, payload.end_date,
        payload.credit_limit.as_deref(),
        Some(user_id),
    ).await {
        Ok(a) => Ok((StatusCode::CREATED, Json(crate::handlers::records::to_json_or_null(a)))),
        Err(e) => {
            error!("Failed to create factoring agreement: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListAgreementsQuery {
    pub status: Option<String>,
    pub factor_company_id: Option<Uuid>,
}

pub async fn list_agreements(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListAgreementsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.receivables_factoring_engine.list_agreements(
        org_id, query.status.as_deref(), query.factor_company_id,
    ).await {
        Ok(agreements) => Ok(Json(serde_json::json!({ "data": agreements }))),
        Err(e) => {
            error!("Failed to list factoring agreements: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_agreement(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.receivables_factoring_engine.get_agreement(id).await {
        Ok(Some(a)) => Ok(Json(crate::handlers::records::to_json_or_null(a))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get factoring agreement: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn activate_agreement(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.receivables_factoring_engine.activate_agreement(id, Some(user_id)).await {
        Ok(a) => Ok(Json(crate::handlers::records::to_json_or_null(a))),
        Err(e) => {
            error!("Failed to activate agreement: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn suspend_agreement(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.receivables_factoring_engine.suspend_agreement(id).await {
        Ok(a) => Ok(Json(crate::handlers::records::to_json_or_null(a))),
        Err(e) => {
            error!("Failed to suspend agreement: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn terminate_agreement(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.receivables_factoring_engine.terminate_agreement(id).await {
        Ok(a) => Ok(Json(crate::handlers::records::to_json_or_null(a))),
        Err(e) => {
            error!("Failed to terminate agreement: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// ============================================================================
// Factoring Requests
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateFactoringRequest {
    pub request_number: String,
    pub agreement_id: Uuid,
    pub request_date: chrono::NaiveDate,
    pub recourse_type: Option<String>,
    pub currency_code: Option<String>,
    pub notes: Option<String>,
}

pub async fn create_factoring_request(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateFactoringRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.receivables_factoring_engine.create_request(
        org_id, &payload.request_number, payload.agreement_id,
        payload.request_date,
        payload.recourse_type.as_deref().unwrap_or("recourse"),
        payload.currency_code.as_deref().unwrap_or("USD"),
        payload.notes.as_deref(), Some(user_id),
    ).await {
        Ok(r) => Ok((StatusCode::CREATED, Json(crate::handlers::records::to_json_or_null(r)))),
        Err(e) => {
            error!("Failed to create factoring request: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListFactoringRequestsQuery {
    pub agreement_id: Option<Uuid>,
    pub status: Option<String>,
}

pub async fn list_factoring_requests(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListFactoringRequestsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.receivables_factoring_engine.list_requests(
        org_id, query.agreement_id, query.status.as_deref(),
    ).await {
        Ok(requests) => Ok(Json(serde_json::json!({ "data": requests }))),
        Err(e) => {
            error!("Failed to list factoring requests: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_factoring_request(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.receivables_factoring_engine.get_request(id).await {
        Ok(Some(r)) => Ok(Json(crate::handlers::records::to_json_or_null(r))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get factoring request: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn submit_factoring_request(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.receivables_factoring_engine.submit_request(id).await {
        Ok(r) => Ok(Json(crate::handlers::records::to_json_or_null(r))),
        Err(e) => {
            error!("Failed to submit factoring request: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn approve_factoring_request(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.receivables_factoring_engine.approve_request(id).await {
        Ok(r) => Ok(Json(crate::handlers::records::to_json_or_null(r))),
        Err(e) => {
            error!("Failed to approve factoring request: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn fund_factoring_request(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.receivables_factoring_engine.fund_request(id).await {
        Ok(r) => Ok(Json(crate::handlers::records::to_json_or_null(r))),
        Err(e) => {
            error!("Failed to fund factoring request: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn settle_factoring_request(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.receivables_factoring_engine.settle_request(id).await {
        Ok(r) => Ok(Json(crate::handlers::records::to_json_or_null(r))),
        Err(e) => {
            error!("Failed to settle factoring request: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn cancel_factoring_request(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.receivables_factoring_engine.cancel_request(id).await {
        Ok(r) => Ok(Json(crate::handlers::records::to_json_or_null(r))),
        Err(e) => {
            error!("Failed to cancel factoring request: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// ============================================================================
// Factoring Request Lines
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct AddRequestLineRequest {
    pub line_number: i32,
    pub transaction_id: Option<Uuid>,
    pub transaction_number: Option<String>,
    pub customer_id: Option<Uuid>,
    pub customer_number: Option<String>,
    pub customer_name: Option<String>,
    pub invoice_date: Option<chrono::NaiveDate>,
    pub invoice_due_date: Option<chrono::NaiveDate>,
    pub invoice_amount: String,
    pub eligible_amount: String,
    pub days_outstanding: Option<i32>,
    pub days_overdue: Option<i32>,
}

pub async fn add_request_line(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(request_id): Path<Uuid>,
    Json(payload): Json<AddRequestLineRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.receivables_factoring_engine.add_request_line(
        org_id, request_id, payload.line_number,
        payload.transaction_id, payload.transaction_number.as_deref(),
        payload.customer_id, payload.customer_number.as_deref(), payload.customer_name.as_deref(),
        payload.invoice_date, payload.invoice_due_date,
        &payload.invoice_amount, &payload.eligible_amount,
        payload.days_outstanding, payload.days_overdue,
    ).await {
        Ok(line) => Ok((StatusCode::CREATED, Json(crate::handlers::records::to_json_or_null(line)))),
        Err(e) => {
            error!("Failed to add request line: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn list_request_lines(
    State(state): State<Arc<AppState>>,
    Path(request_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.receivables_factoring_engine.list_request_lines(request_id).await {
        Ok(lines) => Ok(Json(serde_json::json!({ "data": lines }))),
        Err(e) => {
            error!("Failed to list request lines: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// ============================================================================
// Factoring Settlements
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateSettlementRequest {
    pub settlement_number: String,
    pub agreement_id: Uuid,
    pub request_id: Option<Uuid>,
    pub settlement_date: chrono::NaiveDate,
    pub currency_code: Option<String>,
    pub notes: Option<String>,
}

pub async fn create_settlement(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateSettlementRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.receivables_factoring_engine.create_settlement(
        org_id, &payload.settlement_number, payload.agreement_id,
        payload.request_id, payload.settlement_date,
        payload.currency_code.as_deref().unwrap_or("USD"),
        payload.notes.as_deref(), Some(user_id),
    ).await {
        Ok(s) => Ok((StatusCode::CREATED, Json(crate::handlers::records::to_json_or_null(s)))),
        Err(e) => {
            error!("Failed to create settlement: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn list_settlements(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListSettlementsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.receivables_factoring_engine.list_settlements(
        org_id, query.agreement_id, query.status.as_deref(),
    ).await {
        Ok(settlements) => Ok(Json(serde_json::json!({ "data": settlements }))),
        Err(e) => {
            error!("Failed to list settlements: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListSettlementsQuery {
    pub agreement_id: Option<Uuid>,
    pub status: Option<String>,
}

pub async fn process_settlement(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.receivables_factoring_engine.process_settlement(id, Some(user_id)).await {
        Ok(s) => Ok(Json(crate::handlers::records::to_json_or_null(s))),
        Err(e) => {
            error!("Failed to process settlement: {}", e);
            Err(match e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// ============================================================================
// Dashboard
// ============================================================================

pub async fn get_factoring_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.receivables_factoring_engine.get_dashboard(org_id).await {
        Ok(dashboard) => Ok(Json(crate::handlers::records::to_json_or_null(dashboard))),
        Err(e) => {
            error!("Failed to get factoring dashboard: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
