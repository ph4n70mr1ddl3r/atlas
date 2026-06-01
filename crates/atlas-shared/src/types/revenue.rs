use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
// ============================================================================
// Revenue Recognition (ASC 606 / IFRS 15)
// Oracle Fusion Cloud ERP: Financials > Revenue Management
// ============================================================================

/// Revenue Recognition Policy
/// Defines the accounting policy for revenue recognition.
/// Oracle Fusion equivalent: Revenue Management > Policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevenuePolicy {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Unique policy code (e.g., "`STD_SaaS`", "`STD_CONSULTING`")
    pub code: String,
    /// Human-readable name
    pub name: String,
    /// Description of the policy
    pub description: Option<String>,
    /// Recognition method: "`over_time`", "`point_in_time`"
    pub recognition_method: String,
    /// Over-time method (when `recognition_method` = `over_time)`:
    /// "output", "input", "`straight_line`"
    pub over_time_method: Option<String>,
    /// Allocation basis: "`standalone_selling_price`", "residual", "equal"
    pub allocation_basis: String,
    /// Default standalone selling price (used when SSP is not determined per-product)
    pub default_selling_price: Option<String>,
    /// Whether variable consideration is constrained
    pub constrain_variable_consideration: bool,
    /// Constraint threshold percentage (0-100)
    pub constraint_threshold_percent: Option<String>,
    /// Default revenue account code
    pub revenue_account_code: Option<String>,
    /// Default deferred revenue account code
    pub deferred_revenue_account_code: Option<String>,
    /// Default contra-revenue account code (for allowances)
    pub contra_revenue_account_code: Option<String>,
    /// Whether this policy is active
    pub is_active: bool,
    /// Arbitrary metadata
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Revenue Contract (Revenue Arrangement)
/// Represents a customer contract with one or more performance obligations.
/// Oracle Fusion equivalent: Revenue Management > Revenue Contracts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevenueContract {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Auto-generated contract number (e.g., "RC-0001")
    pub contract_number: String,
    /// Reference to the source sales order or agreement
    pub source_type: Option<String>,
    pub source_id: Option<Uuid>,
    pub source_number: Option<String>,
    /// Customer information
    pub customer_id: Uuid,
    pub customer_number: Option<String>,
    pub customer_name: Option<String>,
    /// Contract dates
    pub contract_date: Option<chrono::NaiveDate>,
    pub start_date: Option<chrono::NaiveDate>,
    pub end_date: Option<chrono::NaiveDate>,
    /// Total transaction price (before allocation)
    pub total_transaction_price: String,
    /// Total allocated revenue across all performance obligations
    pub total_allocated_revenue: String,
    /// Total recognized revenue to date
    pub total_recognized_revenue: String,
    /// Total deferred revenue remaining
    pub total_deferred_revenue: String,
    /// Contract status: "draft", "active", "completed", "cancelled", "modified"
    pub status: String,
    /// ASC 606 step completion tracking
    /// Step 1: Identify the contract
    pub step1_contract_identified: bool,
    /// Step 2: Identify performance obligations (POs created)
    pub step2_obligations_identified: bool,
    /// Step 3: Determine transaction price
    pub step3_price_determined: bool,
    /// Step 4: Allocate transaction price
    pub step4_price_allocated: bool,
    /// Step 5: Recognize revenue
    pub step5_recognition_scheduled: bool,
    /// Currency
    pub currency_code: String,
    /// Optional notes
    pub notes: Option<String>,
    /// Arbitrary metadata
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Performance Obligation
/// A distinct good or service promised in a revenue contract.
/// Oracle Fusion equivalent: Revenue Management > Performance Obligations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceObligation {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Parent revenue contract
    pub contract_id: Uuid,
    /// Line number within the contract
    pub line_number: i32,
    /// Description of the good or service
    pub description: Option<String>,
    /// Product or service reference
    pub product_id: Option<Uuid>,
    pub product_name: Option<String>,
    /// Reference to source line (e.g., sales order line)
    pub source_line_id: Option<Uuid>,
    /// Revenue policy applied to this obligation
    pub revenue_policy_id: Option<Uuid>,
    /// Recognition method for this specific obligation
    /// (overrides policy default if set)
    pub recognition_method: Option<String>,
    /// Over-time method override
    pub over_time_method: Option<String>,
    /// Standalone selling price (SSP)
    pub standalone_selling_price: String,
    /// Allocated transaction price (after SSP allocation)
    pub allocated_transaction_price: String,
    /// Total recognized revenue for this obligation
    pub total_recognized_revenue: String,
    /// Remaining deferred revenue
    pub deferred_revenue: String,
    /// Recognition start date
    pub recognition_start_date: Option<chrono::NaiveDate>,
    /// Recognition end date (for over-time)
    pub recognition_end_date: Option<chrono::NaiveDate>,
    /// Percent complete (for over-time recognition)
    pub percent_complete: Option<String>,
    /// Satisfaction method: "`over_time`", "`point_in_time`"
    pub satisfaction_method: String,
    /// Status: "pending", "`in_progress`", "satisfied", "`partially_satisfied`", "cancelled"
    pub status: String,
    /// Revenue account overrides
    pub revenue_account_code: Option<String>,
    pub deferred_revenue_account_code: Option<String>,
    /// Arbitrary metadata
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Revenue Recognition Schedule Line
/// Individual revenue recognition events for a performance obligation.
/// Oracle Fusion equivalent: Revenue Management > Revenue Schedules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevenueScheduleLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Parent performance obligation
    pub obligation_id: Uuid,
    /// Parent contract (denormalized for querying)
    pub contract_id: Uuid,
    /// Schedule line number
    pub line_number: i32,
    /// Planned recognition date
    pub recognition_date: chrono::NaiveDate,
    /// Amount to recognize
    pub amount: String,
    /// Amount actually recognized
    pub recognized_amount: String,
    /// Status: "planned", "recognized", "reversed", "cancelled"
    pub status: String,
    /// Recognition method used
    pub recognition_method: Option<String>,
    /// Percentage of total for this line
    pub percent_of_total: Option<String>,
    /// Journal entry reference (posted to GL)
    pub journal_entry_id: Option<Uuid>,
    /// When the recognition was actually posted
    pub recognized_at: Option<DateTime<Utc>>,
    /// Reversal reference
    pub reversed_by_id: Option<Uuid>,
    /// Reason for reversal (if reversed)
    pub reversal_reason: Option<String>,
    /// Arbitrary metadata
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// Subscription Management (Oracle Fusion Subscription Management)
// ============================================================================

