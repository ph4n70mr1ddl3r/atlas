//! Chargeback Management E2E Tests
//!
//! Tests for Oracle Fusion Cloud ERP Receivables > Chargebacks:
//! - Chargeback CRUD
//! - Full lifecycle (open → under_review → accepted → rejected → written_off)
//! - Chargeback lines (add/remove with total recalculation)
//! - Assignment
//! - Activity audit trail
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
    // Clean chargeback test data
    sqlx::query("DELETE FROM _atlas.chargeback_activities").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM _atlas.chargeback_lines").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM _atlas.chargebacks").execute(&state.db_pool).await.ok();
    // Reset the sequence by truncating
    sqlx::query("TRUNCATE _atlas.chargebacks CASCADE").execute(&state.db_pool).await.ok();
    sqlx::query("CREATE SCHEMA IF NOT EXISTS _atlas")
        .execute(&state.db_pool)
        .await
        .ok();
    sqlx::raw_sql(include_str!("../../../../migrations/128_chargeback_management.sql"))
        .execute(&state.db_pool)
        .await
        .expect("Failed to run chargeback migration");
    let app = build_router(state.clone());
    (state, app)
}

async fn create_chargeback(
    app: &axum::Router,
    amount: f64,
    reason_code: &str,
    customer_name: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "customerName": customer_name,
        "customerNumber": "CUST-001",
        "chargebackDate": "2024-06-15",
        "currencyCode": "USD",
        "amount": amount,
        "taxAmount": 0.0,
        "reasonCode": reason_code,
        "category": "pricing",
        "priority": "medium",
        "reference": "PO-12345",
        "notes": "Test chargeback",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/chargebacks")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8_lossy(&b);
    eprintln!("CREATE CHARGEBACK status={}: {}", status, body_str);
    assert_eq!(status, StatusCode::CREATED, "Failed to create chargeback: {:?}", body_str);
    serde_json::from_slice(&b).unwrap()
}

async fn add_line(
    app: &axum::Router,
    chargeback_id: Uuid,
    amount: f64,
    line_type: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "lineType": line_type,
        "description": format!("Test {} line", line_type),
        "quantity": 1,
        "amount": amount,
        "taxAmount": 0.0,
        "glAccountCode": "4200",
        "glAccountName": "Chargebacks",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/chargebacks/{}/lines", chargeback_id))
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
// CRUD Tests
// ============================================================================

#[tokio::test]
async fn test_create_chargeback() {
    let (_state, app) = setup_test().await;
    let cb = create_chargeback(&app, 5000.00, "pricing_dispute", "Acme Corp").await;

    assert!(cb["chargebackNumber"].as_str().unwrap().starts_with("CB-"));
    assert_eq!(cb["customerName"], "Acme Corp");
    assert_eq!(cb["reasonCode"], "pricing_dispute");
    assert_eq!(cb["category"], "pricing");
    assert_eq!(cb["priority"], "medium");
    assert_eq!(cb["status"], "open");
    assert_eq!(cb["currencyCode"], "USD");

    let amount: f64 = cb["amount"].as_f64().unwrap();
    assert!((amount - 5000.0).abs() < 0.01);
}

#[tokio::test]
async fn test_create_chargeback_with_tax() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "customerName": "Global Inc",
        "chargebackDate": "2024-07-01",
        "currencyCode": "EUR",
        "amount": 10000.0,
        "taxAmount": 2000.0,
        "reasonCode": "damaged_goods",
        "category": "quality",
        "priority": "high",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/chargebacks")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let total: f64 = body["totalAmount"].as_f64().unwrap();
    assert!((total - 12000.0).abs() < 0.01);
    let open: f64 = body["openAmount"].as_f64().unwrap();
    assert!((open - 12000.0).abs() < 0.01);
}

#[tokio::test]
async fn test_get_chargeback() {
    let (_state, app) = setup_test().await;
    let cb = create_chargeback(&app, 3000.00, "short_shipment", "Test Co").await;
    let cb_id: Uuid = cb["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/chargebacks/{}", cb_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["id"], cb["id"]);
    assert_eq!(body["customerName"], "Test Co");
}

