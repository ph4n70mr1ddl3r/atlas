//! Corporate Card Management E2E Tests
//!
//! Tests for Oracle Fusion Cloud ERP Corporate Card Management:
//! - Card programme CRUD
//! - Card issuance and lifecycle (active → suspended → cancelled / lost / stolen)
//! - Transaction import and management
//! - Transaction-to-expense matching
//! - Dispute handling
//! - Statement import and lifecycle
//! - Spending limit overrides
//! - Dashboard summary

use axum::body::Body;
use http::{Request, StatusCode};
use serde_json::json;
use tower::util::ServiceExt;
use super::common::helpers::*;

async fn setup_corporate_card_test() -> (std::sync::Arc<atlas_gateway::AppState>, axum::Router) {
    let state = build_test_state().await;
    cleanup_test_db(&state.db_pool).await;
    setup_test_db(&state.db_pool).await;
    let app = build_router(state.clone());
    (state, app)
}

async fn create_test_program(
    app: &axum::Router, code: &str, name: &str, issuer_bank: &str,
    card_network: &str, card_type: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/corporate-cards/programs")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "program_code": code,
            "name": name,
            "issuer_bank": issuer_bank,
            "card_network": card_network,
            "card_type": card_type,
            "currency_code": "USD",
            "default_single_purchase_limit": "5000.00",
            "default_monthly_limit": "20000.00",
            "default_cash_limit": "1000.00",
            "default_atm_limit": "500.00",
            "allow_cash_withdrawal": false,
            "allow_international": true,
            "auto_deactivate_on_termination": true,
            "expense_matching_method": "auto",
            "billing_cycle_day": 15,
        })).unwrap())).unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    if status != StatusCode::CREATED {
        panic!("create_test_program failed: {:?} - {}", status, String::from_utf8_lossy(&b));
    }
    serde_json::from_slice(&b).unwrap()
}

async fn create_test_card(
    app: &axum::Router, program_id: &str, masked_number: &str,
    cardholder_name: &str, cardholder_id: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/corporate-cards/cards")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "program_id": program_id,
            "card_number_masked": masked_number,
            "cardholder_name": cardholder_name,
            "cardholder_id": cardholder_id,
            "issue_date": "2024-01-01",
            "expiry_date": "2027-01-01",
        })).unwrap())).unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    if status != StatusCode::CREATED {
        panic!("create_test_card failed: {:?} - {}", status, String::from_utf8_lossy(&b));
    }
    serde_json::from_slice(&b).unwrap()
}

async fn import_test_transaction(
    app: &axum::Router, card_id: &str, reference: &str,
    merchant_name: &str, amount: &str, txn_type: &str,
) -> serde_json::Value {
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/corporate-cards/transactions")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "card_id": card_id,
            "transaction_reference": reference,
            "posting_date": "2026-04-15",
            "transaction_date": "2026-04-14",
            "merchant_name": merchant_name,
            "amount": amount,
            "currency_code": "USD",
            "transaction_type": txn_type,
        })).unwrap())).unwrap()
    ).await.unwrap();
    let status = r.status();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    if status != StatusCode::CREATED {
        panic!("import_test_transaction failed: {:?} - {}", status, String::from_utf8_lossy(&b));
    }
    serde_json::from_slice(&b).unwrap()
}

// ============================================================================
// Card Programme Tests
// ============================================================================

#[tokio::test]
async fn test_create_program() {
    let (_state, app) = setup_corporate_card_test().await;
    let program = create_test_program(&app, "VISA-01", "Corporate Visa", "Chase", "Visa", "corporate").await;
    assert_eq!(program["programCode"], "VISA-01");
    assert_eq!(program["name"], "Corporate Visa");
    assert_eq!(program["issuerBank"], "Chase");
    assert_eq!(program["cardNetwork"], "Visa");
    assert_eq!(program["cardType"], "corporate");
    assert_eq!(program["currencyCode"], "USD");
    assert_eq!(program["isActive"], true);
}

