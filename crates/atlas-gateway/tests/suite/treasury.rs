//! Treasury Management E2E Tests
//!
//! Tests for Oracle Fusion Cloud ERP Treasury Management:
//! - Counterparty CRUD
//! - Treasury deal creation (investment, borrowing, FX)
//! - Deal lifecycle (authorize → settle → mature)
//! - Interest calculations
//! - Settlement tracking
//! - Dashboard summary
//! - Validation edge cases

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_treasury_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    let app = build_router(state.clone());
    (state, app)
}

async fn create_test_counterparty(
    app: &axum::Router,
    code: &str,
    name: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "counterpartyCode": code,
        "name": name,
        "counterpartyType": "bank",
        "countryCode": "US",
        "creditRating": "A+",
        "creditLimit": "10000000.00",
        "settlementCurrency": "USD",
        "contactName": "Jane Smith",
        "contactEmail": "jane@example.com",
    });
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/treasury/counterparties")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Failed to create counterparty: status {}", r.status());
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_investment_deal(
    app: &axum::Router,
    counterparty_id: &str,
    principal: &str,
    rate: &str,
    term_label: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let (start, end) = match term_label {
        "short" => ("2024-01-01", "2024-04-01"),   // 91 days
        "medium" => ("2024-01-01", "2024-07-01"),   // 182 days
        "long" => ("2024-01-01", "2025-01-01"),      // 365 days
        _ => ("2024-01-01", "2024-02-01"),            // 31 days
    };
    let payload = json!({
        "dealType": "investment",
        "description": "Test money market investment",
        "counterpartyId": counterparty_id,
        "currencyCode": "USD",
        "principalAmount": principal,
        "interestRate": rate,
        "interestBasis": "actual_360",
        "startDate": start,
        "maturityDate": end,
        "glAccountCode": "1200",
    });
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/treasury/deals")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Failed to create deal: status {}", r.status());
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Counterparty Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_create_counterparty() {
    let (_state, app) = setup_treasury_test().await;

    let cp = create_test_counterparty(&app, "BANK001", "First National Bank").await;

    assert_eq!(cp["counterparty_code"], "BANK001");
    assert_eq!(cp["name"], "First National Bank");
    assert_eq!(cp["counterparty_type"], "bank");
    assert_eq!(cp["is_active"], true);
}

#[tokio::test]
#[ignore]
async fn test_list_counterparties() {
    let (_state, app) = setup_treasury_test().await;

    create_test_counterparty(&app, "BANK001", "First National Bank").await;
    create_test_counterparty(&app, "BANK002", "Global Finance Corp").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET").uri("/api/v1/treasury/counterparties")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(result["data"].as_array().unwrap().len(), 2);
}

#[tokio::test]
#[ignore]
async fn test_get_counterparty() {
    let (_state, app) = setup_treasury_test().await;

    create_test_counterparty(&app, "BANK001", "First National Bank").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/treasury/counterparties/BANK001")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(result["name"], "First National Bank");
}

#[tokio::test]
#[ignore]
async fn test_delete_counterparty() {
    let (_state, app) = setup_treasury_test().await;

    create_test_counterparty(&app, "BANK001", "First National Bank").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/treasury/counterparties/BANK001")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Deal Creation Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_create_investment_deal() {
    let (_state, app) = setup_treasury_test().await;

    let cp = create_test_counterparty(&app, "BANK001", "First National Bank").await;
    let cp_id = cp["id"].as_str().unwrap();

    let deal = create_test_investment_deal(&app, cp_id, "1000000.00", "0.05", "short").await;

    assert_eq!(deal["deal_type"], "investment");
    assert_eq!(deal["status"], "draft");
    assert_eq!(deal["currency_code"], "USD");
    assert!(deal["deal_number"].as_str().unwrap().starts_with("TRD-"));
    assert_eq!(deal["term_days"], 91);
    assert_eq!(deal["interest_basis"], "actual_360");
}

#[tokio::test]
#[ignore]
async fn test_create_borrowing_deal() {
    let (_state, app) = setup_treasury_test().await;

    let cp = create_test_counterparty(&app, "BANK001", "First National Bank").await;
    let cp_id = cp["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "dealType": "borrowing",
        "description": "Working capital line",
        "counterpartyId": cp_id,
        "currencyCode": "USD",
        "principalAmount": "5000000.00",
        "interestRate": "0.045",
        "interestBasis": "actual_365",
        "startDate": "2024-01-15",
        "maturityDate": "2024-07-15",
        "glAccountCode": "2100",
    });
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/treasury/deals")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let deal: serde_json::Value = serde_json::from_slice(&b).unwrap();

    assert_eq!(deal["deal_type"], "borrowing");
    assert_eq!(deal["status"], "draft");
    assert_eq!(deal["interest_basis"], "actual_365");
}

