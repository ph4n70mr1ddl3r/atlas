use crate::types::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
// ============================================================================
// Payment Management (Oracle Fusion Payables > Payments)
// ============================================================================

/// Payment terms definition
/// Oracle Fusion: Payables > Setup > Payment Terms
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentTerm {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Unique code (e.g., "NET30", "`2_10_NET30`", "`DUE_ON_RECEIPT`")
    pub code: String,
    /// Display name (e.g., "Net 30 Days")
    pub name: String,
    pub description: Option<String>,
    /// Number of days from invoice date until payment is due
    pub due_days: i32,
    /// Days within which a discount is available
    pub discount_days: Option<i32>,
    /// Discount percentage for early payment
    pub discount_percentage: Option<String>,
    /// Whether this is an installment payment term
    pub is_installment: bool,
    /// Number of installments
    pub installment_count: Option<i32>,
    /// Installment frequency: 'monthly', 'quarterly', 'weekly'
    pub installment_frequency: Option<String>,
    /// Default payment method
    pub default_payment_method: Option<String>,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create/update payment terms request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentTermRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(default = "default_thirty")]
    pub due_days: i32,
    pub discount_days: Option<i32>,
    pub discount_percentage: Option<String>,
    #[serde(default)]
    pub is_installment: bool,
    pub installment_count: Option<i32>,
    pub installment_frequency: Option<String>,
    pub default_payment_method: Option<String>,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
}

pub const fn default_thirty() -> i32 {
    30
}

/// Payment batch (payment run)
/// Oracle Fusion: Payables > Payments > Payment Batches
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentBatch {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub batch_number: String,
    pub name: Option<String>,
    pub description: Option<String>,
    /// When payments should be issued
    pub payment_date: chrono::NaiveDate,
    /// Bank account to pay from
    pub bank_account_id: Option<Uuid>,
    /// Payment method: 'check', 'eft', 'wire', 'ach'
    pub payment_method: String,
    pub currency_code: String,
    /// Selection criteria used to select invoices for payment
    pub selection_criteria: serde_json::Value,
    /// Counts and totals
    pub total_invoice_count: i32,
    pub total_payment_count: i32,
    pub total_payment_amount: String,
    pub total_discount_taken: String,
    /// Status: 'draft', 'selected', 'approved', 'formatted', 'confirmed', 'cancelled'
    pub status: String,
    /// Workflow tracking
    pub selected_by: Option<Uuid>,
    pub selected_at: Option<DateTime<Utc>>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub formatted_by: Option<Uuid>,
    pub formatted_at: Option<DateTime<Utc>>,
    pub confirmed_by: Option<Uuid>,
    pub confirmed_at: Option<DateTime<Utc>>,
    pub cancelled_by: Option<Uuid>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub cancellation_reason: Option<String>,
    /// Generated payment file reference
    pub payment_file_name: Option<String>,
    pub payment_file_reference: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create payment batch request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentBatchRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub payment_date: chrono::NaiveDate,
    pub bank_account_id: Option<Uuid>,
    #[serde(default = "default_check_method")]
    pub payment_method: String,
    #[serde(default = "default_currency_usd")]
    pub currency_code: String,
    pub selection_criteria: Option<serde_json::Value>,
}

pub fn default_check_method() -> String {
    "check".to_string()
}

