//! Remittance Batch E2E Tests
//!
//! Tests for Oracle Fusion Cloud ERP AR > Receipts > Remittance Batches:
//! - Batch CRUD and lifecycle (draft → approved → formatted → transmitted → confirmed → settled)
//! - Receipt management (add/remove to batches)
//! - Batch totals recalculation
//! - Cancel and reverse operations
//! - Remittance advice
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
    sqlx::query("DELETE FROM _atlas.remittance_batch_receipts").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM _atlas.remittance_batches").execute(&state.db_pool).await.ok();
    // Reset the sequence by deleting and re-creating
    sqlx::query("TRUNCATE _atlas.remittance_batches CASCADE").execute(&state.db_pool).await.ok();
    sqlx::query("CREATE SCHEMA IF NOT EXISTS _atlas")
        .execute(&state.db_pool)
        .await
        .ok();
    sqlx::query(include_str!("../../../../migrations/127_remittance_batch.sql"))
        .execute(&state.db_pool)
        .await
        .ok();
    let app = build_router(state.clone());
    (state, app)
}

async fn create_batch(
    app: &axum::Router,
    remittance_method: &str,
    currency_code: &str,
    batch_date: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "batch_name": "Test Batch",
        "bank_account_name": "Main Operating Account",
        "bank_name": "First National Bank",
        "remittance_method": remittance_method,
        "currency_code": currency_code,
        "batch_date": batch_date,
        "gl_date": batch_date,
        "notes": "Test remittance batch",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/remittance-batches")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8_lossy(&b);
    eprintln!("CREATE BATCH status={}: {}", status, body_str);
    assert_eq!(status, StatusCode::CREATED, "Failed to create batch: {:?}", body_str);
    serde_json::from_slice(&b).unwrap()
}

async fn add_receipt(
    app: &axum::Router,
    batch_id: Uuid,
    receipt_id: &str,
    amount: &str,
    customer_name: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "receipt_id": receipt_id,
        "receipt_number": format!("RCT-{}", &receipt_id[..4]),
        "customer_name": customer_name,
        "customer_number": "CUST-001",
        "receipt_date": "2024-06-15",
        "receipt_amount": amount,
        "applied_amount": amount,
        "receipt_method": "check",
        "currency_code": "USD",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/remittance-batches/{}/receipts", batch_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8_lossy(&b);
    eprintln!("ADD RECEIPT status={}: {}", status, body_str);
    assert_eq!(status, StatusCode::CREATED, "Failed to add receipt: {:?}", body_str);
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Batch CRUD Tests
// ============================================================================

#[tokio::test]
async fn test_create_batch() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "standard", "USD", "2024-06-30").await;

    assert!(batch["batchNumber"].as_str().unwrap().starts_with("RB-"));
    assert_eq!(batch["remittanceMethod"], "standard");
    assert_eq!(batch["currencyCode"], "USD");
    assert_eq!(batch["status"], "draft");
    assert_eq!(batch["batchName"], "Test Batch");
    assert_eq!(batch["bankName"], "First National Bank");
    assert_eq!(batch["bankAccountName"], "Main Operating Account");
}

#[tokio::test]
async fn test_create_batch_with_factoring() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "factoring", "EUR", "2024-07-15").await;

    assert_eq!(batch["remittanceMethod"], "factoring");
    assert_eq!(batch["currencyCode"], "EUR");
}

#[tokio::test]
async fn test_get_batch() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "standard", "USD", "2024-06-30").await;
    let batch_id: Uuid = batch["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/remittance-batches/{}", batch_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["id"], batch["id"]);
}

#[tokio::test]
async fn test_list_batches() {
    let (_state, app) = setup_test().await;
    create_batch(&app, "standard", "USD", "2024-06-15").await;
    create_batch(&app, "factoring", "EUR", "2024-06-30").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/remittance-batches")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(body["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_list_batches_filter_by_status() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "standard", "USD", "2024-06-30").await;
    let batch_id: Uuid = batch["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Approve one batch
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/remittance-batches/{}/approve", batch_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Filter by draft
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/remittance-batches?status=draft")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let data = body["data"].as_array().unwrap();
    assert!(data.iter().all(|b| b["status"] == "draft"));

    // Filter by approved
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/remittance-batches?status=approved")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let data = body["data"].as_array().unwrap();
    assert!(data.iter().all(|b| b["status"] == "approved"));
}

#[tokio::test]
async fn test_list_batches_filter_by_currency() {
    let (_state, app) = setup_test().await;
    create_batch(&app, "standard", "USD", "2024-06-15").await;
    create_batch(&app, "factoring", "EUR", "2024-06-30").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/remittance-batches?currency_code=EUR")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let data = body["data"].as_array().unwrap();
    assert!(data.iter().all(|b| b["currencyCode"] == "EUR"));
}

// ============================================================================
// Batch Lifecycle Tests
// ============================================================================

