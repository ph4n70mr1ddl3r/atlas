//! Product Information Management (PIM) E2E Tests
//!
//! Tests for Oracle Fusion Cloud Product Hub:
//! - Product item CRUD and lifecycle
//! - Item category hierarchical management
//! - Item category assignments
//! - Item cross-references (GTIN, UPC, supplier)
//! - Item templates
//! - New Item Request (NIR) workflow: draft → submitted → approved → implemented
//! - PIM dashboard summary

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use uuid::Uuid;
use super::common::helpers::*;

async fn setup_pim_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    let app = build_router(state.clone());
    (state, app)
}

// ============================================================================
// Helper functions
// ============================================================================

async fn create_test_item(app: &axum::Router, item_number: &str) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/pim/items")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "item_number": item_number,
            "item_name": format!("Test Item {}", item_number),
            "description": "A test product item",
            "item_type": "finished_good",
            "primary_uom_code": "EA",
            "list_price": "99.99",
            "cost_price": "50.00",
            "currency_code": "USD",
            "inventory_item_flag": true,
            "purchasable_flag": true,
            "sellable_flag": true,
            "stock_enabled_flag": true,
            "invoice_enabled_flag": true
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_category(app: &axum::Router, code: &str, parent_id: Option<&str>) -> serde_json::Value {
    let mut body = json!({
        "code": code,
        "name": format!("Category {}", code),
        "description": "A test category"
    });
    if let Some(pid) = parent_id {
        body["parent_category_id"] = json!(pid);
    }

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/pim/categories")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&body).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_nir(app: &axum::Router, title: &str) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/pim/new-item-requests")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "title": title,
            "description": "New product request for testing",
            "item_type": "finished_good",
            "priority": "high",
            "requested_item_number": format!("ITEM-NIR-{}", Uuid::new_v4()),
            "requested_item_name": format!("New Product {}", title),
            "justification": "Market demand for new product",
            "currency_code": "USD"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_template(app: &axum::Router, code: &str) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/pim/templates")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": code,
            "name": format!("Template {}", code),
            "description": "A test item template",
            "item_type": "finished_good",
            "default_uom_code": "EA",
            "default_inventory_flag": true,
            "default_purchasable_flag": true,
            "default_sellable_flag": true,
            "default_stock_enabled_flag": true,
            "attribute_defaults": {"color": "default", "size": "M"}
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Product Item CRUD Tests
// ============================================================================

#[tokio::test]
async fn test_create_item() {
    let (_state, app) = setup_pim_test().await;
    let item = create_test_item(&app, "ITEM-001").await;

    assert_eq!(item["item_number"], "ITEM-001");
    assert_eq!(item["item_name"], "Test Item ITEM-001");
    assert_eq!(item["item_type"], "finished_good");
    assert_eq!(item["status"], "draft");
    assert_eq!(item["lifecycle_phase"], "concept");
    assert_eq!(item["primary_uom_code"], "EA");
    assert_eq!(item["currency_code"], "USD");
    assert!(item["id"].is_string());
}

