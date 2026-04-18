//! Multi-Book Accounting E2E Tests
//!
//! Tests for Oracle Fusion Cloud ERP Multi-Book Accounting:
//! - Accounting book CRUD (primary + secondary)
//! - Account mapping rules between books
//! - Book journal entry creation and posting
//! - Automatic journal propagation from primary to secondary
//! - Journal entry reversal
//! - Propagation logs
//! - Dashboard summary
//! - Error cases and validation

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_multi_book_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    // Run the multi-book accounting migration
    let migration_sql = include_str!("../../../../migrations/028_multi_book_accounting.sql");
    sqlx::raw_sql(migration_sql).execute(&state.db_pool).await.ok();
    let app = build_router(state.clone());
    (state, app)
}

async fn create_primary_book(
    app: &axum::Router, code: &str, name: &str, coa: &str, calendar: &str, currency: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/multi-book/books")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": code,
            "name": name,
            "book_type": "primary",
            "chart_of_accounts_code": coa,
            "calendar_code": calendar,
            "currency_code": currency,
            "auto_propagation_enabled": false,
            "mapping_level": "journal",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn create_secondary_book(
    app: &axum::Router, code: &str, name: &str, coa: &str, calendar: &str, currency: &str,
    auto_propagate: bool,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/multi-book/books")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": code,
            "name": name,
            "book_type": "secondary",
            "chart_of_accounts_code": coa,
            "calendar_code": calendar,
            "currency_code": currency,
            "auto_propagation_enabled": auto_propagate,
            "mapping_level": "journal",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn create_account_mapping(
    app: &axum::Router, source_book_id: &str, target_book_id: &str,
    source_account: &str, target_account: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/multi-book/mappings")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "source_book_id": source_book_id,
            "target_book_id": target_book_id,
            "source_account_code": source_account,
            "target_account_code": target_account,
            "segment_mappings": {},
            "priority": 10,
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn create_journal_entry(
    app: &axum::Router, book_id: &str, accounting_date: &str,
    lines: Vec<serde_json::Value>,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/multi-book/entries")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "book_id": book_id,
            "header_description": "Test journal entry",
            "accounting_date": accounting_date,
            "currency_code": "USD",
            "lines": lines,
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn post_entry(app: &axum::Router, entry_id: &str) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/multi-book/entries/{}/post", entry_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from("{}")).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Tests
// ============================================================================

#[tokio::test]
async fn test_create_primary_book() {
    let (_state, app) = setup_multi_book_test().await;

    let book = create_primary_book(&app, "US_GAAP", "US GAAP Primary", "US_COA", "CAL_US", "USD").await;

    assert_eq!(book["code"], "US_GAAP");
    assert_eq!(book["name"], "US GAAP Primary");
    assert_eq!(book["bookType"], "primary");
    assert_eq!(book["chartOfAccountsCode"], "US_COA");
    assert_eq!(book["calendarCode"], "CAL_US");
    assert_eq!(book["currencyCode"], "USD");
    assert_eq!(book["status"], "draft");
    assert!(book["is_enabled"].as_bool().unwrap());
}

#[tokio::test]
async fn test_create_secondary_book() {
    let (_state, app) = setup_multi_book_test().await;

    // Must create primary first
    create_primary_book(&app, "US_GAAP", "US GAAP Primary", "US_COA", "CAL_US", "USD").await;

    let secondary = create_secondary_book(&app, "IFRS", "IFRS Secondary", "IFRS_COA", "CAL_IFRS", "EUR", true).await;

    assert_eq!(secondary["code"], "IFRS");
    assert_eq!(secondary["bookType"], "secondary");
    assert_eq!(secondary["currencyCode"], "EUR");
    assert!(secondary["autoPropagationEnabled"].as_bool().unwrap());
}

#[tokio::test]
async fn test_only_one_primary_book_allowed() {
    let (_state, app) = setup_multi_book_test().await;

    create_primary_book(&app, "US_GAAP", "US GAAP Primary", "US_COA", "CAL_US", "USD").await;

    // Try to create a second primary book
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/multi-book/books")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "code": "SECOND_PRIMARY",
            "name": "Second Primary",
            "book_type": "primary",
            "chart_of_accounts_code": "COA2",
            "calendar_code": "CAL2",
            "currency_code": "USD",
        })).unwrap())).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let err: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(err["error"].as_str().unwrap().contains("primary"));
}

