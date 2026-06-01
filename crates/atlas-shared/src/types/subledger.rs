use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
// ============================================================================
// Subledger Accounting Types
// Oracle Fusion: Financials > General Ledger > Subledger Accounting
// ============================================================================

/// Accounting Method
/// Defines how a subledger transaction type is accounted for.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountingMethod {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    /// Application: 'payables', 'receivables', 'expenses', 'assets', 'projects'
    pub application: String,
    /// Transaction type within the application
    pub transaction_type: String,
    /// Event class: 'create', 'update', 'cancel', 'reverse'
    pub event_class: String,
    pub auto_accounting: bool,
    pub allow_manual_entries: bool,
    pub apply_rounding: bool,
    pub rounding_account_code: Option<String>,
    pub rounding_threshold: String,
    pub require_balancing: bool,
    pub intercompany_balancing_account: Option<String>,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Accounting Method Create/Update Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountingMethodRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub application: String,
    pub transaction_type: String,
    pub event_class: Option<String>,
    pub auto_accounting: Option<bool>,
    pub allow_manual_entries: Option<bool>,
    pub apply_rounding: Option<bool>,
    pub rounding_account_code: Option<String>,
    pub rounding_threshold: Option<String>,
    pub require_balancing: Option<bool>,
    pub intercompany_balancing_account: Option<String>,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
}

/// Accounting Derivation Rule
/// Rules for deriving account codes from transaction attributes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountingDerivationRule {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub accounting_method_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    /// Line type: 'debit', 'credit', 'tax', 'discount'
    pub line_type: String,
    /// Priority (lower = higher priority)
    pub priority: i32,
    /// Conditions for rule activation
    pub conditions: serde_json::Value,
    /// Source field from the transaction
    pub source_field: Option<String>,
    /// Derivation type: 'constant', 'lookup', 'formula'
    pub derivation_type: String,
    /// Fixed account code (for 'constant' type)
    pub fixed_account_code: Option<String>,
    /// Lookup table (for 'lookup' type)
    pub account_derivation_lookup: serde_json::Value,
    /// Formula expression (for 'formula' type)
    pub formula_expression: Option<String>,
    pub sequence: i32,
    pub is_active: bool,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Subledger Journal Entry
/// The accounting representation of a subledger transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubledgerJournalEntry {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Source subledger: 'payables', 'receivables', 'expenses', etc.
    pub source_application: String,
    /// Transaction type: 'invoice', 'payment', etc.
    pub source_transaction_type: String,
    /// ID of the source transaction
    pub source_transaction_id: Uuid,
    pub source_transaction_number: Option<String>,
    /// Accounting method applied
    pub accounting_method_id: Option<Uuid>,
    /// Journal entry number (auto-generated)
    pub entry_number: String,
    pub description: Option<String>,
    pub reference_number: Option<String>,
    /// GL date
    pub accounting_date: chrono::NaiveDate,
    pub period_name: Option<String>,
    /// Currency info
    pub currency_code: String,
    pub entered_currency_code: String,
    pub currency_conversion_date: Option<chrono::NaiveDate>,
    pub currency_conversion_type: Option<String>,
    pub currency_conversion_rate: Option<String>,
    /// Totals
    pub total_debit: String,
    pub total_credit: String,
    pub entered_debit: String,
    pub entered_credit: String,
    /// Status: 'draft', 'accounted', 'posted', 'transferred', 'reversed', 'error'
    pub status: String,
    pub error_message: Option<String>,
    /// Balancing
    pub balancing_segment: Option<String>,
    pub is_balanced: bool,
    /// GL transfer tracking
    pub gl_transfer_status: String,
    pub gl_transfer_date: Option<DateTime<Utc>>,
    pub gl_journal_entry_id: Option<Uuid>,
    /// Reversal tracking
    pub is_reversal: bool,
    pub reversal_of_id: Option<Uuid>,
    pub reversal_reason: Option<String>,
    /// Audit
    pub created_by: Option<Uuid>,
    pub posted_by: Option<Uuid>,
    pub accounted_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Subledger Journal Line
/// Individual debit/credit line within a journal entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubledgerJournalLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub journal_entry_id: Uuid,
    pub line_number: i32,
    /// Line type: 'debit', 'credit', 'tax', 'discount', 'rounding'
    pub line_type: String,
    /// Account code
    pub account_code: String,
    pub account_description: Option<String>,
    /// Derivation rule that produced this line
    pub derivation_rule_id: Option<Uuid>,
    /// Amounts
    pub entered_amount: String,
    pub accounted_amount: String,
    /// Currency
    pub currency_code: String,
    pub conversion_date: Option<chrono::NaiveDate>,
    pub conversion_rate: Option<String>,
    /// Descriptive flexfield attributes
    pub attribute_category: Option<String>,
    pub attribute1: Option<String>,
    pub attribute2: Option<String>,
    pub attribute3: Option<String>,
    pub attribute4: Option<String>,
    pub attribute5: Option<String>,
    pub attribute6: Option<String>,
    pub attribute7: Option<String>,
    pub attribute8: Option<String>,
    pub attribute9: Option<String>,
    pub attribute10: Option<String>,
    /// Tax
    pub tax_code: Option<String>,
    pub tax_rate: Option<String>,
    pub tax_amount: Option<String>,
    /// Source reference
    pub source_line_id: Option<Uuid>,
    pub source_line_type: Option<String>,
    /// Reversal
    pub is_reversal_line: bool,
    pub reversal_of_line_id: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Subledger Accounting Event
/// Audit trail of accounting events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlaEvent {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub event_number: String,
    /// Event type: 'creation', 'modification', 'cancellation', 'reversal', 'posting', 'transfer'
    pub event_type: String,
    pub source_application: String,
    pub source_transaction_type: String,
    pub source_transaction_id: Uuid,
    pub journal_entry_id: Option<Uuid>,
    pub event_date: chrono::NaiveDate,
    /// Status: 'processed', 'error', 'skipped'
    pub event_status: String,
    pub description: Option<String>,
    pub error_message: Option<String>,
    pub processed_by: Option<Uuid>,
    pub processed_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// GL Transfer Log
/// Tracks transfers of subledger entries to the General Ledger.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlTransferLog {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub transfer_number: String,
    pub transfer_date: DateTime<Utc>,
    pub from_period: Option<String>,
    /// Status: 'pending', '`in_progress`', 'completed', 'failed', 'reversed'
    pub status: String,
    pub error_message: Option<String>,
    pub total_entries: i32,
    pub total_debit: String,
    pub total_credit: String,
    pub included_applications: serde_json::Value,
    pub transferred_by: Option<Uuid>,
    pub completed_at: Option<DateTime<Utc>>,
    pub entries: serde_json::Value,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Subledger Accounting Dashboard Summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlaDashboardSummary {
    pub total_entries: i32,
    pub draft_count: i32,
    pub accounted_count: i32,
    pub posted_count: i32,
    pub transferred_count: i32,
    pub reversed_count: i32,
    pub error_count: i32,
    pub total_debit: String,
    pub total_credit: String,
    pub entries_by_application: serde_json::Value,
    pub entries_by_status: serde_json::Value,
    pub pending_transfer_count: i32,
    pub unbalanced_count: i32,
}

// ════════════════════════════════════════════════════════════════════════════════
// Encumbrance Management (Oracle Fusion GL > Encumbrance Management)
// ════════════════════════════════════════════════════════════════════════════════
//
// Tracks financial commitments before actual expenditure:
// - Requisitions → Preliminary encumbrances
// - Purchase Orders → Encumbrances (commitments)
// - Invoices → Partial/Full liquidation of encumbrances
// - Contracts → Long-term commitment tracking
//
// Supports budgetary control by reserving funds against budgets.

