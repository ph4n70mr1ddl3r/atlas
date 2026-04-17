//! Tax Management E2E Tests
//!
//! Tests for Oracle Fusion Cloud ERP Tax Management:
//! - Tax regime CRUD
//! - Tax jurisdiction management
//! - Tax rate management
//! - Tax determination rules
//! - Tax calculation (exclusive and inclusive)
//! - Tax recovery
//! - Tax reporting

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_tax_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    let app = build_router(state.clone());
    (state, app)
}

async fn create_test_regime(app: &axum::Router, code: &str, name: &str, tax_type: &str) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/tax/regimes")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": code, "name": name, "tax_type": tax_type,
            "default_inclusive": false, "allows_recovery": false,
            "rounding_rule": "nearest", "rounding_precision": 2
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_jurisdiction(
    app: &axum::Router, regime_code: &str, code: &str, name: &str, country: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/tax/jurisdictions")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "regime_code": regime_code, "code": code, "name": name,
            "geographic_level": "country", "country_code": country
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_rate(
    app: &axum::Router, regime_code: &str, code: &str, name: &str, percentage: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/tax/rates")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "regime_code": regime_code, "code": code, "name": name,
            "rate_percentage": percentage, "rate_type": "standard"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Tax Regime Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_create_tax_regime() {
    let (state, app) = setup_tax_test().await;
    let regime = create_test_regime(&app, "US_SALES_TAX", "US Sales Tax", "sales_tax").await;
    assert_eq!(regime["code"], "US_SALES_TAX");
    assert_eq!(regime["name"], "US Sales Tax");
    assert_eq!(regime["tax_type"], "sales_tax");
    assert_eq!(regime["default_inclusive"], false);
    assert_eq!(regime["is_active"], true);
    cleanup_test_db(&state.db_pool).await;
}

#[tokio::test]
#[ignore]
async fn test_list_tax_regimes() {
    let (state, app) = setup_tax_test().await;
    create_test_regime(&app, "US_SALES_TAX", "US Sales Tax", "sales_tax").await;
    create_test_regime(&app, "EU_VAT", "European VAT", "vat").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().uri("/api/v1/tax/regimes")
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
async fn test_get_tax_regime() {
    let (state, app) = setup_tax_test().await;
    create_test_regime(&app, "EU_VAT", "European VAT", "vat").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().uri("/api/v1/tax/regimes/EU_VAT")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let regime: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(regime["code"], "EU_VAT");
    assert_eq!(regime["name"], "European VAT");
    cleanup_test_db(&state.db_pool).await;
}

#[tokio::test]
#[ignore]
async fn test_create_tax_regime_invalid_type() {
    let (state, app) = setup_tax_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/tax/regimes")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "TEST", "name": "Test", "tax_type": "invalid_type"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
    cleanup_test_db(&state.db_pool).await;
}

#[tokio::test]
#[ignore]
async fn test_delete_tax_regime() {
    let (state, app) = setup_tax_test().await;
    create_test_regime(&app, "EU_VAT", "European VAT", "vat").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE").uri("/api/v1/tax/regimes/EU_VAT")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);

    // Verify it's gone
    let r2 = app.clone().oneshot(Request::builder().uri("/api/v1/tax/regimes/EU_VAT")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r2.status(), StatusCode::NOT_FOUND);
    cleanup_test_db(&state.db_pool).await;
}

// ============================================================================
// Tax Jurisdiction Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_create_tax_jurisdiction() {
    let (state, app) = setup_tax_test().await;
    create_test_regime(&app, "EU_VAT", "European VAT", "vat").await;

    let juris = create_test_jurisdiction(&app, "EU_VAT", "DE", "Germany", "DE").await;
    assert_eq!(juris["code"], "DE");
    assert_eq!(juris["name"], "Germany");
    assert_eq!(juris["geographic_level"], "country");
    assert_eq!(juris["country_code"], "DE");
    cleanup_test_db(&state.db_pool).await;
}

#[tokio::test]
#[ignore]
async fn test_list_jurisdictions() {
    let (state, app) = setup_tax_test().await;
    create_test_regime(&app, "EU_VAT", "European VAT", "vat").await;
    create_test_jurisdiction(&app, "EU_VAT", "DE", "Germany", "DE").await;
    create_test_jurisdiction(&app, "EU_VAT", "FR", "France", "FR").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().uri("/api/v1/tax/jurisdictions?regime_code=EU_VAT")
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
async fn test_create_jurisdiction_invalid_regime() {
    let (state, app) = setup_tax_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/tax/jurisdictions")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "regime_code": "NONEXISTENT", "code": "XX", "name": "Test",
            "geographic_level": "country"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
    cleanup_test_db(&state.db_pool).await;
}

// ============================================================================
// Tax Rate Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_create_tax_rate() {
    let (state, app) = setup_tax_test().await;
    create_test_regime(&app, "EU_VAT", "European VAT", "vat").await;

    let rate = create_test_rate(&app, "EU_VAT", "DE_STANDARD", "Germany Standard VAT", "19.0").await;
    assert_eq!(rate["code"], "DE_STANDARD");
    assert_eq!(rate["name"], "Germany Standard VAT");
    // Rate percentage is a NUMERIC serialized as string/number
    let rate_pct = rate["rate_percentage"].as_str().unwrap_or_else(|| "");
    // Rate percentage might be serialized as a number, so also check to_string
    let rate_str = if rate_pct.is_empty() { rate["rate_percentage"].to_string() } else { rate_pct.to_string() };
    assert!(rate_str.contains("19"));
    assert_eq!(rate["rate_type"], "standard");
    cleanup_test_db(&state.db_pool).await;
}

#[tokio::test]
#[ignore]
async fn test_create_tax_rate_with_jurisdiction() {
    let (state, app) = setup_tax_test().await;
    create_test_regime(&app, "US_SALES_TAX", "US Sales Tax", "sales_tax").await;
    create_test_jurisdiction(&app, "US_SALES_TAX", "CA", "California", "US").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/tax/rates")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "regime_code": "US_SALES_TAX", "jurisdiction_code": "CA",
            "code": "CA_STANDARD", "name": "California Standard",
            "rate_percentage": "8.25", "rate_type": "standard"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let rate: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(rate["code"], "CA_STANDARD");
    assert!(rate["jurisdiction_id"].is_string()); // Should be set
    cleanup_test_db(&state.db_pool).await;
}

#[tokio::test]
#[ignore]
async fn test_list_tax_rates() {
    let (state, app) = setup_tax_test().await;
    create_test_regime(&app, "EU_VAT", "European VAT", "vat").await;
    create_test_rate(&app, "EU_VAT", "DE_STANDARD", "Germany Standard", "19.0").await;
    create_test_rate(&app, "EU_VAT", "DE_REDUCED", "Germany Reduced", "7.0").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().uri("/api/v1/tax/rates/EU_VAT")
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
async fn test_create_tax_rate_negative_percentage() {
    let (state, app) = setup_tax_test().await;
    create_test_regime(&app, "EU_VAT", "European VAT", "vat").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/tax/rates")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "regime_code": "EU_VAT", "code": "BAD", "name": "Bad Rate",
            "rate_percentage": "-5.0", "rate_type": "standard"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
    cleanup_test_db(&state.db_pool).await;
}

#[tokio::test]
#[ignore]
async fn test_delete_tax_rate() {
    let (state, app) = setup_tax_test().await;
    create_test_regime(&app, "EU_VAT", "European VAT", "vat").await;
    create_test_rate(&app, "EU_VAT", "DE_STANDARD", "Germany Standard", "19.0").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/tax/rates/EU_VAT/DE_STANDARD")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
    cleanup_test_db(&state.db_pool).await;
}

// ============================================================================
// Tax Calculation Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_calculate_tax_exclusive() {
    let (state, app) = setup_tax_test().await;
    create_test_regime(&app, "EU_VAT", "European VAT", "vat").await;
    create_test_rate(&app, "EU_VAT", "DE_STANDARD", "Germany Standard VAT", "19.0").await;

    // Calculate tax on 1000 EUR with 19% VAT (exclusive)
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/tax/calculate")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "entity_type": "sales_orders",
            "lines": [{
                "amount": "1000",
                "tax_rate_codes": ["DE_STANDARD"],
                "is_inclusive": false
            }],
            "context": {},
            "persist": false
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();

    // Tax should be 190.00 (1000 * 19%)
    assert_eq!(result["total_taxable_amount"], "1000.00");
    assert_eq!(result["total_tax_amount"], "190.00");
    assert_eq!(result["lines"].as_array().unwrap().len(), 1);
    assert_eq!(result["lines"][0]["tax_rate_code"], "DE_STANDARD");
    assert_eq!(result["lines"][0]["tax_amount"], "190.00");
    cleanup_test_db(&state.db_pool).await;
}

#[tokio::test]
#[ignore]
async fn test_calculate_tax_inclusive() {
    let (state, app) = setup_tax_test().await;
    // Create regime with inclusive tax
    let (k, v) = auth_header(&admin_claims());
    let _r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/tax/regimes")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "IN_GST", "name": "India GST", "tax_type": "gst",
            "default_inclusive": true, "allows_recovery": true,
            "rounding_rule": "nearest", "rounding_precision": 2
        })).unwrap())).unwrap()
    ).await.unwrap();

    create_test_rate(&app, "IN_GST", "GST_18", "GST 18%", "18.0").await;

    // Calculate tax on 1180 INR (inclusive of 18% GST)
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/tax/calculate")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "entity_type": "sales_orders",
            "lines": [{
                "amount": "1180",
                "tax_rate_codes": ["GST_18"],
                "is_inclusive": true
            }],
            "context": {},
            "persist": false
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();

    // Taxable: 1180 / 1.18 = 1000, Tax: 180
    let taxable: f64 = result["total_taxable_amount"].as_str().unwrap().parse().unwrap();
    let tax: f64 = result["total_tax_amount"].as_str().unwrap().parse().unwrap();
    assert!((taxable - 1000.0).abs() < 1.0, "Expected taxable ~1000, got {}", taxable);
    assert!((tax - 180.0).abs() < 1.0, "Expected tax ~180, got {}", tax);
    cleanup_test_db(&state.db_pool).await;
}