/// Subscription product in the catalog
/// Oracle Fusion: Subscription Management > Products
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionProduct {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub product_code: String,
    pub name: String,
    pub description: Option<String>,
    pub product_type: String,
    pub billing_frequency: String,
    pub default_duration_months: i32,
    pub is_auto_renew: bool,
    pub cancellation_notice_days: i32,
    pub setup_fee: String,
    pub tier_type: String,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Subscription product price tier
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionPriceTier {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub product_id: Uuid,
    pub tier_name: Option<String>,
    pub min_quantity: String,
    pub max_quantity: Option<String>,
    pub unit_price: String,
    pub discount_percent: String,
    pub currency_code: String,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Subscription (main header)
/// Oracle Fusion: Subscription Management > Subscriptions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Subscription {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub subscription_number: String,
    pub customer_id: Uuid,
    pub customer_name: Option<String>,
    pub product_id: Uuid,
    pub product_code: Option<String>,
    pub product_name: Option<String>,
    pub description: Option<String>,
    pub status: String,
    pub start_date: chrono::NaiveDate,
    pub end_date: Option<chrono::NaiveDate>,
    pub renewal_date: Option<chrono::NaiveDate>,
    pub billing_frequency: String,
    pub billing_day_of_month: i32,
    pub billing_alignment: String,
    pub currency_code: String,
    pub quantity: String,
    pub unit_price: String,
    pub list_price: String,
    pub discount_percent: String,
    pub setup_fee: String,
    pub recurring_amount: String,
    pub total_contract_value: String,
    pub total_billed: String,
    pub total_revenue_recognized: String,
    pub duration_months: i32,
    pub is_auto_renew: bool,
    pub cancellation_date: Option<chrono::NaiveDate>,
    pub cancellation_reason: Option<String>,
    pub suspension_reason: Option<String>,
    pub sales_rep_id: Option<Uuid>,
    pub sales_rep_name: Option<String>,
    pub gl_revenue_account: Option<String>,
    pub gl_deferred_account: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Subscription amendment (change to an active subscription)
/// Oracle Fusion: Subscription Management > Amendments
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionAmendment {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub subscription_id: Uuid,
    pub amendment_number: String,
    pub amendment_type: String,
    pub description: Option<String>,
    pub old_quantity: Option<String>,
    pub new_quantity: Option<String>,
    pub old_unit_price: Option<String>,
    pub new_unit_price: Option<String>,
    pub old_recurring_amount: Option<String>,
    pub new_recurring_amount: Option<String>,
    pub old_end_date: Option<chrono::NaiveDate>,
    pub new_end_date: Option<chrono::NaiveDate>,
    pub effective_date: chrono::NaiveDate,
    pub proration_credit: Option<String>,
    pub proration_charge: Option<String>,
    pub status: String,
    pub applied_at: Option<DateTime<Utc>>,
    pub applied_by: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Subscription billing schedule line
/// Oracle Fusion: Subscription Management > Billing Schedule
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionBillingLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub subscription_id: Uuid,
    pub schedule_number: i32,
    pub billing_date: chrono::NaiveDate,
    pub period_start: chrono::NaiveDate,
    pub period_end: chrono::NaiveDate,
    pub amount: String,
    pub proration_amount: String,
    pub total_amount: String,
    pub invoice_id: Option<Uuid>,
    pub invoice_number: Option<String>,
    pub status: String,
    pub paid_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Subscription revenue schedule line (ASC 606 / IFRS 15)
/// Oracle Fusion: Subscription Management > Revenue Schedules
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionRevenueLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub subscription_id: Uuid,
    pub billing_schedule_id: Option<Uuid>,
    pub period_name: String,
    pub period_start: chrono::NaiveDate,
    pub period_end: chrono::NaiveDate,
    pub revenue_amount: String,
    pub deferred_amount: String,
    pub recognized_to_date: String,
    pub status: String,
    pub recognized_at: Option<DateTime<Utc>>,
    pub journal_entry_id: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Subscription dashboard summary
/// Oracle Fusion: Subscription Management > Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionDashboardSummary {
    pub total_active_subscriptions: i32,
    pub total_subscribers: i32,
    pub total_monthly_recurring_revenue: String,
    pub total_annual_recurring_revenue: String,
    pub total_contract_value: String,
    pub total_billed: String,
    pub total_revenue_recognized: String,
    pub total_deferred_revenue: String,
    pub churn_rate_percent: String,
    pub renewals_due_30_days: i32,
    pub new_subscriptions_this_month: i32,
    pub cancelled_this_month: i32,
    pub subscriptions_by_status: serde_json::Value,
    pub revenue_by_product: serde_json::Value,
}

// ═══════════════════════════════════════════════════════════════
// Grant Management (Oracle Fusion Grants Management)
// ═══════════════════════════════════════════════════════════════

/// Grant Sponsor (funding organization)
/// Oracle Fusion: Grants Management > Sponsors
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GrantSponsor {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub sponsor_code: String,
    pub name: String,
    pub sponsor_type: String,
    pub country_code: Option<String>,
    pub taxpayer_id: Option<String>,
    pub contact_name: Option<String>,
    pub contact_email: Option<String>,
    pub contact_phone: Option<String>,
    pub address_line1: Option<String>,
    pub address_line2: Option<String>,
    pub city: Option<String>,
    pub state_province: Option<String>,
    pub postal_code: Option<String>,
    pub payment_terms: Option<String>,
    pub billing_frequency: String,
    pub currency_code: String,
    pub credit_limit: Option<String>,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Indirect Cost Rate Agreement
/// Oracle Fusion: Grants Management > Indirect Cost Rates
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GrantIndirectCostRate {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub rate_name: String,
    pub rate_type: String,
    pub rate_percentage: String,
    pub base_type: String,
    pub effective_from: chrono::NaiveDate,
    pub effective_to: Option<chrono::NaiveDate>,
    pub negotiated_by: Option<String>,
    pub approved_by: Option<Uuid>,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Grant Award
/// Oracle Fusion: Grants Management > Awards
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GrantAward {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub award_number: String,
    pub award_title: String,
    pub sponsor_id: Uuid,
    pub sponsor_name: Option<String>,
    pub sponsor_award_number: Option<String>,
    pub status: String,
    pub award_type: String,
    pub award_purpose: Option<String>,
    pub start_date: chrono::NaiveDate,
    pub end_date: chrono::NaiveDate,
    pub budget_start_date: Option<chrono::NaiveDate>,
    pub budget_end_date: Option<chrono::NaiveDate>,
    pub total_award_amount: String,
    pub direct_costs_total: String,
    pub indirect_costs_total: String,
    pub cost_sharing_total: String,
    pub total_funded: String,
    pub total_billed: String,
    pub total_collected: String,
    pub total_expenditures: String,
    pub total_commitments: String,
    pub available_balance: String,
    pub currency_code: String,
    pub indirect_cost_rate_id: Option<Uuid>,
    pub indirect_cost_rate: String,
    pub cost_sharing_required: bool,
    pub cost_sharing_percent: String,
    pub principal_investigator_id: Option<Uuid>,
    pub principal_investigator_name: Option<String>,
    pub department_id: Option<Uuid>,
    pub department_name: Option<String>,
    pub project_id: Option<Uuid>,
    pub cost_center: Option<String>,
    pub gl_revenue_account: Option<String>,
    pub gl_receivable_account: Option<String>,
    pub gl_deferred_account: Option<String>,
    pub billing_frequency: String,
    pub billing_basis: String,
    pub reporting_requirements: Option<String>,
    pub compliance_notes: Option<String>,
    pub closeout_date: Option<chrono::NaiveDate>,
    pub closeout_notes: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Grant Budget Line
/// Oracle Fusion: Grants Management > Award Budgets
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GrantBudgetLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub award_id: Uuid,
    pub line_number: i32,
    pub budget_category: String,
    pub description: Option<String>,
    pub account_code: Option<String>,
    pub budget_amount: String,
    pub committed_amount: String,
    pub expended_amount: String,
    pub billed_amount: String,
    pub available_balance: String,
    pub period_start: Option<chrono::NaiveDate>,
    pub period_end: Option<chrono::NaiveDate>,
    pub fiscal_year: Option<i32>,
    pub notes: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Grant Expenditure
/// Oracle Fusion: Grants Management > Expenditures
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GrantExpenditure {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub award_id: Uuid,
    pub expenditure_number: String,
    pub expenditure_type: String,
    pub expenditure_date: chrono::NaiveDate,
    pub description: Option<String>,
    pub budget_line_id: Option<Uuid>,
    pub budget_category: Option<String>,
    pub amount: String,
    pub indirect_cost_amount: String,
    pub total_amount: String,
    pub cost_sharing_amount: String,
    pub employee_id: Option<Uuid>,
    pub employee_name: Option<String>,
    pub vendor_id: Option<Uuid>,
    pub vendor_name: Option<String>,
    pub source_entity_type: Option<String>,
    pub source_entity_id: Option<Uuid>,
    pub source_entity_number: Option<String>,
    pub gl_debit_account: Option<String>,
    pub gl_credit_account: Option<String>,
    pub status: String,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub billed_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Grant Billing (invoice to sponsor)
/// Oracle Fusion: Grants Management > Billings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GrantBilling {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub award_id: Uuid,
    pub invoice_number: String,
    pub invoice_date: chrono::NaiveDate,
    pub period_start: chrono::NaiveDate,
    pub period_end: chrono::NaiveDate,
    pub due_date: Option<chrono::NaiveDate>,
    pub direct_costs_billed: String,
    pub indirect_costs_billed: String,
    pub cost_sharing_billed: String,
    pub total_amount: String,
    pub amount_received: String,
    pub status: String,
    pub expenditure_ids: serde_json::Value,
    pub notes: Option<String>,
    pub submitted_by: Option<Uuid>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub paid_at: Option<DateTime<Utc>>,
    pub payment_reference: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Grant Compliance Report
/// Oracle Fusion: Grants Management > Compliance Reports
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GrantComplianceReport {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub award_id: Uuid,
    pub report_type: String,
    pub report_title: Option<String>,
    pub reporting_period_start: chrono::NaiveDate,
    pub reporting_period_end: chrono::NaiveDate,
    pub due_date: Option<chrono::NaiveDate>,
    pub status: String,
    pub total_expenditures: String,
    pub total_billed: String,
    pub total_received: String,
    pub cash_draws: String,
    pub obligations: String,
    pub content: serde_json::Value,
    pub notes: Option<String>,
    pub prepared_by: Option<Uuid>,
    pub reviewed_by: Option<Uuid>,
    pub approved_by: Option<Uuid>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// Deferred Revenue/Cost Management (Oracle Fusion Revenue/Cost Deferral)
// ============================================================================

/// Deferral template defines rules for deferring revenue or costs over time.
/// Oracle Fusion equivalent: Financials > Revenue Management > Deferral Templates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeferralTemplate {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    /// "revenue" or "cost"
    pub deferral_type: String,
    /// Recognition method: "`straight_line`", "`daily_rate`", "`front_loaded`", "`back_loaded`", "`fixed_schedule`"
    pub recognition_method: String,
    /// Deferral account (balance sheet: deferred revenue or prepaid expense)
    pub deferral_account_code: String,
    /// Recognition account (income statement: revenue or expense)
    pub recognition_account_code: String,
    /// Contra account for deferral (optional)
    pub contra_account_code: Option<String>,
    /// Default number of periods for deferral
    pub default_periods: i32,
    /// Period type: "monthly", "daily", "quarterly", "yearly"
    pub period_type: String,
    /// Start date basis: "`transaction_date`", "`period_start`", "custom"
    pub start_date_basis: String,
    /// End date basis: "`fixed_periods`", "`end_of_period`", "custom"
    pub end_date_basis: String,
    /// Whether to prorate partial periods
    pub prorate_partial_periods: bool,
    /// Whether to automatically generate recognition schedules
    pub auto_generate_schedule: bool,
    /// Whether to post recognition entries automatically
    pub auto_post: bool,
    /// Template-level rounding threshold
    pub rounding_threshold: Option<String>,
    /// Template-level currency code
    pub currency_code: String,
    pub is_active: bool,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

/// A specific deferral schedule created from a source transaction.
/// Oracle Fusion equivalent: Revenue/Cost Deferral Schedules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeferralSchedule {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub schedule_number: String,
    pub template_id: Uuid,
    pub template_code: Option<String>,
    /// "revenue" or "cost"
    pub deferral_type: String,
    /// Source reference: e.g. "invoice", "po", "expense", "manual"
    pub source_type: String,
    pub source_id: Option<Uuid>,
    pub source_number: Option<String>,
    pub source_line_id: Option<Uuid>,
    pub description: Option<String>,
    /// Total deferred amount
    pub total_amount: String,
    /// Amount recognized to date
    pub recognized_amount: String,
    /// Remaining deferred amount
    pub remaining_amount: String,
    pub currency_code: String,
    pub deferral_account_code: String,
    pub recognition_account_code: String,
    pub contra_account_code: Option<String>,
    /// Recognition method used
    pub recognition_method: String,
    pub start_date: chrono::NaiveDate,
    pub end_date: chrono::NaiveDate,
    pub total_periods: i32,
    pub completed_periods: i32,
    /// "draft", "active", "`on_hold`", "completed", "cancelled"
    pub status: String,
    pub hold_reason: Option<String>,
    pub original_journal_entry_id: Option<Uuid>,
    pub last_recognition_date: Option<chrono::NaiveDate>,
    pub completion_date: Option<chrono::NaiveDate>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

/// A single period line in a deferral schedule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeferralScheduleLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub schedule_id: Uuid,
    pub line_number: i32,
    pub period_name: Option<String>,
    pub period_start_date: chrono::NaiveDate,
    pub period_end_date: chrono::NaiveDate,
    /// Days in this period
    pub days_in_period: i32,
    /// Amount to recognize in this period
    pub amount: String,
    /// Recognized amount (may differ if partially recognized)
    pub recognized_amount: String,
    /// "pending", "recognized", "reversed", "`on_hold`"
    pub status: String,
    pub recognition_date: Option<chrono::NaiveDate>,
    pub journal_entry_id: Option<Uuid>,
    pub reversal_journal_entry_id: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

/// Dashboard summary for deferred revenue/cost management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeferralDashboardSummary {
    pub total_schedules: i32,
    pub active_schedules: i32,
    pub completed_schedules: i32,
    pub on_hold_schedules: i32,
    pub total_deferred_amount: String,
    pub total_recognized_amount: String,
    pub total_remaining_amount: String,
    pub pending_recognition_count: i32,
    pub pending_recognition_amount: String,
    pub revenue_deferred: String,
    pub cost_deferred: String,
}

// ============================================================================
// New Module Types: Standalone Selling Prices, Revenue Recognition Events
// ============================================================================

/// Standalone selling price record
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StandaloneSellingPrice {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub item_code: String,
    pub item_name: String,
    pub estimation_method: String,
    pub price: String,
    pub currency_code: String,
    pub effective_from: chrono::NaiveDate,
    pub effective_to: Option<chrono::NaiveDate>,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Revenue recognition event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RevenueRecognitionEvent {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub contract_id: Uuid,
    pub obligation_id: Uuid,
    pub event_number: String,
    pub description: String,
    pub event_type: String,
    pub amount: String,
    pub recognition_date: chrono::NaiveDate,
    pub gl_account_code: Option<String>,
    pub is_posted: bool,
    pub posted_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Revenue Management dashboard summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RevenueManagementDashboard {
    pub total_contracts: i32,
    pub active_contracts: i32,
    pub total_performance_obligations: i32,
    pub satisfied_obligations: i32,
    pub total_transaction_price: String,
    pub total_allocated: String,
    pub total_recognized: String,
    pub total_unrecognized: String,
    pub unrecognized_by_period: serde_json::Value,
}

/// Cash flow forecast definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CashFlowForecastDef {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub forecast_number: String,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub forecast_horizon: String,
    pub periods_out: i32,
    pub start_date: chrono::NaiveDate,
    pub end_date: chrono::NaiveDate,
    pub base_currency_code: String,
    pub total_inflows: String,
    pub total_outflows: String,
    pub net_cash_flow: String,
    pub opening_balance: String,
    pub closing_balance: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub approved_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Cash flow scenario for what-if analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CashFlowForecastScenario {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub forecast_id: Uuid,
    pub scenario_number: String,
    pub name: String,
    pub description: Option<String>,
    pub scenario_type: String,
    pub adjustment_factor: String,
    pub total_inflows: String,
    pub total_outflows: String,
    pub net_cash_flow: String,
    pub opening_balance: String,
    pub closing_balance: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Cash flow forecast entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CashFlowForecastEntry {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub forecast_id: Uuid,
    pub scenario_id: Option<Uuid>,
    pub period_name: String,
    pub period_start_date: chrono::NaiveDate,
    pub period_end_date: chrono::NaiveDate,
    pub source_category: String,
    pub flow_direction: String,
    pub amount: String,
    pub probability: String,
    pub weighted_amount: String,
    pub is_manual: bool,
    pub description: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Cash position snapshot for forecasting
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CashPositionSnapshot {
    pub organization_id: Uuid,
    pub as_of_date: chrono::NaiveDate,
    pub currency_code: String,
    pub bank_balance: String,
    pub book_balance: String,
    pub net_cash_position: String,
}

/// Cash flow forecast dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CashFlowForecastDashboard {
    pub total_forecasts: i32,
    pub active_forecasts: i32,
    pub current_cash_position: Option<CashPositionSnapshot>,
    pub total_projected_inflows: String,
    pub total_projected_outflows: String,
    pub net_projected_cash_flow: String,
    pub surplus_deficit: String,
    pub scenario_comparison: serde_json::Value,
}

/// Regulatory report template
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegulatoryReportTemplate {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub authority: String,
    pub report_category: String,
    pub filing_frequency: String,
    pub output_format: String,
    pub row_definitions: serde_json::Value,
    pub column_definitions: serde_json::Value,
    pub validation_rules: serde_json::Value,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Regulatory report instance
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegulatoryReportInstance {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub template_id: Uuid,
    pub template_code: Option<String>,
    pub report_number: String,
    pub name: String,
    pub status: String,
    pub period_start: chrono::NaiveDate,
    pub period_end: chrono::NaiveDate,
    pub authority: String,
    pub output_format: String,
    pub total_debits: String,
    pub total_credits: String,
    pub line_count: i32,
    pub generated_at: Option<DateTime<Utc>>,
    pub reviewed_by: Option<Uuid>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub submitted_by: Option<Uuid>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub filing_reference: Option<String>,
    pub rejection_reason: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Regulatory report line
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegulatoryReportLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub report_id: Uuid,
    pub line_number: i32,
    pub row_code: String,
    pub row_label: String,
    pub column_code: String,
    pub column_label: String,
    pub amount: String,
    pub description: Option<String>,
    pub account_range: Option<String>,
    pub is_subtotal: bool,
    pub is_total: bool,
    pub indent_level: i32,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Filing calendar entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilingCalendarEntry {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub template_id: Option<Uuid>,
    pub template_code: Option<String>,
    pub authority: String,
    pub report_name: String,
    pub filing_frequency: String,
    pub period_start: chrono::NaiveDate,
    pub period_end: chrono::NaiveDate,
    pub due_date: chrono::NaiveDate,
    pub status: String,
    pub assigned_to: Option<Uuid>,
    pub report_id: Option<Uuid>,
    pub filed_at: Option<DateTime<Utc>>,
    pub filed_by: Option<Uuid>,
    pub filing_reference: Option<String>,
    pub notes: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Regulatory Reporting dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegulatoryReportingDashboard {
    pub total_templates: i32,
    pub active_templates: i32,
    pub total_reports: i32,
    pub draft_reports: i32,
    pub pending_review: i32,
    pub pending_submission: i32,
    pub submitted_reports: i32,
    pub overdue_filings: i32,
    pub upcoming_filings: i32,
    pub filings_by_authority: serde_json::Value,
}
