//! Time and Labor Management E2E Tests
//!
//! Tests for Oracle Fusion Cloud HCM Time and Labor:
//! - Work schedule CRUD
//! - Overtime rule CRUD
//! - Time card lifecycle (create → add entries → submit → approve/reject → cancel)
//! - Time entry validation (period bounds, positive duration, card status)
//! - Labor distribution
//! - Time card history audit trail
//! - Time and Labor dashboard

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_tl_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    let app = build_router(state.clone());
    (state, app)
}

async fn create_test_schedule(
    app: &axum::Router, code: &str, name: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/time-and-labor/schedules")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": code,
            "name": name,
            "scheduleType": "fixed",
            "standardHoursPerDay": 8.0,
            "standardHoursPerWeek": 40.0,
            "workDaysPerWeek": 5,
            "breakDurationMinutes": 60
        })).unwrap())).unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    if status != StatusCode::CREATED {
        panic!("Expected CREATED but got {}: {}", status, String::from_utf8_lossy(&b));
    }
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_overtime_rule(
    app: &axum::Router, code: &str, name: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/time-and-labor/overtime-rules")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": code,
            "name": name,
            "thresholdType": "weekly",
            "dailyThresholdHours": 8.0,
            "weeklyThresholdHours": 40.0,
            "overtimeMultiplier": 1.5,
            "doubleTimeMultiplier": 2.0
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_time_card(
    app: &axum::Router, employee_id: &str, start: &str, end: &str,
    schedule_code: Option<&str>, overtime_rule_code: Option<&str>,
) -> serde_json::Value {
    let mut body = json!({
        "employeeId": employee_id,
        "employeeName": "Test Employee",
        "periodStart": start,
        "periodEnd": end
    });
    if let Some(sc) = schedule_code {
        body["scheduleCode"] = json!(sc);
    }
    if let Some(oc) = overtime_rule_code {
        body["overtimeRuleCode"] = json!(oc);
    }
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/time-and-labor/time-cards")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&body).unwrap())).unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    if status != StatusCode::CREATED {
        panic!("Expected CREATED but got {}: {}", status, String::from_utf8_lossy(&b));
    }
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_time_entry(
    app: &axum::Router, time_card_id: &str, date: &str, hours: f64, entry_type: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/time-and-labor/entries")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "timeCardId": time_card_id,
            "entryDate": date,
            "entryType": entry_type,
            "durationHours": hours,
            "projectName": "Project Alpha"
        })).unwrap())).unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    if status != StatusCode::CREATED {
        panic!("Expected CREATED but got {}: {}", status, String::from_utf8_lossy(&b));
    }
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Work Schedule Tests
// ============================================================================

#[tokio::test]
async fn test_create_work_schedule() {
    let (_state, app) = setup_tl_test().await;
    let s = create_test_schedule(&app, "STD", "Standard 40h").await;
    assert_eq!(s["code"], "STD");
    assert_eq!(s["name"], "Standard 40h");
    assert_eq!(s["scheduleType"], "fixed");
    assert_eq!(s["standardHoursPerDay"], "8.00");
    assert_eq!(s["standardHoursPerWeek"], "40.00");
    assert!(s["id"].is_string());
    assert_eq!(s["isActive"], true);
}

#[tokio::test]
async fn test_get_work_schedule() {
    let (_state, app) = setup_tl_test().await;
    create_test_schedule(&app, "GET-STD", "Get Standard").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/time-and-labor/schedules/GET-STD")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let s: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(s["code"], "GET-STD");
}

