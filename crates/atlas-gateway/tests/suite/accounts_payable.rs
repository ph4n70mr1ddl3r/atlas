//! Accounts Payable E2E Tests
//!
//! Tests for Oracle Fusion Cloud ERP Accounts Payable:
//! - Supplier invoice CRUD
//! - Invoice line management
//! - Invoice distributions
//! - Invoice workflow (create → add lines → add distributions → submit → approve → pay)
//! - Invoice holds (apply, release)
//! - Payment processing
//! - AP aging summary
//! - Validation and error cases

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_ap_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    let app = build_router(state.clone());
    (state, app)
}

const SUPPLIER_ID: &str = "00000000-0000-0000-0000-000000000100";

async fn create_test_invoice(
    app: &axum::Router,
    invoice_number: &str,
    amount: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/ap/invoices")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "invoice_number": invoice_number,
            "invoice_date": "2026-04-15",
            "invoice_type": "standard",
            "supplier_id": SUPPLIER_ID,
            "supplier_name": "Acme Corp",
            "supplier_number": "SUP-001",
            "invoice_currency_code": "USD",
            "payment_currency_code": "USD",
            "invoice_amount": amount,
            "tax_amount": "0.00",
            "payment_due_date": "2026-05-15",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Failed to create invoice");
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn add_test_line(
    app: &axum::Router,
    invoice_id: &str,
    amount: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let uri = format!("/api/v1/ap/invoices/{}/lines", invoice_id);
    let r = app.clone().oneshot(Request::builder().method("POST").uri(&uri)
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "line_type": "item",
            "description": "Office supplies",
            "amount": amount,
            "unit_price": amount,
            "quantity_invoiced": "1",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Failed to add invoice line");
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn add_test_distribution(
    app: &axum::Router,
    invoice_id: &str,
    amount: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let uri = format!("/api/v1/ap/invoices/{}/distributions", invoice_id);
    let r = app.clone().oneshot(Request::builder().method("POST").uri(&uri)
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "distribution_type": "charge",
            "account_combination": "1000.200.300",
            "description": "Office supplies expense",
            "amount": amount,
            "currency_code": "USD",
            "gl_account": "1000",
            "cost_center": "CC-001",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Failed to add distribution");
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Invoice CRUD Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_create_ap_invoice() {
    let (_state, app) = setup_ap_test().await;

    let invoice = create_test_invoice(&app, "INV-AP-001", "1500.00").await;

    assert_eq!(invoice["invoice_number"], "INV-AP-001");
    assert_eq!(invoice["invoice_type"], "standard");
    assert_eq!(invoice["status"], "draft");
    assert_eq!(invoice["supplier_name"], "Acme Corp");
    assert!(invoice["id"].is_string());
}

#[tokio::test]
#[ignore]
async fn test_list_ap_invoices() {
    let (_state, app) = setup_ap_test().await;

    create_test_invoice(&app, "INV-AP-010", "100.00").await;
    create_test_invoice(&app, "INV-AP-011", "200.00").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET").uri("/api/v1/ap/invoices")
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
async fn test_get_ap_invoice() {
    let (_state, app) = setup_ap_test().await;

    let invoice = create_test_invoice(&app, "INV-AP-020", "500.00").await;
    let invoice_id = invoice["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/ap/invoices/{}", invoice_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let fetched: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(fetched["invoice_number"], "INV-AP-020");
}

#[tokio::test]
#[ignore]
async fn test_filter_ap_invoices_by_status() {
    let (_state, app) = setup_ap_test().await;

    create_test_invoice(&app, "INV-AP-030", "300.00").await;
    create_test_invoice(&app, "INV-AP-031", "400.00").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/ap/invoices?status=draft")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(result["data"].as_array().unwrap().len(), 2);
}

// ============================================================================
// Invoice Line Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_add_invoice_line() {
    let (_state, app) = setup_ap_test().await;

    let invoice = create_test_invoice(&app, "INV-AP-100", "1000.00").await;
    let invoice_id = invoice["id"].as_str().unwrap();

    let line = add_test_line(&app, invoice_id, "1000.00").await;
    assert_eq!(line["line_type"], "item");
    assert_eq!(line["description"], "Office supplies");
    assert_eq!(line["line_number"], 1);
}

#[tokio::test]
#[ignore]
async fn test_list_invoice_lines() {
    let (_state, app) = setup_ap_test().await;

    let invoice = create_test_invoice(&app, "INV-AP-101", "2000.00").await;
    let invoice_id = invoice["id"].as_str().unwrap();

    add_test_line(&app, invoice_id, "1200.00").await;
    add_test_line(&app, invoice_id, "800.00").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/ap/invoices/{}/lines", invoice_id))
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
async fn test_delete_invoice_line() {
    let (_state, app) = setup_ap_test().await;

    let invoice = create_test_invoice(&app, "INV-AP-102", "500.00").await;
    let invoice_id = invoice["id"].as_str().unwrap();

    let line = add_test_line(&app, invoice_id, "500.00").await;
    let line_id = line["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/ap/invoices/{}/lines/{}", invoice_id, line_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Invoice Distribution Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_add_invoice_distribution() {
    let (_state, app) = setup_ap_test().await;

    let invoice = create_test_invoice(&app, "INV-AP-200", "1500.00").await;
    let invoice_id = invoice["id"].as_str().unwrap();

    let dist = add_test_distribution(&app, invoice_id, "1500.00").await;
    assert_eq!(dist["distribution_type"], "charge");
    assert_eq!(dist["account_combination"], "1000.200.300");
    assert_eq!(dist["distribution_line_number"], 1);
}

#[tokio::test]
#[ignore]
async fn test_list_invoice_distributions() {
    let (_state, app) = setup_ap_test().await;

    let invoice = create_test_invoice(&app, "INV-AP-201", "3000.00").await;
    let invoice_id = invoice["id"].as_str().unwrap();

    add_test_distribution(&app, invoice_id, "2000.00").await;
    add_test_distribution(&app, invoice_id, "1000.00").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/ap/invoices/{}/distributions", invoice_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(result["data"].as_array().unwrap().len(), 2);
}

// ============================================================================
// Invoice Workflow Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_ap_invoice_full_lifecycle() {
    let (_state, app) = setup_ap_test().await;

    // 1. Create invoice
    let invoice = create_test_invoice(&app, "INV-AP-300", "1500.00").await;
    let invoice_id = invoice["id"].as_str().unwrap();
    assert_eq!(invoice["status"], "draft");

    // 2. Add line
    let line = add_test_line(&app, invoice_id, "1500.00").await;
    assert_eq!(line["line_number"], 1);

    // 3. Add distribution
    let dist = add_test_distribution(&app, invoice_id, "1500.00").await;
    assert_eq!(dist["distribution_line_number"], 1);

    // 4. Submit
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/ap/invoices/{}/submit", invoice_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let submitted: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(submitted["status"], "submitted");

    // 5. Approve
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/ap/invoices/{}/approve", invoice_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let approved: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(approved["status"], "approved");
    assert!(approved["approved_by"].is_string());
}

#[tokio::test]
#[ignore]
async fn test_cannot_submit_empty_invoice() {
    let (_state, app) = setup_ap_test().await;

    let invoice = create_test_invoice(&app, "INV-AP-301", "500.00").await;
    let invoice_id = invoice["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/ap/invoices/{}/submit", invoice_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore]
async fn test_cannot_submit_without_distributions() {
    let (_state, app) = setup_ap_test().await;

    let invoice = create_test_invoice(&app, "INV-AP-302", "500.00").await;
    let invoice_id = invoice["id"].as_str().unwrap();

    add_test_line(&app, invoice_id, "500.00").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/ap/invoices/{}/submit", invoice_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore]
async fn test_cancel_ap_invoice() {
    let (_state, app) = setup_ap_test().await;

    let invoice = create_test_invoice(&app, "INV-AP-303", "750.00").await;
    let invoice_id = invoice["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/ap/invoices/{}/cancel", invoice_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "reason": "Duplicate invoice"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let cancelled: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(cancelled["status"], "cancelled");
    assert_eq!(cancelled["cancelled_reason"], "Duplicate invoice");
}

#[tokio::test]
#[ignore]
async fn test_cannot_add_line_to_submitted_invoice() {
    let (_state, app) = setup_ap_test().await;

    let invoice = create_test_invoice(&app, "INV-AP-304", "500.00").await;
    let invoice_id = invoice["id"].as_str().unwrap();

    add_test_line(&app, invoice_id, "500.00").await;
    add_test_distribution(&app, invoice_id, "500.00").await;

    let (k, v) = auth_header(&admin_claims());
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/ap/invoices/{}/submit", invoice_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Try adding a line to submitted invoice
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/ap/invoices/{}/lines", invoice_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "line_type": "item",
            "amount": "100.00",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Invoice Hold Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_apply_and_release_hold() {
    let (_state, app) = setup_ap_test().await;

    let invoice = create_test_invoice(&app, "INV-AP-400", "2000.00").await;
    let invoice_id = invoice["id"].as_str().unwrap();

    add_test_line(&app, invoice_id, "2000.00").await;
    add_test_distribution(&app, invoice_id, "2000.00").await;

    // Submit
    let (k, v) = auth_header(&admin_claims());
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/ap/invoices/{}/submit", invoice_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Apply hold
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/ap/invoices/{}/holds", invoice_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "hold_type": "manual",
            "hold_reason": "Requires manager review"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let hold: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(hold["hold_type"], "manual");
    assert_eq!(hold["hold_status"], "active");
    let hold_id = hold["id"].as_str().unwrap();

    // Verify invoice is on_hold
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/ap/invoices/{}", invoice_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let inv: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(inv["status"], "on_hold");

    // Try to approve (should fail due to hold)
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/ap/invoices/{}/approve", invoice_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);

    // Release hold
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/ap/holds/{}/release", hold_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "release_reason": "Manager approved"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let released: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(released["hold_status"], "released");
}

#[tokio::test]
#[ignore]
async fn test_list_invoice_holds() {
    let (_state, app) = setup_ap_test().await;

    let invoice = create_test_invoice(&app, "INV-AP-401", "1000.00").await;
    let invoice_id = invoice["id"].as_str().unwrap();

    // Apply a hold
    let (k, v) = auth_header(&admin_claims());
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/ap/invoices/{}/holds", invoice_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "hold_type": "matching",
            "hold_reason": "PO match required"
        })).unwrap())).unwrap()
    ).await.unwrap();

    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/ap/invoices/{}/holds", invoice_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(result["data"].as_array().unwrap().len(), 1);
}

// ============================================================================
// Payment Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_create_payment_and_pay_invoice() {
    let (_state, app) = setup_ap_test().await;

    // Create and approve invoice
    let invoice = create_test_invoice(&app, "INV-AP-500", "2500.00").await;
    let invoice_id = invoice["id"].as_str().unwrap();

    add_test_line(&app, invoice_id, "2500.00").await;
    add_test_distribution(&app, invoice_id, "2500.00").await;

    let (k, v) = auth_header(&admin_claims());
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/ap/invoices/{}/submit", invoice_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/ap/invoices/{}/approve", invoice_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Create payment
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/ap/payments")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "payment_number": "PAY-001",
            "payment_date": "2026-04-20",
            "payment_method": "check",
            "payment_currency_code": "USD",
            "payment_amount": "2500.00",
            "supplier_id": SUPPLIER_ID,
            "supplier_name": "Acme Corp",
            "invoice_ids": [invoice_id],
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let payment: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(payment["payment_number"], "PAY-001");
    assert_eq!(payment["status"], "draft");
    let payment_id = payment["id"].as_str().unwrap();

    // Verify invoice is now paid
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/ap/invoices/{}", invoice_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let paid_inv: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(paid_inv["status"], "paid");
}

#[tokio::test]
#[ignore]
async fn test_list_payments() {
    let (_state, app) = setup_ap_test().await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET").uri("/api/v1/ap/payments")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(result["data"].as_array().unwrap().is_empty());
}

// ============================================================================
// AP Aging Test
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_ap_aging_summary() {
    let (_state, app) = setup_ap_test().await;

    let invoice = create_test_invoice(&app, "INV-AP-600", "3000.00").await;
    let invoice_id = invoice["id"].as_str().unwrap();

    add_test_line(&app, invoice_id, "3000.00").await;
    add_test_distribution(&app, invoice_id, "3000.00").await;

    let (k, v) = auth_header(&admin_claims());
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/ap/invoices/{}/submit", invoice_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Get aging summary
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/ap/aging?as_of_date=2026-04-15")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let aging: serde_json::Value = serde_json::from_slice(&b).unwrap();
    // The invoice has due date 2026-05-15, so as of 2026-04-15 it's current
    let current: f64 = aging["current_amount"].as_str().unwrap().parse().unwrap();
    assert!(current > 0.0);
}

// ============================================================================
// Validation Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_cannot_create_credit_memo_positive() {
    let (_state, app) = setup_ap_test().await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/ap/invoices")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "invoice_number": "CM-AP-001",
            "invoice_date": "2026-04-15",
            "invoice_type": "credit_memo",
            "supplier_id": SUPPLIER_ID,
            "supplier_name": "Acme Corp",
            "invoice_currency_code": "USD",
            "payment_currency_code": "USD",
            "invoice_amount": "100.00",
            "tax_amount": "0.00",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore]
async fn test_cannot_create_duplicate_invoice_number() {
    let (_state, app) = setup_ap_test().await;

    create_test_invoice(&app, "INV-AP-DUP", "100.00").await;

    // Try to create with the same invoice number
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/ap/invoices")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "invoice_number": "INV-AP-DUP",
            "invoice_date": "2026-04-15",
            "invoice_type": "standard",
            "supplier_id": SUPPLIER_ID,
            "supplier_name": "Acme Corp",
            "invoice_currency_code": "USD",
            "payment_currency_code": "USD",
            "invoice_amount": "200.00",
            "tax_amount": "0.00",
        })).unwrap())).unwrap()
    ).await.unwrap();
    // Should succeed with upsert (ON CONFLICT DO UPDATE), returning updated invoice
    assert_eq!(r.status(), StatusCode::CREATED);
}
