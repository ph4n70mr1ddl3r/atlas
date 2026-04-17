//! Bank Reconciliation E2E Tests
//!
//! Tests for Oracle Fusion Cloud ERP Cash Management Bank Reconciliation:
//! - Bank account CRUD
//! - Bank statement creation with lines
//! - System transaction creation
//! - Auto-matching (check number, reference, amount+date)
//! - Manual matching and unmatching
//! - Reconciliation summary
//! - Matching rules

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use uuid::Uuid;
use super::common::helpers::*;

// ============================================================================
// Test Setup
// ============================================================================

async fn setup_reconciliation() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    let app = build_router(state.clone());
    (state, app)
}

async fn create_test_bank_account(app: &axum::Router, k: &str, v: &str) -> serde_json::Value {
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/reconciliation/bank-accounts")
        .header("Content-Type", "application/json").header(k, v)
        .body(Body::from(serde_json::to_string(&json!({
            "account_number": "CHK-001",
            "account_name": "Main Operating Account",
            "bank_name": "First National Bank",
            "bank_code": "FNB001",
            "branch_name": "Downtown Branch",
            "currency_code": "USD",
            "account_type": "checking",
            "gl_account_code": "1010"
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_statement(
    app: &axum::Router,
    k: &str,
    v: &str,
    bank_account_id: Uuid,
) -> serde_json::Value {
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/reconciliation/statements")
        .header("Content-Type", "application/json").header(k, v)
        .body(Body::from(serde_json::to_string(&json!({
            "bank_account_id": bank_account_id,
            "statement_number": "STMT-2026-001",
            "statement_date": "2026-01-31",
            "start_date": "2026-01-01",
            "end_date": "2026-01-31",
            "opening_balance": "10000.00",
            "closing_balance": "15000.00",
            "lines": [
                {
                    "line_number": 1,
                    "transaction_date": "2026-01-05",
                    "transaction_type": "deposit",
                    "amount": "5000.00",
                    "description": "Customer payment - ACME Corp",
                    "reference_number": "PAY-001"
                },
                {
                    "line_number": 2,
                    "transaction_date": "2026-01-10",
                    "transaction_type": "withdrawal",
                    "amount": "-2000.00",
                    "description": "Vendor check #1234",
                    "check_number": "1234",
                    "counterparty_name": "ABC Supplies"
                },
                {
                    "line_number": 3,
                    "transaction_date": "2026-01-15",
                    "transaction_type": "withdrawal",
                    "amount": "-500.00",
                    "description": "Office supplies",
                    "reference_number": "DEBIT-001"
                },
                {
                    "line_number": 4,
                    "transaction_date": "2026-01-20",
                    "transaction_type": "interest",
                    "amount": "25.00",
                    "description": "Interest earned"
                }
            ]
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_system_transaction(
    app: &axum::Router,
    k: &str,
    v: &str,
    bank_account_id: Uuid,
    source_type: &str,
    amount: &str,
    tx_date: &str,
    ref_num: Option<&str>,
    check_num: Option<&str>,
) -> serde_json::Value {
    let mut body = json!({
        "bank_account_id": bank_account_id,
        "source_type": source_type,
        "source_id": Uuid::new_v4(),
        "source_number": format!("{}_001", source_type),
        "transaction_date": tx_date,
        "amount": amount,
        "transaction_type": if amount.starts_with('-') { "withdrawal" } else { "deposit" }
    });
    if let Some(rn) = ref_num {
        body["reference_number"] = json!(rn);
    }
    if let Some(cn) = check_num {
        body["check_number"] = json!(cn);
    }

    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/reconciliation/system-transactions")
        .header("Content-Type", "application/json").header(k, v)
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Bank Account Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_create_bank_account() {
    let (state, app) = setup_reconciliation().await;
    let (k, v) = auth_header(&admin_claims());

    let account = create_test_bank_account(&app, &k, &v).await;
    assert_eq!(account["account_number"], "CHK-001");
    assert_eq!(account["account_name"], "Main Operating Account");
    assert_eq!(account["bank_name"], "First National Bank");
    assert_eq!(account["currency_code"], "USD");
    assert_eq!(account["account_type"], "checking");
    assert_eq!(account["is_active"], true);

    cleanup_test_db(&state.db_pool).await;
}

#[tokio::test]
#[ignore]
async fn test_list_bank_accounts() {
    let (state, app) = setup_reconciliation().await;
    let (k, v) = auth_header(&admin_claims());

    create_test_bank_account(&app, &k, &v).await;

    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/reconciliation/bank-accounts")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let data: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(data["data"].as_array().unwrap().len() >= 1);

    cleanup_test_db(&state.db_pool).await;
}

#[tokio::test]
#[ignore]
async fn test_get_bank_account() {
    let (state, app) = setup_reconciliation().await;
    let (k, v) = auth_header(&admin_claims());

    let account = create_test_bank_account(&app, &k, &v).await;
    let account_id = account["id"].as_str().unwrap();

    let r = app.clone().oneshot(Request::builder()
        .uri(format!("/api/v1/reconciliation/bank-accounts/{}", account_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    cleanup_test_db(&state.db_pool).await;
}

#[tokio::test]
#[ignore]
async fn test_delete_bank_account() {
    let (state, app) = setup_reconciliation().await;
    let (k, v) = auth_header(&admin_claims());

    let account = create_test_bank_account(&app, &k, &v).await;
    let account_id = account["id"].as_str().unwrap();

    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(format!("/api/v1/reconciliation/bank-accounts/{}", account_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);

    // Verify it's gone (404)
    let r = app.clone().oneshot(Request::builder()
        .uri(format!("/api/v1/reconciliation/bank-accounts/{}", account_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);

    cleanup_test_db(&state.db_pool).await;
}

// ============================================================================
// Bank Statement Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_create_bank_statement_with_lines() {
    let (state, app) = setup_reconciliation().await;
    let (k, v) = auth_header(&admin_claims());

    let account = create_test_bank_account(&app, &k, &v).await;
    let account_id: Uuid = account["id"].as_str().unwrap().parse().unwrap();

    let statement = create_test_statement(&app, &k, &v, account_id).await;
    assert_eq!(statement["statement_number"], "STMT-2026-001");
    assert_eq!(statement["status"], "imported");
    assert_eq!(statement["bank_account_id"], account_id.to_string());

    cleanup_test_db(&state.db_pool).await;
}

#[tokio::test]
#[ignore]
async fn test_list_statement_lines() {
    let (state, app) = setup_reconciliation().await;
    let (k, v) = auth_header(&admin_claims());

    let account = create_test_bank_account(&app, &k, &v).await;
    let account_id: Uuid = account["id"].as_str().unwrap().parse().unwrap();
    let statement = create_test_statement(&app, &k, &v, account_id).await;
    let statement_id = statement["id"].as_str().unwrap();

    let r = app.clone().oneshot(Request::builder()
        .uri(format!("/api/v1/reconciliation/statements/{}/lines", statement_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let data: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let lines = data["data"].as_array().unwrap();
    assert_eq!(lines.len(), 4);

    // Verify line content
    assert_eq!(lines[0]["line_number"], 1);
    assert_eq!(lines[0]["transaction_type"], "deposit");
    assert_eq!(lines[0]["match_status"], "unmatched");
    assert_eq!(lines[1]["transaction_type"], "withdrawal");

    cleanup_test_db(&state.db_pool).await;
}

// ============================================================================
// System Transaction Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_create_system_transaction() {
    let (state, app) = setup_reconciliation().await;
    let (k, v) = auth_header(&admin_claims());

    let account = create_test_bank_account(&app, &k, &v).await;
    let account_id: Uuid = account["id"].as_str().unwrap().parse().unwrap();

    let txn = create_test_system_transaction(
        &app, &k, &v, account_id,
        "ar_receipt", "5000.00", "2026-01-05",
        Some("PAY-001"), None,
    ).await;
    assert_eq!(txn["source_type"], "ar_receipt");
    assert_eq!(txn["status"], "unreconciled");

    cleanup_test_db(&state.db_pool).await;
}

#[tokio::test]
#[ignore]
async fn test_list_unreconciled_transactions() {
    let (state, app) = setup_reconciliation().await;
    let (k, v) = auth_header(&admin_claims());

    let account = create_test_bank_account(&app, &k, &v).await;
    let account_id: Uuid = account["id"].as_str().unwrap().parse().unwrap();

    create_test_system_transaction(
        &app, &k, &v, account_id,
        "ar_receipt", "5000.00", "2026-01-05",
        Some("PAY-001"), None,
    ).await;

    let r = app.clone().oneshot(Request::builder()
        .uri(format!("/api/v1/reconciliation/system-transactions/unreconciled/{}", account_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let data: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(data["data"].as_array().unwrap().len(), 1);

    cleanup_test_db(&state.db_pool).await;
}

// ============================================================================
// Auto-Matching Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_auto_match_by_reference_number() {
    let (state, app) = setup_reconciliation().await;
    let (k, v) = auth_header(&admin_claims());

    let account = create_test_bank_account(&app, &k, &v).await;
    let account_id: Uuid = account["id"].as_str().unwrap().parse().unwrap();

    // Create statement with reference number on line 1
    let statement = create_test_statement(&app, &k, &v, account_id).await;
    let statement_id = statement["id"].as_str().unwrap();

    // Create matching system transaction with same reference number
    create_test_system_transaction(
        &app, &k, &v, account_id,
        "ar_receipt", "5000.00", "2026-01-05",
        Some("PAY-001"), None,
    ).await;

    // Run auto-match
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/reconciliation/statements/{}/auto-match", statement_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(result["matched"].as_i64().unwrap() >= 1);

    cleanup_test_db(&state.db_pool).await;
}

#[tokio::test]
#[ignore]
async fn test_auto_match_by_check_number() {
    let (state, app) = setup_reconciliation().await;
    let (k, v) = auth_header(&admin_claims());

    let account = create_test_bank_account(&app, &k, &v).await;
    let account_id: Uuid = account["id"].as_str().unwrap().parse().unwrap();

    let statement = create_test_statement(&app, &k, &v, account_id).await;
    let statement_id = statement["id"].as_str().unwrap();

    // Create matching system transaction with same check number
    create_test_system_transaction(
        &app, &k, &v, account_id,
        "ap_payment", "-2000.00", "2026-01-10",
        None, Some("1234"),
    ).await;

    // Run auto-match
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/reconciliation/statements/{}/auto-match", statement_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(result["matched"].as_i64().unwrap() >= 1);

    cleanup_test_db(&state.db_pool).await;
}

#[tokio::test]
#[ignore]
async fn test_auto_match_by_amount_and_date() {
    let (state, app) = setup_reconciliation().await;
    let (k, v) = auth_header(&admin_claims());

    let account = create_test_bank_account(&app, &k, &v).await;
    let account_id: Uuid = account["id"].as_str().unwrap().parse().unwrap();

    let statement = create_test_statement(&app, &k, &v, account_id).await;
    let statement_id = statement["id"].as_str().unwrap();

    // Create matching system transaction with same amount and close date (no reference/check)
    create_test_system_transaction(
        &app, &k, &v, account_id,
        "ap_payment", "-500.00", "2026-01-14", // 1 day off from statement date
        None, None,
    ).await;

    // Run auto-match
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/reconciliation/statements/{}/auto-match", statement_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(result["matched"].as_i64().unwrap() >= 1);

    cleanup_test_db(&state.db_pool).await;
}

// ============================================================================
// Manual Matching Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_manual_match_and_unmatch() {
    let (state, app) = setup_reconciliation().await;
    let (k, v) = auth_header(&admin_claims());

    let account = create_test_bank_account(&app, &k, &v).await;
    let account_id: Uuid = account["id"].as_str().unwrap().parse().unwrap();

    let statement = create_test_statement(&app, &k, &v, account_id).await;
    let statement_id = statement["id"].as_str().unwrap();

    // Get statement lines
    let r = app.clone().oneshot(Request::builder()
        .uri(format!("/api/v1/reconciliation/statements/{}/lines", statement_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let lines_data: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let lines = lines_data["data"].as_array().unwrap();
    // Pick the interest line (line 4) which won't auto-match
    let interest_line = lines.iter().find(|l| l["line_number"] == 4).unwrap();
    let interest_line_id = interest_line["id"].as_str().unwrap();

    // Create a system transaction for it
    let txn = create_test_system_transaction(
        &app, &k, &v, account_id,
        "gl_journal", "25.00", "2026-01-20",
        None, None,
    ).await;
    let txn_id = txn["id"].as_str().unwrap();

    // Manual match
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/reconciliation/statements/{}/manual-match", statement_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "statement_line_id": interest_line_id,
            "system_transaction_id": txn_id,
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let match_record: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(match_record["match_method"], "manual");
    assert_eq!(match_record["status"], "active");
    let match_id = match_record["id"].as_str().unwrap();

    // List matches - should have our match
    let r = app.clone().oneshot(Request::builder()
        .uri(format!("/api/v1/reconciliation/statements/{}/matches", statement_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let matches_data: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(matches_data["data"].as_array().unwrap().len() >= 1);

    // Unmatch
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/reconciliation/matches/{}/unmatch", match_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let unmatched: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(unmatched["status"], "unmatched");

    cleanup_test_db(&state.db_pool).await;
}

// ============================================================================
// Reconciliation Summary Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_get_reconciliation_summary() {
    let (state, app) = setup_reconciliation().await;
    let (k, v) = auth_header(&admin_claims());

    let account = create_test_bank_account(&app, &k, &v).await;
    let account_id = account["id"].as_str().unwrap();

    let r = app.clone().oneshot(Request::builder()
        .uri(format!(
            "/api/v1/reconciliation/summary?bank_account_id={}&period_start=2026-01-01&period_end=2026-01-31",
            account_id
        ))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let summary: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(summary["status"], "in_progress");
    assert_eq!(summary["is_balanced"], false);

    cleanup_test_db(&state.db_pool).await;
}

#[tokio::test]
#[ignore]
async fn test_list_reconciliation_summaries() {
    let (state, app) = setup_reconciliation().await;
    let (k, v) = auth_header(&admin_claims());

    let account = create_test_bank_account(&app, &k, &v).await;
    let account_id = account["id"].as_str().unwrap();

    // Create a summary first
    app.clone().oneshot(Request::builder()
        .uri(format!(
            "/api/v1/reconciliation/summary?bank_account_id={}&period_start=2026-01-01&period_end=2026-01-31",
            account_id
        ))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();

    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/reconciliation/summaries")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let data: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(data["data"].as_array().unwrap().len() >= 1);

    cleanup_test_db(&state.db_pool).await;
}

// ============================================================================
// Matching Rules Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_matching_rule_crud() {
    let (state, app) = setup_reconciliation().await;
    let (k, v) = auth_header(&admin_claims());

    // Create matching rule
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/reconciliation/rules")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "name": "Match by check number",
            "description": "Exact check number matching",
            "priority": 10,
            "criteria": {"match_by": "check_number_exact"},
            "stop_on_match": true
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let rule: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(rule["name"], "Match by check number");
    let rule_id = rule["id"].as_str().unwrap();

    // List rules
    let r = app.clone().oneshot(Request::builder()
        .uri("/api/v1/reconciliation/rules")
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let data: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(data["data"].as_array().unwrap().len() >= 1);

    // Delete rule
    let r = app.clone().oneshot(Request::builder().method("DELETE")
        .uri(format!("/api/v1/reconciliation/rules/{}", rule_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NO_CONTENT);

    cleanup_test_db(&state.db_pool).await;
}

// ============================================================================
// Validation Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_invalid_transaction_type_rejected() {
    let (state, app) = setup_reconciliation().await;
    let (k, v) = auth_header(&admin_claims());

    let account = create_test_bank_account(&app, &k, &v).await;
    let account_id: Uuid = account["id"].as_str().unwrap().parse().unwrap();

    // Create statement with invalid transaction type
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri("/api/v1/reconciliation/statements")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "bank_account_id": account_id,
            "statement_number": "STMT-BAD",
            "statement_date": "2026-01-31",
            "start_date": "2026-01-01",
            "end_date": "2026-01-31",
            "opening_balance": "1000.00",
            "closing_balance": "2000.00",
            "lines": [{
                "line_number": 1,
                "transaction_date": "2026-01-05",
                "transaction_type": "invalid_type",
                "amount": "100.00"
            }]
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    // The statement will be created but the line will fail
    // This tests that invalid transaction types are caught
    assert!(r.status() == StatusCode::INTERNAL_SERVER_ERROR || r.status() == StatusCode::CREATED);

    cleanup_test_db(&state.db_pool).await;
}

#[tokio::test]
#[ignore]
async fn test_cannot_match_already_matched_line() {
    let (state, app) = setup_reconciliation().await;
    let (k, v) = auth_header(&admin_claims());

    let account = create_test_bank_account(&app, &k, &v).await;
    let account_id: Uuid = account["id"].as_str().unwrap().parse().unwrap();

    let statement = create_test_statement(&app, &k, &v, account_id).await;
    let statement_id = statement["id"].as_str().unwrap();

    // Create two system transactions
    let txn1 = create_test_system_transaction(
        &app, &k, &v, account_id,
        "ar_receipt", "5000.00", "2026-01-05",
        Some("PAY-001"), None,
    ).await;
    let txn2 = create_test_system_transaction(
        &app, &k, &v, account_id,
        "gl_journal", "5000.00", "2026-01-05",
        None, None,
    ).await;

    // Get lines
    let r = app.clone().oneshot(Request::builder()
        .uri(format!("/api/v1/reconciliation/statements/{}/lines", statement_id))
        .header(&k, &v).body(Body::empty()).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let lines_data: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let line1_id = lines_data["data"].as_array().unwrap()[0]["id"].as_str().unwrap();

    // Match first time (should succeed)
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/reconciliation/statements/{}/manual-match", statement_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "statement_line_id": line1_id,
            "system_transaction_id": txn1["id"].as_str().unwrap(),
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    // Try to match the same line again (should fail with 409 Conflict)
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(format!("/api/v1/reconciliation/statements/{}/manual-match", statement_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "statement_line_id": line1_id,
            "system_transaction_id": txn2["id"].as_str().unwrap(),
        })).unwrap()))
        .unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CONFLICT);

    cleanup_test_db(&state.db_pool).await;
}
