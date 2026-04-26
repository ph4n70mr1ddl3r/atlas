//! Supplier Scorecard Management Handlers
//!
//! Oracle Fusion Supplier Portal: Supplier Performance Management.
//! HTTP handlers for scorecard templates, categories, scorecards,
//! KPI lines, performance reviews, and action items.

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

#[derive(Debug, Deserialize)]
pub struct ListTemplatesQuery { pub evaluation_period: Option<String> }

pub async fn create_template(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let code = payload.get("code").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let name = payload.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let description = payload.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());
    let evaluation_period = payload.get("evaluation_period").and_then(|v| v.as_str()).unwrap_or("quarterly").to_string();
    let result = state.scorecard_engine.create_template(
        org_id, &code, &name, description.as_deref(), &evaluation_period, None,
    ).await.map_err(|e| match e {
        atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
        atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    })?;
    Ok((StatusCode::CREATED, Json(serde_json::to_value(result).unwrap_or_default())))
}

pub async fn list_templates(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let templates = state.scorecard_engine.list_templates(org_id).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({ "data": templates })))
}

pub async fn get_template(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let tmpl = state.scorecard_engine.get_template(id).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(serde_json::to_value(tmpl).unwrap_or_default()))
}

pub async fn delete_template(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(code): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    state.scorecard_engine.delete_template(org_id, &code).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::NO_CONTENT)
}

// Categories
pub async fn create_category(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let template_id: Uuid = payload.get("template_id").and_then(|v| v.as_str()).unwrap_or("").parse().map_err(|_| StatusCode::BAD_REQUEST)?;
    let code = payload.get("code").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let name = payload.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let description = payload.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());
    let weight = payload.get("weight").and_then(|v| v.as_str()).unwrap_or("0").to_string();
    let sort_order = payload.get("sort_order").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let scoring_model = payload.get("scoring_model").and_then(|v| v.as_str()).unwrap_or("manual").to_string();
    let target_score = payload.get("target_score").and_then(|v| v.as_str()).map(|s| s.to_string());
    let result = state.scorecard_engine.create_category(
        org_id, template_id, &code, &name, description.as_deref(),
        &weight, sort_order, &scoring_model, target_score.as_deref(), None,
    ).await.map_err(|e| match e {
        atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
        atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    })?;
    Ok((StatusCode::CREATED, Json(serde_json::to_value(result).unwrap_or_default())))
}

pub async fn list_categories(
    State(state): State<Arc<AppState>>,
    Path(template_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let categories = state.scorecard_engine.list_categories(template_id).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({ "data": categories })))
}

pub async fn delete_category(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    state.scorecard_engine.delete_category(id).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::NO_CONTENT)
}

// Scorecards
pub async fn create_scorecard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let template_id: Uuid = payload.get("template_id").and_then(|v| v.as_str()).unwrap_or("").parse().map_err(|_| StatusCode::BAD_REQUEST)?;
    let scorecard_number = payload.get("scorecard_number").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let supplier_id: Uuid = payload.get("supplier_id").and_then(|v| v.as_str()).unwrap_or("").parse().map_err(|_| StatusCode::BAD_REQUEST)?;
    let supplier_name = payload.get("supplier_name").and_then(|v| v.as_str()).map(|s| s.to_string());
    let supplier_number = payload.get("supplier_number").and_then(|v| v.as_str()).map(|s| s.to_string());
    let period_start: chrono::NaiveDate = payload.get("evaluation_period_start").and_then(|v| v.as_str()).unwrap_or("2024-01-01").parse().map_err(|_| StatusCode::BAD_REQUEST)?;
    let period_end: chrono::NaiveDate = payload.get("evaluation_period_end").and_then(|v| v.as_str()).unwrap_or("2024-03-31").parse().map_err(|_| StatusCode::BAD_REQUEST)?;
    let notes = payload.get("notes").and_then(|v| v.as_str()).map(|s| s.to_string());
    let result = state.scorecard_engine.create_scorecard(
        org_id, template_id, &scorecard_number, supplier_id,
        supplier_name.as_deref(), supplier_number.as_deref(),
        period_start, period_end, notes.as_deref(), None,
    ).await.map_err(|e| match e {
        atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
        atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
        atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    })?;
    Ok((StatusCode::CREATED, Json(serde_json::to_value(result).unwrap_or_default())))
}