#[tokio::test]
async fn test_batch_full_lifecycle() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "standard", "USD", "2024-06-30").await;
    let batch_id: Uuid = batch["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Add a receipt first (needed before formatting)
    let receipt_id = Uuid::new_v4();
    add_receipt(&app, batch_id, &receipt_id.to_string(), "5000.00", "Acme Corp").await;

    // Approve
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/remittance-batches/{}/approve", batch_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "approved");

    // Format
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/remittance-batches/{}/format", batch_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "formatted");
    assert!(body["formatDate"].is_string());

    // Transmit
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/remittance-batches/{}/transmit", batch_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"reference_number": "BANK-REF-001"})).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "transmitted");
    assert!(body["transmissionDate"].is_string());

    // Confirm
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/remittance-batches/{}/confirm", batch_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "confirmed");
    assert!(body["confirmationDate"].is_string());

    // Settle
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/remittance-batches/{}/settle", batch_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "settled");
    assert!(body["settlementDate"].is_string());
}

#[tokio::test]
async fn test_reverse_settled_batch() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "standard", "USD", "2024-06-30").await;
    let batch_id: Uuid = batch["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Add receipt and move to settled
    let receipt_id = Uuid::new_v4();
    add_receipt(&app, batch_id, &receipt_id.to_string(), "3000.00", "Corp Inc").await;

    for action in &["approve", "format", "transmit", "confirm", "settle"] {
        let body_payload = if *action == "transmit" {
            Body::from(serde_json::to_string(&json!({})).unwrap())
        } else {
            Body::empty()
        };
        app.clone().oneshot(Request::builder().method("POST")
            .uri(&format!("/api/v1/remittance-batches/{}/{}", batch_id, action))
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(body_payload)
            .unwrap()
        ).await.unwrap();
    }

    // Reverse
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/remittance-batches/{}/reverse", batch_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"reason": "Bank returned items"})).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "reversed");
    assert!(body["reversalDate"].is_string());
}

#[tokio::test]
async fn test_cancel_draft_batch() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "standard", "USD", "2024-06-30").await;
    let batch_id: Uuid = batch["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/remittance-batches/{}/cancel", batch_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"reason": "Duplicate batch"})).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "cancelled");
}

#[tokio::test]
async fn test_cancel_approved_batch() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "standard", "USD", "2024-06-30").await;
    let batch_id: Uuid = batch["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/remittance-batches/{}/approve", batch_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/remittance-batches/{}/cancel", batch_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"reason": "Changed mind"})).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_invalid_transition_skip_steps() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "standard", "USD", "2024-06-30").await;
    let batch_id: Uuid = batch["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Try to format without approving first
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/remittance-batches/{}/format", batch_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_cancel_from_formatted_fails() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "standard", "USD", "2024-06-30").await;
    let batch_id: Uuid = batch["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Add receipt and approve + format
    let receipt_id = Uuid::new_v4();
    add_receipt(&app, batch_id, &receipt_id.to_string(), "1000.00", "Test").await;
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/remittance-batches/{}/approve", batch_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/remittance-batches/{}/format", batch_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Try to cancel from formatted
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/remittance-batches/{}/cancel", batch_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"reason": "too late"})).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_reverse_non_settled_fails() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "standard", "USD", "2024-06-30").await;
    let batch_id: Uuid = batch["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/remittance-batches/{}/reverse", batch_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"reason": "error"})).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_format_empty_batch_fails() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "standard", "USD", "2024-06-30").await;
    let batch_id: Uuid = batch["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Approve without adding receipts
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/remittance-batches/{}/approve", batch_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Try to format empty batch
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/remittance-batches/{}/format", batch_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Batch Receipts Tests
// ============================================================================

#[tokio::test]
async fn test_add_receipts_and_totals() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "standard", "USD", "2024-06-30").await;
    let batch_id: Uuid = batch["id"].as_str().unwrap().parse().unwrap();

    // Add two receipts
    let r1 = Uuid::new_v4();
    let r2 = Uuid::new_v4();
    add_receipt(&app, batch_id, &r1.to_string(), "3000.00", "Customer A").await;
    add_receipt(&app, batch_id, &r2.to_string(), "2000.00", "Customer B").await;

    // Verify totals updated
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/remittance-batches/{}", batch_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();

    let total: f64 = body["totalAmount"].as_str().unwrap().parse().unwrap();
    assert!((total - 5000.0).abs() < 1.0, "Expected total 5000, got {}", total);
    assert_eq!(body["receiptCount"], 2);
}

