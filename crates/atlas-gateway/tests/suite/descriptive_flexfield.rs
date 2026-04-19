//! Descriptive Flexfields (DFF) E2E Tests
//!
//! Tests for Oracle Fusion Cloud ERP Descriptive Flexfields:
//! - Value set CRUD and entry management
//! - Flexfield creation and lifecycle
//! - Context management (global and context-sensitive)
//! - Segment management with value set binding
//! - Flexfield data CRUD with validation
//! - Dashboard summary
//! - Error cases and edge cases

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_dff_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    let app = build_router(state.clone());
    (state, app)
}

// ═══════════════════════════════════════════════════════════════════════════════
// Value Set Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_create_value_set() {
    let (_state, app) = setup_dff_test().await;
    let (k, v) = auth_header(&admin_claims());

    let resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/flexfields/value-sets")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "VS_DEPARTMENT",
            "name": "Department Values",
            "description": "List of departments",
            "validation_type": "independent",
            "data_type": "string",
            "max_length": 100,
            "min_length": 1,
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let vs: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(vs["code"], "VS_DEPARTMENT");
    assert_eq!(vs["name"], "Department Values");
    assert_eq!(vs["validation_type"], "independent");
    assert_eq!(vs["data_type"], "string");
    assert_eq!(vs["max_length"], 100);
    assert_eq!(vs["is_active"], true);
}

#[tokio::test]
async fn test_list_value_sets() {
    let (_state, app) = setup_dff_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Create two value sets
    for code in &["VS_DEPT", "VS_REGION"] {
        app.clone().oneshot(Request::builder()
            .method("POST")
            .uri("/api/v1/flexfields/value-sets")
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({
                "code": code,
                "name": format!("{} values", code),
                "validation_type": "independent",
                "data_type": "string",
            })).unwrap()))
            .unwrap()
        ).await.unwrap();
    }

    let resp = app.clone().oneshot(Request::builder()
        .method("GET")
        .uri("/api/v1/flexfields/value-sets")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let list: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert_eq!(list.len(), 2);
}

#[tokio::test]
async fn test_get_value_set() {
    let (_state, app) = setup_dff_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Create first
    app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/flexfields/value-sets")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "VS_COLOR",
            "name": "Colors",
            "validation_type": "independent",
            "data_type": "string",
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    // Get it
    let resp = app.clone().oneshot(Request::builder()
        .method("GET")
        .uri("/api/v1/flexfields/value-sets/VS_COLOR")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let vs: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(vs["code"], "VS_COLOR");
}

#[tokio::test]
async fn test_delete_value_set() {
    let (_state, app) = setup_dff_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Create
    app.clone().oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/flexfields/value-sets")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "VS_TEMP",
            "name": "Temporary",
            "validation_type": "none",
            "data_type": "string",
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    // Delete
    let resp = app.clone().oneshot(Request::builder()
        .method("DELETE")
        .uri("/api/v1/flexfields/value-sets/VS_TEMP")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    // Verify gone
    let resp = app.clone().oneshot(Request::builder()
        .method("GET")
        .uri("/api/v1/flexfields/value-sets/VS_TEMP")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_value_set_duplicate_code_rejected() {
    let (_state, app) = setup_dff_test().await;
    let (k, v) = auth_header(&admin_claims());

    let payload = serde_json::to_string(&json!({
        "code": "VS_DUP",
        "name": "First",
        "validation_type": "none",
        "data_type": "string",
    })).unwrap();

    app.clone().oneshot(Request::builder()
        .method("POST").uri("/api/v1/flexfields/value-sets")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(payload.clone())).unwrap()
    ).await.unwrap();

    let resp = app.clone().oneshot(Request::builder()
        .method("POST").uri("/api/v1/flexfields/value-sets")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(payload)).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Value Set Entry Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_create_and_list_value_set_entries() {
    let (_state, app) = setup_dff_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Create value set
    app.clone().oneshot(Request::builder()
        .method("POST").uri("/api/v1/flexfields/value-sets")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "VS_DEPT",
            "name": "Departments",
            "validation_type": "independent",
            "data_type": "string",
        })).unwrap())).unwrap()
    ).await.unwrap();

    // Add entries
    for (val, meaning) in &[("ENGINEERING", "Engineering"), ("MARKETING", "Marketing"), ("FINANCE", "Finance")] {
        let resp = app.clone().oneshot(Request::builder()
            .method("POST").uri("/api/v1/flexfields/value-sets/VS_DEPT/entries")
            .header("Content-Type", "application/json").header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({
                "value": val,
                "meaning": meaning,
                "sort_order": 1,
            })).unwrap())).unwrap()
        ).await.unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);
    }

    // List entries
    let resp = app.clone().oneshot(Request::builder()
        .method("GET").uri("/api/v1/flexfields/value-sets/VS_DEPT/entries")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let entries: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert_eq!(entries.len(), 3);

    // Verify entry values
    let values: Vec<&str> = entries.iter().map(|e| e["value"].as_str().unwrap()).collect();
    assert!(values.contains(&"ENGINEERING"));
    assert!(values.contains(&"MARKETING"));
    assert!(values.contains(&"FINANCE"));
}

