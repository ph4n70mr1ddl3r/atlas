//! Grant Management API Handlers
//!
//! REST endpoints for Oracle Fusion-inspired Grant Management.
//! Manages sponsors, awards, budgets, expenditures, billing, and compliance reporting.

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


/// Parse a UUID from a claim string, returning a JSON error on failure.
///
/// Unlike `unwrap_or_default()`, this does NOT silently fall back to the nil
/// UUID — which would be an auth-scoping bypass.
fn parse_uuid(s: &str) -> Result<Uuid, (axum::http::StatusCode, Json<serde_json::Value>)> {
    Uuid::parse_str(s).map_err(|_| {
        (axum::http::StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "Invalid auth token"})))
    })
}
// ============================================================================
// Sponsor Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSponsorRequest {
    pub sponsor_code: String,
    pub name: String,
    pub sponsor_type: Option<String>,
    pub country_code: Option<String>,
    pub taxpayer_id: Option<String>,
    pub contact_name: Option<String>,
    pub contact_email: Option<String>,
    pub contact_phone: Option<String>,
    pub address_line1: Option<String>,
    pub address_line2: Option<String>,
    pub city: Option<String>,
    pub state_province: Option<String>,
    pub postal_code: Option<String>,
    pub payment_terms: Option<String>,
    pub billing_frequency: Option<String>,
    pub currency_code: Option<String>,
    pub credit_limit: Option<String>,
}

