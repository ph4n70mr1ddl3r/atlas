//! Multi-Period Accounting (MPA) E2E Tests
//!
//! Tests for MPA template management, schedule creation, period recognition,
//! reversal, and auto-completion workflows.
//! Oracle Fusion: Financials > General Ledger > Multi-Period Accounting

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    let migration_sql = include_str!("../../../../migrations/150_multi_period_accounting.sql");
    sqlx::raw_sql(migration_sql).execute(&state.db_pool).await.ok();
    let app = build_router(state.clone());
    (state, app)
}

// ========================================================================
// Template CRUD Tests
// ========================================================================

#[tokio::test]
async fn test_create_template() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/mpa/templates")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "template_name": "Insurance 12-Month",
            "description": "Spread insurance over 12 months",
            "distribution_method": "equal",
            "number_of_periods": 12,
            "period_type": "month",
            "deferred_account_code": "1500",
            "expense_account_code": "6000",
            "currency_code": "USD",
        })).unwrap())).unwrap())
    .await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["template_name"], "Insurance 12-Month");
    assert_eq!(d["distribution_method"], "equal");
    assert_eq!(d["number_of_periods"], 12);
    assert_eq!(d["status"], "draft");
}

#[tokio::test]
async fn test_create_template_empty_name_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/mpa/templates")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "template_name": "",
            "distribution_method": "equal",
            "number_of_periods": 12,
        })).unwrap())).unwrap())
    .await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_template_invalid_method_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/mpa/templates")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "template_name": "Bad Template",
            "distribution_method": "random",
        })).unwrap())).unwrap())
    .await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_duplicate_template_name() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let body = serde_json::to_string(&json!({
        "template_name": "UNIQUE-TPL",
        "distribution_method": "equal",
        "number_of_periods": 6,
    })).unwrap();

    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/mpa/templates")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(body.clone()).unwrap()).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);

    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/mpa/templates")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(body).unwrap()).await.unwrap();
    assert_eq!(r.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_list_templates() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/mpa/templates")
        .header(&k, &v).body(Body::empty()).unwrap())
    .await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_template_not_found() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/mpa/templates/{}", uuid::Uuid::new_v4()))
        .header(&k, &v).body(Body::empty()).unwrap())
    .await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_activate_template() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());

    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/mpa/templates")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "template_name": "ACTIVATE-TPL",
            "distribution_method": "equal",
            "number_of_periods": 6,
        })).unwrap())).unwrap())
    .await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let id = d["id"].as_str().unwrap();

    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/mpa/templates/{}/activate", id))
        .header(&k, &v).body(Body::empty()).unwrap())
    .await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["status"], "active");
}

// ========================================================================
// Custom Template Lines
// ========================================================================

#[tokio::test]
async fn test_add_custom_template_line() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());

    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/mpa/templates")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "template_name": "CUSTOM-TPL",
            "distribution_method": "custom",
            "number_of_periods": 3,
        })).unwrap())).unwrap())
    .await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let tpl_id = d["id"].as_str().unwrap();

    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/mpa/templates/{}/lines", tpl_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "period_sequence": 1,
            "percentage": "50.00",
            "offset_days": 0,
        })).unwrap())).unwrap())
    .await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn test_add_line_to_equal_template_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());

    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/mpa/templates")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "template_name": "EQUAL-NO-LINES",
            "distribution_method": "equal",
            "number_of_periods": 12,
        })).unwrap())).unwrap())
    .await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let tpl_id = d["id"].as_str().unwrap();

    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/mpa/templates/{}/lines", tpl_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "period_sequence": 1,
            "percentage": "50.00",
        })).unwrap())).unwrap())
    .await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ========================================================================
// Schedule Tests
// ========================================================================

#[tokio::test]
async fn test_create_schedule() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/mpa/schedules")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "schedule_number": "MPA-E2E-01",
            "description": "Insurance amortization",
            "total_amount": "12000.00",
            "start_date": "2025-01-01",
            "end_date": "2025-12-31",
            "currency_code": "USD",
            "company_code": "ACME",
            "cost_center": "CC01",
            "account_segment": "6000",
        })).unwrap())).unwrap())
    .await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["schedule_number"], "MPA-E2E-01");
    assert_eq!(d["total_amount"], "12000.00");
    assert_eq!(d["status"], "draft");
}

