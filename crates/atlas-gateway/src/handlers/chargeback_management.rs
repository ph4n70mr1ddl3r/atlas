//! Chargeback Management API Handlers
//!
//! REST endpoints for Oracle Fusion-inspired Chargeback Management.
//! Manages customer payment deductions with full lifecycle:
//! open → under_review → accepted → rejected → written_off

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
// Chargeback CRUD Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateChargebackRequest {
    pub customer_id: Option<String>,
    pub customer_number: Option<String>,
    pub customer_name: Option<String>,
    pub receipt_id: Option<String>,
    pub receipt_number: Option<String>,
    pub invoice_id: Option<String>,
    pub invoice_number: Option<String>,
    pub chargeback_date: String,
    pub gl_date: Option<String>,
    pub currency_code: Option<String>,
    pub exchange_rate_type: Option<String>,
    pub exchange_rate: Option<f64>,
    pub amount: f64,
    pub tax_amount: Option<f64>,
    pub reason_code: String,
    pub reason_description: Option<String>,
    pub category: Option<String>,
    pub priority: Option<String>,
    pub assigned_to: Option<String>,
    pub assigned_team: Option<String>,
    pub due_date: Option<String>,
    pub reference: Option<String>,
    pub customer_reference: Option<String>,
    pub sales_rep: Option<String>,
    pub notes: Option<String>,
}

