//! Corporate Card Management API Handlers
//!
//! Oracle Fusion Cloud ERP: Financials > Expenses > Corporate Cards
//!
//! Endpoints for managing corporate card programmes, card issuance,
//! transaction import, expense matching, statements, spending limit
//! overrides, and dispute handling.

use axum::{
    extract::{Path, Query, State, Extension},
    Json,
    http::StatusCode,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;
use uuid::Uuid;

use crate::AppState;
use crate::handlers::auth::Claims;

// ============================================================================
// Query Parameters
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ListProgramsQuery {
    pub active_only: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ListCardsQuery {
    pub program_id: Option<String>,
    pub cardholder_id: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListTransactionsQuery {
    pub card_id: Option<String>,
    pub status: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListStatementsQuery {
    pub program_id: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListOverridesQuery {
    pub card_id: Option<String>,
    pub status: Option<String>,
}

// ============================================================================
// Card Programme CRUD
// ============================================================================

/// Create a new corporate card programme
pub async fn create_program(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"}))))?;
    let created_by = Uuid::parse_str(&claims.sub).ok();

    let program_code = body["program_code"].as_str().unwrap_or("");
    let name = body["name"].as_str().unwrap_or("");
    let issuer_bank = body["issuer_bank"].as_str().unwrap_or("");
    let card_network = body["card_network"].as_str().unwrap_or("Visa");
    let card_type = body["card_type"].as_str().unwrap_or("corporate");
    let currency_code = body["currency_code"].as_str().unwrap_or("USD");
    let single_limit = body["default_single_purchase_limit"].as_str().unwrap_or("5000.00");
    let monthly_limit = body["default_monthly_limit"].as_str().unwrap_or("20000.00");
    let cash_limit = body["default_cash_limit"].as_str().unwrap_or("1000.00");
    let atm_limit = body["default_atm_limit"].as_str().unwrap_or("500.00");
    let allow_cash = body["allow_cash_withdrawal"].as_bool().unwrap_or(false);
    let allow_intl = body["allow_international"].as_bool().unwrap_or(true);
    let auto_deactivate = body["auto_deactivate_on_termination"].as_bool().unwrap_or(true);
    let matching_method = body["expense_matching_method"].as_str().unwrap_or("auto");
    let billing_day: i32 = body["billing_cycle_day"].as_i64().unwrap_or(1) as i32;
    let description = body["description"].as_str();

    match state.corporate_card_engine.create_program(
        org_id, program_code, name, description,
        issuer_bank, card_network, card_type, currency_code,
        single_limit, monthly_limit, cash_limit, atm_limit,
        allow_cash, allow_intl, auto_deactivate, matching_method,
        billing_day, created_by,
    ).await {
        Ok(program) => Ok((StatusCode::CREATED, Json(serde_json::to_value(program).unwrap()))),
        Err(e) => {
            let status = match &e {
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status, Json(json!({"error": e.to_string()}))))
        }
    }
}

/// Get a programme by code
pub async fn get_program(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(code): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"}))))?;

    match state.corporate_card_engine.get_program(org_id, &code).await {
        Ok(Some(p)) => Ok(Json(serde_json::to_value(p).unwrap())),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(json!({"error": "Program not found"})))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))),
    }
}

/// List programmes
pub async fn list_programs(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<ListProgramsQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"}))))?;
    let active_only = query.active_only.unwrap_or(false);

    match state.corporate_card_engine.list_programs(org_id, active_only).await {
        Ok(programs) => Ok(Json(json!({"data": programs}))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Card Management
// ============================================================================

/// Issue a new card to an employee
pub async fn issue_card(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"}))))?;
    let created_by = Uuid::parse_str(&claims.sub).ok();

    let program_id = body["program_id"].as_str()
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| (StatusCode::BAD_REQUEST, Json(json!({"error": "program_id is required"}))))?;
    let card_number_masked = body["card_number_masked"].as_str().unwrap_or("");
    let cardholder_name = body["cardholder_name"].as_str().unwrap_or("");
    let cardholder_id = body["cardholder_id"].as_str()
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| (StatusCode::BAD_REQUEST, Json(json!({"error": "cardholder_id is required"}))))?;
    let cardholder_email = body["cardholder_email"].as_str();
    let department_id = body["department_id"].as_str().and_then(|s| Uuid::parse_str(s).ok());
    let department_name = body["department_name"].as_str();
    let issue_date = body["issue_date"].as_str()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .ok_or_else(|| (StatusCode::BAD_REQUEST, Json(json!({"error": "issue_date is required (YYYY-MM-DD)"}))))?;
    let expiry_date = body["expiry_date"].as_str()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .ok_or_else(|| (StatusCode::BAD_REQUEST, Json(json!({"error": "expiry_date is required (YYYY-MM-DD)"}))))?;
    let gl_liability = body["gl_liability_account"].as_str();
    let gl_expense = body["gl_expense_account"].as_str();
    let cost_center = body["cost_center"].as_str();

    match state.corporate_card_engine.issue_card(
        org_id, program_id, card_number_masked, cardholder_name,
        cardholder_id, cardholder_email, department_id, department_name,
        issue_date, expiry_date, gl_liability, gl_expense,
        cost_center, created_by,
    ).await {
        Ok(card) => Ok((StatusCode::CREATED, Json(serde_json::to_value(card).unwrap()))),
        Err(e) => {
            let status = match &e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status, Json(json!({"error": e.to_string()}))))
        }
    }
}

