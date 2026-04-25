//! Receiving Management E2E Tests
//!
//! Tests for Oracle Fusion SCM Receiving:
//! - Receiving Locations CRUD
//! - Receipt CRUD + lifecycle (draft -> received -> closed/cancelled)
//! - Receipt Lines
//! - Inspections with quality checks
//! - Deliveries / putaway
//! - Returns to Supplier lifecycle (draft -> submitted -> shipped -> credited)
//! - Dashboard analytics

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_receiving_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    setup_test_db(&state.db_pool).await;
    // Clean receiving data for isolation
    sqlx::query("DELETE FROM _atlas.inspection_details").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM _atlas.receipt_inspections").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM _atlas.receipt_deliveries").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM _atlas.receipt_returns").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM _atlas.receipt_lines").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM _atlas.receipt_headers").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM _atlas.receiving_locations").execute(&state.db_pool).await.ok();
    let app = build_router(state.clone());
    (state, app)
}

async fn create_test_location(app: &axum::Router, code: &str, name: &str) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/receiving/locations")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": code,
            "name": name,
            "location_type": "warehouse",
            "city": "San Francisco",
            "country": "US"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_receipt(app: &axum::Router, number: &str) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/receiving/receipts")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "receipt_number": number,
            "receipt_type": "standard",
            "receipt_source": "purchase_order",
            "supplier_name": "Acme Supplies",
            "purchase_order_number": "PO-001",
            "receiving_date": "2026-04-25",
            "carrier": "FedEx",
            "tracking_number": "FX123456"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Location Tests
// ============================================================================

#[tokio::test]
async fn test_create_location() {
    let (_state, app) = setup_receiving_test().await;
    let loc = create_test_location(&app, "WH-01", "Main Warehouse").await;
    assert_eq!(loc["code"], "WH-01");
    assert_eq!(loc["name"], "Main Warehouse");
    assert_eq!(loc["locationType"], "warehouse");
    assert!(loc["id"].is_string());
}

