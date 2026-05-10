//! Distribution Set E2E Tests
//!
//! Tests for Oracle Fusion Cloud ERP Payables > Distribution Sets:
//! - Set CRUD (create, list, get, activate, deactivate, delete)
//! - Set line management (add, list, remove)
//! - Validation: percentages must not exceed 100%, activate requires lines totaling 100%
//! - Apply to invoice
//! - Usage history tracking
//! - Dashboard summary
//! - Edge cases and validation

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
    sqlx::query("DELETE FROM financials.distribution_set_usage_log").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM financials.distribution_set_lines").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM financials.distribution_sets").execute(&state.db_pool).await.ok();
    sqlx::query("CREATE SCHEMA IF NOT EXISTS financials")
        .execute(&state.db_pool)
        .await
        .ok();
    sqlx::query(include_str!("../../../../migrations/137_distribution_set.sql"))
        .execute(&state.db_pool)
        .await
        .ok();
    let app = build_router(state.clone());
    (state, app)
}

async fn create_set(
    app: &axum::Router,
    set_code: &str,
    set_name: &str,
    distribution_type: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "setCode": set_code,
        "setName": set_name,
        "distributionType": distribution_type,
        "currencyCode": "USD",
        "description": "Test distribution set",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/distribution-sets")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8_lossy(&b);
    assert_eq!(status, StatusCode::CREATED, "Failed to create set: {:?}", body_str);
    serde_json::from_slice(&b).unwrap()
}

async fn add_line(
    app: &axum::Router,
    set_id: &Uuid,
    account_combination: &str,
    percentage: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "accountCombination": account_combination,
        "percentage": percentage,
        "description": "Test line",
        "costCenter": "CC-001",
        "department": "Finance",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/distribution-sets/{}/lines", set_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let body_str = String::from_utf8_lossy(&b);
    assert_eq!(status, StatusCode::CREATED, "Failed to add line: {:?}", body_str);
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Set CRUD Tests
// ============================================================================

#[tokio::test]
async fn test_create_distribution_set() {
    let (_state, app) = setup_test().await;
    let set = create_set(&app, "RENT-01", "Rent Distribution", "percentage").await;

    assert_eq!(set["setCode"], "RENT-01");
    assert_eq!(set["setName"], "Rent Distribution");
    assert_eq!(set["distributionType"], "percentage");
    assert_eq!(set["currencyCode"], "USD");
    assert_eq!(set["status"], "active");
    assert_eq!(set["isDefault"], false);
    assert_eq!(set["usageCount"], 0);
    assert_eq!(set["totalPercentage"], "0.0000");
}

#[tokio::test]
async fn test_create_amount_type_set() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "setCode": "FIXED-01",
        "setName": "Fixed Amount Distribution",
        "distributionType": "amount",
        "currencyCode": "EUR",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/distribution-sets")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let set: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(set["distributionType"], "amount");
    assert_eq!(set["currencyCode"], "EUR");
}

#[tokio::test]
async fn test_create_set_duplicate_code_fails() {
    let (_state, app) = setup_test().await;
    create_set(&app, "DUP-01", "First Set", "percentage").await;

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "setCode": "DUP-01",
        "setName": "Second Set",
        "distributionType": "percentage",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/distribution-sets")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_create_set_invalid_type_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "setCode": "BAD-01",
        "setName": "Bad Type",
        "distributionType": "ratio",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/distribution-sets")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_set_inverted_dates_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "setCode": "DATE-01",
        "setName": "Bad Dates",
        "distributionType": "percentage",
        "effectiveFrom": "2025-12-31",
        "effectiveTo": "2025-01-01",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/distribution-sets")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_distribution_set() {
    let (_state, app) = setup_test().await;
    let set = create_set(&app, "GET-01", "Get Test", "percentage").await;
    let set_id: Uuid = set["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/distribution-sets/{}", set_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["id"], set["id"]);
    assert_eq!(body["setCode"], "GET-01");
}