#[tokio::test]
async fn test_list_accounting_books() {
    let (_state, app) = setup_multi_book_test().await;

    create_primary_book(&app, "US_GAAP", "US GAAP Primary", "US_COA", "CAL_US", "USD").await;
    create_secondary_book(&app, "IFRS", "IFRS Secondary", "IFRS_COA", "CAL_IFRS", "EUR", true).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET").uri("/api/v1/multi-book/books")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(result["data"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_get_accounting_book() {
    let (_state, app) = setup_multi_book_test().await;

    create_primary_book(&app, "US_GAAP", "US GAAP Primary", "US_COA", "CAL_US", "USD").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/multi-book/books/US_GAAP")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let book: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(book["code"], "US_GAAP");
}

#[tokio::test]
async fn test_update_book_status() {
    let (_state, app) = setup_multi_book_test().await;

    create_primary_book(&app, "US_GAAP", "US GAAP Primary", "US_COA", "CAL_US", "USD").await;

    // Activate the primary book
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("PUT")
        .uri("/api/v1/multi-book/books/US_GAAP/status")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "active"})).unwrap())).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let book: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(book["status"], "active");
}

#[tokio::test]
async fn test_cannot_deactivate_primary_book() {
    let (_state, app) = setup_multi_book_test().await;

    create_primary_book(&app, "US_GAAP", "US GAAP Primary", "US_COA", "CAL_US", "USD").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("PUT")
        .uri("/api/v1/multi-book/books/US_GAAP/status")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "inactive"})).unwrap())).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_delete_secondary_book() {
    let (_state, app) = setup_multi_book_test().await;

    create_primary_book(&app, "US_GAAP", "US GAAP Primary", "US_COA", "CAL_US", "USD").await;
    create_secondary_book(&app, "IFRS", "IFRS Secondary", "IFRS_COA", "CAL_IFRS", "EUR", true).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/multi-book/books/IFRS")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_cannot_delete_primary_book() {
    let (_state, app) = setup_multi_book_test().await;

    create_primary_book(&app, "US_GAAP", "US GAAP Primary", "US_COA", "CAL_US", "USD").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri("/api/v1/multi-book/books/US_GAAP")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_account_mapping() {
    let (_state, app) = setup_multi_book_test().await;

    let primary = create_primary_book(&app, "US_GAAP", "US GAAP Primary", "US_COA", "CAL_US", "USD").await;
    let secondary = create_secondary_book(&app, "IFRS", "IFRS Secondary", "IFRS_COA", "CAL_IFRS", "EUR", true).await;

    let mapping = create_account_mapping(&app,
        &primary["id"].as_str().unwrap(),
        &secondary["id"].as_str().unwrap(),
        "1000", "IFRS_1000"
    ).await;

    assert_eq!(mapping["sourceAccountCode"], "1000");
    assert_eq!(mapping["targetAccountCode"], "IFRS_1000");
    assert_eq!(mapping["priority"], 10);
}

#[tokio::test]
async fn test_cannot_map_same_book() {
    let (_state, app) = setup_multi_book_test().await;

    let primary = create_primary_book(&app, "US_GAAP", "US GAAP Primary", "US_COA", "CAL_US", "USD").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/multi-book/mappings")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "source_book_id": primary["id"],
            "target_book_id": primary["id"],
            "source_account_code": "1000",
            "target_account_code": "2000",
        })).unwrap())).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_list_account_mappings() {
    let (_state, app) = setup_multi_book_test().await;

    let primary = create_primary_book(&app, "US_GAAP", "US GAAP Primary", "US_COA", "CAL_US", "USD").await;
    let secondary = create_secondary_book(&app, "IFRS", "IFRS Secondary", "IFRS_COA", "CAL_IFRS", "EUR", true).await;

    create_account_mapping(&app, &primary["id"].as_str().unwrap(), &secondary["id"].as_str().unwrap(), "1000", "IFRS_1000").await;
    create_account_mapping(&app, &primary["id"].as_str().unwrap(), &secondary["id"].as_str().unwrap(), "2000", "IFRS_2000").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET").uri("/api/v1/multi-book/mappings")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(result["data"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_create_journal_entry() {
    let (_state, app) = setup_multi_book_test().await;

    let primary = create_primary_book(&app, "US_GAAP", "US GAAP Primary", "US_COA", "CAL_US", "USD").await;

    // Activate the book first
    let (k, v) = auth_header(&admin_claims());
    app.clone().oneshot(Request::builder().method("PUT")
        .uri("/api/v1/multi-book/books/US_GAAP/status")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "active"})).unwrap())).unwrap()
    ).await.unwrap();

    let entry = create_journal_entry(&app,
        &primary["id"].as_str().unwrap(),
        "2025-01-15",
        vec![
            json!({"account_code": "1000", "account_name": "Cash", "debit_amount": "1000.00", "credit_amount": "0.00", "description": "Debit cash"}),
            json!({"account_code": "4000", "account_name": "Revenue", "debit_amount": "0.00", "credit_amount": "1000.00", "description": "Credit revenue"}),
        ],
    ).await;

    assert_eq!(entry["status"], "draft");
    assert_eq!(entry["totalDebit"], "1000.00");
    assert_eq!(entry["totalCredit"], "1000.00");
    assert_eq!(entry["currencyCode"], "USD");
}

