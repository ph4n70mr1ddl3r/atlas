//! Bank Guarantee Management E2E Tests
//!
//! Tests for Oracle Fusion Cloud ERP Bank Guarantee Management:
//! - Guarantee CRUD (create, get, list, delete)
//! - Lifecycle workflow (draft → pending_approval → approved → issued → active → released/expired)
//! - Amendment management (create, approve, reject)
//! - Dashboard summary
//! - Validation edge cases
//! - Expiry processing

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
    sqlx::query(include_str!("../../../../migrations/121_bank_guarantee_management.sql"))
        .execute(&state.db_pool)
        .await
        .ok();
    // Clean up bank guarantee test data
    sqlx::query("DELETE FROM fin_bank_guarantee_amendments").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM fin_bank_guarantees").execute(&state.db_pool).await.ok();
    let app = build_router(state.clone());
    (state, app)
}

async fn create_guarantee(
    app: &axum::Router,
    guarantee_number: &str,
    guarantee_type: &str,
    amount: &str,
    beneficiary: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "guaranteeNumber": guarantee_number,
        "guaranteeType": guarantee_type,
        "beneficiaryName": beneficiary,
        "applicantName": "Test Corporation",
        "issuingBankName": "First National Bank",
        "guaranteeAmount": amount,
        "currencyCode": "USD",
        "marginPercentage": "10.00",
        "commissionRate": "1.50",
        "effectiveDate": "2025-01-01",
        "expiryDate": "2025-12-31",
        "issueDate": "2025-01-01",
    });

    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/bank-guarantees")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8_lossy(&b);
    eprintln!("CREATE GUARANTEE RESPONSE status={}: {}", status, body_str);
    assert_eq!(status, StatusCode::CREATED, "Failed to create guarantee: {:?}", body_str);
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// CRUD Tests
// ============================================================================

#[tokio::test]
async fn test_create_guarantee() {
    let (_state, app) = setup_test().await;
    let guarantee = create_guarantee(
        &app, "BG-001", "performance_guarantee", "100000", "Acme Corp"
    ).await;

    assert_eq!(guarantee["guaranteeNumber"], "BG-001");
    assert_eq!(guarantee["guaranteeType"], "performance_guarantee");
    assert_eq!(guarantee["beneficiaryName"], "Acme Corp");
    assert_eq!(guarantee["status"], "draft");
    assert_eq!(guarantee["currencyCode"], "USD");
}

#[tokio::test]
async fn test_create_guarantee_calculates_margin() {
    let (_state, app) = setup_test().await;
    let guarantee = create_guarantee(
        &app, "BG-MARGIN", "bid_bond", "200000", "Client A"
    ).await;

    // 10% margin on 200,000 = 20,000
    let margin: f64 = guarantee["marginAmount"].as_str().unwrap().parse().unwrap();
    assert!((margin - 20000.0).abs() < 0.01);

    // 1.5% commission on 200,000 = 3,000
    let commission: f64 = guarantee["commissionAmount"].as_str().unwrap().parse().unwrap();
    assert!((commission - 3000.0).abs() < 0.01);
}

#[tokio::test]
async fn test_get_guarantee() {
    let (_state, app) = setup_test().await;
    create_guarantee(&app, "BG-GET", "financial_guarantee", "50000", "Beneficiary X").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/bank-guarantes/BG-GET")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["guaranteeNumber"], "BG-GET");
    assert_eq!(body["guaranteeType"], "financial_guarantee");
}