/// Encumbrance Type definition
/// Defines the types of commitments an organization tracks.
/// Oracle Fusion equivalent: GL > Encumbrance Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncumbranceType {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Unique code (e.g., "`PURCHASE_ORDER`", "REQUISITION", "CONTRACT")
    pub code: String,
    /// Human-readable name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Category: "commitment", "obligation", "preliminary"
    pub category: String,
    /// Whether this encumbrance type is enabled
    pub is_enabled: bool,
    /// Whether this type can be manually entered
    pub allow_manual_entry: bool,
    /// Default encumbrance account code for this type
    pub default_encumbrance_account_code: Option<String>,
    /// Whether year-end carry-forward is allowed
    pub allow_carry_forward: bool,
    /// Priority for budget control (lower = checked first)
    pub priority: i32,
    /// Metadata
    pub metadata: serde_json::Value,
    /// Audit
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Encumbrance Entry header
/// Represents a commitment transaction (e.g., a purchase order creates an encumbrance).
/// Oracle Fusion equivalent: GL > Encumbrance Entries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncumbranceEntry {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Auto-generated entry number (e.g., "ENC-2024-00001")
    pub entry_number: String,
    /// Encumbrance type ID
    pub encumbrance_type_id: Uuid,
    /// Encumbrance type code (denormalized)
    pub encumbrance_type_code: String,
    /// Source document type (e.g., "`purchase_order`", "requisition", "contract")
    pub source_type: Option<String>,
    /// Source document ID
    pub source_id: Option<Uuid>,
    /// Source document number (e.g., PO-00123)
    pub source_number: Option<String>,
    /// Description/purpose of the encumbrance
    pub description: Option<String>,
    /// Encumbrance date (when the commitment was made)
    pub encumbrance_date: chrono::NaiveDate,
    /// Original encumbrance amount
    pub original_amount: String,
    /// Current remaining encumbrance amount
    pub current_amount: String,
    /// Amount that has been liquidated (matched to actual expenditure)
    pub liquidated_amount: String,
    /// Amount that has been manually adjusted
    pub adjusted_amount: String,
    /// Currency code
    pub currency_code: String,
    /// Status: "draft", "active", "`partially_liquidated`", "`fully_liquidated`", "cancelled", "expired"
    pub status: String,
    /// Budget period or fiscal year reference
    pub fiscal_year: Option<i32>,
    /// Period name
    pub period_name: Option<String>,
    /// Whether this entry has been carried forward from a prior year
    pub is_carry_forward: bool,
    /// Original entry ID if this is a carry-forward
    pub carried_forward_from_id: Option<Uuid>,
    /// Expiry date for time-limited commitments
    pub expiry_date: Option<chrono::NaiveDate>,
    /// Reference to the associated budget line (if applicable)
    pub budget_line_id: Option<Uuid>,
    /// Metadata
    pub metadata: serde_json::Value,
    /// Audit
    pub created_by: Option<Uuid>,
    pub approved_by: Option<Uuid>,
    pub cancelled_by: Option<Uuid>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub cancellation_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Encumbrance Line
/// Individual line within an encumbrance entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncumbranceLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub entry_id: Uuid,
    /// Line number within the entry
    pub line_number: i32,
    /// Account code being encumbered
    pub account_code: String,
    /// Account description
    pub account_description: Option<String>,
    /// Department ID
    pub department_id: Option<Uuid>,
    /// Department name
    pub department_name: Option<String>,
    /// Project ID
    pub project_id: Option<Uuid>,
    /// Project name
    pub project_name: Option<String>,
    /// Cost center
    pub cost_center: Option<String>,
    /// Original encumbered amount
    pub original_amount: String,
    /// Current remaining amount
    pub current_amount: String,
    /// Liquidated amount
    pub liquidated_amount: String,
    /// Encumbrance account code (the account tracking the commitment)
    pub encumbrance_account_code: Option<String>,
    /// Source line reference
    pub source_line_id: Option<Uuid>,
    /// Descriptive flexfields
    pub attribute_category: Option<String>,
    pub attribute1: Option<String>,
    pub attribute2: Option<String>,
    pub attribute3: Option<String>,
    /// Metadata
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Encumbrance Liquidation
/// Records the reduction of an encumbrance when actual expenditure occurs
/// (e.g., when an invoice is matched to a purchase order).
/// Oracle Fusion equivalent: GL > Encumbrance Liquidation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncumbranceLiquidation {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Auto-generated liquidation number
    pub liquidation_number: String,
    /// The encumbrance entry being liquidated
    pub encumbrance_entry_id: Uuid,
    /// Specific line being liquidated (None = header-level liquidation)
    pub encumbrance_line_id: Option<Uuid>,
    /// Liquidation type: "full", "partial", "final"
    pub liquidation_type: String,
    /// Amount being liquidated
    pub liquidation_amount: String,
    /// Source document type (e.g., "invoice", "payment", "`journal_entry`")
    pub source_type: Option<String>,
    /// Source document ID
    pub source_id: Option<Uuid>,
    /// Source document number
    pub source_number: Option<String>,
    /// Description
    pub description: Option<String>,
    /// Liquidation date
    pub liquidation_date: chrono::NaiveDate,
    /// Status: "draft", "processed", "reversed"
    pub status: String,
    /// Reversal reference
    pub reversed_by_id: Option<Uuid>,
    pub reversal_reason: Option<String>,
    /// Metadata
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Encumbrance Year-End Processing
/// Tracks carry-forward of open encumbrances to the next fiscal year.
/// Oracle Fusion equivalent: GL > Encumbrance Year-End Processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncumbranceCarryForward {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Processing batch number
    pub batch_number: String,
    /// Source fiscal year
    pub from_fiscal_year: i32,
    /// Target fiscal year
    pub to_fiscal_year: i32,
    /// Status: "draft", "processing", "completed", "reversed"
    pub status: String,
    /// Total number of entries carried forward
    pub entry_count: i32,
    /// Total amount carried forward
    pub total_amount: String,
    /// Description/notes
    pub description: Option<String>,
    /// Metadata
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub processed_by: Option<Uuid>,
    pub processed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Encumbrance Dashboard Summary
/// Provides an overview of encumbrance activity for budgetary control.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncumbranceSummary {
    /// Total active encumbrance amount
    pub total_active_amount: String,
    /// Total liquidated amount (period)
    pub total_liquidated_amount: String,
    /// Total adjusted amount (period)
    pub total_adjusted_amount: String,
    /// Count of active entries
    pub active_entry_count: i32,
    /// Count of entries by status
    pub entries_by_status: serde_json::Value,
    /// Count of entries by type
    pub entries_by_type: serde_json::Value,
    /// Breakdown by account code
    pub by_account: serde_json::Value,
    /// Breakdown by department
    pub by_department: serde_json::Value,
    /// Expiring soon count (next 30 days)
    pub expiring_soon_count: i32,
    /// Expiring soon amount
    pub expiring_soon_amount: String,
}

// Cash Position & Cash Forecasting (Oracle Fusion Treasury Management)
// ════════════════════════════════════════════════════════════════════════════════
//
// Oracle Fusion Cloud ERP Treasury Management provides:
// - Cash Positions: Real-time view of cash balances across bank accounts
// - Cash Forecasts: Projected cash inflows and outflows over configurable periods
// - Forecast Sources: Configurable sources (AP, AR, Payroll, Purchasing, etc.)
// - Forecast Templates: Define forecast columns, time buckets, and aggregation
//
// Oracle Fusion equivalent: Financials > Treasury > Cash Management

