//! Advance Payment E2E Tests

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
async fn test_create_advance_payment() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/advance-payments")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "advance_number": "ADV-E2E-001",
            "supplier_id": uuid::Uuid::new_v4().to_string(),
            "supplier_name": "Global Supplies Inc",
            "currency_code": "USD",
            "advance_amount": "50000.00",
            "payment_method": "electronic",
            "advance_date": "2026-02-01",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let a: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(a["advance_number"], "ADV-E2E-001");
    assert_eq!(a["status"], "draft");
    assert_eq!(a["advance_amount"], "50000.00");
}

#[tokio::test]
async fn test_create_advance_zero_amount() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/advance-payments")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "advance_number": "ADV-ZERO",
            "supplier_id": uuid::Uuid::new_v4().to_string(),
            "supplier_name": "Supplier",
            "currency_code": "USD",
            "advance_amount": "0.00",
            "advance_date": "2026-01-01",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_advance_empty_number() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/advance-payments")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "advance_number": "",
            "supplier_id": uuid::Uuid::new_v4().to_string(),
            "supplier_name": "Supplier",
            "currency_code": "USD",
            "advance_amount": "1000.00",
            "advance_date": "2026-01-01",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_list_advances() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/advance-payments")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_advance_not_found() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/advance-payments/{}", uuid::Uuid::new_v4()))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_dashboard() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/advance-payments/dashboard")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}
