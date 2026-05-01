//! Financial Statement Engine
//!
//! Core financial statement operations:
//! - Report definition management (create standard financial reports)
//! - Balance Sheet generation from GL account balances
//! - Income Statement generation from revenue/expense accounts
//! - Cash Flow Statement (indirect method) derivation
//! - Statement of Changes in Equity
//! - Financial ratio calculations
//!
//! Oracle Fusion Cloud ERP equivalent: General Ledger > Financial Reporting Center

use atlas_shared::{
    FinancialStatementDefinition, FinancialStatement, FinancialStatementLine,
    FinancialStatementRequest, BalanceSheetSummary, IncomeStatementSummary,
    AtlasError, AtlasResult,
};
use super::FinancialStatementRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Valid report types
const VALID_REPORT_TYPES: &[&str] = &[
    "balance_sheet", "income_statement", "cash_flow_statement",
    "trial_balance", "statement_of_changes_in_equity",
];

/// Valid balance sheet classifications
const VALID_BS_CLASSIFICATIONS: &[&str] = &[
    "current_asset", "non_current_asset",
    "current_liability", "non_current_liability",
    "equity",
];

/// Valid income statement classifications
const VALID_IS_CLASSIFICATIONS: &[&str] = &[
    "revenue", "cost_of_goods_sold", "gross_profit",
    "operating_expense", "operating_income",
    "other_income", "other_expense",
    "income_before_tax", "income_tax_expense", "net_income",
];

/// Valid line types
const VALID_LINE_TYPES: &[&str] = &[
    "header", "detail", "subtotal", "total", "blank",
];

/// Valid sign conventions
const VALID_SIGN_CONVENTIONS: &[&str] = &[
    "normal", "negate",
];

/// Debit-nature account types (assets, expenses)
const DEBIT_NATURE_TYPES: &[&str] = &["asset", "expense"];

/// Credit-nature account types (liabilities, equity, revenue)
const CREDIT_NATURE_TYPES: &[&str] = &["liability", "equity", "revenue"];

/// Financial Statement Engine
pub struct FinancialStatementEngine {
    repository: Arc<dyn FinancialStatementRepository>,
}

