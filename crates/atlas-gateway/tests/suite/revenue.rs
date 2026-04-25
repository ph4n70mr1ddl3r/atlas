//! Revenue Recognition E2E Tests (ASC 606 / IFRS 15)
//!
//! Tests for Oracle Fusion Revenue Management:
//! - Revenue Policies CRUD
//! - Revenue Contracts CRUD + lifecycle (draft -> active -> cancelled)
//! - Performance Obligations (add obligations to contracts)
//! - Transaction Price Allocation (ASC 606 Step 4 - SSP method)
//! - Straight-line recognition schedules
//! - Point-in-time recognition schedules
//! - Revenue recognition execution (recognize / reverse)
//! - Contract modifications
//! - Full end-to-end ASC 606 five-step flow

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_revenue_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    setup_test_db(&state.db_pool).await;
    // Clean revenue data for isolation
    sqlx::query("DELETE FROM _atlas.revenue_schedule_lines").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM _atlas.revenue_modifications").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM _atlas.performance_obligations").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM _atlas.revenue_contracts").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM _atlas.revenue_policies").execute(&state.db_pool).await.ok();
    let app = build_router(state.clone());
    (state, app)
}

async fn create_test_policy(app: &axum::Router, code: &str) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/revenue/policies")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": code,
            "name": format!("Policy {}", code),
            "description": "Test revenue policy",
            "recognition_method": "over_time",
            "over_time_method": "straight_line",
            "allocation_basis": "standalone_selling_price",
            "revenue_account_code": "4000",
            "deferred_revenue_account_code": "2500",
            "contra_revenue_account_code": "4100"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_contract(app: &axum::Router, price: &str) -> serde_json::Value {
    let customer_id = uuid::Uuid::new_v4();
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/revenue/contracts")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "customer_id": customer_id,
            "customer_name": "Acme Corp",
            "customer_number": "CUST-001",
            "total_transaction_price": price,
            "currency_code": "USD",
            "contract_date": "2026-01-15",
            "start_date": "2026-01-01",
            "end_date": "2027-01-01",
            "notes": "Annual software license"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Revenue Policy Tests
// ============================================================================

#[tokio::test]
async fn test_create_revenue_policy() {
    let (_state, app) = setup_revenue_test().await;
    let policy = create_test_policy(&app, "POL-001").await;
    assert_eq!(policy["code"], "POL-001");
    assert_eq!(policy["name"], "Policy POL-001");
    assert_eq!(policy["recognition_method"], "over_time");
    assert_eq!(policy["allocation_basis"], "standalone_selling_price");
    assert!(policy["id"].is_string());
}

#[tokio::test]
async fn test_list_revenue_policies() {
    let (_state, app) = setup_revenue_test().await;
    create_test_policy(&app, "POL-LA").await;
    create_test_policy(&app, "POL-LB").await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/revenue/policies").header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_get_revenue_policy() {
    let (_state, app) = setup_revenue_test().await;
    create_test_policy(&app, "POL-GET").await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/revenue/policies/POL-GET").header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let fetched: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(fetched["code"], "POL-GET");
}

#[tokio::test]
async fn test_delete_revenue_policy() {
    let (_state, app) = setup_revenue_test().await;
    create_test_policy(&app, "POL-DEL").await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/revenue/policies/POL-DEL").header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
    // Verify it's gone (deactivated)
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/revenue/policies/POL-DEL").header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_create_policy_invalid_method() {
    let (_state, app) = setup_revenue_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/revenue/policies")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "POL-BAD",
            "name": "Bad Policy",
            "recognition_method": "invalid_method"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Revenue Contract Tests
// ============================================================================

#[tokio::test]
async fn test_create_revenue_contract() {
    let (_state, app) = setup_revenue_test().await;
    let contract = create_test_contract(&app, "120000").await;
    assert_eq!(contract["status"], "draft");
    assert_eq!(contract["currency_code"], "USD");
    assert_eq!(contract["customer_name"], "Acme Corp");
    assert!(contract["contract_number"].as_str().unwrap().starts_with("RC-"));
    assert!(contract["id"].is_string());
}

#[tokio::test]
async fn test_list_revenue_contracts() {
    let (_state, app) = setup_revenue_test().await;
    create_test_contract(&app, "50000").await;
    create_test_contract(&app, "75000").await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/revenue/contracts?status=draft").header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_get_revenue_contract() {
    let (_state, app) = setup_revenue_test().await;
    let contract = create_test_contract(&app, "100000").await;
    let id = contract["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/revenue/contracts/{}", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let fetched: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(fetched["status"], "draft");
}

#[tokio::test]
async fn test_activate_contract() {
    let (_state, app) = setup_revenue_test().await;
    let contract = create_test_contract(&app, "100000").await;
    let id = contract["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/revenue/contracts/{}/activate", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let activated: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(activated["status"], "active");
}

#[tokio::test]
async fn test_cancel_contract() {
    let (_state, app) = setup_revenue_test().await;
    let contract = create_test_contract(&app, "100000").await;
    let id = contract["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/revenue/contracts/{}/cancel", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "reason": "Customer requested cancellation"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let cancelled: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(cancelled["status"], "cancelled");
}

#[tokio::test]
async fn test_create_contract_negative_price_rejected() {
    let (_state, app) = setup_revenue_test().await;
    let customer_id = uuid::Uuid::new_v4();
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/revenue/contracts")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "customer_id": customer_id,
            "total_transaction_price": "-100"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Performance Obligation Tests
// ============================================================================

#[tokio::test]
async fn test_create_obligation() {
    let (_state, app) = setup_revenue_test().await;
    let contract = create_test_contract(&app, "100000").await;
    let contract_id = contract["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/revenue/contracts/{}/obligations", contract_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "description": "Software License",
            "product_name": "Enterprise Suite",
            "standalone_selling_price": "60000",
            "satisfaction_method": "over_time",
            "recognition_start_date": "2026-01-01",
            "recognition_end_date": "2026-12-31"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let obligation: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(obligation["description"], "Software License");
    assert_eq!(obligation["satisfaction_method"], "over_time");
    assert_eq!(obligation["status"], "pending");
    assert!(obligation["id"].is_string());
}

#[tokio::test]
async fn test_list_obligations() {
    let (_state, app) = setup_revenue_test().await;
    let contract = create_test_contract(&app, "100000").await;
    let contract_id = contract["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());
    // Add two obligations
    for (name, price) in [("License", "60000"), ("Support", "40000")] {
        app.clone().oneshot(Request::builder().method("POST")
            .uri(&format!("/api/v1/revenue/contracts/{}/obligations", contract_id))
            .header("Content-Type", "application/json").header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({
                "description": name,
                "standalone_selling_price": price,
                "satisfaction_method": "over_time"
            })).unwrap())).unwrap()
        ).await.unwrap();
    }
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/revenue/contracts/{}/obligations", contract_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(resp["data"].as_array().unwrap().len(), 2);
}

// ============================================================================
// Transaction Price Allocation Tests (ASC 606 Step 4)
// ============================================================================

#[tokio::test]
async fn test_allocate_transaction_price() {
    let (_state, app) = setup_revenue_test().await;
    let contract = create_test_contract(&app, "100000").await;
    let contract_id = contract["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    // Add two obligations: 60k and 40k SSP = 100k total
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/revenue/contracts/{}/obligations", contract_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "description": "License",
            "standalone_selling_price": "60000",
            "satisfaction_method": "over_time"
        })).unwrap())).unwrap()
    ).await.unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/revenue/contracts/{}/obligations", contract_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "description": "Support",
            "standalone_selling_price": "40000",
            "satisfaction_method": "over_time"
        })).unwrap())).unwrap()
    ).await.unwrap();

    // Allocate
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/revenue/contracts/{}/allocate", contract_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap_or_else(|_| {
        panic!("Allocate response not JSON: {}", String::from_utf8_lossy(&b));
    });
    let obligations = resp["data"].as_array().expect(&format!("Expected data array: {:?}", resp));
    assert_eq!(obligations.len(), 2, "allocate response: {:?}", resp);

    // Verify proportional allocation: 60k/100k * 100k = 60000, 40k/100k * 100k = 40000
    let lic = obligations.iter().find(|o| o["description"] == "License").unwrap();
    let sup = obligations.iter().find(|o| o["description"] == "Support").unwrap();
    // Check allocated prices are roughly correct (may have rounding)
    let lic_alloc: f64 = lic["allocated_transaction_price"].as_str().unwrap().parse().unwrap();
    let sup_alloc: f64 = sup["allocated_transaction_price"].as_str().unwrap().parse().unwrap();
    assert!((lic_alloc - 60000.0).abs() < 1.0);
    assert!((sup_alloc - 40000.0).abs() < 1.0);
}

