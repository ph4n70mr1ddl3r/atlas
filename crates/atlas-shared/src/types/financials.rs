use crate::types::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
// ============================================================================
// Period Close Management (Oracle Fusion General Ledger)
// ============================================================================

/// Accounting calendar definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountingCalendar {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub calendar_type: String,
    pub fiscal_year_start_month: i32,
    pub periods_per_year: i32,
    pub has_adjusting_period: bool,
    pub current_fiscal_year: Option<i32>,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
}

/// Create/update calendar request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountingCalendarRequest {
    pub name: String,
    pub description: Option<String>,
    #[serde(default = "default_monthly")]
    pub calendar_type: String,
    #[serde(default = "default_one")]
    pub fiscal_year_start_month: i32,
    #[serde(default = "default_twelve")]
    pub periods_per_year: i32,
    #[serde(default)]
    pub has_adjusting_period: bool,
    pub current_fiscal_year: Option<i32>,
}

pub fn default_monthly() -> String {
    "monthly".to_string()
}
pub const fn default_one() -> i32 {
    1
}
pub const fn default_twelve() -> i32 {
    12
}

/// Period status within the financial close cycle
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum PeriodStatus {
    #[default]
    NotOpened,
    Future,
    Open,
    PendingClose,
    Closed,
    PermanentlyClosed,
}

impl PeriodStatus {
    /// Whether posting is allowed in this period status
    #[must_use]
    pub const fn allows_posting(&self) -> bool {
        matches!(self, Self::Open | Self::PendingClose)
    }

    /// Whether the status can be changed
    #[must_use]
    pub const fn is_changeable(&self) -> bool {
        !matches!(self, Self::PermanentlyClosed)
    }
}

/// Accounting period within a calendar
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountingPeriod {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub calendar_id: Uuid,
    pub period_name: String,
    pub period_number: i32,
    pub fiscal_year: i32,
    pub quarter: Option<i32>,
    pub start_date: chrono::NaiveDate,
    pub end_date: chrono::NaiveDate,
    pub status: String,
    pub status_changed_by: Option<Uuid>,
    pub status_changed_at: Option<DateTime<Utc>>,
    pub closed_by: Option<Uuid>,
    pub closed_at: Option<DateTime<Utc>>,
    pub period_type: String,
    // Subledger statuses
    pub gl_status: String,
    pub ap_status: String,
    pub ar_status: String,
    pub fa_status: String,
    pub po_status: String,
    // Aggregated balances (NUMERIC from DB, serialized as string/number)
    pub total_debits: serde_json::Value,
    pub total_credits: serde_json::Value,
    pub net_activity: serde_json::Value,
    pub beginning_balance: serde_json::Value,
    pub ending_balance: serde_json::Value,
    pub journal_entry_count: i32,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Period close checklist item
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PeriodCloseChecklistItem {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub period_id: Uuid,
    pub task_name: String,
    pub task_description: Option<String>,
    pub task_order: i32,
    pub category: Option<String>,
    pub subledger: Option<String>,
    pub status: String,
    pub assigned_to: Option<Uuid>,
    pub due_date: Option<chrono::NaiveDate>,
    pub completed_by: Option<Uuid>,
    pub completed_at: Option<DateTime<Utc>>,
    pub depends_on: Option<Uuid>,
    pub notes: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create checklist item request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateChecklistItemRequest {
    pub task_name: String,
    pub task_description: Option<String>,
    pub task_order: Option<i32>,
    pub category: Option<String>,
    pub subledger: Option<String>,
    pub assigned_to: Option<Uuid>,
    pub due_date: Option<chrono::NaiveDate>,
    pub depends_on: Option<Uuid>,
    pub notes: Option<String>,
}

/// Period close dashboard summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PeriodCloseSummary {
    pub calendar_id: Uuid,
    pub calendar_name: String,
    pub fiscal_year: i32,
    pub current_period: Option<AccountingPeriod>,
    pub open_periods: Vec<AccountingPeriod>,
    pub pending_close_periods: Vec<AccountingPeriod>,
    pub total_checklist_items: i32,
    pub completed_checklist_items: i32,
    pub close_progress_percent: f64,
}

// ============================================================================
// Currency & Exchange Rate Management (Oracle Fusion GL Currency)
// ============================================================================

/// Supported exchange rate types
/// Oracle Fusion: Daily Rates, Spot, Corporate, Period Average, Period End, User
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ExchangeRateType {
    #[default]
    Daily,
    Spot,
    Corporate,
    PeriodAverage,
    PeriodEnd,
    User,
    Fixed,
}

/// Currency definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrencyDefinition {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub symbol: String,
    pub precision: i32,
    pub is_base_currency: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create/update currency request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrencyRequest {
    pub code: String,
    pub name: String,
    pub symbol: Option<String>,
    #[serde(default = "default_precision")]
    pub precision: i32,
    #[serde(default)]
    pub is_base_currency: bool,
}

pub const fn default_precision() -> i32 {
    2
}

/// Exchange rate record
/// Oracle Fusion: Daily Rates table with from/to currency and effective date
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExchangeRate {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub from_currency: String,
    pub to_currency: String,
    pub rate_type: String,
    pub rate: String, // NUMERIC from DB serialized as string
    pub effective_date: chrono::NaiveDate,
    pub inverse_rate: Option<String>,
    pub source: Option<String>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create/update exchange rate request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExchangeRateRequest {
    pub from_currency: String,
    pub to_currency: String,
    pub rate_type: String,
    pub rate: String,
    pub effective_date: chrono::NaiveDate,
    pub inverse_rate: Option<String>,
    pub source: Option<String>,
}

/// Result of a currency conversion
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrencyConversionResult {
    pub from_currency: String,
    pub to_currency: String,
    pub from_amount: String,
    pub to_amount: String,
    pub exchange_rate: String,
    pub rate_type: String,
    pub effective_date: chrono::NaiveDate,
    pub gain_loss: Option<String>,
}

/// Unrealized gain/loss on a foreign-currency-denominated balance
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnrealizedGainLoss {
    pub currency: String,
    pub original_amount: String,
    pub original_rate: String,
    pub revalued_amount: String,
    pub current_rate: String,
    pub gain_loss_amount: String,
    pub gain_loss_type: String, // "gain" or "loss"
}

// ============================================================================
// Tax Management (Oracle Fusion Tax)
// ============================================================================

/// Tax regime definition
/// Oracle Fusion: Tax Configuration > Tax Regimes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxRegime {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub tax_type: String,
    pub default_inclusive: bool,
    pub allows_recovery: bool,
    pub rounding_rule: String,
    pub rounding_precision: i32,
    pub is_active: bool,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create/update tax regime request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxRegimeRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(default = "default_tax_type")]
    pub tax_type: String,
    #[serde(default)]
    pub default_inclusive: bool,
    #[serde(default)]
    pub allows_recovery: bool,
    #[serde(default = "default_rounding_rule")]
    pub rounding_rule: String,
    #[serde(default = "default_rounding_precision")]
    pub rounding_precision: i32,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
}

pub fn default_tax_type() -> String {
    "vat".to_string()
}
pub fn default_rounding_rule() -> String {
    "nearest".to_string()
}
pub const fn default_rounding_precision() -> i32 {
    2
}

/// Tax jurisdiction
/// Oracle Fusion: Tax Configuration > Tax Jurisdictions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxJurisdiction {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub regime_id: Uuid,
    pub code: String,
    pub name: String,
    pub geographic_level: String,
    pub country_code: Option<String>,
    pub state_code: Option<String>,
    pub county: Option<String>,
    pub city: Option<String>,
    pub postal_code_pattern: Option<String>,
    pub is_active: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create/update tax jurisdiction request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxJurisdictionRequest {
    pub regime_code: String,
    pub code: String,
    pub name: String,
    #[serde(default = "default_geographic_level")]
    pub geographic_level: String,
    pub country_code: Option<String>,
    pub state_code: Option<String>,
    pub county: Option<String>,
    pub city: Option<String>,
    pub postal_code_pattern: Option<String>,
}

pub fn default_geographic_level() -> String {
    "country".to_string()
}

/// Tax rate definition
/// Oracle Fusion: Tax Configuration > Tax Rates
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxRate {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub regime_id: Uuid,
    pub jurisdiction_id: Option<Uuid>,
    pub code: String,
    pub name: String,
    pub rate_percentage: String, // NUMERIC serialized as string
    pub rate_type: String,
    pub tax_account_code: Option<String>,
    pub recoverable: bool,
    pub recovery_percentage: Option<String>, // NUMERIC serialized as string
    pub effective_from: chrono::NaiveDate,
    pub effective_to: Option<chrono::NaiveDate>,
    pub is_active: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create/update tax rate request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxRateRequest {
    pub regime_code: String,
    pub jurisdiction_code: Option<String>,
    pub code: String,
    pub name: String,
    pub rate_percentage: String,
    #[serde(default = "default_rate_type_tax")]
    pub rate_type: String,
    pub tax_account_code: Option<String>,
    #[serde(default)]
    pub recoverable: bool,
    pub recovery_percentage: Option<String>,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
}

pub fn default_rate_type_tax() -> String {
    "standard".to_string()
}

/// Tax determination rule
/// Oracle Fusion: Tax Rules > Determination Rules
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxDeterminationRule {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub regime_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub priority: i32,
    pub condition: serde_json::Value,
    pub action: serde_json::Value,
    pub stop_on_match: bool,
    pub is_active: bool,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Tax line (calculated tax on a transaction)
/// Oracle Fusion: Tax lines attached to invoice/purchase order lines
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub line_id: Option<Uuid>,
    pub regime_id: Option<Uuid>,
    pub jurisdiction_id: Option<Uuid>,
    pub tax_rate_id: Uuid,
    pub taxable_amount: String,
    pub tax_rate_percentage: String,
    pub tax_amount: String,
    pub is_inclusive: bool,
    pub original_amount: Option<String>,
    pub recoverable_amount: Option<String>,
    pub non_recoverable_amount: Option<String>,
    pub tax_account_code: Option<String>,
    pub determination_rule_id: Option<Uuid>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

/// Tax calculation request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxCalculationRequest {
    /// Entity type (e.g., "`sales_orders`", "`purchase_orders`")
    pub entity_type: String,
    /// Entity ID
    pub entity_id: Option<Uuid>,
    /// Line items to calculate tax for
    pub lines: Vec<TaxCalculationLine>,
    /// Transaction context for determination rules
    pub context: serde_json::Value,
    /// Whether to persist the tax lines
    #[serde(default)]
    pub persist: bool,
}

/// Single line for tax calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxCalculationLine {
    /// Optional line ID (for linking back)
    pub line_id: Option<Uuid>,
    /// Line amount (net)
    pub amount: String,
    /// Optional product category for determination
    pub product_category: Option<String>,
    /// Optional product code
    pub product_code: Option<String>,
    /// Optional ship-from country
    pub ship_from_country: Option<String>,
    /// Optional ship-to country
    pub ship_to_country: Option<String>,
    /// Optional ship-to state/province
    pub ship_to_state: Option<String>,
    /// Optional specific tax rate codes to apply (bypasses determination)
    pub tax_rate_codes: Option<Vec<String>>,
    /// Whether the amount includes tax already
    pub is_inclusive: Option<bool>,
}

/// Tax calculation result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxCalculationResult {
    pub lines: Vec<TaxLineResult>,
    pub total_taxable_amount: String,
    pub total_tax_amount: String,
    pub total_recoverable_amount: String,
    pub total_non_recoverable_amount: String,
}

/// Tax calculation result for a single line
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxLineResult {
    pub line_id: Option<Uuid>,
    pub regime_code: Option<String>,
    pub jurisdiction_code: Option<String>,
    pub tax_rate_code: String,
    pub tax_rate_name: String,
    pub rate_percentage: String,
    pub taxable_amount: String,
    pub tax_amount: String,
    pub is_inclusive: bool,
    pub recoverable: bool,
    pub recovery_percentage: Option<String>,
    pub recoverable_amount: Option<String>,
    pub non_recoverable_amount: Option<String>,
}

// ============================================================================
// Intercompany Transactions (Oracle Fusion Intercompany)
// ============================================================================

