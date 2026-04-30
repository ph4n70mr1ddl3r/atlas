//! Supply Chain Planning (MRP) E2E Tests
//!
//! Tests for Oracle Fusion Supply Chain Planning:
//! - Planning Scenarios CRUD + lifecycle (draft → running → completed / cancelled)
//! - Planning Parameters CRUD
//! - Supply/Demand entry management
//! - MRP run (net supply vs demand, generate planned orders)
//! - Planned order firming and cancellation
//! - Planning exception management (resolve / dismiss)
//! - Planning dashboard
//! - Full end-to-end MRP flow

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_planning_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    setup_test_db(&state.db_pool).await;
    // Clean planning data for isolation
    sqlx::query("DELETE FROM _atlas.planning_exceptions").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM _atlas.planned_orders").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM _atlas.supply_demand_entries").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM _atlas.planning_scenarios").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM _atlas.planning_parameters").execute(&state.db_pool).await.ok();
    let app = build_router(state.clone());
    (state, app)
}

// ============================================================================
// Scenario CRUD Tests
// ============================================================================

#[tokio::test]
async fn test_create_planning_scenario() {
    let (_state, app) = setup_planning_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/scp/scenarios")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "name": "Q2 2026 MRP",
            "description": "Quarterly material requirements planning",
            "scenario_type": "mrp",
            "planning_horizon_days": 90,
            "planning_start_date": "2026-04-01",
            "include_existing_supply": true,
            "include_on_hand": true,
            "include_work_in_progress": true
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let scenario: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(scenario["name"], "Q2 2026 MRP");
    assert_eq!(scenario["scenarioType"], "mrp");
    assert_eq!(scenario["status"], "draft");
    assert_eq!(scenario["planningHorizonDays"], 90);
    assert!(scenario["scenarioNumber"].as_str().unwrap().starts_with("SCP-"));
}

#[tokio::test]
async fn test_list_scenarios() {
    let (_state, app) = setup_planning_test().await;
    let (k, v) = auth_header(&admin_claims());
    // Create two scenarios
    for name in &["Scenario A", "Scenario B"] {
        app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/scp/scenarios")
            .header("Content-Type", "application/json").header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({
                "name": name,
                "scenario_type": "mrp",
                "planning_horizon_days": 30
            })).unwrap())).unwrap()
        ).await.unwrap();
    }
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/scp/scenarios").header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let list: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let arr = list.as_array().unwrap();
    assert!(arr.len() >= 2, "Expected at least 2 scenarios, got {}", arr.len());
}

#[tokio::test]
async fn test_get_scenario() {
    let (_state, app) = setup_planning_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/scp/scenarios")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "name": "Test Scenario",
            "scenario_type": "production_planning"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let scenario: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();

    let id = scenario["id"].as_str().unwrap();
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/scp/scenarios/{}", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let fetched: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    assert_eq!(fetched["name"], "Test Scenario");
}

#[tokio::test]
async fn test_cancel_scenario() {
    let (_state, app) = setup_planning_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/scp/scenarios")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "name": "Cancel Me"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let scenario: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    let id = scenario["id"].as_str().unwrap();

    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/scp/scenarios/{}/cancel", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let cancelled: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    assert_eq!(cancelled["status"], "cancelled");
}

#[tokio::test]
async fn test_filter_scenarios_by_status() {
    let (_state, app) = setup_planning_test().await;
    let (k, v) = auth_header(&admin_claims());
    // Create scenario (draft)
    app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/scp/scenarios")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"name": "Draft One"})).unwrap())).unwrap()
    ).await.unwrap();

    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/scp/scenarios?status=draft")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let list: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    assert!(list.as_array().unwrap().len() >= 1);
}

// ============================================================================
// Planning Parameters Tests
// ============================================================================

#[tokio::test]
async fn test_upsert_planning_parameter() {
    let (_state, app) = setup_planning_test().await;
    let item_id = uuid::Uuid::new_v4();
    let supplier_id = uuid::Uuid::new_v4();
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/scp/parameters")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "item_id": item_id,
            "item_name": "Widget A",
            "item_number": "WDG-001",
            "planner_code": "PLN01",
            "planning_method": "mrp",
            "make_buy": "buy",
            "lead_time_days": 14,
            "safety_stock_quantity": "100",
            "min_order_quantity": "50",
            "lot_size_policy": "lot_for_lot",
            "default_supplier_id": supplier_id,
            "default_supplier_name": "Acme Suppliers"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let param: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    assert_eq!(param["itemId"], item_id.to_string());
    assert_eq!(param["planningMethod"], "mrp");
    assert_eq!(param["makeBuy"], "buy");
    assert_eq!(param["leadTimeDays"], 14);
    assert_eq!(param["safetyStockQuantity"], "100.00");
    assert_eq!(param["minOrderQuantity"], "50.00");
    assert_eq!(param["defaultSupplierName"], "Acme Suppliers");
}

