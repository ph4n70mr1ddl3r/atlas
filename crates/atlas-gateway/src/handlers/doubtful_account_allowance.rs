//! Doubtful Account Allowance API Handlers
//!
//! REST endpoints for Oracle Fusion-inspired Allowance for Doubtful Accounts.
//! Manages provision policies, aging buckets, provision runs, and posting.
//!
//! Oracle Fusion equivalent: Receivables > Collections > Allowance for Doubtful Accounts

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
use crate::handlers::auth::{Claims, parse_uuid};

// ============================================================================
// Request / Response Types
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePolicyRequest {
    pub policy_code: String,
    pub policy_name: String,
    pub description: Option<String>,
    pub calculation_method: String,
    pub flat_percentage: Option<String>,
    pub default_provision_account: Option<String>,
    pub default_expense_account: Option<String>,
    pub currency_code: Option<String>,
    pub effective_from: String,
    pub effective_to: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAgingBucketRequest {
    pub bucket_name: String,
    pub from_days: i32,
    pub to_days: Option<i32>,
    pub provision_percentage: String,
    pub display_order: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProvisionRunRequest {
    pub policy_id: Uuid,
    pub as_of_date: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostProvisionRequest {
    pub journal_entry_number: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListPoliciesQuery {
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListProvisionRunsQuery {
    pub policy_id: Option<Uuid>,
    pub status: Option<String>,
}

// ============================================================================
// Policy Handlers
// ============================================================================

pub async fn create_policy(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreatePolicyRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    let user_id = parse_uuid(&claims.sub).ok();

    let effective_from = chrono::NaiveDate::parse_from_str(&req.effective_from, "%Y-%m-%d")
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(serde_json::json!({
            "error": format!("Invalid effective_from date: {}", e)
        }))))?;

    let effective_to = match req.effective_to.as_deref() {
        Some(d) => Some(chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d")
            .map_err(|e| (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "error": format!("Invalid effective_to date: {}", e)
            }))))?),
        None => None,
    };

    match state.doubtful_account_engine.create_policy(
        org_id,
        &req.policy_code,
        &req.policy_name,
        req.description.as_deref(),
        &req.calculation_method,
        req.flat_percentage.as_deref().unwrap_or("0"),
        req.default_provision_account.as_deref(),
        req.default_expense_account.as_deref(),
        req.currency_code.as_deref().unwrap_or("USD"),
        effective_from,
        effective_to,
        user_id,
    ).await {
        Ok(policy) => Ok((StatusCode::CREATED, Json(serde_json::to_value(policy).unwrap()))),
        Err(e) => {
            error!("Failed to create policy: {}", e);
            Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

pub async fn get_policy(
    State(_state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let _org_id = parse_uuid(&claims.org_id)?;

    match _state.doubtful_account_engine.get_policy(id).await {
        Ok(Some(policy)) => Ok(Json(serde_json::to_value(policy).unwrap())),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Policy not found"})))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()})))),
    }
}

pub async fn get_policy_by_code(
    State(_state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(code): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;

    match _state.doubtful_account_engine.get_policy_by_code(org_id, &code).await {
        Ok(Some(policy)) => Ok(Json(serde_json::to_value(policy).unwrap())),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Policy not found"})))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()})))),
    }
}

pub async fn list_policies(
    State(_state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<ListPoliciesQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;

    match _state.doubtful_account_engine.list_policies(org_id, query.status.as_deref()).await {
        Ok(policies) => Ok(Json(serde_json::json!({"data": policies}))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()})))),
    }
}

pub async fn deactivate_policy(
    State(_state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = parse_uuid(&claims.sub).ok();

    match _state.doubtful_account_engine.deactivate_policy(id, user_id).await {
        Ok(policy) => Ok(Json(serde_json::to_value(policy).unwrap())),
        Err(e) => Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()})))),
    }
}

