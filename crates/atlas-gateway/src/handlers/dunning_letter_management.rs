//! Dunning Letter Management API Handlers
//!
//! REST endpoints for Oracle Fusion-inspired Dunning Letter Management.
//! Manages escalating payment reminder letters for overdue customer invoices.
//!
//! Oracle Fusion: Financials > Receivables > Dunning Letters

use crate::handlers::{to_json, created_json};
use axum::{
    extract::{State, Path, Query, Extension},
    Json,
    http::StatusCode,
};
use serde::Deserialize;
use crate::AppState;
use crate::handlers::auth::Claims;
use std::sync::Arc;
use uuid::Uuid;
use tracing::error;
use sqlx::Row;

// ============================================================================
// Letter Set CRUD Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateLetterSetRequest {
    pub set_name: String,
    pub description: Option<String>,
    pub number_of_levels: Option<i32>,
    pub minimum_overdue_days: Option<i32>,
    pub currency_code: Option<String>,
    pub include_finance_charges: Option<bool>,
    pub include_unapplied_receipts: Option<bool>,
    pub aging_basis: Option<String>,
}

pub async fn create_letter_set(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreateLetterSetRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.financials.dunning_letter_management_engine.create_letter_set(
        org_id,
        &req.set_name,
        req.description.as_deref(),
        req.number_of_levels.unwrap_or(3),
        req.minimum_overdue_days.unwrap_or(1),
        req.currency_code.as_deref().unwrap_or("USD"),
        req.include_finance_charges.unwrap_or(false),
        req.include_unapplied_receipts.unwrap_or(false),
        req.aging_basis.as_deref().unwrap_or("days_overdue"),
        Some(user_id),
    ).await {
        Ok(set) => Ok(created_json(set)),
        Err(e) => {
            error!("Failed to create dunning letter set: {}", e);
            Err(map_error(e))
        }
    }
}

