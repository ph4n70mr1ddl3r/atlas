//! Schema introspection E2E tests

use axum::body::Body;
use http::{Request, StatusCode};
use tower::util::ServiceExt;
use super::common::helpers::*;

#[tokio::test]
async fn test_get_entity_schema() {
    let state = build_test_state().await;
    state.schema_engine.upsert_entity(test_entity_definition()).await.unwrap();
    let app = build_router(state);
    let (k, v) = auth_header(&admin_claims());
    let response = app.oneshot(
        Request::builder().uri("/api/v1/schema/test_items").header(k, v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let schema: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(schema["name"], "test_items");
    assert!(schema["fields"].as_array().unwrap().len() >= 5);
}

#[tokio::test]
async fn test_get_entity_schema_not_found() {
    let state = build_test_state().await;
    let app = build_router(state);
    let (k, v) = auth_header(&admin_claims());
    let response = app.oneshot(
        Request::builder().uri("/api/v1/schema/nonexistent").header(k, v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_entity_form() {
    let state = build_test_state().await;
    state.schema_engine.upsert_entity(test_entity_definition()).await.unwrap();
    let app = build_router(state);
    let (k, v) = auth_header(&admin_claims());
    let response = app.oneshot(
        Request::builder().uri("/api/v1/schema/test_items/form").header(k, v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let form: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(form["entity"], "test_items");
    let fields = form["fields"].as_array().unwrap();
    let name_field = fields.iter().find(|f| f["name"] == "name").unwrap();
    assert_eq!(name_field["field_type"], "text");
    let status_field = fields.iter().find(|f| f["name"] == "status").unwrap();
    assert_eq!(status_field["field_type"], "select");
}

#[tokio::test]
async fn test_get_entity_list_view() {
    let state = build_test_state().await;
    state.schema_engine.upsert_entity(test_entity_definition()).await.unwrap();
    let app = build_router(state);
    let (k, v) = auth_header(&admin_claims());
    let response = app.oneshot(
        Request::builder().uri("/api/v1/schema/test_items/list").header(k, v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let config: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(config["entity"], "test_items");
    assert!(config["columns"].is_array());
}
