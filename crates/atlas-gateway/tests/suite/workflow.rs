//! Workflow E2E tests

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use uuid::Uuid;
use atlas_shared::{EntityDefinition, FieldDefinition, FieldType};
use super::common::helpers::*;

async fn setup_wf() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    state.schema_engine.upsert_entity(test_entity_definition()).await.unwrap();
    if let Some(ref wf) = state.schema_engine.get_entity("test_items").unwrap().workflow {
        state.workflow_engine.load_workflow(wf.clone()).await.unwrap();
    }
    setup_test_db(&state.db_pool).await;
    let app = build_router(state.clone());
    (state, app)
}

async fn create_item(app: &axum::Router, k: &str, v: &str) -> String {
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/test_items")
        .header("Content-Type", "application/json").header(k, v)
        .body(Body::from(serde_json::to_string(&json!({
            "entity": "test_items", "values": {"name": "WF Test", "quantity": 10, "price": 99.99, "status": "draft"}
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice::<serde_json::Value>(&b).unwrap()["id"].as_str().unwrap().to_string()
}

async fn do_action(app: &axum::Router, k: &str, v: &str, id: &str, action: &str) -> serde_json::Value {
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/test_items/{}/{}", id, action))
        .header("Content-Type", "application/json").header(k, v)
        .body(Body::from(serde_json::to_string(&json!({"action": action, "comment": "test"})).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn get_transitions(app: &axum::Router, k: &str, v: &str, id: &str) -> serde_json::Value {
    let r = app.clone().oneshot(Request::builder()
        .uri(format!("/api/v1/test_items/{}/transitions", id)).header(k, v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

#[tokio::test]
#[ignore]
async fn test_get_transitions() {
    let (state, app) = setup_wf().await;
    let (k, v) = auth_header(&admin_claims());
    let id = create_item(&app, &k, &v).await;
    let t = get_transitions(&app, &k, &v, &id).await;
    assert_eq!(t["current_state"], "draft");
    assert_eq!(t["transitions"].as_array().unwrap().len(), 1);
    assert_eq!(t["transitions"][0]["action"], "submit");
    cleanup_test_db(&state.db_pool).await;
}

#[tokio::test]
#[ignore]
async fn test_execute_transition() {
    let (state, app) = setup_wf().await;
    let (k, v) = auth_header(&admin_claims());
    let id = create_item(&app, &k, &v).await;
    let r = do_action(&app, &k, &v, &id, "submit").await;
    assert_eq!(r["success"], true);
    assert_eq!(r["from_state"], "draft");
    assert_eq!(r["to_state"], "submitted");
    cleanup_test_db(&state.db_pool).await;
}

#[tokio::test]
#[ignore]
async fn test_transition_role_denied() {
    let (state, app) = setup_wf().await;
    let (ak, av) = auth_header(&admin_claims());
    let id = create_item(&app, &ak, &av).await;
    do_action(&app, &ak, &av, &id, "submit").await;
    let (uk, uv) = auth_header(&user_claims());
    let r = do_action(&app, &uk, &uv, &id, "approve").await;
    assert_eq!(r["success"], false);
    cleanup_test_db(&state.db_pool).await;
}

#[tokio::test]
#[ignore]
async fn test_invalid_transition() {
    let (state, app) = setup_wf().await;
    let (k, v) = auth_header(&admin_claims());
    let id = create_item(&app, &k, &v).await;
    let r = app.oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/test_items/{}/approve", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"action": "approve", "comment": "skip"})).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
    cleanup_test_db(&state.db_pool).await;
}

#[tokio::test]
#[ignore]
async fn test_full_lifecycle() {
    let (state, app) = setup_wf().await;
    let (k, v) = auth_header(&admin_claims());
    let id = create_item(&app, &k, &v).await;
    let t = get_transitions(&app, &k, &v, &id).await;
    assert_eq!(t["current_state"], "draft");
    do_action(&app, &k, &v, &id, "submit").await;
    let r = do_action(&app, &k, &v, &id, "approve").await;
    assert_eq!(r["success"], true);
    assert_eq!(r["to_state"], "approved");
    let t = get_transitions(&app, &k, &v, &id).await;
    assert_eq!(t["transitions"].as_array().unwrap().len(), 0);
    cleanup_test_db(&state.db_pool).await;
}

#[tokio::test]
#[ignore]
async fn test_rejection_path() {
    let (state, app) = setup_wf().await;
    let (k, v) = auth_header(&admin_claims());
    let id = create_item(&app, &k, &v).await;
    do_action(&app, &k, &v, &id, "submit").await;
    let r = do_action(&app, &k, &v, &id, "reject").await;
    assert_eq!(r["success"], true);
    assert_eq!(r["to_state"], "rejected");
    cleanup_test_db(&state.db_pool).await;
}

#[tokio::test]
#[ignore]
async fn test_record_history() {
    let (state, app) = setup_wf().await;
    let (k, v) = auth_header(&admin_claims());
    let id = create_item(&app, &k, &v).await;
    do_action(&app, &k, &v, &id, "submit").await;
    let r = app.oneshot(Request::builder()
        .uri(format!("/api/v1/test_items/{}/history", id)).header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let h: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(h["entity"], "test_items");
    cleanup_test_db(&state.db_pool).await;
}

#[tokio::test]
async fn test_transitions_no_workflow() {
    let state = build_test_state().await;
    let entity = EntityDefinition {
        id: Some(Uuid::new_v4()), name: "simple".into(), label: "Simple".into(),
        plural_label: "Simples".into(), table_name: Some("simple".into()), description: None,
        fields: vec![FieldDefinition::new("name", "Name", FieldType::String { max_length: None, pattern: None })],
        indexes: vec![], workflow: None, security: None, is_audit_enabled: true, is_soft_delete: true,
        icon: None, color: None, metadata: serde_json::Value::Null,
    };
    state.schema_engine.upsert_entity(entity).await.unwrap();
    let app = build_router(state);
    let (k, v) = auth_header(&admin_claims());
    let r = app.oneshot(Request::builder()
        .uri(format!("/api/v1/simple/{}/transitions", Uuid::new_v4())).header(k, v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["transitions"].as_array().unwrap().len(), 0);
}
