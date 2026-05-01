//! Rebate Management E2E Tests
//!
//! Tests for Oracle Fusion Trade Management > Rebates:
//! - Rebate agreement CRUD and lifecycle (draft -> active -> on_hold -> terminated)
//! - Rebate tier management (volume-based pricing thresholds)
//! - Rebate transaction recording and tiered calculation
//! - Rebate accrual creation, posting, and reversal
//! - Rebate settlement approval and payment workflow
//! - Dashboard analytics
//! - Validation edge cases and error handling
//! - Full end-to-end lifecycle test

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_rebate_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    let app = build_router(state.clone());
    (state, app)
}

// ============================================================================
// Helper functions
// ============================================================================

async fn create_test_agreement(
    app: &axum::Router,
    number: &str,
    name: &str,
    rebate_type: &str,
    direction: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/rebate/agreements")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "agreementNumber": number,
                        "name": name,
                        "rebateType": rebate_type,
                        "direction": direction,
                        "partnerType": "supplier",
                        "partnerName": "Test Supplier Corp",
                        "partnerNumber": "SUP-001",
                        "currencyCode": "USD",
                        "startDate": "2024-01-01",
                        "endDate": "2024-12-31",
                        "calculationMethod": "tiered",
                        "settlementFrequency": "quarterly"
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
        panic!(
            "Expected CREATED for agreement but got {}: {}",
            status,
            String::from_utf8_lossy(&b)
        );
    }
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_tier(
    app: &axum::Router,
    agreement_id: &str,
    tier_number: i32,
    from_value: f64,
    to_value: Option<f64>,
    rate: f64,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let mut body = json!({
        "tierNumber": tier_number,
        "fromValue": from_value,
        "rebateRate": rate,
        "rateType": "percentage"
    });
    if let Some(tv) = to_value {
        body["toValue"] = json!(tv);
    }
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/rebate/agreements/{}/tiers", agreement_id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = resp.status();
    let b = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    if status != StatusCode::CREATED {
        panic!(
            "Expected CREATED for tier but got {}: {}",
            status,
            String::from_utf8_lossy(&b)
        );
    }
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Agreement Tests
// ============================================================================

#[tokio::test]
async fn test_create_agreement() {
    let (_state, app) = setup_rebate_test().await;
    let agreement = create_test_agreement(&app, "RBA-001", "Supplier Volume Rebate", "supplier_rebate", "payable").await;
    assert_eq!(agreement["agreementNumber"], "RBA-001");
    assert_eq!(agreement["name"], "Supplier Volume Rebate");
    assert_eq!(agreement["rebateType"], "supplier_rebate");
    assert_eq!(agreement["direction"], "payable");
    assert_eq!(agreement["status"], "draft");
    assert_eq!(agreement["currencyCode"], "USD");
}

#[tokio::test]
async fn test_create_agreement_duplicate_conflict() {
    let (_state, app) = setup_rebate_test().await;
    create_test_agreement(&app, "DUP-RBA", "First", "supplier_rebate", "payable").await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/rebate/agreements")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "agreementNumber": "DUP-RBA",
                        "name": "Duplicate",
                        "rebateType": "supplier_rebate",
                        "direction": "payable",
                        "partnerType": "supplier",
                        "currencyCode": "USD",
                        "startDate": "2024-01-01",
                        "endDate": "2024-12-31"
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
async fn test_create_agreement_invalid_type() {
    let (_state, app) = setup_rebate_test().await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/rebate/agreements")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "agreementNumber": "BAD-TYPE",
                        "name": "Bad Type",
                        "rebateType": "invalid_type",
                        "direction": "payable",
                        "partnerType": "supplier",
                        "currencyCode": "USD",
                        "startDate": "2024-01-01",
                        "endDate": "2024-12-31"
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
async fn test_create_agreement_invalid_dates() {
    let (_state, app) = setup_rebate_test().await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/rebate/agreements")
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "agreementNumber": "BAD-DATE",
                        "name": "Bad Dates",
                        "rebateType": "supplier_rebate",
                        "direction": "payable",
                        "partnerType": "supplier",
                        "currencyCode": "USD",
                        "startDate": "2025-01-01",
                        "endDate": "2024-01-01"
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
async fn test_get_agreement() {
    let (_state, app) = setup_rebate_test().await;
    let agreement = create_test_agreement(&app, "GET-RBA", "Get Me", "customer_rebate", "receivable").await;
    let id = agreement["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/rebate/agreements/{}", id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let fetched: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(fetched["agreementNumber"], "GET-RBA");
    assert_eq!(fetched["rebateType"], "customer_rebate");
}

#[tokio::test]
async fn test_list_agreements() {
    let (_state, app) = setup_rebate_test().await;
    create_test_agreement(&app, "LIST-1", "Agreement One", "supplier_rebate", "payable").await;
    create_test_agreement(&app, "LIST-2", "Agreement Two", "customer_rebate", "receivable").await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/rebate/agreements")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let list: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(list["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_activate_agreement() {
    let (_state, app) = setup_rebate_test().await;
    let agreement = create_test_agreement(&app, "ACT-RBA", "Activate Me", "supplier_rebate", "payable").await;
    let id = agreement["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/rebate/agreements/{}/activate", id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(updated["status"], "active");
}

#[tokio::test]
async fn test_hold_and_terminate_agreement() {
    let (_state, app) = setup_rebate_test().await;
    let agreement = create_test_agreement(&app, "HOLD-RBA", "Hold Me", "supplier_rebate", "payable").await;
    let id = agreement["id"].as_str().unwrap();

    // Activate first
    let (k, v) = auth_header(&admin_claims());
    let _ = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/rebate/agreements/{}/activate", id))
            .header(&k, &v)
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();

    // Hold
    let resp = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/rebate/agreements/{}/hold", id))
            .header(&k, &v)
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(updated["status"], "on_hold");

    // Terminate
    let resp = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/rebate/agreements/{}/terminate", id))
            .header(&k, &v)
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(updated["status"], "terminated");
}

#[tokio::test]
async fn test_delete_agreement() {
    let (_state, app) = setup_rebate_test().await;
    create_test_agreement(&app, "DEL-RBA", "Delete Me", "supplier_rebate", "payable").await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/api/v1/rebate/agreements/number/DEL-RBA")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Tier Tests
// ============================================================================

#[tokio::test]
async fn test_create_tier() {
    let (_state, app) = setup_rebate_test().await;
    let agreement = create_test_agreement(&app, "TIER-AG", "Tier Agreement", "supplier_rebate", "payable").await;
    let agreement_id = agreement["id"].as_str().unwrap();

    let tier = create_test_tier(&app, agreement_id, 1, 0.0, Some(10000.0), 2.0).await;
    assert_eq!(tier["tierNumber"], 1);
    assert!((tier["fromValue"].as_f64().unwrap() - 0.0).abs() < 0.01);
    assert!((tier["toValue"].as_f64().unwrap() - 10000.0).abs() < 0.01);
    assert!((tier["rebateRate"].as_f64().unwrap() - 2.0).abs() < 0.01);
    assert_eq!(tier["rateType"], "percentage");
}

#[tokio::test]
async fn test_create_tier_invalid_rate_type() {
    let (_state, app) = setup_rebate_test().await;
    let agreement = create_test_agreement(&app, "TIER-INV", "Invalid Tier", "supplier_rebate", "payable").await;
    let agreement_id = agreement["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/rebate/agreements/{}/tiers", agreement_id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "tierNumber": 1,
                        "fromValue": 0,
                        "rebateRate": 5.0,
                        "rateType": "invalid_rate"
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
async fn test_list_tiers() {
    let (_state, app) = setup_rebate_test().await;
    let agreement = create_test_agreement(&app, "TIER-LIST", "List Tiers", "supplier_rebate", "payable").await;
    let agreement_id = agreement["id"].as_str().unwrap();

    create_test_tier(&app, agreement_id, 1, 0.0, Some(10000.0), 2.0).await;
    create_test_tier(&app, agreement_id, 2, 10000.0, Some(50000.0), 3.5).await;
    create_test_tier(&app, agreement_id, 3, 50000.0, None, 5.0).await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/rebate/agreements/{}/tiers", agreement_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let list: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(list["data"].as_array().unwrap().len(), 3);
}

#[tokio::test]
async fn test_delete_tier() {
    let (_state, app) = setup_rebate_test().await;
    let agreement = create_test_agreement(&app, "TIER-DEL", "Del Tier", "supplier_rebate", "payable").await;
    let agreement_id = agreement["id"].as_str().unwrap();
    let tier = create_test_tier(&app, agreement_id, 1, 0.0, Some(10000.0), 2.0).await;
    let tier_id = tier["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/rebate/tiers/{}", tier_id))
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Transaction Tests
// ============================================================================

async fn setup_active_agreement_with_tiers(app: &axum::Router) -> serde_json::Value {
    let agreement = create_test_agreement(app, "TXN-AG", "Transaction Agreement", "supplier_rebate", "payable").await;
    let agreement_id = agreement["id"].as_str().unwrap();

    // Activate
    let (k, v) = auth_header(&admin_claims());
    let _ = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/rebate/agreements/{}/activate", agreement_id))
            .header(&k, &v)
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();

    // Add tiers
    create_test_tier(app, agreement_id, 1, 0.0, Some(10000.0), 2.0).await;
    create_test_tier(app, agreement_id, 2, 10000.0, Some(50000.0), 3.5).await;
    create_test_tier(app, agreement_id, 3, 50000.0, None, 5.0).await;

    serde_json::from_str(&serde_json::to_string(&agreement).unwrap()).unwrap()
}

#[tokio::test]
async fn test_create_transaction() {
    let (_state, app) = setup_rebate_test().await;
    let agreement = setup_active_agreement_with_tiers(&app).await;
    let agreement_id = agreement["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/rebate/agreements/{}/transactions", agreement_id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "transactionNumber": "TXN-001",
                        "sourceType": "purchase_order",
                        "sourceNumber": "PO-12345",
                        "transactionDate": "2024-03-15",
                        "productName": "Widget A",
                        "quantity": 100,
                        "unitPrice": 50.0,
                        "transactionAmount": 5000.0,
                        "currencyCode": "USD"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let txn: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(txn["transactionNumber"], "TXN-001");
    assert_eq!(txn["status"], "eligible");
    // 5000 qualifies for tier 1 (0-10000): 2% -> 100.0
    assert!((txn["rebateAmount"].as_f64().unwrap() - 100.0).abs() < 0.01);
}

#[tokio::test]
async fn test_create_transaction_higher_tier() {
    let (_state, app) = setup_rebate_test().await;
    let agreement = setup_active_agreement_with_tiers(&app).await;
    let agreement_id = agreement["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/rebate/agreements/{}/transactions", agreement_id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "transactionNumber": "TXN-HIGH",
                        "sourceType": "invoice",
                        "transactionDate": "2024-06-01",
                        "transactionAmount": 25000.0,
                        "currencyCode": "USD"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let txn: serde_json::Value = serde_json::from_slice(&body).unwrap();
    // 25000 qualifies for tier 2 (10000-50000): 3.5% -> 875.0
    assert!((txn["rebateAmount"].as_f64().unwrap() - 875.0).abs() < 0.01);
}

#[tokio::test]
async fn test_create_transaction_duplicate_conflict() {
    let (_state, app) = setup_rebate_test().await;
    let agreement = setup_active_agreement_with_tiers(&app).await;
    let agreement_id = agreement["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let body = serde_json::to_string(&json!({
        "transactionNumber": "DUP-TXN",
        "transactionDate": "2024-03-15",
        "transactionAmount": 5000.0
    })).unwrap();

    // First should succeed
    let resp = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/rebate/agreements/{}/transactions", agreement_id))
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(body.clone()))
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    // Second should conflict
    let resp = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/rebate/agreements/{}/transactions", agreement_id))
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(body))
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_list_transactions() {
    let (_state, app) = setup_rebate_test().await;
    let agreement = setup_active_agreement_with_tiers(&app).await;
    let agreement_id = agreement["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    for num in ["TXN-L1", "TXN-L2"] {
        let _ = app.clone().oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/rebate/agreements/{}/transactions", agreement_id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "transactionNumber": num,
                        "transactionDate": "2024-03-15",
                        "transactionAmount": 3000.0
                    })).unwrap(),
                ))
                .unwrap(),
        ).await.unwrap();
    }

    let resp = app.clone().oneshot(
        Request::builder()
            .uri(format!("/api/v1/rebate/agreements/{}/transactions", agreement_id))
            .header(&k, &v)
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let list: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(list["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_update_transaction_status() {
    let (_state, app) = setup_rebate_test().await;
    let agreement = setup_active_agreement_with_tiers(&app).await;
    let agreement_id = agreement["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/rebate/agreements/{}/transactions", agreement_id))
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(
                serde_json::to_string(&json!({
                    "transactionNumber": "STATUS-TXN",
                    "transactionDate": "2024-03-15",
                    "transactionAmount": 5000.0
                })).unwrap(),
            ))
            .unwrap(),
    ).await.unwrap();
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let txn: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let txn_id = txn["id"].as_str().unwrap();

    // Exclude the transaction
    let resp = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/rebate/transactions/{}/status", txn_id))
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(
                serde_json::to_string(&json!({"status": "excluded", "reason": "Duplicate invoice"})).unwrap(),
            ))
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(updated["status"], "excluded");
}

// ============================================================================
// Accrual Tests
// ============================================================================

#[tokio::test]
async fn test_create_accrual() {
    let (_state, app) = setup_rebate_test().await;
    let agreement = setup_active_agreement_with_tiers(&app).await;
    let agreement_id = agreement["id"].as_str().unwrap();

    // Create a transaction first
    let (k, v) = auth_header(&admin_claims());
    let _ = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/rebate/agreements/{}/transactions", agreement_id))
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(
                serde_json::to_string(&json!({
                    "transactionNumber": "ACC-TXN",
                    "transactionDate": "2024-03-15",
                    "transactionAmount": 7500.0
                })).unwrap(),
            ))
            .unwrap(),
    ).await.unwrap();

    // Create accrual
    let resp = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/rebate/agreements/{}/accruals", agreement_id))
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(
                serde_json::to_string(&json!({
                    "accrualNumber": "ACC-001",
                    "accrualDate": "2024-03-31",
                    "accrualPeriod": "2024-Q1",
                    "notes": "Q1 accrual"
                })).unwrap(),
            ))
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let accrual: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(accrual["accrualNumber"], "ACC-001");
    assert_eq!(accrual["status"], "draft");
    // 7500 in tier 1 (0-10000): 2% = 150.0
    assert!((accrual["accruedAmount"].as_f64().unwrap() - 150.0).abs() < 0.01);
}

