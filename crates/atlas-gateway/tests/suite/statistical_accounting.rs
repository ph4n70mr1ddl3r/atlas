//! Statistical Accounting E2E Tests
//!
//! Tests for Oracle Fusion Cloud ERP Statistical Accounting:
//! - Statistical unit CRUD (create, get, list, get by code)
//! - Unit lifecycle (active → inactive → active)
//! - Statistical entry CRUD (create, get, list)
//! - Entry lifecycle (draft → posted → reversed)
//! - Entry validation (inactive unit, zero qty, invalid year/period)
//! - Balance inquiry
//! - Dashboard summary
//! - Validation edge cases

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;
use uuid::Uuid;

async fn setup_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    let migration_sql = include_str!("../../../../migrations/142_statistical_accounting.sql");
    sqlx::raw_sql(migration_sql).execute(&state.db_pool).await.ok();
    let app = build_router(state.clone());
    (state, app)
}

async fn create_unit(
    app: &axum::Router,
    code: &str,
    name: &str,
    stat_type: &str,
    uom: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "code": code,
        "name": name,
        "description": "Test statistical unit",
        "stat_type": stat_type,
        "unit_of_measure": uom,
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/statistical-units")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8_lossy(&b);
    eprintln!("CREATE UNIT RESPONSE status={}: {}", status, body_str);
    assert_eq!(status, StatusCode::CREATED, "Failed to create unit: {:?}", body_str);
    serde_json::from_slice(&b).unwrap()
}

async fn create_entry(
    app: &axum::Router,
    unit_id: Uuid,
    quantity: &str,
    fiscal_year: i32,
    period: i32,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "statistical_unit_id": unit_id.to_string(),
        "account_code": "7000",
        "dimension1": "IT",
        "dimension2": "NYC",
        "fiscal_year": fiscal_year,
        "period_number": period,
        "quantity": quantity,
        "unit_cost": "65000",
        "description": "May headcount",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/statistical-entries")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    eprintln!("CREATE ENTRY RESPONSE status={}: {}", status, String::from_utf8_lossy(&b));
    assert_eq!(status, StatusCode::CREATED, "Failed to create entry");
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Unit CRUD Tests
// ============================================================================

#[tokio::test]
async fn test_create_unit() {
    let (_state, app) = setup_test().await;
    let unit = create_unit(&app, "HEADCOUNT", "Total Headcount", "headcount", "people").await;

    assert_eq!(unit["code"], "HEADCOUNT");
    assert_eq!(unit["name"], "Total Headcount");
    assert_eq!(unit["statType"], "headcount");
    assert_eq!(unit["unitOfMeasure"], "people");
    assert_eq!(unit["isActive"], true);
}

#[tokio::test]
async fn test_create_unit_with_defaults() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "code": "CUSTOM-UOM",
        "name": "Custom Unit",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/statistical-units")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["statType"], "custom");
    assert_eq!(body["unitOfMeasure"], "each");
}

#[tokio::test]
async fn test_get_unit() {
    let (_state, app) = setup_test().await;
    let unit = create_unit(&app, "SQFT", "Square Footage", "square_footage", "sqft").await;
    let unit_id = unit["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/statistical-units/{}", unit_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["code"], "SQFT");
}

#[tokio::test]
async fn test_get_unit_by_code() {
    let (_state, app) = setup_test().await;
    create_unit(&app, "MACH-HRS", "Machine Hours", "machine_hours", "hours").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/statistical-units/code/MACH-HRS")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["code"], "MACH-HRS");
    assert_eq!(body["name"], "Machine Hours");
}

#[tokio::test]
async fn test_get_unit_not_found() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/statistical-units/{}", Uuid::new_v4()))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_list_units() {
    let (_state, app) = setup_test().await;
    create_unit(&app, "LIST-U1", "List Unit 1", "headcount", "people").await;
    create_unit(&app, "LIST-U2", "List Unit 2", "machine_hours", "hours").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/statistical-units")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(body["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_list_units_filter_by_type() {
    let (_state, app) = setup_test().await;
    create_unit(&app, "FILT-HC", "Filter HC", "headcount", "people").await;
    create_unit(&app, "FILT-MH", "Filter MH", "machine_hours", "hours").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/statistical-units?stat_type=headcount")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let units = body["data"].as_array().unwrap();
    assert!(units.iter().all(|u| u["statType"] == "headcount"));
}