/// Get a card by ID
pub async fn get_card(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.corporate_card_engine.get_card(id).await {
        Ok(Some(c)) => Ok(Json(serde_json::to_value(c).unwrap())),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(json!({"error": "Card not found"})))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))),
    }
}

/// List cards
pub async fn list_cards(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<ListCardsQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"}))))?;
    let program_id = query.program_id.as_ref().and_then(|s| Uuid::parse_str(s).ok());
    let cardholder_id = query.cardholder_id.as_ref().and_then(|s| Uuid::parse_str(s).ok());

    match state.corporate_card_engine.list_cards(
        org_id, program_id, cardholder_id, query.status.as_deref(),
    ).await {
        Ok(cards) => Ok(Json(json!({"data": cards}))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))),
    }
}

/// Suspend a card
pub async fn suspend_card(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    match state.corporate_card_engine.suspend_card(id).await {
        Ok(card) => Ok((StatusCode::OK, Json(serde_json::to_value(card).unwrap()))),
        Err(e) => {
            let status = match &e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status, Json(json!({"error": e.to_string()}))))
        }
    }
}

/// Reactivate a suspended card
pub async fn reactivate_card(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    match state.corporate_card_engine.reactivate_card(id).await {
        Ok(card) => Ok((StatusCode::OK, Json(serde_json::to_value(card).unwrap()))),
        Err(e) => {
            let status = match &e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status, Json(json!({"error": e.to_string()}))))
        }
    }
}

/// Cancel a card
pub async fn cancel_card(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    match state.corporate_card_engine.cancel_card(id).await {
        Ok(card) => Ok((StatusCode::OK, Json(serde_json::to_value(card).unwrap()))),
        Err(e) => {
            let status = match &e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status, Json(json!({"error": e.to_string()}))))
        }
    }
}

/// Report card lost
pub async fn report_lost(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    match state.corporate_card_engine.report_lost(id).await {
        Ok(card) => Ok((StatusCode::OK, Json(serde_json::to_value(card).unwrap()))),
        Err(e) => {
            let status = match &e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status, Json(json!({"error": e.to_string()}))))
        }
    }
}

/// Report card stolen
pub async fn report_stolen(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    match state.corporate_card_engine.report_stolen(id).await {
        Ok(card) => Ok((StatusCode::OK, Json(serde_json::to_value(card).unwrap()))),
        Err(e) => {
            let status = match &e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status, Json(json!({"error": e.to_string()}))))
        }
    }
}

// ============================================================================
// Transactions
// ============================================================================

/// Import a card transaction
pub async fn import_transaction(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"}))))?;

    let card_id = body["card_id"].as_str()
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| (StatusCode::BAD_REQUEST, Json(json!({"error": "card_id is required"}))))?;
    let transaction_reference = body["transaction_reference"].as_str().unwrap_or("");
    let posting_date = body["posting_date"].as_str()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .ok_or_else(|| (StatusCode::BAD_REQUEST, Json(json!({"error": "posting_date is required"}))))?;
    let transaction_date = body["transaction_date"].as_str()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .ok_or_else(|| (StatusCode::BAD_REQUEST, Json(json!({"error": "transaction_date is required"}))))?;
    let merchant_name = body["merchant_name"].as_str().unwrap_or("");
    let merchant_category = body["merchant_category"].as_str();
    let merchant_category_code = body["merchant_category_code"].as_str();
    let amount = body["amount"].as_str().unwrap_or("0");
    let currency_code = body["currency_code"].as_str().unwrap_or("USD");
    let original_amount = body["original_amount"].as_str();
    let original_currency = body["original_currency"].as_str();
    let exchange_rate = body["exchange_rate"].as_str();
    let transaction_type = body["transaction_type"].as_str().unwrap_or("charge");

    match state.corporate_card_engine.import_transaction(
        org_id, card_id, transaction_reference, posting_date, transaction_date,
        merchant_name, merchant_category, merchant_category_code, amount,
        currency_code, original_amount, original_currency, exchange_rate,
        transaction_type,
    ).await {
        Ok(txn) => Ok((StatusCode::CREATED, Json(serde_json::to_value(txn).unwrap()))),
        Err(e) => {
            let status = match &e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status, Json(json!({"error": e.to_string()}))))
        }
    }
}

