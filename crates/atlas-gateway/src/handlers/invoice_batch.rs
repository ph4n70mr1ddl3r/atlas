//! AP Invoice Batch Processing API Handlers
//!
//! REST endpoints for Oracle Fusion-inspired AP Invoice Batch Processing.
//! Manages invoice batches with full lifecycle:
//! draft → submitted → approved → posted → cancelled

use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use tracing::error;
use uuid::Uuid;

use crate::handlers::auth::{parse_uuid, Claims};
use crate::AppState;

// ============================================================================
// Batch CRUD Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateBatchRequest {
    pub batch_name: String,
    pub description: Option<String>,
    pub currency_code: Option<String>,
    pub exchange_rate_type: Option<String>,
    pub exchange_rate: Option<f64>,
    pub gl_date: Option<String>,
    pub accounting_period: Option<String>,
    pub source: Option<String>,
    pub control_total: Option<f64>,
    pub control_count: Option<i32>,
}

pub async fn create_batch(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreateBatchRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    let user_id = parse_uuid(&claims.sub).ok();

    let gl_date = req
        .gl_date
        .as_deref()
        .map(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d"))
        .transpose()
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": format!("Invalid gl_date: {}", e)})),
            )
        })?;

    match state
        .financials
        .invoice_batch_engine
        .create_batch(
            org_id,
            &req.batch_name,
            req.description.as_deref(),
            req.currency_code.as_deref().unwrap_or("USD"),
            req.exchange_rate_type.as_deref(),
            req.exchange_rate,
            gl_date,
            req.accounting_period.as_deref(),
            req.source.as_deref().unwrap_or("manual"),
            req.control_total,
            req.control_count,
            user_id,
        )
        .await
    {
        Ok(batch) => Ok((
            StatusCode::CREATED,
            Json(serde_json::to_value(batch).unwrap_or(serde_json::Value::Null)),
        )),
        Err(e) => {
            error!("Failed to create invoice batch: {}", e);
            Err((
                StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(serde_json::json!({"error": e.to_string()})),
            ))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListBatchesQuery {
    pub status: Option<String>,
}

pub async fn list_batches(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<ListBatchesQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;

    match state
        .financials
        .invoice_batch_engine
        .list_batches(org_id, query.status.as_deref())
        .await
    {
        Ok(batches) => Ok(Json(serde_json::json!({"data": batches}))),
        Err(e) => {
            error!("Failed to list invoice batches: {}", e);
            Err((
                StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(serde_json::json!({"error": e.to_string()})),
            ))
        }
    }
}

pub async fn get_batch(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;

    match state
        .financials
        .invoice_batch_engine
        .get_batch(org_id, id)
        .await
    {
        Ok(batch) => Ok(Json(
            serde_json::to_value(batch).unwrap_or(serde_json::Value::Null),
        )),
        Err(e) => {
            error!("Failed to get invoice batch: {}", e);
            let status = if matches!(e, atlas_shared::AtlasError::EntityNotFound(_)) {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
            };
            Err((status, Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

pub async fn get_batch_by_number(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(number): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;

    match state
        .financials
        .invoice_batch_engine
        .get_batch_by_number(org_id, &number)
        .await
    {
        Ok(batch) => Ok(Json(
            serde_json::to_value(batch).unwrap_or(serde_json::Value::Null),
        )),
        Err(e) => {
            error!("Failed to get invoice batch by number: {}", e);
            let status = if matches!(e, atlas_shared::AtlasError::EntityNotFound(_)) {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
            };
            Err((status, Json(serde_json::json!({"error": e.to_string()}))))
        }
    }
}

pub async fn delete_batch(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(number): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;

    match state
        .financials
        .invoice_batch_engine
        .delete_batch(org_id, &number)
        .await
    {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            error!("Failed to delete invoice batch: {}", e);
            Err((
                StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(serde_json::json!({"error": e.to_string()})),
            ))
        }
    }
}

// ============================================================================
// Lifecycle Transition Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct TransitionRequest {
    pub reason: Option<String>,
}

pub async fn submit_batch(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(_req): Json<TransitionRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    let user_id = parse_uuid(&claims.sub)?;

    match state
        .financials
        .invoice_batch_engine
        .submit_batch(org_id, id, user_id)
        .await
    {
        Ok(batch) => Ok(Json(
            serde_json::to_value(batch).unwrap_or(serde_json::Value::Null),
        )),
        Err(e) => {
            error!("Failed to submit invoice batch: {}", e);
            Err((
                StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(serde_json::json!({"error": e.to_string()})),
            ))
        }
    }
}

pub async fn approve_batch(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(_req): Json<TransitionRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    let user_id = parse_uuid(&claims.sub)?;

    match state
        .financials
        .invoice_batch_engine
        .approve_batch(org_id, id, user_id)
        .await
    {
        Ok(batch) => Ok(Json(
            serde_json::to_value(batch).unwrap_or(serde_json::Value::Null),
        )),
        Err(e) => {
            error!("Failed to approve invoice batch: {}", e);
            Err((
                StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(serde_json::json!({"error": e.to_string()})),
            ))
        }
    }
}

pub async fn post_batch(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(_req): Json<TransitionRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    let user_id = parse_uuid(&claims.sub)?;

    match state
        .financials
        .invoice_batch_engine
        .post_batch(org_id, id, user_id)
        .await
    {
        Ok(batch) => Ok(Json(
            serde_json::to_value(batch).unwrap_or(serde_json::Value::Null),
        )),
        Err(e) => {
            error!("Failed to post invoice batch: {}", e);
            Err((
                StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(serde_json::json!({"error": e.to_string()})),
            ))
        }
    }
}

pub async fn cancel_batch(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(req): Json<TransitionRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    let user_id = parse_uuid(&claims.sub)?;

    match state
        .financials
        .invoice_batch_engine
        .cancel_batch(org_id, id, user_id, req.reason.as_deref())
        .await
    {
        Ok(batch) => Ok(Json(
            serde_json::to_value(batch).unwrap_or(serde_json::Value::Null),
        )),
        Err(e) => {
            error!("Failed to cancel invoice batch: {}", e);
            Err((
                StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(serde_json::json!({"error": e.to_string()})),
            ))
        }
    }
}

// ============================================================================
// Invoice Totals Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddInvoiceRequest {
    pub invoice_amount: f64,
    pub tax_amount: f64,
}

pub async fn add_invoice(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(req): Json<AddInvoiceRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    if req.invoice_amount < 0.0 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Invoice amount cannot be negative"})),
        ));
    }
    if req.tax_amount < 0.0 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Tax amount cannot be negative"})),
        ));
    }

    match state
        .financials
        .invoice_batch_engine
        .add_invoice_to_totals(id, req.invoice_amount, req.tax_amount)
        .await
    {
        Ok(batch) => Ok(Json(
            serde_json::to_value(batch).unwrap_or(serde_json::Value::Null),
        )),
        Err(e) => {
            error!("Failed to add invoice to batch: {}", e);
            Err((
                StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(serde_json::json!({"error": e.to_string()})),
            ))
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveInvoiceRequest {
    pub invoice_amount: f64,
    pub tax_amount: f64,
}

pub async fn remove_invoice(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(req): Json<RemoveInvoiceRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state
        .financials
        .invoice_batch_engine
        .remove_invoice_from_totals(id, req.invoice_amount, req.tax_amount)
        .await
    {
        Ok(batch) => Ok(Json(
            serde_json::to_value(batch).unwrap_or(serde_json::Value::Null),
        )),
        Err(e) => {
            error!("Failed to remove invoice from batch: {}", e);
            Err((
                StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(serde_json::json!({"error": e.to_string()})),
            ))
        }
    }
}

// ============================================================================
// Validation & Activities
// ============================================================================

pub async fn validate_batch(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;

    match state
        .financials
        .invoice_batch_engine
        .validate_batch(org_id, id)
        .await
    {
        Ok(batch) => Ok(Json(
            serde_json::to_value(batch).unwrap_or(serde_json::Value::Null),
        )),
        Err(e) => {
            error!("Failed to validate batch: {}", e);
            Err((
                StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(serde_json::json!({"error": e.to_string()})),
            ))
        }
    }
}

pub async fn list_activities(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state
        .financials
        .invoice_batch_engine
        .list_activities(id)
        .await
    {
        Ok(activities) => Ok(Json(serde_json::json!({"data": activities}))),
        Err(e) => {
            error!("Failed to list batch activities: {}", e);
            Err((
                StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(serde_json::json!({"error": e.to_string()})),
            ))
        }
    }
}

// ============================================================================
// Dashboard
// ============================================================================

pub async fn get_dashboard(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;

    match state
        .financials
        .invoice_batch_engine
        .get_dashboard(org_id)
        .await
    {
        Ok(summary) => Ok(Json(
            serde_json::to_value(summary).unwrap_or(serde_json::Value::Null),
        )),
        Err(e) => {
            error!("Failed to get invoice batch dashboard: {}", e);
            Err((
                StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(serde_json::json!({"error": e.to_string()})),
            ))
        }
    }
}
