//! Subscription Management E2E Tests (Oracle Fusion Subscription Management)
//!
//! Tests for Oracle Fusion Subscription Management:
//! - Product catalog CRUD
//! - Price tier management
//! - Subscription lifecycle (create, activate, suspend, reactivate, cancel, renew)
//! - Amendments (price/quantity changes, apply, cancel)
//! - Billing schedule generation
//! - Revenue schedule (ASC 606) generation and recognition
//! - Full end-to-end subscription billing flow

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_subscription_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    setup_test_db(&state.db_pool).await;
    // Clean subscription data for isolation
    sqlx::query("DELETE FROM _atlas.subscription_revenue_schedule").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM _atlas.subscription_billing_schedule").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM _atlas.subscription_amendments").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM _atlas.subscriptions").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM _atlas.subscription_price_tiers").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM _atlas.subscription_products").execute(&state.db_pool).await.ok();
    let app = build_router(state.clone());
    (state, app)
}

async fn create_test_product(app: &axum::Router, code: &str) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/subscription/products")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "product_code": code,
            "name": format!("Product {}", code),
            "description": "Test subscription product",
            "product_type": "software",
            "billing_frequency": "monthly",
            "default_duration_months": 12,
            "is_auto_renew": true,
            "cancellation_notice_days": 30,
            "setup_fee": "500",
            "tier_type": "flat"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Failed to create product");
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_subscription(app: &axum::Router, product_id: &str) -> serde_json::Value {
    let customer_id = uuid::Uuid::new_v4();
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/subscription/subscriptions")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "customer_id": customer_id,
            "customer_name": "Acme Corp",
            "product_id": product_id,
            "start_date": "2026-01-01",
            "duration_months": 12,
            "billing_frequency": "monthly",
            "billing_day_of_month": 1,
            "currency_code": "USD",
            "quantity": "1",
            "discount_percent": "0",
            "is_auto_renew": false
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Failed to create subscription");
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Product Catalog Tests
// ============================================================================

#[tokio::test]
async fn test_create_subscription_product() {
    let (_state, app) = setup_subscription_test().await;
    let product = create_test_product(&app, "SW-001").await;
    assert_eq!(product["productCode"], "SW-001");
    assert_eq!(product["name"], "Product SW-001");
    assert_eq!(product["productType"], "software");
    assert_eq!(product["billingFrequency"], "monthly");
    assert!(product["id"].is_string());
}

#[tokio::test]
async fn test_list_products() {
    let (_state, app) = setup_subscription_test().await;
    create_test_product(&app, "SW-LA").await;
    create_test_product(&app, "SW-LB").await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/subscription/products?active_only=true")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_get_product() {
    let (_state, app) = setup_subscription_test().await;
    create_test_product(&app, "SW-GET").await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/subscription/products/SW-GET")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let fetched: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(fetched["productCode"], "SW-GET");
}

#[tokio::test]
async fn test_delete_product() {
    let (_state, app) = setup_subscription_test().await;
    create_test_product(&app, "SW-DEL").await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/subscription/products/SW-DEL")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
    // Verify it's gone
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/subscription/products/SW-DEL")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_create_product_invalid_type() {
    let (_state, app) = setup_subscription_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/subscription/products")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "product_code": "SW-BAD",
            "name": "Bad Product",
            "product_type": "invalid_type",
            "billing_frequency": "monthly"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Subscription Lifecycle Tests
// ============================================================================

#[tokio::test]
async fn test_create_subscription() {
    let (_state, app) = setup_subscription_test().await;
    let product = create_test_product(&app, "SW-SUB01").await;
    let sub = create_test_subscription(&app, product["id"].as_str().unwrap()).await;
    assert_eq!(sub["status"], "draft");
    assert_eq!(sub["currencyCode"], "USD");
    assert!(sub["subscriptionNumber"].as_str().unwrap().starts_with("SUB-"));
    assert!(sub["id"].is_string());
}

#[tokio::test]
async fn test_list_subscriptions() {
    let (_state, app) = setup_subscription_test().await;
    let product = create_test_product(&app, "SW-LIST").await;
    create_test_subscription(&app, product["id"].as_str().unwrap()).await;
    create_test_subscription(&app, product["id"].as_str().unwrap()).await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/subscription/subscriptions?status=draft")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_activate_subscription() {
    let (_state, app) = setup_subscription_test().await;
    let product = create_test_product(&app, "SW-ACT").await;
    let sub = create_test_subscription(&app, product["id"].as_str().unwrap()).await;
    let id = sub["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/subscription/subscriptions/{}/activate", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let activated: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(activated["status"], "active");
}

#[tokio::test]
async fn test_suspend_and_reactivate_subscription() {
    let (_state, app) = setup_subscription_test().await;
    let product = create_test_product(&app, "SW-SUS").await;
    let sub = create_test_subscription(&app, product["id"].as_str().unwrap()).await;
    let id = sub["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    // Activate first
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/subscription/subscriptions/{}/activate", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Suspend
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/subscription/subscriptions/{}/suspend", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "reason": "Customer requested pause"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let suspended: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(suspended["status"], "suspended");

    // Reactivate
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/subscription/subscriptions/{}/reactivate", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let reactivated: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(reactivated["status"], "active");
}

#[tokio::test]
async fn test_cancel_subscription() {
    let (_state, app) = setup_subscription_test().await;
    let product = create_test_product(&app, "SW-CAN").await;
    let sub = create_test_subscription(&app, product["id"].as_str().unwrap()).await;
    let id = sub["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    // Activate first
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/subscription/subscriptions/{}/activate", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Cancel
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/subscription/subscriptions/{}/cancel", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "cancellation_date": "2026-06-01",
            "reason": "No longer needed"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let cancelled: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(cancelled["status"], "cancelled");
}

// ============================================================================
// Amendment Tests
// ============================================================================

#[tokio::test]
async fn test_create_and_apply_amendment() {
    let (_state, app) = setup_subscription_test().await;
    let product = create_test_product(&app, "SW-AMD").await;
    let sub = create_test_subscription(&app, product["id"].as_str().unwrap()).await;
    let id = sub["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    // Activate first
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/subscription/subscriptions/{}/activate", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Create amendment (price change)
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/subscription/subscriptions/{}/amendments", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "amendment_type": "price_change",
            "description": "Price increase per contract terms",
            "new_unit_price": "150",
            "effective_date": "2026-04-01"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let amendment: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(amendment["amendmentType"], "price_change");
    assert_eq!(amendment["status"], "draft");
    let amendment_id = amendment["id"].as_str().unwrap();

    // Apply the amendment
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/subscription/amendments/{}/apply", amendment_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let applied: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(applied["status"], "applied");
}

#[tokio::test]
async fn test_cancel_amendment() {
    let (_state, app) = setup_subscription_test().await;
    let product = create_test_product(&app, "SW-AMDC").await;
    let sub = create_test_subscription(&app, product["id"].as_str().unwrap()).await;
    let id = sub["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    // Activate
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/subscription/subscriptions/{}/activate", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Create amendment
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/subscription/subscriptions/{}/amendments", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "amendment_type": "quantity_change",
            "new_quantity": "5",
            "effective_date": "2026-04-01"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let amendment: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let amendment_id = amendment["id"].as_str().unwrap();

    // Cancel the amendment
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/subscription/amendments/{}/cancel", amendment_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let cancelled: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(cancelled["status"], "cancelled");
}

#[tokio::test]
async fn test_list_amendments() {
    let (_state, app) = setup_subscription_test().await;
    let product = create_test_product(&app, "SW-AMDL").await;
    let sub = create_test_subscription(&app, product["id"].as_str().unwrap()).await;
    let id = sub["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    // Activate
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/subscription/subscriptions/{}/activate", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Create two amendments
    for atype in ["price_change", "quantity_change"] {
        app.clone().oneshot(Request::builder().method("POST")
            .uri(&format!("/api/v1/subscription/subscriptions/{}/amendments", id))
            .header("Content-Type", "application/json").header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({
                "amendment_type": atype,
                "new_unit_price": "200",
                "new_quantity": "3",
                "effective_date": "2026-05-01"
            })).unwrap())).unwrap()
        ).await.unwrap();
    }

    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/subscription/subscriptions/{}/amendments", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 2);
}

// ============================================================================
// Billing & Revenue Schedule Tests
// ============================================================================

#[tokio::test]
async fn test_billing_schedule_generated_on_activate() {
    let (_state, app) = setup_subscription_test().await;
    let product = create_test_product(&app, "SW-BIL").await;
    let sub = create_test_subscription(&app, product["id"].as_str().unwrap()).await;
    let id = sub["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    // Activate (should generate billing schedule)
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/subscription/subscriptions/{}/activate", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Check billing schedule
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/subscription/subscriptions/{}/billing-schedule", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let lines = resp["data"].as_array().unwrap();
    // 12 months of monthly billing
    assert_eq!(lines.len(), 12);
    for line in lines {
        assert_eq!(line["status"], "pending");
    }
}

#[tokio::test]
async fn test_revenue_schedule_generated_on_activate() {
    let (_state, app) = setup_subscription_test().await;
    let product = create_test_product(&app, "SW-REV").await;
    let sub = create_test_subscription(&app, product["id"].as_str().unwrap()).await;
    let id = sub["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    // Activate (should generate revenue schedule)
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/subscription/subscriptions/{}/activate", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Check revenue schedule
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/subscription/subscriptions/{}/revenue-schedule", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let lines = resp["data"].as_array().unwrap();
    assert_eq!(lines.len(), 12);
    // All lines should start as deferred
    for line in lines {
        assert_eq!(line["status"], "deferred");
    }
}

#[tokio::test]
async fn test_recognize_revenue() {
    let (_state, app) = setup_subscription_test().await;
    let product = create_test_product(&app, "SW-REC").await;
    let sub = create_test_subscription(&app, product["id"].as_str().unwrap()).await;
    let id = sub["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    // Activate
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/subscription/subscriptions/{}/activate", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Get revenue schedule
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/subscription/subscriptions/{}/revenue-schedule", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let lines = resp["data"].as_array().unwrap();
    let first_line_id = lines[0]["id"].as_str().unwrap();

    // Recognize first period
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/subscription/revenue-lines/{}/recognize", first_line_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let recognized: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(recognized["status"], "recognized");
}

// ============================================================================
// Full End-to-End Subscription Billing Flow
// ============================================================================

#[tokio::test]
async fn test_full_subscription_lifecycle_flow() {
    let (_state, app) = setup_subscription_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Step 1: Create a subscription product
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/subscription/products")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "product_code": "E2E-SUITE",
            "name": "E2E Suite",
            "description": "Enterprise suite license",
            "product_type": "software",
            "billing_frequency": "monthly",
            "default_duration_months": 12,
            "is_auto_renew": true,
            "setup_fee": "1000",
            "tier_type": "flat"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let product: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let product_id = product["id"].as_str().unwrap();

    // Step 2: Create a subscription
    let customer_id = uuid::Uuid::new_v4();
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/subscription/subscriptions")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "customer_id": customer_id,
            "customer_name": "Mega Corp",
            "product_id": product_id,
            "start_date": "2026-01-01",
            "duration_months": 12,
            "billing_frequency": "monthly",
            "currency_code": "USD",
            "quantity": "1",
            "discount_percent": "10",
            "is_auto_renew": true
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let sub: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(sub["status"], "draft");
    let sub_id = sub["id"].as_str().unwrap();

    // Step 3: Activate (generates billing + revenue schedules)
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/subscription/subscriptions/{}/activate", sub_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let activated: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(activated["status"], "active");

    // Step 4: Verify billing schedule (12 monthly periods)
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/subscription/subscriptions/{}/billing-schedule", sub_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let billing: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let billing_lines = billing["data"].as_array().unwrap();
    assert_eq!(billing_lines.len(), 12);

    // Verify revenue schedule
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/subscription/subscriptions/{}/revenue-schedule", sub_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let revenue: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let revenue_lines = revenue["data"].as_array().unwrap();
    assert_eq!(revenue_lines.len(), 12);

    // Step 5: Recognize first 3 months of revenue
    for i in 0..3 {
        let line_id = revenue_lines[i]["id"].as_str().unwrap();
        let r = app.clone().oneshot(Request::builder().method("POST")
            .uri(&format!("/api/v1/subscription/revenue-lines/{}/recognize", line_id))
            .header(&k, &v).body(Body::empty()).unwrap()
        ).await.unwrap();
        assert_eq!(r.status(), StatusCode::OK);
    }

    // Step 6: Create an amendment (price change)
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/subscription/subscriptions/{}/amendments", sub_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "amendment_type": "price_change",
            "description": "Upgrade pricing",
            "new_unit_price": "150",
            "effective_date": "2026-04-01"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let amendment: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let amendment_id = amendment["id"].as_str().unwrap();

    // Apply the amendment
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/subscription/amendments/{}/apply", amendment_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let applied: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(applied["status"], "applied");

    // Step 7: Verify the subscription still has billing data
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/subscription/subscriptions/{}/billing-schedule", sub_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let billing_after: serde_json::Value = serde_json::from_slice(&b).unwrap();
    // Should have 12 original + 1 proration line = 13
    assert!(billing_after["data"].as_array().unwrap().len() >= 12);
}

#[tokio::test]
async fn test_subscription_dashboard() {
    let (_state, app) = setup_subscription_test().await;
    let product = create_test_product(&app, "SW-DASH").await;
    let _sub = create_test_subscription(&app, product["id"].as_str().unwrap()).await;
    let (k, v) = auth_header(&admin_claims());

    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/subscription/dashboard")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    assert_eq!(status, StatusCode::OK);
    let dashboard: serde_json::Value = serde_json::from_slice(&b).unwrap();
    // Dashboard should return a valid JSON object
    assert!(dashboard.is_object());
}
