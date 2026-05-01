//! Accounts Receivable E2E Tests
//!
//! Tests for Oracle Fusion Cloud ERP Accounts Receivable:
//! - Transaction CRUD
//! - Transaction lifecycle (create → complete → post → cancel)
//! - Transaction lines
//! - Receipt processing
//! - Credit memo management
//! - AR aging

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_ar_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    let app = build_router(state.clone());
    (state, app)
}

const CUSTOMER_ID: &str = "00000000-0000-0000-0000-000000000200";

async fn create_test_transaction(
    app: &axum::Router,
    transaction_type: &str,
    amount: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/ar/transactions")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "transaction_type": transaction_type,
            "transaction_date": "2026-04-15",
            "customer_id": CUSTOMER_ID,
            "customer_name": "Test Customer",
            "currency_code": "USD",
            "entered_amount": amount,
            "tax_amount": "0.00",
            "due_date": "2026-05-15",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Failed to create AR transaction");
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Transaction CRUD Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_create_ar_invoice() {
    let (_state, app) = setup_ar_test().await;

    let txn = create_test_transaction(&app, "invoice", "1500.00").await;

    assert_eq!(txn["transaction_type"], "invoice");
    assert_eq!(txn["status"], "draft");
    assert!(txn["transaction_number"].as_str().unwrap().starts_with("AR-"));
}

#[tokio::test]
#[ignore]
async fn test_list_ar_transactions() {
    let (_state, app) = setup_ar_test().await;

    create_test_transaction(&app, "invoice", "100.00").await;
    create_test_transaction(&app, "debit_memo", "200.00").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET").uri("/api/v1/ar/transactions")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(result["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
#[ignore]
async fn test_get_ar_transaction() {
    let (_state, app) = setup_ar_test().await;

    let txn = create_test_transaction(&app, "invoice", "500.00").await;
    let txn_id = txn["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/ar/transactions/{}", txn_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}

#[tokio::test]
#[ignore]
async fn test_filter_transactions_by_type() {
    let (_state, app) = setup_ar_test().await;

    create_test_transaction(&app, "invoice", "100.00").await;
    create_test_transaction(&app, "debit_memo", "200.00").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/ar/transactions?transaction_type=invoice")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let txns = result["data"].as_array().unwrap();
    assert!(txns.iter().all(|t| t["transaction_type"] == "invoice"));
}

// ============================================================================
// Transaction Lifecycle Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_ar_transaction_complete_and_post() {
    let (_state, app) = setup_ar_test().await;

    let txn = create_test_transaction(&app, "invoice", "1000.00").await;
    let txn_id = txn["id"].as_str().unwrap();
    assert_eq!(txn["status"], "draft");

    // Complete
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/ar/transactions/{}/complete", txn_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let completed: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(completed["status"], "complete");

    // Post
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/ar/transactions/{}/post", txn_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let posted: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(posted["status"], "open");
}

#[tokio::test]
#[ignore]
async fn test_cancel_ar_transaction() {
    let (_state, app) = setup_ar_test().await;

    let txn = create_test_transaction(&app, "invoice", "500.00").await;
    let txn_id = txn["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/ar/transactions/{}/cancel", txn_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let cancelled: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(cancelled["status"], "cancelled");
}

// ============================================================================
// Transaction Line Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_add_transaction_line() {
    let (_state, app) = setup_ar_test().await;

    let txn = create_test_transaction(&app, "invoice", "1000.00").await;
    let txn_id = txn["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/ar/transactions/{}/lines", txn_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "line_type": "line",
            "description": "Consulting services",
            "item_code": "SRV-001",
            "quantity": "10",
            "unit_price": "100.00",
            "line_amount": "1000.00",
            "tax_amount": "0.00",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let line: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(line["line_type"], "line");
    assert_eq!(line["line_number"], 1);
}

#[tokio::test]
#[ignore]
async fn test_list_transaction_lines() {
    let (_state, app) = setup_ar_test().await;

    let txn = create_test_transaction(&app, "invoice", "2000.00").await;
    let txn_id = txn["id"].as_str().unwrap();

    // Add two lines
    let (k, v) = auth_header(&admin_claims());
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/ar/transactions/{}/lines", txn_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "line_type": "line",
            "description": "Services",
            "line_amount": "1200.00",
            "tax_amount": "0.00",
        })).unwrap())).unwrap()
    ).await.unwrap();
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/ar/transactions/{}/lines", txn_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "line_type": "line",
            "description": "Hardware",
            "line_amount": "800.00",
            "tax_amount": "0.00",
        })).unwrap())).unwrap()
    ).await.unwrap();

    // List lines
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/ar/transactions/{}/lines", txn_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(result["data"].as_array().unwrap().len(), 2);
}

// ============================================================================
// Receipt Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_create_and_confirm_receipt() {
    let (_state, app) = setup_ar_test().await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/ar/receipts")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "receipt_date": "2026-04-20",
            "receipt_type": "cash",
            "receipt_method": "manual_receipt",
            "amount": "1500.00",
            "currency_code": "USD",
            "customer_id": CUSTOMER_ID,
            "customer_name": "Test Customer",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let receipt: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(receipt["status"], "draft");
    let receipt_id = receipt["id"].as_str().unwrap();

    // Confirm
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/ar/receipts/{}/confirm", receipt_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let confirmed: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(confirmed["status"], "confirmed");
}

// ============================================================================
// Credit Memo Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_create_and_approve_credit_memo() {
    let (_state, app) = setup_ar_test().await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/ar/credit-memos")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "customer_id": CUSTOMER_ID,
            "customer_name": "Test Customer",
            "credit_memo_date": "2026-04-20",
            "reason_code": "return",
            "reason_description": "Product returned",
            "amount": "200.00",
            "tax_amount": "0.00",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let memo: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(memo["reason_code"], "return");
    let memo_id = memo["id"].as_str().unwrap();

    // Approve
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/ar/credit-memos/{}/approve", memo_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}

// ============================================================================
// AR Aging Test
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_ar_aging_summary() {
    let (_state, app) = setup_ar_test().await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/ar/aging?as_of_date=2026-04-15")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let aging: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(aging["as_of_date"].is_string());
}