pub async fn get_letter_set(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.financials.dunning_letter_management_engine.get_letter_set(id).await {
        Ok(Some(s)) => Ok(to_json(s)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Failed to get letter set: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListLetterSetsQuery {
    pub status: Option<String>,
}

pub async fn list_letter_sets(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<ListLetterSetsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.financials.dunning_letter_management_engine.list_letter_sets(org_id, query.status.as_deref()).await {
        Ok(sets) => Ok(Json(serde_json::json!({ "data": sets }))),
        Err(e) => {
            error!("Failed to list letter sets: {}", e);
            Err(map_error(e))
        }
    }
}

pub async fn activate_letter_set(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.financials.dunning_letter_management_engine.activate_letter_set(id).await {
        Ok(s) => Ok(to_json(s)),
        Err(e) => {
            error!("Failed to activate letter set: {}", e);
            Err(map_error(e))
        }
    }
}

pub async fn deactivate_letter_set(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.financials.dunning_letter_management_engine.deactivate_letter_set(id).await {
        Ok(s) => Ok(to_json(s)),
        Err(e) => {
            error!("Failed to deactivate letter set: {}", e);
            Err(map_error(e))
        }
    }
}

// ============================================================================
// Letter Set Line Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct AddLetterSetLineRequest {
    pub level_number: i32,
    pub level_name: String,
    pub min_days_overdue: Option<i32>,
    pub max_days_overdue: Option<i32>,
    pub minimum_amount: Option<String>,
    pub letter_template: Option<String>,
    pub delivery_method: Option<String>,
    pub apply_credit_hold: Option<bool>,
    pub assess_finance_charges: Option<bool>,
    pub letter_text: Option<String>,
    pub escalation_days: Option<i32>,
}

pub async fn add_letter_set_line(
    State(state): State<Arc<AppState>>,
    Path(set_id): Path<Uuid>,
    Json(req): Json<AddLetterSetLineRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    match state.financials.dunning_letter_management_engine.add_letter_set_line(
        set_id,
        req.level_number,
        &req.level_name,
        req.min_days_overdue.unwrap_or(0),
        req.max_days_overdue,
        req.minimum_amount.as_deref().unwrap_or("0"),
        req.letter_template.as_deref(),
        req.delivery_method.as_deref().unwrap_or("print"),
        req.apply_credit_hold.unwrap_or(false),
        req.assess_finance_charges.unwrap_or(false),
        req.letter_text.as_deref(),
        req.escalation_days,
    ).await {
        Ok(line) => Ok(created_json(line)),
        Err(e) => {
            error!("Failed to add letter set line: {}", e);
            Err(map_error(e))
        }
    }
}

pub async fn list_letter_set_lines(
    State(state): State<Arc<AppState>>,
    Path(set_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.financials.dunning_letter_management_engine.list_letter_set_lines(set_id).await {
        Ok(lines) => Ok(Json(serde_json::json!({ "data": lines }))),
        Err(e) => {
            error!("Failed to list letter set lines: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// ============================================================================
// Dunning Profile Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateProfileRequest {
    pub customer_id: Uuid,
    pub customer_name: Option<String>,
    pub letter_set_id: Option<Uuid>,
    pub minimum_overdue_amount: Option<String>,
    pub contact_name: Option<String>,
    pub contact_email: Option<String>,
    pub preferred_delivery_method: Option<String>,
    pub notes: Option<String>,
}

pub async fn create_profile(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreateProfileRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match state.financials.dunning_letter_management_engine.create_profile(
        org_id,
        req.customer_id,
        req.customer_name.as_deref(),
        req.letter_set_id,
        req.minimum_overdue_amount.as_deref().unwrap_or("0"),
        req.contact_name.as_deref(),
        req.contact_email.as_deref(),
        req.preferred_delivery_method.as_deref(),
        req.notes.as_deref(),
        Some(user_id),
    ).await {
        Ok(profile) => Ok(created_json(profile)),
        Err(e) => {
            error!("Failed to create dunning profile: {}", e);
            Err(map_error(e))
        }
    }
}

pub async fn get_profile(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.financials.dunning_letter_management_engine.get_profile(id).await {
        Ok(Some(p)) => Ok(to_json(p)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Failed to get profile: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListProfilesQuery {
    pub status: Option<String>,
}

pub async fn list_profiles(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<ListProfilesQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.financials.dunning_letter_management_engine.list_profiles(org_id, query.status.as_deref()).await {
        Ok(profiles) => Ok(Json(serde_json::json!({ "data": profiles }))),
        Err(e) => {
            error!("Failed to list profiles: {}", e);
            Err(map_error(e))
        }
    }
}

pub async fn enable_profile(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.financials.dunning_letter_management_engine.enable_profile(id).await {
        Ok(p) => Ok(to_json(p)),
        Err(e) => { error!("Failed to enable profile: {}", e); Err(map_error(e)) }
    }
}

#[derive(Debug, Deserialize)]
pub struct DisableProfileRequest {
    pub reason: Option<String>,
}

pub async fn disable_profile(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<DisableProfileRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.financials.dunning_letter_management_engine.disable_profile(id, req.reason.as_deref()).await {
        Ok(p) => Ok(to_json(p)),
        Err(e) => { error!("Failed to disable profile: {}", e); Err(map_error(e)) }
    }
}

#[derive(Debug, Deserialize)]
pub struct HoldProfileRequest {
    pub reason: String,
}

pub async fn hold_profile(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<HoldProfileRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.financials.dunning_letter_management_engine.hold_profile(id, &req.reason).await {
        Ok(p) => Ok(to_json(p)),
        Err(e) => { error!("Failed to hold profile: {}", e); Err(map_error(e)) }
    }
}

// ============================================================================
// Dunning Run Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateRunRequest {
    pub run_number: String,
    pub description: Option<String>,
    pub letter_set_id: Option<Uuid>,
    pub run_date: Option<String>,
    pub aging_as_of_date: Option<String>,
    pub currency_code: Option<String>,
    pub minimum_amount_filter: Option<String>,
    pub specific_level: Option<i32>,
    pub customer_id_filter: Option<Uuid>,
    pub notes: Option<String>,
}

pub async fn create_run(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreateRunRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let today = chrono::Utc::now().date_naive();
    let run_date = req.run_date.as_deref()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .unwrap_or(today);
    let aging_date = req.aging_as_of_date.as_deref()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .unwrap_or(run_date);

    match state.financials.dunning_letter_management_engine.create_run(
        org_id,
        &req.run_number,
        req.description.as_deref(),
        req.letter_set_id,
        run_date,
        aging_date,
        req.currency_code.as_deref().unwrap_or("USD"),
        req.minimum_amount_filter.as_deref(),
        req.specific_level,
        req.customer_id_filter,
        req.notes.as_deref(),
        Some(user_id),
    ).await {
        Ok(run) => Ok(created_json(run)),
        Err(e) => {
            error!("Failed to create dunning run: {}", e);
            Err(map_error(e))
        }
    }
}

pub async fn get_run(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.financials.dunning_letter_management_engine.get_run(id).await {
        Ok(Some(r)) => Ok(to_json(r)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Failed to get run: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListRunsQuery {
    pub status: Option<String>,
}

pub async fn list_runs(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<ListRunsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match state.financials.dunning_letter_management_engine.list_runs(org_id, query.status.as_deref()).await {
        Ok(runs) => Ok(Json(serde_json::json!({ "data": runs }))),
        Err(e) => {
            error!("Failed to list runs: {}", e);
            Err(map_error(e))
        }
    }
}

pub async fn submit_run(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.financials.dunning_letter_management_engine.submit_run(id).await {
        Ok(r) => Ok(to_json(r)),
        Err(e) => { error!("Failed to submit run: {}", e); Err(map_error(e)) }
    }
}

pub async fn complete_run(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.financials.dunning_letter_management_engine.complete_run(id).await {
        Ok(r) => Ok(to_json(r)),
        Err(e) => { error!("Failed to complete run: {}", e); Err(map_error(e)) }
    }
}

pub async fn cancel_run(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.financials.dunning_letter_management_engine.cancel_run(id).await {
        Ok(r) => Ok(to_json(r)),
        Err(e) => { error!("Failed to cancel run: {}", e); Err(map_error(e)) }
    }
}

// ============================================================================
// Run Result Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct AddRunResultRequest {
    pub customer_id: Uuid,
    pub customer_name: Option<String>,
    pub customer_number: Option<String>,
    pub profile_id: Option<Uuid>,
    pub dunning_level: i32,
    pub level_name: Option<String>,
    pub number_of_overdue_items: Option<i32>,
    pub total_overdue_amount: Option<String>,
    pub oldest_overdue_date: Option<String>,
    pub days_overdue: Option<i32>,
    pub finance_charge_amount: Option<String>,
    pub letter_template: Option<String>,
    pub delivery_method: Option<String>,
    pub status: Option<String>,
    pub reason: Option<String>,
}

pub async fn add_run_result(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(run_id): Path<Uuid>,
    Json(req): Json<AddRunResultRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let oldest_date = req.oldest_overdue_date.as_deref()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());

    match state.financials.dunning_letter_management_engine.add_run_result(
        org_id,
        run_id,
        req.customer_id,
        req.customer_name.as_deref(),
        req.customer_number.as_deref(),
        req.profile_id,
        req.dunning_level,
        req.level_name.as_deref(),
        req.number_of_overdue_items.unwrap_or(0),
        req.total_overdue_amount.as_deref().unwrap_or("0"),
        oldest_date,
        req.days_overdue.unwrap_or(0),
        req.finance_charge_amount.as_deref().unwrap_or("0"),
        req.letter_template.as_deref(),
        req.delivery_method.as_deref(),
        req.status.as_deref().unwrap_or("pending"),
        req.reason.as_deref(),
    ).await {
        Ok(result) => Ok(created_json(result)),
        Err(e) => {
            error!("Failed to add run result: {}", e);
            Err(map_error(e))
        }
    }
}

pub async fn list_run_results(
    State(state): State<Arc<AppState>>,
    Path(run_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.financials.dunning_letter_management_engine.list_run_results(run_id).await {
        Ok(results) => Ok(Json(serde_json::json!({ "data": results }))),
        Err(e) => { error!("Failed to list run results: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

pub async fn get_run_result(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.financials.dunning_letter_management_engine.get_run_result(id).await {
        Ok(Some(r)) => Ok(to_json(r)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => { error!("Failed to get run result: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

#[derive(Debug, Deserialize)]
pub struct MarkResultSentRequest {
    pub delivery_confirmation: Option<String>,
}

pub async fn mark_result_sent(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<MarkResultSentRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.financials.dunning_letter_management_engine.mark_result_sent(
        id, req.delivery_confirmation.as_deref(),
    ).await {
        Ok(r) => Ok(to_json(r)),
        Err(e) => { error!("Failed to mark result sent: {}", e); Err(map_error(e)) }
    }
}

#[derive(Debug, Deserialize)]
pub struct MarkResultFailedRequest {
    pub reason: String,
}

pub async fn mark_result_failed(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<MarkResultFailedRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.financials.dunning_letter_management_engine.mark_result_failed(id, &req.reason).await {
        Ok(r) => Ok(to_json(r)),
        Err(e) => { error!("Failed to mark result failed: {}", e); Err(map_error(e)) }
    }
}

// ============================================================================
// Dashboard
// ============================================================================

pub async fn get_dashboard(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let org_id = Uuid::parse_str(&claims.org_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let row = sqlx::query(
        "SELECT * FROM financials.dunning_dashboard WHERE organization_id = $1"
    )
    .bind(org_id)
    .fetch_optional(&state.db_pool)
    .await;

    match row {
        Ok(Some(r)) => {
            let json = serde_json::json!({
                "total_letter_sets": r.get::<Option<i64>, _>("total_letter_sets").unwrap_or(0),
                "active_letter_sets": r.get::<Option<i64>, _>("active_letter_sets").unwrap_or(0),
                "total_profiles": r.get::<Option<i64>, _>("total_profiles").unwrap_or(0),
                "enabled_profiles": r.get::<Option<i64>, _>("enabled_profiles").unwrap_or(0),
                "disabled_profiles": r.get::<Option<i64>, _>("disabled_profiles").unwrap_or(0),
                "hold_profiles": r.get::<Option<i64>, _>("hold_profiles").unwrap_or(0),
                "total_runs": r.get::<Option<i64>, _>("total_runs").unwrap_or(0),
                "draft_runs": r.get::<Option<i64>, _>("draft_runs").unwrap_or(0),
                "submitted_runs": r.get::<Option<i64>, _>("submitted_runs").unwrap_or(0),
                "completed_runs": r.get::<Option<i64>, _>("completed_runs").unwrap_or(0),
                "total_letters_generated": r.get::<Option<i64>, _>("total_letters_generated").unwrap_or(0),
                "total_errors": r.get::<Option<i64>, _>("total_errors").unwrap_or(0),
                "letters_sent": r.get::<Option<i64>, _>("letters_sent").unwrap_or(0),
                "letters_failed": r.get::<Option<i64>, _>("letters_failed").unwrap_or(0),
                "letters_pending": r.get::<Option<i64>, _>("letters_pending").unwrap_or(0),
            });
            Ok(Json(json))
        }
        Ok(None) => Ok(Json(serde_json::json!({
            "total_letter_sets": 0,
            "active_letter_sets": 0,
            "total_profiles": 0,
            "enabled_profiles": 0,
            "disabled_profiles": 0,
            "hold_profiles": 0,
            "total_runs": 0,
            "draft_runs": 0,
            "submitted_runs": 0,
            "completed_runs": 0,
            "total_letters_generated": 0,
            "total_errors": 0,
            "letters_sent": 0,
            "letters_failed": 0,
            "letters_pending": 0,
        }))),
        Err(e) => {
            error!("Failed to get dunning dashboard: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// ============================================================================
// Error Mapping
// ============================================================================

fn map_error(e: atlas_shared::AtlasError) -> StatusCode {
    match e {
        atlas_shared::AtlasError::ValidationFailed(_) => StatusCode::BAD_REQUEST,
        atlas_shared::AtlasError::Conflict(_) => StatusCode::CONFLICT,
        atlas_shared::AtlasError::EntityNotFound(_) => StatusCode::NOT_FOUND,
        atlas_shared::AtlasError::WorkflowError(_) => StatusCode::BAD_REQUEST,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