#[tokio::test]
async fn test_create_program_invalid_network() {
    let (_state, app) = setup_corporate_card_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/corporate-cards/programs")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "program_code": "BAD-NET",
            "name": "Bad Network",
            "issuer_bank": "Bank",
            "card_network": "DiscoverCard",
            "card_type": "corporate",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_program() {
    let (_state, app) = setup_corporate_card_test().await;
    let _program = create_test_program(&app, "MC-01", "Mastercard Program", "Citi", "Mastercard", "corporate").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/corporate-cards/programs/MC-01")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let fetched: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(fetched["programCode"], "MC-01");
}

#[tokio::test]
async fn test_list_programs() {
    let (_state, app) = setup_corporate_card_test().await;
    create_test_program(&app, "V-01", "Visa Prog", "Chase", "Visa", "corporate").await;
    create_test_program(&app, "M-01", "MC Prog", "Citi", "Mastercard", "purchasing").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/corporate-cards/programs")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let programs = result["data"].as_array().unwrap();
    assert!(programs.len() >= 2);
}

// ============================================================================
// Card Lifecycle Tests
// ============================================================================

#[tokio::test]
async fn test_issue_card() {
    let (_state, app) = setup_corporate_card_test().await;
    let program = create_test_program(&app, "V-02", "Visa Program", "Chase", "Visa", "corporate").await;
    let program_id = program["id"].as_str().unwrap();

    let cardholder_id = "00000000-0000-0000-0000-000000000099";
    let card = create_test_card(&app, program_id, "****-****-****-1234", "John Doe", cardholder_id).await;

    assert_eq!(card["cardNumberMasked"], "****-****-****-1234");
    assert_eq!(card["cardholderName"], "John Doe");
    assert_eq!(card["status"], "active");
    assert_eq!(card["singlePurchaseLimit"], json!("5000.00"));
    assert_eq!(card["monthlyLimit"], json!("20000.00"));
}

#[tokio::test]
async fn test_issue_card_works_for_active_program() {
    let (_state, app) = setup_corporate_card_test().await;
    let program = create_test_program(&app, "V-03", "Visa Program 3", "Chase", "Visa", "corporate").await;
    let program_id = program["id"].as_str().unwrap();

    let card = create_test_card(&app, program_id, "****-5678", "Jane Smith", "00000000-0000-0000-0000-000000000100").await;
    assert_eq!(card["status"], "active");
}

#[tokio::test]
async fn test_suspend_and_reactivate_card() {
    let (_state, app) = setup_corporate_card_test().await;
    let program = create_test_program(&app, "V-04", "Visa Program 4", "Wells", "Visa", "corporate").await;
    let card = create_test_card(&app, program["id"].as_str().unwrap(), "****-1111", "Test User", "00000000-0000-0000-0000-000000000001").await;
    let card_id = card["id"].as_str().unwrap();

    // Suspend
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/corporate-cards/cards/{}/suspend", card_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let suspended: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(suspended["status"], "suspended");

    // Reactivate
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/corporate-cards/cards/{}/reactivate", card_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let reactivated: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(reactivated["status"], "active");
}

#[tokio::test]
async fn test_cancel_card() {
    let (_state, app) = setup_corporate_card_test().await;
    let program = create_test_program(&app, "V-05", "Visa Program 5", "BOA", "Visa", "corporate").await;
    let card = create_test_card(&app, program["id"].as_str().unwrap(), "****-2222", "Cancel User", "00000000-0000-0000-0000-000000000002").await;
    let card_id = card["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/corporate-cards/cards/{}/cancel", card_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let cancelled: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(cancelled["status"], "cancelled");
}

#[tokio::test]
async fn test_report_lost_card() {
    let (_state, app) = setup_corporate_card_test().await;
    let program = create_test_program(&app, "V-06", "Visa Program 6", "Amex Bank", "Amex", "corporate").await;
    let card = create_test_card(&app, program["id"].as_str().unwrap(), "****-3333", "Lost User", "00000000-0000-0000-0000-000000000003").await;
    let card_id = card["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/corporate-cards/cards/{}/lost", card_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let lost: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(lost["status"], "lost");
}

#[tokio::test]
async fn test_report_stolen_card() {
    let (_state, app) = setup_corporate_card_test().await;
    let program = create_test_program(&app, "V-07", "Visa Program 7", "HSBC", "Visa", "travel").await;
    let card = create_test_card(&app, program["id"].as_str().unwrap(), "****-4444", "Stolen User", "00000000-0000-0000-0000-000000000004").await;
    let card_id = card["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/corporate-cards/cards/{}/stolen", card_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let stolen: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(stolen["status"], "stolen");
}

#[tokio::test]
async fn test_list_cards_by_status() {
    let (_state, app) = setup_corporate_card_test().await;
    let program = create_test_program(&app, "V-08", "Visa Program 8", "Chase", "Visa", "corporate").await;
    create_test_card(&app, program["id"].as_str().unwrap(), "****-5555", "Active User 1", "00000000-0000-0000-0000-000000000010").await;
    create_test_card(&app, program["id"].as_str().unwrap(), "****-6666", "Active User 2", "00000000-0000-0000-0000-000000000011").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/corporate-cards/cards?status=active")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let cards = result["data"].as_array().unwrap();
    assert!(cards.len() >= 2);
    for card in cards {
        assert_eq!(card["status"], "active");
    }
}

// ============================================================================
// Transaction Tests
// ============================================================================

#[tokio::test]
async fn test_import_charge_transaction() {
    let (_state, app) = setup_corporate_card_test().await;
    let program = create_test_program(&app, "V-10", "Visa Prog TXN", "Chase", "Visa", "corporate").await;
    let card = create_test_card(&app, program["id"].as_str().unwrap(), "****-7777", "Txn User", "00000000-0000-0000-0000-000000000020").await;
    let card_id = card["id"].as_str().unwrap();

    let txn = import_test_transaction(&app, card_id, "TXN-001", "Amazon", "150.00", "charge").await;
    assert_eq!(txn["transactionReference"], "TXN-001");
    assert_eq!(txn["merchantName"], "Amazon");
    assert_eq!(txn["status"], "unmatched");
    assert_eq!(txn["transactionType"], "charge");
}

#[tokio::test]
async fn test_import_multiple_transactions() {
    let (_state, app) = setup_corporate_card_test().await;
    let program = create_test_program(&app, "V-11", "Visa Prog Multi", "Citi", "Visa", "corporate").await;
    let card = create_test_card(&app, program["id"].as_str().unwrap(), "****-8888", "Multi User", "00000000-0000-0000-0000-000000000021").await;
    let card_id = card["id"].as_str().unwrap();

    import_test_transaction(&app, card_id, "TXN-010", "Hotel", "350.00", "charge").await;
    import_test_transaction(&app, card_id, "TXN-011", "Airlines", "800.00", "charge").await;
    import_test_transaction(&app, card_id, "TXN-012", "Restaurant", "75.50", "charge").await;

    // List transactions
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/corporate-cards/transactions?card_id={}", card_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let result: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let txns = result["data"].as_array().unwrap();
    assert_eq!(txns.len(), 3);
}

#[tokio::test]
async fn test_match_transaction() {
    let (_state, app) = setup_corporate_card_test().await;
    let program = create_test_program(&app, "V-12", "Visa Prog Match", "Chase", "Visa", "corporate").await;
    let card = create_test_card(&app, program["id"].as_str().unwrap(), "****-9999", "Match User", "00000000-0000-0000-0000-000000000030").await;
    let card_id = card["id"].as_str().unwrap();

    let txn = import_test_transaction(&app, card_id, "TXN-M01", "Delta Airlines", "450.00", "charge").await;
    let txn_id = txn["id"].as_str().unwrap();
    let expense_report_id = "00000000-0000-0000-0000-000000000099";

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/corporate-cards/transactions/{}/match", txn_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "expense_report_id": expense_report_id,
            "match_confidence": "high",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let matched: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(matched["status"], "matched");
}

#[tokio::test]
async fn test_unmatch_transaction() {
    let (_state, app) = setup_corporate_card_test().await;
    let program = create_test_program(&app, "V-13", "Visa Prog Unmatch", "Chase", "Visa", "corporate").await;
    let card = create_test_card(&app, program["id"].as_str().unwrap(), "****-0000", "Unmatch User", "00000000-0000-0000-0000-000000000031").await;
    let card_id = card["id"].as_str().unwrap();

    let txn = import_test_transaction(&app, card_id, "TXN-U01", "Uber", "35.00", "charge").await;
    let txn_id = txn["id"].as_str().unwrap();

    // First match
    let (k, v) = auth_header(&admin_claims());
    let _r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/corporate-cards/transactions/{}/match", txn_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "expense_report_id": "00000000-0000-0000-0000-000000000098",
        })).unwrap())).unwrap()
    ).await.unwrap();

    // Then unmatch
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/corporate-cards/transactions/{}/unmatch", txn_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let unmatched: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(unmatched["status"], "unmatched");
}

