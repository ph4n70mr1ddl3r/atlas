//! Payment Terms Management E2E Tests

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    let migration_sql = include_str!("../../../../migrations/111_payment_terms.sql");
    sqlx::raw_sql(migration_sql).execute(&state.db_pool).await.ok();
    let app = build_router(state.clone());
    (state, app)
}

#[tokio::test]
async fn test_create_payment_term() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/payment-terms")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "term_code": "NET30",
            "name": "Net 30 Days",
            "base_due_days": 30,
            "term_type": "standard",
            "default_discount_percent": "0",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let t: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(t["term_code"], "NET30");
    assert_eq!(t["status"], "active");
}

#[tokio::test]
async fn test_create_payment_term_with_discount() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/payment-terms")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "term_code": "2_10_NET30",
            "name": "2% 10 Net 30",
            "base_due_days": 30,
            "term_type": "standard",
            "default_discount_percent": "2",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn test_create_payment_term_invalid_type() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/payment-terms")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "term_code": "BAD",
            "name": "Bad Term",
            "base_due_days": 30,
            "term_type": "invalid",
            "default_discount_percent": "0",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_payment_term_empty_code() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/payment-terms")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "term_code": "",
            "name": "Empty Code",
            "base_due_days": 30,
            "term_type": "standard",
            "default_discount_percent": "0",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_payment_term_negative_due_days() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/payment-terms")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "term_code": "BAD2",
            "name": "Negative",
            "base_due_days": -5,
            "term_type": "standard",
            "default_discount_percent": "0",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_list_payment_terms() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/payment-terms")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_payment_terms_dashboard() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/payment-terms/dashboard")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(d["total_terms"].is_number());
}

#[tokio::test]
async fn test_create_installment_term() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/payment-terms")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "term_code": "INSTALL_50_50",
            "name": "50/50 Installment",
            "base_due_days": 60,
            "term_type": "installment",
            "default_discount_percent": "0",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn test_create_proxima_term() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/payment-terms")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "term_code": "PROX_25",
            "name": "Proxima 25th",
            "base_due_days": 30,
            "due_date_cutoff_day": 25,
            "term_type": "proxima",
            "default_discount_percent": "0",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn test_duplicate_payment_term() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let body = serde_json::to_string(&json!({
        "term_code": "DUP_NET30",
        "name": "Net 30",
        "base_due_days": 30,
        "term_type": "standard",
        "default_discount_percent": "0",
    })).unwrap();
    let r1 = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/payment-terms")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(body.clone())).unwrap()
    ).await.unwrap();
    assert_eq!(r1.status(), StatusCode::CREATED);
    let r2 = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/payment-terms")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(body)).unwrap()
    ).await.unwrap();
    assert_eq!(r2.status(), StatusCode::CONFLICT);
}
