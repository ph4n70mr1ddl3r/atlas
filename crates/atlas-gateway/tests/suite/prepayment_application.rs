//! Prepayment Application E2E Tests
//!
//! Oracle Fusion: Payables > Prepayment Application

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    let migration_sql = include_str!("../../../../migrations/115_mass_additions_reclassification_budget_transfer.sql");
    sqlx::raw_sql(migration_sql).execute(&state.db_pool).await.ok();
    let app = build_router(state.clone());
    (state, app)
}

#[tokio::test]
async fn test_apply_prepayment_zero_amount() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/prepayment-applications")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "prepayment_invoice_id": uuid::Uuid::new_v4(),
            "standard_invoice_id": uuid::Uuid::new_v4(),
            "supplier_id": uuid::Uuid::new_v4(),
            "applied_amount": "0",
            "remaining_prepayment_amount": "5000",
            "currency_code": "USD",
            "application_date": "2026-05-15",
            "gl_date": "2026-05-15",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_apply_prepayment_negative_amount() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/prepayment-applications")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "prepayment_invoice_id": uuid::Uuid::new_v4(),
            "standard_invoice_id": uuid::Uuid::new_v4(),
            "supplier_id": uuid::Uuid::new_v4(),
            "applied_amount": "-100",
            "remaining_prepayment_amount": "5000",
            "currency_code": "USD",
            "application_date": "2026-05-15",
            "gl_date": "2026-05-15",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_apply_prepayment_invalid_amount() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/prepayment-applications")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "prepayment_invoice_id": uuid::Uuid::new_v4(),
            "standard_invoice_id": uuid::Uuid::new_v4(),
            "supplier_id": uuid::Uuid::new_v4(),
            "applied_amount": "not-a-number",
            "remaining_prepayment_amount": "5000",
            "currency_code": "USD",
            "application_date": "2026-05-15",
            "gl_date": "2026-05-15",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_list_prepayments() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/prepayment-applications")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(d["data"].is_array());
}

#[tokio::test]
async fn test_get_prepayment_not_found() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let id = uuid::Uuid::new_v4();
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/prepayment-applications/{}", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_prepayment_dashboard() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/prepayment-applications/dashboard")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(d["total_applications"].is_number());
}

#[tokio::test]
async fn test_list_prepayments_with_filter() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let supplier_id = uuid::Uuid::new_v4();
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/prepayment-applications?status=draft&supplier_id={}", supplier_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_confirm_not_found() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let id = uuid::Uuid::new_v4();
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/prepayment-applications/{}/confirm", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_cancel_not_found() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let id = uuid::Uuid::new_v4();
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/prepayment-applications/{}/cancel", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}
