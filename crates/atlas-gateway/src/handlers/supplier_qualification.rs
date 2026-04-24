//! Supplier Qualification Management API Handlers
//!
//! Oracle Fusion Cloud ERP: Procurement > Supplier Qualification
//!
//! Endpoints for managing qualification areas, questions, initiatives,
//! supplier invitations, responses, scoring, and certifications.

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
pub struct ListQuery {
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ActiveOnlyQuery {
    pub active_only: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct SupplierQuery {
    pub supplier_id: Option<Uuid>,
    pub status: Option<String>,
}

// ============================================================================
// Qualification Areas
// ============================================================================

/// Create a qualification area
pub async fn create_qualification_area(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    let area_code = body["area_code"].as_str().unwrap_or("").to_string();
    let name = body["name"].as_str().unwrap_or("").to_string();
    let description = body["description"].as_str();
    let area_type = body["area_type"].as_str().unwrap_or("questionnaire").to_string();
    let scoring_model = body["scoring_model"].as_str().unwrap_or("manual").to_string();
    let passing_score = body["passing_score"].as_str().unwrap_or("70").to_string();
    let is_mandatory = body["is_mandatory"].as_bool().unwrap_or(false);
    let renewal_period_days = body["renewal_period_days"].as_i64().unwrap_or(365) as i32;

    match state.supplier_qualification_engine.create_area(
        org_id, &area_code, &name, description, &area_type, &scoring_model,
        &passing_score, is_mandatory, renewal_period_days, parse_uuid(&claims.sub).ok(),
    ).await {
        Ok(area) => Ok((StatusCode::CREATED, Json(serde_json::to_value(area).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR), Json(json!({"error": e.to_string()})))),
    }
}

/// Get a qualification area by code
pub async fn get_qualification_area(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(code): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    match state.supplier_qualification_engine.get_area(org_id, &code).await {
        Ok(Some(area)) => Ok(Json(serde_json::to_value(area).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(json!({"error": "Area not found"})))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR), Json(json!({"error": e.to_string()})))),
    }
}

/// List qualification areas
pub async fn list_qualification_areas(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<ActiveOnlyQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    let active_only = params.active_only.unwrap_or(false);
    match state.supplier_qualification_engine.list_areas(org_id, active_only).await {
        Ok(areas) => Ok(Json(json!({"data": areas}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR), Json(json!({"error": e.to_string()})))),
    }
}

