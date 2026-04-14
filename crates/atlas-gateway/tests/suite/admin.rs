//! Admin API E2E tests

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

#[tokio::test]
#[ignore]
async fn test_create_entity() {
    let state = build_test_state().await;
    let app = build_router(state.clone());
    let (k, v) = auth_header(&admin_claims());
    let def = json!({"definition": {
        "name": "e2e_new", "label": "New", "pluralLabel": "News", "tableName": "e2e_new",
        "fields": [
            {"name": "title", "label": "Title", "fieldType": {"type": "string", "maxLength": 200},
             "isRequired": true, "isSearchable": true, "displayOrder": 1, "validations": [],
             "visibility": {"condition": null, "roles": [], "hidden": false}}
        ],
        "indexes": [], "isAuditEnabled": true, "isSoftDelete": true, "metadata": {}
    }});
    let r = app.oneshot(Request::builder().method("POST").uri("/api/admin/schema")
        .header("Content-Type", "application/json").header(k, v)
        .body(Body::from(serde_json::to_string(&def).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let res: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(res["entity"], "e2e_new");
    sqlx::query("DROP TABLE IF EXISTS e2e_new").execute(&state.db_pool).await.ok();
}

#[tokio::test]
#[ignore]
async fn test_update_entity() {
    let state = build_test_state().await;
    state.schema_engine.upsert_entity(test_entity_definition()).await.unwrap();
    let app = build_router(state.clone());
    let (k, v) = auth_header(&admin_claims());
    let upd = json!({"definition": {
        "name": "test_items", "label": "Updated Item", "pluralLabel": "Updated Items",
        "tableName": "test_items",
        "fields": [
            {"name": "name", "label": "Name", "fieldType": {"type": "string", "maxLength": 300},
             "isRequired": true, "isSearchable": true, "displayOrder": 1, "validations": [],
             "visibility": {"condition": null, "roles": [], "hidden": false}}
        ],
        "indexes": [], "isAuditEnabled": true, "isSoftDelete": true, "metadata": {}
    }});
    let r = app.oneshot(Request::builder().method("PUT").uri("/api/admin/schema/test_items")
        .header("Content-Type", "application/json").header(k, v)
        .body(Body::from(serde_json::to_string(&upd).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let res: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(res["updated"], true);
    assert_eq!(state.schema_engine.get_entity("test_items").unwrap().label, "Updated Item");
}

#[tokio::test]
#[ignore]
async fn test_delete_entity() {
    let state = build_test_state().await;
    state.schema_engine.upsert_entity(test_entity_definition()).await.unwrap();
    let app = build_router(state);
    let (k, v) = auth_header(&admin_claims());
    let r = app.oneshot(Request::builder().method("DELETE").uri("/api/admin/schema/test_items")
        .header("Content-Type", "application/json").header(k, v)
        .body(Body::from(serde_json::to_string(&json!({"drop_table": false})).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_config() {
    let state = build_test_state().await;
    state.schema_engine.upsert_entity(test_entity_definition()).await.unwrap();
    let app = build_router(state);
    let (k, v) = auth_header(&admin_claims());
    let r = app.oneshot(Request::builder().uri("/api/admin/config").header(k, v).body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let c: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(c["entities"].is_array());
}

#[tokio::test]
async fn test_get_config_value() {
    let state = build_test_state().await;
    state.schema_engine.upsert_entity(test_entity_definition()).await.unwrap();
    let app = build_router(state);
    let (k, v) = auth_header(&admin_claims());
    let r = app.oneshot(Request::builder().uri("/api/admin/config/entity.test_items").header(k, v).body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let val: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(val["name"], "test_items");
}

#[tokio::test]
async fn test_config_not_found() {
    let state = build_test_state().await;
    let app = build_router(state);
    let (k, v) = auth_header(&admin_claims());
    let r = app.oneshot(Request::builder().uri("/api/admin/config/entity.nothing").header(k, v).body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_clear_cache() {
    let state = build_test_state().await;
    let app = build_router(state);
    let (k, v) = auth_header(&admin_claims());
    let r = app.oneshot(Request::builder().method("POST").uri("/api/admin/cache/clear").header(k, v).body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let res: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(res["cache_cleared"], true);
}
