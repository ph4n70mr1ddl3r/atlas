//! Auth E2E tests

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;

use super::common::helpers::*;

#[tokio::test]
async fn test_health_check() {
    let app = build_test_app().await;
    let response = app
        .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
        .await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(&body[..], b"OK");
}

#[tokio::test]
async fn test_metrics_endpoint() {
    let app = build_test_app().await;
    let response = app
        .oneshot(Request::builder().uri("/metrics").body(Body::empty()).unwrap())
        .await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let data: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(data["uptime"], "N/A");
}

#[tokio::test]
async fn test_unauthenticated_request_rejected() {
    let app = build_test_app().await;
    let response = app
        .oneshot(Request::builder().uri("/api/v1/schema/test_items").body(Body::empty()).unwrap())
        .await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_invalid_token_rejected() {
    let app = build_test_app().await;
    let response = app
        .oneshot(Request::builder().uri("/api/v1/schema/test_items")
            .header("Authorization", "Bearer invalid-token").body(Body::empty()).unwrap())
        .await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_valid_token_accepted() {
    let state = build_test_state().await;
    state.schema_engine.upsert_entity(test_entity_definition()).await.unwrap();
    let app = build_router(state);
    let (k, v) = auth_header(&admin_claims());
    let response = app
        .oneshot(Request::builder().uri("/api/v1/schema/test_items").header(k, v).body(Body::empty()).unwrap())
        .await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_expired_token_rejected() {
    let state = build_test_state().await;
    let app = build_router(state);
    let expired = Claims {
        sub: "00000000-0000-0000-0000-000000000002".into(),
        email: "admin@atlas.local".into(), name: "Admin".into(),
        roles: vec!["admin".into()],
        org_id: "00000000-0000-0000-0000-000000000001".into(),
        exp: (chrono::Utc::now() - chrono::Duration::hours(1)).timestamp(),
    };
    let (k, v) = auth_header(&expired);
    let response = app
        .oneshot(Request::builder().uri("/api/v1/schema/test_items").header(k, v).body(Body::empty()).unwrap())
        .await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_login_invalid_email() {
    let app = build_test_app().await;
    let response = app
        .oneshot(Request::builder().method("POST").uri("/api/v1/auth/login")
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&json!({"email": "not-an-email", "password": "pw"})).unwrap()))
            .unwrap())
        .await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_login_unknown_email() {
    let app = build_test_app().await;
    let response = app
        .oneshot(Request::builder().method("POST").uri("/api/v1/auth/login")
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&json!({"email": "nobody@example.com", "password": "pw"})).unwrap()))
            .unwrap())
        .await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