#[tokio::test]
async fn test_list_parameters() {
    let (_state, app) = setup_planning_test().await;
    let (k, v) = auth_header(&admin_claims());
    for i in 0..3 {
        let item_id = uuid::Uuid::new_v4();
        app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/scp/parameters")
            .header("Content-Type", "application/json").header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({
                "item_id": item_id,
                "item_name": format!("Item {}", i),
                "planning_method": "mrp",
                "make_buy": "buy"
            })).unwrap())).unwrap()
        ).await.unwrap();
    }
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/scp/parameters").header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let list: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    assert!(list.as_array().unwrap().len() >= 3, "Expected at least 3 params");
    // Clean up for parallel test safety
    for p in list.as_array().unwrap() {
        if let Some(item_id) = p["itemId"].as_str() {
            sqlx::query("DELETE FROM _atlas.planning_parameters WHERE item_id = $1").bind(uuid::Uuid::parse_str(item_id).unwrap()).execute(&_state.db_pool).await.ok();
        }
    }
}

#[tokio::test]
async fn test_delete_parameter() {
    let (_state, app) = setup_planning_test().await;
    let item_id = uuid::Uuid::new_v4();
    let (k, v) = auth_header(&admin_claims());
    app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/scp/parameters")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "item_id": item_id,
            "planning_method": "mrp",
            "make_buy": "buy"
        })).unwrap())).unwrap()
    ).await.unwrap();

    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/scp/parameters/{}", item_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Supply/Demand Tests
// ============================================================================

#[tokio::test]
async fn test_create_supply_demand_entry() {
    let (_state, app) = setup_planning_test().await;
    let item_id = uuid::Uuid::new_v4();
    let (k, v) = auth_header(&admin_claims());

    // Create a scenario first
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/scp/scenarios")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"name": "SD Test"})).unwrap())).unwrap()
    ).await.unwrap();
    let scenario: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    let scenario_id = scenario["id"].as_str().unwrap();

    // Add supply
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/scp/supply-demand")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "scenario_id": scenario_id,
            "item_id": item_id,
            "item_name": "Widget A",
            "entry_type": "supply",
            "source_type": "on_hand",
            "quantity": "500",
            "due_date": "2026-04-01"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let entry: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    assert_eq!(entry["entryType"], "supply");
    assert_eq!(entry["sourceType"], "on_hand");
    assert_eq!(entry["quantity"], "500.00");

    // Add demand
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/scp/supply-demand")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "scenario_id": scenario_id,
            "item_id": item_id,
            "item_name": "Widget A",
            "entry_type": "demand",
            "source_type": "sales_order",
            "quantity": "800",
            "due_date": "2026-04-15"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);

    // List supply/demand
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/scp/scenarios/{}/supply-demand", scenario_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let list: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    assert_eq!(list.as_array().unwrap().len(), 2);
    // Clean up for parallel test safety
    for s in list.as_array().unwrap() {
        if let Some(id) = s["id"].as_str() {
            sqlx::query("DELETE FROM _atlas.planned_orders WHERE scenario_id = $1").bind(uuid::Uuid::parse_str(id).unwrap()).execute(&_state.db_pool).await.ok();
            sqlx::query("DELETE FROM _atlas.planning_exceptions WHERE scenario_id = $1").bind(uuid::Uuid::parse_str(id).unwrap()).execute(&_state.db_pool).await.ok();
            sqlx::query("DELETE FROM _atlas.supply_demand_entries WHERE scenario_id = $1").bind(uuid::Uuid::parse_str(id).unwrap()).execute(&_state.db_pool).await.ok();
        }
    }
}

