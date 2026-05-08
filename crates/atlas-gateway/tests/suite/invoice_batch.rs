//! AP Invoice Batch Processing E2E Tests
//!
//! Tests for Oracle Fusion Cloud ERP Payables > Invoice Batches:
//! - Batch CRUD (create, get, list, delete)
//! - Full lifecycle (draft → submitted → approved → posted → cancelled)
//! - Invoice totals management (add/remove invoices with recalculation)
//! - Control total validation
//! - Activity audit trail
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
    // Clean invoice batch test data
    sqlx::query("DELETE FROM _atlas.ap_invoice_batch_activities").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM _atlas.ap_invoice_batches").execute(&state.db_pool).await.ok();
    sqlx::query("CREATE SCHEMA IF NOT EXISTS _atlas")
        .execute(&state.db_pool)
        .await
        .ok();
    sqlx::raw_sql(include_str!("../../../../migrations/133_ap_invoice_batch.sql"))
        .execute(&state.db_pool)
        .await
        .expect("Failed to run invoice batch migration");
    let app = build_router(state.clone());
    (state, app)
}

async fn create_batch(
    app: &axum::Router,
    batch_name: &str,
    source: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "batchName": batch_name,
        "description": "Test invoice batch",
        "currencyCode": "USD",
        "glDate": "2024-07-01",
        "accountingPeriod": "2024-07",
        "source": source,
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/invoice-batches")
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

async fn add_invoice(
    app: &axum::Router,
    batch_id: Uuid,
    invoice_amount: f64,
    tax_amount: f64,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "invoiceAmount": invoice_amount,
        "taxAmount": tax_amount,
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/invoice-batches/{}/invoices", batch_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8_lossy(&b);
    eprintln!("ADD INVOICE status={}: {}", status, body_str);
    assert_eq!(status, StatusCode::OK, "Failed to add invoice: {:?}", body_str);
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// CRUD Tests
// ============================================================================

#[tokio::test]
async fn test_create_batch() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "Weekly AP Batch", "manual").await;

    assert!(batch["batchNumber"].as_str().unwrap().starts_with("IB-"));
    assert_eq!(batch["batchName"], "Weekly AP Batch");
    assert_eq!(batch["source"], "manual");
    assert_eq!(batch["status"], "draft");
    assert_eq!(batch["currencyCode"], "USD");
    assert_eq!(batch["totalInvoiceCount"], 0);
    assert_eq!(batch["totalAmount"], 0.0);
}

#[tokio::test]
async fn test_create_batch_import_source() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "batchName": "EDI Import",
        "currencyCode": "EUR",
        "glDate": "2024-08-01",
        "source": "edi",
        "controlTotal": 50000.0,
        "controlCount": 10,
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/invoice-batches")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["source"], "edi");
    assert_eq!(body["currencyCode"], "EUR");
    assert_eq!(body["controlTotal"], 50000.0);
    assert_eq!(body["controlCount"], 10);
}

#[tokio::test]
async fn test_get_batch() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "Test Batch", "manual").await;
    let batch_id: Uuid = batch["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/invoice-batches/{}", batch_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["id"], batch["id"]);
    assert_eq!(body["batchName"], "Test Batch");
}

#[tokio::test]
async fn test_get_batch_by_number() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "Number Test", "import").await;
    let number = batch["batchNumber"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/invoice-batches/number/{}", number))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["batchNumber"], number);
}

#[tokio::test]
async fn test_list_batches() {
    let (_state, app) = setup_test().await;
    create_batch(&app, "Batch 1", "manual").await;
    create_batch(&app, "Batch 2", "import").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/invoice-batches")
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
    let batch = create_batch(&app, "Filter Test", "manual").await;
    let batch_id: Uuid = batch["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Add an invoice so we can submit
    add_invoice(&app, batch_id, 5000.0, 500.0).await;

    // Submit the batch
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/invoice-batches/{}/submit", batch_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({})).unwrap()))
        .unwrap()
    ).await.unwrap();

    // Filter by draft
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/invoice-batches?status=draft")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let data = body["data"].as_array().unwrap();
    assert!(data.iter().all(|b| b["status"] == "draft"));

    // Filter by submitted
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/invoice-batches?status=submitted")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let data = body["data"].as_array().unwrap();
    assert!(data.iter().all(|b| b["status"] == "submitted"));
    assert!(data.len() >= 1);
}

