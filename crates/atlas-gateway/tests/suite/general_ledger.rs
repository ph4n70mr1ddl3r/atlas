//! General Ledger E2E Tests
//!
//! Tests for Oracle Fusion Cloud ERP General Ledger:
//! - Chart of Accounts CRUD
//! - Journal Entry creation with lines
//! - Journal Entry workflow (create → add lines → post)
//! - Trial Balance generation
//! - Validation and error cases

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_gl_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    let app = build_router(state.clone());
    (state, app)
}

async fn create_test_account(
    app: &axum::Router,
    code: &str,
    name: &str,
    account_type: &str,
    natural_balance: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/gl/accounts")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "account_code": code,
            "account_name": name,
            "account_type": account_type,
            "natural_balance": natural_balance,
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Failed to create GL account");
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_journal_entry(
    app: &axum::Router,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/gl/journal-entries")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "entry_date": "2026-04-15",
            "gl_date": "2026-04-15",
            "entry_type": "standard",
            "description": "Test journal entry",
            "currency_code": "USD",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED, "Failed to create journal entry");
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn add_test_journal_line(
    app: &axum::Router,
    entry_id: &str,
    account_code: &str,
    dr: &str,
    cr: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let uri = format!("/api/v1/gl/journal-entries/{}/lines", entry_id);
    let r = app.clone().oneshot(Request::builder().method("POST").uri(&uri)
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "line_type": "debit",
            "account_code": account_code,
            "description": "Test line",
            "entered_dr": dr,
            "entered_cr": cr,
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Chart of Accounts Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_create_gl_account() {
    let (_state, app) = setup_gl_test().await;

    let account = create_test_account(&app, "1000", "Cash", "asset", "debit").await;

    assert_eq!(account["account_code"], "1000");
    assert_eq!(account["account_name"], "Cash");
    assert_eq!(account["account_type"], "asset");
    assert_eq!(account["natural_balance"], "debit");
    assert!(account["id"].is_string());
}

#[tokio::test]
#[ignore]
async fn test_list_gl_accounts() {
    let (_state, app) = setup_gl_test().await;

    create_test_account(&app, "1000", "Cash", "asset", "debit").await;
    create_test_account(&app, "2000", "Accounts Payable", "liability", "credit").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET").uri("/api/v1/gl/accounts")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(result["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
#[ignore]
async fn test_get_gl_account() {
    let (_state, app) = setup_gl_test().await;

    let account = create_test_account(&app, "1000", "Cash", "asset", "debit").await;
    let account_id = account["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/gl/accounts/{}", account_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let fetched: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(fetched["account_code"], "1000");
}

#[tokio::test]
#[ignore]
async fn test_cannot_create_duplicate_account_code() {
    let (_state, app) = setup_gl_test().await;

    create_test_account(&app, "1000", "Cash", "asset", "debit").await;

    // Try duplicate
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/gl/accounts")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "account_code": "1000",
            "account_name": "Cash Duplicate",
            "account_type": "asset",
            "natural_balance": "debit",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CONFLICT);
}

// ============================================================================
// Journal Entry Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_create_journal_entry() {
    let (_state, app) = setup_gl_test().await;

    let entry = create_test_journal_entry(&app).await;

    assert_eq!(entry["status"], "draft");
    assert_eq!(entry["entry_type"], "standard");
    assert!(entry["entry_number"].as_str().unwrap().starts_with("JE-"));
}

#[tokio::test]
#[ignore]
async fn test_list_journal_entries() {
    let (_state, app) = setup_gl_test().await;

    create_test_journal_entry(&app).await;
    create_test_journal_entry(&app).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET").uri("/api/v1/gl/journal-entries")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(result["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
#[ignore]
async fn test_add_journal_line() {
    let (_state, app) = setup_gl_test().await;

    create_test_account(&app, "1000", "Cash", "asset", "debit").await;

    let entry = create_test_journal_entry(&app).await;
    let entry_id = entry["id"].as_str().unwrap();

    let line = add_test_journal_line(&app, entry_id, "1000", "1000.00", "0.00").await;

    assert_eq!(line["account_code"], "1000");
    assert_eq!(line["line_number"], 1);
}

// ============================================================================
// Journal Entry Workflow Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_post_balanced_journal_entry() {
    let (_state, app) = setup_gl_test().await;

    // Create accounts
    create_test_account(&app, "1000", "Cash", "asset", "debit").await;
    create_test_account(&app, "4000", "Revenue", "revenue", "credit").await;

    // Create journal entry
    let entry = create_test_journal_entry(&app).await;
    let entry_id = entry["id"].as_str().unwrap();

    // Add debit line
    add_test_journal_line(&app, entry_id, "1000", "500.00", "0.00").await;
    // Add credit line
    add_test_journal_line(&app, entry_id, "4000", "0.00", "500.00").await;

    // Post
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/gl/journal-entries/{}/post", entry_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let posted: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(posted["status"], "posted");
}

#[tokio::test]
#[ignore]
async fn test_cannot_post_unbalanced_journal_entry() {
    let (_state, app) = setup_gl_test().await;

    create_test_account(&app, "1000", "Cash", "asset", "debit").await;

    let entry = create_test_journal_entry(&app).await;
    let entry_id = entry["id"].as_str().unwrap();

    // Add only debit (unbalanced)
    add_test_journal_line(&app, entry_id, "1000", "500.00", "0.00").await;

    // Try to post
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/gl/journal-entries/{}/post", entry_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CONFLICT);
}

#[tokio::test]
#[ignore]
async fn test_reverse_posted_journal_entry() {
    let (_state, app) = setup_gl_test().await;

    create_test_account(&app, "1000", "Cash", "asset", "debit").await;
    create_test_account(&app, "4000", "Revenue", "revenue", "credit").await;

    let entry = create_test_journal_entry(&app).await;
    let entry_id = entry["id"].as_str().unwrap();

    add_test_journal_line(&app, entry_id, "1000", "500.00", "0.00").await;
    add_test_journal_line(&app, entry_id, "4000", "0.00", "500.00").await;

    let (k, v) = auth_header(&admin_claims());
    // Post
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/gl/journal-entries/{}/post", entry_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Reverse
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/gl/journal-entries/{}/reverse", entry_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let reversed: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(reversed["status"], "reversed");
}

// ============================================================================
// Trial Balance Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_generate_trial_balance() {
    let (_state, app) = setup_gl_test().await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/gl/trial-balance?as_of_date=2026-04-15")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let tb: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(tb["as_of_date"].is_string());
    assert!(tb["total_debit"].is_string());
    assert!(tb["total_credit"].is_string());
}
