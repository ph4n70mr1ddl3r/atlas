//! Subledger Accounting E2E Tests
//!
//! Tests for Oracle Fusion Cloud ERP: Financials > General Ledger > Subledger Accounting
//! Tests exercise the engine + repository layer directly to avoid axum route conflicts
//! with the generic /:entity/:id catch-all routes.

use super::common::helpers::*;
use uuid::Uuid;
use chrono::NaiveDate;
use std::sync::Arc;

async fn setup_sla_test() -> Arc<atlas_gateway::AppState> {
    let state = build_test_state().await;
    setup_test_db(&state.db_pool).await;
    cleanup_sla_data(&state.db_pool).await;
    state
}

async fn cleanup_sla_data(pool: &sqlx::PgPool) {
    // Delete in dependency order: child tables first, parent tables last
    sqlx::query("DELETE FROM _atlas.subledger_distributions").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.sla_events").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.subledger_journal_lines").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.subledger_journal_entries").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.gl_transfer_log").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.accounting_derivation_rules").execute(pool).await.ok();
    sqlx::query("DELETE FROM _atlas.accounting_methods").execute(pool).await.ok();
}

fn org_id() -> Uuid {
    Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap()
}

fn user_id() -> Uuid {
    Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap()
}

// ============================================================================
// Accounting Method Tests
// ============================================================================

#[tokio::test]
async fn test_create_accounting_method() {
    let state = setup_sla_test().await;
    let method = state.sla_engine.create_accounting_method(
        org_id(), "AP_STD", "AP Standard Method", Some("Standard payables accounting"),
        "payables", "invoice", None,
        Some(true), Some(false), Some(true), None, None,
        Some(true), None, None, None, Some(user_id()),
    ).await.unwrap();
    assert_eq!(method.code, "AP_STD");
    assert_eq!(method.name, "AP Standard Method");
    assert_eq!(method.application, "payables");
    assert_eq!(method.transaction_type, "invoice");
    assert_eq!(method.event_class, "create");
    assert!(method.auto_accounting);
    assert!(method.require_balancing);
}

#[tokio::test]
async fn test_create_method_invalid_application() {
    let state = setup_sla_test().await;
    let result = state.sla_engine.create_accounting_method(
        org_id(), "BAD", "Bad", None,
        "nonexistent_app", "invoice", None,
        None, None, None, None, None, None, None, None, None, None,
    ).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, atlas_shared::AtlasError::ValidationFailed(msg) if msg.contains("Invalid application")));
}

#[tokio::test]
async fn test_create_method_empty_code_rejected() {
    let state = setup_sla_test().await;
    let result = state.sla_engine.create_accounting_method(
        org_id(), "", "Has Name", None,
        "payables", "invoice", None,
        None, None, None, None, None, None, None, None, None, None,
    ).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, atlas_shared::AtlasError::ValidationFailed(msg) if msg.contains("code")));
}

#[tokio::test]
async fn test_get_accounting_method() {
    let state = setup_sla_test().await;
    state.sla_engine.create_accounting_method(
        org_id(), "GET_TEST", "Get Test", None,
        "receivables", "receipt", None,
        None, None, None, None, None, None, None, None, None, None,
    ).await.unwrap();
    let fetched = state.sla_engine.get_accounting_method(org_id(), "GET_TEST").await.unwrap().unwrap();
    assert_eq!(fetched.code, "GET_TEST");
    assert_eq!(fetched.application, "receivables");
}

