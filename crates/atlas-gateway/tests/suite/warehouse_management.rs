//! Warehouse Management E2E Tests
//!
//! Tests for Oracle Fusion Cloud Warehouse Management:
//! - Warehouse CRUD (create, get, list, delete)
//! - Warehouse zones (create, list, delete)
//! - Put-away rules (create, list, delete)
//! - Warehouse tasks (create, get, list, start, complete, cancel)
//! - Pick waves (create, get, list, release, complete, cancel)
//! - Validation edge cases
//! - Dashboard

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_warehouse_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    let app = build_router(state.clone());
    (state, app)
}

async fn create_test_warehouse(
    app: &axum::Router,
    code: &str,
    name: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/warehouse/warehouses")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": code,
            "name": name,
            "description": "Test warehouse",
            "locationCode": "LOC-001"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Expected 201 creating warehouse");
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_zone(
    app: &axum::Router,
    warehouse_id: &str,
    code: &str,
    name: &str,
    zone_type: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/warehouse/warehouses/{}/zones", warehouse_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": code,
            "name": name,
            "zoneType": zone_type,
            "aisleCount": 5
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Expected 201 creating zone");
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_wave(
    app: &axum::Router,
    warehouse_id: &str,
    wave_number: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/warehouse/warehouses/{}/waves", warehouse_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "waveNumber": wave_number,
            "priority": "high",
            "shippingMethod": "GROUND"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Expected 201 creating wave");
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Warehouse CRUD Tests
// ============================================================================

#[tokio::test]
async fn test_create_warehouse() {
    let (_state, app) = setup_warehouse_test().await;
    let wh = create_test_warehouse(&app, "WH-001", "Main Warehouse").await;
    assert_eq!(wh["code"], "WH-001");
    assert_eq!(wh["name"], "Main Warehouse");
    assert_eq!(wh["locationCode"], "LOC-001");
    assert!(wh["isActive"].as_bool().unwrap());
    assert!(wh["id"].is_string());
}

#[tokio::test]
async fn test_create_warehouse_duplicate_code_rejected() {
    let (_state, app) = setup_warehouse_test().await;
    create_test_warehouse(&app, "WH-DUP", "First").await;
    // Second with same code should fail
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/warehouse/warehouses")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "WH-DUP", "name": "Second"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_get_warehouse() {
    let (_state, app) = setup_warehouse_test().await;
    let wh = create_test_warehouse(&app, "WH-GET", "GetTest Warehouse").await;
    let wh_id = wh["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/warehouse/warehouses/{}", wh_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let fetched: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(fetched["code"], "WH-GET");
}

#[tokio::test]
async fn test_list_warehouses() {
    let (_state, app) = setup_warehouse_test().await;
    create_test_warehouse(&app, "WH-L1", "List Warehouse 1").await;
    create_test_warehouse(&app, "WH-L2", "List Warehouse 2").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/warehouse/warehouses")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let warehouses: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let arr = warehouses.as_array().unwrap();
    assert!(arr.len() >= 2, "Expected at least 2 warehouses");
}

#[tokio::test]
async fn test_delete_warehouse() {
    let (_state, app) = setup_warehouse_test().await;
    let wh = create_test_warehouse(&app, "WH-DEL", "DeleteTest").await;
    let wh_id = wh["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/warehouse/warehouses/{}/delete", wh_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);

    // Verify it's gone
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/warehouse/warehouses/{}", wh_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

// ============================================================================
// Zone Tests
// ============================================================================

#[tokio::test]
async fn test_create_zone() {
    let (_state, app) = setup_warehouse_test().await;
    let wh = create_test_warehouse(&app, "WH-ZONE", "Zone Warehouse").await;
    let wh_id = wh["id"].as_str().unwrap();

    let zone = create_test_zone(&app, wh_id, "RCV-01", "Receiving Zone", "receiving").await;
    assert_eq!(zone["code"], "RCV-01");
    assert_eq!(zone["zoneType"], "receiving");
    assert_eq!(zone["warehouseId"], wh_id);
}

#[tokio::test]
async fn test_create_zone_invalid_type_rejected() {
    let (_state, app) = setup_warehouse_test().await;
    let wh = create_test_warehouse(&app, "WH-IT", "Invalid Zone Type").await;
    let wh_id = wh["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/warehouse/warehouses/{}/zones", wh_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "BAD", "name": "Bad", "zoneType": "invalid_type"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_list_zones() {
    let (_state, app) = setup_warehouse_test().await;
    let wh = create_test_warehouse(&app, "WH-ZL", "Zone List").await;
    let wh_id = wh["id"].as_str().unwrap();

    create_test_zone(&app, wh_id, "RCV", "Receiving", "receiving").await;
    create_test_zone(&app, wh_id, "STG", "Storage", "storage").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/warehouse/warehouses/{}/zones", wh_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let zones: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(zones.as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_delete_zone() {
    let (_state, app) = setup_warehouse_test().await;
    let wh = create_test_warehouse(&app, "WH-ZD", "Zone Delete").await;
    let wh_id = wh["id"].as_str().unwrap();
    let zone = create_test_zone(&app, wh_id, "TMP", "Temporary", "staging").await;
    let zone_id = zone["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/warehouse/zones/{}", zone_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Put-Away Rule Tests
// ============================================================================

#[tokio::test]
async fn test_create_put_away_rule() {
    let (_state, app) = setup_warehouse_test().await;
    let wh = create_test_warehouse(&app, "WH-PA", "Put Away Rule").await;
    let wh_id = wh["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/warehouse/warehouses/{}/put-away-rules", wh_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "ruleName": "Cold Items",
            "priority": 1,
            "itemCategory": "perishable",
            "targetZoneType": "storage",
            "strategy": "closest"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let rule: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(rule["ruleName"], "Cold Items");
    assert_eq!(rule["strategy"], "closest");
}

#[tokio::test]
async fn test_create_put_away_rule_invalid_strategy_rejected() {
    let (_state, app) = setup_warehouse_test().await;
    let wh = create_test_warehouse(&app, "WH-IS", "Invalid Strategy").await;
    let wh_id = wh["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/warehouse/warehouses/{}/put-away-rules", wh_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "ruleName": "Bad", "targetZoneType": "storage", "strategy": "teleport"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_list_put_away_rules() {
    let (_state, app) = setup_warehouse_test().await;
    let wh = create_test_warehouse(&app, "WH-LR", "List Rules").await;
    let wh_id = wh["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    // Create two rules
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/warehouse/warehouses/{}/put-away-rules", wh_id))
        .header("Content-Type", "application/json").header(&k.clone(), &v.clone())
        .body(Body::from(serde_json::to_string(&json!({
            "ruleName": "R1", "priority": 1, "targetZoneType": "storage", "strategy": "closest"
        })).unwrap())).unwrap()
    ).await.unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/warehouse/warehouses/{}/put-away-rules", wh_id))
        .header("Content-Type", "application/json").header(&k.clone(), &v.clone())
        .body(Body::from(serde_json::to_string(&json!({
            "ruleName": "R2", "priority": 2, "targetZoneType": "picking", "strategy": "fixed_location"
        })).unwrap())).unwrap()
    ).await.unwrap();

    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/warehouse/warehouses/{}/put-away-rules", wh_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let rules: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(rules.as_array().unwrap().len() >= 2);
}

// ============================================================================
// Pick Wave Tests
// ============================================================================

#[tokio::test]
async fn test_create_wave() {
    let (_state, app) = setup_warehouse_test().await;
    let wh = create_test_warehouse(&app, "WH-WV", "Wave Warehouse").await;
    let wh_id = wh["id"].as_str().unwrap();

    let wave = create_test_wave(&app, wh_id, "WAVE-001").await;
    assert_eq!(wave["waveNumber"], "WAVE-001");
    assert_eq!(wave["status"], "draft");
    assert_eq!(wave["priority"], "high");
    assert_eq!(wave["totalTasks"], 0);
}

#[tokio::test]
async fn test_release_and_complete_wave() {
    let (_state, app) = setup_warehouse_test().await;
    let wh = create_test_warehouse(&app, "WH-WLC", "Wave Lifecycle").await;
    let wh_id = wh["id"].as_str().unwrap();
    let wave = create_test_wave(&app, wh_id, "WAVE-LC").await;
    let wave_id = wave["id"].as_str().unwrap();
    assert_eq!(wave["status"], "draft");

    let (k, v) = auth_header(&admin_claims());
    // Release
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/warehouse/waves/{}/release", wave_id))
        .header(&k.clone(), &v.clone()).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let released: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(released["status"], "released");

    // Complete
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/warehouse/waves/{}/complete", wave_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let completed: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(completed["status"], "completed");
}

#[tokio::test]
async fn test_cancel_wave() {
    let (_state, app) = setup_warehouse_test().await;
    let wh = create_test_warehouse(&app, "WH-WCA", "Wave Cancel").await;
    let wh_id = wh["id"].as_str().unwrap();
    let wave = create_test_wave(&app, wh_id, "WAVE-CAN").await;
    let wave_id = wave["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/warehouse/waves/{}/cancel", wave_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let cancelled: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(cancelled["status"], "cancelled");
}

#[tokio::test]
async fn test_cancel_completed_wave_rejected() {
    let (_state, app) = setup_warehouse_test().await;
    let wh = create_test_warehouse(&app, "WH-WCR", "Wave Cancel Reject").await;
    let wh_id = wh["id"].as_str().unwrap();
    let wave = create_test_wave(&app, wh_id, "WAVE-CR").await;
    let wave_id = wave["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    // Release then complete
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/warehouse/waves/{}/release", wave_id))
        .header(&k.clone(), &v.clone()).body(Body::empty()).unwrap()
    ).await.unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/warehouse/waves/{}/complete", wave_id))
        .header(&k.clone(), &v.clone()).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Try to cancel completed wave
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/warehouse/waves/{}/cancel", wave_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_list_waves() {
    let (_state, app) = setup_warehouse_test().await;
    let wh = create_test_warehouse(&app, "WH-WL", "Wave List").await;
    let wh_id = wh["id"].as_str().unwrap();
    create_test_wave(&app, wh_id, "WAVE-L1").await;
    create_test_wave(&app, wh_id, "WAVE-L2").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/warehouse/waves")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let waves: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(waves.as_array().unwrap().len() >= 2);
}

// ============================================================================
// Warehouse Task Tests
// ============================================================================

#[tokio::test]
async fn test_create_task_for_warehouse() {
    let (_state, app) = setup_warehouse_test().await;
    let wh = create_test_warehouse(&app, "WH-TK", "Task Warehouse").await;
    let wh_id = wh["id"].as_str().unwrap();
    let zone = create_test_zone(&app, wh_id, "PK", "Picking", "picking").await;
    let zone_id = zone["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/warehouse/warehouses/{}/tasks", wh_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "taskNumber": "TASK-001",
            "taskType": "pick",
            "priority": "high",
            "toZoneId": zone_id,
            "itemDescription": "Widget A",
            "uom": "EA"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let task: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(task["taskNumber"], "TASK-001");
    assert_eq!(task["taskType"], "pick");
    assert_eq!(task["status"], "pending");
    assert_eq!(task["priority"], "high");
}

#[tokio::test]
async fn test_create_task_invalid_type_rejected() {
    let (_state, app) = setup_warehouse_test().await;
    let wh = create_test_warehouse(&app, "WH-TIT", "Task Invalid Type").await;
    let wh_id = wh["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/warehouse/warehouses/{}/tasks", wh_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "taskNumber": "TASK-BAD", "taskType": "fly", "priority": "medium"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_task_lifecycle_start_complete() {
    let (_state, app) = setup_warehouse_test().await;
    let wh = create_test_warehouse(&app, "WH-TLC", "Task Lifecycle").await;
    let wh_id = wh["id"].as_str().unwrap();
    let zone = create_test_zone(&app, wh_id, "STG", "Storage", "storage").await;
    let zone_id = zone["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    // Create task
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/warehouse/warehouses/{}/tasks", wh_id))
        .header("Content-Type", "application/json").header(&k.clone(), &v.clone())
        .body(Body::from(serde_json::to_string(&json!({
            "taskNumber": "TASK-LC", "taskType": "put_away", "priority": "medium",
            "toZoneId": zone_id
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let task: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let task_id = task["id"].as_str().unwrap();
    assert_eq!(task["status"], "pending");

    // Start task
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/warehouse/tasks/{}/start", task_id))
        .header(&k.clone(), &v.clone()).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let started: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(started["status"], "in_progress");

    // Complete task
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/warehouse/tasks/{}/complete", task_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let completed: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(completed["status"], "completed");
}

#[tokio::test]
async fn test_cancel_task() {
    let (_state, app) = setup_warehouse_test().await;
    let wh = create_test_warehouse(&app, "WH-TC", "Task Cancel").await;
    let wh_id = wh["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/warehouse/warehouses/{}/tasks", wh_id))
        .header("Content-Type", "application/json").header(&k.clone(), &v.clone())
        .body(Body::from(serde_json::to_string(&json!({
            "taskNumber": "TASK-CAN", "taskType": "receive", "priority": "low"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let task: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let task_id = task["id"].as_str().unwrap();

    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/warehouse/tasks/{}/cancel", task_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let cancelled: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(cancelled["status"], "cancelled");
}

#[tokio::test]
async fn test_cannot_complete_pending_task() {
    let (_state, app) = setup_warehouse_test().await;
    let wh = create_test_warehouse(&app, "WH-TCP", "Task Complete Pending").await;
    let wh_id = wh["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/warehouse/warehouses/{}/tasks", wh_id))
        .header("Content-Type", "application/json").header(&k.clone(), &v.clone())
        .body(Body::from(serde_json::to_string(&json!({
            "taskNumber": "TASK-NOC", "taskType": "pick", "priority": "medium"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let task: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let task_id = task["id"].as_str().unwrap();

    // Try to complete without starting first
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/warehouse/tasks/{}/complete", task_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_list_tasks_with_filters() {
    let (_state, app) = setup_warehouse_test().await;
    let wh = create_test_warehouse(&app, "WH-TF", "Task Filters").await;
    let wh_id = wh["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    // Create two tasks of different types
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/warehouse/warehouses/{}/tasks", wh_id))
        .header("Content-Type", "application/json").header(&k.clone(), &v.clone())
        .body(Body::from(serde_json::to_string(&json!({
            "taskNumber": "TASK-F1", "taskType": "pick", "priority": "high"
        })).unwrap())).unwrap()
    ).await.unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/warehouse/warehouses/{}/tasks", wh_id))
        .header("Content-Type", "application/json").header(&k.clone(), &v.clone())
        .body(Body::from(serde_json::to_string(&json!({
            "taskNumber": "TASK-F2", "taskType": "pack", "priority": "low"
        })).unwrap())).unwrap()
    ).await.unwrap();

    // Filter by task_type
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/warehouse/tasks?task_type=pick")
        .header(&k.clone(), &v.clone()).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let tasks: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let arr = tasks.as_array().unwrap();
    assert!(arr.len() >= 1);
    for t in arr {
        assert_eq!(t["taskType"], "pick");
    }

    // Filter by status
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/warehouse/tasks?status=pending")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let tasks: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(tasks.as_array().unwrap().len() >= 2);
}

// ============================================================================
// Wave with Tasks Integration Test
// ============================================================================

#[tokio::test]
async fn test_wave_with_tasks_integration() {
    let (_state, app) = setup_warehouse_test().await;
    let wh = create_test_warehouse(&app, "WH-WTI", "Wave Task Integration").await;
    let wh_id = wh["id"].as_str().unwrap();

    // Create a wave
    let wave = create_test_wave(&app, wh_id, "WAVE-INT").await;
    let wave_id = wave["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    // Create task linked to wave
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/warehouse/warehouses/{}/tasks", wh_id))
        .header("Content-Type", "application/json").header(&k.clone(), &v.clone())
        .body(Body::from(serde_json::to_string(&json!({
            "taskNumber": "TASK-WI", "taskType": "pick", "priority": "high",
            "waveId": wave_id
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let task: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let task_id = task["id"].as_str().unwrap();
    assert_eq!(task["waveId"], wave_id);

    // Check wave has updated total tasks
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/warehouse/waves/{}", wave_id))
        .header(&k.clone(), &v.clone()).body(Body::empty()).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let wave_check: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(wave_check["totalTasks"].as_i64().unwrap() >= 1);

    // Release wave
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/warehouse/waves/{}/release", wave_id))
        .header(&k.clone(), &v.clone()).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Start and complete the task
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/warehouse/tasks/{}/start", task_id))
        .header(&k.clone(), &v.clone()).body(Body::empty()).unwrap()
    ).await.unwrap();

    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/warehouse/tasks/{}/complete", task_id))
        .header(&k.clone(), &v.clone()).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Complete wave
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/warehouse/waves/{}/complete", wave_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}

// ============================================================================
// Dashboard Test
// ============================================================================

#[tokio::test]
async fn test_warehouse_dashboard() {
    let (_state, app) = setup_warehouse_test().await;
    create_test_warehouse(&app, "WH-DB", "Dashboard Warehouse").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/warehouse/dashboard")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let dashboard: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(dashboard["totalWarehouses"].as_i64().unwrap() >= 1);
    assert!(dashboard["activeWarehouses"].as_i64().unwrap() >= 1);
    assert!(dashboard["tasksByType"].is_object());
    assert!(dashboard["tasksByPriority"].is_object());
    assert!(dashboard["recentTasks"].is_array());
}

// ============================================================================
// Delete Task and Wave Tests
// ============================================================================

#[tokio::test]
async fn test_delete_task() {
    let (_state, app) = setup_warehouse_test().await;
    let wh = create_test_warehouse(&app, "WH-DT", "Delete Task").await;
    let wh_id = wh["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/warehouse/warehouses/{}/tasks", wh_id))
        .header("Content-Type", "application/json").header(&k.clone(), &v.clone())
        .body(Body::from(serde_json::to_string(&json!({
            "taskNumber": "TASK-DEL", "taskType": "pack", "priority": "medium"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let task: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let task_id = task["id"].as_str().unwrap();

    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/warehouse/tasks/{}", task_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_delete_wave() {
    let (_state, app) = setup_warehouse_test().await;
    let wh = create_test_warehouse(&app, "WH-DW", "Delete Wave").await;
    let wh_id = wh["id"].as_str().unwrap();
    let wave = create_test_wave(&app, wh_id, "WAVE-DEL").await;
    let wave_id = wave["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/warehouse/waves/{}", wave_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);

    // Verify it's gone
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/warehouse/waves/{}", wave_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}