#[tokio::test]
async fn test_post_and_reverse_accrual() {
    let (_state, app) = setup_rebate_test().await;
    let agreement = setup_active_agreement_with_tiers(&app).await;
    let agreement_id = agreement["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    // Create transaction + accrual
    let _ = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/rebate/agreements/{}/transactions", agreement_id))
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(
                serde_json::to_string(&json!({
                    "transactionNumber": "REV-TXN",
                    "transactionDate": "2024-03-15",
                    "transactionAmount": 5000.0
                })).unwrap(),
            ))
            .unwrap(),
    ).await.unwrap();

    let resp = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/rebate/agreements/{}/accruals", agreement_id))
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(
                serde_json::to_string(&json!({
                    "accrualNumber": "REV-ACC",
                    "accrualDate": "2024-03-31"
                })).unwrap(),
            ))
            .unwrap(),
    ).await.unwrap();
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let accrual: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let accrual_id = accrual["id"].as_str().unwrap();

    // Post it
    let resp = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/rebate/accruals/{}/post", accrual_id))
            .header(&k, &v)
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let posted: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(posted["status"], "posted");

    // Reverse it
    let resp = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/rebate/accruals/{}/reverse", accrual_id))
            .header(&k, &v)
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let reversed: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(reversed["status"], "reversed");
}