#[tokio::test]
async fn test_get_nonexistent_method() {
    let state = setup_sla_test().await;
    let result = state.sla_engine.get_accounting_method(org_id(), "NO_SUCH").await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_list_accounting_methods() {
    let state = setup_sla_test().await;
    state.sla_engine.create_accounting_method(
        org_id(), "LIST_A", "List A", None,
        "payables", "invoice", None,
        None, None, None, None, None, None, None, None, None, None,
    ).await.unwrap();
    state.sla_engine.create_accounting_method(
        org_id(), "LIST_B", "List B", None,
        "expenses", "report", None,
        None, None, None, None, None, None, None, None, None, None,
    ).await.unwrap();
    let methods = state.sla_engine.list_accounting_methods(org_id(), None).await.unwrap();
    assert!(methods.len() >= 2);
}

#[tokio::test]
async fn test_list_methods_by_application() {
    let state = setup_sla_test().await;
    state.sla_engine.create_accounting_method(
        org_id(), "FILT_P", "Filter Payables", None,
        "payables", "invoice", None,
        None, None, None, None, None, None, None, None, None, None,
    ).await.unwrap();
    state.sla_engine.create_accounting_method(
        org_id(), "FILT_E", "Filter Expenses", None,
        "expenses", "report", None,
        None, None, None, None, None, None, None, None, None, None,
    ).await.unwrap();
    let payables = state.sla_engine.list_accounting_methods(org_id(), Some("payables")).await.unwrap();
    assert!(payables.len() >= 1);
    for m in &payables {
        assert_eq!(m.application, "payables");
    }
}

#[tokio::test]
async fn test_delete_accounting_method() {
    let state = setup_sla_test().await;
    state.sla_engine.create_accounting_method(
        org_id(), "DEL_ME", "Delete Me", None,
        "general", "transaction", None,
        None, None, None, None, None, None, None, None, None, None,
    ).await.unwrap();
    state.sla_engine.delete_accounting_method(org_id(), "DEL_ME").await.unwrap();
    let result = state.sla_engine.get_accounting_method(org_id(), "DEL_ME").await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_delete_nonexistent_method() {
    let state = setup_sla_test().await;
    let result = state.sla_engine.delete_accounting_method(org_id(), "NO_SUCH").await;
    assert!(result.is_err());
}

// ============================================================================
// Derivation Rule Tests
// ============================================================================

#[tokio::test]
async fn test_create_constant_derivation_rule() {
    let state = setup_sla_test().await;
    let method = state.sla_engine.create_accounting_method(
        org_id(), "DRV_CONST", "Drv Const", None,
        "payables", "invoice", None,
        None, None, None, None, None, None, None, None, None, None,
    ).await.unwrap();
    let rule = state.sla_engine.create_derivation_rule(
        org_id(), method.id, "AP_DEBIT", "AP Debit Account", None,
        "debit", 10, serde_json::json!({}), None,
        "constant", Some("2100"), serde_json::json!({}), None,
        10, None, None, None,
    ).await.unwrap();
    assert_eq!(rule.code, "AP_DEBIT");
    assert_eq!(rule.derivation_type, "constant");
    assert_eq!(rule.fixed_account_code, Some("2100".to_string()));
    assert_eq!(rule.line_type, "debit");
    assert!(rule.is_active);
}

#[tokio::test]
async fn test_create_lookup_derivation_rule() {
    let state = setup_sla_test().await;
    let method = state.sla_engine.create_accounting_method(
        org_id(), "DRV_LOOKUP", "Drv Lookup", None,
        "expenses", "report", None,
        None, None, None, None, None, None, None, None, None, None,
    ).await.unwrap();
    let lookup = serde_json::json!({"Travel": "6100", "Meals": "6200", "Office": "6300"});
    let rule = state.sla_engine.create_derivation_rule(
        org_id(), method.id, "EXP_DEBIT_CAT", "Expense Debit by Category", None,
        "debit", 10, serde_json::json!({}), Some("expense_category"),
        "lookup", None, lookup, None,
        10, None, None, None,
    ).await.unwrap();
    assert_eq!(rule.derivation_type, "lookup");
    assert_eq!(rule.source_field, Some("expense_category".to_string()));
}

#[tokio::test]
async fn test_create_formula_derivation_rule() {
    let state = setup_sla_test().await;
    let method = state.sla_engine.create_accounting_method(
        org_id(), "DRV_FORMULA", "Drv Formula", None,
        "projects", "cost", None,
        None, None, None, None, None, None, None, None, None, None,
    ).await.unwrap();
    let rule = state.sla_engine.create_derivation_rule(
        org_id(), method.id, "PROJ_FORMULA", "Project Cost Formula", None,
        "debit", 5, serde_json::json!({}), None,
        "formula", None, serde_json::json!({}), Some("CONCAT(dept, '-', cc)"),
        5, None, None, None,
    ).await.unwrap();
    assert_eq!(rule.derivation_type, "formula");
    assert_eq!(rule.formula_expression, Some("CONCAT(dept, '-', cc)".to_string()));
}

#[tokio::test]
async fn test_constant_rule_requires_fixed_account() {
    let state = setup_sla_test().await;
    let method = state.sla_engine.create_accounting_method(
        org_id(), "DRV_NOFIX", "No Fixed", None,
        "payables", "invoice", None,
        None, None, None, None, None, None, None, None, None, None,
    ).await.unwrap();
    let result = state.sla_engine.create_derivation_rule(
        org_id(), method.id, "MISSING", "Missing account", None,
        "debit", 10, serde_json::json!({}), None,
        "constant", None, serde_json::json!({}), None,
        10, None, None, None,
    ).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, atlas_shared::AtlasError::ValidationFailed(msg) if msg.contains("fixed_account_code")));
}

