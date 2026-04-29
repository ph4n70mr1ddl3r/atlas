//! Promotions Management E2E Tests
//!
//! Tests for Oracle Fusion Trade Management > Trade Promotion:
//! - Promotion CRUD (create, get, list, update, delete)
//! - Promotion lifecycle (activate, hold, complete, cancel)
//! - Promotional offers (create, list, delete)
//! - Fund allocation (create, list, update committed/spent, delete)
//! - Claims processing (create, review, approve, reject, settle, delete)
//! - Dashboard
//! - Validation edge cases

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use uuid::Uuid;
use super::common::helpers::*;

async fn setup_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    let app = build_router(state.clone());
    (state, app)
}

/// Helper: create a promotion
async fn create_promotion(
    app: &axum::Router,
    code: &str,
    name: &str,
    promotion_type: &str,
    budget: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/promotions")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": code,
            "name": name,
            "promotionType": promotion_type,
            "startDate": "2026-01-01",
            "endDate": "2026-12-31",
            "budgetAmount": budget,
            "currencyCode": "USD"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Expected 201 creating promotion");
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

/// Helper: activate a promotion
async fn activate_promotion(app: &axum::Router, id: &str) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/promotions/{}/activate", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK, "Expected 200 activating promotion");
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Promotion CRUD Tests
// ============================================================================

#[tokio::test]
async fn test_create_promotion() {
    let (_state, app) = setup_test().await;
    let p = create_promotion(&app, "SPRING-26", "Spring Sale 2026", "trade", "50000").await;
    assert_eq!(p["code"], "SPRING-26");
    assert_eq!(p["name"], "Spring Sale 2026");
    assert_eq!(p["promotionType"], "trade");
    assert_eq!(p["status"], "draft");
    assert_eq!(p["budgetAmount"], "50000.00");
    assert_eq!(p["spentAmount"], "0.00");
    assert!(p["id"].is_string());
}

#[tokio::test]
async fn test_create_promotion_all_types() {
    let (_state, app) = setup_test().await;
    for pt in &["trade", "consumer", "channel", "co_op"] {
        let p = create_promotion(&app, &format!("T-{}", pt), &format!("{} Promo", pt), pt, "10000").await;
        assert_eq!(p["promotionType"], *pt);
    }
}

#[tokio::test]
async fn test_create_promotion_with_customer_and_product() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/promotions")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "ACME-26",
            "name": "Acme Promo",
            "promotionType": "trade",
            "startDate": "2026-01-01",
            "endDate": "2026-06-30",
            "budgetAmount": "25000",
            "customerId": Uuid::new_v4().to_string(),
            "customerName": "Acme Corp",
            "productId": Uuid::new_v4().to_string(),
            "productName": "Widget Pro",
            "currencyCode": "USD"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let p: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(p["customerName"], "Acme Corp");
    assert_eq!(p["productName"], "Widget Pro");
}

