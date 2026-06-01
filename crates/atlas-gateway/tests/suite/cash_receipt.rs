//! Cash Receipt Management E2E Tests
//!
//! Tests for cash receipt lifecycle: batch creation, receipt recording,
//! applying receipts to invoices, unapplying, and reversal.
//! Oracle Fusion: Financials > Receivables > Receipts

use super::common::helpers::*;
use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;

async fn setup_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    let migration_sql = include_str!("../../../../migrations/148_cash_receipt_management.sql");
    sqlx::raw_sql(migration_sql)
        .execute(&state.db_pool)
        .await
        .ok();
    let app = build_router(state.clone());
    (state, app)
}

// ========================================================================
// Batch CRUD Tests
// ========================================================================

#[tokio::test]
async fn test_create_batch() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/cash-receipts/batches")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "batch_number": "BATCH-E2E-01",
                        "batch_name": "Daily Receipts Batch",
                        "description": "Today's customer receipts",
                        "receipt_method": "bank",
                        "currency_code": "USD",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["batch_number"], "BATCH-E2E-01");
    assert_eq!(d["batch_name"], "Daily Receipts Batch");
    assert_eq!(d["status"], "draft");
    assert_eq!(d["receipt_method"], "bank");
}

#[tokio::test]
async fn test_create_batch_empty_number_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/cash-receipts/batches")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "batch_number": "",
                        "batch_name": "Batch",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_batch_invalid_method_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/cash-receipts/batches")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "batch_number": "BATCH-BAD",
                        "batch_name": "Bad Batch",
                        "receipt_method": "crypto",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_duplicate_batch_number() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let body = serde_json::to_string(&json!({
        "batch_number": "BATCH-DUP",
        "batch_name": "Batch 1",
        "receipt_method": "bank",
    }))
    .unwrap();

    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/cash-receipts/batches")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(body.clone())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);

    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/cash-receipts/batches")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(body)
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_list_batches() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/cash-receipts/batches")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_list_batches_with_filter() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/cash-receipts/batches?status=draft")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_batch_not_found() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!(
                    "/api/v1/cash-receipts/batches/{}",
                    uuid::Uuid::new_v4()
                ))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_confirm_and_close_batch() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Create batch
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/cash-receipts/batches")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "batch_number": "BATCH-CONF-CLOSE",
                        "batch_name": "Batch",
                        "receipt_method": "bank",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let id = d["id"].as_str().unwrap();
    assert_eq!(d["status"], "draft");

    // Confirm
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/cash-receipts/batches/{}/confirm", id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["status"], "confirmed");

    // Close
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/cash-receipts/batches/{}/close", id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["status"], "closed");
}

#[tokio::test]
async fn test_cancel_draft_batch() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());

    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/cash-receipts/batches")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "batch_number": "BATCH-CANCEL",
                        "batch_name": "Batch",
                        "receipt_method": "cash",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let id = d["id"].as_str().unwrap();

    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/cash-receipts/batches/{}/cancel", id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["status"], "cancelled");
}

#[tokio::test]
async fn test_cancel_confirmed_batch_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());

    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/cash-receipts/batches")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "batch_number": "BATCH-CANCEL-CONF",
                        "batch_name": "Batch",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let id = d["id"].as_str().unwrap();

    // Confirm first
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/cash-receipts/batches/{}/confirm", id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Try to cancel confirmed batch
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/cash-receipts/batches/{}/cancel", id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ========================================================================
// Receipt CRUD Tests
// ========================================================================

#[tokio::test]
async fn test_create_receipt() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/cash-receipts")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "receipt_number": "RCP-E2E-001",
                        "customer_id": "00000000-0000-0000-0000-000000000100",
                        "customer_name": "Acme Corporation",
                        "customer_account_number": "CUST-001",
                        "payment_method": "check",
                        "amount": "5000.00",
                        "currency_code": "USD",
                        "receipt_date": "2025-01-15",
                        "reference_number": "CHK-12345",
                        "bank_name": "First National Bank",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["receipt_number"], "RCP-E2E-001");
    assert_eq!(d["amount"], "5000.00");
    assert_eq!(d["status"], "unapplied");
    assert_eq!(d["unapplied_amount"], "5000.00");
    assert_eq!(d["applied_amount"], "0.00");
}

#[tokio::test]
async fn test_create_receipt_empty_number_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/cash-receipts")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "receipt_number": "",
                        "customer_id": "00000000-0000-0000-0000-000000000100",
                        "amount": "100.00",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_receipt_negative_amount_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/cash-receipts")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "receipt_number": "RCP-NEG",
                        "customer_id": "00000000-0000-0000-0000-000000000100",
                        "amount": "-100.00",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_list_receipts() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/cash-receipts")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_list_receipts_with_status_filter() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/cash-receipts?status=unapplied")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_receipt_not_found() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!("/api/v1/cash-receipts/{}", uuid::Uuid::new_v4()))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

// ========================================================================
// Full Workflow: Create Batch → Create Receipt → Apply → Unapply → Reverse
// ========================================================================

