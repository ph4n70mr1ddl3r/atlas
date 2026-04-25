//! AutoInvoice E2E Tests
//!
//! Tests for Oracle Fusion Receivables AutoInvoice:
//! - Grouping rule CRUD
//! - Validation rule CRUD
//! - Batch import with transaction lines
//! - Batch validation lifecycle
//! - Batch processing and invoice creation
//! - Invoice status transitions
//! - Import-and-process convenience endpoint
//! - Dashboard analytics

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_autoinvoice_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    setup_test_db(&state.db_pool).await;
    // Clean autoinvoice test data for test isolation
    sqlx::query("DELETE FROM _atlas.autoinvoice_result_lines").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM _atlas.autoinvoice_results").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM _atlas.autoinvoice_lines").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM _atlas.autoinvoice_batches").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM _atlas.autoinvoice_validation_rules").execute(&state.db_pool).await.ok();
    sqlx::query("DELETE FROM _atlas.autoinvoice_grouping_rules").execute(&state.db_pool).await.ok();
    let app = build_router(state.clone());
    (state, app)
}

async fn create_test_grouping_rule(app: &axum::Router, name: &str) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/autoinvoice/grouping-rules")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "name": name,
            "description": format!("Test rule {}", name),
            "transaction_types": ["invoice", "credit_memo"],
            "group_by_fields": ["bill_to_customer_id", "currency_code", "transaction_type"],
            "line_order_by": ["line_number"],
            "is_default": false,
            "priority": 10
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_validation_rule(app: &axum::Router, name: &str, field_name: &str) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/autoinvoice/validation-rules")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "name": name,
            "description": format!("Test validation {}", name),
            "field_name": field_name,
            "validation_type": "required",
            "error_message": format!("{} is required", field_name),
            "is_fatal": true,
            "transaction_types": ["invoice"],
            "priority": 10
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn import_test_batch(app: &axum::Router, lines: Vec<serde_json::Value>) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/autoinvoice/batches")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "batch_source": "test_import",
            "description": "Test batch import",
            "lines": lines
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

fn make_test_line(currency: &str, amount: &str, item_code: &str) -> serde_json::Value {
    json!({
        "transaction_type": "invoice",
        "currency_code": currency,
        "line_amount": amount,
        "unit_price": amount,
        "item_code": item_code,
        "item_description": format!("Item {}", item_code),
        "quantity": "1",
        "unit_of_measure": "EA",
        "transaction_date": "2025-01-15",
        "gl_date": "2025-01-15",
        "customer_number": "CUST001",
        "customer_name": "Test Customer",
        "revenue_account_code": "4000",
        "receivable_account_code": "1200"
    })
}

// ============================================================================
// Grouping Rule Tests
// ============================================================================

#[tokio::test]
async fn test_create_grouping_rule() {
    let (_state, app) = setup_autoinvoice_test().await;
    let rule = create_test_grouping_rule(&app, "test-gr-1").await;
    assert_eq!(rule["name"], "test-gr-1");
    assert!(rule["id"].is_string());
    assert!(rule["isDefault"].is_boolean());
}

