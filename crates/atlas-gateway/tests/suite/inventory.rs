//! Inventory Management E2E Tests
//!
//! Tests for Oracle Fusion Cloud ERP Inventory Management:
//! - Inventory organization CRUD
//! - Item category and item creation
//! - Subinventory management
//! - Receiving items into inventory
//! - Issuing items from inventory
//! - Transferring items between subinventories
//! - Adjusting on-hand quantities
//! - On-hand balance tracking
//! - Dashboard summary
//! - Validation edge cases

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_inventory_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    // Run migration for inventory management tables
    sqlx::query(include_str!("../../../../migrations/030_inventory_management.sql"))
        .execute(&state.db_pool)
        .await
        .ok(); // Ignore errors if tables already exist
    let app = build_router(state.clone());
    (state, app)
}

async fn create_test_inv_org(
    app: &axum::Router,
    code: &str,
    name: &str,
    org_type: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "code": code,
        "name": name,
        "org_type": org_type,
        "default_currency_code": "USD",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/inventory/organizations")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Failed to create inventory org");
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_item(
    app: &axum::Router,
    item_code: &str,
    name: &str,
    item_type: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "item_code": item_code,
        "name": name,
        "item_type": item_type,
        "uom": "EA",
        "list_price": "100.00",
        "standard_cost": "50.00",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/inventory/items")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Failed to create item");
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_subinventory(
    app: &axum::Router,
    inv_org_id: &str,
    code: &str,
    name: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "inventory_org_id": inv_org_id,
        "code": code,
        "name": name,
        "subinventory_type": "storage",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/inventory/subinventories")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Failed to create subinventory");
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn receive_test_item(
    app: &axum::Router,
    item_id: &str,
    inv_org_id: &str,
    subinv_id: &str,
    quantity: &str,
    unit_cost: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "item_id": item_id,
        "to_inventory_org_id": inv_org_id,
        "to_subinventory_id": subinv_id,
        "quantity": quantity,
        "uom": "EA",
        "unit_cost": unit_cost,
        "source_type": "manual",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/inventory/transactions/receive")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Failed to receive item");
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Inventory Organization Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_inventory_org_crud() {
    let (_state, app) = setup_inventory_test().await;

    // Create
    let org = create_test_inv_org(&app, "WH-001", "Main Warehouse", "warehouse").await;
    assert_eq!(org["code"], "WH-001");
    assert_eq!(org["name"], "Main Warehouse");
    assert_eq!(org["org_type"], "warehouse");
    assert_eq!(org["is_active"], true);

    // Get
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/inventory/organizations/WH-001")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let got: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(got["code"], "WH-001");

    // List
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/inventory/organizations")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let list: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(list["data"].as_array().unwrap().len() >= 1);

    // Delete
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/inventory/organizations/WH-001")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);

    // Verify deleted
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/inventory/organizations/WH-001")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
#[ignore]
async fn test_inventory_org_validation() {
    let (_state, app) = setup_inventory_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Empty code
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/inventory/organizations")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "",
            "name": "Test",
            "org_type": "warehouse",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);

    // Invalid org type
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/inventory/organizations")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "TEST",
            "name": "Test",
            "org_type": "space_station",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Item Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_item_crud() {
    let (_state, app) = setup_inventory_test().await;

    // Create item
    let item = create_test_item(&app, "LAPTOP-001", "Dell Latitude Laptop", "inventory").await;
    assert_eq!(item["item_code"], "LAPTOP-001");
    assert_eq!(item["name"], "Dell Latitude Laptop");
    assert_eq!(item["item_type"], "inventory");
    assert_eq!(item["uom"], "EA");
    assert_eq!(item["is_active"], true);
    let item_id = item["id"].as_str().unwrap().to_string();

    // Get by ID
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/inventory/items/{}", item_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let got: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(got["item_code"], "LAPTOP-001");

    // List items
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/inventory/items")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let list: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(list["data"].as_array().unwrap().len() >= 1);

    // List with filter
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/inventory/items?item_type=inventory")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}

// ============================================================================
// Receive & Issue Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_receive_and_issue_full_lifecycle() {
    let (_state, app) = setup_inventory_test().await;

    // Setup: Create org, subinventory, and item
    let org = create_test_inv_org(&app, "WH-01", "Warehouse 1", "warehouse").await;
    let org_id = org["id"].as_str().unwrap().to_string();

    let sub = create_test_subinventory(&app, &org_id, "STORE-01", "Main Storage").await;
    let sub_id = sub["id"].as_str().unwrap().to_string();

    let item = create_test_item(&app, "WIDGET-001", "Widget A", "inventory").await;
    let item_id = item["id"].as_str().unwrap().to_string();

    // Step 1: Receive 100 units at $10.00 each
    let txn = receive_test_item(&app, &item_id, &org_id, &sub_id, "100", "10.00").await;
    assert_eq!(txn["transaction_action"], "receive");
    assert_eq!(txn["status"], "processed");
    assert!(txn["transaction_number"].as_str().unwrap().starts_with("RCV-"));
    let total_cost: f64 = txn["total_cost"].as_str().unwrap().parse().unwrap();
    assert!((total_cost - 1000.0).abs() < 0.01, "Expected 1000.0, got {}", total_cost);

    // Verify on-hand balance
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/inventory/on-hand?item_id={}", item_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let balances: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let on_hand = &balances["data"][0];
    let qty: f64 = on_hand["quantity"].as_str().unwrap().parse().unwrap();
    assert!((qty - 100.0).abs() < 0.01, "Expected 100, got {}", qty);

    // Step 2: Issue 30 units
    let payload = json!({
        "item_id": item_id,
        "from_inventory_org_id": org_id,
        "from_subinventory_id": sub_id,
        "quantity": "30",
        "uom": "EA",
        "unit_cost": "10.00",
        "source_type": "manual",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/inventory/transactions/issue")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let issue_txn: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(issue_txn["transaction_action"], "issue");

    // Verify remaining on-hand (should be 70)
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/inventory/on-hand?item_id={}", item_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    let balances: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let remaining: f64 = balances["data"][0]["quantity"].as_str().unwrap().parse().unwrap();
    assert!((remaining - 70.0).abs() < 0.01, "Expected 70, got {}", remaining);

    // Step 3: Try to issue more than available (should fail)
    let payload = json!({
        "item_id": item_id,
        "from_inventory_org_id": org_id,
        "from_subinventory_id": sub_id,
        "quantity": "200",
        "uom": "EA",
        "unit_cost": "10.00",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/inventory/transactions/issue")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Transfer Test
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_transfer_between_subinventories() {
    let (_state, app) = setup_inventory_test().await;

    // Setup
    let org = create_test_inv_org(&app, "WH-02", "Warehouse 2", "warehouse").await;
    let org_id = org["id"].as_str().unwrap().to_string();

    let sub1 = create_test_subinventory(&app, &org_id, "STORE-A", "Storage A").await;
    let sub1_id = sub1["id"].as_str().unwrap().to_string();

    let sub2 = create_test_subinventory(&app, &org_id, "STORE-B", "Storage B").await;
    let sub2_id = sub2["id"].as_str().unwrap().to_string();

    let item = create_test_item(&app, "BOLT-001", "Steel Bolt M8", "inventory").await;
    let item_id = item["id"].as_str().unwrap().to_string();

    // Receive into sub1
    receive_test_item(&app, &item_id, &org_id, &sub1_id, "500", "0.50").await;

    // Transfer from sub1 to sub2
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "item_id": item_id,
        "from_inventory_org_id": org_id,
        "from_subinventory_id": sub1_id,
        "to_inventory_org_id": org_id,
        "to_subinventory_id": sub2_id,
        "quantity": "200",
        "uom": "EA",
        "unit_cost": "0.50",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/inventory/transactions/transfer")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let transfer: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(transfer["transaction_action"], "transfer");

    // Verify both locations
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/inventory/on-hand?item_id={}", item_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    let balances: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();

    // Should have two balance entries
    let entries = balances["data"].as_array().unwrap();
    assert_eq!(entries.len(), 2, "Expected 2 balance entries, got {}", entries.len());
}

// ============================================================================
// Adjustment Test
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_quantity_adjustment() {
    let (_state, app) = setup_inventory_test().await;

    let org = create_test_inv_org(&app, "WH-03", "Warehouse 3", "warehouse").await;
    let org_id = org["id"].as_str().unwrap().to_string();
    let sub = create_test_subinventory(&app, &org_id, "STORE-01", "Main").await;
    let sub_id = sub["id"].as_str().unwrap().to_string();
    let item = create_test_item(&app, "CABLE-001", "USB Cable", "inventory").await;
    let item_id = item["id"].as_str().unwrap().to_string();

    // Receive 50 units
    receive_test_item(&app, &item_id, &org_id, &sub_id, "50", "5.00").await;

    // Positive adjustment (+10)
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "item_id": item_id,
        "inventory_org_id": org_id,
        "subinventory_id": sub_id,
        "quantity_delta": "10",
        "uom": "EA",
        "unit_cost": "5.00",
        "reason_name": "Found during stock check",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/inventory/transactions/adjust")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let adj: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(adj["transaction_action"], "adjustment");
    assert_eq!(adj["notes"], "Found during stock check");

    // Verify on-hand = 60
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/inventory/on-hand?item_id={}", item_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    let balances: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let qty: f64 = balances["data"][0]["quantity"].as_str().unwrap().parse().unwrap();
    assert!((qty - 60.0).abs() < 0.01, "Expected 60, got {}", qty);

    // Negative adjustment (-5)
    let payload = json!({
        "item_id": item_id,
        "inventory_org_id": org_id,
        "subinventory_id": sub_id,
        "quantity_delta": "-5",
        "uom": "EA",
        "unit_cost": "5.00",
        "reason_name": "Damaged units",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/inventory/transactions/adjust")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);

    // Verify on-hand = 55
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/inventory/on-hand?item_id={}", item_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    let balances: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let qty: f64 = balances["data"][0]["quantity"].as_str().unwrap().parse().unwrap();
    assert!((qty - 55.0).abs() < 0.01, "Expected 55, got {}", qty);
}

// ============================================================================
// Transaction Listing Test
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_transaction_history() {
    let (_state, app) = setup_inventory_test().await;

    let org = create_test_inv_org(&app, "WH-04", "Warehouse 4", "warehouse").await;
    let org_id = org["id"].as_str().unwrap().to_string();
    let sub = create_test_subinventory(&app, &org_id, "STORE-01", "Main").await;
    let sub_id = sub["id"].as_str().unwrap().to_string();
    let item = create_test_item(&app, "MONITOR-001", "27\" Monitor", "inventory").await;
    let item_id = item["id"].as_str().unwrap().to_string();

    // Receive then issue
    receive_test_item(&app, &item_id, &org_id, &sub_id, "10", "200.00").await;

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "item_id": item_id,
        "from_inventory_org_id": org_id,
        "from_subinventory_id": sub_id,
        "quantity": "3",
        "uom": "EA",
        "unit_cost": "200.00",
        "source_type": "sales_order",
    });
    app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/inventory/transactions/issue")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();

    // List all transactions
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/inventory/transactions?item_id={}", item_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let txns: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(txns["data"].as_array().unwrap().len(), 2);

    // Filter by action
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/inventory/transactions?item_id={}&transaction_action=receive", item_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let receive_txns: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(receive_txns["data"].as_array().unwrap().len(), 1);
    assert_eq!(receive_txns["data"][0]["transaction_action"], "receive");
}

// ============================================================================
// Dashboard Test
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_inventory_dashboard() {
    let (_state, app) = setup_inventory_test().await;

    // Create some data
    create_test_inv_org(&app, "WH-DASH", "Dashboard Warehouse", "warehouse").await;
    create_test_item(&app, "ITEM-D1", "Dashboard Item 1", "inventory").await;
    create_test_item(&app, "ITEM-D2", "Dashboard Item 2", "non_inventory").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/inventory/dashboard")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let summary: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();

    assert_eq!(summary["total_items"], 2);
    assert_eq!(summary["total_organizations"], 1);
    assert_eq!(summary["active_items"], 2);
}

// ============================================================================
// Validation Edge Cases
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_receive_validation_errors() {
    let (_state, app) = setup_inventory_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Zero quantity
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/inventory/transactions/receive")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "item_id": "00000000-0000-0000-0000-000000000001",
            "to_inventory_org_id": "00000000-0000-0000-0000-000000000002",
            "to_subinventory_id": "00000000-0000-0000-0000-000000000003",
            "quantity": "0",
            "unit_cost": "10.00",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);

    // Negative quantity for receive
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/inventory/transactions/receive")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "item_id": "00000000-0000-0000-0000-000000000001",
            "to_inventory_org_id": "00000000-0000-0000-0000-000000000002",
            "to_subinventory_id": "00000000-0000-0000-0000-000000000003",
            "quantity": "-5",
            "unit_cost": "10.00",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore]
async fn test_item_type_validation() {
    let (_state, app) = setup_inventory_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Invalid item type
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/inventory/items")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "item_code": "BAD-ITEM",
            "name": "Bad Item",
            "item_type": "nuclear_weapon",
            "uom": "EA",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore]
async fn test_zero_adjustment_rejected() {
    let (_state, app) = setup_inventory_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Adjustment with zero delta should fail
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/inventory/transactions/adjust")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "item_id": "00000000-0000-0000-0000-000000000001",
            "inventory_org_id": "00000000-0000-0000-0000-000000000002",
            "subinventory_id": "00000000-0000-0000-0000-000000000003",
            "quantity_delta": "0",
            "uom": "EA",
            "unit_cost": "10.00",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}
