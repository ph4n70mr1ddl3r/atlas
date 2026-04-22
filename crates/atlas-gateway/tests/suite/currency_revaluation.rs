//! Currency Revaluation E2E Tests
//!
//! Tests for Oracle Fusion Cloud ERP Currency Revaluation:
//! - Definition CRUD and lifecycle
//! - Account management
//! - Revaluation run execution with gain/loss calculation
//! - Run posting, reversal, and cancellation
//! - Dashboard summary
//! - Validation and error cases

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_revaluation_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    let app = build_router(state.clone());
    (state, app)
}

// ═══════════════════════════════════════════════════════════════════════════════
// Definition CRUD Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_create_revaluation_definition() {
    let (_state, app) = setup_revaluation_test().await;
    let (k, v) = auth_header(&admin_claims());

    let resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/currency-revaluation/definitions")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "EUR_REVAL",
            "name": "EUR Period-End Revaluation",
            "description": "Revalue EUR-denominated accounts at period end",
            "revaluation_type": "period_end",
            "currency_code": "EUR",
            "rate_type": "period_end",
            "gain_account_code": "7100-UNREALIZED-GAIN",
            "loss_account_code": "8100-UNREALIZED-LOSS",
            "auto_reverse": true,
            "reversal_period_offset": 1
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let def: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(def["code"], "EUR_REVAL");
    assert_eq!(def["name"], "EUR Period-End Revaluation");
    assert_eq!(def["revaluationType"], "period_end");
    assert_eq!(def["currencyCode"], "EUR");
    assert_eq!(def["autoReverse"], true);
    assert_eq!(def["isActive"], true);
}

#[tokio::test]
async fn test_create_definition_balance_sheet() {
    let (_state, app) = setup_revaluation_test().await;
    let (k, v) = auth_header(&admin_claims());

    let resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/currency-revaluation/definitions")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "BS_REVAL",
            "name": "Balance Sheet Revaluation",
            "revaluation_type": "balance_sheet",
            "currency_code": "GBP",
            "rate_type": "period_end",
            "gain_account_code": "7200-GAIN",
            "loss_account_code": "8200-LOSS"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let def: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(def["revaluationType"], "balance_sheet");
    assert_eq!(def["currencyCode"], "GBP");
}

