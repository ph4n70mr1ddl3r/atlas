//! Financial Reporting Repository
//!
//! PostgreSQL storage for report templates, rows, columns, runs, results,
//! and user favourites.

use atlas_shared::{
    FinancialReportTemplate, FinancialReportRow, FinancialReportColumn,
    FinancialReportRun, FinancialReportResult, FinancialReportFavourite,
    FinancialReportingSummary,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

/// Repository trait for financial reporting data storage
#[async_trait]
pub trait FinancialReportingRepository: Send + Sync {
    // ========================================================================
    // Report Templates
    // ========================================================================

    async fn create_template(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        report_type: &str,
        currency_code: &str,
        row_display_order: &str,
        column_display_order: &str,
        rounding_option: &str,
        show_zero_amounts: bool,
        segment_filter: serde_json::Value,
        created_by: Option<Uuid>,
    ) -> AtlasResult<FinancialReportTemplate>;

    async fn get_template(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<FinancialReportTemplate>>;
    async fn get_template_by_id(&self, id: Uuid) -> AtlasResult<Option<FinancialReportTemplate>>;
    async fn list_templates(&self, org_id: Uuid, report_type: Option<&str>) -> AtlasResult<Vec<FinancialReportTemplate>>;
    async fn delete_template(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // ========================================================================
    // Report Rows
    // ========================================================================

    async fn create_row(
        &self,
        org_id: Uuid,
        template_id: Uuid,
        row_number: i32,
        line_type: &str,
        label: &str,
        indent_level: i32,
        account_range_from: Option<&str>,
        account_range_to: Option<&str>,
        account_filter: serde_json::Value,
        compute_action: Option<&str>,
        compute_source_rows: serde_json::Value,
        show_line: bool,
        bold: bool,
        underline: bool,
        double_underline: bool,
        page_break_before: bool,
        scaling_factor: Option<&str>,
        parent_row_id: Option<Uuid>,
    ) -> AtlasResult<FinancialReportRow>;

    async fn get_row(&self, id: Uuid) -> AtlasResult<Option<FinancialReportRow>>;
    async fn list_rows_by_template(&self, template_id: Uuid) -> AtlasResult<Vec<FinancialReportRow>>;
    async fn delete_row(&self, id: Uuid) -> AtlasResult<()>;

    // ========================================================================
    // Report Columns
    // ========================================================================

    async fn create_column(
        &self,
        org_id: Uuid,
        template_id: Uuid,
        column_number: i32,
        column_type: &str,
        header_label: &str,
        sub_header_label: Option<&str>,
        period_offset: i32,
        period_type: &str,
        compute_action: Option<&str>,
        compute_source_columns: serde_json::Value,
        show_column: bool,
        column_width: Option<i32>,
        format_override: Option<&str>,
    ) -> AtlasResult<FinancialReportColumn>;

    async fn get_column(&self, id: Uuid) -> AtlasResult<Option<FinancialReportColumn>>;
    async fn list_columns_by_template(&self, template_id: Uuid) -> AtlasResult<Vec<FinancialReportColumn>>;
    async fn delete_column(&self, id: Uuid) -> AtlasResult<()>;

    // ========================================================================
    // Report Runs
    // ========================================================================

    async fn create_run(
        &self,
        org_id: Uuid,
        template_id: Uuid,
        run_number: &str,
        name: Option<&str>,
        description: Option<&str>,
        as_of_date: Option<chrono::NaiveDate>,
        period_from: Option<chrono::NaiveDate>,
        period_to: Option<chrono::NaiveDate>,
        currency_code: &str,
        segment_filter: serde_json::Value,
        include_unposted: bool,
        created_by: Option<Uuid>,
    ) -> AtlasResult<FinancialReportRun>;

    async fn get_run(&self, id: Uuid) -> AtlasResult<Option<FinancialReportRun>>;
    async fn get_run_by_number(&self, org_id: Uuid, run_number: &str) -> AtlasResult<Option<FinancialReportRun>>;
    async fn list_runs(&self, org_id: Uuid, template_id: Option<Uuid>, status: Option<&str>) -> AtlasResult<Vec<FinancialReportRun>>;
    async fn update_run_status(
        &self,
        id: Uuid,
        status: &str,
        generated_by: Option<Uuid>,
        approved_by: Option<Uuid>,
        published_by: Option<Uuid>,
    ) -> AtlasResult<FinancialReportRun>;
    async fn update_run_totals(
        &self,
        id: Uuid,
        total_debit: &str,
        total_credit: &str,
        net_change: &str,
        beginning_balance: &str,
        ending_balance: &str,
        row_count: i32,
    ) -> AtlasResult<()>;

    // ========================================================================
    // Report Results
    // ========================================================================

    async fn create_result(
        &self,
        org_id: Uuid,
        run_id: Uuid,
        row_id: Uuid,
        column_id: Uuid,
        row_number: i32,
        column_number: i32,
        amount: &str,
        debit_amount: &str,
        credit_amount: &str,
        beginning_balance: &str,
        ending_balance: &str,
        is_computed: bool,
        compute_note: Option<&str>,
        display_amount: Option<&str>,
        display_format: Option<&str>,
    ) -> AtlasResult<FinancialReportResult>;

    async fn list_results_by_run(&self, run_id: Uuid) -> AtlasResult<Vec<FinancialReportResult>>;
    async fn delete_results_by_run(&self, run_id: Uuid) -> AtlasResult<()>;

    // ========================================================================
    // Favourites
    // ========================================================================

    async fn create_favourite(
        &self,
        org_id: Uuid,
        user_id: Uuid,
        template_id: Uuid,
        display_name: Option<&str>,
        position: i32,
    ) -> AtlasResult<FinancialReportFavourite>;

    async fn list_favourites(&self, org_id: Uuid, user_id: Uuid) -> AtlasResult<Vec<FinancialReportFavourite>>;
    async fn delete_favourite(&self, org_id: Uuid, user_id: Uuid, template_id: Uuid) -> AtlasResult<()>;

    // ========================================================================
    // Dashboard
    // ========================================================================

    async fn get_reporting_summary(&self, org_id: Uuid) -> AtlasResult<FinancialReportingSummary>;
}

/// PostgreSQL implementation
pub struct PostgresFinancialReportingRepository {
    pool: PgPool,
}

impl PostgresFinancialReportingRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn row_to_template(row: &sqlx::postgres::PgRow) -> FinancialReportTemplate {
    #[allow(dead_code)]
    fn get_num(row: &sqlx::postgres::PgRow, col: &str) -> String {
        let v: serde_json::Value = row.try_get(col).unwrap_or(serde_json::json!("0"));
        v.to_string()
    }
    FinancialReportTemplate {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        code: row.get("code"),
        name: row.get("name"),
        description: row.get("description"),
        report_type: row.get("report_type"),
        currency_code: row.get("currency_code"),
        row_display_order: row.get("row_display_order"),
        column_display_order: row.get("column_display_order"),
        rounding_option: row.get("rounding_option"),
        show_zero_amounts: row.get("show_zero_amounts"),
        segment_filter: row.try_get("segment_filter").unwrap_or(serde_json::json!({})),
        is_active: row.get("is_active"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_report_row(row: &sqlx::postgres::PgRow) -> FinancialReportRow {
    FinancialReportRow {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        template_id: row.get("template_id"),
        row_number: row.get("row_number"),
        line_type: row.get("line_type"),
        label: row.get("label"),
        indent_level: row.get("indent_level"),
        account_range_from: row.get("account_range_from"),
        account_range_to: row.get("account_range_to"),
        account_filter: row.try_get("account_filter").unwrap_or(serde_json::json!({})),
        compute_action: row.get("compute_action"),
        compute_source_rows: row.try_get("compute_source_rows").unwrap_or(serde_json::json!([])),
        show_line: row.get("show_line"),
        bold: row.get("bold"),
        underline: row.get("underline"),
        double_underline: row.get("double_underline"),
        page_break_before: row.get("page_break_before"),
        scaling_factor: row.get("scaling_factor"),
        parent_row_id: row.get("parent_row_id"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_column(row: &sqlx::postgres::PgRow) -> FinancialReportColumn {
    FinancialReportColumn {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        template_id: row.get("template_id"),
        column_number: row.get("column_number"),
        column_type: row.get("column_type"),
        header_label: row.get("header_label"),
        sub_header_label: row.get("sub_header_label"),
        period_offset: row.get("period_offset"),
        period_type: row.get("period_type"),
        compute_action: row.get("compute_action"),
        compute_source_columns: row.try_get("compute_source_columns").unwrap_or(serde_json::json!([])),
        show_column: row.get("show_column"),
        column_width: row.get("column_width"),
        format_override: row.get("format_override"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_run(row: &sqlx::postgres::PgRow) -> FinancialReportRun {
    FinancialReportRun {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        template_id: row.get("template_id"),
        run_number: row.get("run_number"),
        name: row.get("name"),
        description: row.get("description"),
        status: row.get("status"),
        as_of_date: row.get("as_of_date"),
        period_from: row.get("period_from"),
        period_to: row.get("period_to"),
        currency_code: row.get("currency_code"),
        segment_filter: row.try_get("segment_filter").unwrap_or(serde_json::json!({})),
        include_unposted: row.get("include_unposted"),
        total_debit: {
            let v: serde_json::Value = row.try_get("total_debit").unwrap_or(serde_json::json!("0"));
            v.to_string()
        },
        total_credit: {
            let v: serde_json::Value = row.try_get("total_credit").unwrap_or(serde_json::json!("0"));
            v.to_string()
        },
        net_change: {
            let v: serde_json::Value = row.try_get("net_change").unwrap_or(serde_json::json!("0"));
            v.to_string()
        },
        beginning_balance: {
            let v: serde_json::Value = row.try_get("beginning_balance").unwrap_or(serde_json::json!("0"));
            v.to_string()
        },
        ending_balance: {
            let v: serde_json::Value = row.try_get("ending_balance").unwrap_or(serde_json::json!("0"));
            v.to_string()
        },
        row_count: row.get("row_count"),
        generated_by: row.get("generated_by"),
        generated_at: row.get("generated_at"),
        approved_by: row.get("approved_by"),
        approved_at: row.get("approved_at"),
        published_by: row.get("published_by"),
        published_at: row.get("published_at"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_result(row: &sqlx::postgres::PgRow) -> FinancialReportResult {
    FinancialReportResult {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        run_id: row.get("run_id"),
        row_id: row.get("row_id"),
        column_id: row.get("column_id"),
        row_number: row.get("row_number"),
        column_number: row.get("column_number"),
        amount: {
            let v: serde_json::Value = row.try_get("amount").unwrap_or(serde_json::json!("0"));
            v.to_string()
        },
        debit_amount: {
            let v: serde_json::Value = row.try_get("debit_amount").unwrap_or(serde_json::json!("0"));
            v.to_string()
        },
        credit_amount: {
            let v: serde_json::Value = row.try_get("credit_amount").unwrap_or(serde_json::json!("0"));
            v.to_string()
        },
        beginning_balance: {
            let v: serde_json::Value = row.try_get("beginning_balance").unwrap_or(serde_json::json!("0"));
            v.to_string()
        },
        ending_balance: {
            let v: serde_json::Value = row.try_get("ending_balance").unwrap_or(serde_json::json!("0"));
            v.to_string()
        },
        is_computed: row.get("is_computed"),
        compute_note: row.get("compute_note"),
        display_amount: row.get("display_amount"),
        display_format: row.get("display_format"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.get("created_at"),
    }
}

#[async_trait]
impl FinancialReportingRepository for PostgresFinancialReportingRepository {
    // ========================================================================
    // Report Templates
    // ========================================================================

    async fn create_template(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        report_type: &str,
        currency_code: &str,
        row_display_order: &str,
        column_display_order: &str,
        rounding_option: &str,
        show_zero_amounts: bool,
        segment_filter: serde_json::Value,
        created_by: Option<Uuid>,
    ) -> AtlasResult<FinancialReportTemplate> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.financial_report_templates
                (organization_id, code, name, description, report_type,
                 currency_code, row_display_order, column_display_order,
                 rounding_option, show_zero_amounts, segment_filter, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            ON CONFLICT (organization_id, code) DO UPDATE
                SET name = $3, description = $4, report_type = $5,
                    currency_code = $6, row_display_order = $7,
                    column_display_order = $8, rounding_option = $9,
                    show_zero_amounts = $10, segment_filter = $11,
                    is_active = true, updated_at = now()
            RETURNING *
            "#,
        )
        .bind(org_id).bind(code).bind(name).bind(description).bind(report_type)
        .bind(currency_code).bind(row_display_order).bind(column_display_order)
        .bind(rounding_option).bind(show_zero_amounts).bind(segment_filter).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_template(&row))
    }

    async fn get_template(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<FinancialReportTemplate>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.financial_report_templates WHERE organization_id = $1 AND code = $2 AND is_active = true"
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_template(&r)))
    }

    async fn get_template_by_id(&self, id: Uuid) -> AtlasResult<Option<FinancialReportTemplate>> {
        let row = sqlx::query("SELECT * FROM _atlas.financial_report_templates WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_template(&r)))
    }

    async fn list_templates(&self, org_id: Uuid, report_type: Option<&str>) -> AtlasResult<Vec<FinancialReportTemplate>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM _atlas.financial_report_templates
            WHERE organization_id = $1
              AND is_active = true
              AND ($2::text IS NULL OR report_type = $2)
            ORDER BY name
            "#,
        )
        .bind(org_id).bind(report_type)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_template).collect())
    }

    async fn delete_template(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.financial_report_templates SET is_active = false, updated_at = now() WHERE organization_id = $1 AND code = $2"
        )
        .bind(org_id).bind(code)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Report Rows
    // ========================================================================

    async fn create_row(
        &self,
        org_id: Uuid,
        template_id: Uuid,
        row_number: i32,
        line_type: &str,
        label: &str,
        indent_level: i32,
        account_range_from: Option<&str>,
        account_range_to: Option<&str>,
        account_filter: serde_json::Value,
        compute_action: Option<&str>,
        compute_source_rows: serde_json::Value,
        show_line: bool,
        bold: bool,
        underline: bool,
        double_underline: bool,
        page_break_before: bool,
        scaling_factor: Option<&str>,
        parent_row_id: Option<Uuid>,
    ) -> AtlasResult<FinancialReportRow> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.financial_report_rows
                (organization_id, template_id, row_number, line_type, label, indent_level,
                 account_range_from, account_range_to, account_filter,
                 compute_action, compute_source_rows,
                 show_line, bold, underline, double_underline, page_break_before,
                 scaling_factor, parent_row_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(template_id).bind(row_number).bind(line_type).bind(label)
        .bind(indent_level).bind(account_range_from).bind(account_range_to)
        .bind(account_filter).bind(compute_action).bind(compute_source_rows)
        .bind(show_line).bind(bold).bind(underline).bind(double_underline)
        .bind(page_break_before).bind(scaling_factor).bind(parent_row_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_report_row(&row))
    }

    async fn get_row(&self, id: Uuid) -> AtlasResult<Option<FinancialReportRow>> {
        let row = sqlx::query("SELECT * FROM _atlas.financial_report_rows WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_report_row(&r)))
    }

    async fn list_rows_by_template(&self, template_id: Uuid) -> AtlasResult<Vec<FinancialReportRow>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.financial_report_rows WHERE template_id = $1 ORDER BY row_number"
        )
        .bind(template_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_report_row).collect())
    }

    async fn delete_row(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.financial_report_rows WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Report Columns
    // ========================================================================

    async fn create_column(
        &self,
        org_id: Uuid,
        template_id: Uuid,
        column_number: i32,
        column_type: &str,
        header_label: &str,
        sub_header_label: Option<&str>,
        period_offset: i32,
        period_type: &str,
        compute_action: Option<&str>,
        compute_source_columns: serde_json::Value,
        show_column: bool,
        column_width: Option<i32>,
        format_override: Option<&str>,
    ) -> AtlasResult<FinancialReportColumn> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.financial_report_columns
                (organization_id, template_id, column_number, column_type,
                 header_label, sub_header_label, period_offset, period_type,
                 compute_action, compute_source_columns,
                 show_column, column_width, format_override)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(template_id).bind(column_number).bind(column_type)
        .bind(header_label).bind(sub_header_label).bind(period_offset).bind(period_type)
        .bind(compute_action).bind(compute_source_columns)
        .bind(show_column).bind(column_width).bind(format_override)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_column(&row))
    }

    async fn get_column(&self, id: Uuid) -> AtlasResult<Option<FinancialReportColumn>> {
        let row = sqlx::query("SELECT * FROM _atlas.financial_report_columns WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_column(&r)))
    }

    async fn list_columns_by_template(&self, template_id: Uuid) -> AtlasResult<Vec<FinancialReportColumn>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.financial_report_columns WHERE template_id = $1 ORDER BY column_number"
        )
        .bind(template_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_column).collect())
    }

    async fn delete_column(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.financial_report_columns WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Report Runs
    // ========================================================================

    async fn create_run(
        &self,
        org_id: Uuid,
        template_id: Uuid,
        run_number: &str,
        name: Option<&str>,
        description: Option<&str>,
        as_of_date: Option<chrono::NaiveDate>,
        period_from: Option<chrono::NaiveDate>,
        period_to: Option<chrono::NaiveDate>,
        currency_code: &str,
        segment_filter: serde_json::Value,
        include_unposted: bool,
        created_by: Option<Uuid>,
    ) -> AtlasResult<FinancialReportRun> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.financial_report_runs
                (organization_id, template_id, run_number, name, description,
                 as_of_date, period_from, period_to, currency_code,
                 segment_filter, include_unposted, generated_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(template_id).bind(run_number).bind(name).bind(description)
        .bind(as_of_date).bind(period_from).bind(period_to).bind(currency_code)
        .bind(segment_filter).bind(include_unposted).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_run(&row))
    }

    async fn get_run(&self, id: Uuid) -> AtlasResult<Option<FinancialReportRun>> {
        let row = sqlx::query("SELECT * FROM _atlas.financial_report_runs WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_run(&r)))
    }

    async fn get_run_by_number(&self, org_id: Uuid, run_number: &str) -> AtlasResult<Option<FinancialReportRun>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.financial_report_runs WHERE organization_id = $1 AND run_number = $2"
        )
        .bind(org_id).bind(run_number)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_run(&r)))
    }

    async fn list_runs(&self, org_id: Uuid, template_id: Option<Uuid>, status: Option<&str>) -> AtlasResult<Vec<FinancialReportRun>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM _atlas.financial_report_runs
            WHERE organization_id = $1
              AND ($2::uuid IS NULL OR template_id = $2)
              AND ($3::text IS NULL OR status = $3)
            ORDER BY created_at DESC
            "#,
        )
        .bind(org_id).bind(template_id).bind(status)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_run).collect())
    }

    async fn update_run_status(
        &self,
        id: Uuid,
        status: &str,
        generated_by: Option<Uuid>,
        approved_by: Option<Uuid>,
        published_by: Option<Uuid>,
    ) -> AtlasResult<FinancialReportRun> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.financial_report_runs
            SET status = $2,
                generated_at = CASE WHEN $3 IS NOT NULL THEN now() ELSE generated_at END,
                generated_by = COALESCE($3, generated_by),
                approved_at = CASE WHEN $4 IS NOT NULL THEN now() ELSE approved_at END,
                approved_by = COALESCE($4, approved_by),
                published_at = CASE WHEN $5 IS NOT NULL THEN now() ELSE published_at END,
                published_by = COALESCE($5, published_by),
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(status).bind(generated_by).bind(approved_by).bind(published_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_run(&row))
    }

    async fn update_run_totals(
        &self,
        id: Uuid,
        total_debit: &str,
        total_credit: &str,
        net_change: &str,
        beginning_balance: &str,
        ending_balance: &str,
        row_count: i32,
    ) -> AtlasResult<()> {
        sqlx::query(
            r#"
            UPDATE _atlas.financial_report_runs
            SET total_debit = $2::numeric, total_credit = $3::numeric,
                net_change = $4::numeric, beginning_balance = $5::numeric,
                ending_balance = $6::numeric, row_count = $7, updated_at = now()
            WHERE id = $1
            "#,
        )
        .bind(id).bind(total_debit).bind(total_credit).bind(net_change)
        .bind(beginning_balance).bind(ending_balance).bind(row_count)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Report Results
    // ========================================================================

    async fn create_result(
        &self,
        org_id: Uuid,
        run_id: Uuid,
        row_id: Uuid,
        column_id: Uuid,
        row_number: i32,
        column_number: i32,
        amount: &str,
        debit_amount: &str,
        credit_amount: &str,
        beginning_balance: &str,
        ending_balance: &str,
        is_computed: bool,
        compute_note: Option<&str>,
        display_amount: Option<&str>,
        display_format: Option<&str>,
    ) -> AtlasResult<FinancialReportResult> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.financial_report_results
                (organization_id, run_id, row_id, column_id,
                 row_number, column_number,
                 amount, debit_amount, credit_amount,
                 beginning_balance, ending_balance,
                 is_computed, compute_note,
                 display_amount, display_format)
            VALUES ($1, $2, $3, $4, $5, $6, $7::numeric, $8::numeric, $9::numeric,
                    $10::numeric, $11::numeric, $12, $13, $14, $15)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(run_id).bind(row_id).bind(column_id)
        .bind(row_number).bind(column_number)
        .bind(amount).bind(debit_amount).bind(credit_amount)
        .bind(beginning_balance).bind(ending_balance)
        .bind(is_computed).bind(compute_note)
        .bind(display_amount).bind(display_format)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_result(&row))
    }

    async fn list_results_by_run(&self, run_id: Uuid) -> AtlasResult<Vec<FinancialReportResult>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.financial_report_results WHERE run_id = $1 ORDER BY row_number, column_number"
        )
        .bind(run_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_result).collect())
    }

    async fn delete_results_by_run(&self, run_id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.financial_report_results WHERE run_id = $1")
            .bind(run_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Favourites
    // ========================================================================

    async fn create_favourite(
        &self,
        org_id: Uuid,
        user_id: Uuid,
        template_id: Uuid,
        display_name: Option<&str>,
        position: i32,
    ) -> AtlasResult<FinancialReportFavourite> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.financial_report_favourites
                (organization_id, user_id, template_id, display_name, position)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (organization_id, user_id, template_id) DO UPDATE
                SET display_name = $4, position = $5
            RETURNING *
            "#,
        )
        .bind(org_id).bind(user_id).bind(template_id).bind(display_name).bind(position)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(FinancialReportFavourite {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            user_id: row.get("user_id"),
            template_id: row.get("template_id"),
            display_name: row.get("display_name"),
            position: row.get("position"),
            created_at: row.get("created_at"),
        })
    }

    async fn list_favourites(&self, org_id: Uuid, user_id: Uuid) -> AtlasResult<Vec<FinancialReportFavourite>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.financial_report_favourites WHERE organization_id = $1 AND user_id = $2 ORDER BY position"
        )
        .bind(org_id).bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| FinancialReportFavourite {
            id: r.get("id"),
            organization_id: r.get("organization_id"),
            user_id: r.get("user_id"),
            template_id: r.get("template_id"),
            display_name: r.get("display_name"),
            position: r.get("position"),
            created_at: r.get("created_at"),
        }).collect())
    }

    async fn delete_favourite(&self, org_id: Uuid, user_id: Uuid, template_id: Uuid) -> AtlasResult<()> {
        sqlx::query(
            "DELETE FROM _atlas.financial_report_favourites WHERE organization_id = $1 AND user_id = $2 AND template_id = $3"
        )
        .bind(org_id).bind(user_id).bind(template_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    async fn get_reporting_summary(&self, org_id: Uuid) -> AtlasResult<FinancialReportingSummary> {
        let template_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM _atlas.financial_report_templates WHERE organization_id = $1"
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let active_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM _atlas.financial_report_templates WHERE organization_id = $1 AND is_active = true"
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let run_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM _atlas.financial_report_runs WHERE organization_id = $1"
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let recent_rows = sqlx::query(
            "SELECT * FROM _atlas.financial_report_runs WHERE organization_id = $1 ORDER BY created_at DESC LIMIT 10"
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let total_reported: serde_json::Value = sqlx::query_scalar(
            r#"SELECT COALESCE(SUM(ABS(total_debit) + ABS(total_credit)), 0) FROM _atlas.financial_report_runs WHERE organization_id = $1"#
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(FinancialReportingSummary {
            template_count: template_count as i32,
            active_template_count: active_count as i32,
            run_count: run_count as i32,
            recent_runs: recent_rows.iter().map(row_to_run).collect(),
            templates_by_type: serde_json::json!({}),
            total_amount_reported: total_reported.to_string(),
        })
    }
}
