//! Invoice Matching E2E Tests
//!
//! Tests for 2-way, 3-way, and 4-way invoice matching
//! Oracle Fusion: Financials > Payables > Invoice Matching

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    let migration_sql = include_str!("../../../../migrations/136_invoice_matching.sql");
    sqlx::raw_sql(migration_sql).execute(&state.db_pool).await.ok();
    let app = build_router(state.clone());
    (state, app)
}

// ========================================================================
// Create Match Tests
// ========================================================================

#[tokio::test]
async fn test_create_two_way_match() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/invoice-matches")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "match_number": "IM-E2E-001",
            "invoice_id": uuid::Uuid::new_v4().to_string(),
            "purchase_order_id": uuid::Uuid::new_v4().to_string(),
            "supplier_id": uuid::Uuid::new_v4().to_string(),
            "supplier_name": "Acme Corporation",
            "match_type": "two_way",
            "invoice_amount": "10000.00",
            "po_amount": "10000.00",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["match_number"], "IM-E2E-001");
    assert_eq!(d["match_type"], "two_way");
    assert_eq!(d["status"], "matched");
}

#[tokio::test]
async fn test_create_two_way_match_exception() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/invoice-matches")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "match_number": "IM-E2E-EXC",
            "invoice_id": uuid::Uuid::new_v4().to_string(),
            "purchase_order_id": uuid::Uuid::new_v4().to_string(),
            "supplier_id": uuid::Uuid::new_v4().to_string(),
            "supplier_name": "Acme Corporation",
            "match_type": "two_way",
            "invoice_amount": "50000.00",
            "po_amount": "10000.00",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["status"], "exception");
}

#[tokio::test]
async fn test_create_three_way_match() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/invoice-matches")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "match_number": "IM-E2E-3W",
            "invoice_id": uuid::Uuid::new_v4().to_string(),
            "purchase_order_id": uuid::Uuid::new_v4().to_string(),
            "supplier_id": uuid::Uuid::new_v4().to_string(),
            "supplier_name": "Global Supplies Inc",
            "match_type": "three_way",
            "invoice_amount": "25000.00",
            "po_amount": "25000.00",
            "receipt_amount": "25000.00",
            "receipt_number": "RCT-001",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["match_type"], "three_way");
    assert_eq!(d["status"], "matched");
}

#[tokio::test]
async fn test_create_four_way_match() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/invoice-matches")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "match_number": "IM-E2E-4W",
            "invoice_id": uuid::Uuid::new_v4().to_string(),
            "purchase_order_id": uuid::Uuid::new_v4().to_string(),
            "supplier_id": uuid::Uuid::new_v4().to_string(),
            "supplier_name": "Quality Parts Ltd",
            "match_type": "four_way",
            "invoice_amount": "75000.00",
            "po_amount": "75000.00",
            "receipt_amount": "75000.00",
            "inspection_amount": "75000.00",
            "receipt_number": "RCT-002",
            "inspection_status": "passed",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["match_type"], "four_way");
    assert_eq!(d["status"], "matched");
}

#[tokio::test]
async fn test_create_match_invalid_type() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/invoice-matches")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "match_number": "IM-E2E-BAD",
            "invoice_id": uuid::Uuid::new_v4().to_string(),
            "purchase_order_id": uuid::Uuid::new_v4().to_string(),
            "supplier_id": uuid::Uuid::new_v4().to_string(),
            "supplier_name": "Acme Corp",
            "match_type": "five_way",
            "invoice_amount": "10000.00",
            "po_amount": "10000.00",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_match_negative_amount() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/invoice-matches")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "match_number": "IM-E2E-NEG",
            "invoice_id": uuid::Uuid::new_v4().to_string(),
            "purchase_order_id": uuid::Uuid::new_v4().to_string(),
            "supplier_id": uuid::Uuid::new_v4().to_string(),
            "supplier_name": "Acme Corp",
            "match_type": "two_way",
            "invoice_amount": "-500.00",
            "po_amount": "10000.00",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_three_way_missing_receipt() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/invoice-matches")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "match_number": "IM-E2E-3WN",
            "invoice_id": uuid::Uuid::new_v4().to_string(),
            "purchase_order_id": uuid::Uuid::new_v4().to_string(),
            "supplier_id": uuid::Uuid::new_v4().to_string(),
            "supplier_name": "Acme Corp",
            "match_type": "three_way",
            "invoice_amount": "10000.00",
            "po_amount": "10000.00",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ========================================================================
// List / Get Tests
// ========================================================================

#[tokio::test]
async fn test_list_matches() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/invoice-matches")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_list_matches_with_filter() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/invoice-matches?status=matched&match_type=two_way")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_match_not_found() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/invoice-matches/{}", uuid::Uuid::new_v4()))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

// ========================================================================
// Workflow Tests
// ========================================================================

#[tokio::test]
async fn test_hold_and_override_exception_match() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Create exception match
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/invoice-matches")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "match_number": "IM-E2E-HOV",
            "invoice_id": uuid::Uuid::new_v4().to_string(),
            "purchase_order_id": uuid::Uuid::new_v4().to_string(),
            "supplier_id": uuid::Uuid::new_v4().to_string(),
            "supplier_name": "Acme Corp",
            "match_type": "two_way",
            "invoice_amount": "50000.00",
            "po_amount": "10000.00",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["status"], "exception");
    let match_id = d["id"].as_str().unwrap();

    // Hold
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/invoice-matches/{}/hold", match_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "reason": "Price variance too high"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    // Override
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/invoice-matches/{}/override", match_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "reason": "Approved by procurement director"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["status"], "overridden");
}

#[tokio::test]
async fn test_cancel_exception_match() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Create exception match
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/invoice-matches")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "match_number": "IM-E2E-CAN",
            "invoice_id": uuid::Uuid::new_v4().to_string(),
            "purchase_order_id": uuid::Uuid::new_v4().to_string(),
            "supplier_id": uuid::Uuid::new_v4().to_string(),
            "supplier_name": "Acme Corp",
            "match_type": "two_way",
            "invoice_amount": "50000.00",
            "po_amount": "10000.00",
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let match_id = d["id"].as_str().unwrap();

    // Cancel
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/invoice-matches/{}/cancel", match_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "reason": "Invoice disputed"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["status"], "cancelled");
}

#[tokio::test]
async fn test_override_without_reason_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());

    // Create exception match
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/invoice-matches")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "match_number": "IM-E2E-NOREASON",
            "invoice_id": uuid::Uuid::new_v4().to_string(),
            "purchase_order_id": uuid::Uuid::new_v4().to_string(),
            "supplier_id": uuid::Uuid::new_v4().to_string(),
            "supplier_name": "Acme Corp",
            "match_type": "two_way",
            "invoice_amount": "50000.00",
            "po_amount": "10000.00",
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let match_id = d["id"].as_str().unwrap();

    // Override with empty reason
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/invoice-matches/{}/override", match_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "reason": ""
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
        .uri("/api/v1/invoice-matches/dashboard")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let d: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(d["total_matches"], 0);
    assert_eq!(d["matched_count"], 0);
}
