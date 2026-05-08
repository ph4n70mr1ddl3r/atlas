//! Payment Process Request (PPR) E2E Tests
//!
//! Tests for Oracle Fusion Cloud ERP Payables > Payment Process Requests:
//! - Request CRUD
//! - Full lifecycle (draft → submitted → selection_complete → formatted → confirmed → cancelled)
//! - Document management with total recalculation
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
    // Clean PPR test data
    sqlx::query("DELETE FROM _atlas.ppr_activities").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM _atlas.ppr_selected_documents").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM _atlas.payment_process_requests").execute(&state.db_pool).await.ok();
    sqlx::query("CREATE SCHEMA IF NOT EXISTS _atlas")
        .execute(&state.db_pool)
        .await
        .ok();
    sqlx::raw_sql(include_str!("../../../../migrations/132_payment_process_request.sql"))
        .execute(&state.db_pool)
        .await
        .expect("Failed to run PPR migration");
    let app = build_router(state.clone());
    (state, app)
}

async fn create_ppr(
    app: &axum::Router,
    request_name: &str,
    payment_method: &str,
    selection_criteria: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "requestName": request_name,
        "description": "Test PPR",
        "currencyCode": "USD",
        "paymentDate": "2024-08-01",
        "glDate": "2024-08-01",
        "paymentMethod": payment_method,
        "selectionCriteria": selection_criteria,
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/payment-process-requests")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8_lossy(&b);
    eprintln!("CREATE PPR status={}: {}", status, body_str);
    assert_eq!(status, StatusCode::CREATED, "Failed to create PPR: {:?}", body_str);
    serde_json::from_slice(&b).unwrap()
}

async fn add_document(
    app: &axum::Router,
    ppr_id: Uuid,
    invoice_number: &str,
    amount_to_pay: f64,
    supplier_name: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "invoiceId": Uuid::new_v4().to_string(),
        "invoiceNumber": invoice_number,
        "invoiceDate": "2024-07-15",
        "invoiceAmount": amount_to_pay,
        "supplierId": Uuid::new_v4().to_string(),
        "supplierName": supplier_name,
        "originalAmount": amount_to_pay,
        "amountDue": amount_to_pay,
        "amountToPay": amount_to_pay,
        "discountAvailable": 0,
        "discountTaken": 0,
        "liabilityAccount": "2000",
        "discountAccount": "6500",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/payment-process-requests/{}/documents", ppr_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8_lossy(&b);
    eprintln!("ADD DOCUMENT status={}: {}", status, body_str);
    assert_eq!(status, StatusCode::CREATED, "Failed to add document: {:?}", body_str);
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// CRUD Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_create_ppr() {
    let (_state, app) = setup_test().await;
    let ppr = create_ppr(&app, "Weekly Payment Run", "electronic", "all_open").await;

    assert!(ppr["requestNumber"].as_str().unwrap().starts_with("PPR-"));
    assert_eq!(ppr["requestName"], "Weekly Payment Run");
    assert_eq!(ppr["paymentMethod"], "electronic");
    assert_eq!(ppr["selectionCriteria"], "all_open");
    assert_eq!(ppr["status"], "draft");
    assert_eq!(ppr["currencyCode"], "USD");
    assert_eq!(ppr["totalDocuments"], 0);
}

#[tokio::test]
#[ignore]
async fn test_create_ppr_with_due_date_criteria() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "requestName": "Due Date Run",
        "currencyCode": "EUR",
        "paymentDate": "2024-09-01",
        "glDate": "2024-09-01",
        "paymentMethod": "ach",
        "selectionCriteria": "due_date",
        "dueDateFrom": "2024-08-01",
        "dueDateTo": "2024-08-31",
        "takeDiscount": true,
        "payOnlyDue": true,
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/payment-process-requests")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["paymentMethod"], "ach");
    assert_eq!(body["selectionCriteria"], "due_date");
    assert_eq!(body["currencyCode"], "EUR");
    assert_eq!(body["takeDiscount"], true);
    assert_eq!(body["payOnlyDue"], true);
}

#[tokio::test]
#[ignore]
async fn test_get_ppr() {
    let (_state, app) = setup_test().await;
    let ppr = create_ppr(&app, "Test PPR", "check", "all_open").await;
    let ppr_id: Uuid = ppr["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/payment-process-requests/{}", ppr_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["id"], ppr["id"]);
    assert_eq!(body["requestName"], "Test PPR");
}

