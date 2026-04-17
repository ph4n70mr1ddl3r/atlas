//! Currency Management E2E Tests
//!
//! Tests for Oracle Fusion Cloud ERP Multi-Currency Management:
//! - Currency definition CRUD
//! - Exchange rate management
//! - Currency conversion with direct, reverse, and triangulation
//! - Unrealized gain/loss calculation

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_currency_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    let app = build_router(state.clone());
    (state, app)
}

async fn create_test_currency(app: &axum::Router, code: &str, name: &str, is_base: bool) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/currencies")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": code, "name": name, "symbol": "$", "precision": 2, "is_base_currency": is_base
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn set_test_rate(
    app: &axum::Router,
    from: &str, to: &str, rate: &str, date: &str, rate_type: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/exchange-rates")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "from_currency": from, "to_currency": to, "rate": rate,
            "effective_date": date, "rate_type": rate_type, "source": "test"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Currency Definition Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_create_currency() {
    let (state, app) = setup_currency_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/currencies")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "USD", "name": "US Dollar", "symbol": "$", "precision": 2, "is_base_currency": true
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let c: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(c["code"], "USD");
    assert_eq!(c["name"], "US Dollar");
    assert_eq!(c["is_base_currency"], true);
    cleanup_test_db(&state.db_pool).await;
}

#[tokio::test]
#[ignore]
async fn test_list_currencies() {
    let (state, app) = setup_currency_test().await;
    create_test_currency(&app, "USD", "US Dollar", true).await;
    create_test_currency(&app, "EUR", "Euro", false).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().uri("/api/v1/currencies")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(result["data"].as_array().unwrap().len(), 2);
    cleanup_test_db(&state.db_pool).await;
}

#[tokio::test]
#[ignore]
async fn test_get_base_currency() {
    let (state, app) = setup_currency_test().await;
    create_test_currency(&app, "USD", "US Dollar", true).await;
    create_test_currency(&app, "EUR", "Euro", false).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().uri("/api/v1/currencies/base")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let c: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(c["code"], "USD");
    assert_eq!(c["is_base_currency"], true);
    cleanup_test_db(&state.db_pool).await;
}

#[tokio::test]
#[ignore]
async fn test_create_currency_invalid_code() {
    let (state, app) = setup_currency_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/currencies")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "TOOLONG", "name": "Invalid", "symbol": "X", "precision": 2, "is_base_currency": false
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
    cleanup_test_db(&state.db_pool).await;
}

#[tokio::test]
#[ignore]
async fn test_delete_currency() {
    let (state, app) = setup_currency_test().await;
    create_test_currency(&app, "EUR", "Euro", false).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE").uri("/api/v1/currencies/EUR")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
    cleanup_test_db(&state.db_pool).await;
}

// ============================================================================
// Exchange Rate Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_set_exchange_rate() {
    let (state, app) = setup_currency_test().await;
    create_test_currency(&app, "USD", "US Dollar", true).await;
    create_test_currency(&app, "EUR", "Euro", false).await;

    let rate = set_test_rate(&app, "USD", "EUR", "0.92", "2026-01-15", "daily").await;
    assert_eq!(rate["from_currency"], "USD");
    assert_eq!(rate["to_currency"], "EUR");
    assert_eq!(rate["rate"], "0.9200000000");
    // Inverse rate should be computed: 1/0.92 ≈ 1.087
    let inverse: f64 = rate["inverse_rate"].as_str().unwrap().parse().unwrap();
    assert!((inverse - 1.087).abs() < 0.01);
    cleanup_test_db(&state.db_pool).await;
}

#[tokio::test]
#[ignore]
async fn test_set_exchange_rate_rejects_zero() {
    let (state, app) = setup_currency_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/exchange-rates")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "from_currency": "USD", "to_currency": "EUR", "rate": "0",
            "effective_date": "2026-01-15", "rate_type": "daily"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
    cleanup_test_db(&state.db_pool).await;
}

