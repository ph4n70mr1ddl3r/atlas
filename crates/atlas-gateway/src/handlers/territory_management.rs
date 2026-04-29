//! Territory Management API Handlers
//!
//! Oracle Fusion CX Sales > Territory Management
//!
//! Endpoints for managing sales territories, hierarchy, members,
//! routing rules, quotas, and automatic lead/opportunity routing.

use axum::{
    extract::{Path, Query, State, Extension},
    Json,
    http::StatusCode,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;
use tracing::error;

use crate::AppState;
use crate::handlers::auth::Claims;

// ============================================================================
// Query Parameters
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ListTerritoriesQuery {
    pub territory_type: Option<String>,
    pub parent_id: Option<String>,
    pub include_inactive: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListMembersQuery {
    pub role: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListRulesQuery {
    pub entity_type: Option<String>,
}

// ============================================================================
// Territory CRUD
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTerritoryRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub territory_type: String,
    pub parent_id: Option<String>,
    pub owner_id: Option<String>,
    pub owner_name: Option<String>,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
}

pub async fn create_territory(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateTerritoryRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let parent_id = payload.parent_id
        .map(|s| Uuid::parse_str(&s))
        .transpose()
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let owner_id = payload.owner_id
        .map(|s| Uuid::parse_str(&s))
        .transpose()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    match state.territory_engine.create_territory(
        org_id,
        &payload.code,
        &payload.name,
        payload.description.as_deref(),
        &payload.territory_type,
        parent_id,
        owner_id,
        payload.owner_name.as_deref(),
        payload.effective_from,
        payload.effective_to,
        None,
    ).await {
        Ok(territory) => Ok((StatusCode::CREATED, Json(serde_json::to_value(territory).unwrap_or_else(|e| {
            error!("Serialization error: {}", e); serde_json::Value::Null
        })))),
        Err(e) => {
            error!("Failed to create territory: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                409 => StatusCode::CONFLICT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn get_territory(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let id = Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;
    match state.territory_engine.get_territory(id).await {
        Ok(Some(t)) => Ok(Json(serde_json::to_value(t).unwrap_or(serde_json::Value::Null))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Failed to get territory: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn list_territories(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Query(query): Query<ListTerritoriesQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let parent_id = query.parent_id
        .map(|s| Uuid::parse_str(&s))
        .transpose()
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let include_inactive = query.include_inactive
        .map(|s| s == "true" || s == "1")
        .unwrap_or(false);

    match state.territory_engine.list_territories(
        org_id,
        query.territory_type.as_deref(),
        parent_id,
        include_inactive,
    ).await {
        Ok(territories) => Ok(Json(serde_json::to_value(territories).unwrap_or(serde_json::Value::Null))),
        Err(e) => { error!("Failed to list territories: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTerritoryRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub territory_type: Option<String>,
    pub parent_id: Option<String>, // pass "null" to clear
    pub owner_id: Option<String>,
    pub owner_name: Option<String>,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
}

pub async fn update_territory(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateTerritoryRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let id = Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;
    let owner_id = payload.owner_id
        .map(|s| Uuid::parse_str(&s))
        .transpose()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // Handle parent_id: "null" means clear it, otherwise parse as UUID
    let parent_id: Option<Option<Uuid>> = if let Some(ref pid_str) = payload.parent_id {
        if pid_str == "null" || pid_str.is_empty() {
            Some(None)
        } else {
            Some(Some(Uuid::parse_str(pid_str).map_err(|_| StatusCode::BAD_REQUEST)?))
        }
    } else {
        None
    };

    match state.territory_engine.update_territory(
        id,
        payload.name.as_deref(),
        payload.description.as_deref(),
        payload.territory_type.as_deref(),
        parent_id,
        owner_id,
        payload.owner_name.as_deref(),
        payload.effective_from,
        payload.effective_to,
    ).await {
        Ok(t) => Ok(Json(serde_json::to_value(t).unwrap_or(serde_json::Value::Null))),
        Err(e) => {
            error!("Failed to update territory: {}", e);
            Err(match e.status_code() {
                400 => StatusCode::BAD_REQUEST,
                404 => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

pub async fn activate_territory(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let id = Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;
    match state.territory_engine.activate_territory(id).await {
        Ok(t) => Ok(Json(serde_json::to_value(t).unwrap_or(serde_json::Value::Null))),
        Err(e) => {
            error!("Failed to activate territory: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 404 => StatusCode::NOT_FOUND, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

pub async fn deactivate_territory(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let id = Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;
    match state.territory_engine.deactivate_territory(id).await {
        Ok(t) => Ok(Json(serde_json::to_value(t).unwrap_or(serde_json::Value::Null))),
        Err(e) => {
            error!("Failed to deactivate territory: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 404 => StatusCode::NOT_FOUND, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

pub async fn delete_territory(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let id = Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;
    match state.territory_engine.delete_territory(id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            error!("Failed to delete territory: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 404 => StatusCode::NOT_FOUND, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

// ============================================================================
// Territory Members
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddMemberRequest {
    pub user_id: String,
    pub user_name: String,
    pub role: String,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
}

pub async fn add_member(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(territory_id): Path<String>,
    Json(payload): Json<AddMemberRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let territory_id = Uuid::parse_str(&territory_id).map_err(|_| StatusCode::BAD_REQUEST)?;
    let user_id = Uuid::parse_str(&payload.user_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    match state.territory_engine.add_territory_member(
        org_id,
        territory_id,
        user_id,
        &payload.user_name,
        &payload.role,
        payload.effective_from,
        payload.effective_to,
        None,
    ).await {
        Ok(m) => Ok((StatusCode::CREATED, Json(serde_json::to_value(m).unwrap_or(serde_json::Value::Null)))),
        Err(e) => {
            error!("Failed to add member: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 404 => StatusCode::NOT_FOUND, 409 => StatusCode::CONFLICT, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

pub async fn list_members(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(territory_id): Path<String>,
    Query(query): Query<ListMembersQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let territory_id = Uuid::parse_str(&territory_id).map_err(|_| StatusCode::BAD_REQUEST)?;
    match state.territory_engine.list_territory_members(territory_id, query.role.as_deref()).await {
        Ok(members) => Ok(Json(serde_json::to_value(members).unwrap_or(serde_json::Value::Null))),
        Err(e) => { error!("Failed to list members: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn remove_member(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(member_id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let member_id = Uuid::parse_str(&member_id).map_err(|_| StatusCode::BAD_REQUEST)?;
    match state.territory_engine.remove_territory_member(member_id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => { error!("Failed to remove member: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// ============================================================================
// Territory Rules
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddRuleRequest {
    pub entity_type: String,
    pub field_name: String,
    pub match_operator: String,
    pub match_value: String,
    pub priority: i32,
}

pub async fn add_rule(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(territory_id): Path<String>,
    Json(payload): Json<AddRuleRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let territory_id = Uuid::parse_str(&territory_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    match state.territory_engine.add_territory_rule(
        org_id,
        territory_id,
        &payload.entity_type,
        &payload.field_name,
        &payload.match_operator,
        &payload.match_value,
        payload.priority,
        None,
    ).await {
        Ok(r) => Ok((StatusCode::CREATED, Json(serde_json::to_value(r).unwrap_or(serde_json::Value::Null)))),
        Err(e) => {
            error!("Failed to add rule: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 404 => StatusCode::NOT_FOUND, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

pub async fn list_rules(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(territory_id): Path<String>,
    Query(query): Query<ListRulesQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let territory_id = Uuid::parse_str(&territory_id).map_err(|_| StatusCode::BAD_REQUEST)?;
    match state.territory_engine.list_territory_rules(territory_id, query.entity_type.as_deref()).await {
        Ok(rules) => Ok(Json(serde_json::to_value(rules).unwrap_or(serde_json::Value::Null))),
        Err(e) => { error!("Failed to list rules: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn remove_rule(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(rule_id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let rule_id = Uuid::parse_str(&rule_id).map_err(|_| StatusCode::BAD_REQUEST)?;
    match state.territory_engine.remove_territory_rule(rule_id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => { error!("Failed to remove rule: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// ============================================================================
// Entity Routing
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteEntityRequest {
    pub entity_type: String,
    pub entity_data: serde_json::Value,
}

pub async fn route_entity(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<RouteEntityRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.territory_engine.route_entity(
        org_id,
        &payload.entity_type,
        &payload.entity_data,
    ).await {
        Ok(result) => Ok(Json(serde_json::to_value(result).unwrap_or(serde_json::Value::Null))),
        Err(e) => {
            error!("Failed to route entity: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

// ============================================================================
// Territory Quotas
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetQuotaRequest {
    pub period_name: String,
    pub period_start: chrono::NaiveDate,
    pub period_end: chrono::NaiveDate,
    pub revenue_quota: String,
    pub currency_code: String,
}

pub async fn set_quota(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Path(territory_id): Path<String>,
    Json(payload): Json<SetQuotaRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let territory_id = Uuid::parse_str(&territory_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    match state.territory_engine.set_territory_quota(
        org_id,
        territory_id,
        &payload.period_name,
        payload.period_start,
        payload.period_end,
        &payload.revenue_quota,
        &payload.currency_code,
        None,
    ).await {
        Ok(q) => Ok((StatusCode::CREATED, Json(serde_json::to_value(q).unwrap_or(serde_json::Value::Null)))),
        Err(e) => {
            error!("Failed to set quota: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, 404 => StatusCode::NOT_FOUND, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

pub async fn list_quotas(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(territory_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let territory_id = Uuid::parse_str(&territory_id).map_err(|_| StatusCode::BAD_REQUEST)?;
    match state.territory_engine.list_territory_quotas(territory_id).await {
        Ok(quotas) => Ok(Json(serde_json::to_value(quotas).unwrap_or(serde_json::Value::Null))),
        Err(e) => { error!("Failed to list quotas: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAttainmentRequest {
    pub actual_revenue: String,
}

pub async fn update_attainment(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(quota_id): Path<String>,
    Json(payload): Json<UpdateAttainmentRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let quota_id = Uuid::parse_str(&quota_id).map_err(|_| StatusCode::BAD_REQUEST)?;
    match state.territory_engine.update_quota_attainment(quota_id, &payload.actual_revenue).await {
        Ok(q) => Ok(Json(serde_json::to_value(q).unwrap_or(serde_json::Value::Null))),
        Err(e) => {
            error!("Failed to update attainment: {}", e);
            Err(match e.status_code() { 400 => StatusCode::BAD_REQUEST, _ => StatusCode::INTERNAL_SERVER_ERROR })
        }
    }
}

pub async fn delete_quota(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(quota_id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let quota_id = Uuid::parse_str(&quota_id).map_err(|_| StatusCode::BAD_REQUEST)?;
    match state.territory_engine.delete_territory_quota(quota_id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => { error!("Failed to delete quota: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// ============================================================================
// Dashboard
// ============================================================================

pub async fn get_territory_dashboard(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.territory_engine.get_dashboard(org_id).await {
        Ok(dashboard) => Ok(Json(serde_json::to_value(dashboard).unwrap_or(serde_json::Value::Null))),
        Err(e) => { error!("Failed to get territory dashboard: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}
