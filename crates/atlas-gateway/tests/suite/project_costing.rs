//! Project Costing E2E Tests (Oracle Fusion Cloud ERP)
//!
//! Tests for Oracle Fusion Cloud ERP Project Costing:
//! - Cost transaction CRUD and lifecycle (draft → approved → distributed)
//! - Burden schedule management and overhead calculation
//! - Cost adjustments (increase, decrease, transfer, reversal)
//! - GL cost distributions
//! - Dashboard summary
//! - Validation edge cases

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_project_costing_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    let app = build_router(state.clone());
    (state, app)
}

async fn create_test_burden_schedule(
    app: &axum::Router,
    code: &str,
    name: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "code": code,
        "name": name,
        "effective_from": "2024-01-01",
        "is_default": true,
    });
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/project-costing/burden-schedules")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Failed to create burden schedule");
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_cost_transaction(
    app: &axum::Router,
    project_id: &str,
    cost_type: &str,
    raw_cost: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "project_id": project_id,
        "cost_type": cost_type,
        "raw_cost_amount": raw_cost,
        "currency_code": "USD",
        "transaction_date": "2024-06-15",
        "description": format!("{} cost for project", cost_type),
        "is_billable": true,
        "is_capitalizable": false,
    });
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/project-costing/transactions")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Failed to create cost transaction: status {}", r.status());
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Cost Transaction Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_create_cost_transaction() {
    let (_state, app) = setup_project_costing_test().await;
    let project_id = uuid::Uuid::new_v4().to_string();

    let txn = create_test_cost_transaction(&app, &project_id, "labor", "5000.00").await;

    assert_eq!(txn["cost_type"], "labor");
    assert_eq!(txn["status"], "draft");
    assert_eq!(txn["currency_code"], "USD");
    assert!(txn["raw_cost_amount"].as_str().unwrap().parse::<f64>().unwrap() == 5000.0);
    assert!(txn["transaction_number"].as_str().unwrap().starts_with("PJC-"));
    assert_eq!(txn["is_billable"], true);
}

#[tokio::test]
#[ignore]
async fn test_create_multiple_cost_types() {
    let (_state, app) = setup_project_costing_test().await;
    let project_id = uuid::Uuid::new_v4().to_string();

    let labor = create_test_cost_transaction(&app, &project_id, "labor", "5000.00").await;
    assert_eq!(labor["cost_type"], "labor");

    let material = create_test_cost_transaction(&app, &project_id, "material", "15000.00").await;
    assert_eq!(material["cost_type"], "material");

    let expense = create_test_cost_transaction(&app, &project_id, "expense", "2500.00").await;
    assert_eq!(expense["cost_type"], "expense");

    let equipment = create_test_cost_transaction(&app, &project_id, "equipment", "8000.00").await;
    assert_eq!(equipment["cost_type"], "equipment");
}

#[tokio::test]
#[ignore]
async fn test_list_cost_transactions() {
    let (_state, app) = setup_project_costing_test().await;
    let project_id = uuid::Uuid::new_v4().to_string();

    create_test_cost_transaction(&app, &project_id, "labor", "5000.00").await;
    create_test_cost_transaction(&app, &project_id, "material", "10000.00").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET").uri("/api/v1/project-costing/transactions")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(result["data"].as_array().unwrap().len(), 2);
}

#[tokio::test]
#[ignore]
async fn test_list_cost_transactions_by_project() {
    let (_state, app) = setup_project_costing_test().await;
    let project_a = uuid::Uuid::new_v4().to_string();
    let project_b = uuid::Uuid::new_v4().to_string();

    create_test_cost_transaction(&app, &project_a, "labor", "5000.00").await;
    create_test_cost_transaction(&app, &project_a, "material", "10000.00").await;
    create_test_cost_transaction(&app, &project_b, "expense", "2000.00").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/project-costing/transactions?project_id={}", project_a))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(result["data"].as_array().unwrap().len(), 2);
}

