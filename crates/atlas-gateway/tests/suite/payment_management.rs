//! Payment Management E2E Tests
//!
//! Tests for Oracle Fusion Cloud ERP Payments:
//! - Payment CRUD
//! - Payment lifecycle (create → issue → clear)
//! - Payment voiding
//! - Validation and error cases

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_payment_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    let app = build_router(state.clone());
    (state, app)
}

const SUPPLIER_ID: &str = "00000000-0000-0000-0000-000000000100";

async fn create_test_payment(
    app: &axum::Router,
    amount: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/payments")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "supplier_id": SUPPLIER_ID,
            "supplier_name": "Acme Corp",
            "payment_date": "2026-04-20",
            "payment_method": "check",
            "currency_code": "USD",
            "payment_amount": amount,
            "discount_taken": "0.00",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Failed to create payment");
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

#[tokio::test]
#[ignore]
async fn test_create_payment() {
    let (_state, app) = setup_payment_test().await;

    let payment = create_test_payment(&app, "2500.00").await;

    assert_eq!(payment["status"], "draft");
    assert_eq!(payment["payment_method"], "check");
    assert!(payment["payment_number"].as_str().unwrap().starts_with("PAY-"));
}

#[tokio::test]
#[ignore]
async fn test_list_payments() {
    let (_state, app) = setup_payment_test().await;

    create_test_payment(&app, "1000.00").await;
    create_test_payment(&app, "2000.00").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET").uri("/api/v1/payments")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(result["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
#[ignore]
async fn test_get_payment() {
    let (_state, app) = setup_payment_test().await;

    let payment = create_test_payment(&app, "1500.00").await;
    let payment_id = payment["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/payments/{}", payment_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}

#[tokio::test]
#[ignore]
async fn test_issue_payment() {
    let (_state, app) = setup_payment_test().await;

    let payment = create_test_payment(&app, "3000.00").await;
    let payment_id = payment["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/payments/{}/issue", payment_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let issued: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(issued["status"], "issued");
}

#[tokio::test]
#[ignore]
async fn test_clear_payment() {
    let (_state, app) = setup_payment_test().await;

    let payment = create_test_payment(&app, "1500.00").await;
    let payment_id = payment["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    // Issue first
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/payments/{}/issue", payment_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Clear
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/payments/{}/clear", payment_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let cleared: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(cleared["status"], "cleared");
}

#[tokio::test]
#[ignore]
async fn test_void_payment() {
    let (_state, app) = setup_payment_test().await;

    let payment = create_test_payment(&app, "500.00").await;
    let payment_id = payment["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/payments/{}/void", payment_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "reason": "Duplicate payment"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let voided: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(voided["status"], "voided");
}