/// Individual payment
/// Oracle Fusion: Payables > Payments > Payments
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Payment {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub payment_number: String,
    pub batch_id: Option<Uuid>,
    /// Supplier information
    pub supplier_id: Uuid,
    pub supplier_number: Option<String>,
    pub supplier_name: Option<String>,
    pub supplier_site: Option<String>,
    /// Payment details
    pub payment_date: chrono::NaiveDate,
    pub payment_method: String,
    pub currency_code: String,
    /// Amounts
    pub payment_amount: String,
    pub discount_taken: String,
    pub bank_charges: String,
    /// Bank account (source of funds)
    pub bank_account_id: Option<Uuid>,
    pub bank_account_name: Option<String>,
    /// GL account codes
    pub cash_account_code: Option<String>,
    pub ap_account_code: Option<String>,
    pub discount_account_code: Option<String>,
    /// Status: 'draft', 'issued', 'cleared', 'voided', 'reconciled', 'stopped'
    pub status: String,
    /// Check / reference number
    pub check_number: Option<String>,
    pub reference_number: Option<String>,
    /// Void tracking
    pub voided_by: Option<Uuid>,
    pub voided_at: Option<DateTime<Utc>>,
    pub void_reason: Option<String>,
    /// Reissue tracking
    pub reissued_from_payment_id: Option<Uuid>,
    pub reissued_payment_id: Option<Uuid>,
    /// Clearance tracking
    pub cleared_date: Option<chrono::NaiveDate>,
    pub cleared_by: Option<Uuid>,
    pub cleared_at: Option<DateTime<Utc>>,
    /// GL posting
    pub journal_entry_id: Option<Uuid>,
    pub posted_at: Option<DateTime<Utc>>,
    /// Remittance
    pub remittance_sent: bool,
    pub remittance_sent_at: Option<DateTime<Utc>>,
    pub remittance_method: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Payment line (invoice covered by a payment)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub payment_id: Uuid,
    pub line_number: i32,
    /// Invoice reference
    pub invoice_id: Uuid,
    pub invoice_number: Option<String>,
    pub invoice_date: Option<chrono::NaiveDate>,
    pub invoice_due_date: Option<chrono::NaiveDate>,
    /// Amounts
    pub invoice_amount: Option<String>,
    pub amount_paid: String,
    pub discount_taken: String,
    pub withholding_amount: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Scheduled payment
/// Oracle Fusion: Payables > Payments > Scheduled Payments
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduledPayment {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub invoice_id: Uuid,
    pub invoice_number: Option<String>,
    pub supplier_id: Uuid,
    pub supplier_name: Option<String>,
    /// Scheduling
    pub scheduled_payment_date: chrono::NaiveDate,
    pub scheduled_amount: String,
    pub installment_number: i32,
    pub payment_method: Option<String>,
    pub bank_account_id: Option<Uuid>,
    /// Batch selection
    pub is_selected: bool,
    pub selected_batch_id: Option<Uuid>,
    pub payment_id: Option<Uuid>,
    /// Status: 'pending', 'selected', 'paid', 'cancelled'
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Payment format
/// Oracle Fusion: Payables > Setup > Payment Formats
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentFormat {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    /// Format type: 'file', '`printed_check`', 'edi', 'xml', 'json'
    pub format_type: String,
    pub template_reference: Option<String>,
    pub applicable_methods: serde_json::Value,
    pub is_system: bool,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Remittance advice
/// Oracle Fusion: Payables > Payments > Remittance Advice
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemittanceAdvice {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub payment_id: Uuid,
    /// Delivery
    pub delivery_method: String,
    pub delivery_address: Option<String>,
    pub contact_name: Option<String>,
    pub contact_email: Option<String>,
    /// Content
    pub subject: Option<String>,
    pub body: Option<String>,
    /// Status: 'pending', 'sent', 'delivered', 'failed'
    pub status: String,
    pub sent_at: Option<DateTime<Utc>>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub failure_reason: Option<String>,
    /// Payment summary
    pub payment_summary: serde_json::Value,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Payment dashboard summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentDashboardSummary {
    pub total_pending_payment_count: i32,
    pub total_pending_payment_amount: String,
    pub total_paid_payment_count: i32,
    pub total_paid_payment_amount: String,
    pub total_discount_taken: String,
    pub payments_by_method: serde_json::Value,
    pub payments_by_status: serde_json::Value,
    /// Upcoming scheduled payments (next 7 days)
    pub upcoming_scheduled_count: i32,
    pub upcoming_scheduled_amount: String,
}

/// Revenue Contract Modification
/// Tracks changes/amendments to revenue contracts.
/// Oracle Fusion equivalent: Revenue Management > Contract Modifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevenueModification {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Contract being modified
    pub contract_id: Uuid,
    /// Modification number (sequential)
    pub modification_number: i32,
    /// Type of modification: "`price_change`", "`scope_change`", "`term_extension`",
    /// "termination", "`add_obligation`", "`remove_obligation`"
    pub modification_type: String,
    /// Description of the change
    pub description: Option<String>,
    /// Previous total transaction price
    pub previous_transaction_price: String,
    /// New total transaction price
    pub new_transaction_price: String,
    /// Previous contract end date
    pub previous_end_date: Option<chrono::NaiveDate>,
    /// New contract end date
    pub new_end_date: Option<chrono::NaiveDate>,
    /// Effective date of the modification
    pub effective_date: chrono::NaiveDate,
    /// Status: "draft", "active", "cancelled"
    pub status: String,
    /// Arbitrary metadata
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// Corporate Card Management
// Oracle Fusion Cloud ERP: Financials > Expenses > Corporate Cards
// ============================================================================

/// Corporate Card Program - defines a card programme with an issuer bank.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CorporateCardProgram {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub program_code: String,
    pub name: String,
    pub description: Option<String>,
    pub issuer_bank: String,
    pub card_network: String,
    /// e.g. "corporate", "purchasing", "travel"
    pub card_type: String,
    pub currency_code: String,
    pub default_single_purchase_limit: String,
    pub default_monthly_limit: String,
    pub default_cash_limit: String,
    pub default_atm_limit: String,
    pub allow_cash_withdrawal: bool,
    pub allow_international: bool,
    pub auto_deactivate_on_termination: bool,
    pub expense_matching_method: String,
    pub billing_cycle_day: i32,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Corporate Card - an individual card issued to an employee.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CorporateCard {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub program_id: Uuid,
    pub card_number_masked: String,
    pub cardholder_name: String,
    pub cardholder_id: Uuid,
    pub cardholder_email: Option<String>,
    pub department_id: Option<Uuid>,
    pub department_name: Option<String>,
    /// "active", "suspended", "cancelled", "expired", "lost", "stolen"
    pub status: String,
    pub issue_date: chrono::NaiveDate,
    pub expiry_date: chrono::NaiveDate,
    pub single_purchase_limit: String,
    pub monthly_limit: String,
    pub cash_limit: String,
    pub atm_limit: String,
    pub current_balance: String,
    pub total_spend_current_cycle: String,
    pub last_statement_balance: String,
    pub last_statement_date: Option<chrono::NaiveDate>,
    pub gl_liability_account: Option<String>,
    pub gl_expense_account: Option<String>,
    pub cost_center: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Corporate Card Transaction - a charge or credit on a corporate card.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CorporateCardTransaction {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub card_id: Uuid,
    pub program_id: Uuid,
    pub transaction_reference: String,
    pub posting_date: chrono::NaiveDate,
    pub transaction_date: chrono::NaiveDate,
    pub merchant_name: String,
    pub merchant_category: Option<String>,
    pub merchant_category_code: Option<String>,
    pub amount: String,
    pub currency_code: String,
    pub original_amount: Option<String>,
    pub original_currency: Option<String>,
    pub exchange_rate: Option<String>,
    /// "charge", "credit", "payment", "`cash_withdrawal`", "fee", "interest"
    pub transaction_type: String,
    /// "unmatched", "matched", "disputed", "approved", "rejected"
    pub status: String,
    pub expense_report_id: Option<Uuid>,
    pub expense_line_id: Option<Uuid>,
    pub matched_at: Option<DateTime<Utc>>,
    pub matched_by: Option<Uuid>,
    pub match_confidence: Option<String>,
    pub dispute_reason: Option<String>,
    pub dispute_date: Option<chrono::NaiveDate>,
    pub dispute_resolution: Option<String>,
    pub gl_posted: bool,
    pub gl_journal_id: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Corporate Card Statement - a monthly billing statement from the card issuer.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CorporateCardStatement {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub program_id: Uuid,
    pub statement_number: String,
    pub statement_date: chrono::NaiveDate,
    pub billing_period_start: chrono::NaiveDate,
    pub billing_period_end: chrono::NaiveDate,
    pub opening_balance: String,
    pub closing_balance: String,
    pub total_charges: String,
    pub total_credits: String,
    pub total_payments: String,
    pub total_fees: String,
    pub total_interest: String,
    pub payment_due_date: Option<chrono::NaiveDate>,
    pub minimum_payment: String,
    pub total_transaction_count: i32,
    pub matched_transaction_count: i32,
    pub unmatched_transaction_count: i32,
    /// "imported", "processing", "matched", "reconciled", "paid"
    pub status: String,
    pub payment_reference: Option<String>,
    pub paid_at: Option<DateTime<Utc>>,
    pub gl_payment_journal_id: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub imported_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Corporate Card Spending Limit Override - temporary or permanent limit changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CorporateCardLimitOverride {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub card_id: Uuid,
    pub override_type: String,
    pub original_value: String,
    pub new_value: String,
    pub reason: String,
    pub effective_from: chrono::NaiveDate,
    pub effective_to: Option<chrono::NaiveDate>,
    /// "pending", "approved", "rejected", "expired"
    pub status: String,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Corporate Card Dashboard Summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CorporateCardDashboardSummary {
    pub total_active_cards: i32,
    pub total_programs: i32,
    pub total_cards_by_status: serde_json::Value,
    pub total_spend_current_month: String,
    pub total_spend_previous_month: String,
    pub spend_change_percent: String,
    pub total_unmatched_transactions: i32,
    pub total_unreconciled_statements: i32,
    pub total_disputed_transactions: i32,
    pub top_spenders: serde_json::Value,
    pub spend_by_category: serde_json::Value,
    pub limit_overrides_pending: i32,
}

/// Grant Management Dashboard Summary
/// Oracle Fusion: Grants Management > Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GrantDashboardSummary {
    pub total_active_awards: i32,
    pub total_sponsors: i32,
    pub total_award_value: String,
    pub total_funded: String,
    pub total_expenditures: String,
    pub total_available_balance: String,
    pub total_pending_billings: i32,
    pub total_overdue_reports: i32,
    pub awards_expiring_30_days: i32,
    pub budget_utilization_percent: String,
    pub awards_by_status: serde_json::Value,
    pub expenditures_by_category: serde_json::Value,
    pub top_sponsors: serde_json::Value,
}

// ============================================================================
// Accounts Payable (Oracle Fusion Payables)
// ============================================================================

/// AP Invoice (Supplier Invoice)
/// Oracle Fusion: Payables > Invoices
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApInvoice {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub invoice_number: String,
    pub invoice_date: chrono::NaiveDate,
    pub invoice_type: String, // standard, credit_memo, debit_memo, prepayment, expense_report, po_default
    pub description: Option<String>,
    pub supplier_id: Uuid,
    pub supplier_number: Option<String>,
    pub supplier_name: Option<String>,
    pub supplier_site: Option<String>,
    pub invoice_currency_code: String,
    pub payment_currency_code: String,
    pub exchange_rate: Option<String>,
    pub exchange_rate_type: Option<String>,
    pub exchange_date: Option<chrono::NaiveDate>,
    pub invoice_amount: String,
    pub tax_amount: String,
    pub total_amount: String,
    pub amount_paid: String,
    pub amount_remaining: String,
    pub discount_available: String,
    pub discount_taken: String,
    pub payment_terms: Option<String>,
    pub payment_method: Option<String>,
    pub payment_due_date: Option<chrono::NaiveDate>,
    pub discount_date: Option<chrono::NaiveDate>,
    pub gl_date: Option<chrono::NaiveDate>,
    pub gl_posted_date: Option<chrono::DateTime<chrono::Utc>>,
    pub status: String, // draft, submitted, approved, paid, cancelled, on_hold
    pub approval_status: Option<String>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<chrono::DateTime<chrono::Utc>>,
    pub cancelled_reason: Option<String>,
    pub cancelled_by: Option<Uuid>,
    pub cancelled_at: Option<chrono::DateTime<chrono::Utc>>,
    pub po_number: Option<String>,
    pub receipt_number: Option<String>,
    pub source: Option<String>,
    pub batch_id: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// AP Invoice Line
/// Oracle Fusion: Payables > Invoices > Lines
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApInvoiceLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub invoice_id: Uuid,
    pub line_number: i32,
    pub line_type: String, // item, freight, tax, miscellaneous, withholding
    pub description: Option<String>,
    pub amount: String,
    pub unit_price: Option<String>,
    pub quantity_invoiced: Option<String>,
    pub unit_of_measure: Option<String>,
    pub po_line_id: Option<Uuid>,
    pub po_line_number: Option<String>,
    pub product_code: Option<String>,
    pub tax_code: Option<String>,
    pub tax_amount: Option<String>,
    pub asset_category_code: Option<String>,
    pub project_id: Option<Uuid>,
    pub task_id: Option<Uuid>,
    pub expenditure_type: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// AP Invoice Distribution (accounting)
/// Oracle Fusion: Payables > Invoices > Distributions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApInvoiceDistribution {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub invoice_id: Uuid,
    pub invoice_line_id: Option<Uuid>,
    pub distribution_line_number: i32,
    pub distribution_type: String, // charge, tax, withholding, variance
    pub account_combination: Option<String>,
    pub description: Option<String>,
    pub amount: String,
    pub base_amount: Option<String>,
    pub currency_code: String,
    pub exchange_rate: Option<String>,
    pub gl_account: Option<String>,
    pub cost_center: Option<String>,
    pub department: Option<String>,
    pub project_id: Option<Uuid>,
    pub task_id: Option<Uuid>,
    pub expenditure_type: Option<String>,
    pub tax_code: Option<String>,
    pub tax_recoverable: bool,
    pub tax_recoverable_amount: Option<String>,
    pub accounting_date: Option<chrono::NaiveDate>,
    pub posted_status: String, // unposted, posted, error
    pub posted_at: Option<chrono::DateTime<chrono::Utc>>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// AP Invoice Hold
/// Oracle Fusion: Payables > Invoices > Holds
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApInvoiceHold {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub invoice_id: Uuid,
    pub hold_type: String, // system, manual, matching, approval, variance, budget
    pub hold_reason: String,
    pub hold_status: String, // active, released
    pub released_by: Option<Uuid>,
    pub released_at: Option<chrono::DateTime<chrono::Utc>>,
    pub release_reason: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// AP Payment (payment against invoices)
/// Oracle Fusion: Payables > Payments
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApPayment {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub payment_number: String,
    pub payment_date: chrono::NaiveDate,
    pub payment_method: String,
    pub payment_currency_code: String,
    pub payment_amount: String,
    pub bank_account_id: Option<Uuid>,
    pub bank_account_name: Option<String>,
    pub payment_document: Option<String>,
    pub status: String, // draft, submitted, confirmed, cancelled, reversed
    pub supplier_id: Uuid,
    pub supplier_number: Option<String>,
    pub supplier_name: Option<String>,
    pub invoice_ids: serde_json::Value,
    pub confirmed_by: Option<Uuid>,
    pub confirmed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub cancelled_reason: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// AP Aging Summary
/// Oracle Fusion: Payables > Reports > Aging Report
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApAgingSummary {
    pub organization_id: Uuid,
    pub as_of_date: chrono::NaiveDate,
    pub total_outstanding: String,
    pub current_amount: String,
    pub aging_1_30: String,
    pub aging_31_60: String,
    pub aging_61_90: String,
    pub aging_91_plus: String,
    pub supplier_count: i32,
    pub invoice_count: i32,
    pub by_supplier: Vec<ApAgingBySupplier>,
}

/// AP Aging by Supplier
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApAgingBySupplier {
    pub supplier_id: Uuid,
    pub supplier_name: String,
    pub supplier_number: Option<String>,
    pub total_outstanding: String,
    pub current_amount: String,
    pub aging_1_30: String,
    pub aging_31_60: String,
    pub aging_61_90: String,
    pub aging_91_plus: String,
    pub invoice_count: i32,
}

/// Cost Accounting Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CostAccountingDashboard {
    pub total_cost_books: i32,
    pub active_cost_books: i32,
    pub total_cost_elements: i32,
    pub total_standard_costs: i32,
    pub total_adjustments: i32,
    pub pending_adjustments: i32,
    pub total_variances: i32,
    pub unfavorable_variances: i32,
    pub total_standard_cost_value: String,
    pub total_variance_amount: String,
    pub variance_by_type: serde_json::Value,
    pub variance_by_element: serde_json::Value,
    pub adjustments_by_status: serde_json::Value,
}

// ============================================================================
// Remittance Batch Types (Oracle Fusion: AR > Receipts > Remittance Batches)
// ============================================================================

/// Remittance Batch
/// Oracle Fusion equivalent: Receivables > Receipts > Automatic Receipts > Remittance Batches
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemittanceBatch {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub batch_number: String,
    pub batch_name: Option<String>,
    pub bank_account_id: Option<Uuid>,
    pub bank_account_name: Option<String>,
    pub bank_name: Option<String>,
    pub remittance_method: String,
    pub currency_code: String,
    pub batch_date: chrono::NaiveDate,
    pub gl_date: Option<chrono::NaiveDate>,
    pub receipt_currency_code: Option<String>,
    pub exchange_rate_type: Option<String>,
    pub status: String,
    pub total_amount: String,
    pub receipt_count: i32,
    pub format_program: Option<String>,
    pub format_date: Option<DateTime<Utc>>,
    pub transmission_date: Option<DateTime<Utc>>,
    pub confirmation_date: Option<DateTime<Utc>>,
    pub settlement_date: Option<DateTime<Utc>>,
    pub reversal_date: Option<DateTime<Utc>>,
    pub reference_number: Option<String>,
    pub remittance_advice_sent: Option<bool>,
    pub remittance_advice_date: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub created_by: Option<Uuid>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// Remittance Batch Receipt
/// Individual receipt included in a remittance batch
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemittanceBatchReceipt {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub batch_id: Uuid,
    pub receipt_id: Uuid,
    pub receipt_number: Option<String>,
    pub customer_id: Option<Uuid>,
    pub customer_number: Option<String>,
    pub customer_name: Option<String>,
    pub receipt_date: Option<chrono::NaiveDate>,
    pub receipt_amount: String,
    pub applied_amount: String,
    pub receipt_method: Option<String>,
    pub currency_code: String,
    pub exchange_rate: Option<String>,
    pub status: String,
    pub display_order: i32,
    pub metadata: serde_json::Value,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// Remittance Batch Dashboard Summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemittanceBatchSummary {
    pub total_batches: i32,
    pub draft_count: i32,
    pub approved_count: i32,
    pub settled_count: i32,
    pub total_amount: String,
    pub total_receipts: i32,
    pub by_status: serde_json::Value,
    pub by_currency: serde_json::Value,
}