#[tokio::test]
#[ignore]
async fn test_calculate_tax_multiple_lines() {
    let (state, app) = setup_tax_test().await;
    create_test_regime(&app, "EU_VAT", "European VAT", "vat").await;
    create_test_rate(&app, "EU_VAT", "DE_STANDARD", "Germany Standard VAT", "19.0").await;
    create_test_rate(&app, "EU_VAT", "DE_REDUCED", "Germany Reduced VAT", "7.0").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/tax/calculate")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "entity_type": "sales_orders",
            "lines": [
                {"amount": "1000", "tax_rate_codes": ["DE_STANDARD"], "is_inclusive": false},
                {"amount": "500", "tax_rate_codes": ["DE_REDUCED"], "is_inclusive": false}
            ],
            "context": {},
            "persist": false
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();

    assert_eq!(result["lines"].as_array().unwrap().len(), 2);
    // Total tax: 1000*0.19 + 500*0.07 = 190 + 35 = 225
    let total_tax: f64 = result["total_tax_amount"].as_str().unwrap().parse().unwrap();
    assert!((total_tax - 225.0).abs() < 1.0, "Expected total tax ~225, got {}", total_tax);
    // Total taxable: 1000 + 500 = 1500
    let total_taxable: f64 = result["total_taxable_amount"].as_str().unwrap().parse().unwrap();
    assert!((total_taxable - 1500.0).abs() < 1.0, "Expected total taxable ~1500, got {}", total_taxable);
    cleanup_test_db(&state.db_pool).await;
}