#[tokio::test]
async fn test_create_grouping_rule_duplicate_name() {
    let (_state, app) = setup_autoinvoice_test().await;
    create_test_grouping_rule(&app, "dup-gr").await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/autoinvoice/grouping-rules")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "name": "dup-gr",
            "group_by_fields": ["currency_code"]
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_create_grouping_rule_empty_name() {
    let (_state, app) = setup_autoinvoice_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/autoinvoice/grouping-rules")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "name": "",
            "group_by_fields": ["currency_code"]
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_list_grouping_rules() {
    let (_state, app) = setup_autoinvoice_test().await;
    create_test_grouping_rule(&app, "list-gr-1").await;
    create_test_grouping_rule(&app, "list-gr-2").await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/autoinvoice/grouping-rules").header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_get_grouping_rule() {
    let (_state, app) = setup_autoinvoice_test().await;
    let rule = create_test_grouping_rule(&app, "get-gr").await;
    let id = rule["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/autoinvoice/grouping-rules/{}", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let fetched: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(fetched["name"], "get-gr");
}

#[tokio::test]
async fn test_get_grouping_rule_not_found() {
    let (_state, app) = setup_autoinvoice_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/autoinvoice/grouping-rules/{}", uuid::Uuid::new_v4()))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_grouping_rule() {
    let (_state, app) = setup_autoinvoice_test().await;
    let rule = create_test_grouping_rule(&app, "del-gr").await;
    let id = rule["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/autoinvoice/grouping-rules/{}", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Validation Rule Tests
// ============================================================================

#[tokio::test]
async fn test_create_validation_rule() {
    let (_state, app) = setup_autoinvoice_test().await;
    let rule = create_test_validation_rule(&app, "vr-1", "customer_number").await;
    assert_eq!(rule["name"], "vr-1");
    assert_eq!(rule["fieldName"], "customer_number");
    assert_eq!(rule["validationType"], "required");
    assert!(rule["isFatal"].is_boolean());
    assert!(rule["id"].is_string());
}

#[tokio::test]
async fn test_create_validation_rule_invalid_type() {
    let (_state, app) = setup_autoinvoice_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/autoinvoice/validation-rules")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "name": "bad-type",
            "field_name": "amount",
            "validation_type": "crystal_ball",
            "error_message": "Invalid"
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_list_validation_rules() {
    let (_state, app) = setup_autoinvoice_test().await;
    create_test_validation_rule(&app, "vr-list-1", "customer_number").await;
    create_test_validation_rule(&app, "vr-list-2", "item_code").await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/autoinvoice/validation-rules").header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_delete_validation_rule() {
    let (_state, app) = setup_autoinvoice_test().await;
    let rule = create_test_validation_rule(&app, "vr-del", "customer_name").await;
    let id = rule["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/autoinvoice/validation-rules/{}", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

// ============================================================================
// Batch Import Tests
// ============================================================================

#[tokio::test]
async fn test_import_batch() {
    let (_state, app) = setup_autoinvoice_test().await;
    let batch = import_test_batch(&app, vec![
        make_test_line("USD", "100.00", "ITEM-001"),
        make_test_line("USD", "250.00", "ITEM-002"),
    ]).await;
    assert!(batch["id"].is_string());
    assert_eq!(batch["status"], "pending");
    assert_eq!(batch["totalLines"], 2);
    assert_eq!(batch["batchSource"], "test_import");
}

#[tokio::test]
async fn test_import_batch_empty_lines_rejected() {
    let (_state, app) = setup_autoinvoice_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/autoinvoice/batches")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "batch_source": "test",
            "lines": []
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_list_batches() {
    let (_state, app) = setup_autoinvoice_test().await;
    import_test_batch(&app, vec![make_test_line("USD", "50.00", "L1")]).await;
    import_test_batch(&app, vec![make_test_line("EUR", "75.00", "L2")]).await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/autoinvoice/batches").header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(resp["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_list_batches_by_status() {
    let (_state, app) = setup_autoinvoice_test().await;
    import_test_batch(&app, vec![make_test_line("USD", "50.00", "L1")]).await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/autoinvoice/batches?status=pending").header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let batches = resp["data"].as_array().unwrap();
    assert!(batches.len() >= 1);
    for batch in batches {
        assert_eq!(batch["status"], "pending");
    }
}

#[tokio::test]
async fn test_get_batch() {
    let (_state, app) = setup_autoinvoice_test().await;
    let batch = import_test_batch(&app, vec![make_test_line("USD", "100.00", "ITEM-G")]).await;
    let id = batch["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/autoinvoice/batches/{}", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let fetched: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(fetched["batchSource"], "test_import");
}

#[tokio::test]
async fn test_get_batch_lines() {
    let (_state, app) = setup_autoinvoice_test().await;
    let batch = import_test_batch(&app, vec![
        make_test_line("USD", "100.00", "LINE-A"),
        make_test_line("USD", "200.00", "LINE-B"),
    ]).await;
    let id = batch["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/autoinvoice/batches/{}/lines", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let lines = resp["data"].as_array().unwrap();
    assert_eq!(lines.len(), 2);
}

// ============================================================================
// Validation & Processing Lifecycle Tests
// ============================================================================

#[tokio::test]
async fn test_validate_batch() {
    let (_state, app) = setup_autoinvoice_test().await;
    let batch = import_test_batch(&app, vec![
        make_test_line("USD", "100.00", "VAL-1"),
        make_test_line("USD", "200.00", "VAL-2"),
    ]).await;
    let id = batch["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/autoinvoice/batches/{}/validate", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let validated: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(validated["status"], "validated");
    assert_eq!(validated["validLines"], 2);
    assert_eq!(validated["invalidLines"], 0);
}

#[tokio::test]
async fn test_validate_non_pending_rejected() {
    let (_state, app) = setup_autoinvoice_test().await;
    let batch = import_test_batch(&app, vec![make_test_line("USD", "100.00", "VNP")]).await;
    let id = batch["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());
    // Validate once
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/autoinvoice/batches/{}/validate", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    // Try to validate again
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/autoinvoice/batches/{}/validate", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_process_batch() {
    let (_state, app) = setup_autoinvoice_test().await;
    let batch = import_test_batch(&app, vec![
        make_test_line("USD", "100.00", "PROC-1"),
        make_test_line("USD", "250.00", "PROC-2"),
    ]).await;
    let id = batch["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    // Validate first
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/autoinvoice/batches/{}/validate", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Process
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/autoinvoice/batches/{}/process", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let processed: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(processed["status"], "completed");
    assert_eq!(processed["invoicesCreated"], 1);
}

#[tokio::test]
async fn test_process_non_validated_rejected() {
    let (_state, app) = setup_autoinvoice_test().await;
    let batch = import_test_batch(&app, vec![make_test_line("USD", "100.00", "PNV")]).await;
    let id = batch["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());
    // Try to process without validating
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/autoinvoice/batches/{}/process", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// Import and Process Convenience Endpoint
// ============================================================================

#[tokio::test]
async fn test_import_and_process() {
    let (_state, app) = setup_autoinvoice_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/autoinvoice/import-and-process")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "batch_source": "convenience_test",
            "description": "One-shot import",
            "lines": [
                make_test_line("USD", "500.00", "CONV-1"),
                make_test_line("USD", "300.00", "CONV-2"),
            ]
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let batch: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(batch["status"], "completed");
    assert_eq!(batch["invoicesCreated"], 1);
    assert_eq!(batch["validLines"], 2);
}

// ============================================================================
// Multi-currency / Multi-group Processing
// ============================================================================

#[tokio::test]
async fn test_process_batch_multiple_groups() {
    let (_state, app) = setup_autoinvoice_test().await;
    let batch = import_test_batch(&app, vec![
        make_test_line("USD", "100.00", "GRP-U1"),
        make_test_line("USD", "200.00", "GRP-U2"),
        make_test_line("EUR", "150.00", "GRP-E1"),
    ]).await;
    let id = batch["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    // Validate
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/autoinvoice/batches/{}/validate", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Process — should create 2 invoices (USD group + EUR group)
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/autoinvoice/batches/{}/process", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let processed: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(processed["status"], "completed");
    assert_eq!(processed["invoicesCreated"], 2);
}

#[tokio::test]
async fn test_get_batch_results() {
    let (_state, app) = setup_autoinvoice_test().await;
    let batch = import_test_batch(&app, vec![
        make_test_line("USD", "100.00", "RES-1"),
        make_test_line("USD", "200.00", "RES-2"),
    ]).await;
    let id = batch["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    // Validate and process
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/autoinvoice/batches/{}/validate", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/autoinvoice/batches/{}/process", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Get results
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/autoinvoice/batches/{}/results", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let results = resp["data"].as_array().unwrap();
    assert_eq!(results.len(), 1); // all USD invoices grouped into one
    let invoice = &results[0];
    assert!(invoice["invoiceNumber"].is_string());
    assert_eq!(invoice["currencyCode"], "USD");
}

#[tokio::test]
async fn test_get_invoice_lines() {
    let (_state, app) = setup_autoinvoice_test().await;
    let batch = import_test_batch(&app, vec![
        make_test_line("USD", "100.00", "INV-L1"),
        make_test_line("USD", "200.00", "INV-L2"),
    ]).await;
    let id = batch["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    // Validate and process
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/autoinvoice/batches/{}/validate", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/autoinvoice/batches/{}/process", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Get results to find invoice ID
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/autoinvoice/batches/{}/results", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let invoice_id = resp["data"][0]["id"].as_str().unwrap();

    // Get invoice lines
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/autoinvoice/invoices/{}", invoice_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let lines = resp["data"].as_array().unwrap();
    assert_eq!(lines.len(), 2);
}

// ============================================================================
// Invoice Status Transition Tests
// ============================================================================

#[tokio::test]
async fn test_invoice_status_draft_to_complete() {
    let (_state, app) = setup_autoinvoice_test().await;
    let batch = import_test_batch(&app, vec![make_test_line("USD", "100.00", "ST-1")]).await;
    let id = batch["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    // Process batch
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/autoinvoice/batches/{}/validate", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/autoinvoice/batches/{}/process", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Get invoice
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/autoinvoice/batches/{}/results", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let invoice_id = resp["data"][0]["id"].as_str().unwrap();

    // Transition draft → complete
    let r = app.clone().oneshot(Request::builder().method("PUT")
        .uri(&format!("/api/v1/autoinvoice/invoices/{}/status", invoice_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "complete"})).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let invoice: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(invoice["status"], "complete");
}

#[tokio::test]
async fn test_invoice_status_draft_to_posted_rejected() {
    let (_state, app) = setup_autoinvoice_test().await;
    let batch = import_test_batch(&app, vec![make_test_line("USD", "100.00", "ST-2")]).await;
    let id = batch["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    // Process batch
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/autoinvoice/batches/{}/validate", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/autoinvoice/batches/{}/process", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Get invoice
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/autoinvoice/batches/{}/results", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let invoice_id = resp["data"][0]["id"].as_str().unwrap();

    // Cannot skip directly to posted
    let r = app.clone().oneshot(Request::builder().method("PUT")
        .uri(&format!("/api/v1/autoinvoice/invoices/{}/status", invoice_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "posted"})).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_invoice_full_lifecycle() {
    let (_state, app) = setup_autoinvoice_test().await;
    let batch = import_test_batch(&app, vec![make_test_line("USD", "500.00", "LC")]).await;
    let id = batch["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    // Process batch
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/autoinvoice/batches/{}/validate", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/autoinvoice/batches/{}/process", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Get invoice
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/autoinvoice/batches/{}/results", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let invoice_id = resp["data"][0]["id"].as_str().unwrap();

    // draft → complete
    let r = app.clone().oneshot(Request::builder().method("PUT")
        .uri(&format!("/api/v1/autoinvoice/invoices/{}/status", invoice_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "complete"})).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    // complete → posted
    let r = app.clone().oneshot(Request::builder().method("PUT")
        .uri(&format!("/api/v1/autoinvoice/invoices/{}/status", invoice_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "posted"})).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let posted: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(posted["status"], "posted");
}

#[tokio::test]
async fn test_invoice_cancel_from_draft() {
    let (_state, app) = setup_autoinvoice_test().await;
    let batch = import_test_batch(&app, vec![make_test_line("USD", "100.00", "CNC")]).await;
    let id = batch["id"].as_str().unwrap();
    let (k, v) = auth_header(&admin_claims());

    // Process batch
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/autoinvoice/batches/{}/validate", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/autoinvoice/batches/{}/process", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Get invoice
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/autoinvoice/batches/{}/results", id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resp: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let invoice_id = resp["data"][0]["id"].as_str().unwrap();

    // Cancel from draft
    let r = app.clone().oneshot(Request::builder().method("PUT")
        .uri(&format!("/api/v1/autoinvoice/invoices/{}/status", invoice_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "cancelled"})).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let cancelled: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(cancelled["status"], "cancelled");
}

// ============================================================================
// Dashboard Test
// ============================================================================

#[tokio::test]
async fn test_autoinvoice_dashboard() {
    let (_state, app) = setup_autoinvoice_test().await;
    import_test_batch(&app, vec![make_test_line("USD", "100.00", "DB-1")]).await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/autoinvoice/dashboard").header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let dashboard: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(dashboard["totalBatches"].as_i64().unwrap() >= 1);
    assert!(dashboard["pendingBatches"].as_i64().unwrap() >= 1);
}
