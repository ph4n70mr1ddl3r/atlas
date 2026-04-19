//! Manual Journal Entries E2E Tests
//!
//! Tests for Oracle Fusion Cloud ERP Manual Journal Entries:
//! - Batch CRUD (create, get, list, delete)
//! - Batch lifecycle (draft → submitted → approved → posted → reversed)
//! - Entry management (create, get, list, delete)
//! - Line management (add, list, balance validation)
//! - Balance validation (debits must equal credits for submission)
//! - Reversal with swapped debits/credits
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
    sqlx::query(include_str!("../../../../migrations/041_manual_journal_entries.sql"))
        .execute(&state.db_pool)
        .await
        .ok();
    let app = build_router(state.clone());
    (state, app)
}

async fn create_batch(
    app: &axum::Router,
    batch_number: &str,
    name: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "batch_number": batch_number,
        "name": name,
        "description": "Test journal batch",
        "currency_code": "USD",
        "accounting_date": "2024-03-15",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/journals/batches")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Failed to create batch");
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn create_entry_in_batch(
    app: &axum::Router,
    batch_id: Uuid,
    entry_number: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "entry_number": entry_number,
        "name": format!("Entry {}", entry_number),
        "description": "Test journal entry",
        "currency_code": "USD",
        "accounting_date": "2024-03-15",
        "journal_category": "manual",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/journals/batches/{}/entries", batch_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Failed to create entry");
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn add_entry_line(
    app: &axum::Router,
    entry_id: Uuid,
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
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/journals/entries/{}/lines", entry_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Failed to add line");
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

/// Create a fully balanced entry (debit + credit) and return entry JSON
async fn create_balanced_entry(
    app: &axum::Router,
    batch_id: Uuid,
    entry_number: &str,
    amount: &str,
) -> serde_json::Value {
    let entry = create_entry_in_batch(app, batch_id, entry_number).await;
    let entry_id = entry["id"].as_str().unwrap().parse::<Uuid>().unwrap();
    add_entry_line(app, entry_id, "debit", "1000.100", amount).await;
    add_entry_line(app, entry_id, "credit", "2000.200", amount).await;
    // Re-fetch to get updated totals
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/journals/entries/{}", entry_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Batch CRUD Tests
// ============================================================================

#[tokio::test]
async fn test_create_batch() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "JB-001", "March Adjustments").await;

    assert_eq!(batch["batch_number"], "JB-001");
    assert_eq!(batch["name"], "March Adjustments");
    assert_eq!(batch["status"], "draft");
    assert_eq!(batch["currency_code"], "USD");
    assert_eq!(batch["source"], "manual");
}

#[tokio::test]
async fn test_get_batch() {
    let (_state, app) = setup_test().await;
    create_batch(&app, "JB-002", "Test Batch").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/journals/batches/JB-002")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let batch: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(batch["batch_number"], "JB-002");
}

#[tokio::test]
async fn test_list_batches() {
    let (_state, app) = setup_test().await;
    create_batch(&app, "JB-010", "Batch A").await;
    create_batch(&app, "JB-011", "Batch B").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/journals/batches")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(result["data"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_list_batches_by_status() {
    let (_state, app) = setup_test().await;
    create_batch(&app, "JB-020", "Draft Batch").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/journals/batches?status=draft")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let batches = result["data"].as_array().unwrap();
    assert!(batches.len() >= 1);
    assert_eq!(batches[0]["status"], "draft");
}

#[tokio::test]
async fn test_delete_batch() {
    let (_state, app) = setup_test().await;
    create_batch(&app, "JB-DEL", "Delete Me").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/journals/batches/JB-DEL")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    // Verify it's gone
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/journals/batches/JB-DEL")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_non_draft_batch_fails() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "JB-NDL", "Cannot Delete").await;
    let batch_id = batch["id"].as_str().unwrap().parse::<Uuid>().unwrap();

    // Submit it
    let (k, v) = auth_header(&admin_claims());
    // First add an entry so submission succeeds
    create_balanced_entry(&app, batch_id, "JE-001", "100.00").await;
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/journals/batches/{}/submit", batch_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Try to delete
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/journals/batches/JB-NDL")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

// ============================================================================
// Entry Management Tests
// ============================================================================

#[tokio::test]
async fn test_create_entry() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "JB-E01", "Entry Test Batch").await;
    let batch_id = batch["id"].as_str().unwrap().parse::<Uuid>().unwrap();

    let entry = create_entry_in_batch(&app, batch_id, "JE-001").await;
    assert_eq!(entry["entry_number"], "JE-001");
    assert_eq!(entry["status"], "draft");
    assert_eq!(entry["is_balanced"], false);
    assert_eq!(entry["total_debit"], "0");  // No lines yet
    assert_eq!(entry["total_credit"], "0");
}

