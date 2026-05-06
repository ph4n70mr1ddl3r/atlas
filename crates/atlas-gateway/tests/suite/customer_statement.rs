//! Customer Statement / Balance Forward Billing E2E Tests
//!
//! Tests for Oracle Fusion Cloud ERP AR > Billing > Balance Forward Billing:
//! - Statement CRUD and lifecycle (draft → generated → sent → viewed → archived)
//! - Statement line management
//! - Closing balance and amount due calculations
//! - Cancel and resend operations
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
    // Clean test data
    sqlx::query("DELETE FROM _atlas.customer_statement_lines").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM _atlas.customer_statements").execute(&state.db_pool).await.ok();
    sqlx::query("CREATE SCHEMA IF NOT EXISTS _atlas")
        .execute(&state.db_pool)
        .await
        .ok();
    sqlx::query(include_str!("../../../../migrations/126_customer_statement.sql"))
        .execute(&state.db_pool)
        .await
        .ok();
    let app = build_router(state.clone());
    (state, app)
}

async fn create_statement(
    app: &axum::Router,
    customer_id: &str,
    billing_period_from: &str,
    billing_period_to: &str,
    opening_balance: &str,
    total_charges: &str,
    total_payments: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "customer_id": customer_id,
        "customer_number": "CUST-001",
        "customer_name": "Test Customer",
        "statement_date": "2024-06-30",
        "billing_period_from": billing_period_from,
        "billing_period_to": billing_period_to,
        "billing_cycle": "monthly",
        "opening_balance": opening_balance,
        "total_charges": total_charges,
        "total_payments": total_payments,
        "total_credits": "0.00",
        "total_adjustments": "0.00",
        "aging_current": "5000.00",
        "aging_1_30": "2000.00",
        "aging_31_60": "0.00",
        "aging_61_90": "0.00",
        "aging_91_120": "0.00",
        "aging_121_plus": "0.00",
        "currency_code": "USD",
        "delivery_method": "email",
        "delivery_email": "customer@example.com",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/customer-statements")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8_lossy(&b);
    eprintln!("CREATE STATEMENT status={}: {}", status, body_str);
    assert_eq!(status, StatusCode::CREATED, "Failed to create statement: {:?}", body_str);
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Statement CRUD Tests
// ============================================================================

#[tokio::test]
async fn test_create_statement() {
    let (_state, app) = setup_test().await;
    let customer_id = Uuid::new_v4().to_string();
    let stmt = create_statement(&app, &customer_id, "2024-06-01", "2024-06-30", "10000.00", "2500.00", "3000.00").await;

    assert!(stmt["statementNumber"].as_str().unwrap().starts_with("CS-"));
    assert_eq!(stmt["billingCycle"], "monthly");
    assert_eq!(stmt["currencyCode"], "USD");
    assert_eq!(stmt["status"], "draft");
    assert_eq!(stmt["deliveryMethod"], "email");
    assert_eq!(stmt["customerNumber"], "CUST-001");
    assert_eq!(stmt["customerName"], "Test Customer");

    // Verify closing balance: 10000 + 2500 - 3000 = 9500
    let closing: f64 = stmt["closingBalance"].as_str().unwrap().parse().unwrap();
    assert!((closing - 9500.0).abs() < 1.0, "Expected closing 9500, got {}", closing);

    // Amount due = max(closing, 0)
    let due: f64 = stmt["amountDue"].as_str().unwrap().parse().unwrap();
    assert!((due - 9500.0).abs() < 1.0);
}

#[tokio::test]
async fn test_create_statement_with_negative_balance() {
    let (_state, app) = setup_test().await;
    let customer_id = Uuid::new_v4().to_string();
    // opening=100, charges=0, payments=500 => closing = 100 + 0 - 500 = -400
    let stmt = create_statement(&app, &customer_id, "2024-06-01", "2024-06-30", "100.00", "0.00", "500.00").await;

    let closing: f64 = stmt["closingBalance"].as_str().unwrap().parse().unwrap();
    assert!((closing - (-400.0)).abs() < 1.0);

    // Amount due should be 0 for negative balance
    let due: f64 = stmt["amountDue"].as_str().unwrap().parse().unwrap();
    assert!((due - 0.0).abs() < 0.01, "Expected amount due 0 for credit balance, got {}", due);
}

#[tokio::test]
async fn test_get_statement() {
    let (_state, app) = setup_test().await;
    let customer_id = Uuid::new_v4().to_string();
    let stmt = create_statement(&app, &customer_id, "2024-06-01", "2024-06-30", "5000.00", "1000.00", "0.00").await;
    let stmt_id: Uuid = stmt["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/customer-statements/{}", stmt_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["id"], stmt["id"]);
}

#[tokio::test]
async fn test_list_statements() {
    let (_state, app) = setup_test().await;
    let cust1 = Uuid::new_v4().to_string();
    let cust2 = Uuid::new_v4().to_string();
    create_statement(&app, &cust1, "2024-05-01", "2024-05-31", "1000.00", "500.00", "200.00").await;
    create_statement(&app, &cust2, "2024-06-01", "2024-06-30", "2000.00", "800.00", "300.00").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/customer-statements")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(body["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_list_statements_filter_by_customer() {
    let (_state, app) = setup_test().await;
    let cust1 = Uuid::new_v4().to_string();
    let cust2 = Uuid::new_v4().to_string();
    create_statement(&app, &cust1, "2024-06-01", "2024-06-30", "1000.00", "0.00", "0.00").await;
    create_statement(&app, &cust2, "2024-06-01", "2024-06-30", "2000.00", "0.00", "0.00").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/customer-statements?customer_id={}", cust1))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let data = body["data"].as_array().unwrap();
    assert_eq!(data.len(), 1);
    assert_eq!(data[0]["customerId"], cust1);
}

#[tokio::test]
async fn test_list_statements_filter_by_status() {
    let (_state, app) = setup_test().await;
    let cust1 = Uuid::new_v4().to_string();
    let cust2 = Uuid::new_v4().to_string();
    let stmt1 = create_statement(&app, &cust1, "2024-06-01", "2024-06-30", "1000.00", "0.00", "0.00").await;
    create_statement(&app, &cust2, "2024-06-01", "2024-06-30", "2000.00", "0.00", "0.00").await;

    let (k, v) = auth_header(&admin_claims());

    // Generate stmt1
    let stmt1_id: Uuid = stmt1["id"].as_str().unwrap().parse().unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/customer-statements/{}/generate", stmt1_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Filter by draft
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/customer-statements?status=draft")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let data = body["data"].as_array().unwrap();
    assert_eq!(data.len(), 1);
    assert_eq!(data[0]["status"], "draft");

    // Filter by generated
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/customer-statements?status=generated")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let data = body["data"].as_array().unwrap();
    assert_eq!(data.len(), 1);
    assert_eq!(data[0]["status"], "generated");
}

// ============================================================================
// Statement Lifecycle Tests
// ============================================================================

#[tokio::test]
async fn test_statement_lifecycle() {
    let (_state, app) = setup_test().await;
    let customer_id = Uuid::new_v4().to_string();
    let stmt = create_statement(&app, &customer_id, "2024-06-01", "2024-06-30", "5000.00", "1000.00", "0.00").await;
    let stmt_id: Uuid = stmt["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Generate
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/customer-statements/{}/generate", stmt_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "generated");
    assert!(body["generatedAt"].is_string());

    // Send
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/customer-statements/{}/send", stmt_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "sent");
    assert!(body["sentAt"].is_string());

    // View
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/customer-statements/{}/view", stmt_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "viewed");

    // Archive
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/customer-statements/{}/archive", stmt_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "archived");
}

#[tokio::test]
async fn test_cancel_statement() {
    let (_state, app) = setup_test().await;
    let customer_id = Uuid::new_v4().to_string();
    let stmt = create_statement(&app, &customer_id, "2024-06-01", "2024-06-30", "1000.00", "0.00", "0.00").await;
    let stmt_id: Uuid = stmt["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/customer-statements/{}/cancel", stmt_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"reason": "Customer requested cancellation"})).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "cancelled");
}

