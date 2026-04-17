//! Mock repositories for testing and development

use atlas_shared::{EntityDefinition, AuditEntry};
use atlas_shared::errors::AtlasResult;
use async_trait::async_trait;
use uuid::Uuid;
use crate::schema::SchemaRepository;
use crate::audit::AuditRepository;
use crate::tax::TaxRepository;
use crate::intercompany::IntercompanyRepository;
use crate::reconciliation::ReconciliationRepository;
use crate::expense::ExpenseRepository;
use crate::budget::BudgetRepository;
use crate::fixed_assets::FixedAssetRepository;

/// Mock schema repository
pub struct MockSchemaRepository;

#[async_trait]
impl SchemaRepository for MockSchemaRepository {
    async fn get_all_entities(&self) -> AtlasResult<Vec<EntityDefinition>> { Ok(vec![]) }
    async fn get_entity(&self, _name: &str) -> AtlasResult<Option<EntityDefinition>> { Ok(None) }
    async fn upsert_entity(&self, _entity: &EntityDefinition) -> AtlasResult<()> { Ok(()) }
    async fn delete_entity(&self, _name: &str) -> AtlasResult<()> { Ok(()) }
    async fn get_entity_version(&self, _name: &str) -> AtlasResult<Option<i64>> { Ok(Some(1)) }
    async fn set_entity_version(&self, _name: &str, _version: i64) -> AtlasResult<()> { Ok(()) }
}

/// Mock audit repository
pub struct MockAuditRepository;

#[async_trait]
impl AuditRepository for MockAuditRepository {
    async fn insert(&self, _entry: &AuditEntry) -> AtlasResult<()> { Ok(()) }
    async fn query(&self, _query: &crate::audit::AuditQuery) -> AtlasResult<Vec<AuditEntry>> { Ok(vec![]) }
    async fn get_by_id(&self, _id: Uuid) -> AtlasResult<Option<AuditEntry>> { Ok(None) }
    async fn get_by_ids(&self, _ids: &[Uuid]) -> AtlasResult<Vec<AuditEntry>> { Ok(vec![]) }
}

/// Mock intercompany repository for testing
pub struct MockIntercompanyRepository;

