//! Interest Invoice Management E2E Tests
//!
//! Tests for Oracle Fusion Cloud ERP Interest Invoice Management:
//! - Interest rate schedule CRUD (create, get, list, delete)
//! - Schedule lifecycle (active → inactive)
//! - Overdue invoice registration and tracking
//! - Interest calculation on overdue invoices
//! - Interest invoice generation from calculation runs
//! - Invoice lifecycle (draft → posted → reversed)
//! - Invoice cancellation
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
    sqlx::query(include_str!("../../../../migrations/119_interest_invoice_management.sql"))
        .execute(&state.db_pool)
        .await
        .ok();
    let app = build_router(state.clone());
    (state, app)
}

async fn create_schedule(
    app: &axum::Router,
    code: &str,
    name: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "schedule_code": code,
        "name": name,
        "description": "Test interest schedule",
        "annual_rate": "12.0",
        "compounding_frequency": "daily",
        "charge_type": "interest",
        "grace_period_days": 5,
        "minimum_charge": "10.00",
        "currency_code": "USD",
        "effective_from": "2024-01-01",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/interest-invoices/schedules")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8_lossy(&b);
    eprintln!("RESPONSE status={}: {}", status, body_str);
    assert_eq!(status, StatusCode::CREATED, "Failed to create schedule: {:?}", body_str);
    serde_json::from_slice(&b).unwrap()
}