#[tokio::test]
async fn test_remove_receipt_and_recalc() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "standard", "USD", "2024-06-30").await;
    let batch_id: Uuid = batch["id"].as_str().unwrap().parse().unwrap();

    let r1 = Uuid::new_v4();
    let r2 = Uuid::new_v4();
    add_receipt(&app, batch_id, &r1.to_string(), "3000.00", "Customer A").await;
    add_receipt(&app, batch_id, &r2.to_string(), "2000.00", "Customer B").await;

    let (k, v) = auth_header(&admin_claims());

    // Remove first receipt
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/remittance-batches/{}/receipts/{}", batch_id, r1))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    // Verify totals recalculated
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/remittance-batches/{}", batch_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();

    let total: f64 = body["totalAmount"].as_str().unwrap().parse().unwrap();
    assert!((total - 2000.0).abs() < 1.0, "Expected total 2000 after removal, got {}", total);
    assert_eq!(body["receiptCount"], 1);
}

#[tokio::test]
async fn test_list_batch_receipts() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "standard", "USD", "2024-06-30").await;
    let batch_id: Uuid = batch["id"].as_str().unwrap().parse().unwrap();

    let r1 = Uuid::new_v4();
    let r2 = Uuid::new_v4();
    add_receipt(&app, batch_id, &r1.to_string(), "1500.00", "Customer A").await;
    add_receipt(&app, batch_id, &r2.to_string(), "2500.00", "Customer B").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/remittance-batches/{}/receipts", batch_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["data"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_add_receipt_to_non_draft_fails() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "standard", "USD", "2024-06-30").await;
    let batch_id: Uuid = batch["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Approve the batch first
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/remittance-batches/{}/approve", batch_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Try to add receipt to approved batch
    let receipt_id = Uuid::new_v4();
    let payload = json!({
        "receipt_id": receipt_id.to_string(),
        "receipt_amount": "1000.00",
        "applied_amount": "1000.00",
        "currency_code": "USD",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/remittance-batches/{}/receipts", batch_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_duplicate_receipt_fails() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "standard", "USD", "2024-06-30").await;
    let batch_id: Uuid = batch["id"].as_str().unwrap().parse().unwrap();

    let receipt_id = Uuid::new_v4();
    add_receipt(&app, batch_id, &receipt_id.to_string(), "1000.00", "Customer A").await;

    // Try to add same receipt again
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "receipt_id": receipt_id.to_string(),
        "receipt_amount": "2000.00",
        "applied_amount": "2000.00",
        "currency_code": "USD",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/remittance-batches/{}/receipts", batch_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_add_receipt_negative_amount_fails() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "standard", "USD", "2024-06-30").await;
    let batch_id: Uuid = batch["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let receipt_id = Uuid::new_v4();
    let payload = json!({
        "receipt_id": receipt_id.to_string(),
        "receipt_amount": "-500.00",
        "applied_amount": "0.00",
        "currency_code": "USD",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/remittance-batches/{}/receipts", batch_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Remittance Advice Test
// ============================================================================

#[tokio::test]
async fn test_mark_advice_sent() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "standard", "USD", "2024-06-30").await;
    let batch_id: Uuid = batch["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Move batch to settled
    let receipt_id = Uuid::new_v4();
    add_receipt(&app, batch_id, &receipt_id.to_string(), "5000.00", "Acme").await;
    for action in &["approve", "format", "transmit", "confirm", "settle"] {
        let body_payload = if *action == "transmit" {
            Body::from(serde_json::to_string(&json!({})).unwrap())
        } else {
            Body::empty()
        };
        app.clone().oneshot(Request::builder().method("POST")
            .uri(&format!("/api/v1/remittance-batches/{}/{}", batch_id, action))
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(body_payload)
            .unwrap()
        ).await.unwrap();
    }

    // Send remittance advice
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/remittance-batches/{}/advice", batch_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["remittanceAdviceSent"], true);
    assert!(body["remittanceAdviceDate"].is_string());
}

#[tokio::test]
async fn test_advice_sent_on_draft_fails() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "standard", "USD", "2024-06-30").await;
    let batch_id: Uuid = batch["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/remittance-batches/{}/advice", batch_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Dashboard Test
// ============================================================================

#[tokio::test]
async fn test_batch_dashboard() {
    let (_state, app) = setup_test().await;
    create_batch(&app, "standard", "USD", "2024-06-15").await;
    create_batch(&app, "standard", "USD", "2024-06-30").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/remittance-batches/dashboard")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();

    assert!(body.get("totalBatches").is_some());
    assert!(body.get("draftCount").is_some());
    assert!(body.get("approvedCount").is_some());
    assert!(body.get("settledCount").is_some());
    assert!(body.get("totalAmount").is_some());
    assert!(body.get("totalReceipts").is_some());
    assert!(body.get("byStatus").is_some());
    assert!(body.get("byCurrency").is_some());

    assert!(body["totalBatches"].as_i64().unwrap() >= 2);
    assert!(body["draftCount"].as_i64().unwrap() >= 2);
}

// ============================================================================
// Validation Edge Cases
// ============================================================================

#[tokio::test]
async fn test_create_batch_invalid_method() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "remittance_method": "invalid_method",
        "batch_date": "2024-06-30",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/remittance-batches")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_batch_not_found() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/remittance-batches/{}", Uuid::new_v4()))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}
