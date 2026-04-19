//! Recurring Journals E2E Tests
//!
//! Tests for Oracle Fusion Cloud ERP Recurring Journals:
//! - Schedule CRUD (create, get, list, delete)
//! - Schedule lifecycle (draft → active → inactive)
//! - Template line management (add, list, delete)
//! - Journal generation (standard, skeleton, incremental types)
//! - Generation lifecycle (generate → post → reverse)
//! - Generation cancellation
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
    setup_test_db(&state.db_pool).await;
    sqlx::query(include_str!("../../../../migrations/040_recurring_journals.sql"))
        .execute(&state.db_pool)
        .await
        .ok();
    let app = build_router(state.clone());
    (state, app)
}

async fn create_standard_schedule(
    app: &axum::Router,
    number: &str,
    name: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "schedule_number": number,
        "name": name,
        "description": "Test recurring journal",
        "recurrence_type": "monthly",
        "journal_type": "standard",
        "currency_code": "USD",
        "effective_from": "2024-01-01",
        "effective_to": "2024-12-31",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/recurring-journals/schedules")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Failed to create schedule");
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn add_line(
    app: &axum::Router,
    schedule_id: Uuid,
    line_type: &str,
    account_code: &str,
    amount: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "line_type": line_type,
        "account_code": account_code,
        "account_name": format!("Account {}", account_code),
        "amount": amount,
        "currency_code": "USD",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recurring-journals/schedules/{}/lines", schedule_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Failed to add line");
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Schedule CRUD Tests
// ============================================================================

#[tokio::test]
async fn test_create_schedule() {
    let (_state, app) = setup_test().await;
    let schedule = create_standard_schedule(&app, "RJ-001", "Monthly Rent").await;

    assert_eq!(schedule["schedule_number"], "RJ-001");
    assert_eq!(schedule["name"], "Monthly Rent");
    assert_eq!(schedule["recurrence_type"], "monthly");
    assert_eq!(schedule["journal_type"], "standard");
    assert_eq!(schedule["status"], "draft");
    assert_eq!(schedule["currency_code"], "USD");
}

#[tokio::test]
async fn test_create_incremental_schedule() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "schedule_number": "RJ-INCR-01",
        "name": "Incremental Lease",
        "recurrence_type": "annual",
        "journal_type": "incremental",
        "currency_code": "USD",
        "incremental_percent": "5",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/recurring-journals/schedules")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["journal_type"], "incremental");
}

#[tokio::test]
async fn test_get_schedule() {
    let (_state, app) = setup_test().await;
    create_standard_schedule(&app, "RJ-GET", "Get Test").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/recurring-journals/schedules/RJ-GET")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["schedule_number"], "RJ-GET");
    assert_eq!(body["name"], "Get Test");
}

