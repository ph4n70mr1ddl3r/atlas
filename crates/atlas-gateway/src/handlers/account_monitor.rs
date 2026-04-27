//! Account Monitor & Balance Inquiry Handlers
//!
//! Oracle Fusion General Ledger > Account Monitor endpoints:
//! - Account group CRUD
//! - Group member management
//! - Balance snapshot capture and retrieval
//! - Saved balance inquiry management
//! - Account monitor dashboard summary

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
// Account Groups
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAccountGroupRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub is_shared: Option<bool>,
    pub threshold_warning_pct: Option<String>,
    pub threshold_critical_pct: Option<String>,
    pub comparison_type: Option<String>,
}

/// Create an account group
pub async fn create_account_group(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateAccountGroupRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let group = state.account_monitor_engine.create_account_group(
        org_id,
        &payload.code,
        &payload.name,
        payload.description.as_deref(),
        Some(user_id),
        payload.is_shared.unwrap_or(false),
        payload.threshold_warning_pct.as_deref(),
        payload.threshold_critical_pct.as_deref(),
        payload.comparison_type.as_deref().unwrap_or("prior_period"),
        Some(user_id),
    ).await.map_err(|e| {
        error!("Create account group error: {}", e);
        match e {
            atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(group).unwrap_or_default())))
}