#[tokio::test]
#[ignore]
async fn test_set_exchange_rate_rejects_same_currency() {
    let (state, app) = setup_currency_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/exchange-rates")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "from_currency": "USD", "to_currency": "USD", "rate": "1",
            "effective_date": "2026-01-15", "rate_type": "daily"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
    cleanup_test_db(&state.db_pool).await;
}

#[tokio::test]
#[ignore]
async fn test_list_exchange_rates() {
    let (state, app) = setup_currency_test().await;
    create_test_currency(&app, "USD", "US Dollar", true).await;
    create_test_currency(&app, "EUR", "Euro", false).await;
    create_test_currency(&app, "GBP", "British Pound", false).await;

    set_test_rate(&app, "USD", "EUR", "0.92", "2026-01-15", "daily").await;
    set_test_rate(&app, "USD", "GBP", "0.79", "2026-01-15", "daily").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().uri("/api/v1/exchange-rates?limit=10")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(result["data"].as_array().unwrap().len(), 2);
    cleanup_test_db(&state.db_pool).await;
}

#[tokio::test]
#[ignore]
async fn test_get_exchange_rate() {
    let (state, app) = setup_currency_test().await;
    create_test_currency(&app, "USD", "US Dollar", true).await;
    create_test_currency(&app, "EUR", "Euro", false).await;

    set_test_rate(&app, "USD", "EUR", "0.92", "2026-01-15", "daily").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/exchange-rates/USD/EUR?rate_type=daily&effective_date=2026-01-15")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let rate: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(rate["from_currency"], "USD");
    assert_eq!(rate["to_currency"], "EUR");
    cleanup_test_db(&state.db_pool).await;
}

#[tokio::test]
#[ignore]
async fn test_upsert_exchange_rate() {
    let (state, app) = setup_currency_test().await;
    create_test_currency(&app, "USD", "US Dollar", true).await;
    create_test_currency(&app, "EUR", "Euro", false).await;

    set_test_rate(&app, "USD", "EUR", "0.92", "2026-01-15", "daily").await;
    // Update with new rate for same date
    let updated = set_test_rate(&app, "USD", "EUR", "0.93", "2026-01-15", "daily").await;
    let rate_val: &str = updated["rate"].as_str().unwrap();
    assert!(rate_val.starts_with("0.93"));

    // Should still be only one rate
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().uri("/api/v1/exchange-rates?limit=10")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(result["data"].as_array().unwrap().len(), 1);
    cleanup_test_db(&state.db_pool).await;
}

// ============================================================================
// Currency Conversion Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_convert_currency() {
    let (state, app) = setup_currency_test().await;
    create_test_currency(&app, "USD", "US Dollar", true).await;
    create_test_currency(&app, "EUR", "Euro", false).await;

    set_test_rate(&app, "USD", "EUR", "0.92", "2026-01-15", "daily").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/currency/convert")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "from_currency": "USD", "to_currency": "EUR", "amount": "1000",
            "rate_type": "daily", "effective_date": "2026-01-15"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(result["from_currency"], "USD");
    assert_eq!(result["to_currency"], "EUR");
    assert_eq!(result["from_amount"], "1000");
    assert_eq!(result["to_amount"], "920.00");
    assert_eq!(result["exchange_rate"], "0.9200000000");
    cleanup_test_db(&state.db_pool).await;
}

#[tokio::test]
#[ignore]
async fn test_convert_reverse_direction() {
    let (state, app) = setup_currency_test().await;
    create_test_currency(&app, "USD", "US Dollar", true).await;
    create_test_currency(&app, "EUR", "Euro", false).await;

    // Only set USD->EUR, conversion EUR->USD should use inverse
    set_test_rate(&app, "USD", "EUR", "0.92", "2026-01-15", "daily").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/currency/convert")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "from_currency": "EUR", "to_currency": "USD", "amount": "920",
            "rate_type": "daily", "effective_date": "2026-01-15"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(result["from_currency"], "EUR");
    assert_eq!(result["to_currency"], "USD");
    // 920 * (1/0.92) = 920 * 1.0869... ≈ 1000.00
    let to_amount: f64 = result["to_amount"].as_str().unwrap().parse().unwrap();
    assert!((to_amount - 1000.0).abs() < 1.0);
    cleanup_test_db(&state.db_pool).await;
}