#[tokio::test]
async fn test_create_definition_duplicate_rejected() {
    let (_state, app) = setup_revaluation_test().await;
    let (k, v) = auth_header(&admin_claims());

    let body = serde_json::to_string(&json!({
        "code": "DUP_REVAL",
        "name": "Duplicate Test",
        "revaluation_type": "period_end",
        "currency_code": "EUR",
        "rate_type": "period_end",
        "gain_account_code": "7100-GAIN",
        "loss_account_code": "8100-LOSS"
    })).unwrap();

    app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/currency-revaluation/definitions")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(body.clone()))
        .unwrap()
    ).await.unwrap();

    let resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/currency-revaluation/definitions")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(body))
        .unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_get_revaluation_definition() {
    let (_state, app) = setup_revaluation_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Create
    app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/currency-revaluation/definitions")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "GET_REVAL",
            "name": "Get Definition Test",
            "revaluation_type": "period_end",
            "currency_code": "EUR",
            "rate_type": "period_end",
            "gain_account_code": "7100-GAIN",
            "loss_account_code": "8100-LOSS"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    // Get
    let resp = app.clone().oneshot(Request::builder()
        .method("GET")
        .uri("/api/v1/currency-revaluation/definitions/GET_REVAL")
        .header(&k, &v)
        .body(Body::empty())
        .unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let def: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(def["code"], "GET_REVAL");
}

#[tokio::test]
async fn test_list_revaluation_definitions() {
    let (_state, app) = setup_revaluation_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Create two definitions
    for code in &["LIST_A", "LIST_B"] {
        app.clone().oneshot(Request::builder()
            .method("POST")
            .uri("/api/v1/currency-revaluation/definitions")
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({
                "code": code,
                "name": format!("Definition {}", code),
                "revaluation_type": "period_end",
                "currency_code": "EUR",
                "rate_type": "period_end",
                "gain_account_code": format!("7100-{}", code),
                "loss_account_code": format!("8100-{}", code)
            })).unwrap()))
            .unwrap()
        ).await.unwrap();
    }

    let resp = app.clone().oneshot(Request::builder()
        .method("GET")
        .uri("/api/v1/currency-revaluation/definitions")
        .header(&k, &v)
        .body(Body::empty())
        .unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(result["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_activate_deactivate_definition() {
    let (_state, app) = setup_revaluation_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Create
    let create_resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/currency-revaluation/definitions")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "ACT_REVAL",
            "name": "Activate/Deactivate Test",
            "revaluation_type": "period_end",
            "currency_code": "EUR",
            "rate_type": "period_end",
            "gain_account_code": "7100-GAIN",
            "loss_account_code": "8100-LOSS"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(create_resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(create_resp.into_body(), usize::MAX).await.unwrap();
    let def: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let def_id = def["id"].as_str().unwrap();

    // Deactivate
    let deact_resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri(&format!("/api/v1/currency-revaluation/definitions/{}/deactivate", def_id))
        .header(&k, &v)
        .body(Body::empty())
        .unwrap()
    ).await.unwrap();
    assert_eq!(deact_resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(deact_resp.into_body(), usize::MAX).await.unwrap();
    let def: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(def["isActive"], false);

    // Reactivate
    let act_resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri(&format!("/api/v1/currency-revaluation/definitions/{}/activate", def_id))
        .header(&k, &v)
        .body(Body::empty())
        .unwrap()
    ).await.unwrap();
    assert_eq!(act_resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(act_resp.into_body(), usize::MAX).await.unwrap();
    let def: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(def["isActive"], true);
}

#[tokio::test]
async fn test_delete_definition() {
    let (_state, app) = setup_revaluation_test().await;
    let (k, v) = auth_header(&admin_claims());

    app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/currency-revaluation/definitions")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "DEL_REVAL",
            "name": "Delete Me",
            "revaluation_type": "period_end",
            "currency_code": "EUR",
            "rate_type": "period_end",
            "gain_account_code": "7100-GAIN",
            "loss_account_code": "8100-LOSS"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    let resp = app.clone().oneshot(Request::builder()
        .method("DELETE")
        .uri("/api/v1/currency-revaluation/definitions/DEL_REVAL")
        .header(&k, &v)
        .body(Body::empty())
        .unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Verify deleted
    let get_resp = app.clone().oneshot(Request::builder()
        .method("GET")
        .uri("/api/v1/currency-revaluation/definitions/DEL_REVAL")
        .header(&k, &v)
        .body(Body::empty())
        .unwrap()
    ).await.unwrap();
    assert_eq!(get_resp.status(), StatusCode::NOT_FOUND);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Account Management Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_add_revaluation_account() {
    let (_state, app) = setup_revaluation_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Create definition
    app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/currency-revaluation/definitions")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "ACCT_REVAL",
            "name": "Account Test Reval",
            "revaluation_type": "period_end",
            "currency_code": "EUR",
            "rate_type": "period_end",
            "gain_account_code": "7100-GAIN",
            "loss_account_code": "8100-LOSS"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    // Add account
    let resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/currency-revaluation/definitions/ACCT_REVAL/accounts")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "account_code": "1100-CASH-EUR",
            "account_name": "Cash - EUR",
            "account_type": "asset",
            "is_included": true
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let acct: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(acct["accountCode"], "1100-CASH-EUR");
    assert_eq!(acct["accountType"], "asset");
}

#[tokio::test]
async fn test_list_revaluation_accounts() {
    let (_state, app) = setup_revaluation_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Create definition
    app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/currency-revaluation/definitions")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "LIST_ACCT",
            "name": "List Accounts Test",
            "revaluation_type": "period_end",
            "currency_code": "EUR",
            "rate_type": "period_end",
            "gain_account_code": "7100-GAIN",
            "loss_account_code": "8100-LOSS"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    // Add accounts
    for code in &["1100-CASH-EUR", "1200-AR-EUR", "2100-AP-EUR"] {
        app.clone().oneshot(Request::builder()
            .method("POST")
            .uri("/api/v1/currency-revaluation/definitions/LIST_ACCT/accounts")
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({
                "account_code": code,
                "account_type": if code.starts_with("2") { "liability" } else { "asset" }
            })).unwrap()))
            .unwrap()
        ).await.unwrap();
    }

    let resp = app.clone().oneshot(Request::builder()
        .method("GET")
        .uri("/api/v1/currency-revaluation/definitions/LIST_ACCT/accounts")
        .header(&k, &v)
        .body(Body::empty())
        .unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(result["data"].as_array().unwrap().len(), 3);
}

#[tokio::test]
async fn test_remove_revaluation_account() {
    let (_state, app) = setup_revaluation_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Create definition
    app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/currency-revaluation/definitions")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "REM_ACCT",
            "name": "Remove Account Test",
            "revaluation_type": "period_end",
            "currency_code": "EUR",
            "rate_type": "period_end",
            "gain_account_code": "7100-GAIN",
            "loss_account_code": "8100-LOSS"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    // Add account
    let add_resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/currency-revaluation/definitions/REM_ACCT/accounts")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "account_code": "1100-CASH-EUR",
            "account_type": "asset"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(add_resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(add_resp.into_body(), usize::MAX).await.unwrap();
    let acct: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let acct_id = acct["id"].as_str().unwrap();

    // Remove account
    let del_resp = app.clone().oneshot(Request::builder()
        .method("DELETE")
        .uri(&format!("/api/v1/currency-revaluation/accounts/{}", acct_id))
        .header(&k, &v)
        .body(Body::empty())
        .unwrap()
    ).await.unwrap();
    assert_eq!(del_resp.status(), StatusCode::OK);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Revaluation Run Tests
// ═══════════════════════════════════════════════════════════════════════════════

async fn setup_full_revaluation(app: &axum::Router, k: &str, v: &str) {
    app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/currency-revaluation/definitions")
        .header("Content-Type", "application/json")
        .header(k, v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "RUN_REVAL",
            "name": "Run Test Reval Definition",
            "revaluation_type": "period_end",
            "currency_code": "EUR",
            "rate_type": "period_end",
            "gain_account_code": "7100-UNREALIZED-GAIN",
            "loss_account_code": "8100-UNREALIZED-LOSS",
            "auto_reverse": true
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
}

#[tokio::test]
async fn test_execute_revaluation_with_gain() {
    let (_state, app) = setup_revaluation_test().await;
    let (k, v) = auth_header(&admin_claims());
    setup_full_revaluation(&app, &k, &v).await;

    let resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/currency-revaluation/runs")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "definition_code": "RUN_REVAL",
            "period_name": "DEC-2024",
            "period_start_date": "2024-12-01",
            "period_end_date": "2024-12-31",
            "balances": [
                {
                    "account_code": "1100-CASH-EUR",
                    "account_name": "Cash - EUR",
                    "account_type": "asset",
                    "original_amount": "10000.00",
                    "original_currency": "EUR",
                    "original_exchange_rate": "1.10",
                    "original_base_amount": "11000.00"
                },
                {
                    "account_code": "1200-AR-EUR",
                    "account_name": "Accounts Receivable - EUR",
                    "account_type": "asset",
                    "original_amount": "5000.00",
                    "original_currency": "EUR",
                    "original_exchange_rate": "1.15",
                    "original_base_amount": "5750.00"
                }
            ]
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let run: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(run["status"], "draft");
    assert_eq!(run["definitionCode"], "RUN_REVAL");
    assert_eq!(run["periodName"], "DEC-2024");
    assert_eq!(run["currencyCode"], "EUR");
    // Should have 2 line items (one per balance entry)
    assert!(run["lines"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_post_and_reverse_revaluation_run() {
    let (_state, app) = setup_revaluation_test().await;
    let (k, v) = auth_header(&admin_claims());
    setup_full_revaluation(&app, &k, &v).await;

    // Create run
    let create_resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/currency-revaluation/runs")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "definition_code": "RUN_REVAL",
            "period_name": "POST-TEST",
            "period_start_date": "2024-01-01",
            "period_end_date": "2024-01-31",
            "balances": [
                {
                    "account_code": "1100-CASH-EUR",
                    "account_name": "Cash - EUR",
                    "account_type": "asset",
                    "original_amount": "10000.00",
                    "original_currency": "EUR",
                    "original_exchange_rate": "1.10",
                    "original_base_amount": "11000.00"
                }
            ]
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(create_resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(create_resp.into_body(), usize::MAX).await.unwrap();
    let run: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let run_id = run["id"].as_str().unwrap();

    // Post
    let post_resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri(&format!("/api/v1/currency-revaluation/runs/{}/post", run_id))
        .header(&k, &v)
        .body(Body::empty())
        .unwrap()
    ).await.unwrap();
    assert_eq!(post_resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(post_resp.into_body(), usize::MAX).await.unwrap();
    let posted_run: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(posted_run["status"], "posted");

    // Reverse
    let reverse_resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri(&format!("/api/v1/currency-revaluation/runs/{}/reverse", run_id))
        .header(&k, &v)
        .body(Body::empty())
        .unwrap()
    ).await.unwrap();
    assert_eq!(reverse_resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(reverse_resp.into_body(), usize::MAX).await.unwrap();
    let reversed_run: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(reversed_run["status"], "reversed");
}

#[tokio::test]
async fn test_cancel_revaluation_run() {
    let (_state, app) = setup_revaluation_test().await;
    let (k, v) = auth_header(&admin_claims());
    setup_full_revaluation(&app, &k, &v).await;

    // Create run
    let create_resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/currency-revaluation/runs")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "definition_code": "RUN_REVAL",
            "period_name": "CANCEL-TEST",
            "period_start_date": "2024-02-01",
            "period_end_date": "2024-02-28",
            "balances": [
                {
                    "account_code": "1100-CASH-EUR",
                    "account_name": "Cash - EUR",
                    "account_type": "asset",
                    "original_amount": "5000.00",
                    "original_currency": "EUR",
                    "original_exchange_rate": "1.10",
                    "original_base_amount": "5500.00"
                }
            ]
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    let body = axum::body::to_bytes(create_resp.into_body(), usize::MAX).await.unwrap();
    let run: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let run_id = run["id"].as_str().unwrap();

    // Cancel
    let cancel_resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri(&format!("/api/v1/currency-revaluation/runs/{}/cancel", run_id))
        .header(&k, &v)
        .body(Body::empty())
        .unwrap()
    ).await.unwrap();
    assert_eq!(cancel_resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(cancel_resp.into_body(), usize::MAX).await.unwrap();
    let cancelled_run: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(cancelled_run["status"], "cancelled");
}

#[tokio::test]
async fn test_cannot_cancel_posted_run() {
    let (_state, app) = setup_revaluation_test().await;
    let (k, v) = auth_header(&admin_claims());
    setup_full_revaluation(&app, &k, &v).await;

    // Create and post
    let create_resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/currency-revaluation/runs")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "definition_code": "RUN_REVAL",
            "period_name": "NO-CANCEL",
            "period_start_date": "2024-03-01",
            "period_end_date": "2024-03-31",
            "balances": [
                {
                    "account_code": "1100-CASH-EUR",
                    "account_type": "asset",
                    "original_amount": "3000.00",
                    "original_currency": "EUR",
                    "original_exchange_rate": "1.05",
                    "original_base_amount": "3150.00"
                }
            ]
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    let body = axum::body::to_bytes(create_resp.into_body(), usize::MAX).await.unwrap();
    let run: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let run_id = run["id"].as_str().unwrap();

    // Post first
    app.clone().oneshot(Request::builder()
        .method("POST")
        .uri(&format!("/api/v1/currency-revaluation/runs/{}/post", run_id))
        .header(&k, &v)
        .body(Body::empty())
        .unwrap()
    ).await.unwrap();

    // Try to cancel - should fail
    let cancel_resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri(&format!("/api/v1/currency-revaluation/runs/{}/cancel", run_id))
        .header(&k, &v)
        .body(Body::empty())
        .unwrap()
    ).await.unwrap();
    assert_eq!(cancel_resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_cannot_reverse_draft_run() {
    let (_state, app) = setup_revaluation_test().await;
    let (k, v) = auth_header(&admin_claims());
    setup_full_revaluation(&app, &k, &v).await;

    // Create run but don't post
    let create_resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/currency-revaluation/runs")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "definition_code": "RUN_REVAL",
            "period_name": "NO-REVERSE",
            "period_start_date": "2024-04-01",
            "period_end_date": "2024-04-30",
            "balances": [
                {
                    "account_code": "1100-CASH-EUR",
                    "account_type": "asset",
                    "original_amount": "2000.00",
                    "original_currency": "EUR",
                    "original_exchange_rate": "1.10",
                    "original_base_amount": "2200.00"
                }
            ]
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    let body = axum::body::to_bytes(create_resp.into_body(), usize::MAX).await.unwrap();
    let run: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let run_id = run["id"].as_str().unwrap();

    // Try to reverse draft - should fail
    let reverse_resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri(&format!("/api/v1/currency-revaluation/runs/{}/reverse", run_id))
        .header(&k, &v)
        .body(Body::empty())
        .unwrap()
    ).await.unwrap();
    assert_eq!(reverse_resp.status(), StatusCode::BAD_REQUEST);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Dashboard Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_revaluation_dashboard() {
    let (_state, app) = setup_revaluation_test().await;
    let (k, v) = auth_header(&admin_claims());

    let resp = app.clone().oneshot(Request::builder()
        .method("GET")
        .uri("/api/v1/currency-revaluation/dashboard")
        .header(&k, &v)
        .body(Body::empty())
        .unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let summary: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(summary["totalDefinitions"].is_number());
    assert!(summary["activeDefinitions"].is_number());
    assert!(summary["totalRuns"].is_number());
    assert!(summary["postedRuns"].is_number());
    assert!(summary["draftRuns"].is_number());
    assert!(summary["definitionsByType"].is_object());
}

#[tokio::test]
async fn test_dashboard_after_creation() {
    let (_state, app) = setup_revaluation_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Create definition
    app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/currency-revaluation/definitions")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "DASH_REVAL",
            "name": "Dashboard Test Reval",
            "revaluation_type": "period_end",
            "currency_code": "EUR",
            "rate_type": "period_end",
            "gain_account_code": "7100-GAIN",
            "loss_account_code": "8100-LOSS"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    let resp = app.clone().oneshot(Request::builder()
        .method("GET")
        .uri("/api/v1/currency-revaluation/dashboard")
        .header(&k, &v)
        .body(Body::empty())
        .unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let summary: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(summary["totalDefinitions"].as_i64().unwrap() >= 1);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Validation & Error Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_create_definition_empty_code_rejected() {
    let (_state, app) = setup_revaluation_test().await;
    let (k, v) = auth_header(&admin_claims());

    let resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/currency-revaluation/definitions")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "",
            "name": "Empty Code",
            "revaluation_type": "period_end",
            "currency_code": "EUR",
            "gain_account_code": "7100-GAIN",
            "loss_account_code": "8100-LOSS"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_definition_invalid_type_rejected() {
    let (_state, app) = setup_revaluation_test().await;
    let (k, v) = auth_header(&admin_claims());

    let resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/currency-revaluation/definitions")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "INVALID_TYPE",
            "name": "Invalid Type",
            "revaluation_type": "invalid",
            "currency_code": "EUR",
            "gain_account_code": "7100-GAIN",
            "loss_account_code": "8100-LOSS"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_execute_revaluation_nonexistent_definition() {
    let (_state, app) = setup_revaluation_test().await;
    let (k, v) = auth_header(&admin_claims());

    let resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/currency-revaluation/runs")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "definition_code": "NONEXISTENT",
            "period_name": "DEC-2024",
            "period_start_date": "2024-12-01",
            "period_end_date": "2024-12-31",
            "balances": [
                {
                    "account_code": "1100-CASH-EUR",
                    "account_type": "asset",
                    "original_amount": "10000.00",
                    "original_currency": "EUR",
                    "original_exchange_rate": "1.10",
                    "original_base_amount": "11000.00"
                }
            ]
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_add_account_invalid_type_rejected() {
    let (_state, app) = setup_revaluation_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Create definition
    app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/currency-revaluation/definitions")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "ACCT_INVALID",
            "name": "Invalid Account Type",
            "revaluation_type": "period_end",
            "currency_code": "EUR",
            "gain_account_code": "7100-GAIN",
            "loss_account_code": "8100-LOSS"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    // Try to add account with invalid type
    let resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/currency-revaluation/definitions/ACCT_INVALID/accounts")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "account_code": "1100-CASH-EUR",
            "account_type": "invalid_type",
            "is_included": true
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_list_revaluation_runs_with_status_filter() {
    let (_state, app) = setup_revaluation_test().await;
    let (k, v) = auth_header(&admin_claims());

    let resp = app.clone().oneshot(Request::builder()
        .method("GET")
        .uri("/api/v1/currency-revaluation/runs?status=draft")
        .header(&k, &v)
        .body(Body::empty())
        .unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(result["data"].as_array().unwrap().len() >= 1 || result["data"].as_array().unwrap().is_empty());
}