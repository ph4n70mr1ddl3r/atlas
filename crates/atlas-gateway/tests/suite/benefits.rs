//! Benefits Administration E2E Tests
//!
//! Tests for Oracle Fusion Cloud HCM Benefits Administration:
//! - Benefits plan CRUD
//! - Coverage tier validation
//! - Employee enrollment lifecycle (create → activate → suspend → reactivate → cancel)
//! - Enrollment type validation (open_enrollment, new_hire, life_event)
//! - Deduction generation
//! - Benefits dashboard summary

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_benefits_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    let app = build_router(state.clone());
    (state, app)
}

fn test_coverage_tiers() -> serde_json::Value {
    json!([
        {
            "tierCode": "employee_only",
            "tierName": "Employee Only",
            "employeeCost": "150.00",
            "employerCost": "350.00",
            "totalCost": "500.00"
        },
        {
            "tierCode": "employee_spouse",
            "tierName": "Employee + Spouse",
            "employeeCost": "280.00",
            "employerCost": "420.00",
            "totalCost": "700.00"
        },
        {
            "tierCode": "family",
            "tierName": "Family",
            "employeeCost": "450.00",
            "employerCost": "550.00",
            "totalCost": "1000.00"
        }
    ])
}

async fn create_test_plan(
    app: &axum::Router, code: &str, name: &str, plan_type: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/benefits/plans")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": code,
            "name": name,
            "plan_type": plan_type,
            "coverage_tiers": test_coverage_tiers(),
            "provider_name": "Test Insurance Co",
            "provider_plan_id": "TIC-001",
            "plan_year_start": "2026-01-01",
            "plan_year_end": "2026-12-31",
            "open_enrollment_start": "2025-11-01",
            "open_enrollment_end": "2026-12-31",
            "allow_life_event_changes": true,
            "requires_eoi": false,
            "waiting_period_days": 30,
            "max_dependents": 5
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_enrollment(
    app: &axum::Router, employee_id: &str, plan_code: &str,
    coverage_tier: &str, enrollment_type: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/benefits/enrollments")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "employee_id": employee_id,
            "employee_name": "Test Employee",
            "plan_code": plan_code,
            "coverage_tier": coverage_tier,
            "enrollment_type": enrollment_type,
            "effective_start_date": "2026-01-01",
            "effective_end_date": "2026-12-31",
            "deduction_frequency": "per_pay_period",
            "deduction_account_code": "6010",
            "employer_contribution_account_code": "7010"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Plan Management Tests
// ============================================================================

#[tokio::test]
async fn test_create_benefits_plan() {
    let (_state, app) = setup_benefits_test().await;

    let plan = create_test_plan(&app, "MED-001", "Medical Plan A", "medical").await;
    assert_eq!(plan["code"], "MED-001");
    assert_eq!(plan["name"], "Medical Plan A");
    assert_eq!(plan["planType"], "medical");
    assert_eq!(plan["providerName"], "Test Insurance Co");
    assert_eq!(plan["isActive"], true);
    assert!(plan["id"].is_string());
}

#[tokio::test]
async fn test_create_benefits_plan_dental() {
    let (_state, app) = setup_benefits_test().await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/benefits/plans")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "DENT-001",
            "name": "Dental Plan",
            "plan_type": "dental",
            "coverage_tiers": test_coverage_tiers(),
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let plan: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(plan["planType"], "dental");
}

#[tokio::test]
async fn test_create_plan_invalid_type() {
    let (_state, app) = setup_benefits_test().await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/benefits/plans")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "INVALID",
            "name": "Invalid Plan",
            "plan_type": "nonexistent",
            "coverage_tiers": test_coverage_tiers(),
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_plan_empty_tiers() {
    let (_state, app) = setup_benefits_test().await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/benefits/plans")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "EMPTY",
            "name": "Empty Tiers Plan",
            "plan_type": "medical",
            "coverage_tiers": [],
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_benefits_plan() {
    let (_state, app) = setup_benefits_test().await;

    create_test_plan(&app, "MED-GET", "Medical Get Test", "medical").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/benefits/plans/MED-GET")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let plan: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(plan["code"], "MED-GET");
    assert_eq!(plan["name"], "Medical Get Test");
}

#[tokio::test]
async fn test_list_benefits_plans() {
    let (_state, app) = setup_benefits_test().await;

    create_test_plan(&app, "LIST-MED", "Medical List", "medical").await;
    create_test_plan(&app, "LIST-DEN", "Dental List", "dental").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/benefits/plans")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_list_plans_filtered_by_type() {
    let (_state, app) = setup_benefits_test().await;

    create_test_plan(&app, "FILT-MED", "Medical Filter", "medical").await;
    create_test_plan(&app, "FILT-DEN", "Dental Filter", "dental").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/benefits/plans?plan_type=dental")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let plans = resp["data"].as_array().unwrap();
    assert!(plans.len() >= 1);
    assert!(plans.iter().all(|p| p["planType"] == "dental"));
}

#[tokio::test]
async fn test_delete_benefits_plan() {
    let (_state, app) = setup_benefits_test().await;

    create_test_plan(&app, "DEL-MED", "Medical Delete", "medical").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/benefits/plans/DEL-MED")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);

    // Verify it's gone (inactive plans not returned)
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/benefits/plans/DEL-MED")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

// ============================================================================
// Enrollment Lifecycle Tests
// ============================================================================

#[tokio::test]
async fn test_enrollment_full_lifecycle() {
    let (_state, app) = setup_benefits_test().await;

    // Create plan
    create_test_plan(&app, "LC-MED", "Lifecycle Medical", "medical").await;

    // Create enrollment
    let enrollment = create_test_enrollment(
        &app, "00000000-0000-0000-0000-000000000002", "LC-MED",
        "employee_only", "manual",
    ).await;
    let enrollment_id = enrollment["id"].as_str().unwrap();
    assert_eq!(enrollment["status"], "pending");
    assert_eq!(enrollment["coverageTier"], "employee_only");
    assert_eq!(enrollment["planCode"], "LC-MED");

    // Activate enrollment
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/benefits/enrollments/{}/activate", enrollment_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let activated: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(activated["status"], "active");

    // Suspend enrollment
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/benefits/enrollments/{}/suspend", enrollment_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let suspended: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(suspended["status"], "suspended");

    // Reactivate enrollment
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/benefits/enrollments/{}/reactivate", enrollment_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let reactivated: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(reactivated["status"], "active");

    // Cancel enrollment
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/benefits/enrollments/{}/cancel", enrollment_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "cancellation_reason": "Employee terminated"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let cancelled: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(cancelled["status"], "cancelled");
    assert_eq!(cancelled["cancellationReason"], "Employee terminated");
}

#[tokio::test]
async fn test_enrollment_waive() {
    let (_state, app) = setup_benefits_test().await;

    create_test_plan(&app, "WAV-MED", "Waive Medical", "medical").await;

    let enrollment = create_test_enrollment(
        &app, "00000000-0000-0000-0000-000000000003", "WAV-MED",
        "employee_only", "manual",
    ).await;
    let enrollment_id = enrollment["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/benefits/enrollments/{}/waive", enrollment_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let waived: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(waived["status"], "waived");
}

#[tokio::test]
async fn test_enrollment_duplicate_prevented() {
    let (_state, app) = setup_benefits_test().await;

    create_test_plan(&app, "DUP-MED", "Dup Medical", "medical").await;

    // First enrollment succeeds
    create_test_enrollment(
        &app, "00000000-0000-0000-0000-000000000004", "DUP-MED",
        "employee_only", "manual",
    ).await;

    // Second enrollment for same employee + plan should fail with 409
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/benefits/enrollments")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "employee_id": "00000000-0000-0000-0000-000000000004",
            "employee_name": "Test Employee",
            "plan_code": "DUP-MED",
            "coverage_tier": "employee_only",
            "enrollment_type": "manual",
            "effective_start_date": "2026-01-01",
            "deduction_frequency": "per_pay_period"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_enrollment_invalid_tier() {
    let (_state, app) = setup_benefits_test().await;

    create_test_plan(&app, "TIER-MED", "Tier Medical", "medical").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/benefits/enrollments")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "employee_id": "00000000-0000-0000-0000-000000000005",
            "plan_code": "TIER-MED",
            "coverage_tier": "nonexistent_tier",
            "enrollment_type": "manual",
            "effective_start_date": "2026-01-01",
            "deduction_frequency": "per_pay_period"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_enrollment_nonexistent_plan() {
    let (_state, app) = setup_benefits_test().await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/benefits/enrollments")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "employee_id": "00000000-0000-0000-0000-000000000006",
            "plan_code": "NONEXISTENT",
            "coverage_tier": "employee_only",
            "enrollment_type": "manual",
            "effective_start_date": "2026-01-01",
            "deduction_frequency": "per_pay_period"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_enrollment_invalid_enrollment_type() {
    let (_state, app) = setup_benefits_test().await;

    create_test_plan(&app, "INV-MED", "Invalid Type Medical", "medical").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/benefits/enrollments")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "employee_id": "00000000-0000-0000-0000-000000000007",
            "plan_code": "INV-MED",
            "coverage_tier": "employee_only",
            "enrollment_type": "invalid_type",
            "effective_start_date": "2026-01-01",
            "deduction_frequency": "per_pay_period"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_enrollment_invalid_deduction_frequency() {
    let (_state, app) = setup_benefits_test().await;

    create_test_plan(&app, "FREQ-MED", "Frequency Medical", "medical").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/benefits/enrollments")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "employee_id": "00000000-0000-0000-0000-000000000008",
            "plan_code": "FREQ-MED",
            "coverage_tier": "employee_only",
            "enrollment_type": "manual",
            "effective_start_date": "2026-01-01",
            "deduction_frequency": "invalid"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_activate_non_pending_fails() {
    let (_state, app) = setup_benefits_test().await;

    create_test_plan(&app, "ACT-MED", "Activate Medical", "medical").await;

    let enrollment = create_test_enrollment(
        &app, "00000000-0000-0000-0000-000000000009", "ACT-MED",
        "employee_only", "manual",
    ).await;
    let enrollment_id = enrollment["id"].as_str().unwrap();

    // Activate first time - should work
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/benefits/enrollments/{}/activate", enrollment_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    // Try to activate again - should fail (already active)
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/benefits/enrollments/{}/activate", enrollment_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// List & Filter Tests
// ============================================================================

#[tokio::test]
async fn test_list_enrollments_by_employee() {
    let (_state, app) = setup_benefits_test().await;

    create_test_plan(&app, "LIST-MED1", "List Medical 1", "medical").await;

    let employee_id = "00000000-0000-0000-0000-000000000010";
    create_test_enrollment(&app, employee_id, "LIST-MED1", "employee_only", "manual").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/benefits/enrollments?employee_id={}", employee_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 1);
}

#[tokio::test]
async fn test_list_enrollments_by_status() {
    let (_state, app) = setup_benefits_test().await;

    create_test_plan(&app, "STAT-MED", "Status Medical", "medical").await;

    // Create enrollment (will be pending)
    create_test_enrollment(
        &app, "00000000-0000-0000-0000-000000000011", "STAT-MED",
        "employee_only", "manual",
    ).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/benefits/enrollments?status=pending")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let enrollments = resp["data"].as_array().unwrap();
    assert!(enrollments.len() >= 1);
    assert!(enrollments.iter().all(|e| e["status"] == "pending"));
}

// ============================================================================
// Deduction Tests
// ============================================================================

#[tokio::test]
async fn test_generate_deductions() {
    let (_state, app) = setup_benefits_test().await;

    create_test_plan(&app, "DED-MED", "Deduction Medical", "medical").await;

    // Create and activate enrollment
    let enrollment = create_test_enrollment(
        &app, "00000000-0000-0000-0000-000000000012", "DED-MED",
        "employee_only", "manual",
    ).await;
    let enrollment_id = enrollment["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    // Activate first
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/benefits/enrollments/{}/activate", enrollment_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Generate deductions
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/benefits/deductions/generate")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "pay_period_start": "2026-01-01",
            "pay_period_end": "2026-01-15"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["count"].as_i64().unwrap() >= 1);
    let deductions = resp["data"].as_array().unwrap();
    let ded = &deductions[0];
    assert_eq!(ded["isProcessed"], false);
}

#[tokio::test]
async fn test_list_deductions() {
    let (_state, app) = setup_benefits_test().await;

    create_test_plan(&app, "LD-MED", "List Ded Medical", "medical").await;

    let enrollment = create_test_enrollment(
        &app, "00000000-0000-0000-0000-000000000013", "LD-MED",
        "employee_only", "manual",
    ).await;
    let enrollment_id = enrollment["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/benefits/enrollments/{}/activate", enrollment_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Generate deductions
    app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/benefits/deductions/generate")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "pay_period_start": "2026-02-01",
            "pay_period_end": "2026-02-15"
        })).unwrap())).unwrap()
    ).await.unwrap();

    // List deductions
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/benefits/deductions")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 1);
}

// ============================================================================
// Dashboard Tests
// ============================================================================

#[tokio::test]
async fn test_benefits_dashboard() {
    let (_state, app) = setup_benefits_test().await;

    create_test_plan(&app, "DASH-MED", "Dashboard Medical", "medical").await;

    let enrollment = create_test_enrollment(
        &app, "00000000-0000-0000-0000-000000000014", "DASH-MED",
        "employee_only", "manual",
    ).await;
    let enrollment_id = enrollment["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    // Activate enrollment
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/benefits/enrollments/{}/activate", enrollment_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Get dashboard
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/benefits/dashboard")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let summary: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(summary["totalPlans"].as_i64().unwrap() >= 1);
    assert!(summary["activePlans"].as_i64().unwrap() >= 1);
    assert!(summary["activeEnrollments"].as_i64().unwrap() >= 1);
}

// ============================================================================
// Coverage Tier Cost Extraction Tests
// ============================================================================

#[tokio::test]
async fn test_enrollment_family_tier_costs() {
    let (_state, app) = setup_benefits_test().await;

    create_test_plan(&app, "FAM-MED", "Family Medical", "medical").await;

    let enrollment = create_test_enrollment(
        &app, "00000000-0000-0000-0000-000000000015", "FAM-MED",
        "family", "manual",
    ).await;

    // Family tier: employeeCost should reflect the plan's coverage tier cost
    // The cost may be serialized as number or string depending on DB driver
    let emp_cost = enrollment.get("employeeCost").unwrap();
    let emp_cost_str = if emp_cost.is_string() {
        emp_cost.as_str().unwrap().to_string()
    } else if emp_cost.is_number() {
        format!("{:.2}", emp_cost.as_f64().unwrap_or(0.0))
    } else {
        emp_cost.to_string()
    };
    let parsed: f64 = emp_cost_str.parse().unwrap_or(0.0);
    assert!(parsed > 0.0, "Expected positive cost for family tier, got {} (raw: {:?})", emp_cost_str, emp_cost);
    assert!((parsed - 450.0).abs() < 1.0, "Expected ~450 for family tier, got {}", parsed);
}

// ============================================================================
// Life Event Tests
// ============================================================================

#[tokio::test]
async fn test_life_event_enrollment_requires_reason() {
    let (_state, app) = setup_benefits_test().await;

    create_test_plan(&app, "LE-MED", "Life Event Medical", "medical").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/benefits/enrollments")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "employee_id": "00000000-0000-0000-0000-000000000016",
            "plan_code": "LE-MED",
            "coverage_tier": "employee_only",
            "enrollment_type": "life_event",
            "effective_start_date": "2026-03-01",
            "deduction_frequency": "per_pay_period"
        })).unwrap())).unwrap()
    ).await.unwrap();
    // Should fail because life_event_reason is missing
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_life_event_enrollment_with_reason() {
    let (_state, app) = setup_benefits_test().await;

    create_test_plan(&app, "LER-MED", "Life Event Reason Medical", "medical").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/benefits/enrollments")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "employee_id": "00000000-0000-0000-0000-000000000017",
            "plan_code": "LER-MED",
            "coverage_tier": "employee_spouse",
            "enrollment_type": "life_event",
            "effective_start_date": "2026-03-01",
            "deduction_frequency": "per_pay_period",
            "life_event_reason": "Marriage",
            "life_event_date": "2026-02-14"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let enrollment: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(enrollment["enrollmentType"], "life_event");
    assert_eq!(enrollment["lifeEventReason"], "Marriage");
    assert_eq!(enrollment["coverageTier"], "employee_spouse");
}

// ============================================================================
// Plan Upsert Test
// ============================================================================

#[tokio::test]
async fn test_plan_upsert() {
    let (_state, app) = setup_benefits_test().await;

    // Create initial plan
    let plan1 = create_test_plan(&app, "UPS-MED", "Original Name", "medical").await;
    assert_eq!(plan1["name"], "Original Name");

    // Upsert with same code updates the name
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/benefits/plans")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "UPS-MED",
            "name": "Updated Name",
            "plan_type": "medical",
            "coverage_tiers": test_coverage_tiers(),
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let plan2: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(plan2["name"], "Updated Name");
    // Same ID (upsert)
    assert_eq!(plan1["id"], plan2["id"]);
}