#[tokio::test]
async fn test_delete_draft_batch() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "Delete Me", "manual").await;
    let number = batch["batchNumber"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/invoice-batches/number/{}", number))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Lifecycle Tests
// ============================================================================

#[tokio::test]
async fn test_full_lifecycle_posted() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "Lifecycle Test", "manual").await;
    let batch_id: Uuid = batch["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Add invoices
    add_invoice(&app, batch_id, 5000.0, 500.0).await;
    add_invoice(&app, batch_id, 3000.0, 300.0).await;

    // Submit
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/invoice-batches/{}/submit", batch_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({})).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "submitted");

    // Approve
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/invoice-batches/{}/approve", batch_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({})).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "approved");

    // Post
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/invoice-batches/{}/post", batch_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({})).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "posted");
    assert!(body["postedAt"].is_string());
}

#[tokio::test]
async fn test_cancel_from_draft() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "Cancel Draft", "manual").await;
    let batch_id: Uuid = batch["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/invoice-batches/{}/cancel", batch_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "reason": "No longer needed"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "cancelled");
    assert_eq!(body["cancelReason"], "No longer needed");
}

#[tokio::test]
async fn test_cancel_from_submitted() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "Cancel Submitted", "manual").await;
    let batch_id: Uuid = batch["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    add_invoice(&app, batch_id, 1000.0, 100.0).await;

    // Submit first
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/invoice-batches/{}/submit", batch_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({})).unwrap()))
        .unwrap()
    ).await.unwrap();

    // Cancel from submitted
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/invoice-batches/{}/cancel", batch_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"reason": "Budget cut"})).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "cancelled");
}

#[tokio::test]
async fn test_cancel_from_approved() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "Cancel Approved", "manual").await;
    let batch_id: Uuid = batch["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    add_invoice(&app, batch_id, 2000.0, 200.0).await;

    // Submit and approve
    for endpoint in &["submit", "approve"] {
        app.clone().oneshot(Request::builder().method("POST")
            .uri(&format!("/api/v1/invoice-batches/{}/{}", batch_id, endpoint))
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({})).unwrap()))
            .unwrap()
        ).await.unwrap();
    }

    // Cancel from approved
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/invoice-batches/{}/cancel", batch_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"reason": "Error found"})).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "cancelled");
}

#[tokio::test]
async fn test_invalid_transition_post_from_draft() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "Invalid", "manual").await;
    let batch_id: Uuid = batch["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/invoice-batches/{}/post", batch_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({})).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_submit_without_invoices_fails() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "Empty Batch", "manual").await;
    let batch_id: Uuid = batch["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/invoice-batches/{}/submit", batch_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({})).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_cannot_post_cancelled() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "Post Cancelled", "manual").await;
    let batch_id: Uuid = batch["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    // Cancel from draft
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/invoice-batches/{}/cancel", batch_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"reason": "test"})).unwrap()))
        .unwrap()
    ).await.unwrap();

    // Try to post cancelled
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/invoice-batches/{}/post", batch_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({})).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_delete_non_draft_fails() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "No Delete", "manual").await;
    let batch_id: Uuid = batch["id"].as_str().unwrap().parse().unwrap();
    let number = batch["batchNumber"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    add_invoice(&app, batch_id, 1000.0, 100.0).await;

    // Submit
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/invoice-batches/{}/submit", batch_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({})).unwrap()))
        .unwrap()
    ).await.unwrap();

    // Try to delete submitted batch
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/invoice-batches/number/{}", number))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Invoice Totals Tests
// ============================================================================

#[tokio::test]
async fn test_add_invoices_and_totals() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "Totals Test", "manual").await;
    let batch_id: Uuid = batch["id"].as_str().unwrap().parse().unwrap();

    add_invoice(&app, batch_id, 5000.0, 500.0).await;
    add_invoice(&app, batch_id, 3000.0, 300.0).await;

    // Verify batch totals
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/invoice-batches/{}", batch_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();

    assert_eq!(body["totalInvoiceCount"], 2);
    let total_inv: f64 = body["totalInvoiceAmount"].as_f64().unwrap();
    assert!((total_inv - 8000.0).abs() < 0.01, "Expected 8000, got {}", total_inv);
    let total_tax: f64 = body["totalTaxAmount"].as_f64().unwrap();
    assert!((total_tax - 800.0).abs() < 0.01, "Expected 800, got {}", total_tax);
    let total: f64 = body["totalAmount"].as_f64().unwrap();
    assert!((total - 8800.0).abs() < 0.01, "Expected 8800, got {}", total);
}

