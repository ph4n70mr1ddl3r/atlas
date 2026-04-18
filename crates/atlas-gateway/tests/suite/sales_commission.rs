//! Sales Commission Management E2E Tests
//!
//! Tests for Oracle Fusion Cloud ERP Incentive Compensation:
//! - Sales representative CRUD
//! - Commission plan CRUD and lifecycle (activate/deactivate)
//! - Commission rate tiers
//! - Plan assignments
//! - Sales quotas
//! - Commission transaction crediting and calculation
//! - Tiered commission calculation
//! - Payout processing and lifecycle
//! - Dashboard summary
//! - Validation edge cases

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;
use uuid::Uuid;

async fn setup_commission_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    // Run migration for commission tables
    sqlx::query(include_str!("../../../../migrations/033_sales_commission.sql"))
        .execute(&state.db_pool)
        .await
        .ok(); // Ignore errors if tables already exist
    let app = build_router(state.clone());
    (state, app)
}

async fn create_test_rep(
    app: &axum::Router,
    code: &str,
    first_name: &str,
    last_name: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "rep_code": code,
        "first_name": first_name,
        "last_name": last_name,
        "email": format!("{}@example.com", code.to_lowercase()),
        "territory_code": "WEST",
        "territory_name": "Western Region",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/commission/reps")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Failed to create rep");
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_plan(
    app: &axum::Router,
    code: &str,
    name: &str,
    plan_type: &str,
    calculation_method: &str,
    default_rate: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "code": code,
        "name": name,
        "plan_type": plan_type,
        "basis": "revenue",
        "calculation_method": calculation_method,
        "default_rate": default_rate,
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/commission/plans")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Failed to create plan");
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Sales Representative CRUD Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_rep_crud() {
    let (_state, app) = setup_commission_test().await;

    // Create
    let rep = create_test_rep(&app, "REP-001", "Alice", "Smith").await;
    assert_eq!(rep["rep_code"], "REP-001");
    assert_eq!(rep["first_name"], "Alice");
    assert_eq!(rep["last_name"], "Smith");
    assert!(rep["is_active"].as_bool().unwrap());

    // Get
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/commission/reps/REP-001")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let got: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(got["rep_code"], "REP-001");

    // List
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/commission/reps")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let list: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(list["data"].as_array().unwrap().len() >= 1);

    // Delete
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/commission/reps/REP-001")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
#[ignore]
async fn test_rep_validation() {
    let (_state, app) = setup_commission_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Empty rep code
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/commission/reps")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "rep_code": "",
            "first_name": "Test",
            "last_name": "User",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);

    // Missing names
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/commission/reps")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "rep_code": "BAD",
            "first_name": "",
            "last_name": "",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Commission Plan CRUD & Lifecycle Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_plan_crud_and_lifecycle() {
    let (_state, app) = setup_commission_test().await;

    // Create
    let plan = create_test_plan(&app, "STD-COMM", "Standard Commission", "revenue", "percentage", "5").await;
    assert_eq!(plan["code"], "STD-COMM");
    assert_eq!(plan["name"], "Standard Commission");
    assert_eq!(plan["plan_type"], "revenue");
    assert_eq!(plan["status"], "draft");

    let plan_id = plan["id"].as_str().unwrap().to_string();

    // Activate
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/commission/plans/{}/activate", plan_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let activated: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(activated["status"], "active");

    // Deactivate
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/commission/plans/{}/deactivate", plan_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let deactivated: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(deactivated["status"], "inactive");

    // Get
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/commission/plans/STD-COMM")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    // Delete
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/commission/plans/STD-COMM")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
#[ignore]
async fn test_plan_validation() {
    let (_state, app) = setup_commission_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Invalid plan_type
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/commission/plans")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "BAD",
            "name": "Bad Plan",
            "plan_type": "teleportation",
            "calculation_method": "percentage",
            "default_rate": "5",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);

    // Negative rate
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/commission/plans")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "NEG",
            "name": "Negative",
            "plan_type": "revenue",
            "calculation_method": "percentage",
            "default_rate": "-5",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Rate Tier Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_rate_tiers() {
    let (_state, app) = setup_commission_test().await;

    let plan = create_test_plan(&app, "TIERED-PLAN", "Tiered Plan", "revenue", "tiered", "0").await;
    let plan_id = plan["id"].as_str().unwrap().to_string();

    let (k, v) = auth_header(&admin_claims());

    // Add tiers: 0-10000 at 3%, 10000-50000 at 5%, 50000+ at 8%
    let tier1 = json!({"from_amount": "0", "to_amount": "10000", "rate_percent": "3"});
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/commission/plans/{}/tiers", plan_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&tier1).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);

    let tier2 = json!({"from_amount": "10000", "to_amount": "50000", "rate_percent": "5"});
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/commission/plans/{}/tiers", plan_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&tier2).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);

    let tier3 = json!({"from_amount": "50000", "rate_percent": "8"});
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/commission/plans/{}/tiers", plan_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&tier3).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);

    // List tiers
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/commission/plans/{}/tiers", plan_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let tiers: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(tiers["data"].as_array().unwrap().len(), 3);
}

// ============================================================================
// Plan Assignment Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_plan_assignment() {
    let (_state, app) = setup_commission_test().await;

    let rep = create_test_rep(&app, "REP-ASN", "Bob", "Jones").await;
    let plan = create_test_plan(&app, "ASN-PLAN", "Assignment Plan", "revenue", "percentage", "7").await;

    let (k, v) = auth_header(&admin_claims());

    // Activate plan first
    let plan_id = plan["id"].as_str().unwrap().to_string();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/commission/plans/{}/activate", plan_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Assign
    let rep_id = rep["id"].as_str().unwrap().to_string();
    let payload = json!({
        "rep_id": rep_id,
        "plan_id": plan_id,
        "effective_from": "2025-01-01",
        "effective_to": "2025-12-31",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/commission/assignments")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);

    // List assignments
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/commission/assignments?rep_id={}", rep_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let list: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(list["data"].as_array().unwrap().len() >= 1);
}

// ============================================================================
// Quota Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_quota_management() {
    let (_state, app) = setup_commission_test().await;

    let rep = create_test_rep(&app, "REP-QTA", "Carol", "Williams").await;
    let rep_id = rep["id"].as_str().unwrap().to_string();

    let (k, v) = auth_header(&admin_claims());

    // Create quota
    let payload = json!({
        "rep_id": rep_id,
        "period_name": "Q1 2025",
        "period_start_date": "2025-01-01",
        "period_end_date": "2025-03-31",
        "quota_type": "revenue",
        "target_amount": "50000",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/commission/quotas")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let quota: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(quota["period_name"], "Q1 2025");
    assert!(quota["quota_number"].as_str().unwrap().starts_with("Q-"));
    assert_eq!(quota["status"], "active");

    // List quotas
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/commission/quotas?rep_id={}", rep_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let list: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(list["data"].as_array().unwrap().len() >= 1);
}

// ============================================================================
// Commission Transaction Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_commission_transaction_with_percentage_plan() {
    let (_state, app) = setup_commission_test().await;

    // Setup: create rep, plan, assign
    let rep = create_test_rep(&app, "REP-TX", "Dave", "Brown").await;
    let plan = create_test_plan(&app, "TX-PLAN", "Transaction Plan", "revenue", "percentage", "10").await;
    let rep_id = rep["id"].as_str().unwrap().to_string();
    let plan_id = plan["id"].as_str().unwrap().to_string();

    let (k, v) = auth_header(&admin_claims());

    // Activate and assign
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/commission/plans/{}/activate", plan_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    let assign_payload = json!({
        "rep_id": rep_id,
        "plan_id": plan_id,
        "effective_from": "2025-01-01",
    });
    app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/commission/assignments")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&assign_payload).unwrap())).unwrap()
    ).await.unwrap();

    // Credit transaction: $1000 sale → 10% = $100 commission
    let tx_payload = json!({
        "rep_id": rep_id,
        "source_type": "sales_order",
        "source_number": "SO-001",
        "transaction_date": "2025-02-15",
        "sale_amount": "1000",
        "currency_code": "USD",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/commission/transactions")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&tx_payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let tx: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();

    // Verify commission calculation
    let sale_amount: f64 = tx["sale_amount"].as_str().unwrap().parse().unwrap();
    let commission_rate: f64 = tx["commission_rate"].as_str().unwrap().parse().unwrap();
    let commission_amount: f64 = tx["commission_amount"].as_str().unwrap().parse().unwrap();

    assert!((sale_amount - 1000.0).abs() < 0.01, "Expected sale_amount 1000, got {}", sale_amount);
    assert!((commission_rate - 10.0).abs() < 0.01, "Expected commission_rate 10, got {}", commission_rate);
    assert!((commission_amount - 100.0).abs() < 0.01, "Expected commission_amount 100, got {}", commission_amount);
    assert_eq!(tx["status"], "credited");

    // List transactions
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/commission/transactions?rep_id={}", rep_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}

#[tokio::test]
#[ignore]
async fn test_tiered_commission_calculation() {
    let (_state, app) = setup_commission_test().await;

    let rep = create_test_rep(&app, "REP-TIER", "Eve", "Davis").await;
    let plan = create_test_plan(&app, "TIER-TX", "Tiered TX Plan", "revenue", "tiered", "0").await;
    let rep_id = rep["id"].as_str().unwrap().to_string();
    let plan_id = plan["id"].as_str().unwrap().to_string();

    let (k, v) = auth_header(&admin_claims());

    // Add tiers
    for (from, to, rate) in [("0", Some("10000"), "3"), ("10000", Some("50000"), "5"), ("50000", None, "8")] {
        let tier = json!({"from_amount": from, "rate_percent": rate});
        let mut payload = tier;
        if let Some(t) = to {
            payload["to_amount"] = json!(t);
        }
        app.clone().oneshot(Request::builder().method("POST")
            .uri(&format!("/api/v1/commission/plans/{}/tiers", plan_id))
            .header("Content-Type", "application/json").header(&k, &v)
            .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
        ).await.unwrap();
    }

    // Activate and assign
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/commission/plans/{}/activate", plan_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    let assign_payload = json!({
        "rep_id": rep_id,
        "plan_id": plan_id,
        "effective_from": "2025-01-01",
    });
    app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/commission/assignments")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&assign_payload).unwrap())).unwrap()
    ).await.unwrap();

    // Small sale ($5,000) → 3% tier → $150
    let tx_payload = json!({
        "rep_id": rep_id,
        "source_type": "sales_order",
        "source_number": "SO-SMALL",
        "transaction_date": "2025-02-01",
        "sale_amount": "5000",
        "currency_code": "USD",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/commission/transactions")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&tx_payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let tx_small: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let comm_small: f64 = tx_small["commission_amount"].as_str().unwrap().parse().unwrap();
    assert!((comm_small - 150.0).abs() < 0.01, "Expected 3% of 5000 = 150, got {}", comm_small);

    // Large sale ($75,000) → 8% tier → $6000
    let tx_payload2 = json!({
        "rep_id": rep_id,
        "source_type": "sales_order",
        "source_number": "SO-LARGE",
        "transaction_date": "2025-02-15",
        "sale_amount": "75000",
        "currency_code": "USD",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/commission/transactions")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&tx_payload2).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let tx_large: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let comm_large: f64 = tx_large["commission_amount"].as_str().unwrap().parse().unwrap();
    assert!((comm_large - 6000.0).abs() < 0.01, "Expected 8% of 75000 = 6000, got {}", comm_large);
}

// ============================================================================
// Payout Processing Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_payout_processing_lifecycle() {
    let (_state, app) = setup_commission_test().await;

    // Setup rep + plan + assignment
    let rep1 = create_test_rep(&app, "REP-PAY1", "Frank", "Miller").await;
    let rep2 = create_test_rep(&app, "REP-PAY2", "Grace", "Wilson").await;
    let plan = create_test_plan(&app, "PAY-PLAN", "Payout Plan", "revenue", "percentage", "5").await;

    let (k, v) = auth_header(&admin_claims());
    let plan_id = plan["id"].as_str().unwrap().to_string();
    let rep1_id = rep1["id"].as_str().unwrap().to_string();
    let rep2_id = rep2["id"].as_str().unwrap().to_string();

    // Activate plan
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/commission/plans/{}/activate", plan_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Assign plan to both reps
    for rep_id in [&rep1_id, &rep2_id] {
        let assign_payload = json!({
            "rep_id": rep_id,
            "plan_id": plan_id,
            "effective_from": "2025-01-01",
        });
        app.clone().oneshot(Request::builder().method("POST")
            .uri("/api/v1/commission/assignments")
            .header("Content-Type", "application/json").header(&k, &v)
            .body(Body::from(serde_json::to_string(&assign_payload).unwrap())).unwrap()
        ).await.unwrap();
    }

    // Credit transactions for both reps
    for (rep_id, so_num, amount) in [(&rep1_id, "SO-P1", "2000"), (&rep2_id, "SO-P2", "4000")] {
        let tx_payload = json!({
            "rep_id": rep_id,
            "source_type": "sales_order",
            "source_number": so_num,
            "transaction_date": "2025-02-15",
            "sale_amount": amount,
            "currency_code": "USD",
        });
        app.clone().oneshot(Request::builder().method("POST")
            .uri("/api/v1/commission/transactions")
            .header("Content-Type", "application/json").header(&k, &v)
            .body(Body::from(serde_json::to_string(&tx_payload).unwrap())).unwrap()
        ).await.unwrap();
    }

    // Process payout
    let payout_payload = json!({
        "period_name": "February 2025",
        "period_start_date": "2025-02-01",
        "period_end_date": "2025-02-28",
        "currency_code": "USD",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/commission/payouts")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payout_payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let payout: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();

    assert_eq!(payout["status"], "draft");
    assert_eq!(payout["rep_count"], 2);
    assert_eq!(payout["transaction_count"], 2);

    // Total: 5% of $2000 + 5% of $4000 = $100 + $200 = $300
    let total: f64 = payout["total_payout_amount"].as_str().unwrap().parse().unwrap();
    assert!((total - 300.0).abs() < 0.01, "Expected total payout 300, got {}", total);

    let payout_id = payout["id"].as_str().unwrap().to_string();

    // Get payout lines
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/commission/payouts/{}/lines", payout_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let lines: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(lines["data"].as_array().unwrap().len(), 2);

    // Approve payout
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/commission/payouts/{}/approve", payout_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let approved: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(approved["status"], "approved");
    assert!(approved["approved_at"].is_string());
}

// ============================================================================
// Dashboard Test
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_commission_dashboard() {
    let (_state, app) = setup_commission_test().await;

    // Create some data
    create_test_rep(&app, "REP-DASH1", "Henry", "Taylor").await;
    create_test_rep(&app, "REP-DASH2", "Iris", "Anderson").await;
    create_test_plan(&app, "DASH-PLAN", "Dashboard Plan", "revenue", "percentage", "6").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/commission/dashboard")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let summary: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();

    assert!(summary["total_reps"].as_i64().unwrap() >= 2);
    assert!(summary["active_reps"].as_i64().unwrap() >= 2);
    assert!(summary["total_plans"].as_i64().unwrap() >= 1);
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_cannot_credit_without_plan_assignment() {
    let (_state, app) = setup_commission_test().await;

    let rep = create_test_rep(&app, "REP-NOPLAN", "Jack", "Thomas").await;
    let rep_id = rep["id"].as_str().unwrap().to_string();

    let (k, v) = auth_header(&admin_claims());

    // Try to credit without assignment and without explicit plan_id
    let tx_payload = json!({
        "rep_id": rep_id,
        "source_type": "sales_order",
        "transaction_date": "2025-03-01",
        "sale_amount": "500",
        "currency_code": "USD",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/commission/transactions")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&tx_payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore]
async fn test_cannot_process_payout_with_no_transactions() {
    let (_state, app) = setup_commission_test().await;

    let (k, v) = auth_header(&admin_claims());

    let payout_payload = json!({
        "period_name": "Empty Period",
        "period_start_date": "2025-06-01",
        "period_end_date": "2025-06-30",
        "currency_code": "USD",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/commission/payouts")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payout_payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore]
async fn test_quota_validation() {
    let (_state, app) = setup_commission_test().await;
    let (k, v) = auth_header(&admin_claims());

    let rep = create_test_rep(&app, "REP-QVAL", "Karen", "White").await;
    let rep_id = rep["id"].as_str().unwrap().to_string();

    // Negative target
    let payload = json!({
        "rep_id": rep_id,
        "period_name": "Q2 2025",
        "period_start_date": "2025-04-01",
        "period_end_date": "2025-06-30",
        "quota_type": "revenue",
        "target_amount": "-1000",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/commission/quotas")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);

    // Invalid dates (start >= end)
    let payload = json!({
        "rep_id": rep_id,
        "period_name": "Bad Dates",
        "period_start_date": "2025-06-30",
        "period_end_date": "2025-06-01",
        "quota_type": "revenue",
        "target_amount": "1000",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/commission/quotas")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}