#[tokio::test]
async fn test_list_guarantees() {
    let (_state, app) = setup_test().await;
    create_guarantee(&app, "BG-LIST1", "bid_bond", "10000", "Company A").await;
    create_guarantee(&app, "BG-LIST2", "performance_guarantee", "20000", "Company B").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/bank-guarantees")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(body["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_list_guarantees_with_status_filter() {
    let (_state, app) = setup_test().await;
    create_guarantee(&app, "BG-FILTER", "bid_bond", "15000", "Filter Co").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/bank-guarantees?status=draft")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(body["data"].as_array().unwrap().len() >= 1);
}

#[tokio::test]
async fn test_delete_draft_guarantee() {
    let (_state, app) = setup_test().await;
    create_guarantee(&app, "BG-DEL", "bid_bond", "10000", "Del Co").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/bank-guarantees/BG-DEL")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}

// ============================================================================
// Lifecycle Workflow Tests
// ============================================================================

#[tokio::test]
async fn test_submit_for_approval() {
    let (_state, app) = setup_test().await;
    let guarantee = create_guarantee(
        &app, "BG-SUBMIT", "performance_guarantee", "100000", "Submit Corp"
    ).await;
    let id = guarantee["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/bank-guarantees/{}/submit", id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "pending_approval");
}

#[tokio::test]
async fn test_full_lifecycle_to_active() {
    let (_state, app) = setup_test().await;
    let guarantee = create_guarantee(
        &app, "BG-LIFE", "performance_guarantee", "250000", "Lifecycle Corp"
    ).await;
    let id = guarantee["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Submit
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/bank-guarantees/{}/submit", id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    // Approve
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/bank-guarantees/{}/approve", id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "approved");

    // Issue
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/bank-guarantees/{}/issue", id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(r#"{"issueDate":"2025-01-15"}"#.to_string()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "issued");

    // Activate
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/bank-guarantees/{}/activate", id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "active");
}

#[tokio::test]
async fn test_invoke_and_release() {
    let (_state, app) = setup_test().await;
    let guarantee = create_guarantee(
        &app, "BG-INVOKE", "advance_payment_guarantee", "50000", "Invoke Corp"
    ).await;
    let id = guarantee["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Move to active
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/bank-guarantees/{}/submit", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/bank-guarantees/{}/approve", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/bank-guarantees/{}/issue", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(r#"{"issueDate":"2025-01-01"}"#)).unwrap()
    ).await.unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/bank-guarantees/{}/activate", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Invoke
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/bank-guarantees/{}/invoke", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "invoked");

    // Release
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/bank-guarantees/{}/release", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "released");
}

#[tokio::test]
async fn test_cancel_guarantee() {
    let (_state, app) = setup_test().await;
    let guarantee = create_guarantee(
        &app, "BG-CANCEL", "warranty_guarantee", "30000", "Cancel Corp"
    ).await;
    let id = guarantee["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/bank-guarantees/{}/cancel", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "cancelled");
}

// ============================================================================
// Amendment Tests
// ============================================================================

#[tokio::test]
async fn test_create_amendment() {
    let (_state, app) = setup_test().await;
    let guarantee = create_guarantee(
        &app, "BG-AMD", "performance_guarantee", "100000", "Amendment Corp"
    ).await;
    let id = guarantee["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Move to active first
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/bank-guarantees/{}/submit", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/bank-guarantees/{}/approve", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/bank-guarantees/{}/issue", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(r#"{"issueDate":"2025-01-01"}"#)).unwrap()
    ).await.unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/bank-guarantees/{}/activate", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Create amendment
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/bank-guarantees/{}/amendments", id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "amendmentType": "amount_increase",
            "previousAmount": "100000",
            "newAmount": "150000",
            "reason": "Contract scope increase",
            "effectiveDate": "2025-06-01"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    let status = r.status();
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    eprintln!("CREATE AMENDMENT RESPONSE status={}: {:?}", status, body);
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["amendmentType"], "amount_increase");
    assert!(body["amendmentNumber"].as_str().unwrap().starts_with("AMD-BG-AMD-"));
    assert_eq!(body["status"], "pending_approval");
}

#[tokio::test]
async fn test_list_amendments() {
    let (_state, app) = setup_test().await;
    let guarantee = create_guarantee(
        &app, "BG-AMD-LIST", "bid_bond", "50000", "List Corp"
    ).await;
    let id = guarantee["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Move to active
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/bank-guarantees/{}/submit", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/bank-guarantees/{}/approve", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/bank-guarantees/{}/issue", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(r#"{"issueDate":"2025-01-01"}"#)).unwrap()
    ).await.unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/bank-guarantees/{}/activate", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Create amendment
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/bank-guarantees/{}/amendments", id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "amendmentType": "expiry_extension",
            "previousExpiryDate": "2025-12-31",
            "newExpiryDate": "2026-06-30",
            "reason": "Project extension"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();

    // List amendments
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/bank-guarantees/{}/amendments", id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert!(body["data"].as_array().unwrap().len() >= 1);
}

#[tokio::test]
async fn test_approve_amendment() {
    let (_state, app) = setup_test().await;
    let guarantee = create_guarantee(
        &app, "BG-AMD-APPROVE", "retention_guarantee", "75000", "Approve Corp"
    ).await;
    let id = guarantee["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Move to active
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/bank-guarantees/{}/submit", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/bank-guarantees/{}/approve", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/bank-guarantees/{}/issue", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(r#"{"issueDate":"2025-01-01"}"#)).unwrap()
    ).await.unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/bank-guarantees/{}/activate", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Create amendment
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/bank-guarantees/{}/amendments", id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "amendmentType": "amount_decrease",
            "previousAmount": "75000",
            "newAmount": "50000",
            "reason": "Scope reduction"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    let amendment: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let amendment_id = amendment["id"].as_str().unwrap();

    // Approve amendment
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/bank-guarantees/amendments/{}/approve", amendment_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "applied");
}

#[tokio::test]
async fn test_reject_amendment() {
    let (_state, app) = setup_test().await;
    let guarantee = create_guarantee(
        &app, "BG-AMD-REJECT", "customs_guarantee", "200000", "Reject Corp"
    ).await;
    let id = guarantee["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Move to active
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/bank-guarantees/{}/submit", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/bank-guarantees/{}/approve", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/bank-guarantees/{}/issue", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(r#"{"issueDate":"2025-01-01"}"#)).unwrap()
    ).await.unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/bank-guarantees/{}/activate", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Create amendment
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/bank-guarantees/{}/amendments", id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "amendmentType": "terms_change",
            "previousTerms": "Net 30",
            "newTerms": "Net 60",
            "reason": "Requested by beneficiary"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    let amendment: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let amendment_id = amendment["id"].as_str().unwrap();

    // Reject amendment
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/bank-guarantees/amendments/{}/reject", amendment_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "rejected");
}

// ============================================================================
// Dashboard Test
// ============================================================================

#[tokio::test]
async fn test_bank_guarantee_dashboard() {
    let (_state, app) = setup_test().await;
    create_guarantee(&app, "BG-DASH1", "performance_guarantee", "100000", "Dash Corp").await;
    create_guarantee(&app, "BG-DASH2", "bid_bond", "50000", "Dash Corp 2").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/bank-guarantees/dashboard")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();

    assert!(body.get("totalGuarantees").is_some());
    assert!(body.get("activeGuarantees").is_some());
    assert!(body.get("totalGuaranteeAmount").is_some());
    assert!(body.get("totalMarginHeld").is_some());
    assert!(body.get("expiringWithin30Days").is_some());
    assert!(body.get("expiringWithin90Days").is_some());
    assert!(body.get("pendingApproval").is_some());
    assert!(body.get("amendmentsPending").is_some());
    assert!(body.get("byType").is_some());
    assert!(body.get("byCurrency").is_some());
    // Two draft guarantees
    assert!(body["totalGuarantees"].as_i64().unwrap() >= 2);
}

// ============================================================================
// Validation Edge Cases
// ============================================================================

#[tokio::test]
async fn test_create_guarantee_empty_number_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "guaranteeNumber": "",
        "guaranteeType": "bid_bond",
        "beneficiaryName": "Test",
        "applicantName": "Test",
        "issuingBankName": "Test Bank",
        "guaranteeAmount": "10000",
        "currencyCode": "USD",
        "marginPercentage": "10",
        "commissionRate": "1.5",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/bank-guarantees")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_guarantee_invalid_type_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "guaranteeNumber": "BG-BAD-TYPE",
        "guaranteeType": "invalid_type",
        "beneficiaryName": "Test",
        "applicantName": "Test",
        "issuingBankName": "Test Bank",
        "guaranteeAmount": "10000",
        "currencyCode": "USD",
        "marginPercentage": "10",
        "commissionRate": "1.5",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/bank-guarantees")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_guarantee_zero_amount_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "guaranteeNumber": "BG-ZERO",
        "guaranteeType": "bid_bond",
        "beneficiaryName": "Test",
        "applicantName": "Test",
        "issuingBankName": "Test Bank",
        "guaranteeAmount": "0",
        "currencyCode": "USD",
        "marginPercentage": "10",
        "commissionRate": "1.5",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/bank-guarantees")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_guarantee_invalid_collateral_type_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "guaranteeNumber": "BG-BAD-COLL",
        "guaranteeType": "bid_bond",
        "beneficiaryName": "Test",
        "applicantName": "Test",
        "issuingBankName": "Test Bank",
        "guaranteeAmount": "10000",
        "currencyCode": "USD",
        "marginPercentage": "10",
        "commissionRate": "1.5",
        "collateralType": "nonexistent",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/bank-guarantees")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_submit_non_draft_fails() {
    let (_state, app) = setup_test().await;
    let guarantee = create_guarantee(
        &app, "BG-DBL-SUB", "bid_bond", "10000", "Test"
    ).await;
    let id = guarantee["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Submit once
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/bank-guarantees/{}/submit", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Submit again - should fail
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/bank-guarantees/{}/submit", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_delete_non_draft_fails() {
    let (_state, app) = setup_test().await;
    let guarantee = create_guarantee(
        &app, "BG-DEL-ND", "bid_bond", "10000", "Test"
    ).await;
    let id = guarantee["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Submit it first
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/bank-guarantees/{}/submit", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Now try to delete
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/bank-guarantees/BG-DEL-ND")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_approve_from_draft_fails() {
    let (_state, app) = setup_test().await;
    let guarantee = create_guarantee(
        &app, "BG-APPR-DRAFT", "bid_bond", "10000", "Test"
    ).await;
    let id = guarantee["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Approve directly from draft - should fail
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/bank-guarantees/{}/approve", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_amendment_on_draft_fails() {
    let (_state, app) = setup_test().await;
    let guarantee = create_guarantee(
        &app, "BG-AMD-DRAFT", "bid_bond", "10000", "Test"
    ).await;
    let id = guarantee["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Try to create amendment on draft guarantee - should fail
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/bank-guarantees/{}/amendments", id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "amendmentType": "amount_increase",
            "previousAmount": "10000",
            "newAmount": "20000",
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_expiry_date_before_issue_date_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "guaranteeNumber": "BG-BAD-DATE",
        "guaranteeType": "bid_bond",
        "beneficiaryName": "Test",
        "applicantName": "Test",
        "issuingBankName": "Test Bank",
        "guaranteeAmount": "10000",
        "currencyCode": "USD",
        "marginPercentage": "10",
        "commissionRate": "1.5",
        "issueDate": "2025-12-31",
        "expiryDate": "2025-01-01",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/bank-guarantees")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_margin_over_100_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "guaranteeNumber": "BG-HIGH-MARGIN",
        "guaranteeType": "bid_bond",
        "beneficiaryName": "Test",
        "applicantName": "Test",
        "issuingBankName": "Test Bank",
        "guaranteeAmount": "10000",
        "currencyCode": "USD",
        "marginPercentage": "150",
        "commissionRate": "1.5",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/bank-guarantees")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}