#[tokio::test]
#[ignore]
async fn test_get_ppr_by_number() {
    let (_state, app) = setup_test().await;
    let ppr = create_ppr(&app, "Number Test", "wire", "supplier").await;
    let number = ppr["requestNumber"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/payment-process-requests/number/{}", number))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["requestNumber"], number);
}

#[tokio::test]
#[ignore]
async fn test_list_pprs() {
    let (_state, app) = setup_test().await;
    create_ppr(&app, "PPR 1", "electronic", "all_open").await;
    create_ppr(&app, "PPR 2", "wire", "due_date").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/payment-process-requests")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(body["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
#[ignore]
async fn test_list_pprs_filter_by_status() {
    let (_state, app) = setup_test().await;
    let ppr = create_ppr(&app, "Filter Test", "electronic", "all_open").await;
    let ppr_id: Uuid = ppr["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Add a document so we can submit
    add_document(&app, ppr_id, "INV-001", 5000.0, "Supplier A").await;

    // Submit the PPR
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/payment-process-requests/{}/submit", ppr_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from("{}"))
        .unwrap()
    ).await.unwrap();

    // Filter by draft - should not include the submitted one
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/payment-process-requests?status=draft")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let data = body["data"].as_array().unwrap();
    assert!(data.iter().all(|b| b["status"] == "draft"));

    // Filter by submitted
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/payment-process-requests?status=submitted")
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
#[ignore]
async fn test_delete_draft_ppr() {
    let (_state, app) = setup_test().await;
    let ppr = create_ppr(&app, "Delete Me", "manual", "all_open").await;
    let number = ppr["requestNumber"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/payment-process-requests/number/{}", number))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Lifecycle Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_full_lifecycle_confirmed() {
    let (_state, app) = setup_test().await;
    let ppr = create_ppr(&app, "Lifecycle Test", "electronic", "all_open").await;
    let ppr_id: Uuid = ppr["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Add documents
    add_document(&app, ppr_id, "INV-001", 5000.0, "Acme Corp").await;
    add_document(&app, ppr_id, "INV-002", 3000.0, "Beta Inc").await;

    // Submit
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/payment-process-requests/{}/submit", ppr_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from("{}"))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "submitted");

    // Complete Selection
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/payment-process-requests/{}/complete-selection", ppr_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from("{}"))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "selection_complete");

    // Format
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/payment-process-requests/{}/format", ppr_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from("{}"))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "formatted");

    // Confirm
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/payment-process-requests/{}/confirm", ppr_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from("{}"))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "confirmed");
    assert!(body["confirmedAt"].is_string());
}

#[tokio::test]
#[ignore]
async fn test_cancel_from_draft() {
    let (_state, app) = setup_test().await;
    let ppr = create_ppr(&app, "Cancel Draft", "electronic", "all_open").await;
    let ppr_id: Uuid = ppr["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/payment-process-requests/{}/cancel", ppr_id))
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
#[ignore]
async fn test_cancel_from_submitted() {
    let (_state, app) = setup_test().await;
    let ppr = create_ppr(&app, "Cancel Submitted", "ach", "all_open").await;
    let ppr_id: Uuid = ppr["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    add_document(&app, ppr_id, "INV-010", 1000.0, "Test").await;

    // Submit first
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/payment-process-requests/{}/submit", ppr_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from("{}"))
        .unwrap()
    ).await.unwrap();

    // Cancel from submitted
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/payment-process-requests/{}/cancel", ppr_id))
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
#[ignore]
async fn test_cancel_from_selection_complete() {
    let (_state, app) = setup_test().await;
    let ppr = create_ppr(&app, "Cancel Selection", "check", "all_open").await;
    let ppr_id: Uuid = ppr["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    add_document(&app, ppr_id, "INV-020", 2000.0, "Test").await;

    // Submit and complete selection
    for endpoint in &["submit", "complete-selection"] {
        let uri = format!("/api/v1/payment-process-requests/{}/{}", ppr_id, endpoint);
        app.clone().oneshot(Request::builder().method("POST")
            .uri(&uri)
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from("{}"))
            .unwrap()
        ).await.unwrap();
    }

    // Cancel from selection_complete
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/payment-process-requests/{}/cancel", ppr_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"reason": "Error in selection"})).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "cancelled");
}

#[tokio::test]
#[ignore]
async fn test_invalid_transition_confirm_from_draft() {
    let (_state, app) = setup_test().await;
    let ppr = create_ppr(&app, "Invalid", "electronic", "all_open").await;
    let ppr_id: Uuid = ppr["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/payment-process-requests/{}/confirm", ppr_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from("{}"))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore]
async fn test_submit_without_documents_fails() {
    let (_state, app) = setup_test().await;
    let ppr = create_ppr(&app, "Empty PPR", "electronic", "all_open").await;
    let ppr_id: Uuid = ppr["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/payment-process-requests/{}/submit", ppr_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from("{}"))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore]
async fn test_cannot_confirm_cancelled() {
    let (_state, app) = setup_test().await;
    let ppr = create_ppr(&app, "Cancelled Confirm", "electronic", "all_open").await;
    let ppr_id: Uuid = ppr["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    // Cancel from draft
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/payment-process-requests/{}/cancel", ppr_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"reason": "test"})).unwrap()))
        .unwrap()
    ).await.unwrap();

    // Try to confirm cancelled
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/payment-process-requests/{}/confirm", ppr_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from("{}"))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore]
async fn test_delete_non_draft_fails() {
    let (_state, app) = setup_test().await;
    let ppr = create_ppr(&app, "No Delete", "electronic", "all_open").await;
    let ppr_id: Uuid = ppr["id"].as_str().unwrap().parse().unwrap();
    let number = ppr["requestNumber"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    add_document(&app, ppr_id, "INV-030", 1000.0, "Test").await;

    // Submit
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/payment-process-requests/{}/submit", ppr_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from("{}"))
        .unwrap()
    ).await.unwrap();

    // Try to delete submitted PPR
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/payment-process-requests/number/{}", number))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Document Management Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_add_documents_and_totals() {
    let (_state, app) = setup_test().await;
    let ppr = create_ppr(&app, "Docs Test", "electronic", "all_open").await;
    let ppr_id: Uuid = ppr["id"].as_str().unwrap().parse().unwrap();

    add_document(&app, ppr_id, "INV-100", 5000.0, "Acme Corp").await;
    add_document(&app, ppr_id, "INV-101", 3000.0, "Beta Inc").await;

    // Verify PPR totals
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/payment-process-requests/{}", ppr_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();

    assert_eq!(body["totalDocuments"], 2);
    let total_payment: f64 = body["totalPaymentAmount"].as_f64().unwrap();
    assert!((total_payment - 8000.0).abs() < 0.01, "Expected 8000, got {}", total_payment);
}

#[tokio::test]
#[ignore]
async fn test_remove_document_and_recalc() {
    let (_state, app) = setup_test().await;
    let ppr = create_ppr(&app, "Remove Doc", "electronic", "all_open").await;
    let ppr_id: Uuid = ppr["id"].as_str().unwrap().parse().unwrap();

    let doc1 = add_document(&app, ppr_id, "INV-200", 5000.0, "Acme Corp").await;
    add_document(&app, ppr_id, "INV-201", 3000.0, "Beta Inc").await;

    let doc1_id: Uuid = doc1["id"].as_str().unwrap().parse().unwrap();

    // Remove first document
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/payment-process-requests/{}/documents/{}", ppr_id, doc1_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);

    // Verify totals recalculated
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/payment-process-requests/{}", ppr_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();

    assert_eq!(body["totalDocuments"], 1);
    let total: f64 = body["totalPaymentAmount"].as_f64().unwrap();
    assert!((total - 3000.0).abs() < 0.01, "Expected 3000 after removal, got {}", total);
}

#[tokio::test]
#[ignore]
async fn test_list_documents() {
    let (_state, app) = setup_test().await;
    let ppr = create_ppr(&app, "List Docs", "electronic", "all_open").await;
    let ppr_id: Uuid = ppr["id"].as_str().unwrap().parse().unwrap();

    add_document(&app, ppr_id, "INV-300", 1000.0, "Supplier X").await;
    add_document(&app, ppr_id, "INV-301", 2000.0, "Supplier Y").await;
    add_document(&app, ppr_id, "INV-302", 3000.0, "Supplier Z").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/payment-process-requests/{}/documents", ppr_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["data"].as_array().unwrap().len(), 3);
}

#[tokio::test]
#[ignore]
async fn test_add_document_to_submitted_ppr_fails() {
    let (_state, app) = setup_test().await;
    let ppr = create_ppr(&app, "Submitted Add", "electronic", "all_open").await;
    let ppr_id: Uuid = ppr["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    add_document(&app, ppr_id, "INV-400", 1000.0, "Test").await;

    // Submit
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/payment-process-requests/{}/submit", ppr_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from("{}"))
        .unwrap()
    ).await.unwrap();

    // Try to add document to submitted PPR
    let payload = json!({
        "invoiceId": Uuid::new_v4().to_string(),
        "originalAmount": 500, "amountDue": 500, "amountToPay": 500,
        "invoiceAmount": 500,
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/payment-process-requests/{}/documents", ppr_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore]
async fn test_add_document_zero_amount_fails() {
    let (_state, app) = setup_test().await;
    let ppr = create_ppr(&app, "Zero Doc", "electronic", "all_open").await;
    let ppr_id: Uuid = ppr["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "invoiceId": Uuid::new_v4().to_string(),
        "originalAmount": 0, "amountDue": 0, "amountToPay": 0,
        "invoiceAmount": 0,
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/payment-process-requests/{}/documents", ppr_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore]
async fn test_add_document_exceeds_due_fails() {
    let (_state, app) = setup_test().await;
    let ppr = create_ppr(&app, "Exceed Due", "electronic", "all_open").await;
    let ppr_id: Uuid = ppr["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "invoiceId": Uuid::new_v4().to_string(),
        "originalAmount": 1000, "amountDue": 1000, "amountToPay": 2000,
        "invoiceAmount": 1000,
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/payment-process-requests/{}/documents", ppr_id))
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
#[ignore]
async fn test_activity_trail() {
    let (_state, app) = setup_test().await;
    let ppr = create_ppr(&app, "Activity Trail", "electronic", "all_open").await;
    let ppr_id: Uuid = ppr["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Add document, submit, complete selection, format, confirm
    add_document(&app, ppr_id, "INV-500", 5000.0, "Trail Corp").await;

    for endpoint in &["submit", "complete-selection", "format", "confirm"] {
        let uri = format!("/api/v1/payment-process-requests/{}/{}", ppr_id, endpoint);
        app.clone().oneshot(Request::builder().method("POST")
            .uri(&uri)
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from("{}"))
            .unwrap()
        ).await.unwrap();
    }

    // Check activities
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/payment-process-requests/{}/activities", ppr_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();

    let activities = body["data"].as_array().unwrap();
    // Should have: created + document_added + submitted + selection_complete + formatted + confirmed = 6+
    assert!(activities.len() >= 5, "Expected at least 5 activities, got {}", activities.len());

    // Verify created activity
    let created = activities.iter().find(|a| a["activityType"] == "created").unwrap();
    assert_eq!(created["newStatus"], "draft");

    // Verify submitted activity
    let submitted = activities.iter().find(|a| a["activityType"] == "submitted").unwrap();
    assert_eq!(submitted["oldStatus"], "draft");
    assert_eq!(submitted["newStatus"], "submitted");

    // Verify confirmed activity
    let confirmed = activities.iter().find(|a| a["activityType"] == "confirmed").unwrap();
    assert_eq!(confirmed["oldStatus"], "formatted");
    assert_eq!(confirmed["newStatus"], "confirmed");
}

// ============================================================================
// Dashboard Test
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_dashboard() {
    let (_state, app) = setup_test().await;
    create_ppr(&app, "Dash 1", "electronic", "all_open").await;
    create_ppr(&app, "Dash 2", "wire", "due_date").await;

    let ppr3 = create_ppr(&app, "Dash 3", "ach", "all_open").await;
    let ppr3_id: Uuid = ppr3["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Submit and process ppr3 through the full lifecycle
    add_document(&app, ppr3_id, "INV-600", 10000.0, "Big Corp").await;
    for endpoint in &["submit", "complete-selection", "format", "confirm"] {
        let uri = format!("/api/v1/payment-process-requests/{}/{}", ppr3_id, endpoint);
        app.clone().oneshot(Request::builder().method("POST")
            .uri(&uri)
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from("{}"))
            .unwrap()
        ).await.unwrap();
    }

    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/payment-process-requests/dashboard")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();

    assert!(body.get("totalRequests").is_some());
    assert!(body.get("draftCount").is_some());
    assert!(body.get("submittedCount").is_some());
    assert!(body.get("selectionCompleteCount").is_some());
    assert!(body.get("formattedCount").is_some());
    assert!(body.get("confirmedCount").is_some());
    assert!(body.get("cancelledCount").is_some());
    assert!(body.get("totalPaymentAmount").is_some());
    assert!(body.get("totalDocumentsProcessed").is_some());
    assert!(body.get("byPaymentMethod").is_some());
    assert!(body.get("bySelectionCriteria").is_some());

    assert!(body["totalRequests"].as_i64().unwrap() >= 3);
    assert!(body["draftCount"].as_i64().unwrap() >= 2);
    assert!(body["confirmedCount"].as_i64().unwrap() >= 1);
}

// ============================================================================
// Validation Edge Cases
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_create_ppr_invalid_payment_method() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "requestName": "Invalid Method",
        "paymentDate": "2024-08-01",
        "glDate": "2024-08-01",
        "paymentMethod": "crypto",
        "selectionCriteria": "all_open",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/payment-process-requests")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore]
async fn test_create_ppr_invalid_selection_criteria() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "requestName": "Invalid Criteria",
        "paymentDate": "2024-08-01",
        "glDate": "2024-08-01",
        "paymentMethod": "electronic",
        "selectionCriteria": "unknown",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/payment-process-requests")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore]
async fn test_create_ppr_due_date_criteria_without_range() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "requestName": "No Range",
        "paymentDate": "2024-08-01",
        "glDate": "2024-08-01",
        "paymentMethod": "electronic",
        "selectionCriteria": "due_date",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/payment-process-requests")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore]
async fn test_create_ppr_gl_date_after_payment_date() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "requestName": "Bad GL Date",
        "paymentDate": "2024-08-01",
        "glDate": "2024-08-15",
        "paymentMethod": "electronic",
        "selectionCriteria": "all_open",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/payment-process-requests")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore]
async fn test_get_ppr_not_found() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/payment-process-requests/{}", Uuid::new_v4()))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
#[ignore]
async fn test_invalid_status_filter() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/payment-process-requests?status=nonexistent")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Document with Discount Test
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_add_document_with_discount() {
    let (_state, app) = setup_test().await;
    let ppr = create_ppr(&app, "Discount Test", "electronic", "all_open").await;
    let ppr_id: Uuid = ppr["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "invoiceId": Uuid::new_v4().to_string(),
        "invoiceNumber": "INV-DISC",
        "invoiceDate": "2024-07-15",
        "invoiceAmount": 10000.0,
        "supplierId": Uuid::new_v4().to_string(),
        "supplierName": "Discount Supplier",
        "originalAmount": 10000.0,
        "amountDue": 10000.0,
        "amountToPay": 10000.0,
        "discountAvailable": 200.0,
        "discountTaken": 200.0,
        "discountDate": "2024-07-25",
        "liabilityAccount": "2000",
        "discountAccount": "6500",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/payment-process-requests/{}/documents", ppr_id))
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
    let net: f64 = body["netPayment"].as_f64().unwrap();
    assert!((net - 9800.0).abs() < 0.01, "Net should be 9800 (10000 - 200 discount), got {}", net);
}

#[tokio::test]
#[ignore]
async fn test_discount_exceeds_available_fails() {
    let (_state, app) = setup_test().await;
    let ppr = create_ppr(&app, "Excess Discount", "electronic", "all_open").await;
    let ppr_id: Uuid = ppr["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "invoiceId": Uuid::new_v4().to_string(),
        "originalAmount": 1000, "amountDue": 1000, "amountToPay": 1000,
        "invoiceAmount": 1000, "discountAvailable": 50, "discountTaken": 100,
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/payment-process-requests/{}/documents", ppr_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Confirm marks documents as paid
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_confirm_marks_documents_paid() {
    let (_state, app) = setup_test().await;
    let ppr = create_ppr(&app, "Docs Pay", "electronic", "all_open").await;
    let ppr_id: Uuid = ppr["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    add_document(&app, ppr_id, "INV-700", 5000.0, "Corp A").await;
    add_document(&app, ppr_id, "INV-701", 3000.0, "Corp B").await;

    // Full lifecycle
    for endpoint in &["submit", "complete-selection", "format", "confirm"] {
        let uri = format!("/api/v1/payment-process-requests/{}/{}", ppr_id, endpoint);
        app.clone().oneshot(Request::builder().method("POST")
            .uri(&uri)
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from("{}"))
            .unwrap()
        ).await.unwrap();
    }

    // Check documents are paid
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/payment-process-requests/{}/documents", ppr_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();

    let docs = body["data"].as_array().unwrap();
    assert!(docs.iter().all(|d| d["status"] == "paid"), "All documents should be paid after confirmation");
    assert_eq!(docs.len(), 2);
}