#[tokio::test]
async fn test_lookup_rule_requires_source_field() {
    let state = setup_sla_test().await;
    let method = state.sla_engine.create_accounting_method(
        org_id(), "DRV_NOSRC", "No Source", None,
        "expenses", "report", None,
        None, None, None, None, None, None, None, None, None, None,
    ).await.unwrap();
    let result = state.sla_engine.create_derivation_rule(
        org_id(), method.id, "MISSING_SRC", "Missing source", None,
        "debit", 10, serde_json::json!({}), None,
        "lookup", None, serde_json::json!({}), None,
        10, None, None, None,
    ).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, atlas_shared::AtlasError::ValidationFailed(msg) if msg.contains("source_field")));
}

#[tokio::test]
async fn test_list_derivation_rules() {
    let state = setup_sla_test().await;
    let method = state.sla_engine.create_accounting_method(
        org_id(), "DRV_LIST", "Drv List", None,
        "payables", "invoice", None,
        None, None, None, None, None, None, None, None, None, None,
    ).await.unwrap();
    state.sla_engine.create_derivation_rule(
        org_id(), method.id, "AP_DR", "AP Debit", None,
        "debit", 10, serde_json::json!({}), None,
        "constant", Some("2100"), serde_json::json!({}), None,
        10, None, None, None,
    ).await.unwrap();
    state.sla_engine.create_derivation_rule(
        org_id(), method.id, "AP_CR", "AP Credit", None,
        "credit", 10, serde_json::json!({}), None,
        "constant", Some("1000"), serde_json::json!({}), None,
        10, None, None, None,
    ).await.unwrap();
    let rules = state.sla_engine.list_derivation_rules(org_id(), method.id).await.unwrap();
    assert!(rules.len() >= 2);
}

#[tokio::test]
async fn test_delete_derivation_rule() {
    let state = setup_sla_test().await;
    let method = state.sla_engine.create_accounting_method(
        org_id(), "DRV_DEL", "Drv Del", None,
        "payables", "invoice", None,
        None, None, None, None, None, None, None, None, None, None,
    ).await.unwrap();
    state.sla_engine.create_derivation_rule(
        org_id(), method.id, "DEL_RULE", "Delete Me", None,
        "debit", 10, serde_json::json!({}), None,
        "constant", Some("9999"), serde_json::json!({}), None,
        10, None, None, None,
    ).await.unwrap();
    state.sla_engine.delete_derivation_rule(org_id(), method.id, "DEL_RULE").await.unwrap();
}

// ============================================================================
// Journal Entry Tests
// ============================================================================

#[tokio::test]
async fn test_create_journal_entry() {
    let state = setup_sla_test().await;
    let entry = state.sla_engine.create_journal_entry(
        org_id(), "payables", "invoice",
        Uuid::new_v4(), Some("INV-001"), None,
        Some("Test SLA entry"), Some("REF-001"),
        NaiveDate::from_ymd_opt(2026, 4, 1).unwrap(), Some("Apr-2026"),
        "USD", "USD", None, None, None,
        Some(user_id()),
    ).await.unwrap();
    assert!(entry.entry_number.starts_with("SLA-"));
    assert_eq!(entry.source_application, "payables");
    assert_eq!(entry.source_transaction_type, "invoice");
    assert_eq!(entry.status, "draft");
    assert_eq!(entry.currency_code, "USD");
    assert!(!entry.is_balanced);
}