/// Cash Position
/// Represents a snapshot of cash balances for a bank account at a point in time.
/// Oracle Fusion equivalent: Treasury > Cash Position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashPosition {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Bank account ID (references reconciliation bank account)
    pub bank_account_id: Uuid,
    /// Bank account number (denormalized)
    pub account_number: String,
    /// Bank account name (denormalized)
    pub account_name: String,
    /// Currency code
    pub currency_code: String,
    /// Ledger (book) balance as of position date
    pub book_balance: String,
    /// Available balance (book balance minus holds/outstanding)
    pub available_balance: String,
    /// Float (deposits in transit not yet cleared)
    pub float_amount: String,
    /// One-day float (clearing next business day)
    pub one_day_float: String,
    /// Two-or-more day float
    pub two_day_float: String,
    /// Position date
    pub position_date: chrono::NaiveDate,
    /// Rolling average balance (e.g., 30-day)
    pub average_balance: Option<String>,
    /// Prior day closing balance
    pub prior_day_balance: Option<String>,
    /// Projected inflows for today
    pub projected_inflows: String,
    /// Projected outflows for today
    pub projected_outflows: String,
    /// Net projected change
    pub projected_net: String,
    /// Whether this position is reconciled
    pub is_reconciled: bool,
    /// Metadata
    pub metadata: serde_json::Value,
    /// Audit
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Cash Position Summary
/// Aggregated cash position across all bank accounts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashPositionSummary {
    pub organization_id: Uuid,
    /// Position date
    pub position_date: chrono::NaiveDate,
    /// Total book balance across all accounts
    pub total_book_balance: String,
    /// Total available balance
    pub total_available_balance: String,
    /// Total float
    pub total_float: String,
    /// Total projected inflows
    pub total_projected_inflows: String,
    /// Total projected outflows
    pub total_projected_outflows: String,
    /// Total net projected change
    pub total_projected_net: String,
    /// Number of bank accounts included
    pub account_count: i32,
    /// Breakdown by currency
    pub by_currency: serde_json::Value,
    /// Breakdown by bank account
    pub by_account: serde_json::Value,
}

/// Forecast Template
/// Defines the structure of a cash forecast (columns, time periods, sources).
/// Oracle Fusion equivalent: Treasury > Cash Forecast > Templates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashForecastTemplate {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Unique template code
    pub code: String,
    /// Template name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Time bucket type: "daily", "weekly", "monthly"
    pub bucket_type: String,
    /// Number of periods to forecast
    pub number_of_periods: i32,
    /// From-date offset (e.g., 0 = today, -7 = a week ago)
    pub start_offset_days: i32,
    /// Whether this is the default template
    pub is_default: bool,
    /// Whether the template is active
    pub is_active: bool,
    /// Template columns definition (JSON array of column configs)
    pub columns: serde_json::Value,
    /// Metadata
    pub metadata: serde_json::Value,
    /// Audit
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Forecast Source
/// Defines a data source that feeds into cash forecasts.
/// Oracle Fusion equivalent: Treasury > Cash Forecast > Sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashForecastSource {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Template ID this source belongs to
    pub template_id: Uuid,
    /// Source code (unique within template)
    pub code: String,
    /// Source name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Source type: "`accounts_payable`", "`accounts_receivable`", "payroll",
    ///             "purchasing", "manual", "budget", "intercompany"
    pub source_type: String,
    /// Cash flow direction: "inflow", "outflow", "both"
    pub cash_flow_direction: String,
    /// Whether this source is for actuals or forecasts
    pub is_actual: bool,
    /// Priority for display ordering
    pub display_order: i32,
    /// Whether the source is active
    pub is_active: bool,
    /// Lead time in days (expected delay between transaction and cash impact)
    pub lead_time_days: i32,
    /// Payment terms reference or description
    pub payment_terms_reference: Option<String>,
    /// GL account code filter (optional)
    pub account_code_filter: Option<String>,
    /// Metadata
    pub metadata: serde_json::Value,
    /// Audit
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Cash Forecast
/// A specific forecast run generated from a template.
/// Oracle Fusion equivalent: Treasury > Cash Forecast > Forecasts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashForecast {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Forecast number (auto-generated)
    pub forecast_number: String,
    /// Template used to generate this forecast
    pub template_id: Uuid,
    /// Template name (denormalized)
    pub template_name: String,
    /// Forecast name/description
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Start date of the forecast period
    pub start_date: chrono::NaiveDate,
    /// End date of the forecast period
    pub end_date: chrono::NaiveDate,
    /// Opening balance (actual balance at start date)
    pub opening_balance: String,
    /// Total projected inflows
    pub total_inflows: String,
    /// Total projected outflows
    pub total_outflows: String,
    /// Net cash flow
    pub net_cash_flow: String,
    /// Closing projected balance
    pub closing_balance: String,
    /// Minimum balance encountered during the period
    pub minimum_balance: String,
    /// Maximum balance encountered during the period
    pub maximum_balance: String,
    /// Deficit periods (where balance falls below threshold)
    pub deficit_count: i32,
    /// Surplus periods
    pub surplus_count: i32,
    /// Status: "draft", "generated", "approved", "superseded"
    pub status: String,
    /// Whether this is the latest forecast for this template
    pub is_latest: bool,
    /// Metadata
    pub metadata: serde_json::Value,
    /// Audit
    pub created_by: Option<Uuid>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Cash Forecast Line
/// Individual line within a forecast, representing a source for a time period.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashForecastLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Forecast header ID
    pub forecast_id: Uuid,
    /// Forecast source ID
    pub source_id: Uuid,
    /// Source name (denormalized)
    pub source_name: String,
    /// Source type (denormalized)
    pub source_type: String,
    /// Cash flow direction
    pub cash_flow_direction: String,
    /// Period start date
    pub period_start_date: chrono::NaiveDate,
    /// Period end date
    pub period_end_date: chrono::NaiveDate,
    /// Period label (e.g., "Week 3", "Mar 2025")
    pub period_label: String,
    /// Period sequence number
    pub period_sequence: i32,
    /// Amount for this period
    pub amount: String,
    /// Running cumulative amount
    pub cumulative_amount: String,
    /// Whether this is actual data (vs projected)
    pub is_actual: bool,
    /// Currency code
    pub currency_code: String,
    /// Number of underlying transactions
    pub transaction_count: i32,
    /// Metadata
    pub metadata: serde_json::Value,
    /// Audit
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Cash Forecast Summary
/// Summary view of a cash forecast for dashboard display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashForecastSummary {
    /// Template ID
    pub template_id: Uuid,
    /// Template name
    pub template_name: String,
    /// Forecast ID
    pub forecast_id: Uuid,
    /// Forecast number
    pub forecast_number: String,
    /// Start date
    pub start_date: chrono::NaiveDate,
    /// End date
    pub end_date: chrono::NaiveDate,
    /// Opening balance
    pub opening_balance: String,
    /// Total inflows
    pub total_inflows: String,
    /// Total outflows
    pub total_outflows: String,
    /// Net cash flow
    pub net_cash_flow: String,
    /// Closing balance
    pub closing_balance: String,
    /// Minimum balance
    pub minimum_balance: String,
    /// Deficit count
    pub deficit_count: i32,
    /// Surplus count
    pub surplus_count: i32,
    /// Inflows by source (for chart)
    pub inflows_by_source: serde_json::Value,
    /// Outflows by source (for chart)
    pub outflows_by_source: serde_json::Value,
    /// Balance trend (array of period-end balances)
    pub balance_trend: serde_json::Value,
}