#[tokio::test]
async fn test_delete_value_set_entry() {
    let (_state, app) = setup_dff_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Create value set and entry
    app.clone().oneshot(Request::builder()
        .method("POST").uri("/api/v1/flexfields/value-sets")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "VS_TMP",
            "name": "Temp",
            "validation_type": "independent",
            "data_type": "string",
        })).unwrap())).unwrap()
    ).await.unwrap();

    let resp = app.clone().oneshot(Request::builder()
        .method("POST").uri("/api/v1/flexfields/value-sets/VS_TMP/entries")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "value": "VAL1",
            "meaning": "Value 1",
        })).unwrap())).unwrap()
    ).await.unwrap();

    let entry: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    let entry_id = entry["id"].as_str().unwrap();

    // Delete
    let resp = app.clone().oneshot(Request::builder()
        .method("DELETE")
        .uri(&format!("/api/v1/flexfields/value-sets/entries/{}", entry_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    // Verify gone
    let resp = app.clone().oneshot(Request::builder()
        .method("GET").uri("/api/v1/flexfields/value-sets/VS_TMP/entries")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let entries: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert!(entries.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Flexfield Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_create_flexfield() {
    let (_state, app) = setup_dff_test().await;
    let (k, v) = auth_header(&admin_claims());

    let resp = app.clone().oneshot(Request::builder()
        .method("POST").uri("/api/v1/flexfields")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "PO_DFF",
            "name": "Purchase Order DFF",
            "description": "Custom fields for purchase orders",
            "entity_name": "purchase_orders",
            "context_column": "dff_context",
            "default_context_code": "GLOBAL",
        })).unwrap())).unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let ff: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(ff["code"], "PO_DFF");
    assert_eq!(ff["entity_name"], "purchase_orders");
    assert_eq!(ff["context_column"], "dff_context");
    assert_eq!(ff["default_context_code"], "GLOBAL");
    assert_eq!(ff["is_active"], true);
}

#[tokio::test]
async fn test_list_flexfields() {
    let (_state, app) = setup_dff_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Create flexfield
    app.clone().oneshot(Request::builder()
        .method("POST").uri("/api/v1/flexfields")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "INV_DFF",
            "name": "Invoice DFF",
            "entity_name": "invoices",
        })).unwrap())).unwrap()
    ).await.unwrap();

    let resp = app.clone().oneshot(Request::builder()
        .method("GET").uri("/api/v1/flexfields")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let list: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0]["code"], "INV_DFF");
}