#[tokio::test]
async fn test_get_journal_entry() {
    let state = setup_sla_test().await;
    let entry = state.sla_engine.create_journal_entry(
        org_id(), "receivables", "receipt",
        Uuid::new_v4(), None, None,
        None, None,
        NaiveDate::from_ymd_opt(2026, 4, 1).unwrap(), None,
        "USD", "USD", None, None, None, None,
    ).await.unwrap();
    let fetched = state.sla_engine.get_journal_entry(entry.id).await.unwrap().unwrap();
    assert_eq!(fetched.id, entry.id);
    assert_eq!(fetched.entry_number, entry.entry_number);
}

#[tokio::test]
async fn test_list_journal_entries() {
    let state = setup_sla_test().await;
    state.sla_engine.create_journal_entry(
        org_id(), "payables", "invoice",
        Uuid::new_v4(), None, None,
        None, None,
        NaiveDate::from_ymd_opt(2026, 4, 1).unwrap(), None,
        "USD", "USD", None, None, None, None,
    ).await.unwrap();
    state.sla_engine.create_journal_entry(
        org_id(), "expenses", "report",
        Uuid::new_v4(), None, None,
        None, None,
        NaiveDate::from_ymd_opt(2026, 4, 1).unwrap(), None,
        "USD", "USD", None, None, None, None,
    ).await.unwrap();
    let entries = state.sla_engine.list_journal_entries(org_id(), None, None, None, None, None).await.unwrap();
    assert!(entries.len() >= 2);
}

#[tokio::test]
async fn test_list_entries_by_status() {
    let state = setup_sla_test().await;
    state.sla_engine.create_journal_entry(
        org_id(), "payables", "invoice",
        Uuid::new_v4(), None, None,
        None, None,
        NaiveDate::from_ymd_opt(2026, 4, 1).unwrap(), None,
        "USD", "USD", None, None, None, None,
    ).await.unwrap();
    let drafts = state.sla_engine.list_journal_entries(org_id(), Some("draft"), None, None, None, None).await.unwrap();
    assert!(drafts.len() >= 1);
    for e in &drafts { assert_eq!(e.status, "draft"); }
}

#[tokio::test]
async fn test_list_entries_invalid_status() {
    let state = setup_sla_test().await;
    let result = state.sla_engine.list_journal_entries(org_id(), Some("invalid_status"), None, None, None, None).await;
    assert!(result.is_err());
}

// ============================================================================
// Journal Lines & Balancing Tests
// ============================================================================

#[tokio::test]
async fn test_add_debit_and_credit_lines() {
    let state = setup_sla_test().await;
    let entry = state.sla_engine.create_journal_entry(
        org_id(), "payables", "invoice",
        Uuid::new_v4(), None, None, None, None,
        NaiveDate::from_ymd_opt(2026, 4, 1).unwrap(), None,
        "USD", "USD", None, None, None, None,
    ).await.unwrap();
    // Add debit
    let debit = state.sla_engine.add_journal_line(
        org_id(), entry.id, "debit", "6100", Some("Travel Expense"), None,
        "1500.00", "1500.00", "USD", None, None,
        None, None, None, None, None, None,
        None, None, None, None, None,
    ).await.unwrap();
    assert_eq!(debit.line_type, "debit");
    assert_eq!(debit.account_code, "6100");
    assert_eq!(debit.line_number, 1);
    // Add credit
    let credit = state.sla_engine.add_journal_line(
        org_id(), entry.id, "credit", "2100", Some("Accounts Payable"), None,
        "1500.00", "1500.00", "USD", None, None,
        None, None, None, None, None, None,
        None, None, None, None, None,
    ).await.unwrap();
    assert_eq!(credit.line_type, "credit");
    assert_eq!(credit.line_number, 2);
    // Verify entry is balanced
    let updated = state.sla_engine.get_journal_entry(entry.id).await.unwrap().unwrap();
    assert!(updated.is_balanced);
    assert_eq!(updated.total_debit, "1500.00");
    assert_eq!(updated.total_credit, "1500.00");
}

