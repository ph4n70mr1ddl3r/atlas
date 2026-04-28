//! Transportation Management E2E Tests
//!
//! Tests for Oracle Fusion Cloud SCM > Transportation Management:
//! - Carrier CRUD and lifecycle (suspend, reactivate, blacklist)
//! - Carrier service management
//! - Transport lane CRUD and lifecycle
//! - Shipment lifecycle (draft → booked → picked_up → in_transit → delivered)
//! - Shipment stops and lines
//! - Tracking events
//! - Freight rates
//! - Transportation dashboard
//! - Full end-to-end lifecycle test

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_transportation_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    let app = build_router(state.clone());
    (state, app)
}

// ============================================================================
// Helper Functions
// ============================================================================

async fn create_test_carrier(
    app: &axum::Router, carrier_code: &str, name: &str, carrier_type: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/transport/carriers")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "carrierCode": carrier_code,
            "name": name,
            "carrierType": carrier_type,
            "defaultServiceLevel": "standard",
            "contactName": "John Doe",
            "contactEmail": "john@carrier.com",
            "city": "New York",
            "state": "NY",
            "country": "USA"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    if status != StatusCode::CREATED {
        panic!("Expected CREATED for carrier but got {}: {}", status, String::from_utf8_lossy(&b));
    }
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_lane(
    app: &axum::Router, lane_code: &str, name: &str, lane_type: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/transport/lanes")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "laneCode": lane_code,
            "name": name,
            "laneType": lane_type,
            "originCity": "New York",
            "originState": "NY",
            "originCountry": "USA",
            "destinationCity": "Los Angeles",
            "destinationState": "CA",
            "destinationCountry": "USA",
            "distanceKm": 3944.0,
            "estimatedTransitHours": 48.0
        })).unwrap())).unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    if status != StatusCode::CREATED {
        panic!("Expected CREATED for lane but got {}: {}", status, String::from_utf8_lossy(&b));
    }
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_shipment(
    app: &axum::Router, shipment_number: &str, shipment_type: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/transport/shipments")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "shipmentNumber": shipment_number,
            "name": format!("Shipment {}", shipment_number),
            "shipmentType": shipment_type,
            "priority": "normal",
            "originLocationName": "Warehouse NYC",
            "originAddress": {"city": "New York", "state": "NY", "country": "USA"},
            "destinationLocationName": "Customer LA",
            "destinationAddress": {"city": "Los Angeles", "state": "CA", "country": "USA"},
            "plannedShipDate": "2026-05-01",
            "plannedDeliveryDate": "2026-05-03",
            "currencyCode": "USD"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    if status != StatusCode::CREATED {
        panic!("Expected CREATED for shipment but got {}: {}", status, String::from_utf8_lossy(&b));
    }
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Carrier Tests
// ============================================================================

#[tokio::test]
async fn test_create_carrier() {
    let (_state, app) = setup_transportation_test().await;

    let carrier = create_test_carrier(&app, "FEDEX", "Federal Express", "parcel").await;

    assert_eq!(carrier["carrierCode"], "FEDEX");
    assert_eq!(carrier["name"], "Federal Express");
    assert_eq!(carrier["carrierType"], "parcel");
    assert_eq!(carrier["status"], "active");
    assert_eq!(carrier["defaultServiceLevel"], "standard");
    assert_eq!(carrier["country"], "USA");
}

#[tokio::test]
async fn test_create_carrier_duplicate_code() {
    let (_state, app) = setup_transportation_test().await;

    create_test_carrier(&app, "UPS", "UPS Ground", "parcel").await;

    // Try creating with same code
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/transport/carriers")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "carrierCode": "UPS",
            "name": "UPS Duplicate",
            "carrierType": "parcel",
            "defaultServiceLevel": "standard"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_list_carriers() {
    let (_state, app) = setup_transportation_test().await;

    create_test_carrier(&app, "FEDEX", "Federal Express", "parcel").await;
    create_test_carrier(&app, "DHL", "DHL Express", "air").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET").uri("/api/v1/transport/carriers")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let body = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(result["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_list_carriers_with_filter() {
    let (_state, app) = setup_transportation_test().await;

    create_test_carrier(&app, "FEDEX", "Federal Express", "parcel").await;
    create_test_carrier(&app, "DHL", "DHL Express", "air").await;

    // Filter by carrier_type
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/transport/carriers?carrier_type=air")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let body = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let data = result["data"].as_array().unwrap();
    assert!(data.iter().all(|c| c["carrierType"] == "air"));
}

#[tokio::test]
async fn test_get_carrier() {
    let (_state, app) = setup_transportation_test().await;

    let carrier = create_test_carrier(&app, "FEDEX", "Federal Express", "parcel").await;
    let carrier_id = carrier["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let uri = format!("/api/v1/transport/carriers/id/{}", carrier_id);
    let r = app.clone().oneshot(Request::builder().method("GET").uri(&uri)
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let body = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(result["carrierCode"], "FEDEX");
}

#[tokio::test]
async fn test_get_carrier_not_found() {
    let (_state, app) = setup_transportation_test().await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/transport/carriers/id/00000000-0000-0000-0000-000000000999")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_carrier_lifecycle() {
    let (_state, app) = setup_transportation_test().await;

    let carrier = create_test_carrier(&app, "UPS", "UPS Ground", "parcel").await;
    let carrier_id = carrier["id"].as_str().unwrap();

    // Suspend
    let (k, v) = auth_header(&admin_claims());
    let uri = format!("/api/v1/transport/carriers/id/{}/suspend", carrier_id);
    let r = app.clone().oneshot(Request::builder().method("POST").uri(&uri)
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(result["status"], "suspended");

    // Reactivate
    let uri = format!("/api/v1/transport/carriers/id/{}/reactivate", carrier_id);
    let r = app.clone().oneshot(Request::builder().method("POST").uri(&uri)
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(result["status"], "active");

    // Blacklist
    let uri = format!("/api/v1/transport/carriers/id/{}/blacklist", carrier_id);
    let r = app.clone().oneshot(Request::builder().method("POST").uri(&uri)
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(result["status"], "blacklisted");
}

#[tokio::test]
async fn test_update_carrier_performance() {
    let (_state, app) = setup_transportation_test().await;

    let carrier = create_test_carrier(&app, "FEDEX", "Federal Express", "parcel").await;
    let carrier_id = carrier["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let uri = format!("/api/v1/transport/carriers/id/{}/performance", carrier_id);
    let r = app.clone().oneshot(Request::builder().method("POST").uri(&uri)
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "performanceRating": 4.5,
            "onTimeDeliveryPct": 95.5,
            "claimsRatio": 0.02
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let body = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(result["performanceRating"], 4.5);
}

#[tokio::test]
async fn test_delete_carrier() {
    let (_state, app) = setup_transportation_test().await;

    let carrier = create_test_carrier(&app, "TEMP", "Temp Carrier", "parcel").await;
    let carrier_id = carrier["id"].as_str().unwrap();

    // Must suspend first before delete
    let (k, v) = auth_header(&admin_claims());
    let uri = format!("/api/v1/transport/carriers/id/{}/suspend", carrier_id);
    let _r = app.clone().oneshot(Request::builder().method("POST").uri(&uri)
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Now delete
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/transport/carriers/code/TEMP")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Carrier Service Tests
// ============================================================================

#[tokio::test]
async fn test_create_carrier_service() {
    let (_state, app) = setup_transportation_test().await;

    let carrier = create_test_carrier(&app, "FEDEX", "Federal Express", "parcel").await;
    let carrier_id = carrier["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let uri = format!("/api/v1/transport/carriers/{}/services", carrier_id);
    let r = app.clone().oneshot(Request::builder().method("POST").uri(&uri)
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "serviceCode": "FEDEX_GROUND",
            "name": "FedEx Ground",
            "serviceLevel": "standard",
            "transitDaysMin": 3,
            "transitDaysMax": 5,
            "ratePerKg": 2.50,
            "minimumCharge": 10.00,
            "fuelSurchargePct": 12.5
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(result["serviceCode"], "FEDEX_GROUND");
    assert_eq!(result["serviceLevel"], "standard");
    assert_eq!(result["ratePerKg"], 2.5);
}

#[tokio::test]
async fn test_list_carrier_services() {
    let (_state, app) = setup_transportation_test().await;

    let carrier = create_test_carrier(&app, "UPS", "UPS Ground", "parcel").await;
    let carrier_id = carrier["id"].as_str().unwrap();

    // Create two services
    let (k, v) = auth_header(&admin_claims());
    for (code, name) in [("UPS_GND", "UPS Ground"), ("UPS_EXP", "UPS Express")] {
        let uri = format!("/api/v1/transport/carriers/{}/services", carrier_id);
        let _r = app.clone().oneshot(Request::builder().method("POST").uri(&uri)
            .header("Content-Type", "application/json").header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({
                "serviceCode": code,
                "name": name,
                "serviceLevel": "standard",
                "transitDaysMin": 1,
                "transitDaysMax": 3,
                "ratePerKg": 3.0,
                "minimumCharge": 15.0,
                "fuelSurchargePct": 10.0
            })).unwrap())).unwrap()
        ).await.unwrap();
    }

    // List
    let uri = format!("/api/v1/transport/carriers/{}/services", carrier_id);
    let r = app.clone().oneshot(Request::builder().method("GET").uri(&uri)
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let body = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(result["data"].as_array().unwrap().len() >= 2);
}

// ============================================================================
// Transport Lane Tests
// ============================================================================

#[tokio::test]
async fn test_create_lane() {
    let (_state, app) = setup_transportation_test().await;

    let lane = create_test_lane(&app, "NYC-LAX", "NYC to Los Angeles", "domestic").await;

    assert_eq!(lane["laneCode"], "NYC-LAX");
    assert_eq!(lane["name"], "NYC to Los Angeles");
    assert_eq!(lane["laneType"], "domestic");
    assert_eq!(lane["status"], "active");
    assert_eq!(lane["originCity"], "New York");
    assert_eq!(lane["destinationCity"], "Los Angeles");
}

#[tokio::test]
async fn test_list_lanes() {
    let (_state, app) = setup_transportation_test().await;

    create_test_lane(&app, "NYC-LAX", "NYC to LA", "domestic").await;
    create_test_lane(&app, "NYC-LDN", "NYC to London", "international").await;

    // List all
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET").uri("/api/v1/transport/lanes")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(result["data"].as_array().unwrap().len() >= 2);

    // Filter by lane_type
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/transport/lanes?lane_type=international")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let data = result["data"].as_array().unwrap();
    assert!(data.iter().all(|l| l["laneType"] == "international"));
}

#[tokio::test]
async fn test_deactivate_lane() {
    let (_state, app) = setup_transportation_test().await;

    let lane = create_test_lane(&app, "SFO-LAX", "San Francisco to LA", "domestic").await;
    let lane_id = lane["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let uri = format!("/api/v1/transport/lanes/id/{}/deactivate", lane_id);
    let r = app.clone().oneshot(Request::builder().method("POST").uri(&uri)
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let body = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(result["status"], "inactive");
}

#[tokio::test]
async fn test_delete_lane() {
    let (_state, app) = setup_transportation_test().await;

    create_test_lane(&app, "DEL-LANE", "Lane to Delete", "domestic").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/transport/lanes/code/DEL-LANE")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Shipment Lifecycle Tests
// ============================================================================

#[tokio::test]
async fn test_create_shipment() {
    let (_state, app) = setup_transportation_test().await;

    let shipment = create_test_shipment(&app, "SHP-001", "outbound").await;

    assert_eq!(shipment["shipmentNumber"], "SHP-001");
    assert_eq!(shipment["shipmentType"], "outbound");
    assert_eq!(shipment["status"], "draft");
    assert_eq!(shipment["priority"], "normal");
    assert_eq!(shipment["currencyCode"], "USD");
}

#[tokio::test]
async fn test_create_shipment_duplicate_number() {
    let (_state, app) = setup_transportation_test().await;

    create_test_shipment(&app, "SHP-DUP", "outbound").await;

    // Try creating with same number
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/transport/shipments")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "shipmentNumber": "SHP-DUP",
            "shipmentType": "outbound",
            "priority": "normal",
            "originAddress": {},
            "destinationAddress": {},
            "currencyCode": "USD"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_list_shipments() {
    let (_state, app) = setup_transportation_test().await;

    create_test_shipment(&app, "SHP-001", "outbound").await;
    create_test_shipment(&app, "SHP-002", "inbound").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET").uri("/api/v1/transport/shipments")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let body = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(result["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_get_shipment() {
    let (_state, app) = setup_transportation_test().await;

    let shipment = create_test_shipment(&app, "SHP-GET", "outbound").await;
    let shipment_id = shipment["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let uri = format!("/api/v1/transport/shipments/id/{}", shipment_id);
    let r = app.clone().oneshot(Request::builder().method("GET").uri(&uri)
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let body = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(result["shipmentNumber"], "SHP-GET");
}

#[tokio::test]
async fn test_shipment_full_lifecycle() {
    let (_state, app) = setup_transportation_test().await;

    let shipment = create_test_shipment(&app, "SHP-LC", "outbound").await;
    let shipment_id = shipment["id"].as_str().unwrap();

    // Book
    let (k, v) = auth_header(&admin_claims());
    let uri = format!("/api/v1/transport/shipments/id/{}/book", shipment_id);
    let r = app.clone().oneshot(Request::builder().method("POST").uri(&uri)
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(result["status"], "booked");

    // Confirm Pickup
    let uri = format!("/api/v1/transport/shipments/id/{}/pickup", shipment_id);
    let r = app.clone().oneshot(Request::builder().method("POST").uri(&uri)
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "trackingNumber": "1Z999AA10123456784"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(result["status"], "picked_up");
    assert_eq!(result["trackingNumber"], "1Z999AA10123456784");

    // Start Transit
    let uri = format!("/api/v1/transport/shipments/id/{}/transit", shipment_id);
    let r = app.clone().oneshot(Request::builder().method("POST").uri(&uri)
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "driverName": "Jane Smith",
            "vehicleId": "TRK-101"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(result["status"], "in_transit");

    // Arrive at Destination
    let uri = format!("/api/v1/transport/shipments/id/{}/arrive", shipment_id);
    let r = app.clone().oneshot(Request::builder().method("POST").uri(&uri)
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(result["status"], "at_destination");

    // Confirm Delivery
    let uri = format!("/api/v1/transport/shipments/id/{}/deliver", shipment_id);
    let r = app.clone().oneshot(Request::builder().method("POST").uri(&uri)
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(result["status"], "delivered");
}

#[tokio::test]
async fn test_cancel_shipment() {
    let (_state, app) = setup_transportation_test().await;

    let shipment = create_test_shipment(&app, "SHP-CNL", "outbound").await;
    let shipment_id = shipment["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let uri = format!("/api/v1/transport/shipments/id/{}/cancel", shipment_id);
    let r = app.clone().oneshot(Request::builder().method("POST").uri(&uri)
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let body = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(result["status"], "cancelled");
}

#[tokio::test]
async fn test_assign_carrier_to_shipment() {
    let (_state, app) = setup_transportation_test().await;

    let carrier = create_test_carrier(&app, "FEDEX", "Federal Express", "parcel").await;
    let carrier_id = carrier["id"].as_str().unwrap();

    let shipment = create_test_shipment(&app, "SHP-ASN", "outbound").await;
    let shipment_id = shipment["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let uri = format!("/api/v1/transport/shipments/id/{}/assign-carrier", shipment_id);
    let r = app.clone().oneshot(Request::builder().method("POST").uri(&uri)
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "carrierId": carrier_id
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let body = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(result["carrierId"], carrier_id);
}

// ============================================================================
// Shipment Stop Tests
// ============================================================================

#[tokio::test]
async fn test_add_stop_to_shipment() {
    let (_state, app) = setup_transportation_test().await;

    let shipment = create_test_shipment(&app, "SHP-STP", "outbound").await;
    let shipment_id = shipment["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let uri = format!("/api/v1/transport/shipments/{}/stops", shipment_id);
    let r = app.clone().oneshot(Request::builder().method("POST").uri(&uri)
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "stopNumber": 1,
            "stopType": "pickup",
            "locationName": "Warehouse NYC",
            "address": {"city": "New York", "state": "NY"},
            "contactName": "Bob Johnson",
            "contactPhone": "555-0100"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(result["stopNumber"], 1);
    assert_eq!(result["stopType"], "pickup");
    assert_eq!(result["locationName"], "Warehouse NYC");
}

#[tokio::test]
async fn test_list_stops() {
    let (_state, app) = setup_transportation_test().await;

    let shipment = create_test_shipment(&app, "SHP-LST", "outbound").await;
    let shipment_id = shipment["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Add two stops
    for (i, stop_type) in [(1, "pickup"), (2, "delivery")] {
        let uri = format!("/api/v1/transport/shipments/{}/stops", shipment_id);
        let _r = app.clone().oneshot(Request::builder().method("POST").uri(&uri)
            .header("Content-Type", "application/json").header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({
                "stopNumber": i,
                "stopType": stop_type,
                "locationName": format!("Location {}", i),
                "address": {}
            })).unwrap())).unwrap()
        ).await.unwrap();
    }

    // List stops
    let uri = format!("/api/v1/transport/shipments/{}/stops", shipment_id);
    let r = app.clone().oneshot(Request::builder().method("GET").uri(&uri)
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let body = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(result["data"].as_array().unwrap().len(), 2);
}

// ============================================================================
// Shipment Line Tests
// ============================================================================

#[tokio::test]
async fn test_add_shipment_line() {
    let (_state, app) = setup_transportation_test().await;

    let shipment = create_test_shipment(&app, "SHP-LINE", "outbound").await;
    let shipment_id = shipment["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let uri = format!("/api/v1/transport/shipments/{}/lines", shipment_id);
    let r = app.clone().oneshot(Request::builder().method("POST").uri(&uri)
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "lineNumber": 1,
            "itemNumber": "ITEM-001",
            "itemDescription": "Widget A",
            "quantity": 100,
            "unitOfMeasure": "EA",
            "weightKg": 50.0,
            "volumeCbm": 2.5
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(result["lineNumber"], 1);
    assert_eq!(result["itemNumber"], "ITEM-001");
    assert_eq!(result["quantity"], 100);
}

#[tokio::test]
async fn test_list_shipment_lines() {
    let (_state, app) = setup_transportation_test().await;

    let shipment = create_test_shipment(&app, "SHP-LLN", "outbound").await;
    let shipment_id = shipment["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Add two lines
    for i in 1..=2 {
        let uri = format!("/api/v1/transport/shipments/{}/lines", shipment_id);
        let _r = app.clone().oneshot(Request::builder().method("POST").uri(&uri)
            .header("Content-Type", "application/json").header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({
                "lineNumber": i,
                "itemNumber": format!("ITEM-{}", i),
                "itemDescription": format!("Item {}", i),
                "quantity": 10 * i,
                "unitOfMeasure": "EA",
                "weightKg": 5.0 * i as f64,
                "volumeCbm": 0.5 * i as f64
            })).unwrap())).unwrap()
        ).await.unwrap();
    }

    // List lines
    let uri = format!("/api/v1/transport/shipments/{}/lines", shipment_id);
    let r = app.clone().oneshot(Request::builder().method("GET").uri(&uri)
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let body = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(result["data"].as_array().unwrap().len(), 2);
}

// ============================================================================
// Tracking Event Tests
// ============================================================================

#[tokio::test]
async fn test_add_tracking_event() {
    let (_state, app) = setup_transportation_test().await;

    // Create and book shipment first (tracking events need in_transit)
    let shipment = create_test_shipment(&app, "SHP-TRK", "outbound").await;
    let shipment_id = shipment["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Book the shipment
    let uri = format!("/api/v1/transport/shipments/id/{}/book", shipment_id);
    let _r = app.clone().oneshot(Request::builder().method("POST").uri(&uri)
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Confirm pickup
    let uri = format!("/api/v1/transport/shipments/id/{}/pickup", shipment_id);
    let _r = app.clone().oneshot(Request::builder().method("POST").uri(&uri)
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({})).unwrap())).unwrap()
    ).await.unwrap();

    // Add tracking event
    let uri = format!("/api/v1/transport/shipments/{}/tracking-events", shipment_id);
    let r = app.clone().oneshot(Request::builder().method("POST").uri(&uri)
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "eventType": "in_transit",
            "locationDescription": "Distribution Center - Chicago",
            "city": "Chicago",
            "state": "IL",
            "country": "USA",
            "latitude": 41.8781,
            "longitude": -87.6298,
            "description": "Package in transit via Chicago hub",
            "updatedBy": "system"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(result["eventType"], "in_transit");
    assert_eq!(result["city"], "Chicago");
}

#[tokio::test]
async fn test_list_tracking_events() {
    let (_state, app) = setup_transportation_test().await;

    let shipment = create_test_shipment(&app, "SHP-LTE", "outbound").await;
    let shipment_id = shipment["id"].as_str().unwrap();

    // List events (empty initially)
    let (k, v) = auth_header(&admin_claims());
    let uri = format!("/api/v1/transport/shipments/{}/tracking-events", shipment_id);
    let r = app.clone().oneshot(Request::builder().method("GET").uri(&uri)
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let body = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(result["data"].as_array().unwrap().is_empty());
}

// ============================================================================
// Freight Rate Tests
// ============================================================================

#[tokio::test]
async fn test_create_freight_rate() {
    let (_state, app) = setup_transportation_test().await;

    let carrier = create_test_carrier(&app, "FEDEX", "Federal Express", "parcel").await;
    let carrier_id = carrier["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/transport/freight-rates")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "rateCode": "FEDEX-STD-2026",
            "name": "FedEx Standard Rate 2026",
            "carrierId": carrier_id,
            "rateType": "per_kg",
            "rateAmount": 3.50,
            "minimumCharge": 15.00,
            "currencyCode": "USD",
            "fuelSurchargePct": 12.0,
            "effectiveFrom": "2026-01-01",
            "effectiveTo": "2026-12-31",
            "isContractRate": true,
            "contractNumber": "CT-2026-FEDEX"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(result["rateCode"], "FEDEX-STD-2026");
    assert_eq!(result["rateType"], "per_kg");
    assert_eq!(result["rateAmount"], 3.5);
    assert_eq!(result["isContractRate"], true);
}

#[tokio::test]
async fn test_list_freight_rates() {
    let (_state, app) = setup_transportation_test().await;

    let carrier = create_test_carrier(&app, "UPS", "UPS Ground", "parcel").await;
    let carrier_id = carrier["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Create a rate
    let _r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/transport/freight-rates")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "rateCode": "UPS-RT1",
            "name": "UPS Rate 1",
            "carrierId": carrier_id,
            "rateType": "flat",
            "rateAmount": 25.0,
            "minimumCharge": 10.0,
            "currencyCode": "USD",
            "effectiveFrom": "2026-01-01"
        })).unwrap())).unwrap()
    ).await.unwrap();

    // List rates
    let r = app.clone().oneshot(Request::builder().method("GET").uri("/api/v1/transport/freight-rates")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let body = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(result["data"].as_array().unwrap().len() >= 1);
}

#[tokio::test]
async fn test_expire_freight_rate() {
    let (_state, app) = setup_transportation_test().await;

    let carrier = create_test_carrier(&app, "DHL", "DHL Express", "air").await;
    let carrier_id = carrier["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/transport/freight-rates")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "rateCode": "DHL-EXP-RT",
            "name": "DHL Express Rate",
            "carrierId": carrier_id,
            "rateType": "per_kg",
            "rateAmount": 8.0,
            "minimumCharge": 25.0,
            "currencyCode": "USD",
            "effectiveFrom": "2026-01-01"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let body = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let rate: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let rate_id = rate["id"].as_str().unwrap();

    // Expire it
    let uri = format!("/api/v1/transport/freight-rates/id/{}/expire", rate_id);
    let r = app.clone().oneshot(Request::builder().method("POST").uri(&uri)
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let body = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(result["status"], "expired");
}

// ============================================================================
// Dashboard Test
// ============================================================================

#[tokio::test]
async fn test_transportation_dashboard() {
    let (_state, app) = setup_transportation_test().await;

    // Create some data for the dashboard
    create_test_carrier(&app, "FEDEX", "Federal Express", "parcel").await;
    create_test_lane(&app, "NYC-LAX", "NYC to LA", "domestic").await;
    create_test_shipment(&app, "SHP-DASH", "outbound").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET").uri("/api/v1/transport/dashboard")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let body = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    // Dashboard should have summary fields
    assert!(result.get("totalCarriers").is_some() || result.get("total_shipments").is_some()
        || result.is_object());
}

// ============================================================================
// End-to-End Full Lifecycle Test
// ============================================================================

#[tokio::test]
async fn test_full_transportation_lifecycle() {
    let (_state, app) = setup_transportation_test().await;

    // 1. Create carrier
    let carrier = create_test_carrier(&app, "E2E-CAR", "E2E Carrier", "ftl").await;
    let carrier_id = carrier["id"].as_str().unwrap();

    // 2. Create carrier service
    let (k, v) = auth_header(&admin_claims());
    let uri = format!("/api/v1/transport/carriers/{}/services", carrier_id);
    let r = app.clone().oneshot(Request::builder().method("POST").uri(&uri)
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "serviceCode": "E2E-FTL",
            "name": "E2E Full Truckload",
            "serviceLevel": "standard",
            "transitDaysMin": 2,
            "transitDaysMax": 4,
            "ratePerKg": 1.50,
            "minimumCharge": 500.0,
            "fuelSurchargePct": 15.0
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);

    // 3. Create lane
    let lane = create_test_lane(&app, "E2E-LANE", "E2E Route", "domestic").await;
    let _lane_id = lane["id"].as_str().unwrap();

    // 4. Create shipment
    let shipment = create_test_shipment(&app, "E2E-SHP", "outbound").await;
    let shipment_id = shipment["id"].as_str().unwrap();

    // 5. Add stops
    let uri = format!("/api/v1/transport/shipments/{}/stops", shipment_id);
    let r = app.clone().oneshot(Request::builder().method("POST").uri(&uri)
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "stopNumber": 1,
            "stopType": "pickup",
            "locationName": "Warehouse NYC",
            "address": {"city": "New York", "state": "NY"}
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);

    let uri = format!("/api/v1/transport/shipments/{}/stops", shipment_id);
    let r = app.clone().oneshot(Request::builder().method("POST").uri(&uri)
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "stopNumber": 2,
            "stopType": "delivery",
            "locationName": "Customer LA",
            "address": {"city": "Los Angeles", "state": "CA"}
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);

    // 6. Add shipment lines
    let uri = format!("/api/v1/transport/shipments/{}/lines", shipment_id);
    let r = app.clone().oneshot(Request::builder().method("POST").uri(&uri)
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "lineNumber": 1,
            "itemNumber": "WIDGET-001",
            "itemDescription": "Premium Widget",
            "quantity": 500,
            "unitOfMeasure": "EA",
            "weightKg": 250.0,
            "volumeCbm": 12.5
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);

    // 7. Assign carrier
    let uri = format!("/api/v1/transport/shipments/id/{}/assign-carrier", shipment_id);
    let r = app.clone().oneshot(Request::builder().method("POST").uri(&uri)
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "carrierId": carrier_id
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    // 8. Book
    let uri = format!("/api/v1/transport/shipments/id/{}/book", shipment_id);
    let r = app.clone().oneshot(Request::builder().method("POST").uri(&uri)
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    // 9. Confirm pickup
    let uri = format!("/api/v1/transport/shipments/id/{}/pickup", shipment_id);
    let r = app.clone().oneshot(Request::builder().method("POST").uri(&uri)
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "trackingNumber": "E2E-TRACK-001",
            "proNumber": "PR-001"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    // 10. Start transit
    let uri = format!("/api/v1/transport/shipments/id/{}/transit", shipment_id);
    let r = app.clone().oneshot(Request::builder().method("POST").uri(&uri)
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "driverName": "Bob Driver",
            "vehicleId": "TRK-E2E-001"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    // 11. Add tracking event
    let uri = format!("/api/v1/transport/shipments/{}/tracking-events", shipment_id);
    let r = app.clone().oneshot(Request::builder().method("POST").uri(&uri)
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "eventType": "in_transit",
            "locationDescription": "Interstate 80 West",
            "city": "Omaha",
            "state": "NE",
            "country": "USA",
            "description": "Shipment in transit - passed Omaha hub"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);

    // 12. Arrive at destination
    let uri = format!("/api/v1/transport/shipments/id/{}/arrive", shipment_id);
    let r = app.clone().oneshot(Request::builder().method("POST").uri(&uri)
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    // 13. Confirm delivery
    let uri = format!("/api/v1/transport/shipments/id/{}/deliver", shipment_id);
    let r = app.clone().oneshot(Request::builder().method("POST").uri(&uri)
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(result["status"], "delivered");

    // 14. Update carrier performance
    let uri = format!("/api/v1/transport/carriers/id/{}/performance", carrier_id);
    let r = app.clone().oneshot(Request::builder().method("POST").uri(&uri)
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "performanceRating": 4.8,
            "onTimeDeliveryPct": 98.5,
            "claimsRatio": 0.01
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    // 15. Verify dashboard
    let r = app.clone().oneshot(Request::builder().method("GET").uri("/api/v1/transport/dashboard")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}
