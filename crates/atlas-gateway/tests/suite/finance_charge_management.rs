//! Finance Charge Management E2E Tests
//!
//! Tests for Oracle Fusion Cloud ERP Receivables > Finance Charges:
//! - Finance charge term CRUD (percentage, flat fee, tiered)
//! - Assessment run lifecycle (draft → submitted → approved → applied → cancelled)
//! - Charge line management (add, waive, cancel)
//! - Charge invoice generation
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
    // Clean finance charge test data
    sqlx::query("DELETE FROM _atlas.finance_charge_activities").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM _atlas.finance_charge_lines").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM _atlas.finance_charge_invoices").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM _atlas.finance_charge_runs").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM _atlas.finance_charge_tiers").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM _atlas.finance_charge_terms").execute(&state.db_pool).await.ok();
    sqlx::query("CREATE SCHEMA IF NOT EXISTS _atlas")
        .execute(&state.db_pool)
        .await
        .ok();
    sqlx::raw_sql(include_str!("../../../../migrations/147_finance_charge_management.sql"))
        .execute(&state.db_pool)
        .await
        .expect("Failed to run finance charge migration");
    let app = build_router(state.clone());
    (state, app)
}

async fn create_term(
    app: &axum::Router,
    term_code: &str,
    term_name: &str,
    charge_type: &str,
    charge_rate: Option<f64>,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let mut payload = json!({
        "termCode": term_code,
        "termName": term_name,
        "chargeType": charge_type,
        "gracePeriodDays": 30,
        "currencyCode": "USD",
        "calculationBasis": "monthly",
    });
    if let Some(rate) = charge_rate {
        payload["chargeRate"] = json!(rate);
    }
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/finance-charges/terms")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8_lossy(&b);
    eprintln!("CREATE TERM status={}: {}", status, body_str);
    assert_eq!(status, StatusCode::CREATED, "Failed to create term: {:?}", body_str);
    serde_json::from_slice(&b).unwrap()
}

async fn create_run(
    app: &axum::Router,
    run_date: &str,
    term_code: Option<&str>,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let mut payload = json!({
        "runDate": run_date,
        "currencyCode": "USD",
    });
    if let Some(tc) = term_code {
        payload["termCode"] = json!(tc);
    }
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/finance-charges/runs")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8_lossy(&b);
    eprintln!("CREATE RUN status={}: {}", status, body_str);
    assert_eq!(status, StatusCode::CREATED, "Failed to create run: {:?}", body_str);
    serde_json::from_slice(&b).unwrap()
}