#[tokio::test]
async fn test_create_item_validation_empty_number() {
    let (_state, app) = setup_pim_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/pim/items")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "item_number": "",
            "item_name": "Test",
            "item_type": "finished_good",
            "primary_uom_code": "EA",
            "currency_code": "USD"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_item_validation_invalid_type() {
    let (_state, app) = setup_pim_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/pim/items")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "item_number": "ITEM-BAD",
            "item_name": "Test",
            "item_type": "invalid_type",
            "primary_uom_code": "EA",
            "currency_code": "USD"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_item_duplicate_number() {
    let (_state, app) = setup_pim_test().await;
    create_test_item(&app, "ITEM-DUP").await;

    // Try creating with same number
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/pim/items")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "item_number": "ITEM-DUP",
            "item_name": "Duplicate",
            "item_type": "finished_good",
            "primary_uom_code": "EA",
            "currency_code": "USD"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_get_item() {
    let (_state, app) = setup_pim_test().await;
    let item = create_test_item(&app, "ITEM-GET").await;
    let id = item["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/pim/items/{}", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let fetched: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(fetched["item_number"], "ITEM-GET");
}

#[tokio::test]
async fn test_get_item_by_number() {
    let (_state, app) = setup_pim_test().await;
    create_test_item(&app, "ITEM-BYNUM").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/pim/items/by-number/ITEM-BYNUM")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let fetched: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(fetched["item_number"], "ITEM-BYNUM");
}

#[tokio::test]
async fn test_list_items() {
    let (_state, app) = setup_pim_test().await;
    create_test_item(&app, "ITEM-LIST-1").await;
    create_test_item(&app, "ITEM-LIST-2").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/pim/items")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(result["data"].as_array().unwrap().len() >= 2);
}

// ============================================================================
// Item Status & Lifecycle Tests
// ============================================================================

#[tokio::test]
async fn test_update_item_status_draft_to_active() {
    let (_state, app) = setup_pim_test().await;
    let item = create_test_item(&app, "ITEM-STATUS").await;
    let id = item["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/pim/items/{}/status", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "active"})).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(updated["status"], "active");
    // When activating from concept, lifecycle should auto-advance to production
    assert_eq!(updated["lifecycle_phase"], "production");
}

#[tokio::test]
async fn test_update_item_status_invalid_transition() {
    let (_state, app) = setup_pim_test().await;
    let item = create_test_item(&app, "ITEM-NOACTIVE").await;
    let id = item["id"].as_str().unwrap();

    // Cannot go directly from draft to obsolete
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/pim/items/{}/status", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "obsolete"})).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_update_lifecycle_phase() {
    let (_state, app) = setup_pim_test().await;
    let item = create_test_item(&app, "ITEM-LIFECYCLE").await;
    let id = item["id"].as_str().unwrap();

    // Advance concept -> design
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/pim/items/{}/lifecycle", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"lifecycle_phase": "design"})).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(updated["lifecycle_phase"], "design");
}

#[tokio::test]
async fn test_update_lifecycle_backward_rejected() {
    let (_state, app) = setup_pim_test().await;
    let item = create_test_item(&app, "ITEM-BACKWARD").await;
    let id = item["id"].as_str().unwrap();

    // Advance to design first
    let (k, v) = auth_header(&admin_claims());
    let _r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/pim/items/{}/lifecycle", id))
        .header("Content-Type", "application/json").header(&k.clone(), v.clone())
        .body(Body::from(serde_json::to_string(&json!({"lifecycle_phase": "design"})).unwrap())).unwrap()
    ).await.unwrap();

    // Try going backward: design -> concept (should fail)
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/pim/items/{}/lifecycle", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"lifecycle_phase": "concept"})).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_delete_item() {
    let (_state, app) = setup_pim_test().await;
    let item = create_test_item(&app, "ITEM-DEL").await;
    let id = item["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/pim/items/{}", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_delete_active_item_rejected() {
    let (_state, app) = setup_pim_test().await;
    let item = create_test_item(&app, "ITEM-ACTIVE-DEL").await;
    let id = item["id"].as_str().unwrap();

    // Activate first
    let (k, v) = auth_header(&admin_claims());
    let _r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/pim/items/{}/status", id))
        .header("Content-Type", "application/json").header(&k.clone(), v.clone())
        .body(Body::from(serde_json::to_string(&json!({"status": "active"})).unwrap())).unwrap()
    ).await.unwrap();

    // Now try to delete — should fail
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/pim/items/{}", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Category Tests
// ============================================================================

#[tokio::test]
async fn test_create_category() {
    let (_state, app) = setup_pim_test().await;
    let cat = create_test_category(&app, "ELEC", None).await;

    assert_eq!(cat["code"], "ELEC");
    assert_eq!(cat["name"], "Category ELEC");
    assert_eq!(cat["level_number"], 1);
    assert_eq!(cat["item_count"], 0);
}

#[tokio::test]
async fn test_create_category_hierarchy() {
    let (_state, app) = setup_pim_test().await;
    let parent = create_test_category(&app, "LAPTOP", None).await;
    let parent_id = parent["id"].as_str().unwrap();

    let child = create_test_category(&app, "GAMING-LAPTOP", Some(parent_id)).await;
    assert_eq!(child["level_number"], 2);
    assert_eq!(child["parent_category_id"], parent_id);
}

#[tokio::test]
async fn test_category_duplicate_code() {
    let (_state, app) = setup_pim_test().await;
    create_test_category(&app, "DUP-CAT", None).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/pim/categories")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "DUP-CAT",
            "name": "Duplicate Category"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_list_categories() {
    let (_state, app) = setup_pim_test().await;
    create_test_category(&app, "CAT-A", None).await;
    create_test_category(&app, "CAT-B", None).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/pim/categories")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(result["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_delete_category() {
    let (_state, app) = setup_pim_test().await;
    let cat = create_test_category(&app, "DEL-CAT", None).await;
    let id = cat["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/pim/categories/{}", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Cross-Reference Tests
// ============================================================================

#[tokio::test]
async fn test_create_cross_reference() {
    let (_state, app) = setup_pim_test().await;
    let item = create_test_item(&app, "ITEM-XREF").await;
    let item_id = item["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/pim/items/{}/cross-references", item_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "cross_reference_type": "gtin",
            "cross_reference_value": "01234567890123",
            "description": "GTIN-13 barcode",
            "source_system": "GS1"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let xref: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(xref["cross_reference_type"], "gtin");
    assert_eq!(xref["cross_reference_value"], "01234567890123");
}

#[tokio::test]
async fn test_cross_reference_invalid_type() {
    let (_state, app) = setup_pim_test().await;
    let item = create_test_item(&app, "ITEM-XREF-BAD").await;
    let item_id = item["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/pim/items/{}/cross-references", item_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "cross_reference_type": "invalid",
            "cross_reference_value": "123"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_list_cross_references() {
    let (_state, app) = setup_pim_test().await;
    let item = create_test_item(&app, "ITEM-XREF-LIST").await;
    let item_id = item["id"].as_str().unwrap();

    // Create two cross-refs
    let (k, v) = auth_header(&admin_claims());
    for (xref_type, value) in [("gtin", "1111111111111"), ("upc", "222222222222")] {
        let _ = app.clone().oneshot(Request::builder().method("POST")
            .uri(&format!("/api/v1/pim/items/{}/cross-references", item_id))
            .header("Content-Type", "application/json").header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({
                "cross_reference_type": xref_type,
                "cross_reference_value": value
            })).unwrap())).unwrap()
        ).await.unwrap();
    }

    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/pim/items/{}/cross-references", item_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(result["data"].as_array().unwrap().len() >= 2);
}

// ============================================================================
// Template Tests
// ============================================================================

#[tokio::test]
async fn test_create_template() {
    let (_state, app) = setup_pim_test().await;
    let tmpl = create_test_template(&app, "Tmpl-Std").await;

    assert_eq!(tmpl["code"], "Tmpl-Std");
    assert_eq!(tmpl["item_type"], "finished_good");
    assert_eq!(tmpl["default_uom_code"], "EA");
    assert!(tmpl["id"].is_string());
}

#[tokio::test]
async fn test_list_templates() {
    let (_state, app) = setup_pim_test().await;
    create_test_template(&app, "Tmpl-A").await;
    create_test_template(&app, "Tmpl-B").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/pim/templates")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(result["data"].as_array().unwrap().len() >= 2);
}

// ============================================================================
// New Item Request (NIR) Workflow Tests
// ============================================================================

#[tokio::test]
async fn test_nir_full_workflow() {
    let (_state, app) = setup_pim_test().await;

    // 1. Create NIR
    let nir = create_test_nir(&app, "Widget Pro").await;
    let id = nir["id"].as_str().unwrap();
    assert_eq!(nir["status"], "draft");
    assert_eq!(nir["priority"], "high");

    // 2. Submit for approval
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/pim/new-item-requests/{}/submit", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let submitted: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(submitted["status"], "submitted");

    // 3. Approve
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/pim/new-item-requests/{}/approve", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let approved: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(approved["status"], "approved");
    assert!(approved["approved_by"].is_string());

    // 4. Implement — creates the actual item
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/pim/new-item-requests/{}/implement", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let item: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(item["status"], "draft");
    assert!(item["item_number"].is_string());
}

#[tokio::test]
async fn test_nir_reject_workflow() {
    let (_state, app) = setup_pim_test().await;

    let nir = create_test_nir(&app, "Rejected Widget").await;
    let id = nir["id"].as_str().unwrap();

    // Submit
    let (k, v) = auth_header(&admin_claims());
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/pim/new-item-requests/{}/submit", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Reject
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/pim/new-item-requests/{}/reject", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "rejection_reason": "Not aligned with product strategy"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let rejected: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(rejected["status"], "rejected");
    assert_eq!(rejected["rejection_reason"], "Not aligned with product strategy");
}

#[tokio::test]
async fn test_nir_cancel_workflow() {
    let (_state, app) = setup_pim_test().await;

    let nir = create_test_nir(&app, "Cancelled Widget").await;
    let id = nir["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/pim/new-item-requests/{}/cancel", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let cancelled: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(cancelled["status"], "cancelled");
}

#[tokio::test]
async fn test_nir_cannot_approve_draft() {
    let (_state, app) = setup_pim_test().await;
    let nir = create_test_nir(&app, "Draft Widget").await;
    let id = nir["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/pim/new-item-requests/{}/approve", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_list_nirs() {
    let (_state, app) = setup_pim_test().await;
    create_test_nir(&app, "NIR List 1").await;
    create_test_nir(&app, "NIR List 2").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/pim/new-item-requests")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(result["data"].as_array().unwrap().len() >= 2);
}

// ============================================================================
// Dashboard Tests
// ============================================================================

#[tokio::test]
async fn test_pim_dashboard() {
    let (_state, app) = setup_pim_test().await;
    create_test_item(&app, "ITEM-DASH").await;
    create_test_category(&app, "DASH-CAT", None).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/pim/dashboard")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let dashboard: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(dashboard["total_items"].is_number());
    assert!(dashboard["active_items"].is_number());
    assert!(dashboard["draft_items"].is_number());
    assert!(dashboard["total_categories"].is_number());
    assert!(dashboard["pending_nir_count"].is_number());
}
