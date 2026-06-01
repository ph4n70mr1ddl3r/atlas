//! Withholding Tax Management API Handlers
//!
//! REST endpoints for Oracle Fusion-inspired Withholding Tax Management.
//! Manages tax codes, tax groups, supplier assignments, withholding computation,
//! certificate lifecycle, and dashboard summary.
//!
//! Oracle Fusion equivalent: Financials > Payables > Withholding Tax

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
// Tax Code Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTaxCodeRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub tax_type: String,
    pub rate_percentage: String,
    pub threshold_amount: String,
    pub threshold_is_cumulative: Option<bool>,
    pub withholding_account_code: Option<String>,
    pub expense_account_code: Option<String>,
    pub effective_from: Option<String>,
    pub effective_to: Option<String>,
}

pub async fn create_tax_code(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreateTaxCodeRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    let user_id = parse_uuid(&claims.sub).ok();
    let effective_from = req
        .effective_from
        .as_deref()
        .map(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d"))
        .transpose()
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": format!("Invalid effective_from: {}", e)})),
            )
        })?;
    let effective_to = req
        .effective_to
        .as_deref()
        .map(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d"))
        .transpose()
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": format!("Invalid effective_to: {}", e)})),
            )
        })?;

    match state
        .financials
        .withholding_tax_engine
        .create_tax_code(
            org_id,
            &req.code,
            &req.name,
            req.description.as_deref(),
            &req.tax_type,
            &req.rate_percentage,
            &req.threshold_amount,
            req.threshold_is_cumulative.unwrap_or(false),
            req.withholding_account_code.as_deref(),
            req.expense_account_code.as_deref(),
            effective_from,
            effective_to,
            user_id,
        )
        .await
    {
        Ok(tc) => Ok((
            StatusCode::CREATED,
            Json(serde_json::to_value(tc).unwrap_or(serde_json::Value::Null)),
        )),
        Err(e) => {
            error!("Failed to create withholding tax code: {}", e);
            Err((
                StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(serde_json::json!({"error": e.to_string()})),
            ))
        }
    }
}

pub async fn get_tax_code(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(code): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    match state
        .financials
        .withholding_tax_engine
        .get_tax_code(org_id, &code)
        .await
    {
        Ok(Some(tc)) => Ok(Json(
            serde_json::to_value(tc).unwrap_or(serde_json::Value::Null),
        )),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Tax code not found"})),
        )),
        Err(e) => {
            error!("Failed to get withholding tax code: {}", e);
            Err((
                StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(serde_json::json!({"error": e.to_string()})),
            ))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListTaxCodesQuery {
    pub tax_type: Option<String>,
}

pub async fn list_tax_codes(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<ListTaxCodesQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    match state
        .financials
        .withholding_tax_engine
        .list_tax_codes(org_id, query.tax_type.as_deref())
        .await
    {
        Ok(codes) => Ok(Json(serde_json::json!({"data": codes}))),
        Err(e) => {
            error!("Failed to list withholding tax codes: {}", e);
            Err((
                StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(serde_json::json!({"error": e.to_string()})),
            ))
        }
    }
}

pub async fn delete_tax_code(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(code): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    match state
        .financials
        .withholding_tax_engine
        .delete_tax_code(org_id, &code)
        .await
    {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            error!("Failed to delete withholding tax code: {}", e);
            Err((
                StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(serde_json::json!({"error": e.to_string()})),
            ))
        }
    }
}

// ============================================================================
// Tax Group Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTaxGroupRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub tax_code_ids: Vec<String>,
}

pub async fn create_tax_group(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreateTaxGroupRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    let user_id = parse_uuid(&claims.sub).ok();
    let tax_code_ids: Result<Vec<Uuid>, _> =
        req.tax_code_ids.iter().map(|s| s.parse::<Uuid>()).collect();
    let tax_code_ids = tax_code_ids.map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Invalid tax_code_ids"})),
        )
    })?;

    match state
        .financials
        .withholding_tax_engine
        .create_tax_group(
            org_id,
            &req.code,
            &req.name,
            req.description.as_deref(),
            &tax_code_ids,
            user_id,
        )
        .await
    {
        Ok(group) => Ok((
            StatusCode::CREATED,
            Json(serde_json::to_value(group).unwrap_or(serde_json::Value::Null)),
        )),
        Err(e) => {
            error!("Failed to create tax group: {}", e);
            Err((
                StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(serde_json::json!({"error": e.to_string()})),
            ))
        }
    }
}