// ============================================================================
// Dispute Tests
// ============================================================================

#[tokio::test]
async fn test_dispute_and_resolve_transaction() {
    let (_state, app) = setup_corporate_card_test().await;
    let program = create_test_program(&app, "V-14", "Visa Prog Dispute", "BOA", "Visa", "corporate").await;
    let card = create_test_card(&app, program["id"].as_str().unwrap(), "****-1414", "Dispute User", "00000000-0000-0000-0000-000000000040").await;
    let card_id = card["id"].as_str().unwrap();

    let txn = import_test_transaction(&app, card_id, "TXN-D01", "Mystery Charge", "250.00", "charge").await;
    let txn_id = txn["id"].as_str().unwrap();

    // Dispute
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/corporate-cards/transactions/{}/dispute", txn_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "reason": "I did not make this charge",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let disputed: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(disputed["status"], "disputed");

    // Resolve dispute
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/corporate-cards/transactions/{}/resolve-dispute", txn_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "resolution": "Fraud confirmed, credit issued",
            "resolved_status": "approved",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let resolved: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(resolved["status"], "approved");
}

// ============================================================================
// Statement Tests
// ============================================================================

#[tokio::test]
async fn test_import_statement() {
    let (_state, app) = setup_corporate_card_test().await;
    let program = create_test_program(&app, "V-15", "Visa Prog Stmt", "Chase", "Visa", "corporate").await;
    let program_id = program["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/corporate-cards/statements")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "program_id": program_id,
            "statement_number": "STMT-2026-04",
            "statement_date": "2026-04-30",
            "billing_period_start": "2026-04-01",
            "billing_period_end": "2026-04-30",
            "opening_balance": "1250.00",
            "closing_balance": "3500.00",
            "total_charges": "2250.00",
            "total_credits": "0",
            "total_payments": "0",
            "total_fees": "0",
            "total_interest": "0",
            "payment_due_date": "2026-05-15",
            "minimum_payment": "100.00",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let stmt: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(stmt["statementNumber"], "STMT-2026-04");
    assert_eq!(stmt["status"], "imported");
}