#[tokio::test]
async fn test_filter_supply_demand_by_type() {
    let (_state, app) = setup_planning_test().await;
    let item_id = uuid::Uuid::new_v4();
    let (k, v) = auth_header(&admin_claims());

    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/scp/scenarios")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"name": "SD Filter Test"})).unwrap())).unwrap()
    ).await.unwrap();
    let scenario: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    let scenario_id = scenario["id"].as_str().unwrap();

    // Add supply and demand
    for (et, st) in [("supply", "on_hand"), ("demand", "sales_order")] {
        app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/scp/supply-demand")
            .header("Content-Type", "application/json").header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({
                "scenario_id": scenario_id,
                "item_id": item_id,
                "entry_type": et,
                "source_type": st,
                "quantity": "100",
                "due_date": "2026-05-01"
            })).unwrap())).unwrap()
        ).await.unwrap();
    }

    // Filter supply only
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/scp/scenarios/{}/supply-demand?entry_type=supply", scenario_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let list: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    assert_eq!(list.as_array().unwrap().len(), 1);
    assert_eq!(list[0]["entryType"], "supply");
}

// ============================================================================
// Full MRP Run Test
// ============================================================================

#[tokio::test]
async fn test_full_mrp_flow_with_shortage() {
    let (_state, app) = setup_planning_test().await;
    let item_id = uuid::Uuid::new_v4();
    let supplier_id = uuid::Uuid::new_v4();
    let (k, v) = auth_header(&admin_claims());

    // 1. Set up planning parameter for item
    app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/scp/parameters")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "item_id": item_id,
            "item_name": "Critical Widget",
            "item_number": "CW-001",
            "planning_method": "mrp",
            "make_buy": "buy",
            "lead_time_days": 7,
            "safety_stock_quantity": "50",
            "min_order_quantity": "100",
            "lot_size_policy": "lot_for_lot",
            "default_supplier_id": supplier_id,
            "default_supplier_name": "Widget Corp"
        })).unwrap())).unwrap()
    ).await.unwrap();

    // 2. Create scenario
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/scp/scenarios")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "name": "MRP Shortage Test",
            "planning_start_date": "2026-04-01",
            "planning_horizon_days": 90
        })).unwrap())).unwrap()
    ).await.unwrap();
    let scenario: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    let scenario_id = scenario["id"].as_str().unwrap();

    // 3. Add supply (on-hand: 200 units)
    app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/scp/supply-demand")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "scenario_id": scenario_id,
            "item_id": item_id,
            "item_name": "Critical Widget",
            "item_number": "CW-001",
            "entry_type": "supply",
            "source_type": "on_hand",
            "quantity": "200",
            "due_date": "2026-04-01"
        })).unwrap())).unwrap()
    ).await.unwrap();

    // 4. Add demand (sales orders: 350 units)
    app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/scp/supply-demand")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "scenario_id": scenario_id,
            "item_id": item_id,
            "item_name": "Critical Widget",
            "item_number": "CW-001",
            "entry_type": "demand",
            "source_type": "sales_order",
            "quantity": "350",
            "due_date": "2026-04-15"
        })).unwrap())).unwrap()
    ).await.unwrap();

    // 5. Run MRP
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/scp/scenarios/{}/run", scenario_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let completed: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    assert_eq!(completed["status"], "completed");
    // Should have planned orders and exceptions
    assert!(completed["totalPlannedOrders"].as_i64().unwrap() >= 1);
    assert!(completed["totalExceptions"].as_i64().unwrap() >= 1);

    // 6. Check planned orders
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/scp/scenarios/{}/orders", scenario_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let orders: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    let orders_arr = orders.as_array().unwrap();
    assert!(!orders_arr.is_empty());
    let first_order = &orders_arr[0];
    assert_eq!(first_order["status"], "unfirm");
    assert_eq!(first_order["orderType"], "buy");
    assert_eq!(first_order["itemId"], item_id.to_string());

    // 7. Check exceptions (should have shortage)
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/scp/scenarios/{}/exceptions", scenario_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let exceptions: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    let ex_arr = exceptions.as_array().unwrap();
    assert!(!ex_arr.is_empty());
    // Find the shortage exception
    let shortage: Vec<&serde_json::Value> = ex_arr.iter()
        .filter(|e| e["exceptionType"] == "shortage").collect();
    assert!(!shortage.is_empty());
    assert_eq!(shortage[0]["severity"], "critical");
}

