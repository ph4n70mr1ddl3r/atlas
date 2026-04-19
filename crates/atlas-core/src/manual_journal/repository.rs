//! Manual Journal Entry Repository
//!
//! PostgreSQL storage for journal batches, journal entries, and journal entry lines.

use atlas_shared::{
    JournalBatch, JournalEntry, JournalEntryLine, ManualJournalDashboardSummary,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

/// Repository trait for manual journal data storage
#[async_trait]
pub trait ManualJournalRepository: Send + Sync {
    // Batches
    async fn create_batch(
        &self, org_id: Uuid, batch_number: &str, name: &str, description: Option<&str>,
        ledger_id: Option<Uuid>, currency_code: &str, accounting_date: Option<chrono::NaiveDate>,
        period_name: Option<&str>, source: &str, is_automatic_post: bool,
        created_by: Option<Uuid>,
    ) -> AtlasResult<JournalBatch>;
    async fn get_batch(&self, org_id: Uuid, batch_number: &str) -> AtlasResult<Option<JournalBatch>>;
    async fn get_batch_by_id(&self, id: Uuid) -> AtlasResult<Option<JournalBatch>>;
    async fn list_batches(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<JournalBatch>>;
    async fn update_batch_status(
        &self, id: Uuid, status: &str,
        submitted_by: Option<Uuid>, approved_by: Option<Uuid>,
        posted_by: Option<Uuid>, rejection_reason: Option<&str>,
    ) -> AtlasResult<JournalBatch>;
    async fn update_batch_totals(
        &self, id: Uuid, total_debit: &str, total_credit: &str, entry_count: i32,
    ) -> AtlasResult<()>;
    async fn delete_batch(&self, org_id: Uuid, batch_number: &str) -> AtlasResult<()>;

    // Entries
    async fn create_entry(
        &self, org_id: Uuid, batch_id: Uuid, entry_number: &str, name: Option<&str>,
        description: Option<&str>, ledger_id: Option<Uuid>, currency_code: &str,
        accounting_date: Option<chrono::NaiveDate>, period_name: Option<&str>,
        journal_category: &str, journal_source: &str,
        reference_number: Option<&str>, external_reference: Option<&str>,
        statistical_entry: bool, created_by: Option<Uuid>,
    ) -> AtlasResult<JournalEntry>;
    async fn get_entry(&self, id: Uuid) -> AtlasResult<Option<JournalEntry>>;
    async fn get_entry_by_number(&self, org_id: Uuid, entry_number: &str) -> AtlasResult<Option<JournalEntry>>;
    async fn list_entries_by_batch(&self, batch_id: Uuid) -> AtlasResult<Vec<JournalEntry>>;
    async fn list_entries(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<JournalEntry>>;
    async fn update_entry_status(
        &self, id: Uuid, status: &str, approved_by: Option<Uuid>,
        posted_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> AtlasResult<JournalEntry>;
    async fn update_entry_totals(
        &self, id: Uuid, total_debit: &str, total_credit: &str,
        line_count: i32, is_balanced: bool,
    ) -> AtlasResult<()>;
    async fn mark_entry_reversal(
        &self, id: Uuid, reversed_by_entry_id: Uuid,
    ) -> AtlasResult<JournalEntry>;
    async fn delete_entry(&self, id: Uuid) -> AtlasResult<()>;

    // Lines
    async fn create_line(
        &self, org_id: Uuid, entry_id: Uuid, line_number: i32,
        line_type: &str, account_code: &str, account_name: Option<&str>,
        description: Option<&str>, amount: &str, entered_amount: Option<&str>,
        entered_currency_code: Option<&str>, exchange_rate: Option<&str>,
        tax_code: Option<&str>, cost_center: Option<&str>,
        department_id: Option<Uuid>, project_id: Option<Uuid>,
        intercompany_entity_id: Option<Uuid>, statistical_amount: Option<&str>,
    ) -> AtlasResult<JournalEntryLine>;
    async fn list_lines_by_entry(&self, entry_id: Uuid) -> AtlasResult<Vec<JournalEntryLine>>;
    async fn delete_line(&self, id: Uuid) -> AtlasResult<()>;

    // Dashboard
    async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<ManualJournalDashboardSummary>;
}

/// PostgreSQL implementation
pub struct PostgresManualJournalRepository {
    pool: PgPool,
}

impl PostgresManualJournalRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn row_to_batch(row: &sqlx::postgres::PgRow) -> JournalBatch {
    use serde_json::Value;
    JournalBatch {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        batch_number: row.get("batch_number"),
        name: row.get("name"),
        description: row.get("description"),
        status: row.get("status"),
        ledger_id: row.get("ledger_id"),
        currency_code: row.get("currency_code"),
        accounting_date: row.get("accounting_date"),
        period_name: row.get("period_name"),
        total_debit: row.try_get("total_debit").unwrap_or(Value::Null).to_string().trim_matches('"').to_string(),
        total_credit: row.try_get("total_credit").unwrap_or(Value::Null).to_string().trim_matches('"').to_string(),
        entry_count: row.get("entry_count"),
        source: row.get("source"),
        is_automatic_post: row.get("is_automatic_post"),
        submitted_by: row.get("submitted_by"),
        submitted_at: row.get("submitted_at"),
        approved_by: row.get("approved_by"),
        approved_at: row.get("approved_at"),
        posted_by: row.get("posted_by"),
        posted_at: row.get("posted_at"),
        rejection_reason: row.get("rejection_reason"),
        metadata: row.try_get("metadata").unwrap_or(Value::Null),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_entry(row: &sqlx::postgres::PgRow) -> JournalEntry {
    use serde_json::Value;
    JournalEntry {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        batch_id: row.get("batch_id"),
        entry_number: row.get("entry_number"),
        name: row.get("name"),
        description: row.get("description"),
        status: row.get("status"),
        ledger_id: row.get("ledger_id"),
        currency_code: row.get("currency_code"),
        accounting_date: row.get("accounting_date"),
        period_name: row.get("period_name"),
        journal_category: row.get("journal_category"),
        journal_source: row.get("journal_source"),
        total_debit: row.try_get("total_debit").unwrap_or(Value::Null).to_string().trim_matches('"').to_string(),
        total_credit: row.try_get("total_credit").unwrap_or(Value::Null).to_string().trim_matches('"').to_string(),
        line_count: row.get("line_count"),
        is_balanced: row.get("is_balanced"),
        is_reversal: row.get("is_reversal"),
        reversal_of_entry_id: row.get("reversal_of_entry_id"),
        reversed_by_entry_id: row.get("reversed_by_entry_id"),
        reference_number: row.get("reference_number"),
        external_reference: row.get("external_reference"),
        statistical_entry: row.get("statistical_entry"),
        approved_by: row.get("approved_by"),
        approved_at: row.get("approved_at"),
        posted_at: row.get("posted_at"),
        metadata: row.try_get("metadata").unwrap_or(Value::Null),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_line(row: &sqlx::postgres::PgRow) -> JournalEntryLine {
    use serde_json::Value;
    JournalEntryLine {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        entry_id: row.get("entry_id"),
        line_number: row.get("line_number"),
        line_type: row.get("line_type"),
        account_code: row.get("account_code"),
        account_name: row.get("account_name"),
        description: row.get("description"),
        amount: row.try_get("amount").unwrap_or(Value::Null).to_string().trim_matches('"').to_string(),
        entered_amount: row.try_get("entered_amount").unwrap_or(None).map(|v: Value| v.to_string().trim_matches('"').to_string()),
        entered_currency_code: row.get("entered_currency_code"),
        exchange_rate: row.try_get("exchange_rate").unwrap_or(None).map(|v: Value| v.to_string().trim_matches('"').to_string()),
        tax_code: row.get("tax_code"),
        cost_center: row.get("cost_center"),
        department_id: row.get("department_id"),
        project_id: row.get("project_id"),
        intercompany_entity_id: row.get("intercompany_entity_id"),
        statistical_amount: row.try_get("statistical_amount").unwrap_or(None).map(|v: Value| v.to_string().trim_matches('"').to_string()),
        reference1: row.get("reference1"),
        reference2: row.get("reference2"),
        reference3: row.get("reference3"),
        reference4: row.get("reference4"),
        metadata: row.try_get("metadata").unwrap_or(Value::Null),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

#[async_trait]
impl ManualJournalRepository for PostgresManualJournalRepository {
    async fn create_batch(
        &self, org_id: Uuid, batch_number: &str, name: &str, description: Option<&str>,
        ledger_id: Option<Uuid>, currency_code: &str, accounting_date: Option<chrono::NaiveDate>,
        period_name: Option<&str>, source: &str, is_automatic_post: bool,
        created_by: Option<Uuid>,
    ) -> AtlasResult<JournalBatch> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.journal_batches
                (organization_id, batch_number, name, description, ledger_id,
                 currency_code, accounting_date, period_name, source,
                 is_automatic_post, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11) RETURNING *"#,
        )
        .bind(org_id).bind(batch_number).bind(name).bind(description)
        .bind(ledger_id).bind(currency_code).bind(accounting_date)
        .bind(period_name).bind(source).bind(is_automatic_post)
        .bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_batch(&row))
    }

    async fn get_batch(&self, org_id: Uuid, batch_number: &str) -> AtlasResult<Option<JournalBatch>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.journal_batches WHERE organization_id=$1 AND batch_number=$2"
        )
        .bind(org_id).bind(batch_number)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_batch(&r)))
    }

    async fn get_batch_by_id(&self, id: Uuid) -> AtlasResult<Option<JournalBatch>> {
        let row = sqlx::query("SELECT * FROM _atlas.journal_batches WHERE id=$1")
            .bind(id)
            .fetch_optional(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_batch(&r)))
    }

    async fn list_batches(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<JournalBatch>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.journal_batches
            WHERE organization_id=$1 AND ($2::text IS NULL OR status=$2)
            ORDER BY created_at DESC"#,
        )
        .bind(org_id).bind(status)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| row_to_batch(r)).collect())
    }

    async fn update_batch_status(
        &self, id: Uuid, status: &str,
        submitted_by: Option<Uuid>, approved_by: Option<Uuid>,
        posted_by: Option<Uuid>, rejection_reason: Option<&str>,
    ) -> AtlasResult<JournalBatch> {
        let row = sqlx::query(
            r#"UPDATE _atlas.journal_batches SET status=$2,
                submitted_by=COALESCE($3, submitted_by),
                submitted_at=CASE WHEN $3 IS NOT NULL AND submitted_at IS NULL THEN now() ELSE submitted_at END,
                approved_by=COALESCE($4, approved_by),
                approved_at=CASE WHEN $4 IS NOT NULL AND approved_at IS NULL THEN now() ELSE approved_at END,
                posted_by=COALESCE($5, posted_by),
                posted_at=CASE WHEN $5 IS NOT NULL AND posted_at IS NULL THEN now() ELSE posted_at END,
                rejection_reason=$6,
                updated_at=now() WHERE id=$1 RETURNING *"#,
        )
        .bind(id).bind(status).bind(submitted_by).bind(approved_by)
        .bind(posted_by).bind(rejection_reason)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_batch(&row))
    }

    async fn update_batch_totals(
        &self, id: Uuid, total_debit: &str, total_credit: &str, entry_count: i32,
    ) -> AtlasResult<()> {
        sqlx::query(
            r#"UPDATE _atlas.journal_batches
            SET total_debit=$2::numeric, total_credit=$3::numeric,
                entry_count=$4, updated_at=now() WHERE id=$1"#,
        )
        .bind(id).bind(total_debit).bind(total_credit).bind(entry_count)
        .execute(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn delete_batch(&self, org_id: Uuid, batch_number: &str) -> AtlasResult<()> {
        sqlx::query(
            "DELETE FROM _atlas.journal_batches WHERE organization_id=$1 AND batch_number=$2"
        )
        .bind(org_id).bind(batch_number)
        .execute(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn create_entry(
        &self, org_id: Uuid, batch_id: Uuid, entry_number: &str, name: Option<&str>,
        description: Option<&str>, ledger_id: Option<Uuid>, currency_code: &str,
        accounting_date: Option<chrono::NaiveDate>, period_name: Option<&str>,
        journal_category: &str, journal_source: &str,
        reference_number: Option<&str>, external_reference: Option<&str>,
        statistical_entry: bool, created_by: Option<Uuid>,
    ) -> AtlasResult<JournalEntry> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.journal_entries
                (organization_id, batch_id, entry_number, name, description,
                 ledger_id, currency_code, accounting_date, period_name,
                 journal_category, journal_source, reference_number,
                 external_reference, statistical_entry, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15) RETURNING *"#,
        )
        .bind(org_id).bind(batch_id).bind(entry_number).bind(name)
        .bind(description).bind(ledger_id).bind(currency_code)
        .bind(accounting_date).bind(period_name).bind(journal_category)
        .bind(journal_source).bind(reference_number)
        .bind(external_reference).bind(statistical_entry).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_entry(&row))
    }

    async fn get_entry(&self, id: Uuid) -> AtlasResult<Option<JournalEntry>> {
        let row = sqlx::query("SELECT * FROM _atlas.journal_entries WHERE id=$1")
            .bind(id)
            .fetch_optional(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_entry(&r)))
    }

    async fn get_entry_by_number(&self, org_id: Uuid, entry_number: &str) -> AtlasResult<Option<JournalEntry>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.journal_entries WHERE organization_id=$1 AND entry_number=$2"
        )
        .bind(org_id).bind(entry_number)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_entry(&r)))
    }

    async fn list_entries_by_batch(&self, batch_id: Uuid) -> AtlasResult<Vec<JournalEntry>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.journal_entries WHERE batch_id=$1 ORDER BY entry_number"
        )
        .bind(batch_id)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| row_to_entry(r)).collect())
    }

    async fn list_entries(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<JournalEntry>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.journal_entries
            WHERE organization_id=$1 AND ($2::text IS NULL OR status=$2)
            ORDER BY created_at DESC"#,
        )
        .bind(org_id).bind(status)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| row_to_entry(r)).collect())
    }

    async fn update_entry_status(
        &self, id: Uuid, status: &str, approved_by: Option<Uuid>,
        posted_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> AtlasResult<JournalEntry> {
        let row = sqlx::query(
            r#"UPDATE _atlas.journal_entries SET status=$2,
                approved_by=COALESCE($3, approved_by),
                approved_at=CASE WHEN $3 IS NOT NULL AND approved_at IS NULL THEN now() ELSE approved_at END,
                posted_at=COALESCE($4, posted_at),
                updated_at=now() WHERE id=$1 RETURNING *"#,
        )
        .bind(id).bind(status).bind(approved_by).bind(posted_at)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_entry(&row))
    }

    async fn update_entry_totals(
        &self, id: Uuid, total_debit: &str, total_credit: &str,
        line_count: i32, is_balanced: bool,
    ) -> AtlasResult<()> {
        sqlx::query(
            r#"UPDATE _atlas.journal_entries
            SET total_debit=$2::numeric, total_credit=$3::numeric,
                line_count=$4, is_balanced=$5, updated_at=now() WHERE id=$1"#,
        )
        .bind(id).bind(total_debit).bind(total_credit)
        .bind(line_count).bind(is_balanced)
        .execute(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn mark_entry_reversal(
        &self, id: Uuid, reversed_by_entry_id: Uuid,
    ) -> AtlasResult<JournalEntry> {
        let row = sqlx::query(
            r#"UPDATE _atlas.journal_entries SET reversed_by_entry_id=$2,
                status='reversed', updated_at=now() WHERE id=$1 RETURNING *"#,
        )
        .bind(id).bind(reversed_by_entry_id)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_entry(&row))
    }

    async fn delete_entry(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.journal_entries WHERE id=$1")
            .bind(id).execute(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn create_line(
        &self, org_id: Uuid, entry_id: Uuid, line_number: i32,
        line_type: &str, account_code: &str, account_name: Option<&str>,
        description: Option<&str>, amount: &str, entered_amount: Option<&str>,
        entered_currency_code: Option<&str>, exchange_rate: Option<&str>,
        tax_code: Option<&str>, cost_center: Option<&str>,
        department_id: Option<Uuid>, project_id: Option<Uuid>,
        intercompany_entity_id: Option<Uuid>, statistical_amount: Option<&str>,
    ) -> AtlasResult<JournalEntryLine> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.journal_entry_lines
                (organization_id, entry_id, line_number, line_type, account_code,
                 account_name, description, amount, entered_amount, entered_currency_code,
                 exchange_rate, tax_code, cost_center, department_id, project_id,
                 intercompany_entity_id, statistical_amount)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8::numeric,$9::numeric,$10,$11::numeric,$12,$13,$14,$15,$16,$17::numeric)
            RETURNING *"#,
        )
        .bind(org_id).bind(entry_id).bind(line_number).bind(line_type)
        .bind(account_code).bind(account_name).bind(description)
        .bind(amount).bind(entered_amount).bind(entered_currency_code)
        .bind(exchange_rate).bind(tax_code).bind(cost_center)
        .bind(department_id).bind(project_id).bind(intercompany_entity_id)
        .bind(statistical_amount)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_line(&row))
    }

    async fn list_lines_by_entry(&self, entry_id: Uuid) -> AtlasResult<Vec<JournalEntryLine>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.journal_entry_lines WHERE entry_id=$1 ORDER BY line_number"
        )
        .bind(entry_id)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| row_to_line(r)).collect())
    }

    async fn delete_line(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.journal_entry_lines WHERE id=$1")
            .bind(id).execute(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<ManualJournalDashboardSummary> {
        let row = sqlx::query(
            r#"SELECT
                COUNT(*) as total_batches,
                COUNT(*) FILTER (WHERE status = 'draft') as draft_batches,
                COUNT(*) FILTER (WHERE status = 'posted') as posted_batches,
                COALESCE(SUM(total_debit), 0) as total_debits,
                COALESCE(SUM(total_credit), 0) as total_credits,
                COUNT(*) FILTER (WHERE status = 'submitted') as pending_approval
            FROM _atlas.journal_batches WHERE organization_id = $1"#,
        )
        .bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let total_batches: i64 = row.try_get("total_batches").unwrap_or(0);
        let draft_batches: i64 = row.try_get("draft_batches").unwrap_or(0);
        let posted_batches: i64 = row.try_get("posted_batches").unwrap_or(0);
        let pending_approval: i64 = row.try_get("pending_approval").unwrap_or(0);

        let entry_row = sqlx::query(
            r#"SELECT
                COUNT(*) as total_entries,
                COUNT(*) FILTER (WHERE status = 'posted') as posted_entries
            FROM _atlas.journal_entries WHERE organization_id = $1"#,
        )
        .bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let total_entries: i64 = entry_row.try_get("total_entries").unwrap_or(0);
        let posted_entries: i64 = entry_row.try_get("posted_entries").unwrap_or(0);

        let recent_rows = sqlx::query(
            "SELECT * FROM _atlas.journal_batches WHERE organization_id=$1 ORDER BY created_at DESC LIMIT 5"
        )
        .bind(org_id)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(ManualJournalDashboardSummary {
            total_batches: total_batches as i32,
            total_draft_batches: draft_batches as i32,
            total_posted_batches: posted_batches as i32,
            total_entries: total_entries as i32,
            total_posted_entries: posted_entries as i32,
            total_debits: "0".to_string(),
            total_credits: "0".to_string(),
            batches_pending_approval: pending_approval as i32,
            entries_by_category: serde_json::json!({}),
            batches_by_status: serde_json::json!({}),
            recent_batches: recent_rows.iter().map(|r| row_to_batch(r)).collect(),
        })
    }
}