// ============================================================================
// Cost Transaction Lifecycle Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_approve_cost_transaction() {
    let (_state, app) = setup_project_costing_test().await;
    let project_id = uuid::Uuid::new_v4().to_string();

    let txn = create_test_cost_transaction(&app, &project_id, "labor", "5000.00").await;
    assert_eq!(txn["status"], "draft");

    let txn_id = txn["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/project-costing/transactions/{}/approve", txn_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let approved: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(approved["status"], "approved");
}

#[tokio::test]
#[ignore]
async fn test_reverse_cost_transaction() {
    let (_state, app) = setup_project_costing_test().await;
    let project_id = uuid::Uuid::new_v4().to_string();

    let txn = create_test_cost_transaction(&app, &project_id, "labor", "5000.00").await;
    let txn_id = txn["id"].as_str().unwrap();

    // Approve first
    let (k, v) = auth_header(&admin_claims());
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/project-costing/transactions/{}/approve", txn_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Reverse
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/project-costing/transactions/{}/reverse", txn_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "reason": "Entered in error"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let reversal: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(reversal["status"], "approved");
    assert_eq!(reversal["adjustment_type"], "reversal");
}

// ============================================================================
// Burden Schedule Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_create_burden_schedule() {
    let (_state, app) = setup_project_costing_test().await;

    let schedule = create_test_burden_schedule(&app, "OH-STD-2024", "Standard Overhead 2024").await;

    assert_eq!(schedule["code"], "OH-STD-2024");
    assert_eq!(schedule["name"], "Standard Overhead 2024");
    assert_eq!(schedule["status"], "draft");
    assert_eq!(schedule["is_default"], true);
}

#[tokio::test]
#[ignore]
async fn test_activate_burden_schedule() {
    let (_state, app) = setup_project_costing_test().await;

    let schedule = create_test_burden_schedule(&app, "OH-ACT", "Active Overhead").await;
    let schedule_id = schedule["id"].as_str().unwrap();

    // Activate
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/project-costing/burden-schedules/{}/activate", schedule_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let activated: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(activated["status"], "active");
}

#[tokio::test]
#[ignore]
async fn test_burden_schedule_with_lines() {
    let (_state, app) = setup_project_costing_test().await;

    let schedule = create_test_burden_schedule(&app, "OH-LINES", "Schedule with Lines").await;
    let schedule_id = schedule["id"].as_str().unwrap();

    // Add burden lines
    let (k, v) = auth_header(&admin_claims());

    // Labor at 25% overhead
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/project-costing/burden-schedules/{}/lines", schedule_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "cost_type": "labor",
            "burden_rate_percent": "25.00",
            "burden_account_code": "6200"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);

    // Material at 10% overhead
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/project-costing/burden-schedules/{}/lines", schedule_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "cost_type": "material",
            "burden_rate_percent": "10.00",
            "burden_account_code": "6210"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);

    // List lines
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/project-costing/burden-schedules/{}/lines", schedule_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let lines: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(lines["data"].as_array().unwrap().len(), 2);
}

#[tokio::test]
#[ignore]
async fn test_cost_with_burden_applied() {
    let (_state, app) = setup_project_costing_test().await;

    // Create and activate burden schedule with 25% labor overhead
    let schedule = create_test_burden_schedule(&app, "OH-BURDEN", "Burden Test").await;
    let schedule_id = schedule["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Add 25% labor burden line
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/project-costing/burden-schedules/{}/lines", schedule_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "cost_type": "labor",
            "burden_rate_percent": "25.00"
        })).unwrap())).unwrap()
    ).await.unwrap();

    // Activate the schedule
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/project-costing/burden-schedules/{}/activate", schedule_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Create a labor cost - should have 25% burden applied
    let project_id = uuid::Uuid::new_v4().to_string();
    let txn = create_test_cost_transaction(&app, &project_id, "labor", "10000.00").await;

    let raw: f64 = txn["raw_cost_amount"].as_str().unwrap().parse().unwrap();
    let burdened: f64 = txn["burdened_cost_amount"].as_str().unwrap().parse().unwrap();
    let burden: f64 = txn["burden_amount"].as_str().unwrap().parse().unwrap();

    assert_eq!(raw, 10000.0, "Raw cost should be 10000");
    assert!((burden - 2500.0).abs() < 1.0, "Burden should be ~2500 (25% of 10000), got {}", burden);
    assert!((burdened - 12500.0).abs() < 1.0, "Burdened cost should be ~12500, got {}", burdened);
}