#[tokio::test]
async fn test_list_units_filter_by_active() {
    let (_state, app) = setup_test().await;
    let unit = create_unit(&app, "ACT-FILT", "Active Filter", "headcount", "people").await;

    // Deactivate it
    let (k, v) = auth_header(&admin_claims());
    let unit_id = unit["id"].as_str().unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/statistical-units/{}/deactivate", unit_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // List only active
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/statistical-units?is_active=true")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let codes: Vec<_> = body["data"].as_array().unwrap().iter()
        .map(|u| u["code"].as_str().unwrap()).collect();
    assert!(!codes.contains(&"ACT-FILT"));
}

// ============================================================================
// Unit Lifecycle Tests
// ============================================================================

#[tokio::test]
async fn test_deactivate_activate_unit() {
    let (_state, app) = setup_test().await;
    let unit = create_unit(&app, "LC-UNIT", "Lifecycle Unit", "headcount", "people").await;
    let unit_id = unit["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Deactivate
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/statistical-units/{}/deactivate", unit_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["isActive"], false);

    // Reactivate
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/statistical-units/{}/activate", unit_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["isActive"], true);
}

// ============================================================================
// Statistical Entry CRUD Tests
// ============================================================================

#[tokio::test]
async fn test_create_entry() {
    let (_state, app) = setup_test().await;
    let unit = create_unit(&app, "ENT-UNIT", "Entry Unit", "headcount", "people").await;
    let unit_id: Uuid = unit["id"].as_str().unwrap().parse().unwrap();

    let entry = create_entry(&app, unit_id, "150", 2026, 5).await;

    assert_eq!(entry["status"], "draft");
    assert_eq!(entry["fiscalYear"], 2026);
    assert_eq!(entry["periodNumber"], 5);
    assert_eq!(entry["accountCode"], "7000");
    // The quantity field may be numeric or string
    let qty = entry["quantity"].as_str().unwrap_or(entry["quantity"].to_string().as_str());
    assert!(qty.contains("150"), "Expected quantity to contain '150', got: {}", qty);
}

#[tokio::test]
async fn test_get_entry() {
    let (_state, app) = setup_test().await;
    let unit = create_unit(&app, "GET-ENT", "Get Entry Unit", "headcount", "people").await;
    let unit_id: Uuid = unit["id"].as_str().unwrap().parse().unwrap();
    let entry = create_entry(&app, unit_id, "50", 2026, 3).await;
    let entry_id = entry["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/statistical-entries/{}", entry_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["id"], entry_id);
}

#[tokio::test]
async fn test_list_entries() {
    let (_state, app) = setup_test().await;
    let unit = create_unit(&app, "LIST-ENT", "List Entry Unit", "headcount", "people").await;
    let unit_id: Uuid = unit["id"].as_str().unwrap().parse().unwrap();
    create_entry(&app, unit_id, "100", 2026, 1).await;
    create_entry(&app, unit_id, "120", 2026, 2).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/statistical-entries")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(body["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_list_entries_filter_by_unit() {
    let (_state, app) = setup_test().await;
    let unit1 = create_unit(&app, "FILT-U1", "Filter Unit 1", "headcount", "people").await;
    let unit2 = create_unit(&app, "FILT-U2", "Filter Unit 2", "machine_hours", "hours").await;
    let uid1: Uuid = unit1["id"].as_str().unwrap().parse().unwrap();
    let uid2: Uuid = unit2["id"].as_str().unwrap().parse().unwrap();
    create_entry(&app, uid1, "100", 2026, 1).await;
    create_entry(&app, uid2, "200", 2026, 1).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/statistical-entries?unit_id={}", uid1))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let entries = body["data"].as_array().unwrap();
    assert!(entries.iter().all(|e| e["statisticalUnitId"] == uid1.to_string()));
}

#[tokio::test]
async fn test_list_entries_filter_by_status() {
    let (_state, app) = setup_test().await;
    let unit = create_unit(&app, "STAT-FILT", "Status Filter Unit", "headcount", "people").await;
    let unit_id: Uuid = unit["id"].as_str().unwrap().parse().unwrap();
    let e1 = create_entry(&app, unit_id, "100", 2026, 1).await;
    create_entry(&app, unit_id, "200", 2026, 2).await;

    // Post one entry
    let (k, v) = auth_header(&admin_claims());
    let e1_id = e1["id"].as_str().unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/statistical-entries/{}/post", e1_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Filter by posted
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/statistical-entries?status=posted")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(body["data"].as_array().unwrap().len() >= 1);

    // Filter by draft
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/statistical-entries?status=draft")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(body["data"].as_array().unwrap().len() >= 1);
}

#[tokio::test]
async fn test_list_entries_filter_by_fiscal_year() {
    let (_state, app) = setup_test().await;
    let unit = create_unit(&app, "FY-FILT", "FY Filter Unit", "headcount", "people").await;
    let unit_id: Uuid = unit["id"].as_str().unwrap().parse().unwrap();
    create_entry(&app, unit_id, "100", 2025, 12).await;
    create_entry(&app, unit_id, "200", 2026, 1).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/statistical-entries?fiscal_year=2026")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let entries = body["data"].as_array().unwrap();
    assert!(entries.iter().all(|e| e["fiscalYear"] == 2026));
}

// ============================================================================
// Entry Lifecycle Tests
// ============================================================================

#[tokio::test]
async fn test_post_entry() {
    let (_state, app) = setup_test().await;
    let unit = create_unit(&app, "POST-ENT", "Post Entry Unit", "headcount", "people").await;
    let unit_id: Uuid = unit["id"].as_str().unwrap().parse().unwrap();
    let entry = create_entry(&app, unit_id, "75", 2026, 5).await;
    let entry_id = entry["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/statistical-entries/{}/post", entry_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "posted");
}

#[tokio::test]
async fn test_reverse_entry() {
    let (_state, app) = setup_test().await;
    let unit = create_unit(&app, "REV-ENT", "Reverse Entry Unit", "headcount", "people").await;
    let unit_id: Uuid = unit["id"].as_str().unwrap().parse().unwrap();
    let entry = create_entry(&app, unit_id, "50", 2026, 4).await;
    let entry_id = entry["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Post first
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/statistical-entries/{}/post", entry_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Now reverse
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/statistical-entries/{}/reverse", entry_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "reversed");
}

#[tokio::test]
async fn test_full_entry_lifecycle() {
    let (_state, app) = setup_test().await;
    let unit = create_unit(&app, "LC-ENT", "Lifecycle Entry Unit", "headcount", "people").await;
    let unit_id: Uuid = unit["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Create multiple entries
    let e1 = create_entry(&app, unit_id, "100", 2026, 5).await;
    let e2 = create_entry(&app, unit_id, "25", 2026, 5).await;
    let e1_id = e1["id"].as_str().unwrap();
    let e2_id = e2["id"].as_str().unwrap();

    // Post both
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/statistical-entries/{}/post", e1_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/statistical-entries/{}/post", e2_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Reverse one
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/statistical-entries/{}/reverse", e2_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "reversed");

    // Verify we can list by status
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/statistical-entries?status=posted")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    let posted: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(posted["data"].as_array().unwrap().len() >= 1);
}

// ============================================================================
// Validation Edge Cases
// ============================================================================

#[tokio::test]
async fn test_create_unit_empty_code_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "code": "",
        "name": "No Code",
        "stat_type": "headcount",
        "unit_of_measure": "people",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/statistical-units")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_unit_invalid_stat_type_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "code": "BAD-TYPE",
        "name": "Bad Type",
        "stat_type": "nonexistent",
        "unit_of_measure": "people",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/statistical-units")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_unit_duplicate_code_fails() {
    let (_state, app) = setup_test().await;
    create_unit(&app, "DUP-CODE", "First Unit", "headcount", "people").await;

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "code": "DUP-CODE",
        "name": "Duplicate Unit",
        "stat_type": "machine_hours",
        "unit_of_measure": "hours",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/statistical-units")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_create_entry_inactive_unit_fails() {
    let (_state, app) = setup_test().await;
    let unit = create_unit(&app, "INACT-ENT", "Inactive Entry Unit", "headcount", "people").await;
    let unit_id: Uuid = unit["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Deactivate the unit
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/statistical-units/{}/deactivate", unit_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Try to create an entry with inactive unit
    let payload = json!({
        "statistical_unit_id": unit_id.to_string(),
        "fiscal_year": 2026,
        "period_number": 5,
        "quantity": "50",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/statistical-entries")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_entry_zero_quantity_fails() {
    let (_state, app) = setup_test().await;
    let unit = create_unit(&app, "ZERO-Q", "Zero Qty Unit", "headcount", "people").await;
    let unit_id: Uuid = unit["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "statistical_unit_id": unit_id.to_string(),
        "fiscal_year": 2026,
        "period_number": 5,
        "quantity": "0",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/statistical-entries")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_entry_invalid_year_fails() {
    let (_state, app) = setup_test().await;
    let unit = create_unit(&app, "BAD-YR", "Bad Year Unit", "headcount", "people").await;
    let unit_id: Uuid = unit["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "statistical_unit_id": unit_id.to_string(),
        "fiscal_year": 0,
        "period_number": 5,
        "quantity": "50",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/statistical-entries")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_entry_invalid_period_fails() {
    let (_state, app) = setup_test().await;
    let unit = create_unit(&app, "BAD-PD", "Bad Period Unit", "headcount", "people").await;
    let unit_id: Uuid = unit["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "statistical_unit_id": unit_id.to_string(),
        "fiscal_year": 2026,
        "period_number": 14,
        "quantity": "50",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/statistical-entries")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_post_entry_not_draft_fails() {
    let (_state, app) = setup_test().await;
    let unit = create_unit(&app, "DBP-ENT", "Double Post Unit", "headcount", "people").await;
    let unit_id: Uuid = unit["id"].as_str().unwrap().parse().unwrap();
    let entry = create_entry(&app, unit_id, "50", 2026, 5).await;
    let entry_id = entry["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Post once
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/statistical-entries/{}/post", entry_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Try to post again
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/statistical-entries/{}/post", entry_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_reverse_unposted_entry_fails() {
    let (_state, app) = setup_test().await;
    let unit = create_unit(&app, "REV-UNP", "Reverse Unposted Unit", "headcount", "people").await;
    let unit_id: Uuid = unit["id"].as_str().unwrap().parse().unwrap();
    let entry = create_entry(&app, unit_id, "50", 2026, 5).await;
    let entry_id = entry["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/statistical-entries/{}/reverse", entry_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_entry_nonexistent_unit_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "statistical_unit_id": Uuid::new_v4().to_string(),
        "fiscal_year": 2026,
        "period_number": 5,
        "quantity": "50",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/statistical-entries")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_deactivate_already_inactive_fails() {
    let (_state, app) = setup_test().await;
    let unit = create_unit(&app, "DAI-U", "Double Deact Unit", "headcount", "people").await;
    let unit_id = unit["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Deactivate once
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/statistical-units/{}/deactivate", unit_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Try again
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/statistical-units/{}/deactivate", unit_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_activate_already_active_fails() {
    let (_state, app) = setup_test().await;
    let unit = create_unit(&app, "AAI-U", "Already Active Unit", "headcount", "people").await;
    let unit_id = unit["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/statistical-units/{}/activate", unit_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Negative Quantity (valid use case for adjustments/reductions)
// ============================================================================

#[tokio::test]
async fn test_negative_quantity_entry() {
    let (_state, app) = setup_test().await;
    let unit = create_unit(&app, "NEG-Q", "Negative Qty Unit", "headcount", "people").await;
    let unit_id: Uuid = unit["id"].as_str().unwrap().parse().unwrap();

    let entry = create_entry(&app, unit_id, "-5", 2026, 5).await;
    let qty = entry["quantity"].as_str().unwrap_or(entry["quantity"].to_string().as_str());
    assert!(qty.contains("-5"), "Expected negative quantity, got: {}", qty);
}

// ============================================================================
// Dashboard Test
// ============================================================================

#[tokio::test]
async fn test_dashboard() {
    let (_state, app) = setup_test().await;
    create_unit(&app, "DASH-U1", "Dashboard Unit 1", "headcount", "people").await;
    let unit2 = create_unit(&app, "DASH-U2", "Dashboard Unit 2", "machine_hours", "hours").await;
    let uid2: Uuid = unit2["id"].as_str().unwrap().parse().unwrap();
    create_entry(&app, uid2, "500", 2026, 5).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/statistical-accounting/dashboard")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();

    assert!(body.get("totalUnits").is_some());
    assert!(body.get("activeUnits").is_some());
    assert!(body.get("totalEntries").is_some());
    assert!(body.get("postedEntries").is_some());
    assert!(body.get("byType").is_some());

    // Should have at least 2 units
    assert!(body["totalUnits"].as_i64().unwrap() >= 2);
}

// ============================================================================
// Multiple Stat Types Test
// ============================================================================

#[tokio::test]
async fn test_multiple_stat_types() {
    let (_state, app) = setup_test().await;
    let hc = create_unit(&app, "MULTI-HC", "Multi Headcount", "headcount", "people").await;
    let sf = create_unit(&app, "MULTI-SF", "Multi SqFt", "square_footage", "sqft").await;
    let mh = create_unit(&app, "MULTI-MH", "Multi Machine Hours", "machine_hours", "hours").await;

    let hc_id: Uuid = hc["id"].as_str().unwrap().parse().unwrap();
    let sf_id: Uuid = sf["id"].as_str().unwrap().parse().unwrap();
    let mh_id: Uuid = mh["id"].as_str().unwrap().parse().unwrap();

    create_entry(&app, hc_id, "150", 2026, 5).await;
    create_entry(&app, sf_id, "50000", 2026, 5).await;
    create_entry(&app, mh_id, "2000", 2026, 5).await;

    // List all entries
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/statistical-entries")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(body["data"].as_array().unwrap().len() >= 3);

    // Filter by headcount type
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/statistical-units?stat_type=headcount")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let units: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(units["data"].as_array().unwrap().iter().all(|u| u["statType"] == "headcount"));
}