#[tokio::test]
async fn test_create_schedule_negative_amount_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/mpa/schedules")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "schedule_number": "MPA-BAD",
            "total_amount": "-100.00",
            "start_date": "2025-01-01",
        })).unwrap())).unwrap())
    .await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_list_schedules() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/mpa/schedules")
        .header(&k, &v).body(Body::empty()).unwrap())
    .await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_activate_schedule() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());

    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/mpa/schedules")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "schedule_number": "MPA-ACT",
            "total_amount": "5000.00",
            "start_date": "2025-01-01",
        })).unwrap())).unwrap())
    .await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let id = d["id"].as_str().unwrap();

    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/mpa/schedules/{}/activate", id))
        .header(&k, &v).body(Body::empty()).unwrap())
    .await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["status"], "active");
}

// ========================================================================
// Full Workflow Test: Template → Schedule → Activate → Recognize → Complete
// ========================================================================

#[tokio::test]
async fn test_full_workflow() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());

    // 1. Create template
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/mpa/templates")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "template_name": "WF-12M-TEMPLATE",
            "description": "12-month equal distribution",
            "distribution_method": "equal",
            "number_of_periods": 12,
            "period_type": "month",
            "deferred_account_code": "1500",
            "expense_account_code": "6000",
        })).unwrap())).unwrap())
    .await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let tpl_id = d["id"].as_str().unwrap();
    assert_eq!(d["status"], "draft");

    // 2. Activate template
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/mpa/templates/{}/activate", tpl_id))
        .header(&k, &v).body(Body::empty()).unwrap())
    .await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["status"], "active");

    // 3. Create schedule from template
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/mpa/schedules")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "schedule_number": "MPA-WF-001",
            "description": "Annual insurance premium",
            "template_id": tpl_id,
            "total_amount": "12000.00",
            "start_date": "2025-01-01",
            "end_date": "2025-12-31",
            "currency_code": "USD",
            "company_code": "ACME",
        })).unwrap())).unwrap())
    .await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let sched_id = d["id"].as_str().unwrap();
    assert_eq!(d["status"], "draft");
    assert_eq!(d["total_amount"], "12000.00");

    // 4. Verify schedule has 12 lines
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/mpa/schedules/{}/lines", sched_id))
        .header(&k, &v).body(Body::empty()).unwrap())
    .await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let lines = d["data"].as_array().unwrap();
    assert_eq!(lines.len(), 12);
    assert_eq!(lines[0]["amount"], "1000.00");
    assert_eq!(lines[0]["status"], "pending");
    let line1_id = lines[0]["id"].as_str().unwrap();
    let line2_id = lines[1]["id"].as_str().unwrap();

    // 5. Activate schedule
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/mpa/schedules/{}/activate", sched_id))
        .header(&k, &v).body(Body::empty()).unwrap())
    .await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    // 6. Recognize first line
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/mpa/lines/{}/recognize", line1_id))
        .header(&k, &v).body(Body::empty()).unwrap())
    .await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["status"], "recognized");
    assert!(d["journal_entry_id"].is_string());

    // 7. Verify schedule amounts updated
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/mpa/schedules/{}", sched_id))
        .header(&k, &v).body(Body::empty()).unwrap())
    .await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["recognized_amount"], "1000.00");
    assert_eq!(d["remaining_amount"], "11000.00");
    assert_eq!(d["status"], "active"); // Not completed yet

    // 8. Reverse the first line
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/mpa/lines/{}/reverse", line1_id))
        .header(&k, &v).body(Body::empty()).unwrap())
    .await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["status"], "reversed");

    // 9. Verify amounts after reversal
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/mpa/schedules/{}", sched_id))
        .header(&k, &v).body(Body::empty()).unwrap())
    .await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["recognized_amount"], "0.00");
    assert_eq!(d["remaining_amount"], "12000.00");

    // 10. Hold schedule
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/mpa/schedules/{}/hold", sched_id))
        .header(&k, &v).body(Body::empty()).unwrap())
    .await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["status"], "on_hold");

    // 11. Reactivate
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/mpa/schedules/{}/activate", sched_id))
        .header(&k, &v).body(Body::empty()).unwrap())
    .await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["status"], "active");
}