/// Get a transaction by ID
pub async fn get_transaction(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.corporate_card_engine.get_transaction(id).await {
        Ok(Some(t)) => Ok(Json(serde_json::to_value(t).unwrap())),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(json!({"error": "Transaction not found"})))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))),
    }
}

/// List transactions
pub async fn list_transactions(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<ListTransactionsQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"}))))?;
    let card_id = query.card_id.as_ref().and_then(|s| Uuid::parse_str(s).ok());
    let date_from = query.date_from.as_ref().and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let date_to = query.date_to.as_ref().and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());

    match state.corporate_card_engine.list_transactions(
        org_id, card_id, query.status.as_deref(), date_from, date_to,
    ).await {
        Ok(txns) => Ok(Json(json!({"data": txns}))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))),
    }
}

/// Match a transaction to an expense report
pub async fn match_transaction(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let matched_by = Uuid::parse_str(&claims.sub).ok();
    let expense_report_id = body["expense_report_id"].as_str()
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| (StatusCode::BAD_REQUEST, Json(json!({"error": "expense_report_id is required"}))))?;
    let expense_line_id = body["expense_line_id"].as_str().and_then(|s| Uuid::parse_str(s).ok());
    let match_confidence = body["match_confidence"].as_str();

    match state.corporate_card_engine.match_transaction(
        id, expense_report_id, expense_line_id, matched_by, match_confidence,
    ).await {
        Ok(txn) => Ok((StatusCode::OK, Json(serde_json::to_value(txn).unwrap()))),
        Err(e) => {
            let status = match &e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status, Json(json!({"error": e.to_string()}))))
        }
    }
}

/// Unmatch a transaction
pub async fn unmatch_transaction(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    match state.corporate_card_engine.unmatch_transaction(id).await {
        Ok(txn) => Ok((StatusCode::OK, Json(serde_json::to_value(txn).unwrap()))),
        Err(e) => {
            let status = match &e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status, Json(json!({"error": e.to_string()}))))
        }
    }
}

/// Dispute a transaction
pub async fn dispute_transaction(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let reason = body["reason"].as_str().unwrap_or("");

    match state.corporate_card_engine.dispute_transaction(id, reason).await {
        Ok(txn) => Ok((StatusCode::OK, Json(serde_json::to_value(txn).unwrap()))),
        Err(e) => {
            let status = match &e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status, Json(json!({"error": e.to_string()}))))
        }
    }
}

/// Resolve a disputed transaction
pub async fn resolve_dispute(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let resolution = body["resolution"].as_str().unwrap_or("");
    let resolved_status = body["resolved_status"].as_str().unwrap_or("approved");

    match state.corporate_card_engine.resolve_dispute(id, resolution, resolved_status).await {
        Ok(txn) => Ok((StatusCode::OK, Json(serde_json::to_value(txn).unwrap()))),
        Err(e) => {
            let status = match &e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status, Json(json!({"error": e.to_string()}))))
        }
    }
}

// ============================================================================
// Statements
// ============================================================================

/// Import a card statement
pub async fn import_statement(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"}))))?;
    let imported_by = Uuid::parse_str(&claims.sub).ok();

    let program_id = body["program_id"].as_str()
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| (StatusCode::BAD_REQUEST, Json(json!({"error": "program_id is required"}))))?;
    let statement_number = body["statement_number"].as_str().unwrap_or("");
    let statement_date = body["statement_date"].as_str()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .ok_or_else(|| (StatusCode::BAD_REQUEST, Json(json!({"error": "statement_date is required"}))))?;
    let period_start = body["billing_period_start"].as_str()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .ok_or_else(|| (StatusCode::BAD_REQUEST, Json(json!({"error": "billing_period_start is required"}))))?;
    let period_end = body["billing_period_end"].as_str()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .ok_or_else(|| (StatusCode::BAD_REQUEST, Json(json!({"error": "billing_period_end is required"}))))?;
    let opening_balance = body["opening_balance"].as_str().unwrap_or("0");
    let closing_balance = body["closing_balance"].as_str().unwrap_or("0");
    let total_charges = body["total_charges"].as_str().unwrap_or("0");
    let total_credits = body["total_credits"].as_str().unwrap_or("0");
    let total_payments = body["total_payments"].as_str().unwrap_or("0");
    let total_fees = body["total_fees"].as_str().unwrap_or("0");
    let total_interest = body["total_interest"].as_str().unwrap_or("0");
    let payment_due_date = body["payment_due_date"].as_str()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let minimum_payment = body["minimum_payment"].as_str().unwrap_or("0");

    match state.corporate_card_engine.import_statement(
        org_id, program_id, statement_number, statement_date,
        period_start, period_end, opening_balance, closing_balance,
        total_charges, total_credits, total_payments, total_fees, total_interest,
        payment_due_date, minimum_payment, imported_by,
    ).await {
        Ok(stmt) => Ok((StatusCode::CREATED, Json(serde_json::to_value(stmt).unwrap()))),
        Err(e) => {
            let status = match &e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status, Json(json!({"error": e.to_string()}))))
        }
    }
}

