//! Cash Position E2E Tests

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    let migration_sql = include_str!("../../../../migrations/110_advance_payment_customer_deposit_cash_position.sql");
    sqlx::raw_sql(migration_sql).execute(&state.db_pool).await.ok();
    let app = build_router(state.clone());
    (state, app)
}

#[tokio::test]
async fn test_record_cash_position() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/cash-positions")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "bank_account_id": uuid::Uuid::new_v4().to_string(),
            "bank_account_number": "ACC-001",
            "bank_account_name": "Operating Account",
            "currency_code": "USD",
            "opening_balance": "100000.00",
            "total_inflows": "50000.00",
            "total_outflows": "30000.00",
            "closing_balance": "120000.00",
            "ledger_balance": "120000.00",
            "available_balance": "120000.00",
            "hold_amount": "0.00",
            "position_date": "2026-04-01",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let p: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(p["currency_code"], "USD");
    assert_eq!(p["closing_balance"], "120000.00");
}

#[tokio::test]
async fn test_record_position_closing_mismatch() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/cash-positions")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "bank_account_id": uuid::Uuid::new_v4().to_string(),
            "currency_code": "USD",
            "opening_balance": "100000.00",
            "total_inflows": "50000.00",
            "total_outflows": "30000.00",
            "closing_balance": "99999.00",
            "ledger_balance": "99999.00",
            "available_balance": "99999.00",
            "hold_amount": "0.00",
            "position_date": "2026-04-01",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_list_positions() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/cash-positions")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_dashboard() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/cash-positions/dashboard")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}