/// Intercompany transaction batch
/// Oracle Fusion: Intercompany > Intercompany Batches
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntercompanyBatch {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub batch_number: String,
    pub description: Option<String>,
    /// 'draft', 'submitted', 'approved', 'posted', 'cancelled'
    pub status: String,
    pub from_entity_id: Uuid,
    pub from_entity_name: String,
    pub to_entity_id: Uuid,
    pub to_entity_name: String,
    pub currency_code: String,
    pub total_amount: String,
    pub total_debit: String,
    pub total_credit: String,
    pub transaction_count: i32,
    pub from_journal_id: Option<Uuid>,
    pub to_journal_id: Option<Uuid>,
    pub accounting_date: Option<chrono::NaiveDate>,
    pub posted_at: Option<DateTime<Utc>>,
    pub rejected_reason: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub approved_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create intercompany batch request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntercompanyBatchRequest {
    pub batch_number: String,
    pub description: Option<String>,
    pub from_entity_id: Uuid,
    pub from_entity_name: String,
    pub to_entity_id: Uuid,
    pub to_entity_name: String,
    #[serde(default = "default_currency_usd")]
    pub currency_code: String,
    pub accounting_date: Option<chrono::NaiveDate>,
}

pub fn default_currency_usd() -> String {
    "USD".to_string()
}

/// Intercompany transaction (individual line within a batch)
/// Oracle Fusion: Intercompany > Intercompany Transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntercompanyTransaction {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub batch_id: Uuid,
    pub transaction_number: String,
    /// 'invoice', '`journal_entry`', 'payment', 'charge', 'allocation'
    pub transaction_type: String,
    pub description: Option<String>,
    pub from_entity_id: Uuid,
    pub from_entity_name: String,
    pub to_entity_id: Uuid,
    pub to_entity_name: String,
    pub amount: String,
    pub currency_code: String,
    pub exchange_rate: Option<String>,
    pub from_debit_account: Option<String>,
    pub from_credit_account: Option<String>,
    pub to_debit_account: Option<String>,
    pub to_credit_account: Option<String>,
    pub from_ic_account: String,
    pub to_ic_account: String,
    /// 'draft', 'approved', 'posted', 'settled', 'cancelled'
    pub status: String,
    pub transaction_date: chrono::NaiveDate,
    pub due_date: Option<chrono::NaiveDate>,
    pub settlement_date: Option<chrono::NaiveDate>,
    pub source_entity_type: Option<String>,
    pub source_entity_id: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create intercompany transaction request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntercompanyTransactionRequest {
    pub batch_number: String,
    #[serde(default = "default_ic_transaction_type")]
    pub transaction_type: String,
    pub description: Option<String>,
    pub from_entity_id: Uuid,
    pub from_entity_name: String,
    pub to_entity_id: Uuid,
    pub to_entity_name: String,
    pub amount: String,
    #[serde(default = "default_currency_usd")]
    pub currency_code: String,
    pub exchange_rate: Option<String>,
    pub from_debit_account: Option<String>,
    pub from_credit_account: Option<String>,
    pub to_debit_account: Option<String>,
    pub to_credit_account: Option<String>,
    pub from_ic_account: Option<String>,
    pub to_ic_account: Option<String>,
    pub transaction_date: Option<chrono::NaiveDate>,
    pub due_date: Option<chrono::NaiveDate>,
    pub source_entity_type: Option<String>,
    pub source_entity_id: Option<Uuid>,
}

pub fn default_ic_transaction_type() -> String {
    "invoice".to_string()
}

/// Intercompany settlement
/// Oracle Fusion: Intercompany > Settlements
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntercompanySettlement {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub settlement_number: String,
    /// 'cash', 'netting', 'offset'
    pub settlement_method: String,
    pub from_entity_id: Uuid,
    pub to_entity_id: Uuid,
    pub settled_amount: String,
    pub currency_code: String,
    pub payment_reference: Option<String>,
    /// 'pending', 'completed', 'cancelled'
    pub status: String,
    pub settlement_date: chrono::NaiveDate,
    pub transaction_ids: serde_json::Value,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create intercompany settlement request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntercompanySettlementRequest {
    pub settlement_number: String,
    #[serde(default = "default_settlement_method")]
    pub settlement_method: String,
    pub from_entity_id: Uuid,
    pub to_entity_id: Uuid,
    pub settled_amount: String,
    #[serde(default = "default_currency_usd")]
    pub currency_code: String,
    pub payment_reference: Option<String>,
    pub transaction_ids: Option<Vec<Uuid>>,
}

pub fn default_settlement_method() -> String {
    "cash".to_string()
}

/// Intercompany balance (outstanding due-to/due-from between entities)
/// Oracle Fusion: Intercompany > Balances Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntercompanyBalance {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub from_entity_id: Uuid,
    pub to_entity_id: Uuid,
    pub currency_code: String,
    pub total_outstanding: String,
    pub total_posted: String,
    pub total_settled: String,
    pub open_transaction_count: i32,
    pub as_of_date: chrono::NaiveDate,
    pub metadata: serde_json::Value,
    pub updated_at: DateTime<Utc>,
}

/// Intercompany balance summary across all entity pairs
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntercompanyBalanceSummary {
    pub total_outstanding: String,
    pub entity_pairs: i32,
    pub open_transactions: i32,
    pub balances: Vec<IntercompanyBalance>,
}

/// Tax report summary
/// Oracle Fusion: Tax Reporting > Tax Filing
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxReport {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub regime_id: Uuid,
    pub jurisdiction_id: Option<Uuid>,
    pub period_start: chrono::NaiveDate,
    pub period_end: chrono::NaiveDate,
    pub total_taxable_amount: String,
    pub total_tax_amount: String,
    pub total_recoverable_amount: String,
    pub total_non_recoverable_amount: String,
    pub transaction_count: i32,
    pub status: String,
    pub filed_by: Option<Uuid>,
    pub filed_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// Bank Reconciliation (Oracle Fusion Cash Management)
// ============================================================================

/// Bank account definition
/// Oracle Fusion: Cash Management > Bank Accounts
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BankAccount {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub account_number: String,
    pub account_name: String,
    pub bank_name: String,
    pub bank_code: Option<String>,
    pub branch_name: Option<String>,
    pub branch_code: Option<String>,
    pub gl_account_code: Option<String>,
    pub currency_code: String,
    pub account_type: String,
    pub last_statement_balance: serde_json::Value,
    pub last_statement_date: Option<chrono::NaiveDate>,
    pub is_active: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

/// Create/update bank account request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BankAccountRequest {
    pub account_number: String,
    pub account_name: String,
    pub bank_name: String,
    pub bank_code: Option<String>,
    pub branch_name: Option<String>,
    pub branch_code: Option<String>,
    pub gl_account_code: Option<String>,
    #[serde(default = "default_currency_usd")]
    pub currency_code: String,
    #[serde(default = "default_checking")]
    pub account_type: String,
}

pub fn default_checking() -> String {
    "checking".to_string()
}

/// Bank statement header
/// Oracle Fusion: Cash Management > Bank Statements
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BankStatement {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub bank_account_id: Uuid,
    pub statement_number: String,
    pub statement_date: chrono::NaiveDate,
    pub start_date: chrono::NaiveDate,
    pub end_date: chrono::NaiveDate,
    pub opening_balance: serde_json::Value,
    pub closing_balance: serde_json::Value,
    pub total_deposits: serde_json::Value,
    pub total_withdrawals: serde_json::Value,
    pub total_interest: serde_json::Value,
    pub total_charges: serde_json::Value,
    pub total_lines: i32,
    pub matched_lines: i32,
    pub unmatched_lines: i32,
    pub status: String,
    pub reconciliation_percent: serde_json::Value,
    pub imported_by: Option<Uuid>,
    pub reviewed_by: Option<Uuid>,
    pub reconciled_by: Option<Uuid>,
    pub reconciled_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

/// Bank statement line
/// Oracle Fusion: Individual line items within a bank statement
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BankStatementLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub statement_id: Uuid,
    pub line_number: i32,
    pub transaction_date: chrono::NaiveDate,
    pub transaction_type: String,
    pub amount: serde_json::Value,
    pub description: Option<String>,
    pub reference_number: Option<String>,
    pub check_number: Option<String>,
    pub counterparty_name: Option<String>,
    pub counterparty_account: Option<String>,
    pub match_status: String,
    pub matched_by: Option<Uuid>,
    pub matched_at: Option<DateTime<Utc>>,
    pub match_method: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

/// System transaction (AP payment, AR receipt, GL entry)
/// Oracle Fusion: Reconciliation sources
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemTransaction {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub bank_account_id: Uuid,
    pub source_type: String,
    pub source_id: Uuid,
    pub source_number: Option<String>,
    pub transaction_date: chrono::NaiveDate,
    pub amount: serde_json::Value,
    pub transaction_type: String,
    pub description: Option<String>,
    pub reference_number: Option<String>,
    pub check_number: Option<String>,
    pub counterparty_name: Option<String>,
    pub status: String,
    pub gl_posting_date: Option<chrono::NaiveDate>,
    pub currency_code: String,
    pub exchange_rate: Option<serde_json::Value>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

/// Reconciliation match record
/// Oracle Fusion: Links between statement lines and system transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReconciliationMatch {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub statement_id: Uuid,
    pub statement_line_id: Uuid,
    pub system_transaction_id: Uuid,
    pub match_method: String,
    pub match_confidence: Option<serde_json::Value>,
    pub matched_by: Option<Uuid>,
    pub matched_at: Option<DateTime<Utc>>,
    pub unmatched_by: Option<Uuid>,
    pub unmatched_at: Option<DateTime<Utc>>,
    pub status: String,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

/// Reconciliation summary (per account per period)
/// Oracle Fusion: Reconciliation Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReconciliationSummary {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub bank_account_id: Uuid,
    pub period_start: chrono::NaiveDate,
    pub period_end: chrono::NaiveDate,
    pub statement_id: Option<Uuid>,
    pub statement_balance: serde_json::Value,
    pub book_balance: serde_json::Value,
    pub deposits_in_transit: serde_json::Value,
    pub outstanding_checks: serde_json::Value,
    pub bank_charges: serde_json::Value,
    pub bank_interest: serde_json::Value,
    pub errors_and_omissions: serde_json::Value,
    pub adjusted_book_balance: serde_json::Value,
    pub adjusted_bank_balance: serde_json::Value,
    pub difference: serde_json::Value,
    pub is_balanced: bool,
    pub status: String,
    pub reviewed_by: Option<Uuid>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

/// Auto-matching rule
/// Oracle Fusion: User-defined reconciliation matching rules
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReconciliationMatchingRule {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub bank_account_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub priority: i32,
    pub criteria: serde_json::Value,
    pub stop_on_match: bool,
    pub is_active: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create matching rule request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchingRuleRequest {
    pub name: String,
    pub description: Option<String>,
    pub bank_account_id: Option<Uuid>,
    #[serde(default = "default_priority")]
    pub priority: i32,
    pub criteria: serde_json::Value,
    #[serde(default = "default_true_val")]
    pub stop_on_match: bool,
}

pub const fn default_priority() -> i32 {
    100
}
pub const fn default_true_val() -> bool {
    true
}

/// Auto-match result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoMatchResult {
    pub total_lines: i32,
    pub matched: i32,
    pub unmatched: i32,
    pub already_matched: i32,
    pub matches: Vec<AutoMatchPair>,
}

/// A single auto-matched pair
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoMatchPair {
    pub statement_line_id: Uuid,
    pub system_transaction_id: Uuid,
    pub match_method: String,
    pub confidence: f64,
}

// ============================================================================
// Expense Management (Oracle Fusion Expenses)
// ============================================================================

/// Expense category definition
/// Oracle Fusion: Expenses > Expense Categories
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpenseCategory {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    /// Whether this category requires a receipt above a threshold
    pub receipt_required: bool,
    /// Amount threshold above which a receipt is required
    pub receipt_threshold: Option<String>,
    /// Whether this category is eligible for per-diem
    pub is_per_diem: bool,
    /// Default per-diem rate (if `is_per_diem`)
    pub default_per_diem_rate: Option<String>,
    /// Whether this category is eligible for mileage
    pub is_mileage: bool,
    /// Default mileage rate per unit (if `is_mileage`)
    pub default_mileage_rate: Option<String>,
    /// GL account code for posting
    pub expense_account_code: Option<String>,
    pub is_active: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Expense policy definition
/// Oracle Fusion: Expenses > Expense Policies
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpensePolicy {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    /// The expense category this policy applies to (None = all categories)
    pub category_id: Option<Uuid>,
    /// Minimum expense amount that triggers policy
    pub min_amount: Option<String>,
    /// Maximum expense amount allowed without special approval
    pub max_amount: Option<String>,
    /// Maximum daily total for the category
    pub daily_limit: Option<String>,
    /// Maximum total per expense report for the category
    pub report_limit: Option<String>,
    /// Whether violations require manager approval
    pub requires_approval_on_violation: bool,
    /// Action on violation: "warn", "block", "`require_justification`"
    pub violation_action: String,
    pub is_active: bool,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Expense report header
/// Oracle Fusion: Expenses > Expense Reports
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpenseReport {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub report_number: String,
    pub title: String,
    pub description: Option<String>,
    /// 'draft', 'submitted', 'approved', 'rejected', 'reimbursed', 'cancelled'
    pub status: String,
    /// Employee who submitted the report
    pub employee_id: Uuid,
    pub employee_name: Option<String>,
    /// Department for cost center
    pub department_id: Option<Uuid>,
    /// Purpose of the expense
    pub purpose: Option<String>,
    /// Project reference (for project billing)
    pub project_id: Option<Uuid>,
    /// Currency code
    pub currency_code: String,
    /// Total amount of all expense lines
    pub total_amount: String,
    /// Total reimbursable amount
    pub reimbursable_amount: String,
    /// Total amount requiring receipts
    pub receipt_required_amount: String,
    /// Number of attached receipts
    pub receipt_count: i32,
    /// Business trip start date
    pub trip_start_date: Option<chrono::NaiveDate>,
    /// Business trip end date
    pub trip_end_date: Option<chrono::NaiveDate>,
    /// Cost center override
    pub cost_center: Option<String>,
    /// Approval information
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub rejection_reason: Option<String>,
    /// Payment information
    pub payment_method: Option<String>,
    pub payment_reference: Option<String>,
    pub reimbursed_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Individual expense line within a report
/// Oracle Fusion: Expense Lines within Expense Reports
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpenseLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub report_id: Uuid,
    pub line_number: i32,
    pub expense_category_id: Option<Uuid>,
    pub expense_category_name: Option<String>,
    /// 'expense', '`per_diem`', 'mileage', '`credit_card`'
    pub expense_type: String,
    /// Free-text description of the expense
    pub description: Option<String>,
    /// Date the expense was incurred
    pub expense_date: chrono::NaiveDate,
    /// Amount in the report currency
    pub amount: String,
    /// Original currency if different from report currency
    pub original_currency: Option<String>,
    /// Original amount in the foreign currency
    pub original_amount: Option<String>,
    /// Exchange rate applied
    pub exchange_rate: Option<String>,
    /// Whether this expense is reimbursable
    pub is_reimbursable: bool,
    /// Whether a receipt is attached
    pub has_receipt: bool,
    /// Receipt attachment reference
    pub receipt_reference: Option<String>,
    /// Merchant / vendor name
    pub merchant_name: Option<String>,
    /// Location where expense was incurred
    pub location: Option<String>,
    /// Attendees (for entertainment / meals)
    pub attendees: Option<serde_json::Value>,
    /// For per-diem: number of days
    pub per_diem_days: Option<f64>,
    /// For per-diem: daily rate
    pub per_diem_rate: Option<String>,
    /// For mileage: distance
    pub mileage_distance: Option<f64>,
    /// For mileage: rate per unit
    pub mileage_rate: Option<String>,
    /// For mileage: unit ("miles" or "km")
    pub mileage_unit: Option<String>,
    /// For mileage: starting location
    pub mileage_from: Option<String>,
    /// For mileage: ending location
    pub mileage_to: Option<String>,
    /// Whether this line violates any expense policy
    pub policy_violation: bool,
    /// Policy violation details
    pub policy_violation_message: Option<String>,
    /// GL account code override
    pub expense_account_code: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Expense policy violation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpensePolicyViolation {
    pub line_id: Uuid,
    pub policy_id: Uuid,
    pub policy_name: String,
    pub field: String,
    pub message: String,
    pub severity: String, // "warning" or "error"
}

// ============================================================================
// Budget Management (Oracle Fusion General Ledger > Budgets)
// ============================================================================

/// Budget definition (template)
/// Oracle Fusion: General Ledger > Budgets > Define Budget
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BudgetDefinition {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Budget code (e.g., '`FY2024_OPEx`', '`FY2024_CAPEx`')
    pub code: String,
    /// Display name
    pub name: String,
    pub description: Option<String>,
    /// Reference to accounting calendar
    pub calendar_id: Option<Uuid>,
    /// Fiscal year this budget covers
    pub fiscal_year: Option<i32>,
    /// Budget type: 'operating', 'capital', 'project', '`cash_flow`'
    pub budget_type: String,
    /// Control level: 'none', 'advisory', 'absolute'
    pub control_level: String,
    /// Whether carry-forward of unspent amounts is allowed
    pub allow_carry_forward: bool,
    /// Whether transfers between accounts are allowed
    pub allow_transfers: bool,
    /// Default currency
    pub currency_code: String,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create/update budget definition request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BudgetDefinitionRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub calendar_id: Option<Uuid>,
    pub fiscal_year: Option<i32>,
    #[serde(default = "default_budget_type")]
    pub budget_type: String,
    #[serde(default = "default_control_level")]
    pub control_level: String,
    #[serde(default)]
    pub allow_carry_forward: bool,
    #[serde(default = "default_true")]
    pub allow_transfers: bool,
    #[serde(default = "default_currency_usd")]
    pub currency_code: String,
}

pub fn default_budget_type() -> String {
    "operating".to_string()
}
pub fn default_control_level() -> String {
    "none".to_string()
}

/// Budget version (snapshot with workflow)
/// Oracle Fusion: General Ledger > Budgets > Budget Versions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BudgetVersion {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// The budget definition this version belongs to
    pub definition_id: Uuid,
    /// Auto-incremented version number
    pub version_number: i32,
    /// Version label (e.g., 'Original', 'Revised Q2')
    pub label: Option<String>,
    /// Status: 'draft', 'submitted', 'approved', 'active', 'closed', 'rejected'
    pub status: String,
    /// Totals (calculated from budget lines)
    pub total_budget_amount: String,
    pub total_committed_amount: String,
    pub total_actual_amount: String,
    pub total_variance_amount: String,
    /// Approval workflow
    pub submitted_by: Option<Uuid>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub rejected_reason: Option<String>,
    /// Effective dates
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    /// Notes
    pub notes: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create budget version request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BudgetVersionRequest {
    /// Budget definition code to create version for
    pub budget_code: String,
    pub label: Option<String>,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub notes: Option<String>,
}

/// Budget line (individual budget amount by account/period/dimension)
/// Oracle Fusion: General Ledger > Budgets > Enter Budget Amounts
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BudgetLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Budget version reference
    pub version_id: Uuid,
    /// Line number
    pub line_number: i32,
    /// Account reference
    pub account_code: String,
    pub account_name: Option<String>,
    /// Period reference
    pub period_name: Option<String>,
    pub period_start_date: Option<chrono::NaiveDate>,
    pub period_end_date: Option<chrono::NaiveDate>,
    pub fiscal_year: Option<i32>,
    pub quarter: Option<i32>,
    /// Dimension references
    pub department_id: Option<Uuid>,
    pub department_name: Option<String>,
    pub project_id: Option<Uuid>,
    pub project_name: Option<String>,
    pub cost_center: Option<String>,
    /// Budget amounts
    pub budget_amount: String,
    pub committed_amount: String,
    pub actual_amount: String,
    pub variance_amount: String,
    pub variance_percent: String,
    /// Carry-forward
    pub carry_forward_amount: String,
    /// Transfer tracking
    pub transferred_in_amount: String,
    pub transferred_out_amount: String,
    /// Description
    pub description: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create/update budget line request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BudgetLineRequest {
    pub account_code: String,
    pub account_name: Option<String>,
    pub period_name: Option<String>,
    pub period_start_date: Option<chrono::NaiveDate>,
    pub period_end_date: Option<chrono::NaiveDate>,
    pub fiscal_year: Option<i32>,
    pub quarter: Option<i32>,
    pub department_id: Option<Uuid>,
    pub department_name: Option<String>,
    pub project_id: Option<Uuid>,
    pub project_name: Option<String>,
    pub cost_center: Option<String>,
    /// The budgeted amount
    pub budget_amount: String,
    pub description: Option<String>,
}

/// Budget transfer
/// Oracle Fusion: General Ledger > Budgets > Transfer Budget Amounts
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BudgetTransfer {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Budget version reference
    pub version_id: Uuid,
    /// Transfer number
    pub transfer_number: String,
    pub description: Option<String>,
    /// Source account
    pub from_account_code: String,
    pub from_period_name: Option<String>,
    pub from_department_id: Option<Uuid>,
    pub from_cost_center: Option<String>,
    /// Destination account
    pub to_account_code: String,
    pub to_period_name: Option<String>,
    pub to_department_id: Option<Uuid>,
    pub to_cost_center: Option<String>,
    /// Amount to transfer
    pub amount: String,
    /// Status: 'pending', 'approved', 'rejected', 'cancelled'
    pub status: String,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub rejected_reason: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create budget transfer request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BudgetTransferRequest {
    pub budget_code: String,
    pub from_account_code: String,
    pub from_period_name: Option<String>,
    pub from_department_id: Option<Uuid>,
    pub from_cost_center: Option<String>,
    pub to_account_code: String,
    pub to_period_name: Option<String>,
    pub to_department_id: Option<Uuid>,
    pub to_cost_center: Option<String>,
    pub amount: String,
    pub description: Option<String>,
}

/// Budget vs Actuals variance report
/// Oracle Fusion: General Ledger > Budgets > Budget vs Actuals Report
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BudgetVarianceReport {
    pub definition_id: Uuid,
    pub definition_code: String,
    pub definition_name: String,
    pub version_id: Uuid,
    pub version_label: Option<String>,
    pub fiscal_year: Option<i32>,
    /// Summary totals
    pub total_budget: String,
    pub total_actual: String,
    pub total_committed: String,
    pub total_variance: String,
    pub variance_percent: String,
    /// Line-by-line details
    pub lines: Vec<BudgetVarianceLine>,
}

/// Single line in the budget variance report
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BudgetVarianceLine {
    pub account_code: String,
    pub account_name: Option<String>,
    pub period_name: Option<String>,
    pub department_name: Option<String>,
    pub project_name: Option<String>,
    pub cost_center: Option<String>,
    pub budget_amount: String,
    pub committed_amount: String,
    pub actual_amount: String,
    pub variance_amount: String,
    pub variance_percent: String,
    /// Whether this line is over budget
    pub is_over_budget: bool,
}

// ============================================================================
// Collections & Credit Management (Oracle Fusion Collections)
// ============================================================================

/// Customer credit profile
/// Oracle Fusion: Collections > Customer Credit Profiles
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomerCreditProfile {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub customer_id: Uuid,
    pub customer_number: Option<String>,
    pub customer_name: Option<String>,
    /// Credit limit amount
    pub credit_limit: String,
    /// Currently used credit
    pub credit_used: String,
    /// Available credit (limit - used)
    pub credit_available: String,
    /// Risk classification: 'low', 'medium', 'high', '`very_high`', 'defaulted'
    pub risk_classification: String,
    /// Internal credit score (0-1000)
    pub credit_score: Option<i32>,
    /// External credit rating
    pub external_credit_rating: Option<String>,
    pub external_rating_agency: Option<String>,
    pub external_rating_date: Option<chrono::NaiveDate>,
    /// Default payment terms
    pub payment_terms: String,
    /// Average days to pay
    pub average_days_to_pay: Option<String>,
    /// Overdue invoice count
    pub overdue_invoice_count: i32,
    /// Total overdue amount
    pub total_overdue_amount: String,
    /// Oldest overdue date
    pub oldest_overdue_date: Option<chrono::NaiveDate>,
    /// Whether customer is on credit hold
    pub credit_hold: bool,
    pub credit_hold_reason: Option<String>,
    pub credit_hold_date: Option<DateTime<Utc>>,
    pub credit_hold_by: Option<Uuid>,
    /// Review dates
    pub last_review_date: Option<chrono::NaiveDate>,
    pub next_review_date: Option<chrono::NaiveDate>,
    /// 'active', 'inactive', 'blocked'
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create/update credit profile request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreditProfileRequest {
    pub customer_id: Uuid,
    pub credit_limit: String,
    #[serde(default = "default_risk_medium")]
    pub risk_classification: String,
    pub credit_score: Option<i32>,
    pub external_credit_rating: Option<String>,
    pub external_rating_agency: Option<String>,
    pub external_rating_date: Option<chrono::NaiveDate>,
    #[serde(default = "default_net_30")]
    pub payment_terms: String,
    pub next_review_date: Option<chrono::NaiveDate>,
}

pub fn default_risk_medium() -> String {
    "medium".to_string()
}
pub fn default_net_30() -> String {
    "net_30".to_string()
}

/// Collection strategy definition
/// Oracle Fusion: Collections > Collection Strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionStrategy {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    /// 'automatic' or 'manual'
    pub strategy_type: String,
    /// Applicable risk classifications
    pub applicable_risk_classifications: serde_json::Value,
    /// Aging buckets that trigger this strategy
    pub trigger_aging_buckets: serde_json::Value,
    /// Overdue amount threshold
    pub overdue_amount_threshold: String,
    /// Ordered collection actions
    pub actions: serde_json::Value,
    /// Priority
    pub priority: i32,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Collection case
/// Oracle Fusion: Collections > Collection Cases
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionCase {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub case_number: String,
    pub customer_id: Uuid,
    pub customer_number: Option<String>,
    pub customer_name: Option<String>,
    pub strategy_id: Option<Uuid>,
    /// Assigned collector
    pub assigned_to: Option<Uuid>,
    pub assigned_to_name: Option<String>,
    /// 'collection', 'dispute', 'bankruptcy', '`skip_trace`'
    pub case_type: String,
    /// 'open', '`in_progress`', 'resolved', 'closed', 'escalated', '`written_off`'
    pub status: String,
    /// 'low', 'medium', 'high', 'critical'
    pub priority: String,
    /// Financial summary
    pub total_overdue_amount: String,
    pub total_disputed_amount: String,
    pub total_invoiced_amount: String,
    pub overdue_invoice_count: i32,
    pub oldest_overdue_date: Option<chrono::NaiveDate>,
    /// Current strategy step
    pub current_step: i32,
    /// Key dates
    pub opened_date: chrono::NaiveDate,
    pub target_resolution_date: Option<chrono::NaiveDate>,
    pub resolved_date: Option<chrono::NaiveDate>,
    pub closed_date: Option<chrono::NaiveDate>,
    pub last_action_date: Option<chrono::NaiveDate>,
    pub next_action_date: Option<chrono::NaiveDate>,
    /// Resolution
    pub resolution_type: Option<String>,
    pub resolution_notes: Option<String>,
    /// Related invoices
    pub related_invoice_ids: serde_json::Value,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Customer interaction record
/// Oracle Fusion: Collections > Customer Interactions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomerInteraction {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub case_id: Option<Uuid>,
    pub customer_id: Uuid,
    pub customer_number: Option<String>,
    pub customer_name: Option<String>,
    /// '`phone_call`', 'email', 'letter', 'meeting', 'note', 'sms'
    pub interaction_type: String,
    /// 'outbound', 'inbound'
    pub direction: String,
    pub contact_name: Option<String>,
    pub contact_role: Option<String>,
    pub contact_phone: Option<String>,
    pub contact_email: Option<String>,
    pub subject: Option<String>,
    pub body: Option<String>,
    /// Outcome: 'contacted', '`left_message`', '`no_answer`', '`promised_to_pay`',
    /// 'disputed', 'refused', '`agreed_payment_plan`', 'escalated', '`no_action`'
    pub outcome: Option<String>,
    pub follow_up_date: Option<chrono::NaiveDate>,
    pub follow_up_notes: Option<String>,
    pub performed_by: Option<Uuid>,
    pub performed_by_name: Option<String>,
    pub performed_at: Option<DateTime<Utc>>,
    pub duration_minutes: Option<i32>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Promise to pay
/// Oracle Fusion: Collections > Promises to Pay
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromiseToPay {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub case_id: Option<Uuid>,
    pub customer_id: Uuid,
    pub customer_number: Option<String>,
    pub customer_name: Option<String>,
    /// '`single_payment`', 'installment', '`full_balance`'
    pub promise_type: String,
    pub promised_amount: String,
    pub paid_amount: String,
    pub remaining_amount: String,
    pub promise_date: chrono::NaiveDate,
    pub installment_count: Option<i32>,
    pub installment_frequency: Option<String>,
    /// 'pending', '`partially_kept`', 'kept', 'broken', 'cancelled'
    pub status: String,
    pub broken_date: Option<chrono::NaiveDate>,
    pub broken_reason: Option<String>,
    pub related_invoice_ids: serde_json::Value,
    pub promised_by_name: Option<String>,
    pub promised_by_role: Option<String>,
    pub notes: Option<String>,
    pub recorded_by: Option<Uuid>,
    pub recorded_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Dunning campaign
/// Oracle Fusion: Collections > Dunning Management
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DunningCampaign {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub campaign_number: String,
    pub name: String,
    pub description: Option<String>,
    /// 'reminder', '`first_notice`', '`second_notice`', '`final_notice`', '`pre_legal`', 'legal'
    pub dunning_level: String,
    /// 'email', 'letter', 'sms', 'phone'
    pub communication_method: String,
    pub template_id: Option<Uuid>,
    pub template_name: Option<String>,
    pub min_overdue_days: i32,
    pub min_overdue_amount: String,
    pub target_risk_classifications: serde_json::Value,
    pub exclude_active_cases: bool,
    pub scheduled_date: Option<chrono::NaiveDate>,
    pub sent_date: Option<chrono::NaiveDate>,
    pub target_customer_count: i32,
    pub sent_count: i32,
    pub failed_count: i32,
    /// 'draft', 'scheduled', '`in_progress`', 'completed', 'cancelled'
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Dunning letter (individual)
/// Oracle Fusion: Collections > Dunning Letters
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DunningLetter {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub campaign_id: Option<Uuid>,
    pub customer_id: Uuid,
    pub customer_number: Option<String>,
    pub customer_name: Option<String>,
    pub customer_address: Option<serde_json::Value>,
    pub customer_email: Option<String>,
    pub dunning_level: String,
    pub communication_method: String,
    pub total_overdue_amount: String,
    pub overdue_invoice_count: i32,
    pub oldest_overdue_date: Option<chrono::NaiveDate>,
    /// Aging breakdown
    pub aging_current: String,
    pub aging_1_30: String,
    pub aging_31_60: String,
    pub aging_61_90: String,
    pub aging_91_120: String,
    pub aging_121_plus: String,
    /// 'pending', 'sent', 'delivered', 'bounced', 'failed', 'viewed'
    pub status: String,
    pub sent_at: Option<DateTime<Utc>>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub viewed_at: Option<DateTime<Utc>>,
    pub failure_reason: Option<String>,
    pub invoice_details: serde_json::Value,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Receivables aging snapshot
/// Oracle Fusion: Collections > Aging Analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReceivablesAgingSnapshot {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub snapshot_date: chrono::NaiveDate,
    pub customer_id: Uuid,
    pub customer_number: Option<String>,
    pub customer_name: Option<String>,
    pub total_outstanding: String,
    /// Aging buckets
    pub aging_current: String,
    pub aging_1_30: String,
    pub aging_31_60: String,
    pub aging_61_90: String,
    pub aging_91_120: String,
    pub aging_121_plus: String,
    /// Counts per bucket
    pub count_current: i32,
    pub count_1_30: i32,
    pub count_31_60: i32,
    pub count_61_90: i32,
    pub count_91_120: i32,
    pub count_121_plus: i32,
    pub weighted_average_days_overdue: Option<String>,
    pub overdue_percent: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// Write-off request
/// Oracle Fusion: Collections > Write-Off Management
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WriteOffRequest {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub request_number: String,
    pub customer_id: Uuid,
    pub customer_number: Option<String>,
    pub customer_name: Option<String>,
    /// '`bad_debt`', '`small_balance`', 'dispute', 'adjustment'
    pub write_off_type: String,
    pub write_off_amount: String,
    pub write_off_account_code: Option<String>,
    pub reason: String,
    pub related_invoice_ids: serde_json::Value,
    pub case_id: Option<Uuid>,
    /// 'draft', 'submitted', 'approved', 'rejected', 'processed', 'cancelled'
    pub status: String,
    pub submitted_by: Option<Uuid>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub rejected_reason: Option<String>,
    pub journal_entry_id: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Aging summary report
/// Oracle Fusion: Collections > Aging Report
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgingSummary {
    pub organization_id: Uuid,
    pub as_of_date: chrono::NaiveDate,
    pub total_outstanding: String,
    pub total_overdue: String,
    pub aging_current: String,
    pub aging_1_30: String,
    pub aging_31_60: String,
    pub aging_61_90: String,
    pub aging_91_120: String,
    pub aging_121_plus: String,
    pub customer_count: i32,
    pub overdue_customer_count: i32,
    pub weighted_average_days_overdue: String,
}

// ============================================================================
// Financial Reporting (Oracle Fusion GL > Financial Reporting Center)
// ============================================================================

/// Report template types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FinancialReportType {
    TrialBalance,
    IncomeStatement,
    BalanceSheet,
    CashFlow,
    Custom,
}

impl std::fmt::Display for FinancialReportType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TrialBalance => write!(f, "trial_balance"),
            Self::IncomeStatement => write!(f, "income_statement"),
            Self::BalanceSheet => write!(f, "balance_sheet"),
            Self::CashFlow => write!(f, "cash_flow"),
            Self::Custom => write!(f, "custom"),
        }
    }
}

/// Financial Report Template definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinancialReportTemplate {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    /// `trial_balance`, `income_statement`, `balance_sheet`, `cash_flow`, custom
    pub report_type: String,
    pub currency_code: String,
    /// sequential, tree, grouped
    pub row_display_order: String,
    pub column_display_order: String,
    /// none, thousands, millions, units
    pub rounding_option: String,
    pub show_zero_amounts: bool,
    pub segment_filter: serde_json::Value,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Report Row definition (a line on the report)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinancialReportRow {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub template_id: Uuid,
    pub row_number: i32,
    /// header, data, total, subtotal, separator, text
    pub line_type: String,
    pub label: String,
    pub indent_level: i32,
    /// Account range filter for data rows
    pub account_range_from: Option<String>,
    pub account_range_to: Option<String>,
    pub account_filter: serde_json::Value,
    /// Compute action: total, subtotal, variance, percent, constant
    pub compute_action: Option<String>,
    /// Row IDs to use for computation
    pub compute_source_rows: serde_json::Value,
    pub show_line: bool,
    pub bold: bool,
    pub underline: bool,
    pub double_underline: bool,
    pub page_break_before: bool,
    pub scaling_factor: Option<String>,
    pub parent_row_id: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Report Column definition (period/scenario columns across the top)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinancialReportColumn {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub template_id: Uuid,
    pub column_number: i32,
    /// actuals, budget, variance, `percent_variance`, `prior_year`, ytd, qtd, custom
    pub column_type: String,
    pub header_label: String,
    pub sub_header_label: Option<String>,
    /// Period offset from the base period (-1 = prior period, 0 = current)
    pub period_offset: i32,
    /// period, qtd, ytd, `inception_to_date`
    pub period_type: String,
    /// Compute action for calculated columns
    pub compute_action: Option<String>,
    pub compute_source_columns: serde_json::Value,
    pub show_column: bool,
    pub column_width: Option<i32>,
    pub format_override: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Financial Report Run (an execution instance of a template)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinancialReportRun {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub template_id: Uuid,
    pub run_number: String,
    pub name: Option<String>,
    pub description: Option<String>,
    /// draft, generated, approved, published, archived
    pub status: String,
    pub as_of_date: Option<chrono::NaiveDate>,
    pub period_from: Option<chrono::NaiveDate>,
    pub period_to: Option<chrono::NaiveDate>,
    pub currency_code: String,
    pub segment_filter: serde_json::Value,
    pub include_unposted: bool,
    pub total_debit: String,
    pub total_credit: String,
    pub net_change: String,
    pub beginning_balance: String,
    pub ending_balance: String,
    pub row_count: i32,
    pub generated_by: Option<Uuid>,
    pub generated_at: Option<DateTime<Utc>>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub published_by: Option<Uuid>,
    pub published_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A single cell in a generated report
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinancialReportResult {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub run_id: Uuid,
    pub row_id: Uuid,
    pub column_id: Uuid,
    pub row_number: i32,
    pub column_number: i32,
    pub amount: String,
    pub debit_amount: String,
    pub credit_amount: String,
    pub beginning_balance: String,
    pub ending_balance: String,
    pub is_computed: bool,
    pub compute_note: Option<String>,
    pub display_amount: Option<String>,
    pub display_format: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// Favourite report (user bookmark)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinancialReportFavourite {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub user_id: Uuid,
    pub template_id: Uuid,
    pub display_name: Option<String>,
    pub position: i32,
    pub created_at: DateTime<Utc>,
}

/// Financial Reporting Dashboard summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinancialReportingSummary {
    pub template_count: i32,
    pub active_template_count: i32,
    pub run_count: i32,
    pub recent_runs: Vec<FinancialReportRun>,
    pub templates_by_type: serde_json::Value,
    pub total_amount_reported: String,
}

// ════════════════════════════════════════════════════════════════════════════════
// Withholding Tax Management (Oracle Fusion Payables > Withholding Tax)
// ════════════════════════════════════════════════════════════════════════════════
//
// Oracle Fusion Cloud ERP Withholding Tax provides:
// - Withholding Tax Codes: Define individual withholding tax types with rates
// - Withholding Tax Groups: Group multiple tax codes into reusable sets
// - Supplier Assignments: Assign withholding tax groups to suppliers
// - Withholding Thresholds: Minimum amounts before withholding applies
// - Automatic Computation: Calculate withholding amounts during payment
// - Withholding Certificates: Track and report withheld taxes
// - Exemptions: Manage supplier exemptions from withholding
//
// Oracle Fusion equivalent: Financials > Payables > Withholding Tax

/// Withholding Tax Code definition
/// Defines an individual withholding tax type with its rate and account.
/// Oracle Fusion equivalent: Payables > Withholding Tax > Tax Codes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WithholdingTaxCode {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Unique code (e.g., "`FEDERAL_WHT`", "`STATE_WHT`", "`VAT_WHT`")
    pub code: String,
    /// Display name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Tax type: "`income_tax`", "vat", "`service_tax`", "`contract_tax`", "royalty",
    ///           "dividend", "interest", "other"
    pub tax_type: String,
    /// Withholding rate percentage
    pub rate_percentage: String,
    /// Minimum threshold amount below which no withholding applies
    pub threshold_amount: String,
    /// Whether the threshold is cumulative (year-to-date) or per-invoice
    pub threshold_is_cumulative: bool,
    /// GL account code for the withholding liability
    pub withholding_account_code: Option<String>,
    /// GL account code for the withholding expense
    pub expense_account_code: Option<String>,
    /// Whether this tax code is active
    pub is_active: bool,
    /// Effective dates
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    /// Audit
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create/update withholding tax code request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WithholdingTaxCodeRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(default = "default_wht_tax_type")]
    pub tax_type: String,
    pub rate_percentage: String,
    #[serde(default = "default_zero_str")]
    pub threshold_amount: String,
    #[serde(default)]
    pub threshold_is_cumulative: bool,
    pub withholding_account_code: Option<String>,
    pub expense_account_code: Option<String>,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
}

pub fn default_wht_tax_type() -> String {
    "income_tax".to_string()
}
pub fn default_zero_str() -> String {
    "0".to_string()
}

/// Withholding Tax Group
/// Groups multiple withholding tax codes into a reusable set assignable to suppliers.
/// Oracle Fusion equivalent: Payables > Withholding Tax > Tax Groups
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WithholdingTaxGroup {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Unique group code (e.g., "`STD_WHT`", "`CONTRACTOR_WHT`")
    pub code: String,
    /// Display name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Member tax codes
    pub tax_codes: Vec<WithholdingTaxGroupMember>,
    /// Whether this group is active
    pub is_active: bool,
    /// Audit
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Member of a withholding tax group
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WithholdingTaxGroupMember {
    pub id: Uuid,
    pub group_id: Uuid,
    /// Reference to the withholding tax code
    pub tax_code_id: Uuid,
    /// Tax code (denormalized)
    pub tax_code: String,
    /// Tax code name (denormalized)
    pub tax_code_name: String,
    /// Optional rate override percentage (overrides the tax code default)
    pub rate_override: Option<String>,
    /// Whether this member is active in the group
    pub is_active: bool,
    /// Display order
    pub display_order: i32,
}

/// Create withholding tax group request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WithholdingTaxGroupRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    /// Tax code IDs to include in the group
    pub tax_code_ids: Vec<Uuid>,
}

/// Supplier Withholding Tax Assignment
/// Links a supplier to a withholding tax group.
/// Oracle Fusion equivalent: Payables > Suppliers > Withholding Tax
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupplierWithholdingAssignment {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Supplier reference
    pub supplier_id: Uuid,
    /// Supplier number (denormalized)
    pub supplier_number: Option<String>,
    /// Supplier name (denormalized)
    pub supplier_name: Option<String>,
    /// Assigned withholding tax group
    pub tax_group_id: Uuid,
    /// Tax group code (denormalized)
    pub tax_group_code: String,
    /// Tax group name (denormalized)
    pub tax_group_name: String,
    /// Whether the supplier is exempt from withholding
    pub is_exempt: bool,
    /// Exemption reason (if exempt)
    pub exemption_reason: Option<String>,
    /// Exemption certificate number
    pub exemption_certificate: Option<String>,
    /// Exemption valid until
    pub exemption_valid_until: Option<chrono::NaiveDate>,
    /// Whether this assignment is active
    pub is_active: bool,
    /// Audit
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create/update supplier withholding assignment request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupplierWithholdingAssignmentRequest {
    pub supplier_id: Uuid,
    pub tax_group_code: String,
    pub is_exempt: Option<bool>,
    pub exemption_reason: Option<String>,
    pub exemption_certificate: Option<String>,
    pub exemption_valid_until: Option<chrono::NaiveDate>,
}

/// Withholding Tax Certificate
/// Certificate issued for tax withheld from a supplier payment.
/// Oracle Fusion equivalent: Payables > Withholding Tax > Certificates
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WithholdingCertificate {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Auto-generated certificate number
    pub certificate_number: String,
    /// Supplier information
    pub supplier_id: Uuid,
    pub supplier_number: Option<String>,
    pub supplier_name: Option<String>,
    /// Tax type (from the tax code)
    pub tax_type: String,
    /// Tax code reference
    pub tax_code_id: Uuid,
    pub tax_code: String,
    /// Period the certificate covers
    pub period_start: chrono::NaiveDate,
    pub period_end: chrono::NaiveDate,
    /// Amounts
    pub total_invoice_amount: String,
    pub total_withheld_amount: String,
    /// Rate applied
    pub rate_percentage: String,
    /// Payment references covered by this certificate
    pub payment_ids: serde_json::Value,
    /// Status: "draft", "issued", "acknowledged", "cancelled"
    pub status: String,
    /// Date the certificate was issued
    pub issued_at: Option<DateTime<Utc>>,
    /// Date acknowledged by supplier
    pub acknowledged_at: Option<DateTime<Utc>>,
    /// Notes
    pub notes: Option<String>,
    /// Metadata
    pub metadata: serde_json::Value,
    /// Audit
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Withholding Tax Line (computed withholding on a payment)
/// Records the actual tax withheld from a specific payment.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WithholdingTaxLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Payment reference
    pub payment_id: Uuid,
    pub payment_number: Option<String>,
    /// Invoice reference
    pub invoice_id: Uuid,
    pub invoice_number: Option<String>,
    /// Supplier information
    pub supplier_id: Uuid,
    pub supplier_name: Option<String>,
    /// Tax code applied
    pub tax_code_id: Uuid,
    pub tax_code: String,
    pub tax_code_name: Option<String>,
    /// Tax type
    pub tax_type: String,
    /// Rate applied
    pub rate_percentage: String,
    /// Amounts
    pub taxable_amount: String,
    pub withheld_amount: String,
    /// GL account
    pub withholding_account_code: Option<String>,
    /// Status: "pending", "withheld", "remitted", "refunded"
    pub status: String,
    /// Remittance tracking
    pub remittance_date: Option<chrono::NaiveDate>,
    pub remittance_reference: Option<String>,
    /// Metadata
    pub metadata: serde_json::Value,
    /// Audit
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Result of a withholding tax computation for a payment
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WithholdingComputationResult {
    /// The supplier's tax group (if assigned)
    pub tax_group_code: Option<String>,
    /// Whether the supplier is exempt
    pub is_exempt: bool,
    /// Individual withholding lines computed
    pub lines: Vec<WithholdingComputedLine>,
    /// Total amount subject to withholding
    pub total_taxable_amount: String,
    /// Total withholding amount
    pub total_withheld_amount: String,
    /// Net payment amount (after withholding)
    pub net_payment_amount: String,
}

/// Single computed withholding line
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WithholdingComputedLine {
    pub tax_code_id: Uuid,
    pub tax_code: String,
    pub tax_type: String,
    pub rate_percentage: String,
    pub threshold_amount: String,
    pub taxable_amount: String,
    pub withheld_amount: String,
    pub withholding_account_code: Option<String>,
    /// Whether withholding was skipped due to threshold
    pub threshold_applied: bool,
}

/// Withholding Tax Dashboard Summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WithholdingSummary {
    /// Total active tax codes
    pub active_tax_code_count: i32,
    /// Total tax groups
    pub tax_group_count: i32,
    /// Total assigned suppliers
    pub assigned_supplier_count: i32,
    /// Total exempt suppliers
    pub exempt_supplier_count: i32,
    /// Total withheld (period)
    pub total_withheld_amount: String,
    /// Total remitted (period)
    pub total_remitted_amount: String,
    /// Total pending remittance
    pub total_pending_remittance: String,
    /// Withholding by tax type
    pub by_tax_type: serde_json::Value,
    /// Withholding by supplier (top suppliers)
    pub by_supplier: serde_json::Value,
    /// Recent certificates
    pub certificates_issued: i32,
}

// ============================================================================
// Multi-Book Accounting (Secondary Ledgers)
// Oracle Fusion equivalent: General Ledger > Multi-Book Accounting
// ============================================================================

/// Accounting Book (Primary or Secondary)
/// Represents a complete accounting representation with its own chart of accounts,
/// calendar, and currency. Primary book is the main ledger; secondary books
/// represent alternate accounting standards (e.g., IFRS, local GAAP, statutory).
/// Oracle Fusion equivalent: General Ledger > Accounting Books
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountingBook {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Unique book code (e.g., "`PRIMARY_GAAP`", "`IFRS_BOOK`", "`LOCAL_STATUTORY`")
    pub code: String,
    /// Display name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Book type: "primary" or "secondary"
    pub book_type: String,
    /// Chart of accounts identifier / code
    pub chart_of_accounts_code: String,
    /// Accounting calendar code (references period-close calendars)
    pub calendar_code: String,
    /// Base currency code for this book
    pub currency_code: String,
    /// Whether this book is enabled for posting
    pub is_enabled: bool,
    /// Whether auto-propagation from primary is enabled (secondary books only)
    pub auto_propagation_enabled: bool,
    /// Mapping level: "journal" or "subledger"
    pub mapping_level: String,
    /// Status: "draft", "active", "inactive", "suspended"
    pub status: String,
    /// Metadata
    pub metadata: serde_json::Value,
    /// Audit
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create/update accounting book request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountingBookRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub book_type: String,
    pub chart_of_accounts_code: String,
    pub calendar_code: String,
    pub currency_code: String,
    pub auto_propagation_enabled: Option<bool>,
    pub mapping_level: Option<String>,
}

/// Account Mapping Rule
/// Maps account segments from a source book to a target book.
/// Oracle Fusion equivalent: General Ledger > Multi-Book > Account Mappings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountMapping {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Source accounting book ID
    pub source_book_id: Uuid,
    /// Target accounting book ID
    pub target_book_id: Uuid,
    /// Source account code / range
    pub source_account_code: String,
    /// Target account code
    pub target_account_code: String,
    /// Optional segment-level mappings (JSON: {"`segment_name"`: "value"})
    pub segment_mappings: serde_json::Value,
    /// Priority (lower = higher priority)
    pub priority: i32,
    /// Whether this rule is active
    pub is_active: bool,
    /// Effective dates
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    /// Audit
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create account mapping request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountMappingRequest {
    pub source_book_id: Uuid,
    pub target_book_id: Uuid,
    pub source_account_code: String,
    pub target_account_code: String,
    pub segment_mappings: Option<serde_json::Value>,
    pub priority: Option<i32>,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
}

/// Book Journal Entry
/// A journal entry in a specific accounting book, either posted directly
/// or propagated from another book.
/// Oracle Fusion equivalent: General Ledger > Multi-Book > Journal Entries
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BookJournalEntry {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// The accounting book this entry belongs to
    pub book_id: Uuid,
    /// Journal entry number (auto-generated within the book)
    pub entry_number: String,
    /// Journal header description
    pub header_description: Option<String>,
    /// Source book ID (if propagated)
    pub source_book_id: Option<Uuid>,
    /// Source journal entry ID (if propagated)
    pub source_entry_id: Option<Uuid>,
    /// External reference (e.g., subledger transaction ID)
    pub external_reference: Option<String>,
    /// Accounting date
    pub accounting_date: chrono::NaiveDate,
    /// Period name
    pub period_name: Option<String>,
    /// Total debit amount
    pub total_debit: String,
    /// Total credit amount
    pub total_credit: String,
    /// Status: "draft", "posted", "propagated", "reversed"
    pub status: String,
    /// Whether this was auto-propagated
    pub is_auto_propagated: bool,
    /// Currency
    pub currency_code: String,
    /// Conversion rate (if different from source book currency)
    pub conversion_rate: Option<String>,
    /// Metadata
    pub metadata: serde_json::Value,
    /// Audit
    pub created_by: Option<Uuid>,
    pub posted_by: Option<Uuid>,
    pub posted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Book Journal Line
/// Individual debit/credit line within a book journal entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BookJournalLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Parent journal entry
    pub entry_id: Uuid,
    /// Line number within the entry
    pub line_number: i32,
    /// Account code in this book's chart of accounts
    pub account_code: String,
    /// Account name (denormalized)
    pub account_name: Option<String>,
    /// Debit amount
    pub debit_amount: String,
    /// Credit amount
    pub credit_amount: String,
    /// Description
    pub description: Option<String>,
    /// Tax code
    pub tax_code: Option<String>,
    /// Source line ID (if propagated)
    pub source_line_id: Option<Uuid>,
    /// Metadata
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create book journal entry request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BookJournalEntryRequest {
    pub book_id: Uuid,
    pub header_description: Option<String>,
    pub external_reference: Option<String>,
    pub accounting_date: chrono::NaiveDate,
    pub period_name: Option<String>,
    pub currency_code: String,
    pub lines: Vec<BookJournalLineRequest>,
}

/// Create book journal line request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BookJournalLineRequest {
    pub account_code: String,
    pub account_name: Option<String>,
    pub debit_amount: String,
    pub credit_amount: String,
    pub description: Option<String>,
    pub tax_code: Option<String>,
}

/// Propagation Log Entry
/// Tracks the propagation of journal entries between books.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PropagationLog {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Source book
    pub source_book_id: Uuid,
    /// Target book
    pub target_book_id: Uuid,
    /// Source journal entry
    pub source_entry_id: Uuid,
    /// Created target journal entry
    pub target_entry_id: Option<Uuid>,
    /// Status: "pending", "completed", "failed", "skipped"
    pub status: String,
    /// Number of lines propagated
    pub lines_propagated: i32,
    /// Number of lines unmapped (skipped)
    pub lines_unmapped: i32,
    /// Error message (if failed)
    pub error_message: Option<String>,
    /// Propagation timestamp
    pub propagated_at: DateTime<Utc>,
    /// Metadata
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// Multi-Book Dashboard Summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MultiBookSummary {
    /// Total accounting books
    pub book_count: i32,
    /// Primary book code
    pub primary_book_code: Option<String>,
    /// Count of secondary books
    pub secondary_book_count: i32,
    /// Active mapping rules count
    pub mapping_rule_count: i32,
    /// Recent propagations count
    pub recent_propagation_count: i32,
    /// Propagation success rate
    pub propagation_success_rate: String,
    /// Unposted entries by book
    pub unposted_entries_by_book: serde_json::Value,
    /// Journal entry counts by book
    pub entry_counts_by_book: serde_json::Value,
}

// ============================================================================
// Financial Consolidation (Oracle Fusion General Ledger > Consolidation)
// ============================================================================

/// Consolidation Ledger - defines a consolidation scope with translation method.
/// Oracle Fusion: General Ledger > Consolidation > Consolidation Ledgers
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsolidationLedger {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub base_currency_code: String,
    /// "`current_rate`", "temporal", "`weighted_average`"
    pub translation_method: String,
    /// "full", "proportional", "`equity_method`"
    pub equity_elimination_method: String,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Consolidation Entity - a subsidiary / BU participating in consolidation.
/// Oracle Fusion: Consolidation > Consolidation Entities
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsolidationEntity {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub ledger_id: Uuid,
    pub entity_id: Uuid,
    pub entity_name: String,
    pub entity_code: String,
    pub local_currency_code: String,
    pub ownership_percentage: String,
    /// "full", "proportional", "`equity_method`"
    pub consolidation_method: String,
    pub is_active: bool,
    pub include_in_consolidation: bool,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Consolidation Scenario - a periodic consolidation run.
/// Oracle Fusion: Consolidation > Consolidation Workbench
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsolidationScenario {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub ledger_id: Uuid,
    pub scenario_number: String,
    pub name: String,
    pub description: Option<String>,
    pub fiscal_year: i32,
    pub period_name: String,
    pub period_start_date: chrono::NaiveDate,
    pub period_end_date: chrono::NaiveDate,
    /// "draft", "`in_progress`", "`pending_review`", "approved", "posted", "reversed"
    pub status: String,
    pub translation_date: Option<chrono::NaiveDate>,
    pub translation_rate_type: Option<String>,
    pub total_entities: i32,
    pub total_eliminations: i32,
    pub total_adjustments: i32,
    pub total_debits: String,
    pub total_credits: String,
    pub is_balanced: bool,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub posted_by: Option<Uuid>,
    pub posted_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Consolidation Trial Balance Line.
/// Oracle Fusion: Consolidated Trial Balance report
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsolidationTrialBalanceLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub scenario_id: Uuid,
    pub entity_id: Option<Uuid>,
    pub entity_code: Option<String>,
    pub account_code: String,
    pub account_name: Option<String>,
    pub account_type: Option<String>,
    pub financial_statement: Option<String>,
    pub local_debit: String,
    pub local_credit: String,
    pub local_balance: String,
    pub exchange_rate: Option<String>,
    pub translated_debit: String,
    pub translated_credit: String,
    pub translated_balance: String,
    pub elimination_debit: String,
    pub elimination_credit: String,
    pub elimination_balance: String,
    pub minority_interest_debit: String,
    pub minority_interest_credit: String,
    pub minority_interest_balance: String,
    pub consolidated_debit: String,
    pub consolidated_credit: String,
    pub consolidated_balance: String,
    pub is_elimination_entry: bool,
    /// "entity", "elimination", "adjustment", "minority", "consolidated"
    pub line_type: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Intercompany Elimination Rule.
/// Oracle Fusion: Consolidation > Elimination Rules
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsolidationEliminationRule {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub ledger_id: Uuid,
    pub rule_code: String,
    pub name: String,
    pub description: Option<String>,
    /// "`intercompany_receivable_payable`", "`intercompany_revenue_expense`",
    /// "`investment_equity`", "`intercompany_inventory_profit`", "other"
    pub elimination_type: String,
    pub from_entity_id: Option<Uuid>,
    pub to_entity_id: Option<Uuid>,
    pub from_account_pattern: Option<String>,
    pub to_account_pattern: Option<String>,
    pub offset_account_code: String,
    pub priority: i32,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Consolidation Adjustment - manual journal adjustment within a scenario.
/// Oracle Fusion: Consolidation > Adjustments
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsolidationAdjustment {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub scenario_id: Uuid,
    pub adjustment_number: String,
    pub description: Option<String>,
    pub account_code: String,
    pub account_name: Option<String>,
    pub entity_id: Option<Uuid>,
    pub entity_code: Option<String>,
    pub debit: String,
    pub credit: String,
    /// "manual", "reclassification", "correction"
    pub adjustment_type: String,
    pub reference: Option<String>,
    /// "draft", "approved", "posted"
    pub status: String,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Consolidation Currency Translation Rate.
/// Oracle Fusion: Consolidation > Translation Rates
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsolidationTranslationRate {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub scenario_id: Uuid,
    pub entity_id: Uuid,
    pub from_currency: String,
    pub to_currency: String,
    /// "`period_end`", "average", "historical", "spot"
    pub rate_type: String,
    pub exchange_rate: String,
    pub effective_date: chrono::NaiveDate,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// Financial Consolidation Dashboard Summary.
/// Oracle Fusion: Consolidation > Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsolidationDashboardSummary {
    pub total_ledgers: i32,
    pub total_active_scenarios: i32,
    pub total_entities: i32,
    pub total_elimination_rules: i32,
    pub last_consolidation_date: Option<String>,
    pub last_consolidation_status: Option<String>,
    pub scenarios_by_status: serde_json::Value,
    pub entities_by_method: serde_json::Value,
    pub consolidation_completion_percent: String,
}

// ============================================================================
// Recurring Journals (Oracle Fusion GL > Recurring Journals)
// ============================================================================

/// Recurring journal schedule definition.
/// Oracle Fusion: General Ledger > Journals > Recurring Journals
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecurringJournalSchedule {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub schedule_number: String,
    pub name: String,
    pub description: Option<String>,
    pub recurrence_type: String, // daily, weekly, monthly, quarterly, semi_annual, annual
    pub journal_type: String,    // standard, skeleton, incremental
    pub currency_code: String,
    pub status: String, // draft, active, inactive
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub last_generation_date: Option<chrono::NaiveDate>,
    pub next_generation_date: Option<chrono::NaiveDate>,
    pub total_generations: i32,
    pub incremental_percent: Option<String>,
    pub auto_post: bool,
    pub reversal_method: Option<String>,
    pub ledger_id: Option<Uuid>,
    pub journal_category: Option<String>,
    pub reference_template: Option<String>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub created_by: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A template line within a recurring journal schedule.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecurringJournalScheduleLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub schedule_id: Uuid,
    pub line_number: i32,
    pub line_type: String, // debit, credit
    pub account_code: String,
    pub account_name: Option<String>,
    pub description: Option<String>,
    pub amount: String,
    pub currency_code: String,
    pub tax_code: Option<String>,
    pub cost_center: Option<String>,
    pub department_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A single generation run of a recurring journal.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecurringJournalGeneration {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub schedule_id: Uuid,
    pub generation_number: i32,
    pub journal_entry_id: Option<Uuid>,
    pub journal_entry_number: Option<String>,
    pub generation_date: chrono::NaiveDate,
    pub period_name: Option<String>,
    pub total_debit: String,
    pub total_credit: String,
    pub line_count: i32,
    pub status: String, // generated, posted, reversed, cancelled
    pub reversal_entry_id: Option<Uuid>,
    pub reversed_at: Option<DateTime<Utc>>,
    pub posted_at: Option<DateTime<Utc>>,
    pub generated_by: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A single generated journal line.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecurringJournalGenerationLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub generation_id: Uuid,
    pub schedule_line_id: Option<Uuid>,
    pub line_number: i32,
    pub line_type: String,
    pub account_code: String,
    pub account_name: Option<String>,
    pub description: Option<String>,
    pub amount: String,
    pub currency_code: String,
    pub tax_code: Option<String>,
    pub cost_center: Option<String>,
    pub department_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Recurring Journals Dashboard Summary.
/// Oracle Fusion: General Ledger > Recurring Journals > Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecurringJournalDashboardSummary {
    pub total_active_schedules: i32,
    pub total_draft_schedules: i32,
    pub total_generations: i32,
    pub total_generations_this_month: i32,
    pub total_generated_amount: String,
    pub schedules_due_today: i32,
    pub schedules_overdue: i32,
    pub schedules_by_recurrence: serde_json::Value,
    pub schedules_by_status: serde_json::Value,
    pub recent_generations: Vec<RecurringJournalGeneration>,
}

// ============================================================================
// Manual Journal Entries (Oracle Fusion GL > Journals > New Journal)
// ============================================================================

/// A journal batch that groups multiple journal entries.
/// Oracle Fusion: General Ledger > Journals > Journal Batch
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JournalBatch {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub batch_number: String,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub ledger_id: Option<Uuid>,
    pub currency_code: String,
    pub accounting_date: Option<chrono::NaiveDate>,
    pub period_name: Option<String>,
    pub total_debit: String,
    pub total_credit: String,
    pub entry_count: i32,
    pub source: String,
    pub is_automatic_post: bool,
    pub submitted_by: Option<Uuid>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub posted_by: Option<Uuid>,
    pub posted_at: Option<DateTime<Utc>>,
    pub rejection_reason: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A journal entry within a batch, containing debit/credit lines.
/// Oracle Fusion: General Ledger > Journals > Journal Entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JournalEntry {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub batch_id: Uuid,
    pub entry_number: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: String,
    pub ledger_id: Option<Uuid>,
    pub currency_code: String,
    pub accounting_date: Option<chrono::NaiveDate>,
    pub period_name: Option<String>,
    pub journal_category: String,
    pub journal_source: String,
    pub total_debit: String,
    pub total_credit: String,
    pub line_count: i32,
    pub is_balanced: bool,
    pub is_reversal: bool,
    pub reversal_of_entry_id: Option<Uuid>,
    pub reversed_by_entry_id: Option<Uuid>,
    pub reference_number: Option<String>,
    pub external_reference: Option<String>,
    pub statistical_entry: bool,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub posted_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A journal entry line (debit or credit).
/// Oracle Fusion: General Ledger > Journals > Journal Line
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JournalEntryLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub entry_id: Uuid,
    pub line_number: i32,
    pub line_type: String,
    pub account_code: String,
    pub account_name: Option<String>,
    pub description: Option<String>,
    pub amount: String,
    pub entered_amount: Option<String>,
    pub entered_currency_code: Option<String>,
    pub exchange_rate: Option<String>,
    pub tax_code: Option<String>,
    pub cost_center: Option<String>,
    pub department_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub intercompany_entity_id: Option<Uuid>,
    pub statistical_amount: Option<String>,
    pub reference1: Option<String>,
    pub reference2: Option<String>,
    pub reference3: Option<String>,
    pub reference4: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Manual Journal Entries Dashboard Summary.
/// Oracle Fusion: General Ledger > Journals > Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManualJournalDashboardSummary {
    pub total_batches: i32,
    pub total_draft_batches: i32,
    pub total_posted_batches: i32,
    pub total_entries: i32,
    pub total_posted_entries: i32,
    pub total_debits: String,
    pub total_credits: String,
    pub batches_pending_approval: i32,
    pub entries_by_category: serde_json::Value,
    pub batches_by_status: serde_json::Value,
    pub recent_batches: Vec<JournalBatch>,
}

// ============================================================================
// Currency Revaluation Types
// Oracle Fusion Cloud ERP: General Ledger > Currency Revaluation
// ============================================================================

/// Currency Revaluation Definition
/// Defines which accounts to revalue, what rate to use, and where to post gains/losses
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrencyRevaluationDefinition {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub revaluation_type: String,
    pub currency_code: String,
    pub rate_type: String,
    pub gain_account_code: String,
    pub loss_account_code: String,
    pub unrealized_gain_account_code: Option<String>,
    pub unrealized_loss_account_code: Option<String>,
    pub account_range_from: Option<String>,
    pub account_range_to: Option<String>,
    pub include_subledger: bool,
    pub auto_reverse: bool,
    pub reversal_period_offset: i32,
    pub is_active: bool,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub accounts: Vec<CurrencyRevaluationAccount>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Account included in a revaluation definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrencyRevaluationAccount {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub definition_id: Uuid,
    pub account_code: String,
    pub account_name: Option<String>,
    pub account_type: String,
    pub is_included: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Currency Revaluation Run
/// A batch execution of a revaluation definition for a specific period
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrencyRevaluationRun {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub run_number: String,
    pub definition_id: Uuid,
    pub definition_code: String,
    pub definition_name: String,
    pub period_name: String,
    pub period_start_date: chrono::NaiveDate,
    pub period_end_date: chrono::NaiveDate,
    pub revaluation_date: chrono::NaiveDate,
    pub currency_code: String,
    pub rate_type: String,
    pub total_revalued_amount: String,
    pub total_gain_amount: String,
    pub total_loss_amount: String,
    pub total_entries: i32,
    pub status: String,
    pub reversal_run_id: Option<Uuid>,
    pub original_run_id: Option<Uuid>,
    pub reversed_at: Option<DateTime<Utc>>,
    pub posted_at: Option<DateTime<Utc>>,
    pub posted_by: Option<Uuid>,
    pub lines: Vec<CurrencyRevaluationLine>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Individual line in a revaluation run (one per account)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrencyRevaluationLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub run_id: Uuid,
    pub line_number: i32,
    pub account_code: String,
    pub account_name: Option<String>,
    pub account_type: String,
    pub original_amount: String,
    pub original_currency: String,
    pub original_exchange_rate: String,
    pub original_base_amount: String,
    pub revalued_exchange_rate: String,
    pub revalued_base_amount: String,
    pub gain_loss_amount: String,
    pub gain_loss_type: String,
    pub gain_loss_account_code: String,
    pub reversal_line_id: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create revaluation definition request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrencyRevaluationDefinitionRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(default = "default_revaluation_type")]
    pub revaluation_type: String,
    pub currency_code: String,
    #[serde(default = "default_rate_type_period_end")]
    pub rate_type: String,
    pub gain_account_code: String,
    pub loss_account_code: String,
    pub unrealized_gain_account_code: Option<String>,
    pub unrealized_loss_account_code: Option<String>,
    pub account_range_from: Option<String>,
    pub account_range_to: Option<String>,
    #[serde(default)]
    pub include_subledger: bool,
    #[serde(default = "default_true")]
    pub auto_reverse: bool,
    #[serde(default = "default_one")]
    pub reversal_period_offset: i32,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub accounts: Option<Vec<CurrencyRevaluationAccountRequest>>,
}

pub fn default_revaluation_type() -> String {
    "period_end".to_string()
}
pub fn default_rate_type_period_end() -> String {
    "period_end".to_string()
}
pub fn default_account_type_asset() -> String {
    "asset".to_string()
}

/// Create revaluation account request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrencyRevaluationAccountRequest {
    pub account_code: String,
    pub account_name: Option<String>,
    #[serde(default = "default_account_type_asset")]
    pub account_type: String,
    #[serde(default = "default_true")]
    pub is_included: bool,
}

/// Execute revaluation run request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrencyRevaluationRunRequest {
    pub definition_code: String,
    pub period_name: String,
    pub period_start_date: chrono::NaiveDate,
    pub period_end_date: chrono::NaiveDate,
    pub revaluation_date: Option<chrono::NaiveDate>,
    pub rate_type_override: Option<String>,
    pub balances: Vec<CurrencyRevaluationBalanceRequest>,
}

/// Balance input for revaluation run
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrencyRevaluationBalanceRequest {
    pub account_code: String,
    pub account_name: Option<String>,
    pub account_type: String,
    pub original_amount: String,
    pub original_currency: String,
    pub original_exchange_rate: String,
    pub original_base_amount: String,
}

/// Currency Revaluation dashboard summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrencyRevaluationDashboardSummary {
    pub total_definitions: i32,
    pub active_definitions: i32,
    pub total_runs: i32,
    pub posted_runs: i32,
    pub draft_runs: i32,
    pub reversed_runs: i32,
    pub total_gain_amount: String,
    pub total_loss_amount: String,
    pub definitions_by_type: serde_json::Value,
}

// ============================================================================
// Credit Management Types
// Oracle Fusion Cloud: Receivables > Credit Management
// ============================================================================

/// Credit scoring model defines how customer creditworthiness is assessed.
/// Oracle Fusion: Credit Management > Credit Scoring Models
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreditScoringModel {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub model_type: String,
    pub scoring_criteria: serde_json::Value,
    pub score_ranges: serde_json::Value,
    pub is_active: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Credit profile per customer or customer group.
/// Oracle Fusion: Credit Management > Credit Profiles
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreditProfile {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub profile_number: String,
    pub profile_name: String,
    pub description: Option<String>,
    pub profile_type: String,
    pub customer_id: Option<Uuid>,
    pub customer_name: Option<String>,
    pub customer_group_id: Option<Uuid>,
    pub customer_group_name: Option<String>,
    pub scoring_model_id: Option<Uuid>,
    pub credit_score: Option<String>,
    pub credit_rating: Option<String>,
    pub risk_level: String,
    pub status: String,
    pub review_frequency_days: i32,
    pub last_review_date: Option<chrono::NaiveDate>,
    pub next_review_date: Option<chrono::NaiveDate>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Credit limit (supports multi-currency and global limits).
/// Oracle Fusion: Credit Management > Credit Limits
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreditLimit {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub profile_id: Uuid,
    pub limit_type: String,
    pub currency_code: Option<String>,
    pub credit_limit: String,
    pub temp_limit_increase: String,
    pub temp_limit_expiry: Option<chrono::NaiveDate>,
    pub used_amount: String,
    pub available_amount: String,
    pub hold_amount: String,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub is_active: bool,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Credit check rule defines when credit checks are triggered.
/// Oracle Fusion: Credit Management > Credit Check Rules
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreditCheckRule {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub check_point: String,
    pub check_type: String,
    pub condition: serde_json::Value,
    pub action_on_failure: String,
    pub priority: i32,
    pub is_active: bool,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Credit exposure tracks total exposure per profile.
/// Oracle Fusion: Credit Management > Credit Exposure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreditExposure {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub profile_id: Uuid,
    pub exposure_date: chrono::NaiveDate,
    pub open_receivables: String,
    pub open_orders: String,
    pub open_shipments: String,
    pub open_invoices: String,
    pub unapplied_cash: String,
    pub on_hold_amount: String,
    pub total_exposure: String,
    pub credit_limit: String,
    pub available_credit: String,
    pub utilization_percent: String,
    pub currency_code: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Credit hold placed on transactions when credit limits exceeded.
/// Oracle Fusion: Credit Management > Credit Holds
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreditHold {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub profile_id: Uuid,
    pub hold_number: String,
    pub hold_type: String,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub entity_number: Option<String>,
    pub hold_amount: Option<String>,
    pub reason: Option<String>,
    pub status: String,
    pub released_by: Option<Uuid>,
    pub released_at: Option<DateTime<Utc>>,
    pub release_reason: Option<String>,
    pub overridden_by: Option<Uuid>,
    pub overridden_at: Option<DateTime<Utc>>,
    pub override_reason: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Credit review (periodic or triggered review of a credit profile).
/// Oracle Fusion: Credit Management > Credit Reviews
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreditReview {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub profile_id: Uuid,
    pub review_number: String,
    pub review_type: String,
    pub status: String,
    pub previous_credit_limit: Option<String>,
    pub recommended_credit_limit: Option<String>,
    pub approved_credit_limit: Option<String>,
    pub previous_score: Option<String>,
    pub new_score: Option<String>,
    pub previous_rating: Option<String>,
    pub new_rating: Option<String>,
    pub findings: Option<String>,
    pub recommendations: Option<String>,
    pub reviewer_id: Option<Uuid>,
    pub reviewer_name: Option<String>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub approver_id: Option<Uuid>,
    pub approver_name: Option<String>,
    pub approved_at: Option<DateTime<Utc>>,
    pub rejected_reason: Option<String>,
    pub due_date: Option<chrono::NaiveDate>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Credit management dashboard summary.
/// Oracle Fusion: Credit Management > Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreditManagementDashboard {
    pub total_profiles: i32,
    pub active_profiles: i32,
    pub blocked_profiles: i32,
    pub total_credit_limit: String,
    pub total_exposure: String,
    pub total_available: String,
    pub active_holds: i32,
    pub pending_reviews: i32,
    pub overdue_reviews: i32,
    pub average_utilization: String,
}

// ============================================================================
// Transfer Pricing (Oracle Fusion Financials > Transfer Pricing)
// ============================================================================

/// Transfer Pricing Policy
/// Defines the pricing method and parameters for intercompany transactions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferPricingPolicy {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub policy_code: String,
    pub name: String,
    pub description: Option<String>,
    pub pricing_method: String,
    pub from_entity_id: Option<Uuid>,
    pub from_entity_name: Option<String>,
    pub to_entity_id: Option<Uuid>,
    pub to_entity_name: Option<String>,
    pub product_category: Option<String>,
    pub item_id: Option<Uuid>,
    pub item_code: Option<String>,
    pub geography: Option<String>,
    pub tax_jurisdiction: Option<String>,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub arm_length_range_low: String,
    pub arm_length_range_mid: String,
    pub arm_length_range_high: String,
    pub margin_pct: String,
    pub cost_base: Option<String>,
    pub status: String,
    pub version: i32,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub created_by: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Transfer Price Transaction
/// Individual intercompany transaction with calculated transfer price.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferPriceTransaction {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub transaction_number: String,
    pub policy_id: Option<Uuid>,
    pub from_entity_id: Option<Uuid>,
    pub from_entity_name: Option<String>,
    pub to_entity_id: Option<Uuid>,
    pub to_entity_name: Option<String>,
    pub item_id: Option<Uuid>,
    pub item_code: Option<String>,
    pub item_description: Option<String>,
    pub quantity: String,
    pub unit_cost: String,
    pub transfer_price: String,
    pub total_amount: String,
    pub currency_code: String,
    pub transaction_date: chrono::NaiveDate,
    pub gl_date: Option<chrono::NaiveDate>,
    pub source_type: Option<String>,
    pub source_id: Option<Uuid>,
    pub source_number: Option<String>,
    pub margin_applied: Option<String>,
    pub margin_amount: Option<String>,
    pub is_arm_length_compliant: Option<bool>,
    pub compliance_notes: Option<String>,
    pub status: String,
    pub submitted_at: Option<DateTime<Utc>>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub created_by: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Benchmark Study (Arm's-Length Analysis)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkStudy {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub study_number: String,
    pub title: String,
    pub description: Option<String>,
    pub policy_id: Option<Uuid>,
    pub analysis_method: String,
    pub fiscal_year: Option<i32>,
    pub from_entity_id: Option<Uuid>,
    pub from_entity_name: Option<String>,
    pub to_entity_id: Option<Uuid>,
    pub to_entity_name: Option<String>,
    pub product_category: Option<String>,
    pub tested_party: Option<String>,
    pub interquartile_range_low: String,
    pub interquartile_range_mid: String,
    pub interquartile_range_high: String,
    pub tested_result: String,
    pub is_within_range: Option<bool>,
    pub conclusion: Option<String>,
    pub prepared_by: Option<Uuid>,
    pub prepared_by_name: Option<String>,
    pub reviewed_by: Option<Uuid>,
    pub reviewed_by_name: Option<String>,
    pub status: String,
    pub approved_at: Option<DateTime<Utc>>,
    pub created_by: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Benchmark Comparable Company
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkComparable {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub benchmark_id: Uuid,
    pub comparable_number: i32,
    pub company_name: String,
    pub country: Option<String>,
    pub industry_code: Option<String>,
    pub industry_description: Option<String>,
    pub fiscal_year: Option<i32>,
    pub revenue: String,
    pub operating_income: String,
    pub operating_margin_pct: String,
    pub net_income: String,
    pub total_assets: String,
    pub employees: Option<i32>,
    pub data_source: Option<String>,
    pub is_included: bool,
    pub exclusion_reason: Option<String>,
    pub relevance_score: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Documentation Package (BEPS / Local File / Master File / `CbCR`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferPricingDocumentation {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub doc_number: String,
    pub title: String,
    pub doc_type: String,
    pub fiscal_year: i32,
    pub country: Option<String>,
    pub reporting_entity_id: Option<Uuid>,
    pub reporting_entity_name: Option<String>,
    pub description: Option<String>,
    pub content_summary: Option<String>,
    pub policy_ids: Option<serde_json::Value>,
    pub benchmark_ids: Option<serde_json::Value>,
    pub filing_date: Option<chrono::NaiveDate>,
    pub filing_deadline: Option<chrono::NaiveDate>,
    pub responsible_party: Option<String>,
    pub status: String,
    pub reviewed_by: Option<Uuid>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub filed_at: Option<DateTime<Utc>>,
    pub created_by: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Transfer Pricing Dashboard Summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferPricingDashboard {
    pub total_policies: i32,
    pub active_policies: i32,
    pub total_transactions: i32,
    pub total_transaction_value: String,
    pub pending_transactions: i32,
    pub non_compliant_transactions: i32,
    pub compliance_rate_pct: String,
    pub total_benchmarks: i32,
    pub active_benchmarks: i32,
    pub benchmarks_within_range: i32,
    pub total_documentation: i32,
    pub pending_filings: i32,
    pub overdue_filings: i32,
    pub transactions_by_method: serde_json::Value,
    pub transactions_by_status: serde_json::Value,
}

// ============================================================================
// Account Monitor & Balance Inquiry (Oracle Fusion General Ledger)
// ============================================================================

/// Account Group: a user-defined collection of GL accounts to monitor together.
/// Oracle Fusion equivalent: General Ledger > Journals > Account Monitor > Account Groups
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountGroup {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub owner_id: Option<Uuid>,
    pub is_shared: bool,
    pub threshold_warning_pct: Option<String>,
    pub threshold_critical_pct: Option<String>,
    pub comparison_type: String,
    pub status: String,
    pub metadata: serde_json::Value,
    pub members: Vec<AccountGroupMember>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A single account within an account group.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountGroupMember {
    pub id: Uuid,
    pub group_id: Uuid,
    pub account_segment: String,
    pub account_label: Option<String>,
    pub display_order: i32,
    pub include_children: bool,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Point-in-time GL balance snapshot for a monitored account.
/// Oracle Fusion equivalent: Account Monitor balance rows
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BalanceSnapshot {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub account_group_id: Uuid,
    pub member_id: Option<Uuid>,
    pub account_segment: String,
    pub period_name: String,
    pub period_start: chrono::NaiveDate,
    pub period_end: chrono::NaiveDate,
    pub fiscal_year: i32,
    pub period_number: i32,
    pub beginning_balance: String,
    pub total_debits: String,
    pub total_credits: String,
    pub net_activity: String,
    pub ending_balance: String,
    pub journal_entry_count: i32,
    pub comparison_balance: Option<String>,
    pub comparison_period_name: Option<String>,
    pub variance_amount: Option<String>,
    pub variance_pct: Option<String>,
    pub alert_status: String,
    pub snapshot_date: chrono::NaiveDate,
    pub computed_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// Saved balance inquiry configuration.
/// Oracle Fusion equivalent: General Ledger > Save Balance Inquiry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedBalanceInquiry {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub account_segments: serde_json::Value,
    pub period_from: String,
    pub period_to: String,
    pub currency_code: String,
    pub amount_type: String,
    pub include_zero_balances: bool,
    pub comparison_enabled: bool,
    pub comparison_type: Option<String>,
    pub sort_by: String,
    pub sort_direction: String,
    pub is_shared: bool,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Account Monitor dashboard summary.
/// Oracle Fusion equivalent: Account Monitor summary panel
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountMonitorSummary {
    pub total_groups: i32,
    pub active_groups: i32,
    pub total_members: i32,
    pub snapshots_with_warning: i32,
    pub snapshots_with_critical: i32,
    pub snapshots_on_track: i32,
    pub latest_snapshot_date: Option<chrono::NaiveDate>,
    pub recent_alerts: serde_json::Value,
}

// ============================================================================
// Joint Venture Management Types
// Oracle Fusion Cloud Financials > Joint Venture Management
// ============================================================================

/// Joint Venture
/// Oracle Fusion: Financials > Joint Venture Management > Joint Ventures
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JointVenture {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub venture_number: String,
    pub name: String,
    pub description: Option<String>,
    pub status: String, // draft, active, on_hold, closed
    pub operator_id: Option<Uuid>,
    pub operator_name: Option<String>,
    pub currency_code: String,
    pub start_date: Option<chrono::NaiveDate>,
    pub end_date: Option<chrono::NaiveDate>,
    pub accounting_method: String, // proportional, equity, cost_method
    pub billing_cycle: String,     // monthly, quarterly, semi_annual, annual
    pub cost_cap_amount: Option<String>,
    pub cost_cap_currency: Option<String>,
    pub gl_revenue_account: Option<String>,
    pub gl_cost_account: Option<String>,
    pub gl_billing_account: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Joint Venture Partner
/// Oracle Fusion: Financials > Joint Venture Management > Partners
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JointVenturePartner {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub venture_id: Uuid,
    pub partner_id: Uuid,
    pub partner_name: String,
    pub partner_type: String, // operator, non_operator, carried_interest
    pub ownership_percentage: String,
    pub revenue_interest_pct: Option<String>,
    pub cost_bearing_pct: Option<String>,
    pub role: String, // operator, partner, carried
    pub billing_contact: Option<String>,
    pub billing_email: Option<String>,
    pub billing_address: Option<String>,
    pub effective_from: chrono::NaiveDate,
    pub effective_to: Option<chrono::NaiveDate>,
    pub status: String, // active, withdrawn, suspended
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// AFE (Authorization for Expenditure)
/// Oracle Fusion: Financials > Joint Venture Management > AFEs
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JointVentureAfe {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub venture_id: Uuid,
    pub afe_number: String,
    pub title: String,
    pub description: Option<String>,
    pub status: String, // draft, submitted, approved, rejected, closed
    pub estimated_cost: String,
    pub actual_cost: String,
    pub committed_cost: String,
    pub remaining_budget: String,
    pub currency_code: String,
    pub cost_center: Option<String>,
    pub work_area: Option<String>,
    pub well_name: Option<String>,
    pub requested_by: Option<Uuid>,
    pub requested_at: Option<DateTime<Utc>>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub rejected_reason: Option<String>,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Joint Venture Cost Distribution
/// Oracle Fusion: Financials > Joint Venture Management > Cost Distributions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JvCostDistribution {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub venture_id: Uuid,
    pub distribution_number: String,
    pub afe_id: Option<Uuid>,
    pub description: Option<String>,
    pub status: String, // draft, posted, reversed
    pub total_amount: String,
    pub currency_code: String,
    pub cost_type: String, // operating, capital, aba, overhead
    pub distribution_date: chrono::NaiveDate,
    pub gl_posting_date: Option<chrono::NaiveDate>,
    pub gl_posted_at: Option<DateTime<Utc>>,
    pub source_type: Option<String>,
    pub source_id: Option<Uuid>,
    pub source_number: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Joint Venture Cost Distribution Line
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JvCostDistributionLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub distribution_id: Uuid,
    pub partner_id: Uuid,
    pub partner_name: Option<String>,
    pub ownership_pct: String,
    pub cost_bearing_pct: String,
    pub distributed_amount: String,
    pub gl_account_code: Option<String>,
    pub line_description: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Joint Venture Revenue Distribution
/// Oracle Fusion: Financials > Joint Venture Management > Revenue Distributions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JvRevenueDistribution {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub venture_id: Uuid,
    pub distribution_number: String,
    pub description: Option<String>,
    pub status: String, // draft, posted, reversed
    pub total_amount: String,
    pub currency_code: String,
    pub revenue_type: String, // sales, royalty, bonus, other
    pub distribution_date: chrono::NaiveDate,
    pub gl_posting_date: Option<chrono::NaiveDate>,
    pub gl_posted_at: Option<DateTime<Utc>>,
    pub source_type: Option<String>,
    pub source_id: Option<Uuid>,
    pub source_number: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Joint Venture Revenue Distribution Line
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JvRevenueDistributionLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub distribution_id: Uuid,
    pub partner_id: Uuid,
    pub partner_name: Option<String>,
    pub revenue_interest_pct: String,
    pub distributed_amount: String,
    pub gl_account_code: Option<String>,
    pub line_description: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Joint Venture Billing (Joint Interest Billing / JIB)
/// Oracle Fusion: Financials > Joint Venture Management > Billings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JvBilling {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub venture_id: Uuid,
    pub billing_number: String,
    pub partner_id: Uuid,
    pub partner_name: Option<String>,
    pub billing_type: String, // jib (cost), revenue, adjustment
    pub status: String,       // draft, submitted, approved, paid, disputed, cancelled
    pub total_amount: String,
    pub tax_amount: String,
    pub total_with_tax: String,
    pub currency_code: String,
    pub billing_period_start: chrono::NaiveDate,
    pub billing_period_end: chrono::NaiveDate,
    pub due_date: Option<chrono::NaiveDate>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub paid_at: Option<DateTime<Utc>>,
    pub payment_reference: Option<String>,
    pub dispute_reason: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Joint Venture Billing Line
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JvBillingLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub billing_id: Uuid,
    pub line_number: i32,
    pub cost_distribution_id: Option<Uuid>,
    pub revenue_distribution_id: Option<Uuid>,
    pub description: Option<String>,
    pub cost_type: Option<String>,
    pub amount: String,
    pub ownership_pct: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Joint Venture Dashboard Summary
/// Oracle Fusion: Financials > Joint Venture Management > Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JvDashboard {
    pub total_ventures: i32,
    pub active_ventures: i32,
    pub total_partners: i32,
    pub total_cost_distributed: String,
    pub total_revenue_distributed: String,
    pub total_billed: String,
    pub total_collected: String,
    pub outstanding_balance: String,
    pub pending_afes: i32,
    pub ventures_by_status: serde_json::Value,
}

// ============================================================================
// Accounts Receivable (Oracle Fusion: Financials > Receivables)
// ============================================================================

/// AR Transaction (customer invoice / debit memo / credit memo)
/// Oracle Fusion: Receivables > Transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArTransaction {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub transaction_number: String,
    pub transaction_type: String, // invoice, debit_memo, credit_memo, chargeback, deposit, guarantee
    pub transaction_date: chrono::NaiveDate,
    pub gl_date: Option<chrono::NaiveDate>,
    pub customer_id: Uuid,
    pub customer_number: Option<String>,
    pub customer_name: Option<String>,
    pub bill_to_site: Option<String>,
    pub currency_code: String,
    pub exchange_rate: Option<String>,
    pub exchange_rate_type: Option<String>,
    pub entered_amount: String,
    pub tax_amount: String,
    pub total_amount: String,
    pub amount_due_original: String,
    pub amount_due_remaining: String,
    pub amount_applied: String,
    pub amount_adjusted: String,
    pub payment_terms: Option<String>,
    pub due_date: Option<chrono::NaiveDate>,
    pub discount_due_date: Option<chrono::NaiveDate>,
    pub reference_number: Option<String>,
    pub purchase_order: Option<String>,
    pub sales_rep: Option<String>,
    pub status: String, // draft, complete, open, closed, cancelled
    pub receipt_method: Option<String>,
    pub notes: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// AR Transaction Line
/// Oracle Fusion: Receivables > Transaction Lines
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArTransactionLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub transaction_id: Uuid,
    pub line_number: i32,
    pub description: Option<String>,
    pub line_type: String, // line, tax, freight, charges
    pub item_code: Option<String>,
    pub item_description: Option<String>,
    pub unit_of_measure: Option<String>,
    pub quantity: Option<String>,
    pub unit_price: Option<String>,
    pub line_amount: String,
    pub tax_amount: String,
    pub tax_code: Option<String>,
    pub revenue_account: Option<String>,
    pub tax_account: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// AR Receipt (customer payment)
/// Oracle Fusion: Receivables > Receipts
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArReceipt {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub receipt_number: String,
    pub receipt_date: chrono::NaiveDate,
    pub receipt_type: String, // cash, check, credit_card, wire_transfer, ach, other
    pub receipt_method: String, // automatic_receipt, manual_receipt, quick_cash, miscellaneous
    pub amount: String,
    pub currency_code: String,
    pub exchange_rate: Option<String>,
    pub customer_id: Option<Uuid>,
    pub customer_number: Option<String>,
    pub customer_name: Option<String>,
    pub reference_number: Option<String>,
    pub bank_account_name: Option<String>,
    pub check_number: Option<String>,
    pub maturity_date: Option<chrono::NaiveDate>,
    pub status: String, // draft, confirmed, applied, deposited, reversed
    pub applied_transaction_number: Option<String>,
    pub notes: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// AR Credit Memo
/// Oracle Fusion: Receivables > Credit Memos
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArCreditMemo {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub credit_memo_number: String,
    pub customer_id: Uuid,
    pub customer_number: Option<String>,
    pub customer_name: Option<String>,
    pub transaction_id: Option<Uuid>,
    pub transaction_number: Option<String>,
    pub credit_memo_date: chrono::NaiveDate,
    pub gl_date: Option<chrono::NaiveDate>,
    pub reason_code: String, // return, pricing_error, damaged, wrong_item, discount, other
    pub reason_description: Option<String>,
    pub amount: String,
    pub tax_amount: String,
    pub total_amount: String,
    pub status: String, // draft, submitted, approved, applied, cancelled
    pub notes: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// AR Adjustment
/// Oracle Fusion: Receivables > Adjustments
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArAdjustment {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub adjustment_number: String,
    pub transaction_id: Option<Uuid>,
    pub transaction_number: Option<String>,
    pub customer_id: Option<Uuid>,
    pub customer_number: Option<String>,
    pub adjustment_date: chrono::NaiveDate,
    pub gl_date: Option<chrono::NaiveDate>,
    pub adjustment_type: String, // write_off, write_off_bad_debt, small_balance_write_off, increase, decrease, transfer, revaluation
    pub amount: String,
    pub receivable_account: Option<String>,
    pub adjustment_account: Option<String>,
    pub reason_code: Option<String>,
    pub reason_description: Option<String>,
    pub status: String, // draft, submitted, approved, rejected, posted
    pub approved_by: Option<Uuid>,
    pub notes: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// AR Aging Summary
/// Oracle Fusion: Receivables > Reports > Aging Report
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArAgingSummary {
    pub organization_id: Uuid,
    pub as_of_date: chrono::NaiveDate,
    pub total_outstanding: String,
    pub total_overdue: String,
    pub aging_current: String,
    pub aging_1_30: String,
    pub aging_31_60: String,
    pub aging_61_90: String,
    pub aging_91_plus: String,
    pub customer_count: i32,
    pub overdue_customer_count: i32,
}

/// AR Aging by Customer
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArAgingByCustomer {
    pub customer_id: Uuid,
    pub customer_name: String,
    pub customer_number: Option<String>,
    pub total_outstanding: String,
    pub current_amount: String,
    pub aging_1_30: String,
    pub aging_31_60: String,
    pub aging_61_90: String,
    pub aging_91_plus: String,
    pub invoice_count: i32,
}

// ============================================================================
// General Ledger (Oracle Fusion: Financials > General Ledger)
// ============================================================================

/// GL Account (Chart of Accounts segment)
/// Oracle Fusion: General Ledger > Chart of Accounts
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlAccount {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub account_code: String,
    pub account_name: String,
    pub description: Option<String>,
    pub account_type: String, // asset, liability, equity, revenue, expense
    pub subtype: Option<String>,
    pub parent_account_id: Option<Uuid>,
    pub is_active: bool,
    pub natural_balance: String, // debit, credit
    pub third_party_control: bool,
    pub reconciliation_enabled: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// GL Journal Entry Header
/// Oracle Fusion: General Ledger > Journals
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlJournalEntry {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub entry_number: String,
    pub ledger_id: Option<Uuid>,
    pub entry_date: chrono::NaiveDate,
    pub gl_date: chrono::NaiveDate,
    pub entry_type: String, // standard, adjusting, closing, reversing, budget
    pub description: Option<String>,
    pub currency_code: String,
    pub total_debit: String,
    pub total_credit: String,
    pub is_balanced: bool,
    pub status: String, // draft, submitted, posted, reversed, error
    pub posted_by: Option<Uuid>,
    pub posted_at: Option<chrono::DateTime<chrono::Utc>>,
    pub reversal_entry_id: Option<Uuid>,
    pub source_type: Option<String>,
    pub source_id: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// GL Journal Line
/// Oracle Fusion: General Ledger > Journal Lines
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlJournalLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub journal_entry_id: Uuid,
    pub line_number: i32,
    pub line_type: String, // debit, credit
    pub account_code: String,
    pub account_name: Option<String>,
    pub description: Option<String>,
    pub entered_dr: String,
    pub entered_cr: String,
    pub accounted_dr: String,
    pub accounted_cr: String,
    pub currency_code: String,
    pub exchange_rate: Option<String>,
    pub reference: Option<String>,
    pub tax_code: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// GL Trial Balance
/// Oracle Fusion: General Ledger > Reports > Trial Balance
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlTrialBalance {
    pub organization_id: Uuid,
    pub as_of_date: chrono::NaiveDate,
    pub ledger_id: Option<Uuid>,
    pub lines: Vec<GlTrialBalanceLine>,
    pub total_debit: String,
    pub total_credit: String,
    pub total_net: String,
}

/// Trial Balance Line
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlTrialBalanceLine {
    pub account_code: String,
    pub account_name: String,
    pub account_type: String,
    pub beginning_balance: String,
    pub period_debit: String,
    pub period_credit: String,
    pub ending_balance: String,
    pub net_activity: String,
}

// ============================================================================
// Interest Invoice Management (Oracle Fusion: Receivables > Late Charges)
// ============================================================================

/// Interest rate schedule definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InterestRateSchedule {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub schedule_code: String,
    pub name: String,
    pub description: Option<String>,
    pub annual_rate: String,
    pub compounding_frequency: String,
    pub charge_type: String,
    pub grace_period_days: i32,
    pub minimum_charge: String,
    pub maximum_charge: Option<String>,
    pub currency_code: String,
    pub effective_from: chrono::NaiveDate,
    pub effective_to: Option<chrono::NaiveDate>,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Overdue invoice record
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OverdueInvoice {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub invoice_number: String,
    pub customer_id: Uuid,
    pub customer_name: Option<String>,
    pub original_amount: String,
    pub outstanding_amount: String,
    pub due_date: chrono::NaiveDate,
    pub overdue_days: i32,
    pub currency_code: String,
    pub status: String,
    pub last_interest_date: Option<chrono::NaiveDate>,
    pub total_interest_charged: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Interest calculation run
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InterestCalculationRun {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub run_number: String,
    pub description: Option<String>,
    pub calculation_date: chrono::NaiveDate,
    pub schedule_id: Option<Uuid>,
    pub total_invoices_processed: i32,
    pub total_interest_calculated: String,
    pub currency_code: String,
    pub status: String,
    pub generated_by: Option<Uuid>,
    pub posted_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Individual interest calculation line
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InterestCalculationLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub run_id: Uuid,
    pub overdue_invoice_id: Option<Uuid>,
    pub invoice_number: String,
    pub customer_id: Uuid,
    pub customer_name: Option<String>,
    pub outstanding_amount: String,
    pub overdue_days: i32,
    pub annual_rate_used: String,
    pub interest_amount: String,
    pub currency_code: String,
    pub status: String,
    pub interest_invoice_id: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Interest invoice
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InterestInvoice {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub invoice_number: String,
    pub customer_id: Uuid,
    pub customer_name: Option<String>,
    pub calculation_run_id: Option<Uuid>,
    pub invoice_date: chrono::NaiveDate,
    pub due_date: Option<chrono::NaiveDate>,
    pub total_interest_amount: String,
    pub currency_code: String,
    pub line_count: i32,
    pub status: String,
    pub gl_account_code: Option<String>,
    pub posted_at: Option<DateTime<Utc>>,
    pub reversed_at: Option<DateTime<Utc>>,
    pub reversal_invoice_id: Option<Uuid>,
    pub reference_invoice_number: Option<String>,
    pub notes: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Interest invoice line
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InterestInvoiceLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub interest_invoice_id: Uuid,
    pub calculation_line_id: Option<Uuid>,
    pub line_number: i32,
    pub line_type: String,
    pub description: Option<String>,
    pub reference_invoice_number: Option<String>,
    pub overdue_days: Option<i32>,
    pub outstanding_amount: Option<String>,
    pub annual_rate_used: Option<String>,
    pub interest_amount: String,
    pub currency_code: String,
    pub gl_account_code: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Interest invoice management dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InterestInvoiceDashboard {
    pub total_active_schedules: i32,
    pub total_overdue_invoices: i32,
    pub total_overdue_amount: String,
    pub total_interest_ytd: String,
    pub total_pending_invoices: i32,
    pub total_pending_amount: String,
    pub avg_overdue_days: String,
}

// ============================================================================
// Expense Policy Compliance Engine
// Oracle Fusion: Expenses > Policies > Expense Policy Compliance
// ============================================================================

/// Expense policy rule definition
/// Oracle Fusion: Expenses > Policies > Policy Rules
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpensePolicyRule {
    pub id: Uuid,
    pub org_id: Uuid,
    pub rule_code: String,
    pub name: String,
    pub description: Option<String>,
    pub rule_type: String,
    pub expense_category: String,
    pub severity: String,
    pub evaluation_scope: String,
    pub threshold_amount: Option<String>,
    pub maximum_amount: Option<String>,
    pub threshold_days: i32,
    pub requires_receipt: bool,
    pub requires_justification: bool,
    pub is_active: bool,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub applies_to_department: Option<String>,
    pub applies_to_cost_center: Option<String>,
    pub created_by_id: Option<Uuid>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Expense compliance audit
/// Oracle Fusion: Expenses > Audit > Compliance Audits
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpenseComplianceAudit {
    pub id: Uuid,
    pub org_id: Uuid,
    pub audit_number: String,
    pub report_id: Uuid,
    pub report_number: Option<String>,
    pub employee_id: Option<Uuid>,
    pub employee_name: Option<String>,
    pub department_id: Option<Uuid>,
    pub audit_date: chrono::NaiveDate,
    pub audit_trigger: String,
    pub total_lines: i32,
    pub violations_count: i32,
    pub warnings_count: i32,
    pub blocks_count: i32,
    pub compliance_score: Option<String>,
    pub risk_level: String,
    pub total_flagged_amount: Option<String>,
    pub total_approved_amount: Option<String>,
    pub requires_manager_review: bool,
    pub requires_finance_review: bool,
    pub status: String,
    pub reviewed_by_id: Option<Uuid>,
    pub review_notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Expense compliance violation
/// Oracle Fusion: Expenses > Audit > Violation Details
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpenseComplianceViolation {
    pub id: Uuid,
    pub org_id: Uuid,
    pub audit_id: Uuid,
    pub report_id: Uuid,
    pub report_line_id: Option<Uuid>,
    pub policy_rule_id: Option<Uuid>,
    pub rule_code: String,
    pub rule_name: Option<String>,
    pub rule_type: String,
    pub severity: String,
    pub violation_description: Option<String>,
    pub expense_amount: Option<String>,
    pub threshold_amount: Option<String>,
    pub excess_amount: Option<String>,
    pub resolution_status: String,
    pub justification: Option<String>,
    pub resolved_by_id: Option<Uuid>,
    pub resolution_date: Option<chrono::NaiveDate>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Expense compliance dashboard summary
/// Oracle Fusion: Expenses > Audit > Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpenseComplianceDashboard {
    pub total_active_rules: i32,
    pub total_audits_period: i32,
    pub total_violations_period: i32,
    pub total_warnings_period: i32,
    pub total_blocks_period: i32,
    pub avg_compliance_score: String,
    pub total_flagged_amount: String,
    pub high_risk_audits: i32,
    pub open_violations: i32,
}

// ============================================================================
// Bank Guarantee Management
// Oracle Fusion: Treasury > Bank Guarantees
// ============================================================================

/// Bank Guarantee
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BankGuarantee {
    pub id: Uuid,
    pub org_id: Uuid,
    pub guarantee_number: String,
    pub guarantee_type: String,
    pub description: Option<String>,
    pub beneficiary_name: String,
    pub beneficiary_code: Option<String>,
    pub applicant_name: String,
    pub applicant_code: Option<String>,
    pub issuing_bank_name: String,
    pub issuing_bank_code: Option<String>,
    pub bank_account_number: Option<String>,
    pub guarantee_amount: String,
    pub currency_code: String,
    pub margin_percentage: String,
    pub margin_amount: String,
    pub commission_rate: String,
    pub commission_amount: String,
    pub issue_date: Option<chrono::NaiveDate>,
    pub effective_date: Option<chrono::NaiveDate>,
    pub expiry_date: Option<chrono::NaiveDate>,
    pub claim_expiry_date: Option<chrono::NaiveDate>,
    pub renewal_date: Option<chrono::NaiveDate>,
    pub auto_renew: bool,
    pub reference_contract_number: Option<String>,
    pub reference_purchase_order: Option<String>,
    pub purpose: Option<String>,
    pub collateral_type: Option<String>,
    pub collateral_amount: Option<String>,
    pub status: String,
    pub amendment_count: i32,
    pub latest_amendment_number: Option<String>,
    pub created_by_id: Option<Uuid>,
    pub approved_by_id: Option<Uuid>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Bank Guarantee Amendment
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BankGuaranteeAmendment {
    pub id: Uuid,
    pub org_id: Uuid,
    pub guarantee_id: Uuid,
    pub guarantee_number: String,
    pub amendment_number: String,
    pub amendment_type: String,
    pub previous_amount: Option<String>,
    pub new_amount: Option<String>,
    pub previous_expiry_date: Option<chrono::NaiveDate>,
    pub new_expiry_date: Option<chrono::NaiveDate>,
    pub previous_terms: Option<String>,
    pub new_terms: Option<String>,
    pub reason: Option<String>,
    pub status: String,
    pub effective_date: Option<chrono::NaiveDate>,
    pub approved_by_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Bank Guarantee Dashboard Summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BankGuaranteeDashboard {
    pub total_guarantees: i32,
    pub active_guarantees: i32,
    pub total_guarantee_amount: String,
    pub total_margin_held: String,
    pub expiring_within_30_days: i32,
    pub expiring_within_90_days: i32,
    pub pending_approval: i32,
    pub amendments_pending: i32,
    pub by_type: serde_json::Value,
    pub by_currency: serde_json::Value,
}

// ============================================================================
// Letter of Credit Management
// Oracle Fusion: Treasury > Trade Finance > Letters of Credit
// ============================================================================

/// Letter of Credit
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LetterOfCredit {
    pub id: Uuid,
    pub org_id: Uuid,
    pub lc_number: String,
    pub lc_type: String,
    pub lc_form: String,
    pub description: Option<String>,
    pub applicant_name: String,
    pub applicant_address: Option<String>,
    pub applicant_bank_name: String,
    pub applicant_bank_swift: Option<String>,
    pub beneficiary_name: String,
    pub beneficiary_address: Option<String>,
    pub beneficiary_bank_name: Option<String>,
    pub beneficiary_bank_swift: Option<String>,
    pub advising_bank_name: Option<String>,
    pub advising_bank_swift: Option<String>,
    pub confirming_bank_name: Option<String>,
    pub confirming_bank_swift: Option<String>,
    pub lc_amount: String,
    pub currency_code: String,
    pub tolerance_plus: String,
    pub tolerance_minus: String,
    pub available_with: Option<String>,
    pub available_by: String,
    pub draft_at: Option<String>,
    pub issue_date: Option<chrono::NaiveDate>,
    pub expiry_date: chrono::NaiveDate,
    pub place_of_expiry: Option<String>,
    pub partial_shipments: String,
    pub transshipment: String,
    pub port_of_loading: Option<String>,
    pub port_of_discharge: Option<String>,
    pub shipment_period: Option<chrono::NaiveDate>,
    pub latest_shipment_date: Option<chrono::NaiveDate>,
    pub goods_description: Option<String>,
    pub incoterms: Option<String>,
    pub additional_conditions: Option<String>,
    pub bank_charges: String,
    pub status: String,
    pub amendment_count: i32,
    pub latest_amendment_number: Option<String>,
    pub reference_po_number: Option<String>,
    pub reference_contract_number: Option<String>,
    pub notes: Option<String>,
    pub created_by_id: Option<Uuid>,
    pub approved_by_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// LC Amendment
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LcAmendment {
    pub id: Uuid,
    pub org_id: Uuid,
    pub lc_id: Uuid,
    pub lc_number: String,
    pub amendment_number: String,
    pub amendment_type: String,
    pub previous_amount: Option<String>,
    pub new_amount: Option<String>,
    pub previous_expiry_date: Option<chrono::NaiveDate>,
    pub new_expiry_date: Option<chrono::NaiveDate>,
    pub previous_terms: Option<String>,
    pub new_terms: Option<String>,
    pub reason: Option<String>,
    pub bank_reference: Option<String>,
    pub status: String,
    pub effective_date: Option<chrono::NaiveDate>,
    pub approved_by_id: Option<Uuid>,
    pub created_by_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// LC Required Document
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LcRequiredDocument {
    pub id: Uuid,
    pub org_id: Uuid,
    pub lc_id: Uuid,
    pub document_type: String,
    pub document_code: Option<String>,
    pub description: Option<String>,
    pub original_copies: i32,
    pub copy_count: i32,
    pub is_mandatory: bool,
    pub special_instructions: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// LC Shipment
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LcShipment {
    pub id: Uuid,
    pub org_id: Uuid,
    pub lc_id: Uuid,
    pub shipment_number: String,
    pub vessel_name: Option<String>,
    pub voyage_number: Option<String>,
    pub bill_of_lading_number: Option<String>,
    pub carrier_name: Option<String>,
    pub port_of_loading: Option<String>,
    pub port_of_discharge: Option<String>,
    pub shipment_date: Option<chrono::NaiveDate>,
    pub expected_arrival_date: Option<chrono::NaiveDate>,
    pub actual_arrival_date: Option<chrono::NaiveDate>,
    pub shipping_marks: Option<String>,
    pub container_numbers: Option<String>,
    pub goods_description: Option<String>,
    pub quantity: Option<String>,
    pub unit_price: Option<String>,
    pub shipment_amount: String,
    pub currency_code: String,
    pub status: String,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// LC Presentation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LcPresentation {
    pub id: Uuid,
    pub org_id: Uuid,
    pub lc_id: Uuid,
    pub presentation_number: String,
    pub shipment_id: Option<Uuid>,
    pub presentation_date: chrono::NaiveDate,
    pub presenting_bank_name: Option<String>,
    pub total_amount: String,
    pub currency_code: String,
    pub document_count: i32,
    pub discrepant: bool,
    pub discrepancies: Option<String>,
    pub bank_response: Option<String>,
    pub response_date: Option<chrono::NaiveDate>,
    pub payment_due_date: Option<chrono::NaiveDate>,
    pub payment_date: Option<chrono::NaiveDate>,
    pub paid_amount: Option<String>,
    pub status: String,
    pub notes: Option<String>,
    pub created_by_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// LC Presentation Document
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LcPresentationDocument {
    pub id: Uuid,
    pub org_id: Uuid,
    pub presentation_id: Uuid,
    pub required_document_id: Option<Uuid>,
    pub document_type: String,
    pub document_reference: Option<String>,
    pub description: Option<String>,
    pub original_copies: i32,
    pub copy_count: i32,
    pub is_compliant: bool,
    pub discrepancies: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// LC Dashboard Summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LcDashboard {
    pub total_active_lcs: i32,
    pub total_lc_amount: String,
    pub total_pending_amendments: i32,
    pub total_presentations_pending: i32,
    pub total_discrepant_presentations: i32,
    pub expiring_within_30_days: i32,
    pub expiring_within_90_days: i32,
    pub by_type: serde_json::Value,
    pub by_currency: serde_json::Value,
    pub by_status: serde_json::Value,
}

// ============================================================================
// Hedge Management
// Oracle Fusion: Treasury > Hedge Management
// IFRS 9 / ASC 815 Hedge Accounting
// ============================================================================

/// Derivative Instrument
/// Oracle Fusion: Treasury > Hedge Management > Derivative Instruments
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DerivativeInstrument {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub instrument_number: String,
    pub instrument_type: String,
    pub underlying_type: String,
    pub underlying_description: Option<String>,
    pub currency_code: String,
    pub counter_currency_code: Option<String>,
    pub notional_amount: String,
    pub strike_rate: Option<String>,
    pub forward_rate: Option<String>,
    pub spot_rate: Option<String>,
    pub option_type: Option<String>,
    pub premium_amount: Option<String>,
    pub trade_date: Option<chrono::NaiveDate>,
    pub effective_date: Option<chrono::NaiveDate>,
    pub maturity_date: Option<chrono::NaiveDate>,
    pub settlement_date: Option<chrono::NaiveDate>,
    pub settlement_type: Option<String>,
    pub counterparty_name: Option<String>,
    pub counterparty_reference: Option<String>,
    pub portfolio_code: Option<String>,
    pub trading_book: Option<String>,
    pub accounting_treatment: Option<String>,
    pub fair_value: Option<String>,
    pub unrealized_gain_loss: Option<String>,
    pub realized_gain_loss: Option<String>,
    pub valuation_method: Option<String>,
    pub last_valuation_date: Option<chrono::NaiveDate>,
    pub risk_factor: Option<String>,
    pub status: String,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
}

/// Hedge Relationship
/// Oracle Fusion: Treasury > Hedge Management > Hedge Relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HedgeRelationship {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub hedge_id: String,
    pub hedge_type: String,
    pub derivative_id: Option<Uuid>,
    pub derivative_number: Option<String>,
    pub hedged_item_description: Option<String>,
    pub hedged_item_id: Option<Uuid>,
    pub hedged_risk: String,
    pub hedge_strategy: Option<String>,
    pub hedged_item_reference: Option<String>,
    pub hedged_item_currency: Option<String>,
    pub hedged_amount: String,
    pub hedge_ratio: Option<String>,
    pub designated_start_date: Option<chrono::NaiveDate>,
    pub designated_end_date: Option<chrono::NaiveDate>,
    pub effectiveness_method: String,
    pub critical_terms_match: Option<String>,
    pub prospective_effective: Option<bool>,
    pub retrospective_effective: Option<bool>,
    pub hedge_documentation_ref: Option<String>,
    pub status: String,
    pub last_effectiveness_test_date: Option<chrono::NaiveDate>,
    pub last_effectiveness_result: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
}

/// Hedge Effectiveness Test
/// Oracle Fusion: Treasury > Hedge Management > Effectiveness Testing
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HedgeEffectivenessTest {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub hedge_relationship_id: Uuid,
    pub hedge_id: Option<String>,
    pub test_type: String,
    pub effectiveness_method: String,
    pub test_date: chrono::NaiveDate,
    pub test_period_start: Option<chrono::NaiveDate>,
    pub test_period_end: Option<chrono::NaiveDate>,
    pub derivative_fair_value_change: Option<String>,
    pub hedged_item_fair_value_change: Option<String>,
    pub hedge_ratio_result: Option<String>,
    pub ratio_lower_bound: Option<String>,
    pub ratio_upper_bound: Option<String>,
    pub effectiveness_result: String,
    pub ineffective_amount: Option<String>,
    pub cumulative_gain_loss: Option<String>,
    pub regression_r_squared: Option<String>,
    pub notes: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
}

/// Hedge Documentation
/// Oracle Fusion: Treasury > Hedge Management > Documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HedgeDocumentation {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub hedge_relationship_id: Option<Uuid>,
    pub hedge_id: Option<String>,
    pub document_number: String,
    pub hedge_type: String,
    pub risk_management_objective: Option<String>,
    pub hedging_strategy_description: Option<String>,
    pub hedged_item_description: Option<String>,
    pub hedged_risk_description: Option<String>,
    pub derivative_description: Option<String>,
    pub effectiveness_method_description: Option<String>,
    pub assessment_frequency: Option<String>,
    pub designation_date: Option<chrono::NaiveDate>,
    pub documentation_date: Option<chrono::NaiveDate>,
    pub approval_date: Option<chrono::NaiveDate>,
    pub approved_by: Option<Uuid>,
    pub prepared_by: Option<String>,
    pub status: String,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
}

// ============================================================================
// Tax Registration Management (Oracle Fusion Tax > Tax Registrations)
// ============================================================================

/// Tax Registration
/// Oracle Fusion equivalent: Financials > Tax > Tax Registrations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxRegistration {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// The taxpayer identification number (e.g., VAT number, EIN, GSTIN)
    pub registration_number: String,
    /// Type of registration (tin, vat, gst, ein, sst, pan, cst, `sales_tax`, `withholding_tax`, excise, customs, other)
    pub registration_type: String,
    /// Tax purpose of this registration (`input_tax`, `output_tax`, both, `reporting_only`, withholding, `reverse_charge`, intracommunity)
    pub tax_purpose: String,
    /// Whether this is a first-party (own entity) or third-party registration
    pub party_type: String,
    /// Reference to the party (legal entity or supplier/customer)
    pub party_id: Option<Uuid>,
    pub party_name: Option<String>,
    /// Tax jurisdiction code (e.g., "US-FED", "EU-DE", "IN-GST")
    pub jurisdiction_code: String,
    /// ISO 3166-1 alpha-2 country code
    pub country_code: String,
    /// Optional state/province code
    pub state_code: Option<String>,
    /// Status: active, suspended, deregistered, expired, pending
    pub status: String,
    /// When this registration became/becomes effective
    pub effective_from: chrono::NaiveDate,
    /// When this registration expires/expired (None = indefinite)
    pub effective_to: Option<chrono::NaiveDate>,
    /// Whether this is the default registration for this jurisdiction/type
    pub is_default: bool,
    /// Name used for tax reporting (may differ from legal name)
    pub reporting_name: Option<String>,
    /// Legal entity this registration belongs to
    pub legal_entity_id: Option<Uuid>,
    /// Validation status of the registration number
    pub validation_status: String,
    /// When the registration was last validated
    pub last_validated_at: Option<DateTime<Utc>>,
    /// How this registration was created (manual, import, integration, migration)
    pub source: String,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Tax Registration Summary for dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxRegistrationSummary {
    pub organization_id: Uuid,
    pub total_registrations: i32,
    pub active_registrations: i32,
    pub suspended_registrations: i32,
    pub expired_registrations: i32,
    pub pending_registrations: i32,
    pub first_party_count: i32,
    pub third_party_count: i32,
    pub jurisdictions_covered: i32,
}

/// Hedge Management Dashboard Summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HedgeDashboard {
    pub total_active_derivatives: i32,
    pub total_notional_amount: String,
    pub total_active_hedges: i32,
    pub total_hedged_amount: String,
    pub total_effective_hedges: i32,
    pub total_ineffective_hedges: i32,
    pub total_pending_documentation: i32,
    pub total_unrealized_gain_loss: String,
    pub by_instrument_type: serde_json::Value,
    pub by_hedge_type: serde_json::Value,
}

// ============================================================================
// Cash Concentration / Pooling Types (Oracle Fusion: Treasury > Cash Pooling)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CashPool {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub pool_code: String,
    pub pool_name: String,
    pub pool_type: String,
    pub concentration_account_id: Option<Uuid>,
    pub concentration_account_name: Option<String>,
    pub currency_code: String,
    pub status: String,
    pub effective_date: Option<chrono::NaiveDate>,
    pub termination_date: Option<chrono::NaiveDate>,
    pub sweep_frequency: Option<String>,
    pub sweep_time: Option<String>,
    pub minimum_transfer_amount: Option<String>,
    pub maximum_transfer_amount: Option<String>,
    pub target_balance: Option<String>,
    pub interest_allocation_method: Option<String>,
    pub interest_rate: Option<String>,
    pub description: Option<String>,
    pub notes: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CashPoolParticipant {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub pool_id: Uuid,
    pub participant_code: String,
    pub bank_account_id: Option<Uuid>,
    pub bank_account_name: Option<String>,
    pub bank_name: Option<String>,
    pub account_number: Option<String>,
    pub participant_type: String,
    pub sweep_direction: String,
    pub priority: Option<i32>,
    pub minimum_balance: Option<String>,
    pub maximum_balance: Option<String>,
    pub threshold_amount: Option<String>,
    pub current_balance: Option<String>,
    pub status: String,
    pub effective_date: Option<chrono::NaiveDate>,
    pub termination_date: Option<chrono::NaiveDate>,
    pub entity_id: Option<Uuid>,
    pub entity_name: Option<String>,
    pub description: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CashPoolSweepRule {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub pool_id: Uuid,
    pub rule_code: String,
    pub rule_name: String,
    pub sweep_type: String,
    pub participant_id: Option<Uuid>,
    pub direction: String,
    pub trigger_condition: Option<String>,
    pub threshold_amount: Option<String>,
    pub target_balance: Option<String>,
    pub minimum_transfer: Option<String>,
    pub maximum_transfer: Option<String>,
    pub priority: Option<i32>,
    pub is_active: Option<bool>,
    pub effective_date: Option<chrono::NaiveDate>,
    pub termination_date: Option<chrono::NaiveDate>,
    pub description: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CashPoolSweepRun {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub pool_id: Uuid,
    pub run_number: String,
    pub run_date: chrono::NaiveDate,
    pub run_type: String,
    pub status: String,
    pub total_swept_amount: Option<String>,
    pub total_transactions: Option<i32>,
    pub successful_transactions: Option<i32>,
    pub failed_transactions: Option<i32>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub initiated_by: Option<Uuid>,
    pub notes: Option<String>,
    pub error_message: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CashPoolSweepRunLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub sweep_run_id: Uuid,
    pub pool_id: Uuid,
    pub participant_id: Uuid,
    pub participant_code: Option<String>,
    pub bank_account_name: Option<String>,
    pub sweep_rule_id: Option<Uuid>,
    pub direction: String,
    pub pre_sweep_balance: Option<String>,
    pub sweep_amount: Option<String>,
    pub post_sweep_balance: Option<String>,
    pub status: String,
    pub reference_number: Option<String>,
    pub error_message: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CashPoolDashboard {
    pub total_pools: i32,
    pub active_pools: i32,
    pub total_participants: i32,
    pub total_concentrated_balance: String,
    pub total_swept_today: String,
    pub pending_sweeps: i32,
    pub by_pool_type: serde_json::Value,
    pub by_currency: serde_json::Value,
}

// ============================================================================
// Customer Statement / Balance Forward Billing Types (Oracle Fusion: AR > Billing)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomerStatement {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub statement_number: String,
    pub customer_id: Uuid,
    pub customer_number: Option<String>,
    pub customer_name: Option<String>,
    pub statement_date: chrono::NaiveDate,
    pub billing_period_from: chrono::NaiveDate,
    pub billing_period_to: chrono::NaiveDate,
    pub billing_cycle: String,
    pub opening_balance: String,
    pub total_charges: String,
    pub total_payments: String,
    pub total_credits: String,
    pub total_adjustments: String,
    pub closing_balance: String,
    pub amount_due: String,
    pub aging_current: String,
    pub aging_1_30: String,
    pub aging_31_60: String,
    pub aging_61_90: String,
    pub aging_91_120: String,
    pub aging_121_plus: String,
    pub currency_code: String,
    pub delivery_method: Option<String>,
    pub delivery_email: Option<String>,
    pub status: String,
    pub generated_at: Option<DateTime<Utc>>,
    pub sent_at: Option<DateTime<Utc>>,
    pub viewed_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub previous_statement_id: Option<Uuid>,
    pub created_by: Option<Uuid>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomerStatementLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub statement_id: Uuid,
    pub line_type: String,
    pub transaction_id: Option<Uuid>,
    pub transaction_number: Option<String>,
    pub transaction_date: Option<chrono::NaiveDate>,
    pub due_date: Option<chrono::NaiveDate>,
    pub original_amount: Option<String>,
    pub amount: String,
    pub running_balance: Option<String>,
    pub description: Option<String>,
    pub reference_type: Option<String>,
    pub reference_id: Option<Uuid>,
    pub display_order: i32,
    pub metadata: serde_json::Value,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomerStatementSummary {
    pub total_statements: i32,
    pub draft_count: i32,
    pub sent_count: i32,
    pub total_amount_outstanding: String,
    pub by_billing_cycle: serde_json::Value,
    pub by_currency: serde_json::Value,
}

// ============================================================================
// Doubtful Account Allowance / Bad Debt Provision
// Oracle Fusion: Receivables > Collections > Allowance for Doubtful Accounts
// ============================================================================

/// Provision Policy definition
/// Defines the method and parameters for calculating the allowance for doubtful accounts.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DoubtfulAccountPolicy {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub policy_code: String,
    pub policy_name: String,
    pub description: Option<String>,
    /// Calculation method: `aging_based`, `percentage_based`, `specific_identification`
    pub calculation_method: String,
    /// Flat percentage for `percentage_based` method
    pub flat_percentage: String,
    /// GL account for provision credit (Allowance for Doubtful Accounts)
    pub default_provision_account: Option<String>,
    /// GL account for provision debit (Bad Debt Expense)
    pub default_expense_account: Option<String>,
    pub currency_code: String,
    pub effective_from: chrono::NaiveDate,
    pub effective_to: Option<chrono::NaiveDate>,
    pub is_active: bool,
    pub status: String,
    pub aging_buckets: Vec<AgingBucketDefinition>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Aging bucket definition within a provision policy
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgingBucketDefinition {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub policy_id: Uuid,
    pub bucket_name: String,
    /// Start of aging range in days (inclusive)
    pub from_days: i32,
    /// End of aging range in days (inclusive), None = unlimited
    pub to_days: Option<i32>,
    /// Percentage of outstanding balance to provision
    pub provision_percentage: String,
    pub display_order: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Provision Run - records each execution of the provision calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProvisionRun {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub run_number: String,
    pub policy_id: Uuid,
    pub policy_code: String,
    pub run_date: chrono::NaiveDate,
    /// The date for which the provision is calculated
    pub as_of_date: chrono::NaiveDate,
    pub calculation_method: String,
    /// draft, calculated, posted, reversed, cancelled
    pub status: String,
    pub total_outstanding_amount: String,
    pub total_provision_amount: String,
    pub total_prior_provision: String,
    pub incremental_provision: String,
    pub currency_code: String,
    pub journal_batch_id: Option<Uuid>,
    pub journal_entry_number: Option<String>,
    pub description: Option<String>,
    pub customer_count: i32,
    pub transaction_count: i32,
    pub posted_by: Option<Uuid>,
    pub posted_at: Option<DateTime<Utc>>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Provision Run Detail - line-level detail per aging bucket
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProvisionRunDetail {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub run_id: Uuid,
    pub bucket_id: Option<Uuid>,
    pub bucket_name: String,
    pub from_days: i32,
    pub to_days: Option<i32>,
    pub provision_percentage: String,
    pub outstanding_amount: String,
    pub transaction_count: i32,
    pub customer_count: i32,
    pub provision_amount: String,
    pub created_at: DateTime<Utc>,
}

/// Provision Run Activity - audit trail entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProvisionRunActivity {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub run_id: Option<Uuid>,
    pub policy_id: Option<Uuid>,
    pub action: String,
    pub description: Option<String>,
    pub performed_by: Option<Uuid>,
    pub performed_at: DateTime<Utc>,
    pub old_status: Option<String>,
    pub new_status: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// Dashboard summary for Doubtful Account Allowance
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DoubtfulAccountDashboard {
    pub organization_id: Uuid,
    pub total_policies: i32,
    pub active_policies: i32,
    pub total_runs: i32,
    pub draft_runs: i32,
    pub posted_runs: i32,
    pub latest_provision_amount: String,
    pub latest_run_date: Option<String>,
    pub total_outstanding_ar: String,
    pub overall_provision_rate: String,
}

// ============================================================================
// Automatic Offsets (Intercompany Balancing)
// Oracle Fusion: Financials > General Ledger > Automatic Offsets
// ============================================================================

/// Automatic Offset Template
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoOffsetTemplate {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub template_code: String,
    pub template_name: String,
    pub description: Option<String>,
    pub balancing_segment: String,
    pub intercompany_segment: Option<String>,
    pub generation_method: String,
    pub default_offset_account: String,
    pub default_offset_account_description: Option<String>,
    pub enable_intra_entity: bool,
    pub intra_entity_account: Option<String>,
    pub is_active: bool,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// Automatic Offset Template Line
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoOffsetTemplateLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub template_id: Uuid,
    pub line_number: i32,
    pub balancing_segment_value: String,
    pub due_to_account: String,
    pub due_to_account_description: Option<String>,
    pub due_from_account: String,
    pub due_from_account_description: Option<String>,
    pub clearing_account: Option<String>,
    pub priority: i32,
    pub metadata: serde_json::Value,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// Automatic Offset Generation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoOffsetGeneration {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub generation_number: String,
    pub template_id: Uuid,
    pub source_type: String,
    pub source_id: Option<Uuid>,
    pub source_number: Option<String>,
    pub fiscal_year: i32,
    pub period_name: String,
    pub generation_date: chrono::NaiveDate,
    pub currency_code: String,
    pub total_source_lines: i32,
    pub balancing_segments_affected: i32,
    pub total_offset_lines: i32,
    pub total_debit_amount: f64,
    pub total_credit_amount: f64,
    pub status: String,
    pub gl_batch_id: Option<Uuid>,
    pub posted_by: Option<Uuid>,
    pub posted_at: Option<DateTime<Utc>>,
    pub reversed_by: Option<Uuid>,
    pub reversed_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// Automatic Offset Line
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoOffsetLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub generation_id: Uuid,
    pub line_number: i32,
    pub from_segment_value: String,
    pub to_segment_value: String,
    pub offset_type: String,
    pub account_code: String,
    pub account_description: Option<String>,
    pub amount: f64,
    pub currency_code: String,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// Automatic Offset Activity
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoOffsetActivity {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub generation_id: Uuid,
    pub line_id: Option<Uuid>,
    pub activity_type: String,
    pub description: Option<String>,
    pub old_status: Option<String>,
    pub new_status: Option<String>,
    pub performed_by: Option<Uuid>,
    pub performed_by_name: Option<String>,
    pub details: serde_json::Value,
    pub created_at: Option<DateTime<Utc>>,
}

/// Automatic Offset Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoOffsetDashboard {
    pub total_templates: i32,
    pub active_templates: i32,
    pub total_generations: i32,
    pub generated_count: i32,
    pub posted_count: i32,
    pub reversed_count: i32,
    pub total_offset_lines: i32,
    pub total_offset_amount: f64,
}

// ============================================================================
// Transaction Calendar Management
// Oracle Fusion: General Ledger > Setup > Transaction Calendars
// ============================================================================

/// A transaction calendar definition.
///
/// Defines working days, holidays, and exception dates for business date calculations.
/// Used by AP/AR for due date calculation, GL for posting date validation,
/// and Cash Management for forecasting.
/// Oracle Fusion: General Ledger > Setup > Transaction Calendars
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionCalendar {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    /// Working days as ISO weekday numbers (1=Monday, 7=Sunday). Default: [1,2,3,4,5]
    pub working_days: serde_json::Value,
    pub status: String, // active, inactive
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A calendar exception (holiday, non-working day, or special working day).
/// Oracle Fusion: General Ledger > Setup > Transaction Calendars > Exceptions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarException {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub calendar_id: Uuid,
    pub exception_date: chrono::NaiveDate,
    /// "holiday", "`non_working`", "`special_working`"
    pub exception_type: String,
    pub name: String,
    pub description: Option<String>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// An audit record for a business date calculation.
/// Tracks every date calculation performed for compliance and traceability.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarDateCalculation {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub calendar_id: Uuid,
    pub calendar_code: String,
    /// "`next_business_day`", "`previous_business_day`", "`is_business_day`", "`add_business_days`"
    pub operation: String,
    pub input_date: chrono::NaiveDate,
    pub result_date: Option<chrono::NaiveDate>,
    pub result_boolean: Option<bool>,
    pub business_days_added: Option<i32>,
    pub reference_type: Option<String>,
    pub reference_id: Option<Uuid>,
    pub calculated_at: DateTime<Utc>,
    pub calculated_by: Option<Uuid>,
}

/// Dashboard summary for transaction calendars.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionCalendarDashboard {
    pub total_calendars: i32,
    pub active_calendars: i32,
    pub total_exceptions: i32,
    pub total_calculations: i64,
    pub recent_calculations: Vec<CalendarDateCalculation>,
}