// ============================================================================
// Revenue Schedule Tests
// ============================================================================

#[tokio::test]
async fn test_straight_line_schedule() {
    let (_state, app) = setup_revenue_test().await;
    let contract = create_test_contract(&app, "12000").await;
    let contract_id = contract["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    // Add obligation
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/revenue/contracts/{}/obligations", contract_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "description": "Annual License",
            "standalone_selling_price": "12000",
            "satisfaction_method": "over_time"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let obligation: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let obligation_id = obligation["id"].as_str().unwrap();

    // Allocate first
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/revenue/contracts/{}/allocate", contract_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Generate 12-month schedule
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/revenue/obligations/{}/schedule/straight-line", obligation_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "start_date": "2026-01-01",
            "end_date": "2027-01-01"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let lines = resp["data"].as_array().unwrap();
    assert_eq!(lines.len(), 12);
    // Each line should be planned
    for line in lines {
        assert_eq!(line["status"], "planned");
    }
    // Total should approximately equal 12000
    let total: f64 = lines.iter()
        .map(|l| l["amount"].as_str().unwrap().parse::<f64>().unwrap())
        .sum();
    assert!((total - 12000.0).abs() < 1.0);
}

#[tokio::test]
async fn test_point_in_time_schedule() {
    let (_state, app) = setup_revenue_test().await;
    let contract = create_test_contract(&app, "50000").await;
    let contract_id = contract["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    // Add obligation
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/revenue/contracts/{}/obligations", contract_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "description": "Hardware Delivery",
            "standalone_selling_price": "50000",
            "satisfaction_method": "point_in_time"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let obligation: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let obligation_id = obligation["id"].as_str().unwrap();

    // Allocate
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/revenue/contracts/{}/allocate", contract_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Schedule point-in-time
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/revenue/obligations/{}/schedule/point-in-time", obligation_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "recognition_date": "2026-03-15"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let line: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(line["status"], "planned");
    assert_eq!(line["recognition_method"], "point_in_time");
}

// ============================================================================
// Revenue Recognition Execution Tests
// ============================================================================

#[tokio::test]
async fn test_recognize_revenue() {
    let (_state, app) = setup_revenue_test().await;
    let contract = create_test_contract(&app, "50000").await;
    let contract_id = contract["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    // Add obligation, allocate, and schedule point-in-time
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/revenue/contracts/{}/obligations", contract_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "description": "Consulting",
            "standalone_selling_price": "50000",
            "satisfaction_method": "point_in_time"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let obligation: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let obligation_id = obligation["id"].as_str().unwrap();

    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/revenue/contracts/{}/allocate", contract_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/revenue/obligations/{}/schedule/point-in-time", obligation_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "recognition_date": "2026-06-01"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let line: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let line_id = line["id"].as_str().unwrap();

    // Recognize
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/revenue/schedule-lines/{}/recognize", line_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let recognized: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(recognized["status"], "recognized");
    assert!(recognized["recognized_at"].is_string());
}

#[tokio::test]
async fn test_reverse_recognition() {
    let (_state, app) = setup_revenue_test().await;
    let contract = create_test_contract(&app, "30000").await;
    let contract_id = contract["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    // Full flow: obligation -> allocate -> schedule -> recognize
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/revenue/contracts/{}/obligations", contract_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "description": "Service",
            "standalone_selling_price": "30000",
            "satisfaction_method": "point_in_time"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let obligation: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let obligation_id = obligation["id"].as_str().unwrap();

    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/revenue/contracts/{}/allocate", contract_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/revenue/obligations/{}/schedule/point-in-time", obligation_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "recognition_date": "2026-04-01"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let line: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let line_id = line["id"].as_str().unwrap();

    // Recognize first
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/revenue/schedule-lines/{}/recognize", line_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Now reverse
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/revenue/schedule-lines/{}/reverse", line_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "reason": "Correction: wrong period"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let reversed: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(reversed["status"], "reversed");
}

#[tokio::test]
async fn test_recognize_non_planned_rejected() {
    let (_state, app) = setup_revenue_test().await;
    let contract = create_test_contract(&app, "25000").await;
    let contract_id = contract["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/revenue/contracts/{}/obligations", contract_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "description": "Test",
            "standalone_selling_price": "25000",
            "satisfaction_method": "point_in_time"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let obligation: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let obligation_id = obligation["id"].as_str().unwrap();

    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/revenue/contracts/{}/allocate", contract_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/revenue/obligations/{}/schedule/point-in-time", obligation_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "recognition_date": "2026-05-01"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let line: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let line_id = line["id"].as_str().unwrap();

    // Recognize once
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/revenue/schedule-lines/{}/recognize", line_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Try to recognize again (already recognized)
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/revenue/schedule-lines/{}/recognize", line_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_list_schedule_lines() {
    let (_state, app) = setup_revenue_test().await;
    let contract = create_test_contract(&app, "12000").await;
    let contract_id = contract["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/revenue/contracts/{}/obligations", contract_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "description": "License",
            "standalone_selling_price": "12000",
            "satisfaction_method": "over_time"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let obligation: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let obligation_id = obligation["id"].as_str().unwrap();

    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/revenue/contracts/{}/allocate", contract_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/revenue/obligations/{}/schedule/straight-line", obligation_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "start_date": "2026-01-01",
            "end_date": "2026-07-01"
        })).unwrap())).unwrap()
    ).await.unwrap();

    // List via obligation
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/revenue/obligations/{}/schedule-lines", obligation_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(resp["data"].as_array().unwrap().len(), 6);

    // List via contract
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/revenue/contracts/{}/schedule-lines", contract_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(resp["data"].as_array().unwrap().len(), 6);
}

