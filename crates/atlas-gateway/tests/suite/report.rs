//! Report and Import/Export E2E tests

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

#[tokio::test]
async fn test_entity_report_not_found() {
    let state = build_test_state().await;
    let app = build_router(state);
    let (k, v) = auth_header(&admin_claims());
    let r = app.oneshot(Request::builder().uri("/api/v1/reports/nonexistent").header(k, v).body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_dashboard_report() {
    let state = build_test_state().await;
    state.schema_engine.upsert_entity(test_entity_definition()).await.unwrap();
    let app = build_router(state);
    let (k, v) = auth_header(&admin_claims());
    let r = app.oneshot(Request::builder().uri("/api/v1/reports/dashboard").header(k, v).body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let report: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(report["report_type"], "dashboard_overview");
    assert!(report["data"]["total_entities"].is_number());
}

#[tokio::test]
async fn test_export_not_found() {
    let state = build_test_state().await;
    let app = build_router(state);
    let (k, v) = auth_header(&admin_claims());
    let r = app.oneshot(Request::builder().uri("/api/v1/export/nonexistent?format=json").header(k, v).body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_import_unsupported_format() {
    let state = build_test_state().await;
    state.schema_engine.upsert_entity(test_entity_definition()).await.unwrap();
    let app = build_router(state);
    let (k, v) = auth_header(&admin_claims());
    let r = app.oneshot(Request::builder().method("POST").uri("/api/v1/import")
        .header("Content-Type", "application/json").header(k, v)
        .body(Body::from(serde_json::to_string(&json!({"entity": "test_items", "format": "csv", "data": {}, "upsert": false})).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let res: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(res["imported"], 0);
    assert!(!res["errors"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_import_not_found() {
    let state = build_test_state().await;
    let app = build_router(state);
    let (k, v) = auth_header(&admin_claims());
    let r = app.oneshot(Request::builder().method("POST").uri("/api/v1/import")
        .header("Content-Type", "application/json").header(k, v)
        .body(Body::from(serde_json::to_string(&json!({"entity": "nope", "format": "json", "data": [], "upsert": false})).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}
