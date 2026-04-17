//! Period Close Management E2E tests
//!
//! Tests for Oracle Fusion Cloud ERP General Ledger Period Close feature:
//! - Accounting calendar CRUD
//! - Period generation
//! - Period status lifecycle (open → pending close → close → permanently close)
//! - Subledger close tracking
//! - Period close checklist
//! - Posting validation (prevent posting to closed periods)
//! - Period exceptions

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use uuid::Uuid;
use super::common::helpers::*;

// ============================================================================
// Test Setup
// ============================================================================

async fn setup_period_close() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    let app = build_router(state.clone());
    (state, app)
}

async fn create_test_calendar(app: &axum::Router, k: &str, v: &str) -> serde_json::Value {
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/period-close/calendars")
        .header("Content-Type", "application/json").header(k, v)
        .body(Body::from(serde_json::to_string(&json!({
            "name": "Test Fiscal Calendar",
            "description": "Standard fiscal year starting January",
            "calendar_type": "monthly",
            "fiscal_year_start_month": 1,
            "periods_per_year": 12,
            "has_adjusting_period": false,
            "current_fiscal_year": 2026
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn generate_test_periods(app: &axum::Router, k: &str, v: &str, calendar_id: Uuid, fiscal_year: i32) -> serde_json::Value {
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/period-close/calendars/{}/periods/generate", calendar_id))
        .header("Content-Type", "application/json").header(k, v)
        .body(Body::from(serde_json::to_string(&json!({
            "fiscal_year": fiscal_year
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn get_first_period_id(app: &axum::Router, k: &str, v: &str, calendar_id: Uuid) -> Uuid {
    let r = app.clone().oneshot(Request::builder()
        .uri(format!("/api/v1/period-close/calendars/{}/periods?fiscal_year=2026", calendar_id))
        .header(k, v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let data: serde_json::Value = serde_json::from_slice(&b).unwrap();
    data["data"].as_array().unwrap()[0]["id"].as_str().unwrap().parse().unwrap()
}

// ============================================================================
// Calendar Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_create_calendar() {
    let (state, app) = setup_period_close().await;
    let (k, v) = auth_header(&admin_claims());

    let calendar = create_test_calendar(&app, &k, &v).await;
    assert_eq!(calendar["name"], "Test Fiscal Calendar");
    assert_eq!(calendar["calendar_type"], "monthly");
    assert_eq!(calendar["periods_per_year"], 12);
    assert_eq!(calendar["fiscal_year_start_month"], 1);
    assert_eq!(calendar["is_active"], true);

    cleanup_test_db(&state.db_pool).await;
}

#[tokio::test]
#[ignore]
async fn test_list_calendars() {
    let (state, app) = setup_period_close().await;
    let (k, v) = auth_header(&admin_claims());

    create_test_calendar(&app, &k, &v).await;

    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/period-close/calendars")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let data: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(data["data"].as_array().unwrap().len() >= 1);

    cleanup_test_db(&state.db_pool).await;
}

#[tokio::test]
#[ignore]
async fn test_get_calendar() {
    let (state, app) = setup_period_close().await;
    let (k, v) = auth_header(&admin_claims());

    let calendar = create_test_calendar(&app, &k, &v).await;
    let calendar_id = calendar["id"].as_str().unwrap();

    let r = app.clone().oneshot(Request::builder()
        .uri(format!("/api/v1/period-close/calendars/{}", calendar_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let fetched: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(fetched["name"], "Test Fiscal Calendar");

    cleanup_test_db(&state.db_pool).await;
}

#[tokio::test]
#[ignore]
async fn test_delete_calendar() {
    let (state, app) = setup_period_close().await;
    let (k, v) = auth_header(&admin_claims());

    let calendar = create_test_calendar(&app, &k, &v).await;
    let calendar_id = calendar["id"].as_str().unwrap();

    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(format!("/api/v1/period-close/calendars/{}", calendar_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);

    // Verify it's gone (404)
    let r = app.clone().oneshot(Request::builder()
        .uri(format!("/api/v1/period-close/calendars/{}", calendar_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);

    cleanup_test_db(&state.db_pool).await;
}

// ============================================================================
// Period Generation Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_generate_periods() {
    let (state, app) = setup_period_close().await;
    let (k, v) = auth_header(&admin_claims());

    let calendar = create_test_calendar(&app, &k, &v).await;
    let calendar_id: Uuid = calendar["id"].as_str().unwrap().parse().unwrap();

    let result = generate_test_periods(&app, &k, &v, calendar_id, 2026).await;
    let periods = result["data"].as_array().unwrap();
    assert_eq!(periods.len(), 12); // 12 monthly periods

    // Check first period
    assert_eq!(periods[0]["period_number"], 1);
    assert_eq!(periods[0]["fiscal_year"], 2026);
    assert_eq!(periods[0]["status"], "not_opened");
    assert_eq!(periods[0]["period_type"], "regular");
    assert_eq!(periods[0]["period_name"], "01-2026");

    // Check last period
    let last = periods.last().unwrap();
    assert_eq!(last["period_number"], 12);
    assert_eq!(last["fiscal_year"], 2026);

    cleanup_test_db(&state.db_pool).await;
}

#[tokio::test]
#[ignore]
async fn test_list_periods() {
    let (state, app) = setup_period_close().await;
    let (k, v) = auth_header(&admin_claims());

    let calendar = create_test_calendar(&app, &k, &v).await;
    let calendar_id: Uuid = calendar["id"].as_str().unwrap().parse().unwrap();
    generate_test_periods(&app, &k, &v, calendar_id, 2026).await;

    let r = app.clone().oneshot(Request::builder()
        .uri(format!("/api/v1/period-close/calendars/{}/periods?fiscal_year=2026", calendar_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let data: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(data["data"].as_array().unwrap().len(), 12);

    cleanup_test_db(&state.db_pool).await;
}

// ============================================================================
// Period Status Lifecycle Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_period_lifecycle_not_opened_to_open() {
    let (state, app) = setup_period_close().await;
    let (k, v) = auth_header(&admin_claims());

    let calendar = create_test_calendar(&app, &k, &v).await;
    let calendar_id: Uuid = calendar["id"].as_str().unwrap().parse().unwrap();
    generate_test_periods(&app, &k, &v, calendar_id, 2026).await;
    let period_id = get_first_period_id(&app, &k, &v, calendar_id).await;

    // Open the period
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/period-close/periods/{}/open", period_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"comment": "Opening Jan"})).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let period: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(period["status"], "open");
    assert!(period["status_changed_by"].is_string());

    cleanup_test_db(&state.db_pool).await;
}

#[tokio::test]
#[ignore]
async fn test_period_lifecycle_full() {
    let (state, app) = setup_period_close().await;
    let (k, v) = auth_header(&admin_claims());

    let calendar = create_test_calendar(&app, &k, &v).await;
    let calendar_id: Uuid = calendar["id"].as_str().unwrap().parse().unwrap();
    generate_test_periods(&app, &k, &v, calendar_id, 2026).await;
    let period_id = get_first_period_id(&app, &k, &v, calendar_id).await;

    // Step 1: Open
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/period-close/periods/{}/open", period_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(json!({"comment": "Open"}).to_string())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let period: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(period["status"], "open");

    // Step 2: Close all subledgers first (required for close)
    for subledger in &["gl", "ap", "ar", "fa", "po"] {
        let r = app.clone().oneshot(Request::builder().method("POST")
            .uri(format!("/api/v1/period-close/periods/{}/subledger", period_id))
            .header("Content-Type", "application/json").header(&k, &v)
            .body(Body::from(json!({"subledger": subledger, "status": "closed"}).to_string())).unwrap()
        ).await.unwrap();
        assert_eq!(r.status(), StatusCode::OK);
    }

    // Step 3: Close the period
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/period-close/periods/{}/close", period_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(json!({"comment": "Closing Jan", "force": false}).to_string())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let period: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(period["status"], "closed");

    // Step 4: Permanently close
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/period-close/periods/{}/permanently-close", period_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(json!({"comment": "Permanent"}).to_string())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let period: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(period["status"], "permanently_closed");

    cleanup_test_db(&state.db_pool).await;
}

#[tokio::test]
#[ignore]
async fn test_cannot_close_period_without_subledgers() {
    let (state, app) = setup_period_close().await;
    let (k, v) = auth_header(&admin_claims());

    let calendar = create_test_calendar(&app, &k, &v).await;
    let calendar_id: Uuid = calendar["id"].as_str().unwrap().parse().unwrap();
    generate_test_periods(&app, &k, &v, calendar_id, 2026).await;
    let period_id = get_first_period_id(&app, &k, &v, calendar_id).await;

    // Open the period
    app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/period-close/periods/{}/open", period_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(json!({"comment": "Open"}).to_string())).unwrap()
    ).await.unwrap();

    // Try to close without closing subledgers (should fail)
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/period-close/periods/{}/close", period_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(json!({"comment": "Try close", "force": false}).to_string())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);

    cleanup_test_db(&state.db_pool).await;
}

#[tokio::test]
#[ignore]
async fn test_force_close_period() {
    let (state, app) = setup_period_close().await;
    let (k, v) = auth_header(&admin_claims());

    let calendar = create_test_calendar(&app, &k, &v).await;
    let calendar_id: Uuid = calendar["id"].as_str().unwrap().parse().unwrap();
    generate_test_periods(&app, &k, &v, calendar_id, 2026).await;
    let period_id = get_first_period_id(&app, &k, &v, calendar_id).await;

    // Open the period
    app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/period-close/periods/{}/open", period_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(json!({}).to_string())).unwrap()
    ).await.unwrap();

    // Force close (skip subledger checks)
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/period-close/periods/{}/close", period_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(json!({"comment": "Force close", "force": true}).to_string())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let period: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(period["status"], "closed");

    cleanup_test_db(&state.db_pool).await;
}

#[tokio::test]
#[ignore]
async fn test_reopen_period() {
    let (state, app) = setup_period_close().await;
    let (k, v) = auth_header(&admin_claims());

    let calendar = create_test_calendar(&app, &k, &v).await;
    let calendar_id: Uuid = calendar["id"].as_str().unwrap().parse().unwrap();
    generate_test_periods(&app, &k, &v, calendar_id, 2026).await;
    let period_id = get_first_period_id(&app, &k, &v, calendar_id).await;

    // Open → Force close
    app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/period-close/periods/{}/open", period_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(json!({}).to_string())).unwrap()
    ).await.unwrap();

    app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/period-close/periods/{}/close", period_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(json!({"force": true}).to_string())).unwrap()
    ).await.unwrap();

    // Reopen
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/period-close/periods/{}/reopen", period_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(json!({}).to_string())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let period: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(period["status"], "open");

    cleanup_test_db(&state.db_pool).await;
}

#[tokio::test]
#[ignore]
async fn test_cannot_reopen_permanently_closed_period() {
    let (state, app) = setup_period_close().await;
    let (k, v) = auth_header(&admin_claims());

    let calendar = create_test_calendar(&app, &k, &v).await;
    let calendar_id: Uuid = calendar["id"].as_str().unwrap().parse().unwrap();
    generate_test_periods(&app, &k, &v, calendar_id, 2026).await;
    let period_id = get_first_period_id(&app, &k, &v, calendar_id).await;

    // Open → Close → Permanently Close
    app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/period-close/periods/{}/open", period_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(json!({}).to_string())).unwrap()
    ).await.unwrap();

    app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/period-close/periods/{}/close", period_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(json!({"force": true}).to_string())).unwrap()
    ).await.unwrap();

    app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/period-close/periods/{}/permanently-close", period_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(json!({}).to_string())).unwrap()
    ).await.unwrap();

    // Try to reopen permanently closed period (should fail)
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/period-close/periods/{}/reopen", period_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(json!({}).to_string())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);

    cleanup_test_db(&state.db_pool).await;
}

// ============================================================================
// Posting Validation Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_posting_allowed_for_open_period() {
    let (state, app) = setup_period_close().await;
    let (k, v) = auth_header(&admin_claims());

    let calendar = create_test_calendar(&app, &k, &v).await;
    let calendar_id: Uuid = calendar["id"].as_str().unwrap().parse().unwrap();
    generate_test_periods(&app, &k, &v, calendar_id, 2026).await;

    // First, open the first period (Jan 2026)
    let period_id = get_first_period_id(&app, &k, &v, calendar_id).await;
    app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/period-close/periods/{}/open", period_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(json!({}).to_string())).unwrap()
    ).await.unwrap();

    // Check posting for a date in the open period
    let r = app.clone().oneshot(Request::builder()
        .uri(format!("/api/v1/period-close/calendars/{}/check-posting?date=2026-01-15", calendar_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(result["allowed"], true);

    cleanup_test_db(&state.db_pool).await;
}

#[tokio::test]
#[ignore]
async fn test_posting_blocked_for_closed_period() {
    let (state, app) = setup_period_close().await;
    let (k, v) = auth_header(&admin_claims());

    let calendar = create_test_calendar(&app, &k, &v).await;
    let calendar_id: Uuid = calendar["id"].as_str().unwrap().parse().unwrap();
    generate_test_periods(&app, &k, &v, calendar_id, 2026).await;

    // Don't open the period - check posting (should be blocked since status is 'not_opened')
    let r = app.clone().oneshot(Request::builder()
        .uri(format!("/api/v1/period-close/calendars/{}/check-posting?date=2026-01-15", calendar_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(result["allowed"], false);
    assert!(result["reason"].as_str().unwrap().contains("not_opened"));

    cleanup_test_db(&state.db_pool).await;
}

// ============================================================================
// Period Close Checklist Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_checklist_lifecycle() {
    let (state, app) = setup_period_close().await;
    let (k, v) = auth_header(&admin_claims());

    let calendar = create_test_calendar(&app, &k, &v).await;
    let calendar_id: Uuid = calendar["id"].as_str().unwrap().parse().unwrap();
    generate_test_periods(&app, &k, &v, calendar_id, 2026).await;
    let period_id = get_first_period_id(&app, &k, &v, calendar_id).await;

    // Add checklist items
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/period-close/periods/{}/checklist", period_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(json!({
            "task_name": "Reconcile bank accounts",
            "task_description": "Ensure all bank accounts are reconciled",
            "task_order": 1,
            "category": "reconciliation",
            "subledger": "gl"
        }).to_string())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let item1: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let item1_id = item1["id"].as_str().unwrap();

    // Add second item
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/period-close/periods/{}/checklist", period_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(json!({
            "task_name": "Post accrual entries",
            "task_order": 2,
            "category": "accrual"
        }).to_string())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);

    // List checklist items
    let r = app.clone().oneshot(Request::builder()
        .uri(format!("/api/v1/period-close/periods/{}/checklist", period_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let items: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(items["data"].as_array().unwrap().len(), 2);

    // Update first item to completed
    let r = app.clone().oneshot(Request::builder().method("PUT")
        .uri(format!("/api/v1/period-close/periods/{}/checklist/{}", period_id, item1_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(json!({"status": "completed"}).to_string())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(updated["status"], "completed");
    assert!(updated["completed_by"].is_string());

    // Delete second item
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(format!("/api/v1/period-close/periods/{}/checklist/{}", period_id, item1_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);

    cleanup_test_db(&state.db_pool).await;
}

// ============================================================================
// Period Close Dashboard Summary Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_close_summary() {
    let (state, app) = setup_period_close().await;
    let (k, v) = auth_header(&admin_claims());

    let calendar = create_test_calendar(&app, &k, &v).await;
    let calendar_id: Uuid = calendar["id"].as_str().unwrap().parse().unwrap();
    generate_test_periods(&app, &k, &v, calendar_id, 2026).await;

    // Get close summary
    let r = app.clone().oneshot(Request::builder()
        .uri(format!("/api/v1/period-close/calendars/{}/summary?fiscal_year=2026", calendar_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let summary: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(summary["calendar_name"], "Test Fiscal Calendar");
    assert_eq!(summary["fiscal_year"], 2026);

    cleanup_test_db(&state.db_pool).await;
}

// ============================================================================
// Invalid Transition Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_cannot_open_already_open_period() {
    let (state, app) = setup_period_close().await;
    let (k, v) = auth_header(&admin_claims());

    let calendar = create_test_calendar(&app, &k, &v).await;
    let calendar_id: Uuid = calendar["id"].as_str().unwrap().parse().unwrap();
    generate_test_periods(&app, &k, &v, calendar_id, 2026).await;
    let period_id = get_first_period_id(&app, &k, &v, calendar_id).await;

    // Open
    app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/period-close/periods/{}/open", period_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(json!({}).to_string())).unwrap()
    ).await.unwrap();

    // Try to open again (should fail)
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/period-close/periods/{}/open", period_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(json!({}).to_string())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);

    cleanup_test_db(&state.db_pool).await;
}

#[tokio::test]
#[ignore]
async fn test_cannot_close_not_opened_period() {
    let (state, app) = setup_period_close().await;
    let (k, v) = auth_header(&admin_claims());

    let calendar = create_test_calendar(&app, &k, &v).await;
    let calendar_id: Uuid = calendar["id"].as_str().unwrap().parse().unwrap();
    generate_test_periods(&app, &k, &v, calendar_id, 2026).await;
    let period_id = get_first_period_id(&app, &k, &v, calendar_id).await;

    // Try to close a not_opened period (should fail)
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/period-close/periods/{}/close", period_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(json!({"force": true}).to_string())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);

    cleanup_test_db(&state.db_pool).await;
}