#[tokio::test]
async fn test_mrp_with_no_shortage() {
    let (_state, app) = setup_planning_test().await;
    let item_id = uuid::Uuid::new_v4();
    let (k, v) = auth_header(&admin_claims());

    // Set up parameter
    app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/scp/parameters")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "item_id": item_id,
            "item_name": "Plentiful Item",
            "planning_method": "mrp",
            "make_buy": "buy",
            "lead_time_days": 5,
            "safety_stock_quantity": "10"
        })).unwrap())).unwrap()
    ).await.unwrap();

    // Create scenario
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/scp/scenarios")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "name": "No Shortage Test",
            "planning_start_date": "2026-04-01"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let scenario: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    let scenario_id = scenario["id"].as_str().unwrap();

    // Add supply >> demand
    app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/scp/supply-demand")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "scenario_id": scenario_id,
            "item_id": item_id,
            "entry_type": "supply",
            "source_type": "on_hand",
            "quantity": "1000",
            "due_date": "2026-04-01"
        })).unwrap())).unwrap()
    ).await.unwrap();
    app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/scp/supply-demand")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "scenario_id": scenario_id,
            "item_id": item_id,
            "entry_type": "demand",
            "source_type": "sales_order",
            "quantity": "100",
            "due_date": "2026-04-15"
        })).unwrap())).unwrap()
    ).await.unwrap();

    // Run MRP
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/scp/scenarios/{}/run", scenario_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let completed: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    assert_eq!(completed["status"], "completed");
    // No planned orders since supply >> demand
    assert_eq!(completed["totalPlannedOrders"].as_i64().unwrap(), 0);

    // Should have excess_supply warning
    let exceptions = get_exceptions(&app, scenario_id, &k, &v).await;
    let excess: Vec<&serde_json::Value> = exceptions.iter()
        .filter(|e| e["exceptionType"] == "excess_supply").collect();
    assert!(!excess.is_empty());
}

// ============================================================================
// Planned Order Tests
// ============================================================================

#[tokio::test]
async fn test_firm_and_cancel_planned_order() {
    let (_state, app) = setup_planning_test().await;
    let item_id = uuid::Uuid::new_v4();
    let (k, v) = auth_header(&admin_claims());

    // Setup: create scenario with shortage to generate planned order
    app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/scp/parameters")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "item_id": item_id,
            "planning_method": "mrp",
            "make_buy": "buy",
            "lead_time_days": 10,
            "safety_stock_quantity": "0"
        })).unwrap())).unwrap()
    ).await.unwrap();

    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/scp/scenarios")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "name": "Order Test",
            "planning_start_date": "2026-04-01"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let scenario: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    let scenario_id = scenario["id"].as_str().unwrap();

    // Add more demand than supply
    app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/scp/supply-demand")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "scenario_id": scenario_id,
            "item_id": item_id,
            "entry_type": "supply",
            "source_type": "on_hand",
            "quantity": "50",
            "due_date": "2026-04-01"
        })).unwrap())).unwrap()
    ).await.unwrap();
    app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/scp/supply-demand")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "scenario_id": scenario_id,
            "item_id": item_id,
            "entry_type": "demand",
            "source_type": "sales_order",
            "quantity": "200",
            "due_date": "2026-04-15"
        })).unwrap())).unwrap()
    ).await.unwrap();

    // Run MRP
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/scp/scenarios/{}/run", scenario_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Get planned orders
    let orders = get_planned_orders(&app, scenario_id, &k, &v).await;
    assert!(!orders.is_empty());

    let order_id = orders[0]["id"].as_str().unwrap();

    // Firm the order
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/scp/orders/{}/firm", order_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let firmed: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    assert_eq!(firmed["status"], "firmed", "Expected firmed status, got: {:?}", firmed);

    // Can't firm again
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/scp/orders/{}/firm", order_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    // Should get 409 CONFLICT since order is already firmed
    assert!(r.status() == StatusCode::CONFLICT || r.status() == StatusCode::BAD_REQUEST,
        "Expected 409 or 400, got {:?}", r.status());
}

