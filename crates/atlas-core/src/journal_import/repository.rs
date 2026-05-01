//! Journal Import Repository
//!
//! Storage interface for journal import data.

use atlas_shared::{
    JournalImportFormat, JournalImportColumnMapping,
    JournalImportBatch, JournalImportRow,
    JournalImportDashboardSummary,
    AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

/// Repository trait for journal import data storage
#[async_trait]
pub trait JournalImportRepository: Send + Sync {
    // Format Management
    async fn create_format(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        source_type: &str,
        file_format: &str,
        delimiter: Option<&str>,
        header_row: bool,
        ledger_id: Option<Uuid>,
        currency_code: &str,
        default_date: Option<chrono::NaiveDate>,
        default_journal_type: Option<&str>,
        balancing_segment: Option<&str>,
        validation_enabled: bool,
        auto_post: bool,
        max_errors_allowed: i32,
        column_mappings: serde_json::Value,
        created_by: Option<Uuid>,
    ) -> AtlasResult<JournalImportFormat>;

    async fn get_format(&self, id: Uuid) -> AtlasResult<Option<JournalImportFormat>>;
    async fn get_format_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<JournalImportFormat>>;
    async fn list_formats(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<JournalImportFormat>>;
    async fn delete_format(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // Column Mappings
    async fn create_column_mapping(
        &self,
        org_id: Uuid,
        format_id: Uuid,
        column_position: i32,
        source_column: &str,
        target_field: &str,
        data_type: &str,
        is_required: bool,
        default_value: Option<&str>,
        transformation: Option<&str>,
        validation_rule: Option<&str>,
    ) -> AtlasResult<JournalImportColumnMapping>;

    async fn list_column_mappings(&self, format_id: Uuid) -> AtlasResult<Vec<JournalImportColumnMapping>>;

    // Batch Management
    async fn create_batch(
        &self,
        org_id: Uuid,
        format_id: Uuid,
        batch_number: &str,
        name: Option<&str>,
        description: Option<&str>,
        source: &str,
        source_file_name: Option<&str>,
        ledger_id: Option<Uuid>,
        currency_code: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<JournalImportBatch>;

    async fn get_batch(&self, id: Uuid) -> AtlasResult<Option<JournalImportBatch>>;
    async fn get_batch_by_number(&self, org_id: Uuid, batch_number: &str) -> AtlasResult<Option<JournalImportBatch>>;
    async fn list_batches(&self, org_id: Uuid, format_id: Option<Uuid>, status: Option<&str>) -> AtlasResult<Vec<JournalImportBatch>>;
    async fn update_batch_status(
        &self,
        id: Uuid,
        status: &str,
        started_at: Option<chrono::DateTime<chrono::Utc>>,
        completed_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> AtlasResult<JournalImportBatch>;
    async fn update_batch_totals(
        &self,
        id: Uuid,
        total_rows: i32,
        valid_rows: i32,
        error_rows: i32,
        imported_rows: i32,
        total_debit: &str,
        total_credit: &str,
        is_balanced: bool,
        errors: serde_json::Value,
    ) -> AtlasResult<()>;
    async fn delete_batch(&self, id: Uuid) -> AtlasResult<()>;
    async fn delete_batch_rows(&self, batch_id: Uuid) -> AtlasResult<()>;

    // Row Management
    async fn create_row(
        &self,
        org_id: Uuid,
        batch_id: Uuid,
        row_number: i32,
        raw_data: serde_json::Value,
        account_code: Option<&str>,
        account_name: Option<&str>,
        description: Option<&str>,
        entered_dr: &str,
        entered_cr: &str,
        currency_code: Option<&str>,
        exchange_rate: Option<&str>,
        gl_date: Option<chrono::NaiveDate>,
        reference: Option<&str>,
        line_type: Option<&str>,
        cost_center: Option<&str>,
        department: Option<&str>,
        project_code: Option<&str>,
        status: &str,
        error_message: Option<&str>,
        error_field: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<JournalImportRow>;

    async fn get_row(&self, id: Uuid) -> AtlasResult<Option<JournalImportRow>>;
    async fn list_batch_rows(&self, batch_id: Uuid) -> AtlasResult<Vec<JournalImportRow>>;
    async fn update_row(
        &self,
        id: Uuid,
        account_code: Option<&str>,
        description: Option<&str>,
        entered_dr: Option<&str>,
        entered_cr: Option<&str>,
        status: &str,
        error_message: Option<&str>,
        error_field: Option<&str>,
    ) -> AtlasResult<JournalImportRow>;

    // Dashboard
    async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<JournalImportDashboardSummary>;
}

/// PostgreSQL implementation
pub struct PostgresJournalImportRepository {
    pool: PgPool,
}

impl PostgresJournalImportRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

use sqlx::Row;

fn row_to_format(row: &sqlx::postgres::PgRow) -> JournalImportFormat {
    JournalImportFormat {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        code: row.get("code"),
        name: row.get("name"),
        description: row.get("description"),
        source_type: row.get("source_type"),
        file_format: row.get("file_format"),
        delimiter: row.get("delimiter"),
        header_row: row.get("header_row"),
        ledger_id: row.get("ledger_id"),
        currency_code: row.get("currency_code"),
        default_date: row.get("default_date"),
        default_journal_type: row.get("default_journal_type"),
        balancing_segment: row.get("balancing_segment"),
        status: row.get("status"),
        validation_enabled: row.get("validation_enabled"),
        auto_post: row.get("auto_post"),
        max_errors_allowed: row.try_get("max_errors_allowed").unwrap_or(100),
        column_mappings: row.try_get("column_mappings").unwrap_or(serde_json::json!([])),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_column_mapping(row: &sqlx::postgres::PgRow) -> JournalImportColumnMapping {
    JournalImportColumnMapping {
        id: row.get("id"),
        format_id: row.get("format_id"),
        column_position: row.get("column_position"),
        source_column: row.get("source_column"),
        target_field: row.get("target_field"),
        data_type: row.get("data_type"),
        is_required: row.get("is_required"),
        default_value: row.get("default_value"),
        transformation: row.get("transformation"),
        validation_rule: row.get("validation_rule"),
        created_at: row.get("created_at"),
    }
}

fn row_to_batch(row: &sqlx::postgres::PgRow) -> JournalImportBatch {
    JournalImportBatch {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        format_id: row.get("format_id"),
        batch_number: row.get("batch_number"),
        name: row.get("name"),
        description: row.get("description"),
        source: row.get("source"),
        source_file_name: row.get("source_file_name"),
        status: row.get("status"),
        total_rows: row.try_get("total_rows").unwrap_or(0),
        valid_rows: row.try_get("valid_rows").unwrap_or(0),
        error_rows: row.try_get("error_rows").unwrap_or(0),
        imported_rows: row.try_get("imported_rows").unwrap_or(0),
        ledger_id: row.get("ledger_id"),
        currency_code: row.get("currency_code"),
        journal_batch_id: row.get("journal_batch_id"),
        total_debit: row.try_get("total_debit").unwrap_or("0".to_string()),
        total_credit: row.try_get("total_credit").unwrap_or("0".to_string()),
        is_balanced: row.try_get("is_balanced").unwrap_or(false),
        errors: row.try_get("errors").unwrap_or(serde_json::json!([])),
        started_at: row.get("started_at"),
        completed_at: row.get("completed_at"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_import_row(row: &sqlx::postgres::PgRow) -> JournalImportRow {
    JournalImportRow {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        batch_id: row.get("batch_id"),
        row_number: row.get("row_number"),
        raw_data: row.try_get("raw_data").unwrap_or(serde_json::json!({})),
        status: row.get("status"),
        account_code: row.get("account_code"),
        account_name: row.get("account_name"),
        description: row.get("description"),
        entered_dr: row.try_get("entered_dr").unwrap_or("0".to_string()),
        entered_cr: row.try_get("entered_cr").unwrap_or("0".to_string()),
        currency_code: row.get("currency_code"),
        exchange_rate: row.get("exchange_rate"),
        gl_date: row.get("gl_date"),
        reference: row.get("reference"),
        line_type: row.get("line_type"),
        cost_center: row.get("cost_center"),
        department: row.get("department"),
        project_code: row.get("project_code"),
        error_message: row.get("error_message"),
        error_field: row.get("error_field"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

use atlas_shared::AtlasError;

#[async_trait]
impl JournalImportRepository for PostgresJournalImportRepository {
    async fn create_format(
        &self,
        org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        source_type: &str, file_format: &str, delimiter: Option<&str>,
        header_row: bool, ledger_id: Option<Uuid>, currency_code: &str,
        default_date: Option<chrono::NaiveDate>,
        default_journal_type: Option<&str>, balancing_segment: Option<&str>,
        validation_enabled: bool, auto_post: bool,
        max_errors_allowed: i32, column_mappings: serde_json::Value,
        created_by: Option<Uuid>,
    ) -> AtlasResult<JournalImportFormat> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.journal_import_formats
                (organization_id, code, name, description,
                 source_type, file_format, delimiter, header_row,
                 ledger_id, currency_code, default_date,
                 default_journal_type, balancing_segment,
                 validation_enabled, auto_post,
                 max_errors_allowed, column_mappings, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13,
                    $14, $15, $16, $17, $18)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(code).bind(name).bind(description)
        .bind(source_type).bind(file_format).bind(delimiter).bind(header_row)
        .bind(ledger_id).bind(currency_code).bind(default_date)
        .bind(default_journal_type).bind(balancing_segment)
        .bind(validation_enabled).bind(auto_post)
        .bind(max_errors_allowed).bind(column_mappings).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_format(&row))
    }

    async fn get_format(&self, id: Uuid) -> AtlasResult<Option<JournalImportFormat>> {
        let row = sqlx::query("SELECT * FROM _atlas.journal_import_formats WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_format(&r)))
    }

    async fn get_format_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<JournalImportFormat>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.journal_import_formats WHERE organization_id = $1 AND code = $2"
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_format(&r)))
    }

    async fn list_formats(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<JournalImportFormat>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM _atlas.journal_import_formats
            WHERE organization_id = $1 AND ($2::text IS NULL OR status = $2)
            ORDER BY name
            "#,
        )
        .bind(org_id).bind(status)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_format).collect())
    }

    async fn delete_format(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.journal_import_formats SET status = 'inactive', updated_at = now() WHERE organization_id = $1 AND code = $2"
        )
        .bind(org_id).bind(code)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn create_column_mapping(
        &self,
        org_id: Uuid, format_id: Uuid, column_position: i32,
        source_column: &str, target_field: &str, data_type: &str,
        is_required: bool, default_value: Option<&str>,
        transformation: Option<&str>, validation_rule: Option<&str>,
    ) -> AtlasResult<JournalImportColumnMapping> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.journal_import_column_mappings
                (organization_id, format_id, column_position,
                 source_column, target_field, data_type,
                 is_required, default_value, transformation, validation_rule)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(format_id).bind(column_position)
        .bind(source_column).bind(target_field).bind(data_type)
        .bind(is_required).bind(default_value).bind(transformation).bind(validation_rule)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_column_mapping(&row))
    }

    async fn list_column_mappings(&self, format_id: Uuid) -> AtlasResult<Vec<JournalImportColumnMapping>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.journal_import_column_mappings WHERE format_id = $1 ORDER BY column_position"
        )
        .bind(format_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_column_mapping).collect())
    }

    async fn create_batch(
        &self,
        org_id: Uuid, format_id: Uuid, batch_number: &str,
        name: Option<&str>, description: Option<&str>,
        source: &str, source_file_name: Option<&str>,
        ledger_id: Option<Uuid>, currency_code: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<JournalImportBatch> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.journal_import_batches
                (organization_id, format_id, batch_number, name, description,
                 source, source_file_name, status,
                 total_rows, valid_rows, error_rows, imported_rows,
                 ledger_id, currency_code,
                 total_debit, total_credit, is_balanced,
                 errors, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, 'uploaded',
                    0, 0, 0, 0, $8, $9, '0', '0', false, '[]', $10)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(format_id).bind(batch_number).bind(name).bind(description)
        .bind(source).bind(source_file_name)
        .bind(ledger_id).bind(currency_code).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_batch(&row))
    }

    async fn get_batch(&self, id: Uuid) -> AtlasResult<Option<JournalImportBatch>> {
        let row = sqlx::query("SELECT * FROM _atlas.journal_import_batches WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_batch(&r)))
    }

    async fn get_batch_by_number(&self, org_id: Uuid, batch_number: &str) -> AtlasResult<Option<JournalImportBatch>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.journal_import_batches WHERE organization_id = $1 AND batch_number = $2"
        )
        .bind(org_id).bind(batch_number)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_batch(&r)))
    }

    async fn list_batches(&self, org_id: Uuid, format_id: Option<Uuid>, status: Option<&str>) -> AtlasResult<Vec<JournalImportBatch>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM _atlas.journal_import_batches
            WHERE organization_id = $1
              AND ($2::uuid IS NULL OR format_id = $2)
              AND ($3::text IS NULL OR status = $3)
            ORDER BY created_at DESC
            "#,
        )
        .bind(org_id).bind(format_id).bind(status)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_batch).collect())
    }

    async fn update_batch_status(
        &self, id: Uuid, status: &str,
        started_at: Option<chrono::DateTime<chrono::Utc>>,
        completed_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> AtlasResult<JournalImportBatch> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.journal_import_batches
            SET status = $2,
                started_at = COALESCE($3, started_at),
                completed_at = COALESCE($4, completed_at),
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(status).bind(started_at).bind(completed_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_batch(&row))
    }

    async fn update_batch_totals(
        &self, id: Uuid,
        total_rows: i32, valid_rows: i32, error_rows: i32, imported_rows: i32,
        total_debit: &str, total_credit: &str,
        is_balanced: bool, errors: serde_json::Value,
    ) -> AtlasResult<()> {
        sqlx::query(
            r#"
            UPDATE _atlas.journal_import_batches
            SET total_rows = $2, valid_rows = $3, error_rows = $4, imported_rows = $5,
                total_debit = $6::numeric, total_credit = $7::numeric,
                is_balanced = $8, errors = $9, updated_at = now()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(total_rows).bind(valid_rows).bind(error_rows).bind(imported_rows)
        .bind(total_debit).bind(total_credit).bind(is_balanced).bind(errors)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn delete_batch(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.journal_import_batches WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn delete_batch_rows(&self, batch_id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.journal_import_rows WHERE batch_id = $1")
            .bind(batch_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn create_row(
        &self,
        org_id: Uuid, batch_id: Uuid, row_number: i32,
        raw_data: serde_json::Value,
        account_code: Option<&str>, account_name: Option<&str>,
        description: Option<&str>,
        entered_dr: &str, entered_cr: &str,
        currency_code: Option<&str>, exchange_rate: Option<&str>,
        gl_date: Option<chrono::NaiveDate>, reference: Option<&str>,
        line_type: Option<&str>, cost_center: Option<&str>,
        department: Option<&str>, project_code: Option<&str>,
        status: &str, error_message: Option<&str>, error_field: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<JournalImportRow> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.journal_import_rows
                (organization_id, batch_id, row_number, raw_data,
                 account_code, account_name, description,
                 entered_dr, entered_cr, currency_code, exchange_rate,
                 gl_date, reference, line_type,
                 cost_center, department, project_code,
                 status, error_message, error_field, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8::numeric, $9::numeric,
                    $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(batch_id).bind(row_number).bind(raw_data)
        .bind(account_code).bind(account_name).bind(description)
        .bind(entered_dr).bind(entered_cr).bind(currency_code).bind(exchange_rate)
        .bind(gl_date).bind(reference).bind(line_type)
        .bind(cost_center).bind(department).bind(project_code)
        .bind(status).bind(error_message).bind(error_field).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_import_row(&row))
    }

    async fn get_row(&self, id: Uuid) -> AtlasResult<Option<JournalImportRow>> {
        let row = sqlx::query("SELECT * FROM _atlas.journal_import_rows WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_import_row(&r)))
    }

    async fn list_batch_rows(&self, batch_id: Uuid) -> AtlasResult<Vec<JournalImportRow>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.journal_import_rows WHERE batch_id = $1 ORDER BY row_number"
        )
        .bind(batch_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_import_row).collect())
    }

    async fn update_row(
        &self, id: Uuid,
        account_code: Option<&str>, description: Option<&str>,
        entered_dr: Option<&str>, entered_cr: Option<&str>,
        status: &str, error_message: Option<&str>, error_field: Option<&str>,
    ) -> AtlasResult<JournalImportRow> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.journal_import_rows
            SET account_code = COALESCE($2, account_code),
                description = COALESCE($3, description),
                entered_dr = COALESCE($4::numeric, entered_dr),
                entered_cr = COALESCE($5::numeric, entered_cr),
                status = $6,
                error_message = COALESCE($7, error_message),
                error_field = COALESCE($8, error_field),
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(account_code).bind(description)
        .bind(entered_dr).bind(entered_cr)
        .bind(status).bind(error_message).bind(error_field)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_import_row(&row))
    }

    async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<JournalImportDashboardSummary> {
        let format_row = sqlx::query(
            r#"
            SELECT
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE status = 'active') as active
            FROM _atlas.journal_import_formats WHERE organization_id = $1
            "#,
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let batch_row = sqlx::query(
            r#"
            SELECT
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE status IN ('uploaded', 'validating')) as pending,
                COUNT(*) FILTER (WHERE status IN ('completed', 'completed_with_errors')) as completed,
                COUNT(*) FILTER (WHERE status = 'failed') as failed,
                COALESCE(SUM(imported_rows), 0) as total_imported,
                COALESCE(SUM(error_rows), 0) as total_errors
            FROM _atlas.journal_import_batches WHERE organization_id = $1
            "#,
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let recent_rows = sqlx::query(
            "SELECT * FROM _atlas.journal_import_batches WHERE organization_id = $1 ORDER BY created_at DESC LIMIT 10"
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(JournalImportDashboardSummary {
            total_formats: format_row.try_get::<i64, _>("total").unwrap_or(0) as i32,
            active_formats: format_row.try_get::<i64, _>("active").unwrap_or(0) as i32,
            total_batches: batch_row.try_get::<i64, _>("total").unwrap_or(0) as i32,
            pending_batches: batch_row.try_get::<i64, _>("pending").unwrap_or(0) as i32,
            completed_batches: batch_row.try_get::<i64, _>("completed").unwrap_or(0) as i32,
            failed_batches: batch_row.try_get::<i64, _>("failed").unwrap_or(0) as i32,
            total_rows_imported: batch_row.try_get::<i64, _>("total_imported").unwrap_or(0) as i32,
            total_rows_with_errors: batch_row.try_get::<i64, _>("total_errors").unwrap_or(0) as i32,
            recent_batches: recent_rows.iter().map(row_to_batch).collect(),
        })
    }
}
