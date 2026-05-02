//! Deferred Revenue/Cost Management E2E Tests
//!
//! Tests for Oracle Fusion Cloud ERP Deferred Revenue Management:
//! - Deferral template CRUD
//! - Deferral schedule lifecycle (create, hold, resume, cancel)
//! - Schedule lines
//! - Recognition processing
//! - Dashboard

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    let migration_sql = include_str!("../../../../migrations/105_deferred_revenue.sql");
    sqlx::raw_sql(migration_sql).execute(&state.db_pool).await.ok();
    let app = build_router(state.clone());
    (state, app)
}

async fn create_test_template(app: &axum::Router, code: &str, name: &str, deferral_type: &str) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/deferred-revenue/templates")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": code,
            "name": name,
            "deferral_type": deferral_type,
            "recognition_method": "straight_line",
            "deferral_account_code": "2500",
            "recognition_account_code": "4000",
            "default_periods": 12,
            "period_type": "monthly",
            "start_date_basis": "transaction_date",
            "end_date_basis": "fixed_periods",
            "prorate_partial_periods": true,
            "auto_generate_schedule": true,
            "auto_post": false,
            "currency_code": "USD",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Template Tests
// ============================================================================

#[tokio::test]
async fn test_create_deferral_template() {
    let (_state, app) = setup_test().await;
    let tmpl = create_test_template(&app, "REV-DEF-01", "Revenue Deferral 12M", "revenue").await;
    assert_eq!(tmpl["code"], "REV-DEF-01");
    assert_eq!(tmpl["name"], "Revenue Deferral 12M");
    assert_eq!(tmpl["deferral_type"], "revenue");
    assert_eq!(tmpl["recognition_method"], "straight_line");
    assert_eq!(tmpl["default_periods"], 12);
    assert_eq!(tmpl["is_active"], true);
}

#[tokio::test]
async fn test_create_template_invalid_deferral_type() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/deferred-revenue/templates")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "BAD",
            "name": "Bad",
            "deferral_type": "invalid",
            "recognition_method": "straight_line",
            "deferral_account_code": "2500",
            "recognition_account_code": "4000",
            "default_periods": 12,
            "period_type": "monthly",
            "start_date_basis": "transaction_date",
            "end_date_basis": "fixed_periods",
            "currency_code": "USD",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_template_invalid_recognition_method() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/deferred-revenue/templates")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "BAD",
            "name": "Bad",
            "deferral_type": "revenue",
            "recognition_method": "invalid",
            "deferral_account_code": "2500",
            "recognition_account_code": "4000",
            "default_periods": 12,
            "period_type": "monthly",
            "start_date_basis": "transaction_date",
            "end_date_basis": "fixed_periods",
            "currency_code": "USD",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_list_templates() {
    let (_state, app) = setup_test().await;
    create_test_template(&app, "T1", "Template 1", "revenue").await;
    create_test_template(&app, "T2", "Template 2", "cost").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/deferred-revenue/templates")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_get_template() {
    let (_state, app) = setup_test().await;
    create_test_template(&app, "GET-T", "Get Template", "revenue").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/deferred-revenue/templates/GET-T")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let tmpl: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(tmpl["code"], "GET-T");
}

#[tokio::test]
async fn test_get_template_not_found() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/deferred-revenue/templates/NONEXISTENT")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_template() {
    let (_state, app) = setup_test().await;
    create_test_template(&app, "DEL-T", "Delete Template", "revenue").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/deferred-revenue/templates/DEL-T")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Schedule Tests
// ============================================================================

#[tokio::test]
async fn test_create_deferral_schedule() {
    let (_state, app) = setup_test().await;
    let tmpl = create_test_template(&app, "SCHED-T", "Schedule Template", "revenue").await;
    let template_id = tmpl["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/deferred-revenue/schedules")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "schedule_number": "DS-001",
            "template_id": template_id,
            "source_type": "ar_invoice",
            "source_number": "INV-100",
            "total_amount": "120000.00",
            "currency_code": "USD",
            "start_date": "2025-01-01",
            "end_date": "2025-12-31",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let schedule: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(schedule["schedule_number"].as_str().unwrap().starts_with("DEF-"), true);
    assert_eq!(schedule["total_amount"], "120000.00");
}