pub async fn get_tax_group(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(code): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    match state
        .financials
        .withholding_tax_engine
        .get_tax_group(org_id, &code)
        .await
    {
        Ok(Some(group)) => Ok(Json(
            serde_json::to_value(group).unwrap_or(serde_json::Value::Null),
        )),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Tax group not found"})),
        )),
        Err(e) => {
            error!("Failed to get tax group: {}", e);
            Err((
                StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(serde_json::json!({"error": e.to_string()})),
            ))
        }
    }
}

pub async fn list_tax_groups(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    match state
        .financials
        .withholding_tax_engine
        .list_tax_groups(org_id)
        .await
    {
        Ok(groups) => Ok(Json(serde_json::json!({"data": groups}))),
        Err(e) => {
            error!("Failed to list tax groups: {}", e);
            Err((
                StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(serde_json::json!({"error": e.to_string()})),
            ))
        }
    }
}

pub async fn delete_tax_group(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(code): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    match state
        .financials
        .withholding_tax_engine
        .delete_tax_group(org_id, &code)
        .await
    {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            error!("Failed to delete tax group: {}", e);
            Err((
                StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(serde_json::json!({"error": e.to_string()})),
            ))
        }
    }
}

// ============================================================================
// Supplier Assignment Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssignSupplierRequest {
    pub supplier_id: String,
    pub supplier_number: Option<String>,
    pub supplier_name: Option<String>,
    pub tax_group_code: String,
    pub is_exempt: Option<bool>,
    pub exemption_reason: Option<String>,
    pub exemption_certificate: Option<String>,
    pub exemption_valid_until: Option<String>,
}

pub async fn assign_supplier(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<AssignSupplierRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    let user_id = parse_uuid(&claims.sub).ok();
    let supplier_id = parse_uuid(&req.supplier_id)?;
    let exemption_valid_until = req
        .exemption_valid_until
        .as_deref()
        .map(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d"))
        .transpose()
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": format!("Invalid exemption_valid_until: {}", e)})),
            )
        })?;

    match state
        .financials
        .withholding_tax_engine
        .assign_supplier(
            org_id,
            supplier_id,
            req.supplier_number.as_deref(),
            req.supplier_name.as_deref(),
            &req.tax_group_code,
            req.is_exempt.unwrap_or(false),
            req.exemption_reason.as_deref(),
            req.exemption_certificate.as_deref(),
            exemption_valid_until,
            user_id,
        )
        .await
    {
        Ok(assignment) => Ok((
            StatusCode::CREATED,
            Json(serde_json::to_value(assignment).unwrap_or(serde_json::Value::Null)),
        )),
        Err(e) => {
            error!("Failed to assign supplier: {}", e);
            Err((
                StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(serde_json::json!({"error": e.to_string()})),
            ))
        }
    }
}

pub async fn get_supplier_assignment(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(supplier_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    match state
        .financials
        .withholding_tax_engine
        .get_supplier_assignment(org_id, supplier_id)
        .await
    {
        Ok(Some(a)) => Ok(Json(
            serde_json::to_value(a).unwrap_or(serde_json::Value::Null),
        )),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Supplier assignment not found"})),
        )),
        Err(e) => {
            error!("Failed to get supplier assignment: {}", e);
            Err((
                StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(serde_json::json!({"error": e.to_string()})),
            ))
        }
    }
}

pub async fn list_supplier_assignments(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    match state
        .financials
        .withholding_tax_engine
        .list_supplier_assignments(org_id)
        .await
    {
        Ok(assignments) => Ok(Json(serde_json::json!({"data": assignments}))),
        Err(e) => {
            error!("Failed to list supplier assignments: {}", e);
            Err((
                StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(serde_json::json!({"error": e.to_string()})),
            ))
        }
    }
}

pub async fn remove_supplier_assignment(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    match state
        .financials
        .withholding_tax_engine
        .remove_supplier_assignment(id)
        .await
    {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            error!("Failed to remove supplier assignment: {}", e);
            Err((
                StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(serde_json::json!({"error": e.to_string()})),
            ))
        }
    }
}