// ============================================================================
// Contract Modification Tests
// ============================================================================

#[tokio::test]
async fn test_create_modification() {
    let (_state, app) = setup_revenue_test().await;
    let contract = create_test_contract(&app, "100000").await;
    let contract_id = contract["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    // Activate first
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/revenue/contracts/{}/activate", contract_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Create modification
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/revenue/contracts/{}/modifications", contract_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "modification_type": "price_change",
            "description": "Scope expansion - additional modules",
            "previous_transaction_price": "100000",
            "new_transaction_price": "150000",
            "effective_date": "2026-04-01"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let modification: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(modification["modification_type"], "price_change");
    assert_eq!(modification["status"], "active");
    assert!(modification["id"].is_string());
}

#[tokio::test]
async fn test_list_modifications() {
    let (_state, app) = setup_revenue_test().await;
    let contract = create_test_contract(&app, "100000").await;
    let contract_id = contract["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/revenue/contracts/{}/activate", contract_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/revenue/contracts/{}/modifications", contract_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "modification_type": "term_extension",
            "previous_transaction_price": "100000",
            "new_transaction_price": "120000",
            "effective_date": "2026-06-01"
        })).unwrap())).unwrap()
    ).await.unwrap();

    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/revenue/contracts/{}/modifications", contract_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 1);
}

