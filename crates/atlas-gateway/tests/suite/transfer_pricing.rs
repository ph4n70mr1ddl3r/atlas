//! Transfer Pricing E2E Tests
//!
//! Tests for Oracle Fusion Cloud Financials > Transfer Pricing:
//! - Policy CRUD and lifecycle
//! - Transaction creation and workflow
//! - Benchmark study workflow
//! - Comparable management
//! - Documentation package lifecycle
//! - Dashboard summary

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use uuid::Uuid;
use super::common::helpers::*;

async fn setup_tp_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    let app = build_router(state.clone());
    (state, app)
}

// ============================================================================
// Helper functions
// ============================================================================

async fn create_test_policy(app: &axum::Router, code: &str) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/transfer-pricing/policies")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "policy_code": code,
            "name": format!("Transfer Policy {}", code),
            "description": "Intercompany transfer pricing policy",
            "pricing_method": "cost_plus",
            "from_entity_name": "US Corporation",
            "to_entity_name": "DE GmbH",
            "product_category": "Electronics",
            "geography": "US-DE",
            "arm_length_range_low": "10.00",
            "arm_length_range_mid": "15.00",
            "arm_length_range_high": "20.00",
            "margin_pct": "12.5",
            "cost_base": "full_cost"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_transaction(app: &axum::Router, policy_id: Option<&str>) -> serde_json::Value {
    let mut body = json!({
        "from_entity_name": "US Corporation",
        "to_entity_name": "DE GmbH",
        "item_code": "ITEM-001",
        "item_description": "Electronic Widget",
        "quantity": "100",
        "unit_cost": "10.00",
        "transfer_price": "15.00",
        "currency_code": "USD",
        "transaction_date": "2024-06-15",
        "source_type": "intercompany"
    });
    if let Some(pid) = policy_id {
        body["policy_id"] = json!(pid);
    }

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/transfer-pricing/transactions")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&body).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_benchmark(app: &axum::Router, title: &str) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/transfer-pricing/benchmarks")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "title": title,
            "description": "Arm's-length analysis for intercompany transactions",
            "analysis_method": "cost_plus",
            "fiscal_year": 2024,
            "from_entity_name": "US Corporation",
            "to_entity_name": "DE GmbH",
            "product_category": "Electronics",
            "tested_party": "DE GmbH"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_documentation(app: &axum::Router, title: &str) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/transfer-pricing/documentation")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "title": title,
            "doc_type": "local_file",
            "fiscal_year": 2024,
            "country": "DE",
            "reporting_entity_name": "DE GmbH",
            "description": "Annual TP documentation for German tax authority",
            "responsible_party": "Tax Department"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Policy CRUD Tests
// ============================================================================

#[tokio::test]
async fn test_create_policy() {
    let (_state, app) = setup_tp_test().await;
    let policy = create_test_policy(&app, "TP-POL-001").await;

    assert_eq!(policy["policy_code"], "TP-POL-001");
    assert_eq!(policy["name"], "Transfer Policy TP-POL-001");
    assert_eq!(policy["pricing_method"], "cost_plus");
    assert_eq!(policy["status"], "draft");
    assert_eq!(policy["cost_base"], "full_cost");
    assert!(policy["id"].is_string());
}

