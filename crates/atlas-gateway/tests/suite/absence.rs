//! Absence Management E2E Tests
//!
//! Tests for Oracle Fusion Cloud HCM Absence Management:
//! - Absence type CRUD
//! - Absence plan with accrual rules
//! - Absence entry lifecycle (create → submit → approve/reject → cancel)
//! - Overlapping entry prevention
//! - Auto-approval below threshold
//! - Balance tracking
//! - Entry history audit trail
//! - Absence management dashboard

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_absence_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    let app = build_router(state.clone());
    (state, app)
}

async fn create_test_type(
    app: &axum::Router, code: &str, name: &str, category: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/absence/types")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": code,
            "name": name,
            "category": category,
            "planType": "accrual",
            "requiresApproval": true,
            "requiresDocumentation": false,
            "autoApproveBelowDays": 1.0,
            "allowNegativeBalance": false,
            "allowHalfDay": true
        })).unwrap())).unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    if status != StatusCode::CREATED {
        let body_str = String::from_utf8_lossy(&b);
        panic!("Expected CREATED but got {}: {}", status, body_str);
    }
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_plan(
    app: &axum::Router, code: &str, name: &str, type_code: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/absence/plans")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": code,
            "name": name,
            "absenceTypeCode": type_code,
            "accrualFrequency": "yearly",
            "accrualRate": 15.0,
            "accrualUnit": "days",
            "carryOverMax": 5.0,
            "carryOverExpiryMonths": 3,
            "maxBalance": 30.0,
            "probationPeriodDays": 90,
            "prorateFirstYear": false
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_entry(
    app: &axum::Router, employee_id: &str, type_code: &str,
    plan_code: Option<&str>, start: &str, end: &str, days: f64,
) -> serde_json::Value {
    let mut body = json!({
        "employeeId": employee_id,
        "employeeName": "Test Employee",
        "absenceTypeCode": type_code,
        "startDate": start,
        "endDate": end,
        "durationDays": days,
        "reason": "Personal time off"
    });
    if let Some(pc) = plan_code {
        body["plan_code"] = json!(pc);
    }
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/absence/entries")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&body).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Absence Type Tests
// ============================================================================

#[tokio::test]
async fn test_create_absence_type() {
    let (_state, app) = setup_absence_test().await;

    let at = create_test_type(&app, "VAC", "Vacation", "vacation").await;
    assert_eq!(at["code"], "VAC");
    assert_eq!(at["name"], "Vacation");
    assert_eq!(at["category"], "vacation");
    assert_eq!(at["planType"], "accrual");
    assert_eq!(at["requiresApproval"], true);
    assert_eq!(at["isActive"], true);
    assert!(at["id"].is_string());
}

#[tokio::test]
async fn test_create_absence_type_sick() {
    let (_state, app) = setup_absence_test().await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/absence/types")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "SICK",
            "name": "Sick Leave",
            "category": "sick",
            "planType": "no_entitlement"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let at: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(at["category"], "sick");
}

#[tokio::test]
async fn test_create_absence_type_invalid_category() {
    let (_state, app) = setup_absence_test().await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/absence/types")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "BAD",
            "name": "Bad Category",
            "category": "nonexistent",
            "planType": "accrual"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_absence_type() {
    let (_state, app) = setup_absence_test().await;

    create_test_type(&app, "GET-VAC", "Get Vacation", "vacation").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/absence/types/GET-VAC")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let at: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(at["code"], "GET-VAC");
}

#[tokio::test]
async fn test_list_absence_types() {
    let (_state, app) = setup_absence_test().await;

    create_test_type(&app, "LIST-VAC", "List Vacation", "vacation").await;
    create_test_type(&app, "LIST-SICK", "List Sick", "sick").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/absence/types")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_list_absence_types_filtered() {
    let (_state, app) = setup_absence_test().await;

    create_test_type(&app, "FILT-VAC", "Filter Vacation", "vacation").await;
    create_test_type(&app, "FILT-SICK", "Filter Sick", "sick").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/absence/types?category=sick")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let types = resp["data"].as_array().unwrap();
    assert!(types.len() >= 1);
    assert!(types.iter().all(|t| t["category"] == "sick"));
}

#[tokio::test]
async fn test_delete_absence_type() {
    let (_state, app) = setup_absence_test().await;

    create_test_type(&app, "DEL-VAC", "Delete Vacation", "vacation").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/absence/types/DEL-VAC")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);

    // Verify it's gone
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/absence/types/DEL-VAC")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

// ============================================================================
// Absence Plan Tests
// ============================================================================

#[tokio::test]
async fn test_create_absence_plan() {
    let (_state, app) = setup_absence_test().await;

    create_test_type(&app, "PLAN-VAC", "Plan Vacation", "vacation").await;

    let plan = create_test_plan(&app, "VAC-PLAN", "Vacation Plan", "PLAN-VAC").await;
    assert_eq!(plan["code"], "VAC-PLAN");
    assert_eq!(plan["name"], "Vacation Plan");
    assert_eq!(plan["accrualFrequency"], "yearly");
    assert_eq!(plan["accrualUnit"], "days");
    assert_eq!(plan["isActive"], true);
}

#[tokio::test]
async fn test_create_plan_nonexistent_type() {
    let (_state, app) = setup_absence_test().await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/absence/plans")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "BAD-PLAN",
            "name": "Bad Plan",
            "absenceTypeCode": "NONEXISTENT",
            "accrualFrequency": "yearly",
            "accrualRate": 15.0
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_absence_plan() {
    let (_state, app) = setup_absence_test().await;

    create_test_type(&app, "GET-PLAN-VAC", "Get Plan Vacation", "vacation").await;
    create_test_plan(&app, "GET-VAC-PLAN", "Get Vacation Plan", "GET-PLAN-VAC").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/absence/plans/GET-VAC-PLAN")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let plan: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(plan["code"], "GET-VAC-PLAN");
}

#[tokio::test]
async fn test_list_absence_plans() {
    let (_state, app) = setup_absence_test().await;

    create_test_type(&app, "LIST-PLAN-VAC", "List Plan Vacation", "vacation").await;
    create_test_plan(&app, "LIST-VAC-PLAN1", "List Plan 1", "LIST-PLAN-VAC").await;
    create_test_plan(&app, "LIST-VAC-PLAN2", "List Plan 2", "LIST-PLAN-VAC").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/absence/plans")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_delete_absence_plan() {
    let (_state, app) = setup_absence_test().await;

    create_test_type(&app, "DEL-PLAN-VAC", "Del Plan Vacation", "vacation").await;
    create_test_plan(&app, "DEL-VAC-PLAN", "Del Vacation Plan", "DEL-PLAN-VAC").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/absence/plans/DEL-VAC-PLAN")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Absence Entry Lifecycle Tests
// ============================================================================

#[tokio::test]
async fn test_entry_full_lifecycle() {
    let (_state, app) = setup_absence_test().await;

    create_test_type(&app, "LC-VAC", "Lifecycle Vacation", "vacation").await;
    create_test_plan(&app, "LC-VAC-PLAN", "Lifecycle Plan", "LC-VAC").await;

    // Create entry (should be draft since requires_approval=true and days > threshold)
    let entry = create_test_entry(
        &app, "00000000-0000-0000-0000-000000000001", "LC-VAC",
        Some("LC-VAC-PLAN"), "2026-06-01", "2026-06-03", 3.0,
    ).await;
    let entry_id = entry["id"].as_str().unwrap();
    assert_eq!(entry["status"], "draft");
    assert_eq!(entry["employeeName"], "Test Employee");
    assert_eq!(entry["reason"], "Personal time off");

    // Submit for approval
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/absence/entries/{}/submit", entry_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let submitted: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(submitted["status"], "submitted");

    // Approve
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/absence/entries/{}/approve", entry_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let approved: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(approved["status"], "approved");
    assert!(approved["approvedBy"].is_string());
    assert!(approved["approvedAt"].is_string());
}

#[tokio::test]
async fn test_entry_reject_lifecycle() {
    let (_state, app) = setup_absence_test().await;

    create_test_type(&app, "REJ-VAC", "Reject Vacation", "vacation").await;

    let entry = create_test_entry(
        &app, "00000000-0000-0000-0000-000000000002", "REJ-VAC",
        None, "2026-07-01", "2026-07-02", 2.0,
    ).await;
    let entry_id = entry["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    // Submit
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/absence/entries/{}/submit", entry_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Reject with reason
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/absence/entries/{}/reject", entry_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "reason": "Insufficient documentation"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let rejected: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(rejected["status"], "rejected");
    assert_eq!(rejected["rejectedReason"], "Insufficient documentation");
}

#[tokio::test]
async fn test_entry_cancel_from_draft() {
    let (_state, app) = setup_absence_test().await;

    create_test_type(&app, "CAN-VAC", "Cancel Vacation", "vacation").await;

    let entry = create_test_entry(
        &app, "00000000-0000-0000-0000-000000000003", "CAN-VAC",
        None, "2026-08-01", "2026-08-02", 2.0,
    ).await;
    let entry_id = entry["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/absence/entries/{}/cancel", entry_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "reason": "Changed plans"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let cancelled: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(cancelled["status"], "cancelled");
    assert_eq!(cancelled["cancelledReason"], "Changed plans");
}

// ============================================================================
// Validation Tests
// ============================================================================

#[tokio::test]
async fn test_entry_invalid_dates() {
    let (_state, app) = setup_absence_test().await;

    create_test_type(&app, "INV-VAC", "Invalid Vacation", "vacation").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/absence/entries")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "employeeId": "00000000-0000-0000-0000-000000000004",
            "absenceTypeCode": "INV-VAC",
            "startDate": "2026-08-10",
            "endDate": "2026-08-05",
            "durationDays": 6.0
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_entry_nonexistent_type() {
    let (_state, app) = setup_absence_test().await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/absence/entries")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "employeeId": "00000000-0000-0000-0000-000000000005",
            "absenceTypeCode": "NONEXISTENT",
            "startDate": "2026-09-01",
            "endDate": "2026-09-02",
            "durationDays": 2.0
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_entry_overlapping_prevented() {
    let (_state, app) = setup_absence_test().await;

    create_test_type(&app, "OVER-VAC", "Overlap Vacation", "vacation").await;

    let emp_id = "00000000-0000-0000-0000-000000000006";

    // First entry succeeds (approved status)
    create_test_entry(
        &app, emp_id, "OVER-VAC", None, "2026-10-01", "2026-10-05", 5.0,
    ).await;

    // Second overlapping entry should fail
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/absence/entries")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "employeeId": emp_id,
            "absenceTypeCode": "OVER-VAC",
            "startDate": "2026-10-03",
            "endDate": "2026-10-07",
            "durationDays": 5.0
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_submit_non_draft_fails() {
    let (_state, app) = setup_absence_test().await;

    create_test_type(&app, "SUB-VAC", "Submit Vacation", "vacation").await;

    let entry = create_test_entry(
        &app, "00000000-0000-0000-0000-000000000007", "SUB-VAC",
        None, "2026-11-01", "2026-11-02", 2.0,
    ).await;
    let entry_id = entry["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    // Submit first time
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/absence/entries/{}/submit", entry_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Submit again should fail
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/absence/entries/{}/submit", entry_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_approve_non_submitted_fails() {
    let (_state, app) = setup_absence_test().await;

    create_test_type(&app, "APR-VAC", "Approve Vacation", "vacation").await;

    let entry = create_test_entry(
        &app, "00000000-0000-0000-0000-000000000008", "APR-VAC",
        None, "2026-12-01", "2026-12-02", 2.0,
    ).await;
    let entry_id = entry["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    // Try to approve a draft - should fail
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/absence/entries/{}/approve", entry_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_cancel_approved_fails() {
    let (_state, app) = setup_absence_test().await;

    create_test_type(&app, "CNA-VAC", "Cancel Approved Vacation", "vacation").await;

    let entry = create_test_entry(
        &app, "00000000-0000-0000-0000-000000000009", "CNA-VAC",
        None, "2026-12-10", "2026-12-12", 3.0,
    ).await;
    let entry_id = entry["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    // Submit and approve
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/absence/entries/{}/submit", entry_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/absence/entries/{}/approve", entry_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Try to cancel approved entry - should fail
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/absence/entries/{}/cancel", entry_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "reason": "Too late"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// List & Filter Tests
// ============================================================================

#[tokio::test]
async fn test_list_entries_by_employee() {
    let (_state, app) = setup_absence_test().await;

    create_test_type(&app, "LIST-VAC", "List Vacation", "vacation").await;

    let emp_id = "00000000-0000-0000-0000-000000000010";
    create_test_entry(
        &app, emp_id, "LIST-VAC", None, "2026-05-01", "2026-05-02", 2.0,
    ).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/absence/entries?employee_id={}", emp_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 1);
}

#[tokio::test]
async fn test_list_entries_by_status() {
    let (_state, app) = setup_absence_test().await;

    create_test_type(&app, "STAT-VAC", "Status Vacation", "vacation").await;

    create_test_entry(
        &app, "00000000-0000-0000-0000-000000000011", "STAT-VAC",
        None, "2026-04-01", "2026-04-02", 2.0,
    ).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/absence/entries?status=draft")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let entries = resp["data"].as_array().unwrap();
    assert!(entries.len() >= 1);
    assert!(entries.iter().all(|e| e["status"] == "draft"));
}

// ============================================================================
// Entry History Test
// ============================================================================

#[tokio::test]
async fn test_entry_history() {
    let (_state, app) = setup_absence_test().await;

    create_test_type(&app, "HIST-VAC", "History Vacation", "vacation").await;

    let entry = create_test_entry(
        &app, "00000000-0000-0000-0000-000000000012", "HIST-VAC",
        None, "2026-03-01", "2026-03-02", 2.0,
    ).await;
    let entry_id = entry["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    // Submit
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/absence/entries/{}/submit", entry_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Check history
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/absence/entries/{}/history", entry_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let history = resp["data"].as_array().unwrap();
    assert!(history.len() >= 2); // create + submit
    assert_eq!(history[0]["action"], "create");
    assert_eq!(history[1]["action"], "submit");
}

// ============================================================================
// Type Upsert Test
// ============================================================================

#[tokio::test]
async fn test_absence_type_upsert() {
    let (_state, app) = setup_absence_test().await;

    let at1 = create_test_type(&app, "UPS-VAC", "Original Name", "vacation").await;
    assert_eq!(at1["name"], "Original Name");

    // Upsert with same code updates the name
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/absence/types")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "UPS-VAC",
            "name": "Updated Name",
            "category": "vacation",
            "planType": "accrual"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let at2: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(at2["name"], "Updated Name");
    // Same ID (upsert)
    assert_eq!(at1["id"], at2["id"]);
}

// ============================================================================
// Dashboard Test
// ============================================================================

#[tokio::test]
async fn test_absence_dashboard() {
    let (_state, app) = setup_absence_test().await;

    create_test_type(&app, "DASH-VAC", "Dashboard Vacation", "vacation").await;
    create_test_plan(&app, "DASH-VAC-PLAN", "Dashboard Plan", "DASH-VAC").await;

    create_test_entry(
        &app, "00000000-0000-0000-0000-000000000013", "DASH-VAC",
        None, "2026-02-01", "2026-02-02", 2.0,
    ).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/absence/dashboard")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let dashboard: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(dashboard["totalTypes"].as_i64().unwrap() >= 1);
    assert!(dashboard["activeTypes"].as_i64().unwrap() >= 1);
    assert!(dashboard["totalPlans"].as_i64().unwrap() >= 1);
}

// ============================================================================
// Balance Test
// ============================================================================

#[tokio::test]
async fn test_get_balance() {
    let (_state, app) = setup_absence_test().await;

    create_test_type(&app, "BAL-VAC", "Balance Vacation", "vacation").await;
    create_test_plan(&app, "BAL-VAC-PLAN", "Balance Plan", "BAL-VAC").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/absence/balances?employee_id=00000000-0000-0000-0000-000000000014&plan_code=BAL-VAC-PLAN")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    if status != StatusCode::OK {
        panic!("Expected OK but got {}: {}", status, String::from_utf8_lossy(&b));
    }
    let balance: serde_json::Value = serde_json::from_slice(&b).unwrap();
    // Accrual rate is 15.0 days/year, so accrued should be 15
    let accrued: f64 = balance["accrued"].as_str().unwrap_or("0").parse().unwrap_or(0.0);
    assert!((accrued - 15.0).abs() < 1.0, "Expected accrued ~15.0, got {}", accrued);
}