#[tokio::test]
async fn test_unbalanced_journal_entry_rejected() {
    let (_state, app) = setup_multi_book_test().await;

    let primary = create_primary_book(&app, "US_GAAP", "US GAAP Primary", "US_COA", "CAL_US", "USD").await;

    // Activate
    let (k, v) = auth_header(&admin_claims());
    app.clone().oneshot(Request::builder().method("PUT")
        .uri("/api/v1/multi-book/books/US_GAAP/status")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "active"})).unwrap())).unwrap()
    ).await.unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/multi-book/entries")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "book_id": primary["id"],
            "header_description": "Unbalanced entry",
            "accounting_date": "2025-01-15",
            "currency_code": "USD",
            "lines": [
                {"account_code": "1000", "debit_amount": "1000.00", "credit_amount": "0.00"},
                {"account_code": "4000", "debit_amount": "0.00", "credit_amount": "500.00"},
            ],
        })).unwrap())).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let err: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(err["error"].as_str().unwrap().contains("balanced"));
}

#[tokio::test]
async fn test_post_journal_entry() {
    let (_state, app) = setup_multi_book_test().await;

    let primary = create_primary_book(&app, "US_GAAP", "US GAAP Primary", "US_COA", "CAL_US", "USD").await;

    // Activate
    let (k, v) = auth_header(&admin_claims());
    app.clone().oneshot(Request::builder().method("PUT")
        .uri("/api/v1/multi-book/books/US_GAAP/status")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "active"})).unwrap())).unwrap()
    ).await.unwrap();

    let entry = create_journal_entry(&app,
        &primary["id"].as_str().unwrap(),
        "2025-01-15",
        vec![
            json!({"account_code": "1000", "debit_amount": "5000.00", "credit_amount": "0.00"}),
            json!({"account_code": "4000", "debit_amount": "0.00", "credit_amount": "5000.00"}),
        ],
    ).await;

    let posted = post_entry(&app, entry["id"].as_str().unwrap()).await;
    assert_eq!(posted["status"], "posted");
}

#[tokio::test]
async fn test_reverse_journal_entry() {
    let (_state, app) = setup_multi_book_test().await;

    let primary = create_primary_book(&app, "US_GAAP", "US GAAP Primary", "US_COA", "CAL_US", "USD").await;

    // Activate
    let (k, v) = auth_header(&admin_claims());
    app.clone().oneshot(Request::builder().method("PUT")
        .uri("/api/v1/multi-book/books/US_GAAP/status")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "active"})).unwrap())).unwrap()
    ).await.unwrap();

    let entry = create_journal_entry(&app,
        &primary["id"].as_str().unwrap(),
        "2025-01-15",
        vec![
            json!({"account_code": "1000", "debit_amount": "3000.00", "credit_amount": "0.00"}),
            json!({"account_code": "4000", "debit_amount": "0.00", "credit_amount": "3000.00"}),
        ],
    ).await;

    let posted = post_entry(&app, entry["id"].as_str().unwrap()).await;
    assert_eq!(posted["status"], "posted");

    // Reverse
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/multi-book/entries/{}/reverse", entry["id"].as_str().unwrap()))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from("{}")).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let reversal: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(reversal["status"], "posted");
}