#[tokio::test]
async fn test_reconcile_and_pay_statement() {
    let (state, app) = setup_corporate_card_test().await;
    let program = create_test_program(&app, "V-16", "Visa Prog Rec", "Chase", "Visa", "corporate").await;
    let program_id = program["id"].as_str().unwrap();

    // Import statement
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/corporate-cards/statements")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "program_id": program_id,
            "statement_number": "STMT-PAY-01",
            "statement_date": "2026-03-31",
            "billing_period_start": "2026-03-01",
            "billing_period_end": "2026-03-31",
            "opening_balance": "0",
            "closing_balance": "1500.00",
            "total_charges": "1500.00",
            "total_credits": "0",
            "total_payments": "0",
            "total_fees": "0",
            "total_interest": "0",
            "minimum_payment": "50.00",
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let stmt: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let stmt_id = stmt["id"].as_str().unwrap();

    // Manually set statement to "matched" status for reconciliation test
    sqlx::query("UPDATE _atlas.corporate_card_statements SET status = 'matched' WHERE id = $1")
        .bind(sqlx::types::Uuid::parse_str(stmt_id).unwrap())
        .execute(&state.db_pool).await.ok();

    // Reconcile
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/corporate-cards/statements/{}/reconcile", stmt_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let reconciled: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(reconciled["status"], "reconciled");

    // Pay
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/corporate-cards/statements/{}/pay", stmt_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "payment_reference": "PAY-REF-001",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let paid: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(paid["status"], "paid");
}

// ============================================================================
// Spending Limit Override Tests
// ============================================================================

#[tokio::test]
async fn test_request_and_approve_limit_override() {
    let (_state, app) = setup_corporate_card_test().await;
    let program = create_test_program(&app, "V-17", "Visa Prog Limit", "Chase", "Visa", "corporate").await;
    let card = create_test_card(&app, program["id"].as_str().unwrap(), "****-1717", "Limit User", "00000000-0000-0000-0000-000000000050").await;
    let card_id = card["id"].as_str().unwrap();

    // Request override
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/corporate-cards/limit-overrides")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "card_id": card_id,
            "override_type": "single_purchase",
            "new_value": "10000.00",
            "reason": "Need to purchase laptop for project",
            "effective_from": "2026-04-20",
            "effective_to": "2026-05-20",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::CREATED);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let ovr: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(ovr["status"], "pending");
    assert_eq!(ovr["overrideType"], "single_purchase");
    let ovr_id = ovr["id"].as_str().unwrap();

    // Approve override
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/corporate-cards/limit-overrides/{}/approve", ovr_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let approved: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(approved["status"], "approved");

    // Verify card limits were updated
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri(&format!("/api/v1/corporate-cards/cards/{}", card_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let updated_card: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(updated_card["singlePurchaseLimit"], json!("10000.00"));
}

#[tokio::test]
async fn test_reject_limit_override() {
    let (_state, app) = setup_corporate_card_test().await;
    let program = create_test_program(&app, "V-18", "Visa Prog Rej", "Chase", "Visa", "corporate").await;
    let card = create_test_card(&app, program["id"].as_str().unwrap(), "****-1818", "Reject User", "00000000-0000-0000-0000-000000000051").await;
    let card_id = card["id"].as_str().unwrap();

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST").uri("/api/v1/corporate-cards/limit-overrides")
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "card_id": card_id,
            "override_type": "monthly",
            "new_value": "50000.00",
            "reason": "Conference travel",
            "effective_from": "2026-04-20",
        })).unwrap())).unwrap()
    ).await.unwrap();
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let ovr: serde_json::Value = serde_json::from_slice(&b).unwrap();
    let ovr_id = ovr["id"].as_str().unwrap();

    // Reject
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/corporate-cards/limit-overrides/{}/reject", ovr_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let rejected: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert_eq!(rejected["status"], "rejected");
}

