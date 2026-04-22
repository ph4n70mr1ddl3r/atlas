//! Subledger Accounting Repository
//!
//! PostgreSQL storage for accounting methods, derivation rules,
//! subledger journal entries, journal lines, SLA events, and GL transfer logs.

use atlas_shared::{
    AccountingMethod, AccountingDerivationRule,
    SubledgerJournalEntry, SubledgerJournalLine,
    SlaEvent, GlTransferLog,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

/// Repository trait for Subledger Accounting data storage
#[async_trait]
pub trait SubledgerAccountingRepository: Send + Sync {
    // ========================================================================
    // Accounting Methods
    // ========================================================================

    async fn create_accounting_method(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        application: &str,
        transaction_type: &str,
        event_class: &str,
        auto_accounting: bool,
        allow_manual_entries: bool,
        apply_rounding: bool,
        rounding_account_code: Option<&str>,
        rounding_threshold: &str,
        require_balancing: bool,
        intercompany_balancing_account: Option<&str>,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AccountingMethod>;

    async fn get_accounting_method(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<AccountingMethod>>;
    async fn get_accounting_method_by_id(&self, id: Uuid) -> AtlasResult<Option<AccountingMethod>>;
    async fn list_accounting_methods(&self, org_id: Uuid, application: Option<&str>) -> AtlasResult<Vec<AccountingMethod>>;
    async fn delete_accounting_method(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // ========================================================================
    // Accounting Derivation Rules
    // ========================================================================

    async fn create_derivation_rule(
        &self,
        org_id: Uuid,
        accounting_method_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        line_type: &str,
        priority: i32,
        conditions: serde_json::Value,
        source_field: Option<&str>,
        derivation_type: &str,
        fixed_account_code: Option<&str>,
        account_derivation_lookup: serde_json::Value,
        formula_expression: Option<&str>,
        sequence: i32,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AccountingDerivationRule>;

    async fn get_derivation_rule(&self, org_id: Uuid, method_id: Uuid, code: &str) -> AtlasResult<Option<AccountingDerivationRule>>;
    async fn list_derivation_rules(&self, org_id: Uuid, method_id: Uuid) -> AtlasResult<Vec<AccountingDerivationRule>>;
    async fn list_active_derivation_rules(&self, org_id: Uuid, method_id: Uuid, line_type: &str) -> AtlasResult<Vec<AccountingDerivationRule>>;
    async fn delete_derivation_rule(&self, org_id: Uuid, method_id: Uuid, code: &str) -> AtlasResult<()>;

    // ========================================================================
    // Subledger Journal Entries
    // ========================================================================

    async fn create_journal_entry(
        &self,
        org_id: Uuid,
        source_application: &str,
        source_transaction_type: &str,
        source_transaction_id: Uuid,
        source_transaction_number: Option<&str>,
        accounting_method_id: Option<Uuid>,
        entry_number: &str,
        description: Option<&str>,
        reference_number: Option<&str>,
        accounting_date: chrono::NaiveDate,
        period_name: Option<&str>,
        currency_code: &str,
        entered_currency_code: &str,
        currency_conversion_date: Option<chrono::NaiveDate>,
        currency_conversion_type: Option<&str>,
        currency_conversion_rate: Option<&str>,
        total_debit: &str,
        total_credit: &str,
        entered_debit: &str,
        entered_credit: &str,
        status: &str,
        balancing_segment: Option<&str>,
        is_balanced: bool,
        created_by: Option<Uuid>,
    ) -> AtlasResult<SubledgerJournalEntry>;

    async fn get_journal_entry(&self, id: Uuid) -> AtlasResult<Option<SubledgerJournalEntry>>;
    async fn get_journal_entry_by_number(&self, org_id: Uuid, entry_number: &str) -> AtlasResult<Option<SubledgerJournalEntry>>;
    async fn list_journal_entries(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        source_application: Option<&str>,
        source_transaction_type: Option<&str>,
        accounting_date_from: Option<chrono::NaiveDate>,
        accounting_date_to: Option<chrono::NaiveDate>,
    ) -> AtlasResult<Vec<SubledgerJournalEntry>>;

    async fn update_journal_entry_status(
        &self,
        id: Uuid,
        status: &str,
        error_message: Option<&str>,
        is_balanced: Option<bool>,
        posted_by: Option<Uuid>,
        accounted_by: Option<Uuid>,
    ) -> AtlasResult<SubledgerJournalEntry>;

    async fn update_journal_entry_balances(
        &self,
        id: Uuid,
        total_debit: &str,
        total_credit: &str,
        entered_debit: &str,
        entered_credit: &str,
        is_balanced: bool,
    ) -> AtlasResult<SubledgerJournalEntry>;

    // ========================================================================
    // Subledger Journal Lines
    // ========================================================================

    async fn create_journal_line(
        &self,
        org_id: Uuid,
        journal_entry_id: Uuid,
        line_number: i32,
        line_type: &str,
        account_code: &str,
        account_description: Option<&str>,
        derivation_rule_id: Option<Uuid>,
        entered_amount: &str,
        accounted_amount: &str,
        currency_code: &str,
        conversion_date: Option<chrono::NaiveDate>,
        conversion_rate: Option<&str>,
        attribute_category: Option<&str>,
        attribute1: Option<&str>,
        attribute2: Option<&str>,
        attribute3: Option<&str>,
        attribute4: Option<&str>,
        attribute5: Option<&str>,
        source_line_id: Option<Uuid>,
        source_line_type: Option<&str>,
        tax_code: Option<&str>,
        tax_rate: Option<&str>,
        tax_amount: Option<&str>,
    ) -> AtlasResult<SubledgerJournalLine>;

    async fn list_journal_lines(&self, journal_entry_id: Uuid) -> AtlasResult<Vec<SubledgerJournalLine>>;
    async fn delete_journal_line(&self, id: Uuid) -> AtlasResult<()>;

    // ========================================================================
    // SLA Events
    // ========================================================================

    async fn create_sla_event(
        &self,
        org_id: Uuid,
        event_number: &str,
        event_type: &str,
        source_application: &str,
        source_transaction_type: &str,
        source_transaction_id: Uuid,
        journal_entry_id: Option<Uuid>,
        event_date: chrono::NaiveDate,
        event_status: &str,
        description: Option<&str>,
        error_message: Option<&str>,
        processed_by: Option<Uuid>,
    ) -> AtlasResult<SlaEvent>;

    async fn list_sla_events(
        &self,
        org_id: Uuid,
        source_application: Option<&str>,
        event_type: Option<&str>,
    ) -> AtlasResult<Vec<SlaEvent>>;

    // ========================================================================
    // GL Transfer Log
    // ========================================================================

    async fn create_transfer_log(
        &self,
        org_id: Uuid,
        transfer_number: &str,
        from_period: Option<&str>,
        status: &str,
        total_entries: i32,
        total_debit: &str,
        total_credit: &str,
        included_applications: serde_json::Value,
        transferred_by: Option<Uuid>,
        entries: serde_json::Value,
    ) -> AtlasResult<GlTransferLog>;

    async fn update_transfer_log_status(
        &self,
        id: Uuid,
        status: &str,
        error_message: Option<&str>,
        completed_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> AtlasResult<GlTransferLog>;

    async fn get_transfer_log(&self, id: Uuid) -> AtlasResult<Option<GlTransferLog>>;
    async fn list_transfer_logs(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<GlTransferLog>>;

    // ========================================================================
    // Dashboard
    // ========================================================================

    async fn count_entries_by_status(&self, org_id: Uuid) -> AtlasResult<serde_json::Value>;
}

/// PostgreSQL implementation of the Subledger Accounting repository
pub struct PostgresSubledgerAccountingRepository {
    pool: PgPool,
}

impl PostgresSubledgerAccountingRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

macro_rules! row_to_method {
    ($row:expr) => {{
        AccountingMethod {
            id: $row.get("id"),
            organization_id: $row.get("organization_id"),
            code: $row.get("code"),
            name: $row.get("name"),
            description: $row.get("description"),
            application: $row.get("application"),
            transaction_type: $row.get("transaction_type"),
            event_class: $row.get("event_class"),
            auto_accounting: $row.get("auto_accounting"),
            allow_manual_entries: $row.get("allow_manual_entries"),
            apply_rounding: $row.get("apply_rounding"),
            rounding_account_code: $row.get("rounding_account_code"),
            rounding_threshold: $row.get::<Option<String>, _>("rounding_threshold").unwrap_or_else(|| "0.01".to_string()),
            require_balancing: $row.get("require_balancing"),
            intercompany_balancing_account: $row.get("intercompany_balancing_account"),
            effective_from: $row.get("effective_from"),
            effective_to: $row.get("effective_to"),
            is_active: $row.get("is_active"),
            metadata: $row.get("metadata"),
            created_by: $row.get("created_by"),
            created_at: $row.get("created_at"),
            updated_at: $row.get("updated_at"),
        }
    }};
}

macro_rules! row_to_entry {
    ($row:expr) => {{
        SubledgerJournalEntry {
            id: $row.get("id"),
            organization_id: $row.get("organization_id"),
            source_application: $row.get("source_application"),
            source_transaction_type: $row.get("source_transaction_type"),
            source_transaction_id: $row.get("source_transaction_id"),
            source_transaction_number: $row.get("source_transaction_number"),
            accounting_method_id: $row.get("accounting_method_id"),
            entry_number: $row.get("entry_number"),
            description: $row.get("description"),
            reference_number: $row.get("reference_number"),
            accounting_date: $row.get("accounting_date"),
            period_name: $row.get("period_name"),
            currency_code: $row.get("currency_code"),
            entered_currency_code: $row.get("entered_currency_code"),
            currency_conversion_date: $row.get("currency_conversion_date"),
            currency_conversion_type: $row.get("currency_conversion_type"),
            currency_conversion_rate: $row.get::<Option<String>, _>("currency_conversion_rate"),
            total_debit: $row.get::<String, _>("total_debit"),
            total_credit: $row.get::<String, _>("total_credit"),
            entered_debit: $row.get::<String, _>("entered_debit"),
            entered_credit: $row.get::<String, _>("entered_credit"),
            status: $row.get("status"),
            error_message: $row.get("error_message"),
            balancing_segment: $row.get("balancing_segment"),
            is_balanced: $row.get("is_balanced"),
            gl_transfer_status: $row.get("gl_transfer_status"),
            gl_transfer_date: $row.get("gl_transfer_date"),
            gl_journal_entry_id: $row.get("gl_journal_entry_id"),
            is_reversal: $row.get("is_reversal"),
            reversal_of_id: $row.get("reversal_of_id"),
            reversal_reason: $row.get("reversal_reason"),
            created_by: $row.get("created_by"),
            posted_by: $row.get("posted_by"),
            accounted_by: $row.get("accounted_by"),
            created_at: $row.get("created_at"),
            updated_at: $row.get("updated_at"),
        }
    }};
}

macro_rules! row_to_line {
    ($row:expr) => {{
        SubledgerJournalLine {
            id: $row.get("id"),
            organization_id: $row.get("organization_id"),
            journal_entry_id: $row.get("journal_entry_id"),
            line_number: $row.get("line_number"),
            line_type: $row.get("line_type"),
            account_code: $row.get("account_code"),
            account_description: $row.get("account_description"),
            derivation_rule_id: $row.get("derivation_rule_id"),
            entered_amount: $row.get::<String, _>("entered_amount"),
            accounted_amount: $row.get::<String, _>("accounted_amount"),
            currency_code: $row.get("currency_code"),
            conversion_date: $row.get("conversion_date"),
            conversion_rate: $row.get::<Option<String>, _>("conversion_rate"),
            attribute_category: $row.get("attribute_category"),
            attribute1: $row.get("attribute1"),
            attribute2: $row.get("attribute2"),
            attribute3: $row.get("attribute3"),
            attribute4: $row.get("attribute4"),
            attribute5: $row.get("attribute5"),
            attribute6: $row.get("attribute6"),
            attribute7: $row.get("attribute7"),
            attribute8: $row.get("attribute8"),
            attribute9: $row.get("attribute9"),
            attribute10: $row.get("attribute10"),
            tax_code: $row.get("tax_code"),
            tax_rate: $row.get::<Option<String>, _>("tax_rate"),
            tax_amount: $row.get::<Option<String>, _>("tax_amount"),
            source_line_id: $row.get("source_line_id"),
            source_line_type: $row.get("source_line_type"),
            is_reversal_line: $row.get("is_reversal_line"),
            reversal_of_line_id: $row.get("reversal_of_line_id"),
            metadata: $row.get("metadata"),
            created_at: $row.get("created_at"),
            updated_at: $row.get("updated_at"),
        }
    }};
}

macro_rules! row_to_rule {
    ($row:expr) => {{
        AccountingDerivationRule {
            id: $row.get("id"),
            organization_id: $row.get("organization_id"),
            accounting_method_id: $row.get("accounting_method_id"),
            code: $row.get("code"),
            name: $row.get("name"),
            description: $row.get("description"),
            line_type: $row.get("line_type"),
            priority: $row.get("priority"),
            conditions: $row.get("conditions"),
            source_field: $row.get("source_field"),
            derivation_type: $row.get("derivation_type"),
            fixed_account_code: $row.get("fixed_account_code"),
            account_derivation_lookup: $row.get("account_derivation_lookup"),
            formula_expression: $row.get("formula_expression"),
            sequence: $row.get("sequence"),
            is_active: $row.get("is_active"),
            effective_from: $row.get("effective_from"),
            effective_to: $row.get("effective_to"),
            metadata: $row.get("metadata"),
            created_by: $row.get("created_by"),
            created_at: $row.get("created_at"),
            updated_at: $row.get("updated_at"),
        }
    }};
}

macro_rules! row_to_event {
    ($row:expr) => {{
        SlaEvent {
            id: $row.get("id"),
            organization_id: $row.get("organization_id"),
            event_number: $row.get("event_number"),
            event_type: $row.get("event_type"),
            source_application: $row.get("source_application"),
            source_transaction_type: $row.get("source_transaction_type"),
            source_transaction_id: $row.get("source_transaction_id"),
            journal_entry_id: $row.get("journal_entry_id"),
            event_date: $row.get("event_date"),
            event_status: $row.get("event_status"),
            description: $row.get("description"),
            error_message: $row.get("error_message"),
            processed_by: $row.get("processed_by"),
            processed_at: $row.get("processed_at"),
            metadata: $row.get("metadata"),
            created_at: $row.get("created_at"),
        }
    }};
}

macro_rules! row_to_transfer {
    ($row:expr) => {{
        GlTransferLog {
            id: $row.get("id"),
            organization_id: $row.get("organization_id"),
            transfer_number: $row.get("transfer_number"),
            transfer_date: $row.get("transfer_date"),
            from_period: $row.get("from_period"),
            status: $row.get("status"),
            error_message: $row.get("error_message"),
            total_entries: $row.get::<Option<i32>, _>("total_entries").unwrap_or(0),
            total_debit: $row.get::<Option<String>, _>("total_debit").unwrap_or_else(|| "0.00".to_string()),
            total_credit: $row.get::<Option<String>, _>("total_credit").unwrap_or_else(|| "0.00".to_string()),
            included_applications: $row.get("included_applications"),
            transferred_by: $row.get("transferred_by"),
            completed_at: $row.get("completed_at"),
            entries: $row.get("entries"),
            metadata: $row.get("metadata"),
            created_at: $row.get("created_at"),
            updated_at: $row.get("updated_at"),
        }
    }};
}

#[async_trait]
impl SubledgerAccountingRepository for PostgresSubledgerAccountingRepository {
    // ========================================================================
    // Accounting Methods
    // ========================================================================

    async fn create_accounting_method(
        &self,
        org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        application: &str, transaction_type: &str, event_class: &str,
        auto_accounting: bool, allow_manual_entries: bool, apply_rounding: bool,
        rounding_account_code: Option<&str>, rounding_threshold: &str,
        require_balancing: bool, intercompany_balancing_account: Option<&str>,
        effective_from: Option<chrono::NaiveDate>, effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AccountingMethod> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.accounting_methods
                (organization_id, code, name, description, application, transaction_type,
                 event_class, auto_accounting, allow_manual_entries, apply_rounding,
                 rounding_account_code, rounding_threshold, require_balancing,
                 intercompany_balancing_account, effective_from, effective_to, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
            RETURNING *"#
        )
        .bind(org_id).bind(code).bind(name).bind(description)
        .bind(application).bind(transaction_type).bind(event_class)
        .bind(auto_accounting).bind(allow_manual_entries).bind(apply_rounding)
        .bind(rounding_account_code).bind(rounding_threshold)
        .bind(require_balancing).bind(intercompany_balancing_account)
        .bind(effective_from).bind(effective_to).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_method!(row))
    }

    async fn get_accounting_method(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<AccountingMethod>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.accounting_methods WHERE organization_id = $1 AND code = $2 AND is_active = true"
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| row_to_method!(r)))
    }

    async fn get_accounting_method_by_id(&self, id: Uuid) -> AtlasResult<Option<AccountingMethod>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.accounting_methods WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| row_to_method!(r)))
    }

    async fn list_accounting_methods(&self, org_id: Uuid, application: Option<&str>) -> AtlasResult<Vec<AccountingMethod>> {
        let rows = if let Some(app) = application {
            sqlx::query(
                "SELECT * FROM _atlas.accounting_methods WHERE organization_id = $1 AND application = $2 AND is_active = true ORDER BY created_at DESC"
            )
            .bind(org_id).bind(app)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query(
                "SELECT * FROM _atlas.accounting_methods WHERE organization_id = $1 AND is_active = true ORDER BY created_at DESC"
            )
            .bind(org_id)
            .fetch_all(&self.pool)
            .await
        }.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(|r| row_to_method!(r)).collect())
    }

    async fn delete_accounting_method(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.accounting_methods SET is_active = false, updated_at = now() WHERE organization_id = $1 AND code = $2"
        )
        .bind(org_id).bind(code)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Accounting Derivation Rules
    // ========================================================================

    async fn create_derivation_rule(
        &self,
        org_id: Uuid, accounting_method_id: Uuid, code: &str, name: &str,
        description: Option<&str>, line_type: &str, priority: i32,
        conditions: serde_json::Value, source_field: Option<&str>,
        derivation_type: &str, fixed_account_code: Option<&str>,
        account_derivation_lookup: serde_json::Value,
        formula_expression: Option<&str>, sequence: i32,
        effective_from: Option<chrono::NaiveDate>, effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AccountingDerivationRule> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.accounting_derivation_rules
                (organization_id, accounting_method_id, code, name, description,
                 line_type, priority, conditions, source_field, derivation_type,
                 fixed_account_code, account_derivation_lookup, formula_expression,
                 sequence, effective_from, effective_to, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
            RETURNING *"#
        )
        .bind(org_id).bind(accounting_method_id).bind(code).bind(name).bind(description)
        .bind(line_type).bind(priority).bind(&conditions).bind(source_field)
        .bind(derivation_type).bind(fixed_account_code).bind(&account_derivation_lookup)
        .bind(formula_expression).bind(sequence).bind(effective_from).bind(effective_to).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_rule!(row))
    }

    async fn get_derivation_rule(&self, org_id: Uuid, method_id: Uuid, code: &str) -> AtlasResult<Option<AccountingDerivationRule>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.accounting_derivation_rules WHERE organization_id = $1 AND accounting_method_id = $2 AND code = $3 AND is_active = true"
        )
        .bind(org_id).bind(method_id).bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| row_to_rule!(r)))
    }

    async fn list_derivation_rules(&self, org_id: Uuid, method_id: Uuid) -> AtlasResult<Vec<AccountingDerivationRule>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.accounting_derivation_rules WHERE organization_id = $1 AND accounting_method_id = $2 AND is_active = true ORDER BY priority ASC, sequence ASC"
        )
        .bind(org_id).bind(method_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(|r| row_to_rule!(r)).collect())
    }

    async fn list_active_derivation_rules(&self, org_id: Uuid, method_id: Uuid, line_type: &str) -> AtlasResult<Vec<AccountingDerivationRule>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.accounting_derivation_rules
            WHERE organization_id = $1 AND accounting_method_id = $2 AND line_type = $3
              AND is_active = true
              AND (effective_from IS NULL OR effective_from <= CURRENT_DATE)
              AND (effective_to IS NULL OR effective_to >= CURRENT_DATE)
            ORDER BY priority ASC, sequence ASC"#
        )
        .bind(org_id).bind(method_id).bind(line_type)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(|r| row_to_rule!(r)).collect())
    }

    async fn delete_derivation_rule(&self, org_id: Uuid, method_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.accounting_derivation_rules SET is_active = false, updated_at = now() WHERE organization_id = $1 AND accounting_method_id = $2 AND code = $3"
        )
        .bind(org_id).bind(method_id).bind(code)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Subledger Journal Entries
    // ========================================================================

    async fn create_journal_entry(
        &self,
        org_id: Uuid,
        source_application: &str, source_transaction_type: &str,
        source_transaction_id: Uuid, source_transaction_number: Option<&str>,
        accounting_method_id: Option<Uuid>,
        entry_number: &str, description: Option<&str>, reference_number: Option<&str>,
        accounting_date: chrono::NaiveDate, period_name: Option<&str>,
        currency_code: &str, entered_currency_code: &str,
        currency_conversion_date: Option<chrono::NaiveDate>,
        currency_conversion_type: Option<&str>, currency_conversion_rate: Option<&str>,
        total_debit: &str, total_credit: &str,
        entered_debit: &str, entered_credit: &str,
        status: &str, balancing_segment: Option<&str>, is_balanced: bool,
        created_by: Option<Uuid>,
    ) -> AtlasResult<SubledgerJournalEntry> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.subledger_journal_entries
                (organization_id, source_application, source_transaction_type,
                 source_transaction_id, source_transaction_number, accounting_method_id,
                 entry_number, description, reference_number,
                 accounting_date, period_name,
                 currency_code, entered_currency_code,
                 currency_conversion_date, currency_conversion_type, currency_conversion_rate,
                 total_debit, total_credit, entered_debit, entered_credit,
                 status, balancing_segment, is_balanced,
                 gl_transfer_status, is_reversal,
                 created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16,
                    $17, $18, $19, $20, $21, $22, $23, 'pending', false, $24)
            RETURNING *"#
        )
        .bind(org_id).bind(source_application).bind(source_transaction_type)
        .bind(source_transaction_id).bind(source_transaction_number).bind(accounting_method_id)
        .bind(entry_number).bind(description).bind(reference_number)
        .bind(accounting_date).bind(period_name)
        .bind(currency_code).bind(entered_currency_code)
        .bind(currency_conversion_date).bind(currency_conversion_type).bind(currency_conversion_rate)
        .bind(total_debit).bind(total_credit).bind(entered_debit).bind(entered_credit)
        .bind(status).bind(balancing_segment).bind(is_balanced)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_entry!(row))
    }

    async fn get_journal_entry(&self, id: Uuid) -> AtlasResult<Option<SubledgerJournalEntry>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.subledger_journal_entries WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| row_to_entry!(r)))
    }

    async fn get_journal_entry_by_number(&self, org_id: Uuid, entry_number: &str) -> AtlasResult<Option<SubledgerJournalEntry>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.subledger_journal_entries WHERE organization_id = $1 AND entry_number = $2"
        )
        .bind(org_id).bind(entry_number)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| row_to_entry!(r)))
    }

    async fn list_journal_entries(
        &self,
        org_id: Uuid, status: Option<&str>,
        source_application: Option<&str>,
        source_transaction_type: Option<&str>,
        accounting_date_from: Option<chrono::NaiveDate>,
        accounting_date_to: Option<chrono::NaiveDate>,
    ) -> AtlasResult<Vec<SubledgerJournalEntry>> {
        let mut query = String::from(
            "SELECT * FROM _atlas.subledger_journal_entries WHERE organization_id = $1"
        );
        let mut param_idx = 2;

        if status.is_some() {
            query.push_str(&format!(" AND status = ${}", param_idx));
            param_idx += 1;
        }
        if source_application.is_some() {
            query.push_str(&format!(" AND source_application = ${}", param_idx));
            param_idx += 1;
        }
        if source_transaction_type.is_some() {
            query.push_str(&format!(" AND source_transaction_type = ${}", param_idx));
            param_idx += 1;
        }
        if accounting_date_from.is_some() {
            query.push_str(&format!(" AND accounting_date >= ${}", param_idx));
            param_idx += 1;
        }
        if accounting_date_to.is_some() {
            query.push_str(&format!(" AND accounting_date <= ${}", param_idx));
            // param_idx incremented for potential future filters
        }
        query.push_str(" ORDER BY accounting_date DESC, created_at DESC");

        let mut q = sqlx::query(&query).bind(org_id);
        if let Some(s) = status { q = q.bind(s); }
        if let Some(a) = source_application { q = q.bind(a); }
        if let Some(t) = source_transaction_type { q = q.bind(t); }
        if let Some(f) = accounting_date_from { q = q.bind(f); }
        if let Some(t) = accounting_date_to { q = q.bind(t); }

        let rows = q.fetch_all(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(|r| row_to_entry!(r)).collect())
    }

    async fn update_journal_entry_status(
        &self,
        id: Uuid, status: &str, error_message: Option<&str>,
        is_balanced: Option<bool>, posted_by: Option<Uuid>, accounted_by: Option<Uuid>,
    ) -> AtlasResult<SubledgerJournalEntry> {
        let row = sqlx::query(
            r#"UPDATE _atlas.subledger_journal_entries
            SET status = $1, error_message = $2, is_balanced = COALESCE($3, is_balanced),
                posted_by = COALESCE($4, posted_by), accounted_by = COALESCE($5, accounted_by),
                updated_at = now()
            WHERE id = $6
            RETURNING *"#
        )
        .bind(status).bind(error_message).bind(is_balanced)
        .bind(posted_by).bind(accounted_by).bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_entry!(row))
    }

    async fn update_journal_entry_balances(
        &self,
        id: Uuid, total_debit: &str, total_credit: &str,
        entered_debit: &str, entered_credit: &str, is_balanced: bool,
    ) -> AtlasResult<SubledgerJournalEntry> {
        let row = sqlx::query(
            r#"UPDATE _atlas.subledger_journal_entries
            SET total_debit = $1, total_credit = $2,
                entered_debit = $3, entered_credit = $4,
                is_balanced = $5, updated_at = now()
            WHERE id = $6
            RETURNING *"#
        )
        .bind(total_debit).bind(total_credit).bind(entered_debit).bind(entered_credit)
        .bind(is_balanced).bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_entry!(row))
    }

    // ========================================================================
    // Subledger Journal Lines
    // ========================================================================

    async fn create_journal_line(
        &self,
        org_id: Uuid, journal_entry_id: Uuid, line_number: i32,
        line_type: &str, account_code: &str, account_description: Option<&str>,
        derivation_rule_id: Option<Uuid>,
        entered_amount: &str, accounted_amount: &str,
        currency_code: &str, conversion_date: Option<chrono::NaiveDate>, conversion_rate: Option<&str>,
        attribute_category: Option<&str>,
        attribute1: Option<&str>, attribute2: Option<&str>, attribute3: Option<&str>,
        attribute4: Option<&str>, attribute5: Option<&str>,
        source_line_id: Option<Uuid>, source_line_type: Option<&str>,
        tax_code: Option<&str>, tax_rate: Option<&str>, tax_amount: Option<&str>,
    ) -> AtlasResult<SubledgerJournalLine> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.subledger_journal_lines
                (organization_id, journal_entry_id, line_number, line_type,
                 account_code, account_description, derivation_rule_id,
                 entered_amount, accounted_amount,
                 currency_code, conversion_date, conversion_rate,
                 attribute_category, attribute1, attribute2, attribute3, attribute4, attribute5,
                 source_line_id, source_line_type,
                 tax_code, tax_rate, tax_amount)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18,
                    $19, $20, $21, $22, $23)
            RETURNING *"#
        )
        .bind(org_id).bind(journal_entry_id).bind(line_number).bind(line_type)
        .bind(account_code).bind(account_description).bind(derivation_rule_id)
        .bind(entered_amount).bind(accounted_amount)
        .bind(currency_code).bind(conversion_date).bind(conversion_rate)
        .bind(attribute_category).bind(attribute1).bind(attribute2).bind(attribute3).bind(attribute4).bind(attribute5)
        .bind(source_line_id).bind(source_line_type)
        .bind(tax_code).bind(tax_rate).bind(tax_amount)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_line!(row))
    }

    async fn list_journal_lines(&self, journal_entry_id: Uuid) -> AtlasResult<Vec<SubledgerJournalLine>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.subledger_journal_lines WHERE journal_entry_id = $1 ORDER BY line_number ASC"
        )
        .bind(journal_entry_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(|r| row_to_line!(r)).collect())
    }

    async fn delete_journal_line(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query(
            "DELETE FROM _atlas.subledger_journal_lines WHERE id = $1"
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // SLA Events
    // ========================================================================

    async fn create_sla_event(
        &self,
        org_id: Uuid, event_number: &str, event_type: &str,
        source_application: &str, source_transaction_type: &str,
        source_transaction_id: Uuid, journal_entry_id: Option<Uuid>,
        event_date: chrono::NaiveDate, event_status: &str,
        description: Option<&str>, error_message: Option<&str>,
        processed_by: Option<Uuid>,
    ) -> AtlasResult<SlaEvent> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.sla_events
                (organization_id, event_number, event_type,
                 source_application, source_transaction_type, source_transaction_id,
                 journal_entry_id, event_date, event_status, description,
                 error_message, processed_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING *"#
        )
        .bind(org_id).bind(event_number).bind(event_type)
        .bind(source_application).bind(source_transaction_type).bind(source_transaction_id)
        .bind(journal_entry_id).bind(event_date).bind(event_status)
        .bind(description).bind(error_message).bind(processed_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_event!(row))
    }

    async fn list_sla_events(
        &self,
        org_id: Uuid, source_application: Option<&str>, event_type: Option<&str>,
    ) -> AtlasResult<Vec<SlaEvent>> {
        let mut query = String::from(
            "SELECT * FROM _atlas.sla_events WHERE organization_id = $1"
        );
        let mut param_idx = 2;

        if source_application.is_some() {
            query.push_str(&format!(" AND source_application = ${}", param_idx));
            param_idx += 1;
        }
        if event_type.is_some() {
            query.push_str(&format!(" AND event_type = ${}", param_idx));
            // param_idx incremented for potential future filters
        }
        query.push_str(" ORDER BY event_date DESC, created_at DESC");

        let mut q = sqlx::query(&query).bind(org_id);
        if let Some(a) = source_application { q = q.bind(a); }
        if let Some(t) = event_type { q = q.bind(t); }

        let rows = q.fetch_all(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(|r| row_to_event!(r)).collect())
    }

    // ========================================================================
    // GL Transfer Log
    // ========================================================================

    async fn create_transfer_log(
        &self,
        org_id: Uuid, transfer_number: &str, from_period: Option<&str>,
        status: &str, total_entries: i32, total_debit: &str, total_credit: &str,
        included_applications: serde_json::Value, transferred_by: Option<Uuid>,
        entries: serde_json::Value,
    ) -> AtlasResult<GlTransferLog> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.gl_transfer_log
                (organization_id, transfer_number, from_period, status,
                 total_entries, total_debit, total_credit,
                 included_applications, transferred_by, entries)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING *"#
        )
        .bind(org_id).bind(transfer_number).bind(from_period).bind(status)
        .bind(total_entries).bind(total_debit).bind(total_credit)
        .bind(&included_applications).bind(transferred_by).bind(&entries)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_transfer!(row))
    }

    async fn update_transfer_log_status(
        &self,
        id: Uuid, status: &str, error_message: Option<&str>,
        completed_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> AtlasResult<GlTransferLog> {
        let row = sqlx::query(
            r#"UPDATE _atlas.gl_transfer_log
            SET status = $1, error_message = $2, completed_at = $3, updated_at = now()
            WHERE id = $4
            RETURNING *"#
        )
        .bind(status).bind(error_message).bind(completed_at).bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_transfer!(row))
    }

    async fn get_transfer_log(&self, id: Uuid) -> AtlasResult<Option<GlTransferLog>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.gl_transfer_log WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| row_to_transfer!(r)))
    }

    async fn list_transfer_logs(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<GlTransferLog>> {
        let rows = if let Some(s) = status {
            sqlx::query(
                "SELECT * FROM _atlas.gl_transfer_log WHERE organization_id = $1 AND status = $2 ORDER BY transfer_date DESC"
            )
            .bind(org_id).bind(s)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query(
                "SELECT * FROM _atlas.gl_transfer_log WHERE organization_id = $1 ORDER BY transfer_date DESC"
            )
            .bind(org_id)
            .fetch_all(&self.pool)
            .await
        }.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(|r| row_to_transfer!(r)).collect())
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    async fn count_entries_by_status(&self, org_id: Uuid) -> AtlasResult<serde_json::Value> {
        let rows = sqlx::query(
            r#"SELECT status, COUNT(*) as count, SUM(total_debit) as total_debit, SUM(total_credit) as total_credit
            FROM _atlas.subledger_journal_entries
            WHERE organization_id = $1
            GROUP BY status"#
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let by_status: serde_json::Value = rows.iter().map(|r| {
            let status: String = r.get("status");
            let count: i64 = r.get("count");
            let debit_str: Option<String> = r.get("total_debit");
            let credit_str: Option<String> = r.get("total_credit");
            serde_json::json!({
                "status": status,
                "count": count,
                "total_debit": debit_str.unwrap_or_default(),
                "total_credit": credit_str.unwrap_or_default(),
            })
        }).collect();

        let app_rows = sqlx::query(
            r#"SELECT source_application, COUNT(*) as count
            FROM _atlas.subledger_journal_entries
            WHERE organization_id = $1
            GROUP BY source_application"#
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let by_app: serde_json::Value = app_rows.iter().map(|r| {
            let app: String = r.get("source_application");
            let count: i64 = r.get("count");
            serde_json::json!({"application": app, "count": count})
        }).collect();

        Ok(serde_json::json!({
            "by_status": by_status,
            "by_application": by_app,
        }))
    }
}