//! Lockbox Processing E2E Tests

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    let migration_sql = include_str!("../../../../migrations/112_lockbox.sql");
    sqlx::raw_sql(migration_sql).execute(&state.db_pool).await.ok();
    let app = build_router(state.clone());
    (state, app)
}

#[tokio::test]
async fn test_create_lockbox_batch() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/lockbox/batches")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "batch_number": "LB-E2E-001",
            "lockbox_number": "LB12345",
            "bank_name": "Chase Bank",
            "deposit_date": "2026-05-01",
            "currency_code": "USD",
            "source_file_name": "lockbox_0501.txt",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let batch: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(batch["batch_number"], "LB-E2E-001");
    assert_eq!(batch["status"], "imported");
}

#[tokio::test]
async fn test_create_batch_empty_number() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/lockbox/batches")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "batch_number": "",
            "lockbox_number": "LB123",
            "deposit_date": "2026-05-01",
            "currency_code": "USD",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_batch_bad_currency() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/lockbox/batches")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "batch_number": "LB-BAD",
            "lockbox_number": "LB123",
            "deposit_date": "2026-05-01",
            "currency_code": "US",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_list_batches() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/lockbox/batches")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_lockbox_dashboard() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/lockbox/dashboard")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(d["total_batches"].is_number());
}

#[tokio::test]
async fn test_create_lockbox_format() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/lockbox/formats")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "format_code": "BAI2-STD",
            "name": "Standard BAI2",
            "format_type": "BAI2",
            "field_delimiter": ",",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn test_create_format_invalid_type() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/lockbox/formats")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "format_code": "CSV",
            "name": "CSV Format",
            "format_type": "CSV",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_list_formats() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/lockbox/formats")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_duplicate_batch() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let body = serde_json::to_string(&json!({
        "batch_number": "LB-DUP",
        "lockbox_number": "LB123",
        "deposit_date": "2026-05-01",
        "currency_code": "USD",
    })).unwrap();
    let r1 = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/lockbox/batches")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(body.clone())).unwrap()
    ).await.unwrap();
    assert_eq!(r1.status(), StatusCode::CREATED);
    let r2 = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/lockbox/batches")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(body)).unwrap()
    ).await.unwrap();
    assert_eq!(r2.status(), StatusCode::CONFLICT);
}