async fn add_line(
    app: &axum::Router,
    run_id: Uuid,
    customer_name: &str,
    invoice_number: &str,
    days_overdue: i32,
    outstanding: f64,
    charge_amount: f64,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "customerName": customer_name,
        "invoiceNumber": invoice_number,
        "invoiceDate": "2024-01-15",
        "invoiceDueDate": "2024-02-15",
        "daysOverdue": days_overdue,
        "invoiceAmount": outstanding,
        "outstandingAmount": outstanding,
        "chargeType": "percentage",
        "chargeRate": 1.5,
        "chargeAmount": charge_amount,
        "currencyCode": "USD",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/finance-charges/runs/{}/lines", run_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8_lossy(&b);
    eprintln!("ADD LINE status={}: {}", status, body_str);
    assert_eq!(status, StatusCode::CREATED, "Failed to add line: {:?}", body_str);
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Term CRUD Tests
// ============================================================================

#[tokio::test]
async fn test_create_percentage_term() {
    let (_state, app) = setup_test().await;
    let term = create_term(&app, "FC-STD", "Standard Finance Charge", "percentage", Some(1.5)).await;

    assert_eq!(term["termCode"], "FC-STD");
    assert_eq!(term["termName"], "Standard Finance Charge");
    assert_eq!(term["chargeType"], "percentage");
    let rate: f64 = term["chargeRate"].as_f64().unwrap();
    assert!((rate - 1.5).abs() < 0.001);
    assert_eq!(term["gracePeriodDays"], 30);
    assert_eq!(term["currencyCode"], "USD");
    assert_eq!(term["calculationBasis"], "monthly");
    assert_eq!(term["isActive"], true);
}

#[tokio::test]
async fn test_create_flat_fee_term() {
    let (_state, app) = setup_test().await;
    let term = create_term(&app, "FC-FLAT", "Flat Fee Charge", "flat_fee", None).await;

    assert_eq!(term["chargeType"], "flat_fee");
}

#[tokio::test]
async fn test_create_tiered_term() {
    let (_state, app) = setup_test().await;
    let term = create_term(&app, "FC-TIER", "Tiered Charge", "tiered", None).await;

    assert_eq!(term["chargeType"], "tiered");
}

#[tokio::test]
async fn test_get_term() {
    let (_state, app) = setup_test().await;
    let term = create_term(&app, "FC-GET", "Get Test", "percentage", Some(2.0)).await;
    let term_id: Uuid = term["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/finance-charges/terms/{}", term_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["termCode"], "FC-GET");
}

#[tokio::test]
async fn test_list_terms() {
    let (_state, app) = setup_test().await;
    create_term(&app, "FC-L1", "Term 1", "percentage", Some(1.0)).await;
    create_term(&app, "FC-L2", "Term 2", "flat_fee", None).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/finance-charges/terms")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(body["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_create_term_invalid_charge_type() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "termCode": "FC-BAD",
        "termName": "Bad Type",
        "chargeType": "invalid",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/finance-charges/terms")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_term_percentage_requires_rate() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "termCode": "FC-NORATE",
        "termName": "No Rate",
        "chargeType": "percentage",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/finance-charges/terms")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Assessment Run Tests
// ============================================================================

#[tokio::test]
async fn test_create_run() {
    let (_state, app) = setup_test().await;
    let run = create_run(&app, "2024-06-30", None).await;

    assert!(run["runNumber"].as_str().unwrap().starts_with("FCR-"));
    assert_eq!(run["status"], "draft");
    assert_eq!(run["totalInvoicesAssessed"], 0);
    let total: f64 = run["totalChargesAssessed"].as_f64().unwrap();
    assert!(total.abs() < 0.01);
}

#[tokio::test]
async fn test_get_run() {
    let (_state, app) = setup_test().await;
    let run = create_run(&app, "2024-06-30", None).await;
    let run_id: Uuid = run["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/finance-charges/runs/{}", run_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_run_by_number() {
    let (_state, app) = setup_test().await;
    let run = create_run(&app, "2024-06-30", None).await;
    let number = run["runNumber"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/finance-charges/runs/number/{}", number))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["runNumber"], number);
}

#[tokio::test]
async fn test_list_runs() {
    let (_state, app) = setup_test().await;
    create_run(&app, "2024-06-30", None).await;
    create_run(&app, "2024-07-31", None).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/finance-charges/runs")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(body["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_delete_draft_run() {
    let (_state, app) = setup_test().await;
    let run = create_run(&app, "2024-06-30", None).await;
    let number = run["runNumber"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/finance-charges/runs/number/{}", number))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_delete_non_draft_run_fails() {
    let (_state, app) = setup_test().await;
    let run = create_run(&app, "2024-06-30", None).await;
    let run_id: Uuid = run["id"].as_str().unwrap().parse().unwrap();
    let number = run["runNumber"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Move to submitted
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/finance-charges/runs/{}/transition", run_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "submitted"})).unwrap()))
        .unwrap()
    ).await.unwrap();

    // Try to delete submitted run
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/finance-charges/runs/number/{}", number))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Run Lifecycle Tests
// ============================================================================

#[tokio::test]
async fn test_run_full_lifecycle_submitted_approved_applied() {
    let (_state, app) = setup_test().await;
    let run = create_run(&app, "2024-06-30", None).await;
    let run_id: Uuid = run["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Draft → Submitted
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/finance-charges/runs/{}/transition", run_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "submitted"})).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "submitted");

    // Submitted → Approved
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/finance-charges/runs/{}/transition", run_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "approved"})).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "approved");

    // Add a line (should fail since run is approved, not draft)
    // This validates that lines can only be added in draft status
    let payload = json!({
        "customerName": "Test",
        "invoiceNumber": "INV-001",
        "daysOverdue": 45,
        "invoiceAmount": 10000.0,
        "outstandingAmount": 10000.0,
        "chargeAmount": 150.0,
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/finance-charges/runs/{}/lines", run_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_run_cancel_from_draft() {
    let (_state, app) = setup_test().await;
    let run = create_run(&app, "2024-06-30", None).await;
    let run_id: Uuid = run["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/finance-charges/runs/{}/transition", run_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "cancelled"})).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "cancelled");
}

#[tokio::test]
async fn test_invalid_run_transition() {
    let (_state, app) = setup_test().await;
    let run = create_run(&app, "2024-06-30", None).await;
    let run_id: Uuid = run["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Draft → Applied (invalid, must go through submitted → approved first)
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/finance-charges/runs/{}/transition", run_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "applied"})).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_invalid_run_transition_status() {
    let (_state, app) = setup_test().await;
    let run = create_run(&app, "2024-06-30", None).await;
    let run_id: Uuid = run["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/finance-charges/runs/{}/transition", run_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "invalid_status"})).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Charge Line Tests
// ============================================================================

#[tokio::test]
async fn test_add_charge_line_and_totals() {
    let (_state, app) = setup_test().await;
    let run = create_run(&app, "2024-06-30", None).await;
    let run_id: Uuid = run["id"].as_str().unwrap().parse().unwrap();

    let line1 = add_line(&app, run_id, "Acme Corp", "INV-001", 45, 10000.0, 150.0).await;
    let line2 = add_line(&app, run_id, "Beta Inc", "INV-002", 60, 5000.0, 100.0).await;

    assert_eq!(line1["customerName"], "Acme Corp");
    assert_eq!(line1["daysOverdue"], 45);
    assert_eq!(line1["status"], "pending");

    assert_eq!(line2["customerName"], "Beta Inc");

    // Verify run totals updated
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/finance-charges/runs/{}", run_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();

    assert_eq!(body["totalInvoicesAssessed"], 2);
    let total: f64 = body["totalChargesAssessed"].as_f64().unwrap();
    assert!((total - 250.0).abs() < 0.01, "Expected total charges 250.0, got {}", total);
}

#[tokio::test]
async fn test_list_lines() {
    let (_state, app) = setup_test().await;
    let run = create_run(&app, "2024-06-30", None).await;
    let run_id: Uuid = run["id"].as_str().unwrap().parse().unwrap();

    add_line(&app, run_id, "Cust A", "INV-1", 30, 5000.0, 75.0).await;
    add_line(&app, run_id, "Cust B", "INV-2", 45, 3000.0, 67.5).await;
    add_line(&app, run_id, "Cust C", "INV-3", 60, 2000.0, 60.0).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/finance-charges/runs/{}/lines", run_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["data"].as_array().unwrap().len(), 3);
}

#[tokio::test]
async fn test_waive_line() {
    let (_state, app) = setup_test().await;
    let run = create_run(&app, "2024-06-30", None).await;
    let run_id: Uuid = run["id"].as_str().unwrap().parse().unwrap();

    let line = add_line(&app, run_id, "Waive Corp", "INV-W", 30, 5000.0, 75.0).await;
    let line_id: Uuid = line["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/finance-charges/lines/{}/waive", line_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"reason": "Long-standing customer, one-time courtesy"})).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "waived");
    assert_eq!(body["waivedReason"], "Long-standing customer, one-time courtesy");
}

#[tokio::test]
async fn test_waive_line_without_reason_fails() {
    let (_state, app) = setup_test().await;
    let run = create_run(&app, "2024-06-30", None).await;
    let run_id: Uuid = run["id"].as_str().unwrap().parse().unwrap();

    let line = add_line(&app, run_id, "Test", "INV-T", 30, 5000.0, 75.0).await;
    let line_id: Uuid = line["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/finance-charges/lines/{}/waive", line_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"reason": ""})).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_add_line_negative_charge_fails() {
    let (_state, app) = setup_test().await;
    let run = create_run(&app, "2024-06-30", None).await;
    let run_id: Uuid = run["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "customerName": "Test",
        "invoiceNumber": "INV-NEG",
        "daysOverdue": 30,
        "invoiceAmount": 5000.0,
        "outstandingAmount": 5000.0,
        "chargeAmount": -100.0,
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/finance-charges/runs/{}/lines", run_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Invoice Generation Tests
// ============================================================================

#[tokio::test]
async fn test_generate_invoices_full_flow() {
    let (_state, app) = setup_test().await;
    let run = create_run(&app, "2024-06-30", None).await;
    let run_id: Uuid = run["id"].as_str().unwrap().parse().unwrap();

    // Add lines for two customers
    add_line(&app, run_id, "Acme Corp", "INV-A1", 45, 10000.0, 150.0).await;
    add_line(&app, run_id, "Acme Corp", "INV-A2", 60, 5000.0, 100.0).await;
    add_line(&app, run_id, "Beta Inc", "INV-B1", 30, 3000.0, 45.0).await;

    let (k, v) = auth_header(&admin_claims());

    // Draft → Submitted
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/finance-charges/runs/{}/transition", run_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "submitted"})).unwrap()))
        .unwrap()
    ).await.unwrap();

    // Submitted → Approved
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/finance-charges/runs/{}/transition", run_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "approved"})).unwrap()))
        .unwrap()
    ).await.unwrap();

    // Generate invoices
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/finance-charges/runs/{}/generate-invoices", run_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();

    let invoices = body["data"].as_array().unwrap();
    // Should have 2 invoices: one for Acme Corp (combined), one for Beta Inc
    assert_eq!(invoices.len(), 2, "Expected 2 invoices (one per customer), got {}", invoices.len());

    // Find Acme invoice
    let acme_inv = invoices.iter().find(|i| i["customerName"] == "Acme Corp").unwrap();
    let acme_total: f64 = acme_inv["totalChargeAmount"].as_f64().unwrap();
    assert!((acme_total - 250.0).abs() < 0.01, "Acme total should be 250.0, got {}", acme_total);
    assert!(acme_inv["chargeInvoiceNumber"].as_str().unwrap().starts_with("FCI-"));
    assert_eq!(acme_inv["status"], "open");

    // Find Beta invoice
    let beta_inv = invoices.iter().find(|i| i["customerName"] == "Beta Inc").unwrap();
    let beta_total: f64 = beta_inv["totalChargeAmount"].as_f64().unwrap();
    assert!((beta_total - 45.0).abs() < 0.01, "Beta total should be 45.0, got {}", beta_total);

    // Verify run transitioned to applied
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/finance-charges/runs/{}", run_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    let run_body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(run_body["status"], "applied");
}

#[tokio::test]
async fn test_generate_invoices_only_for_approved_run() {
    let (_state, app) = setup_test().await;
    let run = create_run(&app, "2024-06-30", None).await;
    let run_id: Uuid = run["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Try to generate invoices from draft run
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/finance-charges/runs/{}/generate-invoices", run_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Invoice Lifecycle Tests
// ============================================================================

#[tokio::test]
async fn test_invoice_transition_to_paid() {
    let (_state, app) = setup_test().await;
    let run = create_run(&app, "2024-06-30", None).await;
    let run_id: Uuid = run["id"].as_str().unwrap().parse().unwrap();

    add_line(&app, run_id, "Pay Corp", "INV-P", 30, 5000.0, 75.0).await;

    let (k, v) = auth_header(&admin_claims());

    // Approve run
    for status in &["submitted", "approved"] {
        app.clone().oneshot(Request::builder().method("POST")
            .uri(&format!("/api/v1/finance-charges/runs/{}/transition", run_id))
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({"status": *status})).unwrap()))
            .unwrap()
        ).await.unwrap();
    }

    // Generate invoices
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/finance-charges/runs/{}/generate-invoices", run_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let invoice = &body["data"].as_array().unwrap()[0];
    let inv_id: Uuid = invoice["id"].as_str().unwrap().parse().unwrap();

    // Transition to paid
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/finance-charges/invoices/{}/transition", inv_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "paid"})).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "paid");
}

#[tokio::test]
async fn test_invoice_cancel() {
    let (_state, app) = setup_test().await;
    let run = create_run(&app, "2024-06-30", None).await;
    let run_id: Uuid = run["id"].as_str().unwrap().parse().unwrap();

    add_line(&app, run_id, "Cancel Corp", "INV-C", 30, 5000.0, 75.0).await;

    let (k, v) = auth_header(&admin_claims());

    for status in &["submitted", "approved"] {
        app.clone().oneshot(Request::builder().method("POST")
            .uri(&format!("/api/v1/finance-charges/runs/{}/transition", run_id))
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({"status": *status})).unwrap()))
            .unwrap()
        ).await.unwrap();
    }

    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/finance-charges/runs/{}/generate-invoices", run_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let inv_id: Uuid = body["data"].as_array().unwrap()[0]["id"].as_str().unwrap().parse().unwrap();

    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/finance-charges/invoices/{}/transition", inv_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "cancelled"})).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "cancelled");
}