/// Get a statement
pub async fn get_statement(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.corporate_card_engine.get_statement(id).await {
        Ok(Some(s)) => Ok(Json(serde_json::to_value(s).unwrap())),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(json!({"error": "Statement not found"})))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))),
    }
}

/// List statements
pub async fn list_statements(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<ListStatementsQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"}))))?;
    let program_id = query.program_id.as_ref().and_then(|s| Uuid::parse_str(s).ok());

    match state.corporate_card_engine.list_statements(
        org_id, program_id, query.status.as_deref(),
    ).await {
        Ok(stmts) => Ok(Json(json!({"data": stmts}))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))),
    }
}

/// Reconcile a statement
pub async fn reconcile_statement(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    match state.corporate_card_engine.reconcile_statement(id).await {
        Ok(stmt) => Ok((StatusCode::OK, Json(serde_json::to_value(stmt).unwrap()))),
        Err(e) => {
            let status = match &e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status, Json(json!({"error": e.to_string()}))))
        }
    }
}

/// Pay a statement
pub async fn pay_statement(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let payment_reference = body["payment_reference"].as_str().unwrap_or("");

    match state.corporate_card_engine.pay_statement(id, payment_reference).await {
        Ok(stmt) => Ok((StatusCode::OK, Json(serde_json::to_value(stmt).unwrap()))),
        Err(e) => {
            let status = match &e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status, Json(json!({"error": e.to_string()}))))
        }
    }
}

// ============================================================================
// Spending Limit Overrides
// ============================================================================

/// Request a spending limit override
pub async fn request_limit_override(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"}))))?;
    let created_by = Uuid::parse_str(&claims.sub).ok();

    let card_id = body["card_id"].as_str()
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| (StatusCode::BAD_REQUEST, Json(json!({"error": "card_id is required"}))))?;
    let override_type = body["override_type"].as_str().unwrap_or("single_purchase");
    let new_value = body["new_value"].as_str().unwrap_or("0");
    let reason = body["reason"].as_str().unwrap_or("");
    let effective_from = body["effective_from"].as_str()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .ok_or_else(|| (StatusCode::BAD_REQUEST, Json(json!({"error": "effective_from is required"}))))?;
    let effective_to = body["effective_to"].as_str()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());

    match state.corporate_card_engine.request_limit_override(
        org_id, card_id, override_type, new_value, reason,
        effective_from, effective_to, created_by,
    ).await {
        Ok(ovr) => Ok((StatusCode::CREATED, Json(serde_json::to_value(ovr).unwrap()))),
        Err(e) => {
            let status = match &e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status, Json(json!({"error": e.to_string()}))))
        }
    }
}

/// Approve a limit override
pub async fn approve_limit_override(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let approved_by = Uuid::parse_str(&claims.sub).unwrap_or_else(|_| Uuid::nil());

    match state.corporate_card_engine.approve_limit_override(id, approved_by).await {
        Ok(ovr) => Ok((StatusCode::OK, Json(serde_json::to_value(ovr).unwrap()))),
        Err(e) => {
            let status = match &e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status, Json(json!({"error": e.to_string()}))))
        }
    }
}

/// Reject a limit override
pub async fn reject_limit_override(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let rejected_by = Uuid::parse_str(&claims.sub).unwrap_or_else(|_| Uuid::nil());

    match state.corporate_card_engine.reject_limit_override(id, rejected_by).await {
        Ok(ovr) => Ok((StatusCode::OK, Json(serde_json::to_value(ovr).unwrap()))),
        Err(e) => {
            let status = match &e {
                atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
                atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err((status, Json(json!({"error": e.to_string()}))))
        }
    }
}

/// List limit overrides
pub async fn list_limit_overrides(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListOverridesQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let card_id = query.card_id.as_ref().and_then(|s| Uuid::parse_str(s).ok());

    match state.corporate_card_engine.list_limit_overrides(
        card_id, query.status.as_deref(),
    ).await {
        Ok(overrides) => Ok(Json(json!({"data": overrides}))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Dashboard
// ============================================================================

/// Get corporate card dashboard
pub async fn get_corporate_card_dashboard(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"}))))?;

    match state.corporate_card_engine.get_dashboard_summary(org_id).await {
        Ok(summary) => Ok(Json(serde_json::to_value(summary).unwrap())),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()})))),
    }
}