#[tokio::test]
async fn test_activate_deactivate_flexfield() {
    let (_state, app) = setup_dff_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Create flexfield
    let resp = app.clone().oneshot(Request::builder()
        .method("POST").uri("/api/v1/flexfields")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "TEST_DFF",
            "name": "Test DFF",
            "entity_name": "test_entity_1",
        })).unwrap())).unwrap()
    ).await.unwrap();
    let ff: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    let ff_id = ff["id"].as_str().unwrap();

    // Deactivate
    let resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri(&format!("/api/v1/flexfields/{}/deactivate", ff_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let ff: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(ff["is_active"], false);

    // Reactivate
    let resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri(&format!("/api/v1/flexfields/{}/activate", ff_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let ff: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(ff["is_active"], true);
}

#[tokio::test]
async fn test_duplicate_entity_flexfield_rejected() {
    let (_state, app) = setup_dff_test().await;
    let (k, v) = auth_header(&admin_claims());

    let payload = serde_json::to_string(&json!({
        "code": "DFF1",
        "name": "First DFF",
        "entity_name": "orders",
    })).unwrap();

    app.clone().oneshot(Request::builder()
        .method("POST").uri("/api/v1/flexfields")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(payload.clone())).unwrap()
    ).await.unwrap();

    let resp = app.clone().oneshot(Request::builder()
        .method("POST").uri("/api/v1/flexfields")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "DFF2",
            "name": "Second DFF",
            "entity_name": "orders",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_delete_flexfield() {
    let (_state, app) = setup_dff_test().await;
    let (k, v) = auth_header(&admin_claims());

    app.clone().oneshot(Request::builder()
        .method("POST").uri("/api/v1/flexfields")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "DEL_DFF",
            "name": "To Delete",
            "entity_name": "del_entity",
        })).unwrap())).unwrap()
    ).await.unwrap();

    let resp = app.clone().oneshot(Request::builder()
        .method("DELETE").uri("/api/v1/flexfields/DEL_DFF")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Context Tests
// ═══════════════════════════════════════════════════════════════════════════════

async fn setup_flexfield_with_context(app: &axum::Router) {
    let (k, v) = auth_header(&admin_claims());

    // Create flexfield
    app.clone().oneshot(Request::builder()
        .method("POST").uri("/api/v1/flexfields")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "PO_DFF",
            "name": "PO DFF",
            "entity_name": "purchase_orders",
        })).unwrap())).unwrap()
    ).await.unwrap();

    // Create global context
    app.clone().oneshot(Request::builder()
        .method("POST").uri("/api/v1/flexfields/PO_DFF/contexts")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "GLOBAL",
            "name": "Global Context",
            "description": "Applies to all POs",
            "is_global": true,
        })).unwrap())).unwrap()
    ).await.unwrap();

    // Create a specific context
    app.clone().oneshot(Request::builder()
        .method("POST").uri("/api/v1/flexfields/PO_DFF/contexts")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "IT_EQUIPMENT",
            "name": "IT Equipment",
            "description": "IT equipment specific fields",
            "is_global": false,
        })).unwrap())).unwrap()
    ).await.unwrap();
}

#[tokio::test]
async fn test_create_contexts() {
    let (_state, app) = setup_dff_test().await;
    setup_flexfield_with_context(&app).await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder()
        .method("GET").uri("/api/v1/flexfields/PO_DFF/contexts")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let contexts: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert_eq!(contexts.len(), 2);

    let global = contexts.iter().find(|c| c["code"] == "GLOBAL").unwrap();
    assert_eq!(global["is_global"], true);

    let it = contexts.iter().find(|c| c["code"] == "IT_EQUIPMENT").unwrap();
    assert_eq!(it["is_global"], false);
}

