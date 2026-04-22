//! Financial Reporting Engine
//!
//! Manages report templates, rows, columns, report generation (trial balance,
//! income statement, balance sheet), report lifecycle, and financial dashboards.
//!
//! Oracle Fusion Cloud ERP equivalent: Financials > General Ledger > Financial Reporting Center

use atlas_shared::{
    FinancialReportTemplate, FinancialReportRow, FinancialReportColumn,
    FinancialReportRun, FinancialReportResult,
    FinancialReportFavourite, FinancialReportingSummary,
    AtlasError, AtlasResult,
};
use super::FinancialReportingRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Valid report types
const VALID_REPORT_TYPES: &[&str] = &[
    "trial_balance", "income_statement", "balance_sheet", "cash_flow", "custom",
];

/// Valid line types for report rows
const VALID_LINE_TYPES: &[&str] = &[
    "header", "data", "total", "subtotal", "separator", "text",
];

/// Valid column types
const VALID_COLUMN_TYPES: &[&str] = &[
    "actuals", "budget", "variance", "percent_variance",
    "prior_year", "ytd", "qtd", "custom",
];

/// Valid period types
const VALID_PERIOD_TYPES: &[&str] = &[
    "period", "qtd", "ytd", "inception_to_date",
];

/// Valid compute actions for rows
const VALID_ROW_COMPUTE_ACTIONS: &[&str] = &[
    "total", "subtotal", "variance", "percent", "constant",
];

/// Valid compute actions for columns
const VALID_COLUMN_COMPUTE_ACTIONS: &[&str] = &[
    "total", "variance", "percent_variance", "ratio",
];

/// Valid run statuses
const VALID_RUN_STATUSES: &[&str] = &[
    "draft", "generated", "approved", "published", "archived",
];

/// Valid rounding options
const VALID_ROUNDING_OPTIONS: &[&str] = &[
    "none", "thousands", "millions", "units",
];

/// Financial Reporting engine
pub struct FinancialReportingEngine {
    repository: Arc<dyn FinancialReportingRepository>,
}

