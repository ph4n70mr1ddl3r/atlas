//! Payment Settlement & Clearing E2E Tests
//!
//! Tests for Oracle Fusion Cloud ERP Payables > Settlement:
//! - Batch CRUD
//! - Full lifecycle (draft → submitted → approved → settled → cancelled)
//! - Settlement line management with total recalculation
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
    // Clean settlement test data
    sqlx::query("DELETE FROM _atlas.settlement_activities").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM _atlas.settlement_lines").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM _atlas.settlement_batches").execute(&state.db_pool).await.ok();
    sqlx::query("CREATE SCHEMA IF NOT EXISTS _atlas")
        .execute(&state.db_pool)
        .await
        .ok();
    sqlx::raw_sql(include_str!("../../../../migrations/131_payment_settlement.sql"))
        .execute(&state.db_pool)
        .await
        .expect("Failed to run payment settlement migration");
    let app = build_router(state.clone());
    (state, app)
}

async fn create_batch(
    app: &axum::Router,
    batch_name: &str,
    settlement_method: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "batchName": batch_name,
        "description": "Test settlement batch",
        "currencyCode": "USD",
        "settlementDate": "2024-07-01",
        "glDate": "2024-07-01",
        "settlementMethod": settlement_method,
        "settlementType": "full",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/settlements")
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

async fn add_line(
    app: &axum::Router,
    batch_id: Uuid,
    invoice_number: &str,
    amount_paid: f64,
    supplier_name: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "invoiceId": Uuid::new_v4().to_string(),
        "invoiceNumber": invoice_number,
        "invoiceDate": "2024-06-15",
        "invoiceAmount": amount_paid,
        "supplierId": Uuid::new_v4().to_string(),
        "supplierName": supplier_name,
        "originalAmount": amount_paid,
        "amountDue": amount_paid,
        "amountPaid": amount_paid,
        "discountAvailable": 0,
        "discountTaken": 0,
        "settlementType": "full",
        "liabilityAccount": "2000",
        "discountAccount": "6500",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/settlements/{}/lines", batch_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8_lossy(&b);
    eprintln!("ADD LINE status={}: {}", status, body_str);
    assert_eq!(status, StatusCode::CREATED, "Failed to add line: {:?}", body_str);
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// CRUD Tests
// ============================================================================

#[tokio::test]
async fn test_create_batch() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "Weekly Settlement", "electronic").await;

    assert!(batch["batchNumber"].as_str().unwrap().starts_with("STL-"));
    assert_eq!(batch["batchName"], "Weekly Settlement");
    assert_eq!(batch["settlementMethod"], "electronic");
    assert_eq!(batch["settlementType"], "full");
    assert_eq!(batch["status"], "draft");
    assert_eq!(batch["currencyCode"], "USD");
    assert_eq!(batch["totalInvoices"], 0);
}

#[tokio::test]
async fn test_create_batch_wire_method() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "batchName": "Wire Settlement",
        "currencyCode": "EUR",
        "settlementDate": "2024-08-01",
        "glDate": "2024-08-01",
        "settlementMethod": "wire",
        "settlementType": "partial",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/settlements")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["settlementMethod"], "wire");
    assert_eq!(body["settlementType"], "partial");
    assert_eq!(body["currencyCode"], "EUR");
}

#[tokio::test]
async fn test_get_batch() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "Test Batch", "ach").await;
    let batch_id: Uuid = batch["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/settlements/{}", batch_id))
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
    let batch = create_batch(&app, "Number Test", "check").await;
    let number = batch["batchNumber"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/settlements/number/{}", number))
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
    create_batch(&app, "Batch 1", "electronic").await;
    create_batch(&app, "Batch 2", "wire").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/settlements")
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
    let batch = create_batch(&app, "Filter Test", "electronic").await;
    let batch_id: Uuid = batch["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Add a line so we can submit
    add_line(&app, batch_id, "INV-001", 5000.0, "Supplier A").await;

    // Submit the batch
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/settlements/{}/submit", batch_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({})).unwrap()))
        .unwrap()
    ).await.unwrap();

    // Filter by draft - should not include the submitted one
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/settlements?status=draft")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let data = body["data"].as_array().unwrap();
    assert!(data.iter().all(|b| b["status"] == "draft"));

    // Filter by submitted
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/settlements?status=submitted")
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
        .uri(&format!("/api/v1/settlements/number/{}", number))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Lifecycle Tests
// ============================================================================

#[tokio::test]
async fn test_full_lifecycle_settled() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "Lifecycle Test", "electronic").await;
    let batch_id: Uuid = batch["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Add lines
    add_line(&app, batch_id, "INV-001", 5000.0, "Acme Corp").await;
    add_line(&app, batch_id, "INV-002", 3000.0, "Beta Inc").await;

    // Submit
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/settlements/{}/submit", batch_id))
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
        .uri(&format!("/api/v1/settlements/{}/approve", batch_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({})).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "approved");

    // Settle
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/settlements/{}/settle", batch_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({})).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "settled");
    assert!(body["settledAt"].is_string());
}