#[tokio::test]
async fn test_remove_invoice_and_recalc() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "Remove Test", "manual").await;
    let batch_id: Uuid = batch["id"].as_str().unwrap().parse().unwrap();

    add_invoice(&app, batch_id, 5000.0, 500.0).await;
    add_invoice(&app, batch_id, 3000.0, 300.0).await;

    // Remove first invoice via DELETE with body
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "invoiceAmount": 5000.0,
        "taxAmount": 500.0,
    });
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/invoice-batches/{}/invoices", batch_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    // Verify totals recalculated
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/invoice-batches/{}", batch_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();

    assert_eq!(body["totalInvoiceCount"], 1);
    let total: f64 = body["totalAmount"].as_f64().unwrap();
    assert!((total - 3300.0).abs() < 0.01, "Expected 3300 after removal, got {}", total);
}

#[tokio::test]
async fn test_add_invoice_negative_amount_fails() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "Neg Amount", "manual").await;
    let batch_id: Uuid = batch["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "invoiceAmount": -500.0,
        "taxAmount": 50.0,
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/invoice-batches/{}/invoices", batch_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Control Total Validation Tests
// ============================================================================

#[tokio::test]
async fn test_control_total_mismatch_fails_submit() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Create batch with control total of 10000
    let payload = json!({
        "batchName": "Control Total Test",
        "currencyCode": "USD",
        "glDate": "2024-07-01",
        "source": "manual",
        "controlTotal": 10000.0,
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/invoice-batches")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    let batch: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let batch_id: Uuid = batch["id"].as_str().unwrap().parse().unwrap();

    // Add invoices totaling 8800 (not matching 10000)
    add_invoice(&app, batch_id, 5000.0, 500.0).await;
    add_invoice(&app, batch_id, 3000.0, 300.0).await;

    // Submit should fail due to control total mismatch
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/invoice-batches/{}/submit", batch_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({})).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(body["error"].as_str().unwrap().contains("control total"));
}

#[tokio::test]
async fn test_control_count_mismatch_fails_submit() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Create batch with control count of 5
    let payload = json!({
        "batchName": "Control Count Test",
        "currencyCode": "USD",
        "source": "manual",
        "controlCount": 5,
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/invoice-batches")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    let batch: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let batch_id: Uuid = batch["id"].as_str().unwrap().parse().unwrap();

    // Add only 2 invoices (not matching 5)
    add_invoice(&app, batch_id, 1000.0, 100.0).await;
    add_invoice(&app, batch_id, 2000.0, 200.0).await;

    // Submit should fail due to control count mismatch
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/invoice-batches/{}/submit", batch_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({})).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Activity Trail Tests
// ============================================================================

#[tokio::test]
async fn test_activity_trail() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "Activity Trail", "manual").await;
    let batch_id: Uuid = batch["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Add invoice, submit, approve, post
    add_invoice(&app, batch_id, 5000.0, 500.0).await;

    for endpoint in &["submit", "approve", "post"] {
        app.clone().oneshot(Request::builder().method("POST")
            .uri(&format!("/api/v1/invoice-batches/{}/{}", batch_id, endpoint))
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({})).unwrap()))
            .unwrap()
        ).await.unwrap();
    }

    // Check activities
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/invoice-batches/{}/activities", batch_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();

    let activities = body["data"].as_array().unwrap();
    // Should have: created + invoice_added + submitted + approved + posted = 5
    assert!(activities.len() >= 4, "Expected at least 4 activities, got {}", activities.len());

    // Verify created activity
    let created = activities.iter().find(|a| a["activityType"] == "created").unwrap();
    assert_eq!(created["newStatus"], "draft");

    // Verify submitted activity
    let submitted = activities.iter().find(|a| a["activityType"] == "submitted").unwrap();
    assert_eq!(submitted["oldStatus"], "draft");
    assert_eq!(submitted["newStatus"], "submitted");

    // Verify posted activity
    let posted = activities.iter().find(|a| a["activityType"] == "posted").unwrap();
    assert_eq!(posted["oldStatus"], "approved");
    assert_eq!(posted["newStatus"], "posted");
}

