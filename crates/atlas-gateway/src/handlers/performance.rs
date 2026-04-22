//! Performance Management Handlers
//!
//! Oracle Fusion Cloud HCM: My Client Groups > Performance
//!
//! API endpoints for managing rating models, review cycles, performance documents,
//! goals, competency assessments, and feedback.

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
// Rating Model Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateRatingModelRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub rating_scale: serde_json::Value,
}

pub async fn create_rating_model(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateRatingModelRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.performance_engine.create_rating_model(
        org_id, &payload.code, &payload.name, payload.description.as_deref(),
        payload.rating_scale, Some(user_id),
    ).await {
        Ok(model) => Ok((StatusCode::CREATED, Json(serde_json::to_value(model).unwrap()))),
        Err(e) => {
            error!("Failed to create rating model: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn get_rating_model(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(code): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.performance_engine.get_rating_model(org_id, &code).await {
        Ok(Some(m)) => Ok(Json(serde_json::to_value(m).unwrap())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn list_rating_models(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.performance_engine.list_rating_models(org_id).await {
        Ok(models) => Ok(Json(serde_json::json!({"data": models}))),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn delete_rating_model(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(code): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.performance_engine.delete_rating_model(org_id, &code).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// ============================================================================
// Review Cycle Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateReviewCycleRequest {
    pub name: String,
    pub description: Option<String>,
    pub cycle_type: String,
    pub rating_model_code: Option<String>,
    pub start_date: chrono::NaiveDate,
    pub end_date: chrono::NaiveDate,
    pub goal_setting_start: Option<chrono::NaiveDate>,
    pub goal_setting_end: Option<chrono::NaiveDate>,
    pub self_evaluation_start: Option<chrono::NaiveDate>,
    pub self_evaluation_end: Option<chrono::NaiveDate>,
    pub manager_evaluation_start: Option<chrono::NaiveDate>,
    pub manager_evaluation_end: Option<chrono::NaiveDate>,
    pub calibration_date: Option<chrono::NaiveDate>,
    #[serde(default = "default_true")]
    pub require_goals: bool,
    #[serde(default = "default_true")]
    pub require_competencies: bool,
    #[serde(default = "default_three")]
    pub min_goals: i32,
    #[serde(default = "default_ten")]
    pub max_goals: i32,
    #[serde(default = "default_hundred")]
    pub goal_weight_total: String,
}

fn default_true() -> bool { true }
fn default_three() -> i32 { 3 }
fn default_ten() -> i32 { 10 }
fn default_hundred() -> String { "100.00".to_string() }

pub async fn create_review_cycle(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateReviewCycleRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.performance_engine.create_review_cycle(
        org_id, &payload.name, payload.description.as_deref(), &payload.cycle_type,
        payload.rating_model_code.as_deref(),
        payload.start_date, payload.end_date,
        payload.goal_setting_start, payload.goal_setting_end,
        payload.self_evaluation_start, payload.self_evaluation_end,
        payload.manager_evaluation_start, payload.manager_evaluation_end,
        payload.calibration_date,
        payload.require_goals, payload.require_competencies,
        payload.min_goals, payload.max_goals, &payload.goal_weight_total,
        Some(user_id),
    ).await {
        Ok(cycle) => Ok((StatusCode::CREATED, Json(serde_json::to_value(cycle).unwrap()))),
        Err(e) => {
            error!("Failed to create review cycle: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn get_review_cycle(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.performance_engine.get_review_cycle(id).await {
        Ok(Some(c)) => Ok(Json(serde_json::to_value(c).unwrap())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListCyclesQuery {
    pub status: Option<String>,
}

pub async fn list_review_cycles(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListCyclesQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.performance_engine.list_review_cycles(org_id, query.status.as_deref()).await {
        Ok(cycles) => Ok(Json(serde_json::json!({"data": cycles}))),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

#[derive(Debug, Deserialize)]
pub struct TransitionCycleRequest {
    pub status: String,
}

pub async fn transition_cycle(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<TransitionCycleRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.performance_engine.transition_cycle(id, &payload.status).await {
        Ok(cycle) => Ok(Json(serde_json::to_value(cycle).unwrap())),
        Err(e) => {
            error!("Failed to transition cycle: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// ============================================================================
// Competency Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateCompetencyRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub rating_model_code: Option<String>,
    #[serde(default)]
    pub behavioral_indicators: serde_json::Value,
}

pub async fn create_competency(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateCompetencyRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let indicators = if payload.behavioral_indicators.is_null() || payload.behavioral_indicators.as_array().is_none_or(|a| a.is_empty()) {
        serde_json::json!([])
    } else {
        payload.behavioral_indicators
    };

    match state.performance_engine.create_competency(
        org_id, &payload.code, &payload.name, payload.description.as_deref(),
        payload.category.as_deref(), payload.rating_model_code.as_deref(),
        indicators, Some(user_id),
    ).await {
        Ok(comp) => Ok((StatusCode::CREATED, Json(serde_json::to_value(comp).unwrap()))),
        Err(e) => {
            error!("Failed to create competency: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn get_competency(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(code): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.performance_engine.get_competency(org_id, &code).await {
        Ok(Some(c)) => Ok(Json(serde_json::to_value(c).unwrap())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListCompetenciesQuery {
    pub category: Option<String>,
}

pub async fn list_competencies(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListCompetenciesQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.performance_engine.list_competencies(org_id, query.category.as_deref()).await {
        Ok(comps) => Ok(Json(serde_json::json!({"data": comps}))),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn delete_competency(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(code): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.performance_engine.delete_competency(org_id, &code).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// ============================================================================
// Performance Document Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateDocumentRequest {
    pub review_cycle_id: Uuid,
    pub employee_id: Uuid,
    pub employee_name: Option<String>,
    pub manager_id: Option<Uuid>,
    pub manager_name: Option<String>,
}

pub async fn create_document(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateDocumentRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.performance_engine.create_document(
        org_id, payload.review_cycle_id, payload.employee_id,
        payload.employee_name.as_deref(), payload.manager_id, payload.manager_name.as_deref(),
        Some(user_id),
    ).await {
        Ok(doc) => Ok((StatusCode::CREATED, Json(serde_json::to_value(doc).unwrap()))),
        Err(e) => {
            error!("Failed to create document: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                409 => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn get_document(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.performance_engine.get_document(id).await {
        Ok(Some(d)) => Ok(Json(serde_json::to_value(d).unwrap())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListDocumentsQuery {
    pub review_cycle_id: Option<Uuid>,
    pub employee_id: Option<Uuid>,
    pub status: Option<String>,
}

pub async fn list_documents(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListDocumentsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.performance_engine.list_documents(
        org_id, query.review_cycle_id, query.employee_id, query.status.as_deref(),
    ).await {
        Ok(docs) => Ok(Json(serde_json::json!({"data": docs}))),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

#[derive(Debug, Deserialize)]
pub struct TransitionDocumentRequest {
    pub status: String,
}

pub async fn transition_document(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<TransitionDocumentRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.performance_engine.transition_document(id, &payload.status).await {
        Ok(doc) => Ok(Json(serde_json::to_value(doc).unwrap())),
        Err(e) => {
            error!("Failed to transition document: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SubmitEvaluationRequest {
    pub overall_rating: Option<String>,
    pub comments: Option<String>,
}

pub async fn submit_self_evaluation(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<SubmitEvaluationRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.performance_engine.submit_self_evaluation(
        id, payload.overall_rating.as_deref(), payload.comments.as_deref(),
    ).await {
        Ok(doc) => Ok(Json(serde_json::to_value(doc).unwrap())),
        Err(e) => {
            error!("Failed to submit self-evaluation: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn submit_manager_evaluation(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<SubmitEvaluationRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.performance_engine.submit_manager_evaluation(
        id, payload.overall_rating.as_deref(), payload.comments.as_deref(),
    ).await {
        Ok(doc) => Ok(Json(serde_json::to_value(doc).unwrap())),
        Err(e) => {
            error!("Failed to submit manager evaluation: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct FinalizeDocumentRequest {
    pub final_rating: Option<String>,
    pub final_comments: Option<String>,
}

pub async fn finalize_document(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<FinalizeDocumentRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.performance_engine.finalize_document(
        id, payload.final_rating.as_deref(), payload.final_comments.as_deref(),
    ).await {
        Ok(doc) => Ok(Json(serde_json::to_value(doc).unwrap())),
        Err(e) => {
            error!("Failed to finalize document: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// ============================================================================
// Goal Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateGoalRequest {
    pub document_id: Uuid,
    pub employee_id: Uuid,
    pub goal_name: String,
    pub description: Option<String>,
    pub goal_category: Option<String>,
    pub weight: String,
    pub target_metric: Option<String>,
    pub start_date: Option<chrono::NaiveDate>,
    pub due_date: Option<chrono::NaiveDate>,
}

pub async fn create_goal(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateGoalRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.performance_engine.create_goal(
        org_id, payload.document_id, payload.employee_id,
        &payload.goal_name, payload.description.as_deref(),
        payload.goal_category.as_deref(), &payload.weight,
        payload.target_metric.as_deref(),
        payload.start_date, payload.due_date, Some(user_id),
    ).await {
        Ok(goal) => Ok((StatusCode::CREATED, Json(serde_json::to_value(goal).unwrap()))),
        Err(e) => {
            error!("Failed to create goal: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn list_goals(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(document_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.performance_engine.list_goals(document_id).await {
        Ok(goals) => Ok(Json(serde_json::json!({"data": goals}))),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

#[derive(Debug, Deserialize)]
pub struct CompleteGoalRequest {
    pub actual_result: Option<String>,
}

pub async fn complete_goal(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<CompleteGoalRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.performance_engine.complete_goal(id, payload.actual_result.as_deref()).await {
        Ok(goal) => Ok(Json(serde_json::to_value(goal).unwrap())),
        Err(e) => {
            error!("Failed to complete goal: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct RateGoalRequest {
    pub rating_type: String,
    pub rating: String,
    pub comments: Option<String>,
}

pub async fn rate_goal(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<RateGoalRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.performance_engine.rate_goal(
        id, &payload.rating_type, &payload.rating, payload.comments.as_deref(),
    ).await {
        Ok(goal) => Ok(Json(serde_json::to_value(goal).unwrap())),
        Err(e) => {
            error!("Failed to rate goal: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn delete_goal(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    match state.performance_engine.delete_goal(id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            error!("Failed to delete goal: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// ============================================================================
// Competency Assessment Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct UpsertAssessmentRequest {
    pub document_id: Uuid,
    pub employee_id: Uuid,
    pub competency_id: Uuid,
    pub rating_type: String,
    pub rating: String,
    pub comments: Option<String>,
}

pub async fn upsert_competency_assessment(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<UpsertAssessmentRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.performance_engine.upsert_competency_assessment(
        org_id, payload.document_id, payload.employee_id,
        payload.competency_id, &payload.rating_type, &payload.rating,
        payload.comments.as_deref(), Some(user_id),
    ).await {
        Ok(assessment) => Ok(Json(serde_json::to_value(assessment).unwrap())),
        Err(e) => {
            error!("Failed to upsert assessment: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn list_competency_assessments(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(document_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.performance_engine.list_competency_assessments(document_id).await {
        Ok(assessments) => Ok(Json(serde_json::json!({"data": assessments}))),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// ============================================================================
// Feedback Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateFeedbackRequest {
    pub document_id: Option<Uuid>,
    pub employee_id: Uuid,
    pub from_user_name: Option<String>,
    pub feedback_type: String,
    pub subject: Option<String>,
    pub content: String,
    pub visibility: Option<String>,
}

pub async fn create_feedback(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateFeedbackRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let visibility = payload.visibility.as_deref().unwrap_or("manager_only");

    match state.performance_engine.create_feedback(
        org_id, payload.document_id, payload.employee_id,
        user_id, payload.from_user_name.as_deref(),
        &payload.feedback_type, payload.subject.as_deref(),
        &payload.content, visibility, Some(user_id),
    ).await {
        Ok(fb) => Ok((StatusCode::CREATED, Json(serde_json::to_value(fb).unwrap()))),
        Err(e) => {
            error!("Failed to create feedback: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListFeedbackQuery {
    pub employee_id: Option<Uuid>,
    pub document_id: Option<Uuid>,
}

pub async fn list_feedback(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListFeedbackQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.performance_engine.list_feedback(org_id, query.employee_id, query.document_id).await {
        Ok(fbs) => Ok(Json(serde_json::json!({"data": fbs}))),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn submit_feedback(
    State(state): State<Arc<AppState>>,
    _claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.performance_engine.submit_feedback(id).await {
        Ok(fb) => Ok(Json(serde_json::to_value(fb).unwrap())),
        Err(e) => {
            error!("Failed to submit feedback: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

// ============================================================================
// Dashboard
// ============================================================================

pub async fn get_performance_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(review_cycle_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.performance_engine.get_dashboard(org_id, review_cycle_id).await {
        Ok(dashboard) => Ok(Json(serde_json::to_value(dashboard).unwrap())),
        Err(e) => { error!("Error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}