#[tokio::test]
async fn test_unbalanced_entry() {
    let state = setup_sla_test().await;
    let entry = state.sla_engine.create_journal_entry(
        org_id(), "payables", "invoice",
        Uuid::new_v4(), None, None, None, None,
        NaiveDate::from_ymd_opt(2026, 4, 1).unwrap(), None,
        "USD", "USD", None, None, None, None,
    ).await.unwrap();
    state.sla_engine.add_journal_line(
        org_id(), entry.id, "debit", "6100", None, None,
        "1000.00", "1000.00", "USD", None, None,
        None, None, None, None, None, None,
        None, None, None, None, None,
    ).await.unwrap();
    state.sla_engine.add_journal_line(
        org_id(), entry.id, "credit", "2100", None, None,
        "750.00", "750.00", "USD", None, None,
        None, None, None, None, None, None,
        None, None, None, None, None,
    ).await.unwrap();
    let updated = state.sla_engine.get_journal_entry(entry.id).await.unwrap().unwrap();
    assert!(!updated.is_balanced);
}

#[tokio::test]
async fn test_add_line_invalid_type_rejected() {
    let state = setup_sla_test().await;
    let entry = state.sla_engine.create_journal_entry(
        org_id(), "payables", "invoice",
        Uuid::new_v4(), None, None, None, None,
        NaiveDate::from_ymd_opt(2026, 4, 1).unwrap(), None,
        "USD", "USD", None, None, None, None,
    ).await.unwrap();
    let result = state.sla_engine.add_journal_line(
        org_id(), entry.id, "invalid", "2100", None, None,
        "100.00", "100.00", "USD", None, None,
        None, None, None, None, None, None,
        None, None, None, None, None,
    ).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_add_line_zero_debit_rejected() {
    let state = setup_sla_test().await;
    let entry = state.sla_engine.create_journal_entry(
        org_id(), "payables", "invoice",
        Uuid::new_v4(), None, None, None, None,
        NaiveDate::from_ymd_opt(2026, 4, 1).unwrap(), None,
        "USD", "USD", None, None, None, None,
    ).await.unwrap();
    let result = state.sla_engine.add_journal_line(
        org_id(), entry.id, "debit", "2100", None, None,
        "0.00", "0.00", "USD", None, None,
        None, None, None, None, None, None,
        None, None, None, None, None,
    ).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_list_journal_lines() {
    let state = setup_sla_test().await;
    let entry = state.sla_engine.create_journal_entry(
        org_id(), "payables", "invoice",
        Uuid::new_v4(), None, None, None, None,
        NaiveDate::from_ymd_opt(2026, 4, 1).unwrap(), None,
        "USD", "USD", None, None, None, None,
    ).await.unwrap();
    state.sla_engine.add_journal_line(
        org_id(), entry.id, "debit", "6100", None, None,
        "500.00", "500.00", "USD", None, None,
        None, None, None, None, None, None,
        None, None, None, None, None,
    ).await.unwrap();
    state.sla_engine.add_journal_line(
        org_id(), entry.id, "credit", "2100", None, None,
        "500.00", "500.00", "USD", None, None,
        None, None, None, None, None, None,
        None, None, None, None, None,
    ).await.unwrap();
    let lines = state.sla_engine.list_journal_lines(entry.id).await.unwrap();
    assert_eq!(lines.len(), 2);
}

// ============================================================================
// Entry Lifecycle Tests
// ============================================================================

async fn create_balanced_entry(state: &Arc<atlas_gateway::AppState>, amount: &str) -> atlas_shared::SubledgerJournalEntry {
    let entry = state.sla_engine.create_journal_entry(
        org_id(), "payables", "invoice",
        Uuid::new_v4(), None, None, None, None,
        NaiveDate::from_ymd_opt(2026, 4, 1).unwrap(), Some("Apr-2026"),
        "USD", "USD", None, None, None, Some(user_id()),
    ).await.unwrap();
    state.sla_engine.add_journal_line(
        org_id(), entry.id, "debit", "6100", None, None,
        amount, amount, "USD", None, None,
        None, None, None, None, None, None,
        None, None, None, None, None,
    ).await.unwrap();
    state.sla_engine.add_journal_line(
        org_id(), entry.id, "credit", "2100", None, None,
        amount, amount, "USD", None, None,
        None, None, None, None, None, None,
        None, None, None, None, None,
    ).await.unwrap();
    state.sla_engine.get_journal_entry(entry.id).await.unwrap().unwrap()
}

#[tokio::test]
async fn test_account_unbalanced_entry_rejected() {
    let state = setup_sla_test().await;
    let entry = state.sla_engine.create_journal_entry(
        org_id(), "payables", "invoice",
        Uuid::new_v4(), None, None, None, None,
        NaiveDate::from_ymd_opt(2026, 4, 1).unwrap(), None,
        "USD", "USD", None, None, None, None,
    ).await.unwrap();
    let result = state.sla_engine.account_entry(entry.id, Some(user_id())).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_full_entry_lifecycle() {
    let state = setup_sla_test().await;
    let entry = create_balanced_entry(&state, "2000.00").await;
    // Account (draft → accounted)
    let accounted = state.sla_engine.account_entry(entry.id, Some(user_id())).await.unwrap();
    assert_eq!(accounted.status, "accounted");
    // Post (accounted → posted)
    let posted = state.sla_engine.post_entry(entry.id, Some(user_id())).await.unwrap();
    assert_eq!(posted.status, "posted");
}

#[tokio::test]
async fn test_cannot_post_draft_entry() {
    let state = setup_sla_test().await;
    let entry = create_balanced_entry(&state, "500.00").await;
    let result = state.sla_engine.post_entry(entry.id, Some(user_id())).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_cannot_account_already_accounted() {
    let state = setup_sla_test().await;
    let entry = create_balanced_entry(&state, "500.00").await;
    state.sla_engine.account_entry(entry.id, Some(user_id())).await.unwrap();
    let result = state.sla_engine.account_entry(entry.id, Some(user_id())).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_reverse_posted_entry() {
    let state = setup_sla_test().await;
    let entry = create_balanced_entry(&state, "3000.00").await;
    state.sla_engine.account_entry(entry.id, Some(user_id())).await.unwrap();
    state.sla_engine.post_entry(entry.id, Some(user_id())).await.unwrap();
    let reversed = state.sla_engine.reverse_entry(entry.id, "Incorrect posting", Some(user_id())).await.unwrap();
    assert_eq!(reversed.status, "reversed");
}

#[tokio::test]
async fn test_reverse_requires_reason() {
    let state = setup_sla_test().await;
    let entry = create_balanced_entry(&state, "100.00").await;
    state.sla_engine.account_entry(entry.id, Some(user_id())).await.unwrap();
    state.sla_engine.post_entry(entry.id, Some(user_id())).await.unwrap();
    let result = state.sla_engine.reverse_entry(entry.id, "", Some(user_id())).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_cannot_reverse_draft_entry() {
    let state = setup_sla_test().await;
    let entry = create_balanced_entry(&state, "100.00").await;
    let result = state.sla_engine.reverse_entry(entry.id, "Should fail", Some(user_id())).await;
    assert!(result.is_err());
}

// ============================================================================
// Transfer to GL Tests
// ============================================================================

#[tokio::test]
async fn test_transfer_to_gl() {
    let state = setup_sla_test().await;
    let entry = create_balanced_entry(&state, "5000.00").await;
    state.sla_engine.account_entry(entry.id, Some(user_id())).await.unwrap();
    state.sla_engine.post_entry(entry.id, Some(user_id())).await.unwrap();
    let transfer = state.sla_engine.transfer_to_gl(
        org_id(), None, None, Some(user_id()),
    ).await.unwrap();
    assert_eq!(transfer.status, "completed");
    assert!(transfer.total_entries >= 1);
    assert!(transfer.transfer_number.starts_with("GLX-"));
    // Verify entry is now 'transferred'
    let updated = state.sla_engine.get_journal_entry(entry.id).await.unwrap().unwrap();
    assert_eq!(updated.status, "transferred");
}

#[tokio::test]
async fn test_transfer_with_no_posted_entries() {
    let state = setup_sla_test().await;
    let result = state.sla_engine.transfer_to_gl(org_id(), None, None, Some(user_id())).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_transfer_log() {
    let state = setup_sla_test().await;
    let entry = create_balanced_entry(&state, "1000.00").await;
    state.sla_engine.account_entry(entry.id, Some(user_id())).await.unwrap();
    state.sla_engine.post_entry(entry.id, Some(user_id())).await.unwrap();
    let transfer = state.sla_engine.transfer_to_gl(
        org_id(), None, None, Some(user_id()),
    ).await.unwrap();
    let log = state.sla_engine.get_transfer_log(transfer.id).await.unwrap().unwrap();
    assert_eq!(log.transfer_number, transfer.transfer_number);
    assert_eq!(log.status, "completed");
}

#[tokio::test]
async fn test_list_transfer_logs() {
    let state = setup_sla_test().await;
    let logs = state.sla_engine.list_transfer_logs(org_id(), None).await.unwrap();
    assert!(logs.is_empty() || logs.len() >= 0); // Just verify no error
}

// ============================================================================
// Auto-Accounting (Generate Lines) Tests
// ============================================================================

#[tokio::test]
async fn test_auto_generate_lines() {
    let state = setup_sla_test().await;
    let method = state.sla_engine.create_accounting_method(
        org_id(), "AUTO_GEN", "Auto Gen", None,
        "payables", "invoice", None,
        None, None, None, None, None, None, None, None, None, None,
    ).await.unwrap();
    // Create debit and credit rules
    state.sla_engine.create_derivation_rule(
        org_id(), method.id, "AUTO_DR", "Auto Debit", None,
        "debit", 10, serde_json::json!({}), None,
        "constant", Some("6100"), serde_json::json!({}), None,
        10, None, None, None,
    ).await.unwrap();
    state.sla_engine.create_derivation_rule(
        org_id(), method.id, "AUTO_CR", "Auto Credit", None,
        "credit", 10, serde_json::json!({}), None,
        "constant", Some("2100"), serde_json::json!({}), None,
        10, None, None, None,
    ).await.unwrap();
    // Create entry with the method
    let entry = state.sla_engine.create_journal_entry(
        org_id(), "payables", "invoice",
        Uuid::new_v4(), None, Some(method.id),
        None, None,
        NaiveDate::from_ymd_opt(2026, 4, 1).unwrap(), None,
        "USD", "USD", None, None, None, None,
    ).await.unwrap();
    let lines = state.sla_engine.generate_journal_lines(
        org_id(), entry.id, &serde_json::json!({"amount": 2500.0, "expense_category": "Travel"}),
    ).await.unwrap();
    assert_eq!(lines.len(), 2, "Expected 2 auto-generated lines (debit + credit)");
    // Verify entry is balanced
    let updated = state.sla_engine.get_journal_entry(entry.id).await.unwrap().unwrap();
    assert!(updated.is_balanced);
}

// ============================================================================
// SLA Events Tests
// ============================================================================

#[tokio::test]
async fn test_list_sla_events() {
    let state = setup_sla_test().await;
    let events = state.sla_engine.list_sla_events(org_id(), None, None).await.unwrap();
    // Just verify no error - events are created during reversals
    assert!(events.is_empty() || events.len() >= 0);
}

// ============================================================================
// Dashboard Test
// ============================================================================

#[tokio::test]
async fn test_sla_dashboard() {
    let state = setup_sla_test().await;
    state.sla_engine.create_journal_entry(
        org_id(), "payables", "invoice",
        Uuid::new_v4(), None, None, None, None,
        NaiveDate::from_ymd_opt(2026, 4, 1).unwrap(), None,
        "USD", "USD", None, None, None, None,
    ).await.unwrap();
    state.sla_engine.create_journal_entry(
        org_id(), "expenses", "report",
        Uuid::new_v4(), None, None, None, None,
        NaiveDate::from_ymd_opt(2026, 4, 1).unwrap(), None,
        "USD", "USD", None, None, None, None,
    ).await.unwrap();
    let dashboard = state.sla_engine.get_dashboard_summary(org_id()).await.unwrap();
    assert!(dashboard.total_entries >= 2);
    assert!(dashboard.draft_count >= 2);
    assert!(dashboard.unbalanced_count >= 0);
    assert!(dashboard.pending_transfer_count >= 0);
}

// ============================================================================
// Resolve Account Code Tests
// ============================================================================

#[tokio::test]
async fn test_resolve_account_code_constant() {
    let state = setup_sla_test().await;
    let method = state.sla_engine.create_accounting_method(
        org_id(), "RESOLVE", "Resolve", None,
        "expenses", "report", None,
        None, None, None, None, None, None, None, None, None, None,
    ).await.unwrap();
    state.sla_engine.create_derivation_rule(
        org_id(), method.id, "EXP_DR", "Expense Debit", None,
        "debit", 10, serde_json::json!({}), None,
        "constant", Some("6100"), serde_json::json!({}), None,
        10, None, None, None,
    ).await.unwrap();
    let rules = state.sla_engine.list_active_derivation_rules(org_id(), method.id, "debit").await.unwrap();
    let result = state.sla_engine.resolve_account_code(&rules, "debit", &serde_json::json!({"category": "Travel"}));
    assert_eq!(result, Some("6100".to_string()));
}

#[tokio::test]
async fn test_resolve_account_code_lookup() {
    let state = setup_sla_test().await;
    let method = state.sla_engine.create_accounting_method(
        org_id(), "RESOLVE_LK", "Resolve Lookup", None,
        "expenses", "report", None,
        None, None, None, None, None, None, None, None, None, None,
    ).await.unwrap();
    let lookup = serde_json::json!({"Travel": "6100", "Meals": "6200", "Office": "6300"});
    state.sla_engine.create_derivation_rule(
        org_id(), method.id, "EXP_LOOKUP", "Expense Lookup", None,
        "debit", 10, serde_json::json!({}), Some("expense_category"),
        "lookup", None, lookup, None,
        10, None, None, None,
    ).await.unwrap();
    let rules = state.sla_engine.list_active_derivation_rules(org_id(), method.id, "debit").await.unwrap();
    let result = state.sla_engine.resolve_account_code(&rules, "debit", &serde_json::json!({"expense_category": "Travel"}));
    assert_eq!(result, Some("6100".to_string()));
    let result = state.sla_engine.resolve_account_code(&rules, "debit", &serde_json::json!({"expense_category": "Meals"}));
    assert_eq!(result, Some("6200".to_string()));
    let result = state.sla_engine.resolve_account_code(&rules, "debit", &serde_json::json!({"expense_category": "Unknown"}));
    assert!(result.is_none());
}

#[tokio::test]
async fn test_resolve_wrong_line_type_returns_none() {
    let state = setup_sla_test().await;
    let method = state.sla_engine.create_accounting_method(
        org_id(), "RESOLVE_WT", "Resolve Wrong Type", None,
        "expenses", "report", None,
        None, None, None, None, None, None, None, None, None, None,
    ).await.unwrap();
    state.sla_engine.create_derivation_rule(
        org_id(), method.id, "EXP_DR_WT", "Expense Debit WT", None,
        "debit", 10, serde_json::json!({}), None,
        "constant", Some("6100"), serde_json::json!({}), None,
        10, None, None, None,
    ).await.unwrap();
    let rules = state.sla_engine.list_active_derivation_rules(org_id(), method.id, "debit").await.unwrap();
    let result = state.sla_engine.resolve_account_code(&rules, "credit", &serde_json::json!({}));
    assert!(result.is_none());
}