#[tokio::test]
async fn test_auto_propagation_on_post() {
    let (_state, app) = setup_multi_book_test().await;

    let primary = create_primary_book(&app, "US_GAAP", "US GAAP Primary", "US_COA", "CAL_US", "USD").await;
    let secondary = create_secondary_book(&app, "IFRS", "IFRS Secondary", "IFRS_COA", "CAL_IFRS", "EUR", true).await;

    // Activate primary
    let (k, v) = auth_header(&admin_claims());
    app.clone().oneshot(Request::builder().method("PUT")
        .uri("/api/v1/multi-book/books/US_GAAP/status")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "active"})).unwrap())).unwrap()
    ).await.unwrap();

    // Create account mappings
    create_account_mapping(&app,
        &primary["id"].as_str().unwrap(),
        &secondary["id"].as_str().unwrap(),
        "1000", "IFRS_1000"
    ).await;
    create_account_mapping(&app,
        &primary["id"].as_str().unwrap(),
        &secondary["id"].as_str().unwrap(),
        "4000", "IFRS_4000"
    ).await;

    // Create and post a journal entry in the primary book
    let entry = create_journal_entry(&app,
        &primary["id"].as_str().unwrap(),
        "2025-01-15",
        vec![
            json!({"account_code": "1000", "debit_amount": "7500.00", "credit_amount": "0.00"}),
            json!({"account_code": "4000", "debit_amount": "0.00", "credit_amount": "7500.00"}),
        ],
    ).await;

    let posted = post_entry(&app, entry["id"].as_str().unwrap()).await;
    assert_eq!(posted["status"], "posted");

    // Check that propagation happened - look at journal entries in secondary book
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/multi-book/books/{}/entries?status=propagated", secondary["id"].as_str().unwrap()))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let entries = result["data"].as_array().unwrap();
    assert_eq!(entries.len(), 1);

    let propagated = &entries[0];
    assert_eq!(propagated["status"], "propagated");
    assert!(propagated["isAutoPropagated"].as_bool().unwrap());
    assert_eq!(propagated["totalDebit"], "7500.00");
    assert_eq!(propagated["totalCredit"], "7500.00");
}

#[tokio::test]
async fn test_propagation_logs() {
    let (_state, app) = setup_multi_book_test().await;

    let primary = create_primary_book(&app, "US_GAAP", "US GAAP Primary", "US_COA", "CAL_US", "USD").await;
    let secondary = create_secondary_book(&app, "IFRS", "IFRS Secondary", "IFRS_COA", "CAL_IFRS", "EUR", true).await;

    // Activate primary
    let (k, v) = auth_header(&admin_claims());
    app.clone().oneshot(Request::builder().method("PUT")
        .uri("/api/v1/multi-book/books/US_GAAP/status")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "active"})).unwrap())).unwrap()
    ).await.unwrap();

    // Create mapping for one account only
    create_account_mapping(&app,
        &primary["id"].as_str().unwrap(),
        &secondary["id"].as_str().unwrap(),
        "1000", "IFRS_1000"
    ).await;

    // Create entry with two lines, only one mapped
    let entry = create_journal_entry(&app,
        &primary["id"].as_str().unwrap(),
        "2025-01-15",
        vec![
            json!({"account_code": "1000", "debit_amount": "2000.00", "credit_amount": "0.00"}),
            json!({"account_code": "9999", "debit_amount": "0.00", "credit_amount": "2000.00"}),
        ],
    ).await;

    let _posted = post_entry(&app, entry["id"].as_str().unwrap()).await;

    // Check propagation logs
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/multi-book/propagation-logs")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let logs = result["data"].as_array().unwrap();
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0]["status"], "completed");
    assert_eq!(logs[0]["linesPropagated"], 1);
    assert_eq!(logs[0]["linesUnmapped"], 1);
}

#[tokio::test]
async fn test_manual_propagation() {
    let (_state, app) = setup_multi_book_test().await;

    let primary = create_primary_book(&app, "US_GAAP", "US GAAP Primary", "US_COA", "CAL_US", "USD").await;
    let secondary = create_secondary_book(&app, "IFRS", "IFRS Secondary", "IFRS_COA", "CAL_IFRS", "EUR", false).await;

    // Activate primary
    let (k, v) = auth_header(&admin_claims());
    app.clone().oneshot(Request::builder().method("PUT")
        .uri("/api/v1/multi-book/books/US_GAAP/status")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "active"})).unwrap())).unwrap()
    ).await.unwrap();

    // Create mapping
    create_account_mapping(&app,
        &primary["id"].as_str().unwrap(),
        &secondary["id"].as_str().unwrap(),
        "1000", "IFRS_1000"
    ).await;
    create_account_mapping(&app,
        &primary["id"].as_str().unwrap(),
        &secondary["id"].as_str().unwrap(),
        "4000", "IFRS_4000"
    ).await;

    // Create and post entry (auto-propagation is disabled for this secondary book)
    let entry = create_journal_entry(&app,
        &primary["id"].as_str().unwrap(),
        "2025-01-15",
        vec![
            json!({"account_code": "1000", "debit_amount": "1500.00", "credit_amount": "0.00"}),
            json!({"account_code": "4000", "debit_amount": "0.00", "credit_amount": "1500.00"}),
        ],
    ).await;
    let _posted = post_entry(&app, entry["id"].as_str().unwrap()).await;

    // Manually propagate to secondary book
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/multi-book/entries/{}/propagate", entry["id"].as_str().unwrap()))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "target_book_id": secondary["id"],
        })).unwrap())).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let log: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(log["status"], "completed");
    assert_eq!(log["linesPropagated"], 2);
    assert_eq!(log["linesUnmapped"], 0);
}