// ════════════════════════════════════════════════════════════════════════════════
// Procurement Sourcing Management (Oracle Fusion SCM > Procurement > Sourcing)
// ════════════════════════════════════════════════════════════════════════════════
//
// Oracle Fusion Cloud ERP Procurement Sourcing provides:
// - Sourcing Events: RFQs (Request for Quote), RFPs (Request for Proposal), RFI
// - Supplier Responses: Bids/quotation submissions with line-level pricing
// - Scoring & Evaluation: Weighted scoring criteria, team evaluation
// - Award: Best-value analysis, split awards, multi-supplier awards
// - Templates: Reusable sourcing templates for recurring procurement
// - Negotiation: Multi-round negotiation with suppliers
//
// Oracle Fusion equivalent: Procurement > Sourcing > Negotiations

/// Sourcing Event (Negotiation)
/// Represents an RFQ, RFP, or other sourcing event sent to suppliers.
/// Oracle Fusion equivalent: Procurement > Sourcing > Negotiations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourcingEvent {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Auto-generated event number (e.g., "SE-2024-00001")
    pub event_number: String,
    /// Event title/subject
    pub title: String,
    /// Description of what is being sourced
    pub description: Option<String>,
    /// Event type: "rfq" (Request for Quote), "rfp" (Request for Proposal),
    /// "rfi" (Request for Information), "auction" (Reverse Auction)
    pub event_type: String,
    /// Status: "draft", "published", "`response_open`", "evaluation", "awarded",
    ///         "cancelled", "closed"
    pub status: String,
    /// Style: "sealed" (blind bidding), "open" (visible bids), "`reverse_auction`"
    pub style: String,
    /// Deadline for supplier responses
    pub response_deadline: chrono::NaiveDate,
    /// When the event was published
    pub published_at: Option<DateTime<Utc>>,
    /// When the event was closed/awarded
    pub closed_at: Option<DateTime<Utc>>,
    /// Currency for all pricing
    pub currency_code: String,
    /// Sourcing template reference
    pub template_id: Option<Uuid>,
    /// Template name (denormalized)
    pub template_name: Option<String>,
    /// Evaluation team lead
    pub evaluation_lead_id: Option<Uuid>,
    pub evaluation_lead_name: Option<String>,
    /// Scoring method: "weighted", "`pass_fail`", "manual", "`lowest_price`"
    pub scoring_method: String,
    /// Whether supplier responses are visible to other suppliers
    pub are_bids_visible: bool,
    /// Allow suppliers to see their rank
    pub allow_supplier_rank_visibility: bool,
    /// Contact for inquiries
    pub contact_person_id: Option<Uuid>,
    pub contact_person_name: Option<String>,
    /// Terms and conditions
    pub terms_and_conditions: Option<String>,
    /// Attachment references
    pub attachments: serde_json::Value,
    /// Number of invited suppliers
    pub invited_supplier_count: i32,
    /// Number of suppliers who responded
    pub response_count: i32,
    /// Award summary
    pub award_summary: serde_json::Value,
    /// Metadata
    pub metadata: serde_json::Value,
    /// Audit
    pub created_by: Option<Uuid>,
    pub cancelled_by: Option<Uuid>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub cancellation_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Sourcing Event Line
/// Individual items or services being sourced within an event.
/// Oracle Fusion equivalent: Negotiation Lines
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourcingEventLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Parent sourcing event
    pub event_id: Uuid,
    /// Line number within the event
    pub line_number: i32,
    /// Item / service description
    pub description: String,
    /// Item number / SKU reference
    pub item_number: Option<String>,
    /// Item category
    pub category: Option<String>,
    /// Quantity being sourced
    pub quantity: String,
    /// Unit of measure (e.g., "EA", "KG", "LOT")
    pub uom: String,
    /// Target / estimated unit price
    pub target_price: Option<String>,
    /// Target / estimated total price
    pub target_total: Option<String>,
    /// Required delivery date
    pub need_by_date: Option<chrono::NaiveDate>,
    /// Ship-to location
    pub ship_to: Option<String>,
    /// Technical specifications
    pub specifications: Option<serde_json::Value>,
    /// Whether partial quantity bids are allowed
    pub allow_partial_quantity: bool,
    /// Minimum award quantity (for split awards)
    pub min_award_quantity: Option<String>,
    /// Line status: "open", "awarded", "cancelled"
    pub status: String,
    /// Awarded supplier ID (after award)
    pub awarded_supplier_id: Option<Uuid>,
    /// Awarded supplier name (denormalized)
    pub awarded_supplier_name: Option<String>,
    /// Awarded unit price (after award)
    pub awarded_price: Option<String>,
    /// Awarded quantity (for split awards)
    pub awarded_quantity: Option<String>,
    /// Metadata
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Sourcing Event Invite
/// Tracks which suppliers are invited to participate in a sourcing event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourcingInvite {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Sourcing event reference
    pub event_id: Uuid,
    /// Supplier reference
    pub supplier_id: Uuid,
    /// Supplier name (denormalized)
    pub supplier_name: Option<String>,
    /// Supplier email for notifications
    pub supplier_email: Option<String>,
    /// Whether the supplier has viewed the event
    pub is_viewed: bool,
    pub viewed_at: Option<DateTime<Utc>>,
    /// Whether the supplier has responded
    pub has_responded: bool,
    pub responded_at: Option<DateTime<Utc>>,
    /// Status: "invited", "viewed", "responded", "declined", "disqualified"
    pub status: String,
    /// Metadata
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Supplier Response (Bid/Quotation)
/// A supplier's response to a sourcing event with line-level pricing.
/// Oracle Fusion equivalent: Supplier Response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupplierResponse {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Sourcing event reference
    pub event_id: Uuid,
    /// Response number (auto-generated)
    pub response_number: String,
    /// Supplier reference
    pub supplier_id: Uuid,
    /// Supplier name (denormalized)
    pub supplier_name: Option<String>,
    /// Status: "draft", "submitted", "`under_review`", "shortlisted", "rejected",
    ///         "awarded", "disqualified"
    pub status: String,
    /// Total bid amount (sum of all line amounts)
    pub total_amount: String,
    /// Total score (after evaluation)
    pub total_score: Option<String>,
    /// Rank among all responses (after evaluation)
    pub rank: Option<i32>,
    /// Whether this response meets all requirements
    pub is_compliant: Option<bool>,
    /// Supplier notes / cover letter
    pub cover_letter: Option<String>,
    /// Validity date for the bid
    pub valid_until: Option<chrono::NaiveDate>,
    /// Payment terms offered
    pub payment_terms: Option<String>,
    /// Delivery lead time in days
    pub lead_time_days: Option<i32>,
    /// Warranty offered (months)
    pub warranty_months: Option<i32>,
    /// Attachment references
    pub attachments: serde_json::Value,
    /// Evaluation notes
    pub evaluation_notes: Option<String>,
    /// Submitted at
    pub submitted_at: Option<DateTime<Utc>>,
    /// Metadata
    pub metadata: serde_json::Value,
    /// Audit
    pub created_by: Option<Uuid>,
    pub evaluated_by: Option<Uuid>,
    pub evaluated_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Supplier Response Line
