//! Third-Party Payment E2E Tests
//!
//! Tests for third-party payment management (garnishments, tax levies,
//! insurance payments, court orders) on behalf of suppliers/employees.
//! Oracle Fusion: Financials > Payables > Third-Party Payments

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    let migration_sql = include_str!("../../../../migrations/139_third_party_payment.sql");
    sqlx::raw_sql(migration_sql).execute(&state.db_pool).await.ok();
    let app = build_router(state.clone());
    (state, app)
}

// ========================================================================
// Create Payment Tests
// ========================================================================

#[tokio::test]
async fn test_create_garnishment_payment() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/third-party-payments")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "payment_number": "TPP-E2E-001",
            "payment_type": "garnishment",
            "source_entity_type": "supplier",
            "source_entity_id": uuid::Uuid::new_v4().to_string(),
            "source_entity_name": "John Smith",
            "payee_name": "IRS",
            "payee_tax_id": "12-3456789",
            "amount": "5000.00",
            "currency_code": "USD",
            "description": "Wage garnishment for back taxes",
            "case_number": "Case-2024-001",
            "court_jurisdiction": "Federal",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["payment_number"], "TPP-E2E-001");
    assert_eq!(d["payment_type"], "garnishment");
    assert_eq!(d["status"], "draft");
    assert_eq!(d["payee_name"], "IRS");
    assert_eq!(d["amount"], "5000.00");
}

#[tokio::test]
async fn test_create_tax_levy_payment() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/third-party-payments")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "payment_number": "TPP-E2E-TAX",
            "payment_type": "tax_levy",
            "source_entity_type": "employee",
            "source_entity_id": uuid::Uuid::new_v4().to_string(),
            "source_entity_name": "Jane Doe",
            "payee_name": "State Tax Board",
            "amount": "2500.00",
            "currency_code": "USD",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["payment_type"], "tax_levy");
    assert_eq!(d["status"], "draft");
}

#[tokio::test]
async fn test_create_insurance_payment() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/third-party-payments")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "payment_number": "TPP-E2E-INS",
            "payment_type": "insurance",
            "source_entity_type": "supplier",
            "source_entity_id": uuid::Uuid::new_v4().to_string(),
            "source_entity_name": "Acme Corp",
            "payee_name": "Global Insurance Co",
            "amount": "1200.00",
            "currency_code": "USD",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["payment_type"], "insurance");
}

#[tokio::test]
async fn test_create_recurring_payment() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/third-party-payments")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "payment_number": "TPP-E2E-REC",
            "payment_type": "insurance",
            "source_entity_type": "supplier",
            "source_entity_id": uuid::Uuid::new_v4().to_string(),
            "source_entity_name": "Acme Corp",
            "payee_name": "Insurance Provider",
            "amount": "1500.00",
            "currency_code": "USD",
            "is_recurring": true,
            "recurrence_frequency": "monthly",
            "recurrence_start_date": "2025-01-01",
            "recurrence_end_date": "2025-12-31",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["is_recurring"], true);
    assert_eq!(d["recurrence_frequency"], "monthly");
}

#[tokio::test]
async fn test_create_payment_invalid_type() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/third-party-payments")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "payment_number": "TPP-E2E-BAD",
            "payment_type": "bribe",
            "source_entity_type": "supplier",
            "source_entity_id": uuid::Uuid::new_v4().to_string(),
            "source_entity_name": "John",
            "payee_name": "Someone",
            "amount": "100.00",
            "currency_code": "USD",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_payment_zero_amount() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/third-party-payments")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "payment_number": "TPP-E2E-ZERO",
            "payment_type": "garnishment",
            "source_entity_type": "supplier",
            "source_entity_id": uuid::Uuid::new_v4().to_string(),
            "source_entity_name": "John",
            "payee_name": "IRS",
            "amount": "0.00",
            "currency_code": "USD",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_payment_empty_number() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/third-party-payments")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "payment_number": "",
            "payment_type": "garnishment",
            "source_entity_type": "supplier",
            "source_entity_id": uuid::Uuid::new_v4().to_string(),
            "source_entity_name": "John",
            "payee_name": "IRS",
            "amount": "100.00",
            "currency_code": "USD",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_recurring_missing_frequency() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/third-party-payments")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "payment_number": "TPP-E2E-RECERR",
            "payment_type": "insurance",
            "source_entity_type": "supplier",
            "source_entity_id": uuid::Uuid::new_v4().to_string(),
            "source_entity_name": "Acme",
            "payee_name": "Insurer",
            "amount": "100.00",
            "currency_code": "USD",
            "is_recurring": true,
            "recurrence_start_date": "2025-01-01",
            "recurrence_end_date": "2025-12-31",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ========================================================================
