//! Bank Account Transfer E2E Tests
//!
//! Tests for Oracle Fusion Cloud ERP Bank Account Transfers:
//! - Transfer type creation
//! - Transfer creation and lifecycle
//! - Dashboard summary

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_bank_transfer_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    let app = build_router(state.clone());
    (state, app)
}

const FROM_ACCOUNT: &str = "00000000-0000-0000-0000-000000000401";
const TO_ACCOUNT: &str = "00000000-0000-0000-0000-000000000402";

#[tokio::test]
#[ignore]
async fn test_create_bank_transfer_type() {
    let (_state, app) = setup_bank_transfer_test().await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/bank-transfers/types")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "INTERNAL",
            "name": "Internal Transfer",
            "settlement_method": "immediate",
            "requires_approval": true,
            "approval_threshold": "100000",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
}

#[tokio::test]
#[ignore]
async fn test_create_bank_transfer() {
    let (_state, app) = setup_bank_transfer_test().await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/bank-transfers")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "from_bank_account_id": FROM_ACCOUNT,
            "from_bank_account_number": "CHK-001",
            "from_bank_name": "Main Bank",
            "to_bank_account_id": TO_ACCOUNT,
            "to_bank_account_number": "SAV-001",
            "to_bank_name": "Main Bank",
            "amount": "50000",
            "currency_code": "USD",
            "transfer_date": "2024-12-15",
            "description": "Monthly fund transfer",
            "priority": "normal",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let transfer: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(transfer["status"], "draft");
    assert_eq!(transfer["currency_code"], "USD");
}

#[tokio::test]
#[ignore]
async fn test_bank_transfer_same_account_rejected() {
    let (_state, app) = setup_bank_transfer_test().await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/bank-transfers")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "from_bank_account_id": FROM_ACCOUNT,
            "to_bank_account_id": FROM_ACCOUNT,
            "amount": "1000",
            "currency_code": "USD",
            "transfer_date": "2024-12-15",
            "priority": "normal",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore]
async fn test_list_bank_transfers() {
    let (_state, app) = setup_bank_transfer_test().await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET").uri("/api/v1/bank-transfers")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(!result["data"].as_array().unwrap().is_empty());
}

#[tokio::test]
#[ignore]
async fn test_bank_transfer_dashboard() {
    let (_state, app) = setup_bank_transfer_test().await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/bank-transfers/dashboard")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}