impl FinancialReportingEngine {
    pub fn new(repository: Arc<dyn FinancialReportingRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Report Template Management
    // ========================================================================

    /// Create a new report template
    pub async fn create_template(
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
        if code.is_empty() || name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Template code and name are required".to_string(),
            ));
        }
        if !VALID_REPORT_TYPES.contains(&report_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid report type '{}'. Must be one of: {}",
                report_type, VALID_REPORT_TYPES.join(", ")
            )));
        }
        if !VALID_ROUNDING_OPTIONS.contains(&rounding_option) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid rounding option '{}'. Must be one of: {}",
                rounding_option, VALID_ROUNDING_OPTIONS.join(", ")
            )));
        }

        info!("Creating report template '{}' of type '{}' for org {}", code, report_type, org_id);

        self.repository.create_template(
            org_id, code, name, description, report_type,
            currency_code, row_display_order, column_display_order,
            rounding_option, show_zero_amounts, segment_filter, created_by,
        ).await
    }

    /// Get a template by code
    pub async fn get_template(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<FinancialReportTemplate>> {
        self.repository.get_template(org_id, code).await
    }

    /// Get a template by ID
    pub async fn get_template_by_id(&self, id: Uuid) -> AtlasResult<Option<FinancialReportTemplate>> {
        self.repository.get_template_by_id(id).await
    }

    /// List templates with optional type filter
    pub async fn list_templates(
        &self,
        org_id: Uuid,
        report_type: Option<&str>,
    ) -> AtlasResult<Vec<FinancialReportTemplate>> {
        if let Some(rt) = report_type {
            if !VALID_REPORT_TYPES.contains(&rt) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid report type filter '{}'. Must be one of: {}",
                    rt, VALID_REPORT_TYPES.join(", ")
                )));
            }
        }
        self.repository.list_templates(org_id, report_type).await
    }

    /// Delete (soft-delete) a template
    pub async fn delete_template(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        info!("Deleting report template '{}' for org {}", code, org_id);
        self.repository.delete_template(org_id, code).await
    }

    // ========================================================================
    // Report Row Management
    // ========================================================================

    /// Add a row to a report template
    pub async fn create_row(
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
        if label.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Row label is required".to_string(),
            ));
        }
        if !VALID_LINE_TYPES.contains(&line_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid line type '{}'. Must be one of: {}",
                line_type, VALID_LINE_TYPES.join(", ")
            )));
        }
        if let Some(ca) = compute_action {
            if !VALID_ROW_COMPUTE_ACTIONS.contains(&ca) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid compute action '{}'. Must be one of: {}",
                    ca, VALID_ROW_COMPUTE_ACTIONS.join(", ")
                )));
            }
        }

        // Validate template exists
        let template = self.repository.get_template_by_id(template_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Report template {} not found", template_id)
            ))?;

        if template.organization_id != org_id {
            return Err(AtlasError::Forbidden(
                "Template does not belong to this organization".to_string(),
            ));
        }

        // Data rows should have account ranges
        if line_type == "data" && account_range_from.is_none() && account_range_to.is_none() {
            return Err(AtlasError::ValidationFailed(
                "Data rows must have account range or account filter".to_string(),
            ));
        }

        info!("Adding row {} to template {}", row_number, template_id);

        self.repository.create_row(
            org_id, template_id, row_number, line_type, label, indent_level,
            account_range_from, account_range_to, account_filter,
            compute_action, compute_source_rows,
            show_line, bold, underline, double_underline,
            page_break_before, scaling_factor, parent_row_id,
        ).await
    }

    /// List rows for a template
    pub async fn list_rows(&self, template_id: Uuid) -> AtlasResult<Vec<FinancialReportRow>> {
        self.repository.list_rows_by_template(template_id).await
    }

    /// Delete a row
    pub async fn delete_row(&self, id: Uuid) -> AtlasResult<()> {
        self.repository.delete_row(id).await
    }

    // ========================================================================
    // Report Column Management
    // ========================================================================

    /// Add a column to a report template
    pub async fn create_column(
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
        if header_label.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Column header label is required".to_string(),
            ));
        }
        if !VALID_COLUMN_TYPES.contains(&column_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid column type '{}'. Must be one of: {}",
                column_type, VALID_COLUMN_TYPES.join(", ")
            )));
        }
        if !VALID_PERIOD_TYPES.contains(&period_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid period type '{}'. Must be one of: {}",
                period_type, VALID_PERIOD_TYPES.join(", ")
            )));
        }
        if let Some(ca) = compute_action {
            if !VALID_COLUMN_COMPUTE_ACTIONS.contains(&ca) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid column compute action '{}'. Must be one of: {}",
                    ca, VALID_COLUMN_COMPUTE_ACTIONS.join(", ")
                )));
            }
        }

        // Validate template exists
        let template = self.repository.get_template_by_id(template_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Report template {} not found", template_id)
            ))?;

        if template.organization_id != org_id {
            return Err(AtlasError::Forbidden(
                "Template does not belong to this organization".to_string(),
            ));
        }

        info!("Adding column {} to template {}", column_number, template_id);

        self.repository.create_column(
            org_id, template_id, column_number, column_type,
            header_label, sub_header_label, period_offset, period_type,
            compute_action, compute_source_columns,
            show_column, column_width, format_override,
        ).await
    }

    /// List columns for a template
    pub async fn list_columns(&self, template_id: Uuid) -> AtlasResult<Vec<FinancialReportColumn>> {
        self.repository.list_columns_by_template(template_id).await
    }

    /// Delete a column
    pub async fn delete_column(&self, id: Uuid) -> AtlasResult<()> {
        self.repository.delete_column(id).await
    }

    // ========================================================================
    // Report Generation & Execution
    // ========================================================================

    /// Generate a financial report (create a run)
    pub async fn generate_report(
        &self,
        org_id: Uuid,
        template_code: &str,
        name: Option<&str>,
        description: Option<&str>,
        as_of_date: Option<chrono::NaiveDate>,
        period_from: Option<chrono::NaiveDate>,
        period_to: Option<chrono::NaiveDate>,
        currency_code: Option<&str>,
        segment_filter: serde_json::Value,
        include_unposted: bool,
        generated_by: Option<Uuid>,
    ) -> AtlasResult<FinancialReportRun> {
        let template = self.repository.get_template(org_id, template_code).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Report template '{}' not found", template_code)
            ))?;

        if !template.is_active {
            return Err(AtlasError::ValidationFailed(
                "Cannot generate report from inactive template".to_string(),
            ));
        }

        // Validate date parameters based on report type
        match template.report_type.as_str() {
            "trial_balance" => {
                if as_of_date.is_none() {
                    return Err(AtlasError::ValidationFailed(
                        "Trial balance requires as_of_date".to_string(),
                    ));
                }
            }
            "income_statement" | "cash_flow" => {
                if period_from.is_none() || period_to.is_none() {
                    return Err(AtlasError::ValidationFailed(
                        "Income statement / cash flow requires period_from and period_to".to_string(),
                    ));
                }
            }
            "balance_sheet" => {
                if as_of_date.is_none() {
                    return Err(AtlasError::ValidationFailed(
                        "Balance sheet requires as_of_date".to_string(),
                    ));
                }
            }
            _ => {}
        }

        // Validate period_from <= period_to
        if let (Some(from), Some(to)) = (period_from, period_to) {
            if from > to {
                return Err(AtlasError::ValidationFailed(
                    "period_from must be on or before period_to".to_string(),
                ));
            }
        }

        let run_number = format!("FR-{}", Uuid::new_v4().to_string()[..8].to_uppercase());
        let effective_currency = currency_code.unwrap_or(&template.currency_code);

        info!("Generating report run {} from template '{}'", run_number, template_code);

        let run = self.repository.create_run(
            org_id, template.id, &run_number, name, description,
            as_of_date, period_from, period_to,
            effective_currency, segment_filter, include_unposted, generated_by,
        ).await?;

        // Load template rows and columns
        let rows = self.repository.list_rows_by_template(template.id).await?;
        let columns = self.repository.list_columns_by_template(template.id).await?;

        // Generate result cells
        let mut total_debit = 0.0_f64;
        let mut total_credit = 0.0_f64;
        let mut total_beginning = 0.0_f64;
        let mut total_ending = 0.0_f64;
        let mut result_count = 0i32;

        for row in &rows {
            if !row.show_line { continue; }

            for col in &columns {
                if !col.show_column { continue; }

                let (debit, credit, begin_bal, end_bal, amount) = self.compute_cell(
                    row, col, &template,
                );

                total_debit += debit;
                total_credit += credit;
                total_beginning += begin_bal;
                total_ending += end_bal;

                let amount_str = format!("{:.2}", amount);
                let display = format_amount(&amount_str, &template.rounding_option);

                self.repository.create_result(
                    org_id, run.id, row.id, col.id,
                    row.row_number, col.column_number,
                    &format!("{:.2}", amount),
                    &format!("{:.2}", debit),
                    &format!("{:.2}", credit),
                    &format!("{:.2}", begin_bal),
                    &format!("{:.2}", end_bal),
                    row.compute_action.is_some(),
                    row.compute_action.as_deref(),
                    Some(&display),
                    None,
                ).await?;

                result_count += 1;
            }
        }

        let net_change = total_debit - total_credit;

        // Update run totals
        self.repository.update_run_totals(
            run.id,
            &format!("{:.2}", total_debit),
            &format!("{:.2}", total_credit),
            &format!("{:.2}", net_change),
            &format!("{:.2}", total_beginning),
            &format!("{:.2}", total_ending),
            result_count,
        ).await?;

        // Mark as generated
        let updated_run = self.repository.update_run_status(
            run.id, "generated", generated_by, None, None,
        ).await?;

        Ok(updated_run)
    }

    /// Compute a single cell value based on row and column definition.
    /// In a full implementation, this would query the GL for actual balances.
    /// For now, it returns zero-based placeholder values to demonstrate the structure.
    fn compute_cell(
        &self,
        row: &FinancialReportRow,
        _col: &FinancialReportColumn,
        _template: &FinancialReportTemplate,
    ) -> (f64, f64, f64, f64, f64) {
        // In production, this would:
        // 1. Use row.account_range_from/to to query GL account balances
        // 2. Apply col.period_offset and period_type to determine the period
        // 3. Apply segment_filter to restrict results
        // 4. For computed rows, aggregate results from compute_source_rows
        //
        // For now, return zeroes — the engine structure and lifecycle are fully functional.

        if let Some(action) = &row.compute_action {
            match action.as_str() {
                "total" | "subtotal" => {
                    // Would sum source rows
                    (0.0, 0.0, 0.0, 0.0, 0.0)
                }
                "variance" => {
                    // Would compute difference between source rows
                    (0.0, 0.0, 0.0, 0.0, 0.0)
                }
                "percent" => {
                    // Would compute percentage
                    (0.0, 0.0, 0.0, 0.0, 0.0)
                }
                _ => (0.0, 0.0, 0.0, 0.0, 0.0),
            }
        } else {
            (0.0, 0.0, 0.0, 0.0, 0.0)
        }
    }

    /// Get a report run by ID
    pub async fn get_run(&self, id: Uuid) -> AtlasResult<Option<FinancialReportRun>> {
        self.repository.get_run(id).await
    }

    /// Get a report run by run number
    pub async fn get_run_by_number(&self, org_id: Uuid, run_number: &str) -> AtlasResult<Option<FinancialReportRun>> {
        self.repository.get_run_by_number(org_id, run_number).await
    }

    /// List report runs with optional filters
    pub async fn list_runs(
        &self,
        org_id: Uuid,
        template_id: Option<Uuid>,
        status: Option<&str>,
    ) -> AtlasResult<Vec<FinancialReportRun>> {
        if let Some(s) = status {
            if !VALID_RUN_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_RUN_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_runs(org_id, template_id, status).await
    }

    /// Get all results for a run
    pub async fn get_run_results(&self, run_id: Uuid) -> AtlasResult<Vec<FinancialReportResult>> {
        self.repository.list_results_by_run(run_id).await
    }

    // ========================================================================
    // Report Lifecycle
    // ========================================================================

    /// Approve a generated report
    pub async fn approve_report(
        &self,
        run_id: Uuid,
        approved_by: Uuid,
    ) -> AtlasResult<FinancialReportRun> {
        let run = self.repository.get_run(run_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Report run {} not found", run_id)
            ))?;

        if run.status != "generated" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot approve report in '{}' status. Must be 'generated'.", run.status)
            ));
        }

        info!("Approving report run {}", run.run_number);
        self.repository.update_run_status(run_id, "approved", None, Some(approved_by), None).await
    }

    /// Publish an approved report
    pub async fn publish_report(
        &self,
        run_id: Uuid,
        published_by: Uuid,
    ) -> AtlasResult<FinancialReportRun> {
        let run = self.repository.get_run(run_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Report run {} not found", run_id)
            ))?;

        if run.status != "approved" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot publish report in '{}' status. Must be 'approved'.", run.status)
            ));
        }

        info!("Publishing report run {}", run.run_number);
        self.repository.update_run_status(run_id, "published", None, None, Some(published_by)).await
    }

    /// Archive a published report
    pub async fn archive_report(&self, run_id: Uuid) -> AtlasResult<FinancialReportRun> {
        let run = self.repository.get_run(run_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Report run {} not found", run_id)
            ))?;

        if run.status != "published" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot archive report in '{}' status. Must be 'published'.", run.status)
            ));
        }

        info!("Archiving report run {}", run.run_number);
        self.repository.update_run_status(run_id, "archived", None, None, None).await
    }

    // ========================================================================
    // Quick Reports (pre-built templates)
    // ========================================================================

    /// Create a Trial Balance template with standard rows/columns
    pub async fn create_trial_balance_template(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        currency_code: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<FinancialReportTemplate> {
        let template = self.create_template(
            org_id, code, name,
            Some("Standard Trial Balance report"),
            "trial_balance", currency_code,
            "sequential", "sequential", "none",
            false, serde_json::json!({}), created_by,
        ).await?;

        // Add standard columns: Account, Debit, Credit, Net Balance
        self.repository.create_column(
            org_id, template.id, 1, "actuals",
            "Beginning Balance", None, 0, "period",
            None, serde_json::json!([]), true, None, None,
        ).await?;

        self.repository.create_column(
            org_id, template.id, 2, "actuals",
            "Debit", None, 0, "period",
            None, serde_json::json!([]), true, None, None,
        ).await?;

        self.repository.create_column(
            org_id, template.id, 3, "actuals",
            "Credit", None, 0, "period",
            None, serde_json::json!([]), true, None, None,
        ).await?;

        self.repository.create_column(
            org_id, template.id, 4, "actuals",
            "Ending Balance", None, 0, "period",
            None, serde_json::json!([]), true, None, None,
        ).await?;

        // Add header row and total row
        self.repository.create_row(
            org_id, template.id, 1, "header", "Trial Balance",
            0, None, None, serde_json::json!({}),
            None, serde_json::json!([]),
            true, true, false, true, false, None, None,
        ).await?;

        self.repository.create_row(
            org_id, template.id, 999, "total", "Total",
            0, None, None, serde_json::json!({}),
            Some("total"), serde_json::json!([]),
            true, true, false, true, false, None, None,
        ).await?;

        Ok(template)
    }

    /// Create an Income Statement template
    pub async fn create_income_statement_template(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        currency_code: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<FinancialReportTemplate> {
        let template = self.create_template(
            org_id, code, name,
            Some("Standard Income Statement report"),
            "income_statement", currency_code,
            "sequential", "sequential", "none",
            false, serde_json::json!({}), created_by,
        ).await?;

        // Columns: Current Period, YTD
        self.repository.create_column(
            org_id, template.id, 1, "actuals",
            "Current Period", None, 0, "period",
            None, serde_json::json!([]), true, None, None,
        ).await?;

        self.repository.create_column(
            org_id, template.id, 2, "ytd",
            "Year-to-Date", None, 0, "ytd",
            None, serde_json::json!([]), true, None, None,
        ).await?;

        // Standard rows for Income Statement
        let sections = [
            (1, "header", "Revenue"),
            (2, "data", "Net Revenue"),
            (10, "header", "Cost of Goods Sold"),
            (11, "data", "Total COGS"),
            (15, "subtotal", "Gross Profit"),
            (20, "header", "Operating Expenses"),
            (21, "data", "Selling Expenses"),
            (22, "data", "Administrative Expenses"),
            (25, "subtotal", "Total Operating Expenses"),
            (30, "subtotal", "Operating Income"),
            (40, "header", "Other Income/Expense"),
            (41, "data", "Other Income"),
            (42, "data", "Other Expense"),
            (45, "subtotal", "Income Before Tax"),
            (50, "data", "Income Tax Expense"),
            (55, "total", "Net Income"),
        ];

        for (row_num, line_type, label) in &sections {
            let (account_from, account_to, compute) = match *line_type {
                "data" => (Some(*label), Some(*label), None),
                "subtotal" | "total" => (None, None, Some(*line_type)),
                _ => (None, None, None),
            };

            self.repository.create_row(
                org_id, template.id, *row_num, line_type, label,
                if *line_type == "header" { 0 } else if *line_type == "data" { 1 } else { 0 },
                account_from, account_to, serde_json::json!({}),
                compute, serde_json::json!([]),
                true,
                *line_type == "header" || *line_type == "total",
                *line_type == "total",
                *line_type == "total",
                *line_type == "header",
                None, None,
            ).await?;
        }

        Ok(template)
    }

    /// Create a Balance Sheet template
    pub async fn create_balance_sheet_template(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        currency_code: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<FinancialReportTemplate> {
        let template = self.create_template(
            org_id, code, name,
            Some("Standard Balance Sheet report"),
            "balance_sheet", currency_code,
            "sequential", "sequential", "none",
            false, serde_json::json!({}), created_by,
        ).await?;

        // Single column: As-of date balance
        self.repository.create_column(
            org_id, template.id, 1, "actuals",
            "Balance", None, 0, "period",
            None, serde_json::json!([]), true, None, None,
        ).await?;

        // Standard Balance Sheet rows
        let sections = [
            (1, "header", "ASSETS"),
            (2, "header", "Current Assets"),
            (3, "data", "Cash and Cash Equivalents"),
            (4, "data", "Accounts Receivable"),
            (5, "data", "Inventory"),
            (6, "data", "Prepaid Expenses"),
            (7, "subtotal", "Total Current Assets"),
            (10, "header", "Non-Current Assets"),
            (11, "data", "Property, Plant & Equipment"),
            (12, "data", "Intangible Assets"),
            (13, "data", "Long-term Investments"),
            (14, "subtotal", "Total Non-Current Assets"),
            (15, "total", "TOTAL ASSETS"),
            (20, "header", "LIABILITIES"),
            (21, "header", "Current Liabilities"),
            (22, "data", "Accounts Payable"),
            (23, "data", "Short-term Debt"),
            (24, "data", "Accrued Liabilities"),
            (25, "subtotal", "Total Current Liabilities"),
            (28, "header", "Non-Current Liabilities"),
            (29, "data", "Long-term Debt"),
            (30, "data", "Deferred Tax Liabilities"),
            (31, "subtotal", "Total Non-Current Liabilities"),
            (32, "total", "TOTAL LIABILITIES"),
            (35, "header", "STOCKHOLDERS' EQUITY"),
            (36, "data", "Common Stock"),
            (37, "data", "Retained Earnings"),
            (38, "data", "Accumulated Other Comprehensive Income"),
            (39, "subtotal", "Total Stockholders' Equity"),
            (40, "total", "TOTAL LIABILITIES AND EQUITY"),
        ];

        for (row_num, line_type, label) in &sections {
            let indent = match *line_type {
                "header" => if label.starts_with("ASSETS") || label.starts_with("LIABILITIES") || label.starts_with("STOCKHOLDERS") { 0 } else { 1 },
                "data" => 2,
                _ => 0,
            };
            let (account_from, account_to, compute) = match *line_type {
                "data" => (Some(*label), Some(*label), None),
                "subtotal" | "total" => (None, None, Some(*line_type)),
                _ => (None, None, None),
            };

            self.repository.create_row(
                org_id, template.id, *row_num, line_type, label,
                indent,
                account_from, account_to, serde_json::json!({}),
                compute, serde_json::json!([]),
                true,
                *line_type == "header" || *line_type == "total",
                *line_type == "total",
                *line_type == "total",
                *line_type == "header",
                None, None,
            ).await?;
        }

        Ok(template)
    }

    // ========================================================================
    // Favourites
    // ========================================================================

    /// Add a report template to user favourites
    pub async fn add_favourite(
        &self,
        org_id: Uuid,
        user_id: Uuid,
        template_id: Uuid,
        display_name: Option<&str>,
    ) -> AtlasResult<FinancialReportFavourite> {
        // Validate template exists
        let template = self.repository.get_template_by_id(template_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Template {} not found", template_id)
            ))?;

        if template.organization_id != org_id {
            return Err(AtlasError::Forbidden(
                "Template does not belong to this organization".to_string(),
            ));
        }

        info!("Adding template {} to favourites for user {}", template_id, user_id);

        // Get next position
        let existing = self.repository.list_favourites(org_id, user_id).await?;
        let position = existing.len() as i32;

        self.repository.create_favourite(
            org_id, user_id, template_id, display_name.or(Some(&template.name)), position,
        ).await
    }

    /// List user's favourite reports
    pub async fn list_favourites(
        &self,
        org_id: Uuid,
        user_id: Uuid,
    ) -> AtlasResult<Vec<FinancialReportFavourite>> {
        self.repository.list_favourites(org_id, user_id).await
    }

    /// Remove a report from favourites
    pub async fn remove_favourite(
        &self,
        org_id: Uuid,
        user_id: Uuid,
        template_id: Uuid,
    ) -> AtlasResult<()> {
        self.repository.delete_favourite(org_id, user_id, template_id).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get financial reporting dashboard summary
    pub async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<FinancialReportingSummary> {
        self.repository.get_reporting_summary(org_id).await
    }
}

/// Format an amount string according to rounding option
fn format_amount(amount: &str, rounding_option: &str) -> String {
    let value: f64 = amount.parse().unwrap_or(0.0);
    match rounding_option {
        "thousands" => format!("{:.0}K", value / 1000.0),
        "millions" => format!("{:.1}M", value / 1_000_000.0),
        "units" => format!("{:.0}", value),
        _ => format!("{:.2}", value),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_report_types() {
        assert!(VALID_REPORT_TYPES.contains(&"trial_balance"));
        assert!(VALID_REPORT_TYPES.contains(&"income_statement"));
        assert!(VALID_REPORT_TYPES.contains(&"balance_sheet"));
        assert!(VALID_REPORT_TYPES.contains(&"cash_flow"));
        assert!(VALID_REPORT_TYPES.contains(&"custom"));
    }

    #[test]
    fn test_valid_line_types() {
        assert!(VALID_LINE_TYPES.contains(&"header"));
        assert!(VALID_LINE_TYPES.contains(&"data"));
        assert!(VALID_LINE_TYPES.contains(&"total"));
        assert!(VALID_LINE_TYPES.contains(&"subtotal"));
        assert!(VALID_LINE_TYPES.contains(&"separator"));
        assert!(VALID_LINE_TYPES.contains(&"text"));
    }

    #[test]
    fn test_valid_column_types() {
        assert!(VALID_COLUMN_TYPES.contains(&"actuals"));
        assert!(VALID_COLUMN_TYPES.contains(&"budget"));
        assert!(VALID_COLUMN_TYPES.contains(&"variance"));
        assert!(VALID_COLUMN_TYPES.contains(&"percent_variance"));
        assert!(VALID_COLUMN_TYPES.contains(&"prior_year"));
        assert!(VALID_COLUMN_TYPES.contains(&"ytd"));
        assert!(VALID_COLUMN_TYPES.contains(&"qtd"));
        assert!(VALID_COLUMN_TYPES.contains(&"custom"));
    }

    #[test]
    fn test_valid_period_types() {
        assert!(VALID_PERIOD_TYPES.contains(&"period"));
        assert!(VALID_PERIOD_TYPES.contains(&"qtd"));
        assert!(VALID_PERIOD_TYPES.contains(&"ytd"));
        assert!(VALID_PERIOD_TYPES.contains(&"inception_to_date"));
    }

    #[test]
    fn test_valid_run_statuses() {
        assert!(VALID_RUN_STATUSES.contains(&"draft"));
        assert!(VALID_RUN_STATUSES.contains(&"generated"));
        assert!(VALID_RUN_STATUSES.contains(&"approved"));
        assert!(VALID_RUN_STATUSES.contains(&"published"));
        assert!(VALID_RUN_STATUSES.contains(&"archived"));
    }

    #[test]
    fn test_valid_rounding_options() {
        assert!(VALID_ROUNDING_OPTIONS.contains(&"none"));
        assert!(VALID_ROUNDING_OPTIONS.contains(&"thousands"));
        assert!(VALID_ROUNDING_OPTIONS.contains(&"millions"));
        assert!(VALID_ROUNDING_OPTIONS.contains(&"units"));
    }

    #[test]
    fn test_format_amount_none() {
        assert_eq!(format_amount("1234.56", "none"), "1234.56");
    }

    #[test]
    fn test_format_amount_thousands() {
        assert_eq!(format_amount("1234567.00", "thousands"), "1235K");
    }

    #[test]
    fn test_format_amount_millions() {
        assert_eq!(format_amount("1234567.00", "millions"), "1.2M");
    }

    #[test]
    fn test_format_amount_units() {
        assert_eq!(format_amount("1234.56", "units"), "1235");
    }
}