async fn register_overdue(
    app: &axum::Router,
    invoice_number: &str,
    amount: &str,
    overdue_days: i32,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "invoice_number": invoice_number,
        "customer_id": "11111111-1111-1111-1111-111111111111",
        "customer_name": "Test Customer",
        "original_amount": amount,
        "outstanding_amount": amount,
        "due_date": "2024-01-15",
        "overdue_days": overdue_days,
        "currency_code": "USD",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/interest-invoices/overdue")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    eprintln!("OVERDUE RESPONSE status={}: {}", status, String::from_utf8_lossy(&b));
    assert_eq!(status, StatusCode::CREATED, "Failed to register overdue invoice");
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Schedule CRUD Tests
// ============================================================================

#[tokio::test]
async fn test_create_schedule() {
    let (_state, app) = setup_test().await;
    let schedule = create_schedule(&app, "INT-001", "Standard Late Charge").await;

    assert_eq!(schedule["scheduleCode"], "INT-001");
    assert_eq!(schedule["name"], "Standard Late Charge");
    assert_eq!(schedule["compoundingFrequency"], "daily");
    assert_eq!(schedule["chargeType"], "interest");
    assert_eq!(schedule["status"], "active");
    assert_eq!(schedule["gracePeriodDays"], 5);
}

#[tokio::test]
async fn test_get_schedule() {
    let (_state, app) = setup_test().await;
    create_schedule(&app, "INT-GET", "Get Test").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/interest-invoices/schedules/INT-GET")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["scheduleCode"], "INT-GET");
    assert_eq!(body["name"], "Get Test");
}

#[tokio::test]
async fn test_list_schedules() {
    let (_state, app) = setup_test().await;
    create_schedule(&app, "INT-LIST1", "List 1").await;
    create_schedule(&app, "INT-LIST2", "List 2").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/interest-invoices/schedules")
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
    create_schedule(&app, "INT-FILTER", "Filter Test").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/interest-invoices/schedules?status=active")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(body["data"].as_array().unwrap().len() >= 1);
}

// ============================================================================
// Schedule Lifecycle Tests
// ============================================================================

#[tokio::test]
async fn test_deactivate_activate_schedule() {
    let (_state, app) = setup_test().await;
    let schedule = create_schedule(&app, "INT-DA", "Deactivate Activate").await;
    let schedule_id: Uuid = schedule["id"].as_str().unwrap().parse().unwrap();

    // Deactivate
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/interest-invoices/schedules/{}/deactivate", schedule_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "inactive");

    // Reactivate
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/interest-invoices/schedules/{}/activate", schedule_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "active");
}

#[tokio::test]
async fn test_delete_inactive_schedule() {
    let (_state, app) = setup_test().await;
    let schedule = create_schedule(&app, "INT-DEL", "Delete Test").await;
    let schedule_id: Uuid = schedule["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Must deactivate first
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/interest-invoices/schedules/{}/deactivate", schedule_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Now delete
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/interest-invoices/schedules/INT-DEL")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}

// ============================================================================
// Overdue Invoice Tests
// ============================================================================

#[tokio::test]
async fn test_register_overdue_invoice() {
    let (_state, app) = setup_test().await;
    let inv = register_overdue(&app, "INV-001", "10000.00", 30).await;

    assert_eq!(inv["invoiceNumber"], "INV-001");
    assert!(inv["outstandingAmount"].as_str().unwrap_or(inv["outstandingAmount"].to_string().as_str()).contains("10000"));
    assert_eq!(inv["overdueDays"], 30);
    assert_eq!(inv["status"], "open");
}

#[tokio::test]
async fn test_list_overdue_invoices() {
    let (_state, app) = setup_test().await;
    register_overdue(&app, "INV-LIST1", "5000.00", 15).await;
    register_overdue(&app, "INV-LIST2", "8000.00", 45).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/interest-invoices/overdue")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(body["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_close_overdue_invoice() {
    let (_state, app) = setup_test().await;
    let inv = register_overdue(&app, "INV-CLOSE", "5000.00", 20).await;
    let inv_id: Uuid = inv["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/interest-invoices/overdue/{}/close", inv_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "closed");
}

// ============================================================================
// Interest Calculation Tests
// ============================================================================

#[tokio::test]
async fn test_calculate_interest() {
    let (_state, app) = setup_test().await;
    create_schedule(&app, "INT-CALC", "Calc Test").await;
    register_overdue(&app, "INV-CALC1", "10000.00", 30).await;
    register_overdue(&app, "INV-CALC2", "5000.00", 60).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/interest-invoices/calculate")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "description": "Monthly interest run",
            "calculation_date": "2024-02-15"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::CREATED);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();

    assert_eq!(body["status"], "calculated");
    assert_eq!(body["totalInvoicesProcessed"], 2);

    // Verify interest was calculated (should be > 0)
    let total_interest: f64 = body["totalInterestCalculated"].as_str().unwrap().parse().unwrap();
    assert!(total_interest > 0.0, "Total interest should be positive, got {}", total_interest);
}

#[tokio::test]
async fn test_list_calculation_runs() {
    let (_state, app) = setup_test().await;
    create_schedule(&app, "INT-RUNS", "Runs Test").await;
    register_overdue(&app, "INV-RUNS", "5000.00", 30).await;

    let (k, v) = auth_header(&admin_claims());
    // Create a run
    app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/interest-invoices/calculate")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "calculation_date": "2024-03-01"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    // List runs
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/interest-invoices/runs")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(body["data"].as_array().unwrap().len() >= 1);
}

#[tokio::test]
async fn test_calculation_lines() {
    let (_state, app) = setup_test().await;
    create_schedule(&app, "INT-LINES", "Lines Test").await;
    register_overdue(&app, "INV-LINES1", "10000.00", 30).await;
    register_overdue(&app, "INV-LINES2", "5000.00", 45).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/interest-invoices/calculate")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "calculation_date": "2024-02-15"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    let run: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let run_id = run["id"].as_str().unwrap();

    // Get calculation lines
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/interest-invoices/runs/{}/lines", run_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["data"].as_array().unwrap().len(), 2);
}

// ============================================================================
// Interest Invoice Generation Tests
// ============================================================================

#[tokio::test]
async fn test_full_interest_invoice_workflow() {
    let (_state, app) = setup_test().await;
    create_schedule(&app, "INT-WORKFLOW", "Workflow Test").await;
    register_overdue(&app, "INV-WF1", "10000.00", 30).await;
    register_overdue(&app, "INV-WF2", "8000.00", 45).await;

    let (k, v) = auth_header(&admin_claims());

    // Calculate interest
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/interest-invoices/calculate")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "calculation_date": "2024-02-15"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let run: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let run_id = run["id"].as_str().unwrap();

    // Generate interest invoices from the run
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/interest-invoices/runs/{}/generate", run_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "invoice_date": "2024-02-15",
            "due_date": "2024-03-15",
            "gl_account_code": "4500-100"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let result: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();

    let invoices = result["data"].as_array().unwrap();
    assert!(!invoices.is_empty(), "Should have generated at least one invoice");

    let invoice = &invoices[0];
    assert_eq!(invoice["status"], "draft");
    let inv_id: Uuid = invoice["id"].as_str().unwrap().parse().unwrap();

    // Check invoice lines
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/interest-invoices/invoices/{}/lines", inv_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let lines: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(lines["data"].as_array().unwrap().len() > 0);

    // Post the invoice
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/interest-invoices/invoices/{}/post", inv_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "posted");

    // Reverse the invoice
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/interest-invoices/invoices/{}/reverse", inv_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "reversed");
}

#[tokio::test]
async fn test_cancel_interest_invoice() {
    let (_state, app) = setup_test().await;
    create_schedule(&app, "INT-CANCEL", "Cancel Test").await;
    register_overdue(&app, "INV-CANCEL", "5000.00", 20).await;

    let (k, v) = auth_header(&admin_claims());

    // Calculate
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/interest-invoices/calculate")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "calculation_date": "2024-03-01"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    let run: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let run_id = run["id"].as_str().unwrap();

    // Generate
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/interest-invoices/runs/{}/generate", run_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "invoice_date": "2024-03-01"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    let result: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let inv_id = result["data"].as_array().unwrap()[0]["id"].as_str().unwrap();

    // Cancel
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/interest-invoices/invoices/{}/cancel", inv_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "cancelled");
}

#[tokio::test]
async fn test_list_interest_invoices() {
    let (_state, app) = setup_test().await;
    create_schedule(&app, "INT-LISTINV", "List Inv Test").await;
    register_overdue(&app, "INV-LISTINV", "5000.00", 20).await;

    let (k, v) = auth_header(&admin_claims());

    // Calculate and generate
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/interest-invoices/calculate")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "calculation_date": "2024-04-01"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    let run: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let run_id = run["id"].as_str().unwrap();

    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/interest-invoices/runs/{}/generate", run_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "invoice_date": "2024-04-01"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    // List invoices
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/interest-invoices/invoices")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(body["data"].as_array().unwrap().len() >= 1);
}

// ============================================================================
// Dashboard Test
// ============================================================================

#[tokio::test]
async fn test_interest_invoice_dashboard() {
    let (_state, app) = setup_test().await;
    create_schedule(&app, "INT-DASH", "Dashboard Test").await;
    register_overdue(&app, "INV-DASH", "5000.00", 30).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/interest-invoices/dashboard")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();

    assert!(body.get("totalActiveSchedules").is_some());
    assert!(body.get("totalOverdueInvoices").is_some());
    assert!(body.get("totalOverdueAmount").is_some());
    assert!(body.get("totalInterestYtd").is_some());
    assert!(body.get("avgOverdueDays").is_some());
}

// ============================================================================
// Validation Edge Cases
// ============================================================================

#[tokio::test]
async fn test_create_schedule_empty_code_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "schedule_code": "",
        "name": "No Code",
        "annual_rate": "12.0",
        "compounding_frequency": "daily",
        "charge_type": "interest",
        "effective_from": "2024-01-01",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/interest-invoices/schedules")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_schedule_invalid_frequency_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "schedule_code": "INT-BAD-FREQ",
        "name": "Bad Frequency",
        "annual_rate": "12.0",
        "compounding_frequency": "hourly",
        "charge_type": "interest",
        "effective_from": "2024-01-01",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/interest-invoices/schedules")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_register_overdue_zero_amount_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "invoice_number": "INV-ZERO",
        "customer_id": "11111111-1111-1111-1111-111111111111",
        "original_amount": "0",
        "outstanding_amount": "0",
        "due_date": "2024-01-15",
        "overdue_days": 30,
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/interest-invoices/overdue")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_register_overdue_zero_days_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "invoice_number": "INV-ZERO-DAYS",
        "customer_id": "11111111-1111-1111-1111-111111111111",
        "original_amount": "1000",
        "outstanding_amount": "1000",
        "due_date": "2024-01-15",
        "overdue_days": 0,
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/interest-invoices/overdue")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_calculate_no_schedule_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/interest-invoices/calculate")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "calculation_date": "2024-02-15"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_reverse_unposted_invoice_fails() {
    let (_state, app) = setup_test().await;
    create_schedule(&app, "INT-REV-FAIL", "Reverse Fail Test").await;
    register_overdue(&app, "INV-REV-FAIL", "5000.00", 30).await;

    let (k, v) = auth_header(&admin_claims());

    // Calculate and generate
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/interest-invoices/calculate")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "calculation_date": "2024-05-01"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    let run: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let run_id = run["id"].as_str().unwrap();

    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/interest-invoices/runs/{}/generate", run_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "invoice_date": "2024-05-01"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    let result: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let inv_id = result["data"].as_array().unwrap()[0]["id"].as_str().unwrap();

    // Try to reverse without posting first
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/interest-invoices/invoices/{}/reverse", inv_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_delete_active_schedule_fails() {
    let (_state, app) = setup_test().await;
    create_schedule(&app, "INT-DEL-ACT", "Delete Active Test").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/interest-invoices/schedules/INT-DEL-ACT")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_cancel_calculation_run() {
    let (_state, app) = setup_test().await;
    create_schedule(&app, "INT-CANCEL-RUN", "Cancel Run Test").await;
    register_overdue(&app, "INV-CANCEL-RUN", "5000.00", 20).await;

    let (k, v) = auth_header(&admin_claims());

    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/interest-invoices/calculate")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "calculation_date": "2024-06-01"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    let run: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let run_id = run["id"].as_str().unwrap();

    // Cancel the run
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/interest-invoices/runs/{}/cancel", run_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "cancelled");
}