// ============================================================================
// End-to-End Five-Step ASC 606 Flow
// ============================================================================

#[tokio::test]
async fn test_full_asc606_five_step_flow() {
    let (_state, app) = setup_revenue_test().await;
    let (k, v) = auth_header(&admin_claims());
    let customer_id = uuid::Uuid::new_v4();

    // Step 0: Create policy
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/revenue/policies")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "E2E-POL",
            "name": "E2E Policy",
            "recognition_method": "over_time",
            "over_time_method": "straight_line",
            "allocation_basis": "standalone_selling_price"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);

    // Step 1: Identify the contract
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/revenue/contracts")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "customer_id": customer_id,
            "customer_name": "Mega Corp",
            "total_transaction_price": "240000",
            "currency_code": "USD",
            "start_date": "2026-01-01",
            "end_date": "2027-01-01"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let contract: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let contract_id = contract["id"].as_str().unwrap();
    assert_eq!(contract["status"], "draft");

    // Activate
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/revenue/contracts/{}/activate", contract_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let activated: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(activated["status"], "active");

    // Step 2: Identify performance obligations (two obligations)
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/revenue/contracts/{}/obligations", contract_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "description": "Platform License",
            "standalone_selling_price": "180000",
            "satisfaction_method": "over_time"
        })).unwrap())).unwrap()
    ).await.unwrap();

    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/revenue/contracts/{}/obligations", contract_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "description": "Implementation Services",
            "standalone_selling_price": "60000",
            "satisfaction_method": "over_time"
        })).unwrap())).unwrap()
    ).await.unwrap();

    // Step 3 & 4: Determine and allocate transaction price
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/revenue/contracts/{}/allocate", contract_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let alloc_resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let obligations = alloc_resp["data"].as_array().unwrap();
    assert_eq!(obligations.len(), 2);

    // Verify: 180k/240k * 240k = 180000, 60k/240k * 240k = 60000
    let platform = obligations.iter().find(|o| o["description"] == "Platform License").unwrap();
    let impl_svc = obligations.iter().find(|o| o["description"] == "Implementation Services").unwrap();
    let platform_alloc: f64 = platform["allocated_transaction_price"].as_str().unwrap().parse().unwrap();
    let impl_alloc: f64 = impl_svc["allocated_transaction_price"].as_str().unwrap().parse().unwrap();
    assert!((platform_alloc - 180000.0).abs() < 1.0);
    assert!((impl_alloc - 60000.0).abs() < 1.0);

    let platform_id = platform["id"].as_str().unwrap();
    let impl_id = impl_svc["id"].as_str().unwrap();

    // Step 5: Generate recognition schedules
    // Platform: 12 months straight-line (180k / 12 = 15k/month)
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/revenue/obligations/{}/schedule/straight-line", platform_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "start_date": "2026-01-01",
            "end_date": "2027-01-01"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let platform_schedule: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(platform_schedule["data"].as_array().unwrap().len(), 12);

    // Implementation: 6 months straight-line (60k / 6 = 10k/month)
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/revenue/obligations/{}/schedule/straight-line", impl_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "start_date": "2026-01-01",
            "end_date": "2026-07-01"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let impl_schedule: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(impl_schedule["data"].as_array().unwrap().len(), 6);

    // Recognize first month of platform
    let first_line = &platform_schedule["data"].as_array().unwrap()[0];
    let first_line_id = first_line["id"].as_str().unwrap();
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/revenue/schedule-lines/{}/recognize", first_line_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let recognized: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(recognized["status"], "recognized");

    // Verify the obligation status updated to partially_satisfied
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/revenue/obligations/{}", platform_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let updated_obligation: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(updated_obligation["status"], "partially_satisfied");

    // Verify the contract updated
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/revenue/contracts/{}", contract_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let updated_contract: serde_json::Value = serde_json::from_slice(&b).unwrap();
    // Should have some recognized revenue now
    assert!(updated_contract["total_recognized_revenue"].is_string());
}