// ============================================================================
// Dashboard Test
// ============================================================================

#[tokio::test]
async fn test_corporate_card_dashboard() {
    let (_state, app) = setup_corporate_card_test().await;
    let program = create_test_program(&app, "V-DASH", "Dashboard Program", "Chase", "Visa", "corporate").await;
    create_test_card(&app, program["id"].as_str().unwrap(), "****-D01", "Dash User 1", "00000000-0000-0000-0000-000000000060").await;
    create_test_card(&app, program["id"].as_str().unwrap(), "****-D02", "Dash User 2", "00000000-0000-0000-0000-000000000061").await;

    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/corporate-cards/dashboard")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
    let dashboard: serde_json::Value = serde_json::from_slice(&b).unwrap();
    assert!(dashboard["totalActiveCards"].as_i64().unwrap() >= 2);
    assert!(dashboard["totalPrograms"].as_i64().unwrap() >= 1);
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[tokio::test]
async fn test_suspend_already_suspended_card_fails() {
    let (_state, app) = setup_corporate_card_test().await;
    let program = create_test_program(&app, "V-ERR", "Error Program", "Chase", "Visa", "corporate").await;
    let card = create_test_card(&app, program["id"].as_str().unwrap(), "****-E01", "Error User", "00000000-0000-0000-0000-000000000070").await;
    let card_id = card["id"].as_str().unwrap();

    // Suspend first
    let (k, v) = auth_header(&admin_claims());
    let _ = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/corporate-cards/cards/{}/suspend", card_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();

    // Try to suspend again
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/corporate-cards/cards/{}/suspend", card_id))
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_match_already_matched_transaction_fails() {
    let (_state, app) = setup_corporate_card_test().await;
    let program = create_test_program(&app, "V-ERR2", "Error Program 2", "Chase", "Visa", "corporate").await;
    let card = create_test_card(&app, program["id"].as_str().unwrap(), "****-E02", "Error User 2", "00000000-0000-0000-0000-000000000071").await;
    let txn = import_test_transaction(&app, card["id"].as_str().unwrap(), "TXN-E01", "Test", "100.00", "charge").await;
    let txn_id = txn["id"].as_str().unwrap();

    // Match once
    let (k, v) = auth_header(&admin_claims());
    let _r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/corporate-cards/transactions/{}/match", txn_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "expense_report_id": "00000000-0000-0000-0000-000000000099",
        })).unwrap())).unwrap()
    ).await.unwrap();

    // Try to match again
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/corporate-cards/transactions/{}/match", txn_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "expense_report_id": "00000000-0000-0000-0000-000000000098",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_dispute_without_reason_fails() {
    let (_state, app) = setup_corporate_card_test().await;
    let program = create_test_program(&app, "V-ERR3", "Error Program 3", "Chase", "Visa", "corporate").await;
    let card = create_test_card(&app, program["id"].as_str().unwrap(), "****-E03", "Error User 3", "00000000-0000-0000-0000-000000000072").await;
    let txn = import_test_transaction(&app, card["id"].as_str().unwrap(), "TXN-E02", "Test Merchant", "200.00", "charge").await;
    let txn_id = txn["id"].as_str().unwrap();

    // Dispute with empty reason
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("POST")
        .uri(&format!("/api/v1/corporate-cards/transactions/{}/dispute", txn_id))
        .header("Content-Type", "application/json").header(&k, &v)
        .body(Body::from(serde_json::to_string(&json!({
            "reason": "",
        })).unwrap())).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_nonexistent_card() {
    let (_state, app) = setup_corporate_card_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/corporate-cards/cards/00000000-0000-0000-0000-999999999999")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_nonexistent_transaction() {
    let (_state, app) = setup_corporate_card_test().await;
    let (k, v) = auth_header(&admin_claims());
    let r = app.clone().oneshot(Request::builder().method("GET")
        .uri("/api/v1/corporate-cards/transactions/00000000-0000-0000-0000-999999999999")
        .header(&k, &v)
        .body(Body::empty()).unwrap()
    ).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}