#[tokio::test]
#[ignore]
async fn test_convert_same_currency() {
    let (state, app) = setup_currency_test().await;
    create_test_currency(&app, "USD", "US Dollar", true).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/currency/convert")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "from_currency": "USD", "to_currency": "USD", "amount": "1000",
            "rate_type": "daily", "effective_date": "2026-01-15"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(result["to_amount"], "1000");
    assert_eq!(result["exchange_rate"], "1");
    cleanup_test_db(&state.db_pool).await;
}

#[tokio::test]
#[ignore]
async fn test_convert_no_rate_available() {
    let (state, app) = setup_currency_test().await;
    create_test_currency(&app, "USD", "US Dollar", true).await;
    create_test_currency(&app, "JPY", "Japanese Yen", false).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/currency/convert")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "from_currency": "USD", "to_currency": "JPY", "amount": "100",
            "rate_type": "daily", "effective_date": "2026-01-15"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
    cleanup_test_db(&state.db_pool).await;
}

// ============================================================================
// Triangulation Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_triangulation() {
    let (state, app) = setup_currency_test().await;
    create_test_currency(&app, "USD", "US Dollar", true).await; // base
    create_test_currency(&app, "EUR", "Euro", false).await;
    create_test_currency(&app, "GBP", "British Pound", false).await;

    // Set EUR -> USD and GBP -> USD (no direct EUR -> GBP)
    set_test_rate(&app, "EUR", "USD", "1.0869", "2026-01-15", "daily").await;
    set_test_rate(&app, "GBP", "USD", "1.2658", "2026-01-15", "daily").await;

    // Convert EUR -> GBP should triangulate through USD
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/currency/convert")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "from_currency": "EUR", "to_currency": "GBP", "amount": "1000",
            "rate_type": "daily", "effective_date": "2026-01-15"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(result["from_currency"], "EUR");
    assert_eq!(result["to_currency"], "GBP");
    // EUR->USD = 1.0869, USD->GBP = 1/1.2658 = 0.7900...
    // Cross rate: 1.0869 * (1/1.2658) ≈ 0.8589
    // 1000 EUR * 0.8589 ≈ 858.9 GBP
    let to_amount: f64 = result["to_amount"].as_str().unwrap().parse().unwrap();
    assert!(to_amount > 800.0 && to_amount < 900.0, "Expected ~859 GBP, got {}", to_amount);
    cleanup_test_db(&state.db_pool).await;
}

// ============================================================================
// Gain/Loss Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_unrealized_gain_loss() {
    let (state, app) = setup_currency_test().await;
    create_test_currency(&app, "USD", "US Dollar", true).await;
    create_test_currency(&app, "EUR", "Euro", false).await;

    // Original rate: 1 EUR = 1.08 USD
    set_test_rate(&app, "EUR", "USD", "1.08", "2026-01-01", "daily").await;
    // Current rate: 1 EUR = 1.12 USD (EUR strengthened => gain for EUR holder)
    set_test_rate(&app, "EUR", "USD", "1.12", "2026-03-15", "daily").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/currency/gain-loss")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "currency": "EUR", "original_amount": "10000",
            "original_rate": "1.08", "revaluation_date": "2026-03-15",
            "rate_type": "daily"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(result["gain_loss_type"], "gain");
    // Original: 10000 * 1.08 = 10800 USD
    // Revalued: 10000 * 1.12 = 11200 USD
    // Gain: 400 USD
    let gain_amount: f64 = result["gain_loss_amount"].as_str().unwrap().parse().unwrap();
    assert!((gain_amount - 400.0).abs() < 1.0, "Expected ~400 gain, got {}", gain_amount);
    cleanup_test_db(&state.db_pool).await;
}

