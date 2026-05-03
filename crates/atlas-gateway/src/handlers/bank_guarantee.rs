//! Bank Guarantee Management API Handlers
//!
//! Oracle Fusion Cloud ERP: Treasury > Bank Guarantees
//!
//! Endpoints for managing bank guarantees (bid bonds, performance guarantees,
//! advance payment guarantees, etc.), amendments, lifecycle management,
//! and dashboard reporting.

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
use crate::handlers::auth::{Claims, parse_uuid};

// ============================================================================
// Query Parameters
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct GuaranteeQuery {
    pub status: Option<String>,
    pub guarantee_type: Option<String>,
}

// ============================================================================
// Bank Guarantee CRUD
// ============================================================================

/// Create a bank guarantee
pub async fn create_guarantee(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    let guarantee_number = body["guaranteeNumber"].as_str()
        .or_else(|| body["guarantee_number"].as_str())
        .unwrap_or("").to_string();
    let guarantee_type = body["guaranteeType"].as_str()
        .or_else(|| body["guarantee_type"].as_str())
        .unwrap_or("").to_string();
    let description = body["description"].as_str();
    let beneficiary_name = body["beneficiaryName"].as_str()
        .or_else(|| body["beneficiary_name"].as_str())
        .unwrap_or("").to_string();
    let beneficiary_code = body["beneficiaryCode"].as_str()
        .or_else(|| body["beneficiary_code"].as_str());
    let applicant_name = body["applicantName"].as_str()
        .or_else(|| body["applicant_name"].as_str())
        .unwrap_or("").to_string();
    let applicant_code = body["applicantCode"].as_str()
        .or_else(|| body["applicant_code"].as_str());
    let issuing_bank_name = body["issuingBankName"].as_str()
        .or_else(|| body["issuing_bank_name"].as_str())
        .unwrap_or("").to_string();
    let issuing_bank_code = body["issuingBankCode"].as_str()
        .or_else(|| body["issuing_bank_code"].as_str());
    let bank_account_number = body["bankAccountNumber"].as_str()
        .or_else(|| body["bank_account_number"].as_str());
    let guarantee_amount = body["guaranteeAmount"].as_str()
        .or_else(|| body["guarantee_amount"].as_str())
        .unwrap_or("0").to_string();
    let currency_code = body["currencyCode"].as_str()
        .or_else(|| body["currency_code"].as_str())
        .unwrap_or("USD").to_string();
    let margin_percentage = body["marginPercentage"].as_str()
        .or_else(|| body["margin_percentage"].as_str())
        .unwrap_or("0").to_string();
    let commission_rate = body["commissionRate"].as_str()
        .or_else(|| body["commission_rate"].as_str())
        .unwrap_or("0").to_string();
    let issue_date = body["issueDate"].as_str()
        .or_else(|| body["issue_date"].as_str())
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let effective_date = body["effectiveDate"].as_str()
        .or_else(|| body["effective_date"].as_str())
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let expiry_date = body["expiryDate"].as_str()
        .or_else(|| body["expiry_date"].as_str())
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let claim_expiry_date = body["claimExpiryDate"].as_str()
        .or_else(|| body["claim_expiry_date"].as_str())
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let renewal_date = body["renewalDate"].as_str()
        .or_else(|| body["renewal_date"].as_str())
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let auto_renew = body["autoRenew"].as_bool()
        .or_else(|| body["auto_renew"].as_bool())
        .unwrap_or(false);
    let reference_contract_number = body["referenceContractNumber"].as_str()
        .or_else(|| body["reference_contract_number"].as_str());
    let reference_purchase_order = body["referencePurchaseOrder"].as_str()
        .or_else(|| body["reference_purchase_order"].as_str());
    let purpose = body["purpose"].as_str();
    let collateral_type = body["collateralType"].as_str()
        .or_else(|| body["collateral_type"].as_str());
    let collateral_amount = body["collateralAmount"].as_str()
        .or_else(|| body["collateral_amount"].as_str());
    let notes = body["notes"].as_str();

    match state.bank_guarantee_engine.create_guarantee(
        org_id, &guarantee_number, &guarantee_type, description,
        &beneficiary_name, beneficiary_code,
        &applicant_name, applicant_code,
        &issuing_bank_name, issuing_bank_code, bank_account_number,
        &guarantee_amount, &currency_code,
        &margin_percentage, &commission_rate,
        issue_date, effective_date, expiry_date,
        claim_expiry_date, renewal_date, auto_renew,
        reference_contract_number, reference_purchase_order,
        purpose, collateral_type, collateral_amount,
        notes, parse_uuid(&claims.sub).ok(),
    ).await {
        Ok(guarantee) => Ok((StatusCode::CREATED, Json(serde_json::to_value(&guarantee).unwrap_or(Value::Null)))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Get a guarantee by number
pub async fn get_guarantee(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(guarantee_number): Path<String>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    match state.bank_guarantee_engine.get_guarantee_by_number(org_id, &guarantee_number).await {
        Ok(Some(guarantee)) => Ok((StatusCode::OK, Json(serde_json::to_value(&guarantee).unwrap_or(Value::Null)))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(json!({"error": "Guarantee not found"})))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// List guarantees with optional filters
pub async fn list_guarantees(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<GuaranteeQuery>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    match state.bank_guarantee_engine.list_guarantees(
        org_id, params.status.as_deref(), params.guarantee_type.as_deref(),
    ).await {
        Ok(guarantees) => Ok((StatusCode::OK, Json(json!({"data": guarantees})))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Delete a draft guarantee
pub async fn delete_guarantee(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(guarantee_number): Path<String>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    match state.bank_guarantee_engine.delete_guarantee(org_id, &guarantee_number).await {
        Ok(()) => Ok((StatusCode::OK, Json(json!({"message": "Guarantee deleted"})))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Lifecycle Actions
// ============================================================================

/// Submit a draft guarantee for approval
pub async fn submit_for_approval(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    match state.bank_guarantee_engine.submit_for_approval(id).await {
        Ok(guarantee) => Ok((StatusCode::OK, Json(serde_json::to_value(&guarantee).unwrap_or(Value::Null)))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Approve a pending guarantee
pub async fn approve_guarantee(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let approved_by = parse_uuid(&claims.sub).unwrap_or_default();

    match state.bank_guarantee_engine.approve_guarantee(id, approved_by).await {
        Ok(guarantee) => Ok((StatusCode::OK, Json(serde_json::to_value(&guarantee).unwrap_or(Value::Null)))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Issue a guarantee
pub async fn issue_guarantee(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let issue_date = body["issueDate"].as_str()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .unwrap_or_else(|| chrono::Utc::now().date_naive());

    match state.bank_guarantee_engine.issue_guarantee(id, issue_date).await {
        Ok(guarantee) => Ok((StatusCode::OK, Json(serde_json::to_value(&guarantee).unwrap_or(Value::Null)))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Activate a guarantee
pub async fn activate_guarantee(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    match state.bank_guarantee_engine.activate_guarantee(id).await {
        Ok(guarantee) => Ok((StatusCode::OK, Json(serde_json::to_value(&guarantee).unwrap_or(Value::Null)))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Invoke a guarantee
pub async fn invoke_guarantee(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    match state.bank_guarantee_engine.invoke_guarantee(id).await {
        Ok(guarantee) => Ok((StatusCode::OK, Json(serde_json::to_value(&guarantee).unwrap_or(Value::Null)))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Release a guarantee
pub async fn release_guarantee(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    match state.bank_guarantee_engine.release_guarantee(id).await {
        Ok(guarantee) => Ok((StatusCode::OK, Json(serde_json::to_value(&guarantee).unwrap_or(Value::Null)))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Cancel a guarantee
pub async fn cancel_guarantee(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    match state.bank_guarantee_engine.cancel_guarantee(id).await {
        Ok(guarantee) => Ok((StatusCode::OK, Json(serde_json::to_value(&guarantee).unwrap_or(Value::Null)))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Process expired guarantees
pub async fn process_expired(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    let as_of_date = body["asOfDate"].as_str()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .unwrap_or_else(|| chrono::Utc::now().date_naive());

    match state.bank_guarantee_engine.process_expired_guarantees(org_id, as_of_date).await {
        Ok(expired) => Ok((StatusCode::OK, Json(json!({
            "expired_count": expired.len(),
            "expired_guarantees": expired,
        })))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Amendments
// ============================================================================

/// Create an amendment
pub async fn create_amendment(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(guarantee_id): Path<Uuid>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    let amendment_type = body["amendmentType"].as_str()
        .or_else(|| body["amendment_type"].as_str())
        .unwrap_or("").to_string();
    let previous_amount = body["previousAmount"].as_str()
        .or_else(|| body["previous_amount"].as_str());
    let new_amount = body["newAmount"].as_str()
        .or_else(|| body["new_amount"].as_str());
    let previous_expiry_date = body["previousExpiryDate"].as_str()
        .or_else(|| body["previous_expiry_date"].as_str())
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let new_expiry_date = body["newExpiryDate"].as_str()
        .or_else(|| body["new_expiry_date"].as_str())
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let previous_terms = body["previousTerms"].as_str()
        .or_else(|| body["previous_terms"].as_str());
    let new_terms = body["newTerms"].as_str()
        .or_else(|| body["new_terms"].as_str());
    let reason = body["reason"].as_str();
    let effective_date = body["effectiveDate"].as_str()
        .or_else(|| body["effective_date"].as_str())
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());

    match state.bank_guarantee_engine.create_amendment(
        org_id, guarantee_id, &amendment_type,
        previous_amount, new_amount,
        previous_expiry_date, new_expiry_date,
        previous_terms, new_terms,
        reason, effective_date, parse_uuid(&claims.sub).ok(),
    ).await {
        Ok(amendment) => Ok((StatusCode::CREATED, Json(serde_json::to_value(&amendment).unwrap_or(Value::Null)))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// List amendments for a guarantee
pub async fn list_amendments(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(guarantee_id): Path<Uuid>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    match state.bank_guarantee_engine.list_amendments(guarantee_id).await {
        Ok(amendments) => Ok((StatusCode::OK, Json(json!({"data": amendments})))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Approve an amendment
pub async fn approve_amendment(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(amendment_id): Path<Uuid>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let approved_by = parse_uuid(&claims.sub).unwrap_or_default();

    match state.bank_guarantee_engine.approve_amendment(amendment_id, approved_by).await {
        Ok(amendment) => Ok((StatusCode::OK, Json(serde_json::to_value(&amendment).unwrap_or(Value::Null)))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Reject an amendment
pub async fn reject_amendment(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(amendment_id): Path<Uuid>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    match state.bank_guarantee_engine.reject_amendment(amendment_id).await {
        Ok(amendment) => Ok((StatusCode::OK, Json(serde_json::to_value(&amendment).unwrap_or(Value::Null)))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Dashboard
// ============================================================================

/// Get bank guarantee dashboard
pub async fn get_dashboard(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    match state.bank_guarantee_engine.get_dashboard(org_id).await {
        Ok(dashboard) => Ok((StatusCode::OK, Json(serde_json::to_value(&dashboard).unwrap_or(Value::Null)))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}