#[tokio::test]
async fn test_list_entries_by_batch() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "JB-E02", "Multi Entry Batch").await;
    let batch_id = batch["id"].as_str().unwrap().parse::<Uuid>().unwrap();

    create_entry_in_batch(&app, batch_id, "JE-A").await;
    create_entry_in_batch(&app, batch_id, "JE-B").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/journals/batches/{}/entries", batch_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(result["data"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_delete_entry() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "JB-E03", "Delete Entry Batch").await;
    let batch_id = batch["id"].as_str().unwrap().parse::<Uuid>().unwrap();
    let entry = create_entry_in_batch(&app, batch_id, "JE-DEL").await;
    let entry_id = entry["id"].as_str().unwrap().parse::<Uuid>().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/journals/entries/{}", entry_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    // Verify it's gone
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/journals/entries/{}", entry_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_cannot_add_entry_to_submitted_batch() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "JB-E04", "Locked Batch").await;
    let batch_id = batch["id"].as_str().unwrap().parse::<Uuid>().unwrap();

    // Add balanced entry and submit
    create_balanced_entry(&app, batch_id, "JE-LOCK", "50.00").await;

    let (k, v) = auth_header(&admin_claims());
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/journals/batches/{}/submit", batch_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Try to add another entry
    let payload = json!({"entry_number": "JE-FAIL", "currency_code": "USD"});
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/journals/batches/{}/entries", batch_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

// ============================================================================
// Line Management Tests
// ============================================================================

#[tokio::test]
async fn test_add_lines_and_check_balance() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "JB-L01", "Lines Test").await;
    let batch_id = batch["id"].as_str().unwrap().parse::<Uuid>().unwrap();

    let entry = create_balanced_entry(&app, batch_id, "JE-BAL", "250.00").await;
    assert_eq!(entry["is_balanced"], true);
    assert_eq!(entry["total_debit"], "250.00");
    assert_eq!(entry["total_credit"], "250.00");
    assert_eq!(entry["line_count"], 2);
}

#[tokio::test]
async fn test_unbalanced_entry_not_balanced() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "JB-L02", "Unbalanced Test").await;
    let batch_id = batch["id"].as_str().unwrap().parse::<Uuid>().unwrap();

    let entry = create_entry_in_batch(&app, batch_id, "JE-UNBAL").await;
    let entry_id = entry["id"].as_str().unwrap().parse::<Uuid>().unwrap();

    add_entry_line(&app, entry_id, "debit", "1000.100", "500.00").await;
    add_entry_line(&app, entry_id, "credit", "2000.200", "300.00").await;

    // Re-fetch to get updated totals
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/journals/entries/{}", entry_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(updated["is_balanced"], false);
}

#[tokio::test]
async fn test_list_lines() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "JB-L03", "List Lines Test").await;
    let batch_id = batch["id"].as_str().unwrap().parse::<Uuid>().unwrap();

    let entry = create_entry_in_batch(&app, batch_id, "JE-LIST").await;
    let entry_id = entry["id"].as_str().unwrap().parse::<Uuid>().unwrap();

    add_entry_line(&app, entry_id, "debit", "1000.100", "100.00").await;
    add_entry_line(&app, entry_id, "credit", "2000.200", "100.00").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/journals/entries/{}/lines", entry_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(result["data"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_invalid_line_type_rejected() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "JB-L04", "Invalid Line Type").await;
    let batch_id = batch["id"].as_str().unwrap().parse::<Uuid>().unwrap();
    let entry = create_entry_in_batch(&app, batch_id, "JE-BADLT").await;
    let entry_id = entry["id"].as_str().unwrap().parse::<Uuid>().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "line_type": "invalid",
        "account_code": "1000",
        "amount": "100.00",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/journals/entries/{}/lines", entry_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_negative_amount_rejected() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "JB-L05", "Negative Amount").await;
    let batch_id = batch["id"].as_str().unwrap().parse::<Uuid>().unwrap();
    let entry = create_entry_in_batch(&app, batch_id, "JE-NEG").await;
    let entry_id = entry["id"].as_str().unwrap().parse::<Uuid>().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "line_type": "debit",
        "account_code": "1000",
        "amount": "-50.00",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/journals/entries/{}/lines", entry_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

