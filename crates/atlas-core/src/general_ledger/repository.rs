//! General Ledger Repository
//!
//! PostgreSQL storage for GL accounts, journal entries, journal lines.

use atlas_shared::{
    GlAccount, GlJournalEntry, GlJournalLine,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

/// Repository trait for General Ledger data storage
#[async_trait]
pub trait GeneralLedgerRepository: Send + Sync {
    // Accounts
    async fn create_account(
        &self,
        org_id: Uuid,
        account_code: &str,
        account_name: &str,
        description: Option<&str>,
        account_type: &str,
        subtype: Option<&str>,
        parent_account_id: Option<Uuid>,
        natural_balance: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<GlAccount>;

    async fn get_account(&self, id: Uuid) -> AtlasResult<Option<GlAccount>>;
    async fn get_account_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<GlAccount>>;
    async fn list_accounts(&self, org_id: Uuid, account_type: Option<&str>) -> AtlasResult<Vec<GlAccount>>;

    // Journal Entries
    async fn create_journal_entry(
        &self,
        org_id: Uuid,
        entry_number: &str,
        entry_date: chrono::NaiveDate,
        gl_date: chrono::NaiveDate,
        entry_type: &str,
        description: Option<&str>,
        currency_code: &str,
        source_type: Option<&str>,
        source_id: Option<Uuid>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<GlJournalEntry>;

    async fn get_journal_entry(&self, id: Uuid) -> AtlasResult<Option<GlJournalEntry>>;
    async fn get_journal_entry_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<GlJournalEntry>>;
    async fn list_journal_entries(&self, org_id: Uuid, status: Option<&str>, entry_type: Option<&str>) -> AtlasResult<Vec<GlJournalEntry>>;
    async fn update_journal_status(&self, id: Uuid, status: &str, posted_by: Option<Uuid>, reversal_entry_id: Option<Uuid>) -> AtlasResult<GlJournalEntry>;
    async fn update_journal_totals(&self, id: Uuid, total_debit: &str, total_credit: &str, is_balanced: bool) -> AtlasResult<()>;

    // Journal Lines
    async fn create_journal_line(
        &self,
        org_id: Uuid,
        journal_entry_id: Uuid,
        line_number: i32,
        line_type: &str,
        account_code: &str,
        account_name: Option<&str>,
        description: Option<&str>,
        entered_dr: &str,
        entered_cr: &str,
        accounted_dr: &str,
        accounted_cr: &str,
        currency_code: &str,
        exchange_rate: Option<&str>,
        reference: Option<&str>,
        tax_code: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<GlJournalLine>;

    async fn list_journal_lines(&self, journal_entry_id: Uuid) -> AtlasResult<Vec<GlJournalLine>>;

    // Trial Balance helpers
    async fn get_account_period_activity(&self, org_id: Uuid, account_code: &str, as_of_date: chrono::NaiveDate) -> AtlasResult<(f64, f64)>;
    async fn get_account_balance(&self, org_id: Uuid, account_code: &str, as_of_date: chrono::NaiveDate) -> AtlasResult<f64>;
}

/// PostgreSQL implementation
pub struct PostgresGeneralLedgerRepository {
    pool: PgPool,
}

impl PostgresGeneralLedgerRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

use sqlx::Row;

fn get_num(row: &sqlx::postgres::PgRow, col: &str) -> String {
    let v: f64 = row.try_get(col).unwrap_or(0.0);
    format!("{:.2}", v)
}

fn row_to_account(row: &sqlx::postgres::PgRow) -> GlAccount {
    GlAccount {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        account_code: row.get("account_code"),
        account_name: row.get("account_name"),
        description: row.get("description"),
        account_type: row.get("account_type"),
        subtype: row.get("subtype"),
        parent_account_id: row.get("parent_account_id"),
        is_active: row.get("is_active"),
        natural_balance: row.get("natural_balance"),
        third_party_control: row.get("third_party_control"),
        reconciliation_enabled: row.get("reconciliation_enabled"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_journal_entry(row: &sqlx::postgres::PgRow) -> GlJournalEntry {
    GlJournalEntry {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        entry_number: row.get("entry_number"),
        ledger_id: row.get("ledger_id"),
        entry_date: row.get("entry_date"),
        gl_date: row.get("gl_date"),
        entry_type: row.get("entry_type"),
        description: row.get("description"),
        currency_code: row.get("currency_code"),
        total_debit: get_num(row, "total_debit"),
        total_credit: get_num(row, "total_credit"),
        is_balanced: row.get("is_balanced"),
        status: row.get("status"),
        posted_by: row.get("posted_by"),
        posted_at: row.get("posted_at"),
        reversal_entry_id: row.get("reversal_entry_id"),
        source_type: row.get("source_type"),
        source_id: row.get("source_id"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_journal_line(row: &sqlx::postgres::PgRow) -> GlJournalLine {
    GlJournalLine {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        journal_entry_id: row.get("journal_entry_id"),
        line_number: row.get("line_number"),
        line_type: row.get("line_type"),
        account_code: row.get("account_code"),
        account_name: row.get("account_name"),
        description: row.get("description"),
        entered_dr: get_num(row, "entered_dr"),
        entered_cr: get_num(row, "entered_cr"),
        accounted_dr: get_num(row, "accounted_dr"),
        accounted_cr: get_num(row, "accounted_cr"),
        currency_code: row.get("currency_code"),
        exchange_rate: row.get("exchange_rate"),
        reference: row.get("reference"),
        tax_code: row.get("tax_code"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

#[async_trait]
impl GeneralLedgerRepository for PostgresGeneralLedgerRepository {
    async fn create_account(
        &self,
        org_id: Uuid,
        account_code: &str,
        account_name: &str,
        description: Option<&str>,
        account_type: &str,
        subtype: Option<&str>,
        parent_account_id: Option<Uuid>,
        natural_balance: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<GlAccount> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.gl_accounts
                (organization_id, account_code, account_name, description,
                 account_type, subtype, parent_account_id, natural_balance, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(account_code).bind(account_name).bind(description)
        .bind(account_type).bind(subtype).bind(parent_account_id)
        .bind(natural_balance).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_account(&row))
    }

    async fn get_account(&self, id: Uuid) -> AtlasResult<Option<GlAccount>> {
        let row = sqlx::query("SELECT * FROM _atlas.gl_accounts WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_account(&r)))
    }

    async fn get_account_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<GlAccount>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.gl_accounts WHERE organization_id = $1 AND account_code = $2 AND is_active = true"
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_account(&r)))
    }

    async fn list_accounts(&self, org_id: Uuid, account_type: Option<&str>) -> AtlasResult<Vec<GlAccount>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM _atlas.gl_accounts
            WHERE organization_id = $1 AND is_active = true
              AND ($2::text IS NULL OR account_type = $2)
            ORDER BY account_code
            "#,
        )
        .bind(org_id).bind(account_type)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_account).collect())
    }

    async fn create_journal_entry(
        &self,
        org_id: Uuid,
        entry_number: &str,
        entry_date: chrono::NaiveDate,
        gl_date: chrono::NaiveDate,
        entry_type: &str,
        description: Option<&str>,
        currency_code: &str,
        source_type: Option<&str>,
        source_id: Option<Uuid>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<GlJournalEntry> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.gl_journal_entries
                (organization_id, entry_number, entry_date, gl_date, entry_type,
                 description, currency_code, total_debit, total_credit, is_balanced,
                 status, source_type, source_id, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, 0, 0, false,
                    'draft', $8, $9, $10)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(entry_number).bind(entry_date).bind(gl_date).bind(entry_type)
        .bind(description).bind(currency_code).bind(source_type).bind(source_id).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_journal_entry(&row))
    }

    async fn get_journal_entry(&self, id: Uuid) -> AtlasResult<Option<GlJournalEntry>> {
        let row = sqlx::query("SELECT * FROM _atlas.gl_journal_entries WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_journal_entry(&r)))
    }

    async fn get_journal_entry_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<GlJournalEntry>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.gl_journal_entries WHERE organization_id = $1 AND entry_number = $2"
        )
        .bind(org_id).bind(number)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_journal_entry(&r)))
    }

    async fn list_journal_entries(&self, org_id: Uuid, status: Option<&str>, entry_type: Option<&str>) -> AtlasResult<Vec<GlJournalEntry>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM _atlas.gl_journal_entries
            WHERE organization_id = $1
              AND ($2::text IS NULL OR status = $2)
              AND ($3::text IS NULL OR entry_type = $3)
            ORDER BY gl_date DESC, created_at DESC
            "#,
        )
        .bind(org_id).bind(status).bind(entry_type)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_journal_entry).collect())
    }

    async fn update_journal_status(&self, id: Uuid, status: &str, posted_by: Option<Uuid>, reversal_entry_id: Option<Uuid>) -> AtlasResult<GlJournalEntry> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.gl_journal_entries
            SET status = $2,
                posted_by = CASE WHEN $2 = 'posted' THEN $3 ELSE posted_by END,
                posted_at = CASE WHEN $2 = 'posted' THEN now() ELSE posted_at END,
                reversal_entry_id = COALESCE($4, reversal_entry_id),
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(status).bind(posted_by).bind(reversal_entry_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_journal_entry(&row))
    }

    async fn update_journal_totals(&self, id: Uuid, total_debit: &str, total_credit: &str, is_balanced: bool) -> AtlasResult<()> {
        sqlx::query(
            r#"
            UPDATE _atlas.gl_journal_entries
            SET total_debit = $2::double precision,
                total_credit = $3::double precision,
                is_balanced = $4,
                updated_at = now()
            WHERE id = $1
            "#,
        )
        .bind(id).bind(total_debit).bind(total_credit).bind(is_balanced)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn create_journal_line(
        &self,
        org_id: Uuid,
        journal_entry_id: Uuid,
        line_number: i32,
        line_type: &str,
        account_code: &str,
        account_name: Option<&str>,
        description: Option<&str>,
        entered_dr: &str,
        entered_cr: &str,
        accounted_dr: &str,
        accounted_cr: &str,
        currency_code: &str,
        exchange_rate: Option<&str>,
        reference: Option<&str>,
        tax_code: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<GlJournalLine> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.gl_journal_lines
                (organization_id, journal_entry_id, line_number, line_type,
                 account_code, account_name, description,
                 entered_dr, entered_cr, accounted_dr, accounted_cr,
                 currency_code, exchange_rate, reference, tax_code, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7,
                    $8::double precision, $9::double precision,
                    $10::double precision, $11::double precision,
                    $12, $13, $14, $15, $16)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(journal_entry_id).bind(line_number).bind(line_type)
        .bind(account_code).bind(account_name).bind(description)
        .bind(entered_dr).bind(entered_cr).bind(accounted_dr).bind(accounted_cr)
        .bind(currency_code).bind(exchange_rate).bind(reference).bind(tax_code).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_journal_line(&row))
    }

    async fn list_journal_lines(&self, journal_entry_id: Uuid) -> AtlasResult<Vec<GlJournalLine>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.gl_journal_lines WHERE journal_entry_id = $1 ORDER BY line_number"
        )
        .bind(journal_entry_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_journal_line).collect())
    }

    async fn get_account_period_activity(&self, org_id: Uuid, account_code: &str, as_of_date: chrono::NaiveDate) -> AtlasResult<(f64, f64)> {
        let row = sqlx::query(
            r#"
            SELECT
                COALESCE(SUM(jl.accounted_dr), 0) as period_debit,
                COALESCE(SUM(jl.accounted_cr), 0) as period_credit
            FROM _atlas.gl_journal_lines jl
            JOIN _atlas.gl_journal_entries je ON jl.journal_entry_id = je.id
            WHERE je.organization_id = $1
              AND jl.account_code = $2
              AND je.status = 'posted'
              AND je.gl_date <= $3
            "#,
        )
        .bind(org_id).bind(account_code).bind(as_of_date)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok((
            row.try_get::<f64, _>("period_debit").unwrap_or(0.0),
            row.try_get::<f64, _>("period_credit").unwrap_or(0.0),
        ))
    }

    async fn get_account_balance(&self, _org_id: Uuid, _account_code: &str, _as_of_date: chrono::NaiveDate) -> AtlasResult<f64> {
        // For now, beginning balance is 0 (would need opening balances table)
        Ok(0.0)
    }
}
