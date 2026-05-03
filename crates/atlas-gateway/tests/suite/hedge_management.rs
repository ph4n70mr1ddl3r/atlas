//! Hedge Management E2E Tests
//!
//! Tests for Oracle Fusion Cloud ERP Treasury > Hedge Management:
//! - Derivative instrument CRUD and lifecycle (draft → active → matured → settled)
//! - Derivative valuation (mark-to-market)
//! - Hedge relationship designation and lifecycle
//! - Effectiveness testing (dollar-offset method)
//! - Hedge documentation creation and approval
//! - Dashboard summary
//! - Validation edge cases

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;
use uuid::Uuid;

async fn setup_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    sqlx::query("CREATE SCHEMA IF NOT EXISTS _atlas")
        .execute(&state.db_pool)
        .await
        .ok();
    sqlx::query(include_str!("../../../../migrations/123_hedge_management.sql"))
        .execute(&state.db_pool)
        .await
        .ok();
    let app = build_router(state.clone());
    (state, app)
}

async fn create_derivative(
    app: &axum::Router,
    instrument_type: &str,
    notional: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "instrument_type": instrument_type,
        "underlying_type": "fx",
        "underlying_description": "EUR/USD forward contract",
        "currency_code": "USD",
        "counter_currency_code": "EUR",
        "notional_amount": notional,
        "trade_date": "2024-01-15",
        "effective_date": "2024-02-01",
        "maturity_date": "2024-08-01",
        "counterparty_name": "Goldman Sachs",
        "portfolio_code": "TREASURY-01",
        "accounting_treatment": "hedging",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/hedge/derivatives")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8_lossy(&b);
    eprintln!("CREATE DERIVATIVE status={}: {}", status, body_str);
    assert_eq!(status, StatusCode::CREATED, "Failed to create derivative: {:?}", body_str);
    serde_json::from_slice(&b).unwrap()
}