#[tokio::test]
async fn test_get_nonexistent_set_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let fake_id = Uuid::new_v4();
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/distribution-sets/{}", fake_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_list_distribution_sets() {
    let (_state, app) = setup_test().await;
    create_set(&app, "LIST-A", "Set A", "percentage").await;
    create_set(&app, "LIST-B", "Set B", "amount").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/distribution-sets")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let data = body["data"].as_array().unwrap();
    assert!(data.len() >= 2);
}

#[tokio::test]
async fn test_list_sets_filter_by_type() {
    let (_state, app) = setup_test().await;
    create_set(&app, "TYPE-PCT", "Pct Set", "percentage").await;
    create_set(&app, "TYPE-AMT", "Amt Set", "amount").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/distribution-sets?distribution_type=amount")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let data = body["data"].as_array().unwrap();
    assert_eq!(data.len(), 1);
    assert_eq!(data[0]["distributionType"], "amount");
}

// ============================================================================
// Set Lifecycle Tests
// ============================================================================

#[tokio::test]
async fn test_activate_set_with_valid_lines() {
    let (_state, app) = setup_test().await;
    let set = create_set(&app, "ACT-01", "Activate Test", "percentage").await;
    let set_id: Uuid = set["id"].as_str().unwrap().parse().unwrap();

    // Deactivate first (created active), add lines, then activate
    let (k, v) = auth_header(&admin_claims());

    // Deactivate
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/distribution-sets/{}/deactivate", set_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    // Add lines totaling 100%
    add_line(&app, &set_id, "1000.100.100.100.100", "60.0000").await;
    add_line(&app, &set_id, "1000.100.100.100.200", "40.0000").await;

    // Activate
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/distribution-sets/{}/activate", set_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "active");
}