// ============================================================================
// Settlement Tests
// ============================================================================

#[tokio::test]
async fn test_settlement_lifecycle() {
    let (_state, app) = setup_rebate_test().await;
    let agreement = setup_active_agreement_with_tiers(&app).await;
    let agreement_id = agreement["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Create transactions
    for (num, amt) in [("SET-TXN1", 3000.0), ("SET-TXN2", 4000.0)] {
        let _ = app.clone().oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/rebate/agreements/{}/transactions", agreement_id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "transactionNumber": num,
                        "transactionDate": "2024-03-15",
                        "transactionAmount": amt
                    })).unwrap(),
                ))
                .unwrap(),
        ).await.unwrap();
    }

    // Create accrual to mark them as accrued
    let _ = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/rebate/agreements/{}/accruals", agreement_id))
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(
                serde_json::to_string(&json!({
                    "accrualNumber": "SET-ACC",
                    "accrualDate": "2024-03-31"
                })).unwrap(),
            ))
            .unwrap(),
    ).await.unwrap();

    // Create settlement
    let resp = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/rebate/agreements/{}/settlements", agreement_id))
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(
                serde_json::to_string(&json!({
                    "settlementNumber": "SET-001",
                    "settlementDate": "2024-04-15",
                    "settlementPeriodFrom": "2024-01-01",
                    "settlementPeriodTo": "2024-03-31",
                    "settlementType": "payment",
                    "paymentMethod": "ach",
                    "notes": "Q1 settlement"
                })).unwrap(),
            ))
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let settlement: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(settlement["settlementNumber"], "SET-001");
    assert_eq!(settlement["status"], "pending");
    assert_eq!(settlement["settlementType"], "payment");
    // Total qualifying: 3000 + 4000 = 7000 in tier 1: 2% = 140.0
    assert!((settlement["settlementAmount"].as_f64().unwrap() - 140.0).abs() < 0.01);
    let settlement_id = settlement["id"].as_str().unwrap();

    // Approve
    let resp = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/rebate/settlements/{}/approve", settlement_id))
            .header(&k, &v)
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let approved: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(approved["status"], "approved");

    // Pay
    let resp = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/rebate/settlements/{}/pay", settlement_id))
            .header(&k, &v)
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let paid: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(paid["status"], "paid");

    // Check settlement lines
    let resp = app.clone().oneshot(
        Request::builder()
            .uri(format!("/api/v1/rebate/settlements/{}/lines", settlement_id))
            .header(&k, &v)
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let lines: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(lines["data"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_cancel_settlement() {
    let (_state, app) = setup_rebate_test().await;
    let agreement = setup_active_agreement_with_tiers(&app).await;
    let agreement_id = agreement["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Create transaction + accrual + settlement
    let _ = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/rebate/agreements/{}/transactions", agreement_id))
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(
                serde_json::to_string(&json!({
                    "transactionNumber": "CNL-TXN",
                    "transactionDate": "2024-03-15",
                    "transactionAmount": 5000.0
                })).unwrap(),
            ))
            .unwrap(),
    ).await.unwrap();

    let _ = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/rebate/agreements/{}/accruals", agreement_id))
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(
                serde_json::to_string(&json!({"accrualNumber": "CNL-ACC", "accrualDate": "2024-03-31"})).unwrap(),
            ))
            .unwrap(),
    ).await.unwrap();

    let resp = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/rebate/agreements/{}/settlements", agreement_id))
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(
                serde_json::to_string(&json!({
                    "settlementNumber": "CNL-SET",
                    "settlementDate": "2024-04-15"
                })).unwrap(),
            ))
            .unwrap(),
    ).await.unwrap();
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let settlement: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let settlement_id = settlement["id"].as_str().unwrap();

    // Cancel
    let resp = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/rebate/settlements/{}/cancel", settlement_id))
            .header(&k, &v)
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let cancelled: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(cancelled["status"], "cancelled");
}