#[tokio::test]
async fn test_journal_lines() {
    let (_state, app) = setup_multi_book_test().await;

    let primary = create_primary_book(&app, "US_GAAP", "US GAAP Primary", "US_COA", "CAL_US", "USD").await;

    // Activate
    let (k, v) = auth_header(&admin_claims());
    app.clone().oneshot(Request::builder().method("PUT")
        .uri("/api/v1/multi-book/books/US_GAAP/status")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "active"})).unwrap())).unwrap()
    ).await.unwrap();

    let entry = create_journal_entry(&app,
        &primary["id"].as_str().unwrap(),
        "2025-01-15",
        vec![
            json!({"account_code": "1000", "account_name": "Cash", "debit_amount": "500.00", "credit_amount": "0.00", "description": "Debit"}),
            json!({"account_code": "4000", "account_name": "Revenue", "debit_amount": "0.00", "credit_amount": "500.00", "description": "Credit"}),
        ],
    ).await;

    // Get lines
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/multi-book/entries/{}/lines", entry["id"].as_str().unwrap()))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let lines = result["data"].as_array().unwrap();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0]["accountCode"], "1000");
    assert_eq!(lines[0]["debitAmount"], "500.00");
    assert_eq!(lines[1]["accountCode"], "4000");
    assert_eq!(lines[1]["creditAmount"], "500.00");
}

#[tokio::test]
async fn test_multi_book_summary() {
    let (_state, app) = setup_multi_book_test().await;

    create_primary_book(&app, "US_GAAP", "US GAAP Primary", "US_COA", "CAL_US", "USD").await;
    create_secondary_book(&app, "IFRS", "IFRS Secondary", "IFRS_COA", "CAL_IFRS", "EUR", true).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/multi-book/dashboard")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let summary: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(summary["bookCount"], 2);
    assert_eq!(summary["primaryBookCode"], "US_GAAP");
    assert_eq!(summary["secondaryBookCount"], 1);
}

#[tokio::test]
async fn test_cannot_post_to_inactive_book() {
    let (_state, app) = setup_multi_book_test().await;

    let primary = create_primary_book(&app, "US_GAAP", "US GAAP Primary", "US_COA", "CAL_US", "USD").await;
    // Book is in "draft" status, not "active"

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/multi-book/entries")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "book_id": primary["id"],
            "accounting_date": "2025-01-15",
            "currency_code": "USD",
            "lines": [
                {"account_code": "1000", "debit_amount": "100.00", "credit_amount": "0.00"},
                {"account_code": "4000", "debit_amount": "0.00", "credit_amount": "100.00"},
            ],
        })).unwrap())).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_cannot_post_draft_entry_twice() {
    let (_state, app) = setup_multi_book_test().await;

    let primary = create_primary_book(&app, "US_GAAP", "US GAAP Primary", "US_COA", "CAL_US", "USD").await;

    // Activate
    let (k, v) = auth_header(&admin_claims());
    app.clone().oneshot(Request::builder().method("PUT")
        .uri("/api/v1/multi-book/books/US_GAAP/status")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({"status": "active"})).unwrap())).unwrap()
    ).await.unwrap();

    let entry = create_journal_entry(&app,
        &primary["id"].as_str().unwrap(),
        "2025-01-15",
        vec![
            json!({"account_code": "1000", "debit_amount": "100.00", "credit_amount": "0.00"}),
            json!({"account_code": "4000", "debit_amount": "0.00", "credit_amount": "100.00"}),
        ],
    ).await;

    let _posted = post_entry(&app, entry["id"].as_str().unwrap()).await;

    // Try to post again
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/multi-book/entries/{}/post", entry["id"].as_str().unwrap()))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from("{}")).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_delete_account_mapping() {
    let (_state, app) = setup_multi_book_test().await;

    let primary = create_primary_book(&app, "US_GAAP", "US GAAP Primary", "US_COA", "CAL_US", "USD").await;
    let secondary = create_secondary_book(&app, "IFRS", "IFRS Secondary", "IFRS_COA", "CAL_IFRS", "EUR", true).await;

    let mapping = create_account_mapping(&app,
        &primary["id"].as_str().unwrap(),
        &secondary["id"].as_str().unwrap(),
        "1000", "IFRS_1000"
    ).await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(&format!("/api/v1/multi-book/mappings/{}", mapping["id"].as_str().unwrap()))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    assert_eq!(r.status(), StatusCode::NO_CONTENT);
}