pub async fn create_sponsor(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreateSponsorRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    match state.grant_management_engine.create_sponsor(
        org_id, &req.sponsor_code, &req.name,
        req.sponsor_type.as_deref().unwrap_or("government"),
        req.country_code.as_deref(), req.taxpayer_id.as_deref(),
        req.contact_name.as_deref(), req.contact_email.as_deref(), req.contact_phone.as_deref(),
        req.address_line1.as_deref(), req.address_line2.as_deref(),
        req.city.as_deref(), req.state_province.as_deref(), req.postal_code.as_deref(),
        req.payment_terms.as_deref(),
        req.billing_frequency.as_deref().unwrap_or("monthly"),
        req.currency_code.as_deref().unwrap_or("USD"),
        req.credit_limit.as_deref(), None,
    ).await {
        Ok(s) => Ok((StatusCode::CREATED, Json(serde_json::to_value(s).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => {
            error!("Failed to create sponsor: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListSponsorsQuery { pub active_only: Option<bool> }

pub async fn list_sponsors(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<ListSponsorsQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    match state.grant_management_engine.list_sponsors(org_id, query.active_only.unwrap_or(true)).await {
        Ok(data) => Ok(Json(serde_json::json!({"data": data}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                       Json(serde_json::json!({"error": e.to_string()})))),
    }
}

pub async fn get_sponsor(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(code): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    match state.grant_management_engine.get_sponsor(org_id, &code).await {
        Ok(Some(s)) => Ok(Json(serde_json::to_value(s).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Sponsor not found"})))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                       Json(serde_json::json!({"error": e.to_string()})))),
    }
}

pub async fn delete_sponsor(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(code): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    match state.grant_management_engine.delete_sponsor(org_id, &code).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                       Json(serde_json::json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Indirect Cost Rate Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateIndirectCostRateRequest {
    pub rate_name: String,
    pub rate_type: Option<String>,
    pub rate_percentage: String,
    pub base_type: Option<String>,
    pub effective_from: chrono::NaiveDate,
    pub effective_to: Option<chrono::NaiveDate>,
    pub negotiated_by: Option<String>,
}

pub async fn create_indirect_cost_rate(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreateIndirectCostRateRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    match state.grant_management_engine.create_indirect_cost_rate(
        org_id, &req.rate_name,
        req.rate_type.as_deref().unwrap_or("negotiated"),
        &req.rate_percentage,
        req.base_type.as_deref().unwrap_or("modified_total_direct_costs"),
        req.effective_from, req.effective_to, req.negotiated_by.as_deref(), None,
    ).await {
        Ok(r) => Ok((StatusCode::CREATED, Json(serde_json::to_value(r).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => {
            error!("Failed to create indirect cost rate: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListRatesQuery { pub active_only: Option<bool> }

pub async fn list_indirect_cost_rates(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<ListRatesQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    match state.grant_management_engine.list_indirect_cost_rates(org_id, query.active_only.unwrap_or(true)).await {
        Ok(data) => Ok(Json(serde_json::json!({"data": data}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                       Json(serde_json::json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Award Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAwardRequest {
    pub award_number: String,
    pub award_title: String,
    pub sponsor_id: Uuid,
    pub sponsor_award_number: Option<String>,
    pub award_type: Option<String>,
    pub award_purpose: Option<String>,
    pub start_date: chrono::NaiveDate,
    pub end_date: chrono::NaiveDate,
    pub total_award_amount: String,
    pub direct_costs_total: Option<String>,
    pub indirect_costs_total: Option<String>,
    pub cost_sharing_total: Option<String>,
    pub currency_code: Option<String>,
    pub indirect_cost_rate: Option<String>,
    pub cost_sharing_required: Option<bool>,
    pub cost_sharing_percent: Option<String>,
    pub principal_investigator_id: Option<Uuid>,
    pub principal_investigator_name: Option<String>,
    pub department_id: Option<Uuid>,
    pub department_name: Option<String>,
    pub project_id: Option<Uuid>,
    pub cost_center: Option<String>,
    pub gl_revenue_account: Option<String>,
    pub gl_receivable_account: Option<String>,
    pub gl_deferred_account: Option<String>,
    pub billing_frequency: Option<String>,
    pub billing_basis: Option<String>,
    pub reporting_requirements: Option<String>,
    pub compliance_notes: Option<String>,
}

pub async fn create_award(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreateAwardRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    match state.grant_management_engine.create_award(
        org_id, &req.award_number, &req.award_title, req.sponsor_id,
        req.sponsor_award_number.as_deref(),
        req.award_type.as_deref().unwrap_or("research"),
        req.award_purpose.as_deref(),
        req.start_date, req.end_date,
        &req.total_award_amount,
        req.direct_costs_total.as_deref().unwrap_or("0"),
        req.indirect_costs_total.as_deref().unwrap_or("0"),
        req.cost_sharing_total.as_deref().unwrap_or("0"),
        req.currency_code.as_deref().unwrap_or("USD"),
        None, // indirect_cost_rate_id
        req.indirect_cost_rate.as_deref().unwrap_or("0"),
        req.cost_sharing_required.unwrap_or(false),
        req.cost_sharing_percent.as_deref().unwrap_or("0"),
        req.principal_investigator_id, req.principal_investigator_name.as_deref(),
        req.department_id, req.department_name.as_deref(),
        req.project_id, req.cost_center.as_deref(),
        req.gl_revenue_account.as_deref(), req.gl_receivable_account.as_deref(),
        req.gl_deferred_account.as_deref(),
        req.billing_frequency.as_deref().unwrap_or("monthly"),
        req.billing_basis.as_deref().unwrap_or("cost"),
        req.reporting_requirements.as_deref(), req.compliance_notes.as_deref(),
        None,
    ).await {
        Ok(a) => Ok((StatusCode::CREATED, Json(serde_json::to_value(a).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => {
            error!("Failed to create award: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListAwardsQuery {
    pub status: Option<String>,
    pub sponsor_id: Option<Uuid>,
}

pub async fn list_awards(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<ListAwardsQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    match state.grant_management_engine.list_awards(org_id, query.status.as_deref(), query.sponsor_id).await {
        Ok(data) => Ok(Json(serde_json::json!({"data": data}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                       Json(serde_json::json!({"error": e.to_string()})))),
    }
}

pub async fn get_award(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.grant_management_engine.get_award(id).await {
        Ok(Some(a)) => Ok(Json(serde_json::to_value(a).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Award not found"})))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                       Json(serde_json::json!({"error": e.to_string()})))),
    }
}

pub async fn activate_award(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.grant_management_engine.activate_award(id).await {
        Ok(a) => Ok(Json(serde_json::to_value(a).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                       Json(serde_json::json!({"error": e.to_string()})))),
    }
}

pub async fn suspend_award(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.grant_management_engine.suspend_award(id).await {
        Ok(a) => Ok(Json(serde_json::to_value(a).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                       Json(serde_json::json!({"error": e.to_string()})))),
    }
}

pub async fn complete_award(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.grant_management_engine.complete_award(id, None).await {
        Ok(a) => Ok(Json(serde_json::to_value(a).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                       Json(serde_json::json!({"error": e.to_string()})))),
    }
}

pub async fn terminate_award(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.grant_management_engine.terminate_award(id, None).await {
        Ok(a) => Ok(Json(serde_json::to_value(a).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                       Json(serde_json::json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Budget Line Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateBudgetLineRequest {
    pub budget_category: String,
    pub description: Option<String>,
    pub account_code: Option<String>,
    pub budget_amount: String,
    pub period_start: Option<chrono::NaiveDate>,
    pub period_end: Option<chrono::NaiveDate>,
    pub fiscal_year: Option<i32>,
    pub notes: Option<String>,
}

pub async fn create_budget_line(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(award_id): Path<Uuid>,
    Json(req): Json<CreateBudgetLineRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    match state.grant_management_engine.create_budget_line(
        org_id, award_id, &req.budget_category, req.description.as_deref(),
        req.account_code.as_deref(), &req.budget_amount,
        req.period_start, req.period_end, req.fiscal_year, req.notes.as_deref(), None,
    ).await {
        Ok(bl) => Ok((StatusCode::CREATED, Json(serde_json::to_value(bl).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => {
            error!("Failed to create budget line: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

pub async fn list_budget_lines(
    State(state): State<Arc<AppState>>,
    Path(award_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.grant_management_engine.list_budget_lines(award_id).await {
        Ok(data) => Ok(Json(serde_json::json!({"data": data}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                       Json(serde_json::json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Expenditure Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateExpenditureRequest {
    pub expenditure_type: Option<String>,
    pub expenditure_date: chrono::NaiveDate,
    pub description: Option<String>,
    pub budget_line_id: Option<Uuid>,
    pub budget_category: Option<String>,
    pub amount: String,
    pub employee_id: Option<Uuid>,
    pub employee_name: Option<String>,
    pub vendor_id: Option<Uuid>,
    pub vendor_name: Option<String>,
    pub gl_debit_account: Option<String>,
    pub gl_credit_account: Option<String>,
    pub notes: Option<String>,
}

pub async fn create_expenditure(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(award_id): Path<Uuid>,
    Json(req): Json<CreateExpenditureRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    match state.grant_management_engine.create_expenditure(
        org_id, award_id,
        req.expenditure_type.as_deref().unwrap_or("actual"),
        req.expenditure_date, req.description.as_deref(),
        req.budget_line_id, req.budget_category.as_deref(),
        &req.amount, req.employee_id, req.employee_name.as_deref(),
        req.vendor_id, req.vendor_name.as_deref(),
        None, None, None,
        req.gl_debit_account.as_deref(), req.gl_credit_account.as_deref(),
        req.notes.as_deref(), None,
    ).await {
        Ok(exp) => Ok((StatusCode::CREATED, Json(serde_json::to_value(exp).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => {
            error!("Failed to create expenditure: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListExpendituresQuery { pub status: Option<String> }

pub async fn list_expenditures(
    State(state): State<Arc<AppState>>,
    Path(award_id): Path<Uuid>,
    Query(query): Query<ListExpendituresQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.grant_management_engine.list_expenditures(award_id, query.status.as_deref()).await {
        Ok(data) => Ok(Json(serde_json::json!({"data": data}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                       Json(serde_json::json!({"error": e.to_string()})))),
    }
}

pub async fn approve_expenditure(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = parse_uuid(&claims.sub).ok();
    match state.grant_management_engine.approve_expenditure(id, user_id).await {
        Ok(exp) => Ok(Json(serde_json::to_value(exp).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                       Json(serde_json::json!({"error": e.to_string()})))),
    }
}

pub async fn reverse_expenditure(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.grant_management_engine.reverse_expenditure(id).await {
        Ok(exp) => Ok(Json(serde_json::to_value(exp).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                       Json(serde_json::json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Billing Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateBillingRequest {
    pub period_start: chrono::NaiveDate,
    pub period_end: chrono::NaiveDate,
    pub notes: Option<String>,
}

pub async fn create_billing(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(award_id): Path<Uuid>,
    Json(req): Json<CreateBillingRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    match state.grant_management_engine.create_billing(
        org_id, award_id, req.period_start, req.period_end, req.notes.as_deref(), None,
    ).await {
        Ok(b) => Ok((StatusCode::CREATED, Json(serde_json::to_value(b).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => {
            error!("Failed to create billing: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListBillingsQuery { pub status: Option<String> }

pub async fn list_billings(
    State(state): State<Arc<AppState>>,
    Path(award_id): Path<Uuid>,
    Query(query): Query<ListBillingsQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.grant_management_engine.list_billings(award_id, query.status.as_deref()).await {
        Ok(data) => Ok(Json(serde_json::json!({"data": data}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                       Json(serde_json::json!({"error": e.to_string()})))),
    }
}

pub async fn submit_billing(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = parse_uuid(&claims.sub).ok();
    match state.grant_management_engine.submit_billing(id, user_id).await {
        Ok(b) => Ok(Json(serde_json::to_value(b).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                       Json(serde_json::json!({"error": e.to_string()})))),
    }
}

pub async fn approve_billing(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = parse_uuid(&claims.sub).ok();
    match state.grant_management_engine.approve_billing(id, user_id).await {
        Ok(b) => Ok(Json(serde_json::to_value(b).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                       Json(serde_json::json!({"error": e.to_string()})))),
    }
}

pub async fn mark_billing_paid(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.grant_management_engine.mark_billing_paid(id, None).await {
        Ok(b) => Ok(Json(serde_json::to_value(b).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                       Json(serde_json::json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Compliance Report Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateComplianceReportRequest {
    pub report_type: String,
    pub report_title: Option<String>,
    pub reporting_period_start: chrono::NaiveDate,
    pub reporting_period_end: chrono::NaiveDate,
    pub due_date: Option<chrono::NaiveDate>,
    pub notes: Option<String>,
}

pub async fn create_compliance_report(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(award_id): Path<Uuid>,
    Json(req): Json<CreateComplianceReportRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    match state.grant_management_engine.create_compliance_report(
        org_id, award_id, &req.report_type, req.report_title.as_deref(),
        req.reporting_period_start, req.reporting_period_end, req.due_date,
        req.notes.as_deref(), None,
    ).await {
        Ok(r) => Ok((StatusCode::CREATED, Json(serde_json::to_value(r).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => {
            error!("Failed to create compliance report: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListComplianceReportsQuery { pub report_type: Option<String> }

pub async fn list_compliance_reports(
    State(state): State<Arc<AppState>>,
    Path(award_id): Path<Uuid>,
    Query(query): Query<ListComplianceReportsQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.grant_management_engine.list_compliance_reports(award_id, query.report_type.as_deref()).await {
        Ok(data) => Ok(Json(serde_json::json!({"data": data}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                       Json(serde_json::json!({"error": e.to_string()})))),
    }
}

pub async fn submit_compliance_report(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = parse_uuid(&claims.sub).ok();
    match state.grant_management_engine.submit_compliance_report(id, user_id).await {
        Ok(r) => Ok(Json(serde_json::to_value(r).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                       Json(serde_json::json!({"error": e.to_string()})))),
    }
}

pub async fn approve_compliance_report(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = parse_uuid(&claims.sub).ok();
    match state.grant_management_engine.approve_compliance_report(id, user_id).await {
        Ok(r) => Ok(Json(serde_json::to_value(r).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                       Json(serde_json::json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Dashboard
// ============================================================================

pub async fn get_grant_dashboard(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    match state.grant_management_engine.get_dashboard_summary(org_id).await {
        Ok(summary) => Ok(Json(serde_json::to_value(summary).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                       Json(serde_json::json!({"error": e.to_string()})))),
    }
}
