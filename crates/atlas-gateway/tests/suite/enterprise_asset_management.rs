//! Enterprise Asset Management (eAM) E2E Tests
//!
//! Tests for Oracle Fusion Cloud Maintenance Management:
//! - Asset location CRUD
//! - Physical asset definition CRUD & lifecycle
//! - Work order CRUD, status transitions, completion
//! - Preventive maintenance schedule CRUD
//! - Maintenance dashboard summary
//! - Full lifecycle test

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_eam_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    let app = build_router(state.clone());
    (state, app)
}

async fn create_test_location(
    app: &axum::Router, code: &str, name: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/eam/locations")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": code, "name": name, "locationType": "building"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    if status != StatusCode::CREATED {
        panic!("Expected CREATED for location but got {}: {}", status, String::from_utf8_lossy(&b));
    }
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_asset(
    app: &axum::Router, asset_number: &str, name: &str,
    asset_group: &str, criticality: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/eam/assets")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "assetNumber": asset_number,
            "name": name,
            "assetGroup": asset_group,
            "assetCriticality": criticality,
        })).unwrap())).unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    if status != StatusCode::CREATED {
        panic!("Expected CREATED for asset but got {}: {}", status, String::from_utf8_lossy(&b));
    }
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_work_order(
    app: &axum::Router, wo_number: &str, title: &str,
    wo_type: &str, priority: &str, asset_id: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/eam/work-orders")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "workOrderNumber": wo_number,
            "title": title,
            "workOrderType": wo_type,
            "priority": priority,
            "assetId": asset_id,
        })).unwrap())).unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    if status != StatusCode::CREATED {
        panic!("Expected CREATED for work order but got {}: {}", status, String::from_utf8_lossy(&b));
    }
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Location Tests
// ============================================================================

#[tokio::test]
async fn test_create_location() {
    let (_state, app) = setup_eam_test().await;
    let loc = create_test_location(&app, "PLANT-1", "Main Plant").await;
    assert_eq!(loc["code"], "PLANT-1");
    assert_eq!(loc["name"], "Main Plant");
    assert_eq!(loc["locationType"], "building");
}

