//! Recruiting Management Handlers
//!
//! Oracle Fusion HCM: Recruiting

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

fn rec_map_err(e: atlas_shared::AtlasError) -> StatusCode {
    match e.status_code() { 400=>StatusCode::BAD_REQUEST, 404=>StatusCode::NOT_FOUND, 409=>StatusCode::CONFLICT, _=>StatusCode::INTERNAL_SERVER_ERROR }
}

// ============================================================================
// Requisition Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateRequisitionRequest {
    pub requisition_number: String,
    pub title: String,
    pub description: Option<String>,
    pub department: Option<String>,
    pub location: Option<String>,
    pub employment_type: Option<String>,
    pub position_type: Option<String>,
    pub vacancies: Option<i32>,
    pub priority: Option<String>,
    pub salary_min: Option<String>,
    pub salary_max: Option<String>,
    pub currency: Option<String>,
    pub required_skills: Option<serde_json::Value>,
    pub qualifications: Option<String>,
    pub experience_years_min: Option<i32>,
    pub experience_years_max: Option<i32>,
    pub education_level: Option<String>,
    pub hiring_manager_id: Option<Uuid>,
    pub recruiter_id: Option<Uuid>,
    pub target_start_date: Option<chrono::NaiveDate>,
}

pub async fn create_requisition(
    State(state): State<Arc<AppState>>, claims: Extension<Claims>,
    Json(payload): Json<CreateRequisitionRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).ok();
    match state.recruiting_engine.create_requisition(
        org_id, &payload.requisition_number, &payload.title, payload.description.as_deref(),
        payload.department.as_deref(), payload.location.as_deref(),
        payload.employment_type.as_deref().unwrap_or("full_time"),
        payload.position_type.as_deref().unwrap_or("new"),
        payload.vacancies.unwrap_or(1),
        payload.priority.as_deref().unwrap_or("medium"),
        payload.salary_min.as_deref(), payload.salary_max.as_deref(),
        payload.currency.as_deref(),
        payload.required_skills.as_ref(), payload.qualifications.as_deref(),
        payload.experience_years_min, payload.experience_years_max,
        payload.education_level.as_deref(),
        payload.hiring_manager_id, payload.recruiter_id,
        payload.target_start_date, user_id,
    ).await {
        Ok(r) => Ok((StatusCode::CREATED, Json(serde_json::to_value(r).unwrap_or_default()))),
        Err(e) => { error!("Failed to create requisition: {}", e); Err(rec_map_err(e)) }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListRequisitionsParams {
    status: Option<String>,
}

pub async fn list_requisitions(
    State(state): State<Arc<AppState>>, claims: Extension<Claims>,
    Query(params): Query<ListRequisitionsParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.recruiting_engine.list_requisitions(org_id, params.status.as_deref()).await {
        Ok(r) => Ok(Json(serde_json::json!({"data": r}))),
        Err(e) => { error!("Error: {}", e); Err(rec_map_err(e)) }
    }
}

pub async fn get_requisition(
    State(state): State<Arc<AppState>>, claims: Extension<Claims>, Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let _ = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.recruiting_engine.get_requisition(id).await {
        Ok(Some(r)) => Ok(Json(serde_json::to_value(r).unwrap_or_default())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn open_requisition(
    State(state): State<Arc<AppState>>, claims: Extension<Claims>, Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let _ = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.recruiting_engine.open_requisition(id).await {
        Ok(r) => Ok(Json(serde_json::to_value(r).unwrap_or_default())),
        Err(e) => { error!("Error: {}", e); Err(rec_map_err(e)) }
    }
}

pub async fn hold_requisition(
    State(state): State<Arc<AppState>>, claims: Extension<Claims>, Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.recruiting_engine.hold_requisition(id).await {
        Ok(r) => Ok(Json(serde_json::to_value(r).unwrap_or_default())),
        Err(e) => { error!("Error: {}", e); Err(rec_map_err(e)) }
    }
}

pub async fn close_requisition(
    State(state): State<Arc<AppState>>, claims: Extension<Claims>, Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.recruiting_engine.close_requisition(id).await {
        Ok(r) => Ok(Json(serde_json::to_value(r).unwrap_or_default())),
        Err(e) => { error!("Error: {}", e); Err(rec_map_err(e)) }
    }
}

pub async fn cancel_requisition(
    State(state): State<Arc<AppState>>, claims: Extension<Claims>, Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.recruiting_engine.cancel_requisition(id).await {
        Ok(r) => Ok(Json(serde_json::to_value(r).unwrap_or_default())),
        Err(e) => { error!("Error: {}", e); Err(rec_map_err(e)) }
    }
}

pub async fn delete_requisition(
    State(state): State<Arc<AppState>>, claims: Extension<Claims>, Path(number): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.recruiting_engine.delete_requisition(org_id, &number).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// ============================================================================
// Candidate Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateCandidateRequest {
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub country: Option<String>,
    pub postal_code: Option<String>,
    pub linkedin_url: Option<String>,
    pub source: Option<String>,
    pub source_detail: Option<String>,
    pub resume_url: Option<String>,
    pub current_employer: Option<String>,
    pub current_title: Option<String>,
    pub years_of_experience: Option<i32>,
    pub education_level: Option<String>,
    pub skills: Option<serde_json::Value>,
    pub notes: Option<String>,
}

pub async fn create_candidate(
    State(state): State<Arc<AppState>>, claims: Extension<Claims>,
    Json(payload): Json<CreateCandidateRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).ok();
    match state.recruiting_engine.create_candidate(
        org_id, &payload.first_name, &payload.last_name,
        payload.email.as_deref(), payload.phone.as_deref(), payload.address.as_deref(),
        payload.city.as_deref(), payload.state.as_deref(), payload.country.as_deref(),
        payload.postal_code.as_deref(), payload.linkedin_url.as_deref(),
        payload.source.as_deref(), payload.source_detail.as_deref(),
        payload.resume_url.as_deref(), payload.current_employer.as_deref(),
        payload.current_title.as_deref(), payload.years_of_experience,
        payload.education_level.as_deref(), payload.skills.as_ref(),
        payload.notes.as_deref(), user_id,
    ).await {
        Ok(c) => Ok((StatusCode::CREATED, Json(serde_json::to_value(c).unwrap_or_default()))),
        Err(e) => { error!("Failed: {}", e); Err(rec_map_err(e)) }
    }
}

pub async fn get_candidate(
    State(state): State<Arc<AppState>>, claims: Extension<Claims>, Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let _ = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.recruiting_engine.get_candidate(id).await {
        Ok(Some(c)) => Ok(Json(serde_json::to_value(c).unwrap_or_default())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListCandidatesParams {
    status: Option<String>,
}

pub async fn list_candidates(
    State(state): State<Arc<AppState>>, claims: Extension<Claims>,
    Query(params): Query<ListCandidatesParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.recruiting_engine.list_candidates(org_id, params.status.as_deref()).await {
        Ok(c) => Ok(Json(serde_json::json!({"data": c}))),
        Err(e) => { error!("Error: {}", e); Err(rec_map_err(e)) }
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateCandidateStatusRequest {
    pub status: String,
}

pub async fn update_candidate_status(
    State(state): State<Arc<AppState>>, claims: Extension<Claims>, Path(id): Path<Uuid>,
    Json(payload): Json<UpdateCandidateStatusRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let _ = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.recruiting_engine.update_candidate_status(id, &payload.status).await {
        Ok(c) => Ok(Json(serde_json::to_value(c).unwrap_or_default())),
        Err(e) => { error!("Error: {}", e); Err(rec_map_err(e)) }
    }
}

pub async fn delete_candidate(
    State(state): State<Arc<AppState>>, claims: Extension<Claims>, Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    match state.recruiting_engine.delete_candidate(id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// ============================================================================
// Application Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateApplicationRequest {
    pub requisition_id: Uuid,
    pub candidate_id: Uuid,
}

pub async fn create_application(
    State(state): State<Arc<AppState>>, claims: Extension<Claims>,
    Json(payload): Json<CreateApplicationRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).ok();
    match state.recruiting_engine.create_application(
        org_id, payload.requisition_id, payload.candidate_id, user_id,
    ).await {
        Ok(a) => Ok((StatusCode::CREATED, Json(serde_json::to_value(a).unwrap_or_default()))),
        Err(e) => { error!("Failed: {}", e); Err(rec_map_err(e)) }
    }
}

pub async fn get_application(
    State(state): State<Arc<AppState>>, claims: Extension<Claims>, Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.recruiting_engine.get_application(id).await {
        Ok(Some(a)) => Ok(Json(serde_json::to_value(a).unwrap_or_default())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListApplicationsParams {
    requisition_id: Option<Uuid>,
    candidate_id: Option<Uuid>,
    status: Option<String>,
}

pub async fn list_applications(
    State(state): State<Arc<AppState>>, claims: Extension<Claims>,
    Query(params): Query<ListApplicationsParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.recruiting_engine.list_applications(
        org_id, params.requisition_id, params.candidate_id, params.status.as_deref(),
    ).await {
        Ok(a) => Ok(Json(serde_json::json!({"data": a}))),
        Err(e) => { error!("Error: {}", e); Err(rec_map_err(e)) }
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateApplicationStatusRequest {
    pub status: String,
    pub notes: Option<String>,
}

pub async fn update_application_status(
    State(state): State<Arc<AppState>>, claims: Extension<Claims>, Path(id): Path<Uuid>,
    Json(payload): Json<UpdateApplicationStatusRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.recruiting_engine.update_application_status(id, &payload.status, payload.notes.as_deref()).await {
        Ok(a) => Ok(Json(serde_json::to_value(a).unwrap_or_default())),
        Err(e) => { error!("Error: {}", e); Err(rec_map_err(e)) }
    }
}

pub async fn withdraw_application(
    State(state): State<Arc<AppState>>, claims: Extension<Claims>, Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.recruiting_engine.withdraw_application(id).await {
        Ok(a) => Ok(Json(serde_json::to_value(a).unwrap_or_default())),
        Err(e) => { error!("Error: {}", e); Err(rec_map_err(e)) }
    }
}

// ============================================================================
// Interview Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateInterviewRequest {
    pub interview_type: Option<String>,
    pub round: Option<i32>,
    pub scheduled_at: Option<chrono::DateTime<chrono::Utc>>,
    pub duration_minutes: Option<i32>,
    pub location: Option<String>,
    pub meeting_link: Option<String>,
    pub interviewer_ids: Option<serde_json::Value>,
    pub interviewer_names: Option<serde_json::Value>,
    pub notes: Option<String>,
}

pub async fn create_interview(
    State(state): State<Arc<AppState>>, claims: Extension<Claims>,
    Path(application_id): Path<Uuid>,
    Json(payload): Json<CreateInterviewRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).ok();
    match state.recruiting_engine.create_interview(
        org_id, application_id,
        payload.interview_type.as_deref().unwrap_or("phone"),
        payload.round.unwrap_or(1),
        payload.scheduled_at,
        payload.duration_minutes.unwrap_or(60),
        payload.location.as_deref(), payload.meeting_link.as_deref(),
        payload.interviewer_ids.as_ref(), payload.interviewer_names.as_ref(),
        payload.notes.as_deref(), user_id,
    ).await {
        Ok(i) => Ok((StatusCode::CREATED, Json(serde_json::to_value(i).unwrap_or_default()))),
        Err(e) => { error!("Failed: {}", e); Err(rec_map_err(e)) }
    }
}

pub async fn list_interviews(
    State(state): State<Arc<AppState>>, claims: Extension<Claims>,
    Path(application_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.recruiting_engine.list_interviews(application_id).await {
        Ok(i) => Ok(Json(serde_json::json!({"data": i}))),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

#[derive(Debug, Deserialize)]
pub struct CompleteInterviewRequest {
    pub feedback: Option<String>,
    pub rating: Option<i32>,
    pub recommendation: Option<String>,
}

pub async fn complete_interview(
    State(state): State<Arc<AppState>>, claims: Extension<Claims>, Path(id): Path<Uuid>,
    Json(payload): Json<CompleteInterviewRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.recruiting_engine.complete_interview(
        id, payload.feedback.as_deref(), payload.rating, payload.recommendation.as_deref(),
    ).await {
        Ok(i) => Ok(Json(serde_json::to_value(i).unwrap_or_default())),
        Err(e) => { error!("Error: {}", e); Err(rec_map_err(e)) }
    }
}

pub async fn cancel_interview(
    State(state): State<Arc<AppState>>, claims: Extension<Claims>, Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.recruiting_engine.cancel_interview(id).await {
        Ok(i) => Ok(Json(serde_json::to_value(i).unwrap_or_default())),
        Err(e) => { error!("Error: {}", e); Err(rec_map_err(e)) }
    }
}

pub async fn delete_interview(
    State(state): State<Arc<AppState>>, claims: Extension<Claims>, Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    match state.recruiting_engine.delete_interview(id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// ============================================================================
// Offer Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateOfferRequest {
    pub offer_number: Option<String>,
    pub job_title: String,
    pub department: Option<String>,
    pub location: Option<String>,
    pub employment_type: Option<String>,
    pub start_date: Option<chrono::NaiveDate>,
    pub salary_offered: Option<String>,
    pub salary_currency: Option<String>,
    pub salary_frequency: Option<String>,
    pub signing_bonus: Option<String>,
    pub benefits_summary: Option<String>,
    pub terms_and_conditions: Option<String>,
    pub response_deadline: Option<chrono::DateTime<chrono::Utc>>,
}

pub async fn create_offer(
    State(state): State<Arc<AppState>>, claims: Extension<Claims>,
    Path(application_id): Path<Uuid>,
    Json(payload): Json<CreateOfferRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).ok();
    match state.recruiting_engine.create_offer(
        org_id, application_id, payload.offer_number.as_deref(),
        &payload.job_title, payload.department.as_deref(), payload.location.as_deref(),
        payload.employment_type.as_deref().unwrap_or("full_time"),
        payload.start_date, payload.salary_offered.as_deref(),
        payload.salary_currency.as_deref(), payload.salary_frequency.as_deref(),
        payload.signing_bonus.as_deref(), payload.benefits_summary.as_deref(),
        payload.terms_and_conditions.as_deref(), payload.response_deadline, user_id,
    ).await {
        Ok(o) => Ok((StatusCode::CREATED, Json(serde_json::to_value(o).unwrap_or_default()))),
        Err(e) => { error!("Failed: {}", e); Err(rec_map_err(e)) }
    }
}

pub async fn get_offer(
    State(state): State<Arc<AppState>>, claims: Extension<Claims>, Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.recruiting_engine.get_offer(id).await {
        Ok(Some(o)) => Ok(Json(serde_json::to_value(o).unwrap_or_default())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListOffersParams {
    status: Option<String>,
}

pub async fn list_offers(
    State(state): State<Arc<AppState>>, claims: Extension<Claims>,
    Query(params): Query<ListOffersParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.recruiting_engine.list_offers(org_id, params.status.as_deref()).await {
        Ok(o) => Ok(Json(serde_json::json!({"data": o}))),
        Err(e) => { error!("Error: {}", e); Err(rec_map_err(e)) }
    }
}

pub async fn approve_offer(
    State(state): State<Arc<AppState>>, claims: Extension<Claims>, Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).ok();
    match state.recruiting_engine.approve_offer(id, user_id).await {
        Ok(o) => Ok(Json(serde_json::to_value(o).unwrap_or_default())),
        Err(e) => { error!("Error: {}", e); Err(rec_map_err(e)) }
    }
}

pub async fn extend_offer(
    State(state): State<Arc<AppState>>, claims: Extension<Claims>, Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.recruiting_engine.extend_offer(id).await {
        Ok(o) => Ok(Json(serde_json::to_value(o).unwrap_or_default())),
        Err(e) => { error!("Error: {}", e); Err(rec_map_err(e)) }
    }
}

#[derive(Debug, Deserialize)]
pub struct RespondOfferRequest {
    pub notes: Option<String>,
}

pub async fn accept_offer(
    State(state): State<Arc<AppState>>, claims: Extension<Claims>, Path(id): Path<Uuid>,
    Json(payload): Json<RespondOfferRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.recruiting_engine.accept_offer(id, payload.notes.as_deref()).await {
        Ok(o) => Ok(Json(serde_json::to_value(o).unwrap_or_default())),
        Err(e) => { error!("Error: {}", e); Err(rec_map_err(e)) }
    }
}

pub async fn decline_offer(
    State(state): State<Arc<AppState>>, claims: Extension<Claims>, Path(id): Path<Uuid>,
    Json(payload): Json<RespondOfferRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.recruiting_engine.decline_offer(id, payload.notes.as_deref()).await {
        Ok(o) => Ok(Json(serde_json::to_value(o).unwrap_or_default())),
        Err(e) => { error!("Error: {}", e); Err(rec_map_err(e)) }
    }
}

pub async fn withdraw_offer(
    State(state): State<Arc<AppState>>, claims: Extension<Claims>, Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.recruiting_engine.withdraw_offer(id).await {
        Ok(o) => Ok(Json(serde_json::to_value(o).unwrap_or_default())),
        Err(e) => { error!("Error: {}", e); Err(rec_map_err(e)) }
    }
}

pub async fn delete_offer(
    State(state): State<Arc<AppState>>, claims: Extension<Claims>, Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    match state.recruiting_engine.delete_offer(id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// ============================================================================
// Dashboard
// ============================================================================

pub async fn get_recruiting_dashboard(
    State(state): State<Arc<AppState>>, claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.recruiting_engine.get_dashboard(org_id).await {
        Ok(d) => Ok(Json(serde_json::to_value(d).unwrap_or_default())),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}
