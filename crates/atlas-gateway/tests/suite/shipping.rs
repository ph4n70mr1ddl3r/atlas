//! Shipping Execution E2E Tests
//!
//! Tests for Oracle Fusion SCM Shipping Execution:
//! - Carrier CRUD
//! - Shipping method CRUD
//! - Shipment lifecycle (create -> confirm -> ship -> deliver)
//! - Shipment line management
//! - Packing slips
//! - Dashboard analytics

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_shipping_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    setup_test_db(&state.db_pool).await;
    // Clean shipping data for isolation
    sqlx::query("DELETE FROM _atlas.packing_slip_lines").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM _atlas.packing_slips").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM _atlas.shipment_lines").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM _atlas.shipments").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM _atlas.shipping_methods").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM _atlas.shipping_carriers").execute(&state.db_pool).await.ok();
    let app = build_router(state.clone());
    (state, app)
}

async fn create_test_carrier(app: &axum::Router, code: &str) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/shipping/carriers")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": code,
            "name": format!("{} Express", code),
            "carrier_type": "external",
            "contact_email": "test@carrier.com"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_shipment(app: &axum::Router, num: &str) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/shipping/shipments")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "shipment_number": num,
            "customer_name": "Acme Corp",
            "ship_to_name": "John Doe",
            "ship_to_city": "New York",
            "ship_to_country": "US",
            "ship_from_warehouse": "WH-EAST-01"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Carrier Tests
// ============================================================================

#[tokio::test]
async fn test_create_carrier() {
    let (_state, app) = setup_shipping_test().await;
    let carrier = create_test_carrier(&app, "FEDEX").await;
    assert_eq!(carrier["code"], "FEDEX");
    assert_eq!(carrier["name"], "FEDEX Express");
    assert_eq!(carrier["carrierType"], "external");
    assert!(carrier["id"].is_string());
}

#[tokio::test]
async fn test_create_carrier_duplicate() {
    let (_state, app) = setup_shipping_test().await;
    create_test_carrier(&app, "DUP").await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/shipping/carriers")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "DUP", "name": "Duplicate"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_list_carriers() {
    let (_state, app) = setup_shipping_test().await;
    create_test_carrier(&app, "C1").await;
    create_test_carrier(&app, "C2").await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/shipping/carriers").header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_get_carrier() {
    let (_state, app) = setup_shipping_test().await;
    let carrier = create_test_carrier(&app, "GETC").await;
    let id = carrier["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/shipping/carriers/{}", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let fetched: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(fetched["code"], "GETC");
}

#[tokio::test]
async fn test_delete_carrier() {
    let (_state, app) = setup_shipping_test().await;
    create_test_carrier(&app, "DELC").await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/shipping/carriers-by-code/DELC").header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Shipping Method Tests
// ============================================================================

#[tokio::test]
async fn test_create_shipping_method() {
    let (_state, app) = setup_shipping_test().await;
    let carrier = create_test_carrier(&app, "UPS").await;
    let carrier_id = carrier["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/shipping/methods")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "GROUND",
            "name": "Ground Shipping",
            "carrier_id": carrier_id,
            "transit_time_days": 5,
            "is_express": false
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let method: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(method["code"], "GROUND");
    assert_eq!(method["transitTimeDays"], 5);
    assert_eq!(method["isExpress"], false);
}

#[tokio::test]
async fn test_list_shipping_methods() {
    let (_state, app) = setup_shipping_test().await;
    let (k, v) = auth_header(&admin_claims());
    app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/shipping/methods")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "EXP", "name": "Express", "transit_time_days": 1, "is_express": true
        })).unwrap())).unwrap()
    ).await.unwrap();
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/shipping/methods").header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 1);
}

// ============================================================================
// Shipment Tests
// ============================================================================

#[tokio::test]
async fn test_create_shipment() {
    let (_state, app) = setup_shipping_test().await;
    let shipment = create_test_shipment(&app, "SHIP-001").await;
    assert_eq!(shipment["shipmentNumber"], "SHIP-001");
    assert_eq!(shipment["status"], "draft");
    assert_eq!(shipment["customerName"], "Acme Corp");
    assert_eq!(shipment["shipFromWarehouse"], "WH-EAST-01");
    assert!(shipment["id"].is_string());
}