#[tokio::test]
async fn test_list_locations() {
    let (_state, app) = setup_receiving_test().await;
    create_test_location(&app, "WH-A", "Warehouse A").await;
    create_test_location(&app, "WH-B", "Warehouse B").await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/receiving/locations").header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_delete_location() {
    let (_state, app) = setup_receiving_test().await;
    create_test_location(&app, "WH-DEL", "Delete Me").await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/receiving/locations/WH-DEL").header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Receipt Tests
// ============================================================================

#[tokio::test]
async fn test_create_receipt() {
    let (_state, app) = setup_receiving_test().await;
    let receipt = create_test_receipt(&app, "RCV-001").await;
    assert_eq!(receipt["receiptNumber"], "RCV-001");
    assert_eq!(receipt["receiptType"], "standard");
    assert_eq!(receipt["receiptSource"], "purchase_order");
    assert_eq!(receipt["supplierName"], "Acme Supplies");
    assert_eq!(receipt["status"], "draft");
    assert!(receipt["id"].is_string());
}

#[tokio::test]
async fn test_create_receipt_duplicate() {
    let (_state, app) = setup_receiving_test().await;
    create_test_receipt(&app, "RCV-DUP").await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/receiving/receipts")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "receipt_number": "RCV-DUP", "receiving_date": "2026-04-25"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_list_receipts() {
    let (_state, app) = setup_receiving_test().await;
    create_test_receipt(&app, "RCV-LA").await;
    create_test_receipt(&app, "RCV-LB").await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/receiving/receipts?status=draft").header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_get_receipt() {
    let (_state, app) = setup_receiving_test().await;
    let receipt = create_test_receipt(&app, "RCV-GET").await;
    let id = receipt["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/receiving/receipts/{}", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let fetched: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(fetched["receiptNumber"], "RCV-GET");
}

#[tokio::test]
async fn test_receipt_lifecycle() {
    let (_state, app) = setup_receiving_test().await;
    let receipt = create_test_receipt(&app, "RCV-LC").await;
    let id = receipt["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    // Confirm receipt
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/receiving/receipts/{}/confirm", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let confirmed: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(confirmed["status"], "received");
    assert!(confirmed["receivedAt"].is_string());

    // Close receipt
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/receiving/receipts/{}/close", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let closed: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(closed["status"], "closed");
    assert!(closed["closedAt"].is_string());
}

#[tokio::test]
async fn test_cancel_receipt() {
    let (_state, app) = setup_receiving_test().await;
    let receipt = create_test_receipt(&app, "RCV-CNL").await;
    let id = receipt["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/receiving/receipts/{}/cancel", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let cancelled: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(cancelled["status"], "cancelled");
}

#[tokio::test]
async fn test_confirm_non_draft_rejected() {
    let (_state, app) = setup_receiving_test().await;
    let receipt = create_test_receipt(&app, "RCV-ND").await;
    let id = receipt["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());
    // Confirm once
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/receiving/receipts/{}/confirm", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    // Try to confirm again (already received)
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/receiving/receipts/{}/confirm", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Receipt Line Tests
// ============================================================================

#[tokio::test]
async fn test_add_receipt_line() {
    let (_state, app) = setup_receiving_test().await;
    let receipt = create_test_receipt(&app, "RCV-LN").await;
    let id = receipt["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/receiving/receipts/{}/lines", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "item_code": "ITEM-001",
            "item_description": "Steel Bolts M10",
            "ordered_qty": "100",
            "ordered_uom": "Each",
            "received_qty": "98",
            "received_uom": "Each",
            "lot_number": "LOT-2026-001",
            "unit_price": "2.50",
            "currency": "USD"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let line: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(line["itemCode"], "ITEM-001");
    assert_eq!(line["receivedQty"], "98");
    assert_eq!(line["lineNumber"], 1);
    assert_eq!(line["inspectionStatus"], "pending");
    assert_eq!(line["deliveryStatus"], "pending");
}

#[tokio::test]
async fn test_list_receipt_lines() {
    let (_state, app) = setup_receiving_test().await;
    let receipt = create_test_receipt(&app, "RCV-LL").await;
    let id = receipt["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    // Add two lines
    for item in ["ITEM-A", "ITEM-B"] {
        app.clone().oneshot(Request::builder().method("POST")
            .uri(&format!("/api/v1/receiving/receipts/{}/lines", id))
            .header("Content-Type", "application/json").header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({
                "item_code": item, "received_qty": "10"
            })).unwrap())).unwrap()
        ).await.unwrap();
    }

    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/receiving/receipts/{}/lines", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 2);
}

// ============================================================================
// Inspection Tests
// ============================================================================

#[tokio::test]
async fn test_create_and_complete_inspection() {
    let (_state, app) = setup_receiving_test().await;
    let receipt = create_test_receipt(&app, "RCV-INS").await;
    let rcpt_id = receipt["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    // Add a line
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/receiving/receipts/{}/lines", rcpt_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "item_code": "ITEM-QC", "received_qty": "50"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let line: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let line_id = line["id"].as_str().unwrap();

    // Create inspection
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/receiving/receipts/{}/inspections", rcpt_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "receipt_line_id": line_id,
            "inspector_name": "Jane QC",
            "inspection_date": "2026-04-25",
            "sample_size": "10"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let inspection: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(inspection["status"], "pending");
    assert!(inspection["inspectionNumber"].as_str().unwrap().starts_with("INS-"));
    let insp_id = inspection["id"].as_str().unwrap();

    // Complete inspection
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/receiving/inspections/{}/complete", insp_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "quantity_inspected": "10",
            "quantity_accepted": "9",
            "quantity_rejected": "1",
            "disposition": "conditional",
            "quality_score": "90.5",
            "rejection_reason": "Minor surface scratch"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let completed: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(completed["status"], "completed");
    assert_eq!(completed["disposition"], "conditional");
    // quality_score may come as "90.5" or "90.50" depending on formatting
    let qs = completed["qualityScore"].as_str().unwrap();
    assert!((qs.parse::<f64>().unwrap() - 90.5).abs() < 0.01, "quality_score was {}", qs);
    assert!(completed["completedAt"].is_string());
}

#[tokio::test]
async fn test_inspection_accepted_rejected_must_equal_inspected() {
    let (_state, app) = setup_receiving_test().await;
    let receipt = create_test_receipt(&app, "RCV-VAL").await;
    let rcpt_id = receipt["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    // Add line
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/receiving/receipts/{}/lines", rcpt_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "item_code": "ITEM-V", "received_qty": "50"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let line: serde_json::Value = serde_json::from_slice(&axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()).unwrap();
    let line_id = line["id"].as_str().unwrap();

    // Create inspection
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/receiving/receipts/{}/inspections", rcpt_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "receipt_line_id": line_id, "inspection_date": "2026-04-25"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let insp: serde_json::Value = serde_json::from_slice(&axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()).unwrap();
    let insp_id = insp["id"].as_str().unwrap();

    // Try invalid: inspected=10, accepted=5, rejected=3 (total != 10)
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/receiving/inspections/{}/complete", insp_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "quantity_inspected": "10", "quantity_accepted": "5", "quantity_rejected": "3",
            "disposition": "accept"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_inspection_details() {
    let (_state, app) = setup_receiving_test().await;
    let receipt = create_test_receipt(&app, "RCV-DET").await;
    let rcpt_id = receipt["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/receiving/receipts/{}/lines", rcpt_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "item_code": "ITEM-D", "received_qty": "50"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let line: serde_json::Value = serde_json::from_slice(&axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()).unwrap();
    let line_id = line["id"].as_str().unwrap();

    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/receiving/receipts/{}/inspections", rcpt_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "receipt_line_id": line_id, "inspection_date": "2026-04-25"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let insp: serde_json::Value = serde_json::from_slice(&axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()).unwrap();
    let insp_id = insp["id"].as_str().unwrap();

    // Add quality check details
    for (name, result) in [("Dimensional Check", "pass"), ("Surface Quality", "fail")] {
        let r = app.clone().oneshot(Request::builder().method("POST")
            .uri(&format!("/api/v1/receiving/inspections/{}/details", insp_id))
            .header("Content-Type", "application/json").header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({
                "check_name": name, "check_type": "measurement", "result": result,
                "measured_value": "10.02mm", "expected_value": "10.00mm"
            })).unwrap())).unwrap()
        ).await.unwrap();
        assert_eq!(r.status(), StatusCode::CREATED);
    }

    // List details
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/receiving/inspections/{}/details", insp_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 2);
}

// ============================================================================
// Delivery Tests
// ============================================================================

#[tokio::test]
async fn test_create_delivery() {
    let (_state, app) = setup_receiving_test().await;
    let receipt = create_test_receipt(&app, "RCV-DEL").await;
    let rcpt_id = receipt["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    // Add line
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/receiving/receipts/{}/lines", rcpt_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "item_code": "ITEM-DL", "received_qty": "50"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let line: serde_json::Value = serde_json::from_slice(&axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()).unwrap();
    let line_id = line["id"].as_str().unwrap();

    // Create delivery
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/receiving/receipts/{}/deliveries", rcpt_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "receipt_line_id": line_id,
            "subinventory": "FINISHED_GOODS",
            "locator": "A-01-03",
            "quantity_delivered": "50",
            "uom": "Each",
            "destination_type": "inventory",
            "delivered_by_name": "John Receiver"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let delivery: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(delivery["subinventory"], "FINISHED_GOODS");
    assert_eq!(delivery["quantityDelivered"], "50");
    assert_eq!(delivery["destinationType"], "inventory");
    assert!(delivery["deliveryNumber"].as_str().unwrap().starts_with("DEL-"));
}

#[tokio::test]
async fn test_list_deliveries() {
    let (_state, app) = setup_receiving_test().await;
    let receipt = create_test_receipt(&app, "RCV-LD").await;
    let rcpt_id = receipt["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    // Add line + delivery
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/receiving/receipts/{}/lines", rcpt_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "item_code": "ITEM-LD", "received_qty": "100"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let line: serde_json::Value = serde_json::from_slice(&axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()).unwrap();
    let line_id = line["id"].as_str().unwrap();

    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/receiving/receipts/{}/deliveries", rcpt_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "receipt_line_id": line_id, "quantity_delivered": "50"
        })).unwrap())).unwrap()
    ).await.unwrap();

    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/receiving/receipts/{}/deliveries", rcpt_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 1);
}

// ============================================================================
// Return to Supplier Tests
// ============================================================================

#[tokio::test]
async fn test_return_lifecycle() {
    let (_state, app) = setup_receiving_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Create return
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/receiving/returns")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "supplier_name": "Bad Parts Co",
            "return_type": "damaged",
            "item_code": "ITEM-DMG",
            "item_description": "Damaged widgets",
            "quantity_returned": "25",
            "uom": "Each",
            "unit_price": "10.00",
            "currency": "USD",
            "return_reason": "Items arrived bent and unusable",
            "return_date": "2026-04-25"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let ret: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(ret["status"], "draft");
    assert_eq!(ret["returnType"], "damaged");
    assert!(ret["returnNumber"].as_str().unwrap().starts_with("RTV-"));
    let ret_id = ret["id"].as_str().unwrap();

    // Submit
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/receiving/returns/{}/submit", ret_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let submitted: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(submitted["status"], "submitted");

    // Ship
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/receiving/returns/{}/ship", ret_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "carrier": "UPS", "tracking_number": "UPS-999"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let shipped: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(shipped["status"], "shipped");
    assert_eq!(shipped["carrier"], "UPS");

    // Credit
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/receiving/returns/{}/credit", ret_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let credited: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(credited["status"], "credited");
    assert!(credited["creditedAt"].is_string());
}

#[tokio::test]
async fn test_cancel_return() {
    let (_state, app) = setup_receiving_test().await;
    let (k, v) = auth_header(&admin_claims());

    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/receiving/returns")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "return_type": "excess", "quantity_returned": "5",
            "return_date": "2026-04-25"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let ret: serde_json::Value = serde_json::from_slice(&axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()).unwrap();
    let ret_id = ret["id"].as_str().unwrap();

    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/receiving/returns/{}/cancel", ret_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let cancelled: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(cancelled["status"], "cancelled");
}

#[tokio::test]
async fn test_list_returns() {
    let (_state, app) = setup_receiving_test().await;
    let (k, v) = auth_header(&admin_claims());

    for rt in ["damaged", "excess"] {
        app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/receiving/returns")
            .header("Content-Type", "application/json").header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({
                "return_type": rt, "quantity_returned": "1", "return_date": "2026-04-25"
            })).unwrap())).unwrap()
        ).await.unwrap();
    }

    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/receiving/returns?status=draft").header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 2);
}

// ============================================================================
// Dashboard Test
// ============================================================================

#[tokio::test]
async fn test_receiving_dashboard() {
    let (_state, app) = setup_receiving_test().await;
    create_test_receipt(&app, "RCV-DB1").await;
    create_test_receipt(&app, "RCV-DB2").await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/receiving/dashboard").header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let dashboard: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(dashboard["totalReceipts"].as_i64().unwrap() >= 2);
    assert!(dashboard["pendingReceipts"].as_i64().unwrap() >= 2);
    assert!(dashboard["receiptsByStatus"].is_array());
}
