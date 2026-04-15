//! CRUD E2E tests

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use uuid::Uuid;
use super::common::helpers::*;

#[tokio::test]
async fn test_list_records_entity_not_found() {
    let state = build_test_state().await;
    let app = build_router(state);
    let (k, v) = auth_header(&admin_claims());
    let resp = app.oneshot(
        Request::builder().uri("/api/v1/nonexistent").header(k, v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
#[ignore]
async fn test_crud_full_lifecycle() {
    let state = build_test_state().await;
    state.schema_engine.upsert_entity(test_entity_definition()).await.unwrap();
    if let Some(ref wf) = state.schema_engine.get_entity("test_items").unwrap().workflow {
        state.workflow_engine.load_workflow(wf.clone()).await.unwrap();
    }
    setup_test_db(&state.db_pool).await;
    let app = build_router(state);
    let (k, v) = auth_header(&admin_claims());

    // CREATE
    let resp = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/test_items")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "entity": "test_items", "values": {"name": "E2E Item", "quantity": 42, "price": 19.99, "status": "draft"}
        })).unwrap())).unwrap()
    ).await.unwrap();
    let status = resp.status();
    assert_eq!(status, StatusCode::CREATED, "CREATE failed: {}", status);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let created: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let record_id = created["id"].as_str().unwrap().to_string();
    assert_eq!(created["name"], "E2E Item");

    // READ
    let resp = app.clone().oneshot(
        Request::builder().uri(format!("/api/v1/test_items/{}", record_id)).header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let fetched: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(fetched["name"], "E2E Item");

    // LIST
    let resp = app.clone().oneshot(
        Request::builder().uri("/api/v1/test_items").header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let list: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(!list["data"].as_array().unwrap().is_empty());

    // UPDATE
    let resp = app.clone().oneshot(Request::builder().method("PUT")
        .uri(format!("/api/v1/test_items/{}", record_id)).header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "entity": "test_items", "id": record_id, "values": {"name": "Updated", "quantity": 100}
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(updated["name"], "Updated");
    assert_eq!(updated["quantity"], 100);

    // DELETE
    let resp = app.oneshot(Request::builder().method("DELETE")
        .uri(format!("/api/v1/test_items/{}", record_id)).header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
#[ignore]
async fn test_sql_injection_prevented() {
    let state = build_test_state().await;
    state.schema_engine.upsert_entity(test_entity_definition()).await.unwrap();
    setup_test_db(&state.db_pool).await;
    let app = build_router(state);
    let (k, v) = auth_header(&admin_claims());
    let resp = app.oneshot(Request::builder().method("POST").uri("/api/v1/test_items")
        .header("Content-Type", "application/json").header(k, v)
        .body(Body::from(serde_json::to_string(&json!({
            "entity": "test_items", "values": {"name; DROP TABLE test_items--": "x"}
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore]
async fn test_delete_nonexistent() {
    let state = build_test_state().await;
    state.schema_engine.upsert_entity(test_entity_definition()).await.unwrap();
    setup_test_db(&state.db_pool).await;
    let app = build_router(state);
    let (k, v) = auth_header(&admin_claims());
    let resp = app.oneshot(Request::builder().method("DELETE")
        .uri(format!("/api/v1/test_items/{}", Uuid::new_v4())).header(k, v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}