#[tokio::test]
async fn test_cancel_planned_order() {
    let (_state, app) = setup_planning_test().await;
    let item_id = uuid::Uuid::new_v4();
    let (k, v) = auth_header(&admin_claims());

    app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/scp/parameters")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "item_id": item_id,
            "planning_method": "mrp",
            "make_buy": "buy"
        })).unwrap())).unwrap()
    ).await.unwrap();

    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/scp/scenarios")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "name": "Cancel Order Test",
            "planning_start_date": "2026-04-01"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let scenario: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    let scenario_id = scenario["id"].as_str().unwrap();

    app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/scp/supply-demand")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "scenario_id": scenario_id,
            "item_id": item_id,
            "entry_type": "demand",
            "source_type": "sales_order",
            "quantity": "100",
            "due_date": "2026-04-15"
        })).unwrap())).unwrap()
    ).await.unwrap();

    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/scp/scenarios/{}/run", scenario_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    let orders = get_planned_orders(&app, scenario_id, &k, &v).await;
    let order_id = orders[0]["id"].as_str().unwrap();

    // Cancel the order
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/scp/orders/{}/cancel", order_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let cancelled: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    assert_eq!(cancelled["status"], "cancelled");
}

#[tokio::test]
async fn test_filter_planned_orders_by_status() {
    let (_state, app) = setup_planning_test().await;
    let item_id = uuid::Uuid::new_v4();
    let (k, v) = auth_header(&admin_claims());

    app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/scp/parameters")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "item_id": item_id,
            "planning_method": "mrp",
            "make_buy": "buy"
        })).unwrap())).unwrap()
    ).await.unwrap();

    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/scp/scenarios")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "name": "Filter Test",
            "planning_start_date": "2026-04-01"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let scenario: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    let scenario_id = scenario["id"].as_str().unwrap();

    app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/scp/supply-demand")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "scenario_id": scenario_id,
            "item_id": item_id,
            "entry_type": "demand",
            "source_type": "sales_order",
            "quantity": "100",
            "due_date": "2026-04-15"
        })).unwrap())).unwrap()
    ).await.unwrap();

    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/scp/scenarios/{}/run", scenario_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Filter by unfirm status
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/scp/scenarios/{}/orders?status=unfirm", scenario_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let unfirm: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    assert!(!unfirm.as_array().unwrap().is_empty());
    assert!(unfirm.as_array().unwrap().iter().all(|o| o["status"] == "unfirm"));
}

// ============================================================================
// Exception Management Tests
// ============================================================================

#[tokio::test]
async fn test_resolve_exception() {
    let (_state, app) = setup_planning_test().await;
    let item_id = uuid::Uuid::new_v4();
    let (k, v) = auth_header(&admin_claims());

    app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/scp/parameters")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "item_id": item_id,
            "planning_method": "mrp",
            "make_buy": "buy"
        })).unwrap())).unwrap()
    ).await.unwrap();

    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/scp/scenarios")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "name": "Exception Resolve Test",
            "planning_start_date": "2026-04-01"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let scenario: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    let scenario_id = scenario["id"].as_str().unwrap();

    // Add demand only (shortage)
    app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/scp/supply-demand")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "scenario_id": scenario_id,
            "item_id": item_id,
            "entry_type": "demand",
            "source_type": "sales_order",
            "quantity": "100",
            "due_date": "2026-04-15"
        })).unwrap())).unwrap()
    ).await.unwrap();

    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/scp/scenarios/{}/run", scenario_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    let exceptions = get_exceptions(&app, scenario_id, &k, &v).await;
    let ex_id = exceptions[0]["id"].as_str().unwrap();

    // Resolve
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/scp/exceptions/{}/resolve", ex_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "resolution": "Ordered from alternate supplier"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let resolved: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    assert_eq!(resolved["resolutionStatus"], "resolved");
    assert_eq!(resolved["resolutionNotes"], "Ordered from alternate supplier");
}

#[tokio::test]
async fn test_dismiss_exception() {
    let (_state, app) = setup_planning_test().await;
    let item_id = uuid::Uuid::new_v4();
    let (k, v) = auth_header(&admin_claims());

    app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/scp/parameters")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "item_id": item_id,
            "planning_method": "mrp",
            "make_buy": "buy",
            "safety_stock_quantity": "0"
        })).unwrap())).unwrap()
    ).await.unwrap();

    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/scp/scenarios")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "name": "Dismiss Test",
            "planning_start_date": "2026-04-01"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let scenario: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    let scenario_id = scenario["id"].as_str().unwrap();

    app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/scp/supply-demand")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "scenario_id": scenario_id,
            "item_id": item_id,
            "entry_type": "demand",
            "source_type": "sales_order",
            "quantity": "100",
            "due_date": "2026-04-15"
        })).unwrap())).unwrap()
    ).await.unwrap();

    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/scp/scenarios/{}/run", scenario_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    let exceptions = get_exceptions(&app, scenario_id, &k, &v).await;
    let ex_id = exceptions[0]["id"].as_str().unwrap();

    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/scp/exceptions/{}/dismiss", ex_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "reason": "False alarm - inventory in transit"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let dismissed: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    assert_eq!(dismissed["resolutionStatus"], "dismissed");
}