async fn create_hedge(
    app: &axum::Router,
    hedge_type: &str,
    hedged_amount: &str,
    derivative_id: Option<Uuid>,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let mut payload = json!({
        "hedge_type": hedge_type,
        "hedged_risk": "fx",
        "hedged_item_description": "EUR receivables",
        "hedged_amount": hedged_amount,
        "hedged_item_currency": "EUR",
        "effectiveness_method": "dollar_offset",
        "designated_start_date": "2024-02-01",
        "designated_end_date": "2024-08-01",
    });
    if let Some(did) = derivative_id {
        payload["derivative_id"] = json!(did.to_string());
    }
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/hedge/relationships")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    eprintln!("CREATE HEDGE status={}: {}", status, String::from_utf8_lossy(&b));
    assert_eq!(status, StatusCode::CREATED, "Failed to create hedge relationship");
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Derivative Instrument Tests
// ============================================================================

#[tokio::test]
async fn test_create_derivative() {
    let (_state, app) = setup_test().await;
    let deriv = create_derivative(&app, "forward", "1000000.00").await;

    assert_eq!(deriv["instrumentType"], "forward");
    assert_eq!(deriv["underlyingType"], "fx");
    assert_eq!(deriv["currencyCode"], "USD");
    assert_eq!(deriv["counterCurrencyCode"], "EUR");
    assert!(deriv["instrumentNumber"].as_str().unwrap().starts_with("DERIV-"));
    assert_eq!(deriv["status"], "draft");
}

#[tokio::test]
async fn test_get_derivative() {
    let (_state, app) = setup_test().await;
    let deriv = create_derivative(&app, "swap", "5000000.00").await;
    let instrument_number = deriv["instrumentNumber"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/hedge/derivatives/{}", instrument_number))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["instrumentType"], "swap");
}

#[tokio::test]
async fn test_list_derivatives() {
    let (_state, app) = setup_test().await;
    create_derivative(&app, "forward", "1000000.00").await;
    create_derivative(&app, "option", "2000000.00").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/hedge/derivatives")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(body["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_list_derivatives_with_type_filter() {
    let (_state, app) = setup_test().await;
    create_derivative(&app, "forward", "1000000.00").await;
    create_derivative(&app, "option", "2000000.00").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/hedge/derivatives?instrument_type=forward")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let data = body["data"].as_array().unwrap();
    assert!(data.len() >= 1);
    for d in data {
        assert_eq!(d["instrumentType"], "forward");
    }
}

#[tokio::test]
async fn test_derivative_lifecycle() {
    let (_state, app) = setup_test().await;
    let deriv = create_derivative(&app, "forward", "3000000.00").await;
    let deriv_id: Uuid = deriv["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Activate
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/hedge/derivatives/{}/activate", deriv_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "active");

    // Mark-to-market valuation
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/hedge/derivatives/{}/valuation", deriv_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "fair_value": "15000.00",
            "unrealized_gain_loss": "15000.00",
            "valuation_method": "market_quote"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["fairValue"], "15000.00");

    // Mature
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/hedge/derivatives/{}/mature", deriv_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "matured");

    // Settle
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/hedge/derivatives/{}/settle", deriv_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "settled");
}

#[tokio::test]
async fn test_cancel_derivative() {
    let (_state, app) = setup_test().await;
    let deriv = create_derivative(&app, "option", "1000000.00").await;
    let deriv_id: Uuid = deriv["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/hedge/derivatives/{}/cancel", deriv_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "cancelled");
}

#[tokio::test]
async fn test_delete_derivative() {
    let (_state, app) = setup_test().await;
    let deriv = create_derivative(&app, "cap", "500000.00").await;
    let instrument_number = deriv["instrumentNumber"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/hedge/derivatives/{}", instrument_number))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}

// ============================================================================
// Hedge Relationship Tests
// ============================================================================

#[tokio::test]
async fn test_create_hedge_relationship() {
    let (_state, app) = setup_test().await;
    let hedge = create_hedge(&app, "cash_flow", "500000.00", None).await;

    assert_eq!(hedge["hedgeType"], "cash_flow");
    assert_eq!(hedge["hedgedRisk"], "fx");
    assert!(hedge["hedgeId"].as_str().unwrap().starts_with("HEDGE-"));
    assert_eq!(hedge["status"], "draft");
    assert_eq!(hedge["effectivenessMethod"], "dollar_offset");
}

#[tokio::test]
async fn test_get_hedge_relationship() {
    let (_state, app) = setup_test().await;
    let hedge = create_hedge(&app, "fair_value", "1000000.00", None).await;
    let hedge_id = hedge["hedgeId"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/hedge/relationships/{}", hedge_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["hedgeType"], "fair_value");
}

#[tokio::test]
async fn test_list_hedge_relationships() {
    let (_state, app) = setup_test().await;
    create_hedge(&app, "cash_flow", "500000.00", None).await;
    create_hedge(&app, "fair_value", "800000.00", None).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/hedge/relationships")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(body["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_hedge_lifecycle() {
    let (_state, app) = setup_test().await;
    let hedge = create_hedge(&app, "cash_flow", "1000000.00", None).await;
    let hedge_id_uuid: Uuid = hedge["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Designate
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/hedge/relationships/{}/designate", hedge_id_uuid))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "designated");

    // Activate
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/hedge/relationships/{}/activate", hedge_id_uuid))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "active");

    // De-designate
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/hedge/relationships/{}/de-designate", hedge_id_uuid))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "de-designated");

    // Terminate
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/hedge/relationships/{}/terminate", hedge_id_uuid))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "terminated");
}

#[tokio::test]
async fn test_create_hedge_with_derivative() {
    let (_state, app) = setup_test().await;
    let deriv = create_derivative(&app, "forward", "1000000.00").await;
    let deriv_id: Uuid = deriv["id"].as_str().unwrap().parse().unwrap();

    let hedge = create_hedge(&app, "cash_flow", "1000000.00", Some(deriv_id)).await;
    assert_eq!(hedge["derivativeId"], deriv_id.to_string());
}

#[tokio::test]
async fn test_delete_hedge_relationship() {
    let (_state, app) = setup_test().await;
    let hedge = create_hedge(&app, "fair_value", "500000.00", None).await;
    let hedge_id = hedge["hedgeId"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/hedge/relationships/{}", hedge_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}

// ============================================================================
// Effectiveness Testing
// ============================================================================

#[tokio::test]
async fn test_run_effectiveness_test() {
    let (_state, app) = setup_test().await;
    let hedge = create_hedge(&app, "cash_flow", "1000000.00", None).await;
    let hedge_id_uuid: Uuid = hedge["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Designate and activate
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/hedge/relationships/{}/designate", hedge_id_uuid))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/hedge/relationships/{}/activate", hedge_id_uuid))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Run effectiveness test - effective case (1:1 ratio)
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/hedge/effectiveness-tests")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "hedge_relationship_id": hedge_id_uuid.to_string(),
            "test_type": "ongoing",
            "test_date": "2024-03-15",
            "derivative_fair_value_change": "10000.00",
            "hedged_item_fair_value_change": "10500.00"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::CREATED);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();

    assert_eq!(body["effectivenessResult"], "effective");
    assert_eq!(body["testType"], "ongoing");
    assert_eq!(body["status"], "completed");
    let ratio: f64 = body["hedgeRatioResult"].as_str().unwrap().parse().unwrap();
    assert!((ratio - 0.952).abs() < 0.01);
}

#[tokio::test]
async fn test_ineffective_hedge() {
    let (_state, app) = setup_test().await;
    let hedge = create_hedge(&app, "fair_value", "2000000.00", None).await;
    let hedge_id_uuid: Uuid = hedge["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Designate and activate
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/hedge/relationships/{}/designate", hedge_id_uuid))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/hedge/relationships/{}/activate", hedge_id_uuid))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Run test with very low ratio (ineffective)
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/hedge/effectiveness-tests")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "hedge_relationship_id": hedge_id_uuid.to_string(),
            "test_type": "retrospective",
            "test_date": "2024-06-30",
            "derivative_fair_value_change": "2000.00",
            "hedged_item_fair_value_change": "50000.00"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::CREATED);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["effectivenessResult"], "ineffective");
}

#[tokio::test]
async fn test_list_effectiveness_tests() {
    let (_state, app) = setup_test().await;
    let hedge = create_hedge(&app, "cash_flow", "500000.00", None).await;
    let hedge_id_uuid: Uuid = hedge["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Designate and activate
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/hedge/relationships/{}/designate", hedge_id_uuid))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/hedge/relationships/{}/activate", hedge_id_uuid))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Run two tests
    for i in 0..2 {
        app.clone().oneshot(Request::builder().method("POST")
            .uri("/api/v1/hedge/effectiveness-tests")
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({
                "hedge_relationship_id": hedge_id_uuid.to_string(),
                "test_type": "ongoing",
                "test_date": format!("2024-0{}-15", i + 3),
                "derivative_fair_value_change": "10000.00",
                "hedged_item_fair_value_change": "10500.00"
            })).unwrap()))
            .unwrap()
        ).await.unwrap();
    }

    // List tests
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/hedge/relationships/{}/tests", hedge_id_uuid))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["data"].as_array().unwrap().len(), 2);
}

// ============================================================================
// Documentation Tests
// ============================================================================

#[tokio::test]
async fn test_create_documentation() {
    let (_state, app) = setup_test().await;
    let hedge = create_hedge(&app, "cash_flow", "1000000.00", None).await;
    let hedge_id_str = hedge["hedgeId"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "hedge_relationship_id": hedge["id"].as_str().unwrap(),
        "hedge_id": hedge_id_str,
        "hedge_type": "cash_flow",
        "risk_management_objective": "Hedge EUR/USD FX risk on forecasted sales",
        "hedging_strategy_description": "Use forward contracts to hedge 100% of forecasted EUR revenue",
        "hedged_item_description": "Forecasted EUR revenue Q2-Q3 2024",
        "hedged_risk_description": "EUR/USD exchange rate risk",
        "derivative_description": "6-month EUR/USD forward contract",
        "effectiveness_method_description": "Dollar-offset method with 80-125% threshold",
        "assessment_frequency": "Monthly",
        "designation_date": "2024-02-01",
        "documentation_date": "2024-01-20",
        "prepared_by": "Treasury Analyst",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/hedge/documentation")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    eprintln!("CREATE DOC status={}: {}", status, String::from_utf8_lossy(&b));
    assert_eq!(status, StatusCode::CREATED);
    let doc: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(doc["documentNumber"].as_str().unwrap().starts_with("HDOC-"));
    assert_eq!(doc["hedgeType"], "cash_flow");
    assert_eq!(doc["status"], "draft");
}

#[tokio::test]
async fn test_approve_documentation() {
    let (_state, app) = setup_test().await;

    let (k, v) = auth_header(&admin_claims());

    // Create documentation without hedge relationship link
    let payload = json!({
        "hedge_type": "fair_value",
        "risk_management_objective": "Test objective",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/hedge/documentation")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let doc: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let doc_id: Uuid = doc["id"].as_str().unwrap().parse().unwrap();

    // Approve
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/hedge/documentation/{}/approve", doc_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "approved");
}

#[tokio::test]
async fn test_list_documentation() {
    let (_state, app) = setup_test().await;

    let (k, v) = auth_header(&admin_claims());

    // Create two docs
    for ht in &["cash_flow", "fair_value"] {
        let payload = json!({ "hedge_type": ht });
        app.clone().oneshot(Request::builder().method("POST")
            .uri("/api/v1/hedge/documentation")
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(serde_json::to_string(&payload).unwrap()))
            .unwrap()
        ).await.unwrap();
    }

    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/hedge/documentation")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(body["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_delete_documentation() {
    let (_state, app) = setup_test().await;

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({ "hedge_type": "cash_flow" });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/hedge/documentation")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    let doc: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let doc_number = doc["documentNumber"].as_str().unwrap();

    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/hedge/documentation/{}", doc_number))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}

// ============================================================================
// Dashboard Test
// ============================================================================

#[tokio::test]
async fn test_hedge_dashboard() {
    let (_state, app) = setup_test().await;
    create_derivative(&app, "forward", "1000000.00").await;
    create_hedge(&app, "cash_flow", "500000.00", None).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/hedge/dashboard")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();

    assert!(body.get("totalActiveDerivatives").is_some());
    assert!(body.get("totalNotionalAmount").is_some());
    assert!(body.get("totalActiveHedges").is_some());
    assert!(body.get("totalHedgedAmount").is_some());
    assert!(body.get("totalEffectiveHedges").is_some());
    assert!(body.get("totalIneffectiveHedges").is_some());
    assert!(body.get("totalPendingDocumentation").is_some());
    assert!(body.get("byInstrumentType").is_some());
    assert!(body.get("byHedgeType").is_some());
}

// ============================================================================
// Validation Edge Cases
// ============================================================================

#[tokio::test]
async fn test_create_derivative_invalid_type_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "instrument_type": "warrant",
        "underlying_type": "fx",
        "notional_amount": "1000000",
        "currency_code": "USD",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/hedge/derivatives")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_derivative_invalid_underlying_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "instrument_type": "forward",
        "underlying_type": "weather",
        "notional_amount": "1000000",
        "currency_code": "USD",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/hedge/derivatives")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_derivative_zero_notional_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "instrument_type": "forward",
        "underlying_type": "fx",
        "notional_amount": "0",
        "currency_code": "USD",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/hedge/derivatives")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_hedge_invalid_type_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "hedge_type": "speculative",
        "hedged_risk": "fx",
        "hedged_amount": "1000000",
        "effectiveness_method": "dollar_offset",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/hedge/relationships")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_hedge_zero_amount_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "hedge_type": "cash_flow",
        "hedged_risk": "fx",
        "hedged_amount": "0",
        "effectiveness_method": "dollar_offset",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/hedge/relationships")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_activate_non_draft_derivative_fails() {
    let (_state, app) = setup_test().await;
    let deriv = create_derivative(&app, "forward", "1000000.00").await;
    let deriv_id: Uuid = deriv["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Activate first
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/hedge/derivatives/{}/activate", deriv_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Try to activate again
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/hedge/derivatives/{}/activate", deriv_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_delete_active_derivative_fails() {
    let (_state, app) = setup_test().await;
    let deriv = create_derivative(&app, "forward", "1000000.00").await;
    let instrument_number = deriv["instrumentNumber"].as_str().unwrap();
    let deriv_id: Uuid = deriv["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Activate first
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/hedge/derivatives/{}/activate", deriv_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Try to delete active derivative
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/hedge/derivatives/{}", instrument_number))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_valuation_on_draft_fails() {
    let (_state, app) = setup_test().await;
    let deriv = create_derivative(&app, "forward", "1000000.00").await;
    let deriv_id: Uuid = deriv["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/hedge/derivatives/{}/valuation", deriv_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "fair_value": "5000.00",
            "unrealized_gain_loss": "5000.00"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_terminate_draft_hedge_fails() {
    let (_state, app) = setup_test().await;
    let hedge = create_hedge(&app, "cash_flow", "500000.00", None).await;
    let hedge_id_uuid: Uuid = hedge["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/hedge/relationships/{}/terminate", hedge_id_uuid))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_effectiveness_test_on_draft_hedge_fails() {
    let (_state, app) = setup_test().await;
    let hedge = create_hedge(&app, "cash_flow", "500000.00", None).await;
    let hedge_id_uuid: Uuid = hedge["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/hedge/effectiveness-tests")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "hedge_relationship_id": hedge_id_uuid.to_string(),
            "test_type": "ongoing",
            "test_date": "2024-03-15",
            "derivative_fair_value_change": "10000.00",
            "hedged_item_fair_value_change": "10500.00"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}