#[tokio::test]
async fn test_disable_enable_context() {
    let (_state, app) = setup_dff_test().await;
    setup_flexfield_with_context(&app).await;

    let (k, v) = auth_header(&admin_claims());

    // Get context ID
    let resp = app.clone().oneshot(Request::builder()
        .method("GET").uri("/api/v1/flexfields/PO_DFF/contexts")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let contexts: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    let it_ctx = contexts.iter().find(|c| c["code"] == "IT_EQUIPMENT").unwrap();
    let ctx_id = it_ctx["id"].as_str().unwrap();

    // Disable
    let resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri(&format!("/api/v1/flexfields/contexts/{}/disable", ctx_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let ctx: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(ctx["is_enabled"], false);

    // Enable
    let resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri(&format!("/api/v1/flexfields/contexts/{}/enable", ctx_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let ctx: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(ctx["is_enabled"], true);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Segment Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_create_segments() {
    let (_state, app) = setup_dff_test().await;
    setup_flexfield_with_context(&app).await;
    let (k, v) = auth_header(&admin_claims());

    // Create a value set for department
    app.clone().oneshot(Request::builder()
        .method("POST").uri("/api/v1/flexfields/value-sets")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "VS_DEPT",
            "name": "Departments",
            "validation_type": "independent",
            "data_type": "string",
        })).unwrap())).unwrap()
    ).await.unwrap();

    // Add segment to GLOBAL context
    let resp = app.clone().oneshot(Request::builder()
        .method("POST").uri("/api/v1/flexfields/PO_DFF/contexts/GLOBAL/segments")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "segment_code": "department",
            "name": "Department",
            "description": "Requesting department",
            "display_order": 1,
            "column_name": "attribute1",
            "data_type": "string",
            "is_required": true,
            "value_set_code": "VS_DEPT",
        })).unwrap())).unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let seg: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(seg["segment_code"], "department");
    assert_eq!(seg["name"], "Department");
    assert_eq!(seg["data_type"], "string");
    assert_eq!(seg["is_required"], true);
    assert_eq!(seg["value_set_code"], "VS_DEPT");
    assert_eq!(seg["display_order"], 1);

    // Add another segment
    let resp = app.clone().oneshot(Request::builder()
        .method("POST").uri("/api/v1/flexfields/PO_DFF/contexts/GLOBAL/segments")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "segment_code": "priority",
            "name": "Priority",
            "display_order": 2,
            "column_name": "attribute2",
            "data_type": "string",
            "default_value": "medium",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    // List segments
    let resp = app.clone().oneshot(Request::builder()
        .method("GET").uri("/api/v1/flexfields/PO_DFF/contexts/GLOBAL/segments")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let segments: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert_eq!(segments.len(), 2);
    assert_eq!(segments[0]["segment_code"], "department");
    assert_eq!(segments[1]["segment_code"], "priority");
}

#[tokio::test]
async fn test_segments_in_different_contexts() {
    let (_state, app) = setup_dff_test().await;
    setup_flexfield_with_context(&app).await;
    let (k, v) = auth_header(&admin_claims());

    // Add segment to IT_EQUIPMENT context
    let resp = app.clone().oneshot(Request::builder()
        .method("POST").uri("/api/v1/flexfields/PO_DFF/contexts/IT_EQUIPMENT/segments")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "segment_code": "asset_tag",
            "name": "Asset Tag",
            "display_order": 1,
            "column_name": "attribute1",
            "data_type": "string",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    // List segments for IT_EQUIPMENT
    let resp = app.clone().oneshot(Request::builder()
        .method("GET").uri("/api/v1/flexfields/PO_DFF/contexts/IT_EQUIPMENT/segments")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let segments: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert_eq!(segments.len(), 1);
    assert_eq!(segments[0]["segment_code"], "asset_tag");

    // GLOBAL context should have no segments
    let resp = app.clone().oneshot(Request::builder()
        .method("GET").uri("/api/v1/flexfields/PO_DFF/contexts/GLOBAL/segments")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let segments: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert!(segments.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Flexfield Data Tests (CRUD + Validation)
// ═══════════════════════════════════════════════════════════════════════════════

async fn setup_full_dff(app: &axum::Router) {
    let (k, v) = auth_header(&admin_claims());

    // Create value set with entries
    app.clone().oneshot(Request::builder()
        .method("POST").uri("/api/v1/flexfields/value-sets")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "VS_DEPT",
            "name": "Departments",
            "validation_type": "independent",
            "data_type": "string",
        })).unwrap())).unwrap()
    ).await.unwrap();

    for dept in &["ENGINEERING", "MARKETING", "FINANCE"] {
        app.clone().oneshot(Request::builder()
            .method("POST").uri("/api/v1/flexfields/value-sets/VS_DEPT/entries")
            .header("Content-Type", "application/json").header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({
                "value": dept,
                "meaning": dept,
            })).unwrap())).unwrap()
        ).await.unwrap();
    }

    // Create flexfield
    app.clone().oneshot(Request::builder()
        .method("POST").uri("/api/v1/flexfields")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "PO_DFF",
            "name": "PO DFF",
            "entity_name": "purchase_orders",
        })).unwrap())).unwrap()
    ).await.unwrap();

    // Create context
    app.clone().oneshot(Request::builder()
        .method("POST").uri("/api/v1/flexfields/PO_DFF/contexts")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "GLOBAL",
            "name": "Global",
            "is_global": true,
        })).unwrap())).unwrap()
    ).await.unwrap();

    // Create segments
    app.clone().oneshot(Request::builder()
        .method("POST").uri("/api/v1/flexfields/PO_DFF/contexts/GLOBAL/segments")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "segment_code": "department",
            "name": "Department",
            "display_order": 1,
            "column_name": "attribute1",
            "data_type": "string",
            "is_required": true,
            "value_set_code": "VS_DEPT",
        })).unwrap())).unwrap()
    ).await.unwrap();

    app.clone().oneshot(Request::builder()
        .method("POST").uri("/api/v1/flexfields/PO_DFF/contexts/GLOBAL/segments")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "segment_code": "cost_center",
            "name": "Cost Center",
            "display_order": 2,
            "column_name": "attribute2",
            "data_type": "string",
            "is_required": false,
        })).unwrap())).unwrap()
    ).await.unwrap();

    app.clone().oneshot(Request::builder()
        .method("POST").uri("/api/v1/flexfields/PO_DFF/contexts/GLOBAL/segments")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "segment_code": "budget_amount",
            "name": "Budget Amount",
            "display_order": 3,
            "column_name": "attribute3",
            "data_type": "number",
            "is_required": false,
        })).unwrap())).unwrap()
    ).await.unwrap();
}