#[derive(Debug, Deserialize)]
pub struct ListScorecardsQuery {
    pub supplier_id: Option<Uuid>,
    pub status: Option<String>,
}

pub async fn list_scorecards(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<ListScorecardsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let scorecards = state.scorecard_engine.list_scorecards(org_id, params.supplier_id, params.status.as_deref()).await.map_err(|e| match e {
        atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    })?;
    Ok(Json(serde_json::json!({ "data": scorecards })))
}

pub async fn get_scorecard(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let sc = state.scorecard_engine.get_scorecard(id).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(serde_json::to_value(sc).unwrap_or_default()))
}

pub async fn submit_scorecard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let reviewer_name = payload.get("reviewer_name").and_then(|v| v.as_str()).map(|s| s.to_string());
    let result = state.scorecard_engine.submit_scorecard(id, Some(user_id), reviewer_name.as_deref()).await.map_err(|e| match e {
        atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
        atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    })?;
    Ok(Json(serde_json::to_value(result).unwrap_or_default()))
}

pub async fn approve_scorecard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let result = state.scorecard_engine.approve_scorecard(id, Some(user_id)).await.map_err(|e| match e {
        atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
        atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    })?;
    Ok(Json(serde_json::to_value(result).unwrap_or_default()))
}

pub async fn reject_scorecard(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let result = state.scorecard_engine.reject_scorecard(id).await.map_err(|e| match e {
        atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
        atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    })?;
    Ok(Json(serde_json::to_value(result).unwrap_or_default()))
}

pub async fn delete_scorecard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(scorecard_number): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    state.scorecard_engine.delete_scorecard(org_id, &scorecard_number).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::NO_CONTENT)
}

// Scorecard Lines
pub async fn add_scorecard_line(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(scorecard_id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let category_id: Uuid = payload.get("category_id").and_then(|v| v.as_str()).unwrap_or("").parse().map_err(|_| StatusCode::BAD_REQUEST)?;
    let kpi_name = payload.get("kpi_name").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let kpi_description = payload.get("kpi_description").and_then(|v| v.as_str()).map(|s| s.to_string());
    let weight = payload.get("weight").and_then(|v| v.as_str()).unwrap_or("0").to_string();
    let target_value = payload.get("target_value").and_then(|v| v.as_str()).map(|s| s.to_string());
    let actual_value = payload.get("actual_value").and_then(|v| v.as_str()).map(|s| s.to_string());
    let score = payload.get("score").and_then(|v| v.as_str()).unwrap_or("0").to_string();
    let evidence = payload.get("evidence").and_then(|v| v.as_str()).map(|s| s.to_string());
    let notes = payload.get("notes").and_then(|v| v.as_str()).map(|s| s.to_string());
    let result = state.scorecard_engine.add_scorecard_line(
        org_id, scorecard_id, category_id, &kpi_name, kpi_description.as_deref(),
        &weight, target_value.as_deref(), actual_value.as_deref(),
        &score, evidence.as_deref(), notes.as_deref(),
    ).await.map_err(|e| match e {
        atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
        atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
        atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    })?;
    Ok((StatusCode::CREATED, Json(serde_json::to_value(result).unwrap_or_default())))
}

pub async fn list_scorecard_lines(
    State(state): State<Arc<AppState>>,
    Path(scorecard_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let lines = state.scorecard_engine.list_scorecard_lines(scorecard_id).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({ "data": lines })))
}

pub async fn delete_scorecard_line(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    state.scorecard_engine.delete_scorecard_line(id).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::NO_CONTENT)
}

