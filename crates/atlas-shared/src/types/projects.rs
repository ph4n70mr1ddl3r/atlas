use serde::{Deserialize, Serialize};
use uuid::Uuid;
// ============================================================================
// Project Billing Types
// Oracle Fusion Cloud: Project Management > Project Billing
// ============================================================================

/// Bill rate schedule - defines billable rates for a set of roles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillRateSchedule {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub schedule_number: String,
    pub name: String,
    pub description: String,
    pub schedule_type: String, // standard, overtime, holiday, custom
    pub currency_code: String,
    pub effective_start: chrono::NaiveDate,
    pub effective_end: Option<chrono::NaiveDate>,
    pub status: String, // draft, active, inactive
    pub default_markup_pct: f64,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Individual bill rate line within a schedule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillRateLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub schedule_id: Uuid,
    pub role_name: String,
    pub project_id: Option<Uuid>,
    pub bill_rate: f64,
    pub unit_of_measure: String,
    pub effective_start: chrono::NaiveDate,
    pub effective_end: Option<chrono::NaiveDate>,
    pub markup_pct: Option<f64>,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Billing configuration per project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectBillingConfig {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub project_id: Uuid,
    pub billing_method: String, // time_and_materials, fixed_price, milestone, cost_plus, retention
    pub bill_rate_schedule_id: Option<Uuid>,
    pub contract_amount: f64,
    pub currency_code: String,
    pub invoice_format: String, // detailed, summary, consolidated
    pub billing_cycle: String,  // weekly, biweekly, monthly, milestone
    pub payment_terms_days: i32,
    pub retention_pct: f64,
    pub retention_amount_cap: f64,
    pub customer_id: Option<Uuid>,
    pub customer_name: String,
    pub customer_po_number: String,
    pub contract_number: String,
    pub status: String, // draft, active, completed, cancelled
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Billing event (milestone, progress marker)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingEvent {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub project_id: Uuid,
    pub event_number: String,
    pub event_name: String,
    pub description: String,
    pub event_type: String, // milestone, progress, completion, retention_release
    pub billing_amount: f64,
    pub currency_code: String,
    pub completion_pct: f64,
    pub status: String, // planned, ready, invoiced, partially_invoiced, cancelled
    pub planned_date: Option<chrono::NaiveDate>,
    pub actual_date: Option<chrono::NaiveDate>,
    pub task_id: Option<Uuid>,
    pub task_name: String,
    pub invoice_header_id: Option<Uuid>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<chrono::DateTime<chrono::Utc>>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Project invoice header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInvoiceHeader {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub invoice_number: String,
    pub project_id: Uuid,
    pub project_number: String,
    pub project_name: String,
    pub invoice_type: String, // progress, milestone, t_and_m, retention_release, debit_memo, credit_memo
    pub status: String,       // draft, submitted, approved, rejected, posted, cancelled
    pub customer_id: Option<Uuid>,
    pub customer_name: String,
    pub invoice_amount: f64,
    pub tax_amount: f64,
    pub retention_held: f64,
    pub total_amount: f64,
    pub currency_code: String,
    pub exchange_rate: f64,
    pub billing_period_start: Option<chrono::NaiveDate>,
    pub billing_period_end: Option<chrono::NaiveDate>,
    pub invoice_date: chrono::NaiveDate,
    pub due_date: Option<chrono::NaiveDate>,
    pub billing_event_id: Option<Uuid>,
    pub customer_po_number: String,
    pub contract_number: String,
    pub gl_posted_flag: bool,
    pub gl_posted_date: Option<chrono::DateTime<chrono::Utc>>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<chrono::DateTime<chrono::Utc>>,
    pub rejected_reason: String,
    pub payment_status: String, // unpaid, partially_paid, paid
    pub payment_date: Option<chrono::DateTime<chrono::Utc>>,
    pub notes: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Project invoice line
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInvoiceLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub invoice_header_id: Uuid,
    pub line_number: i32,
    pub line_source: String, // expenditure_item, billing_event, retention, manual
    pub expenditure_item_id: Option<Uuid>,
    pub billing_event_id: Option<Uuid>,
    pub task_id: Option<Uuid>,
    pub task_number: String,
    pub task_name: String,
    pub description: String,
    pub employee_id: Option<Uuid>,
    pub employee_name: String,
    pub role_name: String,
    pub expenditure_type: String,
    pub quantity: f64,
    pub unit_of_measure: String,
    pub bill_rate: f64,
    pub raw_cost_amount: f64,
    pub bill_amount: f64,
    pub markup_amount: f64,
    pub retention_amount: f64,
    pub tax_amount: f64,
    pub transaction_date: Option<chrono::NaiveDate>,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Project billing dashboard summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectBillingDashboard {
    pub total_projects_billable: i32,
    pub total_contract_value: f64,
    pub total_billed: f64,
    pub total_unbilled: f64,
    pub total_retention_held: f64,
    pub total_retention_released: f64,
    pub total_invoices: i32,
    pub draft_invoices: i32,
    pub submitted_invoices: i32,
    pub approved_invoices: i32,
    pub posted_invoices: i32,
    pub overdue_invoices: i32,
    pub total_revenue_recognized: f64,
    pub by_billing_method: serde_json::Value,
    pub by_invoice_status: serde_json::Value,
    pub billing_trend: serde_json::Value,
}

// ============================================================================
// Project Resource Management (Oracle Fusion Cloud: Project Management)
// ============================================================================
// Manages resource profiles, resource requests, assignments, utilization
// tracking, and resource analytics for project staffing.
//
// Key concepts:
// - Resource Profile: employee/contractor with skills, availability, cost rates
// - Resource Request: project manager request for a resource with skill requirements
// - Resource Assignment: resource assigned to a project with planned hours/dates
// - Utilization Entry: actual hours worked on an assignment for tracking
// - Resource Dashboard: analytics on utilization, open requests, assignments
// ============================================================================

/// Resource Profile
/// Represents an employee or contractor available for project assignments.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceProfile {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub resource_number: String,
    pub name: String,
    pub email: String,
    pub resource_type: String, // employee, contractor
    pub department: String,
    pub job_title: String,
    pub skills: String,              // comma-separated skill tags
    pub certifications: String,      // comma-separated
    pub availability_status: String, // available, partially_available, fully_allocated, on_leave
    pub available_hours_per_week: f64,
    pub cost_rate: f64,
    pub cost_rate_currency: String,
    pub bill_rate: f64,
    pub bill_rate_currency: String,
    pub location: String,
    pub manager_id: Option<Uuid>,
    pub manager_name: String,
    pub hire_date: Option<chrono::NaiveDate>,
    pub notes: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Resource Request
/// A project manager's request for a resource with specific skill requirements.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceRequest {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub request_number: String,
    pub project_id: Option<Uuid>,
    pub project_name: String,
    pub project_number: String,
    pub requested_role: String,
    pub required_skills: String,
    pub priority: String, // low, medium, high, critical
    pub status: String,   // draft, submitted, fulfilled, partially_fulfilled, cancelled
    pub start_date: chrono::NaiveDate,
    pub end_date: chrono::NaiveDate,
    pub hours_per_week: f64,
    pub total_planned_hours: f64,
    pub max_cost_rate: Option<f64>,
    pub currency_code: String,
    pub resource_type_preference: String, // any, employee_only, contractor_only
    pub location_requirement: String,
    pub fulfilled_by: Option<Uuid>,
    pub fulfilled_at: Option<chrono::DateTime<chrono::Utc>>,
    pub notes: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Resource Assignment
/// A resource assigned to a project with planned hours and date range.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceAssignment {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub assignment_number: String,
    pub resource_id: Uuid,
    pub resource_name: String,
    pub resource_email: String,
    pub project_id: Option<Uuid>,
    pub project_name: String,
    pub project_number: String,
    pub request_id: Option<Uuid>,
    pub role: String,
    pub status: String, // planned, active, completed, cancelled
    pub start_date: chrono::NaiveDate,
    pub end_date: chrono::NaiveDate,
    pub planned_hours: f64,
    pub actual_hours: f64,
    pub remaining_hours: f64,
    pub utilization_percentage: f64,
    pub cost_rate: f64,
    pub bill_rate: f64,
    pub currency_code: String,
    pub notes: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Utilization Entry
/// Tracks actual hours worked by a resource on an assignment for a given period.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UtilizationEntry {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub assignment_id: Uuid,
    pub resource_id: Uuid,
    pub entry_date: chrono::NaiveDate,
    pub hours_worked: f64,
    pub description: String,
    pub billable: bool,
    pub status: String, // submitted, approved, rejected
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<chrono::DateTime<chrono::Utc>>,
    pub notes: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Resource Management Dashboard
/// Analytics on resource utilization, open requests, and assignment metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceDashboard {
    pub organization_id: Uuid,
    pub total_resources: i64,
    pub available_resources: i64,
    pub fully_allocated_resources: i64,
    pub open_requests: i64,
    pub active_assignments: i64,
    pub average_utilization: f64,
    pub total_planned_hours: f64,
    pub total_actual_hours: f64,
    pub resources_by_type: serde_json::Value,
    pub resources_by_department: serde_json::Value,
    pub top_resources_by_utilization: serde_json::Value,
    pub recent_assignments: serde_json::Value,
}