// List / Get Tests
// ========================================================================

#[tokio::test]
async fn test_list_payments() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/third-party-payments")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_list_payments_with_filter() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/third-party-payments?status=draft&payment_type=garnishment")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_payment_not_found() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/third-party-payments/{}", uuid::Uuid::new_v4()))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

// ========================================================================
// Workflow Tests
// ========================================================================

#[tokio::test]
async fn test_full_workflow_draft_to_paid() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Create
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/third-party-payments")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "payment_number": "TPP-WF-001",
            "payment_type": "garnishment",
            "source_entity_type": "supplier",
            "source_entity_id": uuid::Uuid::new_v4().to_string(),
            "source_entity_name": "John Smith",
            "payee_name": "IRS",
            "amount": "5000.00",
            "currency_code": "USD",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["status"], "draft");
    let id = d["id"].as_str().unwrap();

    // Submit
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/third-party-payments/{}/submit", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["status"], "submitted");

    // Approve
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/third-party-payments/{}/approve", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["status"], "approved");
    assert!(d["approved_by"].is_string());

    // Record payment
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/third-party-payments/{}/pay", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "payment_reference": "PAY-REF-001"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["status"], "paid");
    assert_eq!(d["payment_reference"], "PAY-REF-001");
}

#[tokio::test]
async fn test_reject_payment() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Create & submit
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/third-party-payments")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "payment_number": "TPP-REJ-001",
            "payment_type": "garnishment",
            "source_entity_type": "supplier",
            "source_entity_id": uuid::Uuid::new_v4().to_string(),
            "source_entity_name": "John",
            "payee_name": "IRS",
            "amount": "5000.00",
            "currency_code": "USD",
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let id = d["id"].as_str().unwrap();

    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/third-party-payments/{}/submit", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Reject
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/third-party-payments/{}/reject", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "reason": "Insufficient documentation"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["status"], "rejected");
}

#[tokio::test]
async fn test_reject_without_reason_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());

    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/third-party-payments")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "payment_number": "TPP-REJ-ERR",
            "payment_type": "garnishment",
            "source_entity_type": "supplier",
            "source_entity_id": uuid::Uuid::new_v4().to_string(),
            "source_entity_name": "John",
            "payee_name": "IRS",
            "amount": "5000.00",
            "currency_code": "USD",
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let id = d["id"].as_str().unwrap();

    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/third-party-payments/{}/submit", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/third-party-payments/{}/reject", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "reason": ""
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_hold_and_release() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Create
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/third-party-payments")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "payment_number": "TPP-HOLD-001",
            "payment_type": "garnishment",
            "source_entity_type": "supplier",
            "source_entity_id": uuid::Uuid::new_v4().to_string(),
            "source_entity_name": "John",
            "payee_name": "IRS",
            "amount": "5000.00",
            "currency_code": "USD",
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let id = d["id"].as_str().unwrap();

    // Hold
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/third-party-payments/{}/hold", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "reason": "Pending court order verification"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["status"], "on_hold");

    // Release
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/third-party-payments/{}/release", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["status"], "submitted");
}

#[tokio::test]
async fn test_cancel_payment() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());

    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/third-party-payments")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "payment_number": "TPP-CAN-001",
            "payment_type": "garnishment",
            "source_entity_type": "supplier",
            "source_entity_id": uuid::Uuid::new_v4().to_string(),
            "source_entity_name": "John",
            "payee_name": "IRS",
            "amount": "5000.00",
            "currency_code": "USD",
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let id = d["id"].as_str().unwrap();

    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/third-party-payments/{}/cancel", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "reason": "No longer required"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["status"], "cancelled");
}

// ========================================================================
// Payment Lines Tests
// ========================================================================