#[tokio::test]
#[ignore]
async fn test_create_fx_deal() {
    let (_state, app) = setup_treasury_test().await;

    let cp = create_test_counterparty(&app, "BANK001", "First National Bank").await;
    let cp_id = cp["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "dealType": "fx_forward",
        "description": "EUR/USD forward hedge",
        "counterpartyId": cp_id,
        "currencyCode": "EUR",
        "principalAmount": "0",
        "startDate": "2024-01-15",
        "maturityDate": "2024-04-15",
        "fxBuyCurrency": "EUR",
        "fxBuyAmount": "1000000.00",
        "fxSellCurrency": "USD",
        "fxSellAmount": "1085000.00",
        "fxRate": "1.0850",
    });
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/treasury/deals")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let deal: serde_json::Value = serde_json::from_slice(&b).unwrap();

    assert_eq!(deal["deal_type"], "fx_forward");
    assert_eq!(deal["fx_buy_currency"], "EUR");
    assert_eq!(deal["fx_sell_currency"], "USD");
    assert_eq!(deal["fx_rate"], "1.085");
}

// ============================================================================
// Deal Lifecycle Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_deal_full_lifecycle() {
    let (_state, app) = setup_treasury_test().await;

    let cp = create_test_counterparty(&app, "BANK001", "First National Bank").await;
    let cp_id = cp["id"].as_str().unwrap();

    // 1. Create deal
    let deal = create_test_investment_deal(&app, cp_id, "1000000.00", "0.05", "short").await;
    let deal_id = deal["id"].as_str().unwrap();
    assert_eq!(deal["status"], "draft");

    // 2. Authorize
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/treasury/deals/{}/authorize", deal_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let authorized: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(authorized["status"], "authorized");
    // Accrued interest should be positive
    let interest: f64 = authorized["accrued_interest"].as_str().unwrap().parse().unwrap();
    assert!(interest > 0.0, "Accrued interest should be positive after authorization");

    // 3. Settle
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/treasury/deals/{}/settle", deal_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "settlementType": "full",
            "paymentReference": "PAY-TREAS-001"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let settlement: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(settlement["settlement_type"], "full");
    assert_eq!(settlement["payment_reference"], "PAY-TREAS-001");
    let total: f64 = settlement["total_amount"].as_str().unwrap().parse().unwrap();
    assert!(total > 1000000.0, "Total settlement should exceed principal");

    // 4. Verify deal is settled
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/treasury/deals/{}", deal_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(updated["status"], "settled");

    // 5. Mature
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/treasury/deals/{}/mature", deal_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let matured: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(matured["status"], "matured");
}

#[tokio::test]
#[ignore]
async fn test_cancel_draft_deal() {
    let (_state, app) = setup_treasury_test().await;

    let cp = create_test_counterparty(&app, "BANK001", "First National Bank").await;
    let cp_id = cp["id"].as_str().unwrap();

    let deal = create_test_investment_deal(&app, cp_id, "500000.00", "0.04", "short").await;
    let deal_id = deal["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/treasury/deals/{}/cancel", deal_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let cancelled: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(cancelled["status"], "cancelled");
}

// ============================================================================
// Settlement Listing Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_list_deal_settlements() {
    let (_state, app) = setup_treasury_test().await;

    let cp = create_test_counterparty(&app, "BANK001", "First National Bank").await;
    let cp_id = cp["id"].as_str().unwrap();

    let deal = create_test_investment_deal(&app, cp_id, "1000000.00", "0.05", "short").await;
    let deal_id = deal["id"].as_str().unwrap();

    // Authorize and settle
    let (k, v) = auth_header(&admin_claims());
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/treasury/deals/{}/authorize", deal_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/treasury/deals/{}/settle", deal_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "settlementType": "full"
        })).unwrap())).unwrap()
    ).await.unwrap();

    // List settlements
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/treasury/deals/{}/settlements", deal_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(result["data"].as_array().unwrap().len(), 1);
    let settlement = &result["data"][0];
    assert_eq!(settlement["settlement_type"], "full");
}

// ============================================================================
// Validation Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_cannot_create_deal_with_invalid_type() {
    let (_state, app) = setup_treasury_test().await;

    let cp = create_test_counterparty(&app, "BANK001", "First National Bank").await;
    let cp_id = cp["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/treasury/deals")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "dealType": "derivative",
            "counterpartyId": cp_id,
            "principalAmount": "1000000",
            "interestRate": "0.05",
            "startDate": "2024-01-01",
            "maturityDate": "2024-04-01",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore]
async fn test_cannot_create_deal_with_reversed_dates() {
    let (_state, app) = setup_treasury_test().await;

    let cp = create_test_counterparty(&app, "BANK001", "First National Bank").await;
    let cp_id = cp["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/treasury/deals")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "dealType": "investment",
            "counterpartyId": cp_id,
            "principalAmount": "1000000",
            "interestRate": "0.05",
            "startDate": "2024-04-01",
            "maturityDate": "2024-01-01",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore]