#[async_trait]
impl IntercompanyRepository for MockIntercompanyRepository {
    async fn create_batch(
        &self, _org_id: Uuid, _batch_number: &str, _description: Option<&str>,
        _from_entity_id: Uuid, _from_entity_name: &str,
        _to_entity_id: Uuid, _to_entity_name: &str,
        _currency_code: &str, _accounting_date: Option<chrono::NaiveDate>,
        _created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::IntercompanyBatch> {
        Ok(atlas_shared::IntercompanyBatch {
            id: Uuid::new_v4(), organization_id: _org_id,
            batch_number: _batch_number.to_string(), description: None,
            status: "draft".to_string(),
            from_entity_id: _from_entity_id, from_entity_name: _from_entity_name.to_string(),
            to_entity_id: _to_entity_id, to_entity_name: _to_entity_name.to_string(),
            currency_code: _currency_code.to_string(),
            total_amount: "0".to_string(), total_debit: "0".to_string(),
            total_credit: "0".to_string(), transaction_count: 0,
            from_journal_id: None, to_journal_id: None,
            accounting_date: None, posted_at: None, rejected_reason: None,
            metadata: serde_json::json!({}),
            created_by: None, approved_by: None,
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_batch(&self, _org_id: Uuid, _batch_number: &str) -> AtlasResult<Option<atlas_shared::IntercompanyBatch>> { Ok(None) }
    async fn get_batch_by_id(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::IntercompanyBatch>> { Ok(None) }
    async fn list_batches(&self, _org_id: Uuid, _status: Option<&str>) -> AtlasResult<Vec<atlas_shared::IntercompanyBatch>> { Ok(vec![]) }
    async fn update_batch_status(
        &self, _id: Uuid, _status: &str, _approved_by: Option<Uuid>,
        _posted_at: Option<chrono::DateTime<chrono::Utc>>, _rejected_reason: Option<&str>,
    ) -> AtlasResult<atlas_shared::IntercompanyBatch> {
        Err(atlas_shared::AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn update_batch_totals(
        &self, _id: Uuid, _total_amount: &str, _total_debit: &str,
        _total_credit: &str, _transaction_count: i32,
    ) -> AtlasResult<()> { Ok(()) }
    async fn create_transaction(
        &self, _org_id: Uuid, _batch_id: Uuid, _transaction_number: &str,
        _transaction_type: &str, _description: Option<&str>,
        _from_entity_id: Uuid, _from_entity_name: &str,
        _to_entity_id: Uuid, _to_entity_name: &str,
        _amount: &str, _currency_code: &str, _exchange_rate: Option<&str>,
        _from_debit_account: Option<&str>, _from_credit_account: Option<&str>,
        _to_debit_account: Option<&str>, _to_credit_account: Option<&str>,
        _from_ic_account: &str, _to_ic_account: &str,
        _transaction_date: chrono::NaiveDate, _due_date: Option<chrono::NaiveDate>,
        _source_entity_type: Option<&str>, _source_entity_id: Option<Uuid>,
        _created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::IntercompanyTransaction> {
        Ok(atlas_shared::IntercompanyTransaction {
            id: Uuid::new_v4(), organization_id: _org_id, batch_id: _batch_id,
            transaction_number: _transaction_number.to_string(),
            transaction_type: _transaction_type.to_string(), description: None,
            from_entity_id: _from_entity_id, from_entity_name: _from_entity_name.to_string(),
            to_entity_id: _to_entity_id, to_entity_name: _to_entity_name.to_string(),
            amount: _amount.to_string(), currency_code: _currency_code.to_string(),
            exchange_rate: None, from_debit_account: None, from_credit_account: None,
            to_debit_account: None, to_credit_account: None,
            from_ic_account: _from_ic_account.to_string(),
            to_ic_account: _to_ic_account.to_string(),
            status: "draft".to_string(),
            transaction_date: _transaction_date, due_date: None, settlement_date: None,
            source_entity_type: None, source_entity_id: None,
            metadata: serde_json::json!({}),
            created_by: None, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_transaction(&self, _org_id: Uuid, _transaction_number: &str) -> AtlasResult<Option<atlas_shared::IntercompanyTransaction>> { Ok(None) }
    async fn list_transactions_by_batch(&self, _batch_id: Uuid) -> AtlasResult<Vec<atlas_shared::IntercompanyTransaction>> { Ok(vec![]) }
    async fn list_transactions_by_entity(&self, _org_id: Uuid, _entity_id: Uuid, _status: Option<&str>) -> AtlasResult<Vec<atlas_shared::IntercompanyTransaction>> { Ok(vec![]) }
    async fn update_transaction_status(&self, _id: Uuid, _status: &str, _settlement_date: Option<chrono::NaiveDate>) -> AtlasResult<atlas_shared::IntercompanyTransaction> {
        Err(atlas_shared::AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn create_settlement(
        &self, _org_id: Uuid, _settlement_number: &str, _settlement_method: &str,
        _from_entity_id: Uuid, _to_entity_id: Uuid, _settled_amount: &str,
        _currency_code: &str, _payment_reference: Option<&str>,
        _transaction_ids: serde_json::Value, _created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::IntercompanySettlement> {
        Ok(atlas_shared::IntercompanySettlement {
            id: Uuid::new_v4(), organization_id: _org_id,
            settlement_number: _settlement_number.to_string(),
            settlement_method: _settlement_method.to_string(),
            from_entity_id: _from_entity_id, to_entity_id: _to_entity_id,
            settled_amount: _settled_amount.to_string(),
            currency_code: _currency_code.to_string(),
            payment_reference: None, status: "pending".to_string(),
            settlement_date: chrono::Utc::now().date_naive(),
            transaction_ids: _transaction_ids,
            metadata: serde_json::json!({}),
            created_by: None, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn list_settlements(&self, _org_id: Uuid, _entity_id: Option<Uuid>) -> AtlasResult<Vec<atlas_shared::IntercompanySettlement>> { Ok(vec![]) }
    async fn get_balance(&self, _org_id: Uuid, _from_entity_id: Uuid, _to_entity_id: Uuid, _currency_code: &str) -> AtlasResult<Option<atlas_shared::IntercompanyBalance>> { Ok(None) }
    async fn upsert_balance(
        &self, _org_id: Uuid, _from_entity_id: Uuid, _to_entity_id: Uuid,
        _currency_code: &str, _total_outstanding: &str, _total_posted: &str,
        _total_settled: &str, _open_transaction_count: i32,
    ) -> AtlasResult<atlas_shared::IntercompanyBalance> {
        Ok(atlas_shared::IntercompanyBalance {
            id: Uuid::new_v4(), organization_id: _org_id,
            from_entity_id: _from_entity_id, to_entity_id: _to_entity_id,
            currency_code: _currency_code.to_string(),
            total_outstanding: _total_outstanding.to_string(),
            total_posted: _total_posted.to_string(),
            total_settled: _total_settled.to_string(),
            open_transaction_count: _open_transaction_count,
            as_of_date: chrono::Utc::now().date_naive(),
            metadata: serde_json::json!({}),
            updated_at: chrono::Utc::now(),
        })
    }
    async fn list_balances(&self, _org_id: Uuid) -> AtlasResult<Vec<atlas_shared::IntercompanyBalance>> { Ok(vec![]) }
}

/// Mock tax repository for testing
pub struct MockTaxRepository;

#[async_trait]
impl TaxRepository for MockTaxRepository {
    async fn create_regime(
        &self, _org_id: Uuid, _code: &str, _name: &str, _description: Option<&str>,
        _tax_type: &str, _default_inclusive: bool, _allows_recovery: bool,
        _rounding_rule: &str, _rounding_precision: i32,
        _effective_from: Option<chrono::NaiveDate>, _effective_to: Option<chrono::NaiveDate>,
        _created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::TaxRegime> {
        Ok(atlas_shared::TaxRegime {
            id: Uuid::new_v4(), organization_id: _org_id, code: _code.to_string(),
            name: _name.to_string(), description: None, tax_type: _tax_type.to_string(),
            default_inclusive: _default_inclusive, allows_recovery: _allows_recovery,
            rounding_rule: _rounding_rule.to_string(), rounding_precision: _rounding_precision,
            is_active: true, effective_from: None, effective_to: None,
            created_by: None, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_regime(&self, _org_id: Uuid, _code: &str) -> AtlasResult<Option<atlas_shared::TaxRegime>> { Ok(None) }
    async fn get_regime_by_id(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::TaxRegime>> { Ok(None) }
    async fn list_regimes(&self, _org_id: Uuid) -> AtlasResult<Vec<atlas_shared::TaxRegime>> { Ok(vec![]) }
    async fn delete_regime(&self, _org_id: Uuid, _code: &str) -> AtlasResult<()> { Ok(()) }
    async fn create_jurisdiction(
        &self, _org_id: Uuid, _regime_id: Uuid, _code: &str, _name: &str,
        _geographic_level: &str, _country_code: Option<&str>, _state_code: Option<&str>,
        _county: Option<&str>, _city: Option<&str>, _postal_code_pattern: Option<&str>,
        _created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::TaxJurisdiction> {
        Ok(atlas_shared::TaxJurisdiction {
            id: Uuid::new_v4(), organization_id: _org_id, regime_id: _regime_id,
            code: _code.to_string(), name: _name.to_string(),
            geographic_level: _geographic_level.to_string(),
            country_code: None, state_code: None, county: None, city: None,
            postal_code_pattern: None, is_active: true, created_by: None,
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_jurisdiction(&self, _org_id: Uuid, _regime_id: Uuid, _code: &str) -> AtlasResult<Option<atlas_shared::TaxJurisdiction>> { Ok(None) }
    async fn list_jurisdictions(&self, _org_id: Uuid, _regime_id: Option<Uuid>) -> AtlasResult<Vec<atlas_shared::TaxJurisdiction>> { Ok(vec![]) }
    async fn delete_jurisdiction(&self, _org_id: Uuid, _regime_id: Uuid, _code: &str) -> AtlasResult<()> { Ok(()) }
    async fn create_tax_rate(
        &self, _org_id: Uuid, _regime_id: Uuid, _jurisdiction_id: Option<&Uuid>,
        _code: &str, _name: &str, _rate_percentage: &str, _rate_type: &str,
        _tax_account_code: Option<&str>, _recoverable: bool, _recovery_percentage: Option<&str>,
        _effective_from: chrono::NaiveDate, _effective_to: Option<chrono::NaiveDate>,
        _created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::TaxRate> {
        Ok(atlas_shared::TaxRate {
            id: Uuid::new_v4(), organization_id: _org_id, regime_id: _regime_id,
            jurisdiction_id: None, code: _code.to_string(), name: _name.to_string(),
            rate_percentage: _rate_percentage.to_string(), rate_type: _rate_type.to_string(),
            tax_account_code: None, recoverable: false, recovery_percentage: None,
            effective_from: _effective_from, effective_to: None, is_active: true,
            created_by: None, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_tax_rate(&self, _org_id: Uuid, _regime_id: Uuid, _code: &str) -> AtlasResult<Option<atlas_shared::TaxRate>> { Ok(None) }
    async fn get_tax_rate_by_id(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::TaxRate>> { Ok(None) }
    async fn get_tax_rate_by_code(&self, _org_id: Uuid, _code: &str) -> AtlasResult<Option<atlas_shared::TaxRate>> { Ok(None) }
    async fn get_effective_tax_rates(&self, _org_id: Uuid, _regime_id: Uuid, _on_date: chrono::NaiveDate) -> AtlasResult<Vec<atlas_shared::TaxRate>> { Ok(vec![]) }
    async fn list_tax_rates(&self, _org_id: Uuid, _regime_id: Uuid) -> AtlasResult<Vec<atlas_shared::TaxRate>> { Ok(vec![]) }
    async fn delete_tax_rate(&self, _org_id: Uuid, _regime_id: Uuid, _code: &str) -> AtlasResult<()> { Ok(()) }
    async fn create_determination_rule(
        &self, _org_id: Uuid, _regime_id: Uuid, _name: &str, _description: Option<&str>,
        _priority: i32, _condition: serde_json::Value, _action: serde_json::Value,
        _stop_on_match: bool, _effective_from: Option<chrono::NaiveDate>,
        _effective_to: Option<chrono::NaiveDate>, _created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::TaxDeterminationRule> { Ok(atlas_shared::TaxDeterminationRule {
        id: Uuid::new_v4(), organization_id: _org_id, regime_id: _regime_id,
        name: _name.to_string(), description: None, priority: _priority,
        condition: _condition, action: _action, stop_on_match: _stop_on_match,
        is_active: true, effective_from: None, effective_to: None,
        created_by: None, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
    }) }
    async fn list_determination_rules(&self, _org_id: Uuid, _regime_id: Uuid) -> AtlasResult<Vec<atlas_shared::TaxDeterminationRule>> { Ok(vec![]) }
    async fn create_tax_line(
        &self, _org_id: Uuid, _entity_type: &str, _entity_id: Uuid, _line_id: Option<Uuid>,
        _regime_id: Option<Uuid>, _jurisdiction_id: Option<Uuid>, _tax_rate_id: Uuid,
        _taxable_amount: &str, _tax_rate_percentage: &str, _tax_amount: &str,
        _is_inclusive: bool, _original_amount: Option<&str>,
        _recoverable_amount: Option<&str>, _non_recoverable_amount: Option<&str>,
        _tax_account_code: Option<&str>, _determination_rule_id: Option<Uuid>,
        _created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::TaxLine> { Ok(atlas_shared::TaxLine {
        id: Uuid::new_v4(), organization_id: _org_id,
        entity_type: _entity_type.to_string(), entity_id: _entity_id, line_id: None,
        regime_id: None, jurisdiction_id: None, tax_rate_id: _tax_rate_id,
        taxable_amount: _taxable_amount.to_string(),
        tax_rate_percentage: _tax_rate_percentage.to_string(),
        tax_amount: _tax_amount.to_string(),
        is_inclusive: false, original_amount: None,
        recoverable_amount: None, non_recoverable_amount: None,
        tax_account_code: None, determination_rule_id: None,
        created_by: None, created_at: chrono::Utc::now(),
    }) }
    async fn get_tax_lines(&self, _entity_type: &str, _entity_id: Uuid) -> AtlasResult<Vec<atlas_shared::TaxLine>> { Ok(vec![]) }
    async fn generate_tax_report(
        &self, _org_id: Uuid, _regime_id: Uuid, _jurisdiction_id: Option<&Uuid>,
        _period_start: chrono::NaiveDate, _period_end: chrono::NaiveDate,
        _created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::TaxReport> { Ok(atlas_shared::TaxReport {
        id: Uuid::new_v4(), organization_id: _org_id, regime_id: _regime_id,
        jurisdiction_id: None, period_start: _period_start, period_end: _period_end,
        total_taxable_amount: "0".to_string(), total_tax_amount: "0".to_string(),
        total_recoverable_amount: "0".to_string(), total_non_recoverable_amount: "0".to_string(),
        transaction_count: 0, status: "draft".to_string(),
        filed_by: None, filed_at: None, metadata: serde_json::json!({}),
        created_by: None, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
    }) }
    async fn list_tax_reports(&self, _org_id: Uuid, _regime_id: Option<Uuid>) -> AtlasResult<Vec<atlas_shared::TaxReport>> { Ok(vec![]) }
}

/// Mock reconciliation repository for testing
pub struct MockReconciliationRepository;

#[async_trait]
impl ReconciliationRepository for MockReconciliationRepository {
    async fn create_bank_account(
        &self, org_id: Uuid, account_number: &str, account_name: &str, bank_name: &str,
        bank_code: Option<&str>, branch_name: Option<&str>, branch_code: Option<&str>,
        gl_account_code: Option<&str>, currency_code: &str, account_type: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::BankAccount> {
        Ok(atlas_shared::BankAccount {
            id: Uuid::new_v4(), organization_id: org_id,
            account_number: account_number.to_string(), account_name: account_name.to_string(),
            bank_name: bank_name.to_string(), bank_code: bank_code.map(String::from),
            branch_name: branch_name.map(String::from), branch_code: branch_code.map(String::from),
            gl_account_code: gl_account_code.map(String::from),
            currency_code: currency_code.to_string(), account_type: account_type.to_string(),
            last_statement_balance: serde_json::json!(0),
            last_statement_date: None, is_active: true,
            created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            metadata: serde_json::json!({}),
        })
    }
    async fn get_bank_account(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::BankAccount>> { Ok(None) }
    async fn list_bank_accounts(&self, _org_id: Uuid) -> AtlasResult<Vec<atlas_shared::BankAccount>> { Ok(vec![]) }
    async fn delete_bank_account(&self, _id: Uuid) -> AtlasResult<()> { Ok(()) }

    async fn create_bank_statement(
        &self, org_id: Uuid, bank_account_id: Uuid, statement_number: &str,
        statement_date: chrono::NaiveDate, start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate, opening_balance: &str, closing_balance: &str,
        imported_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::BankStatement> {
        Ok(atlas_shared::BankStatement {
            id: Uuid::new_v4(), organization_id: org_id, bank_account_id,
            statement_number: statement_number.to_string(), statement_date,
            start_date, end_date,
            opening_balance: serde_json::json!(opening_balance),
            closing_balance: serde_json::json!(closing_balance),
            total_deposits: serde_json::json!(0), total_withdrawals: serde_json::json!(0),
            total_interest: serde_json::json!(0), total_charges: serde_json::json!(0),
            total_lines: 0, matched_lines: 0, unmatched_lines: 0,
            status: "imported".to_string(), reconciliation_percent: serde_json::json!(0),
            imported_by, reviewed_by: None, reconciled_by: None, reconciled_at: None,
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            metadata: serde_json::json!({}),
        })
    }
    async fn get_bank_statement(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::BankStatement>> { Ok(None) }
    async fn list_bank_statements(&self, _org_id: Uuid, _bank_account_id: Uuid) -> AtlasResult<Vec<atlas_shared::BankStatement>> { Ok(vec![]) }
    async fn update_statement_counts(&self, _id: Uuid, _total: i32, _matched: i32, _unmatched: i32, _pct: f64) -> AtlasResult<()> { Ok(()) }
    async fn update_statement_status(&self, _id: Uuid, _status: &str, _by: Option<Uuid>) -> AtlasResult<atlas_shared::BankStatement> {
        Err(atlas_shared::AtlasError::EntityNotFound("Mock".to_string()))
    }

    async fn create_statement_line(
        &self, org_id: Uuid, statement_id: Uuid, line_number: i32,
        transaction_date: chrono::NaiveDate, transaction_type: &str, amount: &str,
        description: Option<&str>, reference_number: Option<&str>, check_number: Option<&str>,
        counterparty_name: Option<&str>, counterparty_account: Option<&str>,
    ) -> AtlasResult<atlas_shared::BankStatementLine> {
        Ok(atlas_shared::BankStatementLine {
            id: Uuid::new_v4(), organization_id: org_id, statement_id, line_number,
            transaction_date, transaction_type: transaction_type.to_string(),
            amount: serde_json::json!(amount), description: description.map(String::from),
            reference_number: reference_number.map(String::from),
            check_number: check_number.map(String::from),
            counterparty_name: counterparty_name.map(String::from),
            counterparty_account: counterparty_account.map(String::from),
            match_status: "unmatched".to_string(),
            matched_by: None, matched_at: None, match_method: None,
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            metadata: serde_json::json!({}),
        })
    }
    async fn list_statement_lines(&self, _statement_id: Uuid) -> AtlasResult<Vec<atlas_shared::BankStatementLine>> { Ok(vec![]) }

    async fn create_system_transaction(
        &self, org_id: Uuid, bank_account_id: Uuid, source_type: &str,
        source_id: Uuid, source_number: Option<&str>, transaction_date: chrono::NaiveDate,
        amount: &str, transaction_type: &str, description: Option<&str>,
        reference_number: Option<&str>, check_number: Option<&str>,
        counterparty_name: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::SystemTransaction> {
        Ok(atlas_shared::SystemTransaction {
            id: Uuid::new_v4(), organization_id: org_id, bank_account_id,
            source_type: source_type.to_string(), source_id,
            source_number: source_number.map(String::from), transaction_date,
            amount: serde_json::json!(amount),
            transaction_type: transaction_type.to_string(),
            description: description.map(String::from),
            reference_number: reference_number.map(String::from),
            check_number: check_number.map(String::from),
            counterparty_name: counterparty_name.map(String::from),
            status: "unreconciled".to_string(), gl_posting_date: None,
            currency_code: "USD".to_string(), exchange_rate: None,
            created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            metadata: serde_json::json!({}),
        })
    }
    async fn get_system_transaction(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::SystemTransaction>> { Ok(None) }
    async fn list_unreconciled_transactions(&self, _org_id: Uuid, _bank_account_id: Uuid) -> AtlasResult<Vec<atlas_shared::SystemTransaction>> { Ok(vec![]) }

    async fn create_match(
        &self, org_id: Uuid, statement_id: Uuid, statement_line_id: Uuid,
        system_transaction_id: Uuid, match_method: &str, match_confidence: Option<f64>,
        matched_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::ReconciliationMatch> {
        Ok(atlas_shared::ReconciliationMatch {
            id: Uuid::new_v4(), organization_id: org_id, statement_id,
            statement_line_id, system_transaction_id,
            match_method: match_method.to_string(),
            match_confidence: match_confidence.map(|c| serde_json::json!(c)),
            matched_by, matched_at: Some(chrono::Utc::now()),
            unmatched_by: None, unmatched_at: None,
            status: "active".to_string(), notes: None,
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            metadata: serde_json::json!({}),
        })
    }
    async fn get_match(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::ReconciliationMatch>> { Ok(None) }
    async fn list_matches(&self, _statement_id: Uuid) -> AtlasResult<Vec<atlas_shared::ReconciliationMatch>> { Ok(vec![]) }
    async fn unmatch(&self, _id: Uuid, _unmatched_by: Option<Uuid>) -> AtlasResult<atlas_shared::ReconciliationMatch> {
        Err(atlas_shared::AtlasError::EntityNotFound("Mock".to_string()))
    }

    async fn get_or_create_summary(
        &self, org_id: Uuid, bank_account_id: Uuid,
        period_start: chrono::NaiveDate, period_end: chrono::NaiveDate,
    ) -> AtlasResult<atlas_shared::ReconciliationSummary> {
        Ok(atlas_shared::ReconciliationSummary {
            id: Uuid::new_v4(), organization_id: org_id, bank_account_id,
            period_start, period_end, statement_id: None,
            statement_balance: serde_json::json!(0), book_balance: serde_json::json!(0),
            deposits_in_transit: serde_json::json!(0), outstanding_checks: serde_json::json!(0),
            bank_charges: serde_json::json!(0), bank_interest: serde_json::json!(0),
            errors_and_omissions: serde_json::json!(0),
            adjusted_book_balance: serde_json::json!(0), adjusted_bank_balance: serde_json::json!(0),
            difference: serde_json::json!(0), is_balanced: false,
            status: "in_progress".to_string(),
            reviewed_by: None, reviewed_at: None, approved_by: None, approved_at: None,
            created_by: None, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            metadata: serde_json::json!({}),
        })
    }
    async fn list_summaries(&self, _org_id: Uuid) -> AtlasResult<Vec<atlas_shared::ReconciliationSummary>> { Ok(vec![]) }

    async fn create_matching_rule(
        &self, org_id: Uuid, name: &str, description: Option<&str>,
        bank_account_id: Option<Uuid>, priority: i32, criteria: serde_json::Value,
        stop_on_match: bool, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::ReconciliationMatchingRule> {
        Ok(atlas_shared::ReconciliationMatchingRule {
            id: Uuid::new_v4(), organization_id: org_id, bank_account_id,
            name: name.to_string(), description: description.map(String::from),
            priority, criteria, stop_on_match, is_active: true,
            created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn list_matching_rules(&self, _org_id: Uuid) -> AtlasResult<Vec<atlas_shared::ReconciliationMatchingRule>> { Ok(vec![]) }
    async fn delete_matching_rule(&self, _id: Uuid) -> AtlasResult<()> { Ok(()) }
}

/// Mock expense repository for testing
pub struct MockExpenseRepository;

#[async_trait]
impl ExpenseRepository for MockExpenseRepository {
    async fn create_category(
        &self, org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        receipt_required: bool, receipt_threshold: Option<&str>,
        is_per_diem: bool, default_per_diem_rate: Option<&str>,
        is_mileage: bool, default_mileage_rate: Option<&str>,
        expense_account_code: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::ExpenseCategory> {
        Ok(atlas_shared::ExpenseCategory {
            id: Uuid::new_v4(), organization_id: org_id,
            code: code.to_string(), name: name.to_string(), description: description.map(String::from),
            receipt_required, receipt_threshold: receipt_threshold.map(String::from),
            is_per_diem, default_per_diem_rate: default_per_diem_rate.map(String::from),
            is_mileage, default_mileage_rate: default_mileage_rate.map(String::from),
            expense_account_code: expense_account_code.map(String::from),
            is_active: true, created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_category(&self, _org_id: Uuid, _code: &str) -> AtlasResult<Option<atlas_shared::ExpenseCategory>> { Ok(None) }
    async fn get_category_by_id(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::ExpenseCategory>> { Ok(None) }
    async fn list_categories(&self, _org_id: Uuid) -> AtlasResult<Vec<atlas_shared::ExpenseCategory>> { Ok(vec![]) }
    async fn delete_category(&self, _org_id: Uuid, _code: &str) -> AtlasResult<()> { Ok(()) }

    async fn create_policy(
        &self, org_id: Uuid, name: &str, description: Option<&str>, category_id: Option<Uuid>,
        min_amount: Option<&str>, max_amount: Option<&str>, daily_limit: Option<&str>, report_limit: Option<&str>,
        requires_approval_on_violation: bool, violation_action: &str,
        effective_from: Option<chrono::NaiveDate>, effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::ExpensePolicy> {
        Ok(atlas_shared::ExpensePolicy {
            id: Uuid::new_v4(), organization_id: org_id, name: name.to_string(),
            description: description.map(String::from), category_id,
            min_amount: min_amount.map(String::from), max_amount: max_amount.map(String::from),
            daily_limit: daily_limit.map(String::from), report_limit: report_limit.map(String::from),
            requires_approval_on_violation, violation_action: violation_action.to_string(),
            is_active: true, effective_from, effective_to,
            created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_policy(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::ExpensePolicy>> { Ok(None) }
    async fn list_policies(&self, _org_id: Uuid, _category_id: Option<Uuid>) -> AtlasResult<Vec<atlas_shared::ExpensePolicy>> { Ok(vec![]) }
    async fn delete_policy(&self, _id: Uuid) -> AtlasResult<()> { Ok(()) }

    async fn create_report(
        &self, org_id: Uuid, report_number: &str, title: &str, description: Option<&str>,
        employee_id: Uuid, employee_name: Option<&str>, department_id: Option<Uuid>,
        purpose: Option<&str>, project_id: Option<Uuid>, currency_code: &str,
        trip_start_date: Option<chrono::NaiveDate>, trip_end_date: Option<chrono::NaiveDate>,
        cost_center: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::ExpenseReport> {
        Ok(atlas_shared::ExpenseReport {
            id: Uuid::new_v4(), organization_id: org_id,
            report_number: report_number.to_string(), title: title.to_string(),
            description: description.map(String::from), status: "draft".to_string(),
            employee_id, employee_name: employee_name.map(String::from),
            department_id, purpose: purpose.map(String::from), project_id,
            currency_code: currency_code.to_string(),
            total_amount: "0".to_string(), reimbursable_amount: "0".to_string(),
            receipt_required_amount: "0".to_string(), receipt_count: 0,
            trip_start_date, trip_end_date, cost_center: cost_center.map(String::from),
            approved_by: None, approved_at: None, rejection_reason: None,
            payment_method: None, payment_reference: None, reimbursed_at: None,
            metadata: serde_json::json!({}), created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_report(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::ExpenseReport>> { Ok(None) }
    async fn get_report_by_number(&self, _org_id: Uuid, _report_number: &str) -> AtlasResult<Option<atlas_shared::ExpenseReport>> { Ok(None) }
    async fn list_reports(&self, _org_id: Uuid, _employee_id: Option<Uuid>, _status: Option<&str>) -> AtlasResult<Vec<atlas_shared::ExpenseReport>> { Ok(vec![]) }
    async fn update_report_status(
        &self, _id: Uuid, _status: &str, _approved_by: Option<Uuid>, _rejection_reason: Option<&str>, _reimbursed_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> AtlasResult<atlas_shared::ExpenseReport> {
        Err(atlas_shared::AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn update_report_totals(&self, _id: Uuid, _total: &str, _reimbursable: &str, _receipt_required: &str, _count: i32) -> AtlasResult<()> { Ok(()) }

    async fn create_line(
        &self, org_id: Uuid, report_id: Uuid, line_number: i32,
        expense_category_id: Option<Uuid>, expense_category_name: Option<&str>, expense_type: &str,
        description: Option<&str>, expense_date: chrono::NaiveDate, amount: &str,
        original_currency: Option<&str>, original_amount: Option<&str>, exchange_rate: Option<&str>,
        is_reimbursable: bool, has_receipt: bool, receipt_reference: Option<&str>,
        merchant_name: Option<&str>, location: Option<&str>, attendees: Option<serde_json::Value>,
        per_diem_days: Option<f64>, per_diem_rate: Option<&str>,
        mileage_distance: Option<f64>, mileage_rate: Option<&str>, mileage_unit: Option<&str>,
        mileage_from: Option<&str>, mileage_to: Option<&str>,
        policy_violation: bool, policy_violation_message: Option<&str>,
        expense_account_code: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::ExpenseLine> {
        Ok(atlas_shared::ExpenseLine {
            id: Uuid::new_v4(), organization_id: org_id, report_id, line_number,
            expense_category_id, expense_category_name: expense_category_name.map(String::from),
            expense_type: expense_type.to_string(), description: description.map(String::from),
            expense_date, amount: amount.to_string(),
            original_currency: original_currency.map(String::from),
            original_amount: original_amount.map(String::from), exchange_rate: exchange_rate.map(String::from),
            is_reimbursable, has_receipt, receipt_reference: receipt_reference.map(String::from),
            merchant_name: merchant_name.map(String::from), location: location.map(String::from),
            attendees, per_diem_days, per_diem_rate: per_diem_rate.map(String::from),
            mileage_distance, mileage_rate: mileage_rate.map(String::from),
            mileage_unit: mileage_unit.map(String::from),
            mileage_from: mileage_from.map(String::from), mileage_to: mileage_to.map(String::from),
            policy_violation, policy_violation_message: policy_violation_message.map(String::from),
            expense_account_code: expense_account_code.map(String::from),
            metadata: serde_json::json!({}), created_by,
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_line(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::ExpenseLine>> { Ok(None) }
    async fn list_lines_by_report(&self, _report_id: Uuid) -> AtlasResult<Vec<atlas_shared::ExpenseLine>> { Ok(vec![]) }
    async fn delete_line(&self, _id: Uuid) -> AtlasResult<()> { Ok(()) }
}

/// Mock budget repository for testing
pub struct MockBudgetRepository;

#[async_trait]
impl BudgetRepository for MockBudgetRepository {
    async fn create_definition(
        &self, org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        calendar_id: Option<Uuid>, fiscal_year: Option<i32>, budget_type: &str,
        control_level: &str, allow_carry_forward: bool, allow_transfers: bool,
        currency_code: &str, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::BudgetDefinition> {
        Ok(atlas_shared::BudgetDefinition {
            id: Uuid::new_v4(), organization_id: org_id,
            code: code.to_string(), name: name.to_string(), description: description.map(String::from),
            calendar_id, fiscal_year, budget_type: budget_type.to_string(),
            control_level: control_level.to_string(),
            allow_carry_forward, allow_transfers,
            currency_code: currency_code.to_string(),
            is_active: true, metadata: serde_json::json!({}),
            created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_definition(&self, _org_id: Uuid, _code: &str) -> AtlasResult<Option<atlas_shared::BudgetDefinition>> { Ok(None) }
    async fn get_definition_by_id(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::BudgetDefinition>> { Ok(None) }
    async fn list_definitions(&self, _org_id: Uuid) -> AtlasResult<Vec<atlas_shared::BudgetDefinition>> { Ok(vec![]) }
    async fn delete_definition(&self, _org_id: Uuid, _code: &str) -> AtlasResult<()> { Ok(()) }

    async fn create_version(
        &self, org_id: Uuid, definition_id: Uuid, version_number: i32, label: Option<&str>,
        effective_from: Option<chrono::NaiveDate>, effective_to: Option<chrono::NaiveDate>,
        notes: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::BudgetVersion> {
        Ok(atlas_shared::BudgetVersion {
            id: Uuid::new_v4(), organization_id: org_id, definition_id,
            version_number, label: label.map(String::from), status: "draft".to_string(),
            total_budget_amount: "0".to_string(), total_committed_amount: "0".to_string(),
            total_actual_amount: "0".to_string(), total_variance_amount: "0".to_string(),
            submitted_by: None, submitted_at: None,
            approved_by: None, approved_at: None, rejected_reason: None,
            effective_from, effective_to, notes: notes.map(String::from),
            metadata: serde_json::json!({}), created_by,
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_version(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::BudgetVersion>> { Ok(None) }
    async fn get_active_version(&self, _definition_id: Uuid) -> AtlasResult<Option<atlas_shared::BudgetVersion>> { Ok(None) }
    async fn list_versions(&self, _definition_id: Uuid) -> AtlasResult<Vec<atlas_shared::BudgetVersion>> { Ok(vec![]) }
    async fn get_next_version_number(&self, _definition_id: Uuid) -> AtlasResult<i32> { Ok(1) }
    async fn update_version_status(
        &self, _id: Uuid, _status: &str, _submitted_by: Option<Uuid>,
        _approved_by: Option<Uuid>, _rejected_reason: Option<&str>,
    ) -> AtlasResult<atlas_shared::BudgetVersion> {
        Err(atlas_shared::AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn update_version_totals(
        &self, _id: Uuid, _total_budget: &str, _total_committed: &str,
        _total_actual: &str, _total_variance: &str,
    ) -> AtlasResult<()> { Ok(()) }

    async fn create_line(
        &self, org_id: Uuid, version_id: Uuid, line_number: i32,
        account_code: &str, account_name: Option<&str>,
        period_name: Option<&str>, period_start_date: Option<chrono::NaiveDate>,
        period_end_date: Option<chrono::NaiveDate>, fiscal_year: Option<i32>,
        quarter: Option<i32>, department_id: Option<Uuid>,
        department_name: Option<&str>, project_id: Option<Uuid>,
        project_name: Option<&str>, cost_center: Option<&str>,
        budget_amount: &str, description: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::BudgetLine> {
        Ok(atlas_shared::BudgetLine {
            id: Uuid::new_v4(), organization_id: org_id, version_id, line_number,
            account_code: account_code.to_string(), account_name: account_name.map(String::from),
            period_name: period_name.map(String::from), period_start_date, period_end_date,
            fiscal_year, quarter,
            department_id, department_name: department_name.map(String::from),
            project_id, project_name: project_name.map(String::from),
            cost_center: cost_center.map(String::from),
            budget_amount: budget_amount.to_string(),
            committed_amount: "0".to_string(), actual_amount: "0".to_string(),
            variance_amount: budget_amount.to_string(), variance_percent: "100".to_string(),
            carry_forward_amount: "0".to_string(),
            transferred_in_amount: "0".to_string(), transferred_out_amount: "0".to_string(),
            description: description.map(String::from),
            metadata: serde_json::json!({}), created_by,
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_line(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::BudgetLine>> { Ok(None) }
    async fn list_lines_by_version(&self, _version_id: Uuid) -> AtlasResult<Vec<atlas_shared::BudgetLine>> { Ok(vec![]) }
    async fn find_line(
        &self, _version_id: Uuid, _account_code: &str, _period_name: Option<&str>,
        _department_id: Option<&Uuid>, _cost_center: Option<&str>,
    ) -> AtlasResult<Option<atlas_shared::BudgetLine>> { Ok(None) }
    async fn update_line_amount(&self, _id: Uuid, _budget_amount: &str) -> AtlasResult<atlas_shared::BudgetLine> {
        Err(atlas_shared::AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn delete_line(&self, _id: Uuid) -> AtlasResult<()> { Ok(()) }

    async fn create_transfer(
        &self, org_id: Uuid, version_id: Uuid, transfer_number: &str,
        description: Option<&str>, from_account_code: &str, from_period_name: Option<&str>,
        from_department_id: Option<Uuid>, from_cost_center: Option<&str>,
        to_account_code: &str, to_period_name: Option<&str>,
        to_department_id: Option<Uuid>, to_cost_center: Option<&str>,
        amount: &str, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::BudgetTransfer> {
        Ok(atlas_shared::BudgetTransfer {
            id: Uuid::new_v4(), organization_id: org_id, version_id,
            transfer_number: transfer_number.to_string(), description: description.map(String::from),
            from_account_code: from_account_code.to_string(), from_period_name: from_period_name.map(String::from),
            from_department_id, from_cost_center: from_cost_center.map(String::from),
            to_account_code: to_account_code.to_string(), to_period_name: to_period_name.map(String::from),
            to_department_id, to_cost_center: to_cost_center.map(String::from),
            amount: amount.to_string(), status: "pending".to_string(),
            approved_by: None, approved_at: None, rejected_reason: None,
            metadata: serde_json::json!({}), created_by,
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_transfer(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::BudgetTransfer>> { Ok(None) }
    async fn list_transfers(&self, _version_id: Uuid) -> AtlasResult<Vec<atlas_shared::BudgetTransfer>> { Ok(vec![]) }
    async fn update_transfer_status(
        &self, _id: Uuid, _status: &str, _approved_by: Option<Uuid>, _rejected_reason: Option<&str>,
    ) -> AtlasResult<atlas_shared::BudgetTransfer> {
        Err(atlas_shared::AtlasError::EntityNotFound("Mock".to_string()))
    }
}

/// Mock fixed asset repository for testing
pub struct MockFixedAssetRepository;

#[async_trait]
impl FixedAssetRepository for MockFixedAssetRepository {
    async fn create_category(
        &self, org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        default_depreciation_method: &str, default_useful_life_months: i32,
        default_salvage_value_percent: &str,
        default_asset_account_code: Option<&str>, default_accum_depr_account_code: Option<&str>,
        default_depr_expense_account_code: Option<&str>, default_gain_loss_account_code: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::AssetCategory> {
        Ok(atlas_shared::AssetCategory {
            id: Uuid::new_v4(), organization_id: org_id,
            code: code.to_string(), name: name.to_string(), description: description.map(String::from),
            default_depreciation_method: default_depreciation_method.to_string(),
            default_useful_life_months,
            default_salvage_value_percent: default_salvage_value_percent.to_string(),
            default_asset_account_code: default_asset_account_code.map(String::from),
            default_accum_depr_account_code: default_accum_depr_account_code.map(String::from),
            default_depr_expense_account_code: default_depr_expense_account_code.map(String::from),
            default_gain_loss_account_code: default_gain_loss_account_code.map(String::from),
            is_active: true, created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_category(&self, _org_id: Uuid, _code: &str) -> AtlasResult<Option<atlas_shared::AssetCategory>> { Ok(None) }
    async fn get_category_by_id(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::AssetCategory>> { Ok(None) }
    async fn list_categories(&self, _org_id: Uuid) -> AtlasResult<Vec<atlas_shared::AssetCategory>> { Ok(vec![]) }
    async fn delete_category(&self, _org_id: Uuid, _code: &str) -> AtlasResult<()> { Ok(()) }

    async fn create_book(
        &self, org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        book_type: &str, auto_depreciation: bool, depreciation_calendar: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::AssetBook> {
        Ok(atlas_shared::AssetBook {
            id: Uuid::new_v4(), organization_id: org_id,
            code: code.to_string(), name: name.to_string(), description: description.map(String::from),
            book_type: book_type.to_string(), auto_depreciation, depreciation_calendar: depreciation_calendar.to_string(),
            current_fiscal_year: None, last_depreciation_date: None, is_active: true,
            metadata: serde_json::json!({}), created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_book(&self, _org_id: Uuid, _code: &str) -> AtlasResult<Option<atlas_shared::AssetBook>> { Ok(None) }
    async fn get_book_by_id(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::AssetBook>> { Ok(None) }
    async fn list_books(&self, _org_id: Uuid) -> AtlasResult<Vec<atlas_shared::AssetBook>> { Ok(vec![]) }
    async fn delete_book(&self, _org_id: Uuid, _code: &str) -> AtlasResult<()> { Ok(()) }

    async fn create_asset(
        &self, org_id: Uuid, asset_number: &str, asset_name: &str, description: Option<&str>,
        category_id: Option<Uuid>, category_code: Option<&str>,
        book_id: Option<Uuid>, book_code: Option<&str>,
        asset_type: &str, original_cost: &str, salvage_value: &str, salvage_value_percent: &str,
        depreciation_method: &str, useful_life_months: i32, declining_balance_rate: Option<&str>,
        acquisition_date: Option<chrono::NaiveDate>,
        location: Option<&str>, department_id: Option<Uuid>, department_name: Option<&str>,
        custodian_id: Option<Uuid>, custodian_name: Option<&str>,
        serial_number: Option<&str>, tag_number: Option<&str>, manufacturer: Option<&str>, model: Option<&str>,
        warranty_expiry: Option<chrono::NaiveDate>, insurance_policy_number: Option<&str>,
        insurance_expiry: Option<chrono::NaiveDate>, lease_number: Option<&str>, lease_expiry: Option<chrono::NaiveDate>,
        asset_account_code: Option<&str>, accum_depr_account_code: Option<&str>,
        depr_expense_account_code: Option<&str>, gain_loss_account_code: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::FixedAsset> {
        Ok(atlas_shared::FixedAsset {
            id: Uuid::new_v4(), organization_id: org_id,
            asset_number: asset_number.to_string(), asset_name: asset_name.to_string(),
            description: description.map(String::from),
            category_id, category_code: category_code.map(String::from),
            book_id, book_code: book_code.map(String::from),
            asset_type: asset_type.to_string(), status: "draft".to_string(),
            original_cost: original_cost.to_string(), current_cost: original_cost.to_string(),
            salvage_value: salvage_value.to_string(), salvage_value_percent: salvage_value_percent.to_string(),
            depreciation_method: depreciation_method.to_string(), useful_life_months,
            declining_balance_rate: declining_balance_rate.map(String::from),
            depreciable_basis: "0".to_string(), accumulated_depreciation: "0".to_string(),
            net_book_value: original_cost.to_string(), depreciation_per_period: "0".to_string(),
            periods_depreciated: 0, last_depreciation_date: None, last_depreciation_amount: "0".to_string(),
            acquisition_date, in_service_date: None, disposal_date: None, retirement_date: None,
            location: location.map(String::from),
            department_id, department_name: department_name.map(String::from),
            custodian_id, custodian_name: custodian_name.map(String::from),
            serial_number: serial_number.map(String::from), tag_number: tag_number.map(String::from),
            manufacturer: manufacturer.map(String::from), model: model.map(String::from),
            warranty_expiry, insurance_policy_number: insurance_policy_number.map(String::from),
            insurance_expiry, lease_number: lease_number.map(String::from), lease_expiry,
            asset_account_code: asset_account_code.map(String::from),
            accum_depr_account_code: accum_depr_account_code.map(String::from),
            depr_expense_account_code: depr_expense_account_code.map(String::from),
            gain_loss_account_code: gain_loss_account_code.map(String::from),
            metadata: serde_json::json!({}), created_by,
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_asset(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::FixedAsset>> { Ok(None) }
    async fn get_asset_by_number(&self, _org_id: Uuid, _asset_number: &str) -> AtlasResult<Option<atlas_shared::FixedAsset>> { Ok(None) }
    async fn list_assets(&self, _org_id: Uuid, _status: Option<&str>, _category_code: Option<&str>, _book_code: Option<&str>) -> AtlasResult<Vec<atlas_shared::FixedAsset>> { Ok(vec![]) }
    async fn update_asset_status(&self, _id: Uuid, _status: &str, _in_service_date: Option<chrono::NaiveDate>, _disposal_date: Option<chrono::NaiveDate>, _retirement_date: Option<chrono::NaiveDate>) -> AtlasResult<atlas_shared::FixedAsset> {
        Err(atlas_shared::AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn update_asset_depreciation(&self, _id: Uuid, _accumulated_depreciation: &str, _net_book_value: &str, _periods_depreciated: i32, _last_depreciation_date: Option<chrono::NaiveDate>, _last_depreciation_amount: &str) -> AtlasResult<atlas_shared::FixedAsset> {
        Err(atlas_shared::AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn update_asset_assignment(&self, _id: Uuid, _department_id: Option<Uuid>, _department_name: Option<&str>, _location: Option<&str>, _custodian_id: Option<Uuid>, _custodian_name: Option<&str>) -> AtlasResult<atlas_shared::FixedAsset> {
        Err(atlas_shared::AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn delete_asset(&self, _org_id: Uuid, _asset_number: &str) -> AtlasResult<()> { Ok(()) }

    async fn create_depreciation_entry(
        &self, org_id: Uuid, asset_id: Uuid, fiscal_year: i32, period_number: i32,
        period_name: Option<&str>, depreciation_date: chrono::NaiveDate,
        depreciation_amount: &str, accumulated_depreciation: &str, net_book_value: &str,
        depreciation_method: &str, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::AssetDepreciationHistory> {
        Ok(atlas_shared::AssetDepreciationHistory {
            id: Uuid::new_v4(), organization_id: org_id, asset_id,
            fiscal_year, period_number, period_name: period_name.map(String::from),
            depreciation_date, depreciation_amount: depreciation_amount.to_string(),
            accumulated_depreciation: accumulated_depreciation.to_string(),
            net_book_value: net_book_value.to_string(),
            depreciation_method: depreciation_method.to_string(),
            journal_entry_id: None, created_by, created_at: chrono::Utc::now(),
        })
    }
    async fn list_depreciation_history(&self, _asset_id: Uuid) -> AtlasResult<Vec<atlas_shared::AssetDepreciationHistory>> { Ok(vec![]) }
    async fn get_depreciation_for_period(&self, _asset_id: Uuid, _fiscal_year: i32, _period_number: i32) -> AtlasResult<Option<atlas_shared::AssetDepreciationHistory>> { Ok(None) }

    async fn create_transfer(
        &self, org_id: Uuid, transfer_number: &str, asset_id: Uuid,
        from_department_id: Option<Uuid>, from_department_name: Option<&str>,
        from_location: Option<&str>, from_custodian_id: Option<Uuid>, from_custodian_name: Option<&str>,
        to_department_id: Option<Uuid>, to_department_name: Option<&str>,
        to_location: Option<&str>, to_custodian_id: Option<Uuid>, to_custodian_name: Option<&str>,
        transfer_date: chrono::NaiveDate, reason: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::AssetTransfer> {
        Ok(atlas_shared::AssetTransfer {
            id: Uuid::new_v4(), organization_id: org_id,
            transfer_number: transfer_number.to_string(), asset_id,
            from_department_id, from_department_name: from_department_name.map(String::from),
            from_location: from_location.map(String::from),
            from_custodian_id, from_custodian_name: from_custodian_name.map(String::from),
            to_department_id, to_department_name: to_department_name.map(String::from),
            to_location: to_location.map(String::from),
            to_custodian_id, to_custodian_name: to_custodian_name.map(String::from),
            transfer_date, reason: reason.map(String::from),
            status: "pending".to_string(), approved_by: None, approved_at: None, rejected_reason: None,
            metadata: serde_json::json!({}), created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_transfer(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::AssetTransfer>> { Ok(None) }
    async fn list_transfers(&self, _org_id: Uuid, _asset_id: Option<Uuid>) -> AtlasResult<Vec<atlas_shared::AssetTransfer>> { Ok(vec![]) }
    async fn update_transfer_status(&self, _id: Uuid, _status: &str, _approved_by: Option<Uuid>, _rejected_reason: Option<&str>) -> AtlasResult<atlas_shared::AssetTransfer> {
        Err(atlas_shared::AtlasError::EntityNotFound("Mock".to_string()))
    }

    async fn create_retirement(
        &self, org_id: Uuid, retirement_number: &str, asset_id: Uuid,
        retirement_type: &str, retirement_date: chrono::NaiveDate,
        proceeds: &str, removal_cost: &str,
        net_book_value_at_retirement: &str, accumulated_depreciation_at_retirement: &str,
        gain_loss_amount: &str, gain_loss_type: Option<&str>,
        gain_account_code: Option<&str>, loss_account_code: Option<&str>,
        cash_account_code: Option<&str>, asset_account_code: Option<&str>,
        accum_depr_account_code: Option<&str>,
        reference_number: Option<&str>, buyer_name: Option<&str>, notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::AssetRetirement> {
        Ok(atlas_shared::AssetRetirement {
            id: Uuid::new_v4(), organization_id: org_id,
            retirement_number: retirement_number.to_string(), asset_id,
            retirement_type: retirement_type.to_string(), retirement_date,
            proceeds: proceeds.to_string(), removal_cost: removal_cost.to_string(),
            net_book_value_at_retirement: net_book_value_at_retirement.to_string(),
            accumulated_depreciation_at_retirement: accumulated_depreciation_at_retirement.to_string(),
            gain_loss_amount: gain_loss_amount.to_string(), gain_loss_type: gain_loss_type.map(String::from),
            gain_account_code: gain_account_code.map(String::from), loss_account_code: loss_account_code.map(String::from),
            cash_account_code: cash_account_code.map(String::from),
            asset_account_code: asset_account_code.map(String::from),
            accum_depr_account_code: accum_depr_account_code.map(String::from),
            reference_number: reference_number.map(String::from), buyer_name: buyer_name.map(String::from),
            notes: notes.map(String::from),
            status: "pending".to_string(), approved_by: None, approved_at: None, journal_entry_id: None,
            metadata: serde_json::json!({}), created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_retirement(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::AssetRetirement>> { Ok(None) }
    async fn list_retirements(&self, _org_id: Uuid, _asset_id: Option<Uuid>) -> AtlasResult<Vec<atlas_shared::AssetRetirement>> { Ok(vec![]) }
    async fn update_retirement_status(&self, _id: Uuid, _status: &str, _approved_by: Option<Uuid>) -> AtlasResult<atlas_shared::AssetRetirement> {
        Err(atlas_shared::AtlasError::EntityNotFound("Mock".to_string()))
    }
}