#[tokio::test]
async fn test_resend_statement() {
    let (_state, app) = setup_test().await;
    let customer_id = Uuid::new_v4().to_string();
    let stmt = create_statement(&app, &customer_id, "2024-06-01", "2024-06-30", "1000.00", "0.00", "0.00").await;
    let stmt_id: Uuid = stmt["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Generate and send first
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/customer-statements/{}/generate", stmt_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/customer-statements/{}/send", stmt_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Resend
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/customer-statements/{}/resend", stmt_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "sent");
}

#[tokio::test]
async fn test_cancel_non_draft_fails() {
    let (_state, app) = setup_test().await;
    let customer_id = Uuid::new_v4().to_string();
    let stmt = create_statement(&app, &customer_id, "2024-06-01", "2024-06-30", "1000.00", "0.00", "0.00").await;
    let stmt_id: Uuid = stmt["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Generate first
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/customer-statements/{}/generate", stmt_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Try to cancel generated statement
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/customer-statements/{}/cancel", stmt_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"reason": "too late"})).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_generate_non_draft_fails() {
    let (_state, app) = setup_test().await;
    let customer_id = Uuid::new_v4().to_string();
    let stmt = create_statement(&app, &customer_id, "2024-06-01", "2024-06-30", "1000.00", "0.00", "0.00").await;
    let stmt_id: Uuid = stmt["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Generate once
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/customer-statements/{}/generate", stmt_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Try to generate again
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/customer-statements/{}/generate", stmt_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Statement Lines Tests
// ============================================================================

#[tokio::test]
async fn test_add_statement_lines() {
    let (_state, app) = setup_test().await;
    let customer_id = Uuid::new_v4().to_string();
    let stmt = create_statement(&app, &customer_id, "2024-06-01", "2024-06-30", "5000.00", "0.00", "0.00").await;
    let stmt_id: Uuid = stmt["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Add invoice line
    let payload = json!({
        "line_type": "invoice",
        "transaction_number": "INV-001",
        "transaction_date": "2024-06-10",
        "due_date": "2024-07-10",
        "amount": "2500.00",
        "description": "Service invoice",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/customer-statements/{}/lines", stmt_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let line: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(line["lineType"], "invoice");
    assert_eq!(line["transactionNumber"], "INV-001");

    // Add payment line
    let payload = json!({
        "line_type": "payment",
        "transaction_number": "RECEIPT-001",
        "transaction_date": "2024-06-20",
        "amount": "1000.00",
        "description": "Customer payment",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/customer-statements/{}/lines", stmt_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn test_list_statement_lines() {
    let (_state, app) = setup_test().await;
    let customer_id = Uuid::new_v4().to_string();
    let stmt = create_statement(&app, &customer_id, "2024-06-01", "2024-06-30", "1000.00", "0.00", "0.00").await;
    let stmt_id: Uuid = stmt["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Add two lines
    for (lt, amt) in &[("invoice", "2000.00"), ("payment", "500.00")] {
        let payload = json!({
            "line_type": lt,
            "amount": amt,
        });
        app.clone().oneshot(Request::builder().method("POST")
            .uri(&format!("/api/v1/customer-statements/{}/lines", stmt_id))
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(serde_json::to_string(&payload).unwrap()))
            .unwrap()
        ).await.unwrap();
    }

    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/customer-statements/{}/lines", stmt_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["data"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_remove_statement_line() {
    let (_state, app) = setup_test().await;
    let customer_id = Uuid::new_v4().to_string();
    let stmt = create_statement(&app, &customer_id, "2024-06-01", "2024-06-30", "1000.00", "0.00", "0.00").await;
    let stmt_id: Uuid = stmt["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Add line
    let payload = json!({
        "line_type": "invoice",
        "amount": "1000.00",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/customer-statements/{}/lines", stmt_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    let line: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let line_id: Uuid = line["id"].as_str().unwrap().parse().unwrap();

    // Remove line
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/customer-statements/{}/lines/{}", stmt_id, line_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    // Verify line is gone
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/customer-statements/{}/lines", stmt_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["data"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_add_line_to_non_draft_fails() {
    let (_state, app) = setup_test().await;
    let customer_id = Uuid::new_v4().to_string();
    let stmt = create_statement(&app, &customer_id, "2024-06-01", "2024-06-30", "1000.00", "0.00", "0.00").await;
    let stmt_id: Uuid = stmt["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Generate the statement first
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/customer-statements/{}/generate", stmt_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Try to add line to generated statement
    let payload = json!({
        "line_type": "invoice",
        "amount": "1000.00",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/customer-statements/{}/lines", stmt_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Dashboard Test
// ============================================================================

#[tokio::test]
async fn test_statement_dashboard() {
    let (_state, app) = setup_test().await;
    let customer_id = Uuid::new_v4().to_string();
    create_statement(&app, &customer_id, "2024-06-01", "2024-06-30", "5000.00", "1000.00", "500.00").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/customer-statements/dashboard")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();

    assert!(body.get("totalStatements").is_some());
    assert!(body.get("draftCount").is_some());
    assert!(body.get("sentCount").is_some());
    assert!(body.get("totalAmountOutstanding").is_some());
    assert!(body.get("byBillingCycle").is_some());
    assert!(body.get("byCurrency").is_some());
}

// ============================================================================
// Validation Edge Cases
// ============================================================================

#[tokio::test]
async fn test_create_statement_invalid_billing_cycle() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let customer_id = Uuid::new_v4().to_string();
    let payload = json!({
        "customer_id": customer_id,
        "statement_date": "2024-06-30",
        "billing_period_from": "2024-06-01",
        "billing_period_to": "2024-06-30",
        "billing_cycle": "yearly",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/customer-statements")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_statement_inverted_period_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let customer_id = Uuid::new_v4().to_string();
    let payload = json!({
        "customer_id": customer_id,
        "statement_date": "2024-06-30",
        "billing_period_from": "2024-06-30",
        "billing_period_to": "2024-06-01",
        "billing_cycle": "monthly",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/customer-statements")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_statement_invalid_delivery_method() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let customer_id = Uuid::new_v4().to_string();
    let payload = json!({
        "customer_id": customer_id,
        "statement_date": "2024-06-30",
        "billing_period_from": "2024-06-01",
        "billing_period_to": "2024-06-30",
        "billing_cycle": "monthly",
        "delivery_method": "fax",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/customer-statements")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_add_line_invalid_type_fails() {
    let (_state, app) = setup_test().await;
    let customer_id = Uuid::new_v4().to_string();
    let stmt = create_statement(&app, &customer_id, "2024-06-01", "2024-06-30", "1000.00", "0.00", "0.00").await;
    let stmt_id: Uuid = stmt["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "line_type": "invalid_type",
        "amount": "100.00",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/customer-statements/{}/lines", stmt_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_statement_quarterly_cycle() {
    let (_state, app) = setup_test().await;
    let customer_id = Uuid::new_v4().to_string();
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "customer_id": customer_id,
        "customer_name": "Quarterly Customer",
        "statement_date": "2024-06-30",
        "billing_period_from": "2024-04-01",
        "billing_period_to": "2024-06-30",
        "billing_cycle": "quarterly",
        "opening_balance": "10000.00",
        "total_charges": "5000.00",
        "total_payments": "2000.00",
        "total_credits": "500.00",
        "total_adjustments": "100.00",
        "currency_code": "EUR",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/customer-statements")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let stmt: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(stmt["billingCycle"], "quarterly");
    assert_eq!(stmt["currencyCode"], "EUR");

    // closing = 10000 + 5000 - 2000 - 500 + 100 = 12600
    let closing: f64 = stmt["closingBalance"].as_str().unwrap().parse().unwrap();
    assert!((closing - 12600.0).abs() < 1.0);
}