async fn test_cannot_authorize_non_draft_deal() {
    let (_state, app) = setup_treasury_test().await;

    let cp = create_test_counterparty(&app, "BANK001", "First National Bank").await;
    let cp_id = cp["id"].as_str().unwrap();

    let deal = create_test_investment_deal(&app, cp_id, "500000.00", "0.04", "short").await;
    let deal_id = deal["id"].as_str().unwrap();

    // Authorize once
    let (k, v) = auth_header(&admin_claims());
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/treasury/deals/{}/authorize", deal_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Try to authorize again
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/treasury/deals/{}/authorize", deal_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore]
async fn test_cannot_settle_draft_deal() {
    let (_state, app) = setup_treasury_test().await;

    let cp = create_test_counterparty(&app, "BANK001", "First National Bank").await;
    let cp_id = cp["id"].as_str().unwrap();

    let deal = create_test_investment_deal(&app, cp_id, "500000.00", "0.04", "short").await;
    let deal_id = deal["id"].as_str().unwrap();

    // Try to settle without authorizing
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/treasury/deals/{}/settle", deal_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "settlementType": "full"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore]
async fn test_cannot_cancel_authorized_deal() {
    let (_state, app) = setup_treasury_test().await;

    let cp = create_test_counterparty(&app, "BANK001", "First National Bank").await;
    let cp_id = cp["id"].as_str().unwrap();

    let deal = create_test_investment_deal(&app, cp_id, "500000.00", "0.04", "short").await;
    let deal_id = deal["id"].as_str().unwrap();

    // Authorize first
    let (k, v) = auth_header(&admin_claims());
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/treasury/deals/{}/authorize", deal_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Try to cancel an authorized deal
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/treasury/deals/{}/cancel", deal_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore]
async fn test_fx_deal_requires_fx_fields() {
    let (_state, app) = setup_treasury_test().await;

    let cp = create_test_counterparty(&app, "BANK001", "First National Bank").await;
    let cp_id = cp["id"].as_str().unwrap();

    // Missing fx fields
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/treasury/deals")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "dealType": "fx_spot",
            "counterpartyId": cp_id,
            "principalAmount": "0",
            "startDate": "2024-01-01",
            "maturityDate": "2024-01-02",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Dashboard Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_treasury_dashboard() {
    let (_state, app) = setup_treasury_test().await;

    let cp = create_test_counterparty(&app, "BANK001", "First National Bank").await;
    let cp_id = cp["id"].as_str().unwrap();

    // Create and authorize deals
    let deal1 = create_test_investment_deal(&app, cp_id, "1000000.00", "0.05", "short").await;
    let deal2 = create_test_investment_deal(&app, cp_id, "2000000.00", "0.04", "medium").await;

    let (k, v) = auth_header(&admin_claims());
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/treasury/deals/{}/authorize", deal1["id"].as_str().unwrap()))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/treasury/deals/{}/authorize", deal2["id"].as_str().unwrap()))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Get dashboard
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/treasury/dashboard")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let summary: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(summary["total_active_deals"], 2);
    assert_eq!(summary["investment_count"], 2);
    assert!(summary["total_investments"].as_str().unwrap().parse::<f64>().unwrap() > 0.0);
    assert!(summary["total_accrued_interest"].as_str().unwrap().parse::<f64>().unwrap() > 0.0);
    assert_eq!(summary["active_counterparties"], 1);
}

// ============================================================================
// List & Filter Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_list_deals_by_type() {
    let (_state, app) = setup_treasury_test().await;

    let cp = create_test_counterparty(&app, "BANK001", "First National Bank").await;
    let cp_id = cp["id"].as_str().unwrap();

    create_test_investment_deal(&app, cp_id, "1000000.00", "0.05", "short").await;
    create_test_investment_deal(&app, cp_id, "2000000.00", "0.04", "medium").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/treasury/deals?deal_type=investment")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(result["data"].as_array().unwrap().len(), 2);
    for deal in result["data"].as_array().unwrap() {
        assert_eq!(deal["deal_type"], "investment");
    }
}

#[tokio::test]
#[ignore]
async fn test_list_deals_by_status() {
    let (_state, app) = setup_treasury_test().await;

    let cp = create_test_counterparty(&app, "BANK001", "First National Bank").await;
    let cp_id = cp["id"].as_str().unwrap();

    create_test_investment_deal(&app, cp_id, "1000000.00", "0.05", "short").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/treasury/deals?status=draft")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(result["data"].as_array().unwrap().len(), 1);
    assert_eq!(result["data"][0]["status"], "draft");
}