#[tokio::test]
async fn test_cancel_from_draft() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "Cancel Draft", "electronic").await;
    let batch_id: Uuid = batch["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/settlements/{}/cancel", batch_id))
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
    let batch = create_batch(&app, "Cancel Submitted", "ach").await;
    let batch_id: Uuid = batch["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    add_line(&app, batch_id, "INV-010", 1000.0, "Test").await;

    // Submit first
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/settlements/{}/submit", batch_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({})).unwrap()))
        .unwrap()
    ).await.unwrap();

    // Cancel from submitted
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/settlements/{}/cancel", batch_id))
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
    let batch = create_batch(&app, "Cancel Approved", "check").await;
    let batch_id: Uuid = batch["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    add_line(&app, batch_id, "INV-020", 2000.0, "Test").await;

    // Submit and approve
    for endpoint in &["submit", "approve"] {
        app.clone().oneshot(Request::builder().method("POST")
            .uri(&format!("/api/v1/settlements/{}/{}", batch_id, endpoint))
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({})).unwrap()))
            .unwrap()
        ).await.unwrap();
    }

    // Cancel from approved
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/settlements/{}/cancel", batch_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"reason": "Payment error"})).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "cancelled");
}

#[tokio::test]
async fn test_invalid_transition_settle_from_draft() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "Invalid", "electronic").await;
    let batch_id: Uuid = batch["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/settlements/{}/settle", batch_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({})).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_submit_without_lines_fails() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "Empty Batch", "electronic").await;
    let batch_id: Uuid = batch["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/settlements/{}/submit", batch_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({})).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_cannot_settle_cancelled() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "Settled Cancel", "electronic").await;
    let batch_id: Uuid = batch["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    // Cancel from draft
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/settlements/{}/cancel", batch_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"reason": "test"})).unwrap()))
        .unwrap()
    ).await.unwrap();

    // Try to settle cancelled
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/settlements/{}/settle", batch_id))
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
    let batch = create_batch(&app, "No Delete", "electronic").await;
    let batch_id: Uuid = batch["id"].as_str().unwrap().parse().unwrap();
    let number = batch["batchNumber"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    add_line(&app, batch_id, "INV-030", 1000.0, "Test").await;

    // Submit
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/settlements/{}/submit", batch_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({})).unwrap()))
        .unwrap()
    ).await.unwrap();

    // Try to delete submitted batch
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/settlements/number/{}", number))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Line Management Tests
// ============================================================================

#[tokio::test]
async fn test_add_lines_and_totals() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "Lines Test", "electronic").await;
    let batch_id: Uuid = batch["id"].as_str().unwrap().parse().unwrap();

    add_line(&app, batch_id, "INV-100", 5000.0, "Acme Corp").await;
    add_line(&app, batch_id, "INV-101", 3000.0, "Beta Inc").await;

    // Verify batch totals
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/settlements/{}", batch_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();

    assert_eq!(body["totalInvoices"], 2);
    let total_settled: f64 = body["totalSettledAmount"].as_f64().unwrap();
    assert!((total_settled - 8000.0).abs() < 0.01, "Expected 8000, got {}", total_settled);
}