// Performance Reviews
pub async fn create_review(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let review_number = payload.get("review_number").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let supplier_id: Uuid = payload.get("supplier_id").and_then(|v| v.as_str()).unwrap_or("").parse().map_err(|_| StatusCode::BAD_REQUEST)?;
    let supplier_name = payload.get("supplier_name").and_then(|v| v.as_str()).map(|s| s.to_string());
    let scorecard_id = payload.get("scorecard_id").and_then(|v| v.as_str()).and_then(|s| s.parse::<Uuid>().ok());
    let review_type = payload.get("review_type").and_then(|v| v.as_str()).unwrap_or("periodic").to_string();
    let review_period = payload.get("review_period").and_then(|v| v.as_str()).map(|s| s.to_string());
    let period_start: chrono::NaiveDate = payload.get("period_start").and_then(|v| v.as_str()).unwrap_or("2024-01-01").parse().map_err(|_| StatusCode::BAD_REQUEST)?;
    let period_end: chrono::NaiveDate = payload.get("period_end").and_then(|v| v.as_str()).unwrap_or("2024-03-31").parse().map_err(|_| StatusCode::BAD_REQUEST)?;
    let result = state.scorecard_engine.create_review(
        org_id, &review_number, supplier_id, supplier_name.as_deref(),
        scorecard_id, &review_type, review_period.as_deref(),
        period_start, period_end, None,
    ).await.map_err(|e| match e {
        atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
        atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    })?;
    Ok((StatusCode::CREATED, Json(serde_json::to_value(result).unwrap_or_default())))
}

#[derive(Debug, Deserialize)]
pub struct ListReviewsQuery {
    pub supplier_id: Option<Uuid>,
    pub status: Option<String>,
}

pub async fn list_reviews(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(params): Query<ListReviewsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let reviews = state.scorecard_engine.list_reviews(org_id, params.supplier_id, params.status.as_deref()).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({ "data": reviews })))
}

pub async fn get_review(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let review = state.scorecard_engine.get_review(id).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(serde_json::to_value(review).unwrap_or_default()))
}

pub async fn complete_review(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let current_score = payload.get("current_score").and_then(|v| v.as_str()).map(|s| s.to_string());
    let rating = payload.get("rating").and_then(|v| v.as_str()).map(|s| s.to_string());
    let strengths = payload.get("strengths").and_then(|v| v.as_str()).map(|s| s.to_string());
    let improvement_areas = payload.get("improvement_areas").and_then(|v| v.as_str()).map(|s| s.to_string());
    let action_items = payload.get("action_items").and_then(|v| v.as_str()).map(|s| s.to_string());
    let reviewer_name = payload.get("reviewer_name").and_then(|v| v.as_str()).map(|s| s.to_string());
    let result = state.scorecard_engine.complete_review(
        id, current_score.as_deref(), rating.as_deref(), strengths.as_deref(),
        improvement_areas.as_deref(), action_items.as_deref(),
        Some(user_id), reviewer_name.as_deref(),
    ).await.map_err(|e| match e {
        atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
        atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
        atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    })?;
    Ok(Json(serde_json::to_value(result).unwrap_or_default()))
}

pub async fn delete_review(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(review_number): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    state.scorecard_engine.delete_review(org_id, &review_number).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::NO_CONTENT)
}

// Action Items
pub async fn create_action_item(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(review_id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let description = payload.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let assignee_id = payload.get("assignee_id").and_then(|v| v.as_str()).and_then(|s| s.parse::<Uuid>().ok());
    let assignee_name = payload.get("assignee_name").and_then(|v| v.as_str()).map(|s| s.to_string());
    let priority = payload.get("priority").and_then(|v| v.as_str()).unwrap_or("medium").to_string();
    let due_date = payload.get("due_date").and_then(|v| v.as_str()).and_then(|s| s.parse::<chrono::NaiveDate>().ok());
    let result = state.scorecard_engine.create_action_item(
        org_id, review_id, &description, assignee_id, assignee_name.as_deref(),
        &priority, due_date, None,
    ).await.map_err(|e| match e {
        atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
        atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    })?;
    Ok((StatusCode::CREATED, Json(serde_json::to_value(result).unwrap_or_default())))
}

pub async fn list_action_items(
    State(state): State<Arc<AppState>>,
    Path(review_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let items = state.scorecard_engine.list_action_items(review_id).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({ "data": items })))
}

pub async fn complete_action_item(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let result = state.scorecard_engine.complete_action_item(id).await.map_err(|e| match e {
        atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
        atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    })?;
    Ok(Json(serde_json::to_value(result).unwrap_or_default()))
}

pub async fn delete_action_item(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    state.scorecard_engine.delete_action_item(id).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::NO_CONTENT)
}

// Dashboard
pub async fn get_scorecard_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let dashboard = state.scorecard_engine.get_dashboard(org_id).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::to_value(dashboard).unwrap_or_default()))
}