pub async fn reactivate_policy(
    State(_state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = parse_uuid(&claims.sub).ok();

    match _state.doubtful_account_engine.reactivate_policy(id, user_id).await {
        Ok(policy) => Ok(Json(serde_json::to_value(policy).unwrap())),
        Err(e) => Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Aging Bucket Handlers
// ============================================================================

pub async fn create_aging_bucket(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(policy_id): Path<Uuid>,
    Json(req): Json<CreateAgingBucketRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;

    match state.doubtful_account_engine.create_aging_bucket(
        org_id,
        policy_id,
        &req.bucket_name,
        req.from_days,
        req.to_days,
        &req.provision_percentage,
        req.display_order.unwrap_or(0),
    ).await {
        Ok(bucket) => Ok((StatusCode::CREATED, Json(serde_json::to_value(bucket).unwrap()))),
        Err(e) => {
            error!("Failed to create aging bucket: {}", e);
            Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

pub async fn list_aging_buckets(
    State(_state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(policy_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match _state.doubtful_account_engine.list_aging_buckets_for_policy(policy_id).await {
        Ok(buckets) => Ok(Json(serde_json::json!({"data": buckets}))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Provision Run Handlers
// ============================================================================

pub async fn create_provision_run(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreateProvisionRunRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    let user_id = parse_uuid(&claims.sub).ok();

    let as_of_date = chrono::NaiveDate::parse_from_str(&req.as_of_date, "%Y-%m-%d")
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(serde_json::json!({
            "error": format!("Invalid as_of_date: {}", e)
        }))))?;

    match state.doubtful_account_engine.create_provision_run(
        org_id,
        req.policy_id,
        as_of_date,
        req.description.as_deref(),
        user_id,
    ).await {
        Ok(run) => Ok((StatusCode::CREATED, Json(serde_json::to_value(run).unwrap()))),
        Err(e) => {
            error!("Failed to create provision run: {}", e);
            Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

pub async fn get_provision_run(
    State(_state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match _state.doubtful_account_engine.get_provision_run(id).await {
        Ok(Some(run)) => Ok(Json(serde_json::to_value(run).unwrap())),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Provision run not found"})))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()})))),
    }
}

pub async fn get_provision_run_by_number(
    State(_state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(number): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;

    match _state.doubtful_account_engine.get_provision_run_by_number(org_id, &number).await {
        Ok(Some(run)) => Ok(Json(serde_json::to_value(run).unwrap())),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Provision run not found"})))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()})))),
    }
}

pub async fn list_provision_runs(
    State(_state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<ListProvisionRunsQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;

    match _state.doubtful_account_engine.list_provision_runs(
        org_id,
        query.policy_id,
        query.status.as_deref(),
    ).await {
        Ok(runs) => Ok(Json(serde_json::json!({"data": runs}))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()})))),
    }
}

pub async fn calculate_provision(
    State(_state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = parse_uuid(&claims.sub).ok();

    match _state.doubtful_account_engine.calculate_provision(id, user_id).await {
        Ok(run) => Ok(Json(serde_json::to_value(run).unwrap())),
        Err(e) => Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()})))),
    }
}

pub async fn post_provision(
    State(_state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(req): Json<PostProvisionRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = parse_uuid(&claims.sub).ok();

    match _state.doubtful_account_engine.post_provision(
        id, user_id, req.journal_entry_number.as_deref(),
    ).await {
        Ok(run) => Ok(Json(serde_json::to_value(run).unwrap())),
        Err(e) => Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()})))),
    }
}

pub async fn reverse_provision(
    State(_state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = parse_uuid(&claims.sub).ok();

    match _state.doubtful_account_engine.reverse_provision(id, user_id).await {
        Ok(run) => Ok(Json(serde_json::to_value(run).unwrap())),
        Err(e) => Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()})))),
    }
}

pub async fn cancel_provision(
    State(_state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = parse_uuid(&claims.sub).ok();

    match _state.doubtful_account_engine.cancel_provision(id, user_id).await {
        Ok(run) => Ok(Json(serde_json::to_value(run).unwrap())),
        Err(e) => Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()})))),
    }
}

pub async fn list_provision_details(
    State(_state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match _state.doubtful_account_engine.list_provision_details(id).await {
        Ok(details) => Ok(Json(serde_json::json!({"data": details}))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()})))),
    }
}

pub async fn list_run_activities(
    State(_state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match _state.doubtful_account_engine.list_activities(id).await {
        Ok(activities) => Ok(Json(serde_json::json!({"data": activities}))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Dashboard
// ============================================================================

pub async fn get_dashboard(
    State(_state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;

    match _state.doubtful_account_engine.get_dashboard(org_id).await {
        Ok(dashboard) => Ok(Json(serde_json::to_value(dashboard).unwrap())),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()})))),
    }
}
