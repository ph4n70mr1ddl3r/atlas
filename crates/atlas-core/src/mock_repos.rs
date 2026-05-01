//! Mock repositories for testing and development

use atlas_shared::{EntityDefinition, AuditEntry};
use atlas_shared::errors::{AtlasResult, AtlasError};
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
use crate::revenue::RevenueRepository;
use crate::segregation_of_duties::SegregationOfDutiesRepository;
use crate::project_costing::ProjectCostingRepository;
use crate::autoinvoice::AutoInvoiceRepository;
use crate::product_information::ProductInformationRepository;

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

/// Mock collections repository for testing
pub struct MockCollectionsRepository;

#[async_trait]
impl crate::collections::CollectionsRepository for MockCollectionsRepository {
    // Credit Profiles
    async fn create_credit_profile(
        &self, org_id: Uuid, customer_id: Uuid, customer_number: Option<&str>,
        customer_name: Option<&str>, credit_limit: &str, risk_classification: &str,
        credit_score: Option<i32>, external_credit_rating: Option<&str>,
        external_rating_agency: Option<&str>, external_rating_date: Option<chrono::NaiveDate>,
        payment_terms: &str, next_review_date: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::CustomerCreditProfile> {
        Ok(atlas_shared::CustomerCreditProfile {
            id: Uuid::new_v4(), organization_id: org_id, customer_id,
            customer_number: customer_number.map(String::from),
            customer_name: customer_name.map(String::from),
            credit_limit: credit_limit.to_string(), credit_used: "0".to_string(),
            credit_available: credit_limit.to_string(),
            risk_classification: risk_classification.to_string(),
            credit_score, external_credit_rating: external_credit_rating.map(String::from),
            external_rating_agency: external_rating_agency.map(String::from),
            external_rating_date, payment_terms: payment_terms.to_string(),
            average_days_to_pay: None, overdue_invoice_count: 0,
            total_overdue_amount: "0".to_string(), oldest_overdue_date: None,
            credit_hold: false, credit_hold_reason: None, credit_hold_date: None,
            credit_hold_by: None, last_review_date: None, next_review_date,
            status: "active".to_string(), metadata: serde_json::json!({}),
            created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_credit_profile(&self, _org_id: Uuid, _customer_id: Uuid) -> AtlasResult<Option<atlas_shared::CustomerCreditProfile>> { Ok(None) }
    async fn get_credit_profile_by_id(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::CustomerCreditProfile>> { Ok(None) }
    async fn list_credit_profiles(&self, _org_id: Uuid, _status: Option<&str>, _risk: Option<&str>) -> AtlasResult<Vec<atlas_shared::CustomerCreditProfile>> { Ok(vec![]) }
    async fn update_credit_profile(
        &self, _id: Uuid, _credit_limit: Option<&str>, _credit_used: Option<&str>,
        _risk: Option<&str>, _score: Option<i32>, _terms: Option<&str>,
        _avg_days: Option<&str>, _overdue_count: Option<i32>, _overdue_amount: Option<&str>,
        _oldest: Option<chrono::NaiveDate>, _hold: Option<bool>, _hold_reason: Option<&str>,
        _hold_date: Option<chrono::DateTime<chrono::Utc>>, _hold_by: Option<Uuid>,
        _last_review: Option<chrono::NaiveDate>, _next_review: Option<chrono::NaiveDate>,
        _status: Option<&str>,
    ) -> AtlasResult<atlas_shared::CustomerCreditProfile> {
        Err(atlas_shared::AtlasError::EntityNotFound("Mock".to_string()))
    }

    // Strategies
    async fn create_strategy(
        &self, org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        strategy_type: &str, applicable_risk: serde_json::Value, trigger_buckets: serde_json::Value,
        threshold: &str, actions: serde_json::Value, priority: i32, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::CollectionStrategy> {
        Ok(atlas_shared::CollectionStrategy {
            id: Uuid::new_v4(), organization_id: org_id, code: code.to_string(),
            name: name.to_string(), description: description.map(String::from),
            strategy_type: strategy_type.to_string(), applicable_risk_classifications: applicable_risk,
            trigger_aging_buckets: trigger_buckets, overdue_amount_threshold: threshold.to_string(),
            actions, priority, is_active: true, metadata: serde_json::json!({}),
            created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_strategy(&self, _org_id: Uuid, _code: &str) -> AtlasResult<Option<atlas_shared::CollectionStrategy>> { Ok(None) }
    async fn list_strategies(&self, _org_id: Uuid) -> AtlasResult<Vec<atlas_shared::CollectionStrategy>> { Ok(vec![]) }
    async fn delete_strategy(&self, _org_id: Uuid, _code: &str) -> AtlasResult<()> { Ok(()) }

    // Cases
    async fn create_case(
        &self, org_id: Uuid, case_number: &str, customer_id: Uuid,
        customer_number: Option<&str>, customer_name: Option<&str>,
        strategy_id: Option<Uuid>, assigned_to: Option<Uuid>, assigned_to_name: Option<&str>,
        case_type: &str, priority: &str, total_overdue: &str, total_disputed: &str,
        total_invoiced: &str, overdue_count: i32, oldest_overdue: Option<chrono::NaiveDate>,
        invoice_ids: serde_json::Value, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::CollectionCase> {
        Ok(atlas_shared::CollectionCase {
            id: Uuid::new_v4(), organization_id: org_id,
            case_number: case_number.to_string(), customer_id,
            customer_number: customer_number.map(String::from),
            customer_name: customer_name.map(String::from),
            strategy_id, assigned_to, assigned_to_name: assigned_to_name.map(String::from),
            case_type: case_type.to_string(), status: "open".to_string(),
            priority: priority.to_string(),
            total_overdue_amount: total_overdue.to_string(),
            total_disputed_amount: total_disputed.to_string(),
            total_invoiced_amount: total_invoiced.to_string(),
            overdue_invoice_count: overdue_count, oldest_overdue_date: oldest_overdue,
            current_step: 1, opened_date: chrono::Utc::now().date_naive(),
            target_resolution_date: None, resolved_date: None, closed_date: None,
            last_action_date: None, next_action_date: None,
            resolution_type: None, resolution_notes: None,
            related_invoice_ids: invoice_ids, metadata: serde_json::json!({}),
            created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_case(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::CollectionCase>> { Ok(None) }
    async fn get_case_by_number(&self, _org_id: Uuid, _case_number: &str) -> AtlasResult<Option<atlas_shared::CollectionCase>> { Ok(None) }
    async fn list_cases(&self, _org_id: Uuid, _status: Option<&str>, _customer_id: Option<Uuid>, _assigned_to: Option<Uuid>) -> AtlasResult<Vec<atlas_shared::CollectionCase>> { Ok(vec![]) }
    async fn update_case_status(
        &self, _id: Uuid, _status: &str, _step: Option<i32>, _assigned: Option<Uuid>,
        _assigned_name: Option<&str>, _last_action: Option<chrono::NaiveDate>,
        _next_action: Option<chrono::NaiveDate>, _resolution_type: Option<&str>,
        _resolution_notes: Option<&str>, _resolved: Option<chrono::NaiveDate>,
        _closed: Option<chrono::NaiveDate>,
    ) -> AtlasResult<atlas_shared::CollectionCase> {
        Err(atlas_shared::AtlasError::EntityNotFound("Mock".to_string()))
    }

    // Interactions
    async fn create_interaction(
        &self, org_id: Uuid, case_id: Option<Uuid>, customer_id: Uuid,
        customer_number: Option<&str>, customer_name: Option<&str>,
        interaction_type: &str, direction: &str, contact_name: Option<&str>,
        contact_role: Option<&str>, contact_phone: Option<&str>, contact_email: Option<&str>,
        subject: Option<&str>, body: Option<&str>, outcome: Option<&str>,
        follow_up_date: Option<chrono::NaiveDate>, follow_up_notes: Option<&str>,
        performed_by: Option<Uuid>, performed_by_name: Option<&str>, duration_minutes: Option<i32>,
    ) -> AtlasResult<atlas_shared::CustomerInteraction> {
        Ok(atlas_shared::CustomerInteraction {
            id: Uuid::new_v4(), organization_id: org_id, case_id, customer_id,
            customer_number: customer_number.map(String::from),
            customer_name: customer_name.map(String::from),
            interaction_type: interaction_type.to_string(), direction: direction.to_string(),
            contact_name: contact_name.map(String::from), contact_role: contact_role.map(String::from),
            contact_phone: contact_phone.map(String::from), contact_email: contact_email.map(String::from),
            subject: subject.map(String::from), body: body.map(String::from),
            outcome: outcome.map(String::from), follow_up_date, follow_up_notes: follow_up_notes.map(String::from),
            performed_by, performed_by_name: performed_by_name.map(String::from),
            performed_at: Some(chrono::Utc::now()), duration_minutes,
            metadata: serde_json::json!({}), created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_interaction(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::CustomerInteraction>> { Ok(None) }
    async fn list_interactions(&self, _org_id: Uuid, _case_id: Option<Uuid>, _customer_id: Option<Uuid>) -> AtlasResult<Vec<atlas_shared::CustomerInteraction>> { Ok(vec![]) }

    // Promises to Pay
    async fn create_promise_to_pay(
        &self, org_id: Uuid, case_id: Option<Uuid>, customer_id: Uuid,
        customer_number: Option<&str>, customer_name: Option<&str>,
        promise_type: &str, promised_amount: &str, promise_date: chrono::NaiveDate,
        installment_count: Option<i32>, installment_frequency: Option<&str>,
        invoice_ids: serde_json::Value, promised_by_name: Option<&str>,
        promised_by_role: Option<&str>, notes: Option<&str>, recorded_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::PromiseToPay> {
        Ok(atlas_shared::PromiseToPay {
            id: Uuid::new_v4(), organization_id: org_id, case_id, customer_id,
            customer_number: customer_number.map(String::from),
            customer_name: customer_name.map(String::from),
            promise_type: promise_type.to_string(),
            promised_amount: promised_amount.to_string(), paid_amount: "0".to_string(),
            remaining_amount: promised_amount.to_string(), promise_date,
            installment_count, installment_frequency: installment_frequency.map(String::from),
            status: "pending".to_string(), broken_date: None, broken_reason: None,
            related_invoice_ids: invoice_ids,
            promised_by_name: promised_by_name.map(String::from),
            promised_by_role: promised_by_role.map(String::from),
            notes: notes.map(String::from), recorded_by, recorded_at: Some(chrono::Utc::now()),
            metadata: serde_json::json!({}), created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_promise_to_pay(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::PromiseToPay>> { Ok(None) }
    async fn list_promises_to_pay(&self, _org_id: Uuid, _customer_id: Option<Uuid>, _status: Option<&str>) -> AtlasResult<Vec<atlas_shared::PromiseToPay>> { Ok(vec![]) }
    async fn update_promise_status(
        &self, _id: Uuid, _status: &str, _paid: Option<&str>, _remaining: Option<&str>,
        _broken_date: Option<chrono::NaiveDate>, _broken_reason: Option<&str>,
    ) -> AtlasResult<atlas_shared::PromiseToPay> {
        Err(atlas_shared::AtlasError::EntityNotFound("Mock".to_string()))
    }

    // Dunning Campaigns
    async fn create_dunning_campaign(
        &self, org_id: Uuid, campaign_number: &str, name: &str, description: Option<&str>,
        dunning_level: &str, comm_method: &str, template_id: Option<Uuid>, template_name: Option<&str>,
        min_days: i32, min_amount: &str, target_risk: serde_json::Value,
        exclude_active: bool, scheduled_date: Option<chrono::NaiveDate>, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::DunningCampaign> {
        Ok(atlas_shared::DunningCampaign {
            id: Uuid::new_v4(), organization_id: org_id,
            campaign_number: campaign_number.to_string(), name: name.to_string(),
            description: description.map(String::from), dunning_level: dunning_level.to_string(),
            communication_method: comm_method.to_string(), template_id, template_name: template_name.map(String::from),
            min_overdue_days: min_days, min_overdue_amount: min_amount.to_string(),
            target_risk_classifications: target_risk, exclude_active_cases: exclude_active,
            scheduled_date, sent_date: None, target_customer_count: 0, sent_count: 0,
            failed_count: 0, status: "draft".to_string(), metadata: serde_json::json!({}),
            created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_dunning_campaign(&self, _org_id: Uuid, _num: &str) -> AtlasResult<Option<atlas_shared::DunningCampaign>> { Ok(None) }
    async fn list_dunning_campaigns(&self, _org_id: Uuid, _status: Option<&str>) -> AtlasResult<Vec<atlas_shared::DunningCampaign>> { Ok(vec![]) }
    async fn update_dunning_campaign_status(&self, _id: Uuid, _status: &str, _sent: Option<chrono::NaiveDate>) -> AtlasResult<atlas_shared::DunningCampaign> {
        Err(atlas_shared::AtlasError::EntityNotFound("Mock".to_string()))
    }

    // Dunning Letters
    async fn create_dunning_letter(
        &self, _org_id: Uuid, _campaign_id: Option<Uuid>, _customer_id: Uuid,
        _customer_number: Option<&str>, _customer_name: Option<&str>, _level: &str,
        _method: &str, _overdue: &str, _count: i32, _oldest: Option<chrono::NaiveDate>,
        _current: &str, _a1: &str, _a2: &str, _a3: &str, _a4: &str, _a5: &str,
        _details: serde_json::Value, _created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::DunningLetter> {
        Err(atlas_shared::AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn list_dunning_letters(&self, _org_id: Uuid, _campaign_id: Option<Uuid>, _customer_id: Option<Uuid>) -> AtlasResult<Vec<atlas_shared::DunningLetter>> { Ok(vec![]) }
    async fn update_dunning_letter_status(&self, _id: Uuid, _status: &str) -> AtlasResult<atlas_shared::DunningLetter> {
        Err(atlas_shared::AtlasError::EntityNotFound("Mock".to_string()))
    }

    // Aging Snapshots
    async fn create_aging_snapshot(
        &self, _org_id: Uuid, _date: chrono::NaiveDate, _customer_id: Uuid,
        _customer_number: Option<&str>, _customer_name: Option<&str>,
        _total: &str, _current: &str, _a1: &str, _a2: &str, _a3: &str, _a4: &str, _a5: &str,
        _c0: i32, _c1: i32, _c2: i32, _c3: i32, _c4: i32, _c5: i32,
        _weighted: Option<&str>, _pct: Option<&str>,
    ) -> AtlasResult<atlas_shared::ReceivablesAgingSnapshot> {
        Err(atlas_shared::AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn list_aging_snapshots(&self, _org_id: Uuid, _date: chrono::NaiveDate) -> AtlasResult<Vec<atlas_shared::ReceivablesAgingSnapshot>> { Ok(vec![]) }

    // Write-Off Requests
    async fn create_write_off_request(
        &self, org_id: Uuid, request_number: &str, customer_id: Uuid,
        customer_number: Option<&str>, customer_name: Option<&str>,
        write_off_type: &str, write_off_amount: &str, account_code: Option<&str>,
        reason: &str, invoice_ids: serde_json::Value, case_id: Option<Uuid>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::WriteOffRequest> {
        Ok(atlas_shared::WriteOffRequest {
            id: Uuid::new_v4(), organization_id: org_id,
            request_number: request_number.to_string(), customer_id,
            customer_number: customer_number.map(String::from),
            customer_name: customer_name.map(String::from),
            write_off_type: write_off_type.to_string(), write_off_amount: write_off_amount.to_string(),
            write_off_account_code: account_code.map(String::from), reason: reason.to_string(),
            related_invoice_ids: invoice_ids, case_id, status: "draft".to_string(),
            submitted_by: None, submitted_at: None, approved_by: None, approved_at: None,
            rejected_reason: None, journal_entry_id: None, metadata: serde_json::json!({}),
            created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_write_off_request(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::WriteOffRequest>> { Ok(None) }
    async fn list_write_off_requests(&self, _org_id: Uuid, _status: Option<&str>) -> AtlasResult<Vec<atlas_shared::WriteOffRequest>> { Ok(vec![]) }
    async fn update_write_off_status(
        &self, _id: Uuid, _status: &str, _submitted_by: Option<Uuid>,
        _approved_by: Option<Uuid>, _rejected_reason: Option<&str>, _je: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::WriteOffRequest> {
        Err(atlas_shared::AtlasError::EntityNotFound("Mock".to_string()))
    }
}

/// Mock revenue recognition repository for testing
pub struct MockRevenueRepository;

#[async_trait]
impl RevenueRepository for MockRevenueRepository {
    // Policies
    async fn create_policy(
        &self, org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        recognition_method: &str, over_time_method: Option<&str>, allocation_basis: &str,
        default_selling_price: Option<&str>, constrain_variable_consideration: bool,
        constraint_threshold_percent: Option<&str>,
        revenue_account_code: Option<&str>, deferred_revenue_account_code: Option<&str>,
        contra_revenue_account_code: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::RevenuePolicy> {
        Ok(atlas_shared::RevenuePolicy {
            id: Uuid::new_v4(), organization_id: org_id,
            code: code.to_string(), name: name.to_string(),
            description: description.map(String::from),
            recognition_method: recognition_method.to_string(),
            over_time_method: over_time_method.map(String::from),
            allocation_basis: allocation_basis.to_string(),
            default_selling_price: default_selling_price.map(String::from),
            constrain_variable_consideration,
            constraint_threshold_percent: constraint_threshold_percent.map(String::from),
            revenue_account_code: revenue_account_code.map(String::from),
            deferred_revenue_account_code: deferred_revenue_account_code.map(String::from),
            contra_revenue_account_code: contra_revenue_account_code.map(String::from),
            is_active: true, metadata: serde_json::json!({}),
            created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_policy(&self, _org_id: Uuid, _code: &str) -> AtlasResult<Option<atlas_shared::RevenuePolicy>> { Ok(None) }
    async fn get_policy_by_id(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::RevenuePolicy>> { Ok(None) }
    async fn list_policies(&self, _org_id: Uuid) -> AtlasResult<Vec<atlas_shared::RevenuePolicy>> { Ok(vec![]) }
    async fn delete_policy(&self, _org_id: Uuid, _code: &str) -> AtlasResult<()> { Ok(()) }

    // Contracts
    async fn create_contract(
        &self, org_id: Uuid, contract_number: &str,
        source_type: Option<&str>, source_id: Option<Uuid>, source_number: Option<&str>,
        customer_id: Uuid, customer_number: Option<&str>, customer_name: Option<&str>,
        contract_date: Option<chrono::NaiveDate>, start_date: Option<chrono::NaiveDate>,
        end_date: Option<chrono::NaiveDate>, total_transaction_price: &str,
        currency_code: &str, notes: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::RevenueContract> {
        Ok(atlas_shared::RevenueContract {
            id: Uuid::new_v4(), organization_id: org_id,
            contract_number: contract_number.to_string(),
            source_type: source_type.map(String::from), source_id,
            source_number: source_number.map(String::from),
            customer_id, customer_number: customer_number.map(String::from),
            customer_name: customer_name.map(String::from),
            contract_date, start_date, end_date,
            total_transaction_price: total_transaction_price.to_string(),
            total_allocated_revenue: "0".to_string(),
            total_recognized_revenue: "0".to_string(),
            total_deferred_revenue: total_transaction_price.to_string(),
            status: "draft".to_string(),
            step1_contract_identified: false, step2_obligations_identified: false,
            step3_price_determined: false, step4_price_allocated: false,
            step5_recognition_scheduled: false,
            currency_code: currency_code.to_string(),
            notes: notes.map(String::from),
            metadata: serde_json::json!({}),
            created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_contract(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::RevenueContract>> { Ok(None) }
    async fn get_contract_by_number(&self, _org_id: Uuid, _num: &str) -> AtlasResult<Option<atlas_shared::RevenueContract>> { Ok(None) }
    async fn list_contracts(&self, _org_id: Uuid, _status: Option<&str>, _customer_id: Option<Uuid>) -> AtlasResult<Vec<atlas_shared::RevenueContract>> { Ok(vec![]) }
    async fn update_contract_status(
        &self, _id: Uuid, _status: Option<&str>,
        _step1: Option<bool>, _step2: Option<bool>, _step3: Option<bool>,
        _step4: Option<bool>, _step5: Option<bool>,
        _total_allocated: Option<&str>, _total_recognized: Option<&str>,
        _total_deferred: Option<&str>, _total_price: Option<&str>,
        _notes: Option<Option<&str>>,
    ) -> AtlasResult<atlas_shared::RevenueContract> {
        Err(atlas_shared::AtlasError::EntityNotFound("Mock".to_string()))
    }

    // Obligations
    async fn create_obligation(
        &self, org_id: Uuid, contract_id: Uuid, line_number: i32,
        description: Option<&str>, product_id: Option<Uuid>, product_name: Option<&str>,
        source_line_id: Option<Uuid>, revenue_policy_id: Option<Uuid>,
        recognition_method: Option<&str>, over_time_method: Option<&str>,
        standalone_selling_price: &str, allocated_transaction_price: &str,
        satisfaction_method: &str, recognition_start_date: Option<chrono::NaiveDate>,
        recognition_end_date: Option<chrono::NaiveDate>,
        revenue_account_code: Option<&str>, deferred_revenue_account_code: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::PerformanceObligation> {
        Ok(atlas_shared::PerformanceObligation {
            id: Uuid::new_v4(), organization_id: org_id, contract_id, line_number,
            description: description.map(String::from),
            product_id, product_name: product_name.map(String::from),
            source_line_id, revenue_policy_id,
            recognition_method: recognition_method.map(String::from),
            over_time_method: over_time_method.map(String::from),
            standalone_selling_price: standalone_selling_price.to_string(),
            allocated_transaction_price: allocated_transaction_price.to_string(),
            total_recognized_revenue: "0".to_string(),
            deferred_revenue: allocated_transaction_price.to_string(),
            recognition_start_date, recognition_end_date,
            percent_complete: None,
            satisfaction_method: satisfaction_method.to_string(),
            status: "pending".to_string(),
            revenue_account_code: revenue_account_code.map(String::from),
            deferred_revenue_account_code: deferred_revenue_account_code.map(String::from),
            metadata: serde_json::json!({}),
            created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_obligation(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::PerformanceObligation>> { Ok(None) }
    async fn list_obligations(&self, _contract_id: Uuid) -> AtlasResult<Vec<atlas_shared::PerformanceObligation>> { Ok(vec![]) }
    async fn update_obligation_allocation(
        &self, _id: Uuid, _allocated: &str, _deferred: &str,
    ) -> AtlasResult<atlas_shared::PerformanceObligation> {
        Err(atlas_shared::AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn update_obligation_status(
        &self, _id: Uuid, _status: &str, _start: Option<&str>, _end: Option<&str>,
    ) -> AtlasResult<atlas_shared::PerformanceObligation> {
        Err(atlas_shared::AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn update_obligation_recognition(
        &self, _id: Uuid, _recognized: &str, _deferred: &str, _pct: &str, _status: &str,
    ) -> AtlasResult<atlas_shared::PerformanceObligation> {
        Err(atlas_shared::AtlasError::EntityNotFound("Mock".to_string()))
    }

    // Schedule Lines
    async fn create_schedule_line(
        &self, org_id: Uuid, obligation_id: Uuid, contract_id: Uuid,
        line_number: i32, recognition_date: chrono::NaiveDate, amount: &str,
        percent_of_total: &str, recognition_method: &str, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::RevenueScheduleLine> {
        Ok(atlas_shared::RevenueScheduleLine {
            id: Uuid::new_v4(), organization_id: org_id,
            obligation_id, contract_id, line_number,
            recognition_date, amount: amount.to_string(),
            recognized_amount: "0".to_string(),
            status: "planned".to_string(),
            recognition_method: Some(recognition_method.to_string()),
            percent_of_total: Some(percent_of_total.to_string()),
            journal_entry_id: None, recognized_at: None,
            reversed_by_id: None, reversal_reason: None,
            metadata: serde_json::json!({}),
            created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_schedule_line(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::RevenueScheduleLine>> { Ok(None) }
    async fn list_schedule_lines(&self, _obligation_id: Uuid) -> AtlasResult<Vec<atlas_shared::RevenueScheduleLine>> { Ok(vec![]) }
    async fn list_schedule_lines_by_contract(&self, _contract_id: Uuid) -> AtlasResult<Vec<atlas_shared::RevenueScheduleLine>> { Ok(vec![]) }
    async fn update_schedule_line_status(
        &self, _id: Uuid, _status: &str, _recognized: Option<&str>, _reversal: Option<&str>,
    ) -> AtlasResult<atlas_shared::RevenueScheduleLine> {
        Err(atlas_shared::AtlasError::EntityNotFound("Mock".to_string()))
    }

    // Modifications
    async fn create_modification(
        &self, org_id: Uuid, contract_id: Uuid, modification_number: i32,
        modification_type: &str, description: Option<&str>,
        previous_transaction_price: &str, new_transaction_price: &str,
        previous_end_date: Option<chrono::NaiveDate>, new_end_date: Option<chrono::NaiveDate>,
        effective_date: chrono::NaiveDate, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::RevenueModification> {
        Ok(atlas_shared::RevenueModification {
            id: Uuid::new_v4(), organization_id: org_id, contract_id,
            modification_number, modification_type: modification_type.to_string(),
            description: description.map(String::from),
            previous_transaction_price: previous_transaction_price.to_string(),
            new_transaction_price: new_transaction_price.to_string(),
            previous_end_date, new_end_date, effective_date,
            status: "draft".to_string(), metadata: serde_json::json!({}),
            created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn list_modifications(&self, _contract_id: Uuid) -> AtlasResult<Vec<atlas_shared::RevenueModification>> { Ok(vec![]) }
}

/// Mock payment repository for testing
pub struct MockPaymentRepository;

#[async_trait]
impl crate::payment::PaymentRepository for MockPaymentRepository {
    // Payment Terms
    async fn create_payment_term(
        &self, org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        due_days: i32, discount_days: Option<i32>, discount_percentage: Option<&str>,
        is_installment: bool, installment_count: Option<i32>, installment_frequency: Option<&str>,
        default_payment_method: Option<&str>, effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::PaymentTerm> {
        Ok(atlas_shared::PaymentTerm {
            id: Uuid::new_v4(), organization_id: org_id,
            code: code.to_string(), name: name.to_string(), description: description.map(String::from),
            due_days, discount_days, discount_percentage: discount_percentage.map(String::from),
            is_installment, installment_count, installment_frequency: installment_frequency.map(String::from),
            default_payment_method: default_payment_method.map(String::from),
            effective_from, effective_to, is_active: true,
            metadata: serde_json::json!({}), created_by,
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_payment_term(&self, _org_id: Uuid, _code: &str) -> AtlasResult<Option<atlas_shared::PaymentTerm>> { Ok(None) }
    async fn get_payment_term_by_id(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::PaymentTerm>> { Ok(None) }
    async fn list_payment_terms(&self, _org_id: Uuid) -> AtlasResult<Vec<atlas_shared::PaymentTerm>> { Ok(vec![]) }
    async fn delete_payment_term(&self, _org_id: Uuid, _code: &str) -> AtlasResult<()> { Ok(()) }

    // Payment Batches
    async fn create_payment_batch(
        &self, org_id: Uuid, batch_number: &str, name: Option<&str>, description: Option<&str>,
        payment_date: chrono::NaiveDate, bank_account_id: Option<Uuid>, payment_method: &str,
        currency_code: &str, selection_criteria: serde_json::Value, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::PaymentBatch> {
        Ok(atlas_shared::PaymentBatch {
            id: Uuid::new_v4(), organization_id: org_id,
            batch_number: batch_number.to_string(), name: name.map(String::from),
            description: description.map(String::from),
            payment_date, bank_account_id, payment_method: payment_method.to_string(),
            currency_code: currency_code.to_string(), selection_criteria,
            total_invoice_count: 0, total_payment_count: 0,
            total_payment_amount: "0".to_string(), total_discount_taken: "0".to_string(),
            status: "draft".to_string(),
            selected_by: None, selected_at: None, approved_by: None, approved_at: None,
            formatted_by: None, formatted_at: None, confirmed_by: None, confirmed_at: None,
            cancelled_by: None, cancelled_at: None, cancellation_reason: None,
            payment_file_name: None, payment_file_reference: None,
            metadata: serde_json::json!({}), created_by,
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_payment_batch(&self, _org_id: Uuid, _batch_number: &str) -> AtlasResult<Option<atlas_shared::PaymentBatch>> { Ok(None) }
    async fn get_payment_batch_by_id(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::PaymentBatch>> { Ok(None) }
    async fn list_payment_batches(&self, _org_id: Uuid, _status: Option<&str>) -> AtlasResult<Vec<atlas_shared::PaymentBatch>> { Ok(vec![]) }
    async fn update_payment_batch_status(
        &self, _id: Uuid, _status: &str, _action_by: Option<Uuid>, _cancellation_reason: Option<&str>,
    ) -> AtlasResult<atlas_shared::PaymentBatch> {
        Err(atlas_shared::AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn update_payment_batch_totals(
        &self, _id: Uuid, _invoice_count: i32, _payment_count: i32, _payment_amount: &str, _discount_taken: &str,
    ) -> AtlasResult<()> { Ok(()) }

    // Payments
    async fn create_payment(
        &self, org_id: Uuid, payment_number: &str, batch_id: Option<Uuid>,
        supplier_id: Uuid, supplier_number: Option<&str>, supplier_name: Option<&str>,
        supplier_site: Option<&str>, payment_date: chrono::NaiveDate, payment_method: &str,
        currency_code: &str, payment_amount: &str, discount_taken: &str,
        bank_account_id: Option<Uuid>, bank_account_name: Option<&str>,
        cash_account_code: Option<&str>, ap_account_code: Option<&str>,
        discount_account_code: Option<&str>, check_number: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::Payment> {
        Ok(atlas_shared::Payment {
            id: Uuid::new_v4(), organization_id: org_id,
            payment_number: payment_number.to_string(), batch_id,
            supplier_id, supplier_number: supplier_number.map(String::from),
            supplier_name: supplier_name.map(String::from), supplier_site: supplier_site.map(String::from),
            payment_date, payment_method: payment_method.to_string(),
            currency_code: currency_code.to_string(),
            payment_amount: payment_amount.to_string(), discount_taken: discount_taken.to_string(),
            bank_charges: "0".to_string(),
            bank_account_id, bank_account_name: bank_account_name.map(String::from),
            cash_account_code: cash_account_code.map(String::from),
            ap_account_code: ap_account_code.map(String::from),
            discount_account_code: discount_account_code.map(String::from),
            status: "draft".to_string(), check_number: check_number.map(String::from),
            reference_number: None,
            voided_by: None, voided_at: None, void_reason: None,
            reissued_from_payment_id: None, reissued_payment_id: None,
            cleared_date: None, cleared_by: None, cleared_at: None,
            journal_entry_id: None, posted_at: None,
            remittance_sent: false, remittance_sent_at: None, remittance_method: None,
            metadata: serde_json::json!({}), created_by,
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_payment(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::Payment>> { Ok(None) }
    async fn get_payment_by_number(&self, _org_id: Uuid, _payment_number: &str) -> AtlasResult<Option<atlas_shared::Payment>> { Ok(None) }
    async fn list_payments(&self, _org_id: Uuid, _status: Option<&str>, _supplier_id: Option<Uuid>, _batch_id: Option<Uuid>) -> AtlasResult<Vec<atlas_shared::Payment>> { Ok(vec![]) }
    async fn update_payment_status(
        &self, _id: Uuid, _status: &str, _cleared_date: Option<chrono::NaiveDate>,
        _cleared_by: Option<Uuid>, _void_reason: Option<&str>, _voided_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::Payment> {
        Err(atlas_shared::AtlasError::EntityNotFound("Mock".to_string()))
    }

    // Payment Lines
    async fn create_payment_line(
        &self, org_id: Uuid, payment_id: Uuid, line_number: i32,
        invoice_id: Uuid, invoice_number: Option<&str>, invoice_date: Option<chrono::NaiveDate>,
        invoice_due_date: Option<chrono::NaiveDate>, invoice_amount: Option<&str>,
        amount_paid: &str, discount_taken: &str, withholding_amount: &str,
    ) -> AtlasResult<atlas_shared::PaymentLine> {
        Ok(atlas_shared::PaymentLine {
            id: Uuid::new_v4(), organization_id: org_id, payment_id, line_number,
            invoice_id, invoice_number: invoice_number.map(String::from),
            invoice_date, invoice_due_date,
            invoice_amount: invoice_amount.map(String::from),
            amount_paid: amount_paid.to_string(), discount_taken: discount_taken.to_string(),
            withholding_amount: withholding_amount.to_string(),
            metadata: serde_json::json!({}),
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn list_payment_lines(&self, _payment_id: Uuid) -> AtlasResult<Vec<atlas_shared::PaymentLine>> { Ok(vec![]) }

    // Scheduled Payments
    async fn create_scheduled_payment(
        &self, org_id: Uuid, invoice_id: Uuid, invoice_number: Option<&str>,
        supplier_id: Uuid, supplier_name: Option<&str>,
        scheduled_payment_date: chrono::NaiveDate, scheduled_amount: &str,
        installment_number: i32, payment_method: Option<&str>,
        bank_account_id: Option<Uuid>, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::ScheduledPayment> {
        Ok(atlas_shared::ScheduledPayment {
            id: Uuid::new_v4(), organization_id: org_id,
            invoice_id, invoice_number: invoice_number.map(String::from),
            supplier_id, supplier_name: supplier_name.map(String::from),
            scheduled_payment_date, scheduled_amount: scheduled_amount.to_string(),
            installment_number, payment_method: payment_method.map(String::from),
            bank_account_id, is_selected: false, selected_batch_id: None, payment_id: None,
            status: "pending".to_string(), metadata: serde_json::json!({}), created_by,
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn list_scheduled_payments(&self, _org_id: Uuid, _status: Option<&str>, _supplier_id: Option<Uuid>) -> AtlasResult<Vec<atlas_shared::ScheduledPayment>> { Ok(vec![]) }
    async fn update_scheduled_payment_status(
        &self, _id: Uuid, _status: &str, _selected_batch_id: Option<Uuid>, _payment_id: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::ScheduledPayment> {
        Err(atlas_shared::AtlasError::EntityNotFound("Mock".to_string()))
    }

    // Payment Formats
    async fn create_payment_format(
        &self, org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        format_type: &str, template_reference: Option<&str>,
        applicable_methods: serde_json::Value, is_system: bool,
    ) -> AtlasResult<atlas_shared::PaymentFormat> {
        Ok(atlas_shared::PaymentFormat {
            id: Uuid::new_v4(), organization_id: org_id,
            code: code.to_string(), name: name.to_string(), description: description.map(String::from),
            format_type: format_type.to_string(), template_reference: template_reference.map(String::from),
            applicable_methods, is_system, is_active: true,
            metadata: serde_json::json!({}), created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn list_payment_formats(&self, _org_id: Uuid) -> AtlasResult<Vec<atlas_shared::PaymentFormat>> { Ok(vec![]) }
    async fn delete_payment_format(&self, _org_id: Uuid, _code: &str) -> AtlasResult<()> { Ok(()) }

    // Remittance Advice
    async fn create_remittance_advice(
        &self, org_id: Uuid, payment_id: Uuid, delivery_method: &str,
        delivery_address: Option<&str>, contact_name: Option<&str>,
        contact_email: Option<&str>, subject: Option<&str>, body: Option<&str>,
        payment_summary: serde_json::Value, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::RemittanceAdvice> {
        Ok(atlas_shared::RemittanceAdvice {
            id: Uuid::new_v4(), organization_id: org_id, payment_id,
            delivery_method: delivery_method.to_string(), delivery_address: delivery_address.map(String::from),
            contact_name: contact_name.map(String::from), contact_email: contact_email.map(String::from),
            subject: subject.map(String::from), body: body.map(String::from),
            status: "pending".to_string(), sent_at: None, delivered_at: None, failure_reason: None,
            payment_summary, metadata: serde_json::json!({}), created_by,
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn list_remittance_advices(&self, _org_id: Uuid, _payment_id: Option<Uuid>) -> AtlasResult<Vec<atlas_shared::RemittanceAdvice>> { Ok(vec![]) }
    async fn update_remittance_advice_status(&self, _id: Uuid, _status: &str, _failure_reason: Option<&str>) -> AtlasResult<atlas_shared::RemittanceAdvice> {
        Err(atlas_shared::AtlasError::EntityNotFound("Mock".to_string()))
    }
}

// ============================================================================
// Mock Subledger Accounting Repository
// ============================================================================

pub struct MockSubledgerAccountingRepository;

#[async_trait::async_trait]
impl crate::subledger_accounting::SubledgerAccountingRepository for MockSubledgerAccountingRepository {
    // Accounting Methods
    async fn create_accounting_method(
        &self, org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        application: &str, transaction_type: &str, event_class: &str,
        auto_accounting: bool, allow_manual_entries: bool, apply_rounding: bool,
        rounding_account_code: Option<&str>, rounding_threshold: &str,
        require_balancing: bool, intercompany_balancing_account: Option<&str>,
        effective_from: Option<chrono::NaiveDate>, effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::AccountingMethod> {
        Ok(atlas_shared::AccountingMethod {
            id: Uuid::new_v4(), organization_id: org_id,
            code: code.to_string(), name: name.to_string(), description: description.map(String::from),
            application: application.to_string(), transaction_type: transaction_type.to_string(),
            event_class: event_class.to_string(),
            auto_accounting, allow_manual_entries, apply_rounding,
            rounding_account_code: rounding_account_code.map(String::from),
            rounding_threshold: rounding_threshold.to_string(),
            require_balancing, intercompany_balancing_account: intercompany_balancing_account.map(String::from),
            effective_from, effective_to, is_active: true,
            metadata: serde_json::json!({}), created_by,
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_accounting_method(&self, _org_id: Uuid, _code: &str) -> AtlasResult<Option<atlas_shared::AccountingMethod>> { Ok(None) }
    async fn get_accounting_method_by_id(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::AccountingMethod>> { Ok(None) }
    async fn list_accounting_methods(&self, _org_id: Uuid, _application: Option<&str>) -> AtlasResult<Vec<atlas_shared::AccountingMethod>> { Ok(vec![]) }
    async fn delete_accounting_method(&self, _org_id: Uuid, _code: &str) -> AtlasResult<()> { Ok(()) }

    // Derivation Rules
    async fn create_derivation_rule(
        &self, org_id: Uuid, accounting_method_id: Uuid, code: &str, name: &str,
        description: Option<&str>, line_type: &str, priority: i32,
        conditions: serde_json::Value, source_field: Option<&str>,
        derivation_type: &str, fixed_account_code: Option<&str>,
        account_derivation_lookup: serde_json::Value,
        formula_expression: Option<&str>, sequence: i32,
        effective_from: Option<chrono::NaiveDate>, effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::AccountingDerivationRule> {
        Ok(atlas_shared::AccountingDerivationRule {
            id: Uuid::new_v4(), organization_id: org_id, accounting_method_id,
            code: code.to_string(), name: name.to_string(), description: description.map(String::from),
            line_type: line_type.to_string(), priority, conditions, source_field: source_field.map(String::from),
            derivation_type: derivation_type.to_string(), fixed_account_code: fixed_account_code.map(String::from),
            account_derivation_lookup, formula_expression: formula_expression.map(String::from),
            sequence, is_active: true, effective_from, effective_to,
            metadata: serde_json::json!({}), created_by,
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_derivation_rule(&self, _org_id: Uuid, _method_id: Uuid, _code: &str) -> AtlasResult<Option<atlas_shared::AccountingDerivationRule>> { Ok(None) }
    async fn list_derivation_rules(&self, _org_id: Uuid, _method_id: Uuid) -> AtlasResult<Vec<atlas_shared::AccountingDerivationRule>> { Ok(vec![]) }
    async fn list_active_derivation_rules(&self, _org_id: Uuid, _method_id: Uuid, _line_type: &str) -> AtlasResult<Vec<atlas_shared::AccountingDerivationRule>> { Ok(vec![]) }
    async fn delete_derivation_rule(&self, _org_id: Uuid, _method_id: Uuid, _code: &str) -> AtlasResult<()> { Ok(()) }

    // Journal Entries
    async fn create_journal_entry(
        &self, org_id: Uuid, source_application: &str, source_transaction_type: &str,
        source_transaction_id: Uuid, source_transaction_number: Option<&str>,
        accounting_method_id: Option<Uuid>, entry_number: &str, description: Option<&str>,
        reference_number: Option<&str>, accounting_date: chrono::NaiveDate, period_name: Option<&str>,
        currency_code: &str, entered_currency_code: &str,
        currency_conversion_date: Option<chrono::NaiveDate>, currency_conversion_type: Option<&str>,
        currency_conversion_rate: Option<&str>,
        total_debit: &str, total_credit: &str, entered_debit: &str, entered_credit: &str,
        status: &str, balancing_segment: Option<&str>, is_balanced: bool,
        created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::SubledgerJournalEntry> {
        Ok(atlas_shared::SubledgerJournalEntry {
            id: Uuid::new_v4(), organization_id: org_id,
            source_application: source_application.to_string(),
            source_transaction_type: source_transaction_type.to_string(),
            source_transaction_id, source_transaction_number: source_transaction_number.map(String::from),
            accounting_method_id,
            entry_number: entry_number.to_string(), description: description.map(String::from),
            reference_number: reference_number.map(String::from),
            accounting_date, period_name: period_name.map(String::from),
            currency_code: currency_code.to_string(), entered_currency_code: entered_currency_code.to_string(),
            currency_conversion_date, currency_conversion_type: currency_conversion_type.map(String::from),
            currency_conversion_rate: currency_conversion_rate.map(String::from),
            total_debit: total_debit.to_string(), total_credit: total_credit.to_string(),
            entered_debit: entered_debit.to_string(), entered_credit: entered_credit.to_string(),
            status: status.to_string(), error_message: None,
            balancing_segment: balancing_segment.map(String::from), is_balanced,
            gl_transfer_status: "pending".to_string(), gl_transfer_date: None, gl_journal_entry_id: None,
            is_reversal: false, reversal_of_id: None, reversal_reason: None,
            created_by, posted_by: None, accounted_by: None,
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_journal_entry(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::SubledgerJournalEntry>> { Ok(None) }
    async fn get_journal_entry_by_number(&self, _org_id: Uuid, _entry_number: &str) -> AtlasResult<Option<atlas_shared::SubledgerJournalEntry>> { Ok(None) }
    async fn list_journal_entries(
        &self, _org_id: Uuid, _status: Option<&str>, _source_application: Option<&str>,
        _source_transaction_type: Option<&str>, _accounting_date_from: Option<chrono::NaiveDate>,
        _accounting_date_to: Option<chrono::NaiveDate>,
    ) -> AtlasResult<Vec<atlas_shared::SubledgerJournalEntry>> { Ok(vec![]) }
    async fn update_journal_entry_status(
        &self, _id: Uuid, _status: &str, _error_message: Option<&str>,
        _is_balanced: Option<bool>, _posted_by: Option<Uuid>, _accounted_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::SubledgerJournalEntry> {
        Err(atlas_shared::AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn update_journal_entry_balances(
        &self, _id: Uuid, _total_debit: &str, _total_credit: &str,
        _entered_debit: &str, _entered_credit: &str, _is_balanced: bool,
    ) -> AtlasResult<atlas_shared::SubledgerJournalEntry> {
        Err(atlas_shared::AtlasError::EntityNotFound("Mock".to_string()))
    }

    // Journal Lines
    async fn create_journal_line(
        &self, org_id: Uuid, journal_entry_id: Uuid, line_number: i32,
        line_type: &str, account_code: &str, account_description: Option<&str>,
        derivation_rule_id: Option<Uuid>, entered_amount: &str, accounted_amount: &str,
        currency_code: &str, conversion_date: Option<chrono::NaiveDate>, conversion_rate: Option<&str>,
        attribute_category: Option<&str>, attribute1: Option<&str>, attribute2: Option<&str>,
        attribute3: Option<&str>, attribute4: Option<&str>, attribute5: Option<&str>,
        source_line_id: Option<Uuid>, source_line_type: Option<&str>,
        tax_code: Option<&str>, tax_rate: Option<&str>, tax_amount: Option<&str>,
    ) -> AtlasResult<atlas_shared::SubledgerJournalLine> {
        Ok(atlas_shared::SubledgerJournalLine {
            id: Uuid::new_v4(), organization_id: org_id, journal_entry_id, line_number,
            line_type: line_type.to_string(), account_code: account_code.to_string(),
            account_description: account_description.map(String::from), derivation_rule_id,
            entered_amount: entered_amount.to_string(), accounted_amount: accounted_amount.to_string(),
            currency_code: currency_code.to_string(), conversion_date, conversion_rate: conversion_rate.map(String::from),
            attribute_category: attribute_category.map(String::from), attribute1: attribute1.map(String::from),
            attribute2: attribute2.map(String::from), attribute3: attribute3.map(String::from),
            attribute4: attribute4.map(String::from), attribute5: attribute5.map(String::from),
            attribute6: None, attribute7: None, attribute8: None, attribute9: None, attribute10: None,
            tax_code: tax_code.map(String::from), tax_rate: tax_rate.map(String::from),
            tax_amount: tax_amount.map(String::from),
            source_line_id, source_line_type: source_line_type.map(String::from),
            is_reversal_line: false, reversal_of_line_id: None,
            metadata: serde_json::json!({}), created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn list_journal_lines(&self, _journal_entry_id: Uuid) -> AtlasResult<Vec<atlas_shared::SubledgerJournalLine>> { Ok(vec![]) }
    async fn delete_journal_line(&self, _id: Uuid) -> AtlasResult<()> { Ok(()) }

    // SLA Events
    async fn create_sla_event(
        &self, org_id: Uuid, event_number: &str, event_type: &str,
        source_application: &str, source_transaction_type: &str,
        source_transaction_id: Uuid, journal_entry_id: Option<Uuid>,
        event_date: chrono::NaiveDate, event_status: &str,
        description: Option<&str>, error_message: Option<&str>, processed_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::SlaEvent> {
        Ok(atlas_shared::SlaEvent {
            id: Uuid::new_v4(), organization_id: org_id,
            event_number: event_number.to_string(), event_type: event_type.to_string(),
            source_application: source_application.to_string(),
            source_transaction_type: source_transaction_type.to_string(),
            source_transaction_id, journal_entry_id, event_date,
            event_status: event_status.to_string(), description: description.map(String::from),
            error_message: error_message.map(String::from), processed_by,
            processed_at: Some(chrono::Utc::now()), metadata: serde_json::json!({}),
            created_at: chrono::Utc::now(),
        })
    }
    async fn list_sla_events(&self, _org_id: Uuid, _source_application: Option<&str>, _event_type: Option<&str>) -> AtlasResult<Vec<atlas_shared::SlaEvent>> { Ok(vec![]) }

    // GL Transfer Log
    async fn create_transfer_log(
        &self, org_id: Uuid, transfer_number: &str, from_period: Option<&str>,
        status: &str, total_entries: i32, total_debit: &str, total_credit: &str,
        included_applications: serde_json::Value, transferred_by: Option<Uuid>,
        entries: serde_json::Value,
    ) -> AtlasResult<atlas_shared::GlTransferLog> {
        Ok(atlas_shared::GlTransferLog {
            id: Uuid::new_v4(), organization_id: org_id,
            transfer_number: transfer_number.to_string(), transfer_date: chrono::Utc::now(),
            from_period: from_period.map(String::from), status: status.to_string(),
            error_message: None, total_entries, total_debit: total_debit.to_string(),
            total_credit: total_credit.to_string(), included_applications, transferred_by,
            completed_at: Some(chrono::Utc::now()), entries, metadata: serde_json::json!({}),
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn update_transfer_log_status(&self, _id: Uuid, _status: &str, _error_message: Option<&str>, _completed_at: Option<chrono::DateTime<chrono::Utc>>) -> AtlasResult<atlas_shared::GlTransferLog> {
        Err(atlas_shared::AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn get_transfer_log(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::GlTransferLog>> { Ok(None) }
    async fn list_transfer_logs(&self, _org_id: Uuid, _status: Option<&str>) -> AtlasResult<Vec<atlas_shared::GlTransferLog>> { Ok(vec![]) }

    // Dashboard
    async fn count_entries_by_status(&self, _org_id: Uuid) -> AtlasResult<serde_json::Value> {
        Ok(serde_json::json!({"by_status": [], "by_application": []}))
    }
}

// ============================================================================
// Mock Encumbrance Repository
// ============================================================================

pub struct MockEncumbranceRepository;

#[async_trait]
impl crate::encumbrance::EncumbranceRepository for MockEncumbranceRepository {
    // Encumbrance Types
    async fn create_encumbrance_type(
        &self, org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        category: &str, allow_manual_entry: bool, default_encumbrance_account_code: Option<&str>,
        allow_carry_forward: bool, priority: i32, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::EncumbranceType> {
        Ok(atlas_shared::EncumbranceType {
            id: Uuid::new_v4(), organization_id: org_id,
            code: code.to_string(), name: name.to_string(), description: description.map(String::from),
            category: category.to_string(), is_enabled: true, allow_manual_entry,
            default_encumbrance_account_code: default_encumbrance_account_code.map(String::from),
            allow_carry_forward, priority, metadata: serde_json::json!({}),
            created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_encumbrance_type(&self, _org_id: Uuid, _code: &str) -> AtlasResult<Option<atlas_shared::EncumbranceType>> { Ok(None) }
    async fn get_encumbrance_type_by_id(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::EncumbranceType>> { Ok(None) }
    async fn list_encumbrance_types(&self, _org_id: Uuid) -> AtlasResult<Vec<atlas_shared::EncumbranceType>> { Ok(vec![]) }
    async fn delete_encumbrance_type(&self, _org_id: Uuid, _code: &str) -> AtlasResult<()> { Ok(()) }

    // Entries
    async fn create_entry(
        &self, org_id: Uuid, entry_number: &str, encumbrance_type_id: Uuid,
        encumbrance_type_code: &str, source_type: Option<&str>, source_id: Option<Uuid>,
        source_number: Option<&str>, description: Option<&str>, encumbrance_date: chrono::NaiveDate,
        original_amount: &str, current_amount: &str, currency_code: &str, status: &str,
        fiscal_year: Option<i32>, period_name: Option<&str>, expiry_date: Option<chrono::NaiveDate>,
        budget_line_id: Option<Uuid>, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::EncumbranceEntry> {
        Ok(atlas_shared::EncumbranceEntry {
            id: Uuid::new_v4(), organization_id: org_id,
            entry_number: entry_number.to_string(), encumbrance_type_id, encumbrance_type_code: encumbrance_type_code.to_string(),
            source_type: source_type.map(String::from), source_id, source_number: source_number.map(String::from),
            description: description.map(String::from), encumbrance_date,
            original_amount: original_amount.to_string(), current_amount: current_amount.to_string(),
            liquidated_amount: "0".to_string(), adjusted_amount: "0".to_string(),
            currency_code: currency_code.to_string(), status: status.to_string(),
            fiscal_year, period_name: period_name.map(String::from),
            is_carry_forward: false, carried_forward_from_id: None,
            expiry_date, budget_line_id, metadata: serde_json::json!({}),
            created_by, approved_by: None, cancelled_by: None, cancelled_at: None, cancellation_reason: None,
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_entry(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::EncumbranceEntry>> { Ok(None) }
    async fn get_entry_by_number(&self, _org_id: Uuid, _entry_number: &str) -> AtlasResult<Option<atlas_shared::EncumbranceEntry>> { Ok(None) }
    async fn list_entries(&self, _org_id: Uuid, _status: Option<&str>, _encumbrance_type_code: Option<&str>, _source_type: Option<&str>, _fiscal_year: Option<i32>) -> AtlasResult<Vec<atlas_shared::EncumbranceEntry>> { Ok(vec![]) }
    async fn update_entry_amounts(&self, _id: Uuid, _current: &str, _liquidated: &str, _adjusted: &str, _status: &str) -> AtlasResult<atlas_shared::EncumbranceEntry> {
        Err(atlas_shared::AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn update_entry_status(&self, _id: Uuid, _status: &str, _approved_by: Option<Uuid>, _cancelled_by: Option<Uuid>, _cancellation_reason: Option<&str>) -> AtlasResult<atlas_shared::EncumbranceEntry> {
        Err(atlas_shared::AtlasError::EntityNotFound("Mock".to_string()))
    }

    // Lines
    async fn create_line(
        &self, org_id: Uuid, entry_id: Uuid, line_number: i32, account_code: &str,
        account_description: Option<&str>, department_id: Option<Uuid>, department_name: Option<&str>,
        project_id: Option<Uuid>, project_name: Option<&str>, cost_center: Option<&str>,
        original_amount: &str, current_amount: &str, encumbrance_account_code: Option<&str>,
        source_line_id: Option<Uuid>, _created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::EncumbranceLine> {
        Ok(atlas_shared::EncumbranceLine {
            id: Uuid::new_v4(), organization_id: org_id, entry_id, line_number,
            account_code: account_code.to_string(), account_description: account_description.map(String::from),
            department_id, department_name: department_name.map(String::from),
            project_id, project_name: project_name.map(String::from),
            cost_center: cost_center.map(String::from),
            original_amount: original_amount.to_string(), current_amount: current_amount.to_string(),
            liquidated_amount: "0".to_string(),
            encumbrance_account_code: encumbrance_account_code.map(String::from),
            source_line_id, attribute_category: None, attribute1: None, attribute2: None, attribute3: None,
            metadata: serde_json::json!({}), created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_line(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::EncumbranceLine>> { Ok(None) }
    async fn list_lines_by_entry(&self, _entry_id: Uuid) -> AtlasResult<Vec<atlas_shared::EncumbranceLine>> { Ok(vec![]) }
    async fn update_line_amounts(&self, _id: Uuid, _current: &str, _liquidated: &str) -> AtlasResult<atlas_shared::EncumbranceLine> {
        Err(atlas_shared::AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn delete_line(&self, _id: Uuid) -> AtlasResult<()> { Ok(()) }

    // Liquidations
    async fn create_liquidation(
        &self, org_id: Uuid, liquidation_number: &str, encumbrance_entry_id: Uuid,
        encumbrance_line_id: Option<Uuid>, liquidation_type: &str, liquidation_amount: &str,
        source_type: Option<&str>, source_id: Option<Uuid>, source_number: Option<&str>,
        description: Option<&str>, liquidation_date: chrono::NaiveDate, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::EncumbranceLiquidation> {
        Ok(atlas_shared::EncumbranceLiquidation {
            id: Uuid::new_v4(), organization_id: org_id,
            liquidation_number: liquidation_number.to_string(), encumbrance_entry_id,
            encumbrance_line_id, liquidation_type: liquidation_type.to_string(),
            liquidation_amount: liquidation_amount.to_string(),
            source_type: source_type.map(String::from), source_id, source_number: source_number.map(String::from),
            description: description.map(String::from), liquidation_date,
            status: "draft".to_string(), reversed_by_id: None, reversal_reason: None,
            metadata: serde_json::json!({}), created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_liquidation(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::EncumbranceLiquidation>> { Ok(None) }
    async fn list_liquidations(&self, _org_id: Uuid, _entry_id: Option<Uuid>, _status: Option<&str>) -> AtlasResult<Vec<atlas_shared::EncumbranceLiquidation>> { Ok(vec![]) }
    async fn update_liquidation_status(&self, _id: Uuid, _status: &str, _reversed_by_id: Option<Uuid>, _reversal_reason: Option<&str>) -> AtlasResult<atlas_shared::EncumbranceLiquidation> {
        Err(atlas_shared::AtlasError::EntityNotFound("Mock".to_string()))
    }

    // Carry-Forward
    async fn create_carry_forward(
        &self, org_id: Uuid, batch_number: &str, from_fiscal_year: i32, to_fiscal_year: i32,
        description: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::EncumbranceCarryForward> {
        Ok(atlas_shared::EncumbranceCarryForward {
            id: Uuid::new_v4(), organization_id: org_id,
            batch_number: batch_number.to_string(), from_fiscal_year, to_fiscal_year,
            status: "draft".to_string(), entry_count: 0, total_amount: "0".to_string(),
            description: description.map(String::from), metadata: serde_json::json!({}),
            created_by, processed_by: None, processed_at: None,
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_carry_forward(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::EncumbranceCarryForward>> { Ok(None) }
    async fn list_carry_forwards(&self, _org_id: Uuid) -> AtlasResult<Vec<atlas_shared::EncumbranceCarryForward>> { Ok(vec![]) }
    async fn update_carry_forward_status(&self, _id: Uuid, _status: &str, _count: i32, _amount: &str, _processed_by: Option<Uuid>) -> AtlasResult<atlas_shared::EncumbranceCarryForward> {
        Err(atlas_shared::AtlasError::EntityNotFound("Mock".to_string()))
    }
}

// ============================================================================
// Mock Cash Management Repository
// ============================================================================

pub struct MockCashManagementRepository;

#[async_trait]
impl crate::cash_management::CashManagementRepository for MockCashManagementRepository {
    // Cash Positions
    async fn upsert_cash_position(
        &self, org_id: Uuid, bank_account_id: Uuid, account_number: &str, account_name: &str,
        currency_code: &str, book_balance: &str, available_balance: &str,
        float_amount: &str, one_day_float: &str, two_day_float: &str,
        position_date: chrono::NaiveDate, average_balance: Option<&str>, prior_day_balance: Option<&str>,
        projected_inflows: &str, projected_outflows: &str, projected_net: &str,
        is_reconciled: bool, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::CashPosition> {
        Ok(atlas_shared::CashPosition {
            id: Uuid::new_v4(), organization_id: org_id, bank_account_id,
            account_number: account_number.to_string(), account_name: account_name.to_string(),
            currency_code: currency_code.to_string(),
            book_balance: book_balance.to_string(), available_balance: available_balance.to_string(),
            float_amount: float_amount.to_string(), one_day_float: one_day_float.to_string(),
            two_day_float: two_day_float.to_string(),
            position_date, average_balance: average_balance.map(String::from),
            prior_day_balance: prior_day_balance.map(String::from),
            projected_inflows: projected_inflows.to_string(), projected_outflows: projected_outflows.to_string(),
            projected_net: projected_net.to_string(), is_reconciled,
            metadata: serde_json::json!({}), created_by,
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_cash_position(&self, _org_id: Uuid, _bank_account_id: Uuid, _position_date: chrono::NaiveDate) -> AtlasResult<Option<atlas_shared::CashPosition>> { Ok(None) }
    async fn get_cash_position_by_id(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::CashPosition>> { Ok(None) }
    async fn list_cash_positions(&self, _org_id: Uuid, _position_date: Option<chrono::NaiveDate>) -> AtlasResult<Vec<atlas_shared::CashPosition>> { Ok(vec![]) }

    // Forecast Templates
    async fn create_forecast_template(
        &self, org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        bucket_type: &str, number_of_periods: i32, start_offset_days: i32,
        is_default: bool, columns: serde_json::Value, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::CashForecastTemplate> {
        Ok(atlas_shared::CashForecastTemplate {
            id: Uuid::new_v4(), organization_id: org_id,
            code: code.to_string(), name: name.to_string(), description: description.map(String::from),
            bucket_type: bucket_type.to_string(), number_of_periods, start_offset_days,
            is_default, is_active: true, columns, metadata: serde_json::json!({}),
            created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_forecast_template(&self, _org_id: Uuid, _code: &str) -> AtlasResult<Option<atlas_shared::CashForecastTemplate>> { Ok(None) }
    async fn get_forecast_template_by_id(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::CashForecastTemplate>> { Ok(None) }
    async fn list_forecast_templates(&self, _org_id: Uuid) -> AtlasResult<Vec<atlas_shared::CashForecastTemplate>> { Ok(vec![]) }
    async fn delete_forecast_template(&self, _org_id: Uuid, _code: &str) -> AtlasResult<()> { Ok(()) }

    // Forecast Sources
    async fn create_forecast_source(
        &self, org_id: Uuid, template_id: Uuid, code: &str, name: &str, description: Option<&str>,
        source_type: &str, cash_flow_direction: &str, is_actual: bool, display_order: i32,
        lead_time_days: i32, payment_terms_reference: Option<&str>, account_code_filter: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::CashForecastSource> {
        Ok(atlas_shared::CashForecastSource {
            id: Uuid::new_v4(), organization_id: org_id, template_id,
            code: code.to_string(), name: name.to_string(), description: description.map(String::from),
            source_type: source_type.to_string(), cash_flow_direction: cash_flow_direction.to_string(),
            is_actual, display_order, is_active: true, lead_time_days,
            payment_terms_reference: payment_terms_reference.map(String::from),
            account_code_filter: account_code_filter.map(String::from),
            metadata: serde_json::json!({}), created_by,
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_forecast_source(&self, _org_id: Uuid, _template_id: Uuid, _code: &str) -> AtlasResult<Option<atlas_shared::CashForecastSource>> { Ok(None) }
    async fn list_forecast_sources(&self, _template_id: Uuid) -> AtlasResult<Vec<atlas_shared::CashForecastSource>> { Ok(vec![]) }
    async fn delete_forecast_source(&self, _org_id: Uuid, _template_id: Uuid, _code: &str) -> AtlasResult<()> { Ok(()) }

    // Cash Forecasts
    async fn create_forecast(
        &self, org_id: Uuid, forecast_number: &str, template_id: Uuid, template_name: &str,
        name: &str, description: Option<&str>, start_date: chrono::NaiveDate, end_date: chrono::NaiveDate,
        opening_balance: &str, total_inflows: &str, total_outflows: &str, net_cash_flow: &str,
        closing_balance: &str, minimum_balance: &str, maximum_balance: &str,
        deficit_count: i32, surplus_count: i32, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::CashForecast> {
        Ok(atlas_shared::CashForecast {
            id: Uuid::new_v4(), organization_id: org_id,
            forecast_number: forecast_number.to_string(), template_id, template_name: template_name.to_string(),
            name: name.to_string(), description: description.map(String::from),
            start_date, end_date,
            opening_balance: opening_balance.to_string(), total_inflows: total_inflows.to_string(),
            total_outflows: total_outflows.to_string(), net_cash_flow: net_cash_flow.to_string(),
            closing_balance: closing_balance.to_string(),
            minimum_balance: minimum_balance.to_string(), maximum_balance: maximum_balance.to_string(),
            deficit_count, surplus_count,
            status: "generated".to_string(), is_latest: true, metadata: serde_json::json!({}),
            created_by, approved_by: None, approved_at: None,
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_forecast(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::CashForecast>> { Ok(None) }
    async fn get_forecast_by_number(&self, _org_id: Uuid, _forecast_number: &str) -> AtlasResult<Option<atlas_shared::CashForecast>> { Ok(None) }
    async fn list_forecasts(&self, _org_id: Uuid, _template_id: Option<Uuid>, _status: Option<&str>) -> AtlasResult<Vec<atlas_shared::CashForecast>> { Ok(vec![]) }
    async fn update_forecast_status(&self, _id: Uuid, _status: &str, _approved_by: Option<Uuid>) -> AtlasResult<atlas_shared::CashForecast> {
        Err(atlas_shared::AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn supersede_previous_forecasts(&self, _template_id: Uuid, _new_forecast_id: Uuid) -> AtlasResult<()> { Ok(()) }

    // Forecast Lines
    async fn create_forecast_line(
        &self, org_id: Uuid, forecast_id: Uuid, source_id: Uuid, source_name: &str,
        source_type: &str, cash_flow_direction: &str, period_start_date: chrono::NaiveDate,
        period_end_date: chrono::NaiveDate, period_label: &str, period_sequence: i32,
        amount: &str, cumulative_amount: &str, is_actual: bool, currency_code: &str,
        transaction_count: i32, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::CashForecastLine> {
        Ok(atlas_shared::CashForecastLine {
            id: Uuid::new_v4(), organization_id: org_id, forecast_id, source_id,
            source_name: source_name.to_string(), source_type: source_type.to_string(),
            cash_flow_direction: cash_flow_direction.to_string(),
            period_start_date, period_end_date, period_label: period_label.to_string(),
            period_sequence, amount: amount.to_string(), cumulative_amount: cumulative_amount.to_string(),
            is_actual, currency_code: currency_code.to_string(), transaction_count,
            metadata: serde_json::json!({}), created_by,
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn list_forecast_lines(&self, _forecast_id: Uuid) -> AtlasResult<Vec<atlas_shared::CashForecastLine>> { Ok(vec![]) }
    async fn list_forecast_lines_by_period(&self, _forecast_id: Uuid, _start: chrono::NaiveDate, _end: chrono::NaiveDate) -> AtlasResult<Vec<atlas_shared::CashForecastLine>> { Ok(vec![]) }
}

/// Mock lease accounting repository for testing
pub struct MockLeaseAccountingRepository;

#[async_trait]
impl crate::lease::LeaseAccountingRepository for MockLeaseAccountingRepository {
    async fn create_lease(
        &self, org_id: Uuid, lease_number: &str, title: &str, description: Option<&str>,
        classification: &str,
        lessor_id: Option<Uuid>, lessor_name: Option<&str>,
        asset_description: Option<&str>, location: Option<&str>,
        department_id: Option<Uuid>, department_name: Option<&str>,
        commencement_date: chrono::NaiveDate, end_date: chrono::NaiveDate, lease_term_months: i32,
        purchase_option_exists: bool, purchase_option_likely: bool,
        renewal_option_exists: bool, renewal_option_months: Option<i32>, renewal_option_likely: bool,
        discount_rate: &str, currency_code: &str, payment_frequency: &str,
        _annual_payment_amount: &str,
        escalation_rate: Option<&str>, escalation_frequency_months: Option<i32>,
        total_lease_payments: &str, initial_lease_liability: &str, initial_rou_asset_value: &str,
        residual_guarantee_amount: Option<&str>,
        current_lease_liability: &str, current_rou_asset_value: &str,
        accumulated_rou_depreciation: &str, total_payments_made: &str, periods_elapsed: i32,
        rou_asset_account_code: Option<&str>, rou_depreciation_account_code: Option<&str>,
        lease_liability_account_code: Option<&str>, lease_expense_account_code: Option<&str>,
        interest_expense_account_code: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::LeaseContract> {
        Ok(atlas_shared::LeaseContract {
            id: Uuid::new_v4(), organization_id: org_id,
            lease_number: lease_number.to_string(), title: title.to_string(),
            description: description.map(String::from), classification: classification.to_string(),
            lessor_id, lessor_name: lessor_name.map(String::from),
            asset_description: asset_description.map(String::from),
            location: location.map(String::from),
            department_id, department_name: department_name.map(String::from),
            commencement_date, end_date, lease_term_months,
            purchase_option_exists, purchase_option_likely,
            renewal_option_exists, renewal_option_months, renewal_option_likely,
            discount_rate: discount_rate.to_string(),
            currency_code: currency_code.to_string(),
            payment_frequency: payment_frequency.to_string(),
            escalation_rate: escalation_rate.map(String::from),
            escalation_frequency_months,
            total_lease_payments: total_lease_payments.to_string(),
            initial_lease_liability: initial_lease_liability.to_string(),
            initial_rou_asset_value: initial_rou_asset_value.to_string(),
            residual_guarantee_amount: residual_guarantee_amount.map(String::from),
            current_lease_liability: current_lease_liability.to_string(),
            current_rou_asset_value: current_rou_asset_value.to_string(),
            accumulated_rou_depreciation: accumulated_rou_depreciation.to_string(),
            total_payments_made: total_payments_made.to_string(),
            periods_elapsed, status: "draft".to_string(),
            rou_asset_account_code: rou_asset_account_code.map(String::from),
            rou_depreciation_account_code: rou_depreciation_account_code.map(String::from),
            lease_liability_account_code: lease_liability_account_code.map(String::from),
            lease_expense_account_code: lease_expense_account_code.map(String::from),
            interest_expense_account_code: interest_expense_account_code.map(String::from),
            impairment_amount: None, impairment_date: None,
            metadata: serde_json::json!({}), created_by,
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_lease(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::LeaseContract>> { Ok(None) }
    async fn get_lease_by_number(&self, _org_id: Uuid, _lease_number: &str) -> AtlasResult<Option<atlas_shared::LeaseContract>> { Ok(None) }
    async fn list_leases(&self, _org_id: Uuid, _status: Option<&str>, _classification: Option<&str>) -> AtlasResult<Vec<atlas_shared::LeaseContract>> { Ok(vec![]) }
    async fn update_lease_status(&self, _id: Uuid, _status: &str, _impairment_amount: Option<&str>, _impairment_date: Option<chrono::NaiveDate>) -> AtlasResult<atlas_shared::LeaseContract> {
        Err(atlas_shared::AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn update_lease_balances(&self, _id: Uuid, _current_liability: &str, _current_rou: &str, _accumulated_depreciation: &str, _total_payments_made: &str, _periods_elapsed: i32) -> AtlasResult<()> { Ok(()) }

    async fn create_payment(
        &self, org_id: Uuid, lease_id: Uuid, period_number: i32, payment_date: chrono::NaiveDate,
        payment_amount: &str, interest_amount: &str, principal_amount: &str,
        remaining_liability: &str, rou_asset_value: &str, rou_depreciation: &str,
        accumulated_depreciation: &str, lease_expense: &str, is_paid: bool,
        payment_reference: Option<&str>, journal_entry_id: Option<Uuid>, status: &str,
    ) -> AtlasResult<atlas_shared::LeasePayment> {
        Ok(atlas_shared::LeasePayment {
            id: Uuid::new_v4(), organization_id: org_id, lease_id, period_number, payment_date,
            payment_amount: payment_amount.to_string(), interest_amount: interest_amount.to_string(),
            principal_amount: principal_amount.to_string(), remaining_liability: remaining_liability.to_string(),
            rou_asset_value: rou_asset_value.to_string(), rou_depreciation: rou_depreciation.to_string(),
            accumulated_depreciation: accumulated_depreciation.to_string(),
            lease_expense: lease_expense.to_string(), is_paid,
            payment_reference: payment_reference.map(String::from),
            journal_entry_id, status: status.to_string(),
            metadata: serde_json::json!({}),
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_payment_by_period(&self, _lease_id: Uuid, _period_number: i32) -> AtlasResult<Option<atlas_shared::LeasePayment>> { Ok(None) }
    async fn list_payments(&self, _lease_id: Uuid) -> AtlasResult<Vec<atlas_shared::LeasePayment>> { Ok(vec![]) }
    async fn update_payment_status(&self, _id: Uuid, _status: &str, _is_paid: bool, _payment_reference: Option<&str>, _journal_entry_id: Option<Uuid>) -> AtlasResult<atlas_shared::LeasePayment> {
        Err(atlas_shared::AtlasError::EntityNotFound("Mock".to_string()))
    }

    async fn create_modification(
        &self, org_id: Uuid, lease_id: Uuid, modification_number: i32, modification_type: &str,
        description: Option<&str>, effective_date: chrono::NaiveDate,
        previous_term_months: Option<i32>, new_term_months: Option<i32>,
        previous_end_date: Option<chrono::NaiveDate>, new_end_date: Option<chrono::NaiveDate>,
        previous_discount_rate: Option<&str>, new_discount_rate: Option<&str>,
        liability_adjustment: &str, rou_asset_adjustment: &str, status: &str, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::LeaseModification> {
        Ok(atlas_shared::LeaseModification {
            id: Uuid::new_v4(), organization_id: org_id, lease_id,
            modification_number, modification_type: modification_type.to_string(),
            description: description.map(String::from), effective_date,
            previous_term_months, new_term_months, previous_end_date, new_end_date,
            previous_discount_rate: previous_discount_rate.map(String::from),
            new_discount_rate: new_discount_rate.map(String::from),
            liability_adjustment: liability_adjustment.to_string(),
            rou_asset_adjustment: rou_asset_adjustment.to_string(),
            status: status.to_string(),
            metadata: serde_json::json!({}), created_by,
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_next_modification_number(&self, _lease_id: Uuid) -> AtlasResult<i32> { Ok(1) }
    async fn list_modifications(&self, _lease_id: Uuid) -> AtlasResult<Vec<atlas_shared::LeaseModification>> { Ok(vec![]) }

    async fn create_termination(
        &self, org_id: Uuid, lease_id: Uuid, termination_type: &str, termination_date: chrono::NaiveDate,
        reason: Option<&str>, remaining_liability: &str, remaining_rou_asset: &str,
        termination_penalty: &str, gain_loss_amount: &str, gain_loss_type: Option<&str>,
        journal_entry_id: Option<Uuid>, status: &str, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::LeaseTermination> {
        Ok(atlas_shared::LeaseTermination {
            id: Uuid::new_v4(), organization_id: org_id, lease_id,
            termination_type: termination_type.to_string(), termination_date, reason: reason.map(String::from),
            remaining_liability: remaining_liability.to_string(), remaining_rou_asset: remaining_rou_asset.to_string(),
            termination_penalty: termination_penalty.to_string(),
            gain_loss_amount: gain_loss_amount.to_string(), gain_loss_type: gain_loss_type.map(String::from),
            journal_entry_id, status: status.to_string(),
            metadata: serde_json::json!({}), created_by,
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn list_terminations(&self, _lease_id: Uuid) -> AtlasResult<Vec<atlas_shared::LeaseTermination>> { Ok(vec![]) }

    async fn get_dashboard_summary(&self, _org_id: Uuid) -> AtlasResult<atlas_shared::LeaseDashboardSummary> {
        Ok(atlas_shared::LeaseDashboardSummary {
            total_active_leases: 0, total_lease_liability: "0".to_string(),
            total_rou_assets: "0".to_string(), total_rou_depreciation: "0".to_string(),
            total_net_rou_assets: "0".to_string(), total_payments_made: "0".to_string(),
            operating_lease_count: 0, finance_lease_count: 0,
            upcoming_payments_count: 0, upcoming_payments_amount: "0".to_string(),
            leases_expiring_90_days: 0,
            leases_by_classification: serde_json::json!({}),
            leases_by_status: serde_json::json!({}),
            liability_by_period: serde_json::json!({}),
        })
    }
}

/// Mock project costing repository for testing
pub struct MockProjectCostingRepository;

#[async_trait]
impl ProjectCostingRepository for MockProjectCostingRepository {
    async fn create_cost_transaction(
        &self, org_id: Uuid, transaction_number: &str,
        project_id: Uuid, _project_number: Option<&str>,
        task_id: Option<Uuid>, _task_number: Option<&str>,
        cost_type: &str, raw_cost_amount: &str, burdened_cost_amount: &str,
        burden_amount: &str, currency_code: &str, transaction_date: chrono::NaiveDate,
        gl_date: Option<chrono::NaiveDate>, description: Option<&str>,
        supplier_id: Option<Uuid>, supplier_name: Option<&str>,
        employee_id: Option<Uuid>, employee_name: Option<&str>,
        expenditure_category: Option<&str>, quantity: Option<&str>,
        unit_of_measure: Option<&str>, unit_rate: Option<&str>,
        is_billable: bool, is_capitalizable: bool,
        original_transaction_id: Option<Uuid>, adjustment_type: Option<&str>,
        adjustment_reason: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::ProjectCostTransaction> {
        Ok(atlas_shared::ProjectCostTransaction {
            id: Uuid::new_v4(), organization_id: org_id,
            transaction_number: transaction_number.to_string(),
            project_id, project_number: None, task_id, task_number: None,
            cost_type: cost_type.to_string(),
            raw_cost_amount: raw_cost_amount.to_string(),
            burdened_cost_amount: burdened_cost_amount.to_string(),
            burden_amount: burden_amount.to_string(),
            currency_code: currency_code.to_string(),
            transaction_date, gl_date,
            description: description.map(String::from),
            supplier_id, supplier_name: supplier_name.map(String::from),
            employee_id, employee_name: employee_name.map(String::from),
            expenditure_category: expenditure_category.map(String::from),
            quantity: quantity.map(String::from),
            unit_of_measure: unit_of_measure.map(String::from),
            unit_rate: unit_rate.map(String::from),
            is_billable, is_capitalizable,
            status: "draft".to_string(),
            distribution_id: None, original_transaction_id,
            adjustment_type: adjustment_type.map(String::from),
            adjustment_reason: adjustment_reason.map(String::from),
            metadata: serde_json::json!({}),
            created_by, approved_by: None,
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_cost_transaction(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::ProjectCostTransaction>> { Ok(None) }
    async fn get_cost_transaction_by_number(&self, _org_id: Uuid, _transaction_number: &str) -> AtlasResult<Option<atlas_shared::ProjectCostTransaction>> { Ok(None) }
    async fn list_cost_transactions(&self, _org_id: Uuid, _project_id: Option<Uuid>, _cost_type: Option<&str>, _status: Option<&str>) -> AtlasResult<Vec<atlas_shared::ProjectCostTransaction>> { Ok(vec![]) }
    async fn update_cost_transaction_status(&self, _id: Uuid, _status: &str, _approved_by: Option<Uuid>) -> AtlasResult<atlas_shared::ProjectCostTransaction> {
        Err(atlas_shared::AtlasError::EntityNotFound("Mock".to_string()))
    }

    async fn create_burden_schedule(
        &self, org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        status: &str, effective_from: chrono::NaiveDate, effective_to: Option<chrono::NaiveDate>,
        is_default: bool, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::BurdenSchedule> {
        Ok(atlas_shared::BurdenSchedule {
            id: Uuid::new_v4(), organization_id: org_id,
            code: code.to_string(), name: name.to_string(),
            description: description.map(String::from),
            status: status.to_string(), effective_from, effective_to,
            is_default, metadata: serde_json::json!({}),
            created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_burden_schedule(&self, _org_id: Uuid, _code: &str) -> AtlasResult<Option<atlas_shared::BurdenSchedule>> { Ok(None) }
    async fn get_burden_schedule_by_id(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::BurdenSchedule>> { Ok(None) }
    async fn list_burden_schedules(&self, _org_id: Uuid) -> AtlasResult<Vec<atlas_shared::BurdenSchedule>> { Ok(vec![]) }
    async fn get_default_burden_schedule(&self, _org_id: Uuid) -> AtlasResult<Option<atlas_shared::BurdenSchedule>> { Ok(None) }
    async fn update_burden_schedule_status(&self, _id: Uuid, _status: &str) -> AtlasResult<atlas_shared::BurdenSchedule> {
        Err(atlas_shared::AtlasError::EntityNotFound("Mock".to_string()))
    }

    async fn create_burden_schedule_line(
        &self, org_id: Uuid, schedule_id: Uuid, line_number: i32,
        cost_type: &str, expenditure_category: Option<&str>,
        burden_rate_percent: &str, burden_account_code: Option<&str>,
    ) -> AtlasResult<atlas_shared::BurdenScheduleLine> {
        Ok(atlas_shared::BurdenScheduleLine {
            id: Uuid::new_v4(), organization_id: org_id, schedule_id, line_number,
            cost_type: cost_type.to_string(),
            expenditure_category: expenditure_category.map(String::from),
            burden_rate_percent: burden_rate_percent.to_string(),
            burden_account_code: burden_account_code.map(String::from),
            is_active: true, metadata: serde_json::json!({}),
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn list_burden_schedule_lines(&self, _schedule_id: Uuid) -> AtlasResult<Vec<atlas_shared::BurdenScheduleLine>> { Ok(vec![]) }
    async fn get_applicable_burden_rate(&self, _schedule_id: Uuid, _cost_type: &str, _expenditure_category: Option<&str>) -> AtlasResult<Option<atlas_shared::BurdenScheduleLine>> { Ok(None) }

    async fn create_cost_adjustment(
        &self, org_id: Uuid, adjustment_number: &str, original_transaction_id: Uuid,
        adjustment_type: &str, adjustment_amount: &str, new_raw_cost: &str,
        new_burdened_cost: &str, reason: &str, description: Option<&str>,
        effective_date: chrono::NaiveDate, transfer_to_project_id: Option<Uuid>,
        transfer_to_task_id: Option<Uuid>, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::ProjectCostAdjustment> {
        Ok(atlas_shared::ProjectCostAdjustment {
            id: Uuid::new_v4(), organization_id: org_id,
            adjustment_number: adjustment_number.to_string(),
            original_transaction_id,
            adjustment_type: adjustment_type.to_string(),
            adjustment_amount: adjustment_amount.to_string(),
            new_raw_cost: new_raw_cost.to_string(),
            new_burdened_cost: new_burdened_cost.to_string(),
            reason: reason.to_string(),
            description: description.map(String::from),
            effective_date, transfer_to_project_id, transfer_to_task_id,
            status: "pending".to_string(), created_transaction_id: None,
            metadata: serde_json::json!({}),
            created_by, approved_by: None, approved_at: None,
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_cost_adjustment(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::ProjectCostAdjustment>> { Ok(None) }
    async fn list_cost_adjustments(&self, _org_id: Uuid, _status: Option<&str>) -> AtlasResult<Vec<atlas_shared::ProjectCostAdjustment>> { Ok(vec![]) }
    async fn update_cost_adjustment_status(
        &self, _id: Uuid, _status: &str, _approved_by: Option<Uuid>, _created_transaction_id: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::ProjectCostAdjustment> {
        Err(atlas_shared::AtlasError::EntityNotFound("Mock".to_string()))
    }

    async fn create_cost_distribution(
        &self, org_id: Uuid, transaction_id: Uuid, line_number: i32,
        debit_account_code: &str, credit_account_code: &str, amount: &str,
        distribution_type: &str, gl_date: chrono::NaiveDate,
    ) -> AtlasResult<atlas_shared::ProjectCostDistribution> {
        Ok(atlas_shared::ProjectCostDistribution {
            id: Uuid::new_v4(), organization_id: org_id, transaction_id, line_number,
            debit_account_code: debit_account_code.to_string(),
            credit_account_code: credit_account_code.to_string(),
            amount: amount.to_string(), distribution_type: distribution_type.to_string(),
            gl_date, is_posted: false, gl_batch_id: None,
            metadata: serde_json::json!({}),
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn list_cost_distributions(&self, _transaction_id: Uuid) -> AtlasResult<Vec<atlas_shared::ProjectCostDistribution>> { Ok(vec![]) }
    async fn list_unposted_distributions(&self, _org_id: Uuid) -> AtlasResult<Vec<atlas_shared::ProjectCostDistribution>> { Ok(vec![]) }
    async fn mark_distribution_posted(&self, _id: Uuid, _gl_batch_id: Option<Uuid>) -> AtlasResult<atlas_shared::ProjectCostDistribution> {
        Err(atlas_shared::AtlasError::EntityNotFound("Mock".to_string()))
    }

    async fn get_costing_summary(&self, _org_id: Uuid) -> AtlasResult<atlas_shared::ProjectCostingSummary> {
        Ok(atlas_shared::ProjectCostingSummary {
            project_count: 0, total_raw_costs: "0".to_string(),
            total_burdened_costs: "0".to_string(), total_burden: "0".to_string(),
            total_capitalized: "0".to_string(), total_billed: "0".to_string(),
            costs_by_type: serde_json::json!({}), costs_by_project: serde_json::json!({}),
            costs_by_month: serde_json::json!({}),
            pending_adjustments: 0, pending_distributions: 0,
        })
    }
}

/// Mock withholding tax repository for testing
pub struct MockWithholdingTaxRepository;

#[async_trait]
impl crate::withholding_tax::WithholdingTaxRepository for MockWithholdingTaxRepository {
    async fn create_tax_code(
        &self, org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        tax_type: &str, rate_percentage: &str, threshold_amount: &str,
        threshold_is_cumulative: bool, withholding_account_code: Option<&str>,
        expense_account_code: Option<&str>, effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::WithholdingTaxCode> {
        Ok(atlas_shared::WithholdingTaxCode {
            id: Uuid::new_v4(), organization_id: org_id,
            code: code.to_string(), name: name.to_string(),
            description: description.map(String::from),
            tax_type: tax_type.to_string(),
            rate_percentage: rate_percentage.to_string(),
            threshold_amount: threshold_amount.to_string(),
            threshold_is_cumulative,
            withholding_account_code: withholding_account_code.map(String::from),
            expense_account_code: expense_account_code.map(String::from),
            is_active: true, effective_from, effective_to,
            created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_tax_code(&self, _org_id: Uuid, _code: &str) -> AtlasResult<Option<atlas_shared::WithholdingTaxCode>> { Ok(None) }
    async fn get_tax_code_by_id(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::WithholdingTaxCode>> { Ok(None) }
    async fn list_tax_codes(&self, _org_id: Uuid, _tax_type: Option<&str>) -> AtlasResult<Vec<atlas_shared::WithholdingTaxCode>> { Ok(vec![]) }
    async fn delete_tax_code(&self, _org_id: Uuid, _code: &str) -> AtlasResult<()> { Ok(()) }

    async fn create_tax_group(
        &self, org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::WithholdingTaxGroup> {
        Ok(atlas_shared::WithholdingTaxGroup {
            id: Uuid::new_v4(), organization_id: org_id,
            code: code.to_string(), name: name.to_string(),
            description: description.map(String::from),
            tax_codes: vec![], is_active: true,
            created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_tax_group(&self, _org_id: Uuid, _code: &str) -> AtlasResult<Option<atlas_shared::WithholdingTaxGroup>> { Ok(None) }
    async fn get_tax_group_by_id(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::WithholdingTaxGroup>> { Ok(None) }
    async fn list_tax_groups(&self, _org_id: Uuid) -> AtlasResult<Vec<atlas_shared::WithholdingTaxGroup>> { Ok(vec![]) }
    async fn delete_tax_group(&self, _org_id: Uuid, _code: &str) -> AtlasResult<()> { Ok(()) }

    async fn add_group_member(
        &self, group_id: Uuid, tax_code_id: Uuid, rate_override: Option<&str>,
        display_order: i32,
    ) -> AtlasResult<atlas_shared::WithholdingTaxGroupMember> {
        Ok(atlas_shared::WithholdingTaxGroupMember {
            id: Uuid::new_v4(), group_id, tax_code_id,
            tax_code: "MOCK".to_string(), tax_code_name: "Mock Tax Code".to_string(),
            rate_override: rate_override.map(String::from),
            is_active: true, display_order,
        })
    }
    async fn list_group_members(&self, _group_id: Uuid) -> AtlasResult<Vec<atlas_shared::WithholdingTaxGroupMember>> { Ok(vec![]) }
    async fn remove_group_member(&self, _id: Uuid) -> AtlasResult<()> { Ok(()) }

    async fn create_supplier_assignment(
        &self, org_id: Uuid, supplier_id: Uuid, supplier_number: Option<&str>,
        supplier_name: Option<&str>, tax_group_id: Uuid, is_exempt: bool,
        exemption_reason: Option<&str>, exemption_certificate: Option<&str>,
        exemption_valid_until: Option<chrono::NaiveDate>, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::SupplierWithholdingAssignment> {
        Ok(atlas_shared::SupplierWithholdingAssignment {
            id: Uuid::new_v4(), organization_id: org_id, supplier_id,
            supplier_number: supplier_number.map(String::from),
            supplier_name: supplier_name.map(String::from),
            tax_group_id,
            tax_group_code: "MOCK_GROUP".to_string(),
            tax_group_name: "Mock Group".to_string(),
            is_exempt, exemption_reason: exemption_reason.map(String::from),
            exemption_certificate: exemption_certificate.map(String::from),
            exemption_valid_until, is_active: true,
            created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_supplier_assignment(&self, _org_id: Uuid, _supplier_id: Uuid) -> AtlasResult<Option<atlas_shared::SupplierWithholdingAssignment>> { Ok(None) }
    async fn list_supplier_assignments(&self, _org_id: Uuid) -> AtlasResult<Vec<atlas_shared::SupplierWithholdingAssignment>> { Ok(vec![]) }
    async fn delete_supplier_assignment(&self, _id: Uuid) -> AtlasResult<()> { Ok(()) }

    async fn create_withholding_line(
        &self, org_id: Uuid, payment_id: Uuid, payment_number: Option<&str>,
        invoice_id: Uuid, invoice_number: Option<&str>,
        supplier_id: Uuid, supplier_name: Option<&str>,
        tax_code_id: Uuid, tax_code: &str, tax_code_name: Option<&str>,
        tax_type: &str, rate_percentage: &str, taxable_amount: &str,
        withheld_amount: &str, withholding_account_code: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::WithholdingTaxLine> {
        Ok(atlas_shared::WithholdingTaxLine {
            id: Uuid::new_v4(), organization_id: org_id,
            payment_id, payment_number: payment_number.map(String::from),
            invoice_id, invoice_number: invoice_number.map(String::from),
            supplier_id, supplier_name: supplier_name.map(String::from),
            tax_code_id, tax_code: tax_code.to_string(),
            tax_code_name: tax_code_name.map(String::from),
            tax_type: tax_type.to_string(),
            rate_percentage: rate_percentage.to_string(),
            taxable_amount: taxable_amount.to_string(),
            withheld_amount: withheld_amount.to_string(),
            withholding_account_code: withholding_account_code.map(String::from),
            status: "pending".to_string(),
            remittance_date: None, remittance_reference: None,
            metadata: serde_json::json!({}),
            created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_withholding_lines_by_payment(&self, _payment_id: Uuid) -> AtlasResult<Vec<atlas_shared::WithholdingTaxLine>> { Ok(vec![]) }
    async fn get_withholding_lines_by_supplier(
        &self, _org_id: Uuid, _supplier_id: Uuid,
        _from_date: Option<chrono::NaiveDate>, _to_date: Option<chrono::NaiveDate>,
    ) -> AtlasResult<Vec<atlas_shared::WithholdingTaxLine>> { Ok(vec![]) }
    async fn update_withholding_line_status(
        &self, _id: Uuid, _status: &str,
        _remittance_date: Option<chrono::NaiveDate>, _remittance_reference: Option<&str>,
    ) -> AtlasResult<atlas_shared::WithholdingTaxLine> {
        Err(atlas_shared::AtlasError::EntityNotFound("Mock".to_string()))
    }

    async fn create_certificate(
        &self, org_id: Uuid, certificate_number: &str,
        supplier_id: Uuid, supplier_number: Option<&str>,
        supplier_name: Option<&str>, tax_type: &str,
        tax_code_id: Uuid, tax_code: &str,
        period_start: chrono::NaiveDate, period_end: chrono::NaiveDate,
        total_invoice_amount: &str, total_withheld_amount: &str,
        rate_percentage: &str, payment_ids: serde_json::Value,
        created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::WithholdingCertificate> {
        Ok(atlas_shared::WithholdingCertificate {
            id: Uuid::new_v4(), organization_id: org_id,
            certificate_number: certificate_number.to_string(),
            supplier_id, supplier_number: supplier_number.map(String::from),
            supplier_name: supplier_name.map(String::from),
            tax_type: tax_type.to_string(), tax_code_id,
            tax_code: tax_code.to_string(),
            period_start, period_end,
            total_invoice_amount: total_invoice_amount.to_string(),
            total_withheld_amount: total_withheld_amount.to_string(),
            rate_percentage: rate_percentage.to_string(),
            payment_ids, status: "draft".to_string(),
            issued_at: None, acknowledged_at: None,
            notes: None, metadata: serde_json::json!({}),
            created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_certificate(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::WithholdingCertificate>> { Ok(None) }
    async fn get_certificate_by_number(&self, _org_id: Uuid, _certificate_number: &str) -> AtlasResult<Option<atlas_shared::WithholdingCertificate>> { Ok(None) }
    async fn list_certificates(&self, _org_id: Uuid, _supplier_id: Option<Uuid>) -> AtlasResult<Vec<atlas_shared::WithholdingCertificate>> { Ok(vec![]) }
    async fn update_certificate_status(&self, _id: Uuid, _status: &str) -> AtlasResult<atlas_shared::WithholdingCertificate> {
        Err(atlas_shared::AtlasError::EntityNotFound("Mock".to_string()))
    }
}

/// Mock procurement contract repository for testing
pub struct MockProcurementContractRepository;

#[async_trait]
impl crate::procurement_contracts::ProcurementContractRepository for MockProcurementContractRepository {
    async fn create_contract_type(
        &self, org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        contract_classification: &str, requires_approval: bool,
        default_duration_days: Option<i32>,
        allow_amount_commitment: bool, allow_quantity_commitment: bool,
        allow_line_additions: bool, allow_price_adjustment: bool,
        allow_renewal: bool, allow_termination: bool,
        max_renewals: Option<i32>,
        default_payment_terms_code: Option<&str>,
        default_currency_code: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::ContractType> {
        Ok(atlas_shared::ContractType {
            id: Uuid::new_v4(), organization_id: org_id,
            code: code.to_string(), name: name.to_string(),
            description: description.map(String::from),
            contract_classification: contract_classification.to_string(),
            requires_approval, default_duration_days,
            allow_amount_commitment, allow_quantity_commitment,
            allow_line_additions, allow_price_adjustment,
            allow_renewal, allow_termination, max_renewals,
            default_payment_terms_code: default_payment_terms_code.map(String::from),
            default_currency_code: default_currency_code.map(String::from),
            is_active: true, metadata: serde_json::json!({}),
            created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_contract_type(&self, _org_id: Uuid, _code: &str) -> AtlasResult<Option<atlas_shared::ContractType>> { Ok(None) }
    async fn get_contract_type_by_id(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::ContractType>> { Ok(None) }
    async fn list_contract_types(&self, _org_id: Uuid) -> AtlasResult<Vec<atlas_shared::ContractType>> { Ok(vec![]) }
    async fn delete_contract_type(&self, _org_id: Uuid, _code: &str) -> AtlasResult<()> { Ok(()) }

    async fn create_contract(
        &self, org_id: Uuid, contract_number: &str, title: &str, description: Option<&str>,
        contract_type_code: Option<&str>, contract_classification: &str,
        supplier_id: Uuid, supplier_number: Option<&str>,
        supplier_name: Option<&str>, supplier_contact: Option<&str>,
        buyer_id: Option<Uuid>, buyer_name: Option<&str>,
        start_date: Option<chrono::NaiveDate>, end_date: Option<chrono::NaiveDate>,
        total_committed_amount: &str, currency_code: &str,
        payment_terms_code: Option<&str>, price_type: &str,
        max_renewals: Option<i32>, notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::ProcurementContract> {
        Ok(atlas_shared::ProcurementContract {
            id: Uuid::new_v4(), organization_id: org_id,
            contract_number: contract_number.to_string(),
            title: title.to_string(), description: description.map(String::from),
            contract_type_code: contract_type_code.map(String::from),
            contract_classification: contract_classification.to_string(),
            status: "draft".to_string(),
            supplier_id, supplier_number: supplier_number.map(String::from),
            supplier_name: supplier_name.map(String::from),
            supplier_contact: supplier_contact.map(String::from),
            buyer_id, buyer_name: buyer_name.map(String::from),
            start_date, end_date,
            total_committed_amount: total_committed_amount.to_string(),
            total_released_amount: "0".to_string(),
            total_invoiced_amount: "0".to_string(),
            currency_code: currency_code.to_string(),
            payment_terms_code: payment_terms_code.map(String::from),
            price_type: price_type.to_string(),
            renewal_count: 0, max_renewals, line_count: 0, milestone_count: 0,
            approved_by: None, approved_at: None, rejection_reason: None,
            termination_reason: None, terminated_by: None, terminated_at: None,
            notes: notes.map(String::from),
            metadata: serde_json::json!({}),
            created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_contract(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::ProcurementContract>> { Ok(None) }
    async fn get_contract_by_number(&self, _org_id: Uuid, _contract_number: &str) -> AtlasResult<Option<atlas_shared::ProcurementContract>> { Ok(None) }
    async fn list_contracts(&self, _org_id: Uuid, _status: Option<&str>, _supplier_id: Option<Uuid>) -> AtlasResult<Vec<atlas_shared::ProcurementContract>> { Ok(vec![]) }
    async fn update_contract_status(
        &self, _id: Uuid, _status: &str, _approved_by: Option<Uuid>,
        _rejection_reason: Option<&str>, _terminated_by: Option<Uuid>, _termination_reason: Option<&str>,
    ) -> AtlasResult<atlas_shared::ProcurementContract> {
        Err(atlas_shared::AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn update_contract_totals(
        &self, _id: Uuid, _total_committed: Option<&str>, _total_released: Option<&str>,
        _total_invoiced: Option<&str>, _line_count: Option<i32>, _milestone_count: Option<i32>,
    ) -> AtlasResult<atlas_shared::ProcurementContract> {
        Err(atlas_shared::AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn update_contract_dates(
        &self, _id: Uuid, _start_date: Option<chrono::NaiveDate>, _end_date: Option<chrono::NaiveDate>,
    ) -> AtlasResult<atlas_shared::ProcurementContract> {
        Err(atlas_shared::AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn increment_renewal_count(&self, _id: Uuid) -> AtlasResult<atlas_shared::ProcurementContract> {
        Err(atlas_shared::AtlasError::EntityNotFound("Mock".to_string()))
    }

    async fn create_contract_line(
        &self, org_id: Uuid, contract_id: Uuid, line_number: i32,
        item_description: &str, item_code: Option<&str>,
        category: Option<&str>, uom: Option<&str>,
        quantity_committed: Option<&str>, quantity_released: &str,
        unit_price: &str, line_amount: &str, amount_released: &str,
        delivery_date: Option<chrono::NaiveDate>, supplier_part_number: Option<&str>,
        account_code: Option<&str>, cost_center: Option<&str>,
        project_id: Option<Uuid>, notes: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::ContractLine> {
        Ok(atlas_shared::ContractLine {
            id: Uuid::new_v4(), organization_id: org_id, contract_id, line_number,
            item_description: item_description.to_string(),
            item_code: item_code.map(String::from),
            category: category.map(String::from),
            uom: uom.map(String::from),
            quantity_committed: quantity_committed.map(String::from),
            quantity_released: quantity_released.to_string(),
            unit_price: unit_price.to_string(),
            line_amount: line_amount.to_string(),
            amount_released: amount_released.to_string(),
            delivery_date, supplier_part_number: supplier_part_number.map(String::from),
            account_code: account_code.map(String::from),
            cost_center: cost_center.map(String::from),
            project_id, notes: notes.map(String::from),
            metadata: serde_json::json!({}),
            created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_contract_line(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::ContractLine>> { Ok(None) }
    async fn list_contract_lines(&self, _contract_id: Uuid) -> AtlasResult<Vec<atlas_shared::ContractLine>> { Ok(vec![]) }
    async fn delete_contract_line(&self, _id: Uuid) -> AtlasResult<()> { Ok(()) }

    async fn create_milestone(
        &self, org_id: Uuid, contract_id: Uuid, contract_line_id: Option<Uuid>,
        milestone_number: i32, name: &str, description: Option<&str>,
        milestone_type: &str, target_date: chrono::NaiveDate,
        amount: &str, percent_of_total: &str, deliverable: Option<&str>,
        is_billable: bool, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::ContractMilestone> {
        Ok(atlas_shared::ContractMilestone {
            id: Uuid::new_v4(), organization_id: org_id, contract_id, contract_line_id,
            milestone_number, name: name.to_string(), description: description.map(String::from),
            milestone_type: milestone_type.to_string(), target_date, actual_date: None,
            status: "pending".to_string(), amount: amount.to_string(),
            percent_of_total: percent_of_total.to_string(),
            deliverable: deliverable.map(String::from), is_billable,
            approved_by: None, approved_at: None, notes: None,
            metadata: serde_json::json!({}),
            created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_milestone(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::ContractMilestone>> { Ok(None) }
    async fn list_milestones(&self, _contract_id: Uuid) -> AtlasResult<Vec<atlas_shared::ContractMilestone>> { Ok(vec![]) }
    async fn update_milestone_status(
        &self, _id: Uuid, _status: &str, _actual_date: Option<chrono::NaiveDate>,
    ) -> AtlasResult<atlas_shared::ContractMilestone> {
        Err(atlas_shared::AtlasError::EntityNotFound("Mock".to_string()))
    }

    async fn create_renewal(
        &self, org_id: Uuid, contract_id: Uuid, renewal_number: i32,
        previous_end_date: chrono::NaiveDate, new_end_date: chrono::NaiveDate,
        renewal_type: &str, terms_changed: Option<&str>,
        renewed_by: Option<Uuid>, notes: Option<&str>,
    ) -> AtlasResult<atlas_shared::ContractRenewal> {
        Ok(atlas_shared::ContractRenewal {
            id: Uuid::new_v4(), organization_id: org_id, contract_id, renewal_number,
            previous_end_date, new_end_date,
            renewal_type: renewal_type.to_string(),
            terms_changed: terms_changed.map(String::from),
            renewed_by, renewed_at: chrono::Utc::now(),
            notes: notes.map(String::from),
            metadata: serde_json::json!({}), created_at: chrono::Utc::now(),
        })
    }
    async fn list_renewals(&self, _contract_id: Uuid) -> AtlasResult<Vec<atlas_shared::ContractRenewal>> { Ok(vec![]) }

    async fn create_spend_entry(
        &self, org_id: Uuid, contract_id: Uuid, contract_line_id: Option<Uuid>,
        source_type: &str, source_id: Option<Uuid>, source_number: Option<&str>,
        transaction_date: chrono::NaiveDate, amount: &str, quantity: Option<&str>,
        description: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::ContractSpend> {
        Ok(atlas_shared::ContractSpend {
            id: Uuid::new_v4(), organization_id: org_id, contract_id, contract_line_id,
            source_type: source_type.to_string(), source_id,
            source_number: source_number.map(String::from),
            transaction_date, amount: amount.to_string(),
            quantity: quantity.map(String::from), description: description.map(String::from),
            metadata: serde_json::json!({}),
            created_by, created_at: chrono::Utc::now(),
        })
    }
    async fn list_spend_entries(&self, _contract_id: Uuid) -> AtlasResult<Vec<atlas_shared::ContractSpend>> { Ok(vec![]) }
}

/// Mock treasury repository for testing
pub struct MockTreasuryRepository;

#[async_trait]
impl crate::treasury::TreasuryRepository for MockTreasuryRepository {
    async fn create_counterparty(
        &self, org_id: Uuid, counterparty_code: &str, name: &str, counterparty_type: &str,
        country_code: Option<&str>, credit_rating: Option<&str>, credit_limit: Option<&str>,
        settlement_currency: Option<&str>, contact_name: Option<&str>, contact_email: Option<&str>,
        contact_phone: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::TreasuryCounterparty> {
        Ok(atlas_shared::TreasuryCounterparty {
            id: Uuid::new_v4(), organization_id: org_id,
            counterparty_code: counterparty_code.to_string(), name: name.to_string(),
            counterparty_type: counterparty_type.to_string(),
            country_code: country_code.map(String::from), credit_rating: credit_rating.map(String::from),
            credit_limit: credit_limit.map(String::from),
            settlement_currency: settlement_currency.map(String::from),
            contact_name: contact_name.map(String::from), contact_email: contact_email.map(String::from),
            contact_phone: contact_phone.map(String::from),
            is_active: true, metadata: serde_json::json!({}),
            created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_counterparty(&self, _org_id: Uuid, _code: &str) -> AtlasResult<Option<atlas_shared::TreasuryCounterparty>> { Ok(None) }
    async fn get_counterparty_by_id(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::TreasuryCounterparty>> { Ok(None) }
    async fn list_counterparties(&self, _org_id: Uuid, _active_only: bool) -> AtlasResult<Vec<atlas_shared::TreasuryCounterparty>> { Ok(vec![]) }
    async fn delete_counterparty(&self, _org_id: Uuid, _code: &str) -> AtlasResult<()> { Ok(()) }

    async fn create_deal(
        &self, org_id: Uuid, deal_number: &str, deal_type: &str, description: Option<&str>,
        counterparty_id: Uuid, counterparty_name: Option<&str>, currency_code: &str,
        principal_amount: &str, interest_rate: Option<&str>, interest_basis: Option<&str>,
        start_date: chrono::NaiveDate, maturity_date: chrono::NaiveDate, term_days: i32,
        fx_buy_currency: Option<&str>, fx_buy_amount: Option<&str>,
        fx_sell_currency: Option<&str>, fx_sell_amount: Option<&str>, fx_rate: Option<&str>,
        gl_account_code: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::TreasuryDeal> {
        Ok(atlas_shared::TreasuryDeal {
            id: Uuid::new_v4(), organization_id: org_id,
            deal_number: deal_number.to_string(), deal_type: deal_type.to_string(),
            description: description.map(String::from),
            counterparty_id, counterparty_name: counterparty_name.map(String::from),
            currency_code: currency_code.to_string(),
            principal_amount: principal_amount.to_string(),
            interest_rate: interest_rate.map(String::from),
            interest_basis: interest_basis.map(String::from),
            start_date, maturity_date, term_days,
            fx_buy_currency: fx_buy_currency.map(String::from),
            fx_buy_amount: fx_buy_amount.map(String::from),
            fx_sell_currency: fx_sell_currency.map(String::from),
            fx_sell_amount: fx_sell_amount.map(String::from),
            fx_rate: fx_rate.map(String::from),
            accrued_interest: "0".to_string(), settlement_amount: None,
            gl_account_code: gl_account_code.map(String::from),
            status: "draft".to_string(),
            authorized_by: None, authorized_at: None,
            settled_at: None, matured_at: None,
            metadata: serde_json::json!({}),
            created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_deal(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::TreasuryDeal>> { Ok(None) }
    async fn get_deal_by_number(&self, _org_id: Uuid, _deal_number: &str) -> AtlasResult<Option<atlas_shared::TreasuryDeal>> { Ok(None) }
    async fn list_deals(&self, _org_id: Uuid, _deal_type: Option<&str>, _status: Option<&str>) -> AtlasResult<Vec<atlas_shared::TreasuryDeal>> { Ok(vec![]) }
    async fn update_deal_status(
        &self, _id: Uuid, _status: &str, _authorized_by: Option<Uuid>,
        _settled_at: Option<chrono::DateTime<chrono::Utc>>, _matured_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> AtlasResult<atlas_shared::TreasuryDeal> {
        Err(atlas_shared::AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn update_deal_interest(&self, _id: Uuid, _accrued_interest: &str, _settlement_amount: Option<&str>) -> AtlasResult<()> { Ok(()) }

    async fn create_settlement(
        &self, org_id: Uuid, deal_id: Uuid, settlement_number: &str, settlement_type: &str,
        settlement_date: chrono::NaiveDate, principal_amount: &str, interest_amount: &str,
        total_amount: &str, payment_reference: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::TreasurySettlement> {
        Ok(atlas_shared::TreasurySettlement {
            id: Uuid::new_v4(), organization_id: org_id, deal_id,
            settlement_number: settlement_number.to_string(),
            settlement_type: settlement_type.to_string(),
            settlement_date, principal_amount: principal_amount.to_string(),
            interest_amount: interest_amount.to_string(),
            total_amount: total_amount.to_string(),
            payment_reference: payment_reference.map(String::from),
            journal_entry_id: None, status: "pending".to_string(),
            metadata: serde_json::json!({}),
            created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn list_settlements(&self, _deal_id: Uuid) -> AtlasResult<Vec<atlas_shared::TreasurySettlement>> { Ok(vec![]) }
    async fn update_settlement_status(&self, _id: Uuid, _status: &str) -> AtlasResult<atlas_shared::TreasurySettlement> {
        Err(atlas_shared::AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn get_dashboard_summary(&self, _org_id: Uuid) -> AtlasResult<atlas_shared::TreasuryDashboardSummary> {
        Ok(atlas_shared::TreasuryDashboardSummary {
            total_active_deals: 0, total_investments: "0".to_string(),
            total_borrowings: "0".to_string(), total_fx_exposure: "0".to_string(),
            total_accrued_interest: "0".to_string(),
            deals_maturing_7_days: 0, deals_maturing_30_days: 0,
            investment_count: 0, borrowing_count: 0, fx_deal_count: 0,
            active_counterparties: 0,
            deals_by_status: serde_json::json!({}), deals_by_type: serde_json::json!({}),
            maturity_profile: serde_json::json!({}),
        })
    }
}

/// Mock subscription repository for testing
pub struct MockSubscriptionRepository;

#[async_trait]
impl crate::subscription::SubscriptionRepository for MockSubscriptionRepository {
    // Products
    async fn create_product(&self, _org_id: Uuid, _product_code: &str, _name: &str,
        _description: Option<&str>, _product_type: &str, _billing_frequency: &str,
        _default_duration_months: i32, _is_auto_renew: bool, _cancellation_notice_days: i32,
        _setup_fee: &str, _tier_type: &str, _created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::SubscriptionProduct> {
        Ok(atlas_shared::SubscriptionProduct {
            id: Uuid::new_v4(), organization_id: _org_id,
            product_code: _product_code.to_string(), name: _name.to_string(),
            description: None, product_type: _product_type.to_string(),
            billing_frequency: _billing_frequency.to_string(),
            default_duration_months: _default_duration_months,
            is_auto_renew: _is_auto_renew,
            cancellation_notice_days: _cancellation_notice_days,
            setup_fee: _setup_fee.to_string(), tier_type: _tier_type.to_string(),
            is_active: true, metadata: serde_json::json!({}),
            created_by: None, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_product(&self, _org_id: Uuid, _code: &str) -> AtlasResult<Option<atlas_shared::SubscriptionProduct>> { Ok(None) }
    async fn get_product_by_id(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::SubscriptionProduct>> { Ok(None) }
    async fn list_products(&self, _org_id: Uuid, _active_only: bool) -> AtlasResult<Vec<atlas_shared::SubscriptionProduct>> { Ok(vec![]) }
    async fn delete_product(&self, _org_id: Uuid, _code: &str) -> AtlasResult<()> { Ok(()) }

    // Price Tiers
    async fn create_price_tier(&self, _org_id: Uuid, _product_id: Uuid, _tier_name: Option<&str>,
        _min_quantity: &str, _max_quantity: Option<&str>, _unit_price: &str,
        _discount_percent: &str, _currency_code: &str,
        _effective_from: Option<chrono::NaiveDate>, _effective_to: Option<chrono::NaiveDate>,
    ) -> AtlasResult<atlas_shared::SubscriptionPriceTier> {
        Ok(atlas_shared::SubscriptionPriceTier {
            id: Uuid::new_v4(), organization_id: _org_id, product_id: _product_id,
            tier_name: None, min_quantity: _min_quantity.to_string(),
            max_quantity: None, unit_price: _unit_price.to_string(),
            discount_percent: _discount_percent.to_string(),
            currency_code: _currency_code.to_string(),
            effective_from: None, effective_to: None, is_active: true,
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn list_price_tiers(&self, _org_id: Uuid, _product_id: Uuid) -> AtlasResult<Vec<atlas_shared::SubscriptionPriceTier>> { Ok(vec![]) }

    // Subscriptions
    #[allow(clippy::too_many_arguments)]
    async fn create_subscription(&self, _org_id: Uuid, _subscription_number: &str,
        _customer_id: Uuid, _customer_name: Option<&str>, _product_id: Uuid,
        _product_code: Option<&str>, _product_name: Option<&str>, _description: Option<&str>,
        _status: &str, _start_date: chrono::NaiveDate, _end_date: Option<chrono::NaiveDate>,
        _renewal_date: Option<&chrono::NaiveDate>, _billing_frequency: &str,
        _billing_day_of_month: i32, _billing_alignment: &str, _currency_code: &str,
        _quantity: &str, _unit_price: &str, _list_price: &str, _discount_percent: &str,
        _setup_fee: &str, _recurring_amount: &str, _total_contract_value: &str,
        _total_billed: &str, _total_revenue_recognized: &str, _duration_months: i32,
        _is_auto_renew: bool, _cancellation_date: Option<chrono::NaiveDate>,
        _cancellation_reason: Option<&str>, _suspension_reason: Option<&str>,
        _sales_rep_id: Option<Uuid>, _sales_rep_name: Option<&str>,
        _gl_revenue_account: Option<&str>, _gl_deferred_account: Option<&str>,
        _created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::Subscription> {
        Ok(atlas_shared::Subscription {
            id: Uuid::new_v4(), organization_id: _org_id,
            subscription_number: _subscription_number.to_string(),
            customer_id: _customer_id, customer_name: _customer_name.map(String::from),
            product_id: _product_id, product_code: _product_code.map(String::from),
            product_name: _product_name.map(String::from), description: None,
            status: _status.to_string(), start_date: _start_date, end_date: _end_date,
            renewal_date: _renewal_date.copied(), billing_frequency: _billing_frequency.to_string(),
            billing_day_of_month: _billing_day_of_month,
            billing_alignment: _billing_alignment.to_string(),
            currency_code: _currency_code.to_string(),
            quantity: _quantity.to_string(), unit_price: _unit_price.to_string(),
            list_price: _list_price.to_string(), discount_percent: _discount_percent.to_string(),
            setup_fee: _setup_fee.to_string(), recurring_amount: _recurring_amount.to_string(),
            total_contract_value: _total_contract_value.to_string(),
            total_billed: _total_billed.to_string(),
            total_revenue_recognized: _total_revenue_recognized.to_string(),
            duration_months: _duration_months, is_auto_renew: _is_auto_renew,
            cancellation_date: None, cancellation_reason: None, suspension_reason: None,
            sales_rep_id: None, sales_rep_name: None,
            gl_revenue_account: None, gl_deferred_account: None,
            metadata: serde_json::json!({}), created_by: None,
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_subscription(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::Subscription>> { Ok(None) }
    async fn get_subscription_by_number(&self, _org_id: Uuid, _number: &str) -> AtlasResult<Option<atlas_shared::Subscription>> { Ok(None) }
    async fn list_subscriptions(&self, _org_id: Uuid, _status: Option<&str>, _customer_id: Option<Uuid>) -> AtlasResult<Vec<atlas_shared::Subscription>> { Ok(vec![]) }
    async fn update_subscription_status(&self, _id: Uuid, _status: &str, _cancellation_date: Option<chrono::NaiveDate>, _cancellation_reason: Option<&str>, _suspension_reason: Option<&str>) -> AtlasResult<atlas_shared::Subscription> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn update_subscription_dates(&self, _id: Uuid, _end_date: Option<chrono::NaiveDate>, _renewal_date: Option<&chrono::NaiveDate>) -> AtlasResult<atlas_shared::Subscription> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn update_subscription_pricing(&self, _id: Uuid, _quantity: &str, _unit_price: &str, _recurring_amount: &str) -> AtlasResult<()> { Ok(()) }

    // Amendments
    #[allow(clippy::too_many_arguments)]
    async fn create_amendment(&self, _org_id: Uuid, _subscription_id: Uuid, _amendment_number: &str,
        _amendment_type: &str, _description: Option<&str>,
        _old_quantity: Option<&str>, _new_quantity: Option<&str>,
        _old_unit_price: Option<&str>, _new_unit_price: Option<&str>,
        _old_recurring_amount: Option<&str>, _new_recurring_amount: Option<&str>,
        _old_end_date: Option<&chrono::NaiveDate>, _new_end_date: Option<&chrono::NaiveDate>,
        _effective_date: chrono::NaiveDate, _proration_credit: &str, _proration_charge: &str,
        _status: &str, _created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::SubscriptionAmendment> {
        Ok(atlas_shared::SubscriptionAmendment {
            id: Uuid::new_v4(), organization_id: _org_id, subscription_id: _subscription_id,
            amendment_number: _amendment_number.to_string(),
            amendment_type: _amendment_type.to_string(), description: None,
            old_quantity: None, new_quantity: None, old_unit_price: None, new_unit_price: None,
            old_recurring_amount: None, new_recurring_amount: None,
            old_end_date: None, new_end_date: None, effective_date: _effective_date,
            proration_credit: None, proration_charge: None,
            status: _status.to_string(), applied_at: None, applied_by: None,
            metadata: serde_json::json!({}), created_by: None,
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_amendment(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::SubscriptionAmendment>> { Ok(None) }
    async fn list_amendments(&self, _subscription_id: Uuid) -> AtlasResult<Vec<atlas_shared::SubscriptionAmendment>> { Ok(vec![]) }
    async fn update_amendment_status(&self, _id: Uuid, _status: &str, _applied_by: Option<Uuid>) -> AtlasResult<atlas_shared::SubscriptionAmendment> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }

    // Billing Schedule
    async fn create_billing_line(&self, _org_id: Uuid, _subscription_id: Uuid, _schedule_number: i32,
        _billing_date: chrono::NaiveDate, _period_start: chrono::NaiveDate, _period_end: chrono::NaiveDate,
        _amount: &str, _proration_amount: &str, _total_amount: &str,
    ) -> AtlasResult<atlas_shared::SubscriptionBillingLine> {
        Ok(atlas_shared::SubscriptionBillingLine {
            id: Uuid::new_v4(), organization_id: _org_id, subscription_id: _subscription_id,
            schedule_number: _schedule_number, billing_date: _billing_date,
            period_start: _period_start, period_end: _period_end,
            amount: _amount.to_string(), proration_amount: _proration_amount.to_string(),
            total_amount: _total_amount.to_string(), invoice_id: None, invoice_number: None,
            status: "pending".to_string(), paid_at: None, metadata: serde_json::json!({}),
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn list_billing_lines(&self, _subscription_id: Uuid) -> AtlasResult<Vec<atlas_shared::SubscriptionBillingLine>> { Ok(vec![]) }

    // Revenue Schedule
    async fn create_revenue_line(&self, _org_id: Uuid, _subscription_id: Uuid, _billing_schedule_id: Option<Uuid>,
        _period_name: &str, _period_start: chrono::NaiveDate, _period_end: chrono::NaiveDate,
        _revenue_amount: &str, _deferred_amount: &str, _recognized_to_date: &str, _status: &str,
    ) -> AtlasResult<atlas_shared::SubscriptionRevenueLine> {
        Ok(atlas_shared::SubscriptionRevenueLine {
            id: Uuid::new_v4(), organization_id: _org_id, subscription_id: _subscription_id,
            billing_schedule_id: _billing_schedule_id, period_name: _period_name.to_string(),
            period_start: _period_start, period_end: _period_end,
            revenue_amount: _revenue_amount.to_string(),
            deferred_amount: _deferred_amount.to_string(),
            recognized_to_date: _recognized_to_date.to_string(),
            status: _status.to_string(), recognized_at: None, journal_entry_id: None,
            metadata: serde_json::json!({}), created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_revenue_line(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::SubscriptionRevenueLine>> { Ok(None) }
    async fn list_revenue_lines(&self, _subscription_id: Uuid) -> AtlasResult<Vec<atlas_shared::SubscriptionRevenueLine>> { Ok(vec![]) }
    async fn update_revenue_line_status(&self, _id: Uuid, _status: &str) -> AtlasResult<atlas_shared::SubscriptionRevenueLine> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }

    // Dashboard
    async fn get_dashboard_summary(&self, _org_id: Uuid) -> AtlasResult<atlas_shared::SubscriptionDashboardSummary> {
        Ok(atlas_shared::SubscriptionDashboardSummary {
            total_active_subscriptions: 0, total_subscribers: 0,
            total_monthly_recurring_revenue: "0".to_string(),
            total_annual_recurring_revenue: "0".to_string(),
            total_contract_value: "0".to_string(), total_billed: "0".to_string(),
            total_revenue_recognized: "0".to_string(), total_deferred_revenue: "0".to_string(),
            churn_rate_percent: "0".to_string(), renewals_due_30_days: 0,
            new_subscriptions_this_month: 0, cancelled_this_month: 0,
            subscriptions_by_status: serde_json::json!({}),
            revenue_by_product: serde_json::json!({}),
        })
    }
}

/// Mock corporate card repository for testing
pub struct MockCorporateCardRepository;

#[async_trait]
impl crate::corporate_card::CorporateCardRepository for MockCorporateCardRepository {
    // ── Card Programmes ─────────────────────────────────────────────
    async fn create_program(
        &self, org_id: Uuid, program_code: &str, name: &str, description: Option<&str>,
        issuer_bank: &str, card_network: &str, card_type: &str, currency_code: &str,
        default_single_purchase_limit: &str, default_monthly_limit: &str,
        default_cash_limit: &str, default_atm_limit: &str,
        allow_cash_withdrawal: bool, allow_international: bool,
        auto_deactivate_on_termination: bool, expense_matching_method: &str,
        billing_cycle_day: i32, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::CorporateCardProgram> {
        Ok(atlas_shared::CorporateCardProgram {
            id: Uuid::new_v4(),
            organization_id: org_id,
            program_code: program_code.to_string(),
            name: name.to_string(),
            description: description.map(String::from),
            issuer_bank: issuer_bank.to_string(),
            card_network: card_network.to_string(),
            card_type: card_type.to_string(),
            currency_code: currency_code.to_string(),
            default_single_purchase_limit: default_single_purchase_limit.to_string(),
            default_monthly_limit: default_monthly_limit.to_string(),
            default_cash_limit: default_cash_limit.to_string(),
            default_atm_limit: default_atm_limit.to_string(),
            allow_cash_withdrawal,
            allow_international,
            auto_deactivate_on_termination,
            expense_matching_method: expense_matching_method.to_string(),
            billing_cycle_day,
            is_active: true,
            metadata: serde_json::json!({}),
            created_by,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })
    }

    async fn get_program(&self, _org_id: Uuid, _code: &str) -> AtlasResult<Option<atlas_shared::CorporateCardProgram>> { Ok(None) }
    async fn get_program_by_id(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::CorporateCardProgram>> { Ok(None) }
    async fn list_programs(&self, _org_id: Uuid, _active_only: bool) -> AtlasResult<Vec<atlas_shared::CorporateCardProgram>> { Ok(vec![]) }

    // ── Cards ───────────────────────────────────────────────────────
    async fn create_card(
        &self, org_id: Uuid, program_id: Uuid, card_number_masked: &str,
        cardholder_name: &str, cardholder_id: Uuid, cardholder_email: Option<&str>,
        department_id: Option<Uuid>, department_name: Option<&str>,
        status: &str, issue_date: chrono::NaiveDate, expiry_date: chrono::NaiveDate,
        single_purchase_limit: &str, monthly_limit: &str,
        cash_limit: &str, atm_limit: &str,
        gl_liability_account: Option<&str>, gl_expense_account: Option<&str>,
        cost_center: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::CorporateCard> {
        Ok(atlas_shared::CorporateCard {
            id: Uuid::new_v4(), organization_id: org_id, program_id,
            card_number_masked: card_number_masked.to_string(),
            cardholder_name: cardholder_name.to_string(),
            cardholder_id, cardholder_email: cardholder_email.map(String::from),
            department_id, department_name: department_name.map(String::from),
            status: status.to_string(), issue_date, expiry_date,
            single_purchase_limit: single_purchase_limit.to_string(),
            monthly_limit: monthly_limit.to_string(),
            cash_limit: cash_limit.to_string(),
            atm_limit: atm_limit.to_string(),
            current_balance: "0".to_string(),
            total_spend_current_cycle: "0".to_string(),
            last_statement_balance: "0".to_string(),
            last_statement_date: None,
            gl_liability_account: gl_liability_account.map(String::from),
            gl_expense_account: gl_expense_account.map(String::from),
            cost_center: cost_center.map(String::from),
            metadata: serde_json::json!({}),
            created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }

    async fn get_card(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::CorporateCard>> { Ok(None) }
    async fn get_card_by_masked_number(&self, _org_id: Uuid, _masked: &str) -> AtlasResult<Option<atlas_shared::CorporateCard>> { Ok(None) }
    async fn list_cards(&self, _org_id: Uuid, _program_id: Option<Uuid>, _cardholder_id: Option<Uuid>, _status: Option<&str>) -> AtlasResult<Vec<atlas_shared::CorporateCard>> { Ok(vec![]) }
    async fn update_card_status(&self, _id: Uuid, _status: &str) -> AtlasResult<atlas_shared::CorporateCard> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn update_card_limits(&self, _id: Uuid, _single_purchase: &str, _monthly: &str, _cash: &str, _atm: &str) -> AtlasResult<()> { Ok(()) }
    async fn update_card_spend(&self, _id: Uuid, _amount: &str, _balance: &str) -> AtlasResult<()> { Ok(()) }

    // ── Transactions ────────────────────────────────────────────────
    async fn create_transaction(
        &self, org_id: Uuid, card_id: Uuid, program_id: Uuid,
        transaction_reference: &str, posting_date: chrono::NaiveDate,
        transaction_date: chrono::NaiveDate, merchant_name: &str,
        merchant_category: Option<&str>, merchant_category_code: Option<&str>,
        amount: &str, currency_code: &str,
        original_amount: Option<&str>, original_currency: Option<&str>,
        exchange_rate: Option<&str>, transaction_type: &str,
    ) -> AtlasResult<atlas_shared::CorporateCardTransaction> {
        Ok(atlas_shared::CorporateCardTransaction {
            id: Uuid::new_v4(), organization_id: org_id, card_id, program_id,
            transaction_reference: transaction_reference.to_string(),
            posting_date, transaction_date,
            merchant_name: merchant_name.to_string(),
            merchant_category: merchant_category.map(String::from),
            merchant_category_code: merchant_category_code.map(String::from),
            amount: amount.to_string(),
            currency_code: currency_code.to_string(),
            original_amount: original_amount.map(String::from),
            original_currency: original_currency.map(String::from),
            exchange_rate: exchange_rate.map(String::from),
            transaction_type: transaction_type.to_string(),
            status: "unmatched".to_string(),
            expense_report_id: None, expense_line_id: None,
            matched_at: None, matched_by: None, match_confidence: None,
            dispute_reason: None, dispute_date: None, dispute_resolution: None,
            gl_posted: false, gl_journal_id: None,
            metadata: serde_json::json!({}),
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }

    async fn get_transaction(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::CorporateCardTransaction>> { Ok(None) }
    async fn get_transaction_by_ref(&self, _org_id: Uuid, _reference: &str) -> AtlasResult<Option<atlas_shared::CorporateCardTransaction>> { Ok(None) }
    async fn list_transactions(&self, _org_id: Uuid, _card_id: Option<Uuid>, _status: Option<&str>, _date_from: Option<chrono::NaiveDate>, _date_to: Option<chrono::NaiveDate>) -> AtlasResult<Vec<atlas_shared::CorporateCardTransaction>> { Ok(vec![]) }
    async fn update_transaction_match(&self, _id: Uuid, _expense_report_id: Option<Uuid>, _expense_line_id: Option<Uuid>, _status: &str, _matched_by: Option<Uuid>, _match_confidence: Option<&str>) -> AtlasResult<atlas_shared::CorporateCardTransaction> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn update_transaction_dispute(&self, _id: Uuid, _reason: Option<&str>, _dispute_date: Option<chrono::NaiveDate>, _resolution: Option<&str>, _status: &str) -> AtlasResult<atlas_shared::CorporateCardTransaction> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }

    // ── Statements ──────────────────────────────────────────────────
    async fn create_statement(
        &self, org_id: Uuid, program_id: Uuid, statement_number: &str,
        statement_date: chrono::NaiveDate, billing_period_start: chrono::NaiveDate, billing_period_end: chrono::NaiveDate,
        opening_balance: &str, closing_balance: &str,
        total_charges: &str, total_credits: &str, total_payments: &str,
        total_fees: &str, total_interest: &str,
        payment_due_date: Option<chrono::NaiveDate>, minimum_payment: &str,
        imported_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::CorporateCardStatement> {
        Ok(atlas_shared::CorporateCardStatement {
            id: Uuid::new_v4(), organization_id: org_id, program_id,
            statement_number: statement_number.to_string(),
            statement_date, billing_period_start, billing_period_end,
            opening_balance: opening_balance.to_string(),
            closing_balance: closing_balance.to_string(),
            total_charges: total_charges.to_string(),
            total_credits: total_credits.to_string(),
            total_payments: total_payments.to_string(),
            total_fees: total_fees.to_string(),
            total_interest: total_interest.to_string(),
            payment_due_date, minimum_payment: minimum_payment.to_string(),
            total_transaction_count: 0, matched_transaction_count: 0, unmatched_transaction_count: 0,
            status: "imported".to_string(),
            payment_reference: None, paid_at: None, gl_payment_journal_id: None,
            metadata: serde_json::json!({}),
            imported_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }

    async fn get_statement(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::CorporateCardStatement>> { Ok(None) }
    async fn list_statements(&self, _org_id: Uuid, _program_id: Option<Uuid>, _status: Option<&str>) -> AtlasResult<Vec<atlas_shared::CorporateCardStatement>> { Ok(vec![]) }
    async fn update_statement_counts(&self, _id: Uuid, _total: i32, _matched: i32, _unmatched: i32) -> AtlasResult<()> { Ok(()) }
    async fn update_statement_status(&self, _id: Uuid, _status: &str, _payment_reference: Option<&str>) -> AtlasResult<atlas_shared::CorporateCardStatement> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }

    // ── Limit Overrides ─────────────────────────────────────────────
    async fn create_limit_override(
        &self, org_id: Uuid, card_id: Uuid, override_type: &str,
        original_value: &str, new_value: &str, reason: &str,
        effective_from: chrono::NaiveDate, effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::CorporateCardLimitOverride> {
        Ok(atlas_shared::CorporateCardLimitOverride {
            id: Uuid::new_v4(), organization_id: org_id, card_id,
            override_type: override_type.to_string(),
            original_value: original_value.to_string(),
            new_value: new_value.to_string(),
            reason: reason.to_string(),
            effective_from, effective_to,
            status: "pending".to_string(),
            approved_by: None, approved_at: None,
            metadata: serde_json::json!({}),
            created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }

    async fn get_limit_override(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::CorporateCardLimitOverride>> { Ok(None) }
    async fn list_limit_overrides(&self, _card_id: Option<Uuid>, _status: Option<&str>) -> AtlasResult<Vec<atlas_shared::CorporateCardLimitOverride>> { Ok(vec![]) }
    async fn update_limit_override_status(&self, _id: Uuid, _status: &str, _approved_by: Option<Uuid>) -> AtlasResult<atlas_shared::CorporateCardLimitOverride> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }

    // ── Dashboard ───────────────────────────────────────────────────
    async fn get_dashboard_summary(&self, _org_id: Uuid) -> AtlasResult<atlas_shared::CorporateCardDashboardSummary> {
        Ok(atlas_shared::CorporateCardDashboardSummary {
            total_active_cards: 0, total_programs: 0,
            total_cards_by_status: serde_json::json!({}),
            total_spend_current_month: "0".to_string(),
            total_spend_previous_month: "0".to_string(),
            spend_change_percent: "0".to_string(),
            total_unmatched_transactions: 0,
            total_unreconciled_statements: 0,
            total_disputed_transactions: 0,
            top_spenders: serde_json::json!([]),
            spend_by_category: serde_json::json!({}),
            limit_overrides_pending: 0,
        })
    }
}

/// Mock financial consolidation repository for testing
pub struct MockFinancialConsolidationRepository;

#[async_trait]
impl crate::financial_consolidation::FinancialConsolidationRepository for MockFinancialConsolidationRepository {
    // ── Consolidation Ledgers ───────────────────────────────────────
    async fn create_ledger(
        &self, org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        base_currency_code: &str, translation_method: &str,
        equity_elimination_method: &str, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::ConsolidationLedger> {
        Ok(atlas_shared::ConsolidationLedger {
            id: Uuid::new_v4(), organization_id: org_id,
            code: code.to_string(), name: name.to_string(),
            description: description.map(String::from),
            base_currency_code: base_currency_code.to_string(),
            translation_method: translation_method.to_string(),
            equity_elimination_method: equity_elimination_method.to_string(),
            is_active: true,
            metadata: serde_json::json!({}),
            created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }

    async fn get_ledger(&self, _org_id: Uuid, _code: &str) -> AtlasResult<Option<atlas_shared::ConsolidationLedger>> { Ok(None) }
    async fn get_ledger_by_id(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::ConsolidationLedger>> { Ok(None) }
    async fn list_ledgers(&self, _org_id: Uuid, _active_only: bool) -> AtlasResult<Vec<atlas_shared::ConsolidationLedger>> { Ok(vec![]) }

    // ── Consolidation Entities ──────────────────────────────────────
    async fn create_entity(
        &self, org_id: Uuid, ledger_id: Uuid, entity_id: Uuid,
        entity_name: &str, entity_code: &str, local_currency_code: &str,
        ownership_percentage: &str, consolidation_method: &str,
        effective_from: Option<chrono::NaiveDate>, effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::ConsolidationEntity> {
        Ok(atlas_shared::ConsolidationEntity {
            id: Uuid::new_v4(), organization_id: org_id, ledger_id, entity_id,
            entity_name: entity_name.to_string(), entity_code: entity_code.to_string(),
            local_currency_code: local_currency_code.to_string(),
            ownership_percentage: ownership_percentage.to_string(),
            consolidation_method: consolidation_method.to_string(),
            is_active: true, include_in_consolidation: true,
            effective_from, effective_to,
            metadata: serde_json::json!({}),
            created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }

    async fn get_entity(&self, _ledger_id: Uuid, _entity_code: &str) -> AtlasResult<Option<atlas_shared::ConsolidationEntity>> { Ok(None) }
    async fn get_entity_by_id(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::ConsolidationEntity>> { Ok(None) }
    async fn list_entities(&self, _ledger_id: Uuid, _active_only: bool) -> AtlasResult<Vec<atlas_shared::ConsolidationEntity>> { Ok(vec![]) }

    // ── Consolidation Scenarios ─────────────────────────────────────
    async fn create_scenario(
        &self, org_id: Uuid, ledger_id: Uuid, scenario_number: &str,
        name: &str, description: Option<&str>,
        fiscal_year: i32, period_name: &str,
        period_start_date: chrono::NaiveDate, period_end_date: chrono::NaiveDate,
        translation_rate_type: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::ConsolidationScenario> {
        Ok(atlas_shared::ConsolidationScenario {
            id: Uuid::new_v4(), organization_id: org_id, ledger_id,
            scenario_number: scenario_number.to_string(),
            name: name.to_string(), description: description.map(String::from),
            fiscal_year, period_name: period_name.to_string(),
            period_start_date, period_end_date,
            status: "draft".to_string(),
            translation_date: None, translation_rate_type: translation_rate_type.map(String::from),
            total_entities: 0, total_eliminations: 0, total_adjustments: 0,
            total_debits: "0".to_string(), total_credits: "0".to_string(),
            is_balanced: false,
            approved_by: None, approved_at: None,
            posted_by: None, posted_at: None,
            metadata: serde_json::json!({}),
            created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }

    async fn get_scenario(&self, _org_id: Uuid, _scenario_number: &str) -> AtlasResult<Option<atlas_shared::ConsolidationScenario>> { Ok(None) }
    async fn get_scenario_by_id(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::ConsolidationScenario>> { Ok(None) }
    async fn list_scenarios(&self, _org_id: Uuid, _ledger_id: Option<Uuid>, _status: Option<&str>) -> AtlasResult<Vec<atlas_shared::ConsolidationScenario>> { Ok(vec![]) }
    async fn update_scenario_status(
        &self, _id: Uuid, _status: &str, _approved_by: Option<Uuid>, _posted_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::ConsolidationScenario> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn update_scenario_totals(
        &self, _id: Uuid, _total_entities: i32, _total_eliminations: i32,
        _total_adjustments: i32, _total_debits: &str, _total_credits: &str, _is_balanced: bool,
    ) -> AtlasResult<()> { Ok(()) }

    // ── Trial Balance Lines ─────────────────────────────────────────
    async fn create_trial_balance_line(
        &self, org_id: Uuid, scenario_id: Uuid,
        entity_id: Option<Uuid>, entity_code: Option<&str>,
        account_code: &str, account_name: Option<&str>,
        account_type: Option<&str>, financial_statement: Option<&str>,
        local_debit: &str, local_credit: &str, local_balance: &str,
        exchange_rate: Option<&str>,
        translated_debit: &str, translated_credit: &str, translated_balance: &str,
        elimination_debit: &str, elimination_credit: &str, elimination_balance: &str,
        minority_interest_debit: &str, minority_interest_credit: &str, minority_interest_balance: &str,
        consolidated_debit: &str, consolidated_credit: &str, consolidated_balance: &str,
        is_elimination_entry: bool, line_type: &str,
    ) -> AtlasResult<atlas_shared::ConsolidationTrialBalanceLine> {
        Ok(atlas_shared::ConsolidationTrialBalanceLine {
            id: Uuid::new_v4(), organization_id: org_id, scenario_id,
            entity_id, entity_code: entity_code.map(String::from),
            account_code: account_code.to_string(),
            account_name: account_name.map(String::from),
            account_type: account_type.map(String::from),
            financial_statement: financial_statement.map(String::from),
            local_debit: local_debit.to_string(), local_credit: local_credit.to_string(),
            local_balance: local_balance.to_string(),
            exchange_rate: exchange_rate.map(String::from),
            translated_debit: translated_debit.to_string(), translated_credit: translated_credit.to_string(),
            translated_balance: translated_balance.to_string(),
            elimination_debit: elimination_debit.to_string(), elimination_credit: elimination_credit.to_string(),
            elimination_balance: elimination_balance.to_string(),
            minority_interest_debit: minority_interest_debit.to_string(),
            minority_interest_credit: minority_interest_credit.to_string(),
            minority_interest_balance: minority_interest_balance.to_string(),
            consolidated_debit: consolidated_debit.to_string(),
            consolidated_credit: consolidated_credit.to_string(),
            consolidated_balance: consolidated_balance.to_string(),
            is_elimination_entry, line_type: line_type.to_string(),
            metadata: serde_json::json!({}),
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }

    async fn list_trial_balance(
        &self, _scenario_id: Uuid, _entity_id: Option<Uuid>, _line_type: Option<&str>,
    ) -> AtlasResult<Vec<atlas_shared::ConsolidationTrialBalanceLine>> { Ok(vec![]) }
    async fn delete_trial_balance_by_scenario(&self, _scenario_id: Uuid) -> AtlasResult<()> { Ok(()) }

    // ── Elimination Rules ───────────────────────────────────────────
    async fn create_elimination_rule(
        &self, org_id: Uuid, ledger_id: Uuid, rule_code: &str,
        name: &str, description: Option<&str>, elimination_type: &str,
        from_entity_id: Option<Uuid>, to_entity_id: Option<Uuid>,
        from_account_pattern: Option<&str>, to_account_pattern: Option<&str>,
        offset_account_code: &str, priority: i32, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::ConsolidationEliminationRule> {
        Ok(atlas_shared::ConsolidationEliminationRule {
            id: Uuid::new_v4(), organization_id: org_id, ledger_id,
            rule_code: rule_code.to_string(), name: name.to_string(),
            description: description.map(String::from),
            elimination_type: elimination_type.to_string(),
            from_entity_id, to_entity_id,
            from_account_pattern: from_account_pattern.map(String::from),
            to_account_pattern: to_account_pattern.map(String::from),
            offset_account_code: offset_account_code.to_string(),
            priority, is_active: true,
            metadata: serde_json::json!({}),
            created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }

    async fn get_elimination_rule(&self, _ledger_id: Uuid, _rule_code: &str) -> AtlasResult<Option<atlas_shared::ConsolidationEliminationRule>> { Ok(None) }
    async fn list_elimination_rules(&self, _ledger_id: Uuid, _active_only: bool) -> AtlasResult<Vec<atlas_shared::ConsolidationEliminationRule>> { Ok(vec![]) }

    // ── Adjustments ─────────────────────────────────────────────────
    async fn create_adjustment(
        &self, org_id: Uuid, scenario_id: Uuid, adjustment_number: &str,
        description: Option<&str>, account_code: &str, account_name: Option<&str>,
        entity_id: Option<Uuid>, entity_code: Option<&str>,
        debit: &str, credit: &str, adjustment_type: &str,
        reference: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::ConsolidationAdjustment> {
        Ok(atlas_shared::ConsolidationAdjustment {
            id: Uuid::new_v4(), organization_id: org_id, scenario_id,
            adjustment_number: adjustment_number.to_string(),
            description: description.map(String::from),
            account_code: account_code.to_string(),
            account_name: account_name.map(String::from),
            entity_id, entity_code: entity_code.map(String::from),
            debit: debit.to_string(), credit: credit.to_string(),
            adjustment_type: adjustment_type.to_string(),
            reference: reference.map(String::from),
            status: "draft".to_string(),
            approved_by: None, approved_at: None,
            metadata: serde_json::json!({}),
            created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }

    async fn get_adjustment(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::ConsolidationAdjustment>> { Ok(None) }
    async fn list_adjustments(&self, _scenario_id: Uuid, _status: Option<&str>) -> AtlasResult<Vec<atlas_shared::ConsolidationAdjustment>> { Ok(vec![]) }
    async fn update_adjustment_status(
        &self, _id: Uuid, _status: &str, _approved_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::ConsolidationAdjustment> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }

    // ── Translation Rates ───────────────────────────────────────────
    async fn create_translation_rate(
        &self, org_id: Uuid, scenario_id: Uuid, entity_id: Uuid,
        from_currency: &str, to_currency: &str,
        rate_type: &str, exchange_rate: &str,
        effective_date: chrono::NaiveDate,
    ) -> AtlasResult<atlas_shared::ConsolidationTranslationRate> {
        Ok(atlas_shared::ConsolidationTranslationRate {
            id: Uuid::new_v4(), organization_id: org_id, scenario_id, entity_id,
            from_currency: from_currency.to_string(),
            to_currency: to_currency.to_string(),
            rate_type: rate_type.to_string(),
            exchange_rate: exchange_rate.to_string(),
            effective_date,
            metadata: serde_json::json!({}),
            created_at: chrono::Utc::now(),
        })
    }

    async fn get_translation_rate(
        &self, _scenario_id: Uuid, _entity_id: Uuid, _rate_type: &str,
    ) -> AtlasResult<Option<atlas_shared::ConsolidationTranslationRate>> { Ok(None) }
    async fn list_translation_rates(&self, _scenario_id: Uuid) -> AtlasResult<Vec<atlas_shared::ConsolidationTranslationRate>> { Ok(vec![]) }

    // ── Dashboard ───────────────────────────────────────────────────
    async fn get_dashboard_summary(&self, _org_id: Uuid) -> AtlasResult<atlas_shared::ConsolidationDashboardSummary> {
        Ok(atlas_shared::ConsolidationDashboardSummary {
            total_ledgers: 0,
            total_active_scenarios: 0,
            total_entities: 0,
            total_elimination_rules: 0,
            last_consolidation_date: None,
            last_consolidation_status: None,
            scenarios_by_status: serde_json::json!({}),
            entities_by_method: serde_json::json!({}),
            consolidation_completion_percent: "0".to_string(),
        })
    }
}

/// Mock supplier qualification repository for testing
pub struct MockSupplierQualificationRepository;

#[async_trait::async_trait]
impl crate::supplier_qualification::SupplierQualificationRepository for MockSupplierQualificationRepository {
    async fn create_area(
        &self, _org_id: Uuid, _area_code: &str, _name: &str, _description: Option<&str>,
        _area_type: &str, _scoring_model: &str, _passing_score: &str,
        _is_mandatory: bool, _renewal_period_days: i32, _created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::QualificationArea> {
        Ok(atlas_shared::QualificationArea {
            id: Uuid::new_v4(), organization_id: _org_id,
            area_code: _area_code.to_string(), name: _name.to_string(),
            description: None, area_type: _area_type.to_string(),
            scoring_model: _scoring_model.to_string(), passing_score: _passing_score.to_string(),
            is_mandatory: _is_mandatory, renewal_period_days: _renewal_period_days,
            is_active: true, metadata: serde_json::json!({}),
            created_by: None, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_area(&self, _org_id: Uuid, _code: &str) -> AtlasResult<Option<atlas_shared::QualificationArea>> { Ok(None) }
    async fn get_area_by_id(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::QualificationArea>> { Ok(None) }
    async fn list_areas(&self, _org_id: Uuid, _active_only: bool) -> AtlasResult<Vec<atlas_shared::QualificationArea>> { Ok(vec![]) }
    async fn delete_area(&self, _org_id: Uuid, _code: &str) -> AtlasResult<()> { Ok(()) }
    async fn create_question(
        &self, _org_id: Uuid, _area_id: Uuid, _question_number: i32, _question_text: &str,
        _description: Option<&str>, _response_type: &str, _choices: Option<serde_json::Value>,
        _is_required: bool, _weight: &str, _max_score: &str, _help_text: Option<&str>, _display_order: i32,
    ) -> AtlasResult<atlas_shared::QualificationQuestion> {
        Ok(atlas_shared::QualificationQuestion {
            id: Uuid::new_v4(), organization_id: _org_id, area_id: _area_id,
            question_number: _question_number, question_text: _question_text.to_string(),
            description: None, response_type: _response_type.to_string(), choices: None,
            is_required: _is_required, weight: _weight.to_string(), max_score: _max_score.to_string(),
            help_text: None, display_order: _display_order, is_active: true,
            metadata: serde_json::json!({}), created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn list_questions(&self, _area_id: Uuid) -> AtlasResult<Vec<atlas_shared::QualificationQuestion>> { Ok(vec![]) }
    async fn delete_question(&self, _id: Uuid) -> AtlasResult<()> { Ok(()) }
    async fn create_initiative(
        &self, _org_id: Uuid, _initiative_number: &str, _name: &str, _description: Option<&str>,
        _area_id: Uuid, _qualification_purpose: &str, _deadline: Option<chrono::NaiveDate>, _created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::SupplierQualificationInitiative> {
        Ok(atlas_shared::SupplierQualificationInitiative {
            id: Uuid::new_v4(), organization_id: _org_id, initiative_number: _initiative_number.to_string(),
            name: _name.to_string(), description: None, area_id: _area_id,
            qualification_purpose: _qualification_purpose.to_string(), status: "draft".to_string(),
            deadline: None, total_invited: 0, total_responded: 0, total_qualified: 0,
            total_disqualified: 0, total_pending: 0, completed_at: None,
            metadata: serde_json::json!({}), created_by: None, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_initiative(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::SupplierQualificationInitiative>> { Ok(None) }
    async fn list_initiatives(&self, _org_id: Uuid, _status: Option<&str>) -> AtlasResult<Vec<atlas_shared::SupplierQualificationInitiative>> { Ok(vec![]) }
    async fn update_initiative_status(&self, _id: Uuid, _status: &str) -> AtlasResult<atlas_shared::SupplierQualificationInitiative> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn update_initiative_counts(&self, _id: Uuid, _invited: i32, _responded: i32, _qualified: i32, _disqualified: i32, _pending: i32) -> AtlasResult<()> { Ok(()) }
    async fn create_invitation(
        &self, _org_id: Uuid, _initiative_id: Uuid, _supplier_id: Uuid, _supplier_name: &str,
        _supplier_contact_name: Option<&str>, _supplier_contact_email: Option<&str>,
        _expiry_date: Option<chrono::NaiveDate>, _created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::SupplierQualificationInvitation> {
        Ok(atlas_shared::SupplierQualificationInvitation {
            id: Uuid::new_v4(), organization_id: _org_id, initiative_id: _initiative_id,
            supplier_id: _supplier_id, supplier_name: _supplier_name.to_string(),
            supplier_contact_name: None, supplier_contact_email: None,
            status: "initiated".to_string(), invitation_date: None, response_date: None,
            evaluation_date: None, expiry_date: None, overall_score: "0".to_string(),
            max_possible_score: "0".to_string(), score_percentage: "0".to_string(),
            qualified_by: None, disqualified_reason: None, evaluation_notes: None,
            metadata: serde_json::json!({}), created_by: None, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_invitation(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::SupplierQualificationInvitation>> { Ok(None) }
    async fn list_invitations_by_initiative(&self, _initiative_id: Uuid) -> AtlasResult<Vec<atlas_shared::SupplierQualificationInvitation>> { Ok(vec![]) }
    async fn list_invitations_by_supplier(&self, _org_id: Uuid, _supplier_id: Uuid) -> AtlasResult<Vec<atlas_shared::SupplierQualificationInvitation>> { Ok(vec![]) }
    async fn update_invitation_status(&self, _id: Uuid, _status: &str, _response_date: Option<chrono::DateTime<chrono::Utc>>, _evaluation_date: Option<chrono::DateTime<chrono::Utc>>) -> AtlasResult<atlas_shared::SupplierQualificationInvitation> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn update_invitation_scores(&self, _id: Uuid, _overall_score: &str, _max_possible_score: &str, _score_percentage: &str, _qualified_by: Option<Uuid>, _disqualified_reason: Option<&str>, _evaluation_notes: Option<&str>) -> AtlasResult<atlas_shared::SupplierQualificationInvitation> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn create_response(
        &self, _org_id: Uuid, _invitation_id: Uuid, _question_id: Uuid, _response_text: Option<&str>,
        _response_value: Option<serde_json::Value>, _file_reference: Option<&str>,
    ) -> AtlasResult<atlas_shared::SupplierQualificationResponse> {
        Ok(atlas_shared::SupplierQualificationResponse {
            id: Uuid::new_v4(), organization_id: _org_id, invitation_id: _invitation_id,
            question_id: _question_id, response_text: _response_text.map(String::from),
            response_value: None, file_reference: None, score: "0".to_string(),
            max_score: "0".to_string(), evaluator_notes: None, evaluated_by: None,
            evaluated_at: None, metadata: serde_json::json!({}),
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_response(&self, _invitation_id: Uuid, _question_id: Uuid) -> AtlasResult<Option<atlas_shared::SupplierQualificationResponse>> { Ok(None) }
    async fn list_responses(&self, _invitation_id: Uuid) -> AtlasResult<Vec<atlas_shared::SupplierQualificationResponse>> { Ok(vec![]) }
    async fn score_response(&self, _id: Uuid, _score: &str, _evaluator_notes: Option<&str>, _evaluated_by: Option<Uuid>) -> AtlasResult<atlas_shared::SupplierQualificationResponse> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn create_certification(
        &self, _org_id: Uuid, _supplier_id: Uuid, _supplier_name: &str,
        _certification_type: &str, _certification_name: &str, _certifying_body: Option<&str>,
        _certificate_number: Option<&str>, _status: &str, _issued_date: Option<chrono::NaiveDate>,
        _expiry_date: Option<chrono::NaiveDate>, _renewal_date: Option<chrono::NaiveDate>,
        _qualification_invitation_id: Option<Uuid>, _document_reference: Option<&str>,
        _notes: Option<&str>, _created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::SupplierCertification> {
        Ok(atlas_shared::SupplierCertification {
            id: Uuid::new_v4(), organization_id: _org_id, supplier_id: _supplier_id,
            supplier_name: _supplier_name.to_string(), certification_type: _certification_type.to_string(),
            certification_name: _certification_name.to_string(), certifying_body: None,
            certificate_number: None, status: "active".to_string(), issued_date: None,
            expiry_date: None, renewal_date: None, qualification_invitation_id: None,
            document_reference: None, notes: None, metadata: serde_json::json!({}),
            created_by: None, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_certification(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::SupplierCertification>> { Ok(None) }
    async fn list_certifications(&self, _org_id: Uuid, _supplier_id: Option<Uuid>, _status: Option<&str>) -> AtlasResult<Vec<atlas_shared::SupplierCertification>> { Ok(vec![]) }
    async fn update_certification_status(&self, _id: Uuid, _status: &str) -> AtlasResult<atlas_shared::SupplierCertification> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn get_dashboard_summary(&self, _org_id: Uuid) -> AtlasResult<atlas_shared::SupplierQualificationDashboardSummary> {
        Ok(atlas_shared::SupplierQualificationDashboardSummary {
            total_active_areas: 0, total_active_initiatives: 0, total_suppliers_invited: 0,
            total_suppliers_qualified: 0, total_suppliers_pending: 0, total_suppliers_disqualified: 0,
            total_certifications_active: 0, total_certifications_expiring_30_days: 0,
            qualification_rate_percent: "0".to_string(), initiatives_by_status: serde_json::json!({}),
            certifications_by_type: serde_json::json!({}),
        })
    }
}

/// Mock Segregation of Duties repository for testing
pub struct MockSegregationOfDutiesRepository;

#[async_trait]
impl SegregationOfDutiesRepository for MockSegregationOfDutiesRepository {
    async fn create_rule(
        &self, org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        first_duties: Vec<String>, second_duties: Vec<String>,
        enforcement_mode: &str, risk_level: &str,
        effective_from: Option<chrono::NaiveDate>, effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::SodRule> {
        Ok(atlas_shared::SodRule {
            id: Uuid::new_v4(), organization_id: org_id,
            code: code.to_string(), name: name.to_string(), description: description.map(String::from),
            first_duties, second_duties,
            enforcement_mode: enforcement_mode.to_string(), risk_level: risk_level.to_string(),
            is_active: true, effective_from, effective_to,
            created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_rule(&self, _org_id: Uuid, _code: &str) -> AtlasResult<Option<atlas_shared::SodRule>> { Ok(None) }
    async fn get_rule_by_id(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::SodRule>> { Ok(None) }
    async fn list_rules(&self, _org_id: Uuid, _active_only: bool) -> AtlasResult<Vec<atlas_shared::SodRule>> { Ok(vec![]) }
    async fn update_rule_status(&self, _id: Uuid, _is_active: bool) -> AtlasResult<atlas_shared::SodRule> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn delete_rule(&self, _org_id: Uuid, _code: &str) -> AtlasResult<()> { Ok(()) }

    async fn create_role_assignment(
        &self, org_id: Uuid, user_id: Uuid, role_name: &str, duty_code: &str, assigned_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::SodRoleAssignment> {
        Ok(atlas_shared::SodRoleAssignment {
            id: Uuid::new_v4(), organization_id: org_id, user_id,
            role_name: role_name.to_string(), duty_code: duty_code.to_string(),
            assigned_by, assigned_at: chrono::Utc::now(), is_active: true,
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_role_assignments_for_user(&self, _org_id: Uuid, _user_id: Uuid) -> AtlasResult<Vec<atlas_shared::SodRoleAssignment>> { Ok(vec![]) }
    async fn list_role_assignments(&self, _org_id: Uuid, _user_id: Option<Uuid>) -> AtlasResult<Vec<atlas_shared::SodRoleAssignment>> { Ok(vec![]) }
    async fn deactivate_role_assignment(&self, _id: Uuid) -> AtlasResult<atlas_shared::SodRoleAssignment> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }

    async fn create_violation(
        &self, org_id: Uuid, rule_id: Uuid, rule_code: &str, user_id: Uuid,
        first_matched_duties: Vec<String>, second_matched_duties: Vec<String>,
    ) -> AtlasResult<atlas_shared::SodViolation> {
        Ok(atlas_shared::SodViolation {
            id: Uuid::new_v4(), organization_id: org_id,
            rule_id, rule_code: rule_code.to_string(), user_id,
            first_matched_duties, second_matched_duties,
            violation_status: "open".to_string(),
            detected_at: chrono::Utc::now(), resolved_at: None, resolved_by: None,
            metadata: serde_json::json!({}),
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_violation(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::SodViolation>> { Ok(None) }
    async fn list_violations(&self, _org_id: Uuid, _user_id: Option<Uuid>, _status: Option<&str>, _risk_level: Option<&str>) -> AtlasResult<Vec<atlas_shared::SodViolation>> { Ok(vec![]) }
    async fn update_violation_status(&self, _id: Uuid, _status: &str, _resolved_by: Option<Uuid>) -> AtlasResult<atlas_shared::SodViolation> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn find_existing_violation(&self, _rule_id: Uuid, _user_id: Uuid) -> AtlasResult<Option<atlas_shared::SodViolation>> { Ok(None) }

    async fn create_mitigating_control(
        &self, org_id: Uuid, violation_id: Uuid, control_name: &str, control_description: &str,
        control_owner_id: Option<Uuid>, review_frequency: &str,
        effective_from: Option<chrono::NaiveDate>, effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::SodMitigatingControl> {
        Ok(atlas_shared::SodMitigatingControl {
            id: Uuid::new_v4(), organization_id: org_id, violation_id,
            control_name: control_name.to_string(), control_description: control_description.to_string(),
            control_owner_id, review_frequency: review_frequency.to_string(),
            effective_from, effective_to, approved_by: None, approved_at: None,
            status: "pending_approval".to_string(),
            created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_mitigating_controls_for_violation(&self, _violation_id: Uuid) -> AtlasResult<Vec<atlas_shared::SodMitigatingControl>> { Ok(vec![]) }
    async fn list_mitigating_controls(&self, _org_id: Uuid) -> AtlasResult<Vec<atlas_shared::SodMitigatingControl>> { Ok(vec![]) }
    async fn approve_mitigating_control(&self, _id: Uuid, _approved_by: Uuid) -> AtlasResult<atlas_shared::SodMitigatingControl> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn revoke_mitigating_control(&self, _id: Uuid) -> AtlasResult<atlas_shared::SodMitigatingControl> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }

    async fn get_dashboard_summary(&self, _org_id: Uuid) -> AtlasResult<atlas_shared::SodDashboardSummary> {
        Ok(atlas_shared::SodDashboardSummary {
            total_rules: 0, active_rules: 0,
            total_violations: 0, open_violations: 0,
            mitigated_violations: 0, exception_violations: 0,
            violations_by_risk_level: serde_json::json!({}),
            recent_violations: vec![],
            rules_summary: serde_json::json!({}),
        })
    }
}

/// Mock AutoInvoice repository for testing
pub struct MockAutoInvoiceRepository;

#[async_trait]
impl AutoInvoiceRepository for MockAutoInvoiceRepository {
    async fn create_grouping_rule(
        &self, org_id: Uuid, name: &str, description: Option<&str>,
        transaction_types: serde_json::Value, group_by_fields: serde_json::Value,
        line_order_by: serde_json::Value, is_default: bool, priority: i32,
        created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::AutoInvoiceGroupingRule> {
        Ok(atlas_shared::AutoInvoiceGroupingRule {
            id: Uuid::new_v4(), organization_id: org_id,
            name: name.to_string(), description: description.map(String::from),
            transaction_types, group_by_fields, line_order_by,
            is_default, is_active: true, priority,
            created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_grouping_rule(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::AutoInvoiceGroupingRule>> { Ok(None) }
    async fn get_grouping_rule_by_name(&self, _org_id: Uuid, _name: &str) -> AtlasResult<Option<atlas_shared::AutoInvoiceGroupingRule>> { Ok(None) }
    async fn get_default_grouping_rule(&self, _org_id: Uuid) -> AtlasResult<Option<atlas_shared::AutoInvoiceGroupingRule>> { Ok(None) }
    async fn list_grouping_rules(&self, _org_id: Uuid) -> AtlasResult<Vec<atlas_shared::AutoInvoiceGroupingRule>> { Ok(vec![]) }
    async fn delete_grouping_rule(&self, _id: Uuid) -> AtlasResult<()> { Ok(()) }

    async fn create_validation_rule(
        &self, org_id: Uuid, name: &str, description: Option<&str>,
        field_name: &str, validation_type: &str, validation_expression: Option<&str>,
        error_message: &str, is_fatal: bool, transaction_types: serde_json::Value,
        priority: i32, effective_from: Option<chrono::NaiveDate>, effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::AutoInvoiceValidationRule> {
        Ok(atlas_shared::AutoInvoiceValidationRule {
            id: Uuid::new_v4(), organization_id: org_id,
            name: name.to_string(), description: description.map(String::from),
            field_name: field_name.to_string(), validation_type: validation_type.to_string(),
            validation_expression: validation_expression.map(String::from),
            error_message: error_message.to_string(), is_fatal, transaction_types,
            is_active: true, priority, effective_from, effective_to,
            created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_validation_rules(&self, _org_id: Uuid, _transaction_type: Option<&str>) -> AtlasResult<Vec<atlas_shared::AutoInvoiceValidationRule>> { Ok(vec![]) }
    async fn list_validation_rules(&self, _org_id: Uuid) -> AtlasResult<Vec<atlas_shared::AutoInvoiceValidationRule>> { Ok(vec![]) }
    async fn delete_validation_rule(&self, _id: Uuid) -> AtlasResult<()> { Ok(()) }

    async fn create_batch(
        &self, org_id: Uuid, batch_number: &str, batch_source: &str,
        description: Option<&str>, grouping_rule_id: Option<Uuid>, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::AutoInvoiceBatch> {
        Ok(atlas_shared::AutoInvoiceBatch {
            id: Uuid::new_v4(), organization_id: org_id,
            batch_number: batch_number.to_string(), batch_source: batch_source.to_string(),
            description: description.map(String::from),
            status: "pending".to_string(),
            total_lines: 0, valid_lines: 0, invalid_lines: 0,
            invoices_created: 0, invoices_total_amount: "0".to_string(),
            grouping_rule_id, validation_errors: serde_json::json!([]),
            started_at: None, completed_at: None,
            metadata: serde_json::json!({}),
            created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_batch(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::AutoInvoiceBatch>> { Ok(None) }
    async fn get_batch_by_number(&self, _org_id: Uuid, _batch_number: &str) -> AtlasResult<Option<atlas_shared::AutoInvoiceBatch>> { Ok(None) }
    async fn list_batches(&self, _org_id: Uuid, _status: Option<&str>) -> AtlasResult<Vec<atlas_shared::AutoInvoiceBatch>> { Ok(vec![]) }
    async fn update_batch_status(&self, _id: Uuid, _status: &str) -> AtlasResult<atlas_shared::AutoInvoiceBatch> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn update_batch_counts(
        &self, _id: Uuid, _total_lines: i32, _valid_lines: i32, _invalid_lines: i32,
        _invoices_created: i32, _invoices_total_amount: &str, _validation_errors: serde_json::Value,
    ) -> AtlasResult<()> { Ok(()) }

    async fn create_line(
        &self, org_id: Uuid, batch_id: Uuid, line_number: i32,
        source_line_id: Option<&str>, transaction_type: &str,
        customer_id: Option<Uuid>, customer_number: Option<&str>, customer_name: Option<&str>,
        bill_to_customer_id: Option<Uuid>, bill_to_site_id: Option<Uuid>,
        ship_to_customer_id: Option<Uuid>, ship_to_site_id: Option<Uuid>,
        item_code: Option<&str>, item_description: Option<&str>,
        quantity: Option<&str>, unit_of_measure: Option<&str>,
        unit_price: &str, line_amount: &str, currency_code: &str,
        exchange_rate: Option<&str>,
        transaction_date: chrono::NaiveDate, gl_date: chrono::NaiveDate, due_date: Option<chrono::NaiveDate>,
        revenue_account_code: Option<&str>, receivable_account_code: Option<&str>,
        tax_code: Option<&str>, tax_amount: Option<&str>,
        sales_rep_id: Option<Uuid>, sales_rep_name: Option<&str>,
        memo_line: Option<&str>, reference_number: Option<&str>,
        sales_order_number: Option<&str>, sales_order_line: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::AutoInvoiceLine> {
        Ok(atlas_shared::AutoInvoiceLine {
            id: Uuid::new_v4(), organization_id: org_id, batch_id, line_number,
            source_line_id: source_line_id.map(String::from),
            transaction_type: transaction_type.to_string(),
            customer_id, customer_number: customer_number.map(String::from),
            customer_name: customer_name.map(String::from),
            bill_to_customer_id, bill_to_site_id,
            ship_to_customer_id, ship_to_site_id,
            item_code: item_code.map(String::from), item_description: item_description.map(String::from),
            quantity: quantity.map(String::from), unit_of_measure: unit_of_measure.map(String::from),
            unit_price: unit_price.to_string(), line_amount: line_amount.to_string(),
            currency_code: currency_code.to_string(), exchange_rate: exchange_rate.map(String::from),
            transaction_date, gl_date, due_date,
            revenue_account_code: revenue_account_code.map(String::from),
            receivable_account_code: receivable_account_code.map(String::from),
            tax_code: tax_code.map(String::from), tax_amount: tax_amount.map(String::from),
            sales_rep_id, sales_rep_name: sales_rep_name.map(String::from),
            memo_line: memo_line.map(String::from), reference_number: reference_number.map(String::from),
            sales_order_number: sales_order_number.map(String::from),
            sales_order_line: sales_order_line.map(String::from),
            status: "pending".to_string(), validation_errors: serde_json::json!([]),
            invoice_id: None, invoice_line_number: None,
            metadata: serde_json::json!({}), created_by,
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_line(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::AutoInvoiceLine>> { Ok(None) }
    async fn list_lines_by_batch(&self, _batch_id: Uuid) -> AtlasResult<Vec<atlas_shared::AutoInvoiceLine>> { Ok(vec![]) }
    async fn list_lines_by_status(&self, _batch_id: Uuid, _status: &str) -> AtlasResult<Vec<atlas_shared::AutoInvoiceLine>> { Ok(vec![]) }
    async fn update_line_status(&self, _id: Uuid, _status: &str, _validation_errors: serde_json::Value) -> AtlasResult<()> { Ok(()) }
    async fn update_line_invoice(&self, _id: Uuid, _invoice_id: Uuid, _invoice_line_number: i32) -> AtlasResult<()> { Ok(()) }

    async fn create_result(
        &self, org_id: Uuid, batch_id: Uuid, invoice_number: &str,
        transaction_type: &str, customer_id: Option<Uuid>,
        bill_to_customer_id: Option<Uuid>, bill_to_site_id: Option<Uuid>,
        ship_to_customer_id: Option<Uuid>, ship_to_site_id: Option<Uuid>,
        currency_code: &str, exchange_rate: Option<&str>,
        transaction_date: chrono::NaiveDate, gl_date: chrono::NaiveDate,
        due_date: Option<chrono::NaiveDate>,
        receivable_account_code: Option<&str>, sales_rep_id: Option<Uuid>,
        sales_order_number: Option<&str>, reference_number: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::AutoInvoiceResult> {
        Ok(atlas_shared::AutoInvoiceResult {
            id: Uuid::new_v4(), organization_id: org_id, batch_id,
            invoice_number: invoice_number.to_string(),
            transaction_type: transaction_type.to_string(),
            customer_id, bill_to_customer_id, bill_to_site_id,
            ship_to_customer_id, ship_to_site_id,
            currency_code: currency_code.to_string(),
            exchange_rate: exchange_rate.map(String::from),
            transaction_date, gl_date, due_date,
            subtotal: "0".to_string(), tax_amount: "0".to_string(),
            total_amount: "0".to_string(), line_count: 0,
            receivable_account_code: receivable_account_code.map(String::from),
            sales_rep_id, sales_order_number: sales_order_number.map(String::from),
            reference_number: reference_number.map(String::from),
            status: "draft".to_string(), metadata: serde_json::json!({}),
            created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_result(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::AutoInvoiceResult>> { Ok(None) }
    async fn get_result_by_invoice_number(&self, _org_id: Uuid, _invoice_number: &str) -> AtlasResult<Option<atlas_shared::AutoInvoiceResult>> { Ok(None) }
    async fn list_results_by_batch(&self, _batch_id: Uuid) -> AtlasResult<Vec<atlas_shared::AutoInvoiceResult>> { Ok(vec![]) }
    async fn update_result_totals(&self, _id: Uuid, _subtotal: &str, _tax_amount: &str, _total_amount: &str, _line_count: i32) -> AtlasResult<()> { Ok(()) }
    async fn update_result_status(&self, _id: Uuid, _status: &str) -> AtlasResult<atlas_shared::AutoInvoiceResult> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }

    async fn create_result_line(
        &self, org_id: Uuid, invoice_id: Uuid, line_number: i32,
        source_line_id: Option<&str>, item_code: Option<&str>, item_description: Option<&str>,
        quantity: Option<&str>, unit_of_measure: Option<&str>,
        unit_price: &str, line_amount: &str, tax_code: Option<&str>, tax_amount: Option<&str>,
        revenue_account_code: Option<&str>, sales_order_number: Option<&str>, sales_order_line: Option<&str>,
    ) -> AtlasResult<atlas_shared::AutoInvoiceResultLine> {
        Ok(atlas_shared::AutoInvoiceResultLine {
            id: Uuid::new_v4(), organization_id: org_id, invoice_id, line_number,
            source_line_id: source_line_id.map(String::from),
            item_code: item_code.map(String::from), item_description: item_description.map(String::from),
            quantity: quantity.map(String::from), unit_of_measure: unit_of_measure.map(String::from),
            unit_price: unit_price.to_string(), line_amount: line_amount.to_string(),
            tax_code: tax_code.map(String::from), tax_amount: tax_amount.map(String::from),
            revenue_account_code: revenue_account_code.map(String::from),
            sales_order_number: sales_order_number.map(String::from),
            sales_order_line: sales_order_line.map(String::from),
            metadata: serde_json::json!({}), created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn list_result_lines(&self, _invoice_id: Uuid) -> AtlasResult<Vec<atlas_shared::AutoInvoiceResultLine>> { Ok(vec![]) }

    async fn get_summary(&self, _org_id: Uuid) -> AtlasResult<atlas_shared::AutoInvoiceSummary> {
        Ok(atlas_shared::AutoInvoiceSummary {
            total_batches: 0, pending_batches: 0, completed_batches: 0,
            failed_batches: 0, total_lines_imported: 0,
            total_invoices_created: 0, total_invoice_amount: "0".to_string(),
        })
    }
}

/// Mock Product Information repository for testing
pub struct MockProductInformationRepository;

#[async_trait]
impl ProductInformationRepository for MockProductInformationRepository {
    async fn create_item(
        &self, org_id: Uuid, item_number: &str, item_name: &str,
        description: Option<&str>, long_description: Option<&str>,
        item_type: &str, status: &str, lifecycle_phase: &str,
        primary_uom_code: &str, secondary_uom_code: Option<&str>,
        weight: Option<&str>, weight_uom: Option<&str>,
        volume: Option<&str>, volume_uom: Option<&str>,
        hazmat_flag: bool, lot_control_flag: bool, serial_control_flag: bool,
        shelf_life_days: Option<i32>,
        min_order_quantity: Option<&str>, max_order_quantity: Option<&str>,
        lead_time_days: Option<i32>,
        list_price: Option<&str>, cost_price: Option<&str>,
        currency_code: &str,
        inventory_item_flag: bool, purchasable_flag: bool,
        sellable_flag: bool, stock_enabled_flag: bool, invoice_enabled_flag: bool,
        default_buyer_id: Option<Uuid>, default_supplier_id: Option<Uuid>,
        template_id: Option<Uuid>, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::ProductItem> {
        Ok(atlas_shared::ProductItem {
            id: Uuid::new_v4(), organization_id: org_id,
            item_number: item_number.to_string(), item_name: item_name.to_string(),
            description: description.map(String::from),
            long_description: long_description.map(String::from),
            item_type: item_type.to_string(),
            status: status.to_string(), lifecycle_phase: lifecycle_phase.to_string(),
            primary_uom_code: primary_uom_code.to_string(),
            secondary_uom_code: secondary_uom_code.map(String::from),
            weight: weight.map(String::from), weight_uom: weight_uom.map(String::from),
            volume: volume.map(String::from), volume_uom: volume_uom.map(String::from),
            hazmat_flag, lot_control_flag, serial_control_flag, shelf_life_days,
            min_order_quantity: min_order_quantity.map(String::from),
            max_order_quantity: max_order_quantity.map(String::from),
            lead_time_days,
            list_price: list_price.map(String::from),
            cost_price: cost_price.map(String::from),
            currency_code: currency_code.to_string(),
            inventory_item_flag, purchasable_flag, sellable_flag,
            stock_enabled_flag, invoice_enabled_flag,
            default_buyer_id, default_supplier_id, template_id,
            thumbnail_url: None, image_url: None,
            metadata: serde_json::json!({}),
            created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_item(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::ProductItem>> { Ok(None) }
    async fn get_item_by_number(&self, _org_id: Uuid, _item_number: &str) -> AtlasResult<Option<atlas_shared::ProductItem>> { Ok(None) }
    async fn list_items(&self, _org_id: Uuid, _status: Option<&str>, _item_type: Option<&str>, _category_id: Option<Uuid>) -> AtlasResult<Vec<atlas_shared::ProductItem>> { Ok(vec![]) }
    async fn update_item_status(&self, id: Uuid, status: &str, lifecycle_phase: Option<&str>) -> AtlasResult<atlas_shared::ProductItem> {
        Ok(atlas_shared::ProductItem {
            id, organization_id: Uuid::new_v4(),
            item_number: "MOCK".to_string(), item_name: "Mock".to_string(),
            description: None, long_description: None,
            item_type: "finished_good".to_string(),
            status: status.to_string(),
            lifecycle_phase: lifecycle_phase.unwrap_or("concept").to_string(),
            primary_uom_code: "EA".to_string(), secondary_uom_code: None,
            weight: None, weight_uom: None, volume: None, volume_uom: None,
            hazmat_flag: false, lot_control_flag: false, serial_control_flag: false,
            shelf_life_days: None,
            min_order_quantity: None, max_order_quantity: None, lead_time_days: None,
            list_price: None, cost_price: None,
            currency_code: "USD".to_string(),
            inventory_item_flag: true, purchasable_flag: true, sellable_flag: true,
            stock_enabled_flag: true, invoice_enabled_flag: true,
            default_buyer_id: None, default_supplier_id: None, template_id: None,
            thumbnail_url: None, image_url: None,
            metadata: serde_json::json!({}),
            created_by: None, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn update_item_lifecycle(&self, id: Uuid, lifecycle_phase: &str) -> AtlasResult<atlas_shared::ProductItem> {
        Ok(atlas_shared::ProductItem {
            id, organization_id: Uuid::new_v4(),
            item_number: "MOCK".to_string(), item_name: "Mock".to_string(),
            description: None, long_description: None,
            item_type: "finished_good".to_string(),
            status: "draft".to_string(), lifecycle_phase: lifecycle_phase.to_string(),
            primary_uom_code: "EA".to_string(), secondary_uom_code: None,
            weight: None, weight_uom: None, volume: None, volume_uom: None,
            hazmat_flag: false, lot_control_flag: false, serial_control_flag: false,
            shelf_life_days: None,
            min_order_quantity: None, max_order_quantity: None, lead_time_days: None,
            list_price: None, cost_price: None,
            currency_code: "USD".to_string(),
            inventory_item_flag: true, purchasable_flag: true, sellable_flag: true,
            stock_enabled_flag: true, invoice_enabled_flag: true,
            default_buyer_id: None, default_supplier_id: None, template_id: None,
            thumbnail_url: None, image_url: None,
            metadata: serde_json::json!({}),
            created_by: None, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn delete_item(&self, _id: Uuid) -> AtlasResult<()> { Ok(()) }

    async fn create_category(
        &self, org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        parent_category_id: Option<Uuid>, level_number: i32, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::PimCategory> {
        Ok(atlas_shared::PimCategory {
            id: Uuid::new_v4(), organization_id: org_id,
            code: code.to_string(), name: name.to_string(),
            description: description.map(String::from),
            parent_category_id, level_number,
            item_count: 0, is_active: true,
            metadata: serde_json::json!({}),
            created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_category(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::PimCategory>> { Ok(None) }
    async fn get_category_by_code(&self, _org_id: Uuid, _code: &str) -> AtlasResult<Option<atlas_shared::PimCategory>> { Ok(None) }
    async fn list_categories(&self, _org_id: Uuid, _parent_id: Option<Uuid>) -> AtlasResult<Vec<atlas_shared::PimCategory>> { Ok(vec![]) }
    async fn delete_category(&self, _id: Uuid) -> AtlasResult<()> { Ok(()) }

    async fn assign_item_category(
        &self, org_id: Uuid, item_id: Uuid, category_id: Uuid, is_primary: bool, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::PimCategoryAssignment> {
        Ok(atlas_shared::PimCategoryAssignment {
            id: Uuid::new_v4(), organization_id: org_id,
            item_id, category_id, is_primary, created_by,
            created_at: chrono::Utc::now(),
        })
    }
    async fn get_primary_category_assignment(&self, _item_id: Uuid) -> AtlasResult<Option<atlas_shared::PimCategoryAssignment>> { Ok(None) }
    async fn list_item_categories(&self, _item_id: Uuid) -> AtlasResult<Vec<atlas_shared::PimCategoryAssignment>> { Ok(vec![]) }
    async fn remove_item_category(&self, _assignment_id: Uuid) -> AtlasResult<()> { Ok(()) }

    async fn create_cross_reference(
        &self, org_id: Uuid, item_id: Uuid, cross_reference_type: &str, cross_reference_value: &str,
        description: Option<&str>, source_system: Option<&str>,
        effective_from: Option<chrono::NaiveDate>, effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::PimCrossReference> {
        Ok(atlas_shared::PimCrossReference {
            id: Uuid::new_v4(), organization_id: org_id, item_id,
            cross_reference_type: cross_reference_type.to_string(),
            cross_reference_value: cross_reference_value.to_string(),
            description: description.map(String::from),
            source_system: source_system.map(String::from),
            effective_from, effective_to,
            is_active: true, metadata: serde_json::json!({}),
            created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_cross_reference_by_value(&self, _org_id: Uuid, _xref_type: &str, _value: &str) -> AtlasResult<Option<atlas_shared::PimCrossReference>> { Ok(None) }
    async fn list_cross_references(&self, _item_id: Uuid) -> AtlasResult<Vec<atlas_shared::PimCrossReference>> { Ok(vec![]) }
    async fn list_all_cross_references(&self, _org_id: Uuid, _xref_type: Option<&str>) -> AtlasResult<Vec<atlas_shared::PimCrossReference>> { Ok(vec![]) }
    async fn delete_cross_reference(&self, _id: Uuid) -> AtlasResult<()> { Ok(()) }

    async fn create_template(
        &self, org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        item_type: &str, default_uom_code: Option<&str>, default_category_id: Option<Uuid>,
        default_inventory_flag: bool, default_purchasable_flag: bool,
        default_sellable_flag: bool, default_stock_enabled_flag: bool,
        attribute_defaults: serde_json::Value, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::PimItemTemplate> {
        Ok(atlas_shared::PimItemTemplate {
            id: Uuid::new_v4(), organization_id: org_id,
            code: code.to_string(), name: name.to_string(),
            description: description.map(String::from),
            item_type: item_type.to_string(),
            default_uom_code: default_uom_code.map(String::from),
            default_category_id,
            default_inventory_flag, default_purchasable_flag,
            default_sellable_flag, default_stock_enabled_flag,
            attribute_defaults, is_active: true,
            metadata: serde_json::json!({}),
            created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_template(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::PimItemTemplate>> { Ok(None) }
    async fn get_template_by_code(&self, _org_id: Uuid, _code: &str) -> AtlasResult<Option<atlas_shared::PimItemTemplate>> { Ok(None) }
    async fn list_templates(&self, _org_id: Uuid) -> AtlasResult<Vec<atlas_shared::PimItemTemplate>> { Ok(vec![]) }
    async fn delete_template(&self, _id: Uuid) -> AtlasResult<()> { Ok(()) }

    async fn create_new_item_request(
        &self, org_id: Uuid, request_number: &str, title: &str, description: Option<&str>,
        item_type: &str, priority: &str, status: &str,
        requested_item_number: Option<&str>, requested_item_name: Option<&str>,
        requested_category_id: Option<Uuid>, justification: Option<&str>,
        target_launch_date: Option<chrono::NaiveDate>, estimated_cost: Option<&str>,
        currency_code: &str, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::PimNewItemRequest> {
        Ok(atlas_shared::PimNewItemRequest {
            id: Uuid::new_v4(), organization_id: org_id,
            request_number: request_number.to_string(),
            title: title.to_string(), description: description.map(String::from),
            item_type: item_type.to_string(), priority: priority.to_string(),
            status: status.to_string(),
            requested_item_number: requested_item_number.map(String::from),
            requested_item_name: requested_item_name.map(String::from),
            requested_category_id, justification: justification.map(String::from),
            target_launch_date, estimated_cost: estimated_cost.map(String::from),
            currency_code: currency_code.to_string(),
            requested_by: created_by, approved_by: None, approved_at: None,
            rejection_reason: None, implemented_item_id: None, implemented_at: None,
            metadata: serde_json::json!({}),
            created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_new_item_request(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::PimNewItemRequest>> { Ok(None) }
    async fn list_new_item_requests(&self, _org_id: Uuid, _status: Option<&str>) -> AtlasResult<Vec<atlas_shared::PimNewItemRequest>> { Ok(vec![]) }
    async fn update_nir_status(
        &self, id: Uuid, status: &str, approved_by: Option<Uuid>,
        approved_at: Option<chrono::DateTime<chrono::Utc>>, rejection_reason: Option<&str>,
    ) -> AtlasResult<atlas_shared::PimNewItemRequest> {
        Ok(atlas_shared::PimNewItemRequest {
            id, organization_id: Uuid::new_v4(),
            request_number: "MOCK".to_string(),
            title: "Mock".to_string(), description: None,
            item_type: "finished_good".to_string(), priority: "medium".to_string(),
            status: status.to_string(),
            requested_item_number: None, requested_item_name: None,
            requested_category_id: None, justification: None,
            target_launch_date: None, estimated_cost: None,
            currency_code: "USD".to_string(),
            requested_by: None, approved_by, approved_at,
            rejection_reason: rejection_reason.map(String::from),
            implemented_item_id: None, implemented_at: None,
            metadata: serde_json::json!({}),
            created_by: None, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn update_nir_implemented(&self, _id: Uuid, _implemented_item_id: Uuid, _implemented_at: Option<chrono::DateTime<chrono::Utc>>) -> AtlasResult<()> { Ok(()) }

    async fn get_dashboard(&self, _org_id: Uuid) -> AtlasResult<atlas_shared::PimDashboard> {
        Ok(atlas_shared::PimDashboard {
            total_items: 0, active_items: 0, draft_items: 0, obsolete_items: 0,
            total_categories: 0, pending_nir_count: 0, approved_nir_count: 0,
            cross_reference_count: 0, recently_created_items: 0,
            items_by_type: serde_json::json!({}),
        })
    }
}

/// Mock absence repository for testing
pub struct MockAbsenceRepository;

#[async_trait]
impl crate::absence::AbsenceRepository for MockAbsenceRepository {
    async fn create_absence_type(
        &self, _org_id: Uuid, _code: &str, _name: &str, _description: Option<&str>,
        _category: &str, _plan_type: &str, _requires_approval: bool,
        _requires_documentation: bool, _auto_approve_below_days: &str,
        _allow_negative_balance: bool, _allow_half_day: bool, _created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::AbsenceType> {
        Ok(atlas_shared::AbsenceType {
            id: Uuid::new_v4(), organization_id: _org_id, code: _code.to_string(),
            name: _name.to_string(), description: _description.map(|s| s.to_string()),
            category: _category.to_string(), plan_type: _plan_type.to_string(),
            requires_approval: _requires_approval, requires_documentation: _requires_documentation,
            auto_approve_below_days: _auto_approve_below_days.to_string(),
            allow_negative_balance: _allow_negative_balance, allow_half_day: _allow_half_day,
            is_active: true, metadata: serde_json::json!({}),
            created_by: _created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_absence_type(&self, _org_id: Uuid, _code: &str) -> AtlasResult<Option<atlas_shared::AbsenceType>> { Ok(None) }
    async fn list_absence_types(&self, _org_id: Uuid, _category: Option<&str>) -> AtlasResult<Vec<atlas_shared::AbsenceType>> { Ok(vec![]) }
    async fn delete_absence_type(&self, _org_id: Uuid, _code: &str) -> AtlasResult<()> { Ok(()) }
    async fn create_absence_plan(
        &self, _org_id: Uuid, _code: &str, _name: &str, _description: Option<&str>,
        _absence_type_id: Uuid, _accrual_frequency: &str, _accrual_rate: &str,
        _accrual_unit: &str, _carry_over_max: Option<String>, _carry_over_expiry_months: Option<i32>,
        _max_balance: Option<String>, _probation_period_days: i32, _prorate_first_year: bool,
        _created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::AbsencePlan> {
        Ok(atlas_shared::AbsencePlan {
            id: Uuid::new_v4(), organization_id: _org_id, code: _code.to_string(),
            name: _name.to_string(), description: _description.map(|s| s.to_string()),
            absence_type_id: _absence_type_id, accrual_frequency: _accrual_frequency.to_string(),
            accrual_rate: _accrual_rate.to_string(), accrual_unit: _accrual_unit.to_string(),
            carry_over_max: _carry_over_max, carry_over_expiry_months: _carry_over_expiry_months,
            max_balance: _max_balance, probation_period_days: _probation_period_days,
            prorate_first_year: _prorate_first_year, is_active: true,
            metadata: serde_json::json!({}), created_by: _created_by,
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_absence_plan(&self, _org_id: Uuid, _code: &str) -> AtlasResult<Option<atlas_shared::AbsencePlan>> { Ok(None) }
    async fn get_plan_by_id(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::AbsencePlan>> { Ok(None) }
    async fn list_absence_plans(&self, _org_id: Uuid, _absence_type_id: Option<Uuid>) -> AtlasResult<Vec<atlas_shared::AbsencePlan>> { Ok(vec![]) }
    async fn delete_absence_plan(&self, _org_id: Uuid, _code: &str) -> AtlasResult<()> { Ok(()) }
    async fn create_entry(
        &self, _org_id: Uuid, _employee_id: Uuid, _employee_name: Option<&str>,
        _absence_type_id: Uuid, _plan_id: Option<Uuid>, _entry_number: &str, _status: &str,
        _start_date: chrono::NaiveDate, _end_date: chrono::NaiveDate, _duration_days: &str,
        _duration_hours: Option<String>, _is_half_day: bool, _half_day_period: Option<&str>,
        _reason: Option<&str>, _comments: Option<&str>, _documentation_provided: bool,
        _approved_by: Option<&str>, _created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::AbsenceEntry> { todo!() }
    async fn get_entry(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::AbsenceEntry>> { Ok(None) }
    async fn list_entries(&self, _org_id: Uuid, _employee_id: Option<Uuid>, _absence_type_id: Option<Uuid>, _status: Option<&str>) -> AtlasResult<Vec<atlas_shared::AbsenceEntry>> { Ok(vec![]) }
    async fn update_entry_status(&self, _id: Uuid, _status: &str, _approved_by: Option<Uuid>, _rejected_reason: Option<&str>, _cancelled_reason: Option<&str>) -> AtlasResult<atlas_shared::AbsenceEntry> { todo!() }
    async fn find_overlapping_entries(&self, _org_id: Uuid, _employee_id: Uuid, _start_date: chrono::NaiveDate, _end_date: chrono::NaiveDate) -> AtlasResult<Vec<atlas_shared::AbsenceEntry>> { Ok(vec![]) }
    async fn create_balance(
        &self, _org_id: Uuid, _employee_id: Uuid, _plan_id: Uuid,
        _period_start: chrono::NaiveDate, _period_end: chrono::NaiveDate,
        _accrued: &str, _taken: &str, _adjusted: &str, _carried_over: &str, _remaining: &str,
    ) -> AtlasResult<atlas_shared::AbsenceBalance> { todo!() }
    async fn get_balance(&self, _employee_id: Uuid, _plan_id: Uuid, _period_start: chrono::NaiveDate, _period_end: chrono::NaiveDate) -> AtlasResult<Option<atlas_shared::AbsenceBalance>> { Ok(None) }
    async fn get_balance_for_previous_period(&self, _employee_id: Uuid, _plan_id: Uuid, _current_period_start: chrono::NaiveDate) -> AtlasResult<Option<atlas_shared::AbsenceBalance>> { Ok(None) }
    async fn list_balances(&self, _org_id: Uuid, _employee_id: Uuid) -> AtlasResult<Vec<atlas_shared::AbsenceBalance>> { Ok(vec![]) }
    async fn update_balance(&self, _id: Uuid, _taken: &str, _adjusted: &str, _remaining: &str) -> AtlasResult<()> { Ok(()) }
    async fn add_history(&self, _entry_id: Uuid, _action: &str, _from_status: Option<&str>, _to_status: Option<&str>, _performed_by: Option<Uuid>, _comment: Option<&str>) -> AtlasResult<()> { Ok(()) }
    async fn get_entry_history(&self, _entry_id: Uuid) -> AtlasResult<Vec<atlas_shared::AbsenceEntryHistory>> { Ok(vec![]) }
}

/// Mock Time and Labor repository for testing
pub struct MockTimeAndLaborRepository;

#[async_trait]
impl crate::time_and_labor::TimeAndLaborRepository for MockTimeAndLaborRepository {
    async fn create_work_schedule(
        &self, _org_id: Uuid, _code: &str, _name: &str, _description: Option<&str>,
        _schedule_type: &str, _standard_hours_per_day: &str, _standard_hours_per_week: &str,
        _work_days_per_week: i32, _start_time: Option<chrono::NaiveTime>,
        _end_time: Option<chrono::NaiveTime>, _break_duration_minutes: i32,
        _created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::WorkSchedule> {
        Ok(atlas_shared::WorkSchedule {
            id: Uuid::new_v4(), organization_id: _org_id,
            code: _code.to_string(), name: _name.to_string(), description: None,
            schedule_type: _schedule_type.to_string(),
            standard_hours_per_day: _standard_hours_per_day.to_string(),
            standard_hours_per_week: _standard_hours_per_week.to_string(),
            work_days_per_week: _work_days_per_week,
            start_time: None, end_time: None, break_duration_minutes: _break_duration_minutes,
            is_active: true, metadata: serde_json::json!({}),
            created_by: None, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_work_schedule(&self, _org_id: Uuid, _code: &str) -> AtlasResult<Option<atlas_shared::WorkSchedule>> { Ok(None) }
    async fn get_schedule_by_id(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::WorkSchedule>> { Ok(None) }
    async fn list_work_schedules(&self, _org_id: Uuid) -> AtlasResult<Vec<atlas_shared::WorkSchedule>> { Ok(vec![]) }
    async fn delete_work_schedule(&self, _org_id: Uuid, _code: &str) -> AtlasResult<()> { Ok(()) }

    async fn create_overtime_rule(
        &self, _org_id: Uuid, _code: &str, _name: &str, _description: Option<&str>,
        _threshold_type: &str, _daily_threshold_hours: &str, _weekly_threshold_hours: &str,
        _overtime_multiplier: &str, _double_time_threshold_hours: Option<&str>,
        _double_time_multiplier: &str, _include_holidays: bool, _include_weekends: bool,
        _effective_from: Option<chrono::NaiveDate>, _effective_to: Option<chrono::NaiveDate>,
        _created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::OvertimeRule> {
        Ok(atlas_shared::OvertimeRule {
            id: Uuid::new_v4(), organization_id: _org_id,
            code: _code.to_string(), name: _name.to_string(), description: None,
            threshold_type: _threshold_type.to_string(),
            daily_threshold_hours: _daily_threshold_hours.to_string(),
            weekly_threshold_hours: _weekly_threshold_hours.to_string(),
            overtime_multiplier: _overtime_multiplier.to_string(),
            double_time_threshold_hours: None,
            double_time_multiplier: _double_time_multiplier.to_string(),
            include_holidays: false, include_weekends: false,
            is_active: true, effective_from: None, effective_to: None,
            metadata: serde_json::json!({}),
            created_by: None, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_overtime_rule(&self, _org_id: Uuid, _code: &str) -> AtlasResult<Option<atlas_shared::OvertimeRule>> { Ok(None) }
    async fn get_overtime_rule_by_id(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::OvertimeRule>> { Ok(None) }
    async fn list_overtime_rules(&self, _org_id: Uuid) -> AtlasResult<Vec<atlas_shared::OvertimeRule>> { Ok(vec![]) }
    async fn delete_overtime_rule(&self, _org_id: Uuid, _code: &str) -> AtlasResult<()> { Ok(()) }

    async fn create_time_card(
        &self, _org_id: Uuid, _employee_id: Uuid, _employee_name: Option<&str>,
        _card_number: &str, _period_start: chrono::NaiveDate, _period_end: chrono::NaiveDate,
        _schedule_id: Option<Uuid>, _overtime_rule_id: Option<Uuid>, _created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::TimeCard> {
        Ok(atlas_shared::TimeCard {
            id: Uuid::new_v4(), organization_id: _org_id,
            employee_id: _employee_id, employee_name: _employee_name.map(String::from),
            card_number: _card_number.to_string(), status: "draft".to_string(),
            period_start: _period_start, period_end: _period_end,
            total_regular_hours: "0.0000".to_string(), total_overtime_hours: "0.0000".to_string(),
            total_double_time_hours: "0.0000".to_string(), total_hours: "0.0000".to_string(),
            schedule_id: None, overtime_rule_id: None,
            submitted_at: None, approved_by: None, approved_at: None,
            rejected_reason: None, comments: None,
            metadata: serde_json::json!({}), created_by: None,
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_time_card(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::TimeCard>> { Ok(None) }
    async fn get_time_card_by_number(&self, _org_id: Uuid, _card_number: &str) -> AtlasResult<Option<atlas_shared::TimeCard>> { Ok(None) }
    async fn list_time_cards(&self, _org_id: Uuid, _employee_id: Option<Uuid>, _status: Option<&str>) -> AtlasResult<Vec<atlas_shared::TimeCard>> { Ok(vec![]) }
    async fn update_time_card_status(&self, _id: Uuid, _status: &str, _approved_by: Option<Uuid>, _rejected_reason: Option<&str>) -> AtlasResult<atlas_shared::TimeCard> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn update_time_card_totals(&self, _id: Uuid, _regular: &str, _overtime: &str, _double_time: &str, _total: &str) -> AtlasResult<()> { Ok(()) }

    async fn create_time_entry(
        &self, _org_id: Uuid, _time_card_id: Uuid, _entry_date: chrono::NaiveDate,
        _entry_type: &str, _start_time: Option<chrono::NaiveTime>, _end_time: Option<chrono::NaiveTime>,
        _duration_hours: &str, _project_id: Option<Uuid>, _project_name: Option<&str>,
        _department_id: Option<Uuid>, _department_name: Option<&str>, _task_name: Option<&str>,
        _location: Option<&str>, _cost_center: Option<&str>, _labor_category: Option<&str>,
        _comments: Option<&str>, _created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::TimeEntry> {
        Ok(atlas_shared::TimeEntry {
            id: Uuid::new_v4(), organization_id: _org_id, time_card_id: _time_card_id,
            entry_date: _entry_date, entry_type: _entry_type.to_string(),
            start_time: None, end_time: None, duration_hours: _duration_hours.to_string(),
            project_id: None, project_name: None, department_id: None, department_name: None,
            task_name: None, location: None, cost_center: None, labor_category: None,
            comments: None, metadata: serde_json::json!({}), created_by: None,
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_time_entry(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::TimeEntry>> { Ok(None) }
    async fn list_time_entries_by_card(&self, _time_card_id: Uuid) -> AtlasResult<Vec<atlas_shared::TimeEntry>> { Ok(vec![]) }
    async fn delete_time_entry(&self, _id: Uuid) -> AtlasResult<()> { Ok(()) }
    async fn add_history(&self, _time_card_id: Uuid, _action: &str, _from_status: Option<&str>, _to_status: Option<&str>, _performed_by: Option<Uuid>, _comment: Option<&str>) -> AtlasResult<()> { Ok(()) }
    async fn get_time_card_history(&self, _time_card_id: Uuid) -> AtlasResult<Vec<atlas_shared::TimeCardHistory>> { Ok(vec![]) }
    async fn create_labor_distribution(
        &self, _org_id: Uuid, _time_entry_id: Uuid, _distribution_percent: &str,
        _cost_center: Option<&str>, _project_id: Option<Uuid>, _project_name: Option<&str>,
        _department_id: Option<Uuid>, _department_name: Option<&str>, _gl_account_code: Option<&str>,
        _allocated_hours: &str,
    ) -> AtlasResult<atlas_shared::LaborDistribution> {
        Ok(atlas_shared::LaborDistribution {
            id: Uuid::new_v4(), organization_id: _org_id, time_entry_id: _time_entry_id,
            distribution_percent: _distribution_percent.to_string(),
            cost_center: None, project_id: None, project_name: None,
            department_id: None, department_name: None, gl_account_code: None,
            allocated_hours: _allocated_hours.to_string(),
            metadata: serde_json::json!({}),
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn list_labor_distributions_by_entry(&self, _time_entry_id: Uuid) -> AtlasResult<Vec<atlas_shared::LaborDistribution>> { Ok(vec![]) }
    async fn delete_labor_distribution(&self, _id: Uuid) -> AtlasResult<()> { Ok(()) }
    async fn delete_labor_distribution_org_scoped(&self, _org_id: Uuid, _id: Uuid) -> AtlasResult<()> { Ok(()) }
    async fn get_labor_distribution(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::LaborDistribution>> { Ok(None) }
}

/// Mock payroll repository for testing
pub struct MockPayrollRepository;

#[async_trait]
impl crate::payroll::PayrollRepository for MockPayrollRepository {
    async fn create_payroll(
        &self, _org_id: Uuid, _name: &str, _description: Option<&str>,
        _pay_frequency: &str, _currency_code: &str,
        _salary_expense_account: Option<&str>, _liability_account: Option<&str>,
        _employer_tax_account: Option<&str>, _payment_account: Option<&str>,
        _created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::PayrollDefinition> {
        Ok(atlas_shared::PayrollDefinition {
            id: Uuid::new_v4(), organization_id: _org_id,
            name: _name.to_string(), description: None,
            pay_frequency: _pay_frequency.to_string(),
            currency_code: _currency_code.to_string(),
            salary_expense_account: None, liability_account: None,
            employer_tax_account: None, payment_account: None,
            is_active: true, created_by: None,
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            metadata: serde_json::json!({}),
        })
    }
    async fn get_payroll(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::PayrollDefinition>> { Ok(None) }
    async fn get_payroll_by_name(&self, _org_id: Uuid, _name: &str) -> AtlasResult<Option<atlas_shared::PayrollDefinition>> { Ok(None) }
    async fn list_payrolls(&self, _org_id: Uuid) -> AtlasResult<Vec<atlas_shared::PayrollDefinition>> { Ok(vec![]) }
    async fn delete_payroll(&self, _id: Uuid) -> AtlasResult<()> { Ok(()) }
    async fn create_element(
        &self, _org_id: Uuid, _code: &str, _name: &str, _description: Option<&str>,
        _element_type: &str, _category: &str, _calculation_method: &str,
        _default_value: Option<&str>, _is_recurring: bool,
        _has_employer_contribution: bool, _employer_contribution_rate: Option<&str>,
        _gl_account_code: Option<&str>, _is_pretax: bool,
        _effective_from: Option<chrono::NaiveDate>, _effective_to: Option<chrono::NaiveDate>,
        _created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::PayrollElement> {
        Ok(atlas_shared::PayrollElement {
            id: Uuid::new_v4(), organization_id: _org_id,
            code: _code.to_string(), name: _name.to_string(), description: None,
            element_type: _element_type.to_string(), category: _category.to_string(),
            calculation_method: _calculation_method.to_string(),
            default_value: _default_value.map(|v| v.to_string()),
            is_recurring: _is_recurring,
            has_employer_contribution: _has_employer_contribution,
            employer_contribution_rate: _employer_contribution_rate.map(|v| v.to_string()),
            gl_account_code: None, is_pretax: _is_pretax, is_active: true,
            effective_from: None, effective_to: None, created_by: None,
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            metadata: serde_json::json!({}),
        })
    }
    async fn get_element(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::PayrollElement>> { Ok(None) }
    async fn get_element_by_code(&self, _org_id: Uuid, _code: &str) -> AtlasResult<Option<atlas_shared::PayrollElement>> { Ok(None) }
    async fn list_elements(&self, _org_id: Uuid, _element_type: Option<&str>) -> AtlasResult<Vec<atlas_shared::PayrollElement>> { Ok(vec![]) }
    async fn delete_element(&self, _id: Uuid) -> AtlasResult<()> { Ok(()) }
    async fn create_entry(
        &self, _org_id: Uuid, _employee_id: Uuid, _element_id: Uuid,
        _element_code: &str, _element_name: &str, _element_type: &str,
        _entry_value: &str, _remaining_periods: Option<i32>,
        _effective_from: Option<chrono::NaiveDate>, _effective_to: Option<chrono::NaiveDate>,
        _created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::PayrollElementEntry> {
        Ok(atlas_shared::PayrollElementEntry {
            id: Uuid::new_v4(), organization_id: _org_id,
            employee_id: _employee_id, element_id: _element_id,
            element_code: _element_code.to_string(), element_name: _element_name.to_string(),
            element_type: _element_type.to_string(),
            entry_value: _entry_value.to_string(),
            remaining_periods: _remaining_periods, is_active: true,
            effective_from: None, effective_to: None, created_by: None,
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            metadata: serde_json::json!({}),
        })
    }
    async fn get_entries_by_employee(&self, _employee_id: Uuid) -> AtlasResult<Vec<atlas_shared::PayrollElementEntry>> { Ok(vec![]) }
    async fn delete_entry(&self, _id: Uuid) -> AtlasResult<()> { Ok(()) }
    async fn create_run(
        &self, _org_id: Uuid, _payroll_id: Uuid, _run_number: &str,
        _period_start: chrono::NaiveDate, _period_end: chrono::NaiveDate,
        _pay_date: chrono::NaiveDate, _created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::PayrollRun> {
        Ok(atlas_shared::PayrollRun {
            id: Uuid::new_v4(), organization_id: _org_id, payroll_id: _payroll_id,
            run_number: _run_number.to_string(), status: "open".to_string(),
            period_start: _period_start, period_end: _period_end, pay_date: _pay_date,
            total_gross: "0".to_string(), total_deductions: "0".to_string(),
            total_net: "0".to_string(), total_employer_cost: "0".to_string(),
            employee_count: 0, confirmed_by: None, confirmed_at: None,
            paid_by: None, paid_at: None, created_by: None,
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            metadata: serde_json::json!({}),
        })
    }
    async fn get_run(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::PayrollRun>> { Ok(None) }
    async fn get_run_by_number(&self, _org_id: Uuid, _run_number: &str) -> AtlasResult<Option<atlas_shared::PayrollRun>> { Ok(None) }
    async fn list_runs(&self, _org_id: Uuid, _status: Option<&str>) -> AtlasResult<Vec<atlas_shared::PayrollRun>> { Ok(vec![]) }
    async fn update_run_status(&self, _id: Uuid, _status: &str, _action_by: Option<Uuid>) -> AtlasResult<atlas_shared::PayrollRun> {
        Err(atlas_shared::AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn update_run_totals(&self, _id: Uuid, _total_gross: &str, _total_deductions: &str, _total_net: &str, _total_employer_cost: &str, _employee_count: i32) -> AtlasResult<()> { Ok(()) }
    async fn create_pay_slip(
        &self, _org_id: Uuid, _payroll_run_id: Uuid, _employee_id: Uuid,
        _employee_name: Option<&str>, _gross_earnings: &str, _total_deductions: &str,
        _net_pay: &str, _employer_cost: &str, _currency_code: &str,
        _payment_method: Option<&str>, _bank_account_last4: Option<&str>,
    ) -> AtlasResult<atlas_shared::PaySlip> {
        Ok(atlas_shared::PaySlip {
            id: Uuid::new_v4(), organization_id: _org_id,
            payroll_run_id: _payroll_run_id, employee_id: _employee_id,
            employee_name: _employee_name.map(|s| s.to_string()),
            gross_earnings: _gross_earnings.to_string(),
            total_deductions: _total_deductions.to_string(),
            net_pay: _net_pay.to_string(),
            employer_cost: _employer_cost.to_string(),
            currency_code: _currency_code.to_string(),
            payment_method: None, bank_account_last4: None,
            lines: vec![],
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            metadata: serde_json::json!({}),
        })
    }
    async fn get_pay_slip(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::PaySlip>> { Ok(None) }
    async fn list_pay_slips_by_run(&self, _payroll_run_id: Uuid) -> AtlasResult<Vec<atlas_shared::PaySlip>> { Ok(vec![]) }
    async fn list_pay_slips_by_employee(&self, _employee_id: Uuid) -> AtlasResult<Vec<atlas_shared::PaySlip>> { Ok(vec![]) }
    async fn create_pay_slip_line(
        &self, _pay_slip_id: Uuid, _element_code: &str, _element_name: &str,
        _element_type: &str, _category: &str, _hours_or_units: Option<&str>,
        _rate: Option<&str>, _amount: &str, _is_pretax: bool, _is_employer: bool,
        _gl_account_code: Option<&str>,
    ) -> AtlasResult<atlas_shared::PaySlipLine> {
        Ok(atlas_shared::PaySlipLine {
            id: Uuid::new_v4(), pay_slip_id: _pay_slip_id,
            element_code: _element_code.to_string(),
            element_name: _element_name.to_string(),
            element_type: _element_type.to_string(),
            category: _category.to_string(),
            hours_or_units: None, rate: None,
            amount: _amount.to_string(),
            is_pretax: _is_pretax, is_employer: _is_employer,
            gl_account_code: None,
            created_at: chrono::Utc::now(),
        })
    }
    async fn list_pay_slip_lines(&self, _pay_slip_id: Uuid) -> AtlasResult<Vec<atlas_shared::PaySlipLine>> { Ok(vec![]) }
    async fn get_dashboard(&self, _org_id: Uuid) -> AtlasResult<atlas_shared::PayrollDashboard> {
        Ok(atlas_shared::PayrollDashboard {
            total_gross: "0".to_string(), total_deductions: "0".to_string(),
            total_net: "0".to_string(), total_employer_cost: "0".to_string(),
            employee_count: 0, payroll_runs_this_period: 0,
            recent_runs: vec![],
            top_earnings_by_category: serde_json::json!({}),
            top_deductions_by_category: serde_json::json!({}),
        })
    }
}

/// Mock Joint Venture repository for testing
pub struct MockJointVentureRepository;

#[async_trait]
impl crate::joint_venture::JointVentureRepository for MockJointVentureRepository {
    async fn create_venture(
        &self, org_id: Uuid, venture_number: &str, name: &str, description: Option<&str>,
        operator_id: Option<Uuid>, operator_name: Option<&str>,
        currency_code: &str, start_date: Option<chrono::NaiveDate>,
        end_date: Option<chrono::NaiveDate>, accounting_method: &str,
        billing_cycle: &str, cost_cap_amount: Option<&str>,
        cost_cap_currency: Option<&str>,
        gl_revenue_account: Option<&str>, gl_cost_account: Option<&str>,
        gl_billing_account: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::JointVenture> {
        Ok(atlas_shared::JointVenture {
            id: Uuid::new_v4(), organization_id: org_id,
            venture_number: venture_number.to_string(),
            name: name.to_string(),
            description: description.map(String::from),
            status: "draft".to_string(),
            operator_id, operator_name: operator_name.map(String::from),
            currency_code: currency_code.to_string(),
            start_date, end_date,
            accounting_method: accounting_method.to_string(),
            billing_cycle: billing_cycle.to_string(),
            cost_cap_amount: cost_cap_amount.map(String::from),
            cost_cap_currency: cost_cap_currency.map(String::from),
            gl_revenue_account: gl_revenue_account.map(String::from),
            gl_cost_account: gl_cost_account.map(String::from),
            gl_billing_account: gl_billing_account.map(String::from),
            metadata: serde_json::json!({}),
            created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_venture(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::JointVenture>> { Ok(None) }
    async fn get_venture_by_number(&self, _org_id: Uuid, _venture_number: &str) -> AtlasResult<Option<atlas_shared::JointVenture>> { Ok(None) }
    async fn list_ventures(&self, _org_id: Uuid, _status: Option<&str>) -> AtlasResult<Vec<atlas_shared::JointVenture>> { Ok(vec![]) }
    async fn update_venture_status(&self, _id: Uuid, _status: &str) -> AtlasResult<atlas_shared::JointVenture> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }

    async fn create_partner(
        &self, org_id: Uuid, venture_id: Uuid, partner_id: Uuid, partner_name: &str,
        partner_type: &str, ownership_percentage: &str,
        revenue_interest_pct: Option<&str>, cost_bearing_pct: Option<&str>,
        role: &str, billing_contact: Option<&str>,
        billing_email: Option<&str>, billing_address: Option<&str>,
        effective_from: chrono::NaiveDate, effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::JointVenturePartner> {
        Ok(atlas_shared::JointVenturePartner {
            id: Uuid::new_v4(), organization_id: org_id, venture_id,
            partner_id, partner_name: partner_name.to_string(),
            partner_type: partner_type.to_string(),
            ownership_percentage: ownership_percentage.to_string(),
            revenue_interest_pct: revenue_interest_pct.map(String::from),
            cost_bearing_pct: cost_bearing_pct.map(String::from),
            role: role.to_string(),
            billing_contact: billing_contact.map(String::from),
            billing_email: billing_email.map(String::from),
            billing_address: billing_address.map(String::from),
            effective_from, effective_to,
            status: "active".to_string(),
            metadata: serde_json::json!({}),
            created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_partner(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::JointVenturePartner>> { Ok(None) }
    async fn list_partners_by_venture(&self, _venture_id: Uuid) -> AtlasResult<Vec<atlas_shared::JointVenturePartner>> { Ok(vec![]) }
    async fn list_active_partners(&self, _venture_id: Uuid, _on_date: chrono::NaiveDate) -> AtlasResult<Vec<atlas_shared::JointVenturePartner>> { Ok(vec![]) }
    async fn update_partner_status(&self, _id: Uuid, _status: &str) -> AtlasResult<atlas_shared::JointVenturePartner> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn delete_partner(&self, _id: Uuid) -> AtlasResult<()> { Ok(()) }

    async fn create_afe(
        &self, org_id: Uuid, venture_id: Uuid, afe_number: &str, title: &str,
        description: Option<&str>, estimated_cost: &str, currency_code: &str,
        cost_center: Option<&str>, work_area: Option<&str>, well_name: Option<&str>,
        effective_from: Option<chrono::NaiveDate>, effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::JointVentureAfe> {
        Ok(atlas_shared::JointVentureAfe {
            id: Uuid::new_v4(), organization_id: org_id, venture_id,
            afe_number: afe_number.to_string(), title: title.to_string(),
            description: description.map(String::from),
            status: "draft".to_string(),
            estimated_cost: estimated_cost.to_string(),
            actual_cost: "0".to_string(),
            committed_cost: "0".to_string(),
            remaining_budget: estimated_cost.to_string(),
            currency_code: currency_code.to_string(),
            cost_center: cost_center.map(String::from),
            work_area: work_area.map(String::from),
            well_name: well_name.map(String::from),
            requested_by: None, requested_at: None,
            approved_by: None, approved_at: None,
            rejected_reason: None,
            effective_from, effective_to,
            metadata: serde_json::json!({}),
            created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_afe(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::JointVentureAfe>> { Ok(None) }
    async fn get_afe_by_number(&self, _org_id: Uuid, _afe_number: &str) -> AtlasResult<Option<atlas_shared::JointVentureAfe>> { Ok(None) }
    async fn list_afes_by_venture(&self, _venture_id: Uuid, _status: Option<&str>) -> AtlasResult<Vec<atlas_shared::JointVentureAfe>> { Ok(vec![]) }
    async fn update_afe_status(
        &self, _id: Uuid, _status: &str, _approved_by: Option<Uuid>, _rejected_reason: Option<&str>,
    ) -> AtlasResult<atlas_shared::JointVentureAfe> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn update_afe_costs(&self, _id: Uuid, _actual: &str, _committed: &str, _remaining: &str) -> AtlasResult<()> { Ok(()) }

    async fn create_cost_distribution(
        &self, org_id: Uuid, venture_id: Uuid, distribution_number: &str,
        afe_id: Option<Uuid>, description: Option<&str>,
        total_amount: &str, currency_code: &str, cost_type: &str,
        distribution_date: chrono::NaiveDate,
        source_type: Option<&str>, source_id: Option<Uuid>, source_number: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::JvCostDistribution> {
        Ok(atlas_shared::JvCostDistribution {
            id: Uuid::new_v4(), organization_id: org_id, venture_id,
            distribution_number: distribution_number.to_string(),
            afe_id, description: description.map(String::from),
            status: "draft".to_string(),
            total_amount: total_amount.to_string(),
            currency_code: currency_code.to_string(),
            cost_type: cost_type.to_string(),
            distribution_date,
            gl_posting_date: None, gl_posted_at: None,
            source_type: source_type.map(String::from),
            source_id, source_number: source_number.map(String::from),
            metadata: serde_json::json!({}),
            created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_cost_distribution(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::JvCostDistribution>> { Ok(None) }
    async fn list_cost_distributions(&self, _venture_id: Uuid, _status: Option<&str>) -> AtlasResult<Vec<atlas_shared::JvCostDistribution>> { Ok(vec![]) }
    async fn update_cost_distribution_status(&self, _id: Uuid, _status: &str) -> AtlasResult<atlas_shared::JvCostDistribution> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn create_cost_distribution_line(
        &self, org_id: Uuid, distribution_id: Uuid, partner_id: Uuid, partner_name: Option<&str>,
        ownership_pct: &str, cost_bearing_pct: &str, distributed_amount: &str,
        gl_account_code: Option<&str>, line_description: Option<&str>,
    ) -> AtlasResult<atlas_shared::JvCostDistributionLine> {
        Ok(atlas_shared::JvCostDistributionLine {
            id: Uuid::new_v4(), organization_id: org_id, distribution_id,
            partner_id, partner_name: partner_name.map(String::from),
            ownership_pct: ownership_pct.to_string(),
            cost_bearing_pct: cost_bearing_pct.to_string(),
            distributed_amount: distributed_amount.to_string(),
            gl_account_code: gl_account_code.map(String::from),
            line_description: line_description.map(String::from),
            metadata: serde_json::json!({}),
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn list_cost_distribution_lines(&self, _distribution_id: Uuid) -> AtlasResult<Vec<atlas_shared::JvCostDistributionLine>> { Ok(vec![]) }

    async fn create_revenue_distribution(
        &self, org_id: Uuid, venture_id: Uuid, distribution_number: &str,
        description: Option<&str>, total_amount: &str, currency_code: &str,
        revenue_type: &str, distribution_date: chrono::NaiveDate,
        source_type: Option<&str>, source_id: Option<Uuid>, source_number: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::JvRevenueDistribution> {
        Ok(atlas_shared::JvRevenueDistribution {
            id: Uuid::new_v4(), organization_id: org_id, venture_id,
            distribution_number: distribution_number.to_string(),
            description: description.map(String::from),
            status: "draft".to_string(),
            total_amount: total_amount.to_string(),
            currency_code: currency_code.to_string(),
            revenue_type: revenue_type.to_string(),
            distribution_date,
            gl_posting_date: None, gl_posted_at: None,
            source_type: source_type.map(String::from),
            source_id, source_number: source_number.map(String::from),
            metadata: serde_json::json!({}),
            created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_revenue_distribution(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::JvRevenueDistribution>> { Ok(None) }
    async fn list_revenue_distributions(&self, _venture_id: Uuid, _status: Option<&str>) -> AtlasResult<Vec<atlas_shared::JvRevenueDistribution>> { Ok(vec![]) }
    async fn update_revenue_distribution_status(&self, _id: Uuid, _status: &str) -> AtlasResult<atlas_shared::JvRevenueDistribution> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn create_revenue_distribution_line(
        &self, org_id: Uuid, distribution_id: Uuid, partner_id: Uuid, partner_name: Option<&str>,
        revenue_interest_pct: &str, distributed_amount: &str,
        gl_account_code: Option<&str>, line_description: Option<&str>,
    ) -> AtlasResult<atlas_shared::JvRevenueDistributionLine> {
        Ok(atlas_shared::JvRevenueDistributionLine {
            id: Uuid::new_v4(), organization_id: org_id, distribution_id,
            partner_id, partner_name: partner_name.map(String::from),
            revenue_interest_pct: revenue_interest_pct.to_string(),
            distributed_amount: distributed_amount.to_string(),
            gl_account_code: gl_account_code.map(String::from),
            line_description: line_description.map(String::from),
            metadata: serde_json::json!({}),
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn list_revenue_distribution_lines(&self, _distribution_id: Uuid) -> AtlasResult<Vec<atlas_shared::JvRevenueDistributionLine>> { Ok(vec![]) }

    async fn create_billing(
        &self, org_id: Uuid, venture_id: Uuid, billing_number: &str,
        partner_id: Uuid, partner_name: Option<&str>, billing_type: &str,
        total_amount: &str, tax_amount: &str, total_with_tax: &str,
        currency_code: &str,
        billing_period_start: chrono::NaiveDate, billing_period_end: chrono::NaiveDate,
        due_date: Option<chrono::NaiveDate>, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::JvBilling> {
        Ok(atlas_shared::JvBilling {
            id: Uuid::new_v4(), organization_id: org_id, venture_id,
            billing_number: billing_number.to_string(),
            partner_id, partner_name: partner_name.map(String::from),
            billing_type: billing_type.to_string(),
            status: "draft".to_string(),
            total_amount: total_amount.to_string(),
            tax_amount: tax_amount.to_string(),
            total_with_tax: total_with_tax.to_string(),
            currency_code: currency_code.to_string(),
            billing_period_start, billing_period_end, due_date,
            approved_by: None, approved_at: None,
            paid_at: None, payment_reference: None, dispute_reason: None,
            metadata: serde_json::json!({}),
            created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_billing(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::JvBilling>> { Ok(None) }
    async fn get_billing_by_number(&self, _org_id: Uuid, _billing_number: &str) -> AtlasResult<Option<atlas_shared::JvBilling>> { Ok(None) }
    async fn list_billings(&self, _venture_id: Uuid, _status: Option<&str>, _billing_type: Option<&str>) -> AtlasResult<Vec<atlas_shared::JvBilling>> { Ok(vec![]) }
    async fn update_billing_status(
        &self, _id: Uuid, _status: &str, _approved_by: Option<Uuid>,
        _payment_reference: Option<&str>, _dispute_reason: Option<&str>,
    ) -> AtlasResult<atlas_shared::JvBilling> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn create_billing_line(
        &self, org_id: Uuid, billing_id: Uuid, line_number: i32,
        cost_distribution_id: Option<Uuid>, revenue_distribution_id: Option<Uuid>,
        description: Option<&str>, cost_type: Option<&str>, amount: &str,
        ownership_pct: Option<&str>,
    ) -> AtlasResult<atlas_shared::JvBillingLine> {
        Ok(atlas_shared::JvBillingLine {
            id: Uuid::new_v4(), organization_id: org_id, billing_id,
            line_number,
            cost_distribution_id, revenue_distribution_id,
            description: description.map(String::from),
            cost_type: cost_type.map(String::from),
            amount: amount.to_string(),
            ownership_pct: ownership_pct.map(String::from),
            metadata: serde_json::json!({}),
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn list_billing_lines(&self, _billing_id: Uuid) -> AtlasResult<Vec<atlas_shared::JvBillingLine>> { Ok(vec![]) }

    async fn get_dashboard(&self, _org_id: Uuid) -> AtlasResult<atlas_shared::JvDashboard> {
        Ok(atlas_shared::JvDashboard {
            total_ventures: 0,
            active_ventures: 0,
            total_partners: 0,
            total_cost_distributed: "0".to_string(),
            total_revenue_distributed: "0".to_string(),
            total_billed: "0".to_string(),
            total_collected: "0".to_string(),
            outstanding_balance: "0.00".to_string(),
            pending_afes: 0,
            ventures_by_status: serde_json::json!({}),
        })
    }
}

/// Mock Sustainability repository for testing
pub struct MockSustainabilityRepository;

#[async_trait]
impl crate::sustainability::SustainabilityRepository for MockSustainabilityRepository {
    async fn create_facility(
        &self, org_id: Uuid, facility_code: &str, name: &str, description: Option<&str>,
        country_code: Option<&str>, region: Option<&str>, city: Option<&str>,
        address: Option<&str>, latitude: Option<f64>, longitude: Option<f64>,
        facility_type: &str, industry_sector: Option<&str>,
        total_area_sqm: Option<f64>, employee_count: Option<i32>,
        operating_hours_per_year: i32, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::SustainabilityFacility> {
        Ok(atlas_shared::SustainabilityFacility {
            id: Uuid::new_v4(), organization_id: org_id,
            facility_code: facility_code.to_string(), name: name.to_string(),
            description: description.map(String::from),
            country_code: country_code.map(String::from),
            region: region.map(String::from),
            city: city.map(String::from),
            address: address.map(String::from),
            latitude, longitude,
            facility_type: facility_type.to_string(),
            industry_sector: industry_sector.map(String::from),
            total_area_sqm, employee_count,
            operating_hours_per_year,
            status: "active".to_string(),
            metadata: serde_json::json!({}),
            created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_facility(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::SustainabilityFacility>> { Ok(None) }
    async fn get_facility_by_code(&self, _org_id: Uuid, _code: &str) -> AtlasResult<Option<atlas_shared::SustainabilityFacility>> { Ok(None) }
    async fn list_facilities(&self, _org_id: Uuid, _status: Option<&str>, _facility_type: Option<&str>) -> AtlasResult<Vec<atlas_shared::SustainabilityFacility>> { Ok(vec![]) }
    async fn update_facility_status(&self, _id: Uuid, _status: &str) -> AtlasResult<atlas_shared::SustainabilityFacility> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn delete_facility(&self, _org_id: Uuid, _code: &str) -> AtlasResult<()> { Ok(()) }

    async fn create_emission_factor(
        &self, org_id: Uuid, factor_code: &str, name: &str, description: Option<&str>,
        scope: &str, category: &str, activity_type: &str, factor_value: f64,
        unit_of_measure: &str, gas_type: &str, factor_source: Option<&str>,
        effective_from: chrono::NaiveDate, effective_to: Option<chrono::NaiveDate>,
        region_code: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::EmissionFactor> {
        Ok(atlas_shared::EmissionFactor {
            id: Uuid::new_v4(), organization_id: org_id,
            factor_code: factor_code.to_string(), name: name.to_string(),
            description: description.map(String::from),
            scope: scope.to_string(), category: category.to_string(),
            activity_type: activity_type.to_string(), factor_value,
            unit_of_measure: unit_of_measure.to_string(),
            gas_type: gas_type.to_string(),
            factor_source: factor_source.map(String::from),
            effective_from, effective_to, region_code: region_code.map(String::from),
            status: "active".to_string(),
            metadata: serde_json::json!({}),
            created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_emission_factor(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::EmissionFactor>> { Ok(None) }
    async fn get_emission_factor_by_code(&self, _org_id: Uuid, _code: &str) -> AtlasResult<Option<atlas_shared::EmissionFactor>> { Ok(None) }
    async fn list_emission_factors(&self, _org_id: Uuid, _scope: Option<&str>, _category: Option<&str>, _activity_type: Option<&str>) -> AtlasResult<Vec<atlas_shared::EmissionFactor>> { Ok(vec![]) }
    async fn delete_emission_factor(&self, _org_id: Uuid, _code: &str) -> AtlasResult<()> { Ok(()) }

    async fn create_activity(
        &self, org_id: Uuid, activity_number: &str,
        facility_id: Option<Uuid>, facility_code: Option<&str>,
        activity_type: &str, scope: &str, category: Option<&str>,
        quantity: f64, unit_of_measure: &str,
        emission_factor_id: Option<Uuid>,
        co2e_kg: f64, co2_kg: Option<f64>, ch4_kg: Option<f64>, n2o_kg: Option<f64>,
        cost_amount: Option<f64>, cost_currency: Option<&str>,
        activity_date: chrono::NaiveDate, reporting_period: Option<&str>,
        source_type: Option<&str>, source_reference: Option<&str>,
        department_id: Option<Uuid>, project_id: Option<Uuid>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::EnvironmentalActivity> {
        Ok(atlas_shared::EnvironmentalActivity {
            id: Uuid::new_v4(), organization_id: org_id,
            activity_number: activity_number.to_string(),
            facility_id, facility_code: facility_code.map(String::from),
            activity_type: activity_type.to_string(), scope: scope.to_string(),
            category: category.map(String::from),
            quantity, unit_of_measure: unit_of_measure.to_string(),
            emission_factor_id, co2e_kg, co2_kg, ch4_kg, n2o_kg,
            cost_amount, cost_currency: cost_currency.map(String::from),
            activity_date, reporting_period: reporting_period.map(String::from),
            source_type: source_type.map(String::from), source_reference: source_reference.map(String::from),
            department_id, project_id,
            status: "confirmed".to_string(),
            verified_by: None, verified_at: None,
            notes: None,
            metadata: serde_json::json!({}),
            created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_activity(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::EnvironmentalActivity>> { Ok(None) }
    async fn get_activity_by_number(&self, _org_id: Uuid, _number: &str) -> AtlasResult<Option<atlas_shared::EnvironmentalActivity>> { Ok(None) }
    async fn list_activities(&self, _org_id: Uuid, _scope: Option<&str>, _facility_id: Option<&Uuid>, _activity_type: Option<&str>, _period: Option<&str>) -> AtlasResult<Vec<atlas_shared::EnvironmentalActivity>> { Ok(vec![]) }
    async fn update_activity_status(&self, _id: Uuid, _status: &str) -> AtlasResult<atlas_shared::EnvironmentalActivity> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn delete_activity(&self, _org_id: Uuid, _number: &str) -> AtlasResult<()> { Ok(()) }

    async fn create_metric(
        &self, org_id: Uuid, metric_code: &str, name: &str, description: Option<&str>,
        pillar: &str, category: &str, unit_of_measure: &str,
        gri_standard: Option<&str>, sasb_standard: Option<&str>,
        tcfd_category: Option<&str>, eu_taxonomy_code: Option<&str>,
        target_value: Option<f64>, warning_threshold: Option<f64>, direction: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::EsgMetric> {
        Ok(atlas_shared::EsgMetric {
            id: Uuid::new_v4(), organization_id: org_id,
            metric_code: metric_code.to_string(), name: name.to_string(),
            description: description.map(String::from),
            pillar: pillar.to_string(), category: category.to_string(),
            unit_of_measure: unit_of_measure.to_string(),
            gri_standard: gri_standard.map(String::from),
            sasb_standard: sasb_standard.map(String::from),
            tcfd_category: tcfd_category.map(String::from),
            eu_taxonomy_code: eu_taxonomy_code.map(String::from),
            target_value, warning_threshold,
            direction: direction.to_string(),
            status: "active".to_string(),
            metadata: serde_json::json!({}),
            created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_metric(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::EsgMetric>> { Ok(None) }
    async fn get_metric_by_code(&self, _org_id: Uuid, _code: &str) -> AtlasResult<Option<atlas_shared::EsgMetric>> { Ok(None) }
    async fn list_metrics(&self, _org_id: Uuid, _pillar: Option<&str>, _category: Option<&str>) -> AtlasResult<Vec<atlas_shared::EsgMetric>> { Ok(vec![]) }
    async fn delete_metric(&self, _org_id: Uuid, _code: &str) -> AtlasResult<()> { Ok(()) }

    async fn create_metric_reading(
        &self, org_id: Uuid, metric_id: Uuid, metric_value: f64,
        reading_date: chrono::NaiveDate, reporting_period: Option<&str>,
        facility_id: Option<Uuid>, notes: Option<&str>, source: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::EsgMetricReading> {
        Ok(atlas_shared::EsgMetricReading {
            id: Uuid::new_v4(), organization_id: org_id, metric_id, metric_value,
            reading_date, reporting_period: reporting_period.map(String::from),
            facility_id, notes: notes.map(String::from), source: source.map(String::from),
            verified_by: None, verified_at: None,
            status: "confirmed".to_string(),
            metadata: serde_json::json!({}),
            created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_metric_reading(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::EsgMetricReading>> { Ok(None) }
    async fn list_metric_readings(&self, _metric_id: Uuid, _from: Option<chrono::NaiveDate>, _to: Option<chrono::NaiveDate>) -> AtlasResult<Vec<atlas_shared::EsgMetricReading>> { Ok(vec![]) }
    async fn delete_metric_reading(&self, _id: Uuid) -> AtlasResult<()> { Ok(()) }

    async fn create_goal(
        &self, org_id: Uuid, goal_code: &str, name: &str, description: Option<&str>,
        goal_type: &str, scope: Option<&str>,
        baseline_value: f64, baseline_year: i32, baseline_unit: &str,
        target_value: f64, target_year: i32, target_unit: &str,
        target_reduction_pct: Option<f64>, milestones: serde_json::Value,
        progress_pct: f64,
        facility_id: Option<Uuid>, owner_id: Option<Uuid>, owner_name: Option<&str>,
        framework: Option<&str>, framework_reference: Option<&str>,
        effective_from: Option<chrono::NaiveDate>, effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::SustainabilityGoal> {
        Ok(atlas_shared::SustainabilityGoal {
            id: Uuid::new_v4(), organization_id: org_id,
            goal_code: goal_code.to_string(), name: name.to_string(),
            description: description.map(String::from),
            goal_type: goal_type.to_string(), scope: scope.map(String::from),
            baseline_value, baseline_year, baseline_unit: baseline_unit.to_string(),
            target_value, target_year, target_unit: target_unit.to_string(),
            target_reduction_pct, milestones,
            current_value: baseline_value, progress_pct,
            facility_id, status: "on_track".to_string(),
            owner_id, owner_name: owner_name.map(String::from),
            framework: framework.map(String::from),
            framework_reference: framework_reference.map(String::from),
            effective_from, effective_to,
            metadata: serde_json::json!({}),
            created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_goal(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::SustainabilityGoal>> { Ok(None) }
    async fn get_goal_by_code(&self, _org_id: Uuid, _code: &str) -> AtlasResult<Option<atlas_shared::SustainabilityGoal>> { Ok(None) }
    async fn list_goals(&self, _org_id: Uuid, _goal_type: Option<&str>, _status: Option<&str>) -> AtlasResult<Vec<atlas_shared::SustainabilityGoal>> { Ok(vec![]) }
    async fn update_goal_progress(&self, _id: Uuid, _current_value: f64) -> AtlasResult<atlas_shared::SustainabilityGoal> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn update_goal_status(&self, _id: Uuid, _status: &str) -> AtlasResult<atlas_shared::SustainabilityGoal> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn delete_goal(&self, _org_id: Uuid, _code: &str) -> AtlasResult<()> { Ok(()) }

    async fn create_carbon_offset(
        &self, org_id: Uuid, offset_number: &str, name: &str, description: Option<&str>,
        project_name: &str, project_type: &str, project_location: Option<&str>,
        registry: Option<&str>, registry_id: Option<&str>, certification_standard: Option<&str>,
        quantity_tonnes: f64, remaining_tonnes: f64,
        unit_price: Option<f64>, total_cost: Option<f64>, currency_code: Option<&str>,
        vintage_year: i32, effective_from: chrono::NaiveDate, effective_to: Option<chrono::NaiveDate>,
        supplier_name: Option<&str>, supplier_id: Option<Uuid>, notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::CarbonOffset> {
        Ok(atlas_shared::CarbonOffset {
            id: Uuid::new_v4(), organization_id: org_id,
            offset_number: offset_number.to_string(), name: name.to_string(),
            description: description.map(String::from),
            project_name: project_name.to_string(),
            project_type: project_type.to_string(),
            project_location: project_location.map(String::from),
            registry: registry.map(String::from),
            registry_id: registry_id.map(String::from),
            certification_standard: certification_standard.map(String::from),
            quantity_tonnes, remaining_tonnes,
            unit_price, total_cost, currency_code: currency_code.map(String::from),
            vintage_year, retired_quantity: 0.0, retired_date: None,
            effective_from, effective_to,
            status: "active".to_string(),
            supplier_name: supplier_name.map(String::from), supplier_id, notes: notes.map(String::from),
            metadata: serde_json::json!({}),
            created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_carbon_offset(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::CarbonOffset>> { Ok(None) }
    async fn get_offset_by_number(&self, _org_id: Uuid, _number: &str) -> AtlasResult<Option<atlas_shared::CarbonOffset>> { Ok(None) }
    async fn list_carbon_offsets(&self, _org_id: Uuid, _status: Option<&str>, _project_type: Option<&str>) -> AtlasResult<Vec<atlas_shared::CarbonOffset>> { Ok(vec![]) }
    async fn retire_carbon_offset(&self, _id: Uuid, _qty: f64) -> AtlasResult<atlas_shared::CarbonOffset> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn delete_carbon_offset(&self, _org_id: Uuid, _number: &str) -> AtlasResult<()> { Ok(()) }

    async fn get_dashboard(&self, _org_id: Uuid) -> AtlasResult<atlas_shared::SustainabilityDashboard> {
        Ok(atlas_shared::SustainabilityDashboard {
            total_facilities: 0, active_facilities: 0,
            total_emissions_co2e_tonnes: 0.0,
            scope1_emissions_tonnes: 0.0, scope2_emissions_tonnes: 0.0, scope3_emissions_tonnes: 0.0,
            total_energy_consumed_kwh: 0.0, renewable_energy_pct: 0.0,
            total_water_consumed_cubic_m: 0.0,
            total_waste_generated_tonnes: 0.0, waste_diverted_pct: 0.0,
            total_offsets_tonnes: 0.0, net_emissions_tonnes: 0.0,
            active_goals: 0, goals_on_track: 0, goals_achieved: 0,
            esg_metrics_count: 0,
            emissions_by_scope: serde_json::json!({}),
            emissions_by_category: serde_json::json!({}),
            emissions_trend: serde_json::json!({}),
            goals_by_status: serde_json::json!({}),
        })
    }
}

// ============================================================================
// Engineering Change Management Mock Repository
// ============================================================================

use crate::engineering_change_management::EngineeringChangeManagementRepository;

/// Mock Engineering Change Management repository
pub struct MockEngineeringChangeManagementRepository;

#[async_trait]
impl EngineeringChangeManagementRepository for MockEngineeringChangeManagementRepository {
    async fn create_change_type(
        &self, org_id: Uuid, type_code: &str, name: &str, description: Option<&str>,
        category: &str, approval_required: bool, default_priority: &str,
        number_prefix: &str, description_template: Option<&str>,
        statuses: serde_json::Value, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::EngineeringChangeType> {
        Ok(atlas_shared::EngineeringChangeType {
            id: Uuid::new_v4(), organization_id: org_id,
            type_code: type_code.to_string(), name: name.to_string(),
            description: description.map(String::from),
            category: category.to_string(),
            approval_required, default_priority: default_priority.to_string(),
            number_prefix: number_prefix.to_string(),
            description_template: description_template.map(String::from),
            status: "active".to_string(),
            statuses, metadata: serde_json::json!({}),
            created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_change_type(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::EngineeringChangeType>> { Ok(None) }
    async fn get_change_type_by_code(&self, _org_id: Uuid, _code: &str) -> AtlasResult<Option<atlas_shared::EngineeringChangeType>> { Ok(None) }
    async fn list_change_types(&self, _org_id: Uuid, _category: Option<&str>) -> AtlasResult<Vec<atlas_shared::EngineeringChangeType>> { Ok(vec![]) }
    async fn delete_change_type(&self, _org_id: Uuid, _code: &str) -> AtlasResult<()> { Ok(()) }

    async fn create_change(
        &self, org_id: Uuid, change_number: &str, change_type_id: Option<Uuid>,
        category: &str, title: &str, description: Option<&str>,
        change_reason: Option<&str>, change_reason_description: Option<&str>,
        priority: &str, status: &str, revision: &str,
        assigned_to: Option<Uuid>, assigned_to_name: Option<&str>,
        submitted_at: Option<chrono::DateTime<chrono::Utc>>,
        approved_at: Option<chrono::DateTime<chrono::Utc>>,
        implemented_at: Option<chrono::DateTime<chrono::Utc>>,
        target_date: Option<chrono::NaiveDate>,
        effective_date: Option<chrono::NaiveDate>,
        resolution_code: Option<&str>,
        resolution_notes: Option<&str>,
        parent_change_id: Option<Uuid>,
        superseded_by_id: Option<Uuid>,
        impact_analysis: serde_json::Value,
        estimated_cost: Option<f64>,
        actual_cost: Option<f64>,
        currency_code: &str,
        estimated_hours: Option<f64>,
        actual_hours: Option<f64>,
        regulatory_impact: Option<&str>,
        safety_impact: Option<&str>,
        validation_required: bool,
        created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::EngineeringChange> {
        Ok(atlas_shared::EngineeringChange {
            id: Uuid::new_v4(), organization_id: org_id,
            change_number: change_number.to_string(),
            change_type_id, category: category.to_string(),
            title: title.to_string(), description: description.map(String::from),
            change_reason: change_reason.map(String::from),
            change_reason_description: change_reason_description.map(String::from),
            priority: priority.to_string(), status: status.to_string(),
            revision: revision.to_string(),
            assigned_to, assigned_to_name: assigned_to_name.map(String::from),
            submitted_at, approved_at, implemented_at,
            target_date, effective_date,
            resolution_code: resolution_code.map(String::from),
            resolution_notes: resolution_notes.map(String::from),
            parent_change_id, superseded_by_id,
            impact_analysis,
            estimated_cost, actual_cost, currency_code: currency_code.to_string(),
            estimated_hours, actual_hours,
            regulatory_impact: regulatory_impact.map(String::from),
            safety_impact: safety_impact.map(String::from),
            validation_required,
            metadata: serde_json::json!({}),
            created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_change(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::EngineeringChange>> { Ok(None) }
    async fn get_change_by_number(&self, _org_id: Uuid, _number: &str) -> AtlasResult<Option<atlas_shared::EngineeringChange>> { Ok(None) }
    async fn list_changes(&self, _org_id: Uuid, _status: Option<&str>, _category: Option<&str>, _priority: Option<&str>, _assigned_to: Option<&Uuid>) -> AtlasResult<Vec<atlas_shared::EngineeringChange>> { Ok(vec![]) }
    async fn update_change_status(
        &self, _id: Uuid, _status: &str,
        _submitted_at: Option<chrono::DateTime<chrono::Utc>>,
        _approved_at: Option<chrono::DateTime<chrono::Utc>>,
        _implemented_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> AtlasResult<atlas_shared::EngineeringChange> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn update_change_with_resolution(
        &self, _id: Uuid, _status: &str, _resolution_notes: Option<&str>, _resolution_code: Option<&str>,
    ) -> AtlasResult<atlas_shared::EngineeringChange> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn implement_change(
        &self, _id: Uuid, _actual_cost: Option<f64>, _actual_hours: Option<f64>, _implemented_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> AtlasResult<atlas_shared::EngineeringChange> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn return_for_rework(&self, _id: Uuid, _comments: Option<&str>) -> AtlasResult<atlas_shared::EngineeringChange> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn delete_change(&self, _org_id: Uuid, _number: &str) -> AtlasResult<()> { Ok(()) }

    async fn create_change_line(
        &self, org_id: Uuid, change_id: Uuid, line_number: i32,
        item_id: Option<Uuid>, item_number: Option<&str>, item_name: Option<&str>,
        change_category: &str, field_name: Option<&str>,
        old_value: Option<&str>, new_value: Option<&str>,
        old_revision: Option<&str>, new_revision: Option<&str>,
        component_item_id: Option<Uuid>, component_item_number: Option<&str>,
        bom_quantity_old: Option<f64>, bom_quantity_new: Option<f64>,
        effectivity_date: Option<chrono::NaiveDate>,
        effectivity_end_date: Option<chrono::NaiveDate>,
        status: &str, completion_notes: Option<&str>,
        sequence_number: i32, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::EngineeringChangeLine> {
        Ok(atlas_shared::EngineeringChangeLine {
            id: Uuid::new_v4(), organization_id: org_id, change_id, line_number,
            item_id, item_number: item_number.map(String::from),
            item_name: item_name.map(String::from),
            change_category: change_category.to_string(),
            field_name: field_name.map(String::from),
            old_value: old_value.map(String::from), new_value: new_value.map(String::from),
            old_revision: old_revision.map(String::from), new_revision: new_revision.map(String::from),
            component_item_id, component_item_number: component_item_number.map(String::from),
            bom_quantity_old, bom_quantity_new,
            effectivity_date, effectivity_end_date,
            status: status.to_string(), completion_notes: completion_notes.map(String::from),
            sequence_number, metadata: serde_json::json!({}),
            created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_change_line(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::EngineeringChangeLine>> { Ok(None) }
    async fn list_change_lines(&self, _change_id: Uuid) -> AtlasResult<Vec<atlas_shared::EngineeringChangeLine>> { Ok(vec![]) }
    async fn update_change_line_status(&self, _id: Uuid, _status: &str, _completion_notes: Option<&str>) -> AtlasResult<atlas_shared::EngineeringChangeLine> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn delete_change_line(&self, _id: Uuid) -> AtlasResult<()> { Ok(()) }

    async fn create_affected_item(
        &self, org_id: Uuid, change_id: Uuid, item_id: Uuid,
        item_number: &str, item_name: Option<&str>,
        impact_type: &str, impact_description: Option<&str>,
        current_revision: Option<&str>, new_revision: Option<&str>,
        disposition: Option<&str>,
        old_item_status: Option<&str>, new_item_status: Option<&str>,
        phase_in_date: Option<chrono::NaiveDate>,
        phase_out_date: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::EngineeringChangeAffectedItem> {
        Ok(atlas_shared::EngineeringChangeAffectedItem {
            id: Uuid::new_v4(), organization_id: org_id, change_id, item_id,
            item_number: item_number.to_string(), item_name: item_name.map(String::from),
            impact_type: impact_type.to_string(),
            impact_description: impact_description.map(String::from),
            current_revision: current_revision.map(String::from),
            new_revision: new_revision.map(String::from),
            disposition: disposition.map(String::from),
            old_item_status: old_item_status.map(String::from),
            new_item_status: new_item_status.map(String::from),
            phase_in_date, phase_out_date,
            metadata: serde_json::json!({}),
            created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_affected_item(&self, _change_id: Uuid, _item_id: Uuid) -> AtlasResult<Option<atlas_shared::EngineeringChangeAffectedItem>> { Ok(None) }
    async fn get_affected_item_by_id(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::EngineeringChangeAffectedItem>> { Ok(None) }
    async fn list_affected_items(&self, _change_id: Uuid) -> AtlasResult<Vec<atlas_shared::EngineeringChangeAffectedItem>> { Ok(vec![]) }
    async fn remove_affected_item(&self, _id: Uuid) -> AtlasResult<()> { Ok(()) }

    async fn create_approval(
        &self, org_id: Uuid, change_id: Uuid, approval_level: i32,
        approver_id: Option<Uuid>, approver_name: Option<&str>,
        approver_role: Option<&str>, status: &str,
        action_date: Option<chrono::DateTime<chrono::Utc>>,
        comments: Option<&str>,
        delegated_from_id: Option<Uuid>,
        approval_conditions: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::EngineeringChangeApproval> {
        Ok(atlas_shared::EngineeringChangeApproval {
            id: Uuid::new_v4(), organization_id: org_id, change_id, approval_level,
            approver_id, approver_name: approver_name.map(String::from),
            approver_role: approver_role.map(String::from),
            status: status.to_string(), action_date,
            comments: comments.map(String::from),
            delegated_from_id, approval_conditions: approval_conditions.map(String::from),
            metadata: serde_json::json!({}),
            created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn list_approvals(&self, _change_id: Uuid) -> AtlasResult<Vec<atlas_shared::EngineeringChangeApproval>> { Ok(vec![]) }
    async fn get_pending_approvals(&self, _approver_id: Uuid) -> AtlasResult<Vec<atlas_shared::EngineeringChangeApproval>> { Ok(vec![]) }

    async fn get_dashboard(&self, _org_id: Uuid) -> AtlasResult<atlas_shared::EcmDashboard> {
        Ok(atlas_shared::EcmDashboard {
            total_changes: 0, open_changes: 0, pending_approval: 0,
            approved_changes: 0, implemented_changes: 0, rejected_changes: 0,
            ecr_count: 0, eco_count: 0, ecn_count: 0,
            critical_open: 0, high_open: 0, medium_open: 0, low_open: 0,
            avg_days_to_implement: 0.0, avg_days_to_approve: 0.0,
            total_items_affected: 0, total_estimated_cost: 0.0, total_actual_cost: 0.0,
            changes_by_reason: serde_json::json!({}),
            changes_by_status: serde_json::json!({}),
            changes_trend: serde_json::json!([]),
        })
    }
}

// ============================================================================
// Mock Channel Revenue Management Repository
// ============================================================================

pub struct MockChannelRevenueRepository;

#[async_trait]
impl crate::channel_revenue::ChannelRevenueRepository for MockChannelRevenueRepository {
    async fn create_promotion(
        &self,
        org_id: Uuid, promotion_number: &str, name: &str, description: Option<&str>,
        promotion_type: &str, status: &str, priority: Option<&str>,
        category: Option<&str>,
        partner_id: Option<Uuid>, partner_number: Option<&str>, partner_name: Option<&str>,
        fund_id: Option<Uuid>,
        start_date: chrono::NaiveDate, end_date: chrono::NaiveDate,
        sell_in_start_date: Option<chrono::NaiveDate>, sell_in_end_date: Option<chrono::NaiveDate>,
        sell_out_start_date: Option<chrono::NaiveDate>, sell_out_end_date: Option<chrono::NaiveDate>,
        product_category: Option<&str>,
        product_id: Option<Uuid>, product_number: Option<&str>, product_name: Option<&str>,
        customer_segment: Option<&str>, territory: Option<&str>,
        expected_revenue: f64, planned_budget: f64, currency_code: &str,
        discount_pct: Option<f64>, discount_amount: Option<f64>,
        volume_threshold: Option<f64>, volume_uom: Option<&str>,
        tier_config: serde_json::Value,
        objectives: Option<&str>, terms_and_conditions: Option<&str>,
        approval_status: &str,
        owner_id: Option<Uuid>, owner_name: Option<&str>,
        effective_from: Option<chrono::NaiveDate>, effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::TradePromotion> {
        Ok(atlas_shared::TradePromotion {
            id: Uuid::new_v4(), organization_id: org_id,
            promotion_number: promotion_number.to_string(),
            name: name.to_string(),
            description: description.map(String::from),
            promotion_type: promotion_type.to_string(),
            status: status.to_string(),
            priority: priority.map(String::from),
            category: category.map(String::from),
            partner_id, partner_number: partner_number.map(String::from),
            partner_name: partner_name.map(String::from),
            fund_id,
            start_date, end_date,
            sell_in_start_date, sell_in_end_date,
            sell_out_start_date, sell_out_end_date,
            product_category: product_category.map(String::from),
            product_id, product_number: product_number.map(String::from),
            product_name: product_name.map(String::from),
            customer_segment: customer_segment.map(String::from),
            territory: territory.map(String::from),
            expected_revenue, planned_budget,
            actual_spend: 0.0, accrued_amount: 0.0, claimed_amount: 0.0, settled_amount: 0.0,
            currency_code: currency_code.to_string(),
            discount_pct, discount_amount, volume_threshold,
            volume_uom: volume_uom.map(String::from),
            tier_config, objectives: objectives.map(String::from),
            terms_and_conditions: terms_and_conditions.map(String::from),
            approval_status: approval_status.to_string(),
            approved_by: None, approved_at: None,
            owner_id, owner_name: owner_name.map(String::from),
            effective_from, effective_to,
            metadata: serde_json::json!({}),
            created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_promotion(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::TradePromotion>> { Ok(None) }
    async fn get_promotion_by_number(&self, _org_id: Uuid, _promotion_number: &str) -> AtlasResult<Option<atlas_shared::TradePromotion>> { Ok(None) }
    async fn list_promotions(&self, _org_id: Uuid, _status: Option<&str>, _promotion_type: Option<&str>, _partner_id: Option<&Uuid>) -> AtlasResult<Vec<atlas_shared::TradePromotion>> { Ok(vec![]) }
    async fn update_promotion_status(&self, _id: Uuid, _status: &str) -> AtlasResult<atlas_shared::TradePromotion> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn update_promotion_approval(&self, _id: Uuid, _approval_status: &str, _approved_by: Option<Uuid>) -> AtlasResult<atlas_shared::TradePromotion> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn update_promotion_spend(&self, _id: Uuid, _actual_spend: f64, _accrued_amount: f64) -> AtlasResult<atlas_shared::TradePromotion> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn delete_promotion(&self, _org_id: Uuid, _promotion_number: &str) -> AtlasResult<()> { Ok(()) }

    async fn create_promotion_line(
        &self, org_id: Uuid, promotion_id: Uuid, line_number: i32,
        product_id: Option<Uuid>, product_number: Option<&str>, product_name: Option<&str>,
        product_category: Option<&str>, discount_type: &str, discount_value: f64,
        unit_of_measure: Option<&str>, quantity_from: Option<f64>, quantity_to: Option<f64>,
        planned_quantity: f64, planned_amount: f64, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::TradePromotionLine> {
        Ok(atlas_shared::TradePromotionLine {
            id: Uuid::new_v4(), organization_id: org_id, promotion_id, line_number,
            product_id, product_number: product_number.map(String::from),
            product_name: product_name.map(String::from),
            product_category: product_category.map(String::from),
            discount_type: discount_type.to_string(), discount_value,
            unit_of_measure: unit_of_measure.map(String::from),
            quantity_from, quantity_to, planned_quantity,
            actual_quantity: 0.0, planned_amount, actual_amount: 0.0, accrual_amount: 0.0,
            status: "active".to_string(),
            metadata: serde_json::json!({}),
            created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_promotion_line(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::TradePromotionLine>> { Ok(None) }
    async fn list_promotion_lines(&self, _promotion_id: Uuid) -> AtlasResult<Vec<atlas_shared::TradePromotionLine>> { Ok(vec![]) }
    async fn update_promotion_line_actuals(&self, _id: Uuid, _actual_quantity: f64, _actual_amount: f64, _accrual_amount: f64) -> AtlasResult<atlas_shared::TradePromotionLine> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn delete_promotion_line(&self, _id: Uuid) -> AtlasResult<()> { Ok(()) }

    async fn create_fund(
        &self, org_id: Uuid, fund_number: &str, name: &str, description: Option<&str>,
        fund_type: &str, status: &str, partner_id: Option<Uuid>, partner_number: Option<&str>, partner_name: Option<&str>,
        total_budget: f64, currency_code: &str, fund_year: Option<i32>, fund_quarter: Option<&str>,
        start_date: Option<chrono::NaiveDate>, end_date: Option<chrono::NaiveDate>,
        owner_id: Option<Uuid>, owner_name: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::PromotionFund> {
        Ok(atlas_shared::PromotionFund {
            id: Uuid::new_v4(), organization_id: org_id,
            fund_number: fund_number.to_string(), name: name.to_string(),
            description: description.map(String::from),
            fund_type: fund_type.to_string(), status: status.to_string(),
            partner_id, partner_number: partner_number.map(String::from),
            partner_name: partner_name.map(String::from),
            total_budget, allocated_amount: 0.0, committed_amount: 0.0,
            utilized_amount: 0.0, available_amount: total_budget,
            currency_code: currency_code.to_string(),
            fund_year, fund_quarter: fund_quarter.map(String::from),
            start_date, end_date, owner_id, owner_name: owner_name.map(String::from),
            approval_status: "not_submitted".to_string(),
            approved_by: None, approved_at: None,
            metadata: serde_json::json!({}),
            created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_fund(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::PromotionFund>> { Ok(None) }
    async fn get_fund_by_number(&self, _org_id: Uuid, _fund_number: &str) -> AtlasResult<Option<atlas_shared::PromotionFund>> { Ok(None) }
    async fn list_funds(&self, _org_id: Uuid, _status: Option<&str>, _fund_type: Option<&str>) -> AtlasResult<Vec<atlas_shared::PromotionFund>> { Ok(vec![]) }
    async fn update_fund_status(&self, _id: Uuid, _status: &str) -> AtlasResult<atlas_shared::PromotionFund> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn update_fund_budget(&self, _id: Uuid, _total_budget: f64) -> AtlasResult<atlas_shared::PromotionFund> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn update_fund_utilization(&self, _id: Uuid, _allocated_amount: f64, _committed_amount: f64, _utilized_amount: f64, _available_amount: f64) -> AtlasResult<atlas_shared::PromotionFund> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn delete_fund(&self, _org_id: Uuid, _fund_number: &str) -> AtlasResult<()> { Ok(()) }

    async fn create_claim(
        &self, org_id: Uuid, claim_number: &str,
        promotion_id: Option<Uuid>, promotion_number: Option<&str>,
        fund_id: Option<Uuid>, fund_number: Option<&str>,
        claim_type: &str, status: &str, priority: Option<&str>,
        partner_id: Option<Uuid>, partner_number: Option<&str>, partner_name: Option<&str>,
        claim_date: chrono::NaiveDate, sell_in_from: Option<chrono::NaiveDate>, sell_in_to: Option<chrono::NaiveDate>,
        product_id: Option<Uuid>, product_number: Option<&str>, product_name: Option<&str>,
        quantity: f64, unit_of_measure: Option<&str>, unit_price: Option<f64>,
        claimed_amount: f64, currency_code: &str,
        invoice_number: Option<&str>, invoice_date: Option<chrono::NaiveDate>,
        reference_document: Option<&str>, proof_of_performance: serde_json::Value,
        assigned_to: Option<Uuid>, assigned_to_name: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::TradeClaim> {
        Ok(atlas_shared::TradeClaim {
            id: Uuid::new_v4(), organization_id: org_id,
            claim_number: claim_number.to_string(),
            promotion_id, promotion_number: promotion_number.map(String::from),
            fund_id, fund_number: fund_number.map(String::from),
            claim_type: claim_type.to_string(), status: status.to_string(),
            priority: priority.map(String::from),
            partner_id, partner_number: partner_number.map(String::from),
            partner_name: partner_name.map(String::from),
            claim_date, sell_in_from, sell_in_to,
            product_id, product_number: product_number.map(String::from),
            product_name: product_name.map(String::from),
            quantity, unit_of_measure: unit_of_measure.map(String::from), unit_price,
            claimed_amount, approved_amount: 0.0, paid_amount: 0.0,
            currency_code: currency_code.to_string(),
            invoice_number: invoice_number.map(String::from), invoice_date,
            reference_document: reference_document.map(String::from),
            proof_of_performance,
            rejection_reason: None, resolution_notes: None,
            assigned_to, assigned_to_name: assigned_to_name.map(String::from),
            submitted_at: None, approved_at: None, paid_at: None,
            metadata: serde_json::json!({}),
            created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_claim(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::TradeClaim>> { Ok(None) }
    async fn get_claim_by_number(&self, _org_id: Uuid, _claim_number: &str) -> AtlasResult<Option<atlas_shared::TradeClaim>> { Ok(None) }
    async fn list_claims(&self, _org_id: Uuid, _status: Option<&str>, _claim_type: Option<&str>, _promotion_id: Option<&Uuid>) -> AtlasResult<Vec<atlas_shared::TradeClaim>> { Ok(vec![]) }
    async fn update_claim_status(&self, _id: Uuid, _status: &str, _approved_amount: Option<f64>, _rejection_reason: Option<&str>, _resolution_notes: Option<&str>) -> AtlasResult<atlas_shared::TradeClaim> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn update_claim_payment(&self, _id: Uuid, _paid_amount: f64) -> AtlasResult<atlas_shared::TradeClaim> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn delete_claim(&self, _org_id: Uuid, _claim_number: &str) -> AtlasResult<()> { Ok(()) }

    async fn create_settlement(
        &self, org_id: Uuid, settlement_number: &str,
        claim_id: Option<Uuid>, claim_number: Option<&str>,
        promotion_id: Option<Uuid>, promotion_number: Option<&str>,
        partner_id: Option<Uuid>, partner_number: Option<&str>, partner_name: Option<&str>,
        settlement_type: &str, status: &str,
        settlement_date: chrono::NaiveDate, settlement_amount: f64, currency_code: &str,
        payment_method: Option<&str>, payment_reference: Option<&str>,
        bank_account: Option<&str>, gl_account: Option<&str>, cost_center: Option<&str>,
        notes: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::TradeSettlement> {
        Ok(atlas_shared::TradeSettlement {
            id: Uuid::new_v4(), organization_id: org_id,
            settlement_number: settlement_number.to_string(),
            claim_id, claim_number: claim_number.map(String::from),
            promotion_id, promotion_number: promotion_number.map(String::from),
            partner_id, partner_number: partner_number.map(String::from),
            partner_name: partner_name.map(String::from),
            settlement_type: settlement_type.to_string(),
            status: status.to_string(),
            settlement_date, settlement_amount,
            currency_code: currency_code.to_string(),
            payment_method: payment_method.map(String::from),
            payment_reference: payment_reference.map(String::from),
            bank_account: bank_account.map(String::from),
            gl_account: gl_account.map(String::from),
            cost_center: cost_center.map(String::from),
            approved_by: None, approved_at: None, paid_at: None,
            notes: notes.map(String::from),
            metadata: serde_json::json!({}),
            created_by, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_settlement(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::TradeSettlement>> { Ok(None) }
    async fn get_settlement_by_number(&self, _org_id: Uuid, _settlement_number: &str) -> AtlasResult<Option<atlas_shared::TradeSettlement>> { Ok(None) }
    async fn list_settlements(&self, _org_id: Uuid, _status: Option<&str>, _settlement_type: Option<&str>) -> AtlasResult<Vec<atlas_shared::TradeSettlement>> { Ok(vec![]) }
    async fn update_settlement_status(&self, _id: Uuid, _status: &str, _approved_by: Option<Uuid>) -> AtlasResult<atlas_shared::TradeSettlement> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn delete_settlement(&self, _org_id: Uuid, _settlement_number: &str) -> AtlasResult<()> { Ok(()) }

    async fn get_dashboard(&self, _org_id: Uuid) -> AtlasResult<atlas_shared::ChannelRevenueDashboard> {
        Ok(atlas_shared::ChannelRevenueDashboard {
            total_promotions: 0, active_promotions: 0,
            total_planned_budget: 0.0, total_actual_spend: 0.0,
            budget_utilization_pct: 0.0, total_expected_revenue: 0.0,
            roi_pct: 0.0,
            total_claims: 0, pending_claims: 0, approved_claims: 0, rejected_claims: 0,
            total_claimed_amount: 0.0, total_approved_amount: 0.0, total_paid_amount: 0.0,
            total_funds: 0, active_funds: 0,
            total_fund_budget: 0.0, total_fund_utilized: 0.0, fund_utilization_pct: 0.0,
            total_settlements: 0, pending_settlements: 0, completed_settlements: 0,
            total_settlement_amount: 0.0,
            promotions_by_status: serde_json::json!({}),
            promotions_by_type: serde_json::json!({}),
            claims_by_status: serde_json::json!({}),
            spend_trend: serde_json::json!([]),
            top_partners: serde_json::json!([]),
        })
    }
}

/// Mock Product Configurator Repository
pub struct MockProductConfiguratorRepository;

#[async_trait]
impl crate::product_configurator::ProductConfiguratorRepository for MockProductConfiguratorRepository {
    // Models
    async fn create_model(
        &self,
        org_id: Uuid, model_number: &str, name: &str, description: Option<&str>,
        base_product_id: Option<Uuid>, base_product_number: Option<&str>, base_product_name: Option<&str>,
        model_type: &str, status: &str, version: i32,
        effective_from: Option<chrono::NaiveDate>, effective_to: Option<chrono::NaiveDate>,
        default_config: serde_json::Value, validation_mode: &str,
        ui_layout: serde_json::Value, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::ConfigModel> {
        Ok(atlas_shared::ConfigModel {
            id: Uuid::new_v4(),
            organization_id: org_id,
            model_number: model_number.to_string(),
            name: name.to_string(),
            description: description.map(|s| s.to_string()),
            base_product_id,
            base_product_number: base_product_number.map(|s| s.to_string()),
            base_product_name: base_product_name.map(|s| s.to_string()),
            model_type: model_type.to_string(),
            status: status.to_string(),
            version,
            effective_from,
            effective_to,
            default_config,
            validation_mode: validation_mode.to_string(),
            ui_layout,
            metadata: serde_json::json!({}),
            created_by,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })
    }
    async fn get_model(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::ConfigModel>> { Ok(None) }
    async fn get_model_by_number(&self, _org_id: Uuid, _model_number: &str) -> AtlasResult<Option<atlas_shared::ConfigModel>> { Ok(None) }
    async fn list_models(&self, _org_id: Uuid, _status: Option<&str>, _model_type: Option<&str>) -> AtlasResult<Vec<atlas_shared::ConfigModel>> { Ok(vec![]) }
    async fn update_model_status(&self, _id: Uuid, _status: &str) -> AtlasResult<atlas_shared::ConfigModel> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn delete_model(&self, _org_id: Uuid, _model_number: &str) -> AtlasResult<()> { Ok(()) }

    // Features
    async fn create_feature(
        &self,
        org_id: Uuid, model_id: Uuid, feature_code: &str, name: &str,
        description: Option<&str>, feature_type: &str, is_required: bool,
        display_order: i32, ui_hints: serde_json::Value,
    ) -> AtlasResult<atlas_shared::ConfigFeature> {
        Ok(atlas_shared::ConfigFeature {
            id: Uuid::new_v4(),
            organization_id: org_id,
            model_id,
            feature_code: feature_code.to_string(),
            name: name.to_string(),
            description: description.map(|s| s.to_string()),
            feature_type: feature_type.to_string(),
            is_required,
            display_order,
            ui_hints,
            metadata: serde_json::json!({}),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })
    }
    async fn get_feature(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::ConfigFeature>> { Ok(None) }
    async fn list_features(&self, _model_id: Uuid) -> AtlasResult<Vec<atlas_shared::ConfigFeature>> { Ok(vec![]) }
    async fn delete_feature(&self, _id: Uuid) -> AtlasResult<()> { Ok(()) }

    // Options
    async fn create_option(
        &self,
        org_id: Uuid, feature_id: Uuid, option_code: &str, name: &str,
        description: Option<&str>, option_type: &str,
        price_adjustment: f64, cost_adjustment: f64, lead_time_days: i32,
        is_default: bool, is_available: bool, display_order: i32,
    ) -> AtlasResult<atlas_shared::ConfigOption> {
        Ok(atlas_shared::ConfigOption {
            id: Uuid::new_v4(),
            organization_id: org_id,
            feature_id,
            option_code: option_code.to_string(),
            name: name.to_string(),
            description: description.map(|s| s.to_string()),
            option_type: option_type.to_string(),
            price_adjustment,
            cost_adjustment,
            lead_time_days,
            is_default,
            is_available,
            display_order,
            metadata: serde_json::json!({}),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })
    }
    async fn get_option(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::ConfigOption>> { Ok(None) }
    async fn list_options(&self, _feature_id: Uuid) -> AtlasResult<Vec<atlas_shared::ConfigOption>> { Ok(vec![]) }
    async fn update_option_availability(&self, _id: Uuid, _is_available: bool) -> AtlasResult<atlas_shared::ConfigOption> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn delete_option(&self, _id: Uuid) -> AtlasResult<()> { Ok(()) }

    // Rules
    async fn create_rule(
        &self,
        org_id: Uuid, model_id: Uuid, rule_code: &str, name: &str,
        description: Option<&str>, rule_type: &str,
        source_feature_id: Option<Uuid>, source_option_id: Option<Uuid>,
        target_feature_id: Option<Uuid>, target_option_id: Option<Uuid>,
        condition_expression: Option<&str>, severity: &str,
        is_active: bool, priority: i32, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::ConfigRule> {
        Ok(atlas_shared::ConfigRule {
            id: Uuid::new_v4(),
            organization_id: org_id,
            model_id,
            rule_code: rule_code.to_string(),
            name: name.to_string(),
            description: description.map(|s| s.to_string()),
            rule_type: rule_type.to_string(),
            source_feature_id,
            source_option_id,
            target_feature_id,
            target_option_id,
            condition_expression: condition_expression.map(|s| s.to_string()),
            severity: severity.to_string(),
            is_active,
            priority,
            metadata: serde_json::json!({}),
            created_by,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })
    }
    async fn get_rule(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::ConfigRule>> { Ok(None) }
    async fn get_rule_by_code(&self, _model_id: Uuid, _rule_code: &str) -> AtlasResult<Option<atlas_shared::ConfigRule>> { Ok(None) }
    async fn list_rules(&self, _model_id: Uuid, _rule_type: Option<&str>) -> AtlasResult<Vec<atlas_shared::ConfigRule>> { Ok(vec![]) }
    async fn update_rule_active(&self, _id: Uuid, _is_active: bool) -> AtlasResult<atlas_shared::ConfigRule> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn delete_rule(&self, _id: Uuid) -> AtlasResult<()> { Ok(()) }

    // Instances
    async fn create_instance(
        &self,
        org_id: Uuid, instance_number: &str, model_id: Uuid,
        model_number: Option<&str>, name: Option<&str>, description: Option<&str>,
        status: &str, selections: serde_json::Value,
        validation_errors: serde_json::Value, validation_warnings: serde_json::Value,
        base_price: f64, total_price: f64, currency_code: &str,
        config_hash: Option<&str>,
        effective_date: Option<chrono::NaiveDate>,
        configured_by: Option<Uuid>, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::ConfigInstance> {
        Ok(atlas_shared::ConfigInstance {
            id: Uuid::new_v4(),
            organization_id: org_id,
            instance_number: instance_number.to_string(),
            model_id,
            model_number: model_number.map(|s| s.to_string()),
            name: name.map(|s| s.to_string()),
            description: description.map(|s| s.to_string()),
            status: status.to_string(),
            selections,
            validation_errors,
            validation_warnings,
            base_price,
            total_price,
            currency_code: currency_code.to_string(),
            config_hash: config_hash.map(|s| s.to_string()),
            effective_date,
            valid_from: None,
            valid_to: None,
            sales_order_id: None,
            sales_order_number: None,
            sales_order_line: None,
            configured_by,
            approved_by: None,
            approved_at: None,
            metadata: serde_json::json!({}),
            created_by,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })
    }
    async fn get_instance(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::ConfigInstance>> { Ok(None) }
    async fn get_instance_by_number(&self, _org_id: Uuid, _instance_number: &str) -> AtlasResult<Option<atlas_shared::ConfigInstance>> { Ok(None) }
    async fn list_instances(&self, _org_id: Uuid, _status: Option<&str>, _model_id: Option<&Uuid>) -> AtlasResult<Vec<atlas_shared::ConfigInstance>> { Ok(vec![]) }
    async fn update_instance_status(&self, _id: Uuid, _status: &str) -> AtlasResult<atlas_shared::ConfigInstance> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn update_instance_validation(&self, _id: Uuid, _validation_errors: serde_json::Value, _validation_warnings: serde_json::Value) -> AtlasResult<atlas_shared::ConfigInstance> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn update_instance_selections(&self, _id: Uuid, _selections: serde_json::Value, _total_price: f64, _config_hash: Option<&str>) -> AtlasResult<atlas_shared::ConfigInstance> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn update_instance_approval(&self, _id: Uuid, _approved_by: Option<Uuid>) -> AtlasResult<atlas_shared::ConfigInstance> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn link_instance_to_order(&self, _id: Uuid, _sales_order_id: Option<Uuid>, _sales_order_number: Option<&str>, _sales_order_line: Option<i32>) -> AtlasResult<atlas_shared::ConfigInstance> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn delete_instance(&self, _org_id: Uuid, _instance_number: &str) -> AtlasResult<()> { Ok(()) }

    // Dashboard
    async fn get_dashboard(&self, _org_id: Uuid) -> AtlasResult<atlas_shared::ConfiguratorDashboard> {
        Ok(atlas_shared::ConfiguratorDashboard {
            total_models: 0, active_models: 0,
            total_configurations: 0, valid_configurations: 0, invalid_configurations: 0, ordered_configurations: 0,
            total_rules: 0, active_rules: 0,
            avg_configuration_price: 0.0, total_configured_value: 0.0,
            models_by_status: serde_json::json!({}),
            configurations_by_status: serde_json::json!({}),
            top_configured_models: serde_json::json!([]),
        })
    }
}

/// Mock Transportation Management Repository
pub struct MockTransportationManagementRepository;

#[async_trait]
impl crate::transportation_management::TransportationManagementRepository for MockTransportationManagementRepository {
    // Carriers
    async fn create_carrier(
        &self,
        org_id: Uuid, carrier_code: &str, name: &str, description: Option<&str>,
        carrier_type: &str, status: &str,
        scac_code: Option<&str>, dot_number: Option<&str>, mc_number: Option<&str>, tax_id: Option<&str>,
        contact_name: Option<&str>, contact_email: Option<&str>, contact_phone: Option<&str>,
        address_line1: Option<&str>, address_line2: Option<&str>,
        city: Option<&str>, state: Option<&str>, postal_code: Option<&str>, country: &str,
        currency_code: &str, payment_terms: &str,
        insurance_policy_number: Option<&str>, insurance_expiry_date: Option<chrono::NaiveDate>,
        default_service_level: &str,
        capabilities: serde_json::Value, metadata: serde_json::Value,
        created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::Carrier> {
        Ok(atlas_shared::Carrier {
            id: Uuid::new_v4(),
            organization_id: org_id,
            carrier_code: carrier_code.to_string(),
            name: name.to_string(),
            description: description.map(|s| s.to_string()),
            carrier_type: carrier_type.to_string(),
            status: status.to_string(),
            scac_code: scac_code.map(|s| s.to_string()),
            dot_number: dot_number.map(|s| s.to_string()),
            mc_number: mc_number.map(|s| s.to_string()),
            tax_id: tax_id.map(|s| s.to_string()),
            contact_name: contact_name.map(|s| s.to_string()),
            contact_email: contact_email.map(|s| s.to_string()),
            contact_phone: contact_phone.map(|s| s.to_string()),
            address_line1: address_line1.map(|s| s.to_string()),
            address_line2: address_line2.map(|s| s.to_string()),
            city: city.map(|s| s.to_string()),
            state: state.map(|s| s.to_string()),
            postal_code: postal_code.map(|s| s.to_string()),
            country: country.to_string(),
            currency_code: currency_code.to_string(),
            payment_terms: payment_terms.to_string(),
            insurance_policy_number: insurance_policy_number.map(|s| s.to_string()),
            insurance_expiry_date,
            performance_rating: 0.0,
            on_time_delivery_pct: 0.0,
            claims_ratio: 0.0,
            default_service_level: default_service_level.to_string(),
            capabilities,
            metadata,
            created_by,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })
    }
    async fn get_carrier(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::Carrier>> { Ok(None) }
    async fn get_carrier_by_code(&self, _org_id: Uuid, _carrier_code: &str) -> AtlasResult<Option<atlas_shared::Carrier>> { Ok(None) }
    async fn list_carriers(&self, _org_id: Uuid, _status: Option<&str>, _carrier_type: Option<&str>) -> AtlasResult<Vec<atlas_shared::Carrier>> { Ok(vec![]) }
    async fn update_carrier_status(&self, _id: Uuid, _status: &str) -> AtlasResult<atlas_shared::Carrier> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn update_carrier_performance(&self, _id: Uuid, _rating: f64, _on_time_pct: f64, _claims: f64) -> AtlasResult<atlas_shared::Carrier> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn delete_carrier(&self, _org_id: Uuid, _carrier_code: &str) -> AtlasResult<()> { Ok(()) }

    // Carrier Services
    async fn create_carrier_service(
        &self,
        org_id: Uuid, carrier_id: Uuid, service_code: &str, name: &str,
        description: Option<&str>, service_level: &str,
        transit_days_min: i32, transit_days_max: i32,
        max_weight_kg: Option<f64>, max_dimensions: Option<serde_json::Value>,
        cutoff_time: Option<chrono::NaiveTime>,
        operates_on_weekends: bool, is_international: bool,
        rate_per_kg: f64, minimum_charge: f64, fuel_surcharge_pct: f64,
        is_active: bool, metadata: serde_json::Value,
    ) -> AtlasResult<atlas_shared::CarrierService> {
        Ok(atlas_shared::CarrierService {
            id: Uuid::new_v4(),
            organization_id: org_id,
            carrier_id,
            service_code: service_code.to_string(),
            name: name.to_string(),
            description: description.map(|s| s.to_string()),
            service_level: service_level.to_string(),
            transit_days_min,
            transit_days_max,
            max_weight_kg,
            max_dimensions,
            cutoff_time,
            operates_on_weekends,
            is_international,
            rate_per_kg,
            minimum_charge,
            fuel_surcharge_pct,
            is_active,
            metadata,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })
    }
    async fn get_carrier_service(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::CarrierService>> { Ok(None) }
    async fn list_carrier_services(&self, _carrier_id: Uuid, _active_only: bool) -> AtlasResult<Vec<atlas_shared::CarrierService>> { Ok(vec![]) }
    async fn update_carrier_service_active(&self, _id: Uuid, _is_active: bool) -> AtlasResult<atlas_shared::CarrierService> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn delete_carrier_service(&self, _id: Uuid) -> AtlasResult<()> { Ok(()) }

    // Transport Lanes
    async fn create_lane(
        &self,
        org_id: Uuid, lane_code: &str, name: &str, description: Option<&str>,
        origin_location_id: Option<Uuid>, origin_location_name: Option<&str>,
        origin_city: Option<&str>, origin_state: Option<&str>, origin_country: &str, origin_postal_code: Option<&str>,
        destination_location_id: Option<Uuid>, destination_location_name: Option<&str>,
        destination_city: Option<&str>, destination_state: Option<&str>, destination_country: &str, destination_postal_code: Option<&str>,
        distance_km: Option<f64>, estimated_transit_hours: Option<f64>,
        lane_type: &str, preferred_carrier_id: Option<Uuid>, preferred_service_id: Option<Uuid>,
        status: &str, effective_from: Option<chrono::NaiveDate>, effective_to: Option<chrono::NaiveDate>,
        restrictions: serde_json::Value, metadata: serde_json::Value, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::TransportLane> {
        Ok(atlas_shared::TransportLane {
            id: Uuid::new_v4(),
            organization_id: org_id,
            lane_code: lane_code.to_string(),
            name: name.to_string(),
            description: description.map(|s| s.to_string()),
            origin_location_id, origin_location_name: origin_location_name.map(|s| s.to_string()),
            origin_city: origin_city.map(|s| s.to_string()), origin_state: origin_state.map(|s| s.to_string()),
            origin_country: origin_country.to_string(), origin_postal_code: origin_postal_code.map(|s| s.to_string()),
            destination_location_id, destination_location_name: destination_location_name.map(|s| s.to_string()),
            destination_city: destination_city.map(|s| s.to_string()), destination_state: destination_state.map(|s| s.to_string()),
            destination_country: destination_country.to_string(), destination_postal_code: destination_postal_code.map(|s| s.to_string()),
            distance_km, estimated_transit_hours,
            lane_type: lane_type.to_string(),
            preferred_carrier_id, preferred_service_id,
            status: status.to_string(),
            effective_from, effective_to,
            restrictions, metadata,
            created_by,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })
    }
    async fn get_lane(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::TransportLane>> { Ok(None) }
    async fn get_lane_by_code(&self, _org_id: Uuid, _lane_code: &str) -> AtlasResult<Option<atlas_shared::TransportLane>> { Ok(None) }
    async fn list_lanes(&self, _org_id: Uuid, _status: Option<&str>, _lane_type: Option<&str>) -> AtlasResult<Vec<atlas_shared::TransportLane>> { Ok(vec![]) }
    async fn update_lane_status(&self, _id: Uuid, _status: &str) -> AtlasResult<atlas_shared::TransportLane> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn delete_lane(&self, _org_id: Uuid, _lane_code: &str) -> AtlasResult<()> { Ok(()) }

    // Shipments
    #[allow(clippy::too_many_arguments)]
    async fn create_shipment(
        &self,
        org_id: Uuid, shipment_number: &str, name: Option<&str>, description: Option<&str>,
        status: &str, shipment_type: &str, priority: &str,
        carrier_id: Option<Uuid>, carrier_code: Option<&str>, carrier_name: Option<&str>,
        carrier_service_id: Option<Uuid>, carrier_service_code: Option<&str>,
        lane_id: Option<Uuid>, lane_code: Option<&str>,
        origin_location_id: Option<Uuid>, origin_location_name: Option<&str>, origin_address: serde_json::Value,
        destination_location_id: Option<Uuid>, destination_location_name: Option<&str>, destination_address: serde_json::Value,
        planned_ship_date: Option<chrono::NaiveDate>, planned_delivery_date: Option<chrono::NaiveDate>,
        pickup_window_start: Option<chrono::DateTime<chrono::Utc>>,
        pickup_window_end: Option<chrono::DateTime<chrono::Utc>>,
        delivery_window_start: Option<chrono::DateTime<chrono::Utc>>,
        delivery_window_end: Option<chrono::DateTime<chrono::Utc>>,
        currency_code: &str,
        tracking_number: Option<&str>, pro_number: Option<&str>, bill_of_lading: Option<&str>,
        sales_order_id: Option<Uuid>, sales_order_number: Option<&str>,
        purchase_order_id: Option<Uuid>, purchase_order_number: Option<&str>,
        transfer_order_id: Option<Uuid>,
        special_instructions: Option<&str>,
        declared_value: Option<f64>, insurance_required: bool, signature_required: bool,
        temperature_requirements: Option<serde_json::Value>, hazmat_info: Option<serde_json::Value>,
        metadata: serde_json::Value, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::TransportShipment> {
        Ok(atlas_shared::TransportShipment {
            id: Uuid::new_v4(),
            organization_id: org_id,
            shipment_number: shipment_number.to_string(),
            name: name.map(|s| s.to_string()),
            description: description.map(|s| s.to_string()),
            status: status.to_string(),
            shipment_type: shipment_type.to_string(),
            priority: priority.to_string(),
            carrier_id, carrier_code: carrier_code.map(|s| s.to_string()), carrier_name: carrier_name.map(|s| s.to_string()),
            carrier_service_id, carrier_service_code: carrier_service_code.map(|s| s.to_string()),
            lane_id, lane_code: lane_code.map(|s| s.to_string()),
            origin_location_id, origin_location_name: origin_location_name.map(|s| s.to_string()), origin_address,
            destination_location_id, destination_location_name: destination_location_name.map(|s| s.to_string()), destination_address,
            planned_ship_date, actual_ship_date: None,
            planned_delivery_date, actual_delivery_date: None,
            pickup_window_start, pickup_window_end,
            delivery_window_start, delivery_window_end,
            total_weight_kg: 0.0, total_volume_cbm: 0.0, total_pieces: 0,
            freight_cost: 0.0, fuel_surcharge: 0.0, accessorial_charges: 0.0, total_cost: 0.0,
            currency_code: currency_code.to_string(),
            tracking_number: tracking_number.map(|s| s.to_string()),
            tracking_url: None,
            pro_number: pro_number.map(|s| s.to_string()),
            bill_of_lading: bill_of_lading.map(|s| s.to_string()),
            sales_order_id, sales_order_number: sales_order_number.map(|s| s.to_string()),
            purchase_order_id, purchase_order_number: purchase_order_number.map(|s| s.to_string()),
            transfer_order_id,
            special_instructions: special_instructions.map(|s| s.to_string()),
            declared_value, insurance_required, signature_required,
            temperature_requirements, hazmat_info,
            driver_name: None, vehicle_id: None,
            metadata,
            booked_by: None, shipped_by: None, received_by: None,
            created_by,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })
    }
    async fn get_shipment(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::TransportShipment>> { Ok(None) }
    async fn get_shipment_by_number(&self, _org_id: Uuid, _shipment_number: &str) -> AtlasResult<Option<atlas_shared::TransportShipment>> { Ok(None) }
    async fn list_shipments(&self, _org_id: Uuid, _status: Option<&str>, _shipment_type: Option<&str>) -> AtlasResult<Vec<atlas_shared::TransportShipment>> { Ok(vec![]) }
    async fn update_shipment_status(&self, _id: Uuid, _status: &str) -> AtlasResult<atlas_shared::TransportShipment> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn update_shipment_carrier(&self, _id: Uuid, _carrier_id: Option<Uuid>, _carrier_code: Option<&str>, _carrier_name: Option<&str>, _carrier_service_id: Option<Uuid>, _carrier_service_code: Option<&str>) -> AtlasResult<atlas_shared::TransportShipment> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn update_shipment_dates(&self, _id: Uuid, _actual_ship_date: Option<chrono::NaiveDate>, _actual_delivery_date: Option<chrono::NaiveDate>) -> AtlasResult<atlas_shared::TransportShipment> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn update_shipment_tracking(&self, _id: Uuid, _tracking_number: Option<&str>, _tracking_url: Option<&str>, _pro_number: Option<&str>, _bill_of_lading: Option<&str>) -> AtlasResult<atlas_shared::TransportShipment> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn update_shipment_totals(&self, _id: Uuid, _weight: f64, _volume: f64, _pieces: i32, _freight: f64, _fuel: f64, _accessorial: f64, _total: f64) -> AtlasResult<atlas_shared::TransportShipment> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn update_shipment_delivery(&self, _id: Uuid, _received_by: Option<Uuid>) -> AtlasResult<atlas_shared::TransportShipment> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn delete_shipment(&self, _org_id: Uuid, _shipment_number: &str) -> AtlasResult<()> { Ok(()) }

    // TransportShipment Stops
    async fn create_stop(
        &self,
        org_id: Uuid, shipment_id: Uuid, stop_number: i32, stop_type: &str,
        location_id: Option<Uuid>, location_name: Option<&str>, address: serde_json::Value,
        planned_arrival: Option<chrono::DateTime<chrono::Utc>>,
        planned_departure: Option<chrono::DateTime<chrono::Utc>>,
        contact_name: Option<&str>, contact_phone: Option<&str>,
        special_instructions: Option<&str>,
        metadata: serde_json::Value,
    ) -> AtlasResult<atlas_shared::TransportShipmentStop> {
        Ok(atlas_shared::TransportShipmentStop {
            id: Uuid::new_v4(),
            organization_id: org_id, shipment_id, stop_number,
            stop_type: stop_type.to_string(),
            location_id, location_name: location_name.map(|s| s.to_string()),
            address, planned_arrival, actual_arrival: None,
            planned_departure, actual_departure: None,
            status: "pending".to_string(),
            contact_name: contact_name.map(|s| s.to_string()),
            contact_phone: contact_phone.map(|s| s.to_string()),
            special_instructions: special_instructions.map(|s| s.to_string()),
            pieces: 0, weight_kg: 0.0, metadata,
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_stop(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::TransportShipmentStop>> { Ok(None) }
    async fn list_stops(&self, _shipment_id: Uuid) -> AtlasResult<Vec<atlas_shared::TransportShipmentStop>> { Ok(vec![]) }
    async fn update_stop_status(&self, _id: Uuid, _status: &str, _actual_arrival: Option<chrono::DateTime<chrono::Utc>>, _actual_departure: Option<chrono::DateTime<chrono::Utc>>) -> AtlasResult<atlas_shared::TransportShipmentStop> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn delete_stop(&self, _id: Uuid) -> AtlasResult<()> { Ok(()) }

    // TransportShipment Lines
    async fn create_shipment_line(
        &self,
        org_id: Uuid, shipment_id: Uuid, line_number: i32,
        item_id: Option<Uuid>, item_number: Option<&str>, item_description: Option<&str>,
        quantity: i32, unit_of_measure: &str,
        weight_kg: f64, volume_cbm: f64,
        lot_number: Option<&str>, serial_numbers: serde_json::Value,
        source_line_id: Option<Uuid>, source_line_type: Option<&str>,
        stop_id: Option<Uuid>,
        freight_class: Option<&str>, nmfc_code: Option<&str>, hazmat_class: Option<&str>,
        metadata: serde_json::Value,
    ) -> AtlasResult<atlas_shared::TransportShipmentLine> {
        Ok(atlas_shared::TransportShipmentLine {
            id: Uuid::new_v4(),
            organization_id: org_id, shipment_id, line_number,
            item_id, item_number: item_number.map(|s| s.to_string()),
            item_description: item_description.map(|s| s.to_string()),
            quantity, quantity_shipped: 0, quantity_received: 0,
            unit_of_measure: unit_of_measure.to_string(),
            weight_kg, volume_cbm,
            lot_number: lot_number.map(|s| s.to_string()), serial_numbers,
            source_line_id, source_line_type: source_line_type.map(|s| s.to_string()),
            stop_id, freight_class: freight_class.map(|s| s.to_string()),
            nmfc_code: nmfc_code.map(|s| s.to_string()), hazmat_class: hazmat_class.map(|s| s.to_string()),
            metadata, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_shipment_line(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::TransportShipmentLine>> { Ok(None) }
    async fn list_shipment_lines(&self, _shipment_id: Uuid) -> AtlasResult<Vec<atlas_shared::TransportShipmentLine>> { Ok(vec![]) }
    async fn update_shipment_line_quantities(&self, _id: Uuid, _shipped: i32, _received: i32) -> AtlasResult<atlas_shared::TransportShipmentLine> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn delete_shipment_line(&self, _id: Uuid) -> AtlasResult<()> { Ok(()) }

    // Tracking Events
    async fn create_tracking_event(
        &self,
        org_id: Uuid, shipment_id: Uuid, event_type: &str,
        event_timestamp: chrono::DateTime<chrono::Utc>,
        location_description: Option<&str>,
        city: Option<&str>, state: Option<&str>, country: Option<&str>,
        latitude: Option<f64>, longitude: Option<f64>,
        description: Option<&str>,
        carrier_event_code: Option<&str>, carrier_event_description: Option<&str>,
        updated_by: Option<&str>,
        metadata: serde_json::Value,
    ) -> AtlasResult<atlas_shared::TransportShipmentTrackingEvent> {
        Ok(atlas_shared::TransportShipmentTrackingEvent {
            id: Uuid::new_v4(),
            organization_id: org_id, shipment_id,
            event_type: event_type.to_string(), event_timestamp,
            location_description: location_description.map(|s| s.to_string()),
            city: city.map(|s| s.to_string()), state: state.map(|s| s.to_string()),
            country: country.map(|s| s.to_string()),
            latitude, longitude, description: description.map(|s| s.to_string()),
            carrier_event_code: carrier_event_code.map(|s| s.to_string()),
            carrier_event_description: carrier_event_description.map(|s| s.to_string()),
            updated_by: updated_by.map(|s| s.to_string()),
            metadata, created_at: chrono::Utc::now(),
        })
    }
    async fn list_tracking_events(&self, _shipment_id: Uuid) -> AtlasResult<Vec<atlas_shared::TransportShipmentTrackingEvent>> { Ok(vec![]) }

    // Freight Rates
    async fn create_freight_rate(
        &self,
        org_id: Uuid, rate_code: &str, name: &str, description: Option<&str>,
        carrier_id: Uuid, carrier_service_id: Option<Uuid>, lane_id: Option<Uuid>,
        rate_type: &str, rate_amount: f64, minimum_charge: f64, currency_code: &str,
        fuel_surcharge_pct: f64, accessorial_rates: serde_json::Value,
        effective_from: chrono::NaiveDate, effective_to: Option<chrono::NaiveDate>,
        status: &str, is_contract_rate: bool, contract_number: Option<&str>,
        volume_threshold_min: Option<f64>, volume_threshold_max: Option<f64>,
        metadata: serde_json::Value, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::FreightRate> {
        Ok(atlas_shared::FreightRate {
            id: Uuid::new_v4(),
            organization_id: org_id,
            rate_code: rate_code.to_string(), name: name.to_string(),
            description: description.map(|s| s.to_string()),
            carrier_id, carrier_service_id, lane_id,
            rate_type: rate_type.to_string(), rate_amount, minimum_charge,
            currency_code: currency_code.to_string(), fuel_surcharge_pct,
            accessorial_rates, effective_from, effective_to,
            status: status.to_string(), is_contract_rate,
            contract_number: contract_number.map(|s| s.to_string()),
            volume_threshold_min, volume_threshold_max,
            metadata, created_by,
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_freight_rate(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::FreightRate>> { Ok(None) }
    async fn get_freight_rate_by_code(&self, _org_id: Uuid, _rate_code: &str) -> AtlasResult<Option<atlas_shared::FreightRate>> { Ok(None) }
    async fn list_freight_rates(&self, _org_id: Uuid, _carrier_id: Option<&Uuid>, _status: Option<&str>) -> AtlasResult<Vec<atlas_shared::FreightRate>> { Ok(vec![]) }
    async fn update_freight_rate_status(&self, _id: Uuid, _status: &str) -> AtlasResult<atlas_shared::FreightRate> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn delete_freight_rate(&self, _org_id: Uuid, _rate_code: &str) -> AtlasResult<()> { Ok(()) }

    // Dashboard
    async fn get_dashboard(&self, _org_id: Uuid) -> AtlasResult<atlas_shared::TransportationDashboard> {
        Ok(atlas_shared::TransportationDashboard {
            total_shipments: 0, active_shipments: 0, delivered_shipments: 0,
            delayed_shipments: 0, exception_shipments: 0,
            total_carriers: 0, active_carriers: 0,
            total_lanes: 0, active_lanes: 0,
            on_time_delivery_pct: 0.0, avg_transit_days: 0.0,
            total_freight_cost: 0.0, avg_cost_per_kg: 0.0,
            shipments_by_status: serde_json::json!({}),
            shipments_by_carrier: serde_json::json!({}),
            cost_by_carrier: serde_json::json!({}),
            top_lanes: serde_json::json!([]),
        })
    }
}

// ===========================================================================
// Workplace Health & Safety (EHS)
// ===========================================================================

pub struct MockHealthSafetyRepository;

#[async_trait]
impl crate::health_safety::HealthSafetyRepository for MockHealthSafetyRepository {
    // Incidents
    async fn create_incident(
        &self, org_id: Uuid, incident_number: &str, title: &str, description: Option<&str>,
        incident_type: &str, severity: &str, status: &str, priority: &str,
        incident_date: chrono::NaiveDate, incident_time: Option<&str>,
        location: Option<&str>, facility_id: Option<Uuid>, department_id: Option<Uuid>,
        reported_by_id: Option<Uuid>, reported_by_name: Option<&str>,
        assigned_to_id: Option<Uuid>, assigned_to_name: Option<&str>,
        root_cause: Option<&str>, immediate_action: Option<&str>,
        osha_recordable: bool, osha_classification: Option<&str>,
        days_away_from_work: i32, days_restricted: i32,
        body_part: Option<&str>, injury_source: Option<&str>,
        event_type: Option<&str>, environment_factor: Option<&str>,
        involved_parties: serde_json::Value, witness_statements: serde_json::Value,
        attachments: serde_json::Value,
        resolution_date: Option<chrono::NaiveDate>, closed_date: Option<chrono::NaiveDate>,
        closed_by: Option<Uuid>,
        metadata: serde_json::Value, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::SafetyIncident> {
        Ok(atlas_shared::SafetyIncident {
            id: Uuid::new_v4(), organization_id: org_id,
            incident_number: incident_number.to_string(),
            title: title.to_string(),
            description: description.map(String::from),
            incident_type: incident_type.to_string(),
            severity: severity.to_string(),
            status: status.to_string(),
            priority: priority.to_string(),
            incident_date, incident_time: incident_time.map(String::from),
            location: location.map(String::from),
            facility_id, department_id,
            reported_by_id, reported_by_name: reported_by_name.map(String::from),
            assigned_to_id, assigned_to_name: assigned_to_name.map(String::from),
            root_cause: root_cause.map(String::from),
            immediate_action: immediate_action.map(String::from),
            osha_recordable, osha_classification: osha_classification.map(String::from),
            days_away_from_work, days_restricted,
            body_part: body_part.map(String::from),
            injury_source: injury_source.map(String::from),
            event_type: event_type.map(String::from),
            environment_factor: environment_factor.map(String::from),
            involved_parties, witness_statements, attachments,
            resolution_date, closed_date, closed_by,
            metadata, created_by,
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_incident(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::SafetyIncident>> { Ok(None) }
    async fn get_incident_by_number(&self, _org_id: Uuid, _number: &str) -> AtlasResult<Option<atlas_shared::SafetyIncident>> { Ok(None) }
    async fn list_incidents(&self, _org_id: Uuid, _status: Option<&str>, _severity: Option<&str>, _incident_type: Option<&str>, _facility_id: Option<&Uuid>) -> AtlasResult<Vec<atlas_shared::SafetyIncident>> { Ok(vec![]) }
    async fn update_incident_status(&self, _id: Uuid, _status: &str) -> AtlasResult<atlas_shared::SafetyIncident> {
        Ok(atlas_shared::SafetyIncident {
            id: _id, organization_id: Uuid::new_v4(),
            incident_number: "MOCK".to_string(), title: "Mock".to_string(),
            description: None, incident_type: "injury".to_string(),
            severity: "medium".to_string(), status: _status.to_string(),
            priority: "medium".to_string(),
            incident_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            incident_time: None, location: None,
            facility_id: None, department_id: None,
            reported_by_id: None, reported_by_name: None,
            assigned_to_id: None, assigned_to_name: None,
            root_cause: None, immediate_action: None,
            osha_recordable: false, osha_classification: None,
            days_away_from_work: 0, days_restricted: 0,
            body_part: None, injury_source: None,
            event_type: None, environment_factor: None,
            involved_parties: serde_json::json!([]),
            witness_statements: serde_json::json!([]),
            attachments: serde_json::json!([]),
            resolution_date: None, closed_date: None, closed_by: None,
            metadata: serde_json::json!({}), created_by: None,
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn update_incident_investigation(&self, _id: Uuid, _root_cause: Option<&str>, _immediate_action: Option<&str>, _assigned_to_id: Option<Uuid>, _assigned_to_name: Option<&str>, _days_away: Option<i32>, _days_restricted: Option<i32>) -> AtlasResult<atlas_shared::SafetyIncident> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn close_incident(&self, _id: Uuid, _closed_by: Option<Uuid>) -> AtlasResult<atlas_shared::SafetyIncident> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn delete_incident(&self, _org_id: Uuid, _number: &str) -> AtlasResult<()> { Ok(()) }

    // Hazards
    async fn create_hazard(
        &self, org_id: Uuid, hazard_code: &str, title: &str, description: Option<&str>,
        hazard_category: &str, risk_level: &str, likelihood: &str, consequence: &str,
        risk_score: i32, status: &str,
        location: Option<&str>, facility_id: Option<Uuid>, department_id: Option<Uuid>,
        identified_by_id: Option<Uuid>, identified_by_name: Option<&str>,
        identified_date: chrono::NaiveDate,
        mitigation_measures: serde_json::Value,
        residual_risk_level: Option<&str>, residual_risk_score: Option<i32>,
        review_date: Option<chrono::NaiveDate>,
        owner_id: Option<Uuid>, owner_name: Option<&str>,
        metadata: serde_json::Value, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::Hazard> {
        Ok(atlas_shared::Hazard {
            id: Uuid::new_v4(), organization_id: org_id,
            hazard_code: hazard_code.to_string(),
            title: title.to_string(),
            description: description.map(String::from),
            hazard_category: hazard_category.to_string(),
            risk_level: risk_level.to_string(),
            likelihood: likelihood.to_string(),
            consequence: consequence.to_string(),
            risk_score, status: status.to_string(),
            location: location.map(String::from),
            facility_id, department_id,
            identified_by_id, identified_by_name: identified_by_name.map(String::from),
            identified_date, mitigation_measures,
            residual_risk_level: residual_risk_level.map(String::from),
            residual_risk_score, review_date,
            owner_id, owner_name: owner_name.map(String::from),
            metadata, created_by,
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_hazard(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::Hazard>> { Ok(None) }
    async fn get_hazard_by_code(&self, _org_id: Uuid, _code: &str) -> AtlasResult<Option<atlas_shared::Hazard>> { Ok(None) }
    async fn list_hazards(&self, _org_id: Uuid, _status: Option<&str>, _risk_level: Option<&str>, _hazard_category: Option<&str>, _facility_id: Option<&Uuid>) -> AtlasResult<Vec<atlas_shared::Hazard>> { Ok(vec![]) }
    async fn update_hazard_status(&self, _id: Uuid, _status: &str) -> AtlasResult<atlas_shared::Hazard> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn update_residual_risk(&self, _id: Uuid, _residual_risk_level: &str, _residual_risk_score: i32) -> AtlasResult<atlas_shared::Hazard> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn delete_hazard(&self, _org_id: Uuid, _code: &str) -> AtlasResult<()> { Ok(()) }

    // Inspections
    async fn create_inspection(
        &self, org_id: Uuid, inspection_number: &str, title: &str, description: Option<&str>,
        inspection_type: &str, status: &str, priority: &str,
        scheduled_date: chrono::NaiveDate, _completed_date: Option<chrono::NaiveDate>,
        location: Option<&str>, facility_id: Option<Uuid>, department_id: Option<Uuid>,
        inspector_id: Option<Uuid>, inspector_name: Option<&str>,
        findings_summary: Option<&str>,
        total_findings: i32, critical_findings: i32, non_conformities: i32, observations: i32,
        score: Option<f64>, max_score: Option<f64>, score_pct: Option<f64>,
        findings: serde_json::Value, attachments: serde_json::Value,
        metadata: serde_json::Value, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::SafetyInspection> {
        Ok(atlas_shared::SafetyInspection {
            id: Uuid::new_v4(), organization_id: org_id,
            inspection_number: inspection_number.to_string(),
            title: title.to_string(),
            description: description.map(String::from),
            inspection_type: inspection_type.to_string(),
            status: status.to_string(),
            priority: priority.to_string(),
            scheduled_date, completed_date: None,
            location: location.map(String::from),
            facility_id, department_id,
            inspector_id, inspector_name: inspector_name.map(String::from),
            findings_summary: findings_summary.map(String::from),
            total_findings, critical_findings, non_conformities, observations,
            score, max_score, score_pct,
            findings, attachments, metadata, created_by,
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_inspection(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::SafetyInspection>> { Ok(None) }
    async fn get_inspection_by_number(&self, _org_id: Uuid, _number: &str) -> AtlasResult<Option<atlas_shared::SafetyInspection>> { Ok(None) }
    async fn list_inspections(&self, _org_id: Uuid, _status: Option<&str>, _inspection_type: Option<&str>, _facility_id: Option<&Uuid>) -> AtlasResult<Vec<atlas_shared::SafetyInspection>> { Ok(vec![]) }
    async fn complete_inspection(&self, _id: Uuid, _findings_summary: Option<&str>, _total: i32, _critical: i32, _nc: i32, _obs: i32, _score: Option<f64>, _max_score: Option<f64>, _score_pct: Option<f64>, _findings: serde_json::Value) -> AtlasResult<atlas_shared::SafetyInspection> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn update_inspection_status(&self, _id: Uuid, _status: &str) -> AtlasResult<atlas_shared::SafetyInspection> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn delete_inspection(&self, _org_id: Uuid, _number: &str) -> AtlasResult<()> { Ok(()) }

    // CAPA
    async fn create_corrective_action(
        &self, org_id: Uuid, action_number: &str, title: &str, description: Option<&str>,
        action_type: &str, status: &str, priority: &str,
        source_type: Option<&str>, source_id: Option<Uuid>, source_number: Option<&str>,
        root_cause: Option<&str>, corrective_action_plan: Option<&str>, preventive_action_plan: Option<&str>,
        assigned_to_id: Option<Uuid>, assigned_to_name: Option<&str>,
        due_date: Option<chrono::NaiveDate>, completed_date: Option<chrono::NaiveDate>,
        verified_by: Option<Uuid>, verified_date: Option<chrono::NaiveDate>, effectiveness: Option<&str>,
        facility_id: Option<Uuid>, department_id: Option<Uuid>,
        estimated_cost: Option<f64>, actual_cost: Option<f64>, currency_code: Option<&str>,
        notes: Option<&str>, attachments: serde_json::Value,
        metadata: serde_json::Value, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::SafetyCorrectiveAction> {
        Ok(atlas_shared::SafetyCorrectiveAction {
            id: Uuid::new_v4(), organization_id: org_id,
            action_number: action_number.to_string(),
            title: title.to_string(),
            description: description.map(String::from),
            action_type: action_type.to_string(),
            status: status.to_string(),
            priority: priority.to_string(),
            source_type: source_type.map(String::from),
            source_id, source_number: source_number.map(String::from),
            root_cause: root_cause.map(String::from),
            corrective_action_plan: corrective_action_plan.map(String::from),
            preventive_action_plan: preventive_action_plan.map(String::from),
            assigned_to_id, assigned_to_name: assigned_to_name.map(String::from),
            due_date, completed_date, verified_by, verified_date,
            effectiveness: effectiveness.map(String::from),
            facility_id, department_id,
            estimated_cost, actual_cost, currency_code: currency_code.map(String::from),
            notes: notes.map(String::from), attachments, metadata, created_by,
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_corrective_action(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::SafetyCorrectiveAction>> { Ok(None) }
    async fn get_corrective_action_by_number(&self, _org_id: Uuid, _number: &str) -> AtlasResult<Option<atlas_shared::SafetyCorrectiveAction>> { Ok(None) }
    async fn list_corrective_actions(&self, _org_id: Uuid, _status: Option<&str>, _action_type: Option<&str>, _source_type: Option<&str>) -> AtlasResult<Vec<atlas_shared::SafetyCorrectiveAction>> { Ok(vec![]) }
    async fn update_corrective_action_status(&self, _id: Uuid, _status: &str) -> AtlasResult<atlas_shared::SafetyCorrectiveAction> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn complete_corrective_action(&self, _id: Uuid, _effectiveness: &str, _actual_cost: Option<f64>, _verified_by: Option<Uuid>) -> AtlasResult<atlas_shared::SafetyCorrectiveAction> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }
    async fn delete_corrective_action(&self, _org_id: Uuid, _number: &str) -> AtlasResult<()> { Ok(()) }

    // Dashboard
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<atlas_shared::HealthSafetyDashboard> {
        Ok(atlas_shared::HealthSafetyDashboard {
            organization_id: org_id,
            total_incidents: 0, open_incidents: 0, closed_incidents: 0, critical_incidents: 0,
            total_hazards: 0, open_hazards: 0, high_risk_hazards: 0,
            total_inspections: 0, open_inspections: 0, completed_inspections: 0,
            total_capa: 0, open_capa: 0, overdue_capa: 0,
            osha_recordable_count: 0, days_since_last_incident: -1,
            incidents_by_type: serde_json::json!({}),
            incidents_by_severity: serde_json::json!({}),
            hazards_by_risk: serde_json::json!({}),
            inspection_pass_rate: 0.0,
        })
    }
}

/// Mock Funds Reservation Repository for testing
pub struct MockFundsReservationRepository;

use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    static FR_STORE: RefCell<HashMap<String, atlas_shared::FundReservation>> = RefCell::new(HashMap::new());
}

#[async_trait]
impl crate::funds_reservation::FundsReservationRepository for MockFundsReservationRepository {
    async fn create_reservation(
        &self, org_id: Uuid, reservation_number: &str,
        budget_id: Uuid, budget_code: &str, budget_version_id: Option<Uuid>,
        description: Option<&str>,
        source_type: Option<&str>, source_id: Option<Uuid>, source_number: Option<&str>,
        reserved_amount: f64, currency_code: &str,
        reservation_date: chrono::NaiveDate, expiry_date: Option<chrono::NaiveDate>,
        status: &str, control_level: &str,
        fiscal_year: Option<i32>, period_name: Option<&str>,
        department_id: Option<Uuid>, department_name: Option<&str>,
        fund_check_passed: bool, fund_check_message: Option<&str>,
        metadata: serde_json::Value, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::FundReservation> {
        let fr = atlas_shared::FundReservation {
            id: Uuid::new_v4(), organization_id: org_id,
            reservation_number: reservation_number.to_string(),
            budget_id, budget_code: budget_code.to_string(), budget_version_id,
            description: description.map(String::from),
            source_type: source_type.map(String::from),
            source_id, source_number: source_number.map(String::from),
            reserved_amount, consumed_amount: 0.0, released_amount: 0.0, remaining_amount: reserved_amount,
            currency_code: currency_code.to_string(),
            reservation_date, expiry_date,
            status: status.to_string(), control_level: control_level.to_string(),
            fiscal_year, period_name: period_name.map(String::from),
            department_id, department_name: department_name.map(String::from),
            fund_check_passed, fund_check_message: fund_check_message.map(String::from),
            metadata, created_by,
            approved_by: None, cancelled_by: None, cancelled_at: None, cancellation_reason: None,
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        };
        FR_STORE.with(|s| s.borrow_mut().insert(reservation_number.to_string(), fr.clone()));
        Ok(fr)
    }

    async fn get_reservation(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::FundReservation>> {
        FR_STORE.with(|s| Ok(s.borrow().values().find(|r| r.id == _id).cloned()))
    }

    async fn get_reservation_by_number(&self, _org_id: Uuid, reservation_number: &str) -> AtlasResult<Option<atlas_shared::FundReservation>> {
        FR_STORE.with(|s| Ok(s.borrow().get(reservation_number).cloned()))
    }

    async fn list_reservations(
        &self, _org_id: Uuid, _status: Option<&str>, _budget_id: Option<&Uuid>, _department_id: Option<&Uuid>,
    ) -> AtlasResult<Vec<atlas_shared::FundReservation>> {
        FR_STORE.with(|s| Ok(s.borrow().values().cloned().collect()))
    }

    async fn update_reservation_status(&self, id: Uuid, status: &str) -> AtlasResult<atlas_shared::FundReservation> {
        FR_STORE.with(|s| {
            let mut store = s.borrow_mut();
            for (_, fr) in store.iter_mut() {
                if fr.id == id {
                    fr.status = status.to_string();
                    fr.updated_at = chrono::Utc::now();
                    return Ok(fr.clone());
                }
            }
            Err(AtlasError::EntityNotFound(format!("Reservation {} not found", id)))
        })
    }

    async fn update_reservation_amounts(
        &self, id: Uuid, consumed_amount: f64, released_amount: f64, remaining_amount: f64,
    ) -> AtlasResult<atlas_shared::FundReservation> {
        FR_STORE.with(|s| {
            let mut store = s.borrow_mut();
            for (_, fr) in store.iter_mut() {
                if fr.id == id {
                    fr.consumed_amount = consumed_amount;
                    fr.released_amount = released_amount;
                    fr.remaining_amount = remaining_amount;
                    fr.updated_at = chrono::Utc::now();
                    return Ok(fr.clone());
                }
            }
            Err(AtlasError::EntityNotFound(format!("Reservation {} not found", id)))
        })
    }

    async fn cancel_reservation(
        &self, id: Uuid, cancelled_by: Option<Uuid>, reason: Option<&str>,
    ) -> AtlasResult<atlas_shared::FundReservation> {
        FR_STORE.with(|s| {
            let mut store = s.borrow_mut();
            for (_, fr) in store.iter_mut() {
                if fr.id == id {
                    fr.status = "cancelled".to_string();
                    fr.remaining_amount = 0.0;
                    fr.released_amount = fr.reserved_amount - fr.consumed_amount;
                    fr.cancelled_by = cancelled_by;
                    fr.cancelled_at = Some(chrono::Utc::now());
                    fr.cancellation_reason = reason.map(String::from);
                    fr.updated_at = chrono::Utc::now();
                    return Ok(fr.clone());
                }
            }
            Err(AtlasError::EntityNotFound(format!("Reservation {} not found", id)))
        })
    }

    async fn delete_reservation(&self, _org_id: Uuid, reservation_number: &str) -> AtlasResult<()> {
        FR_STORE.with(|s| s.borrow_mut().remove(reservation_number));
        Ok(())
    }

    // Lines - minimal mock
    async fn create_reservation_line(
        &self, org_id: Uuid, reservation_id: Uuid, line_number: i32,
        account_code: &str, account_description: Option<&str>,
        budget_line_id: Option<Uuid>, department_id: Option<Uuid>,
        project_id: Option<Uuid>, cost_center: Option<&str>,
        reserved_amount: f64, metadata: serde_json::Value,
    ) -> AtlasResult<atlas_shared::FundReservationLine> {
        Ok(atlas_shared::FundReservationLine {
            id: Uuid::new_v4(), organization_id: org_id, reservation_id, line_number,
            account_code: account_code.to_string(),
            account_description: account_description.map(String::from),
            budget_line_id, department_id, project_id,
            cost_center: cost_center.map(String::from),
            reserved_amount, consumed_amount: 0.0, released_amount: 0.0, remaining_amount: reserved_amount,
            metadata, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }

    async fn list_reservation_lines(&self, _reservation_id: Uuid) -> AtlasResult<Vec<atlas_shared::FundReservationLine>> { Ok(vec![]) }

    async fn update_reservation_line_amounts(
        &self, _id: Uuid, _consumed_amount: f64, _released_amount: f64, _remaining_amount: f64,
    ) -> AtlasResult<atlas_shared::FundReservationLine> {
        Err(AtlasError::EntityNotFound("Mock".to_string()))
    }

    // Fund availability - return generous defaults
    async fn check_fund_availability(
        &self, org_id: Uuid, budget_id: Uuid, account_code: &str,
        as_of_date: chrono::NaiveDate, fiscal_year: Option<i32>, period_name: Option<&str>,
    ) -> AtlasResult<atlas_shared::FundAvailability> {
        Ok(atlas_shared::FundAvailability {
            organization_id: org_id, budget_id, budget_code: "MOCK".to_string(),
            account_code: account_code.to_string(),
            budget_amount: 1_000_000.0, total_reserved: 0.0, total_consumed: 0.0, total_released: 0.0,
            available_balance: 1_000_000.0, check_passed: true,
            control_level: "advisory".to_string(),
            message: "Funds available (mock)".to_string(),
            as_of_date, fiscal_year, period_name: period_name.map(String::from),
        })
    }

    // Dashboard
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<atlas_shared::BudgetaryControlDashboard> {
        Ok(atlas_shared::BudgetaryControlDashboard {
            organization_id: org_id,
            total_reservations: 0, active_reservations: 0,
            total_reserved_amount: 0.0, total_consumed_amount: 0.0,
            total_released_amount: 0.0, total_available_amount: 0.0,
            reservations_by_status: serde_json::json!({}),
            top_departments_by_reservation: serde_json::json!([]),
            budget_utilization_pct: 0.0,
        })
    }
}

/// Mock Rebate Management repository
pub struct MockRebateManagementRepository;

#[async_trait]
impl crate::rebate_management::RebateManagementRepository for MockRebateManagementRepository {
    async fn create_agreement(
        &self, org_id: Uuid, agreement_number: &str, name: &str, _description: Option<&str>,
        rebate_type: &str, direction: &str, partner_type: &str,
        partner_id: Option<Uuid>, partner_name: Option<&str>, partner_number: Option<&str>,
        product_category: Option<&str>, product_id: Option<Uuid>, product_name: Option<&str>,
        uom: Option<&str>, currency_code: &str,
        start_date: chrono::NaiveDate, end_date: chrono::NaiveDate,
        calculation_method: &str,
        accrual_account: Option<&str>, liability_account: Option<&str>, expense_account: Option<&str>,
        payment_terms: Option<&str>, settlement_frequency: Option<&str>,
        minimum_amount: f64, maximum_amount: Option<f64>,
        auto_accrue: bool, requires_approval: bool,
        notes: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::RebateAgreement> {
        Ok(atlas_shared::RebateAgreement {
            id: Uuid::new_v4(), organization_id: org_id,
            agreement_number: agreement_number.to_string(), name: name.to_string(),
            description: _description.unwrap_or("").to_string(),
            rebate_type: rebate_type.to_string(), direction: direction.to_string(),
            partner_type: partner_type.to_string(),
            partner_id, partner_name: partner_name.unwrap_or("").to_string(),
            partner_number: partner_number.unwrap_or("").to_string(),
            product_category: product_category.unwrap_or("").to_string(),
            product_id, product_name: product_name.unwrap_or("").to_string(),
            uom: uom.unwrap_or("").to_string(),
            currency_code: currency_code.to_string(),
            start_date, end_date, status: "draft".to_string(),
            calculation_method: calculation_method.to_string(),
            accrual_account: accrual_account.unwrap_or("").to_string(),
            liability_account: liability_account.unwrap_or("").to_string(),
            expense_account: expense_account.unwrap_or("").to_string(),
            payment_terms: payment_terms.unwrap_or("").to_string(),
            settlement_frequency: settlement_frequency.unwrap_or("").to_string(),
            minimum_amount, maximum_amount,
            auto_accrue, requires_approval,
            approved_by: None, approved_at: None,
            notes: notes.unwrap_or("").to_string(),
            metadata: serde_json::json!({}), created_by,
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_agreement(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::RebateAgreement>> { Ok(None) }
    async fn get_agreement_by_number(&self, _org_id: Uuid, _agreement_number: &str) -> AtlasResult<Option<atlas_shared::RebateAgreement>> { Ok(None) }
    async fn list_agreements(&self, _org_id: Uuid, _status: Option<&str>, _rebate_type: Option<&str>, _partner_type: Option<&str>) -> AtlasResult<Vec<atlas_shared::RebateAgreement>> { Ok(vec![]) }
    async fn update_agreement_status(&self, id: Uuid, _status: &str) -> AtlasResult<atlas_shared::RebateAgreement> {
        Err(AtlasError::EntityNotFound(format!("Agreement {} not found", id)))
    }
    async fn delete_agreement(&self, _org_id: Uuid, _agreement_number: &str) -> AtlasResult<()> { Ok(()) }
    async fn create_tier(&self, org_id: Uuid, agreement_id: Uuid, tier_number: i32, from_value: f64, to_value: Option<f64>, rebate_rate: f64, rate_type: &str, description: Option<&str>) -> AtlasResult<atlas_shared::RebateTier> {
        Ok(atlas_shared::RebateTier {
            id: Uuid::new_v4(), organization_id: org_id, agreement_id, tier_number,
            from_value, to_value, rebate_rate, rate_type: rate_type.to_string(),
            description: description.unwrap_or("").to_string(),
            metadata: serde_json::json!({}), created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn list_tiers(&self, _agreement_id: Uuid) -> AtlasResult<Vec<atlas_shared::RebateTier>> { Ok(vec![]) }
    async fn delete_tier(&self, _id: Uuid) -> AtlasResult<()> { Ok(()) }
    async fn create_transaction(
        &self, org_id: Uuid, agreement_id: Uuid, transaction_number: &str,
        source_type: Option<&str>, source_id: Option<Uuid>, source_number: Option<&str>,
        transaction_date: chrono::NaiveDate, product_id: Option<Uuid>, product_name: Option<&str>,
        quantity: f64, unit_price: f64, transaction_amount: f64,
        currency_code: &str, applicable_rate: f64, rebate_amount: f64,
        tier_id: Option<Uuid>, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::RebateTransaction> {
        Ok(atlas_shared::RebateTransaction {
            id: Uuid::new_v4(), organization_id: org_id, agreement_id,
            transaction_number: transaction_number.to_string(),
            source_type: source_type.unwrap_or("").to_string(),
            source_id, source_number: source_number.unwrap_or("").to_string(),
            transaction_date, product_id, product_name: product_name.unwrap_or("").to_string(),
            quantity, unit_price, transaction_amount, currency_code: currency_code.to_string(),
            applicable_rate, rebate_amount, status: "eligible".to_string(), tier_id,
            excluded_reason: String::new(), metadata: serde_json::json!({}), created_by,
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_transaction(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::RebateTransaction>> { Ok(None) }
    async fn get_transaction_by_number(&self, _org_id: Uuid, _transaction_number: &str) -> AtlasResult<Option<atlas_shared::RebateTransaction>> { Ok(None) }
    async fn list_transactions(&self, _agreement_id: Uuid, _status: Option<&str>) -> AtlasResult<Vec<atlas_shared::RebateTransaction>> { Ok(vec![]) }
    async fn update_transaction_status(&self, id: Uuid, _status: &str, _reason: Option<&str>) -> AtlasResult<atlas_shared::RebateTransaction> {
        Err(AtlasError::EntityNotFound(format!("Transaction {} not found", id)))
    }
    async fn delete_transaction(&self, _org_id: Uuid, _transaction_number: &str) -> AtlasResult<()> { Ok(()) }
    async fn create_accrual(
        &self, org_id: Uuid, agreement_id: Uuid, accrual_number: &str,
        accrual_date: chrono::NaiveDate, accrual_period: Option<&str>,
        accumulated_quantity: f64, accumulated_amount: f64,
        applicable_tier_id: Option<Uuid>, applicable_rate: f64, accrued_amount: f64,
        currency_code: &str, notes: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::RebateAccrual> {
        Ok(atlas_shared::RebateAccrual {
            id: Uuid::new_v4(), organization_id: org_id, agreement_id,
            accrual_number: accrual_number.to_string(), accrual_date,
            accrual_period: accrual_period.unwrap_or("").to_string(),
            accumulated_quantity, accumulated_amount,
            applicable_tier_id, applicable_rate, accrued_amount,
            currency_code: currency_code.to_string(),
            gl_posted: false, gl_journal_id: None, status: "draft".to_string(),
            notes: notes.unwrap_or("").to_string(),
            metadata: serde_json::json!({}), created_by,
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_accrual(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::RebateAccrual>> { Ok(None) }
    async fn get_accrual_by_number(&self, _org_id: Uuid, _accrual_number: &str) -> AtlasResult<Option<atlas_shared::RebateAccrual>> { Ok(None) }
    async fn list_accruals(&self, _agreement_id: Uuid, _status: Option<&str>) -> AtlasResult<Vec<atlas_shared::RebateAccrual>> { Ok(vec![]) }
    async fn update_accrual_status(&self, id: Uuid, _status: &str) -> AtlasResult<atlas_shared::RebateAccrual> {
        Err(AtlasError::EntityNotFound(format!("Accrual {} not found", id)))
    }
    async fn delete_accrual(&self, _org_id: Uuid, _accrual_number: &str) -> AtlasResult<()> { Ok(()) }
    async fn create_settlement(
        &self, org_id: Uuid, agreement_id: Uuid, settlement_number: &str,
        settlement_date: chrono::NaiveDate, settlement_period_from: Option<chrono::NaiveDate>,
        settlement_period_to: Option<chrono::NaiveDate>,
        total_qualifying_amount: f64, total_qualifying_quantity: f64,
        applicable_tier_id: Option<Uuid>, applicable_rate: f64, settlement_amount: f64,
        currency_code: &str, settlement_type: &str, payment_method: Option<&str>,
        notes: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::RebateSettlement> {
        Ok(atlas_shared::RebateSettlement {
            id: Uuid::new_v4(), organization_id: org_id, agreement_id,
            settlement_number: settlement_number.to_string(), settlement_date,
            settlement_period_from, settlement_period_to,
            total_qualifying_amount, total_qualifying_quantity,
            applicable_tier_id, applicable_rate, settlement_amount,
            currency_code: currency_code.to_string(),
            settlement_type: settlement_type.to_string(),
            payment_method: payment_method.unwrap_or("").to_string(),
            payment_reference: String::new(), status: "pending".to_string(),
            approved_by: None, approved_at: None, paid_at: None,
            notes: notes.unwrap_or("").to_string(),
            metadata: serde_json::json!({}), created_by,
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        })
    }
    async fn get_settlement(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::RebateSettlement>> { Ok(None) }
    async fn get_settlement_by_number(&self, _org_id: Uuid, _settlement_number: &str) -> AtlasResult<Option<atlas_shared::RebateSettlement>> { Ok(None) }
    async fn list_settlements(&self, _agreement_id: Uuid, _status: Option<&str>) -> AtlasResult<Vec<atlas_shared::RebateSettlement>> { Ok(vec![]) }
    async fn update_settlement_status(&self, id: Uuid, _status: &str) -> AtlasResult<atlas_shared::RebateSettlement> {
        Err(AtlasError::EntityNotFound(format!("Settlement {} not found", id)))
    }
    async fn approve_settlement(&self, id: Uuid, _approved_by: Uuid) -> AtlasResult<atlas_shared::RebateSettlement> {
        Err(AtlasError::EntityNotFound(format!("Settlement {} not found", id)))
    }
    async fn pay_settlement(&self, id: Uuid) -> AtlasResult<atlas_shared::RebateSettlement> {
        Err(AtlasError::EntityNotFound(format!("Settlement {} not found", id)))
    }
    async fn delete_settlement(&self, _org_id: Uuid, _settlement_number: &str) -> AtlasResult<()> { Ok(()) }
    async fn create_settlement_line(&self, settlement_id: Uuid, transaction_id: Uuid, amount: f64) -> AtlasResult<atlas_shared::RebateSettlementLine> {
        Ok(atlas_shared::RebateSettlementLine {
            id: Uuid::new_v4(), settlement_id, transaction_id, settlement_amount: amount,
            metadata: serde_json::json!({}), created_at: chrono::Utc::now(),
        })
    }
    async fn list_settlement_lines(&self, _settlement_id: Uuid) -> AtlasResult<Vec<atlas_shared::RebateSettlementLine>> { Ok(vec![]) }
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<atlas_shared::RebateDashboard> {
        Ok(atlas_shared::RebateDashboard {
            organization_id: org_id,
            total_agreements: 0, active_agreements: 0, total_transactions: 0,
            total_qualifying_amount: 0.0, total_accrued_amount: 0.0, total_settled_amount: 0.0,
            pending_settlements: 0, agreements_by_type: serde_json::json!({}),
            top_rebate_agreements: serde_json::json!([]), recent_settlements: serde_json::json!([]),
        })
    }
}

/// Mock asset depreciation repository for testing
pub struct MockAssetDepreciationRepository;

#[async_trait]
impl crate::asset_depreciation::AssetDepreciationRepository for MockAssetDepreciationRepository {
    async fn get_asset(&self, _id: Uuid) -> AtlasResult<Option<atlas_shared::FixedAsset>> { Ok(None) }
    async fn create_depreciation_history(
        &self, org_id: Uuid, asset_id: Uuid, fiscal_year: i32, period_number: i32,
        period_name: Option<&str>, depreciation_date: chrono::NaiveDate,
        depreciation_amount: &str, accumulated_depreciation: &str, net_book_value: &str,
        depreciation_method: &str, created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::AssetDepreciationHistory> {
        Ok(atlas_shared::AssetDepreciationHistory {
            id: Uuid::new_v4(), organization_id: org_id, asset_id,
            fiscal_year, period_number, period_name: period_name.map(String::from),
            depreciation_date,
            depreciation_amount: depreciation_amount.to_string(),
            accumulated_depreciation: accumulated_depreciation.to_string(),
            net_book_value: net_book_value.to_string(),
            depreciation_method: depreciation_method.to_string(),
            journal_entry_id: None, created_by, created_at: chrono::Utc::now(),
        })
    }
    async fn list_depreciation_history(&self, _asset_id: Uuid) -> AtlasResult<Vec<atlas_shared::AssetDepreciationHistory>> { Ok(vec![]) }
    async fn update_asset_depreciation(
        &self, _asset_id: Uuid, _accumulated_depreciation: &str, _net_book_value: &str,
        _last_depreciation_amount: &str, _periods_depreciated: i32, _last_depreciation_date: chrono::NaiveDate,
    ) -> AtlasResult<()> { Ok(()) }
}