#[tokio::test]
async fn test_create_shipment_duplicate() {
    let (_state, app) = setup_shipping_test().await;
    create_test_shipment(&app, "SHIP-DUP").await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/shipping/shipments")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "shipment_number": "SHIP-DUP", "customer_name": "Test"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_list_shipments() {
    let (_state, app) = setup_shipping_test().await;
    create_test_shipment(&app, "SHIP-LA").await;
    create_test_shipment(&app, "SHIP-LB").await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/shipping/shipments?status=draft").header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_shipment_lifecycle() {
    let (_state, app) = setup_shipping_test().await;
    let shipment = create_test_shipment(&app, "SHIP-LC").await;
    let id = shipment["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    // Confirm
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/shipping/shipments/{}/confirm", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let confirmed: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(confirmed["status"], "confirmed");
    assert!(confirmed["confirmedAt"].is_string());

    // Ship confirm
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/shipping/shipments/{}/ship", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "tracking_number": "1Z999AA10123456784"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let shipped: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(shipped["status"], "shipped");
    assert_eq!(shipped["trackingNumber"], "1Z999AA10123456784");
    assert!(shipped["shippedDate"].is_string());

    // Deliver
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/shipping/shipments/{}/deliver", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let delivered: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(delivered["status"], "delivered");
    assert!(delivered["actualDelivery"].is_string());
}

#[tokio::test]
async fn test_confirm_non_draft_rejected() {
    let (_state, app) = setup_shipping_test().await;
    let shipment = create_test_shipment(&app, "SHIP-ND").await;
    let id = shipment["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());
    // Confirm once
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/shipping/shipments/{}/confirm", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    // Try to confirm again
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/shipping/shipments/{}/confirm", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_cancel_shipment() {
    let (_state, app) = setup_shipping_test().await;
    let shipment = create_test_shipment(&app, "SHIP-CNL").await;
    let id = shipment["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/shipping/shipments/{}/cancel", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let cancelled: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(cancelled["status"], "cancelled");
}

#[tokio::test]
async fn test_cancel_shipped_rejected() {
    let (_state, app) = setup_shipping_test().await;
    let shipment = create_test_shipment(&app, "SHIP-CNS").await;
    let id = shipment["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());
    // Confirm & ship
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/shipping/shipments/{}/confirm", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/shipping/shipments/{}/ship", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"tracking_number": "T123"})).unwrap())).unwrap()
    ).await.unwrap();
    // Try to cancel shipped
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/shipping/shipments/{}/cancel", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Shipment Line Tests
// ============================================================================

#[tokio::test]
async fn test_add_shipment_line() {
    let (_state, app) = setup_shipping_test().await;
    let shipment = create_test_shipment(&app, "SHIP-LN").await;
    let id = shipment["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/shipping/shipments/{}/lines", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "item_code": "SKU-100",
            "item_name": "Widget A",
            "requested_quantity": "50",
            "weight": "2.5",
            "is_fragile": true
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let line: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(line["itemCode"], "SKU-100");
    assert_eq!(line["itemName"], "Widget A");
    assert!(line["requestedQuantity"].as_str().unwrap().contains("50"));
    assert!(line["lineNumber"].as_i64().unwrap() >= 1);
}

#[tokio::test]
async fn test_list_shipment_lines() {
    let (_state, app) = setup_shipping_test().await;
    let shipment = create_test_shipment(&app, "SHIP-LL").await;
    let id = shipment["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());
    for item in ["SKU-A", "SKU-B"] {
        app.clone().oneshot(Request::builder().method("POST")
            .uri(&format!("/api/v1/shipping/shipments/{}/lines", id))
            .header("Content-Type", "application/json").header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({
                "item_code": item, "requested_quantity": "10"
            })).unwrap())).unwrap()
        ).await.unwrap();
    }
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/shipping/shipments/{}/lines", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_add_line_to_non_draft_rejected() {
    let (_state, app) = setup_shipping_test().await;
    let shipment = create_test_shipment(&app, "SHIP-NDA").await;
    let id = shipment["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());
    // Confirm
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/shipping/shipments/{}/confirm", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    // Try adding line
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/shipping/shipments/{}/lines", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "item_code": "SKU-LATE", "requested_quantity": "5"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Packing Slip Tests
// ============================================================================

#[tokio::test]
async fn test_create_packing_slip() {
    let (_state, app) = setup_shipping_test().await;
    let shipment = create_test_shipment(&app, "SHIP-PS").await;
    let id = shipment["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/shipping/shipments/{}/packing-slips", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "packing_slip_number": "PS-001",
            "package_number": 1,
            "package_type": "box",
            "weight": "3.5",
            "dimensions_length": "30",
            "dimensions_width": "20",
            "dimensions_height": "15"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let ps: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(ps["packingSlipNumber"], "PS-001");
    assert_eq!(ps["packageType"], "box");
    assert!(ps["weight"].as_str().unwrap().contains("3.5"));
}

#[tokio::test]
async fn test_list_packing_slips() {
    let (_state, app) = setup_shipping_test().await;
    let shipment = create_test_shipment(&app, "SHIP-PSL").await;
    let id = shipment["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());
    for i in 1..=2 {
        app.clone().oneshot(Request::builder().method("POST")
            .uri(&format!("/api/v1/shipping/shipments/{}/packing-slips", id))
            .header("Content-Type", "application/json").header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({
                "packing_slip_number": format!("PS-{}{}", id.chars().take(4).collect::<String>(), i),
                "package_number": i
            })).unwrap())).unwrap()
        ).await.unwrap();
    }
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/shipping/shipments/{}/packing-slips", id))
        .header(&k, &v).body(Body::empty()).unwrap()
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
async fn test_shipping_dashboard() {
    let (_state, app) = setup_shipping_test().await;
    create_test_shipment(&app, "SHIP-DB").await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/shipping/dashboard").header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let dashboard: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(dashboard["totalShipments"].as_i64().unwrap() >= 1);
}