#[tokio::test]
#[ignore]
async fn test_unrealized_loss() {
    let (state, app) = setup_currency_test().await;
    create_test_currency(&app, "USD", "US Dollar", true).await;
    create_test_currency(&app, "GBP", "British Pound", false).await;

    // Original: 1 GBP = 1.30 USD
    set_test_rate(&app, "GBP", "USD", "1.30", "2026-01-01", "daily").await;
    // Current: 1 GBP = 1.20 USD (GBP weakened => loss for GBP holder)
    set_test_rate(&app, "GBP", "USD", "1.20", "2026-06-15", "daily").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/currency/gain-loss")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "currency": "GBP", "original_amount": "5000",
            "original_rate": "1.30", "revaluation_date": "2026-06-15",
            "rate_type": "daily"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(result["gain_loss_type"], "loss");
    // Original: 5000 * 1.30 = 6500 USD
    // Revalued: 5000 * 1.20 = 6000 USD
    // Loss: 500 USD
    let loss_amount: f64 = result["gain_loss_amount"].as_str().unwrap().parse().unwrap();
    assert!((loss_amount - 500.0).abs() < 1.0, "Expected ~500 loss, got {}", loss_amount);
    cleanup_test_db(&state.db_pool).await;
}

// ============================================================================
// Bulk Import Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_import_rates() {
    let (state, app) = setup_currency_test().await;
    create_test_currency(&app, "USD", "US Dollar", true).await;
    create_test_currency(&app, "EUR", "Euro", false).await;
    create_test_currency(&app, "GBP", "British Pound", false).await;
    create_test_currency(&app, "JPY", "Japanese Yen", false).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/exchange-rates/import")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "rates": [
                {"from_currency": "USD", "to_currency": "EUR", "rate_type": "daily", "rate": "0.92", "effective_date": "2026-01-15", "source": "ECB"},
                {"from_currency": "USD", "to_currency": "GBP", "rate_type": "daily", "rate": "0.79", "effective_date": "2026-01-15", "source": "ECB"},
                {"from_currency": "USD", "to_currency": "JPY", "rate_type": "daily", "rate": "149.50", "effective_date": "2026-01-15", "source": "BOJ"}
            ]
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(result["imported"], 3);
    assert_eq!(result["failed"], 0);

    // Verify rates are there
    let r2 = app.clone().oneshot(Request::builder().uri("/api/v1/exchange-rates?limit=10")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let b2 = axum::body::to_bytes(r2.into_body(), usize::MAX).await.unwrap();
    let list: serde_json::Value = serde_json::from_slice(&b2).unwrap();
    assert_eq!(list["data"].as_array().unwrap().len(), 3);
    cleanup_test_db(&state.db_pool).await;
}

#[tokio::test]
#[ignore]
async fn test_import_rates_partial_failure() {
    let (state, app) = setup_currency_test().await;
    create_test_currency(&app, "USD", "US Dollar", true).await;
    create_test_currency(&app, "EUR", "Euro", false).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/exchange-rates/import")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "rates": [
                {"from_currency": "USD", "to_currency": "EUR", "rate_type": "daily", "rate": "0.92", "effective_date": "2026-01-15"},
                {"from_currency": "USD", "to_currency": "USD", "rate_type": "daily", "rate": "1", "effective_date": "2026-01-15"}
            ]
        })).unwrap())).unwrap()
    ).await.unwrap();
    // Partial content since one failed
    assert!(r.status() == StatusCode::OK || r.status() == StatusCode::PARTIAL_CONTENT);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(result["imported"], 1);
    assert_eq!(result["failed"], 1);
    cleanup_test_db(&state.db_pool).await;
}