#[tokio::test]
async fn test_get_promotion() {
    let (_state, app) = setup_test().await;
    let p = create_promotion(&app, "GET-P", "Get Test", "trade", "10000").await;
    let id = p["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/promotions/{}", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let fetched: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(fetched["code"], "GET-P");
    assert_eq!(fetched["name"], "Get Test");
}

#[tokio::test]
async fn test_list_promotions() {
    let (_state, app) = setup_test().await;
    create_promotion(&app, "LIST-A", "Promo A", "trade", "10000").await;
    create_promotion(&app, "LIST-B", "Promo B", "consumer", "20000").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/promotions")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let list: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(list.as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_list_promotions_filter_by_type() {
    let (_state, app) = setup_test().await;
    create_promotion(&app, "FT-TRADE", "Trade", "trade", "10000").await;
    create_promotion(&app, "FT-CONSUMER", "Consumer", "consumer", "10000").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/promotions?promotion_type=trade")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let list: serde_json::Value = serde_json::from_slice(&b).unwrap();
    for p in list.as_array().unwrap() {
        assert_eq!(p["promotionType"], "trade");
    }
}

#[tokio::test]
async fn test_update_promotion() {
    let (_state, app) = setup_test().await;
    let p = create_promotion(&app, "UPD-P", "Original", "trade", "10000").await;
    let id = p["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("PUT")
        .uri(&format!("/api/v1/promotions/{}", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "name": "Updated Promo",
            "budgetAmount": "75000"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(updated["name"], "Updated Promo");
    assert_eq!(updated["budgetAmount"], "75000.00");
    assert_eq!(updated["code"], "UPD-P");
}

#[tokio::test]
async fn test_delete_promotion() {
    let (_state, app) = setup_test().await;
    let p = create_promotion(&app, "DEL-P", "Delete Me", "trade", "10000").await;
    let id = p["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/promotions/{}", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);

    // Verify gone
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/promotions/{}", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

// ============================================================================
// Validation Tests
// ============================================================================

#[tokio::test]
async fn test_create_empty_code_rejected() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/promotions")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "",
            "name": "Bad Code",
            "promotionType": "trade",
            "startDate": "2026-01-01",
            "endDate": "2026-12-31",
            "budgetAmount": "10000"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_invalid_type_rejected() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/promotions")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "BAD-TYPE",
            "name": "Bad Type",
            "promotionType": "unknown",
            "startDate": "2026-01-01",
            "endDate": "2026-12-31",
            "budgetAmount": "10000"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_duplicate_code_rejected() {
    let (_state, app) = setup_test().await;
    create_promotion(&app, "DUP", "First", "trade", "10000").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/promotions")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "DUP",
            "name": "Second",
            "promotionType": "trade",
            "startDate": "2026-01-01",
            "endDate": "2026-12-31",
            "budgetAmount": "10000"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_create_invalid_dates_rejected() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/promotions")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "BAD-DATES",
            "name": "Bad Dates",
            "promotionType": "trade",
            "startDate": "2027-01-01",
            "endDate": "2026-01-01",
            "budgetAmount": "10000"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_negative_budget_rejected() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/promotions")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "NEG-BUDGET",
            "name": "Negative Budget",
            "promotionType": "trade",
            "startDate": "2026-01-01",
            "endDate": "2026-12-31",
            "budgetAmount": "-5000"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Lifecycle Tests
// ============================================================================

#[tokio::test]
async fn test_promotion_lifecycle() {
    let (_state, app) = setup_test().await;
    let p = create_promotion(&app, "LC-P", "Lifecycle", "trade", "50000").await;
    let id = p["id"].as_str().unwrap();
    assert_eq!(p["status"], "draft");

    // Activate
    let activated = activate_promotion(&app, id).await;
    assert_eq!(activated["status"], "active");

    // Hold
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/promotions/{}/hold", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let held: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(held["status"], "on_hold");

    // Re-activate from hold
    let reactivated = activate_promotion(&app, id).await;
    assert_eq!(reactivated["status"], "active");

    // Complete
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/promotions/{}/complete", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let completed: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(completed["status"], "completed");
}

#[tokio::test]
async fn test_cancel_promotion() {
    let (_state, app) = setup_test().await;
    let p = create_promotion(&app, "CANCEL-P", "Cancel Me", "trade", "10000").await;
    let id = p["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/promotions/{}/cancel", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let cancelled: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(cancelled["status"], "cancelled");
}

#[tokio::test]
async fn test_cannot_delete_active_promotion() {
    let (_state, app) = setup_test().await;
    let p = create_promotion(&app, "DEL-ACT", "Active Promo", "trade", "10000").await;
    let id = p["id"].as_str().unwrap();
    activate_promotion(&app, id).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/promotions/{}", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Offer Tests
// ============================================================================

#[tokio::test]
async fn test_create_offer() {
    let (_state, app) = setup_test().await;
    let p = create_promotion(&app, "OFF-P", "Offer Promo", "trade", "50000").await;
    let promo_id = p["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/promotions/{}/offers", promo_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "offerType": "discount",
            "description": "20% off all items",
            "discountType": "percentage",
            "discountValue": "20",
            "minimumPurchase": "100"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let offer: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(offer["offerType"], "discount");
    assert_eq!(offer["discountType"], "percentage");
    assert_eq!(offer["discountValue"], "20.00");
    assert_eq!(offer["minimumPurchase"], "100.00");
}

#[tokio::test]
async fn test_create_buy_get_offer() {
    let (_state, app) = setup_test().await;
    let p = create_promotion(&app, "BG-P", "Buy Get Promo", "trade", "50000").await;
    let promo_id = p["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/promotions/{}/offers", promo_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "offerType": "buy_get",
            "description": "Buy 2 get 1 free",
            "discountType": "fixed_amount",
            "discountValue": "0",
            "buyQuantity": 2,
            "getQuantity": 1
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let offer: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(offer["offerType"], "buy_get");
    assert_eq!(offer["buyQuantity"], 2);
    assert_eq!(offer["getQuantity"], 1);
}

#[tokio::test]
async fn test_list_offers() {
    let (_state, app) = setup_test().await;
    let p = create_promotion(&app, "LO-P", "List Offers", "trade", "50000").await;
    let promo_id = p["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    // Add two offers
    for (ot, dt, dv) in &[("discount", "percentage", "10"), ("rebate", "fixed_amount", "50")] {
        app.clone().oneshot(Request::builder().method("POST")
            .uri(&format!("/api/v1/promotions/{}/offers", promo_id))
            .header("Content-Type", "application/json").header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({
                "offerType": ot,
                "discountType": dt,
                "discountValue": dv
            })).unwrap())).unwrap()
        ).await.unwrap();
    }

    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/promotions/{}/offers", promo_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let offers: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(offers.as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_delete_offer() {
    let (_state, app) = setup_test().await;
    let p = create_promotion(&app, "DO-P", "Delete Offer", "trade", "50000").await;
    let promo_id = p["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/promotions/{}/offers", promo_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "offerType": "discount",
            "discountType": "percentage",
            "discountValue": "15"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let offer: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let offer_id = offer["id"].as_str().unwrap();

    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/promotions/offers/{}", offer_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_percentage_over_100_rejected() {
    let (_state, app) = setup_test().await;
    let p = create_promotion(&app, "PCT-P", "Pct Offer", "trade", "50000").await;
    let promo_id = p["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/promotions/{}/offers", promo_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "offerType": "discount",
            "discountType": "percentage",
            "discountValue": "150"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Fund Allocation Tests
// ============================================================================

#[tokio::test]
async fn test_create_fund() {
    let (_state, app) = setup_test().await;
    let p = create_promotion(&app, "FUND-P", "Fund Promo", "trade", "50000").await;
    let promo_id = p["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/promotions/{}/funds", promo_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "fundType": "marketing_development",
            "allocatedAmount": "30000",
            "currencyCode": "USD"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let fund: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(fund["fundType"], "marketing_development");
    assert_eq!(fund["allocatedAmount"], "30000.00");
    assert_eq!(fund["committedAmount"], "0.00");
    assert_eq!(fund["spentAmount"], "0.00");
}

#[tokio::test]
async fn test_list_funds() {
    let (_state, app) = setup_test().await;
    let p = create_promotion(&app, "LF-P", "List Funds", "trade", "50000").await;
    let promo_id = p["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    for ft in &["marketing_development", "cooperative"] {
        app.clone().oneshot(Request::builder().method("POST")
            .uri(&format!("/api/v1/promotions/{}/funds", promo_id))
            .header("Content-Type", "application/json").header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({
                "fundType": ft,
                "allocatedAmount": "10000"
            })).unwrap())).unwrap()
        ).await.unwrap();
    }

    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/promotions/{}/funds", promo_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let funds: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(funds.as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_update_fund_committed_and_spent() {
    let (_state, app) = setup_test().await;
    let p = create_promotion(&app, "UF-P", "Update Fund", "trade", "50000").await;
    let promo_id = p["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/promotions/{}/funds", promo_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "fundType": "trade_spend",
            "allocatedAmount": "20000"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let fund: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let fund_id = fund["id"].as_str().unwrap();

    // Update committed
    let r = app.clone().oneshot(Request::builder().method("PUT")
        .uri(&format!("/api/v1/promotions/funds/{}/committed", fund_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "amount": "15000"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(updated["committedAmount"], "15000.00");

    // Update spent
    let r = app.clone().oneshot(Request::builder().method("PUT")
        .uri(&format!("/api/v1/promotions/funds/{}/spent", fund_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "amount": "8000"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(updated["spentAmount"], "8000.00");
}

#[tokio::test]
async fn test_delete_fund() {
    let (_state, app) = setup_test().await;
    let p = create_promotion(&app, "DF-P", "Delete Fund", "trade", "50000").await;
    let promo_id = p["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/promotions/{}/funds", promo_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "fundType": "display",
            "allocatedAmount": "5000"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let fund: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let fund_id = fund["id"].as_str().unwrap();

    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/promotions/funds/{}", fund_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Claims Tests
// ============================================================================

#[tokio::test]
async fn test_create_claim() {
    let (_state, app) = setup_test().await;
    let p = create_promotion(&app, "CLM-P", "Claim Promo", "trade", "50000").await;
    let promo_id = p["id"].as_str().unwrap();
    activate_promotion(&app, promo_id).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/promotions/{}/claims", promo_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "claimType": "accrual",
            "amount": "5000",
            "claimDate": "2026-03-15",
            "description": "Q1 accrual"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let claim: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(claim["claimType"], "accrual");
    assert_eq!(claim["status"], "submitted");
    assert_eq!(claim["amount"], "5000.00");
    assert!(claim["claimNumber"].as_str().unwrap().starts_with("CLM-"));
}

#[tokio::test]
async fn test_cannot_claim_against_draft_promotion() {
    let (_state, app) = setup_test().await;
    let p = create_promotion(&app, "DRAFT-CLM", "Draft Promo", "trade", "50000").await;
    let promo_id = p["id"].as_str().unwrap();
    // Not activating - stays in draft

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/promotions/{}/claims", promo_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "claimType": "accrual",
            "amount": "5000",
            "claimDate": "2026-03-15"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_claim_full_lifecycle() {
    let (_state, app) = setup_test().await;
    let p = create_promotion(&app, "CLM-LC", "Claim LC", "trade", "50000").await;
    let promo_id = p["id"].as_str().unwrap();
    activate_promotion(&app, promo_id).await;

    let (k, v) = auth_header(&admin_claims());
    // Create claim
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/promotions/{}/claims", promo_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "claimType": "settlement",
            "amount": "10000",
            "claimDate": "2026-06-15",
            "customerName": "Big Customer"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let claim: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let claim_id = claim["id"].as_str().unwrap();
    assert_eq!(claim["status"], "submitted");

    // Review
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/promotions/claims/{}/review", claim_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let reviewed: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(reviewed["status"], "under_review");

    // Approve with partial amount
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/promotions/claims/{}/approve", claim_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "approvedAmount": "9500"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let approved: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(approved["status"], "approved");
    assert_eq!(approved["approvedAmount"], "9500.00");

    // Settle
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/promotions/claims/{}/settle", claim_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "paidAmount": "9500"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let settled: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(settled["status"], "paid");
    assert_eq!(settled["paidAmount"], "9500.00");
    assert!(settled["settlementDate"].is_string());
}

#[tokio::test]
async fn test_reject_claim() {
    let (_state, app) = setup_test().await;
    let p = create_promotion(&app, "REJ-P", "Reject Claim", "trade", "50000").await;
    let promo_id = p["id"].as_str().unwrap();
    activate_promotion(&app, promo_id).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/promotions/{}/claims", promo_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "claimType": "deduction",
            "amount": "2000",
            "claimDate": "2026-04-01"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let claim: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let claim_id = claim["id"].as_str().unwrap();

    // Reject
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/promotions/claims/{}/reject", claim_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "reason": "Insufficient documentation"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let rejected: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(rejected["status"], "rejected");
    assert_eq!(rejected["rejectionReason"], "Insufficient documentation");
}

#[tokio::test]
async fn test_list_claims() {
    let (_state, app) = setup_test().await;
    let p = create_promotion(&app, "LC-P", "List Claims", "trade", "50000").await;
    let promo_id = p["id"].as_str().unwrap();
    activate_promotion(&app, promo_id).await;

    let (k, v) = auth_header(&admin_claims());
    for ct in &["accrual", "settlement"] {
        app.clone().oneshot(Request::builder().method("POST")
            .uri(&format!("/api/v1/promotions/{}/claims", promo_id))
            .header("Content-Type", "application/json").header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({
                "claimType": ct,
                "amount": "1000",
                "claimDate": "2026-04-01"
            })).unwrap())).unwrap()
        ).await.unwrap();
    }

    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/promotions/{}/claims", promo_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let claims: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(claims.as_array().unwrap().len(), 2);

    // Filter by status
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/promotions/{}/claims?status=submitted", promo_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let submitted: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(submitted.as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_delete_claim() {
    let (_state, app) = setup_test().await;
    let p = create_promotion(&app, "DC-P", "Delete Claim", "trade", "50000").await;
    let promo_id = p["id"].as_str().unwrap();
    activate_promotion(&app, promo_id).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/promotions/{}/claims", promo_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "claimType": "lump_sum",
            "amount": "3000",
            "claimDate": "2026-05-01"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let claim: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let claim_id = claim["id"].as_str().unwrap();

    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/promotions/claims/{}", claim_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Dashboard Tests
// ============================================================================

#[tokio::test]
async fn test_promotions_dashboard() {
    let (_state, app) = setup_test().await;
    create_promotion(&app, "DASH-1", "Dashboard Trade", "trade", "30000").await;
    create_promotion(&app, "DASH-2", "Dashboard Consumer", "consumer", "20000").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/promotions/dashboard")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let dashboard: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(dashboard["totalPromotions"].as_i64().unwrap() >= 2);
    assert!(dashboard["byStatus"].is_array());
    assert!(dashboard["byType"].is_array());
}

// ============================================================================
// Not Found Tests
// ============================================================================

#[tokio::test]
async fn test_get_nonexistent_promotion() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/promotions/{}", Uuid::new_v4()))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_nonexistent_claim() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/promotions/claims/{}", Uuid::new_v4()))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}
