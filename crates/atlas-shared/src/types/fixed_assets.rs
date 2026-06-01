use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
// ============================================================================
// Fixed Assets Management (Oracle Fusion Fixed Assets)
// ============================================================================

/// Asset category definition
/// Oracle Fusion: Fixed Assets > Asset Categories
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetCategory {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    /// Default depreciation method for assets in this category
    /// '`straight_line`', '`declining_balance`', '`sum_of_years_digits`'
    pub default_depreciation_method: String,
    /// Default useful life in months
    pub default_useful_life_months: i32,
    /// Default salvage value percentage
    pub default_salvage_value_percent: String,
    /// Default GL account codes
    pub default_asset_account_code: Option<String>,
    pub default_accum_depr_account_code: Option<String>,
    pub default_depr_expense_account_code: Option<String>,
    pub default_gain_loss_account_code: Option<String>,
    pub is_active: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Asset book definition (corporate, tax)
/// Oracle Fusion: Fixed Assets > Books
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetBook {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    /// Book type: 'corporate', 'tax'
    pub book_type: String,
    pub auto_depreciation: bool,
    /// Depreciation calendar: 'monthly', 'quarterly', 'yearly'
    pub depreciation_calendar: String,
    pub current_fiscal_year: Option<i32>,
    pub last_depreciation_date: Option<chrono::NaiveDate>,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Fixed asset record
/// Oracle Fusion: Fixed Assets > Assets
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FixedAsset {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub asset_number: String,
    pub asset_name: String,
    pub description: Option<String>,
    pub category_id: Option<Uuid>,
    pub category_code: Option<String>,
    pub book_id: Option<Uuid>,
    pub book_code: Option<String>,
    /// Asset type: 'tangible', 'intangible', 'leased', 'cipc'
    pub asset_type: String,
    /// Lifecycle status: 'draft', 'acquired', '`in_service`', '`under_construction`',
    /// 'disposed', 'retired', 'transferred'
    pub status: String,
    // Financial details
    pub original_cost: String,
    pub current_cost: String,
    pub salvage_value: String,
    pub salvage_value_percent: String,
    // Depreciation parameters
    pub depreciation_method: String,
    pub useful_life_months: i32,
    pub declining_balance_rate: Option<String>,
    // Depreciation calculations
    pub depreciable_basis: String,
    pub accumulated_depreciation: String,
    pub net_book_value: String,
    pub depreciation_per_period: String,
    // Depreciation tracking
    pub periods_depreciated: i32,
    pub last_depreciation_date: Option<chrono::NaiveDate>,
    pub last_depreciation_amount: String,
    // Date tracking
    pub acquisition_date: Option<chrono::NaiveDate>,
    pub in_service_date: Option<chrono::NaiveDate>,
    pub disposal_date: Option<chrono::NaiveDate>,
    pub retirement_date: Option<chrono::NaiveDate>,
    // Location and assignment
    pub location: Option<String>,
    pub department_id: Option<Uuid>,
    pub department_name: Option<String>,
    pub custodian_id: Option<Uuid>,
    pub custodian_name: Option<String>,
    // Physical details
    pub serial_number: Option<String>,
    pub tag_number: Option<String>,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    // Dates
    pub warranty_expiry: Option<chrono::NaiveDate>,
    pub insurance_policy_number: Option<String>,
    pub insurance_expiry: Option<chrono::NaiveDate>,
    pub lease_number: Option<String>,
    pub lease_expiry: Option<chrono::NaiveDate>,
    // GL account codes
    pub asset_account_code: Option<String>,
    pub accum_depr_account_code: Option<String>,
    pub depr_expense_account_code: Option<String>,
    pub gain_loss_account_code: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Asset depreciation history entry
/// Oracle Fusion: Fixed Assets > Depreciation History
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetDepreciationHistory {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub asset_id: Uuid,
    pub fiscal_year: i32,
    pub period_number: i32,
    pub period_name: Option<String>,
    pub depreciation_date: chrono::NaiveDate,
    pub depreciation_amount: String,
    pub accumulated_depreciation: String,
    pub net_book_value: String,
    pub depreciation_method: String,
    pub journal_entry_id: Option<Uuid>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

/// Asset transfer record
/// Oracle Fusion: Fixed Assets > Asset Transfers
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetTransfer {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub transfer_number: String,
    pub asset_id: Uuid,
    // From
    pub from_department_id: Option<Uuid>,
    pub from_department_name: Option<String>,
    pub from_location: Option<String>,
    pub from_custodian_id: Option<Uuid>,
    pub from_custodian_name: Option<String>,
    // To
    pub to_department_id: Option<Uuid>,
    pub to_department_name: Option<String>,
    pub to_location: Option<String>,
    pub to_custodian_id: Option<Uuid>,
    pub to_custodian_name: Option<String>,
    // Details
    pub transfer_date: chrono::NaiveDate,
    pub reason: Option<String>,
    /// Status: 'pending', 'approved', 'rejected', 'completed'
    pub status: String,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub rejected_reason: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Asset retirement record
/// Oracle Fusion: Fixed Assets > Asset Retirements
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetRetirement {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub retirement_number: String,
    pub asset_id: Uuid,
    /// Retirement type: 'sale', 'scrap', 'donation', '`write_off`', 'casualty'
    pub retirement_type: String,
    pub retirement_date: chrono::NaiveDate,
    // Financial details
    pub proceeds: String,
    pub removal_cost: String,
    pub net_book_value_at_retirement: String,
    pub accumulated_depreciation_at_retirement: String,
    pub gain_loss_amount: String,
    /// 'gain' or 'loss'
    pub gain_loss_type: Option<String>,
    // Account references
    pub gain_account_code: Option<String>,
    pub loss_account_code: Option<String>,
    pub cash_account_code: Option<String>,
    pub asset_account_code: Option<String>,
    pub accum_depr_account_code: Option<String>,
    // Reference
    pub reference_number: Option<String>,
    pub buyer_name: Option<String>,
    pub notes: Option<String>,
    /// Status: 'pending', 'approved', 'completed', 'cancelled'
    pub status: String,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub journal_entry_id: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// Asset Depreciation Engine (Oracle Fusion: Fixed Assets > Depreciation)
// ============================================================================

/// Depreciation calculation result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DepreciationResult {
    pub asset_id: Uuid,
    pub fiscal_year: i32,
    pub period_number: i32,
    pub depreciation_date: chrono::NaiveDate,
    pub depreciation_amount: String,
    pub accumulated_depreciation: String,
    pub net_book_value: String,
    pub depreciation_method: String,
}

/// Depreciation schedule (all periods for an asset)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DepreciationSchedule {
    pub asset_id: Uuid,
    pub asset_number: String,
    pub asset_name: String,
    pub original_cost: String,
    pub salvage_value: String,
    pub depreciable_basis: String,
    pub useful_life_months: i32,
    pub depreciation_method: String,
    pub declining_balance_rate: Option<String>,
    pub periods: Vec<DepreciationResult>,
    pub total_depreciation: String,
}

// ════════════════════════════════════════════════════════════════════════════════
// AP/AR Netting (Oracle Fusion: Financials > Netting)
// ════════════════════════════════════════════════════════════════════════════════
//
// Allows organizations to settle payables and receivables with the same
// trading partner by netting amounts against each other, reducing cash
// movement. Only the net difference is paid or received.
//
// Workflow: Create Agreement → Create Netting Batch → Select Transactions →
//           Approve → Settle (auto-generate payment/receipt for net difference)

/// Netting Agreement between two trading partners
/// Oracle Fusion: Financials > Netting > Netting Agreements
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NettingAgreement {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Unique agreement code
    pub agreement_number: String,
    /// Agreement name/description
    pub name: String,
    pub description: Option<String>,
    /// The trading partner (supplier AND customer)
    pub partner_id: Uuid,
    pub partner_number: Option<String>,
    pub partner_name: Option<String>,
    /// Netting currency
    pub currency_code: String,
    /// Netting direction: '`payables_to_receivables`', '`receivables_to_payables`', '`bi_directional`'
    pub netting_direction: String,
    /// Settlement method for the net difference: 'automatic', 'manual'
    pub settlement_method: String,
    /// Minimum netting amount (don't net if difference is below this)
    pub minimum_netting_amount: String,
    /// Maximum netting amount per batch
    pub maximum_netting_amount: Option<String>,
    /// Whether to auto-select eligible transactions
    pub auto_select_transactions: bool,
    /// Selection criteria for auto-selection
    pub selection_criteria: serde_json::Value,
    /// GL accounts for netting entries
    pub netting_clearing_account: Option<String>,
    pub ap_clearing_account: Option<String>,
    pub ar_clearing_account: Option<String>,
    /// Approval required flag
    pub approval_required: bool,
    /// Status: 'draft', 'active', 'inactive', 'terminated'
    pub status: String,
    /// Effective dates
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Netting Batch (a single netting run)
/// Oracle Fusion: Financials > Netting > Netting Batches
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NettingBatch {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Batch number (auto-generated)
    pub batch_number: String,
    /// Reference to the netting agreement
    pub agreement_id: Uuid,
    /// Netting date
    pub netting_date: chrono::NaiveDate,
    /// GL posting date
    pub gl_date: Option<chrono::NaiveDate>,
    /// Partner info (denormalized)
    pub partner_id: Uuid,
    pub partner_name: Option<String>,
    /// Currency
    pub currency_code: String,
    /// Totals
    pub total_payables_amount: String,
    pub total_receivables_amount: String,
    pub net_difference: String,
    /// Direction of settlement: 'pay', 'receive', 'zero'
    pub settlement_direction: String,
    /// Status: 'draft', 'submitted', 'approved', 'settled', 'cancelled', 'reversed'
    pub status: String,
    /// Counts
    pub payable_transaction_count: i32,
    pub receivable_transaction_count: i32,
    /// Approval
    pub submitted_by: Option<Uuid>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub rejected_reason: Option<String>,
    /// Settlement references
    pub settlement_payment_id: Option<Uuid>,
    pub settlement_receipt_id: Option<Uuid>,
    pub journal_entry_id: Option<Uuid>,
    pub settled_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Netting Transaction Line (selected AP or AR transaction in a netting batch)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NettingTransactionLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub batch_id: Uuid,
    /// Line number within batch
    pub line_number: i32,
    /// Source: 'payable' or 'receivable'
    pub source_type: String,
    /// Source transaction ID (AP invoice or AR transaction)
    pub source_id: Uuid,
    /// Source transaction number
    pub source_number: Option<String>,
    /// Source transaction date
    pub source_date: Option<chrono::NaiveDate>,
    /// Original amount of the source transaction
    pub original_amount: String,
    /// Amount selected for netting (may be partial)
    pub netting_amount: String,
    /// Remaining amount after netting
    pub remaining_amount: String,
    /// Currency code
    pub currency_code: String,
    /// Status: 'selected', 'netted', 'cancelled'
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Netting settlement summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NettingSettlementSummary {
    pub batch_id: Uuid,
    pub batch_number: String,
    pub partner_name: Option<String>,
    pub total_payables: String,
    pub total_receivables: String,
    pub net_difference: String,
    pub settlement_direction: String,
    pub payable_transaction_count: i32,
    pub receivable_transaction_count: i32,
}

/// Netting dashboard summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NettingDashboardSummary {
    pub total_agreements: i32,
    pub active_agreements: i32,
    pub total_batches: i32,
    pub draft_batches: i32,
    pub pending_approval_batches: i32,
    pub settled_batches: i32,
    pub total_payables_netted: String,
    pub total_receivables_netted: String,
    pub total_net_difference_settled: String,
}

// ════════════════════════════════════════════════════════════════════════════════
// Financial Statement Generation (Oracle Fusion: General Ledger > Financial Reporting)
// ════════════════════════════════════════════════════════════════════════════════
//
// Generates standard financial statements from General Ledger data:
// - Balance Sheet (Statement of Financial Position)
// - Income Statement (Profit & Loss)
// - Cash Flow Statement (indirect method)
// - Trial Balance (already exists, but this is the formal report version)
// - Statement of Changes in Equity

/// Financial statement report type (re-exports the existing `FinancialReportType`)
pub use crate::types::FinancialReportType as FinancialStatementReportType;

/// Balance sheet classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BalanceSheetClassification {
    CurrentAsset,
    NonCurrentAsset,
    CurrentLiability,
    NonCurrentLiability,
    Equity,
}

/// Income statement classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IncomeStatementClassification {
    Revenue,
    CostOfGoodsSold,
    GrossProfit,
    OperatingExpense,
    OperatingIncome,
    OtherIncome,
    OtherExpense,
    IncomeBeforeTax,
    IncomeTaxExpense,
    NetIncome,
}

/// Financial statement report definition
/// Oracle Fusion: General Ledger > Financial Reporting > Report Definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinancialStatementDefinition {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Report code
    pub code: String,
    /// Report name
    pub name: String,
    pub description: Option<String>,
    /// Type of financial statement
    pub report_type: String,
    /// Currency for reporting
    pub currency_code: String,
    /// Whether to include comparative periods
    pub include_comparative: bool,
    /// Number of comparative periods
    pub comparative_period_count: i32,
    /// Row definitions (account ranges and subtotals)
    pub row_definitions: serde_json::Value,
    /// Column definitions (current period, YTD, budget, variance)
    pub column_definitions: serde_json::Value,
    /// Accounting period filter
    pub period_name: Option<String>,
    /// Fiscal year
    pub fiscal_year: Option<i32>,
    /// Whether this is a system report
    pub is_system: bool,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Generated financial statement (the actual report)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinancialStatement {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Report definition used
    pub definition_id: Uuid,
    /// Report name
    pub report_name: String,
    /// Type of report
    pub report_type: String,
    /// As-of date for the report
    pub as_of_date: chrono::NaiveDate,
    /// Period name (e.g., "JAN-2026", "Q1-2026")
    pub period_name: Option<String>,
    /// Fiscal year
    pub fiscal_year: Option<i32>,
    /// Currency
    pub currency_code: String,
    /// Report lines
    pub lines: Vec<FinancialStatementLine>,
    /// Totals and subtotals
    pub totals: serde_json::Value,
    /// Whether the report balances
    pub is_balanced: bool,
    /// Generation timestamp
    pub generated_at: DateTime<Utc>,
    pub generated_by: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// Single line in a financial statement
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinancialStatementLine {
    /// Line number for ordering
    pub line_number: i32,
    /// Display indentation level (0 = top level, 1 = sub-group, etc.)
    pub indent_level: i32,
    /// Line type: 'header', 'detail', 'subtotal', 'total', 'blank'
    pub line_type: String,
    /// Account code range (for detail lines)
    pub account_code_range: Option<String>,
    /// Display label
    pub label: String,
    /// Classification for grouping
    pub classification: Option<String>,
    /// Amount for the current period
    pub amount: String,
    /// Comparative amount (prior period)
    pub comparative_amount: Option<String>,
    /// Variance amount (current - comparative)
    pub variance_amount: Option<String>,
    /// Variance percentage
    pub variance_percent: Option<String>,
    /// Year-to-date amount
    pub ytd_amount: Option<String>,
    /// Budget amount
    pub budget_amount: Option<String>,
    /// Budget variance
    pub budget_variance: Option<String>,
    /// Whether this line represents a debit balance nature
    pub is_debit_nature: Option<bool>,
    /// Sign convention: 'normal', 'negate' (for contra-accounts)
    pub sign_convention: String,
    pub metadata: serde_json::Value,
}

/// Financial statement generation request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinancialStatementRequest {
    /// Report definition ID (None = generate ad-hoc)
    pub definition_id: Option<Uuid>,
    /// Report type (required for ad-hoc)
    pub report_type: Option<String>,
    /// As-of date
    pub as_of_date: chrono::NaiveDate,
    /// Period name filter
    pub period_name: Option<String>,
    /// Fiscal year filter
    pub fiscal_year: Option<i32>,
    /// Currency code (defaults to base currency)
    pub currency_code: Option<String>,
    /// Whether to include comparative period
    pub include_comparative: Option<bool>,
    /// Row definitions (for ad-hoc reports)
    pub row_definitions: Option<serde_json::Value>,
    /// Column definitions (for ad-hoc reports)
    pub column_definitions: Option<serde_json::Value>,
}

/// Balance sheet summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BalanceSheetSummary {
    pub total_current_assets: String,
    pub total_non_current_assets: String,
    pub total_assets: String,
    pub total_current_liabilities: String,
    pub total_non_current_liabilities: String,
    pub total_liabilities: String,
    pub total_equity: String,
    pub total_liabilities_and_equity: String,
    pub is_balanced: bool,
}

/// Income statement summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IncomeStatementSummary {
    pub total_revenue: String,
    pub total_cost_of_goods_sold: String,
    pub gross_profit: String,
    pub gross_profit_margin: String,
    pub total_operating_expenses: String,
    pub operating_income: String,
    pub operating_margin: String,
    pub total_other_income: String,
    pub total_other_expense: String,
    pub income_before_tax: String,
    pub income_tax_expense: String,
    pub net_income: String,
    pub net_profit_margin: String,
}

// ═══════════════════════════════════════════════════════════════
// Journal Import (Oracle Fusion GL > Import Journals)
// ═══════════════════════════════════════════════════════════════

/// Journal Import Format Definition
/// Defines the structure and column mappings for importing journal data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JournalImportFormat {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub source_type: String, // "file", "api", "subledger"
    pub file_format: String, // "csv", "json", "fixed_width"
    pub delimiter: Option<String>,
    pub header_row: bool,
    pub ledger_id: Option<Uuid>,
    pub currency_code: String,
    pub default_date: Option<chrono::NaiveDate>,
    pub default_journal_type: Option<String>,
    pub balancing_segment: Option<String>,
    pub status: String, // "active", "inactive"
    pub validation_enabled: bool,
    pub auto_post: bool,
    pub max_errors_allowed: i32,
    pub column_mappings: serde_json::Value,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Column mapping for journal import format
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JournalImportColumnMapping {
    pub id: Uuid,
    pub format_id: Uuid,
    pub column_position: i32,
    pub source_column: String,
    pub target_field: String, // "account_code", "debit", "credit", "description", etc.
    pub data_type: String,    // "string", "number", "date"
    pub is_required: bool,
    pub default_value: Option<String>,
    pub transformation: Option<String>,
    pub validation_rule: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Journal Import Batch
/// Represents a single import run
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JournalImportBatch {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub format_id: Uuid,
    pub batch_number: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub source: String,
    pub source_file_name: Option<String>,
    pub status: String, // "uploaded", "validating", "validated", "importing",
    // "completed", "completed_with_errors", "failed"
    pub total_rows: i32,
    pub valid_rows: i32,
    pub error_rows: i32,
    pub imported_rows: i32,
    pub ledger_id: Option<Uuid>,
    pub currency_code: String,
    pub journal_batch_id: Option<Uuid>,
    pub total_debit: String,
    pub total_credit: String,
    pub is_balanced: bool,
    /// Row-level errors encountered during import
    pub errors: serde_json::Value,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Journal Import Row
/// Individual row from import data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JournalImportRow {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub batch_id: Uuid,
    pub row_number: i32,
    pub raw_data: serde_json::Value,
    pub status: String, // "pending", "valid", "error", "imported", "skipped"
    pub account_code: Option<String>,
    pub account_name: Option<String>,
    pub description: Option<String>,
    pub entered_dr: String,
    pub entered_cr: String,
    pub currency_code: Option<String>,
    pub exchange_rate: Option<String>,
    pub gl_date: Option<chrono::NaiveDate>,
    pub reference: Option<String>,
    pub line_type: Option<String>,
    pub cost_center: Option<String>,
    pub department: Option<String>,
    pub project_code: Option<String>,
    pub error_message: Option<String>,
    pub error_field: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Journal Import Error
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JournalImportError {
    pub row_number: i32,
    pub field: String,
    pub error: String,
    pub severity: String, // "error", "warning"
    pub raw_value: Option<String>,
}

/// Journal Import Dashboard Summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JournalImportDashboardSummary {
    pub total_formats: i32,
    pub active_formats: i32,
    pub total_batches: i32,
    pub pending_batches: i32,
    pub completed_batches: i32,
    pub failed_batches: i32,
    pub total_rows_imported: i32,
    pub total_rows_with_errors: i32,
    pub recent_batches: Vec<JournalImportBatch>,
}

// ═══════════════════════════════════════════════════════════════════════════
// Inflation Adjustment (IAS 29 Hyperinflationary Economy Accounting)
// Oracle Fusion equivalent: Financials > General Ledger > Inflation Adjustment
// ═══════════════════════════════════════════════════════════════════════════

/// Inflation index definition
/// Tracks CPI or other indices for hyperinflationary economies.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InflationIndex {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub country_code: String,
    pub currency_code: String,
    pub index_type: String,
    pub is_hyperinflationary: bool,
    pub hyperinflationary_start_date: Option<chrono::NaiveDate>,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Inflation index rate (periodic rate)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InflationIndexRate {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub index_id: Uuid,
    pub period_start: chrono::NaiveDate,
    pub period_end: chrono::NaiveDate,
    pub index_value: String,
    pub cumulative_factor: String,
    pub period_factor: String,
    pub source: Option<String>,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Inflation adjustment run
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InflationAdjustmentRun {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub run_number: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub index_id: Uuid,
    pub ledger_id: Option<Uuid>,
    pub from_period: chrono::NaiveDate,
    pub to_period: chrono::NaiveDate,
    pub adjustment_method: String,
    pub total_debit_adjustment: String,
    pub total_credit_adjustment: String,
    pub total_monetary_gain_loss: String,
    pub account_count: i32,
    pub status: String,
    pub submitted_by: Option<Uuid>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub journal_entry_id: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Inflation adjustment line (per account)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InflationAdjustmentLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub run_id: Uuid,
    pub line_number: i32,
    pub account_code: String,
    pub account_name: Option<String>,
    pub account_type: Option<String>,
    pub balance_type: Option<String>,
    pub original_balance: String,
    pub restated_balance: String,
    pub adjustment_amount: String,
    pub inflation_factor: String,
    pub acquisition_date: Option<chrono::NaiveDate>,
    pub gain_loss_amount: String,
    pub gain_loss_account: Option<String>,
    pub currency_code: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Inflation adjustment dashboard summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InflationDashboardSummary {
    pub total_indices: i32,
    pub hyperinflationary_indices: i32,
    pub total_runs: i32,
    pub draft_runs: i32,
    pub completed_runs: i32,
    pub total_adjustments: String,
    pub total_gain_loss: String,
}

// ═══════════════════════════════════════════════════════════════════════════
// Impairment Management (IAS 36 / ASC 360)
// Oracle Fusion equivalent: Financials > Fixed Assets > Impairment Management
// ═══════════════════════════════════════════════════════════════════════════

/// Impairment indicator
/// Defines triggers that may indicate asset impairment.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImpairmentIndicator {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub indicator_type: String,
    pub severity: String,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Impairment test
/// Tests whether an asset's carrying amount exceeds its recoverable amount.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImpairmentTest {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub test_number: String,
    pub name: String,
    pub description: Option<String>,
    pub test_type: String,
    pub test_method: String,
    pub test_date: chrono::NaiveDate,
    pub reporting_period: Option<String>,
    pub indicator_id: Option<Uuid>,
    pub carrying_amount: String,
    pub recoverable_amount: String,
    pub impairment_loss: String,
    pub reversal_amount: Option<String>,
    pub status: String,
    pub impairment_account: Option<String>,
    pub reversal_account: Option<String>,
    pub asset_id: Option<Uuid>,
    pub cgu_id: Option<Uuid>,
    pub discount_rate: Option<String>,
    pub growth_rate: Option<String>,
    pub terminal_value: Option<String>,
    pub metadata: serde_json::Value,
    pub submitted_by: Option<Uuid>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Impairment cash flow projection
/// Used for value-in-use calculation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImpairmentCashFlow {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub test_id: Uuid,
    pub period_year: i32,
    pub period_number: i32,
    pub description: Option<String>,
    pub cash_inflow: String,
    pub cash_outflow: String,
    pub net_cash_flow: String,
    pub discount_factor: String,
    pub present_value: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Impairment test asset
/// Links an impairment test to a specific asset.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImpairmentTestAsset {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub test_id: Uuid,
    pub asset_id: Uuid,
    pub asset_number: Option<String>,
    pub asset_name: Option<String>,
    pub asset_category: Option<String>,
    pub carrying_amount: String,
    pub recoverable_amount: String,
    pub impairment_loss: String,
    pub status: String,
    pub impairment_date: Option<chrono::NaiveDate>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Impairment management dashboard summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImpairmentDashboardSummary {
    pub total_indicators: i32,
    pub active_indicators: i32,
    pub total_tests: i32,
    pub pending_tests: i32,
    pub completed_tests: i32,
    pub total_impairment_loss: String,
    pub total_reversals: String,
    pub assets_under_review: i32,
}

// ═══════════════════════════════════════════════════════════════════════════
// Bank Account Transfer (Internal Fund Transfers)
// Oracle Fusion equivalent: Financials > Cash Management > Bank Transfers
// ═══════════════════════════════════════════════════════════════════════════

/// Bank transfer type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BankTransferType {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub settlement_method: String,
    pub requires_approval: bool,
    pub approval_threshold: Option<String>,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Bank account transfer
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BankAccountTransfer {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub transfer_number: String,
    pub transfer_type_id: Option<Uuid>,
    pub from_bank_account_id: Uuid,
    pub from_bank_account_number: Option<String>,
    pub from_bank_name: Option<String>,
    pub to_bank_account_id: Uuid,
    pub to_bank_account_number: Option<String>,
    pub to_bank_name: Option<String>,
    pub amount: String,
    pub currency_code: String,
    pub exchange_rate: Option<String>,
    pub from_currency: Option<String>,
    pub to_currency: Option<String>,
    pub transferred_amount: Option<String>,
    pub transfer_date: chrono::NaiveDate,
    pub value_date: Option<chrono::NaiveDate>,
    pub settlement_date: Option<chrono::NaiveDate>,
    pub reference_number: Option<String>,
    pub description: Option<String>,
    pub purpose: Option<String>,
    pub status: String,
    pub priority: String,
    pub from_journal_id: Option<Uuid>,
    pub to_journal_id: Option<Uuid>,
    pub submitted_by: Option<Uuid>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub completed_by: Option<Uuid>,
    pub completed_at: Option<DateTime<Utc>>,
    pub cancelled_by: Option<Uuid>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub cancellation_reason: Option<String>,
    pub failure_reason: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Bank transfer dashboard summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BankTransferDashboardSummary {
    pub total_transfers: i32,
    pub pending_transfers: i32,
    pub completed_transfers: i32,
    pub cancelled_transfers: i32,
    pub total_amount_transferred: String,
    pub average_transfer_amount: String,
    pub total_transfer_types: i32,
}

// ═══════════════════════════════════════════════════════════════════════════
// Tax Reporting & Filing
// Oracle Fusion equivalent: Financials > Tax > Tax Reporting
// ═══════════════════════════════════════════════════════════════════════════

/// Tax return template
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxReturnTemplate {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub tax_type: String,
    pub jurisdiction_code: Option<String>,
    pub filing_frequency: String,
    pub return_form_number: Option<String>,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Tax return template line
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxReturnTemplateLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub template_id: Uuid,
    pub line_number: i32,
    pub box_code: String,
    pub box_name: String,
    pub description: Option<String>,
    pub line_type: String,
    pub calculation_formula: Option<String>,
    pub account_code_filter: Option<String>,
    pub tax_rate_code_filter: Option<String>,
    pub is_debit: bool,
    pub display_order: i32,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Tax return
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxReturn {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub return_number: String,
    pub template_id: Uuid,
    pub template_name: Option<String>,
    pub tax_type: Option<String>,
    pub jurisdiction_code: Option<String>,
    pub filing_period_start: chrono::NaiveDate,
    pub filing_period_end: chrono::NaiveDate,
    pub filing_due_date: Option<chrono::NaiveDate>,
    pub total_tax_amount: String,
    pub total_taxable_amount: String,
    pub total_exempt_amount: String,
    pub total_input_tax: String,
    pub total_output_tax: String,
    pub net_tax_due: String,
    pub penalty_amount: String,
    pub interest_amount: String,
    pub total_amount_due: String,
    pub payment_amount: String,
    pub refund_amount: String,
    pub status: String,
    pub filing_method: Option<String>,
    pub filing_reference: Option<String>,
    pub filing_date: Option<chrono::NaiveDate>,
    pub payment_date: Option<chrono::NaiveDate>,
    pub payment_reference: Option<String>,
    pub amendment_reason: Option<String>,
    pub notes: Option<String>,
    pub submitted_by: Option<Uuid>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub filed_by: Option<Uuid>,
    pub filed_at: Option<DateTime<Utc>>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Tax return line value
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxReturnLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub tax_return_id: Uuid,
    pub template_line_id: Option<Uuid>,
    pub line_number: i32,
    pub box_code: String,
    pub box_name: Option<String>,
    pub line_type: String,
    pub amount: String,
    pub calculated_amount: String,
    pub override_amount: Option<String>,
    pub final_amount: String,
    pub description: Option<String>,
    pub source_count: i32,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Tax filing calendar entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxFilingCalendarEntry {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub template_id: Uuid,
    pub period_start: chrono::NaiveDate,
    pub period_end: chrono::NaiveDate,
    pub due_date: chrono::NaiveDate,
    pub filing_status: String,
    pub return_id: Option<Uuid>,
    pub extension_filed: bool,
    pub extension_due_date: Option<chrono::NaiveDate>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Tax reporting dashboard summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxReportingDashboardSummary {
    pub total_templates: i32,
    pub active_templates: i32,
    pub total_returns: i32,
    pub draft_returns: i32,
    pub filed_returns: i32,
    pub overdue_returns: i32,
    pub total_tax_paid: String,
    pub total_tax_due: String,
    pub total_refunds: String,
    pub upcoming_filings: i32,
}
