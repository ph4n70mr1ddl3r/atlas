//! AR Aging Analysis E2E Tests

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    let migration_sql = include_str!("../../../../migrations/113_ar_aging.sql");
    sqlx::raw_sql(migration_sql).execute(&state.db_pool).await.ok();
    let app = build_router(state.clone());
    (state, app)
}

#[tokio::test]
async fn test_create_aging_definition() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/ar-aging/definitions")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "definition_code": "STD-AGING",
            "name": "Standard Aging",
            "aging_basis": "due_date",
            "num_buckets": 5,
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["definition_code"], "STD-AGING");
    assert_eq!(d["status"], "active");
    assert_eq!(d["num_buckets"], 5);
}

#[tokio::test]
async fn test_create_definition_invalid_basis() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/ar-aging/definitions")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "definition_code": "BAD",
            "name": "Bad",
            "aging_basis": "invalid",
            "num_buckets": 5,
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_definition_zero_buckets() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/ar-aging/definitions")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "definition_code": "ZERO",
            "name": "Zero Buckets",
            "aging_basis": "due_date",
            "num_buckets": 0,
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_definition_too_many_buckets() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/ar-aging/definitions")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "definition_code": "TOO_MANY",
            "name": "Too Many",
            "aging_basis": "due_date",
            "num_buckets": 25,
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_definition_empty_code() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/ar-aging/definitions")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "definition_code": "",
            "name": "Empty Code",
            "aging_basis": "due_date",
            "num_buckets": 5,
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_list_definitions() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/ar-aging/definitions")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_ar_aging_dashboard() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/ar-aging/dashboard")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(d["total_definitions"].is_number());
}

#[tokio::test]
async fn test_duplicate_definition() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let body = serde_json::to_string(&json!({
        "definition_code": "DUP-DEF",
        "name": "Dup Def",
        "aging_basis": "due_date",
        "num_buckets": 5,
    })).unwrap();
    let r1 = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/ar-aging/definitions")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(body.clone())).unwrap()
    ).await.unwrap();
    assert_eq!(r1.status(), StatusCode::CREATED);
    let r2 = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/ar-aging/definitions")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(body)).unwrap()
    ).await.unwrap();
    assert_eq!(r2.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_create_definition_invoice_date_basis() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/ar-aging/definitions")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "definition_code": "INV-DATE",
            "name": "Invoice Date Aging",
            "aging_basis": "invoice_date",
            "num_buckets": 4,
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
}