#[tokio::test]
async fn test_filter_exceptions_by_severity() {
    let (_state, app) = setup_planning_test().await;
    let item_id = uuid::Uuid::new_v4();
    let (k, v) = auth_header(&admin_claims());

    app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/scp/parameters")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "item_id": item_id,
            "planning_method": "mrp",
            "make_buy": "buy"
        })).unwrap())).unwrap()
    ).await.unwrap();

    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/scp/scenarios")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "name": "Severity Filter",
            "planning_start_date": "2026-04-01"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let scenario: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    let scenario_id = scenario["id"].as_str().unwrap();

    app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/scp/supply-demand")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "scenario_id": scenario_id,
            "item_id": item_id,
            "entry_type": "demand",
            "source_type": "sales_order",
            "quantity": "100",
            "due_date": "2026-04-15"
        })).unwrap())).unwrap()
    ).await.unwrap();

    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/scp/scenarios/{}/run", scenario_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Filter critical only
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/scp/scenarios/{}/exceptions?severity=critical", scenario_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let critical: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    assert!(critical.as_array().unwrap().iter().all(|e| e["severity"] == "critical"));
}

// ============================================================================
// Dashboard Test
// ============================================================================

#[tokio::test]
async fn test_planning_dashboard() {
    let (_state, app) = setup_planning_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Initially empty
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/scp/dashboard").header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let dashboard: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    assert!(dashboard["totalScenarios"].as_i64().unwrap() >= 0);
    assert!(dashboard["totalPlannedOrders"].as_i64().unwrap() >= 0);
    assert!(dashboard["totalExceptions"].as_i64().unwrap() >= 0);
}

// ============================================================================
// Validation Tests
// ============================================================================

#[tokio::test]
async fn test_create_scenario_validation_empty_name() {
    let (_state, app) = setup_planning_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/scp/scenarios")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "name": ""
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_invalid_scenario_type() {
    let (_state, app) = setup_planning_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/scp/scenarios")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "name": "Bad Type",
            "scenario_type": "invalid_type"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_invalid_supply_demand_entry_type() {
    let (_state, app) = setup_planning_test().await;
    let item_id = uuid::Uuid::new_v4();
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/scp/supply-demand")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "item_id": item_id,
            "entry_type": "invalid",
            "source_type": "on_hand",
            "quantity": "100",
            "due_date": "2026-04-01"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_cannot_run_non_draft_scenario() {
    let (_state, app) = setup_planning_test().await;
    let (k, v) = auth_header(&admin_claims());

    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/scp/scenarios")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"name": "Already Done"})).unwrap())).unwrap()
    ).await.unwrap();
    let scenario: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    let id = scenario["id"].as_str().unwrap();

    // Cancel first
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/scp/scenarios/{}/cancel", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Try to run - should fail
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/scp/scenarios/{}/run", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    // Should get CONFLICT or BAD_REQUEST since scenario is cancelled
    assert!(r.status() == StatusCode::CONFLICT || r.status() == StatusCode::BAD_REQUEST,
        "Expected 409 or 400, got {:?}", r.status());
}

// ============================================================================
// Helpers
// ============================================================================

async fn get_planned_orders(
    app: &axum::Router,
    scenario_id: &str,
    key: &str,
    val: &str,
) -> Vec<serde_json::Value> {
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/scp/scenarios/{}/orders", scenario_id))
        .header(key, val).body(Body::empty()).unwrap()
    ).await.unwrap();
    serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap()
}

async fn get_exceptions(
    app: &axum::Router,
    scenario_id: &str,
    key: &str,
    val: &str,
) -> Vec<serde_json::Value> {
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/scp/scenarios/{}/exceptions", scenario_id))
        .header(key, val).body(Body::empty()).unwrap()
    ).await.unwrap();
    serde_json::from_slice(
        &axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()
    ).unwrap()
}