/// Get an account group by ID
pub async fn get_account_group(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let group = state.account_monitor_engine.get_account_group(id).await
        .map_err(|e| { error!("Get account group error: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;
    match group {
        Some(g) => Ok(Json(serde_json::to_value(g).unwrap_or_default())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// List account groups
pub async fn list_account_groups(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let groups = state.account_monitor_engine.list_account_groups(org_id, None).await
        .map_err(|e| { error!("List account groups error: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;

    Ok(Json(serde_json::json!({
        "data": groups,
        "meta": { "total": groups.len() }
    })))
}

/// Delete an account group by code
pub async fn delete_account_group(
    State(state): State<Arc<AppState>>,
    Path(code): Path<String>,
    claims: Extension<Claims>,
) -> Result<StatusCode, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    state.account_monitor_engine.delete_account_group(org_id, &code).await.map_err(|e| {
        error!("Delete account group error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Group Members
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddMemberRequest {
    pub account_segment: String,
    pub account_label: Option<String>,
    pub display_order: Option<i32>,
    pub include_children: Option<bool>,
}

/// Add a member to an account group
pub async fn add_group_member(
    State(state): State<Arc<AppState>>,
    Path(group_id): Path<Uuid>,
    claims: Extension<Claims>,
    Json(payload): Json<AddMemberRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let _org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let member = state.account_monitor_engine.add_group_member(
        group_id,
        &payload.account_segment,
        payload.account_label.as_deref(),
        payload.display_order.unwrap_or(0),
        payload.include_children.unwrap_or(true),
    ).await.map_err(|e| {
        error!("Add member error: {}", e);
        match e {
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(member).unwrap_or_default())))
}

/// Remove a member from an account group
pub async fn remove_group_member(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    state.account_monitor_engine.remove_group_member(id).await.map_err(|e| {
        error!("Remove member error: {}", e);
        match e {
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;
    Ok(StatusCode::NO_CONTENT)
}

/// List members of an account group
pub async fn list_group_members(
    State(state): State<Arc<AppState>>,
    Path(group_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let members = state.account_monitor_engine.list_group_members(group_id).await
        .map_err(|e| { error!("List members error: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;

    Ok(Json(serde_json::json!({
        "data": members,
        "meta": { "groupId": group_id, "total": members.len() }
    })))
}

// ============================================================================
// Balance Snapshots
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureSnapshotRequest {
    pub period_name: String,
    pub period_start: String,
    pub period_end: String,
    pub fiscal_year: i32,
    pub period_number: i32,
}

/// Capture balance snapshots for all members of a group
pub async fn capture_snapshot(
    State(state): State<Arc<AppState>>,
    Path(group_id): Path<Uuid>,
    claims: Extension<Claims>,
    Json(payload): Json<CaptureSnapshotRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let period_start = chrono::NaiveDate::parse_from_str(&payload.period_start, "%Y-%m-%d")
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let period_end = chrono::NaiveDate::parse_from_str(&payload.period_end, "%Y-%m-%d")
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let snapshots = state.account_monitor_engine.capture_snapshot(
        org_id, group_id, &payload.period_name,
        period_start, period_end,
        payload.fiscal_year, payload.period_number,
    ).await.map_err(|e| {
        error!("Capture snapshot error: {}", e);
        match e {
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;

    Ok((StatusCode::CREATED, Json(serde_json::json!({
        "data": snapshots,
        "meta": { "groupId": group_id, "total": snapshots.len() }
    }))))
}

#[derive(Debug, Deserialize)]
pub struct ListSnapshotsParams {
    pub snapshot_date: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Get balance snapshots for a group
pub async fn list_snapshots(
    State(state): State<Arc<AppState>>,
    Path(group_id): Path<Uuid>,
    Query(params): Query<ListSnapshotsParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let limit = params.limit.unwrap_or(50).clamp(1, 200);
    let offset = params.offset.unwrap_or(0).max(0);
    let snapshot_date = params.snapshot_date
        .as_deref()
        .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());

    let snapshots = state.account_monitor_engine.get_group_snapshots(
        group_id, snapshot_date, limit, offset,
    ).await.map_err(|e| { error!("List snapshots error: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;

    Ok(Json(serde_json::json!({
        "data": snapshots,
        "meta": { "groupId": group_id, "limit": limit, "offset": offset }
    })))
}

/// Get snapshots with alerts
pub async fn get_alert_snapshots(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let snapshots = state.account_monitor_engine.get_alert_snapshots(org_id).await
        .map_err(|e| { error!("Get alerts error: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;

    Ok(Json(serde_json::json!({
        "data": snapshots,
        "meta": { "total": snapshots.len() }
    })))
}

/// Delete a balance snapshot
pub async fn delete_snapshot(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    state.account_monitor_engine.delete_snapshot(id).await.map_err(|e| {
        error!("Delete snapshot error: {}", e);
        match e {
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Saved Balance Inquiries
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSavedInquiryRequest {
    pub name: String,
    pub description: Option<String>,
    pub account_segments: Option<serde_json::Value>,
    pub period_from: String,
    pub period_to: String,
    pub currency_code: Option<String>,
    pub amount_type: Option<String>,
    pub include_zero_balances: Option<bool>,
    pub comparison_enabled: Option<bool>,
    pub comparison_type: Option<String>,
    pub sort_by: Option<String>,
    pub sort_direction: Option<String>,
    pub is_shared: Option<bool>,
}

/// Create a saved balance inquiry
pub async fn create_saved_inquiry(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
    Json(payload): Json<CreateSavedInquiryRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let inquiry = state.account_monitor_engine.create_saved_inquiry(
        org_id, user_id,
        &payload.name, payload.description.as_deref(),
        payload.account_segments.clone().unwrap_or(serde_json::json!([])),
        &payload.period_from, &payload.period_to,
        payload.currency_code.as_deref().unwrap_or("USD"),
        payload.amount_type.as_deref().unwrap_or("ending_balance"),
        payload.include_zero_balances.unwrap_or(false),
        payload.comparison_enabled.unwrap_or(false),
        payload.comparison_type.as_deref(),
        payload.sort_by.as_deref().unwrap_or("account_segment"),
        payload.sort_direction.as_deref().unwrap_or("asc"),
        payload.is_shared.unwrap_or(false),
    ).await.map_err(|e| {
        error!("Create saved inquiry error: {}", e);
        match e {
            atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;

    Ok((StatusCode::CREATED, Json(serde_json::to_value(inquiry).unwrap_or_default())))
}

/// Get a saved balance inquiry
pub async fn get_saved_inquiry(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let inquiry = state.account_monitor_engine.get_saved_inquiry(id).await
        .map_err(|e| { error!("Get saved inquiry error: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;
    match inquiry {
        Some(i) => Ok(Json(serde_json::to_value(i).unwrap_or_default())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// List saved balance inquiries
pub async fn list_saved_inquiries(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let inquiries = state.account_monitor_engine.list_saved_inquiries(org_id, None).await
        .map_err(|e| { error!("List saved inquiries error: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;

    Ok(Json(serde_json::json!({
        "data": inquiries,
        "meta": { "total": inquiries.len() }
    })))
}

/// Delete a saved balance inquiry
pub async fn delete_saved_inquiry(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    state.account_monitor_engine.delete_saved_inquiry(id).await.map_err(|e| {
        error!("Delete saved inquiry error: {}", e);
        match e {
            atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Account Monitor Dashboard Summary
// ============================================================================

/// Get the account monitor dashboard summary
pub async fn get_account_monitor_summary(
    State(state): State<Arc<AppState>>,
    claims: Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let summary = state.account_monitor_engine.get_monitor_summary(org_id).await
        .map_err(|e| { error!("Account monitor summary error: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;

    Ok(Json(serde_json::to_value(summary).unwrap_or_default()))
}
