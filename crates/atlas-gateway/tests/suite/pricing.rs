//! Advanced Pricing Management E2E Tests
//!
//! Tests for Oracle Fusion Cloud ERP Advanced Pricing:
//! - Price list CRUD and lifecycle
//! - Price list line management
//! - Tiered pricing (quantity breaks)
//! - Discount rule CRUD
//! - Charge definition CRUD
//! - Pricing strategy management
//! - Price calculation engine
//! - Dashboard summary
//! - Validation edge cases

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;
use uuid::Uuid;

async fn setup_pricing_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    // Run migration for pricing tables
    sqlx::query(include_str!("../../../../migrations/032_advanced_pricing.sql"))
        .execute(&state.db_pool)
        .await
        .ok(); // Ignore errors if tables already exist
    let app = build_router(state.clone());
    (state, app)
}

async fn create_test_price_list(
    app: &axum::Router,
    code: &str,
    name: &str,
    list_type: &str,
    currency: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "code": code,
        "name": name,
        "list_type": list_type,
        "currency_code": currency,
        "pricing_basis": "fixed",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/pricing/price-lists")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Failed to create price list");
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn add_test_price_line(
    app: &axum::Router,
    price_list_id: &str,
    item_code: &str,
    list_price: &str,
    unit_price: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "item_code": item_code,
        "item_description": format!("Test item {}", item_code),
        "pricing_unit_of_measure": "Ea",
        "list_price": list_price,
        "unit_price": unit_price,
        "cost_price": "0",
        "margin_percent": "0",
        "minimum_quantity": "1",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/pricing/price-lists/{}/lines", price_list_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Failed to add price list line");
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_discount_rule(
    app: &axum::Router,
    code: &str,
    name: &str,
    discount_type: &str,
    discount_value: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "code": code,
        "name": name,
        "discount_type": discount_type,
        "discount_value": discount_value,
        "application_method": "line",
        "stacking_rule": "exclusive",
        "priority": 10,
        "condition": {},
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/pricing/discount-rules")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Failed to create discount rule");
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_charge(
    app: &axum::Router,
    code: &str,
    name: &str,
    charge_type: &str,
    calculation_method: &str,
    charge_amount: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "code": code,
        "name": name,
        "charge_type": charge_type,
        "charge_category": "handling",
        "calculation_method": calculation_method,
        "charge_amount": charge_amount,
        "charge_percent": "0",
        "minimum_charge": "0",
        "taxable": false,
        "condition": {},
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/pricing/charges")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Failed to create charge definition");
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Price List CRUD Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_price_list_crud() {
    let (_state, app) = setup_pricing_test().await;

    // Create
    let pl = create_test_price_list(&app, "STD-SALE", "Standard Sale Price List", "sale", "USD").await;
    assert_eq!(pl["code"], "STD-SALE");
    assert_eq!(pl["name"], "Standard Sale Price List");
    assert_eq!(pl["list_type"], "sale");
    assert_eq!(pl["status"], "draft");

    // Get
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/pricing/price-lists/STD-SALE")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let got: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(got["code"], "STD-SALE");

    // List
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/pricing/price-lists")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let list: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(list["data"].as_array().unwrap().len() >= 1);

    // Delete
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/pricing/price-lists/STD-SALE")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
#[ignore]
async fn test_price_list_validation() {
    let (_state, app) = setup_pricing_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Empty code
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/pricing/price-lists")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "",
            "name": "Test",
            "list_type": "sale",
            "currency_code": "USD",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);

    // Invalid list type
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/pricing/price-lists")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "TEST",
            "name": "Test",
            "list_type": "teleport",
            "currency_code": "USD",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Price List Lifecycle Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_price_list_activate_lifecycle() {
    let (_state, app) = setup_pricing_test().await;

    // Create price list
    let pl = create_test_price_list(&app, "LIFECYCLE-PL", "Lifecycle Test", "sale", "USD").await;
    let pl_id = pl["id"].as_str().unwrap().to_string();

    let (k, v) = auth_header(&admin_claims());

    // Cannot activate without lines
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/pricing/price-lists/{}/activate", pl_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);

    // Add a line
    add_test_price_line(&app, &pl_id, "ITEM-001", "100.00", "90.00").await;

    // Now activate should succeed
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/pricing/price-lists/{}/activate", pl_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let activated: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(activated["status"], "active");

    // Deactivate
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/pricing/price-lists/{}/deactivate", pl_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let deactivated: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(deactivated["status"], "inactive");
}

// ============================================================================
// Price List Line Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_price_list_lines() {
    let (_state, app) = setup_pricing_test().await;

    let pl = create_test_price_list(&app, "LINES-PL", "Lines Test", "sale", "USD").await;
    let pl_id = pl["id"].as_str().unwrap().to_string();

    // Add two lines
    let line1 = add_test_price_line(&app, &pl_id, "WIDGET-A", "100.00", "90.00").await;
    assert_eq!(line1["line_number"], 1);
    assert_eq!(line1["item_code"], "WIDGET-A");

    let line2 = add_test_price_line(&app, &pl_id, "WIDGET-B", "50.00", "45.00").await;
    assert_eq!(line2["line_number"], 2);

    // List lines
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/pricing/price-lists/{}/lines", pl_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let lines: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(lines["data"].as_array().unwrap().len(), 2);

    // Delete a line
    let line_id = line1["id"].as_str().unwrap().to_string();
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/pricing/lines/{}", line_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Tiered Pricing Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_tiered_pricing() {
    let (_state, app) = setup_pricing_test().await;

    let pl = create_test_price_list(&app, "TIERED-PL", "Tiered Pricing Test", "sale", "USD").await;
    let pl_id = pl["id"].as_str().unwrap().to_string();
    let line = add_test_price_line(&app, &pl_id, "BULK-ITEM", "100.00", "100.00").await;
    let line_id = line["id"].as_str().unwrap().to_string();

    let (k, v) = auth_header(&admin_claims());

    // Add tiers: 1-9 at $100, 10-49 at $90, 50+ at $80
    let tier1 = json!({
        "from_quantity": "1",
        "to_quantity": "9",
        "price": "100.00",
        "discount_percent": "0",
        "price_type": "fixed",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/pricing/lines/{}/tiers", line_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&tier1).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);

    let tier2 = json!({
        "from_quantity": "10",
        "to_quantity": "49",
        "price": "90.00",
        "discount_percent": "0",
        "price_type": "fixed",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/pricing/lines/{}/tiers", line_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&tier2).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);

    let tier3 = json!({
        "from_quantity": "50",
        "price": "80.00",
        "discount_percent": "0",
        "price_type": "fixed",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/pricing/lines/{}/tiers", line_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&tier3).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);

    // List tiers
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/pricing/lines/{}/tiers", line_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let tiers: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(tiers["data"].as_array().unwrap().len(), 3);
}

// ============================================================================
// Discount Rule Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_discount_rule_crud() {
    let (_state, app) = setup_pricing_test().await;

    // Create
    let rule = create_test_discount_rule(&app, "SUMMER-10", "Summer 10% Off", "percentage", "10").await;
    assert_eq!(rule["code"], "SUMMER-10");
    assert_eq!(rule["discount_type"], "percentage");
    assert_eq!(rule["status"], "active");

    // Get
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/pricing/discount-rules/SUMMER-10")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let got: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(got["code"], "SUMMER-10");

    // List
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/pricing/discount-rules")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let list: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(list["data"].as_array().unwrap().len() >= 1);

    // Delete
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/pricing/discount-rules/SUMMER-10")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
#[ignore]
async fn test_discount_rule_validation() {
    let (_state, app) = setup_pricing_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Percentage > 100 should fail
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/pricing/discount-rules")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "BAD-PCT",
            "name": "Bad Percent",
            "discount_type": "percentage",
            "discount_value": "150",
            "application_method": "line",
            "stacking_rule": "exclusive",
            "priority": 10,
            "condition": {},
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);

    // Invalid discount type
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/pricing/discount-rules")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "BAD-TYPE",
            "name": "Bad Type",
            "discount_type": "bogo",
            "discount_value": "10",
            "application_method": "line",
            "stacking_rule": "exclusive",
            "priority": 10,
            "condition": {},
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Charge Definition Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_charge_definition_crud() {
    let (_state, app) = setup_pricing_test().await;

    // Create
    let charge = create_test_charge(&app, "SHIP-STD", "Standard Shipping", "shipping", "fixed", "9.99").await;
    assert_eq!(charge["code"], "SHIP-STD");
    assert_eq!(charge["charge_type"], "shipping");

    // Get
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/pricing/charges/SHIP-STD")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let got: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(got["code"], "SHIP-STD");

    // List
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/pricing/charges")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let list: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(list["data"].as_array().unwrap().len() >= 1);

    // List by type
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/pricing/charges?charge_type=shipping")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    // Delete
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/pricing/charges/SHIP-STD")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Pricing Strategy Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_pricing_strategy() {
    let (_state, app) = setup_pricing_test().await;

    let (k, v) = auth_header(&admin_claims());

    // Create a price list first
    let pl = create_test_price_list(&app, "STRAT-PL", "Strategy PL", "sale", "USD").await;
    let pl_id = pl["id"].as_str().unwrap().to_string();

    // Create strategy
    let payload = json!({
        "code": "STD-STRATEGY",
        "name": "Standard Pricing Strategy",
        "strategy_type": "price_list",
        "priority": 10,
        "condition": {},
        "price_list_id": pl_id,
        "markup_percent": "10",
        "markdown_percent": "0",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/pricing/strategies")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let strategy: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(strategy["code"], "STD-STRATEGY");
    assert_eq!(strategy["strategy_type"], "price_list");

    // List strategies
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/pricing/strategies")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let list: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(list["data"].as_array().unwrap().len() >= 1);
}

// ============================================================================
// Price Calculation Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_price_calculation_with_discount_and_charge() {
    let (_state, app) = setup_pricing_test().await;

    // Setup: create active price list with an item
    let pl = create_test_price_list(&app, "CALC-PL", "Calc Price List", "sale", "USD").await;
    let pl_id = pl["id"].as_str().unwrap().to_string();
    add_test_price_line(&app, &pl_id, "CALC-ITEM", "100.00", "100.00").await;

    // Activate the price list
    let (k, v) = auth_header(&admin_claims());
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/pricing/price-lists/{}/activate", pl_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Create a discount rule (10% off)
    create_test_discount_rule(&app, "CALC-DISC", "10% Off", "percentage", "10").await;

    // Create a charge (fixed $5.00 shipping)
    create_test_charge(&app, "CALC-SHIP", "Calc Shipping", "shipping", "fixed", "5.00").await;

    // Calculate price for 3 units
    let entity_id = Uuid::new_v4().to_string();
    let payload = json!({
        "item_code": "CALC-ITEM",
        "quantity": "3",
        "currency_code": "USD",
        "entity_type": "sales_order",
        "entity_id": entity_id,
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/pricing/calculate")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let result: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();

    // Verify calculation: list_price = 100, discount = 10, unit_selling = 90, extended = 90*3 + 5 = 275
    let list_price: f64 = result["list_price"].as_str().unwrap().parse().unwrap();
    let discount: f64 = result["discount_amount"].as_str().unwrap().parse().unwrap();
    let selling: f64 = result["unit_selling_price"].as_str().unwrap().parse().unwrap();
    let extended: f64 = result["extended_price"].as_str().unwrap().parse().unwrap();

    assert!((list_price - 100.0).abs() < 0.01, "Expected list_price 100, got {}", list_price);
    assert!((discount - 10.0).abs() < 0.01, "Expected discount 10, got {}", discount);
    assert!((selling - 90.0).abs() < 0.01, "Expected unit_selling 90, got {}", selling);
    assert!((extended - 275.0).abs() < 0.01, "Expected extended 275, got {}", extended);

    // Verify applied rules
    assert_eq!(result["applied_discount_rule_code"], "CALC-DISC");
    assert_eq!(result["applied_charge_code"], "CALC-SHIP");
    assert_eq!(result["applied_price_list_code"], "CALC-PL");

    // Verify calculation steps were recorded
    assert!(result["calculation_steps"].as_array().unwrap().len() >= 2);

    // Check calculation logs
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/pricing/calculation-logs?entity_type=sales_order&entity_id={}", entity_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let logs: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(logs["data"].as_array().unwrap().len() >= 1);
}

#[tokio::test]
#[ignore]
async fn test_price_calculation_item_not_found() {
    let (_state, app) = setup_pricing_test().await;

    let (k, v) = auth_header(&admin_claims());
    let entity_id = Uuid::new_v4().to_string();
    let payload = json!({
        "item_code": "NONEXISTENT-ITEM",
        "quantity": "1",
        "currency_code": "USD",
        "entity_type": "test",
        "entity_id": entity_id,
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/pricing/calculate")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Dashboard Test
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_pricing_dashboard() {
    let (_state, app) = setup_pricing_test().await;

    // Create some data
    create_test_price_list(&app, "DASH-PL", "Dashboard PL", "sale", "USD").await;
    create_test_discount_rule(&app, "DASH-DISC", "Dashboard Discount", "percentage", "5").await;
    create_test_charge(&app, "DASH-CHG", "Dashboard Charge", "handling", "fixed", "2.50").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/pricing/dashboard")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let summary: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();

    assert!(summary["total_price_lists"].as_i64().unwrap() >= 1);
    assert!(summary["total_discount_rules"].as_i64().unwrap() >= 1);
    assert!(summary["total_charge_definitions"].as_i64().unwrap() >= 1);
}

// ============================================================================
// Price List Filter Test
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_list_price_lists_by_type() {
    let (_state, app) = setup_pricing_test().await;

    create_test_price_list(&app, "FILTER-SALE", "Sale PL", "sale", "USD").await;
    create_test_price_list(&app, "FILTER-PURCH", "Purchase PL", "purchase", "EUR").await;

    let (k, v) = auth_header(&admin_claims());

    // List all
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/pricing/price-lists")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let all: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(all["data"].as_array().unwrap().len() >= 2);

    // Filter by sale type
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/pricing/price-lists?list_type=sale")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let sale_only: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let items = sale_only["data"].as_array().unwrap();
    assert!(items.len() >= 1);
    assert!(items.iter().all(|pl| pl["list_type"] == "sale"));
}

// ============================================================================
// Edge Cases
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_negative_price_rejected() {
    let (_state, app) = setup_pricing_test().await;

    let pl = create_test_price_list(&app, "NEG-PL", "Negative Test", "sale", "USD").await;
    let pl_id = pl["id"].as_str().unwrap().to_string();

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "item_code": "NEG-ITEM",
        "list_price": "-10.00",
        "unit_price": "10.00",
        "minimum_quantity": "1",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/pricing/price-lists/{}/lines", pl_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}
