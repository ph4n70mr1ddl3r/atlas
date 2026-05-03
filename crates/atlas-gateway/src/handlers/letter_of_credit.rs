//! Letter of Credit Management API Handlers
//!
//! Oracle Fusion Cloud ERP: Treasury > Trade Finance > Letters of Credit
//!
//! Endpoints for managing letters of credit (import/export/back-to-back/standby),
//! amendments, required documents, shipments, presentations, presentation documents,
//! lifecycle management, and dashboard reporting.

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
pub struct LcQuery {
    pub status: Option<String>,
    pub lc_type: Option<String>,
}

// ============================================================================
// Letter of Credit CRUD
// ============================================================================

/// Create a letter of credit
pub async fn create_lc(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    let lc_number = body["lcNumber"].as_str()
        .or_else(|| body["lc_number"].as_str())
        .unwrap_or("").to_string();
    let lc_type = body["lcType"].as_str()
        .or_else(|| body["lc_type"].as_str())
        .unwrap_or("import").to_string();
    let lc_form = body["lcForm"].as_str()
        .or_else(|| body["lc_form"].as_str())
        .unwrap_or("irrevocable").to_string();
    let description = body["description"].as_str();
    let applicant_name = body["applicantName"].as_str()
        .or_else(|| body["applicant_name"].as_str())
        .unwrap_or("").to_string();
    let applicant_address = body["applicantAddress"].as_str()
        .or_else(|| body["applicant_address"].as_str());
    let applicant_bank_name = body["applicantBankName"].as_str()
        .or_else(|| body["applicant_bank_name"].as_str())
        .unwrap_or("").to_string();
    let applicant_bank_swift = body["applicantBankSwift"].as_str()
        .or_else(|| body["applicant_bank_swift"].as_str());
    let beneficiary_name = body["beneficiaryName"].as_str()
        .or_else(|| body["beneficiary_name"].as_str())
        .unwrap_or("").to_string();
    let beneficiary_address = body["beneficiaryAddress"].as_str()
        .or_else(|| body["beneficiary_address"].as_str());
    let beneficiary_bank_name = body["beneficiaryBankName"].as_str()
        .or_else(|| body["beneficiary_bank_name"].as_str());
    let beneficiary_bank_swift = body["beneficiaryBankSwift"].as_str()
        .or_else(|| body["beneficiary_bank_swift"].as_str());
    let advising_bank_name = body["advisingBankName"].as_str()
        .or_else(|| body["advising_bank_name"].as_str());
    let advising_bank_swift = body["advisingBankSwift"].as_str()
        .or_else(|| body["advising_bank_swift"].as_str());
    let confirming_bank_name = body["confirmingBankName"].as_str()
        .or_else(|| body["confirming_bank_name"].as_str());
    let confirming_bank_swift = body["confirmingBankSwift"].as_str()
        .or_else(|| body["confirming_bank_swift"].as_str());
    let lc_amount = body["lcAmount"].as_str()
        .or_else(|| body["lc_amount"].as_str())
        .unwrap_or("0").to_string();
    let currency_code = body["currencyCode"].as_str()
        .or_else(|| body["currency_code"].as_str())
        .unwrap_or("USD").to_string();
    let tolerance_plus = body["tolerancePlus"].as_str()
        .or_else(|| body["tolerance_plus"].as_str())
        .unwrap_or("0").to_string();
    let tolerance_minus = body["toleranceMinus"].as_str()
        .or_else(|| body["tolerance_minus"].as_str())
        .unwrap_or("0").to_string();
    let available_with = body["availableWith"].as_str()
        .or_else(|| body["available_with"].as_str());
    let available_by = body["availableBy"].as_str()
        .or_else(|| body["available_by"].as_str())
        .unwrap_or("payment").to_string();
    let draft_at = body["draftAt"].as_str()
        .or_else(|| body["draft_at"].as_str());
    let expiry_date = body["expiryDate"].as_str()
        .or_else(|| body["expiry_date"].as_str())
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .unwrap_or_else(|| chrono::Utc::now().date_naive() + chrono::Duration::days(90));
    let place_of_expiry = body["placeOfExpiry"].as_str()
        .or_else(|| body["place_of_expiry"].as_str());
    let partial_shipments = body["partialShipments"].as_str()
        .or_else(|| body["partial_shipments"].as_str())
        .unwrap_or("allowed").to_string();
    let transshipment = body["transshipment"].as_str()
        .or_else(|| body["transshipment"].as_str())
        .unwrap_or("allowed").to_string();
    let port_of_loading = body["portOfLoading"].as_str()
        .or_else(|| body["port_of_loading"].as_str());
    let port_of_discharge = body["portOfDischarge"].as_str()
        .or_else(|| body["port_of_discharge"].as_str());
    let shipment_period = body["shipmentPeriod"].as_str()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let latest_shipment_date = body["latestShipmentDate"].as_str()
        .or_else(|| body["latest_shipment_date"].as_str())
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let goods_description = body["goodsDescription"].as_str()
        .or_else(|| body["goods_description"].as_str());
    let incoterms = body["incoterms"].as_str()
        .or_else(|| body["incoterms"].as_str());
    let additional_conditions = body["additionalConditions"].as_str()
        .or_else(|| body["additional_conditions"].as_str());
    let bank_charges = body["bankCharges"].as_str()
        .or_else(|| body["bank_charges"].as_str())
        .unwrap_or("beneficiary").to_string();
    let reference_po_number = body["referencePoNumber"].as_str()
        .or_else(|| body["reference_po_number"].as_str());
    let reference_contract_number = body["referenceContractNumber"].as_str()
        .or_else(|| body["reference_contract_number"].as_str());
    let notes = body["notes"].as_str();

    match state.letter_of_credit_engine.create_lc(
        org_id, &lc_number, &lc_type, &lc_form,
        description,
        &applicant_name, applicant_address, &applicant_bank_name, applicant_bank_swift,
        &beneficiary_name, beneficiary_address, beneficiary_bank_name, beneficiary_bank_swift,
        advising_bank_name, advising_bank_swift,
        confirming_bank_name, confirming_bank_swift,
        &lc_amount, &currency_code, &tolerance_plus, &tolerance_minus,
        available_with, &available_by, draft_at,
        expiry_date, place_of_expiry,
        &partial_shipments, &transshipment,
        port_of_loading, port_of_discharge,
        shipment_period, latest_shipment_date,
        goods_description, incoterms, additional_conditions,
        &bank_charges,
        reference_po_number, reference_contract_number,
        notes, parse_uuid(&claims.sub).ok(),
    ).await {
        Ok(lc) => Ok((StatusCode::CREATED, Json(serde_json::to_value(&lc).unwrap_or(Value::Null)))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Get an LC by number
pub async fn get_lc(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(lc_number): Path<String>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    match state.letter_of_credit_engine.get_lc_by_number(org_id, &lc_number).await {
        Ok(Some(lc)) => Ok((StatusCode::OK, Json(serde_json::to_value(&lc).unwrap_or(Value::Null)))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(json!({"error": "Letter of credit not found"})))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// List LCs with optional filters
pub async fn list_lcs(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<LcQuery>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    match state.letter_of_credit_engine.list_lcs(
        org_id, params.status.as_deref(), params.lc_type.as_deref(),
    ).await {
        Ok(lcs) => Ok((StatusCode::OK, Json(json!({"data": lcs})))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Delete a draft LC
pub async fn delete_lc(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(lc_number): Path<String>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    match state.letter_of_credit_engine.delete_lc(org_id, &lc_number).await {
        Ok(()) => Ok((StatusCode::OK, Json(json!({"message": "Letter of credit deleted"})))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// LC Lifecycle Actions
// ============================================================================

/// Issue a draft LC
pub async fn issue_lc(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let issue_date = body["issueDate"].as_str()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .unwrap_or_else(|| chrono::Utc::now().date_naive());

    match state.letter_of_credit_engine.issue_lc(id, issue_date).await {
        Ok(lc) => Ok((StatusCode::OK, Json(serde_json::to_value(&lc).unwrap_or(Value::Null)))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Advise an issued LC
pub async fn advise_lc(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    match state.letter_of_credit_engine.advise_lc(id).await {
        Ok(lc) => Ok((StatusCode::OK, Json(serde_json::to_value(&lc).unwrap_or(Value::Null)))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Confirm an advised LC
pub async fn confirm_lc(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let approved_by = parse_uuid(&claims.sub).unwrap_or_default();

    match state.letter_of_credit_engine.confirm_lc(id, approved_by).await {
        Ok(lc) => Ok((StatusCode::OK, Json(serde_json::to_value(&lc).unwrap_or(Value::Null)))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Accept an LC
pub async fn accept_lc(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    match state.letter_of_credit_engine.accept_lc(id).await {
        Ok(lc) => Ok((StatusCode::OK, Json(serde_json::to_value(&lc).unwrap_or(Value::Null)))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Pay an accepted LC
pub async fn pay_lc(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    match state.letter_of_credit_engine.pay_lc(id).await {
        Ok(lc) => Ok((StatusCode::OK, Json(serde_json::to_value(&lc).unwrap_or(Value::Null)))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Cancel an LC
pub async fn cancel_lc(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    match state.letter_of_credit_engine.cancel_lc(id).await {
        Ok(lc) => Ok((StatusCode::OK, Json(serde_json::to_value(&lc).unwrap_or(Value::Null)))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Process expired LCs
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

    match state.letter_of_credit_engine.process_expired(org_id, as_of_date).await {
        Ok(expired) => Ok((StatusCode::OK, Json(json!({
            "expired_count": expired.len(),
            "expired_lcs": expired,
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
    Path(lc_id): Path<Uuid>,
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
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let new_expiry_date = body["newExpiryDate"].as_str()
        .or_else(|| body["new_expiry_date"].as_str())
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let previous_terms = body["previousTerms"].as_str()
        .or_else(|| body["previous_terms"].as_str());
    let new_terms = body["newTerms"].as_str()
        .or_else(|| body["new_terms"].as_str());
    let reason = body["reason"].as_str();
    let bank_reference = body["bankReference"].as_str()
        .or_else(|| body["bank_reference"].as_str());
    let effective_date = body["effectiveDate"].as_str()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());

    match state.letter_of_credit_engine.create_amendment(
        org_id, lc_id, &amendment_type,
        previous_amount, new_amount,
        previous_expiry_date, new_expiry_date,
        previous_terms, new_terms,
        reason, bank_reference,
        effective_date, parse_uuid(&claims.sub).ok(),
    ).await {
        Ok(amendment) => Ok((StatusCode::CREATED, Json(serde_json::to_value(&amendment).unwrap_or(Value::Null)))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// List amendments for an LC
pub async fn list_amendments(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(lc_id): Path<Uuid>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    match state.letter_of_credit_engine.list_amendments(lc_id).await {
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

    match state.letter_of_credit_engine.approve_amendment(amendment_id, approved_by).await {
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
    match state.letter_of_credit_engine.reject_amendment(amendment_id).await {
        Ok(amendment) => Ok((StatusCode::OK, Json(serde_json::to_value(&amendment).unwrap_or(Value::Null)))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Required Documents
// ============================================================================

/// Add a required document
pub async fn add_required_document(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(lc_id): Path<Uuid>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    let document_type = body["documentType"].as_str()
        .or_else(|| body["document_type"].as_str())
        .unwrap_or("").to_string();
    let document_code = body["documentCode"].as_str()
        .or_else(|| body["document_code"].as_str());
    let description = body["description"].as_str();
    let original_copies = body["originalCopies"].as_i64()
        .or_else(|| body["original_copies"].as_i64())
        .unwrap_or(1) as i32;
    let copy_count = body["copyCount"].as_i64()
        .or_else(|| body["copy_count"].as_i64())
        .unwrap_or(0) as i32;
    let is_mandatory = body["isMandatory"].as_bool()
        .or_else(|| body["is_mandatory"].as_bool())
        .unwrap_or(true);
    let special_instructions = body["specialInstructions"].as_str()
        .or_else(|| body["special_instructions"].as_str());

    match state.letter_of_credit_engine.add_required_document(
        org_id, lc_id, &document_type, document_code, description,
        original_copies, copy_count, is_mandatory, special_instructions,
    ).await {
        Ok(doc) => Ok((StatusCode::CREATED, Json(serde_json::to_value(&doc).unwrap_or(Value::Null)))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// List required documents for an LC
pub async fn list_required_documents(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(lc_id): Path<Uuid>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    match state.letter_of_credit_engine.list_required_documents(lc_id).await {
        Ok(docs) => Ok((StatusCode::OK, Json(json!({"data": docs})))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Delete a required document
pub async fn delete_required_document(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(doc_id): Path<Uuid>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    match state.letter_of_credit_engine.delete_required_document(doc_id).await {
        Ok(()) => Ok((StatusCode::OK, Json(json!({"message": "Required document deleted"})))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Shipments
// ============================================================================

/// Create a shipment
pub async fn create_shipment(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(lc_id): Path<Uuid>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    let shipment_number = body["shipmentNumber"].as_str()
        .or_else(|| body["shipment_number"].as_str())
        .unwrap_or("").to_string();
    let vessel_name = body["vesselName"].as_str()
        .or_else(|| body["vessel_name"].as_str());
    let voyage_number = body["voyageNumber"].as_str()
        .or_else(|| body["voyage_number"].as_str());
    let bill_of_lading_number = body["billOfLadingNumber"].as_str()
        .or_else(|| body["bill_of_lading_number"].as_str());
    let carrier_name = body["carrierName"].as_str()
        .or_else(|| body["carrier_name"].as_str());
    let port_of_loading = body["portOfLoading"].as_str()
        .or_else(|| body["port_of_loading"].as_str());
    let port_of_discharge = body["portOfDischarge"].as_str()
        .or_else(|| body["port_of_discharge"].as_str());
    let shipment_date = body["shipmentDate"].as_str()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let expected_arrival_date = body["expectedArrivalDate"].as_str()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let goods_description = body["goodsDescription"].as_str()
        .or_else(|| body["goods_description"].as_str());
    let quantity = body["quantity"].as_str();
    let unit_price = body["unitPrice"].as_str()
        .or_else(|| body["unit_price"].as_str());
    let shipment_amount = body["shipmentAmount"].as_str()
        .or_else(|| body["shipment_amount"].as_str())
        .unwrap_or("0").to_string();
    let currency_code = body["currencyCode"].as_str()
        .or_else(|| body["currency_code"].as_str())
        .unwrap_or("USD").to_string();
    let notes = body["notes"].as_str();

    match state.letter_of_credit_engine.create_shipment(
        org_id, lc_id, &shipment_number,
        vessel_name, voyage_number, bill_of_lading_number, carrier_name,
        port_of_loading, port_of_discharge,
        shipment_date, expected_arrival_date,
        goods_description, quantity, unit_price,
        &shipment_amount, &currency_code, notes,
    ).await {
        Ok(shipment) => Ok((StatusCode::CREATED, Json(serde_json::to_value(&shipment).unwrap_or(Value::Null)))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// List shipments for an LC
pub async fn list_shipments(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(lc_id): Path<Uuid>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    match state.letter_of_credit_engine.list_shipments(lc_id).await {
        Ok(shipments) => Ok((StatusCode::OK, Json(json!({"data": shipments})))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Update shipment status
pub async fn update_shipment_status(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path((shipment_id, status)): Path<(Uuid, String)>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    match state.letter_of_credit_engine.update_shipment_status(shipment_id, &status).await {
        Ok(shipment) => Ok((StatusCode::OK, Json(serde_json::to_value(&shipment).unwrap_or(Value::Null)))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Presentations
// ============================================================================

/// Create a presentation
pub async fn create_presentation(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(lc_id): Path<Uuid>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    let presentation_number = body["presentationNumber"].as_str()
        .or_else(|| body["presentation_number"].as_str())
        .unwrap_or("").to_string();
    let shipment_id = body["shipmentId"].as_str()
        .and_then(|s| Uuid::parse_str(s).ok());
    let presentation_date = body["presentationDate"].as_str()
        .or_else(|| body["presentation_date"].as_str())
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .unwrap_or_else(|| chrono::Utc::now().date_naive());
    let presenting_bank_name = body["presentingBankName"].as_str()
        .or_else(|| body["presenting_bank_name"].as_str());
    let total_amount = body["totalAmount"].as_str()
        .or_else(|| body["total_amount"].as_str())
        .unwrap_or("0").to_string();
    let currency_code = body["currencyCode"].as_str()
        .or_else(|| body["currency_code"].as_str())
        .unwrap_or("USD").to_string();
    let discrepant = body["discrepant"].as_bool().unwrap_or(false);
    let discrepancies = body["discrepancies"].as_str();
    let notes = body["notes"].as_str();

    match state.letter_of_credit_engine.create_presentation(
        org_id, lc_id, &presentation_number, shipment_id,
        presentation_date, presenting_bank_name,
        &total_amount, &currency_code,
        discrepant, discrepancies, notes,
        parse_uuid(&claims.sub).ok(),
    ).await {
        Ok(presentation) => Ok((StatusCode::CREATED, Json(serde_json::to_value(&presentation).unwrap_or(Value::Null)))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// List presentations for an LC
pub async fn list_presentations(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(lc_id): Path<Uuid>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    match state.letter_of_credit_engine.list_presentations(lc_id).await {
        Ok(presentations) => Ok((StatusCode::OK, Json(json!({"data": presentations})))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Accept a presentation
pub async fn accept_presentation(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(presentation_id): Path<Uuid>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    match state.letter_of_credit_engine.accept_presentation(presentation_id).await {
        Ok(presentation) => Ok((StatusCode::OK, Json(serde_json::to_value(&presentation).unwrap_or(Value::Null)))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Pay a presentation
pub async fn pay_presentation(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(presentation_id): Path<Uuid>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let paid_amount = body["paidAmount"].as_str()
        .or_else(|| body["paid_amount"].as_str())
        .unwrap_or("0").to_string();
    let payment_date = body["paymentDate"].as_str()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .unwrap_or_else(|| chrono::Utc::now().date_naive());

    match state.letter_of_credit_engine.pay_presentation(presentation_id, &paid_amount, payment_date).await {
        Ok(presentation) => Ok((StatusCode::OK, Json(serde_json::to_value(&presentation).unwrap_or(Value::Null)))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Reject a presentation
pub async fn reject_presentation(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(presentation_id): Path<Uuid>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    match state.letter_of_credit_engine.reject_presentation(presentation_id).await {
        Ok(presentation) => Ok((StatusCode::OK, Json(serde_json::to_value(&presentation).unwrap_or(Value::Null)))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Presentation Documents
// ============================================================================

/// Add a document to a presentation
pub async fn add_presentation_document(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(presentation_id): Path<Uuid>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    let required_document_id = body["requiredDocumentId"].as_str()
        .and_then(|s| Uuid::parse_str(s).ok());
    let document_type = body["documentType"].as_str()
        .or_else(|| body["document_type"].as_str())
        .unwrap_or("").to_string();
    let document_reference = body["documentReference"].as_str()
        .or_else(|| body["document_reference"].as_str());
    let description = body["description"].as_str();
    let original_copies = body["originalCopies"].as_i64()
        .or_else(|| body["original_copies"].as_i64())
        .unwrap_or(1) as i32;
    let copy_count = body["copyCount"].as_i64()
        .or_else(|| body["copy_count"].as_i64())
        .unwrap_or(0) as i32;
    let is_compliant = body["isCompliant"].as_bool()
        .or_else(|| body["is_compliant"].as_bool())
        .unwrap_or(true);
    let discrepancies = body["discrepancies"].as_str();

    match state.letter_of_credit_engine.add_presentation_document(
        org_id, presentation_id,
        required_document_id, &document_type, document_reference,
        description, original_copies, copy_count,
        is_compliant, discrepancies,
    ).await {
        Ok(doc) => Ok((StatusCode::CREATED, Json(serde_json::to_value(&doc).unwrap_or(Value::Null)))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// List documents for a presentation
pub async fn list_presentation_documents(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(presentation_id): Path<Uuid>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    match state.letter_of_credit_engine.list_presentation_documents(presentation_id).await {
        Ok(docs) => Ok((StatusCode::OK, Json(json!({"data": docs})))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Dashboard
// ============================================================================

/// Get LC dashboard
pub async fn get_dashboard(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    match state.letter_of_credit_engine.get_dashboard(org_id).await {
        Ok(dashboard) => Ok((StatusCode::OK, Json(serde_json::to_value(&dashboard).unwrap_or(Value::Null)))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}