impl FinancialStatementEngine {
    pub fn new(repository: Arc<dyn FinancialStatementRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Report Definition Management
    // ========================================================================

    /// Create a financial statement report definition
    pub async fn create_report_definition(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        report_type: &str,
        currency_code: &str,
        include_comparative: bool,
        comparative_period_count: i32,
        row_definitions: serde_json::Value,
        column_definitions: serde_json::Value,
        period_name: Option<&str>,
        fiscal_year: Option<i32>,
        is_system: bool,
        created_by: Option<Uuid>,
    ) -> AtlasResult<FinancialStatementDefinition> {
        if code.is_empty() || name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Report code and name are required".to_string(),
            ));
        }
        if !VALID_REPORT_TYPES.contains(&report_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid report_type '{}'. Must be one of: {}",
                report_type, VALID_REPORT_TYPES.join(", ")
            )));
        }
        if currency_code.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Currency code is required".to_string(),
            ));
        }
        if include_comparative && comparative_period_count <= 0 {
            return Err(AtlasError::ValidationFailed(
                "Comparative period count must be positive when comparative is enabled".to_string(),
            ));
        }

        // Check uniqueness
        if let Some(_) = self.repository.get_definition_by_code(org_id, code).await? {
            return Err(AtlasError::Conflict(
                format!("Report definition code '{}' already exists", code)
            ));
        }

        info!("Creating financial report definition '{}' of type {}", code, report_type);

        self.repository.create_definition(
            org_id, code, name, description,
            report_type, currency_code, include_comparative, comparative_period_count,
            row_definitions, column_definitions, period_name, fiscal_year,
            is_system, created_by,
        ).await
    }

    /// Get report definition by ID
    pub async fn get_report_definition(&self, id: Uuid) -> AtlasResult<Option<FinancialStatementDefinition>> {
        self.repository.get_definition(id).await
    }

    /// List report definitions
    pub async fn list_report_definitions(&self, org_id: Uuid, report_type: Option<&str>) -> AtlasResult<Vec<FinancialStatementDefinition>> {
        if let Some(rt) = report_type {
            if !VALID_REPORT_TYPES.contains(&rt) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid report_type '{}'. Must be one of: {}", rt, VALID_REPORT_TYPES.join(", ")
                )));
            }
        }
        self.repository.list_definitions(org_id, report_type).await
    }

    // ========================================================================
    // Financial Statement Generation
    // ========================================================================

    /// Generate a financial statement from GL data
    pub async fn generate_statement(
        &self,
        org_id: Uuid,
        request: FinancialStatementRequest,
        generated_by: Option<Uuid>,
    ) -> AtlasResult<FinancialStatement> {
        let report_type = request.report_type.as_deref().unwrap_or("balance_sheet");

        if !VALID_REPORT_TYPES.contains(&report_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid report_type '{}'. Must be one of: {}",
                report_type, VALID_REPORT_TYPES.join(", ")
            )));
        }

        info!("Generating {} report for org {} as of {}",
            report_type, org_id, request.as_of_date);

        let currency_code = request.currency_code.as_deref().unwrap_or("USD");

        match report_type {
            "balance_sheet" => self.generate_balance_sheet(org_id, &request, currency_code, generated_by).await,
            "income_statement" => self.generate_income_statement(org_id, &request, currency_code, generated_by).await,
            "trial_balance" => self.generate_trial_balance_report(org_id, &request, currency_code, generated_by).await,
            "cash_flow_statement" => self.generate_cash_flow_statement(org_id, &request, currency_code, generated_by).await,
            "statement_of_changes_in_equity" => self.generate_equity_statement(org_id, &request, currency_code, generated_by).await,
            _ => Err(AtlasError::ValidationFailed(format!("Unsupported report type: {}", report_type))),
        }
    }

    /// Generate a Balance Sheet
    async fn generate_balance_sheet(
        &self,
        org_id: Uuid,
        request: &FinancialStatementRequest,
        currency_code: &str,
        generated_by: Option<Uuid>,
    ) -> AtlasResult<FinancialStatement> {
        let accounts = self.repository.get_account_balances(
            org_id, request.as_of_date, request.period_name.as_deref(),
        ).await?;

        let mut lines = Vec::new();
        let mut line_num = 0i32;
        let mut total_current_assets = 0.0_f64;
        let mut total_non_current_assets = 0.0_f64;
        let mut total_current_liabilities = 0.0_f64;
        let mut total_non_current_liabilities = 0.0_f64;
        let mut total_equity = 0.0_f64;

        // Header: ASSETS
        line_num += 1;
        lines.push(FinancialStatementLine {
            line_number: line_num, indent_level: 0,
            line_type: "header".to_string(), account_code_range: None,
            label: "ASSETS".to_string(), classification: None,
            amount: "".to_string(), comparative_amount: None,
            variance_amount: None, variance_percent: None,
            ytd_amount: None, budget_amount: None, budget_variance: None,
            is_debit_nature: None, sign_convention: "normal".to_string(),
            metadata: serde_json::json!({}),
        });

        // Current Assets
        line_num += 1;
        lines.push(FinancialStatementLine {
            line_number: line_num, indent_level: 1,
            line_type: "header".to_string(), account_code_range: None,
            label: "Current Assets".to_string(), classification: Some("current_asset".to_string()),
            amount: "".to_string(), comparative_amount: None,
            variance_amount: None, variance_percent: None,
            ytd_amount: None, budget_amount: None, budget_variance: None,
            is_debit_nature: None, sign_convention: "normal".to_string(),
            metadata: serde_json::json!({}),
        });

        for acct in &accounts {
            if acct.account_type == "asset" && matches!(acct.subtype.as_deref(), Some("current_asset") | None) {
                let balance = acct.balance;
                total_current_assets += balance;
                line_num += 1;
                lines.push(FinancialStatementLine {
                    line_number: line_num, indent_level: 2,
                    line_type: "detail".to_string(),
                    account_code_range: Some(acct.account_code.clone()),
                    label: acct.account_name.clone(),
                    classification: Some("current_asset".to_string()),
                    amount: format!("{:.2}", balance),
                    comparative_amount: None, variance_amount: None, variance_percent: None,
                    ytd_amount: None, budget_amount: None, budget_variance: None,
                    is_debit_nature: Some(true), sign_convention: "normal".to_string(),
                    metadata: serde_json::json!({}),
                });
            }
        }

        // Subtotal: Total Current Assets
        line_num += 1;
        lines.push(FinancialStatementLine {
            line_number: line_num, indent_level: 1,
            line_type: "subtotal".to_string(), account_code_range: None,
            label: "Total Current Assets".to_string(), classification: Some("current_asset".to_string()),
            amount: format!("{:.2}", total_current_assets),
            comparative_amount: None, variance_amount: None, variance_percent: None,
            ytd_amount: None, budget_amount: None, budget_variance: None,
            is_debit_nature: Some(true), sign_convention: "normal".to_string(),
            metadata: serde_json::json!({}),
        });

        // Non-Current Assets
        line_num += 1;
        lines.push(FinancialStatementLine {
            line_number: line_num, indent_level: 1,
            line_type: "header".to_string(), account_code_range: None,
            label: "Non-Current Assets".to_string(), classification: Some("non_current_asset".to_string()),
            amount: "".to_string(), comparative_amount: None,
            variance_amount: None, variance_percent: None,
            ytd_amount: None, budget_amount: None, budget_variance: None,
            is_debit_nature: None, sign_convention: "normal".to_string(),
            metadata: serde_json::json!({}),
        });

        for acct in &accounts {
            if acct.account_type == "asset" && acct.subtype.as_deref() == Some("fixed_asset") {
                let balance = acct.balance;
                total_non_current_assets += balance;
                line_num += 1;
                lines.push(FinancialStatementLine {
                    line_number: line_num, indent_level: 2,
                    line_type: "detail".to_string(),
                    account_code_range: Some(acct.account_code.clone()),
                    label: acct.account_name.clone(),
                    classification: Some("non_current_asset".to_string()),
                    amount: format!("{:.2}", balance),
                    comparative_amount: None, variance_amount: None, variance_percent: None,
                    ytd_amount: None, budget_amount: None, budget_variance: None,
                    is_debit_nature: Some(true), sign_convention: "normal".to_string(),
                    metadata: serde_json::json!({}),
                });
            }
        }

        line_num += 1;
        lines.push(FinancialStatementLine {
            line_number: line_num, indent_level: 1,
            line_type: "subtotal".to_string(), account_code_range: None,
            label: "Total Non-Current Assets".to_string(), classification: Some("non_current_asset".to_string()),
            amount: format!("{:.2}", total_non_current_assets),
            comparative_amount: None, variance_amount: None, variance_percent: None,
            ytd_amount: None, budget_amount: None, budget_variance: None,
            is_debit_nature: Some(true), sign_convention: "normal".to_string(),
            metadata: serde_json::json!({}),
        });

        let total_assets = total_current_assets + total_non_current_assets;

        line_num += 1;
        lines.push(FinancialStatementLine {
            line_number: line_num, indent_level: 0,
            line_type: "total".to_string(), account_code_range: None,
            label: "TOTAL ASSETS".to_string(), classification: None,
            amount: format!("{:.2}", total_assets),
            comparative_amount: None, variance_amount: None, variance_percent: None,
            ytd_amount: None, budget_amount: None, budget_variance: None,
            is_debit_nature: Some(true), sign_convention: "normal".to_string(),
            metadata: serde_json::json!({}),
        });

        // LIABILITIES & EQUITY section
        for acct in &accounts {
            if acct.account_type == "liability" {
                let balance = acct.balance;
                if matches!(acct.subtype.as_deref(), Some("current_liability") | None) {
                    total_current_liabilities += balance;
                } else {
                    total_non_current_liabilities += balance;
                }
            }
            if acct.account_type == "equity" {
                total_equity += acct.balance;
            }
        }

        let total_liabilities = total_current_liabilities + total_non_current_liabilities;
        let total_liabilities_and_equity = total_liabilities + total_equity;
        let is_balanced = (total_assets - total_liabilities_and_equity).abs() < 0.01;

        // Simplified: just add the total lines for L&E
        line_num += 1;
        lines.push(FinancialStatementLine {
            line_number: line_num, indent_level: 0,
            line_type: "header".to_string(), account_code_range: None,
            label: "LIABILITIES AND EQUITY".to_string(), classification: None,
            amount: "".to_string(), comparative_amount: None,
            variance_amount: None, variance_percent: None,
            ytd_amount: None, budget_amount: None, budget_variance: None,
            is_debit_nature: None, sign_convention: "normal".to_string(),
            metadata: serde_json::json!({}),
        });

        line_num += 1;
        lines.push(FinancialStatementLine {
            line_number: line_num, indent_level: 1,
            line_type: "total".to_string(), account_code_range: None,
            label: "Total Current Liabilities".to_string(), classification: Some("current_liability".to_string()),
            amount: format!("{:.2}", total_current_liabilities),
            comparative_amount: None, variance_amount: None, variance_percent: None,
            ytd_amount: None, budget_amount: None, budget_variance: None,
            is_debit_nature: Some(false), sign_convention: "normal".to_string(),
            metadata: serde_json::json!({}),
        });

        line_num += 1;
        lines.push(FinancialStatementLine {
            line_number: line_num, indent_level: 1,
            line_type: "total".to_string(), account_code_range: None,
            label: "Total Non-Current Liabilities".to_string(), classification: Some("non_current_liability".to_string()),
            amount: format!("{:.2}", total_non_current_liabilities),
            comparative_amount: None, variance_amount: None, variance_percent: None,
            ytd_amount: None, budget_amount: None, budget_variance: None,
            is_debit_nature: Some(false), sign_convention: "normal".to_string(),
            metadata: serde_json::json!({}),
        });

        line_num += 1;
        lines.push(FinancialStatementLine {
            line_number: line_num, indent_level: 1,
            line_type: "total".to_string(), account_code_range: None,
            label: "Total Equity".to_string(), classification: Some("equity".to_string()),
            amount: format!("{:.2}", total_equity),
            comparative_amount: None, variance_amount: None, variance_percent: None,
            ytd_amount: None, budget_amount: None, budget_variance: None,
            is_debit_nature: Some(false), sign_convention: "normal".to_string(),
            metadata: serde_json::json!({}),
        });

        line_num += 1;
        lines.push(FinancialStatementLine {
            line_number: line_num, indent_level: 0,
            line_type: "total".to_string(), account_code_range: None,
            label: "TOTAL LIABILITIES AND EQUITY".to_string(), classification: None,
            amount: format!("{:.2}", total_liabilities_and_equity),
            comparative_amount: None, variance_amount: None, variance_percent: None,
            ytd_amount: None, budget_amount: None, budget_variance: None,
            is_debit_nature: Some(false), sign_convention: "normal".to_string(),
            metadata: serde_json::json!({}),
        });

        let bs_summary = BalanceSheetSummary {
            total_current_assets: format!("{:.2}", total_current_assets),
            total_non_current_assets: format!("{:.2}", total_non_current_assets),
            total_assets: format!("{:.2}", total_assets),
            total_current_liabilities: format!("{:.2}", total_current_liabilities),
            total_non_current_liabilities: format!("{:.2}", total_non_current_liabilities),
            total_liabilities: format!("{:.2}", total_liabilities),
            total_equity: format!("{:.2}", total_equity),
            total_liabilities_and_equity: format!("{:.2}", total_liabilities_and_equity),
            is_balanced,
        };

        let stmt = FinancialStatement {
            id: Uuid::new_v4(),
            organization_id: org_id,
            definition_id: request.definition_id.unwrap_or(Uuid::nil()),
            report_name: "Balance Sheet".to_string(),
            report_type: "balance_sheet".to_string(),
            as_of_date: request.as_of_date,
            period_name: request.period_name.clone(),
            fiscal_year: request.fiscal_year,
            currency_code: currency_code.to_string(),
            lines,
            totals: serde_json::to_value(&bs_summary).unwrap_or(serde_json::json!({})),
            is_balanced,
            generated_at: chrono::Utc::now(),
            generated_by,
            metadata: serde_json::json!({}),
            created_at: chrono::Utc::now(),
        };

        // Persist the generated statement
        self.repository.save_statement(&stmt).await?;

        Ok(stmt)
    }

    /// Generate an Income Statement
    async fn generate_income_statement(
        &self,
        org_id: Uuid,
        request: &FinancialStatementRequest,
        currency_code: &str,
        generated_by: Option<Uuid>,
    ) -> AtlasResult<FinancialStatement> {
        let accounts = self.repository.get_account_balances(
            org_id, request.as_of_date, request.period_name.as_deref(),
        ).await?;

        let mut lines = Vec::new();
        let mut line_num = 0i32;
        let mut total_revenue = 0.0_f64;
        let mut total_cogs = 0.0_f64;
        let mut total_operating_expenses = 0.0_f64;
        let mut total_other_income = 0.0_f64;
        let mut total_other_expense = 0.0_f64;
        let mut income_tax = 0.0_f64;

        // Revenue
        line_num += 1;
        lines.push(Self::make_line(line_num, 0, "header", "REVENUE", "", None, None));

        for acct in &accounts {
            if acct.account_type == "revenue" {
                let balance = acct.balance;
                total_revenue += balance;
                line_num += 1;
                lines.push(Self::make_detail_line(line_num, 1, &acct.account_code, &acct.account_name, balance, "revenue"));
            }
        }

        line_num += 1;
        lines.push(Self::make_line(line_num, 0, "total", "Total Revenue", &format!("{:.2}", total_revenue), None, None));

        // Cost of Goods Sold
        for acct in &accounts {
            if acct.account_type == "expense" && acct.subtype.as_deref() == Some("cost_of_goods") {
                total_cogs += acct.balance;
            }
        }

        if total_cogs > 0.0 {
            line_num += 1;
            lines.push(Self::make_line(line_num, 0, "total", "Cost of Goods Sold", &format!("{:.2}", total_cogs), None, None));
        }

        let gross_profit = total_revenue - total_cogs;
        let gross_margin = if total_revenue > 0.0 { (gross_profit / total_revenue) * 100.0 } else { 0.0 };

        line_num += 1;
        lines.push(Self::make_line(line_num, 0, "subtotal", "Gross Profit", &format!("{:.2}", gross_profit), None, None));

        // Operating Expenses
        line_num += 1;
        lines.push(Self::make_line(line_num, 0, "header", "OPERATING EXPENSES", "", None, None));

        for acct in &accounts {
            if acct.account_type == "expense" && acct.subtype.as_deref() != Some("cost_of_goods") {
                let balance = acct.balance;
                total_operating_expenses += balance;
                line_num += 1;
                lines.push(Self::make_detail_line(line_num, 1, &acct.account_code, &acct.account_name, balance, "operating_expense"));
            }
        }

        line_num += 1;
        lines.push(Self::make_line(line_num, 0, "total", "Total Operating Expenses", &format!("{:.2}", total_operating_expenses), None, None));

        let operating_income = gross_profit - total_operating_expenses;
        let operating_margin = if total_revenue > 0.0 { (operating_income / total_revenue) * 100.0 } else { 0.0 };

        line_num += 1;
        lines.push(Self::make_line(line_num, 0, "subtotal", "Operating Income", &format!("{:.2}", operating_income), None, None));

        let income_before_tax = operating_income + total_other_income - total_other_expense - income_tax;
        let net_income = income_before_tax - income_tax;
        let net_margin = if total_revenue > 0.0 { (net_income / total_revenue) * 100.0 } else { 0.0 };

        line_num += 1;
        lines.push(Self::make_line(line_num, 0, "total", "NET INCOME", &format!("{:.2}", net_income), None, None));

        let is_summary = IncomeStatementSummary {
            total_revenue: format!("{:.2}", total_revenue),
            total_cost_of_goods_sold: format!("{:.2}", total_cogs),
            gross_profit: format!("{:.2}", gross_profit),
            gross_profit_margin: format!("{:.2}%", gross_margin),
            total_operating_expenses: format!("{:.2}", total_operating_expenses),
            operating_income: format!("{:.2}", operating_income),
            operating_margin: format!("{:.2}%", operating_margin),
            total_other_income: format!("{:.2}", total_other_income),
            total_other_expense: format!("{:.2}", total_other_expense),
            income_before_tax: format!("{:.2}", income_before_tax),
            income_tax_expense: format!("{:.2}", income_tax),
            net_income: format!("{:.2}", net_income),
            net_profit_margin: format!("{:.2}%", net_margin),
        };

        let stmt = FinancialStatement {
            id: Uuid::new_v4(),
            organization_id: org_id,
            definition_id: request.definition_id.unwrap_or(Uuid::nil()),
            report_name: "Income Statement".to_string(),
            report_type: "income_statement".to_string(),
            as_of_date: request.as_of_date,
            period_name: request.period_name.clone(),
            fiscal_year: request.fiscal_year,
            currency_code: currency_code.to_string(),
            lines,
            totals: serde_json::to_value(&is_summary).unwrap_or(serde_json::json!({})),
            is_balanced: true,
            generated_at: chrono::Utc::now(),
            generated_by,
            metadata: serde_json::json!({}),
            created_at: chrono::Utc::now(),
        };

        self.repository.save_statement(&stmt).await?;
        Ok(stmt)
    }

    /// Generate a Trial Balance report (formal version)
    async fn generate_trial_balance_report(
        &self,
        org_id: Uuid,
        request: &FinancialStatementRequest,
        currency_code: &str,
        generated_by: Option<Uuid>,
    ) -> AtlasResult<FinancialStatement> {
        let accounts = self.repository.get_account_balances(
            org_id, request.as_of_date, request.period_name.as_deref(),
        ).await?;

        let mut lines = Vec::new();
        let mut total_debit = 0.0_f64;
        let mut total_credit = 0.0_f64;
        let mut line_num = 0i32;

        for acct in &accounts {
            let is_debit = DEBIT_NATURE_TYPES.contains(&acct.account_type.as_str());
            let balance = acct.balance;
            if balance.abs() < 0.01 { continue; }

            let (debit, credit) = if is_debit {
                total_debit += balance;
                (format!("{:.2}", balance), "".to_string())
            } else {
                total_credit += balance;
                ("".to_string(), format!("{:.2}", balance))
            };

            line_num += 1;
            lines.push(FinancialStatementLine {
                line_number: line_num, indent_level: 0,
                line_type: "detail".to_string(),
                account_code_range: Some(acct.account_code.clone()),
                label: format!("{} - {}", acct.account_code, acct.account_name),
                classification: Some(acct.account_type.clone()),
                amount: debit,
                comparative_amount: Some(credit),
                variance_amount: None, variance_percent: None,
                ytd_amount: None, budget_amount: None, budget_variance: None,
                is_debit_nature: Some(is_debit),
                sign_convention: "normal".to_string(),
                metadata: serde_json::json!({}),
            });
        }

        let is_balanced = (total_debit - total_credit).abs() < 0.01;

        line_num += 1;
        lines.push(FinancialStatementLine {
            line_number: line_num, indent_level: 0,
            line_type: "total".to_string(), account_code_range: None,
            label: "TOTAL".to_string(), classification: None,
            amount: format!("{:.2}", total_debit),
            comparative_amount: Some(format!("{:.2}", total_credit)),
            variance_amount: Some(format!("{:.2}", total_debit - total_credit)),
            variance_percent: None,
            ytd_amount: None, budget_amount: None, budget_variance: None,
            is_debit_nature: None, sign_convention: "normal".to_string(),
            metadata: serde_json::json!({}),
        });

        let stmt = FinancialStatement {
            id: Uuid::new_v4(),
            organization_id: org_id,
            definition_id: request.definition_id.unwrap_or(Uuid::nil()),
            report_name: "Trial Balance".to_string(),
            report_type: "trial_balance".to_string(),
            as_of_date: request.as_of_date,
            period_name: request.period_name.clone(),
            fiscal_year: request.fiscal_year,
            currency_code: currency_code.to_string(),
            lines,
            totals: serde_json::json!({
                "total_debit": format!("{:.2}", total_debit),
                "total_credit": format!("{:.2}", total_credit),
                "difference": format!("{:.2}", total_debit - total_credit),
            }),
            is_balanced,
            generated_at: chrono::Utc::now(),
            generated_by,
            metadata: serde_json::json!({}),
            created_at: chrono::Utc::now(),
        };

        self.repository.save_statement(&stmt).await?;
        Ok(stmt)
    }

    /// Generate a Cash Flow Statement (indirect method)
    async fn generate_cash_flow_statement(
        &self,
        org_id: Uuid,
        request: &FinancialStatementRequest,
        currency_code: &str,
        generated_by: Option<Uuid>,
    ) -> AtlasResult<FinancialStatement> {
        let accounts = self.repository.get_account_balances(
            org_id, request.as_of_date, request.period_name.as_deref(),
        ).await?;

        let mut lines = Vec::new();
        let mut line_num = 0i32;

        // Operating Activities (simplified: start from net income equivalent)
        let net_income: f64 = accounts.iter()
            .filter(|a| a.account_type == "revenue")
            .map(|a| a.balance)
            .sum::<f64>()
            - accounts.iter()
            .filter(|a| a.account_type == "expense")
            .map(|a| a.balance)
            .sum::<f64>();

        line_num += 1;
        lines.push(Self::make_line(line_num, 0, "header", "CASH FLOWS FROM OPERATING ACTIVITIES", "", None, None));

        line_num += 1;
        lines.push(Self::make_line(line_num, 1, "detail", "Net Income", &format!("{:.2}", net_income), None, None));

        line_num += 1;
        lines.push(Self::make_line(line_num, 1, "detail", "Adjustments for changes in working capital", "", None, None));

        let operating_cash = net_income; // Simplified

        line_num += 1;
        lines.push(Self::make_line(line_num, 0, "subtotal", "Net Cash from Operating Activities", &format!("{:.2}", operating_cash), None, None));

        // Investing Activities
        line_num += 1;
        lines.push(Self::make_line(line_num, 0, "header", "CASH FLOWS FROM INVESTING ACTIVITIES", "", None, None));

        let investing_cash = 0.0; // Simplified: no investing activity data

        line_num += 1;
        lines.push(Self::make_line(line_num, 0, "subtotal", "Net Cash from Investing Activities", &format!("{:.2}", investing_cash), None, None));

        // Financing Activities
        line_num += 1;
        lines.push(Self::make_line(line_num, 0, "header", "CASH FLOWS FROM FINANCING ACTIVITIES", "", None, None));

        let financing_cash = 0.0; // Simplified

        line_num += 1;
        lines.push(Self::make_line(line_num, 0, "subtotal", "Net Cash from Financing Activities", &format!("{:.2}", financing_cash), None, None));

        let net_change = operating_cash + investing_cash + financing_cash;

        line_num += 1;
        lines.push(Self::make_line(line_num, 0, "total", "Net Change in Cash", &format!("{:.2}", net_change), None, None));

        let stmt = FinancialStatement {
            id: Uuid::new_v4(),
            organization_id: org_id,
            definition_id: request.definition_id.unwrap_or(Uuid::nil()),
            report_name: "Cash Flow Statement".to_string(),
            report_type: "cash_flow_statement".to_string(),
            as_of_date: request.as_of_date,
            period_name: request.period_name.clone(),
            fiscal_year: request.fiscal_year,
            currency_code: currency_code.to_string(),
            lines,
            totals: serde_json::json!({
                "operating_activities": format!("{:.2}", operating_cash),
                "investing_activities": format!("{:.2}", investing_cash),
                "financing_activities": format!("{:.2}", financing_cash),
                "net_change": format!("{:.2}", net_change),
            }),
            is_balanced: true,
            generated_at: chrono::Utc::now(),
            generated_by,
            metadata: serde_json::json!({}),
            created_at: chrono::Utc::now(),
        };

        self.repository.save_statement(&stmt).await?;
        Ok(stmt)
    }

    /// Generate Statement of Changes in Equity
    async fn generate_equity_statement(
        &self,
        org_id: Uuid,
        request: &FinancialStatementRequest,
        currency_code: &str,
        generated_by: Option<Uuid>,
    ) -> AtlasResult<FinancialStatement> {
        let accounts = self.repository.get_account_balances(
            org_id, request.as_of_date, request.period_name.as_deref(),
        ).await?;

        let mut lines = Vec::new();
        let mut line_num = 0i32;

        let beginning_equity = 0.0_f64; // Would come from prior period
        let net_income: f64 = accounts.iter()
            .filter(|a| a.account_type == "revenue").map(|a| a.balance).sum::<f64>()
            - accounts.iter()
            .filter(|a| a.account_type == "expense").map(|a| a.balance).sum::<f64>();
        let dividends = 0.0_f64; // Would come from equity transactions
        let other_comprehensive = 0.0_f64;
        let ending_equity = beginning_equity + net_income - dividends + other_comprehensive;

        line_num += 1;
        lines.push(Self::make_line(line_num, 0, "detail", "Beginning Balance", &format!("{:.2}", beginning_equity), None, None));
        line_num += 1;
        lines.push(Self::make_line(line_num, 0, "detail", "Net Income", &format!("{:.2}", net_income), None, None));
        line_num += 1;
        lines.push(Self::make_line(line_num, 0, "detail", "Dividends", &format!("{:.2}", dividends), None, None));
        line_num += 1;
        lines.push(Self::make_line(line_num, 0, "detail", "Other Comprehensive Income", &format!("{:.2}", other_comprehensive), None, None));
        line_num += 1;
        lines.push(Self::make_line(line_num, 0, "total", "Ending Balance", &format!("{:.2}", ending_equity), None, None));

        let stmt = FinancialStatement {
            id: Uuid::new_v4(),
            organization_id: org_id,
            definition_id: request.definition_id.unwrap_or(Uuid::nil()),
            report_name: "Statement of Changes in Equity".to_string(),
            report_type: "statement_of_changes_in_equity".to_string(),
            as_of_date: request.as_of_date,
            period_name: request.period_name.clone(),
            fiscal_year: request.fiscal_year,
            currency_code: currency_code.to_string(),
            lines,
            totals: serde_json::json!({
                "beginning_equity": format!("{:.2}", beginning_equity),
                "net_income": format!("{:.2}", net_income),
                "dividends": format!("{:.2}", dividends),
                "ending_equity": format!("{:.2}", ending_equity),
            }),
            is_balanced: true,
            generated_at: chrono::Utc::now(),
            generated_by,
            metadata: serde_json::json!({}),
            created_at: chrono::Utc::now(),
        };

        self.repository.save_statement(&stmt).await?;
        Ok(stmt)
    }

    // ========================================================================
    // Helper Functions
    // ========================================================================

    fn make_line(
        line_number: i32, indent_level: i32, line_type: &str,
        label: &str, amount: &str,
        classification: Option<&str>, account_code_range: Option<&str>,
    ) -> FinancialStatementLine {
        FinancialStatementLine {
            line_number, indent_level, line_type: line_type.to_string(),
            account_code_range: account_code_range.map(|s| s.to_string()),
            label: label.to_string(),
            classification: classification.map(|s| s.to_string()),
            amount: amount.to_string(),
            comparative_amount: None, variance_amount: None, variance_percent: None,
            ytd_amount: None, budget_amount: None, budget_variance: None,
            is_debit_nature: None, sign_convention: "normal".to_string(),
            metadata: serde_json::json!({}),
        }
    }

    fn make_detail_line(
        line_number: i32, indent_level: i32,
        account_code: &str, account_name: &str, balance: f64,
        classification: &str,
    ) -> FinancialStatementLine {
        FinancialStatementLine {
            line_number, indent_level, line_type: "detail".to_string(),
            account_code_range: Some(account_code.to_string()),
            label: format!("{} - {}", account_code, account_name),
            classification: Some(classification.to_string()),
            amount: format!("{:.2}", balance),
            comparative_amount: None, variance_amount: None, variance_percent: None,
            ytd_amount: None, budget_amount: None, budget_variance: None,
            is_debit_nature: None, sign_convention: "normal".to_string(),
            metadata: serde_json::json!({}),
        }
    }

    /// Calculate financial ratios from balance sheet and income statement data
    pub fn calculate_ratios(
        total_current_assets: f64,
        total_current_liabilities: f64,
        total_assets: f64,
        total_equity: f64,
        net_income: f64,
        total_revenue: f64,
    ) -> serde_json::Value {
        let current_ratio = if total_current_liabilities > 0.0 {
            total_current_assets / total_current_liabilities
        } else { 0.0 };

        let debt_to_equity = if total_equity > 0.0 {
            (total_assets - total_equity) / total_equity
        } else { 0.0 };

        let roe = if total_equity > 0.0 {
            (net_income / total_equity) * 100.0
        } else { 0.0 };

        let roa = if total_assets > 0.0 {
            (net_income / total_assets) * 100.0
        } else { 0.0 };

        let net_margin = if total_revenue > 0.0 {
            (net_income / total_revenue) * 100.0
        } else { 0.0 };

        serde_json::json!({
            "current_ratio": format!("{:.2}", current_ratio),
            "debt_to_equity": format!("{:.2}", debt_to_equity),
            "return_on_equity": format!("{:.2}%", roe),
            "return_on_assets": format!("{:.2}%", roa),
            "net_profit_margin": format!("{:.2}%", net_margin),
        })
    }

    /// Get a previously generated statement
    pub async fn get_statement(&self, id: Uuid) -> AtlasResult<Option<FinancialStatement>> {
        self.repository.get_statement(id).await
    }

    /// List generated statements
    pub async fn list_statements(&self, org_id: Uuid, report_type: Option<&str>) -> AtlasResult<Vec<FinancialStatement>> {
        self.repository.list_statements(org_id, report_type).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_report_types() {
        assert!(VALID_REPORT_TYPES.contains(&"balance_sheet"));
        assert!(VALID_REPORT_TYPES.contains(&"income_statement"));
        assert!(VALID_REPORT_TYPES.contains(&"cash_flow_statement"));
        assert!(VALID_REPORT_TYPES.contains(&"trial_balance"));
        assert!(VALID_REPORT_TYPES.contains(&"statement_of_changes_in_equity"));
        assert_eq!(VALID_REPORT_TYPES.len(), 5);
    }

    #[test]
    fn test_valid_bs_classifications() {
        assert!(VALID_BS_CLASSIFICATIONS.contains(&"current_asset"));
        assert!(VALID_BS_CLASSIFICATIONS.contains(&"non_current_asset"));
        assert!(VALID_BS_CLASSIFICATIONS.contains(&"current_liability"));
        assert!(VALID_BS_CLASSIFICATIONS.contains(&"non_current_liability"));
        assert!(VALID_BS_CLASSIFICATIONS.contains(&"equity"));
        assert_eq!(VALID_BS_CLASSIFICATIONS.len(), 5);
    }

    #[test]
    fn test_valid_is_classifications() {
        assert!(VALID_IS_CLASSIFICATIONS.contains(&"revenue"));
        assert!(VALID_IS_CLASSIFICATIONS.contains(&"cost_of_goods_sold"));
        assert!(VALID_IS_CLASSIFICATIONS.contains(&"gross_profit"));
        assert!(VALID_IS_CLASSIFICATIONS.contains(&"operating_expense"));
        assert!(VALID_IS_CLASSIFICATIONS.contains(&"operating_income"));
        assert!(VALID_IS_CLASSIFICATIONS.contains(&"net_income"));
        assert_eq!(VALID_IS_CLASSIFICATIONS.len(), 10);
    }

    #[test]
    fn test_valid_line_types() {
        assert!(VALID_LINE_TYPES.contains(&"header"));
        assert!(VALID_LINE_TYPES.contains(&"detail"));
        assert!(VALID_LINE_TYPES.contains(&"subtotal"));
        assert!(VALID_LINE_TYPES.contains(&"total"));
        assert!(VALID_LINE_TYPES.contains(&"blank"));
        assert_eq!(VALID_LINE_TYPES.len(), 5);
    }

    #[test]
    fn test_valid_sign_conventions() {
        assert!(VALID_SIGN_CONVENTIONS.contains(&"normal"));
        assert!(VALID_SIGN_CONVENTIONS.contains(&"negate"));
        assert_eq!(VALID_SIGN_CONVENTIONS.len(), 2);
    }

    #[test]
    fn test_debit_nature_types() {
        assert!(DEBIT_NATURE_TYPES.contains(&"asset"));
        assert!(DEBIT_NATURE_TYPES.contains(&"expense"));
        assert_eq!(DEBIT_NATURE_TYPES.len(), 2);
    }

    #[test]
    fn test_credit_nature_types() {
        assert!(CREDIT_NATURE_TYPES.contains(&"liability"));
        assert!(CREDIT_NATURE_TYPES.contains(&"equity"));
        assert!(CREDIT_NATURE_TYPES.contains(&"revenue"));
        assert_eq!(CREDIT_NATURE_TYPES.len(), 3);
    }

    #[test]
    fn test_calculate_ratios() {
        let ratios = FinancialStatementEngine::calculate_ratios(
            500000.0,  // current assets
            250000.0,  // current liabilities
            1000000.0, // total assets
            600000.0,  // total equity
            120000.0,  // net income
            800000.0,  // total revenue
        );

        assert_eq!(ratios["current_ratio"], "2.00");
        assert!(ratios["debt_to_equity"].as_str().unwrap().contains("0.67"));
        assert!(ratios["return_on_equity"].as_str().unwrap().contains("20.00"));
        assert!(ratios["return_on_assets"].as_str().unwrap().contains("12.00"));
        assert!(ratios["net_profit_margin"].as_str().unwrap().contains("15.00"));
    }

    #[test]
    fn test_calculate_ratios_zero_equity() {
        let ratios = FinancialStatementEngine::calculate_ratios(
            100.0, 50.0, 200.0, 0.0, 10.0, 100.0,
        );
        assert_eq!(ratios["return_on_equity"], "0.00%");
        assert_eq!(ratios["debt_to_equity"], "0.00");
    }

    #[test]
    fn test_calculate_ratios_zero_revenue() {
        let ratios = FinancialStatementEngine::calculate_ratios(
            100.0, 50.0, 200.0, 100.0, 0.0, 0.0,
        );
        assert_eq!(ratios["net_profit_margin"], "0.00%");
    }

    #[test]
    fn test_make_line() {
        let line = FinancialStatementEngine::make_line(
            1, 0, "header", "ASSETS", "", None, None,
        );
        assert_eq!(line.line_number, 1);
        assert_eq!(line.indent_level, 0);
        assert_eq!(line.line_type, "header");
        assert_eq!(line.label, "ASSETS");
        assert_eq!(line.amount, "");
        assert_eq!(line.sign_convention, "normal");
    }

    #[test]
    fn test_make_detail_line() {
        let line = FinancialStatementEngine::make_detail_line(
            5, 1, "1000", "Cash", 50000.0, "current_asset",
        );
        assert_eq!(line.line_number, 5);
        assert_eq!(line.line_type, "detail");
        assert_eq!(line.account_code_range, Some("1000".to_string()));
        assert_eq!(line.label, "1000 - Cash");
        assert_eq!(line.amount, "50000.00");
        assert_eq!(line.classification, Some("current_asset".to_string()));
    }

    #[test]
    fn test_balance_sheet_account_classification() {
        // Test that account types correctly map to debit/credit nature
        assert!(DEBIT_NATURE_TYPES.contains(&"asset"));
        assert!(DEBIT_NATURE_TYPES.contains(&"expense"));
        assert!(!DEBIT_NATURE_TYPES.contains(&"liability"));
        assert!(!DEBIT_NATURE_TYPES.contains(&"equity"));
        assert!(!DEBIT_NATURE_TYPES.contains(&"revenue"));

        assert!(CREDIT_NATURE_TYPES.contains(&"liability"));
        assert!(CREDIT_NATURE_TYPES.contains(&"equity"));
        assert!(CREDIT_NATURE_TYPES.contains(&"revenue"));
        assert!(!CREDIT_NATURE_TYPES.contains(&"asset"));
        assert!(!CREDIT_NATURE_TYPES.contains(&"expense"));
    }
}