#[tokio::test]
async fn test_create_policy_validation_empty_code() {
    let (_state, app) = setup_tp_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/transfer-pricing/policies")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "policy_code": "",
            "name": "Test",
            "pricing_method": "cost_plus"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_policy_validation_invalid_method() {
    let (_state, app) = setup_tp_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/transfer-pricing/policies")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "policy_code": "POL-BAD",
            "name": "Test",
            "pricing_method": "invalid_method"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_policy_duplicate_code() {
    let (_state, app) = setup_tp_test().await;
    create_test_policy(&app, "TP-DUP").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/transfer-pricing/policies")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "policy_code": "TP-DUP",
            "name": "Duplicate",
            "pricing_method": "cost_plus"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_get_policy() {
    let (_state, app) = setup_tp_test().await;
    let policy = create_test_policy(&app, "TP-GET").await;
    // Get by code
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/transfer-pricing/policies/TP-GET")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let fetched: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(fetched["policy_code"], "TP-GET");
    assert_eq!(fetched["pricing_method"], "cost_plus");
}

#[tokio::test]
async fn test_list_policies() {
    let (_state, app) = setup_tp_test().await;
    create_test_policy(&app, "TP-LIST-1").await;
    create_test_policy(&app, "TP-LIST-2").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/transfer-pricing/policies")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(result["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_activate_policy() {
    let (_state, app) = setup_tp_test().await;
    let policy = create_test_policy(&app, "TP-ACT").await;
    let id = policy["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/transfer-pricing/policies/{}/activate", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(updated["status"], "active");
}

#[tokio::test]
async fn test_deactivate_policy() {
    let (_state, app) = setup_tp_test().await;
    let policy = create_test_policy(&app, "TP-DEACT").await;
    let id = policy["id"].as_str().unwrap();

    // Activate first
    let (k, v) = auth_header(&admin_claims());
    let _r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/transfer-pricing/policies/{}/activate", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Now deactivate
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/transfer-pricing/policies/{}/deactivate", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(updated["status"], "inactive");
}

#[tokio::test]
async fn test_delete_policy_draft() {
    let (_state, app) = setup_tp_test().await;
    let policy = create_test_policy(&app, "TP-DEL").await;
    assert_eq!(policy["status"], "draft");

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/transfer-pricing/policies/TP-DEL")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_delete_active_policy_rejected() {
    let (_state, app) = setup_tp_test().await;
    let policy = create_test_policy(&app, "TP-DELACT").await;
    let id = policy["id"].as_str().unwrap();

    // Activate first
    let (k, v) = auth_header(&admin_claims());
    let _r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/transfer-pricing/policies/{}/activate", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Try deleting active policy
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/transfer-pricing/policies/TP-DELACT")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Transaction Tests
// ============================================================================

#[tokio::test]
async fn test_create_transaction() {
    let (_state, app) = setup_tp_test().await;
    let txn = create_test_transaction(&app, None).await;

    assert!(txn["transaction_number"].as_str().unwrap().starts_with("TPT-"));
    assert_eq!(txn["status"], "draft");
    assert_eq!(txn["quantity"], "100.0000");
    assert_eq!(txn["currency_code"], "USD");
    assert_eq!(txn["source_type"], "intercompany");
    // Total amount should be 100 * 15 = 1500
    assert!(txn["total_amount"].as_str().unwrap().contains("1500"));
}

#[tokio::test]
async fn test_create_transaction_with_policy() {
    let (_state, app) = setup_tp_test().await;
    let policy = create_test_policy(&app, "TP-TXN-POL").await;
    let policy_id = policy["id"].as_str().unwrap();

    let txn = create_test_transaction(&app, Some(policy_id)).await;
    // With arm's-length range 10-20 and transfer price 15, should be compliant
    assert_eq!(txn["is_arm_length_compliant"], true);
}

#[tokio::test]
async fn test_create_transaction_non_compliant() {
    let (_state, app) = setup_tp_test().await;
    let policy = create_test_policy(&app, "TP-NC").await;
    let policy_id = policy["id"].as_str().unwrap();

    // Transfer price 25 is outside 10-20 range
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/transfer-pricing/transactions")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "policy_id": policy_id,
            "from_entity_name": "US Corp",
            "to_entity_name": "DE GmbH",
            "item_code": "ITEM-001",
            "quantity": "100",
            "unit_cost": "10.00",
            "transfer_price": "25.00",
            "currency_code": "USD",
            "transaction_date": "2024-06-15"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let txn: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(txn["is_arm_length_compliant"], false);
}

#[tokio::test]
async fn test_transaction_workflow() {
    let (_state, app) = setup_tp_test().await;
    let txn = create_test_transaction(&app, None).await;
    let id = txn["id"].as_str().unwrap();
    assert_eq!(txn["status"], "draft");

    // Submit
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/transfer-pricing/transactions/{}/submit", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let submitted: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(submitted["status"], "submitted");

    // Approve
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/transfer-pricing/transactions/{}/approve", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let approved: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(approved["status"], "approved");
}

#[tokio::test]
async fn test_transaction_reject() {
    let (_state, app) = setup_tp_test().await;
    let txn = create_test_transaction(&app, None).await;
    let id = txn["id"].as_str().unwrap();

    // Submit
    let (k, v) = auth_header(&admin_claims());
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/transfer-pricing/transactions/{}/submit", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Reject
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/transfer-pricing/transactions/{}/reject", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let rejected: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(rejected["status"], "rejected");
}

#[tokio::test]
async fn test_cannot_approve_draft_transaction() {
    let (_state, app) = setup_tp_test().await;
    let txn = create_test_transaction(&app, None).await;
    let id = txn["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/transfer-pricing/transactions/{}/approve", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_list_transactions() {
    let (_state, app) = setup_tp_test().await;
    create_test_transaction(&app, None).await;
    create_test_transaction(&app, None).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/transfer-pricing/transactions")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(result["data"].as_array().unwrap().len() >= 2);
}

// ============================================================================
// Benchmark Tests
// ============================================================================

#[tokio::test]
async fn test_create_benchmark() {
    let (_state, app) = setup_tp_test().await;
    let bm = create_test_benchmark(&app, "FY2024 Cost Plus Study").await;

    assert!(bm["study_number"].as_str().unwrap().starts_with("BMS-"));
    assert_eq!(bm["status"], "draft");
    assert_eq!(bm["analysis_method"], "cost_plus");
    assert_eq!(bm["fiscal_year"], 2024);
    assert_eq!(bm["tested_party"], "DE GmbH");
}

#[tokio::test]
async fn test_benchmark_workflow() {
    let (_state, app) = setup_tp_test().await;
    let bm = create_test_benchmark(&app, "Workflow Study").await;
    let id = bm["id"].as_str().unwrap();
    assert_eq!(bm["status"], "draft");

    // Submit for review
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/transfer-pricing/benchmarks/{}/submit", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let in_review: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(in_review["status"], "in_review");

    // Approve
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/transfer-pricing/benchmarks/{}/approve", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let approved: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(approved["status"], "approved");
}

#[tokio::test]
async fn test_benchmark_reject() {
    let (_state, app) = setup_tp_test().await;
    let bm = create_test_benchmark(&app, "Reject Study").await;
    let id = bm["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/transfer-pricing/benchmarks/{}/submit", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/transfer-pricing/benchmarks/{}/reject", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let rejected: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(rejected["status"], "rejected");
}

#[tokio::test]
async fn test_cannot_approve_draft_benchmark() {
    let (_state, app) = setup_tp_test().await;
    let bm = create_test_benchmark(&app, "Draft Study").await;
    let id = bm["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/transfer-pricing/benchmarks/{}/approve", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_delete_benchmark() {
    let (_state, app) = setup_tp_test().await;
    let bm = create_test_benchmark(&app, "Delete Study").await;
    let id = bm["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/transfer-pricing/benchmarks/{}", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Comparables Tests
// ============================================================================

#[tokio::test]
async fn test_add_comparable() {
    let (_state, app) = setup_tp_test().await;
    let bm = create_test_benchmark(&app, "Comp Study").await;
    let bm_id = bm["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/transfer-pricing/benchmarks/{}/comparables", bm_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "comparable_number": 1,
            "company_name": "Comparable Corp A",
            "country": "DE",
            "industry_code": "NAICS-334",
            "industry_description": "Electronics Manufacturing",
            "fiscal_year": 2023,
            "revenue": "500000000",
            "operating_income": "50000000",
            "operating_margin_pct": "10.0",
            "net_income": "35000000",
            "total_assets": "750000000",
            "employees": 5000,
            "data_source": "Bureau van Dijk"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let comp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(comp["company_name"], "Comparable Corp A");
    assert_eq!(comp["country"], "DE");
    assert_eq!(comp["is_included"], true);
    assert_eq!(comp["comparable_number"], 1);
}

#[tokio::test]
async fn test_list_comparables() {
    let (_state, app) = setup_tp_test().await;
    let bm = create_test_benchmark(&app, "List Comp Study").await;
    let bm_id = bm["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    // Add two comparables
    for i in 1..=2 {
        let _ = app.clone().oneshot(Request::builder().method("POST")
            .uri(&format!("/api/v1/transfer-pricing/benchmarks/{}/comparables", bm_id))
            .header("Content-Type", "application/json").header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({
                "comparable_number": i,
                "company_name": format!("Company {}", i),
                "country": "DE",
                "revenue": "1000000",
                "operating_income": "100000",
                "operating_margin_pct": "10.0",
                "net_income": "70000",
                "total_assets": "2000000"
            })).unwrap())).unwrap()
        ).await.unwrap();
    }

    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/transfer-pricing/benchmarks/{}/comparables", bm_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(result["data"].as_array().unwrap().len() >= 2);
}

// ============================================================================
// Documentation Tests
// ============================================================================

#[tokio::test]
async fn test_create_documentation() {
    let (_state, app) = setup_tp_test().await;
    let doc = create_test_documentation(&app, "BEPS Local File FY2024").await;

    assert!(doc["doc_number"].as_str().unwrap().starts_with("TPD-"));
    assert_eq!(doc["doc_type"], "local_file");
    assert_eq!(doc["fiscal_year"], 2024);
    assert_eq!(doc["country"], "DE");
    assert_eq!(doc["status"], "draft");
}

#[tokio::test]
async fn test_documentation_workflow() {
    let (_state, app) = setup_tp_test().await;
    let doc = create_test_documentation(&app, "Workflow Doc").await;
    let id = doc["id"].as_str().unwrap();

    // Submit for review
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/transfer-pricing/documentation/{}/submit", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let in_review: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(in_review["status"], "in_review");

    // Approve
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/transfer-pricing/documentation/{}/approve", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let approved: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(approved["status"], "approved");

    // File
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/transfer-pricing/documentation/{}/file", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let filed: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(filed["status"], "filed");
}

#[tokio::test]
async fn test_cannot_file_unapproved_doc() {
    let (_state, app) = setup_tp_test().await;
    let doc = create_test_documentation(&app, "Unapproved Doc").await;
    let id = doc["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/transfer-pricing/documentation/{}/file", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_list_documentation() {
    let (_state, app) = setup_tp_test().await;
    create_test_documentation(&app, "Doc 1").await;
    create_test_documentation(&app, "Doc 2").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/transfer-pricing/documentation")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(result["data"].as_array().unwrap().len() >= 2);
}

// ============================================================================
// Dashboard Tests
// ============================================================================

#[tokio::test]
async fn test_tp_dashboard() {
    let (_state, app) = setup_tp_test().await;
    create_test_policy(&app, "TP-DASH").await;
    create_test_transaction(&app, None).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/transfer-pricing/dashboard")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let dashboard: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(dashboard["total_policies"].is_number());
    assert!(dashboard["active_policies"].is_number());
    assert!(dashboard["total_transactions"].is_number());
    assert!(dashboard["pending_transactions"].is_number());
    assert!(dashboard["total_benchmarks"].is_number());
    assert!(dashboard["total_documentation"].is_number());
}