#[tokio::test]
async fn test_activate_set_without_lines_fails() {
    let (_state, app) = setup_test().await;
    let set = create_set(&app, "ACT-NO-LINES", "No Lines Set", "percentage").await;
    let set_id: Uuid = set["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Deactivate first
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/distribution-sets/{}/deactivate", set_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Try to activate without lines
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/distribution-sets/{}/activate", set_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_activate_set_with_wrong_total_fails() {
    let (_state, app) = setup_test().await;
    let set = create_set(&app, "ACT-BAD-PCT", "Bad Total Set", "percentage").await;
    let set_id: Uuid = set["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Deactivate first
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/distribution-sets/{}/deactivate", set_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Add lines totaling only 80%
    add_line(&app, &set_id, "1000.100.100.100.100", "50.0000").await;
    add_line(&app, &set_id, "1000.100.100.100.200", "30.0000").await;

    // Try to activate - should fail
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/distribution-sets/{}/activate", set_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_deactivate_set() {
    let (_state, app) = setup_test().await;
    let set = create_set(&app, "DEACT-01", "Deactivate Test", "percentage").await;
    let set_id: Uuid = set["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/distribution-sets/{}/deactivate", set_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["status"], "inactive");
}

#[tokio::test]
async fn test_delete_inactive_set() {
    let (_state, app) = setup_test().await;
    let set = create_set(&app, "DEL-01", "Delete Test", "percentage").await;
    let set_id: Uuid = set["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Deactivate first
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/distribution-sets/{}/deactivate", set_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Delete
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/distribution-sets/{}", set_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_delete_active_set_fails() {
    let (_state, app) = setup_test().await;
    let set = create_set(&app, "DEL-ACTIVE", "Active Delete Test", "percentage").await;
    let set_id: Uuid = set["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/distribution-sets/{}", set_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Distribution Set Lines Tests
// ============================================================================

#[tokio::test]
async fn test_add_distribution_lines() {
    let (_state, app) = setup_test().await;
    let set = create_set(&app, "LINE-01", "Line Test", "percentage").await;
    let set_id: Uuid = set["id"].as_str().unwrap().parse().unwrap();

    let line1 = add_line(&app, &set_id, "1000.100.100.100.100", "60.0000").await;
    assert_eq!(line1["accountCombination"], "1000.100.100.100.100");
    assert_eq!(line1["percentage"], "60.0000");
    assert_eq!(line1["lineNumber"], 1);

    let line2 = add_line(&app, &set_id, "1000.100.100.100.200", "40.0000").await;
    assert_eq!(line2["lineNumber"], 2);

    // Verify total percentage updated on set
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/distribution-sets/{}", set_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let total_pct: f64 = body["totalPercentage"].as_str().unwrap().parse().unwrap();
    assert!((total_pct - 100.0).abs() < 0.01);
}

#[tokio::test]
async fn test_add_line_exceeds_100_percent_fails() {
    let (_state, app) = setup_test().await;
    let set = create_set(&app, "EXCEED-01", "Exceed Test", "percentage").await;
    let set_id: Uuid = set["id"].as_str().unwrap().parse().unwrap();

    add_line(&app, &set_id, "1000.100.100.100.100", "80.0000").await;

    // Try to add another 30% - should fail
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "accountCombination": "1000.100.100.100.200",
        "percentage": "30.0000",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/distribution-sets/{}/lines", set_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_list_distribution_lines() {
    let (_state, app) = setup_test().await;
    let set = create_set(&app, "LIST-LINES", "List Lines Test", "percentage").await;
    let set_id: Uuid = set["id"].as_str().unwrap().parse().unwrap();

    add_line(&app, &set_id, "1000.100.100.100.100", "50.0000").await;
    add_line(&app, &set_id, "1000.100.100.100.200", "50.0000").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/distribution-sets/{}/lines", set_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["data"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_remove_distribution_line() {
    let (_state, app) = setup_test().await;
    let set = create_set(&app, "REM-LINE", "Remove Line Test", "percentage").await;
    let set_id: Uuid = set["id"].as_str().unwrap().parse().unwrap();

    let line1 = add_line(&app, &set_id, "1000.100.100.100.100", "30.0000").await;
    let _line2 = add_line(&app, &set_id, "1000.100.100.100.200", "70.0000").await;

    let line1_id: Uuid = line1["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Remove line1
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/distribution-sets/lines/{}", line1_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);

    // Verify only one line remains
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/distribution-sets/{}/lines", set_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["data"].as_array().unwrap().len(), 1);

    // Verify total percentage updated
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/distribution-sets/{}", set_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    let set_body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let total_pct: f64 = set_body["totalPercentage"].as_str().unwrap().parse().unwrap();
    assert!((total_pct - 70.0).abs() < 0.01);
}

#[tokio::test]
async fn test_add_line_invalid_percentage() {
    let (_state, app) = setup_test().await;
    let set = create_set(&app, "BAD-PCT", "Bad Pct Test", "percentage").await;
    let set_id: Uuid = set["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "accountCombination": "1000.100.100.100.100",
        "percentage": "150.0000",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/distribution-sets/{}/lines", set_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_add_line_empty_account_fails() {
    let (_state, app) = setup_test().await;
    let set = create_set(&app, "NO-ACCT", "No Account Test", "percentage").await;
    let set_id: Uuid = set["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "accountCombination": "",
        "percentage": "50.0000",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/distribution-sets/{}/lines", set_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Apply to Invoice Tests
// ============================================================================

#[tokio::test]
async fn test_apply_distribution_set_to_invoice() {
    let (_state, app) = setup_test().await;
    let set = create_set(&app, "APPLY-01", "Apply Test", "percentage").await;
    let set_id: Uuid = set["id"].as_str().unwrap().parse().unwrap();

    // Add lines totaling 100%
    add_line(&app, &set_id, "1000.100.100.100.100", "60.0000").await;
    add_line(&app, &set_id, "1000.100.100.100.200", "40.0000").await;

    let invoice_id = Uuid::new_v4();
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "invoiceId": invoice_id,
        "invoiceNumber": "INV-TEST-001",
        "invoiceAmount": "10000.00",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/distribution-sets/{}/apply", set_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["applied"], true);
    assert_eq!(body["lineCount"], 2);
    assert!(body["lines"].is_array());
    assert_eq!(body["lines"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_apply_inactive_set_fails() {
    let (_state, app) = setup_test().await;
    let set = create_set(&app, "APPLY-INACT", "Apply Inactive Test", "percentage").await;
    let set_id: Uuid = set["id"].as_str().unwrap().parse().unwrap();

    let (k, v) = auth_header(&admin_claims());

    // Deactivate
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/distribution-sets/{}/deactivate", set_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Try to apply
    let invoice_id = Uuid::new_v4();
    let payload = json!({
        "invoiceId": invoice_id,
        "invoiceAmount": "5000.00",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/distribution-sets/{}/apply", set_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_apply_updates_usage_count() {
    let (_state, app) = setup_test().await;
    let set = create_set(&app, "USAGE-01", "Usage Count Test", "percentage").await;
    let set_id: Uuid = set["id"].as_str().unwrap().parse().unwrap();

    add_line(&app, &set_id, "1000.100.100.100.100", "100.0000").await;

    let (k, v) = auth_header(&admin_claims());

    // Apply twice
    for _ in 0..2 {
        let invoice_id = Uuid::new_v4();
        let payload = json!({
            "invoiceId": invoice_id,
            "invoiceAmount": "1000.00",
        });
        app.clone().oneshot(Request::builder().method("POST")
            .uri(&format!("/api/v1/distribution-sets/{}/apply", set_id))
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(serde_json::to_string(&payload).unwrap()))
            .unwrap()
        ).await.unwrap();
    }

    // Check usage count
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/distribution-sets/{}", set_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(body["usageCount"], 2);
}

// ============================================================================
// Usage History Tests
// ============================================================================

#[tokio::test]
async fn test_list_usage_history() {
    let (_state, app) = setup_test().await;
    let set = create_set(&app, "HIST-01", "History Test", "percentage").await;
    let set_id: Uuid = set["id"].as_str().unwrap().parse().unwrap();

    add_line(&app, &set_id, "1000.100.100.100.100", "100.0000").await;

    let (k, v) = auth_header(&admin_claims());

    // Apply to generate usage
    let invoice_id = Uuid::new_v4();
    let payload = json!({
        "invoiceId": invoice_id,
        "invoiceNumber": "INV-HIST-001",
        "invoiceAmount": "5000.00",
    });
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/distribution-sets/{}/apply", set_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();

    // List usage
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/distribution-sets/usage")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let data = body["data"].as_array().unwrap();
    assert!(!data.is_empty());
    assert_eq!(data[0]["setCode"], "HIST-01");
    assert_eq!(data[0]["targetEntityType"], "ap_invoice");
    assert_eq!(data[0]["lineCount"], 1);
}

#[tokio::test]
async fn test_list_usage_filter_by_set() {
    let (_state, app) = setup_test().await;
    let set1 = create_set(&app, "FILTER-01", "Filter Set 1", "percentage").await;
    let set2 = create_set(&app, "FILTER-02", "Filter Set 2", "percentage").await;
    let set_id1: Uuid = set1["id"].as_str().unwrap().parse().unwrap();
    let set_id2: Uuid = set2["id"].as_str().unwrap().parse().unwrap();

    add_line(&app, &set_id1, "1000.100.100.100.100", "100.0000").await;
    add_line(&app, &set_id2, "1000.100.100.100.200", "100.0000").await;

    let (k, v) = auth_header(&admin_claims());

    // Apply both
    for set_id in [&set_id1, &set_id2] {
        let payload = json!({
            "invoiceId": Uuid::new_v4(),
            "invoiceAmount": "1000.00",
        });
        app.clone().oneshot(Request::builder().method("POST")
            .uri(&format!("/api/v1/distribution-sets/{}/apply", set_id))
            .header("Content-Type", "application/json")
            .header(&k, &v)
            .body(Body::from(serde_json::to_string(&payload).unwrap()))
            .unwrap()
        ).await.unwrap();
    }

    // Filter by set1
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/distribution-sets/usage?distribution_set_id={}", set_id1))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    let data = body["data"].as_array().unwrap();
    assert_eq!(data.len(), 1);
    assert_eq!(data[0]["setCode"], "FILTER-01");
}

// ============================================================================
// Dashboard Tests
// ============================================================================

#[tokio::test]
async fn test_distribution_set_dashboard() {
    let (_state, app) = setup_test().await;
    create_set(&app, "DASH-01", "Dashboard Set 1", "percentage").await;
    create_set(&app, "DASH-02", "Dashboard Set 2", "amount").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/distribution-sets/dashboard")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let body: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();

    assert!(body.get("totalSets").is_some());
    assert!(body.get("activeSets").is_some());
    assert!(body.get("inactiveSets").is_some());
    assert!(body.get("defaultSets").is_some());
    assert!(body.get("totalUsages").is_some());
    assert!(body.get("percentageSets").is_some());
    assert!(body.get("amountSets").is_some());

    let total: i32 = body["totalSets"].as_i64().unwrap() as i32;
    assert!(total >= 2, "Expected at least 2 total sets, got {}", total);
}

// ============================================================================
// Edge Case: Amount-based Distribution Set
// ============================================================================

#[tokio::test]
async fn test_amount_based_distribution_set() {
    let (_state, app) = setup_test().await;
    let set = create_set(&app, "AMT-01", "Amount Based Set", "amount").await;
    let set_id: Uuid = set["id"].as_str().unwrap().parse().unwrap();

    // Amount-based sets don't need to sum to 100%
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "accountCombination": "1000.100.100.100.100",
        "percentage": "0.0000",
        "amount": "5000.00",
        "description": "Fixed amount line",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/distribution-sets/{}/lines", set_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let line: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(line["amount"], "5000.00");
}

// ============================================================================
// Full Lifecycle Integration Test
// ============================================================================

#[tokio::test]
async fn test_full_distribution_set_lifecycle() {
    let (_state, app) = setup_test().await;

    // 1. Create set
    let set = create_set(&app, "LIFECYCLE", "Full Lifecycle Test", "percentage").await;
    let set_id: Uuid = set["id"].as_str().unwrap().parse().unwrap();
    assert_eq!(set["status"], "active");

    // 2. Add lines
    let _line1 = add_line(&app, &set_id, "1000.100.100.100.100", "50.0000").await;
    let line2 = add_line(&app, &set_id, "1000.100.100.100.200", "30.0000").await;
    let _line3 = add_line(&app, &set_id, "1000.100.100.100.300", "20.0000").await;

    // 3. Verify lines
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/distribution-sets/{}/lines", set_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    let lines: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(lines["data"].as_array().unwrap().len(), 3);

    // 4. Remove one line
    let line2_id: Uuid = line2["id"].as_str().unwrap().parse().unwrap();
    app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/distribution-sets/lines/{}", line2_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // 5. Verify remaining lines
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/distribution-sets/{}/lines", set_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    let lines: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(lines["data"].as_array().unwrap().len(), 2);

    // 6. Apply to invoice
    let invoice_id = Uuid::new_v4();
    let payload = json!({
        "invoiceId": invoice_id,
        "invoiceNumber": "INV-LC-001",
        "invoiceAmount": "10000.00",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/distribution-sets/{}/apply", set_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    // 7. Check usage history
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/distribution-sets/usage?distribution_set_id={}", set_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    let usage: serde_json::Value = axum::body::to_bytes(r.into_body(), usize::MAX).await
        .map(|b| serde_json::from_slice(&b).unwrap()).unwrap();
    assert_eq!(usage["data"].as_array().unwrap().len(), 1);

    // 8. Deactivate
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/distribution-sets/{}/deactivate", set_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    // 9. Delete
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/distribution-sets/{}", set_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}
