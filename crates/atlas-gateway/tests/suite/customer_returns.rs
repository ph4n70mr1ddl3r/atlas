//! Customer Returns Management / RMA E2E Tests
//!
//! Tests for Oracle Fusion Cloud ERP Customer Returns:
//! - Return reason code CRUD
//! - RMA creation and lifecycle (draft -> submitted -> approved -> received)
//! - Return line management
//! - Return receipt and inspection
//! - Credit memo generation and lifecycle
//! - Dashboard summary
//! - Validation edge cases

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_returns_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    // Run migration for customer returns tables
    sqlx::query(include_str!("../../../../migrations/031_customer_returns.sql"))
        .execute(&state.db_pool)
        .await
        .ok(); // Ignore errors if tables already exist
    let app = build_router(state.clone());
    (state, app)
}

async fn create_test_return_reason(
    app: &axum::Router,
    code: &str,
    name: &str,
    return_type: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "code": code,
        "name": name,
        "return_type": return_type,
        "default_disposition": "return_to_stock",
        "requires_approval": false,
        "credit_issued_automatically": true,
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/returns/reasons")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Failed to create return reason");
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_rma(
    app: &axum::Router,
    customer_id: &str,
    return_type: &str,
    reason_code: Option<&str>,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let mut payload = json!({
        "customer_id": customer_id,
        "return_type": return_type,
        "currency_code": "USD",
    });
    if let Some(rc) = reason_code {
        payload["reason_code"] = json!(rc);
    }
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/returns/rmas")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Failed to create RMA");
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn add_test_return_line(
    app: &axum::Router,
    rma_id: &str,
    item_code: &str,
    return_qty: &str,
    unit_price: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "item_code": item_code,
        "item_description": format!("Test item {}", item_code),
        "original_quantity": "100",
        "return_quantity": return_qty,
        "unit_price": unit_price,
        "condition": "defective",
        "disposition": "return_to_stock",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/returns/rmas/{}/lines", rma_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Failed to add return line");
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Return Reason Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_return_reason_crud() {
    let (_state, app) = setup_returns_test().await;

    // Create
    let reason = create_test_return_reason(&app, "DEFECTIVE", "Defective Product", "standard_return").await;
    assert_eq!(reason["code"], "DEFECTIVE");
    assert_eq!(reason["name"], "Defective Product");
    assert_eq!(reason["return_type"], "standard_return");
    assert_eq!(reason["is_active"], true);

    // Get
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/returns/reasons/DEFECTIVE")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let got: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(got["code"], "DEFECTIVE");

    // List
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/returns/reasons")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let list: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(list["data"].as_array().unwrap().len() >= 1);

    // Delete
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/returns/reasons/DEFECTIVE")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);

    // Verify deleted
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/returns/reasons/DEFECTIVE")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
#[ignore]
async fn test_return_reason_validation() {
    let (_state, app) = setup_returns_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Empty code
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/returns/reasons")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "",
            "name": "Test",
            "return_type": "standard_return",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);

    // Invalid return type
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/returns/reasons")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "TEST",
            "name": "Test",
            "return_type": "teleport",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// RMA Lifecycle Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_rma_full_lifecycle() {
    let (_state, app) = setup_returns_test().await;

    let customer_id = "00000000-0000-0000-0000-000000000100";

    // Create RMA
    let rma = create_test_rma(&app, customer_id, "standard_return", None).await;
    assert!(rma["rma_number"].as_str().unwrap().starts_with("RMA-"));
    assert_eq!(rma["status"], "draft");
    assert_eq!(rma["return_type"], "standard_return");
    let rma_id = rma["id"].as_str().unwrap().to_string();

    // Add a return line
    let line = add_test_return_line(&app, &rma_id, "WIDGET-001", "10", "50.00").await;
    assert_eq!(line["line_number"], 1);
    assert_eq!(line["return_quantity"], "10");
    assert_eq!(line["condition"], "defective");
    let line_id = line["id"].as_str().unwrap().to_string();

    // Verify RMA totals updated
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/returns/rmas/{}", rma_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let updated_rma: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let total_qty: f64 = updated_rma["total_quantity"].as_str().unwrap().parse().unwrap();
    assert!((total_qty - 10.0).abs() < 0.01, "Expected total_quantity 10, got {}", total_qty);
    let total_amt: f64 = updated_rma["total_amount"].as_str().unwrap().parse().unwrap();
    assert!((total_amt - 500.0).abs() < 0.01, "Expected total_amount 500, got {}", total_amt);

    // Submit RMA
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/returns/rmas/{}/submit", rma_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let submitted: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(submitted["status"], "submitted");

    // Approve RMA
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/returns/rmas/{}/approve", rma_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let approved: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(approved["status"], "approved");
    assert!(approved["approved_by"].is_string());

    // Receive the returned item
    let payload = json!({ "received_quantity": "10" });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/returns/lines/{}/receive", line_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let received: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(received["received_quantity"], "10");

    // Verify RMA status changed to received
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/returns/rmas/{}", rma_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    let final_rma: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(final_rma["status"], "received");

    // Inspect the return line
    let payload = json!({
        "inspection_status": "passed",
        "inspection_notes": "Product confirmed defective",
        "disposition": "scrap",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/returns/lines/{}/inspect", line_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let inspected: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(inspected["inspection_status"], "passed");
    assert_eq!(inspected["disposition"], "scrap");
}

// ============================================================================
// Credit Memo Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_credit_memo_generation_and_lifecycle() {
    let (_state, app) = setup_returns_test().await;

    let customer_id = "00000000-0000-0000-0000-000000000200";

    // Create and approve an RMA with a line
    let rma = create_test_rma(&app, customer_id, "standard_return", None).await;
    let rma_id = rma["id"].as_str().unwrap().to_string();
    add_test_return_line(&app, &rma_id, "GADGET-001", "5", "100.00").await;

    let (k, v) = auth_header(&admin_claims());

    // Submit and approve
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/returns/rmas/{}/submit", rma_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/returns/rmas/{}/approve", rma_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Generate credit memo
    let payload = json!({ "gl_account_code": "4100-RETURNS" });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/returns/rmas/{}/credit-memo", rma_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let memo: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(memo["credit_memo_number"].as_str().unwrap().starts_with("CM-"));
    assert_eq!(memo["status"], "draft");
    let memo_amount: f64 = memo["amount"].as_str().unwrap().parse().unwrap();
    assert!((memo_amount - 500.0).abs() < 0.01, "Expected 500, got {}", memo_amount);
    let memo_id = memo["id"].as_str().unwrap().to_string();

    // Issue the credit memo
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/returns/credit-memos/{}/issue", memo_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let issued: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(issued["status"], "issued");

    // List credit memos
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/returns/credit-memos")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let list: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(list["data"].as_array().unwrap().len() >= 1);

    // Get credit memo by ID
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/returns/credit-memos/{}", memo_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    // Verify RMA has credit memo reference
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/returns/rmas/{}", rma_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let updated_rma: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(updated_rma["credit_memo_number"].is_string());
}

// ============================================================================
// RMA Workflow Validation Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_cannot_submit_empty_rma() {
    let (_state, app) = setup_returns_test().await;

    let customer_id = "00000000-0000-0000-0000-000000000300";
    let rma = create_test_rma(&app, customer_id, "standard_return", None).await;
    let rma_id = rma["id"].as_str().unwrap().to_string();

    // Try to submit without lines
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/returns/rmas/{}/submit", rma_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore]
async fn test_cannot_approve_draft_rma() {
    let (_state, app) = setup_returns_test().await;

    let customer_id = "00000000-0000-0000-0000-000000000400";
    let rma = create_test_rma(&app, customer_id, "standard_return", None).await;
    let rma_id = rma["id"].as_str().unwrap().to_string();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/returns/rmas/{}/approve", rma_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore]
async fn test_cannot_add_line_to_submitted_rma() {
    let (_state, app) = setup_returns_test().await;

    let customer_id = "00000000-0000-0000-0000-000000000500";
    let rma = create_test_rma(&app, customer_id, "standard_return", None).await;
    let rma_id = rma["id"].as_str().unwrap().to_string();

    // Add a line, submit, then try to add another
    add_test_return_line(&app, &rma_id, "ITEM-001", "1", "10.00").await;

    let (k, v) = auth_header(&admin_claims());
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/returns/rmas/{}/submit", rma_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Try to add another line after submission
    let payload = json!({
        "item_code": "ITEM-002",
        "original_quantity": "10",
        "return_quantity": "1",
        "unit_price": "10.00",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/returns/rmas/{}/lines", rma_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// RMA Cancellation Test
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_rma_cancellation() {
    let (_state, app) = setup_returns_test().await;

    let customer_id = "00000000-0000-0000-0000-000000000600";
    let rma = create_test_rma(&app, customer_id, "warranty", None).await;
    let rma_id = rma["id"].as_str().unwrap().to_string();

    let (k, v) = auth_header(&admin_claims());

    // Cancel the draft RMA
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/returns/rmas/{}/cancel", rma_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let cancelled: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(cancelled["status"], "cancelled");
}

// ============================================================================
// Return Reason Filter Test
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_list_return_reasons_by_type() {
    let (_state, app) = setup_returns_test().await;

    create_test_return_reason(&app, "WRONG-ITEM", "Wrong Item Shipped", "standard_return").await;
    create_test_return_reason(&app, "WARRANTY-FAIL", "Warranty Failure", "warranty").await;

    let (k, v) = auth_header(&admin_claims());

    // List all
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/returns/reasons")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let all: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(all["data"].as_array().unwrap().len() >= 2);

    // Filter by warranty type
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/returns/reasons?return_type=warranty")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let warranty: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let items = warranty["data"].as_array().unwrap();
    assert!(items.len() >= 1);
    assert!(items.iter().all(|r| r["return_type"] == "warranty"));
}

// ============================================================================
// Dashboard Test
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_returns_dashboard() {
    let (_state, app) = setup_returns_test().await;

    // Create some data
    create_test_return_reason(&app, "DASH-REASON", "Dashboard Reason", "standard_return").await;
    create_test_rma(&app, "00000000-0000-0000-0000-000000000999", "standard_return", None).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/returns/dashboard")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let summary: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();

    assert!(summary["total_rmas"].as_i64().unwrap() >= 1);
    assert!(summary["open_rmas"].as_i64().unwrap() >= 1);
}

// ============================================================================
// Validation Edge Cases
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_return_quantity_exceeds_original() {
    let (_state, app) = setup_returns_test().await;

    let customer_id = "00000000-0000-0000-0000-000000000700";
    let rma = create_test_rma(&app, customer_id, "standard_return", None).await;
    let rma_id = rma["id"].as_str().unwrap().to_string();

    // Try to return more than original
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "item_code": "OVERFLOW-001",
        "original_quantity": "5",
        "return_quantity": "100",
        "unit_price": "10.00",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/returns/rmas/{}/lines", rma_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore]
async fn test_receive_exceeds_return_quantity() {
    let (_state, app) = setup_returns_test().await;

    let customer_id = "00000000-0000-0000-0000-000000000800";
    let rma = create_test_rma(&app, customer_id, "standard_return", None).await;
    let rma_id = rma["id"].as_str().unwrap().to_string();

    let line = add_test_return_line(&app, &rma_id, "LIMIT-001", "5", "10.00").await;
    let line_id = line["id"].as_str().unwrap().to_string();

    // Try to receive more than the return quantity
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({ "received_quantity": "100" });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/returns/lines/{}/receive", line_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore]
async fn test_cannot_generate_credit_memo_for_draft_rma() {
    let (_state, app) = setup_returns_test().await;

    let customer_id = "00000000-0000-0000-0000-000000000900";
    let rma = create_test_rma(&app, customer_id, "standard_return", None).await;
    let rma_id = rma["id"].as_str().unwrap().to_string();

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({});
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/returns/rmas/{}/credit-memo", rma_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}