pub async fn create_chargeback(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreateChargebackRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    let chargeback_date = chrono::NaiveDate::parse_from_str(&req.chargeback_date, "%Y-%m-%d")
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": format!("Invalid chargeback_date: {}", e)}))))?;
    let gl_date = req.gl_date.as_deref()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let due_date = req.due_date.as_deref()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let customer_id = req.customer_id.as_deref().and_then(|s| s.parse::<Uuid>().ok());
    let receipt_id = req.receipt_id.as_deref().and_then(|s| s.parse::<Uuid>().ok());
    let invoice_id = req.invoice_id.as_deref().and_then(|s| s.parse::<Uuid>().ok());
    let currency = req.currency_code.as_deref().unwrap_or("USD");
    let tax = req.tax_amount.unwrap_or(0.0);

    match state.chargeback_engine.create_chargeback(
        org_id,
        customer_id,
        req.customer_number.as_deref(),
        req.customer_name.as_deref(),
        receipt_id,
        req.receipt_number.as_deref(),
        invoice_id,
        req.invoice_number.as_deref(),
        chargeback_date,
        gl_date,
        currency,
        req.exchange_rate_type.as_deref(),
        req.exchange_rate,
        req.amount,
        tax,
        &req.reason_code,
        req.reason_description.as_deref(),
        req.category.as_deref(),
        req.priority.as_deref(),
        req.assigned_to.as_deref(),
        req.assigned_team.as_deref(),
        due_date,
        req.reference.as_deref(),
        req.customer_reference.as_deref(),
        req.sales_rep.as_deref(),
        req.notes.as_deref(),
        None,
    ).await {
        Ok(cb) => Ok((StatusCode::CREATED, Json(serde_json::to_value(cb).unwrap_or(serde_json::Value::Null)))),
        Err(e) => {
            error!("Failed to create chargeback: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListChargebacksQuery {
    pub status: Option<String>,
    pub customer_id: Option<String>,
    pub reason_code: Option<String>,
    pub category: Option<String>,
    pub priority: Option<String>,
}

pub async fn list_chargebacks(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<ListChargebacksQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    let customer_id = query.customer_id.as_deref().and_then(|s| s.parse::<Uuid>().ok());

    match state.chargeback_engine.list_chargebacks(
        org_id,
        query.status.as_deref(),
        customer_id,
        query.reason_code.as_deref(),
        query.category.as_deref(),
        query.priority.as_deref(),
    ).await {
        Ok(cbs) => Ok(Json(serde_json::json!({"data": cbs}))),
        Err(e) => {
            error!("Failed to list chargebacks: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

pub async fn get_chargeback(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.chargeback_engine.get_chargeback(id).await {
        Ok(Some(cb)) => Ok(Json(serde_json::to_value(cb).unwrap_or(serde_json::Value::Null))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Chargeback not found"})))),
        Err(e) => {
            error!("Failed to get chargeback: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

pub async fn get_chargeback_by_number(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(number): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    match state.chargeback_engine.get_chargeback_by_number(org_id, &number).await {
        Ok(Some(cb)) => Ok(Json(serde_json::to_value(cb).unwrap_or(serde_json::Value::Null))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Chargeback not found"})))),
        Err(e) => {
            error!("Failed to get chargeback: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

pub async fn delete_chargeback(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(number): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    match state.chargeback_engine.delete_chargeback(org_id, &number).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            error!("Failed to delete chargeback: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

// ============================================================================
// Status Transition Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransitionRequest {
    pub status: String,
    pub resolution_notes: Option<String>,
    pub resolved_by_name: Option<String>,
}

pub async fn transition_chargeback(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(req): Json<TransitionRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.chargeback_engine.transition_chargeback(
        id,
        &req.status,
        req.resolution_notes.as_deref(),
        None,
        req.resolved_by_name.as_deref(),
    ).await {
        Ok(cb) => Ok(Json(serde_json::to_value(cb).unwrap_or(serde_json::Value::Null))),
        Err(e) => {
            error!("Failed to transition chargeback: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssignRequest {
    pub assigned_to: Option<String>,
    pub assigned_team: Option<String>,
}

pub async fn assign_chargeback(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(req): Json<AssignRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.chargeback_engine.assign_chargeback(
        id,
        req.assigned_to.as_deref(),
        req.assigned_team.as_deref(),
    ).await {
        Ok(cb) => Ok(Json(serde_json::to_value(cb).unwrap_or(serde_json::Value::Null))),
        Err(e) => {
            error!("Failed to assign chargeback: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateNotesRequest {
    pub notes: Option<String>,
}

pub async fn update_notes(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateNotesRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.chargeback_engine.update_notes(id, req.notes.as_deref()).await {
        Ok(cb) => Ok(Json(serde_json::to_value(cb).unwrap_or(serde_json::Value::Null))),
        Err(e) => {
            error!("Failed to update chargeback notes: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

// ============================================================================
// Chargeback Line Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddLineRequest {
    pub line_type: Option<String>,
    pub description: Option<String>,
    pub quantity: Option<i32>,
    pub unit_price: Option<f64>,
    pub amount: f64,
    pub tax_amount: Option<f64>,
    pub reason_code: Option<String>,
    pub reason_description: Option<String>,
    pub item_number: Option<String>,
    pub item_description: Option<String>,
    pub gl_account_code: Option<String>,
    pub gl_account_name: Option<String>,
    pub reference: Option<String>,
}

pub async fn add_line(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(chargeback_id): Path<Uuid>,
    Json(req): Json<AddLineRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    let line_type = req.line_type.as_deref().unwrap_or("chargeback");

    match state.chargeback_engine.add_chargeback_line(
        org_id,
        chargeback_id,
        line_type,
        req.description.as_deref(),
        req.quantity,
        req.unit_price,
        req.amount,
        req.tax_amount,
        req.reason_code.as_deref(),
        req.reason_description.as_deref(),
        req.item_number.as_deref(),
        req.item_description.as_deref(),
        req.gl_account_code.as_deref(),
        req.gl_account_name.as_deref(),
        req.reference.as_deref(),
    ).await {
        Ok(line) => Ok((StatusCode::CREATED, Json(serde_json::to_value(line).unwrap_or(serde_json::Value::Null)))),
        Err(e) => {
            error!("Failed to add chargeback line: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

pub async fn list_lines(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(chargeback_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.chargeback_engine.list_chargeback_lines(chargeback_id).await {
        Ok(lines) => Ok(Json(serde_json::json!({"data": lines}))),
        Err(e) => {
            error!("Failed to list chargeback lines: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

pub async fn remove_line(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path((chargeback_id, line_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    match state.chargeback_engine.remove_chargeback_line(chargeback_id, line_id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            error!("Failed to remove chargeback line: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

// ============================================================================
// Activity Handlers
// ============================================================================

pub async fn list_activities(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(chargeback_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.chargeback_engine.list_activities(chargeback_id).await {
        Ok(activities) => Ok(Json(serde_json::json!({"data": activities}))),
        Err(e) => {
            error!("Failed to list chargeback activities: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

// ============================================================================
// Dashboard Handler
// ============================================================================

pub async fn get_dashboard(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    match state.chargeback_engine.get_dashboard(org_id).await {
        Ok(summary) => Ok(Json(serde_json::to_value(summary).unwrap_or(serde_json::Value::Null))),
        Err(e) => {
            error!("Failed to get chargeback dashboard: {}", e);
            Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                 Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}