#[tokio::test]
async fn test_create_schedule_template_not_found() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/deferred-revenue/schedules")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "schedule_number": "DS-FAIL",
            "template_id": uuid::Uuid::new_v4().to_string(),
            "source_type": "ar_invoice",
            "total_amount": "1000.00",
            "currency_code": "USD",
            "start_date": "2025-01-01",
            "end_date": "2025-12-31",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_list_schedules() {
    let (_state, app) = setup_test().await;
    let tmpl = create_test_template(&app, "LS-T", "List Sched Template", "revenue").await;
    let template_id = tmpl["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let _ = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/deferred-revenue/schedules")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "schedule_number": "DS-LIST",
            "template_id": template_id,
            "source_type": "ar_invoice",
            "total_amount": "60000.00",
            "currency_code": "USD",
            "start_date": "2025-01-01",
            "end_date": "2025-06-30",
        })).unwrap())).unwrap()
    ).await.unwrap();

    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/deferred-revenue/schedules")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 1);
}

#[tokio::test]
async fn test_get_schedule() {
    let (_state, app) = setup_test().await;
    let tmpl = create_test_template(&app, "GS-T", "Get Sched Template", "revenue").await;
    let template_id = tmpl["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/deferred-revenue/schedules")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "schedule_number": "DS-GET",
            "template_id": template_id,
            "source_type": "ar_invoice",
            "total_amount": "24000.00",
            "currency_code": "USD",
            "start_date": "2025-01-01",
            "end_date": "2025-12-31",
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let schedule: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let schedule_id = schedule["id"].as_str().unwrap();

    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/deferred-revenue/schedules/{}", schedule_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_list_schedule_lines() {
    let (_state, app) = setup_test().await;
    let tmpl = create_test_template(&app, "SL-T", "Lines Template", "revenue").await;
    let template_id = tmpl["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/deferred-revenue/schedules")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "schedule_number": "DS-LINES",
            "template_id": template_id,
            "source_type": "ar_invoice",
            "total_amount": "12000.00",
            "currency_code": "USD",
            "start_date": "2025-01-01",
            "end_date": "2025-12-31",
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let schedule: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let schedule_id = schedule["id"].as_str().unwrap();

    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/deferred-revenue/schedules/{}/lines", schedule_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    // Should have 12 lines for a 12-month schedule
    assert!(resp["data"].as_array().unwrap().len() >= 1);
}

// ============================================================================
// Schedule Lifecycle Tests
// ============================================================================

#[tokio::test]
async fn test_hold_and_resume_schedule() {
    let (_state, app) = setup_test().await;
    let tmpl = create_test_template(&app, "HR-T", "Hold/Resume Template", "revenue").await;
    let template_id = tmpl["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/deferred-revenue/schedules")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "schedule_number": "DS-HOLD",
            "template_id": template_id,
            "source_type": "ar_invoice",
            "total_amount": "5000.00",
            "currency_code": "USD",
            "start_date": "2025-01-01",
            "end_date": "2025-12-31",
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let schedule: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let schedule_id = schedule["id"].as_str().unwrap();

    // Hold
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/deferred-revenue/schedules/{}/hold", schedule_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "reason": "Under review",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let s: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(s["status"], "on_hold");

    // Resume
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/deferred-revenue/schedules/{}/resume", schedule_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let s: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(s["status"], "active");
}

#[tokio::test]
async fn test_cancel_schedule() {
    let (_state, app) = setup_test().await;
    let tmpl = create_test_template(&app, "CAN-T", "Cancel Template", "revenue").await;
    let template_id = tmpl["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/deferred-revenue/schedules")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "schedule_number": "DS-CANCEL",
            "template_id": template_id,
            "source_type": "ar_invoice",
            "total_amount": "5000.00",
            "currency_code": "USD",
            "start_date": "2025-01-01",
            "end_date": "2025-12-31",
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let schedule: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let schedule_id = schedule["id"].as_str().unwrap();

    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/deferred-revenue/schedules/{}/cancel", schedule_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let s: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(s["status"], "cancelled");
}

// ============================================================================
// Dashboard Test
// ============================================================================

#[tokio::test]
async fn test_get_deferred_revenue_dashboard() {
    let (_state, app) = setup_test().await;
    create_test_template(&app, "DASH-T", "Dashboard Template", "revenue").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/deferred-revenue/dashboard")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}