// ============================================================================
// Withholding Computation Handler
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComputeWithholdingRequest {
    pub supplier_id: String,
    pub invoice_amount: f64,
    pub invoice_id: String,
}

pub async fn compute_withholding(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<ComputeWithholdingRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    let supplier_id = parse_uuid(&req.supplier_id)?;
    let invoice_id = parse_uuid(&req.invoice_id)?;

    match state
        .financials
        .withholding_tax_engine
        .compute_withholding(org_id, supplier_id, req.invoice_amount, invoice_id)
        .await
    {
        Ok(result) => Ok(Json(
            serde_json::to_value(result).unwrap_or(serde_json::Value::Null),
        )),
        Err(e) => {
            error!("Failed to compute withholding: {}", e);
            Err((
                StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(serde_json::json!({"error": e.to_string()})),
            ))
        }
    }
}

// ============================================================================
// Withholding Lines Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordWithholdingRequest {
    pub payment_id: String,
    pub payment_number: Option<String>,
    pub invoice_id: String,
    pub invoice_number: Option<String>,
    pub supplier_id: String,
    pub supplier_name: Option<String>,
}

pub async fn record_withholding(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<RecordWithholdingRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    let user_id = parse_uuid(&claims.sub).ok();
    let payment_id = parse_uuid(&req.payment_id)?;
    let invoice_id = parse_uuid(&req.invoice_id)?;
    let supplier_id = parse_uuid(&req.supplier_id)?;

    // First compute the withholding
    let computation = state
        .financials
        .withholding_tax_engine
        .compute_withholding(
            org_id,
            supplier_id,
            0.0,
            invoice_id, // amount already known from computation context
        )
        .await
        .map_err(|e| {
            (
                StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(serde_json::json!({"error": e.to_string()})),
            )
        })?;

    match state
        .financials
        .withholding_tax_engine
        .record_withholding(
            org_id,
            payment_id,
            req.payment_number.as_deref(),
            invoice_id,
            req.invoice_number.as_deref(),
            supplier_id,
            req.supplier_name.as_deref(),
            &computation,
            user_id,
        )
        .await
    {
        Ok(lines) => Ok((
            StatusCode::CREATED,
            Json(serde_json::json!({"data": lines})),
        )),
        Err(e) => {
            error!("Failed to record withholding: {}", e);
            Err((
                StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(serde_json::json!({"error": e.to_string()})),
            ))
        }
    }
}

pub async fn get_withholding_lines_by_payment(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(payment_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state
        .financials
        .withholding_tax_engine
        .get_withholding_lines_by_payment(payment_id)
        .await
    {
        Ok(lines) => Ok(Json(serde_json::json!({"data": lines}))),
        Err(e) => {
            error!("Failed to get withholding lines: {}", e);
            Err((
                StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(serde_json::json!({"error": e.to_string()})),
            ))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct RemitWithholdingRequest {
    pub line_ids: Vec<String>,
    pub remittance_date: String,
    pub remittance_reference: Option<String>,
}

pub async fn remit_withholding(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Json(req): Json<RemitWithholdingRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let line_ids: Result<Vec<Uuid>, _> = req.line_ids.iter().map(|s| s.parse::<Uuid>()).collect();
    let line_ids = line_ids.map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Invalid line_ids"})),
        )
    })?;
    let remittance_date = chrono::NaiveDate::parse_from_str(&req.remittance_date, "%Y-%m-%d")
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": format!("Invalid remittance_date: {}", e)})),
            )
        })?;

    match state
        .financials
        .withholding_tax_engine
        .remit_withholding(
            &line_ids,
            remittance_date,
            req.remittance_reference.as_deref(),
        )
        .await
    {
        Ok(lines) => Ok(Json(serde_json::json!({"data": lines}))),
        Err(e) => {
            error!("Failed to remit withholding: {}", e);
            Err((
                StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(serde_json::json!({"error": e.to_string()})),
            ))
        }
    }
}

// ============================================================================
// Certificate Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateCertificateRequest {
    pub supplier_id: String,
    pub supplier_number: Option<String>,
    pub supplier_name: Option<String>,
    pub tax_code_id: String,
    pub period_start: String,
    pub period_end: String,
}