#[tokio::test]
async fn test_add_and_list_lines() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Create payment
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/third-party-payments")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "payment_number": "TPP-LINE-001",
            "payment_type": "garnishment",
            "source_entity_type": "supplier",
            "source_entity_id": uuid::Uuid::new_v4().to_string(),
            "source_entity_name": "John Smith",
            "payee_name": "IRS",
            "amount": "5000.00",
            "currency_code": "USD",
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let payment_id = d["id"].as_str().unwrap();

    // Add principal line
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/third-party-payments/lines")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "payment_id": payment_id,
            "line_number": 1,
            "line_type": "principal",
            "description": "Principal garnishment",
            "amount": "4000.00",
            "gl_account": "2100-100",
            "cost_center": "LEGAL",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["line_type"], "principal");
    assert_eq!(d["amount"], "4000.00");

    // Add interest line
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/third-party-payments/lines")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "payment_id": payment_id,
            "line_number": 2,
            "line_type": "interest",
            "description": "Accrued interest",
            "amount": "500.00",
            "gl_account": "2100-200",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);

    // Add penalty line
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/third-party-payments/lines")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "payment_id": payment_id,
            "line_number": 3,
            "line_type": "penalty",
            "description": "Late payment penalty",
            "amount": "500.00",
            "gl_account": "2100-300",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);

    // List lines
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/third-party-payments/{}/lines", payment_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["data"].as_array().unwrap().len(), 3);
}

#[tokio::test]
async fn test_add_line_invalid_type() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());

    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/third-party-payments")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "payment_number": "TPP-LINE-BAD",
            "payment_type": "garnishment",
            "source_entity_type": "supplier",
            "source_entity_id": uuid::Uuid::new_v4().to_string(),
            "source_entity_name": "John",
            "payee_name": "IRS",
            "amount": "100.00",
            "currency_code": "USD",
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let payment_id = d["id"].as_str().unwrap();

    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/third-party-payments/lines")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "payment_id": payment_id,
            "line_number": 1,
            "line_type": "invalid_type",
            "amount": "50.00",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ========================================================================
// Dashboard Test
// ========================================================================

#[tokio::test]
async fn test_get_dashboard() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/third-party-payments/dashboard")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["total_payments"], 0);
    assert_eq!(d["draft_count"], 0);
}

// ========================================================================
// Workflow Guard Tests
// ========================================================================

#[tokio::test]
async fn test_approve_draft_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());

    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/third-party-payments")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "payment_number": "TPP-APR-DRAFT",
            "payment_type": "garnishment",
            "source_entity_type": "supplier",
            "source_entity_id": uuid::Uuid::new_v4().to_string(),
            "source_entity_name": "John",
            "payee_name": "IRS",
            "amount": "100.00",
            "currency_code": "USD",
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let id = d["id"].as_str().unwrap();

    // Try to approve a draft (should fail)
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/third-party-payments/{}/approve", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_cancel_paid_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Create & full workflow to paid
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/third-party-payments")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "payment_number": "TPP-CAN-PAID",
            "payment_type": "garnishment",
            "source_entity_type": "supplier",
            "source_entity_id": uuid::Uuid::new_v4().to_string(),
            "source_entity_name": "John",
            "payee_name": "IRS",
            "amount": "100.00",
            "currency_code": "USD",
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let id = d["id"].as_str().unwrap();

    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/third-party-payments/{}/submit", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/third-party-payments/{}/approve", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/third-party-payments/{}/pay", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "payment_reference": "REF-001"
        })).unwrap())).unwrap()
    ).await.unwrap();

    // Try to cancel a paid payment (should fail)
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/third-party-payments/{}/cancel", id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "reason": "Changed mind"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_duplicate_payment_number() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let body = serde_json::to_string(&json!({
        "payment_number": "TPP-DUP-E2E",
        "payment_type": "garnishment",
        "source_entity_type": "supplier",
        "source_entity_id": uuid::Uuid::new_v4().to_string(),
        "source_entity_name": "John",
        "payee_name": "IRS",
        "amount": "100.00",
        "currency_code": "USD",
    })).unwrap();

    // First should succeed
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/third-party-payments")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(body.clone()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);

    // Second should conflict
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/third-party-payments")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(body).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CONFLICT);
}