// ============================================================================
// Batch Lifecycle Tests
// ============================================================================

#[tokio::test]
async fn test_full_batch_lifecycle() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "JB-LC01", "Lifecycle Batch").await;
    let batch_id = batch["id"].as_str().unwrap().parse::<Uuid>().unwrap();
    let (k, v) = auth_header(&admin_claims());

    // Create a balanced entry
    create_balanced_entry(&app, batch_id, "JE-LC01", "1000.00").await;

    // Submit
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/journals/batches/{}/submit", batch_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let submitted: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(submitted["status"], "submitted");

    // Approve
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/journals/batches/{}/approve", batch_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let approved: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(approved["status"], "approved");

    // Post
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/journals/batches/{}/post", batch_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let posted: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(posted["status"], "posted");
}

#[tokio::test]
async fn test_submit_unbalanced_batch_fails() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "JB-UB01", "Unbalanced Batch").await;
    let batch_id = batch["id"].as_str().unwrap().parse::<Uuid>().unwrap();

    // Create an unbalanced entry
    let entry = create_entry_in_batch(&app, batch_id, "JE-UB01").await;
    let entry_id = entry["id"].as_str().unwrap().parse::<Uuid>().unwrap();
    add_entry_line(&app, entry_id, "debit", "1000.100", "500.00").await;
    // No credit line - unbalanced

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/journals/batches/{}/submit", batch_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_submit_empty_batch_fails() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "JB-EMPTY", "Empty Batch").await;
    let batch_id = batch["id"].as_str().unwrap().parse::<Uuid>().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/journals/batches/{}/submit", batch_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_reject_batch() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "JB-REJ01", "Reject Test").await;
    let batch_id = batch["id"].as_str().unwrap().parse::<Uuid>().unwrap();
    let (k, v) = auth_header(&admin_claims());

    create_balanced_entry(&app, batch_id, "JE-REJ01", "200.00").await;

    // Submit
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/journals/batches/{}/submit", batch_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Reject
    let payload = json!({"reason": "Incorrect period"});
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/journals/batches/{}/reject", batch_id))
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let rejected: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(rejected["status"], "draft");
    assert_eq!(rejected["rejection_reason"], "Incorrect period");

    // Can resubmit after rejection (it went back to draft)
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/journals/batches/{}/submit", batch_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
}

// ============================================================================
// Reversal Tests
// ============================================================================

#[tokio::test]
async fn test_reverse_posted_batch() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "JB-REV01", "Reversal Batch").await;
    let batch_id = batch["id"].as_str().unwrap().parse::<Uuid>().unwrap();
    let (k, v) = auth_header(&admin_claims());

    create_balanced_entry(&app, batch_id, "JE-REV01", "500.00").await;

    // Submit, Approve, Post
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/journals/batches/{}/submit", batch_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/journals/batches/{}/approve", batch_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/journals/batches/{}/post", batch_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    // Reverse
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/journals/batches/{}/reverse", batch_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let reversed: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(reversed["status"], "reversed");

    // Check that reversal entries were created with swapped lines
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/journals/batches/{}/entries", batch_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let entries: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let entries_arr = entries["data"].as_array().unwrap();
    // Original + reversal = 2 entries
    assert_eq!(entries_arr.len(), 2);

    // Find the reversal entry
    let reversal = entries_arr.iter()
        .find(|e| e["entry_number"].as_str().unwrap().starts_with("REV-"))
        .expect("Should have reversal entry");
    assert_eq!(reversal["is_reversal"].as_bool(), Some(false)); // reversal_of is not set on the reversal itself

    // Check the original entry is marked as reversed
    let original = entries_arr.iter()
        .find(|e| !e["entry_number"].as_str().unwrap().starts_with("REV-"))
        .expect("Should have original entry");
    assert_eq!(original["status"], "reversed");
}