pub async fn generate_certificate(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<GenerateCertificateRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    let user_id = parse_uuid(&claims.sub).ok();
    let supplier_id = parse_uuid(&req.supplier_id)?;
    let tax_code_id = parse_uuid(&req.tax_code_id)?;
    let period_start =
        chrono::NaiveDate::parse_from_str(&req.period_start, "%Y-%m-%d").map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": format!("Invalid period_start: {}", e)})),
            )
        })?;
    let period_end =
        chrono::NaiveDate::parse_from_str(&req.period_end, "%Y-%m-%d").map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": format!("Invalid period_end: {}", e)})),
            )
        })?;

    match state
        .financials
        .withholding_tax_engine
        .generate_certificate(
            org_id,
            supplier_id,
            req.supplier_number.as_deref(),
            req.supplier_name.as_deref(),
            tax_code_id,
            period_start,
            period_end,
            user_id,
        )
        .await
    {
        Ok(cert) => Ok((
            StatusCode::CREATED,
            Json(serde_json::to_value(cert).unwrap_or(serde_json::Value::Null)),
        )),
        Err(e) => {
            error!("Failed to generate certificate: {}", e);
            Err((
                StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(serde_json::json!({"error": e.to_string()})),
            ))
        }
    }
}

pub async fn get_certificate(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state
        .financials
        .withholding_tax_engine
        .get_certificate(id)
        .await
    {
        Ok(Some(cert)) => Ok(Json(
            serde_json::to_value(cert).unwrap_or(serde_json::Value::Null),
        )),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Certificate not found"})),
        )),
        Err(e) => {
            error!("Failed to get certificate: {}", e);
            Err((
                StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(serde_json::json!({"error": e.to_string()})),
            ))
        }
    }
}

pub async fn get_certificate_by_number(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(number): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    match state
        .financials
        .withholding_tax_engine
        .get_certificate_by_number(org_id, &number)
        .await
    {
        Ok(Some(cert)) => Ok(Json(
            serde_json::to_value(cert).unwrap_or(serde_json::Value::Null),
        )),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Certificate not found"})),
        )),
        Err(e) => {
            error!("Failed to get certificate by number: {}", e);
            Err((
                StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(serde_json::json!({"error": e.to_string()})),
            ))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListCertificatesQuery {
    pub supplier_id: Option<String>,
}

pub async fn list_certificates(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<ListCertificatesQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    let supplier_id = query
        .supplier_id
        .as_deref()
        .map(str::parse::<Uuid>)
        .transpose()
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": format!("Invalid supplier_id: {}", e)})),
            )
        })?;

    match state
        .financials
        .withholding_tax_engine
        .list_certificates(org_id, supplier_id)
        .await
    {
        Ok(certs) => Ok(Json(serde_json::json!({"data": certs}))),
        Err(e) => {
            error!("Failed to list certificates: {}", e);
            Err((
                StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(serde_json::json!({"error": e.to_string()})),
            ))
        }
    }
}

pub async fn issue_certificate(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state
        .financials
        .withholding_tax_engine
        .issue_certificate(id)
        .await
    {
        Ok(cert) => Ok(Json(
            serde_json::to_value(cert).unwrap_or(serde_json::Value::Null),
        )),
        Err(e) => {
            error!("Failed to issue certificate: {}", e);
            Err((
                StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(serde_json::json!({"error": e.to_string()})),
            ))
        }
    }
}

pub async fn cancel_certificate(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state
        .financials
        .withholding_tax_engine
        .cancel_certificate(id)
        .await
    {
        Ok(cert) => Ok(Json(
            serde_json::to_value(cert).unwrap_or(serde_json::Value::Null),
        )),
        Err(e) => {
            error!("Failed to cancel certificate: {}", e);
            Err((
                StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(serde_json::json!({"error": e.to_string()})),
            ))
        }
    }
}

// ============================================================================
// Dashboard Handler
// ============================================================================

pub async fn get_withholding_dashboard(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let org_id = parse_uuid(&claims.org_id)?;
    match state
        .financials
        .withholding_tax_engine
        .get_summary(org_id)
        .await
    {
        Ok(summary) => Ok(Json(
            serde_json::to_value(summary).unwrap_or(serde_json::Value::Null),
        )),
        Err(e) => {
            error!("Failed to get withholding tax dashboard: {}", e);
            Err((
                StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                Json(serde_json::json!({"error": e.to_string()})),
            ))
        }
    }
}