#[tokio::test]
async fn test_remove_line_and_recalc() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "Remove Line", "electronic").await;
    let batch_id: Uuid = batch["id"].as_str().unwrap().parse().unwrap();

    let line1 = add_line(&app, batch_id, "INV-200", 5000.0, "Acme Corp").await;
    add_line(&app, batch_id, "INV-201", 3000.0, "Beta Inc").await;

    let line1_id: Uuid = line1["id"].as_str().unwrap().parse().unwrap();

    // Remove first line
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/settlements/{}/lines/{}", batch_id, line1_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);

    // Verify totals recalculated
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/settlements/{}", batch_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();

    assert_eq!(body["totalInvoices"], 1);
    let total: f64 = body["totalSettledAmount"].as_f64().unwrap();
    assert!((total - 3000.0).abs() < 0.01, "Expected 3000 after removal, got {}", total);
}

#[tokio::test]
async fn test_list_lines() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "List Lines", "electronic").await;
    let batch_id: Uuid = batch["id"].as_str().unwrap().parse().unwrap();

    add_line(&app, batch_id, "INV-300", 1000.0, "Supplier X").await;
    add_line(&app, batch_id, "INV-301", 2000.0, "Supplier Y").await;
    add_line(&app, batch_id, "INV-302", 3000.0, "Supplier Z").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/settlements/{}/lines", batch_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["data"].as_array().unwrap().len(), 3);
}

#[tokio::test]
async fn test_add_line_to_submitted_batch_fails() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "Submitted Add", "electronic").await;
    let batch_id: Uuid = batch["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    add_line(&app, batch_id, "INV-400", 1000.0, "Test").await;

    // Submit
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/settlements/{}/submit", batch_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({})).unwrap()))
        .unwrap()
    ).await.unwrap();

    // Try to add line to submitted batch
    let payload = json!({
        "invoiceId": Uuid::new_v4().to_string(),
        "originalAmount": 500, "amountDue": 500, "amountPaid": 500,
        "invoiceAmount": 500, "settlementType": "full"
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/settlements/{}/lines", batch_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_add_line_zero_amount_fails() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "Zero Line", "electronic").await;
    let batch_id: Uuid = batch["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "invoiceId": Uuid::new_v4().to_string(),
        "originalAmount": 0, "amountDue": 0, "amountPaid": 0,
        "invoiceAmount": 0, "settlementType": "full"
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/settlements/{}/lines", batch_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_add_line_exceeds_due_fails() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "Exceed Due", "electronic").await;
    let batch_id: Uuid = batch["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "invoiceId": Uuid::new_v4().to_string(),
        "originalAmount": 1000, "amountDue": 1000, "amountPaid": 2000,
        "invoiceAmount": 1000, "settlementType": "full"
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/settlements/{}/lines", batch_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
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
    let batch = create_batch(&app, "Activity Trail", "electronic").await;
    let batch_id: Uuid = batch["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Add line, submit, approve, settle
    add_line(&app, batch_id, "INV-500", 5000.0, "Trail Corp").await;

    for endpoint in &["submit", "approve", "settle"] {
        app.clone().oneshot(Request::builder().method("POST")
            .uri(&format!("/api/v1/settlements/{}/{}", batch_id, endpoint))
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({})).unwrap()))
            .unwrap()
        ).await.unwrap();
    }

    // Check activities
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/settlements/{}/activities", batch_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();

    let activities = body["data"].as_array().unwrap();
    // Should have: created + line_added + submitted + approved + line_settled + settled = 6+
    assert!(activities.len() >= 4, "Expected at least 4 activities, got {}", activities.len());

    // Verify created activity
    let created = activities.iter().find(|a| a["activityType"] == "created").unwrap();
    assert_eq!(created["newStatus"], "draft");

    // Verify submitted activity
    let submitted = activities.iter().find(|a| a["activityType"] == "submitted").unwrap();
    assert_eq!(submitted["oldStatus"], "draft");
    assert_eq!(submitted["newStatus"], "submitted");
}

// ============================================================================
// Dashboard Test
// ============================================================================

#[tokio::test]
async fn test_dashboard() {
    let (_state, app) = setup_test().await;
    create_batch(&app, "Dash 1", "electronic").await;
    create_batch(&app, "Dash 2", "wire").await;

    let batch3 = create_batch(&app, "Dash 3", "ach").await;
    let batch3_id: Uuid = batch3["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Submit and approve batch3
    add_line(&app, batch3_id, "INV-600", 10000.0, "Big Corp").await;
    for endpoint in &["submit", "approve"] {
        app.clone().oneshot(Request::builder().method("POST")
            .uri(&format!("/api/v1/settlements/{}/{}", batch3_id, endpoint))
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({})).unwrap()))
            .unwrap()
        ).await.unwrap();
    }

    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/settlements/dashboard")
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
    assert!(body.get("settledCount").is_some());
    assert!(body.get("cancelledCount").is_some());
    assert!(body.get("totalSettledAmount").is_some());
    assert!(body.get("totalDiscountTaken").is_some());
    assert!(body.get("totalInvoicesSettled").is_some());
    assert!(body.get("bySettlementMethod").is_some());
    assert!(body.get("bySettlementType").is_some());

    assert!(body["totalBatches"].as_i64().unwrap() >= 3);
    assert!(body["draftCount"].as_i64().unwrap() >= 2);
    assert!(body["approvedCount"].as_i64().unwrap() >= 1);
}

