//! Hedge Management API Handlers
//!
//! Oracle Fusion Cloud ERP: Treasury > Hedge Management
//!
//! Endpoints for managing derivative instruments, hedge relationships,
//! effectiveness testing, and hedge documentation.

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
use crate::handlers::auth::{Claims, parse_uuid};

// ============================================================================
// Query Parameters
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct StatusQuery {
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DerivativeListQuery {
    pub status: Option<String>,
    pub instrument_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct HedgeListQuery {
    pub status: Option<String>,
    pub hedge_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DocumentationListQuery {
    pub hedge_relationship_id: Option<String>,
}

// ============================================================================
// Derivative Instruments
// ============================================================================

/// Create a derivative instrument
pub async fn create_derivative(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    let instrument_type = body["instrument_type"].as_str().unwrap_or("").to_string();
    let underlying_type = body["underlying_type"].as_str().unwrap_or("").to_string();
    let underlying_description = body["underlying_description"].as_str();
    let currency_code = body["currency_code"].as_str().unwrap_or("USD").to_string();
    let counter_currency_code = body["counter_currency_code"].as_str();
    let notional_amount = body["notional_amount"].as_str().unwrap_or("0").to_string();
    let strike_rate = body["strike_rate"].as_str();
    let forward_rate = body["forward_rate"].as_str();
    let spot_rate = body["spot_rate"].as_str();
    let option_type = body["option_type"].as_str();
    let premium_amount = body["premium_amount"].as_str();
    let trade_date = body["trade_date"].as_str().and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let effective_date = body["effective_date"].as_str().and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let maturity_date = body["maturity_date"].as_str().and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let settlement_date = body["settlement_date"].as_str().and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let settlement_type = body["settlement_type"].as_str();
    let counterparty_name = body["counterparty_name"].as_str();
    let counterparty_reference = body["counterparty_reference"].as_str();
    let portfolio_code = body["portfolio_code"].as_str();
    let trading_book = body["trading_book"].as_str();
    let accounting_treatment = body["accounting_treatment"].as_str();
    let risk_factor = body["risk_factor"].as_str();
    let notes = body["notes"].as_str();

    match state.hedge_management_engine.create_derivative(
        org_id, &instrument_type, &underlying_type, underlying_description,
        &currency_code, counter_currency_code, &notional_amount,
        strike_rate, forward_rate, spot_rate, option_type, premium_amount,
        trade_date, effective_date, maturity_date, settlement_date,
        settlement_type, counterparty_name, counterparty_reference,
        portfolio_code, trading_book, accounting_treatment, risk_factor,
        notes, parse_uuid(&claims.sub).ok(),
    ).await {
        Ok(deriv) => Ok((StatusCode::CREATED, Json(serde_json::to_value(deriv).unwrap_or(Value::Null)))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Get a derivative by instrument number
pub async fn get_derivative(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(instrument_number): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    match state.hedge_management_engine.get_derivative(org_id, &instrument_number).await {
        Ok(Some(deriv)) => Ok(Json(serde_json::to_value(deriv).unwrap_or(Value::Null))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(json!({"error": "Derivative not found"})))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// List derivatives with optional filters
pub async fn list_derivatives(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<DerivativeListQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    match state.hedge_management_engine.list_derivatives(
        org_id, params.status.as_deref(), params.instrument_type.as_deref(),
    ).await {
        Ok(derivs) => Ok(Json(json!({"data": derivs}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Activate a derivative
pub async fn activate_derivative(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.hedge_management_engine.activate_derivative(id).await {
        Ok(deriv) => Ok(Json(serde_json::to_value(deriv).unwrap_or(Value::Null))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Mature a derivative
pub async fn mature_derivative(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.hedge_management_engine.mature_derivative(id).await {
        Ok(deriv) => Ok(Json(serde_json::to_value(deriv).unwrap_or(Value::Null))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Settle a derivative
pub async fn settle_derivative(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.hedge_management_engine.settle_derivative(id).await {
        Ok(deriv) => Ok(Json(serde_json::to_value(deriv).unwrap_or(Value::Null))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Cancel a derivative
pub async fn cancel_derivative(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.hedge_management_engine.cancel_derivative(id).await {
        Ok(deriv) => Ok(Json(serde_json::to_value(deriv).unwrap_or(Value::Null))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Update derivative valuation
pub async fn update_derivative_valuation(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let fair_value = body["fair_value"].as_str().unwrap_or("0");
    let unrealized_gain_loss = body["unrealized_gain_loss"].as_str().unwrap_or("0");
    let valuation_method = body["valuation_method"].as_str();

    match state.hedge_management_engine.update_derivative_valuation(
        id, fair_value, unrealized_gain_loss, valuation_method,
    ).await {
        Ok(deriv) => Ok(Json(serde_json::to_value(deriv).unwrap_or(Value::Null))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Delete a derivative
pub async fn delete_derivative(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(instrument_number): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    match state.hedge_management_engine.delete_derivative(org_id, &instrument_number).await {
        Ok(()) => Ok(Json(json!({"message": "Derivative deleted"}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Hedge Relationships
// ============================================================================

/// Create a hedge relationship
pub async fn create_hedge_relationship(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    let hedge_type = body["hedge_type"].as_str().unwrap_or("").to_string();
    let derivative_id = body["derivative_id"].as_str().and_then(|s| Uuid::parse_str(s).ok());
    let derivative_number = body["derivative_number"].as_str();
    let hedged_item_description = body["hedged_item_description"].as_str();
    let hedged_item_id = body["hedged_item_id"].as_str().and_then(|s| Uuid::parse_str(s).ok());
    let hedged_risk = body["hedged_risk"].as_str().unwrap_or("").to_string();
    let hedge_strategy = body["hedge_strategy"].as_str();
    let hedged_item_reference = body["hedged_item_reference"].as_str();
    let hedged_item_currency = body["hedged_item_currency"].as_str();
    let hedged_amount = body["hedged_amount"].as_str().unwrap_or("0").to_string();
    let hedge_ratio = body["hedge_ratio"].as_str();
    let designated_start_date = body["designated_start_date"].as_str().and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let designated_end_date = body["designated_end_date"].as_str().and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let effectiveness_method = body["effectiveness_method"].as_str().unwrap_or("dollar_offset").to_string();
    let critical_terms_match = body["critical_terms_match"].as_str();
    let hedge_documentation_ref = body["hedge_documentation_ref"].as_str();
    let notes = body["notes"].as_str();

    match state.hedge_management_engine.create_hedge_relationship(
        org_id, &hedge_type, derivative_id, derivative_number,
        hedged_item_description, hedged_item_id, &hedged_risk,
        hedge_strategy, hedged_item_reference, hedged_item_currency,
        &hedged_amount, hedge_ratio, designated_start_date, designated_end_date,
        &effectiveness_method, critical_terms_match, hedge_documentation_ref,
        notes, parse_uuid(&claims.sub).ok(),
    ).await {
        Ok(hedge) => Ok((StatusCode::CREATED, Json(serde_json::to_value(hedge).unwrap_or(Value::Null)))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Get a hedge relationship by hedge_id
pub async fn get_hedge_relationship(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(hedge_id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    match state.hedge_management_engine.get_hedge_relationship(org_id, &hedge_id).await {
        Ok(Some(hedge)) => Ok(Json(serde_json::to_value(hedge).unwrap_or(Value::Null))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(json!({"error": "Hedge relationship not found"})))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// List hedge relationships
pub async fn list_hedge_relationships(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<HedgeListQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    match state.hedge_management_engine.list_hedge_relationships(
        org_id, params.status.as_deref(), params.hedge_type.as_deref(),
    ).await {
        Ok(hedges) => Ok(Json(json!({"data": hedges}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Designate a hedge
pub async fn designate_hedge(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.hedge_management_engine.designate_hedge(id).await {
        Ok(hedge) => Ok(Json(serde_json::to_value(hedge).unwrap_or(Value::Null))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Activate a hedge
pub async fn activate_hedge(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.hedge_management_engine.activate_hedge(id).await {
        Ok(hedge) => Ok(Json(serde_json::to_value(hedge).unwrap_or(Value::Null))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// De-designate a hedge
pub async fn de_designate_hedge(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.hedge_management_engine.de_designate_hedge(id).await {
        Ok(hedge) => Ok(Json(serde_json::to_value(hedge).unwrap_or(Value::Null))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Terminate a hedge
pub async fn terminate_hedge(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.hedge_management_engine.terminate_hedge(id).await {
        Ok(hedge) => Ok(Json(serde_json::to_value(hedge).unwrap_or(Value::Null))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Delete a hedge relationship
pub async fn delete_hedge_relationship(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(hedge_id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    match state.hedge_management_engine.delete_hedge_relationship(org_id, &hedge_id).await {
        Ok(()) => Ok(Json(json!({"message": "Hedge relationship deleted"}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Effectiveness Testing
// ============================================================================

/// Run an effectiveness test
pub async fn run_effectiveness_test(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    let hedge_relationship_id = body["hedge_relationship_id"].as_str()
        .and_then(|s| Uuid::parse_str(s).ok())
        .unwrap_or_else(Uuid::nil);
    let test_type = body["test_type"].as_str().unwrap_or("ongoing").to_string();
    let test_date = body["test_date"].as_str()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .unwrap_or_else(|| chrono::Utc::now().date_naive());
    let derivative_fair_value_change = body["derivative_fair_value_change"].as_str().unwrap_or("0").to_string();
    let hedged_item_fair_value_change = body["hedged_item_fair_value_change"].as_str().unwrap_or("0").to_string();
    let test_period_start = body["test_period_start"].as_str().and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let test_period_end = body["test_period_end"].as_str().and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let notes = body["notes"].as_str();

    match state.hedge_management_engine.run_effectiveness_test(
        org_id, hedge_relationship_id, &test_type, test_date,
        &derivative_fair_value_change, &hedged_item_fair_value_change,
        test_period_start, test_period_end, notes, parse_uuid(&claims.sub).ok(),
    ).await {
        Ok(test) => Ok((StatusCode::CREATED, Json(serde_json::to_value(test).unwrap_or(Value::Null)))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Get an effectiveness test
pub async fn get_effectiveness_test(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.hedge_management_engine.get_effectiveness_test(id).await {
        Ok(Some(test)) => Ok(Json(serde_json::to_value(test).unwrap_or(Value::Null))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(json!({"error": "Effectiveness test not found"})))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// List effectiveness tests for a hedge relationship
pub async fn list_effectiveness_tests(
    State(state): State<Arc<AppState>>,
    Extension(_claims): Extension<Claims>,
    Path(hedge_relationship_id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.hedge_management_engine.list_effectiveness_tests(hedge_relationship_id).await {
        Ok(tests) => Ok(Json(json!({"data": tests}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Documentation
// ============================================================================

/// Create hedge documentation
pub async fn create_documentation(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    let hedge_relationship_id = body["hedge_relationship_id"].as_str().and_then(|s| Uuid::parse_str(s).ok());
    let hedge_id = body["hedge_id"].as_str();
    let hedge_type = body["hedge_type"].as_str().unwrap_or("fair_value").to_string();
    let risk_management_objective = body["risk_management_objective"].as_str();
    let hedging_strategy_description = body["hedging_strategy_description"].as_str();
    let hedged_item_description = body["hedged_item_description"].as_str();
    let hedged_risk_description = body["hedged_risk_description"].as_str();
    let derivative_description = body["derivative_description"].as_str();
    let effectiveness_method_description = body["effectiveness_method_description"].as_str();
    let assessment_frequency = body["assessment_frequency"].as_str();
    let designation_date = body["designation_date"].as_str().and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let documentation_date = body["documentation_date"].as_str().and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let prepared_by = body["prepared_by"].as_str();
    let notes = body["notes"].as_str();

    match state.hedge_management_engine.create_documentation(
        org_id, hedge_relationship_id, hedge_id, &hedge_type,
        risk_management_objective, hedging_strategy_description,
        hedged_item_description, hedged_risk_description,
        derivative_description, effectiveness_method_description,
        assessment_frequency, designation_date, documentation_date,
        prepared_by, notes, parse_uuid(&claims.sub).ok(),
    ).await {
        Ok(doc) => Ok((StatusCode::CREATED, Json(serde_json::to_value(doc).unwrap_or(Value::Null)))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Get documentation by document number
pub async fn get_documentation(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(document_number): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    match state.hedge_management_engine.get_documentation(org_id, &document_number).await {
        Ok(Some(doc)) => Ok(Json(serde_json::to_value(doc).unwrap_or(Value::Null))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(json!({"error": "Documentation not found"})))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// List documentation
pub async fn list_documentation(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<DocumentationListQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    let hedge_relationship_id = params.hedge_relationship_id.as_deref()
        .and_then(|s| Uuid::parse_str(s).ok());

    match state.hedge_management_engine.list_documentation(org_id, hedge_relationship_id).await {
        Ok(docs) => Ok(Json(json!({"data": docs}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Approve documentation
pub async fn approve_documentation(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.hedge_management_engine.approve_documentation(id, parse_uuid(&claims.sub).ok()).await {
        Ok(doc) => Ok(Json(serde_json::to_value(doc).unwrap_or(Value::Null))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

/// Delete documentation
pub async fn delete_documentation(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(document_number): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    match state.hedge_management_engine.delete_documentation(org_id, &document_number).await {
        Ok(()) => Ok(Json(json!({"message": "Documentation deleted"}))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}

// ============================================================================
// Dashboard
// ============================================================================

/// Get hedge management dashboard
pub async fn get_hedge_dashboard(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let org_id = match Uuid::parse_str(&claims.org_id) {
        Ok(id) => id,
        Err(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid org_id"})))),
    };

    match state.hedge_management_engine.get_dashboard_summary(org_id).await {
        Ok(summary) => Ok(Json(serde_json::to_value(summary).unwrap_or(Value::Null))),
        Err(e) => Err((StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(json!({"error": e.to_string()})))),
    }
}
