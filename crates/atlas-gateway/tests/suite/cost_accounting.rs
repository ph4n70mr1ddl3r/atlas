//! Cost Accounting E2E Tests
//!
//! Tests for Oracle Fusion Cost Management > Cost Accounting:
//! - Cost Books CRUD and lifecycle (active ↔ inactive)
//! - Cost Elements CRUD and validation
//! - Cost Profiles CRUD
//! - Standard Costs CRUD and lifecycle (active → superseded, pending → deleted)
//! - Cost Adjustments full lifecycle (draft → submitted → approved → posted)
//! - Cost Adjustments rejection path (draft → submitted → rejected)
//! - Cost Adjustment Lines
//! - Cost Variances and analysis
//! - Cost Accounting Dashboard
//! - Validation edge cases and error handling

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_cost_accounting_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    let app = build_router(state.clone());
    (state, app)
}

// ============================================================================
// Helper functions
// ============================================================================

async fn create_test_cost_book(app: &axum::Router, code: &str, name: &str, method: &str) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/cost-accounting/books")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "code": code,
                        "name": name,
                        "costingMethod": method,
                        "currencyCode": "USD",
                        "effectiveFrom": "2026-01-01",
                        "effectiveTo": "2026-12-31"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = resp.status();
    let b = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    if status != StatusCode::CREATED {
        panic!("Expected CREATED for cost book but got {}: {}", status, String::from_utf8_lossy(&b));
    }
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_cost_element(
    app: &axum::Router,
    code: &str,
    name: &str,
    element_type: &str,
    cost_book_id: Option<&str>,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let mut payload = json!({
        "code": code,
        "name": name,
        "elementType": element_type,
        "defaultRate": "10.00"
    });
    if let Some(cb_id) = cost_book_id {
        payload["costBookId"] = json!(cb_id);
    }
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/cost-accounting/elements")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(serde_json::to_string(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = resp.status();
    let b = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    if status != StatusCode::CREATED {
        panic!("Expected CREATED for cost element but got {}: {}", status, String::from_utf8_lossy(&b));
    }
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_standard_cost(
    app: &axum::Router,
    book_id: &str,
    element_id: &str,
    item_id: &str,
    cost: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/cost-accounting/standard-costs")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "costBookId": book_id,
                        "costElementId": element_id,
                        "itemId": item_id,
                        "standardCost": cost,
                        "currencyCode": "USD",
                        "effectiveDate": "2026-01-01"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = resp.status();
    let b = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    if status != StatusCode::CREATED {
        panic!("Expected CREATED for standard cost but got {}: {}", status, String::from_utf8_lossy(&b));
    }
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_adjustment(
    app: &axum::Router,
    book_id: &str,
    adj_type: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/cost-accounting/adjustments")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "costBookId": book_id,
                        "adjustmentType": adj_type,
                        "description": "Test adjustment",
                        "reason": "Cost update",
                        "currencyCode": "USD",
                        "effectiveDate": "2026-04-01"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = resp.status();
    let b = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    if status != StatusCode::CREATED {
        panic!("Expected CREATED for adjustment but got {}: {}", status, String::from_utf8_lossy(&b));
    }
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Cost Books
// ============================================================================

#[tokio::test]
async fn test_cost_book_crud() {
    let (_state, app) = setup_cost_accounting_test().await;

    // Create
    let book = create_test_cost_book(&app, "STD-BOOK-01", "Standard Cost Book", "standard").await;
    assert_eq!(book["code"], "STD-BOOK-01");
    assert_eq!(book["name"], "Standard Cost Book");
    assert_eq!(book["costingMethod"], "standard");
    assert_eq!(book["isActive"], true);
    let book_id = book["id"].as_str().unwrap();

    // Get
    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!("/api/v1/cost-accounting/books/{}", book_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let got: serde_json::Value =
        serde_json::from_slice(&axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(got["code"], "STD-BOOK-01");

    // List
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/cost-accounting/books")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let list: serde_json::Value =
        serde_json::from_slice(&axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert!(list["data"].as_array().unwrap().len() >= 1);

    // Update
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(&format!("/api/v1/cost-accounting/books/{}", book_id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({"name": "Updated Book"})).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let updated: serde_json::Value =
        serde_json::from_slice(&axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(updated["name"], "Updated Book");

    // Delete
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(&format!("/api/v1/cost-accounting/books/{}", book_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_cost_book_activate_deactivate() {
    let (_state, app) = setup_cost_accounting_test().await;

    let book = create_test_cost_book(&app, "STD-LIFE", "Lifecycle Book", "average").await;
    let book_id = book["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    // Deactivate
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/cost-accounting/books/{}/deactivate", book_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let deactivated: serde_json::Value =
        serde_json::from_slice(&axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(deactivated["isActive"], false);

    // Activate
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/cost-accounting/books/{}/activate", book_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let activated: serde_json::Value =
        serde_json::from_slice(&axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(activated["isActive"], true);
}

#[tokio::test]
async fn test_cost_book_validation_duplicate_code() {
    let (_state, app) = setup_cost_accounting_test().await;

    create_test_cost_book(&app, "DUP-CODE", "First Book", "standard").await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/cost-accounting/books")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "code": "DUP-CODE",
                        "name": "Second Book",
                        "costingMethod": "standard"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_cost_book_invalid_costing_method() {
    let (_state, app) = setup_cost_accounting_test().await;
    let (k, v) = auth_header(&admin_claims());

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/cost-accounting/books")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "code": "BAD-METHOD",
                        "name": "Bad Method",
                        "costingMethod": "weighted_average"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Cost Elements
// ============================================================================

#[tokio::test]
async fn test_cost_element_update() {
    let (_state, app) = setup_cost_accounting_test().await;

    let elem = create_test_cost_element(&app, "UPD-ELEM", "Original Name", "labor", None).await;
    let elem_id = elem["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    // Update
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(&format!("/api/v1/cost-accounting/elements/{}", elem_id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "name": "Updated Element",
                        "defaultRate": "20.00"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let updated: serde_json::Value =
        serde_json::from_slice(&axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(updated["name"], "Updated Element");
    assert!(updated["defaultRate"].as_str().unwrap().contains("20"));
}

#[tokio::test]
async fn test_cost_element_crud() {
    let (_state, app) = setup_cost_accounting_test().await;

    // Create
    let elem = create_test_cost_element(&app, "MAT-01", "Material", "material", None).await;
    assert_eq!(elem["code"], "MAT-01");
    assert_eq!(elem["elementType"], "material");
    let elem_id = elem["id"].as_str().unwrap();

    // Get
    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!("/api/v1/cost-accounting/elements/{}", elem_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // List
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/cost-accounting/elements?elementType=material")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let list: serde_json::Value =
        serde_json::from_slice(&axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert!(list["data"].as_array().unwrap().len() >= 1);

    // Delete
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(&format!("/api/v1/cost-accounting/elements/{}", elem_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_cost_element_invalid_type() {
    let (_state, app) = setup_cost_accounting_test().await;
    let (k, v) = auth_header(&admin_claims());

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/cost-accounting/elements")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "code": "BAD-ELEM",
                        "name": "Bad Element",
                        "elementType": "freight",
                        "defaultRate": "5.00"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_cost_element_duplicate_code() {
    let (_state, app) = setup_cost_accounting_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Create first
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/cost-accounting/elements")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "code": "DUP-ELEM",
                        "name": "First",
                        "elementType": "overhead",
                        "defaultRate": "5.00"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    // Create second with same code
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/cost-accounting/elements")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "code": "DUP-ELEM",
                        "name": "Second",
                        "elementType": "overhead",
                        "defaultRate": "10.00"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

// ============================================================================
// Cost Profiles
// ============================================================================

#[tokio::test]
async fn test_cost_profile_duplicate_code() {
    let (_state, app) = setup_cost_accounting_test().await;

    let book = create_test_cost_book(&app, "DUP-PROF-BOOK", "Profile Book", "fifo").await;
    let book_id = book["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    // Create first profile
    let payload = json!({
        "code": "DUP-PROF",
        "name": "First Profile",
        "costBookId": book_id,
        "costType": "fifo",
        "lotLevelCosting": false,
        "includeLandedCosts": true,
        "overheadAbsorptionMethod": "rate"
    });
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/cost-accounting/profiles")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(serde_json::to_string(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    // Create second with same code
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/cost-accounting/profiles")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(serde_json::to_string(&json!({
        "code": "DUP-PROF",
        "name": "Second Profile",
        "costBookId": book_id,
        "costType": "fifo",
        "lotLevelCosting": false,
        "includeLandedCosts": true,
        "overheadAbsorptionMethod": "rate"
    })).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_cost_profile_crud() {
    let (_state, app) = setup_cost_accounting_test().await;

    let book = create_test_cost_book(&app, "PROF-BOOK", "Profile Book", "fifo").await;
    let book_id = book["id"].as_str().unwrap();

    // Create
    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/cost-accounting/profiles")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "code": "PROF-01",
                        "name": "Test Profile",
                        "costBookId": book_id,
                        "costType": "fifo",
                        "lotLevelCosting": false,
                        "includeLandedCosts": true,
                        "overheadAbsorptionMethod": "rate"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let profile: serde_json::Value =
        serde_json::from_slice(&axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(profile["code"], "PROF-01");
    let profile_id = profile["id"].as_str().unwrap();

    // Get
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!("/api/v1/cost-accounting/profiles/{}", profile_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Delete
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(&format!("/api/v1/cost-accounting/profiles/{}", profile_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Standard Costs
// ============================================================================

#[tokio::test]
async fn test_standard_cost_crud() {
    let (_state, app) = setup_cost_accounting_test().await;

    let book = create_test_cost_book(&app, "SC-BOOK", "SC Book", "standard").await;
    let elem = create_test_cost_element(&app, "SC-MAT", "SC Material", "material", None).await;
    let item_id = uuid::Uuid::new_v4().to_string();

    // Create
    let sc = create_test_standard_cost(
        &app,
        book["id"].as_str().unwrap(),
        elem["id"].as_str().unwrap(),
        &item_id,
        "50.00",
    )
    .await;
    assert_eq!(sc["status"], "active");
    let sc_id = sc["id"].as_str().unwrap();

    // Get
    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!("/api/v1/cost-accounting/standard-costs/{}", sc_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Update
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(&format!("/api/v1/cost-accounting/standard-costs/{}", sc_id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(r#"{"standardCost": "55.00"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let updated: serde_json::Value =
        serde_json::from_slice(&axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert!(updated["standardCost"].as_str().unwrap().contains("55"));

    // Supersede
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/cost-accounting/standard-costs/{}/supersede", sc_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let superseded: serde_json::Value =
        serde_json::from_slice(&axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(superseded["status"], "superseded");
}

// ============================================================================
// Cost Adjustments - Full Lifecycle
// ============================================================================

#[tokio::test]
async fn test_adjustment_full_lifecycle() {
    let (_state, app) = setup_cost_accounting_test().await;

    let book = create_test_cost_book(&app, "ADJ-BOOK", "Adjustment Book", "standard").await;
    let book_id = book["id"].as_str().unwrap();

    // Create adjustment
    let adj = create_test_adjustment(&app, book_id, "standard_cost_update").await;
    assert_eq!(adj["status"], "draft");
    let adj_id = adj["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    // Add adjustment line
    let item_id = uuid::Uuid::new_v4().to_string();
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/cost-accounting/adjustments/{}/lines", adj_id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "lineNumber": 1,
                        "itemId": item_id,
                        "itemName": "Test Item",
                        "oldCost": "50.00",
                        "newCost": "55.00",
                        "currencyCode": "USD"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    // List lines
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!("/api/v1/cost-accounting/adjustments/{}/lines", adj_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let lines: serde_json::Value =
        serde_json::from_slice(&axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(lines["data"].as_array().unwrap().len(), 1);

    // Submit
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/cost-accounting/adjustments/{}/submit", adj_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let submitted: serde_json::Value =
        serde_json::from_slice(&axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(submitted["status"], "submitted");

    // Approve
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/cost-accounting/adjustments/{}/approve", adj_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let approved: serde_json::Value =
        serde_json::from_slice(&axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(approved["status"], "approved");
    assert!(approved["approvedBy"].is_string());

    // Post
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/cost-accounting/adjustments/{}/post", adj_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let posted: serde_json::Value =
        serde_json::from_slice(&axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(posted["status"], "posted");
    assert!(posted["postedAt"].is_string());
}

#[tokio::test]
async fn test_adjustment_rejection_path() {
    let (_state, app) = setup_cost_accounting_test().await;

    let book = create_test_cost_book(&app, "REJ-BOOK", "Rejection Book", "standard").await;
    let adj = create_test_adjustment(&app, book["id"].as_str().unwrap(), "cost_correction").await;
    let adj_id = adj["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    // Submit
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/cost-accounting/adjustments/{}/submit", adj_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Reject
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/cost-accounting/adjustments/{}/reject", adj_id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(r#"{"reason": "Incorrect pricing data"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let rejected: serde_json::Value =
        serde_json::from_slice(&axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(rejected["status"], "rejected");
}

#[tokio::test]
async fn test_adjustment_delete_only_draft() {
    let (_state, app) = setup_cost_accounting_test().await;

    let book = create_test_cost_book(&app, "DEL-BOOK", "Delete Book", "standard").await;
    let adj = create_test_adjustment(&app, book["id"].as_str().unwrap(), "revaluation").await;
    let adj_id = adj["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    // Delete draft should work
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(&format!("/api/v1/cost-accounting/adjustments/{}", adj_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    // Create another and submit, then try to delete
    let adj2 = create_test_adjustment(&app, book["id"].as_str().unwrap(), "overhead_adjustment").await;
    let adj2_id = adj2["id"].as_str().unwrap();

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/cost-accounting/adjustments/{}/submit", adj2_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Delete submitted should fail
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(&format!("/api/v1/cost-accounting/adjustments/{}", adj2_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Cost Variances
// ============================================================================

#[tokio::test]
async fn test_cost_variance_crud() {
    let (_state, app) = setup_cost_accounting_test().await;

    let book = create_test_cost_book(&app, "VAR-BOOK", "Variance Book", "standard").await;
    let book_id = book["id"].as_str().unwrap();
    let item_id = uuid::Uuid::new_v4().to_string();
    let (k, v) = auth_header(&admin_claims());

    // Create variance
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/cost-accounting/variances")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "costBookId": book_id,
                        "varianceType": "purchase_price",
                        "varianceDate": "2026-04-15",
                        "itemId": item_id,
                        "itemName": "Test Item",
                        "standardCost": "100.00",
                        "actualCost": "105.00",
                        "quantity": "500.00",
                        "currencyCode": "USD",
                        "accountingPeriod": "2026-04"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let variance: serde_json::Value =
        serde_json::from_slice(&axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(variance["varianceType"], "purchase_price");
    assert!(variance["varianceAmount"].as_str().unwrap().parse::<f64>().unwrap() > 0.0); // unfavorable
    let var_id = variance["id"].as_str().unwrap();

    // Get
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!("/api/v1/cost-accounting/variances/{}", var_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // List
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!("/api/v1/cost-accounting/variances?costBookId={}", book_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Analyze
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/cost-accounting/variances/{}/analyze", var_id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(r#"{"notes": "Supplier price increase due to raw material shortage"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let analyzed: serde_json::Value =
        serde_json::from_slice(&axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(analyzed["isAnalyzed"], true);
}

#[tokio::test]
async fn test_variance_invalid_type() {
    let (_state, app) = setup_cost_accounting_test().await;

    let book = create_test_cost_book(&app, "VAR-INV", "Invalid Var Book", "standard").await;
    let item_id = uuid::Uuid::new_v4().to_string();
    let (k, v) = auth_header(&admin_claims());

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/cost-accounting/variances")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "costBookId": book["id"],
                        "varianceType": "unknown_type",
                        "varianceDate": "2026-04-15",
                        "itemId": item_id,
                        "standardCost": "100.00",
                        "actualCost": "105.00",
                        "quantity": "500.00"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Dashboard
// ============================================================================

#[tokio::test]
async fn test_cost_accounting_dashboard() {
    let (_state, app) = setup_cost_accounting_test().await;

    // Create some data first
    create_test_cost_book(&app, "DASH-BOOK", "Dashboard Book", "standard").await;
    create_test_cost_element(&app, "DASH-MAT", "Dashboard Material", "material", None).await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/cost-accounting/dashboard")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let dashboard: serde_json::Value =
        serde_json::from_slice(&axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert!(dashboard["totalCostBooks"].as_i64().unwrap() >= 1);
    assert!(dashboard["activeCostBooks"].as_i64().unwrap() >= 1);
    assert!(dashboard["totalCostElements"].as_i64().unwrap() >= 1);
}

// ============================================================================
// Not Found Cases
// ============================================================================

#[tokio::test]
async fn test_get_nonexistent_cost_book() {
    let (_state, app) = setup_cost_accounting_test().await;
    let (k, v) = auth_header(&admin_claims());
    let fake_id = uuid::Uuid::new_v4();

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!("/api/v1/cost-accounting/books/{}", fake_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_nonexistent_cost_adjustment() {
    let (_state, app) = setup_cost_accounting_test().await;
    let (k, v) = auth_header(&admin_claims());
    let fake_id = uuid::Uuid::new_v4();

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!("/api/v1/cost-accounting/adjustments/{}", fake_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}