#[tokio::test]
#[ignore]
async fn test_calculate_tax_with_recovery() {
    let (state, app) = setup_tax_test().await;
    create_test_regime(&app, "EU_VAT", "European VAT", "vat").await;

    // Create a recoverable rate (input tax credit)
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/tax/rates")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "regime_code": "EU_VAT", "code": "DE_RECOVERABLE", "name": "Germany Recoverable",
            "rate_percentage": "19", "rate_type": "standard",
            "recoverable": true, "recovery_percentage": "100"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);

    // Calculate
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/tax/calculate")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "entity_type": "purchase_orders",
            "lines": [{
                "amount": "1000",
                "tax_rate_codes": ["DE_RECOVERABLE"],
                "is_inclusive": false
            }],
            "context": {},
            "persist": false
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();

    let line = &result["lines"].as_array().unwrap()[0];
    assert_eq!(line["recoverable"], true);
    // Full recovery: recoverable_amount = 190, non_recoverable = 0
    let recoverable: f64 = line["recoverable_amount"].as_str().unwrap().parse().unwrap();
    let non_recoverable: f64 = line["non_recoverable_amount"].as_str().unwrap().parse().unwrap();
    assert!((recoverable - 190.0).abs() < 1.0, "Expected recoverable ~190, got {}", recoverable);
    assert!((non_recoverable - 0.0).abs() < 0.01, "Expected non-recoverable ~0, got {}", non_recoverable);
    cleanup_test_db(&state.db_pool).await;
}

