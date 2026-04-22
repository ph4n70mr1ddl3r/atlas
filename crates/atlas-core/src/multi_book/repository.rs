//! Multi-Book Accounting Repository
//!
//! PostgreSQL storage for accounting books, account mappings,
//! book journal entries, journal lines, and propagation logs.

use atlas_shared::{
    AccountingBook, AccountMapping, BookJournalEntry, BookJournalLine,
    PropagationLog, AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

/// Repository trait for multi-book accounting data storage
#[async_trait]
pub trait MultiBookAccountingRepository: Send + Sync {
    // Accounting Books
    async fn create_book(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        book_type: &str,
        chart_of_accounts_code: &str,
        calendar_code: &str,
        currency_code: &str,
        auto_propagation_enabled: bool,
        mapping_level: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AccountingBook>;

    async fn get_book(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<AccountingBook>>;
    async fn get_book_by_id(&self, id: Uuid) -> AtlasResult<Option<AccountingBook>>;
    async fn get_primary_book(&self, org_id: Uuid) -> AtlasResult<Option<AccountingBook>>;
    async fn list_books(&self, org_id: Uuid) -> AtlasResult<Vec<AccountingBook>>;
    async fn update_book_status(&self, id: Uuid, status: &str) -> AtlasResult<AccountingBook>;
    async fn delete_book(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // Account Mappings
    async fn create_account_mapping(
        &self,
        org_id: Uuid,
        source_book_id: Uuid,
        target_book_id: Uuid,
        source_account_code: &str,
        target_account_code: &str,
        segment_mappings: serde_json::Value,
        priority: i32,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AccountMapping>;

    async fn list_account_mappings(
        &self,
        org_id: Uuid,
        source_book_id: Option<Uuid>,
        target_book_id: Option<Uuid>,
    ) -> AtlasResult<Vec<AccountMapping>>;

    async fn find_account_mapping(
        &self,
        org_id: Uuid,
        source_book_id: Uuid,
        target_book_id: Uuid,
        source_account_code: &str,
    ) -> AtlasResult<Option<AccountMapping>>;

    async fn delete_account_mapping(&self, id: Uuid) -> AtlasResult<()>;

    // Journal Entries
    async fn create_journal_entry(
        &self,
        org_id: Uuid,
        book_id: Uuid,
        entry_number: &str,
        header_description: Option<&str>,
        source_book_id: Option<Uuid>,
        source_entry_id: Option<Uuid>,
        external_reference: Option<&str>,
        accounting_date: chrono::NaiveDate,
        period_name: Option<&str>,
        total_debit: &str,
        total_credit: &str,
        status: &str,
        is_auto_propagated: bool,
        currency_code: &str,
        conversion_rate: Option<&str>,
        metadata: serde_json::Value,
        created_by: Option<Uuid>,
    ) -> AtlasResult<BookJournalEntry>;

    async fn get_journal_entry_by_id(&self, id: Uuid) -> AtlasResult<Option<BookJournalEntry>>;
    async fn list_journal_entries(
        &self,
        org_id: Uuid,
        book_id: Uuid,
        status: Option<&str>,
    ) -> AtlasResult<Vec<BookJournalEntry>>;
    async fn update_journal_entry_status(
        &self,
        id: Uuid,
        status: &str,
        posted_by: Option<Uuid>,
    ) -> AtlasResult<BookJournalEntry>;

    // Journal Lines
    async fn create_journal_line(
        &self,
        org_id: Uuid,
        entry_id: Uuid,
        line_number: i32,
        account_code: &str,
        account_name: Option<&str>,
        debit_amount: &str,
        credit_amount: &str,
        description: Option<&str>,
        tax_code: Option<&str>,
        source_line_id: Option<Uuid>,
        metadata: serde_json::Value,
    ) -> AtlasResult<BookJournalLine>;

    async fn list_journal_lines(&self, entry_id: Uuid) -> AtlasResult<Vec<BookJournalLine>>;

    // Propagation Logs
    async fn create_propagation_log(
        &self,
        org_id: Uuid,
        source_book_id: Uuid,
        target_book_id: Uuid,
        source_entry_id: Uuid,
        target_entry_id: Option<Uuid>,
        status: &str,
        lines_propagated: i32,
        lines_unmapped: i32,
        error_message: Option<&str>,
        metadata: serde_json::Value,
    ) -> AtlasResult<PropagationLog>;

    async fn list_propagation_logs(
        &self,
        org_id: Uuid,
        source_book_id: Option<Uuid>,
        target_book_id: Option<Uuid>,
    ) -> AtlasResult<Vec<PropagationLog>>;
}

/// PostgreSQL implementation
pub struct PostgresMultiBookAccountingRepository {
    pool: PgPool,
}

impl PostgresMultiBookAccountingRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn row_to_book(&self, row: &sqlx::postgres::PgRow) -> AccountingBook {
        AccountingBook {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            code: row.get("code"),
            name: row.get("name"),
            description: row.get("description"),
            book_type: row.get("book_type"),
            chart_of_accounts_code: row.get("chart_of_accounts_code"),
            calendar_code: row.get("calendar_code"),
            currency_code: row.get("currency_code"),
            is_enabled: row.get("is_enabled"),
            auto_propagation_enabled: row.get("auto_propagation_enabled"),
            mapping_level: row.get("mapping_level"),
            status: row.get("status"),
            metadata: row.get("metadata"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_mapping(&self, row: &sqlx::postgres::PgRow) -> AccountMapping {
        AccountMapping {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            source_book_id: row.get("source_book_id"),
            target_book_id: row.get("target_book_id"),
            source_account_code: row.get("source_account_code"),
            target_account_code: row.get("target_account_code"),
            segment_mappings: row.get("segment_mappings"),
            priority: row.get("priority"),
            is_active: row.get("is_active"),
            effective_from: row.get("effective_from"),
            effective_to: row.get("effective_to"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_entry(&self, row: &sqlx::postgres::PgRow) -> BookJournalEntry {
        BookJournalEntry {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            book_id: row.get("book_id"),
            entry_number: row.get("entry_number"),
            header_description: row.get("header_description"),
            source_book_id: row.get("source_book_id"),
            source_entry_id: row.get("source_entry_id"),
            external_reference: row.get("external_reference"),
            accounting_date: row.get("accounting_date"),
            period_name: row.get("period_name"),
            total_debit: row.get("total_debit"),
            total_credit: row.get("total_credit"),
            status: row.get("status"),
            is_auto_propagated: row.get("is_auto_propagated"),
            currency_code: row.get("currency_code"),
            conversion_rate: row.get("conversion_rate"),
            metadata: row.get("metadata"),
            created_by: row.get("created_by"),
            posted_by: row.get("posted_by"),
            posted_at: row.get("posted_at"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_line(&self, row: &sqlx::postgres::PgRow) -> BookJournalLine {
        BookJournalLine {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            entry_id: row.get("entry_id"),
            line_number: row.get("line_number"),
            account_code: row.get("account_code"),
            account_name: row.get("account_name"),
            debit_amount: row.get("debit_amount"),
            credit_amount: row.get("credit_amount"),
            description: row.get("description"),
            tax_code: row.get("tax_code"),
            source_line_id: row.get("source_line_id"),
            metadata: row.get("metadata"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_propagation_log(&self, row: &sqlx::postgres::PgRow) -> PropagationLog {
        PropagationLog {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            source_book_id: row.get("source_book_id"),
            target_book_id: row.get("target_book_id"),
            source_entry_id: row.get("source_entry_id"),
            target_entry_id: row.get("target_entry_id"),
            status: row.get("status"),
            lines_propagated: row.get("lines_propagated"),
            lines_unmapped: row.get("lines_unmapped"),
            error_message: row.get("error_message"),
            propagated_at: row.get("propagated_at"),
            metadata: row.get("metadata"),
            created_at: row.get("created_at"),
        }
    }
}

#[async_trait]
impl MultiBookAccountingRepository for PostgresMultiBookAccountingRepository {
    // ========================================================================
    // Accounting Books
    // ========================================================================

    async fn create_book(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        book_type: &str,
        chart_of_accounts_code: &str,
        calendar_code: &str,
        currency_code: &str,
        auto_propagation_enabled: bool,
        mapping_level: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AccountingBook> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.accounting_books
                (organization_id, code, name, description, book_type,
                 chart_of_accounts_code, calendar_code, currency_code,
                 auto_propagation_enabled, mapping_level, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            ON CONFLICT (organization_id, code) DO UPDATE
                SET name = $3, description = $4, book_type = $5,
                    chart_of_accounts_code = $6, calendar_code = $7,
                    currency_code = $8, auto_propagation_enabled = $9,
                    mapping_level = $10, updated_at = now()
            RETURNING *
            "#,
        )
        .bind(org_id).bind(code).bind(name).bind(description).bind(book_type)
        .bind(chart_of_accounts_code).bind(calendar_code).bind(currency_code)
        .bind(auto_propagation_enabled).bind(mapping_level).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_book(&row))
    }

    async fn get_book(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<AccountingBook>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.accounting_books WHERE organization_id = $1 AND code = $2"
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_book(&r)))
    }

    async fn get_book_by_id(&self, id: Uuid) -> AtlasResult<Option<AccountingBook>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.accounting_books WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_book(&r)))
    }

    async fn get_primary_book(&self, org_id: Uuid) -> AtlasResult<Option<AccountingBook>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.accounting_books WHERE organization_id = $1 AND book_type = 'primary' AND status != 'inactive'"
        )
        .bind(org_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_book(&r)))
    }

    async fn list_books(&self, org_id: Uuid) -> AtlasResult<Vec<AccountingBook>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.accounting_books WHERE organization_id = $1 ORDER BY book_type, code"
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_book(r)).collect())
    }

    async fn update_book_status(&self, id: Uuid, status: &str) -> AtlasResult<AccountingBook> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.accounting_books
            SET status = $2, updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(status)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_book(&row))
    }

    async fn delete_book(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.accounting_books SET status = 'inactive', is_enabled = false, updated_at = now() WHERE organization_id = $1 AND code = $2"
        )
        .bind(org_id).bind(code)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Account Mappings
    // ========================================================================

    async fn create_account_mapping(
        &self,
        org_id: Uuid,
        source_book_id: Uuid,
        target_book_id: Uuid,
        source_account_code: &str,
        target_account_code: &str,
        segment_mappings: serde_json::Value,
        priority: i32,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AccountMapping> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.account_mappings
                (organization_id, source_book_id, target_book_id,
                 source_account_code, target_account_code,
                 segment_mappings, priority,
                 effective_from, effective_to, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(source_book_id).bind(target_book_id)
        .bind(source_account_code).bind(target_account_code)
        .bind(segment_mappings).bind(priority)
        .bind(effective_from).bind(effective_to).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_mapping(&row))
    }

    async fn list_account_mappings(
        &self,
        org_id: Uuid,
        source_book_id: Option<Uuid>,
        target_book_id: Option<Uuid>,
    ) -> AtlasResult<Vec<AccountMapping>> {
        let rows = match (source_book_id, target_book_id) {
            (Some(s), Some(t)) => sqlx::query(
                "SELECT * FROM _atlas.account_mappings WHERE organization_id = $1 AND source_book_id = $2 AND target_book_id = $3 AND is_active = true ORDER BY priority"
            )
            .bind(org_id).bind(s).bind(t)
            .fetch_all(&self.pool).await,
            (Some(s), None) => sqlx::query(
                "SELECT * FROM _atlas.account_mappings WHERE organization_id = $1 AND source_book_id = $2 AND is_active = true ORDER BY priority"
            )
            .bind(org_id).bind(s)
            .fetch_all(&self.pool).await,
            (None, Some(t)) => sqlx::query(
                "SELECT * FROM _atlas.account_mappings WHERE organization_id = $1 AND target_book_id = $2 AND is_active = true ORDER BY priority"
            )
            .bind(org_id).bind(t)
            .fetch_all(&self.pool).await,
            (None, None) => sqlx::query(
                "SELECT * FROM _atlas.account_mappings WHERE organization_id = $1 AND is_active = true ORDER BY priority"
            )
            .bind(org_id)
            .fetch_all(&self.pool).await,
        }
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_mapping(r)).collect())
    }

    async fn find_account_mapping(
        &self,
        org_id: Uuid,
        source_book_id: Uuid,
        target_book_id: Uuid,
        source_account_code: &str,
    ) -> AtlasResult<Option<AccountMapping>> {
        let today = chrono::Utc::now().date_naive();
        // Try exact match first, then prefix match
        let row = sqlx::query(
            r#"
            SELECT * FROM _atlas.account_mappings
            WHERE organization_id = $1
              AND source_book_id = $2
              AND target_book_id = $3
              AND source_account_code = $4
              AND is_active = true
              AND (effective_from IS NULL OR effective_from <= $5)
              AND (effective_to IS NULL OR effective_to >= $5)
            ORDER BY priority
            LIMIT 1
            "#,
        )
        .bind(org_id).bind(source_book_id).bind(target_book_id)
        .bind(source_account_code).bind(today)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        match row {
            Some(r) => Ok(Some(self.row_to_mapping(&r))),
            None => {
                // Try prefix/wildcard match
                let prefix = if source_account_code.len() > 4 {
                    &source_account_code[..4]
                } else {
                    source_account_code
                };
                let pattern = format!("{}%", prefix);
                let row = sqlx::query(
                    r#"
                    SELECT * FROM _atlas.account_mappings
                    WHERE organization_id = $1
                      AND source_book_id = $2
                      AND target_book_id = $3
                      AND source_account_code LIKE $4
                      AND is_active = true
                      AND (effective_from IS NULL OR effective_from <= $5)
                      AND (effective_to IS NULL OR effective_to >= $5)
                    ORDER BY priority
                    LIMIT 1
                    "#,
                )
                .bind(org_id).bind(source_book_id).bind(target_book_id)
                .bind(&pattern).bind(today)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
                Ok(row.map(|r| self.row_to_mapping(&r)))
            }
        }
    }

    async fn delete_account_mapping(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.account_mappings SET is_active = false WHERE id = $1"
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Journal Entries
    // ========================================================================

    async fn create_journal_entry(
        &self,
        org_id: Uuid,
        book_id: Uuid,
        entry_number: &str,
        header_description: Option<&str>,
        source_book_id: Option<Uuid>,
        source_entry_id: Option<Uuid>,
        external_reference: Option<&str>,
        accounting_date: chrono::NaiveDate,
        period_name: Option<&str>,
        total_debit: &str,
        total_credit: &str,
        status: &str,
        is_auto_propagated: bool,
        currency_code: &str,
        conversion_rate: Option<&str>,
        metadata: serde_json::Value,
        created_by: Option<Uuid>,
    ) -> AtlasResult<BookJournalEntry> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.book_journal_entries
                (organization_id, book_id, entry_number, header_description,
                 source_book_id, source_entry_id, external_reference,
                 accounting_date, period_name,
                 total_debit, total_credit, status, is_auto_propagated,
                 currency_code, conversion_rate, metadata, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9,
                    $10::numeric, $11::numeric, $12, $13, $14, $15::numeric, $16, $17)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(book_id).bind(entry_number).bind(header_description)
        .bind(source_book_id).bind(source_entry_id).bind(external_reference)
        .bind(accounting_date).bind(period_name)
        .bind(total_debit).bind(total_credit).bind(status)
        .bind(is_auto_propagated).bind(currency_code).bind(conversion_rate)
        .bind(metadata).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_entry(&row))
    }

    async fn get_journal_entry_by_id(&self, id: Uuid) -> AtlasResult<Option<BookJournalEntry>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.book_journal_entries WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_entry(&r)))
    }

    async fn list_journal_entries(
        &self,
        org_id: Uuid,
        book_id: Uuid,
        status: Option<&str>,
    ) -> AtlasResult<Vec<BookJournalEntry>> {
        let rows = match status {
            Some(s) => sqlx::query(
                "SELECT * FROM _atlas.book_journal_entries WHERE organization_id = $1 AND book_id = $2 AND status = $3 ORDER BY created_at DESC"
            )
            .bind(org_id).bind(book_id).bind(s)
            .fetch_all(&self.pool).await,
            None => sqlx::query(
                "SELECT * FROM _atlas.book_journal_entries WHERE organization_id = $1 AND book_id = $2 ORDER BY created_at DESC"
            )
            .bind(org_id).bind(book_id)
            .fetch_all(&self.pool).await,
        }
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_entry(r)).collect())
    }

    async fn update_journal_entry_status(
        &self,
        id: Uuid,
        status: &str,
        posted_by: Option<Uuid>,
    ) -> AtlasResult<BookJournalEntry> {
        let posted_at_expr = if status == "posted" { "COALESCE(posted_at, now())" } else { "posted_at" };
        let query_str = format!(
            r#"UPDATE _atlas.book_journal_entries
            SET status = $2, posted_by = $3, posted_at = {}, updated_at = now()
            WHERE id = $1
            RETURNING *"#,
            posted_at_expr
        );
        let row = sqlx::query(&query_str)
            .bind(id).bind(status).bind(posted_by)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_entry(&row))
    }

    // ========================================================================
    // Journal Lines
    // ========================================================================

    async fn create_journal_line(
        &self,
        org_id: Uuid,
        entry_id: Uuid,
        line_number: i32,
        account_code: &str,
        account_name: Option<&str>,
        debit_amount: &str,
        credit_amount: &str,
        description: Option<&str>,
        tax_code: Option<&str>,
        source_line_id: Option<Uuid>,
        metadata: serde_json::Value,
    ) -> AtlasResult<BookJournalLine> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.book_journal_lines
                (organization_id, entry_id, line_number, account_code, account_name,
                 debit_amount, credit_amount, description, tax_code,
                 source_line_id, metadata)
            VALUES ($1, $2, $3, $4, $5, $6::numeric, $7::numeric, $8, $9, $10, $11)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(entry_id).bind(line_number).bind(account_code)
        .bind(account_name).bind(debit_amount).bind(credit_amount)
        .bind(description).bind(tax_code).bind(source_line_id).bind(metadata)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_line(&row))
    }

    async fn list_journal_lines(&self, entry_id: Uuid) -> AtlasResult<Vec<BookJournalLine>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.book_journal_lines WHERE entry_id = $1 ORDER BY line_number"
        )
        .bind(entry_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_line(r)).collect())
    }

    // ========================================================================
    // Propagation Logs
    // ========================================================================

    async fn create_propagation_log(
        &self,
        org_id: Uuid,
        source_book_id: Uuid,
        target_book_id: Uuid,
        source_entry_id: Uuid,
        target_entry_id: Option<Uuid>,
        status: &str,
        lines_propagated: i32,
        lines_unmapped: i32,
        error_message: Option<&str>,
        metadata: serde_json::Value,
    ) -> AtlasResult<PropagationLog> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.propagation_logs
                (organization_id, source_book_id, target_book_id,
                 source_entry_id, target_entry_id,
                 status, lines_propagated, lines_unmapped,
                 error_message, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(source_book_id).bind(target_book_id)
        .bind(source_entry_id).bind(target_entry_id)
        .bind(status).bind(lines_propagated).bind(lines_unmapped)
        .bind(error_message).bind(metadata)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_propagation_log(&row))
    }

    async fn list_propagation_logs(
        &self,
        org_id: Uuid,
        source_book_id: Option<Uuid>,
        target_book_id: Option<Uuid>,
    ) -> AtlasResult<Vec<PropagationLog>> {
        let rows = match (source_book_id, target_book_id) {
            (Some(s), Some(t)) => sqlx::query(
                "SELECT * FROM _atlas.propagation_logs WHERE organization_id = $1 AND source_book_id = $2 AND target_book_id = $3 ORDER BY created_at DESC"
            )
            .bind(org_id).bind(s).bind(t)
            .fetch_all(&self.pool).await,
            (Some(s), None) => sqlx::query(
                "SELECT * FROM _atlas.propagation_logs WHERE organization_id = $1 AND source_book_id = $2 ORDER BY created_at DESC"
            )
            .bind(org_id).bind(s)
            .fetch_all(&self.pool).await,
            (None, Some(t)) => sqlx::query(
                "SELECT * FROM _atlas.propagation_logs WHERE organization_id = $1 AND target_book_id = $2 ORDER BY created_at DESC"
            )
            .bind(org_id).bind(t)
            .fetch_all(&self.pool).await,
            (None, None) => sqlx::query(
                "SELECT * FROM _atlas.propagation_logs WHERE organization_id = $1 ORDER BY created_at DESC LIMIT 100"
            )
            .bind(org_id)
            .fetch_all(&self.pool).await,
        }
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_propagation_log(r)).collect())
    }
}