// ============================================================================
// Validation Edge Cases
// ============================================================================

#[tokio::test]
async fn test_create_batch_invalid_method() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "batchName": "Invalid Method",
        "settlementDate": "2024-07-01",
        "glDate": "2024-07-01",
        "settlementMethod": "crypto",
        "settlementType": "full",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/settlements")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_batch_invalid_settlement_type() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "batchName": "Invalid Type",
        "settlementDate": "2024-07-01",
        "glDate": "2024-07-01",
        "settlementMethod": "electronic",
        "settlementType": "unknown",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/settlements")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_batch_gl_date_before_settlement_date() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "batchName": "Bad GL Date",
        "settlementDate": "2024-07-15",
        "glDate": "2024-07-01",
        "settlementMethod": "electronic",
        "settlementType": "full",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/settlements")
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
        .uri(&format!("/api/v1/settlements/{}", Uuid::new_v4()))
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
        .uri("/api/v1/settlements?status=nonexistent")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Line with Discount Test
// ============================================================================

#[tokio::test]
async fn test_add_line_with_discount() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "Discount Test", "electronic").await;
    let batch_id: Uuid = batch["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "invoiceId": Uuid::new_v4().to_string(),
        "invoiceNumber": "INV-DISC",
        "invoiceDate": "2024-06-15",
        "invoiceAmount": 10000.0,
        "supplierId": Uuid::new_v4().to_string(),
        "supplierName": "Discount Supplier",
        "originalAmount": 10000.0,
        "amountDue": 10000.0,
        "amountPaid": 10000.0,
        "discountAvailable": 200.0,
        "discountTaken": 200.0,
        "discountDate": "2024-06-25",
        "settlementType": "full",
        "liabilityAccount": "2000",
        "discountAccount": "6500",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/settlements/{}/lines", batch_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let discount: f64 = body["discountTaken"].as_f64().unwrap();
    assert!((discount - 200.0).abs() < 0.01);
    let net: f64 = body["netSettlement"].as_f64().unwrap();
    assert!((net - 9800.0).abs() < 0.01, "Net should be 9800 (10000 - 200 discount), got {}", net);
}

#[tokio::test]
async fn test_discount_exceeds_available_fails() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "Excess Discount", "electronic").await;
    let batch_id: Uuid = batch["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "invoiceId": Uuid::new_v4().to_string(),
        "originalAmount": 1000, "amountDue": 1000, "amountPaid": 1000,
        "invoiceAmount": 1000, "discountAvailable": 50, "discountTaken": 100,
        "settlementType": "full"
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/settlements/{}/lines", batch_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Settling marks lines as settled
// ============================================================================

#[tokio::test]
async fn test_settle_marks_lines_settled() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "Lines Settle", "electronic").await;
    let batch_id: Uuid = batch["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    add_line(&app, batch_id, "INV-700", 5000.0, "Corp A").await;
    add_line(&app, batch_id, "INV-701", 3000.0, "Corp B").await;

    // Full lifecycle
    for endpoint in &["submit", "approve", "settle"] {
        app.clone().oneshot(Request::builder().method("POST")
            .uri(&format!("/api/v1/settlements/{}/{}", batch_id, endpoint))
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({})).unwrap()))
            .unwrap()
        ).await.unwrap();
    }

    // Check lines are settled
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/settlements/{}/lines", batch_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();

    let lines = body["data"].as_array().unwrap();
    assert!(lines.iter().all(|l| l["status"] == "settled"), "All lines should be settled");
    assert_eq!(lines.len(), 2);
}