#[tokio::test]
async fn test_full_workflow() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());

    // 1. Create batch
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/cash-receipts/batches")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "batch_number": "BATCH-WF",
                        "batch_name": "Workflow Batch",
                        "receipt_method": "bank",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let batch_d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let batch_id = batch_d["id"].as_str().unwrap();

    // 2. Create receipt in the batch
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/cash-receipts")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "batch_id": batch_id,
                        "receipt_number": "RCP-WF-001",
                        "customer_id": "00000000-0000-0000-0000-000000000100",
                        "customer_name": "Acme Corporation",
                        "payment_method": "check",
                        "amount": "10000.00",
                        "receipt_date": "2025-01-15",
                        "reference_number": "CHK-99999",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let receipt_d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let receipt_id = receipt_d["id"].as_str().unwrap();
    assert_eq!(receipt_d["status"], "unapplied");
    assert_eq!(receipt_d["unapplied_amount"], "10000.00");

    // 3. Apply receipt to invoice (partial: $3,000)
    let inv_id = uuid::Uuid::new_v4().to_string();
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/cash-receipts/applications")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "receipt_id": receipt_id,
                        "invoice_id": inv_id,
                        "invoice_number": "INV-001",
                        "applied_amount": "3000.00",
                        "discount_taken": "0.00",
                        "application_date": "2025-01-15",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let app_d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let app_id = app_d["id"].as_str().unwrap();
    assert_eq!(app_d["status"], "applied");
    assert_eq!(app_d["applied_amount"], "3000.00");

    // Verify receipt is partially applied
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!("/api/v1/cash-receipts/{}", receipt_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["status"], "partially_applied");
    assert_eq!(d["applied_amount"], "3000.00");
    assert_eq!(d["unapplied_amount"], "7000.00");

    // 4. Apply remaining amount to another invoice
    let inv2_id = uuid::Uuid::new_v4().to_string();
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/cash-receipts/applications")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "receipt_id": receipt_id,
                        "invoice_id": inv2_id,
                        "invoice_number": "INV-002",
                        "applied_amount": "7000.00",
                        "discount_taken": "0.00",
                        "application_date": "2025-01-15",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);

    // Verify receipt is now fully applied
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!("/api/v1/cash-receipts/{}", receipt_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["status"], "applied");
    assert_eq!(d["applied_amount"], "10000.00");
    assert_eq!(d["unapplied_amount"], "0.00");

    // 5. Unapply the first application
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!(
                    "/api/v1/cash-receipts/applications/{}/unapply",
                    app_id
                ))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["status"], "reversed");

    // Verify receipt is partially applied again
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!("/api/v1/cash-receipts/{}", receipt_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["status"], "partially_applied");
    assert_eq!(d["applied_amount"], "7000.00");
    assert_eq!(d["unapplied_amount"], "3000.00");

    // 6. Reverse the entire receipt
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/cash-receipts/{}/reverse", receipt_id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "reason": "Payment returned - NSF",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["status"], "reversed");
    assert_eq!(d["reversal_reason"], "Payment returned - NSF");
}

#[tokio::test]
async fn test_apply_exceeds_amount_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Create receipt for $500
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/cash-receipts")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "receipt_number": "RCP-EXC",
                        "customer_id": "00000000-0000-0000-0000-000000000100",
                        "amount": "500.00",
                        "payment_method": "cash",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let receipt_id = d["id"].as_str().unwrap();

    // Try to apply $600
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/cash-receipts/applications")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "receipt_id": receipt_id,
                        "invoice_id": uuid::Uuid::new_v4().to_string(),
                        "applied_amount": "600.00",
                        "discount_taken": "0.00",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_reverse_receipt_no_reason_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());

    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/cash-receipts")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "receipt_number": "RCP-REV-NR",
                        "customer_id": "00000000-0000-0000-0000-000000000100",
                        "amount": "100.00",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let receipt_id = d["id"].as_str().unwrap();

    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/cash-receipts/{}/reverse", receipt_id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "reason": "",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_reverse_already_reversed_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());

    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/cash-receipts")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "receipt_number": "RCP-REV-DUP",
                        "customer_id": "00000000-0000-0000-0000-000000000100",
                        "amount": "100.00",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let receipt_id = d["id"].as_str().unwrap();

    // First reversal
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/cash-receipts/{}/reverse", receipt_id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "reason": "Duplicate payment",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // Second reversal should fail
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/cash-receipts/{}/reverse", receipt_id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "reason": "Try again",
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ========================================================================
// List Applications Tests
// ========================================================================

#[tokio::test]
async fn test_list_applications() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!(
                    "/api/v1/cash-receipts/{}/applications",
                    uuid::Uuid::new_v4()
                ))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}

// ========================================================================
// Dashboard Test
// ========================================================================

#[tokio::test]
async fn test_get_dashboard() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/cash-receipts/dashboard")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX)
        .await
        .unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["total_receipts"], 0);
    assert_eq!(d["total_receipt_amount"], "0.00");
}