#[tokio::test]
#[ignore]
async fn test_calculate_tax_unknown_rate() {
    let (state, app) = setup_tax_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/tax/calculate")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "entity_type": "sales_orders",
            "lines": [{
                "amount": "1000",
                "tax_rate_codes": ["NONEXISTENT_RATE"],
                "is_inclusive": false
            }],
            "context": {},
            "persist": false
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
    cleanup_test_db(&state.db_pool).await;
}

// ============================================================================
// Tax Determination Rule Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_create_determination_rule() {
    let (state, app) = setup_tax_test().await;
    create_test_regime(&app, "US_SALES_TAX", "US Sales Tax", "sales_tax").await;
    create_test_rate(&app, "US_SALES_TAX", "CA_STANDARD", "California Standard", "8.25").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/tax/rules")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "regime_code": "US_SALES_TAX",
            "name": "California Goods Tax",
            "description": "Apply CA sales tax to goods shipped to CA",
            "priority": 10,
            "condition": {
                "product_category": "goods",
                "ship_to_state": "CA"
            },
            "action": {
                "tax_rate_codes": ["CA_STANDARD"]
            },
            "stop_on_match": true
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let rule: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(rule["name"], "California Goods Tax");
    assert_eq!(rule["priority"], 10);
    assert_eq!(rule["stop_on_match"], true);
    cleanup_test_db(&state.db_pool).await;
}

#[tokio::test]
#[ignore]
async fn test_list_determination_rules() {
    let (state, app) = setup_tax_test().await;
    create_test_regime(&app, "US_SALES_TAX", "US Sales Tax", "sales_tax").await;
    create_test_rate(&app, "US_SALES_TAX", "CA_STANDARD", "California Standard", "8.25").await;
    create_test_rate(&app, "US_SALES_TAX", "NY_STANDARD", "New York Standard", "8.0").await;

    let (k, v) = auth_header(&admin_claims());

    // Create two rules
    let _ = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/tax/rules")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "regime_code": "US_SALES_TAX", "name": "CA Rule", "priority": 10,
            "condition": {"ship_to_state": "CA"},
            "action": {"tax_rate_codes": ["CA_STANDARD"]}
        })).unwrap())).unwrap()
    ).await.unwrap();

    let _ = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/tax/rules")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "regime_code": "US_SALES_TAX", "name": "NY Rule", "priority": 20,
            "condition": {"ship_to_state": "NY"},
            "action": {"tax_rate_codes": ["NY_STANDARD"]}
        })).unwrap())).unwrap()
    ).await.unwrap();

    // List rules
    let r = app.clone().oneshot(Request::builder().uri("/api/v1/tax/rules/US_SALES_TAX")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(result["data"].as_array().unwrap().len(), 2);
    // Should be sorted by priority
    assert_eq!(result["data"][0]["name"], "CA Rule");
    assert_eq!(result["data"][1]["name"], "NY Rule");
    cleanup_test_db(&state.db_pool).await;
}