#[tokio::test]
async fn test_auto_complete_schedule() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Create a 2-period template
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/mpa/templates")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "template_name": "AUTO-COMP-TPL",
            "distribution_method": "equal",
            "number_of_periods": 2,
            "period_type": "month",
        })).unwrap())).unwrap())
    .await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let tpl_id = d["id"].as_str().unwrap();

    // Activate template
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/mpa/templates/{}/activate", tpl_id))
        .header(&k, &v).body(Body::empty()).unwrap()).await.unwrap();

    // Create schedule
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/mpa/schedules")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "schedule_number": "MPA-AUTO-COMP",
            "template_id": tpl_id,
            "total_amount": "2000.00",
            "start_date": "2025-01-01",
        })).unwrap())).unwrap())
    .await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let sched_id = d["id"].as_str().unwrap();

    // Activate schedule
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/mpa/schedules/{}/activate", sched_id))
        .header(&k, &v).body(Body::empty()).unwrap()).await.unwrap();

    // Get lines
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/mpa/schedules/{}/lines", sched_id))
        .header(&k, &v).body(Body::empty()).unwrap())
    .await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let lines = d["data"].as_array().unwrap();
    assert_eq!(lines.len(), 2);

    // Recognize both lines
    for line in lines {
        let r = app.clone().oneshot(Request::builder().method("POST")
            .uri(&format!("/api/v1/mpa/lines/{}/recognize", line["id"].as_str().unwrap()))
            .header(&k, &v).body(Body::empty()).unwrap())
        .await.unwrap();
        assert_eq!(r.status(), StatusCode::OK);
    }

    // Verify schedule is auto-completed
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/mpa/schedules/{}", sched_id))
        .header(&k, &v).body(Body::empty()).unwrap())
    .await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["status"], "completed");
    assert_eq!(d["recognized_amount"], "2000.00");
    assert_eq!(d["remaining_amount"], "0.00");
}

#[tokio::test]
async fn test_recognize_on_draft_schedule_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Create schedule (stays in draft)
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/mpa/schedules")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "schedule_number": "MPA-DRAFT-REC",
            "total_amount": "1000.00",
            "start_date": "2025-01-01",
        })).unwrap())).unwrap())
    .await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let sched_id = d["id"].as_str().unwrap();

    // Get lines
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/mpa/schedules/{}/lines", sched_id))
        .header(&k, &v).body(Body::empty()).unwrap())
    .await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let lines = d["data"].as_array().unwrap();

    if !lines.is_empty() {
        // Try to recognize on draft schedule
        let r = app.clone().oneshot(Request::builder().method("POST")
            .uri(&format!("/api/v1/mpa/lines/{}/recognize", lines[0]["id"].as_str().unwrap()))
            .header(&k, &v).body(Body::empty()).unwrap())
        .await.unwrap();
        assert_eq!(r.status(), StatusCode::BAD_REQUEST);
    }
}

#[tokio::test]
async fn test_cancel_schedule() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());

    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/mpa/schedules")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "schedule_number": "MPA-CANCEL",
            "total_amount": "5000.00",
            "start_date": "2025-01-01",
        })).unwrap())).unwrap())
    .await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let sched_id = d["id"].as_str().unwrap();

    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/mpa/schedules/{}/cancel", sched_id))
        .header(&k, &v).body(Body::empty()).unwrap())
    .await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["status"], "cancelled");
}

// ========================================================================
// Dashboard Test
// ========================================================================

#[tokio::test]
async fn test_get_dashboard() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/mpa/dashboard")
        .header(&k, &v).body(Body::empty()).unwrap())
    .await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["total_templates"], 0);
    assert_eq!(d["total_schedules"], 0);
}