#[tokio::test]
async fn test_list_schedules() {
    let (_state, app) = setup_test().await;
    create_standard_schedule(&app, "RJ-LIST1", "List 1").await;
    create_standard_schedule(&app, "RJ-LIST2", "List 2").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/recurring-journals/schedules")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(body["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_list_schedules_with_status_filter() {
    let (_state, app) = setup_test().await;
    create_standard_schedule(&app, "RJ-FILTER", "Filter Test").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/recurring-journals/schedules?status=draft")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(body["data"].as_array().unwrap().len() >= 1);
}

#[tokio::test]
async fn test_delete_schedule() {
    let (_state, app) = setup_test().await;
    create_standard_schedule(&app, "RJ-DEL", "Deletable").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/recurring-journals/schedules/RJ-DEL")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
}

// ============================================================================
// Schedule Line Tests
// ============================================================================

#[tokio::test]
async fn test_add_and_list_schedule_lines() {
    let (_state, app) = setup_test().await;
    let schedule = create_standard_schedule(&app, "RJ-LINES", "Lines Test").await;
    let schedule_id: Uuid = schedule["id"].as_str().unwrap().parse().unwrap();

    let debit = add_line(&app, schedule_id, "debit", "1000-100", "5000.00").await;
    let credit = add_line(&app, schedule_id, "credit", "2000-200", "5000.00").await;

    assert_eq!(debit["line_type"], "debit");
    assert_eq!(debit["account_code"], "1000-100");
    assert_eq!(credit["line_type"], "credit");
    assert_eq!(credit["account_code"], "2000-200");

    // List lines
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/recurring-journals/schedules/{}/lines", schedule_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["data"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_delete_schedule_line() {
    let (_state, app) = setup_test().await;
    let schedule = create_standard_schedule(&app, "RJ-DELLINE", "Delete Line Test").await;
    let schedule_id: Uuid = schedule["id"].as_str().unwrap().parse().unwrap();

    let line = add_line(&app, schedule_id, "debit", "1000-100", "1000.00").await;
    let line_id = line["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/recurring-journals/lines/{}", line_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
}

// ============================================================================
// Schedule Lifecycle Tests
// ============================================================================

#[tokio::test]
async fn test_activate_schedule() {
    let (_state, app) = setup_test().await;
    let schedule = create_standard_schedule(&app, "RJ-ACT", "Activate Test").await;
    let schedule_id: Uuid = schedule["id"].as_str().unwrap().parse().unwrap();

    // Add balanced lines first
    add_line(&app, schedule_id, "debit", "1000-100", "3000.00").await;
    add_line(&app, schedule_id, "credit", "2000-200", "3000.00").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recurring-journals/schedules/{}/activate", schedule_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "active");
}

#[tokio::test]
async fn test_activate_unbalanced_fails() {
    let (_state, app) = setup_test().await;
    let schedule = create_standard_schedule(&app, "RJ-UNBAL", "Unbalanced Test").await;
    let schedule_id: Uuid = schedule["id"].as_str().unwrap().parse().unwrap();

    // Add unbalanced lines
    add_line(&app, schedule_id, "debit", "1000-100", "3000.00").await;
    add_line(&app, schedule_id, "credit", "2000-200", "2000.00").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recurring-journals/schedules/{}/activate", schedule_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_deactivate_schedule() {
    let (_state, app) = setup_test().await;
    let schedule = create_standard_schedule(&app, "RJ-DEACT", "Deactivate Test").await;
    let schedule_id: Uuid = schedule["id"].as_str().unwrap().parse().unwrap();

    // Activate first
    add_line(&app, schedule_id, "debit", "1000-100", "1000.00").await;
    add_line(&app, schedule_id, "credit", "2000-200", "1000.00").await;

    let (k, v) = auth_header(&admin_claims());
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recurring-journals/schedules/{}/activate", schedule_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Deactivate
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recurring-journals/schedules/{}/deactivate", schedule_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "inactive");
}

// ============================================================================
// Full Generation Workflow Tests
// ============================================================================

#[tokio::test]
async fn test_standard_generation_workflow() {
    let (_state, app) = setup_test().await;
    let schedule = create_standard_schedule(&app, "RJ-GEN1", "Generation Test").await;
    let schedule_id: Uuid = schedule["id"].as_str().unwrap().parse().unwrap();

    // Add balanced lines
    add_line(&app, schedule_id, "debit", "1000-100", "5000.00").await;
    add_line(&app, schedule_id, "credit", "2000-200", "5000.00").await;

    // Activate
    let (k, v) = auth_header(&admin_claims());
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recurring-journals/schedules/{}/activate", schedule_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Generate
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recurring-journals/schedules/{}/generate", schedule_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "generation_date": "2024-02-01"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::CREATED);
    let gen: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(gen["status"], "generated");
    assert_eq!(gen["generation_number"], 1);
    assert_eq!(gen["line_count"], 2);

    let gen_id = gen["id"].as_str().unwrap();

    // Check generation lines
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/recurring-journals/generations/{}/lines", gen_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let lines: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(lines["data"].as_array().unwrap().len(), 2);

    // Post
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recurring-journals/generations/{}/post", gen_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "posted");

    // Reverse
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recurring-journals/generations/{}/reverse", gen_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "reversed");
}

#[tokio::test]
async fn test_cancel_generation() {
    let (_state, app) = setup_test().await;
    let schedule = create_standard_schedule(&app, "RJ-CANCEL-GEN", "Cancel Gen Test").await;
    let schedule_id: Uuid = schedule["id"].as_str().unwrap().parse().unwrap();

    add_line(&app, schedule_id, "debit", "1000-100", "2000.00").await;
    add_line(&app, schedule_id, "credit", "2000-200", "2000.00").await;

    let (k, v) = auth_header(&admin_claims());
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recurring-journals/schedules/{}/activate", schedule_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Generate
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recurring-journals/schedules/{}/generate", schedule_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "generation_date": "2024-03-01"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    let gen: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let gen_id = gen["id"].as_str().unwrap();

    // Cancel
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recurring-journals/generations/{}/cancel", gen_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "cancelled");
}

#[tokio::test]
async fn test_list_generations() {
    let (_state, app) = setup_test().await;
    let schedule = create_standard_schedule(&app, "RJ-GEN-LIST", "List Gen Test").await;
    let schedule_id: Uuid = schedule["id"].as_str().unwrap().parse().unwrap();

    add_line(&app, schedule_id, "debit", "1000-100", "1000.00").await;
    add_line(&app, schedule_id, "credit", "2000-200", "1000.00").await;

    let (k, v) = auth_header(&admin_claims());
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recurring-journals/schedules/{}/activate", schedule_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Generate two journals
    for date in &["2024-04-01", "2024-05-01"] {
        app.clone().oneshot(Request::builder().method("POST")
            .uri(&format!("/api/v1/recurring-journals/schedules/{}/generate", schedule_id))
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({
                "generation_date": *date
            })).unwrap()))
            .unwrap()
        ).await.unwrap();
    }

    // List generations
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/recurring-journals/schedules/{}/generations", schedule_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["data"].as_array().unwrap().len(), 2);
}