// ============================================================================
// Tax Lines (persisted) Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_calculate_tax_with_persist() {
    let (state, app) = setup_tax_test().await;
    create_test_regime(&app, "EU_VAT", "European VAT", "vat").await;
    create_test_rate(&app, "EU_VAT", "DE_STANDARD", "Germany Standard VAT", "19.0").await;

    let entity_id = uuid::Uuid::new_v4();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/tax/calculate")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "entity_type": "sales_orders",
            "entity_id": entity_id,
            "lines": [{
                "amount": "1000",
                "tax_rate_codes": ["DE_STANDARD"],
                "is_inclusive": false
            }],
            "context": {},
            "persist": true
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    // Verify tax lines were persisted
    let r2 = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/tax/lines/sales_orders/{}", entity_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r2.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r2.into_body(), usize::MAX).await.unwrap();
    let lines: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(lines["data"].as_array().unwrap().len(), 1);
    cleanup_test_db(&state.db_pool).await;
}

// ============================================================================
// Tax Reporting Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_generate_tax_report() {
    let (state, app) = setup_tax_test().await;
    create_test_regime(&app, "EU_VAT", "European VAT", "vat").await;
    create_test_rate(&app, "EU_VAT", "DE_STANDARD", "Germany Standard VAT", "19.0").await;

    // Create some tax lines first
    let entity_id = uuid::Uuid::new_v4();
    let (k, v) = auth_header(&admin_claims());
    let _ = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/tax/calculate")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "entity_type": "sales_orders",
            "entity_id": entity_id,
            "lines": [{"amount": "1000", "tax_rate_codes": ["DE_STANDARD"], "is_inclusive": false}],
            "context": {}, "persist": true
        })).unwrap())).unwrap()
    ).await.unwrap();

    // Generate report
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/tax/reports")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "regime_code": "EU_VAT",
            "period_start": "2020-01-01",
            "period_end": "2030-12-31"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let report: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(report["status"], "draft");
    assert_eq!(report["transaction_count"], 1);
    cleanup_test_db(&state.db_pool).await;
}

#[tokio::test]
#[ignore]
async fn test_list_tax_reports() {
    let (state, app) = setup_tax_test().await;
    create_test_regime(&app, "EU_VAT", "European VAT", "vat").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().uri("/api/v1/tax/reports?regime_code=EU_VAT")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(result["data"].as_array().unwrap().is_empty());
    cleanup_test_db(&state.db_pool).await;
}

// ============================================================================
// Upsert / Update Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_upsert_tax_regime() {
    let (state, app) = setup_tax_test().await;

    // Create
    create_test_regime(&app, "EU_VAT", "European VAT", "vat").await;

    // Update with same code
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/tax/regimes")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "EU_VAT", "name": "European Union VAT Updated", "tax_type": "vat",
            "rounding_rule": "nearest", "rounding_precision": 2
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let regime: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(regime["name"], "European Union VAT Updated");

    // Should still be only one regime
    let r2 = app.clone().oneshot(Request::builder().uri("/api/v1/tax/regimes")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let b2 = axum::body::to_bytes(r2.into_body(), usize::MAX).await.unwrap();
    let list: serde_json::Value = serde_json::from_slice(&b2).unwrap();
    assert_eq!(list["data"].as_array().unwrap().len(), 1);
    cleanup_test_db(&state.db_pool).await;
}