#[tokio::test]
async fn test_get_chargeback_by_number() {
    let (_state, app) = setup_test().await;
    let cb = create_chargeback(&app, 1500.00, "quality_issue", "Mega Corp").await;
    let number = cb["chargebackNumber"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/chargebacks/number/{}", number))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["chargebackNumber"], number);
}

#[tokio::test]
async fn test_list_chargebacks() {
    let (_state, app) = setup_test().await;
    create_chargeback(&app, 1000.00, "pricing_dispute", "Customer A").await;
    create_chargeback(&app, 2000.00, "damaged_goods", "Customer B").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/chargebacks")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(body["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_list_chargebacks_filter_by_status() {
    let (_state, app) = setup_test().await;
    let cb = create_chargeback(&app, 5000.00, "pricing_dispute", "Acme Corp").await;
    let cb_id: Uuid = cb["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Move to under_review
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/chargebacks/{}/transition", cb_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "under_review"})).unwrap()))
        .unwrap()
    ).await.unwrap();

    // Filter by open - should not include the moved one
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/chargebacks?status=open")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let data = body["data"].as_array().unwrap();
    assert!(data.iter().all(|b| b["status"] == "open"));

    // Filter by under_review
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/chargebacks?status=under_review")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let data = body["data"].as_array().unwrap();
    assert!(data.iter().all(|b| b["status"] == "under_review"));
    assert!(data.len() >= 1);
}

#[tokio::test]
async fn test_delete_open_chargeback() {
    let (_state, app) = setup_test().await;
    let cb = create_chargeback(&app, 1000.00, "other", "Delete Me").await;
    let number = cb["chargebackNumber"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/chargebacks/number/{}", number))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Lifecycle Tests
// ============================================================================

#[tokio::test]
async fn test_chargeback_full_lifecycle_accepted() {
    let (_state, app) = setup_test().await;
    let cb = create_chargeback(&app, 7500.00, "pricing_dispute", "Acme Corp").await;
    let cb_id: Uuid = cb["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Open -> Under Review
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/chargebacks/{}/transition", cb_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "under_review"})).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "under_review");

    // Under Review -> Accepted
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/chargebacks/{}/transition", cb_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "status": "accepted",
            "resolutionNotes": "Customer provided valid evidence"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "accepted");
    assert!(body["resolutionDate"].is_string());
}

#[tokio::test]
async fn test_chargeback_full_lifecycle_rejected() {
    let (_state, app) = setup_test().await;
    let cb = create_chargeback(&app, 2500.00, "quality_issue", "Reject Corp").await;
    let cb_id: Uuid = cb["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Open -> Under Review
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/chargebacks/{}/transition", cb_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "under_review"})).unwrap()))
        .unwrap()
    ).await.unwrap();

    // Under Review -> Rejected
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/chargebacks/{}/transition", cb_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "status": "rejected",
            "resolutionNotes": "No evidence provided"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "rejected");
}

#[tokio::test]
async fn test_chargeback_write_off_from_open() {
    let (_state, app) = setup_test().await;
    let cb = create_chargeback(&app, 500.00, "other", "WriteOff Corp").await;
    let cb_id: Uuid = cb["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Open -> Written Off
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/chargebacks/{}/transition", cb_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "status": "written_off",
            "resolutionNotes": "Below threshold"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "written_off");
}

#[tokio::test]
async fn test_invalid_transition_accepted_from_open() {
    let (_state, app) = setup_test().await;
    let cb = create_chargeback(&app, 1000.00, "other", "Test").await;
    let cb_id: Uuid = cb["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Try to accept directly from open (must go through under_review first)
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/chargebacks/{}/transition", cb_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "accepted"})).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_invalid_transition_reopen() {
    let (_state, app) = setup_test().await;
    let cb = create_chargeback(&app, 1000.00, "other", "Test").await;
    let cb_id: Uuid = cb["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Move to under_review
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/chargebacks/{}/transition", cb_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "under_review"})).unwrap()))
        .unwrap()
    ).await.unwrap();

    // Try to go back to open
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/chargebacks/{}/transition", cb_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "open"})).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_delete_non_open_fails() {
    let (_state, app) = setup_test().await;
    let cb = create_chargeback(&app, 1000.00, "other", "Test").await;
    let cb_id: Uuid = cb["id"].as_str().unwrap().parse().unwrap();
    let number = cb["chargebackNumber"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Move to under_review
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/chargebacks/{}/transition", cb_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "under_review"})).unwrap()))
        .unwrap()
    ).await.unwrap();

    // Try to delete non-open chargeback
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/chargebacks/number/{}", number))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Line Management Tests
// ============================================================================

#[tokio::test]
async fn test_add_lines_and_totals() {
    let (_state, app) = setup_test().await;
    let cb = create_chargeback(&app, 5000.00, "pricing_dispute", "Acme Corp").await;
    let cb_id: Uuid = cb["id"].as_str().unwrap().parse().unwrap();

    // Add two lines
    let line1 = add_line(&app, cb_id, 3000.00, "chargeback").await;
    let line2 = add_line(&app, cb_id, 2000.00, "tax").await;

    assert_eq!(line1["lineType"], "chargeback");
    assert_eq!(line2["lineType"], "tax");

    // Verify chargeback totals recalculated
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/chargebacks/{}", cb_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();

    let total: f64 = body["totalAmount"].as_f64().unwrap();
    assert!((total - 5000.0).abs() < 0.01, "Expected total 5000, got {}", total);
}

#[tokio::test]
async fn test_remove_line_and_recalc() {
    let (_state, app) = setup_test().await;
    let cb = create_chargeback(&app, 5000.00, "pricing_dispute", "Acme Corp").await;
    let cb_id: Uuid = cb["id"].as_str().unwrap().parse().unwrap();

    let line1 = add_line(&app, cb_id, 3000.00, "chargeback").await;
    let _line2 = add_line(&app, cb_id, 2000.00, "chargeback").await;

    let line1_id: Uuid = line1["id"].as_str().unwrap().parse().unwrap();

    // Remove first line
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/chargebacks/{}/lines/{}", cb_id, line1_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);

    // Verify totals recalculated
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/chargebacks/{}", cb_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();

    let total: f64 = body["totalAmount"].as_f64().unwrap();
    assert!((total - 2000.0).abs() < 0.01, "Expected total 2000 after removal, got {}", total);
}

#[tokio::test]
async fn test_list_lines() {
    let (_state, app) = setup_test().await;
    let cb = create_chargeback(&app, 5000.00, "pricing_dispute", "Acme Corp").await;
    let cb_id: Uuid = cb["id"].as_str().unwrap().parse().unwrap();

    add_line(&app, cb_id, 3000.00, "chargeback").await;
    add_line(&app, cb_id, 1500.00, "discount").await;
    add_line(&app, cb_id, 500.00, "freight").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/chargebacks/{}/lines", cb_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["data"].as_array().unwrap().len(), 3);
}

#[tokio::test]
async fn test_add_line_to_accepted_fails() {
    let (_state, app) = setup_test().await;
    let cb = create_chargeback(&app, 1000.00, "pricing_dispute", "Acme").await;
    let cb_id: Uuid = cb["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Move to under_review -> accepted
    for status in &["under_review", "accepted"] {
        app.clone().oneshot(Request::builder().method("POST")
            .uri(&format!("/api/v1/chargebacks/{}/transition", cb_id))
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({"status": status})).unwrap()))
            .unwrap()
        ).await.unwrap();
    }

    // Try to add line to accepted chargeback
    let payload = json!({"lineType": "chargeback", "amount": 500.0});
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/chargebacks/{}/lines", cb_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Assignment Tests
// ============================================================================

#[tokio::test]
async fn test_assign_chargeback() {
    let (_state, app) = setup_test().await;
    let cb = create_chargeback(&app, 3000.00, "quality_issue", "Test Corp").await;
    let cb_id: Uuid = cb["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/chargebacks/{}/assign", cb_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "assignedTo": "john.doe",
            "assignedTeam": "AR Collections"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["assignedTo"], "john.doe");
    assert_eq!(body["assignedTeam"], "AR Collections");
}

// ============================================================================
// Activity Trail Tests
// ============================================================================

#[tokio::test]
async fn test_activity_trail() {
    let (_state, app) = setup_test().await;
    let cb = create_chargeback(&app, 2000.00, "damaged_goods", "Trail Corp").await;
    let cb_id: Uuid = cb["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Transition through states
    for status in &["under_review", "accepted"] {
        app.clone().oneshot(Request::builder().method("POST")
            .uri(&format!("/api/v1/chargebacks/{}/transition", cb_id))
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(serde_json::to_string(&json!({"status": *status})).unwrap()))
            .unwrap()
        ).await.unwrap();
    }

    // Check activities
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/chargebacks/{}/activities", cb_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();

    let activities = body["data"].as_array().unwrap();
    // Should have: created + under_review + accepted = 3 activities
    assert!(activities.len() >= 3, "Expected at least 3 activities, got {}", activities.len());

    // Verify created activity
    let created = activities.iter().find(|a| a["activityType"] == "created").unwrap();
    assert_eq!(created["newStatus"], "open");

    // Verify under_review activity
    let review = activities.iter().find(|a| a["activityType"] == "status_change_under_review").unwrap();
    assert_eq!(review["oldStatus"], "open");
    assert_eq!(review["newStatus"], "under_review");
}

// ============================================================================
// Dashboard Test
// ============================================================================

#[tokio::test]
async fn test_dashboard() {
    let (_state, app) = setup_test().await;
    create_chargeback(&app, 1000.00, "pricing_dispute", "Cust A").await;
    create_chargeback(&app, 2000.00, "damaged_goods", "Cust B").await;
    let cb3 = create_chargeback(&app, 5000.00, "quality_issue", "Cust C").await;
    let cb3_id: Uuid = cb3["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Move one to under_review
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/chargebacks/{}/transition", cb3_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "under_review"})).unwrap()))
        .unwrap()
    ).await.unwrap();

    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/chargebacks/dashboard")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();

    assert!(body.get("totalChargebacks").is_some());
    assert!(body.get("openCount").is_some());
    assert!(body.get("underReviewCount").is_some());
    assert!(body.get("acceptedCount").is_some());
    assert!(body.get("rejectedCount").is_some());
    assert!(body.get("writtenOffCount").is_some());
    assert!(body.get("totalAmount").is_some());
    assert!(body.get("openAmount").is_some());
    assert!(body.get("byReason").is_some());
    assert!(body.get("byCategory").is_some());
    assert!(body.get("byPriority").is_some());

    assert!(body["totalChargebacks"].as_i64().unwrap() >= 3);
    assert!(body["openCount"].as_i64().unwrap() >= 2);
    assert!(body["underReviewCount"].as_i64().unwrap() >= 1);
}

// ============================================================================
// Validation Edge Cases
// ============================================================================

#[tokio::test]
async fn test_create_chargeback_invalid_reason() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "chargebackDate": "2024-06-15",
        "amount": 1000.0,
        "reasonCode": "invalid_reason",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/chargebacks")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_chargeback_zero_amount_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "chargebackDate": "2024-06-15",
        "amount": 0.0,
        "reasonCode": "pricing_dispute",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/chargebacks")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_chargeback_negative_amount_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "chargebackDate": "2024-06-15",
        "amount": -500.0,
        "reasonCode": "pricing_dispute",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/chargebacks")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_chargeback_not_found() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/chargebacks/{}", Uuid::new_v4()))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_invalid_status_in_transition() {
    let (_state, app) = setup_test().await;
    let cb = create_chargeback(&app, 1000.00, "other", "Test").await;
    let cb_id: Uuid = cb["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/chargebacks/{}/transition", cb_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "invalid_status"})).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Notes Update Test
// ============================================================================

#[tokio::test]
async fn test_update_notes() {
    let (_state, app) = setup_test().await;
    let cb = create_chargeback(&app, 1500.00, "other", "Notes Corp").await;
    let cb_id: Uuid = cb["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("PUT")
        .uri(&format!("/api/v1/chargebacks/{}/notes", cb_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"notes": "Updated notes for investigation"})).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["notes"], "Updated notes for investigation");
}