/// Individual line pricing within a supplier's response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupplierResponseLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Response header reference
    pub response_id: Uuid,
    /// Sourcing event line reference
    pub event_line_id: Uuid,
    /// Line number
    pub line_number: i32,
    /// Quoted unit price
    pub unit_price: String,
    /// Quoted quantity
    pub quantity: String,
    /// Total line amount (`unit_price` × quantity)
    pub line_amount: String,
    /// Discount percentage offered
    pub discount_percent: Option<String>,
    /// Effective price after discount
    pub effective_price: Option<String>,
    /// Promised delivery date
    pub promised_delivery_date: Option<chrono::NaiveDate>,
    /// Lead time in days
    pub lead_time_days: Option<i32>,
    /// Whether this line meets specifications
    pub is_compliant: Option<bool>,
    /// Line score (after evaluation)
    pub score: Option<String>,
    /// Notes from the supplier
    pub supplier_notes: Option<String>,
    /// Metadata
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Scoring Criterion
/// Defines an evaluation criterion for scoring supplier responses.
/// Oracle Fusion equivalent: Negotiation > Requirements & Scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScoringCriterion {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Sourcing event reference
    pub event_id: Uuid,
    /// Criterion name (e.g., "Price", "Quality", "Delivery Time", "Technical Fit")
    pub name: String,
    /// Description of how to evaluate this criterion
    pub description: Option<String>,
    /// Weight in total score (0-100, all criteria should sum to 100)
    pub weight: String,
    /// Maximum possible score
    pub max_score: String,
    /// Criterion type: "price", "quality", "delivery", "technical", "compliance", "custom"
    pub criterion_type: String,
    /// Display order
    pub display_order: i32,
    /// Whether this is a mandatory (knockout) criterion
    pub is_mandatory: bool,
    /// Metadata
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Score given to a specific response for a specific criterion.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponseScore {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Response reference
    pub response_id: Uuid,
    /// Scoring criterion reference
    pub criterion_id: Uuid,
    /// Score given (0 to `max_score`)
    pub score: String,
    /// Weighted score (score × weight / 100)
    pub weighted_score: String,
    /// Evaluator's notes
    pub notes: Option<String>,
    /// Evaluator
    pub scored_by: Option<Uuid>,
    pub scored_at: Option<DateTime<Utc>>,
    /// Metadata
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Sourcing Award
/// Records the award decision for a sourcing event.
/// Oracle Fusion equivalent: Negotiation > Award
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourcingAward {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Sourcing event reference
    pub event_id: Uuid,
    /// Award number (auto-generated)
    pub award_number: String,
    /// Status: "pending", "approved", "rejected", "cancelled"
    pub status: String,
    /// Award method: "single", "split", "`best_value`", "`lowest_price`"
    pub award_method: String,
    /// Total awarded amount
    pub total_awarded_amount: String,
    /// Reason for award decision
    pub award_rationale: Option<String>,
    /// Awarded by
    pub awarded_by: Option<Uuid>,
    pub awarded_at: Option<DateTime<Utc>>,
    /// Approved by
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    /// Rejected reason
    pub rejected_reason: Option<String>,
    /// Award lines (supplier-level awards)
    pub lines: serde_json::Value,
    /// Metadata
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Sourcing Award Line
/// Individual line in an award (maps event lines to winning suppliers).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourcingAwardLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Award header reference
    pub award_id: Uuid,
    /// Sourcing event line reference
    pub event_line_id: Uuid,
    /// Winning supplier response reference
    pub response_id: Uuid,
    /// Winning supplier
    pub supplier_id: Uuid,
    pub supplier_name: Option<String>,
    /// Awarded quantity
    pub awarded_quantity: String,
    /// Awarded unit price
    pub awarded_unit_price: String,
    /// Awarded total amount
    pub awarded_amount: String,
    /// Metadata
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Sourcing Template
/// Reusable template for creating sourcing events.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourcingTemplate {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Template code
    pub code: String,
    /// Template name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Default event type
    pub default_event_type: String,
    /// Default style
    pub default_style: String,
    /// Default scoring method
    pub default_scoring_method: String,
    /// Default response deadline offset (days from publish)
    pub default_response_deadline_days: i32,
    /// Default currency
    pub currency_code: String,
    /// Whether bids are visible by default
    pub default_bids_visible: bool,
    /// Default terms and conditions
    pub default_terms: Option<String>,
    /// Predefined scoring criteria (JSON array)
    pub default_scoring_criteria: serde_json::Value,
    /// Predefined line templates (JSON array)
    pub default_lines: serde_json::Value,
    /// Is active
    pub is_active: bool,
    /// Metadata
    pub metadata: serde_json::Value,
    /// Audit
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Sourcing Dashboard Summary
/// Overview of sourcing activity for the procurement dashboard.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourcingSummary {
    /// Total active events
    pub active_event_count: i32,
    /// Total draft events
    pub draft_event_count: i32,
    /// Events pending evaluation
    pub pending_evaluation_count: i32,
    /// Events awarded (period)
    pub awarded_event_count: i32,
    /// Total awarded value (period)
    pub total_awarded_value: String,
    /// Average savings percentage vs target price
    pub average_savings_percent: String,
    /// Events by status
    pub events_by_status: serde_json::Value,
    /// Events by type
    pub events_by_type: serde_json::Value,
    /// Top suppliers by award value
    pub top_suppliers: serde_json::Value,
    /// Upcoming deadlines
    pub upcoming_deadlines: serde_json::Value,
}

// ════════════════════════════════════════════════════════════════════════════════
// Lease Accounting (ASC 842 / IFRS 16)
// Oracle Fusion Cloud ERP: Financials > Lease Management
// ════════════════════════════════════════════════════════════════════════════════
//
// Oracle Fusion Cloud ERP Lease Management provides:
// - Lease Contracts: Track lease agreements with classification (operating/finance)
// - Right-of-Use (ROU) Assets: Asset recognition for leased assets
// - Lease Liability: Present value of future lease payments
// - Amortization Schedules: Liability amortization and asset depreciation
// - Lease Payments: Payment schedules with escalation/renewal terms
// - Lease Modifications: Accounting for changes to lease terms
// - Lease Impairment: ROU asset impairment review
// - Lease Termination: Early termination accounting
//
// Oracle Fusion equivalent: Financials > Lease Management

/// Lease Accounting Method
/// ASC 842 distinguishes between operating and finance leases for lessees.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum LeaseClassification {
    #[default]
    Operating,
    Finance,
}