// ============================================================================
// Dashboard Test
// ============================================================================

#[tokio::test]
async fn test_dashboard() {
    let (_state, app) = setup_test().await;
    create_batch(&app, "Dash 1", "manual").await;
    create_batch(&app, "Dash 2", "import").await;

    let batch3 = create_batch(&app, "Dash 3", "manual").await;
    let batch3_id: Uuid = batch3["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Submit and approve batch3
    add_invoice(&app, batch3_id, 10000.0, 1000.0).await;
    for endpoint in &["submit", "approve"] {
        app.clone().oneshot(Request::builder().method("POST")
            .uri(&format!("/api/v1/invoice-batches/{}/{}", batch3_id, endpoint))
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({})).unwrap()))
            .unwrap()
        ).await.unwrap();
    }

    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/invoice-batches/dashboard")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();

    assert!(body.get("totalBatches").is_some());
    assert!(body.get("draftCount").is_some());
    assert!(body.get("submittedCount").is_some());
    assert!(body.get("approvedCount").is_some());
    assert!(body.get("postedCount").is_some());
    assert!(body.get("cancelledCount").is_some());
    assert!(body.get("totalInvoiceAmount").is_some());
    assert!(body.get("totalInvoices").is_some());
    assert!(body.get("bySource").is_some());

    assert!(body["totalBatches"].as_i64().unwrap() >= 3);
    assert!(body["draftCount"].as_i64().unwrap() >= 2);
    assert!(body["approvedCount"].as_i64().unwrap() >= 1);
}

// ============================================================================
// Validation Edge Cases
// ============================================================================

#[tokio::test]
async fn test_create_batch_invalid_source() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "batchName": "Invalid Source",
        "source": "unknown",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/invoice-batches")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_batch_empty_name_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "batchName": "",
        "source": "manual",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/invoice-batches")
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
        .uri(&format!("/api/v1/invoice-batches/{}", Uuid::new_v4()))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_invalid_status_filter() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/invoice-batches?status=nonexistent")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_validate_batch() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Create batch with matching control total
    let payload = json!({
        "batchName": "Validate Test",
        "currencyCode": "USD",
        "source": "manual",
        "controlTotal": 5500.0,
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/invoice-batches")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    let batch: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let batch_id: Uuid = batch["id"].as_str().unwrap().parse().unwrap();

    // Add invoice that makes total = 5000 + 500 = 5500
    add_invoice(&app, batch_id, 5000.0, 500.0).await;

    // Validate
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/invoice-batches/{}/validate", batch_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["validationStatus"], "valid");
}

// ============================================================================
// Lifecycle with control totals matching
// ============================================================================

#[tokio::test]
async fn test_full_lifecycle_with_control_totals() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Create batch with control total and count
    let payload = json!({
        "batchName": "Controlled Batch",
        "currencyCode": "USD",
        "glDate": "2024-07-01",
        "source": "import",
        "controlTotal": 11000.0,
        "controlCount": 2,
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/invoice-batches")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    let batch: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let batch_id: Uuid = batch["id"].as_str().unwrap().parse().unwrap();

    // Add 2 invoices totaling 11000 (5000+500 + 5000+500 = 11000)
    add_invoice(&app, batch_id, 5000.0, 500.0).await;
    add_invoice(&app, batch_id, 5000.0, 500.0).await;

    // Full lifecycle should succeed since control totals match
    for endpoint in &["submit", "approve", "post"] {
        let r = app.clone().oneshot(Request::builder().method("POST")
            .uri(&format!("/api/v1/invoice-batches/{}/{}", batch_id, endpoint))
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({})).unwrap()))
            .unwrap()
        ).await.unwrap();
        let status = r.status();
        let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
            .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
        eprintln!("{} -> {}", endpoint, body["status"]);
        assert_eq!(status, StatusCode::OK, "Failed at {} step", endpoint);
    }

    // Verify final state
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/invoice-batches/{}", batch_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "posted");
    assert_eq!(body["totalInvoiceCount"], 2);
    let total: f64 = body["totalAmount"].as_f64().unwrap();
    assert!((total - 11000.0).abs() < 0.01);
}