// ============================================================================
// Cost Adjustment Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_create_cost_increase_adjustment() {
    let (_state, app) = setup_project_costing_test().await;
    let project_id = uuid::Uuid::new_v4().to_string();

    let txn = create_test_cost_transaction(&app, &project_id, "labor", "5000.00").await;
    let txn_id = txn["id"].as_str().unwrap();

    // Approve the transaction first
    let (k, v) = auth_header(&admin_claims());
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/project-costing/transactions/{}/approve", txn_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Create increase adjustment
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/project-costing/adjustments")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "original_transaction_id": txn_id,
            "adjustment_type": "increase",
            "adjustment_amount": "1000.00",
            "reason": "Additional scope of work",
            "effective_date": "2024-07-01"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let adj: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(adj["adjustment_type"], "increase");
    assert_eq!(adj["status"], "pending");
    assert!(adj["new_raw_cost"].as_str().unwrap().parse::<f64>().unwrap() > 5000.0);
}

#[tokio::test]
#[ignore]
async fn test_approve_cost_adjustment() {
    let (_state, app) = setup_project_costing_test().await;
    let project_id = uuid::Uuid::new_v4().to_string();

    let txn = create_test_cost_transaction(&app, &project_id, "material", "8000.00").await;
    let txn_id = txn["id"].as_str().unwrap();

    // Approve transaction
    let (k, v) = auth_header(&admin_claims());
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/project-costing/transactions/{}/approve", txn_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Create adjustment
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/project-costing/adjustments")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "original_transaction_id": txn_id,
            "adjustment_type": "decrease",
            "adjustment_amount": "2000.00",
            "reason": "Vendor credit received",
            "effective_date": "2024-07-01"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let adj: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let adj_id = adj["id"].as_str().unwrap();

    // Approve the adjustment
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/project-costing/adjustments/{}/approve", adj_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let approved: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(approved["status"], "approved");
}

// ============================================================================
// Cost Distribution Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_distribute_cost_transaction() {
    let (_state, app) = setup_project_costing_test().await;
    let project_id = uuid::Uuid::new_v4().to_string();

    let txn = create_test_cost_transaction(&app, &project_id, "expense", "3000.00").await;
    let txn_id = txn["id"].as_str().unwrap();

    // Approve first
    let (k, v) = auth_header(&admin_claims());
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/project-costing/transactions/{}/approve", txn_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Distribute
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/project-costing/transactions/{}/distribute", txn_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "raw_cost_account": "5100",
            "burden_account": "5200",
            "ap_ar_account": "2000"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let distributions = result["data"].as_array().unwrap();

    // Should have at least a raw_cost distribution
    assert!(distributions.len() >= 1, "Should have at least 1 distribution line");

    // Verify distribution structure
    let raw_dist = &distributions[0];
    assert_eq!(raw_dist["distribution_type"], "raw_cost");
    assert_eq!(raw_dist["debit_account_code"], "5100");
    assert_eq!(raw_dist["credit_account_code"], "2000");
    assert_eq!(raw_dist["is_posted"], false);
}