/// Lease contract header
/// Represents a lease agreement between a lessee and lessor.
/// Oracle Fusion equivalent: Lease Management > Lease Contracts
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeaseContract {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Auto-generated lease number (e.g., "LSE-2024-00001")
    pub lease_number: String,
    /// Lease title / description
    pub title: String,
    pub description: Option<String>,
    /// Classification: "operating" or "finance"
    pub classification: String,
    /// Lessor / supplier information
    pub lessor_id: Option<Uuid>,
    pub lessor_name: Option<String>,
    /// Asset being leased
    pub asset_description: Option<String>,
    /// Asset location
    pub location: Option<String>,
    /// Department
    pub department_id: Option<Uuid>,
    pub department_name: Option<String>,
    /// Lease dates
    pub commencement_date: chrono::NaiveDate,
    pub end_date: chrono::NaiveDate,
    /// Lease term in months
    pub lease_term_months: i32,
    /// Whether the lease includes a purchase option
    pub purchase_option_exists: bool,
    /// Whether the purchase option is reasonably certain to be exercised
    pub purchase_option_likely: bool,
    /// Whether there is a renewal option
    pub renewal_option_exists: bool,
    /// Renewal option term in months
    pub renewal_option_months: Option<i32>,
    /// Whether renewal is reasonably certain to be exercised
    pub renewal_option_likely: bool,
    /// Discount rate (incremental borrowing rate)
    pub discount_rate: String,
    /// Currency
    pub currency_code: String,
    /// Payment frequency: "monthly", "quarterly", "annually"
    pub payment_frequency: String,
    /// Annual escalation rate percentage
    pub escalation_rate: Option<String>,
    /// Escalation frequency in months (e.g., 12 for annual)
    pub escalation_frequency_months: Option<i32>,
    /// Financial summary
    pub total_lease_payments: String,
    pub initial_lease_liability: String,
    pub initial_rou_asset_value: String,
    pub residual_guarantee_amount: Option<String>,
    /// Current balances
    pub current_lease_liability: String,
    pub current_rou_asset_value: String,
    pub accumulated_rou_depreciation: String,
    /// Payment tracking
    pub total_payments_made: String,
    pub periods_elapsed: i32,
    /// GL account codes
    pub rou_asset_account_code: Option<String>,
    pub rou_depreciation_account_code: Option<String>,
    pub lease_liability_account_code: Option<String>,
    pub lease_expense_account_code: Option<String>,
    pub interest_expense_account_code: Option<String>,
    /// Status: "draft", "active", "modified", "impaired", "terminated", "expired"
    pub status: String,
    /// Impairment tracking
    pub impairment_amount: Option<String>,
    pub impairment_date: Option<chrono::NaiveDate>,
    /// Metadata
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Lease payment schedule line
/// Oracle Fusion equivalent: Lease Management > Payment Schedule
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeasePayment {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub lease_id: Uuid,
    /// Period sequence number
    pub period_number: i32,
    /// Payment due date
    pub payment_date: chrono::NaiveDate,
    /// Total payment amount
    pub payment_amount: String,
    /// Interest portion of the payment
    pub interest_amount: String,
    /// Principal portion of the payment
    pub principal_amount: String,
    /// Remaining lease liability after this payment
    pub remaining_liability: String,
    /// ROU asset value after depreciation for this period
    pub rou_asset_value: String,
    /// ROU depreciation for this period
    pub rou_depreciation: String,
    /// Accumulated ROU depreciation after this period
    pub accumulated_depreciation: String,
    /// Straight-line lease expense (for operating leases)
    pub lease_expense: String,
    /// Whether this payment has been made
    pub is_paid: bool,
    /// Payment reference
    pub payment_reference: Option<String>,
    /// GL journal entry reference
    pub journal_entry_id: Option<Uuid>,
    /// Status: "scheduled", "paid", "overdue", "cancelled"
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Lease modification
/// Tracks changes to lease terms that require remeasurement.
/// Oracle Fusion equivalent: Lease Management > Modifications
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeaseModification {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub lease_id: Uuid,
    /// Modification number (sequential)
    pub modification_number: i32,
    /// Type: "`term_extension`", "`scope_change`", "`payment_change`", "`rate_change`",
    ///       "reclassification"
    pub modification_type: String,
    /// Description of the modification
    pub description: Option<String>,
    /// Effective date of the modification
    pub effective_date: chrono::NaiveDate,
    /// Previous lease term (months)
    pub previous_term_months: Option<i32>,
    /// New lease term (months)
    pub new_term_months: Option<i32>,
    /// Previous end date
    pub previous_end_date: Option<chrono::NaiveDate>,
    /// New end date
    pub new_end_date: Option<chrono::NaiveDate>,
    /// Previous discount rate
    pub previous_discount_rate: Option<String>,
    /// New discount rate
    pub new_discount_rate: Option<String>,
    /// Change in lease liability due to modification
    pub liability_adjustment: String,
    /// Change in ROU asset due to modification
    pub rou_asset_adjustment: String,
    /// Status: "pending", "processed", "reversed"
    pub status: String,
    /// Metadata
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Lease termination
/// Oracle Fusion equivalent: Lease Management > Termination
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeaseTermination {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub lease_id: Uuid,
    /// Termination type: "early", "`end_of_term`", "`mutual_agreement`", "default"
    pub termination_type: String,
    /// Termination date
    pub termination_date: chrono::NaiveDate,
    /// Reason for termination
    pub reason: Option<String>,
    /// Remaining lease liability at termination
    pub remaining_liability: String,
    /// ROU asset value at termination (net of depreciation)
    pub remaining_rou_asset: String,
    /// Termination penalty / fee
    pub termination_penalty: String,
    /// Gain/loss on termination
    pub gain_loss_amount: String,
    /// "gain" or "loss"
    pub gain_loss_type: Option<String>,
    /// GL journal entry reference
    pub journal_entry_id: Option<Uuid>,
    /// Status: "pending", "processed", "reversed"
    pub status: String,
    /// Metadata
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Lease Accounting Dashboard Summary
/// Oracle Fusion equivalent: Lease Management Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeaseDashboardSummary {
    pub total_active_leases: i32,
    pub total_lease_liability: String,
    pub total_rou_assets: String,
    pub total_rou_depreciation: String,
    pub total_net_rou_assets: String,
    pub total_payments_made: String,
    pub operating_lease_count: i32,
    pub finance_lease_count: i32,
    pub upcoming_payments_count: i32,
    pub upcoming_payments_amount: String,
    pub leases_expiring_90_days: i32,
    pub leases_by_classification: serde_json::Value,
    pub leases_by_status: serde_json::Value,
    pub liability_by_period: serde_json::Value,
}

// ════════════════════════════════════════════════════════════════════════════════
// Project Costing (Oracle Fusion Cloud ERP: Project Management > Project Costing)
// ════════════════════════════════════════════════════════════════════════════════
//
// Oracle Fusion Cloud ERP Project Costing provides:
// - Cost Transactions: Track labor, material, expense, and other costs against projects/tasks
// - Burden Schedules: Define overhead/burden rate schedules for cost types
// - Cost Burdening: Apply burden rates to raw costs to compute burdened amounts
// - Cost Adjustments: Adjust previously recorded costs (increase, decrease, transfer)
// - Cost Distributions: Distribute project costs to GL accounts
// - Capitalization: Capitalize eligible project costs as fixed assets
// - Cost Reporting: Dashboard with cost breakdowns by project, type, and period
//
// Oracle Fusion equivalent: Project Management > Project Costing

/// Project cost transaction
/// Records a cost incurred against a project/task.
/// Oracle Fusion: Project Costing > Cost Transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectCostTransaction {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Auto-generated transaction number (e.g., "PJC-2024-00001")
    pub transaction_number: String,
    /// Project reference
    pub project_id: Uuid,
    /// Project number (denormalized for display)
    pub project_number: Option<String>,
    /// Task reference (optional - cost can be at project level)
    pub task_id: Option<Uuid>,
    /// Task number (denormalized)
    pub task_number: Option<String>,
    /// Cost type: "labor", "material", "expense", "equipment", "other"
    pub cost_type: String,
    /// Raw cost amount (before burdening)
    pub raw_cost_amount: String,
    /// Burdened cost amount (raw + burden)
    pub burdened_cost_amount: String,
    /// Burden amount (overhead applied)
    pub burden_amount: String,
    /// Currency code
    pub currency_code: String,
    /// Transaction date (when cost was incurred)
    pub transaction_date: chrono::NaiveDate,
    /// GL posting date
    pub gl_date: Option<chrono::NaiveDate>,
    /// Description of the cost
    pub description: Option<String>,
    /// Supplier/vendor reference (for material/expense costs)
    pub supplier_id: Option<Uuid>,
    /// Supplier name (denormalized)
    pub supplier_name: Option<String>,
    /// Employee reference (for labor costs)
    pub employee_id: Option<Uuid>,
    /// Employee name (denormalized)
    pub employee_name: Option<String>,
    /// Expenditure type / category
    pub expenditure_category: Option<String>,
    /// Quantity (hours for labor, units for material)
    pub quantity: Option<String>,
    /// Unit of measure ("hours", "each", "lot")
    pub unit_of_measure: Option<String>,
    /// Rate per unit
    pub unit_rate: Option<String>,
    /// Billable flag (whether this cost can be billed to customer)
    pub is_billable: bool,
    /// Capitalizable flag (whether this cost can be capitalized as an asset)
    pub is_capitalizable: bool,
    /// Status: "draft", "approved", "distributed", "adjusted", "reversed", "capitalized"
    pub status: String,
    /// GL distribution reference
    pub distribution_id: Option<Uuid>,
    /// Original transaction reference (for adjustments)
    pub original_transaction_id: Option<Uuid>,
    /// Adjustment type (if this is an adjustment): "increase", "decrease", "transfer"
    pub adjustment_type: Option<String>,
    /// Adjustment reason
    pub adjustment_reason: Option<String>,
    /// Metadata
    pub metadata: serde_json::Value,
    /// Audit
    pub created_by: Option<Uuid>,
    pub approved_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Burden schedule definition
/// Defines overhead/burden rates to be applied to project costs.
/// Oracle Fusion: Project Costing > Burden Schedules
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BurdenSchedule {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Schedule code (e.g., "OH-STD-2024")
    pub code: String,
    /// Schedule name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Status: "draft", "active", "inactive"
    pub status: String,
    /// Effective from date
    pub effective_from: chrono::NaiveDate,
    /// Effective to date
    pub effective_to: Option<chrono::NaiveDate>,
    /// Whether this is the default schedule for the organization
    pub is_default: bool,
    /// Metadata
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Burden schedule line (maps cost type to burden rate)
/// Oracle Fusion: Project Costing > Burden Schedule Lines
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BurdenScheduleLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Parent schedule reference
    pub schedule_id: Uuid,
    /// Line number within schedule
    pub line_number: i32,
    /// Cost type this line applies to: "labor", "material", "expense", "equipment", "other"
    pub cost_type: String,
    /// Expenditure category filter (None = applies to all of this cost type)
    pub expenditure_category: Option<String>,
    /// Burden rate percentage (e.g., "25.00" means 25% overhead)
    pub burden_rate_percent: String,
    /// GL account code for the burden amount
    pub burden_account_code: Option<String>,
    /// Whether this line is active
    pub is_active: bool,
    /// Metadata
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Cost adjustment
/// Records adjustments to previously recorded cost transactions.
/// Oracle Fusion: Project Costing > Cost Adjustments
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectCostAdjustment {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Auto-generated adjustment number
    pub adjustment_number: String,
    /// Original cost transaction being adjusted
    pub original_transaction_id: Uuid,
    /// Adjustment type: "increase", "decrease", "transfer", "reversal"
    pub adjustment_type: String,
    /// Adjustment amount (absolute value)
    pub adjustment_amount: String,
    /// New total raw cost after adjustment
    pub new_raw_cost: String,
    /// New burdened cost after adjustment
    pub new_burdened_cost: String,
    /// Reason for the adjustment
    pub reason: String,
    /// Description
    pub description: Option<String>,
    /// Effective date of the adjustment
    pub effective_date: chrono::NaiveDate,
    /// For transfers: destination project
    pub transfer_to_project_id: Option<Uuid>,
    /// For transfers: destination task
    pub transfer_to_task_id: Option<Uuid>,
    /// Status: "pending", "approved", "rejected", "processed"
    pub status: String,
    /// Created cost transaction (the adjustment transaction)
    pub created_transaction_id: Option<Uuid>,
    /// Metadata
    pub metadata: serde_json::Value,
    /// Audit
    pub created_by: Option<Uuid>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Cost distribution line (GL posting for a cost transaction)
/// Oracle Fusion: Project Costing > Cost Distributions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectCostDistribution {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Source cost transaction
    pub transaction_id: Uuid,
    /// Distribution line number
    pub line_number: i32,
    /// GL account code for the debit
    pub debit_account_code: String,
    /// GL account code for the credit
    pub credit_account_code: String,
    /// Distribution amount
    pub amount: String,
    /// Distribution type: "`raw_cost`", "burden", "total"
    pub distribution_type: String,
    /// GL posting date
    pub gl_date: chrono::NaiveDate,
    /// Whether this has been posted to GL
    pub is_posted: bool,
    /// GL batch reference
    pub gl_batch_id: Option<Uuid>,
    /// Metadata
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Project Costing Dashboard Summary
/// Oracle Fusion: Project Costing Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectCostingSummary {
    /// Total number of projects with costs
    pub project_count: i32,
    /// Total raw costs across all projects
    pub total_raw_costs: String,
    /// Total burdened costs across all projects
    pub total_burdened_costs: String,
    /// Total burden (overhead) across all projects
    pub total_burden: String,
    /// Total capitalized costs
    pub total_capitalized: String,
    /// Total billed to customers
    pub total_billed: String,
    /// Costs by type breakdown
    pub costs_by_type: serde_json::Value,
    /// Costs by project breakdown (top projects)
    pub costs_by_project: serde_json::Value,
    /// Costs by month trend
    pub costs_by_month: serde_json::Value,
    /// Pending adjustments count
    pub pending_adjustments: i32,
    /// Pending distributions count
    pub pending_distributions: i32,
}

// ════════════════════════════════════════════════════════════════════════════════
// Cost Allocations (Oracle Fusion GL > Cost Allocation / Mass Allocations)
// ════════════════════════════════════════════════════════════════════════════════
//
// Oracle Fusion Cloud ERP Cost Allocations provide:
// - Allocation Pools: Define cost pools (groups of accounts) to be allocated
// - Allocation Bases: Statistical or financial bases for distribution
//   (e.g., headcount, square footage, revenue, direct costs)
// - Allocation Rules: Map pools to target cost centers using bases
// - Rule Versions: Versioned rules with effective dates and approval workflow
// - Rule Execution: Run allocations to generate journal entries
// - Recurring Schedules: Schedule periodic allocation runs
// - Allocation History: Audit trail of all allocation runs
//
// Oracle Fusion equivalent: Financials > General Ledger > Allocations

/// Cost allocation pool definition
/// A pool defines a group of cost accounts whose balances are to be distributed.
/// Oracle Fusion equivalent: GL > Allocations > Cost Pools
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllocationPool {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Unique pool code (e.g., "`RENT_POOL`", "`IT_OVERHEAD`")
    pub code: String,
    /// Display name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Pool type: "`cost_center`", "project", "department", "custom"
    pub pool_type: String,
    /// Account code filter for selecting pool source balances
    /// (e.g., {"`account_codes"`: ["6100", "6110"]})
    pub source_account_codes: serde_json::Value,
    /// Department filter for pool source (optional)
    pub source_department_id: Option<Uuid>,
    /// Cost center filter for pool source (optional)
    pub source_cost_center: Option<String>,
    /// Whether this pool is active
    pub is_active: bool,
    /// Metadata
    pub metadata: serde_json::Value,
    /// Audit
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Allocation base definition
/// Defines the statistical or financial measure used to distribute costs.
/// Oracle Fusion equivalent: GL > Allocations > Allocation Bases
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllocationBase {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Unique base code (e.g., "HEADCOUNT", "SQFT", "REVENUE")
    pub code: String,
    /// Display name (e.g., "Employee Headcount", "Square Footage")
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Base type: "statistical" (user-entered), "financial" (from GL)
    pub base_type: String,
    /// For financial bases: the account code pattern to use as the base
    pub financial_account_code: Option<String>,
    /// Unit of measure (e.g., "persons", "`sq_meters`", "USD")
    pub unit_of_measure: Option<String>,
    /// Whether this base is active
    pub is_active: bool,
    /// Metadata
    pub metadata: serde_json::Value,
    /// Audit
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Statistical base value
/// User-entered or imported statistical measures per cost center/department.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllocationBaseValue {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Reference to the allocation base
    pub base_id: Uuid,
    /// Base code (denormalized)
    pub base_code: String,
    /// Dimension reference (department, cost center, or project)
    pub department_id: Option<Uuid>,
    pub department_name: Option<String>,
    pub cost_center: Option<String>,
    pub project_id: Option<Uuid>,
    /// The statistical value
    pub value: String,
    /// Effective date of this value
    pub effective_date: chrono::NaiveDate,
    /// Source: "manual", "import", "system"
    pub source: String,
    /// Metadata
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Allocation rule definition
/// Maps a cost pool to target cost centers using an allocation base.
/// Oracle Fusion equivalent: GL > Allocations > Allocation Rules
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllocationRule {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Auto-generated rule number (e.g., "ALLOC-001")
    pub rule_number: String,
    /// Rule name (e.g., "Rent Allocation by SQFT")
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Reference to the cost pool
    pub pool_id: Uuid,
    /// Pool code (denormalized)
    pub pool_code: String,
    /// Reference to the allocation base
    pub base_id: Uuid,
    /// Base code (denormalized)
    pub base_code: String,
    /// Allocation method: "proportional", "`fixed_percent`", "`fixed_amount`"
    pub allocation_method: String,
    /// Journal entry description template
    pub journal_description: Option<String>,
    /// Offset (contra) account for the credit side of the allocation
    pub offset_account_code: Option<String>,
    /// Rule status: "draft", "active", "inactive"
    pub status: String,
    /// Current version number
    pub current_version: i32,
    /// Effective dates
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    /// Whether this rule generates reversing entries
    pub is_reversing: bool,
    /// Currency code
    pub currency_code: String,
    /// Metadata
    pub metadata: serde_json::Value,
    /// Audit
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Allocation rule target line
/// Defines a target cost center and optional fixed percentage/amount.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllocationRuleTarget {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Rule reference
    pub rule_id: Uuid,
    /// Line number
    pub line_number: i32,
    /// Target department
    pub department_id: Option<Uuid>,
    pub department_name: Option<String>,
    /// Target cost center
    pub cost_center: Option<String>,
    /// Target project
    pub project_id: Option<Uuid>,
    pub project_name: Option<String>,
    /// Target debit account code (where the allocated cost goes)
    pub target_account_code: String,
    /// Fixed percentage (for "`fixed_percent`" method)
    pub fixed_percent: Option<String>,
    /// Fixed amount (for "`fixed_amount`" method)
    pub fixed_amount: Option<String>,
    /// Whether this target line is active
    pub is_active: bool,
    /// Metadata
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Allocation run (execution)
/// Represents a single execution of an allocation rule.
/// Oracle Fusion equivalent: GL > Allocations > Run Allocations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllocationRun {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Auto-generated run number (e.g., "ARUN-2024-00001")
    pub run_number: String,
    /// Rule reference
    pub rule_id: Uuid,
    /// Rule name (denormalized)
    pub rule_name: String,
    /// Rule number (denormalized)
    pub rule_number: String,
    /// Allocation period start
    pub period_start: chrono::NaiveDate,
    /// Allocation period end
    pub period_end: chrono::NaiveDate,
    /// Total source amount (from pool)
    pub total_source_amount: String,
    /// Total allocated amount
    pub total_allocated_amount: String,
    /// Number of target lines generated
    pub line_count: i32,
    /// Status: "draft", "posted", "reversed"
    pub status: String,
    /// Journal entry reference (the generated GL batch)
    pub journal_entry_id: Option<Uuid>,
    /// Run date
    pub run_date: chrono::NaiveDate,
    /// Reversal reference
    pub reversed_by_id: Option<Uuid>,
    pub reversal_reason: Option<String>,
    /// Metadata
    pub metadata: serde_json::Value,
    /// Audit
    pub created_by: Option<Uuid>,
    pub posted_by: Option<Uuid>,
    pub posted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Allocation run line (individual debit/credit)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllocationRunLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Run header reference
    pub run_id: Uuid,
    /// Line number
    pub line_number: i32,
    /// Line type: "debit" (target), "credit" (offset/source)
    pub line_type: String,
    /// Account code
    pub account_code: String,
    /// Department ID
    pub department_id: Option<Uuid>,
    /// Department name
    pub department_name: Option<String>,
    /// Cost center
    pub cost_center: Option<String>,
    /// Project ID
    pub project_id: Option<Uuid>,
    /// Allocated amount
    pub amount: String,
    /// Base value used for this allocation
    pub base_value_used: Option<String>,
    /// Percentage of total base
    pub percent_of_total: Option<String>,
    /// Description
    pub description: Option<String>,
    /// Metadata
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Allocation summary for dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllocationSummary {
    /// Total active rules
    pub active_rule_count: i32,
    /// Total pools
    pub pool_count: i32,
    /// Total allocation runs (period)
    pub run_count: i32,
    /// Total allocated amount (period)
    pub total_allocated_amount: String,
    /// Runs by status
    pub runs_by_status: serde_json::Value,
    /// Allocations by pool
    pub allocations_by_pool: serde_json::Value,
    /// Top allocation rules by amount
    pub top_rules: serde_json::Value,
}

// ============================================================================
// Accounting Hub (Oracle Fusion Accounting Hub Cloud)
// ============================================================================

/// External system registration for the Accounting Hub.
/// Oracle Fusion equivalent: Accounting Hub > External Systems
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalSystem {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    /// System type: "erp", "billing", "pos", "banking", "insurance", "custom"
    pub system_type: String,
    /// Connection details (API endpoint, etc.)
    pub connection_config: serde_json::Value,
    /// Whether the system is active
    pub is_active: bool,
    pub last_event_received: Option<chrono::DateTime<Utc>>,
    pub total_events_received: i32,
    pub total_events_processed: i32,
    pub total_events_failed: i32,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

/// An event received from an external system to be accounted.
/// Oracle Fusion equivalent: Accounting Hub > Accounting Events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountingEvent {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub event_number: String,
    pub external_system_id: Uuid,
    pub external_system_code: Option<String>,
    /// The event type from the external system
    pub event_type: String,
    /// The class of the event: "invoice", "payment", "adjustment", "transfer", "custom"
    pub event_class: String,
    /// Unique identifier from the source system
    pub source_event_id: String,
    /// Raw event payload from external system
    pub payload: serde_json::Value,
    /// Processed/extracted transaction attributes
    pub transaction_attributes: serde_json::Value,
    /// The accounting method to apply
    pub accounting_method_id: Option<Uuid>,
    /// "received", "validated", "accounted", "posted", "transferred", "error"
    pub status: String,
    pub error_message: Option<String>,
    /// The resulting journal entry
    pub journal_entry_id: Option<Uuid>,
    pub event_date: chrono::NaiveDate,
    pub accounting_date: Option<chrono::NaiveDate>,
    pub currency_code: String,
    pub total_amount: Option<String>,
    pub description: Option<String>,
    pub processed_by: Option<Uuid>,
    pub processed_at: Option<chrono::DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

/// A mapping rule that transforms external event attributes into accounting data.
/// Oracle Fusion equivalent: Accounting Hub > Transaction Mapping Rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionMappingRule {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub external_system_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    /// Event type this rule applies to
    pub event_type: String,
    /// Event class this rule applies to
    pub event_class: String,
    /// Priority (lower = higher priority)
    pub priority: i32,
    /// Conditions to match (JSON map of field → expected value)
    pub conditions: serde_json::Value,
    /// Mapping expressions (`source_field` → `target_field`)
    pub field_mappings: serde_json::Value,
    /// Accounting method to use when this rule matches
    pub accounting_method_id: Option<Uuid>,
    pub stop_on_match: bool,
    pub is_active: bool,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

/// Dashboard summary for Accounting Hub
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountingHubDashboardSummary {
    pub total_systems: i32,
    pub active_systems: i32,
    pub total_events: i32,
    pub received_events: i32,
    pub accounted_events: i32,
    pub posted_events: i32,
    pub error_events: i32,
    pub total_amount_processed: String,
    pub events_by_system: serde_json::Value,
    pub events_by_type: serde_json::Value,
}