#[tokio::test]
async fn test_create_location_duplicate_conflict() {
    let (_state, app) = setup_eam_test().await;
    create_test_location(&app, "DUP", "First").await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/eam/locations")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "DUP", "name": "Duplicate"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_list_locations() {
    let (_state, app) = setup_eam_test().await;
    create_test_location(&app, "WH-1", "Warehouse 1").await;
    create_test_location(&app, "WH-2", "Warehouse 2").await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder()
        .uri("/api/v1/eam/locations")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let list: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(list["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_delete_location() {
    let (_state, app) = setup_eam_test().await;
    create_test_location(&app, "DEL", "Delete Me").await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/eam/locations/code/DEL")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Asset Definition Tests
// ============================================================================

#[tokio::test]
async fn test_create_asset() {
    let (_state, app) = setup_eam_test().await;
    let asset = create_test_asset(&app, "PUMP-001", "Main Cooling Pump", "pump", "critical").await;
    assert_eq!(asset["assetNumber"], "PUMP-001");
    assert_eq!(asset["name"], "Main Cooling Pump");
    assert_eq!(asset["assetGroup"], "pump");
    assert_eq!(asset["assetCriticality"], "critical");
    assert_eq!(asset["assetStatus"], "active");
}

#[tokio::test]
async fn test_create_asset_duplicate_conflict() {
    let (_state, app) = setup_eam_test().await;
    create_test_asset(&app, "DUP-A", "First", "motor", "medium").await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/eam/assets")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "assetNumber": "DUP-A", "name": "Duplicate",
            "assetGroup": "motor", "assetCriticality": "medium"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_create_asset_invalid_group() {
    let (_state, app) = setup_eam_test().await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/eam/assets")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "assetNumber": "BAD-G", "name": "Bad Group",
            "assetGroup": "nonexistent", "assetCriticality": "medium"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_asset_invalid_criticality() {
    let (_state, app) = setup_eam_test().await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/eam/assets")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "assetNumber": "BAD-C", "name": "Bad Criticality",
            "assetGroup": "general", "assetCriticality": "super_high"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_asset() {
    let (_state, app) = setup_eam_test().await;
    let asset = create_test_asset(&app, "GET-A", "Get Asset", "hvac", "high").await;
    let id = asset["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder()
        .uri(format!("/api/v1/eam/assets/id/{}", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let fetched: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(fetched["assetNumber"], "GET-A");
}

#[tokio::test]
async fn test_list_assets() {
    let (_state, app) = setup_eam_test().await;
    create_test_asset(&app, "LIST-1", "Asset One", "pump", "medium").await;
    create_test_asset(&app, "LIST-2", "Asset Two", "motor", "high").await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder()
        .uri("/api/v1/eam/assets")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let list: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(list["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_update_asset_status() {
    let (_state, app) = setup_eam_test().await;
    let asset = create_test_asset(&app, "STAT-A", "Status Asset", "general", "low").await;
    let id = asset["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/eam/assets/id/{}/status", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "status": "in_repair"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(updated["assetStatus"], "in_repair");
}

#[tokio::test]
async fn test_delete_asset() {
    let (_state, app) = setup_eam_test().await;
    create_test_asset(&app, "DEL-A", "Delete Asset", "general", "low").await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/eam/assets/number/DEL-A")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Work Order Tests
// ============================================================================

#[tokio::test]
async fn test_create_work_order() {
    let (_state, app) = setup_eam_test().await;
    let asset = create_test_asset(&app, "WO-A1", "WO Asset", "pump", "high").await;
    let asset_id = asset["id"].as_str().unwrap();

    let wo = create_test_work_order(&app, "WO-001", "Fix Pump Bearing", "corrective", "high", asset_id).await;
    assert_eq!(wo["workOrderNumber"], "WO-001");
    assert_eq!(wo["title"], "Fix Pump Bearing");
    assert_eq!(wo["workOrderType"], "corrective");
    assert_eq!(wo["priority"], "high");
    assert_eq!(wo["status"], "draft");
    assert_eq!(wo["assetNumber"], "WO-A1");
}

#[tokio::test]
async fn test_create_work_order_duplicate_conflict() {
    let (_state, app) = setup_eam_test().await;
    let asset = create_test_asset(&app, "DUP-WO-A", "Asset", "general", "medium").await;
    create_test_work_order(&app, "DUP-WO", "First", "corrective", "normal", asset["id"].as_str().unwrap()).await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/eam/work-orders")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "workOrderNumber": "DUP-WO", "title": "Duplicate",
            "workOrderType": "corrective", "priority": "normal",
            "assetId": asset["id"]
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_create_work_order_invalid_type() {
    let (_state, app) = setup_eam_test().await;
    let asset = create_test_asset(&app, "IT-A", "Invalid Type Asset", "general", "low").await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/eam/work-orders")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "workOrderNumber": "IT-WO", "title": "Bad Type",
            "workOrderType": "nonexistent", "priority": "normal",
            "assetId": asset["id"]
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_work_order_invalid_priority() {
    let (_state, app) = setup_eam_test().await;
    let asset = create_test_asset(&app, "IP-A", "Invalid Priority Asset", "general", "low").await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/eam/work-orders")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "workOrderNumber": "IP-WO", "title": "Bad Priority",
            "workOrderType": "corrective", "priority": "immediate",
            "assetId": asset["id"]
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_work_order_status_lifecycle() {
    let (_state, app) = setup_eam_test().await;
    let asset = create_test_asset(&app, "LCYC-A", "Lifecycle Asset", "motor", "high").await;
    let wo = create_test_work_order(&app, "WO-LC", "Lifecycle Test", "preventive", "normal", asset["id"].as_str().unwrap()).await;
    let wo_id = wo["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Draft -> Approved
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/eam/work-orders/id/{}/status", wo_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "approved"})).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(updated["status"], "approved");
    assert!(updated["approvedAt"].is_string());

    // Approved -> In Progress
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/eam/work-orders/id/{}/status", wo_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "in_progress"})).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(updated["status"], "in_progress");
    assert!(updated["actualStart"].is_string());
}

#[tokio::test]
async fn test_complete_work_order() {
    let (_state, app) = setup_eam_test().await;
    let asset = create_test_asset(&app, "COMP-A", "Complete Asset", "hvac", "medium").await;
    let wo = create_test_work_order(&app, "WO-COMP", "Complete Test", "corrective", "high", asset["id"].as_str().unwrap()).await;
    let wo_id = wo["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Start the work order
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/eam/work-orders/id/{}/status", wo_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "in_progress"})).unwrap())).unwrap()
    ).await.unwrap();

    // Complete with actuals
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/eam/work-orders/id/{}/complete", wo_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "actualCost": "1250.00",
            "downtimeHours": 4.5,
            "resolutionCode": "repaired",
            "completionNotes": "Replaced bearing and sealed housing",
            "materials": [{"item": "Bearing 6205", "quantity": 2, "unitCost": "75.00"}],
            "labor": [{"name": "John Mechanic", "hours": 6.0, "rate": "85.00"}]
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let completed: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(completed["status"], "completed");
    assert_eq!(completed["actualCost"], "1250.00");
    assert_eq!(completed["downtimeHours"], 4.5);
    assert_eq!(completed["resolutionCode"], "repaired");
    assert!(completed["actualEnd"].is_string());
    assert!(completed["materials"].is_array());
    assert!(completed["labor"].is_array());
}

#[tokio::test]
async fn test_list_work_orders_filtered() {
    let (_state, app) = setup_eam_test().await;
    let asset = create_test_asset(&app, "FILT-A", "Filter Asset", "general", "medium").await;

    let (k, v) = auth_header(&admin_claims());
    // Create work orders of different types
    for (num, wo_type) in [("WO-F1", "corrective"), ("WO-F2", "preventive")] {
        let _ = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/eam/work-orders")
            .header("Content-Type", "application/json").header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({
                "workOrderNumber": num, "title": num,
                "workOrderType": wo_type, "priority": "normal",
                "assetId": asset["id"]
            })).unwrap())).unwrap()
        ).await.unwrap();
    }

    // Filter by type
    let resp = app.clone().oneshot(Request::builder()
        .uri("/api/v1/eam/work-orders?workOrderType=corrective")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let list: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let wos = list["data"].as_array().unwrap();
    assert!(wos.iter().all(|w| w["workOrderType"] == "corrective"));
}

#[tokio::test]
async fn test_delete_work_order() {
    let (_state, app) = setup_eam_test().await;
    let asset = create_test_asset(&app, "DEL-WO-A", "Delete WO Asset", "general", "low").await;
    create_test_work_order(&app, "WO-DEL", "Delete Me", "corrective", "low", asset["id"].as_str().unwrap()).await;
    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/eam/work-orders/number/WO-DEL")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Preventive Maintenance Schedule Tests
// ============================================================================

#[tokio::test]
async fn test_create_pm_schedule() {
    let (_state, app) = setup_eam_test().await;
    let asset = create_test_asset(&app, "PM-A1", "PM Asset", "compressor", "high").await;
    let asset_id = asset["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/eam/pm-schedules")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "scheduleNumber": "PM-001",
            "name": "Monthly Compressor Inspection",
            "scheduleType": "time_based",
            "frequency": "monthly",
            "intervalValue": 1,
            "intervalUnit": "months",
            "assetId": asset_id,
            "estimatedDurationHours": 4.0,
            "estimatedCost": "500.00",
            "autoGenerate": true,
            "leadTimeDays": 7
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let sched: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(sched["scheduleNumber"], "PM-001");
    assert_eq!(sched["name"], "Monthly Compressor Inspection");
    assert_eq!(sched["scheduleType"], "time_based");
    assert_eq!(sched["frequency"], "monthly");
    assert_eq!(sched["status"], "active");
    assert_eq!(sched["autoGenerate"], true);
    assert_eq!(sched["estimatedDurationHours"], 4.0);
}

#[tokio::test]
async fn test_create_pm_schedule_meter_based() {
    let (_state, app) = setup_eam_test().await;
    let asset = create_test_asset(&app, "PM-MB", "Meter Based Asset", "vehicle", "medium").await;
    let asset_id = asset["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/eam/pm-schedules")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "scheduleNumber": "PM-MB1",
            "name": "Oil Change Every 5000 Miles",
            "scheduleType": "meter_based",
            "frequency": "monthly",
            "intervalValue": 5000,
            "intervalUnit": "miles",
            "meterType": "miles",
            "assetId": asset_id
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let sched: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(sched["scheduleType"], "meter_based");
    assert_eq!(sched["intervalUnit"], "miles");
    assert_eq!(sched["meterType"], "miles");
}

#[tokio::test]
async fn test_create_pm_schedule_duplicate_conflict() {
    let (_state, app) = setup_eam_test().await;
    let asset = create_test_asset(&app, "PM-DUP-A", "Dup Asset", "general", "low").await;
    let (k, v) = auth_header(&admin_claims());
    let body = serde_json::to_string(&json!({
        "scheduleNumber": "PM-DUP", "name": "First",
        "scheduleType": "time_based", "frequency": "monthly",
        "assetId": asset["id"]
    })).unwrap();
    let _ = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/eam/pm-schedules")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(body.clone())).unwrap()
    ).await.unwrap();
    let resp = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/eam/pm-schedules")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(body)).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_update_pm_schedule_status() {
    let (_state, app) = setup_eam_test().await;
    let asset = create_test_asset(&app, "PM-ST-A", "Status Asset", "general", "low").await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/eam/pm-schedules")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "scheduleNumber": "PM-ST", "name": "Status Test",
            "scheduleType": "time_based", "frequency": "monthly",
            "assetId": asset["id"]
        })).unwrap())).unwrap()
    ).await.unwrap();
    let sched: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    let sched_id = sched["id"].as_str().unwrap();

    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/eam/pm-schedules/id/{}/status", sched_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "inactive"})).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(updated["status"], "inactive");
}

#[tokio::test]
async fn test_delete_pm_schedule() {
    let (_state, app) = setup_eam_test().await;
    let asset = create_test_asset(&app, "PM-DEL-A", "Delete Asset", "general", "low").await;
    let (k, v) = auth_header(&admin_claims());
    let _ = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/eam/pm-schedules")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "scheduleNumber": "PM-DEL", "name": "Delete Me",
            "scheduleType": "time_based", "frequency": "monthly",
            "assetId": asset["id"]
        })).unwrap())).unwrap()
    ).await.unwrap();

    let resp = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/eam/pm-schedules/number/PM-DEL")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Dashboard Test
// ============================================================================

#[tokio::test]
async fn test_maintenance_dashboard() {
    let (_state, app) = setup_eam_test().await;

    // Create data
    create_test_asset(&app, "DASH-A1", "Dashboard Pump", "pump", "critical").await;
    create_test_asset(&app, "DASH-A2", "Dashboard Motor", "motor", "medium").await;

    let (k, v) = auth_header(&admin_claims());
    let resp = app.clone().oneshot(Request::builder()
        .uri("/api/v1/eam/dashboard")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let dashboard: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(dashboard["totalAssets"].as_i64().unwrap() >= 2);
    assert!(dashboard["activeAssets"].as_i64().unwrap() >= 2);
    assert!(dashboard["criticalAssets"].as_i64().unwrap() >= 1);
    assert!(dashboard["totalWorkOrders"].as_i64().unwrap() >= 0);
    assert!(dashboard["totalSchedules"].as_i64().unwrap() >= 0);
}

// ============================================================================
// Full Lifecycle Test
// ============================================================================

#[tokio::test]
async fn test_eam_full_lifecycle() {
    let (_state, app) = setup_eam_test().await;
    let (k, v) = auth_header(&admin_claims());

    // 1. Create location
    let _loc = create_test_location(&app, "MAIN-PLANT", "Main Manufacturing Plant").await;

    // 2. Create critical asset
    let asset = create_test_asset(&app, "LIFE-001", "CNC Machine #7", "motor", "critical").await;
    let asset_id = asset["id"].as_str().unwrap();
    assert_eq!(asset["assetStatus"], "active");
    assert_eq!(asset["assetCriticality"], "critical");

    // 3. Create PM schedule for the asset
    let resp = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/eam/pm-schedules")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "scheduleNumber": "PM-LIFE",
            "name": "Quarterly CNC Maintenance",
            "scheduleType": "time_based",
            "frequency": "quarterly",
            "intervalValue": 3,
            "intervalUnit": "months",
            "assetId": asset_id,
            "estimatedDurationHours": 8.0,
            "estimatedCost": "2000.00",
            "autoGenerate": true,
            "leadTimeDays": 14
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let sched: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    assert_eq!(sched["status"], "active");
    assert_eq!(sched["autoGenerate"], true);

    // 4. Report failure - create emergency work order
    let wo = create_test_work_order(&app, "WO-LIFE", "CNC Spindle Vibration", "emergency", "urgent", asset_id).await;
    let wo_id = wo["id"].as_str().unwrap();
    assert_eq!(wo["status"], "draft");
    assert_eq!(wo["workOrderType"], "emergency");
    assert_eq!(wo["priority"], "urgent");

    // 5. Approve work order
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/eam/work-orders/id/{}/status", wo_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "approved"})).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 6. Set asset to in_repair
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/eam/assets/id/{}/status", asset_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "in_repair"})).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let updated_asset: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(updated_asset["assetStatus"], "in_repair");

    // 7. Start work
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/eam/work-orders/id/{}/status", wo_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "in_progress"})).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 8. Complete work order with full details
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/eam/work-orders/id/{}/complete", wo_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "actualCost": "3500.00",
            "downtimeHours": 12.0,
            "resolutionCode": "repaired",
            "completionNotes": "Replaced spindle bearings, recalibrated alignment",
            "failureCode": "vibration",
            "causeCode": "normal_wear",
            "materials": [
                {"item": "Spindle Bearing Set", "quantity": 1, "unitCost": "1200.00"},
                {"item": "Alignment Shims", "quantity": 4, "unitCost": "25.00"}
            ],
            "labor": [
                {"name": "Senior Technician", "hours": 10.0, "rate": "95.00"},
                {"name": "Junior Technician", "hours": 8.0, "rate": "65.00"}
            ]
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let completed: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(completed["status"], "completed");
    assert_eq!(completed["actualCost"], "3500.00");
    assert_eq!(completed["downtimeHours"], 12.0);
    assert_eq!(completed["resolutionCode"], "repaired");

    // 9. Set asset back to active
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/eam/assets/id/{}/status", asset_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "active"})).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 10. Update asset meter reading
    let resp = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/eam/assets/id/{}/meter", asset_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "meterReading": {"type": "hours", "value": 15280, "unit": "hours", "lastRead": "2025-01-15"}
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 11. Verify dashboard reflects full lifecycle
    let resp = app.clone().oneshot(Request::builder()
        .uri("/api/v1/eam/dashboard")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let dashboard: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(dashboard["totalAssets"].as_i64().unwrap() >= 1);
    assert!(dashboard["completedWorkOrders"].as_i64().unwrap() >= 1);
    assert!(dashboard["activeSchedules"].as_i64().unwrap() >= 1);
}