#[tokio::test]
async fn test_list_work_schedules() {
    let (_state, app) = setup_tl_test().await;
    create_test_schedule(&app, "LIST-STD1", "List Schedule 1").await;
    create_test_schedule(&app, "LIST-STD2", "List Schedule 2").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/time-and-labor/schedules")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_delete_work_schedule() {
    let (_state, app) = setup_tl_test().await;
    create_test_schedule(&app, "DEL-STD", "Delete Schedule").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/time-and-labor/schedules/DEL-STD")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);

    // Verify it's gone
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/time-and-labor/schedules/DEL-STD")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_create_schedule_invalid_type() {
    let (_state, app) = setup_tl_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/time-and-labor/schedules")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "BAD", "name": "Bad Type", "scheduleType": "nonexistent"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Overtime Rule Tests
// ============================================================================

#[tokio::test]
async fn test_create_overtime_rule() {
    let (_state, app) = setup_tl_test().await;
    let rule = create_test_overtime_rule(&app, "OT-STD", "Standard OT").await;
    assert_eq!(rule["code"], "OT-STD");
    assert_eq!(rule["thresholdType"], "weekly");
    assert_eq!(rule["overtimeMultiplier"], "1.5000");
    assert_eq!(rule["isActive"], true);
}

#[tokio::test]
async fn test_list_overtime_rules() {
    let (_state, app) = setup_tl_test().await;
    create_test_overtime_rule(&app, "OT-LIST1", "OT List 1").await;
    create_test_overtime_rule(&app, "OT-LIST2", "OT List 2").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/time-and-labor/overtime-rules")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_delete_overtime_rule() {
    let (_state, app) = setup_tl_test().await;
    create_test_overtime_rule(&app, "OT-DEL", "OT Delete").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/time-and-labor/overtime-rules/OT-DEL")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Time Card Lifecycle Tests
// ============================================================================

#[tokio::test]
async fn test_time_card_full_lifecycle() {
    let (_state, app) = setup_tl_test().await;
    create_test_schedule(&app, "LC-STD", "Lifecycle Schedule").await;
    create_test_overtime_rule(&app, "LC-OT", "Lifecycle OT").await;

    let emp_id = "00000000-0000-0000-0000-000000000001";

    // Create time card
    let card = create_test_time_card(
        &app, emp_id, "2026-06-01", "2026-06-07",
        Some("LC-STD"), Some("LC-OT"),
    ).await;
    let card_id = card["id"].as_str().unwrap();
    assert_eq!(card["status"], "draft");
    assert_eq!(card["employeeName"], "Test Employee");
    assert_eq!(card["totalHours"], "0.0000");
    assert!(card["cardNumber"].as_str().unwrap().starts_with("TC-"));

    // Add entries
    let entry1 = create_test_time_entry(&app, card_id, "2026-06-01", 8.0, "regular").await;
    assert_eq!(entry1["entryType"], "regular");
    assert_eq!(entry1["durationHours"], "8.0000");

    let entry2 = create_test_time_entry(&app, card_id, "2026-06-02", 2.0, "overtime").await;
    assert_eq!(entry2["entryType"], "overtime");

    // Verify totals updated
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/time-and-labor/time-cards/{}", card_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let updated_card: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(updated_card["totalRegularHours"], "8.0000");
    assert_eq!(updated_card["totalOvertimeHours"], "2.0000");
    assert_eq!(updated_card["totalHours"], "10.0000");

    // Submit for approval
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/time-and-labor/time-cards/{}/submit", card_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    // Approve
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/time-and-labor/time-cards/{}/approve", card_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let approved: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(approved["status"], "approved");
    assert!(approved["approvedBy"].is_string());
    assert!(approved["approvedAt"].is_string());
}

#[tokio::test]
async fn test_time_card_reject_lifecycle() {
    let (_state, app) = setup_tl_test().await;

    let emp_id = "00000000-0000-0000-0000-000000000002";
    let card = create_test_time_card(&app, emp_id, "2026-07-01", "2026-07-07", None, None).await;
    let card_id = card["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    // Submit
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/time-and-labor/time-cards/{}/submit", card_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Reject with reason
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/time-and-labor/time-cards/{}/reject", card_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "reason": "Missing project code"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let rejected: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(rejected["status"], "rejected");
    assert_eq!(rejected["rejectedReason"], "Missing project code");
}

#[tokio::test]
async fn test_time_card_cancel_from_draft() {
    let (_state, app) = setup_tl_test().await;

    let emp_id = "00000000-0000-0000-0000-000000000003";
    let card = create_test_time_card(&app, emp_id, "2026-08-01", "2026-08-07", None, None).await;
    let card_id = card["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/time-and-labor/time-cards/{}/cancel", card_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "reason": "Mistake"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let cancelled: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(cancelled["status"], "cancelled");
}

// ============================================================================
// Validation Tests
// ============================================================================

#[tokio::test]
async fn test_time_card_invalid_dates() {
    let (_state, app) = setup_tl_test().await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/time-and-labor/time-cards")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "employeeId": "00000000-0000-0000-0000-000000000004",
            "periodStart": "2026-09-07",
            "periodEnd": "2026-09-01"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_entry_outside_period() {
    let (_state, app) = setup_tl_test().await;

    let emp_id = "00000000-0000-0000-0000-000000000005";
    let card = create_test_time_card(&app, emp_id, "2026-10-01", "2026-10-07", None, None).await;
    let card_id = card["id"].as_str().unwrap();

    // Entry date outside the card period
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/time-and-labor/entries")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "timeCardId": card_id,
            "entryDate": "2026-10-15",
            "entryType": "regular",
            "durationHours": 8.0
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_entry_invalid_type() {
    let (_state, app) = setup_tl_test().await;

    let emp_id = "00000000-0000-0000-0000-000000000006";
    let card = create_test_time_card(&app, emp_id, "2026-11-01", "2026-11-07", None, None).await;
    let card_id = card["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/time-and-labor/entries")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "timeCardId": card_id,
            "entryDate": "2026-11-02",
            "entryType": "nonexistent",
            "durationHours": 8.0
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_submit_non_draft_fails() {
    let (_state, app) = setup_tl_test().await;

    let emp_id = "00000000-0000-0000-0000-000000000007";
    let card = create_test_time_card(&app, emp_id, "2026-12-01", "2026-12-07", None, None).await;
    let card_id = card["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    // Submit first time
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/time-and-labor/time-cards/{}/submit", card_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Submit again should fail
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/time-and-labor/time-cards/{}/submit", card_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_approve_non_submitted_fails() {
    let (_state, app) = setup_tl_test().await;

    let emp_id = "00000000-0000-0000-0000-000000000008";
    let card = create_test_time_card(&app, emp_id, "2027-01-01", "2027-01-07", None, None).await;
    let card_id = card["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    // Try to approve a draft
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/time-and-labor/time-cards/{}/approve", card_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_cancel_approved_fails() {
    let (_state, app) = setup_tl_test().await;

    let emp_id = "00000000-0000-0000-0000-000000000009";
    let card = create_test_time_card(&app, emp_id, "2027-02-01", "2027-02-07", None, None).await;
    let card_id = card["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    // Submit and approve
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/time-and-labor/time-cards/{}/submit", card_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/time-and-labor/time-cards/{}/approve", card_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Cancel approved should fail
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/time-and-labor/time-cards/{}/cancel", card_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"reason": "Too late"})).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_add_entry_to_submitted_card_fails() {
    let (_state, app) = setup_tl_test().await;

    let emp_id = "00000000-0000-0000-0000-000000000010";
    let card = create_test_time_card(&app, emp_id, "2027-03-01", "2027-03-07", None, None).await;
    let card_id = card["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    // Submit
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/time-and-labor/time-cards/{}/submit", card_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Try to add entry to submitted card
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/time-and-labor/entries")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "timeCardId": card_id,
            "entryDate": "2027-03-01",
            "entryType": "regular",
            "durationHours": 8.0
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// List & Filter Tests
// ============================================================================

#[tokio::test]
async fn test_list_time_cards_by_employee() {
    let (_state, app) = setup_tl_test().await;

    let emp_id = "00000000-0000-0000-0000-000000000011";
    create_test_time_card(&app, emp_id, "2027-04-01", "2027-04-07", None, None).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/time-and-labor/time-cards?employee_id={}", emp_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 1);
}

#[tokio::test]
async fn test_list_time_cards_by_status() {
    let (_state, app) = setup_tl_test().await;

    let emp_id = "00000000-0000-0000-0000-000000000012";
    create_test_time_card(&app, emp_id, "2027-05-01", "2027-05-07", None, None).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/time-and-labor/time-cards?status=draft")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let cards = resp["data"].as_array().unwrap();
    assert!(cards.len() >= 1);
    assert!(cards.iter().all(|c| c["status"] == "draft"));
}

// ============================================================================
// Time Entries List Test
// ============================================================================

#[tokio::test]
async fn test_list_time_entries_for_card() {
    let (_state, app) = setup_tl_test().await;

    let emp_id = "00000000-0000-0000-0000-000000000013";
    let card = create_test_time_card(&app, emp_id, "2027-06-01", "2027-06-07", None, None).await;
    let card_id = card["id"].as_str().unwrap();

    create_test_time_entry(&app, card_id, "2027-06-01", 8.0, "regular").await;
    create_test_time_entry(&app, card_id, "2027-06-02", 2.0, "overtime").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/time-and-labor/entries/time-card/{}", card_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 2);
}

// ============================================================================
// Entry History Test
// ============================================================================

#[tokio::test]
async fn test_time_card_history() {
    let (_state, app) = setup_tl_test().await;

    let emp_id = "00000000-0000-0000-0000-000000000014";
    let card = create_test_time_card(&app, emp_id, "2027-07-01", "2027-07-07", None, None).await;
    let card_id = card["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    // Submit
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/time-and-labor/time-cards/{}/submit", card_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Check history
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/time-and-labor/time-cards/{}/history", card_id))
        .header(&k, &v).body(Body::empty()).unwrap()
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
// Upsert Test
// ============================================================================

#[tokio::test]
async fn test_work_schedule_upsert() {
    let (_state, app) = setup_tl_test().await;

    let s1 = create_test_schedule(&app, "UPS-STD", "Original Name").await;
    assert_eq!(s1["name"], "Original Name");

    // Upsert with same code updates the name
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/time-and-labor/schedules")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "UPS-STD",
            "name": "Updated Name",
            "scheduleType": "flexible"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let s2: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(s2["name"], "Updated Name");
    // Same ID (upsert)
    assert_eq!(s1["id"], s2["id"]);
}

// ============================================================================
// Dashboard Test
// ============================================================================

#[tokio::test]
async fn test_time_and_labor_dashboard() {
    let (_state, app) = setup_tl_test().await;
    create_test_schedule(&app, "DASH-STD", "Dashboard Schedule").await;
    create_test_overtime_rule(&app, "DASH-OT", "Dashboard OT").await;

    let emp_id = "00000000-0000-0000-0000-000000000015";
    let card = create_test_time_card(&app, emp_id, "2027-08-01", "2027-08-07", None, None).await;
    create_test_time_entry(&app, card["id"].as_str().unwrap(), "2027-08-01", 8.0, "regular").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/time-and-labor/dashboard")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let dashboard: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(dashboard["totalSchedules"].as_i64().unwrap() >= 1);
    assert!(dashboard["activeSchedules"].as_i64().unwrap() >= 1);
    assert!(dashboard["totalOvertimeRules"].as_i64().unwrap() >= 1);
    assert!(dashboard["totalTimeCards"].as_i64().unwrap() >= 1);
}

// ============================================================================
// Labor Distribution Test
// ============================================================================

#[tokio::test]
async fn test_create_labor_distribution() {
    let (_state, app) = setup_tl_test().await;

    let emp_id = "00000000-0000-0000-0000-000000000016";
    let card = create_test_time_card(&app, emp_id, "2027-09-01", "2027-09-07", None, None).await;
    let entry = create_test_time_entry(&app, card["id"].as_str().unwrap(), "2027-09-01", 8.0, "regular").await;
    let entry_id = entry["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/time-and-labor/distributions")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "timeEntryId": entry_id,
            "distributionPercent": 60.0,
            "costCenter": "CC-001",
            "projectName": "Project Alpha"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let dist: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(dist["distributionPercent"], "60.00");
    assert_eq!(dist["costCenter"], "CC-001");

    // Verify allocated hours: 8.0 * 0.6 = 4.8
    let allocated: f64 = dist["allocatedHours"].as_str().unwrap().parse().unwrap();
    assert!((allocated - 4.8).abs() < 0.01, "Expected allocated ~4.8, got {}", allocated);

    // List distributions for the entry
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/time-and-labor/distributions/entry/{}", entry_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 1);
}
