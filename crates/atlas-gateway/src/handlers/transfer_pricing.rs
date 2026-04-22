//! Transfer Pricing API Handlers
//!
//! Oracle Fusion Cloud ERP: Financials > Transfer Pricing
//!
//! Endpoints for managing intercompany transfer price policies,
//! transactions, benchmark studies, comparables, and documentation.

use axum::{
    extract::{Path, Query, State, Extension},
    Json,
    http::StatusCode,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;
use uuid::Uuid;

use crate::AppState;
use crate::handlers::auth::Claims;

// ============================================================================
// Query Parameters
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ListPoliciesQuery {
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListTransactionsQuery {
    pub status: Option<String>,
    pub policy_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListBenchmarksQuery {
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListDocumentationQuery {
    pub doc_type: Option<String>,
    pub status: Option<String>,
}

// ============================================================================
// Policy Handlers
// ============================================================================

/// Create a transfer pricing policy
pub async fn create_policy(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"}))))?;
    let created_by = Uuid::parse_str(&claims.sub).ok();

    let policy_code = body["policy_code"].as_str().unwrap_or("");
    let name = body["name"].as_str().unwrap_or("");
    let description = body["description"].as_str();
    let pricing_method = body["pricing_method"].as_str().unwrap_or("");
    let from_entity_id = body["from_entity_id"].as_str().and_then(|s| Uuid::parse_str(s).ok());
    let from_entity_name = body["from_entity_name"].as_str();
    let to_entity_id = body["to_entity_id"].as_str().and_then(|s| Uuid::parse_str(s).ok());
    let to_entity_name = body["to_entity_name"].as_str();
    let product_category = body["product_category"].as_str();
    let item_id = body["item_id"].as_str().and_then(|s| Uuid::parse_str(s).ok());
    let item_code = body["item_code"].as_str();
    let geography = body["geography"].as_str();
    let tax_jurisdiction = body["tax_jurisdiction"].as_str();
    let effective_from = body["effective_from"].as_str().and_then(|s| s.parse().ok());
    let effective_to = body["effective_to"].as_str().and_then(|s| s.parse().ok());
    let arm_length_range_low = body["arm_length_range_low"].as_str();
    let arm_length_range_mid = body["arm_length_range_mid"].as_str();
    let arm_length_range_high = body["arm_length_range_high"].as_str();
    let margin_pct = body["margin_pct"].as_str();
    let cost_base = body["cost_base"].as_str();

    match state.transfer_pricing_engine.create_policy(
        org_id, policy_code, name, description, pricing_method,
        from_entity_id, from_entity_name, to_entity_id, to_entity_name,
        product_category, item_id, item_code, geography, tax_jurisdiction,
        effective_from, effective_to,
        arm_length_range_low, arm_length_range_mid, arm_length_range_high,
        margin_pct, cost_base, created_by,
    ).await {
        Ok(policy) => Ok((StatusCode::CREATED, Json(serde_json::to_value(policy).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => {
            let msg = format!("{}", e);
            let status = if msg.contains("not found") { StatusCode::NOT_FOUND }
                else if msg.contains("unique") || msg.contains("duplicate") { StatusCode::CONFLICT }
                else { StatusCode::BAD_REQUEST };
            Err((status, Json(json!({"error": msg}))))
        }
    }
}

/// Get a policy by code
pub async fn get_policy(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(code): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"}))))?;

    match state.transfer_pricing_engine.get_policy(org_id, &code).await {
        Ok(Some(policy)) => Ok(Json(serde_json::to_value(policy).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(json!({"error": "Policy not found"})))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("{}", e)})))),
    }
}

/// List policies
pub async fn list_policies(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<ListPoliciesQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"}))))?;

    match state.transfer_pricing_engine.list_policies(org_id, params.status.as_deref()).await {
        Ok(policies) => Ok(Json(json!({"data": policies}))),
        Err(e) => Err((StatusCode::BAD_REQUEST, Json(json!({"error": format!("{}", e)})))),
    }
}

/// Activate a policy
pub async fn activate_policy(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.transfer_pricing_engine.activate_policy(id).await {
        Ok(policy) => Ok(Json(serde_json::to_value(policy).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => {
            let msg = format!("{}", e);
            let status = if msg.contains("not found") { StatusCode::NOT_FOUND } else { StatusCode::BAD_REQUEST };
            Err((status, Json(json!({"error": msg}))))
        }
    }
}

/// Deactivate a policy
pub async fn deactivate_policy(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.transfer_pricing_engine.deactivate_policy(id).await {
        Ok(policy) => Ok(Json(serde_json::to_value(policy).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => {
            let msg = format!("{}", e);
            let status = if msg.contains("not found") { StatusCode::NOT_FOUND } else { StatusCode::BAD_REQUEST };
            Err((status, Json(json!({"error": msg}))))
        }
    }
}

/// Delete a policy
pub async fn delete_policy(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(code): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<Value>)> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"}))))?;

    match state.transfer_pricing_engine.delete_policy(org_id, &code).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            let msg = format!("{}", e);
            let status = if msg.contains("not found") { StatusCode::NOT_FOUND } else { StatusCode::BAD_REQUEST };
            Err((status, Json(json!({"error": msg}))))
        }
    }
}

// ============================================================================
// Transaction Handlers
// ============================================================================

/// Create a transfer price transaction
pub async fn create_transaction(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"}))))?;
    let created_by = Uuid::parse_str(&claims.sub).ok();

    let policy_id = body["policy_id"].as_str().and_then(|s| Uuid::parse_str(s).ok());
    let from_entity_id = body["from_entity_id"].as_str().and_then(|s| Uuid::parse_str(s).ok());
    let from_entity_name = body["from_entity_name"].as_str();
    let to_entity_id = body["to_entity_id"].as_str().and_then(|s| Uuid::parse_str(s).ok());
    let to_entity_name = body["to_entity_name"].as_str();
    let item_id = body["item_id"].as_str().and_then(|s| Uuid::parse_str(s).ok());
    let item_code = body["item_code"].as_str();
    let item_description = body["item_description"].as_str();
    let quantity = body["quantity"].as_str().unwrap_or("0");
    let unit_cost = body["unit_cost"].as_str().unwrap_or("0");
    let transfer_price = body["transfer_price"].as_str().unwrap_or("0");
    let currency_code = body["currency_code"].as_str().unwrap_or("USD");
    let transaction_date: chrono::NaiveDate = body["transaction_date"].as_str()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| (StatusCode::BAD_REQUEST, Json(json!({"error": "transaction_date is required (YYYY-MM-DD)"}))))?;
    let source_type = body["source_type"].as_str();
    let source_id = body["source_id"].as_str().and_then(|s| Uuid::parse_str(s).ok());
    let source_number = body["source_number"].as_str();

    match state.transfer_pricing_engine.create_transaction(
        org_id, policy_id, from_entity_id, from_entity_name,
        to_entity_id, to_entity_name, item_id, item_code, item_description,
        quantity, unit_cost, transfer_price, currency_code, transaction_date,
        source_type, source_id, source_number, created_by,
    ).await {
        Ok(txn) => Ok((StatusCode::CREATED, Json(serde_json::to_value(txn).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => {
            let msg = format!("{}", e);
            let status = if msg.contains("not found") { StatusCode::NOT_FOUND }
                else { StatusCode::BAD_REQUEST };
            Err((status, Json(json!({"error": msg}))))
        }
    }
}

/// Get a transaction
pub async fn get_transaction(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.transfer_pricing_engine.get_transaction(id).await {
        Ok(Some(txn)) => Ok(Json(serde_json::to_value(txn).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(json!({"error": "Transaction not found"})))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("{}", e)})))),
    }
}

/// List transactions
pub async fn list_transactions(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<ListTransactionsQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"}))))?;
    let policy_id = params.policy_id.as_deref().and_then(|s| Uuid::parse_str(s).ok());

    match state.transfer_pricing_engine.list_transactions(org_id, params.status.as_deref(), policy_id).await {
        Ok(txns) => Ok(Json(json!({"data": txns}))),
        Err(e) => Err((StatusCode::BAD_REQUEST, Json(json!({"error": format!("{}", e)})))),
    }
}

/// Submit a transaction
pub async fn submit_transaction(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.transfer_pricing_engine.submit_transaction(id).await {
        Ok(txn) => Ok(Json(serde_json::to_value(txn).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => {
            let msg = format!("{}", e);
            let status = if msg.contains("not found") { StatusCode::NOT_FOUND } else { StatusCode::BAD_REQUEST };
            Err((status, Json(json!({"error": msg}))))
        }
    }
}

/// Approve a transaction
pub async fn approve_transaction(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let approved_by = Uuid::parse_str(&claims.sub).ok();
    match state.transfer_pricing_engine.approve_transaction(id, approved_by).await {
        Ok(txn) => Ok(Json(serde_json::to_value(txn).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => {
            let msg = format!("{}", e);
            let status = if msg.contains("not found") { StatusCode::NOT_FOUND } else { StatusCode::BAD_REQUEST };
            Err((status, Json(json!({"error": msg}))))
        }
    }
}

/// Reject a transaction
pub async fn reject_transaction(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.transfer_pricing_engine.reject_transaction(id).await {
        Ok(txn) => Ok(Json(serde_json::to_value(txn).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => {
            let msg = format!("{}", e);
            let status = if msg.contains("not found") { StatusCode::NOT_FOUND } else { StatusCode::BAD_REQUEST };
            Err((status, Json(json!({"error": msg}))))
        }
    }
}

// ============================================================================
// Benchmark Handlers
// ============================================================================

/// Create a benchmark study
pub async fn create_benchmark(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"}))))?;
    let created_by = Uuid::parse_str(&claims.sub).ok();

    let title = body["title"].as_str().unwrap_or("");
    let description = body["description"].as_str();
    let policy_id = body["policy_id"].as_str().and_then(|s| Uuid::parse_str(s).ok());
    let analysis_method = body["analysis_method"].as_str().unwrap_or("");
    let fiscal_year = body["fiscal_year"].as_i64().map(|v| v as i32);
    let from_entity_id = body["from_entity_id"].as_str().and_then(|s| Uuid::parse_str(s).ok());
    let from_entity_name = body["from_entity_name"].as_str();
    let to_entity_id = body["to_entity_id"].as_str().and_then(|s| Uuid::parse_str(s).ok());
    let to_entity_name = body["to_entity_name"].as_str();
    let product_category = body["product_category"].as_str();
    let tested_party = body["tested_party"].as_str();
    let prepared_by = body["prepared_by"].as_str().and_then(|s| Uuid::parse_str(s).ok());
    let prepared_by_name = body["prepared_by_name"].as_str();

    match state.transfer_pricing_engine.create_benchmark(
        org_id, title, description, policy_id, analysis_method, fiscal_year,
        from_entity_id, from_entity_name, to_entity_id, to_entity_name,
        product_category, tested_party, prepared_by, prepared_by_name, created_by,
    ).await {
        Ok(bm) => Ok((StatusCode::CREATED, Json(serde_json::to_value(bm).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => Err((StatusCode::BAD_REQUEST, Json(json!({"error": format!("{}", e)})))),
    }
}

/// Get a benchmark
pub async fn get_benchmark(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.transfer_pricing_engine.get_benchmark(id).await {
        Ok(Some(bm)) => Ok(Json(serde_json::to_value(bm).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(json!({"error": "Benchmark not found"})))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("{}", e)})))),
    }
}

/// List benchmarks
pub async fn list_benchmarks(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<ListBenchmarksQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"}))))?;

    match state.transfer_pricing_engine.list_benchmarks(org_id, params.status.as_deref()).await {
        Ok(bms) => Ok(Json(json!({"data": bms}))),
        Err(e) => Err((StatusCode::BAD_REQUEST, Json(json!({"error": format!("{}", e)})))),
    }
}

/// Submit benchmark for review
pub async fn submit_benchmark(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.transfer_pricing_engine.submit_benchmark_for_review(id).await {
        Ok(bm) => Ok(Json(serde_json::to_value(bm).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => {
            let msg = format!("{}", e);
            let status = if msg.contains("not found") { StatusCode::NOT_FOUND } else { StatusCode::BAD_REQUEST };
            Err((status, Json(json!({"error": msg}))))
        }
    }
}

/// Approve a benchmark
pub async fn approve_benchmark(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let reviewed_by = Uuid::parse_str(&claims.sub).ok();
    match state.transfer_pricing_engine.approve_benchmark(id, reviewed_by, None).await {
        Ok(bm) => Ok(Json(serde_json::to_value(bm).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => {
            let msg = format!("{}", e);
            let status = if msg.contains("not found") { StatusCode::NOT_FOUND } else { StatusCode::BAD_REQUEST };
            Err((status, Json(json!({"error": msg}))))
        }
    }
}

/// Reject a benchmark
pub async fn reject_benchmark(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.transfer_pricing_engine.reject_benchmark(id).await {
        Ok(bm) => Ok(Json(serde_json::to_value(bm).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => {
            let msg = format!("{}", e);
            let status = if msg.contains("not found") { StatusCode::NOT_FOUND } else { StatusCode::BAD_REQUEST };
            Err((status, Json(json!({"error": msg}))))
        }
    }
}

/// Delete a benchmark
pub async fn delete_benchmark(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<Value>)> {
    match state.transfer_pricing_engine.delete_benchmark(id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            let msg = format!("{}", e);
            let status = if msg.contains("not found") { StatusCode::NOT_FOUND } else { StatusCode::BAD_REQUEST };
            Err((status, Json(json!({"error": msg}))))
        }
    }
}

// ============================================================================
// Comparable Handlers
// ============================================================================

/// Add a comparable to a benchmark
pub async fn add_comparable(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(benchmark_id): Path<Uuid>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"}))))?;

    let comparable_number = body["comparable_number"].as_i64().unwrap_or(1) as i32;
    let company_name = body["company_name"].as_str().unwrap_or("");
    let country = body["country"].as_str();
    let industry_code = body["industry_code"].as_str();
    let industry_description = body["industry_description"].as_str();
    let fiscal_year = body["fiscal_year"].as_i64().map(|v| v as i32);
    let revenue = body["revenue"].as_str();
    let operating_income = body["operating_income"].as_str();
    let operating_margin_pct = body["operating_margin_pct"].as_str();
    let net_income = body["net_income"].as_str();
    let total_assets = body["total_assets"].as_str();
    let employees = body["employees"].as_i64().map(|v| v as i32);
    let data_source = body["data_source"].as_str();

    match state.transfer_pricing_engine.add_comparable(
        org_id, benchmark_id, comparable_number, company_name,
        country, industry_code, industry_description, fiscal_year,
        revenue, operating_income, operating_margin_pct,
        net_income, total_assets, employees, data_source,
    ).await {
        Ok(comp) => Ok((StatusCode::CREATED, Json(serde_json::to_value(comp).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => {
            let msg = format!("{}", e);
            let status = if msg.contains("not found") { StatusCode::NOT_FOUND } else { StatusCode::BAD_REQUEST };
            Err((status, Json(json!({"error": msg}))))
        }
    }
}

/// List comparables for a benchmark
pub async fn list_comparables(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(benchmark_id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.transfer_pricing_engine.list_comparables(benchmark_id).await {
        Ok(comps) => Ok(Json(json!({"data": comps}))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("{}", e)})))),
    }
}

// ============================================================================
// Documentation Handlers
// ============================================================================

/// Create documentation package
pub async fn create_documentation(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"}))))?;
    let created_by = Uuid::parse_str(&claims.sub).ok();

    let title = body["title"].as_str().unwrap_or("");
    let doc_type = body["doc_type"].as_str().unwrap_or("");
    let fiscal_year = body["fiscal_year"].as_i64().unwrap_or(0) as i32;
    let country = body["country"].as_str();
    let reporting_entity_id = body["reporting_entity_id"].as_str().and_then(|s| Uuid::parse_str(s).ok());
    let reporting_entity_name = body["reporting_entity_name"].as_str();
    let description = body["description"].as_str();
    let content_summary = body["content_summary"].as_str();
    let filing_deadline = body["filing_deadline"].as_str().and_then(|s| s.parse().ok());
    let responsible_party = body["responsible_party"].as_str();

    match state.transfer_pricing_engine.create_documentation(
        org_id, title, doc_type, fiscal_year, country,
        reporting_entity_id, reporting_entity_name, description,
        content_summary, filing_deadline, responsible_party, created_by,
    ).await {
        Ok(doc) => Ok((StatusCode::CREATED, Json(serde_json::to_value(doc).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null })))),
        Err(e) => Err((StatusCode::BAD_REQUEST, Json(json!({"error": format!("{}", e)})))),
    }
}

/// Get documentation
pub async fn get_documentation(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.transfer_pricing_engine.get_documentation(id).await {
        Ok(Some(doc)) => Ok(Json(serde_json::to_value(doc).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(json!({"error": "Documentation not found"})))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("{}", e)})))),
    }
}

/// List documentation
pub async fn list_documentation(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<ListDocumentationQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"}))))?;

    match state.transfer_pricing_engine.list_documentation(
        org_id, params.doc_type.as_deref(), params.status.as_deref()
    ).await {
        Ok(docs) => Ok(Json(json!({"data": docs}))),
        Err(e) => Err((StatusCode::BAD_REQUEST, Json(json!({"error": format!("{}", e)})))),
    }
}

/// Submit documentation for review
pub async fn submit_documentation(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.transfer_pricing_engine.submit_documentation_for_review(id).await {
        Ok(doc) => Ok(Json(serde_json::to_value(doc).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => {
            let msg = format!("{}", e);
            let status = if msg.contains("not found") { StatusCode::NOT_FOUND } else { StatusCode::BAD_REQUEST };
            Err((status, Json(json!({"error": msg}))))
        }
    }
}

/// Approve documentation
pub async fn approve_documentation(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let approved_by = Uuid::parse_str(&claims.sub).ok();
    match state.transfer_pricing_engine.approve_documentation(id, approved_by).await {
        Ok(doc) => Ok(Json(serde_json::to_value(doc).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => {
            let msg = format!("{}", e);
            let status = if msg.contains("not found") { StatusCode::NOT_FOUND } else { StatusCode::BAD_REQUEST };
            Err((status, Json(json!({"error": msg}))))
        }
    }
}

/// File documentation
pub async fn file_documentation(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.transfer_pricing_engine.file_documentation(id).await {
        Ok(doc) => Ok(Json(serde_json::to_value(doc).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => {
            let msg = format!("{}", e);
            let status = if msg.contains("not found") { StatusCode::NOT_FOUND } else { StatusCode::BAD_REQUEST };
            Err((status, Json(json!({"error": msg}))))
        }
    }
}

// ============================================================================
// Dashboard
// ============================================================================

/// Get transfer pricing dashboard
pub async fn get_tp_dashboard(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = Uuid::parse_str(&claims.org_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"}))))?;

    match state.transfer_pricing_engine.get_dashboard(org_id).await {
        Ok(dashboard) => Ok(Json(serde_json::to_value(dashboard).unwrap_or_else(|e| { tracing::error!("Serialization error: {}", e); serde_json::Value::Null }))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": format!("{}", e)})))),
    }
}