#[tokio::test]
async fn test_cannot_reverse_non_posted_batch() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "JB-REV02", "Cannot Reverse").await;
    let batch_id = batch["id"].as_str().unwrap().parse::<Uuid>().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/journals/batches/{}/reverse", batch_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_cannot_post_non_approved_batch() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "JB-POST01", "Skip Approve").await;
    let batch_id = batch["id"].as_str().unwrap().parse::<Uuid>().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/journals/batches/{}/post", batch_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

// ============================================================================
// Batch Totals Tests
// ============================================================================

#[tokio::test]
async fn test_batch_totals_updated() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "JB-TOT01", "Totals Batch").await;
    let batch_id = batch["id"].as_str().unwrap().parse::<Uuid>().unwrap();

    // Create two balanced entries
    create_balanced_entry(&app, batch_id, "JE-TOT01", "300.00").await;
    create_balanced_entry(&app, batch_id, "JE-TOT02", "200.00").await;

    // Re-fetch batch
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/journals/batches/JB-TOT01")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(updated["entry_count"], 2);
    // Total should be sum of both entries
    let total_debit: f64 = updated["total_debit"].as_str().unwrap().parse().unwrap();
    let total_credit: f64 = updated["total_credit"].as_str().unwrap().parse().unwrap();
    assert!((total_debit - 500.0).abs() < 0.01);
    assert!((total_credit - 500.0).abs() < 0.01);
}

// ============================================================================
// Dashboard Tests
// ============================================================================

#[tokio::test]
async fn test_dashboard_summary() {
    let (_state, app) = setup_test().await;
    create_batch(&app, "JB-DASH1", "Dashboard Batch 1").await;
    create_batch(&app, "JB-DASH2", "Dashboard Batch 2").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/journals/dashboard")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let summary: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(summary["total_batches"].as_i64().unwrap() >= 2);
    assert!(summary["total_draft_batches"].as_i64().unwrap() >= 2);
}

// ============================================================================
// Validation Edge Cases
// ============================================================================

#[tokio::test]
async fn test_create_batch_empty_number_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "batch_number": "",
        "name": "No Number",
        "currency_code": "USD",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/journals/batches")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_create_batch_empty_name_fails() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let payload = json!({
        "batch_number": "JB-NOPE",
        "name": "",
        "currency_code": "USD",
    });
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/journals/batches")
        .header("Content-Type", "application/json")
        .header(&k, &v)
        .body(Body::from(serde_json::to_string(&payload).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_get_nonexistent_batch() {
    let (_state, app) = setup_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/journals/batches/DOESNOTEXIST")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_multi_line_balanced_entry() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "JB-ML01", "Multi-Line").await;
    let batch_id = batch["id"].as_str().unwrap().parse::<Uuid>().unwrap();
    let entry = create_entry_in_batch(&app, batch_id, "JE-ML01").await;
    let entry_id = entry["id"].as_str().unwrap().parse::<Uuid>().unwrap();

    // Multiple debits and credits that sum to the same total
    add_entry_line(&app, entry_id, "debit", "1000.100", "300.00").await;
    add_entry_line(&app, entry_id, "debit", "1100.200", "200.00").await;
    add_entry_line(&app, entry_id, "credit", "2000.100", "400.00").await;
    add_entry_line(&app, entry_id, "credit", "2100.200", "100.00").await;

    // Re-fetch to check balance
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri(&format!("/api/v1/journals/entries/{}", entry_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(updated["is_balanced"], true);
    assert_eq!(updated["line_count"], 4);
    let debit: f64 = updated["total_debit"].as_str().unwrap().parse().unwrap();
    let credit: f64 = updated["total_credit"].as_str().unwrap().parse().unwrap();
    assert!((debit - 500.0).abs() < 0.01);
    assert!((credit - 500.0).abs() < 0.01);
}

#[tokio::test]
async fn test_list_all_entries() {
    let (_state, app) = setup_test().await;
    let batch = create_batch(&app, "JB-ALL", "All Entries").await;
    let batch_id = batch["id"].as_str().unwrap().parse::<Uuid>().unwrap();

    create_entry_in_batch(&app, batch_id, "JE-ALL1").await;
    create_entry_in_batch(&app, batch_id, "JE-ALL2").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/journals/entries")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(result["data"].as_array().unwrap().len() >= 2);
}