#[tokio::test]
async fn test_set_and_get_flexfield_data() {
    let (_state, app) = setup_dff_test().await;
    setup_full_dff(&app).await;

    let (k, v) = auth_header(&admin_claims());
    let entity_id = uuid::Uuid::new_v4().to_string();

    // Set flexfield data
    let resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri(&format!("/api/v1/flexfields/data/purchase_orders/{}", entity_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "context_code": "GLOBAL",
            "segment_values": {
                "department": "ENGINEERING",
                "cost_center": "CC-100",
                "budget_amount": "50000.00",
            }
        })).unwrap())).unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let data: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(data["context_code"], "GLOBAL");
    assert_eq!(data["segment_values"]["department"], "ENGINEERING");
    assert_eq!(data["segment_values"]["cost_center"], "CC-100");
    assert_eq!(data["segment_values"]["budget_amount"], "50000.00");

    // Get flexfield data
    let resp = app.clone().oneshot(Request::builder()
        .method("GET")
        .uri(&format!("/api/v1/flexfields/data/purchase_orders/{}", entity_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let data_list: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert_eq!(data_list.len(), 1);
    assert_eq!(data_list[0]["context_code"], "GLOBAL");
    assert_eq!(data_list[0]["segment_values"]["department"], "ENGINEERING");
}

#[tokio::test]
async fn test_flexfield_data_update_upsert() {
    let (_state, app) = setup_dff_test().await;
    setup_full_dff(&app).await;

    let (k, v) = auth_header(&admin_claims());
    let entity_id = uuid::Uuid::new_v4().to_string();

    // Set initial data
    app.clone().oneshot(Request::builder()
        .method("POST")
        .uri(&format!("/api/v1/flexfields/data/purchase_orders/{}", entity_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "context_code": "GLOBAL",
            "segment_values": {
                "department": "ENGINEERING",
                "cost_center": "CC-100",
            }
        })).unwrap())).unwrap()
    ).await.unwrap();

    // Update data (same context = upsert)
    let resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri(&format!("/api/v1/flexfields/data/purchase_orders/{}", entity_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "context_code": "GLOBAL",
            "segment_values": {
                "department": "MARKETING",
                "cost_center": "CC-200",
            }
        })).unwrap())).unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    // Verify only one record
    let resp = app.clone().oneshot(Request::builder()
        .method("GET")
        .uri(&format!("/api/v1/flexfields/data/purchase_orders/{}", entity_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let data_list: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert_eq!(data_list.len(), 1);
    assert_eq!(data_list[0]["segment_values"]["department"], "MARKETING");
}

#[tokio::test]
async fn test_flexfield_data_validation_invalid_value_set() {
    let (_state, app) = setup_dff_test().await;
    setup_full_dff(&app).await;

    let (k, v) = auth_header(&admin_claims());
    let entity_id = uuid::Uuid::new_v4().to_string();

    // Set data with invalid department value
    let resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri(&format!("/api/v1/flexfields/data/purchase_orders/{}", entity_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "context_code": "GLOBAL",
            "segment_values": {
                "department": "INVALID_DEPT",
                "cost_center": "CC-100",
            }
        })).unwrap())).unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_flexfield_data_validation_missing_required() {
    let (_state, app) = setup_dff_test().await;
    setup_full_dff(&app).await;

    let (k, v) = auth_header(&admin_claims());
    let entity_id = uuid::Uuid::new_v4().to_string();

    // Set data without required field
    let resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri(&format!("/api/v1/flexfields/data/purchase_orders/{}", entity_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "context_code": "GLOBAL",
            "segment_values": {
                "cost_center": "CC-100",
            }
        })).unwrap())).unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_flexfield_data_validation_invalid_number() {
    let (_state, app) = setup_dff_test().await;
    setup_full_dff(&app).await;

    let (k, v) = auth_header(&admin_claims());
    let entity_id = uuid::Uuid::new_v4().to_string();

    // Set data with non-numeric value in number field
    let resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri(&format!("/api/v1/flexfields/data/purchase_orders/{}", entity_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "context_code": "GLOBAL",
            "segment_values": {
                "department": "ENGINEERING",
                "budget_amount": "not-a-number",
            }
        })).unwrap())).unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_delete_flexfield_data() {
    let (_state, app) = setup_dff_test().await;
    setup_full_dff(&app).await;

    let (k, v) = auth_header(&admin_claims());
    let entity_id = uuid::Uuid::new_v4().to_string();

    // Set data
    app.clone().oneshot(Request::builder()
        .method("POST")
        .uri(&format!("/api/v1/flexfields/data/purchase_orders/{}", entity_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "context_code": "GLOBAL",
            "segment_values": {
                "department": "ENGINEERING",
            }
        })).unwrap())).unwrap()
    ).await.unwrap();

    // Delete
    let resp = app.clone().oneshot(Request::builder()
        .method("DELETE")
        .uri(&format!("/api/v1/flexfields/data/purchase_orders/{}", entity_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    // Verify empty
    let resp = app.clone().oneshot(Request::builder()
        .method("GET")
        .uri(&format!("/api/v1/flexfields/data/purchase_orders/{}", entity_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let data_list: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert!(data_list.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Dashboard Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_dashboard_summary() {
    let (_state, app) = setup_dff_test().await;
    setup_full_dff(&app).await;

    let (k, v) = auth_header(&admin_claims());

    let resp = app.clone().oneshot(Request::builder()
        .method("GET").uri("/api/v1/flexfields/dashboard")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let summary: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(summary["total_flexfields"], 1);
    assert_eq!(summary["active_flexfields"], 1);
    assert_eq!(summary["total_contexts"], 1);
    assert_eq!(summary["total_segments"], 3);
    assert_eq!(summary["total_value_sets"], 1);
    assert!(summary["flexfields_by_entity"]["purchase_orders"].is_number());
}

#[tokio::test]
async fn test_dashboard_empty() {
    let (_state, app) = setup_dff_test().await;

    let (k, v) = auth_header(&admin_claims());

    let resp = app.clone().oneshot(Request::builder()
        .method("GET").uri("/api/v1/flexfields/dashboard")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let summary: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(summary["total_flexfields"], 0);
    assert_eq!(summary["active_flexfields"], 0);
    assert_eq!(summary["total_contexts"], 0);
    assert_eq!(summary["total_segments"], 0);
    assert_eq!(summary["total_value_sets"], 0);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Validation Edge Case Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_value_set_invalid_validation_type_rejected() {
    let (_state, app) = setup_dff_test().await;
    let (k, v) = auth_header(&admin_claims());

    let resp = app.clone().oneshot(Request::builder()
        .method("POST").uri("/api/v1/flexfields/value-sets")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "VS_BAD",
            "name": "Bad Type",
            "validation_type": "invalid_type",
            "data_type": "string",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_value_set_invalid_data_type_rejected() {
    let (_state, app) = setup_dff_test().await;
    let (k, v) = auth_header(&admin_claims());

    let resp = app.clone().oneshot(Request::builder()
        .method("POST").uri("/api/v1/flexfields/value-sets")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "VS_BAD",
            "name": "Bad Data Type",
            "validation_type": "none",
            "data_type": "binary",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_flexfield_empty_code_rejected() {
    let (_state, app) = setup_dff_test().await;
    let (k, v) = auth_header(&admin_claims());

    let resp = app.clone().oneshot(Request::builder()
        .method("POST").uri("/api/v1/flexfields")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "",
            "name": "No Code",
            "entity_name": "test",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_segment_invalid_data_type_rejected() {
    let (_state, app) = setup_dff_test().await;
    setup_flexfield_with_context(&app).await;
    let (k, v) = auth_header(&admin_claims());

    let resp = app.clone().oneshot(Request::builder()
        .method("POST").uri("/api/v1/flexfields/PO_DFF/contexts/GLOBAL/segments")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "segment_code": "bad_seg",
            "name": "Bad Segment",
            "column_name": "attribute1",
            "data_type": "blob",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_segment_nonexistent_value_set_rejected() {
    let (_state, app) = setup_dff_test().await;
    setup_flexfield_with_context(&app).await;
    let (k, v) = auth_header(&admin_claims());

    let resp = app.clone().oneshot(Request::builder()
        .method("POST").uri("/api/v1/flexfields/PO_DFF/contexts/GLOBAL/segments")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "segment_code": "dept",
            "name": "Department",
            "column_name": "attribute1",
            "data_type": "string",
            "value_set_code": "NONEXISTENT_VS",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_flexfield_data_for_missing_flexfield() {
    let (_state, app) = setup_dff_test().await;
    let (k, v) = auth_header(&admin_claims());
    let entity_id = uuid::Uuid::new_v4().to_string();

    // No flexfield defined on "unknown_entity"
    let resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri(&format!("/api/v1/flexfields/data/unknown_entity/{}", entity_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "context_code": "GLOBAL",
            "segment_values": {"field1": "value1"}
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Full Workflow Integration Test
// ═══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_full_dff_workflow() {
    let (_state, app) = setup_dff_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Step 1: Create value set
    let resp = app.clone().oneshot(Request::builder()
        .method("POST").uri("/api/v1/flexfields/value-sets")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "VS_VEHICLE_TYPE",
            "name": "Vehicle Types",
            "validation_type": "independent",
            "data_type": "string",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    // Step 2: Add value set entries
    for vtype in &["SEDAN", "SUV", "TRUCK", "VAN"] {
        let resp = app.clone().oneshot(Request::builder()
            .method("POST").uri("/api/v1/flexfields/value-sets/VS_VEHICLE_TYPE/entries")
            .header("Content-Type", "application/json").header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({
                "value": vtype,
                "meaning": vtype,
            })).unwrap())).unwrap()
        ).await.unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);
    }

    // Step 3: Create flexfield on expense_reports entity
    let resp = app.clone().oneshot(Request::builder()
        .method("POST").uri("/api/v1/flexfields")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "EXPENSE_DFF",
            "name": "Expense Report DFF",
            "entity_name": "expense_reports",
            "default_context_code": "TRAVEL",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    // Step 4: Create TRAVEL context
    let resp = app.clone().oneshot(Request::builder()
        .method("POST").uri("/api/v1/flexfields/EXPENSE_DFF/contexts")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "TRAVEL",
            "name": "Travel Expenses",
            "is_global": false,
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    // Step 5: Add segments
    let resp = app.clone().oneshot(Request::builder()
        .method("POST").uri("/api/v1/flexfields/EXPENSE_DFF/contexts/TRAVEL/segments")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "segment_code": "vehicle_type",
            "name": "Vehicle Type",
            "display_order": 1,
            "column_name": "attribute1",
            "data_type": "string",
            "is_required": true,
            "value_set_code": "VS_VEHICLE_TYPE",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    let resp = app.clone().oneshot(Request::builder()
        .method("POST").uri("/api/v1/flexfields/EXPENSE_DFF/contexts/TRAVEL/segments")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "segment_code": "mileage",
            "name": "Mileage",
            "display_order": 2,
            "column_name": "attribute2",
            "data_type": "number",
            "is_required": false,
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    // Step 6: Set flexfield data for an expense report
    let expense_id = uuid::Uuid::new_v4().to_string();

    // Valid data
    let resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri(&format!("/api/v1/flexfields/data/expense_reports/{}", expense_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "context_code": "TRAVEL",
            "segment_values": {
                "vehicle_type": "SUV",
                "mileage": "150.5",
            }
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Step 7: Read it back
    let resp = app.clone().oneshot(Request::builder()
        .method("GET")
        .uri(&format!("/api/v1/flexfields/data/expense_reports/{}?context_code=TRAVEL", expense_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let data: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert_eq!(data.len(), 1);
    assert_eq!(data[0]["segment_values"]["vehicle_type"], "SUV");
    assert_eq!(data[0]["segment_values"]["mileage"], "150.5");

    // Step 8: Try invalid vehicle type - should fail
    let resp = app.clone().oneshot(Request::builder()
        .method("POST")
        .uri(&format!("/api/v1/flexfields/data/expense_reports/{}", expense_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "context_code": "TRAVEL",
            "segment_values": {
                "vehicle_type": "SPACESHIP",
                "mileage": "999",
            }
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    // Step 9: Check dashboard
    let resp = app.clone().oneshot(Request::builder()
        .method("GET").uri("/api/v1/flexfields/dashboard")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let summary: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(summary["total_flexfields"], 1);
    assert_eq!(summary["total_value_sets"], 1);
    assert_eq!(summary["total_contexts"], 1);
    assert_eq!(summary["total_segments"], 2);
}