// ============================================================================
// Full Lifecycle Test
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_project_costing_full_lifecycle() {
    let (_state, app) = setup_project_costing_test().await;
    let project_id = uuid::Uuid::new_v4().to_string();

    // 1. Create cost transaction
    let txn = create_test_cost_transaction(&app, &project_id, "labor", "10000.00").await;
    let txn_id = txn["id"].as_str().unwrap();
    assert_eq!(txn["status"], "draft");

    // 2. Approve
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/project-costing/transactions/{}/approve", txn_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let approved: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    assert_eq!(approved["status"], "approved");

    // 3. Distribute to GL
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/project-costing/transactions/{}/distribute", txn_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "raw_cost_account": "5100",
            "burden_account": "5200",
            "ap_ar_account": "2000"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    // 4. Verify transaction is now distributed
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/project-costing/transactions/{}", txn_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(updated["status"], "distributed");
}

// ============================================================================
// Validation Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_cannot_create_transaction_with_invalid_cost_type() {
    let (_state, app) = setup_project_costing_test().await;

    let (k, v) = auth_header(&admin_claims());
    let project_id = uuid::Uuid::new_v4();
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/project-costing/transactions")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "project_id": project_id.to_string(),
            "cost_type": "invalid",
            "raw_cost_amount": "1000",
            "currency_code": "USD",
            "transaction_date": "2024-06-15"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore]
async fn test_cannot_approve_non_draft_transaction() {
    let (_state, app) = setup_project_costing_test().await;
    let project_id = uuid::Uuid::new_v4().to_string();

    let txn = create_test_cost_transaction(&app, &project_id, "labor", "5000.00").await;
    let txn_id = txn["id"].as_str().unwrap();

    // Approve first
    let (k, v) = auth_header(&admin_claims());
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/project-costing/transactions/{}/approve", txn_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Try to approve again
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/project-costing/transactions/{}/approve", txn_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore]
async fn test_cannot_distribute_unapproved_transaction() {
    let (_state, app) = setup_project_costing_test().await;
    let project_id = uuid::Uuid::new_v4().to_string();

    let txn = create_test_cost_transaction(&app, &project_id, "labor", "5000.00").await;
    let txn_id = txn["id"].as_str().unwrap();

    // Try to distribute without approving first
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/project-costing/transactions/{}/distribute", txn_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "raw_cost_account": "5100",
            "burden_account": "5200",
            "ap_ar_account": "2000"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Dashboard Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_project_costing_dashboard() {
    let (_state, app) = setup_project_costing_test().await;

    // Create some cost transactions
    let project_a = uuid::Uuid::new_v4().to_string();
    let project_b = uuid::Uuid::new_v4().to_string();

    create_test_cost_transaction(&app, &project_a, "labor", "10000.00").await;
    create_test_cost_transaction(&app, &project_a, "material", "5000.00").await;
    create_test_cost_transaction(&app, &project_b, "expense", "2000.00").await;

    // Get dashboard
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/project-costing/dashboard")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let summary: serde_json::Value = serde_json::from_slice(&b).unwrap();

    assert_eq!(summary["project_count"], 2);
    let total_raw: f64 = summary["total_raw_costs"].as_str().unwrap().parse().unwrap();
    assert!(total_raw > 0.0, "Total raw costs should be positive");
}

// ============================================================================
// List and Filter Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_list_transactions_by_cost_type() {
    let (_state, app) = setup_project_costing_test().await;
    let project_id = uuid::Uuid::new_v4().to_string();

    create_test_cost_transaction(&app, &project_id, "labor", "5000.00").await;
    create_test_cost_transaction(&app, &project_id, "material", "10000.00").await;
    create_test_cost_transaction(&app, &project_id, "labor", "3000.00").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/project-costing/transactions?cost_type=labor")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(result["data"].as_array().unwrap().len(), 2);
    for txn in result["data"].as_array().unwrap() {
        assert_eq!(txn["cost_type"], "labor");
    }
}

#[tokio::test]
#[ignore]
async fn test_list_burden_schedules() {
    let (_state, app) = setup_project_costing_test().await;

    create_test_burden_schedule(&app, "OH-001", "Schedule 1").await;
    create_test_burden_schedule(&app, "OH-002", "Schedule 2").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/project-costing/burden-schedules")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(result["data"].as_array().unwrap().len(), 2);
}