#[tokio::test]
async fn test_list_invoices() {
    let (_state, app) = setup_test().await;
    let run = create_run(&app, "2024-06-30", None).await;
    let run_id: Uuid = run["id"].as_str().unwrap().parse().unwrap();

    add_line(&app, run_id, "List Corp", "INV-L", 30, 5000.0, 75.0).await;

    let (k, v) = auth_header(&admin_claims());

    for status in &["submitted", "approved"] {
        app.clone().oneshot(Request::builder().method("POST")
            .uri(&format!("/api/v1/finance-charges/runs/{}/transition", run_id))
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({"status": *status})).unwrap()))
            .unwrap()
        ).await.unwrap();
    }

    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/finance-charges/runs/{}/generate-invoices", run_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/finance-charges/invoices")
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
async fn test_dashboard() {
    let (_state, app) = setup_test().await;
    create_term(&app, "FC-DASH", "Dashboard Term", "percentage", Some(1.5)).await;
    create_run(&app, "2024-06-30", None).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/finance-charges/dashboard")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8_lossy(&b);
    eprintln!("DASHBOARD status={}: {}", status, body_str);
    assert_eq!(status, StatusCode::OK, "Dashboard failed: {:?}", body_str);
    let body: serde_json::Value = serde_json::from_slice(&b).unwrap();

    assert!(body.get("totalTerms").is_some());
    assert!(body.get("activeTerms").is_some());
    assert!(body.get("totalRuns").is_some());
    assert!(body.get("pendingRuns").is_some());
    assert!(body.get("completedRuns").is_some());
    assert!(body.get("totalChargesAssessed").is_some());
    assert!(body.get("totalChargesCollected").is_some());
    assert!(body.get("totalChargesOutstanding").is_some());
    assert!(body.get("byChargeType").is_some());
    assert!(body.get("byStatus").is_some());

    assert!(body["totalTerms"].as_i64().unwrap() >= 1);
    assert!(body["activeTerms"].as_i64().unwrap() >= 1);
    assert!(body["totalRuns"].as_i64().unwrap() >= 1);
}

// ============================================================================
// Not Found Tests
// ============================================================================

#[tokio::test]
async fn test_get_term_not_found() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/finance-charges/terms/{}", Uuid::new_v4()))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_run_not_found() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/finance-charges/runs/{}", Uuid::new_v4()))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_invoice_not_found() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/finance-charges/invoices/{}", Uuid::new_v4()))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}