// ============================================================================
// Incremental Type Tests
// ============================================================================

#[tokio::test]
async fn test_incremental_generation() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Create incremental schedule
    let payload = json!({
        "schedule_number": "RJ-INCR-GEN",
        "name": "Incremental Gen Test",
        "recurrence_type": "annual",
        "journal_type": "incremental",
        "currency_code": "USD",
        "incremental_percent": "10",
        "effective_from": "2024-01-01",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/recurring-journals/schedules")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    let schedule: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let schedule_id: Uuid = schedule["id"].as_str().unwrap().parse().unwrap();

    // Add lines (1000 debit, 1000 credit - base amount)
    add_line(&app, schedule_id, "debit", "1000-100", "1000.00").await;
    add_line(&app, schedule_id, "credit", "2000-200", "1000.00").await;

    // Activate
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recurring-journals/schedules/{}/activate", schedule_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // First generation (base amount: 1000)
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recurring-journals/schedules/{}/generate", schedule_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "generation_date": "2024-01-01"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    let gen1: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let gen1_id = gen1["id"].as_str().unwrap();

    // Check first gen lines - should be 1000 each
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/recurring-journals/generations/{}/lines", gen1_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    let _lines1: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();

    // Second generation (incremented: 1000 * 1.1 = 1100)
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recurring-journals/schedules/{}/generate", schedule_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "generation_date": "2025-01-01"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    let gen2: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let gen2_id = gen2["id"].as_str().unwrap();

    // Check second gen lines
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/recurring-journals/generations/{}/lines", gen2_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    let lines2: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();

    // Verify the second generation amount is higher than first (1100 vs 1000)
    let gen2_debits: f64 = lines2["data"].as_array().unwrap().iter()
        .filter(|l| l["line_type"] == "debit")
        .map(|l| l["amount"].as_str().unwrap().parse::<f64>().unwrap())
        .sum();
    assert!(gen2_debits > 1000.0, "Second generation should have incremented amount, got {}", gen2_debits);
}

// ============================================================================
// Skeleton Type Tests
// ============================================================================

#[tokio::test]
async fn test_skeleton_generation_with_overrides() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());

    let payload = json!({
        "schedule_number": "RJ-SKEL",
        "name": "Skeleton Schedule",
        "recurrence_type": "monthly",
        "journal_type": "skeleton",
        "currency_code": "USD",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/recurring-journals/schedules")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    let schedule: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let schedule_id: Uuid = schedule["id"].as_str().unwrap().parse().unwrap();

    // Add template lines with zero amounts for skeleton
    add_line(&app, schedule_id, "debit", "5000-100", "0").await;
    add_line(&app, schedule_id, "credit", "6000-200", "0").await;

    // Activate (skeleton type doesn't require balanced amounts)
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recurring-journals/schedules/{}/activate", schedule_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Generate with override amounts
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recurring-journals/schedules/{}/generate", schedule_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "generation_date": "2024-06-01",
            "override_amounts": [
                {"line_number": 1, "amount": "7500.00"},
                {"line_number": 2, "amount": "7500.00"}
            ]
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let gen: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(gen["line_count"], 2);
}

// ============================================================================
// Dashboard Tests
// ============================================================================

#[tokio::test]
async fn test_recurring_journal_dashboard() {
    let (_state, app) = setup_test().await;
    create_standard_schedule(&app, "RJ-DASH", "Dashboard Test").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/recurring-journals/dashboard")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();

    assert!(body.get("total_active_schedules").is_some());
    assert!(body.get("total_draft_schedules").is_some());
    assert!(body.get("total_generations").is_some());
    assert!(body.get("schedules_due_today").is_some());
    assert!(body.get("schedules_overdue").is_some());
}

// ============================================================================
// Validation Tests
// ============================================================================

#[tokio::test]
async fn test_create_schedule_empty_number_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "schedule_number": "",
        "name": "No Number",
        "recurrence_type": "monthly",
        "journal_type": "standard",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/recurring-journals/schedules")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_schedule_invalid_recurrence_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "schedule_number": "RJ-BAD-REC",
        "name": "Bad Recurrence",
        "recurrence_type": "biweekly",
        "journal_type": "standard",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/recurring-journals/schedules")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_schedule_invalid_journal_type_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "schedule_number": "RJ-BAD-TYPE",
        "name": "Bad Type",
        "recurrence_type": "monthly",
        "journal_type": "recurring",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/recurring-journals/schedules")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_activate_without_lines_fails() {
    let (_state, app) = setup_test().await;
    let schedule = create_standard_schedule(&app, "RJ-NO-LINES", "No Lines").await;
    let schedule_id: Uuid = schedule["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recurring-journals/schedules/{}/activate", schedule_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_generate_from_draft_schedule_fails() {
    let (_state, app) = setup_test().await;
    let schedule = create_standard_schedule(&app, "RJ-GEN-DRAFT", "Gen Draft").await;
    let schedule_id: Uuid = schedule["id"].as_str().unwrap().parse().unwrap();

    add_line(&app, schedule_id, "debit", "1000-100", "1000.00").await;
    add_line(&app, schedule_id, "credit", "2000-200", "1000.00").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recurring-journals/schedules/{}/generate", schedule_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "generation_date": "2024-01-01"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_reverse_unposted_generation_fails() {
    let (_state, app) = setup_test().await;
    let schedule = create_standard_schedule(&app, "RJ-REV-FAIL", "Reverse Fail Test").await;
    let schedule_id: Uuid = schedule["id"].as_str().unwrap().parse().unwrap();

    add_line(&app, schedule_id, "debit", "1000-100", "500.00").await;
    add_line(&app, schedule_id, "credit", "2000-200", "500.00").await;

    let (k, v) = auth_header(&admin_claims());
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recurring-journals/schedules/{}/activate", schedule_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Generate but don't post
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recurring-journals/schedules/{}/generate", schedule_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "generation_date": "2024-01-01"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    let gen: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let gen_id = gen["id"].as_str().unwrap();

    // Try to reverse without posting first
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recurring-journals/generations/{}/reverse", gen_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_delete_active_schedule_fails() {
    let (_state, app) = setup_test().await;
    let schedule = create_standard_schedule(&app, "RJ-DEL-ACTIVE", "Delete Active Test").await;
    let schedule_id: Uuid = schedule["id"].as_str().unwrap().parse().unwrap();

    add_line(&app, schedule_id, "debit", "1000-100", "500.00").await;
    add_line(&app, schedule_id, "credit", "2000-200", "500.00").await;

    let (k, v) = auth_header(&admin_claims());
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/recurring-journals/schedules/{}/activate", schedule_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Try to delete active schedule
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/recurring-journals/schedules/RJ-DEL-ACTIVE")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}