/// Delete a qualification area
pub async fn delete_qualification_area(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(code): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    match state.supplier_qualification_engine.delete_area(org_id, &code).await {
        Ok(()) => Ok(Json(json!({"message": "Area deleted"}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR), Json(json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Qualification Questions
// ============================================================================

/// Create a question
pub async fn create_qualification_question(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(area_id): Path<Uuid>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    let question_number = body["question_number"].as_i64().unwrap_or(1) as i32;
    let question_text = body["question_text"].as_str().unwrap_or("").to_string();
    let description = body["description"].as_str();
    let response_type = body["response_type"].as_str().unwrap_or("text").to_string();
    let choices = body.get("choices").cloned();
    let is_required = body["is_required"].as_bool().unwrap_or(true);
    let weight = body["weight"].as_str().unwrap_or("1").to_string();
    let max_score = body["max_score"].as_str().unwrap_or("10").to_string();
    let help_text = body["help_text"].as_str();
    let display_order = body["display_order"].as_i64().unwrap_or(0) as i32;

    match state.supplier_qualification_engine.create_question(
        org_id, area_id, question_number, &question_text, description,
        &response_type, choices, is_required, &weight, &max_score, help_text, display_order,
    ).await {
        Ok(question) => Ok((StatusCode::CREATED, Json(serde_json::to_value(question).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR), Json(json!({"error": e.to_string()})))),
    }
}

/// List questions for an area
pub async fn list_qualification_questions(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(area_id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.supplier_qualification_engine.list_questions(area_id).await {
        Ok(questions) => Ok(Json(json!({"data": questions}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR), Json(json!({"error": e.to_string()})))),
    }
}

/// Delete a question
pub async fn delete_qualification_question(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.supplier_qualification_engine.delete_question(id).await {
        Ok(()) => Ok(Json(json!({"message": "Question deleted"}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR), Json(json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Initiatives
// ============================================================================

/// Create an initiative
pub async fn create_initiative(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    let name = body["name"].as_str().unwrap_or("").to_string();
    let description = body["description"].as_str();
    let area_id: Uuid = body["area_id"].as_str()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| (StatusCode::BAD_REQUEST, Json(json!({"error": "area_id is required"}))))?;
    let qualification_purpose = body["qualification_purpose"].as_str().unwrap_or("new_supplier").to_string();
    let deadline = body["deadline"].as_str().and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());

    match state.supplier_qualification_engine.create_initiative(
        org_id, &name, description, area_id, &qualification_purpose,
        deadline, parse_uuid(&claims.sub).ok(),
    ).await {
        Ok(initiative) => Ok((StatusCode::CREATED, Json(serde_json::to_value(initiative).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR), Json(json!({"error": e.to_string()})))),
    }
}

/// Get an initiative
pub async fn get_initiative(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.supplier_qualification_engine.get_initiative(id).await {
        Ok(Some(initiative)) => Ok(Json(serde_json::to_value(initiative).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(json!({"error": "Initiative not found"})))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR), Json(json!({"error": e.to_string()})))),
    }
}

/// List initiatives
pub async fn list_initiatives(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<ListQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    match state.supplier_qualification_engine.list_initiatives(org_id, params.status.as_deref()).await {
        Ok(initiatives) => Ok(Json(json!({"data": initiatives}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR), Json(json!({"error": e.to_string()})))),
    }
}

/// Activate an initiative
pub async fn activate_initiative(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.supplier_qualification_engine.activate_initiative(id).await {
        Ok(initiative) => Ok(Json(serde_json::to_value(initiative).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR), Json(json!({"error": e.to_string()})))),
    }
}

/// Complete an initiative
pub async fn complete_initiative(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.supplier_qualification_engine.complete_initiative(id).await {
        Ok(initiative) => Ok(Json(serde_json::to_value(initiative).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR), Json(json!({"error": e.to_string()})))),
    }
}

/// Cancel an initiative
pub async fn cancel_initiative(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.supplier_qualification_engine.cancel_initiative(id).await {
        Ok(initiative) => Ok(Json(serde_json::to_value(initiative).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR), Json(json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Invitations
// ============================================================================

/// Invite a supplier to an initiative
pub async fn invite_supplier(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(initiative_id): Path<Uuid>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    let supplier_id: Uuid = body["supplier_id"].as_str()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| (StatusCode::BAD_REQUEST, Json(json!({"error": "supplier_id is required"}))))?;
    let supplier_name = body["supplier_name"].as_str().unwrap_or("").to_string();
    let supplier_contact_name = body["supplier_contact_name"].as_str();
    let supplier_contact_email = body["supplier_contact_email"].as_str();
    let expiry_date = body["expiry_date"].as_str().and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());

    match state.supplier_qualification_engine.invite_supplier(
        org_id, initiative_id, supplier_id, &supplier_name,
        supplier_contact_name, supplier_contact_email, expiry_date,
        parse_uuid(&claims.sub).ok(),
    ).await {
        Ok(invitation) => Ok((StatusCode::CREATED, Json(serde_json::to_value(invitation).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR), Json(json!({"error": e.to_string()})))),
    }
}

/// List invitations for an initiative
pub async fn list_invitations(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(initiative_id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.supplier_qualification_engine.list_invitations(initiative_id).await {
        Ok(invitations) => Ok(Json(json!({"data": invitations}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR), Json(json!({"error": e.to_string()})))),
    }
}

/// Submit supplier response (transition invitation to pending_response)
pub async fn submit_invitation_response(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(invitation_id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.supplier_qualification_engine.submit_response(invitation_id).await {
        Ok(invitation) => Ok(Json(serde_json::to_value(invitation).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR), Json(json!({"error": e.to_string()})))),
    }
}

/// Start evaluation of an invitation
pub async fn start_evaluation(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(invitation_id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.supplier_qualification_engine.start_evaluation(invitation_id).await {
        Ok(invitation) => Ok(Json(serde_json::to_value(invitation).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR), Json(json!({"error": e.to_string()})))),
    }
}

/// Qualify a supplier
pub async fn qualify_supplier(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(invitation_id): Path<Uuid>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let user_id: Uuid = parse_uuid(&claims.sub)?;
    let evaluation_notes = body["evaluation_notes"].as_str();

    match state.supplier_qualification_engine.qualify_supplier(
        invitation_id, Some(user_id), evaluation_notes,
    ).await {
        Ok(invitation) => Ok(Json(serde_json::to_value(invitation).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR), Json(json!({"error": e.to_string()})))),
    }
}

/// Disqualify a supplier
pub async fn disqualify_supplier(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(invitation_id): Path<Uuid>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let user_id: Uuid = parse_uuid(&claims.sub)?;
    let reason = body["reason"].as_str().unwrap_or("");

    match state.supplier_qualification_engine.disqualify_supplier(
        invitation_id, reason, Some(user_id),
    ).await {
        Ok(invitation) => Ok(Json(serde_json::to_value(invitation).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR), Json(json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Responses
// ============================================================================

/// Submit a response to a question
pub async fn create_response(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(invitation_id): Path<Uuid>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    let question_id: Uuid = body["question_id"].as_str()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| (StatusCode::BAD_REQUEST, Json(json!({"error": "question_id is required"}))))?;
    let response_text = body["response_text"].as_str();
    let response_value = body.get("response_value").cloned();
    let file_reference = body["file_reference"].as_str();

    match state.supplier_qualification_engine.create_response(
        org_id, invitation_id, question_id, response_text, response_value, file_reference,
    ).await {
        Ok(response) => Ok((StatusCode::CREATED, Json(serde_json::to_value(response).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR), Json(json!({"error": e.to_string()})))),
    }
}

/// List responses for an invitation
pub async fn list_responses(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(invitation_id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.supplier_qualification_engine.list_responses(invitation_id).await {
        Ok(responses) => Ok(Json(json!({"data": responses}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR), Json(json!({"error": e.to_string()})))),
    }
}

/// Score a response
pub async fn score_response(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(response_id): Path<Uuid>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let user_id: Uuid = parse_uuid(&claims.sub)?;
    let score = body["score"].as_str().unwrap_or("0");
    let evaluator_notes = body["evaluator_notes"].as_str();

    match state.supplier_qualification_engine.score_response(
        response_id, score, evaluator_notes, Some(user_id),
    ).await {
        Ok(response) => Ok(Json(serde_json::to_value(response).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR), Json(json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Certifications
// ============================================================================

/// Create a supplier certification
pub async fn create_certification(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    let supplier_id: Uuid = body["supplier_id"].as_str()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| (StatusCode::BAD_REQUEST, Json(json!({"error": "supplier_id is required"}))))?;
    let supplier_name = body["supplier_name"].as_str().unwrap_or("").to_string();
    let certification_type = body["certification_type"].as_str().unwrap_or("").to_string();
    let certification_name = body["certification_name"].as_str().unwrap_or("").to_string();
    let certifying_body = body["certifying_body"].as_str();
    let certificate_number = body["certificate_number"].as_str();
    let issued_date = body["issued_date"].as_str().and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let expiry_date = body["expiry_date"].as_str().and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let renewal_date = body["renewal_date"].as_str().and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let qualification_invitation_id = body["qualification_invitation_id"].as_str().and_then(|s| s.parse().ok());
    let document_reference = body["document_reference"].as_str();
    let notes = body["notes"].as_str();

    match state.supplier_qualification_engine.create_certification(
        org_id, supplier_id, &supplier_name, &certification_type, &certification_name,
        certifying_body, certificate_number, issued_date, expiry_date, renewal_date,
        qualification_invitation_id, document_reference, notes,
        parse_uuid(&claims.sub).ok(),
    ).await {
        Ok(cert) => Ok((StatusCode::CREATED, Json(serde_json::to_value(cert).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR), Json(json!({"error": e.to_string()})))),
    }
}

/// List certifications
pub async fn list_certifications(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<SupplierQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    match state.supplier_qualification_engine.list_certifications(
        org_id, params.supplier_id, params.status.as_deref(),
    ).await {
        Ok(certs) => Ok(Json(json!({"data": certs}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR), Json(json!({"error": e.to_string()})))),
    }
}

/// Revoke a certification
pub async fn revoke_certification(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.supplier_qualification_engine.revoke_certification(id).await {
        Ok(cert) => Ok(Json(serde_json::to_value(cert).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR), Json(json!({"error": e.to_string()})))),
    }
}

/// Renew a certification
pub async fn renew_certification(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let new_expiry = match body["expiry_date"].as_str()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok()) {
        Some(d) => d,
        None => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "expiry_date is required"})))),
    };
    let new_cert_number = body["certificate_number"].as_str();

    match state.supplier_qualification_engine.renew_certification(id, new_expiry, new_cert_number).await {
        Ok(cert) => Ok(Json(serde_json::to_value(cert).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR), Json(json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Dashboard
// ============================================================================

/// Get supplier qualification dashboard
pub async fn get_qualification_dashboard(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    match state.supplier_qualification_engine.get_dashboard_summary(org_id).await {
        Ok(summary) => Ok(Json(serde_json::to_value(summary).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR), Json(json!({"error": e.to_string()})))),
    }
}
