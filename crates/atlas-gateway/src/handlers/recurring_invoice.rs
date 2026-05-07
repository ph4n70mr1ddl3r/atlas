//! Recurring Invoice Management API Handlers
//!
//! REST endpoints for Oracle Fusion-inspired Recurring Invoice Management.
//! Manages recurring AP invoice templates with automatic generation:
//! draft → active → suspended → completed → cancelled

use axum::{
    extract::{Path, Query, State, Extension},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::AppState;
use crate::handlers::auth::{Claims, parse_uuid};

// ============================================================================
// Query Parameters
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub status: Option<String>,
    pub supplier_id: Option<String>,
}

// ============================================================================
// Template Handlers
// ============================================================================

/// Create a recurring invoice template
pub async fn create_template(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let org_id = parse_uuid(&claims.org_id).unwrap_or_default();
    let user_id = claims.sub.parse().ok();

    let template_number = body["templateNumber"].as_str().unwrap_or("").to_string();
    let template_name = body["templateName"].as_str().unwrap_or("").to_string();
    let description = body["description"].as_str().map(|s| s.to_string());
    let supplier_id = body["supplierId"].as_str().and_then(|s| s.parse().ok());
    let supplier_number = body["supplierNumber"].as_str().map(|s| s.to_string());
    let supplier_name = body["supplierName"].as_str().map(|s| s.to_string());
    let supplier_site = body["supplierSite"].as_str().map(|s| s.to_string());
    let invoice_type = body["invoiceType"].as_str().unwrap_or("standard").to_string();
    let invoice_currency_code = body["invoiceCurrencyCode"].as_str().unwrap_or("USD").to_string();
    let payment_currency_code = body["paymentCurrencyCode"].as_str().map(|s| s.to_string());
    let exchange_rate_type = body["exchangeRateType"].as_str().map(|s| s.to_string());
    let payment_terms = body["paymentTerms"].as_str().map(|s| s.to_string());
    let payment_method = body["paymentMethod"].as_str().map(|s| s.to_string());
    let payment_due_days = body["paymentDueDays"].as_i64().unwrap_or(30) as i32;
    let liability_account_code = body["liabilityAccountCode"].as_str().map(|s| s.to_string());
    let expense_account_code = body["expenseAccountCode"].as_str().map(|s| s.to_string());
    let amount_type = body["amountType"].as_str().unwrap_or("fixed").to_string();
    let recurrence_type = body["recurrenceType"].as_str().unwrap_or("monthly").to_string();
    let recurrence_interval = body["recurrenceInterval"].as_i64().unwrap_or(1) as i32;
    let generation_day = body["generationDay"].as_i64().map(|v| v as i32);
    let days_in_advance = body["daysInAdvance"].as_i64().unwrap_or(0) as i32;
    let effective_from = body["effectiveFrom"].as_str()
        .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
        .unwrap_or(chrono::Utc::now().naive_utc().date());
    let effective_to = body["effectiveTo"].as_str()
        .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());
    let maximum_generations = body["maximumGenerations"].as_i64().map(|v| v as i32);
    let auto_submit = body["autoSubmit"].as_bool().unwrap_or(false);
    let auto_approve = body["autoApprove"].as_bool().unwrap_or(false);
    let hold_for_review = body["holdForReview"].as_bool().unwrap_or(true);
    let po_number = body["poNumber"].as_str().map(|s| s.to_string());
    let gl_date_basis = body["glDateBasis"].as_str().unwrap_or("generation_date").to_string();

    match state.recurring_invoice_engine.create_template(
        org_id,
        &template_number,
        &template_name,
        description.as_deref(),
        supplier_id,
        supplier_number.as_deref(),
        supplier_name.as_deref(),
        supplier_site.as_deref(),
        &invoice_type,
        &invoice_currency_code,
        payment_currency_code.as_deref(),
        exchange_rate_type.as_deref(),
        payment_terms.as_deref(),
        payment_method.as_deref(),
        payment_due_days,
        liability_account_code.as_deref(),
        expense_account_code.as_deref(),
        &amount_type,
        &recurrence_type,
        recurrence_interval,
        generation_day,
        days_in_advance,
        effective_from,
        effective_to,
        maximum_generations,
        auto_submit,
        auto_approve,
        hold_for_review,
        po_number.as_deref(),
        &gl_date_basis,
        user_id,
    ).await {
        Ok(tmpl) => (StatusCode::CREATED, Json(serde_json::to_value(tmpl).unwrap())).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// List recurring invoice templates
pub async fn list_templates(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<ListQuery>,
) -> impl IntoResponse {
    let org_id = parse_uuid(&claims.org_id).unwrap_or_default();
    let supplier_id = query.supplier_id.as_ref().and_then(|s| s.parse().ok());
    match state.recurring_invoice_engine.list_templates(
        org_id,
        query.status.as_deref(),
        supplier_id,
    ).await {
        Ok(templates) => Json(serde_json::json!({"data": templates})).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// Get a template by ID
pub async fn get_template(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.recurring_invoice_engine.get_template(id).await {
        Ok(Some(tmpl)) => Json(serde_json::to_value(tmpl).unwrap()).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Template not found"}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// Transition template status
pub async fn transition_template(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let new_status = body["status"].as_str().unwrap_or("");
    match state.recurring_invoice_engine.transition_template(id, new_status).await {
        Ok(tmpl) => Json(serde_json::to_value(tmpl).unwrap()).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// Delete a draft template by number
pub async fn delete_template(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(template_number): Path<String>,
) -> impl IntoResponse {
    let org_id = parse_uuid(&claims.org_id).unwrap_or_default();
    match state.recurring_invoice_engine.delete_template(org_id, &template_number).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

// ============================================================================
// Template Line Handlers
// ============================================================================

/// Add a line to a template
pub async fn add_template_line(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(template_id): Path<Uuid>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let org_id = parse_uuid(&claims.org_id).unwrap_or_default();

    let line_type = body["lineType"].as_str().unwrap_or("item").to_string();
    let description = body["description"].as_str().map(|s| s.to_string());
    let item_code = body["itemCode"].as_str().map(|s| s.to_string());
    let unit_of_measure = body["unitOfMeasure"].as_str().map(|s| s.to_string());
    let amount = body["amount"].as_f64().unwrap_or(0.0);
    let quantity = body["quantity"].as_f64().unwrap_or(1.0);
    let unit_price = body["unitPrice"].as_f64();
    let gl_account_code = body["glAccountCode"].as_str().unwrap_or("").to_string();
    let cost_center = body["costCenter"].as_str().map(|s| s.to_string());
    let department = body["department"].as_str().map(|s| s.to_string());
    let tax_code = body["taxCode"].as_str().map(|s| s.to_string());
    let tax_amount = body["taxAmount"].as_f64().unwrap_or(0.0);
    let project_id = body["projectId"].as_str().and_then(|s| s.parse().ok());
    let expenditure_type = body["expenditureType"].as_str().map(|s| s.to_string());

    match state.recurring_invoice_engine.add_template_line(
        org_id,
        template_id,
        &line_type,
        description.as_deref(),
        item_code.as_deref(),
        unit_of_measure.as_deref(),
        amount,
        quantity,
        unit_price,
        &gl_account_code,
        cost_center.as_deref(),
        department.as_deref(),
        tax_code.as_deref(),
        tax_amount,
        project_id,
        expenditure_type.as_deref(),
    ).await {
        Ok(line) => (StatusCode::CREATED, Json(serde_json::to_value(line).unwrap())).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// List lines for a template
pub async fn list_template_lines(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(template_id): Path<Uuid>,
) -> impl IntoResponse {
    match state.recurring_invoice_engine.list_template_lines(template_id).await {
        Ok(lines) => Json(serde_json::json!({"data": lines})).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// Remove a line from a template
pub async fn remove_template_line(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path((template_id, line_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    match state.recurring_invoice_engine.remove_template_line(template_id, line_id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

// ============================================================================
// Generation Handlers
// ============================================================================

/// Generate an invoice from a template
pub async fn generate_invoice(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(template_id): Path<Uuid>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let user_id = claims.sub.parse().ok();

    let invoice_date = body["invoiceDate"].as_str()
        .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
        .unwrap_or(chrono::Utc::now().naive_utc().date());
    let period_name = body["periodName"].as_str().map(|s| s.to_string());
    let fiscal_year = body["fiscalYear"].as_i64().map(|v| v as i32);
    let period_number = body["periodNumber"].as_i64().map(|v| v as i32);

    match state.recurring_invoice_engine.generate_invoice(
        template_id,
        invoice_date,
        period_name.as_deref(),
        fiscal_year,
        period_number,
        user_id,
    ).await {
        Ok(gen) => (StatusCode::CREATED, Json(serde_json::to_value(gen).unwrap())).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

/// List generations
pub async fn list_generations(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<GenerationListQuery>,
) -> impl IntoResponse {
    let org_id = parse_uuid(&claims.org_id).unwrap_or_default();
    let template_id = query.template_id.as_ref().and_then(|s| s.parse().ok());
    match state.recurring_invoice_engine.list_generations(
        org_id,
        template_id,
        query.generation_status.as_deref(),
    ).await {
        Ok(generations) => Json(serde_json::json!({"data": generations})).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct GenerationListQuery {
    pub template_id: Option<String>,
    pub generation_status: Option<String>,
}

// ============================================================================
// Dashboard Handler
// ============================================================================

/// Get the recurring invoice dashboard
pub async fn get_dashboard(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> impl IntoResponse {
    let org_id = parse_uuid(&claims.org_id).unwrap_or_default();
    match state.recurring_invoice_engine.get_dashboard(org_id).await {
        Ok(dashboard) => Json(serde_json::to_value(dashboard).unwrap()).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}
