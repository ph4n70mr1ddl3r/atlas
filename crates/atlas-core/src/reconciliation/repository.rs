//! Bank Reconciliation Repository
//!
//! PostgreSQL storage for bank accounts, statements, statement lines,
//! system transactions, reconciliation matches, summaries, and matching rules.

use atlas_shared::{
    BankAccount, BankStatement, BankStatementLine, SystemTransaction,
    ReconciliationMatch, ReconciliationSummary, ReconciliationMatchingRule,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

/// Repository trait for reconciliation storage
#[async_trait]
pub trait ReconciliationRepository: Send + Sync {
    // Bank Accounts
    async fn create_bank_account(
        &self,
        org_id: Uuid,
        account_number: &str,
        account_name: &str,
        bank_name: &str,
        bank_code: Option<&str>,
        branch_name: Option<&str>,
        branch_code: Option<&str>,
        gl_account_code: Option<&str>,
        currency_code: &str,
        account_type: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<BankAccount>;

    async fn get_bank_account(&self, id: Uuid) -> AtlasResult<Option<BankAccount>>;
    async fn list_bank_accounts(&self, org_id: Uuid) -> AtlasResult<Vec<BankAccount>>;
    async fn delete_bank_account(&self, id: Uuid) -> AtlasResult<()>;

    // Bank Statements
    async fn create_bank_statement(
        &self,
        org_id: Uuid,
        bank_account_id: Uuid,
        statement_number: &str,
        statement_date: chrono::NaiveDate,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
        opening_balance: &str,
        closing_balance: &str,
        imported_by: Option<Uuid>,
    ) -> AtlasResult<BankStatement>;

    async fn get_bank_statement(&self, id: Uuid) -> AtlasResult<Option<BankStatement>>;
    async fn list_bank_statements(
        &self,
        org_id: Uuid,
        bank_account_id: Uuid,
    ) -> AtlasResult<Vec<BankStatement>>;
    async fn update_statement_counts(
        &self,
        id: Uuid,
        total_lines: i32,
        matched_lines: i32,
        unmatched_lines: i32,
        reconciliation_percent: f64,
    ) -> AtlasResult<()>;
    async fn update_statement_status(
        &self,
        id: Uuid,
        status: &str,
        reconciled_by: Option<Uuid>,
    ) -> AtlasResult<BankStatement>;

    // Statement Lines
    async fn create_statement_line(
        &self,
        org_id: Uuid,
        statement_id: Uuid,
        line_number: i32,
        transaction_date: chrono::NaiveDate,
        transaction_type: &str,
        amount: &str,
        description: Option<&str>,
        reference_number: Option<&str>,
        check_number: Option<&str>,
        counterparty_name: Option<&str>,
        counterparty_account: Option<&str>,
    ) -> AtlasResult<BankStatementLine>;

    async fn list_statement_lines(
        &self,
        statement_id: Uuid,
    ) -> AtlasResult<Vec<BankStatementLine>>;

    // System Transactions
    async fn create_system_transaction(
        &self,
        org_id: Uuid,
        bank_account_id: Uuid,
        source_type: &str,
        source_id: Uuid,
        source_number: Option<&str>,
        transaction_date: chrono::NaiveDate,
        amount: &str,
        transaction_type: &str,
        description: Option<&str>,
        reference_number: Option<&str>,
        check_number: Option<&str>,
        counterparty_name: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<SystemTransaction>;

    async fn get_system_transaction(&self, id: Uuid) -> AtlasResult<Option<SystemTransaction>>;
    async fn list_unreconciled_transactions(
        &self,
        org_id: Uuid,
        bank_account_id: Uuid,
    ) -> AtlasResult<Vec<SystemTransaction>>;

    // Matches
    async fn create_match(
        &self,
        org_id: Uuid,
        statement_id: Uuid,
        statement_line_id: Uuid,
        system_transaction_id: Uuid,
        match_method: &str,
        match_confidence: Option<f64>,
        matched_by: Option<Uuid>,
    ) -> AtlasResult<ReconciliationMatch>;

    async fn get_match(&self, id: Uuid) -> AtlasResult<Option<ReconciliationMatch>>;
    async fn list_matches(&self, statement_id: Uuid) -> AtlasResult<Vec<ReconciliationMatch>>;
    async fn unmatch(
        &self,
        id: Uuid,
        unmatched_by: Option<Uuid>,
    ) -> AtlasResult<ReconciliationMatch>;

    // Summaries
    async fn get_or_create_summary(
        &self,
        org_id: Uuid,
        bank_account_id: Uuid,
        period_start: chrono::NaiveDate,
        period_end: chrono::NaiveDate,
    ) -> AtlasResult<ReconciliationSummary>;
    async fn list_summaries(&self, org_id: Uuid) -> AtlasResult<Vec<ReconciliationSummary>>;

    // Matching Rules
    async fn create_matching_rule(
        &self,
        org_id: Uuid,
        name: &str,
        description: Option<&str>,
        bank_account_id: Option<Uuid>,
        priority: i32,
        criteria: serde_json::Value,
        stop_on_match: bool,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ReconciliationMatchingRule>;

    async fn list_matching_rules(&self, org_id: Uuid) -> AtlasResult<Vec<ReconciliationMatchingRule>>;
    async fn delete_matching_rule(&self, id: Uuid) -> AtlasResult<()>;
}

/// PostgreSQL implementation
pub struct PostgresReconciliationRepository {
    pool: PgPool,
}

impl PostgresReconciliationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn row_to_bank_account(&self, row: &sqlx::postgres::PgRow) -> BankAccount {
        BankAccount {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            account_number: row.get("account_number"),
            account_name: row.get("account_name"),
            bank_name: row.get("bank_name"),
            bank_code: row.get("bank_code"),
            branch_name: row.get("branch_name"),
            branch_code: row.get("branch_code"),
            gl_account_code: row.get("gl_account_code"),
            currency_code: row.get("currency_code"),
            account_type: row.get("account_type"),
            last_statement_balance: row_to_json_value(row, "last_statement_balance"),
            last_statement_date: row.get("last_statement_date"),
            is_active: row.get("is_active"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            metadata: row.get("metadata"),
        }
    }

    fn row_to_bank_statement(&self, row: &sqlx::postgres::PgRow) -> BankStatement {
        BankStatement {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            bank_account_id: row.get("bank_account_id"),
            statement_number: row.get("statement_number"),
            statement_date: row.get("statement_date"),
            start_date: row.get("start_date"),
            end_date: row.get("end_date"),
            opening_balance: row_to_json_value(row, "opening_balance"),
            closing_balance: row_to_json_value(row, "closing_balance"),
            total_deposits: row_to_json_value(row, "total_deposits"),
            total_withdrawals: row_to_json_value(row, "total_withdrawals"),
            total_interest: row_to_json_value(row, "total_interest"),
            total_charges: row_to_json_value(row, "total_charges"),
            total_lines: row.get("total_lines"),
            matched_lines: row.get("matched_lines"),
            unmatched_lines: row.get("unmatched_lines"),
            status: row.get("status"),
            reconciliation_percent: row_to_json_value(row, "reconciliation_percent"),
            imported_by: row.get("imported_by"),
            reviewed_by: row.get("reviewed_by"),
            reconciled_by: row.get("reconciled_by"),
            reconciled_at: row.get("reconciled_at"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            metadata: row.get("metadata"),
        }
    }

    fn row_to_statement_line(&self, row: &sqlx::postgres::PgRow) -> BankStatementLine {
        BankStatementLine {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            statement_id: row.get("statement_id"),
            line_number: row.get("line_number"),
            transaction_date: row.get("transaction_date"),
            transaction_type: row.get("transaction_type"),
            amount: row_to_json_value(row, "amount"),
            description: row.get("description"),
            reference_number: row.get("reference_number"),
            check_number: row.get("check_number"),
            counterparty_name: row.get("counterparty_name"),
            counterparty_account: row.get("counterparty_account"),
            match_status: row.get("match_status"),
            matched_by: row.get("matched_by"),
            matched_at: row.get("matched_at"),
            match_method: row.get("match_method"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            metadata: row.get("metadata"),
        }
    }

    fn row_to_system_transaction(&self, row: &sqlx::postgres::PgRow) -> SystemTransaction {
        SystemTransaction {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            bank_account_id: row.get("bank_account_id"),
            source_type: row.get("source_type"),
            source_id: row.get("source_id"),
            source_number: row.get("source_number"),
            transaction_date: row.get("transaction_date"),
            amount: row_to_json_value(row, "amount"),
            transaction_type: row.get("transaction_type"),
            description: row.get("description"),
            reference_number: row.get("reference_number"),
            check_number: row.get("check_number"),
            counterparty_name: row.get("counterparty_name"),
            status: row.get("status"),
            gl_posting_date: row.get("gl_posting_date"),
            currency_code: row.get("currency_code"),
            exchange_rate: row.get("exchange_rate"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            metadata: row.get("metadata"),
        }
    }

    fn row_to_reconciliation_match(&self, row: &sqlx::postgres::PgRow) -> ReconciliationMatch {
        ReconciliationMatch {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            statement_id: row.get("statement_id"),
            statement_line_id: row.get("statement_line_id"),
            system_transaction_id: row.get("system_transaction_id"),
            match_method: row.get("match_method"),
            match_confidence: row.get("match_confidence"),
            matched_by: row.get("matched_by"),
            matched_at: row.get("matched_at"),
            unmatched_by: row.get("unmatched_by"),
            unmatched_at: row.get("unmatched_at"),
            status: row.get("status"),
            notes: row.get("notes"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            metadata: row.get("metadata"),
        }
    }

    fn row_to_summary(&self, row: &sqlx::postgres::PgRow) -> ReconciliationSummary {
        ReconciliationSummary {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            bank_account_id: row.get("bank_account_id"),
            period_start: row.get("period_start"),
            period_end: row.get("period_end"),
            statement_id: row.get("statement_id"),
            statement_balance: row_to_json_value(row, "statement_balance"),
            book_balance: row_to_json_value(row, "book_balance"),
            deposits_in_transit: row_to_json_value(row, "deposits_in_transit"),
            outstanding_checks: row_to_json_value(row, "outstanding_checks"),
            bank_charges: row_to_json_value(row, "bank_charges"),
            bank_interest: row_to_json_value(row, "bank_interest"),
            errors_and_omissions: row_to_json_value(row, "errors_and_omissions"),
            adjusted_book_balance: row_to_json_value(row, "adjusted_book_balance"),
            adjusted_bank_balance: row_to_json_value(row, "adjusted_bank_balance"),
            difference: row_to_json_value(row, "difference"),
            is_balanced: row.get("is_balanced"),
            status: row.get("status"),
            reviewed_by: row.get("reviewed_by"),
            reviewed_at: row.get("reviewed_at"),
            approved_by: row.get("approved_by"),
            approved_at: row.get("approved_at"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            metadata: row.get("metadata"),
        }
    }

    fn row_to_matching_rule(&self, row: &sqlx::postgres::PgRow) -> ReconciliationMatchingRule {
        ReconciliationMatchingRule {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            bank_account_id: row.get("bank_account_id"),
            name: row.get("name"),
            description: row.get("description"),
            priority: row.get("priority"),
            criteria: row.get("criteria"),
            stop_on_match: row.get("stop_on_match"),
            is_active: row.get("is_active"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}

fn row_to_json_value(row: &sqlx::postgres::PgRow, col: &str) -> serde_json::Value {
    row.try_get::<serde_json::Value, _>(col).unwrap_or(serde_json::json!(0))
}

#[async_trait]
impl ReconciliationRepository for PostgresReconciliationRepository {
    // ========================================================================
    // Bank Accounts
    // ========================================================================

    async fn create_bank_account(
        &self,
        org_id: Uuid,
        account_number: &str,
        account_name: &str,
        bank_name: &str,
        bank_code: Option<&str>,
        branch_name: Option<&str>,
        branch_code: Option<&str>,
        gl_account_code: Option<&str>,
        currency_code: &str,
        account_type: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<BankAccount> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.bank_accounts
                (organization_id, account_number, account_name, bank_name,
                 bank_code, branch_name, branch_code, gl_account_code,
                 currency_code, account_type, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING *
            "#,
        )
        .bind(org_id)
        .bind(account_number)
        .bind(account_name)
        .bind(bank_name)
        .bind(bank_code)
        .bind(branch_name)
        .bind(branch_code)
        .bind(gl_account_code)
        .bind(currency_code)
        .bind(account_type)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_bank_account(&row))
    }

    async fn get_bank_account(&self, id: Uuid) -> AtlasResult<Option<BankAccount>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.bank_accounts WHERE id = $1 AND is_active = true",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| self.row_to_bank_account(&r)))
    }

    async fn list_bank_accounts(&self, org_id: Uuid) -> AtlasResult<Vec<BankAccount>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.bank_accounts WHERE organization_id = $1 AND is_active = true ORDER BY account_name",
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(|r| self.row_to_bank_account(r)).collect())
    }

    async fn delete_bank_account(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("UPDATE _atlas.bank_accounts SET is_active = false WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Bank Statements
    // ========================================================================

    async fn create_bank_statement(
        &self,
        org_id: Uuid,
        bank_account_id: Uuid,
        statement_number: &str,
        statement_date: chrono::NaiveDate,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
        opening_balance: &str,
        closing_balance: &str,
        imported_by: Option<Uuid>,
    ) -> AtlasResult<BankStatement> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.bank_statements
                (organization_id, bank_account_id, statement_number,
                 statement_date, start_date, end_date,
                 opening_balance, closing_balance, status, imported_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7::numeric, $8::numeric, 'imported', $9)
            RETURNING *
            "#,
        )
        .bind(org_id)
        .bind(bank_account_id)
        .bind(statement_number)
        .bind(statement_date)
        .bind(start_date)
        .bind(end_date)
        .bind(opening_balance)
        .bind(closing_balance)
        .bind(imported_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_bank_statement(&row))
    }

    async fn get_bank_statement(&self, id: Uuid) -> AtlasResult<Option<BankStatement>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.bank_statements WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| self.row_to_bank_statement(&r)))
    }

    async fn list_bank_statements(
        &self,
        org_id: Uuid,
        bank_account_id: Uuid,
    ) -> AtlasResult<Vec<BankStatement>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.bank_statements WHERE organization_id = $1 AND bank_account_id = $2 ORDER BY statement_date DESC",
        )
        .bind(org_id)
        .bind(bank_account_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(|r| self.row_to_bank_statement(r)).collect())
    }

    async fn update_statement_counts(
        &self,
        id: Uuid,
        total_lines: i32,
        matched_lines: i32,
        unmatched_lines: i32,
        reconciliation_percent: f64,
    ) -> AtlasResult<()> {
        sqlx::query(
            r#"
            UPDATE _atlas.bank_statements
            SET total_lines = $2, matched_lines = $3, unmatched_lines = $4,
                reconciliation_percent = $5, updated_at = now()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(total_lines)
        .bind(matched_lines)
        .bind(unmatched_lines)
        .bind(reconciliation_percent)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn update_statement_status(
        &self,
        id: Uuid,
        status: &str,
        reconciled_by: Option<Uuid>,
    ) -> AtlasResult<BankStatement> {
        let reconciled_at = if status == "reconciled" {
            Some(chrono::Utc::now())
        } else {
            None
        };

        let row = sqlx::query(
            r#"
            UPDATE _atlas.bank_statements
            SET status = $2, reconciled_by = $3, reconciled_at = $4, updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(status)
        .bind(reconciled_by)
        .bind(reconciled_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_bank_statement(&row))
    }

    // ========================================================================
    // Statement Lines
    // ========================================================================

    async fn create_statement_line(
        &self,
        org_id: Uuid,
        statement_id: Uuid,
        line_number: i32,
        transaction_date: chrono::NaiveDate,
        transaction_type: &str,
        amount: &str,
        description: Option<&str>,
        reference_number: Option<&str>,
        check_number: Option<&str>,
        counterparty_name: Option<&str>,
        counterparty_account: Option<&str>,
    ) -> AtlasResult<BankStatementLine> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.bank_statement_lines
                (organization_id, statement_id, line_number,
                 transaction_date, transaction_type, amount,
                 description, reference_number, check_number,
                 counterparty_name, counterparty_account, match_status)
            VALUES ($1, $2, $3, $4, $5, $6::numeric, $7, $8, $9, $10, $11, 'unmatched')
            RETURNING *
            "#,
        )
        .bind(org_id)
        .bind(statement_id)
        .bind(line_number)
        .bind(transaction_date)
        .bind(transaction_type)
        .bind(amount)
        .bind(description)
        .bind(reference_number)
        .bind(check_number)
        .bind(counterparty_name)
        .bind(counterparty_account)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_statement_line(&row))
    }

    async fn list_statement_lines(
        &self,
        statement_id: Uuid,
    ) -> AtlasResult<Vec<BankStatementLine>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.bank_statement_lines WHERE statement_id = $1 ORDER BY line_number",
        )
        .bind(statement_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(|r| self.row_to_statement_line(r)).collect())
    }

    // ========================================================================
    // System Transactions
    // ========================================================================

    async fn create_system_transaction(
        &self,
        org_id: Uuid,
        bank_account_id: Uuid,
        source_type: &str,
        source_id: Uuid,
        source_number: Option<&str>,
        transaction_date: chrono::NaiveDate,
        amount: &str,
        transaction_type: &str,
        description: Option<&str>,
        reference_number: Option<&str>,
        check_number: Option<&str>,
        counterparty_name: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<SystemTransaction> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.system_transactions
                (organization_id, bank_account_id, source_type, source_id,
                 source_number, transaction_date, amount, transaction_type,
                 description, reference_number, check_number, counterparty_name,
                 status, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7::numeric, $8, $9, $10, $11, $12, 'unreconciled', $13)
            RETURNING *
            "#,
        )
        .bind(org_id)
        .bind(bank_account_id)
        .bind(source_type)
        .bind(source_id)
        .bind(source_number)
        .bind(transaction_date)
        .bind(amount)
        .bind(transaction_type)
        .bind(description)
        .bind(reference_number)
        .bind(check_number)
        .bind(counterparty_name)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_system_transaction(&row))
    }

    async fn get_system_transaction(&self, id: Uuid) -> AtlasResult<Option<SystemTransaction>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.system_transactions WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| self.row_to_system_transaction(&r)))
    }

    async fn list_unreconciled_transactions(
        &self,
        org_id: Uuid,
        bank_account_id: Uuid,
    ) -> AtlasResult<Vec<SystemTransaction>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.system_transactions WHERE organization_id = $1 AND bank_account_id = $2 AND status = 'unreconciled' ORDER BY transaction_date",
        )
        .bind(org_id)
        .bind(bank_account_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(|r| self.row_to_system_transaction(r)).collect())
    }

    // ========================================================================
    // Matches
    // ========================================================================

    async fn create_match(
        &self,
        org_id: Uuid,
        statement_id: Uuid,
        statement_line_id: Uuid,
        system_transaction_id: Uuid,
        match_method: &str,
        match_confidence: Option<f64>,
        matched_by: Option<Uuid>,
    ) -> AtlasResult<ReconciliationMatch> {
        // Create the match record
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.reconciliation_matches
                (organization_id, statement_id, statement_line_id,
                 system_transaction_id, match_method, match_confidence,
                 matched_by, status)
            VALUES ($1, $2, $3, $4, $5, $6, $7, 'active')
            RETURNING *
            "#,
        )
        .bind(org_id)
        .bind(statement_id)
        .bind(statement_line_id)
        .bind(system_transaction_id)
        .bind(match_method)
        .bind(match_confidence)
        .bind(matched_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        // Update statement line match status
        let match_status = if match_method == "manual" {
            "manually_matched"
        } else {
            "matched"
        };
        sqlx::query(
            r#"
            UPDATE _atlas.bank_statement_lines
            SET match_status = $2, matched_by = $3, matched_at = now(),
                match_method = $4, updated_at = now()
            WHERE id = $1
            "#,
        )
        .bind(statement_line_id)
        .bind(match_status)
        .bind(matched_by)
        .bind(match_method)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        // Update system transaction status
        sqlx::query(
            r#"
            UPDATE _atlas.system_transactions
            SET status = 'reconciled', updated_at = now()
            WHERE id = $1
            "#,
        )
        .bind(system_transaction_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_reconciliation_match(&row))
    }

    async fn get_match(&self, id: Uuid) -> AtlasResult<Option<ReconciliationMatch>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.reconciliation_matches WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| self.row_to_reconciliation_match(&r)))
    }

    async fn list_matches(&self, statement_id: Uuid) -> AtlasResult<Vec<ReconciliationMatch>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.reconciliation_matches WHERE statement_id = $1 AND status = 'active' ORDER BY matched_at",
        )
        .bind(statement_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(|r| self.row_to_reconciliation_match(r)).collect())
    }

    async fn unmatch(
        &self,
        id: Uuid,
        unmatched_by: Option<Uuid>,
    ) -> AtlasResult<ReconciliationMatch> {
        // Get the match to find the linked records
        let existing = self
            .get_match(id)
            .await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Match {}", id)))?;

        // Update match status
        let row = sqlx::query(
            r#"
            UPDATE _atlas.reconciliation_matches
            SET status = 'unmatched', unmatched_by = $2, unmatched_at = now(), updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(unmatched_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        // Revert statement line to unmatched
        sqlx::query(
            r#"
            UPDATE _atlas.bank_statement_lines
            SET match_status = 'unmatched', matched_by = NULL, matched_at = NULL,
                match_method = NULL, updated_at = now()
            WHERE id = $1
            "#,
        )
        .bind(existing.statement_line_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        // Revert system transaction to unreconciled
        sqlx::query(
            r#"
            UPDATE _atlas.system_transactions
            SET status = 'unreconciled', updated_at = now()
            WHERE id = $1
            "#,
        )
        .bind(existing.system_transaction_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_reconciliation_match(&row))
    }

    // ========================================================================
    // Summaries
    // ========================================================================

    async fn get_or_create_summary(
        &self,
        org_id: Uuid,
        bank_account_id: Uuid,
        period_start: chrono::NaiveDate,
        period_end: chrono::NaiveDate,
    ) -> AtlasResult<ReconciliationSummary> {
        // Try to get existing
        let row = sqlx::query(
            r#"
            SELECT * FROM _atlas.reconciliation_summaries
            WHERE organization_id = $1 AND bank_account_id = $2
              AND period_start = $3 AND period_end = $4
            "#,
        )
        .bind(org_id)
        .bind(bank_account_id)
        .bind(period_start)
        .bind(period_end)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        if let Some(r) = row {
            return Ok(self.row_to_summary(&r));
        }

        // Create new
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.reconciliation_summaries
                (organization_id, bank_account_id, period_start, period_end, status)
            VALUES ($1, $2, $3, $4, 'in_progress')
            RETURNING *
            "#,
        )
        .bind(org_id)
        .bind(bank_account_id)
        .bind(period_start)
        .bind(period_end)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_summary(&row))
    }

    async fn list_summaries(&self, org_id: Uuid) -> AtlasResult<Vec<ReconciliationSummary>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.reconciliation_summaries WHERE organization_id = $1 ORDER BY period_start DESC",
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(|r| self.row_to_summary(r)).collect())
    }

    // ========================================================================
    // Matching Rules
    // ========================================================================

    async fn create_matching_rule(
        &self,
        org_id: Uuid,
        name: &str,
        description: Option<&str>,
        bank_account_id: Option<Uuid>,
        priority: i32,
        criteria: serde_json::Value,
        stop_on_match: bool,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ReconciliationMatchingRule> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.reconciliation_matching_rules
                (organization_id, name, description, bank_account_id,
                 priority, criteria, stop_on_match, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
        )
        .bind(org_id)
        .bind(name)
        .bind(description)
        .bind(bank_account_id)
        .bind(priority)
        .bind(&criteria)
        .bind(stop_on_match)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_matching_rule(&row))
    }

    async fn list_matching_rules(&self, org_id: Uuid) -> AtlasResult<Vec<ReconciliationMatchingRule>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.reconciliation_matching_rules WHERE organization_id = $1 AND is_active = true ORDER BY priority",
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(|r| self.row_to_matching_rule(r)).collect())
    }

    async fn delete_matching_rule(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.reconciliation_matching_rules WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}