// ============================================================================
// Dashboard Test
// ============================================================================

#[tokio::test]
async fn test_rebate_dashboard() {
    let (_state, app) = setup_rebate_test().await;
    create_test_agreement(&app, "DASH-1", "Dashboard Agreement 1", "supplier_rebate", "payable").await;
    create_test_agreement(&app, "DASH-2", "Dashboard Agreement 2", "customer_rebate", "receivable").await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/rebate/dashboard")
                .header(&k, &v)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let dashboard: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(dashboard["totalAgreements"].as_i64().unwrap() >= 2);
}

// ============================================================================
// Full End-to-End Lifecycle Test
// ============================================================================

#[tokio::test]
async fn test_rebate_full_lifecycle() {
    let (_state, app) = setup_rebate_test().await;

    let (k, v) = auth_header(&admin_claims());

    // 1. Create a supplier rebate agreement
    let agreement = create_test_agreement(&app, "LIFE-AG", "Full Lifecycle Agreement", "supplier_rebate", "payable").await;
    let agreement_id = agreement["id"].as_str().unwrap();
    assert_eq!(agreement["status"], "draft");

    // 2. Add tiered pricing
    create_test_tier(&app, agreement_id, 1, 0.0, Some(10000.0), 2.0).await;
    create_test_tier(&app, agreement_id, 2, 10000.0, Some(50000.0), 3.5).await;
    create_test_tier(&app, agreement_id, 3, 50000.0, None, 5.0).await;

    // 3. Activate the agreement
    let resp = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/rebate/agreements/{}/activate", agreement_id))
            .header(&k, &v)
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 4. Record qualifying transactions
    for (num, amt) in [("LIFE-TXN1", 6000.0), ("LIFE-TXN2", 8000.0)] {
        let resp = app.clone().oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/rebate/agreements/{}/transactions", agreement_id))
                .header("Content-Type", "application/json")
                .header(&k, &v)
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "transactionNumber": num,
                        "transactionDate": "2024-02-15",
                        "transactionAmount": amt
                    })).unwrap(),
                ))
                .unwrap(),
        ).await.unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);
    }

    // 5. Create an accrual
    let resp = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/rebate/agreements/{}/accruals", agreement_id))
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(
                serde_json::to_string(&json!({
                    "accrualNumber": "LIFE-ACC",
                    "accrualDate": "2024-03-31",
                    "accrualPeriod": "2024-Q1"
                })).unwrap(),
            ))
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let accrual: serde_json::Value = serde_json::from_slice(&body).unwrap();
    // Total: 6000 + 8000 = 14000 in tier 2: 3.5% = 490.0
    assert!((accrual["accruedAmount"].as_f64().unwrap() - 490.0).abs() < 0.01);
    let accrual_id = accrual["id"].as_str().unwrap();

    // 6. Post the accrual
    let resp = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/rebate/accruals/{}/post", accrual_id))
            .header(&k, &v)
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 7. Create settlement
    let resp = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/rebate/agreements/{}/settlements", agreement_id))
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(
                serde_json::to_string(&json!({
                    "settlementNumber": "LIFE-SET",
                    "settlementDate": "2024-04-10",
                    "settlementType": "payment",
                    "paymentMethod": "wire"
                })).unwrap(),
            ))
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let settlement: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let settlement_id = settlement["id"].as_str().unwrap();
    assert!((settlement["settlementAmount"].as_f64().unwrap() - 490.0).abs() < 0.01);

    // 8. Approve settlement
    let resp = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/rebate/settlements/{}/approve", settlement_id))
            .header(&k, &v)
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 9. Pay settlement
    let resp = app.clone().oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/rebate/settlements/{}/pay", settlement_id))
            .header(&k, &v)
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let paid: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(paid["status"], "paid");

    // 10. Verify dashboard shows the data
    let resp = app.clone().oneshot(
        Request::builder()
            .uri("/api/v1/rebate/dashboard")
            .header(&k, &v)
            .body(Body::empty())
            .unwrap(),
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let dashboard: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(dashboard["totalAgreements"].as_i64().unwrap() >= 1);
}
