use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
// ============================================================================
// Benefits Administration (Oracle Fusion HCM > Benefits)
// ============================================================================

/// Benefits plan definition
/// Oracle Fusion: Benefits > Benefits Plans
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BenefitsPlan {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    /// Plan type: medical, dental, vision, `life_insurance`, disability, retirement, hsa, fsa
    pub plan_type: String,
    /// Coverage tier options available for this plan
    pub coverage_tiers: serde_json::Value,
    /// Provider/insurance carrier name
    pub provider_name: Option<String>,
    /// External plan ID from carrier
    pub provider_plan_id: Option<String>,
    /// Plan year start date
    pub plan_year_start: Option<chrono::NaiveDate>,
    /// Plan year end date
    pub plan_year_end: Option<chrono::NaiveDate>,
    /// Open enrollment start date
    pub open_enrollment_start: Option<chrono::NaiveDate>,
    /// Open enrollment end date
    pub open_enrollment_end: Option<chrono::NaiveDate>,
    /// Whether employees can make mid-year changes (qualifying life events)
    pub allow_life_event_changes: bool,
    /// Whether the plan requires evidence of insurability
    pub requires_eoi: bool,
    /// Waiting period in days before new hires can enroll
    pub waiting_period_days: i32,
    /// Maximum number of dependents allowed
    pub max_dependents: Option<i32>,
    /// Whether the plan is currently active and available for enrollment
    pub is_active: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Benefits plan coverage tier (e.g., Employee Only, Employee + Spouse, Family)
/// Oracle Fusion: Benefits > Coverage Options
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoverageTier {
    /// Tier code: `employee_only`, `employee_spouse`, `employee_child`, family
    pub tier_code: String,
    /// Human-readable tier name
    pub tier_name: String,
    /// Employee's contribution per pay period (employer-paid portion excluded)
    pub employee_cost: String,
    /// Employer's contribution per pay period
    pub employer_cost: String,
    /// Total cost per pay period
    pub total_cost: String,
}

/// Employee benefits enrollment
/// Oracle Fusion: Benefits > Enrollments
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BenefitsEnrollment {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub employee_id: Uuid,
    pub employee_name: Option<String>,
    pub plan_id: Uuid,
    pub plan_code: Option<String>,
    pub plan_name: Option<String>,
    pub plan_type: Option<String>,
    /// Selected coverage tier code
    pub coverage_tier: String,
    /// Enrollment type: `open_enrollment`, `new_hire`, `life_event`, manual
    pub enrollment_type: String,
    /// Enrollment status: pending, active, waived, cancelled, suspended
    pub status: String,
    /// Effective start date of coverage
    pub effective_start_date: chrono::NaiveDate,
    /// Effective end date of coverage
    pub effective_end_date: Option<chrono::NaiveDate>,
    /// Employee cost per pay period (deduction amount)
    pub employee_cost: String,
    /// Employer cost per pay period
    pub employer_cost: String,
    /// Total cost per pay period
    pub total_cost: String,
    /// Payroll deduction frequency: `per_pay_period`, monthly, `semi_monthly`
    pub deduction_frequency: String,
    /// GL account code for employee deduction
    pub deduction_account_code: Option<String>,
    /// GL account code for employer contribution
    pub employer_contribution_account_code: Option<String>,
    /// Enrolled dependents
    pub dependents: serde_json::Value,
    /// Qualifying life event reason (if enrollment due to life event)
    pub life_event_reason: Option<String>,
    /// Life event date
    pub life_event_date: Option<chrono::NaiveDate>,
    /// Who processed this enrollment
    pub processed_by: Option<Uuid>,
    /// When the enrollment was processed
    pub processed_at: Option<DateTime<Utc>>,
    /// Cancellation reason
    pub cancellation_reason: Option<String>,
    /// When the enrollment was cancelled
    pub cancelled_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Benefits deduction record (tracks each payroll deduction)
/// Oracle Fusion: Payroll > Element Entries > Benefits Deductions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BenefitsDeduction {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub enrollment_id: Uuid,
    pub employee_id: Uuid,
    pub plan_id: Uuid,
    pub plan_code: Option<String>,
    pub plan_name: Option<String>,
    /// Deduction amount (employee portion)
    pub employee_amount: String,
    /// Employer contribution amount
    pub employer_amount: String,
    /// Total deduction amount
    pub total_amount: String,
    /// Pay period this deduction applies to
    pub pay_period_start: chrono::NaiveDate,
    pub pay_period_end: chrono::NaiveDate,
    /// GL account code for the deduction
    pub deduction_account_code: Option<String>,
    /// Whether the deduction has been processed through payroll
    pub is_processed: bool,
    pub processed_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Benefits enrollment summary (dashboard view)
/// Oracle Fusion: Benefits > Benefits Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BenefitsSummary {
    pub total_plans: i32,
    pub active_plans: i32,
    pub total_enrollments: i32,
    pub active_enrollments: i32,
    pub pending_enrollments: i32,
    pub waived_enrollments: i32,
    pub total_employee_cost: String,
    pub total_employer_cost: String,
    pub enrollments_by_plan_type: serde_json::Value,
}

// ============================================================================
// Performance Management (Oracle Fusion HCM Performance Review)
// ============================================================================

/// Performance rating model (defines the rating scale).
/// Oracle Fusion: My Client Groups > Performance > Rating Models
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceRatingModel {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    /// Rating scale entries, e.g. [{"value":1,"label":"Below Expectations"}, ...]
    pub rating_scale: serde_json::Value,
    pub is_active: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Performance review cycle (defines a review period).
/// Oracle Fusion: My Client Groups > Performance > Review Cycles
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceReviewCycle {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    /// Cycle type: annual, `mid_year`, quarterly, `project_end`, probation
    pub cycle_type: String,
    /// Status: draft, planning, `goal_setting`, `self_evaluation`, `manager_evaluation`, calibration, completed, cancelled
    pub status: String,
    pub rating_model_id: Option<Uuid>,
    pub start_date: chrono::NaiveDate,
    pub end_date: chrono::NaiveDate,
    pub goal_setting_start: Option<chrono::NaiveDate>,
    pub goal_setting_end: Option<chrono::NaiveDate>,
    pub self_evaluation_start: Option<chrono::NaiveDate>,
    pub self_evaluation_end: Option<chrono::NaiveDate>,
    pub manager_evaluation_start: Option<chrono::NaiveDate>,
    pub manager_evaluation_end: Option<chrono::NaiveDate>,
    pub calibration_date: Option<chrono::NaiveDate>,
    pub require_goals: bool,
    pub require_competencies: bool,
    pub min_goals: i32,
    pub max_goals: i32,
    pub goal_weight_total: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Competency definition.
/// Oracle Fusion: My Client Groups > Performance > Competencies
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceCompetency {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    /// Category: core, leadership, technical, functional
    pub category: Option<String>,
    pub rating_model_id: Option<Uuid>,
    pub behavioral_indicators: serde_json::Value,
    pub is_active: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Performance document (one per employee per review cycle).
/// Oracle Fusion: My Client Groups > Performance > Performance Documents
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceDocument {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub review_cycle_id: Uuid,
    pub employee_id: Uuid,
    pub employee_name: Option<String>,
    pub manager_id: Option<Uuid>,
    pub manager_name: Option<String>,
    pub document_number: String,
    /// Status: `not_started`, `goal_setting`, `self_evaluation`, `manager_evaluation`, calibration, completed, cancelled
    pub status: String,
    pub overall_rating: Option<String>,
    pub overall_rating_label: Option<String>,
    pub self_overall_rating: Option<String>,
    pub self_comments: Option<String>,
    pub manager_overall_rating: Option<String>,
    pub manager_comments: Option<String>,
    pub calibration_rating: Option<String>,
    pub calibration_comments: Option<String>,
    pub final_rating: Option<String>,
    pub final_comments: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Performance goal (linked to a performance document).
/// Oracle Fusion: My Client Groups > Performance > Goals
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceGoal {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub document_id: Uuid,
    pub employee_id: Uuid,
    pub goal_name: String,
    pub description: Option<String>,
    /// Category: performance, development, project, behavioral
    pub goal_category: Option<String>,
    /// Status: draft, active, completed, cancelled
    pub status: String,
    pub weight: String,
    pub target_metric: Option<String>,
    pub actual_result: Option<String>,
    pub self_rating: Option<String>,
    pub self_comments: Option<String>,
    pub manager_rating: Option<String>,
    pub manager_comments: Option<String>,
    pub start_date: Option<chrono::NaiveDate>,
    pub due_date: Option<chrono::NaiveDate>,
    pub completed_date: Option<chrono::NaiveDate>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Competency assessment (linked to a performance document).
/// Oracle Fusion: My Client Groups > Performance > Competency Assessments
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompetencyAssessment {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub document_id: Uuid,
    pub employee_id: Uuid,
    pub competency_id: Uuid,
    pub self_rating: Option<String>,
    pub self_comments: Option<String>,
    pub manager_rating: Option<String>,
    pub manager_comments: Option<String>,
    pub calibration_rating: Option<String>,
    pub calibration_comments: Option<String>,
    pub final_rating: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Performance feedback (360-degree feedback, ad-hoc).
/// Oracle Fusion: My Client Groups > Performance > Feedback
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceFeedback {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub document_id: Option<Uuid>,
    pub employee_id: Uuid,
    pub from_user_id: Uuid,
    pub from_user_name: Option<String>,
    /// Feedback type: peer, manager, `direct_report`, external, self
    pub feedback_type: String,
    pub subject: Option<String>,
    pub content: String,
    /// Visibility: private, `manager_only`, `manager_and_employee`, everyone
    pub visibility: String,
    /// Status: draft, submitted, acknowledged, withdrawn
    pub status: String,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Performance dashboard summary for a review cycle.
/// Oracle Fusion: My Client Groups > Performance > Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceDashboard {
    pub review_cycle_id: Uuid,
    pub total_documents: i32,
    pub not_started_count: i32,
    pub goal_setting_count: i32,
    pub self_evaluation_count: i32,
    pub manager_evaluation_count: i32,
    pub calibration_count: i32,
    pub completed_count: i32,
    pub cancelled_count: i32,
    pub average_rating: Option<String>,
    pub goals_total: i32,
    pub goals_completed: i32,
    pub feedback_count: i32,
}

// ============================================================================
// Time and Labor Management (Oracle Fusion Cloud HCM Time and Labor)
// ============================================================================

/// Work schedule definition
/// Oracle Fusion: HCM > Time and Labor > Work Schedules
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkSchedule {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub schedule_type: String,
    pub standard_hours_per_day: String,
    pub standard_hours_per_week: String,
    pub work_days_per_week: i32,
    pub start_time: Option<chrono::NaiveTime>,
    pub end_time: Option<chrono::NaiveTime>,
    pub break_duration_minutes: i32,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Overtime rule definition
/// Oracle Fusion: HCM > Time and Labor > Overtime Rules
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OvertimeRule {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub threshold_type: String,
    pub daily_threshold_hours: String,
    pub weekly_threshold_hours: String,
    pub overtime_multiplier: String,
    pub double_time_threshold_hours: Option<String>,
    pub double_time_multiplier: String,
    pub include_holidays: bool,
    pub include_weekends: bool,
    pub is_active: bool,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Time card (one per employee per period)
/// Oracle Fusion: HCM > Time and Labor > Time Cards
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeCard {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub employee_id: Uuid,
    pub employee_name: Option<String>,
    pub card_number: String,
    pub status: String,
    pub period_start: chrono::NaiveDate,
    pub period_end: chrono::NaiveDate,
    pub total_regular_hours: String,
    pub total_overtime_hours: String,
    pub total_double_time_hours: String,
    pub total_hours: String,
    pub schedule_id: Option<Uuid>,
    pub overtime_rule_id: Option<Uuid>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub rejected_reason: Option<String>,
    pub comments: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Time entry (individual time punch within a time card)
/// Oracle Fusion: HCM > Time and Labor > Time Entries
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeEntry {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub time_card_id: Uuid,
    pub entry_date: chrono::NaiveDate,
    pub entry_type: String,
    pub start_time: Option<chrono::NaiveTime>,
    pub end_time: Option<chrono::NaiveTime>,
    pub duration_hours: String,
    pub project_id: Option<Uuid>,
    pub project_name: Option<String>,
    pub department_id: Option<Uuid>,
    pub department_name: Option<String>,
    pub task_name: Option<String>,
    pub location: Option<String>,
    pub cost_center: Option<String>,
    pub labor_category: Option<String>,
    pub comments: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Time card history entry (audit trail)
/// Oracle Fusion: HCM > Time and Labor > Time Card History
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeCardHistory {
    pub id: Uuid,
    pub time_card_id: Uuid,
    pub action: String,
    pub from_status: Option<String>,
    pub to_status: Option<String>,
    pub performed_by: Option<Uuid>,
    pub comment: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Labor distribution (cost allocation for time entries)
/// Oracle Fusion: HCM > Time and Labor > Labor Distribution
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaborDistribution {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub time_entry_id: Uuid,
    pub distribution_percent: String,
    pub cost_center: Option<String>,
    pub project_id: Option<Uuid>,
    pub project_name: Option<String>,
    pub department_id: Option<Uuid>,
    pub department_name: Option<String>,
    pub gl_account_code: Option<String>,
    pub allocated_hours: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Time and Labor dashboard summary
/// Oracle Fusion: HCM > Time and Labor > Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeAndLaborDashboard {
    pub total_schedules: i64,
    pub active_schedules: i64,
    pub total_overtime_rules: i64,
    pub total_time_cards: i64,
    pub pending_approval_count: i64,
    pub submitted_today_count: i64,
    pub cards_by_status: serde_json::Value,
    pub hours_by_type: serde_json::Value,
    pub recent_time_cards: Vec<TimeCard>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Approval Authority Limits
// Oracle Fusion: BPM > Approval Configuration > Document Approval Limits
// ═══════════════════════════════════════════════════════════════════════════════

/// Approval authority limit - defines the maximum monetary amount a user
/// or role is authorised to approve for a given document type.
///
/// Oracle Fusion Cloud calls these "Document Approval Limits" or
/// "Signing Limits". They restrict who can approve what, up to how much,
/// for which business unit / cost center.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalAuthorityLimit {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub limit_code: String,
    pub name: String,
    pub description: Option<String>,
    /// "user" or "role"
    pub owner_type: String,
    /// User ID when `owner_type` = "user"
    pub user_id: Option<Uuid>,
    /// Role name when `owner_type` = "role"
    pub role_name: Option<String>,
    /// Document type this limit applies to
    pub document_type: String,
    /// Maximum amount the owner can approve in a single transaction
    pub approval_limit_amount: String,
    /// Currency code (e.g. "USD")
    pub currency_code: String,
    /// Optional business unit scope
    pub business_unit_id: Option<Uuid>,
    /// Optional cost center scope
    pub cost_center: Option<String>,
    /// "active" or "inactive"
    pub status: String,
    /// Effective date range
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Audit trail entry for authority-limit checks
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorityCheckAudit {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub limit_id: Option<Uuid>,
    /// The user attempting the approval
    pub checked_user_id: Uuid,
    /// The role being checked (if role-based)
    pub checked_role: Option<String>,
    /// Document type
    pub document_type: String,
    /// The document being approved
    pub document_id: Option<Uuid>,
    /// The amount being approved
    pub requested_amount: String,
    /// The limit that was found (or 0 if none)
    pub applicable_limit: String,
    /// "approved", "denied"
    pub result: String,
    /// Reason for the result
    pub reason: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// Request to create an approval authority limit
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateApprovalAuthorityLimitRequest {
    pub limit_code: String,
    pub name: String,
    pub description: Option<String>,
    /// "user" or "role"
    pub owner_type: String,
    pub user_id: Option<String>,
    pub role_name: Option<String>,
    pub document_type: String,
    pub approval_limit_amount: String,
    pub currency_code: String,
    pub business_unit_id: Option<String>,
    pub cost_center: Option<String>,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
}

/// Dashboard summary for approval authority limits
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalAuthorityDashboard {
    pub total_limits: i64,
    pub active_limits: i64,
    pub limits_by_document_type: serde_json::Value,
    pub limits_by_owner_type: serde_json::Value,
    pub recent_checks: Vec<AuthorityCheckAudit>,
    pub total_checks: i64,
    pub approved_checks: i64,
    pub denied_checks: i64,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Data Archiving and Retention Management
// Oracle Fusion: Information Lifecycle Management (ILM)
// ═══════════════════════════════════════════════════════════════════════════════

/// Retention policy - defines how long data of a given entity type
/// must be retained before it can be archived or purged.
///
/// Oracle Fusion Cloud: Tools > Information Lifecycle Management > Retention Policies
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetentionPolicy {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub policy_code: String,
    pub name: String,
    pub description: Option<String>,
    pub entity_type: String,
    pub retention_days: i32,
    /// "archive", "purge", "`archive_then_purge`"
    pub action_type: String,
    pub purge_after_days: Option<i32>,
    pub condition_expression: Option<String>,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request to create/update a retention policy
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRetentionPolicyRequest {
    pub policy_code: String,
    pub name: String,
    pub description: Option<String>,
    pub entity_type: String,
    #[serde(default = "default_365")]
    pub retention_days: i32,
    #[serde(default = "default_action_type")]
    pub action_type: String,
    pub purge_after_days: Option<i32>,
    pub condition_expression: Option<String>,
}

pub const fn default_365() -> i32 {
    365
}
pub fn default_action_type() -> String {
    "archive_then_purge".to_string()
}

/// Legal hold - prevents archival or purging of specific records.
///
/// Oracle Fusion Cloud: Information Lifecycle Management > Legal Holds
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LegalHold {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub hold_number: String,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub reason: Option<String>,
    pub case_reference: Option<String>,
    pub authorized_by: Option<Uuid>,
    pub released_at: Option<DateTime<Utc>>,
    pub released_by: Option<Uuid>,
    pub release_reason: Option<String>,
    pub effective_from: chrono::NaiveDate,
    pub effective_to: Option<chrono::NaiveDate>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request to create a legal hold
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateLegalHoldRequest {
    pub hold_number: String,
    pub name: String,
    pub description: Option<String>,
    pub reason: Option<String>,
    pub case_reference: Option<String>,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
}

/// Legal hold item - a specific record under a legal hold
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LegalHoldItem {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub legal_hold_id: Uuid,
    pub entity_type: String,
    pub record_id: Uuid,
    pub created_at: DateTime<Utc>,
}

/// Archived record - tracks a record that has been archived
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchivedRecord {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub entity_type: String,
    pub original_record_id: Uuid,
    pub original_data: serde_json::Value,
    pub retention_policy_id: Option<Uuid>,
    pub archive_batch_id: Option<Uuid>,
    pub status: String,
    pub original_created_at: Option<DateTime<Utc>>,
    pub original_updated_at: Option<DateTime<Utc>>,
    pub archived_at: DateTime<Utc>,
    pub archived_by: Option<Uuid>,
    pub restored_at: Option<DateTime<Utc>>,
    pub restored_by: Option<Uuid>,
    pub purged_at: Option<DateTime<Utc>>,
    pub purged_by: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// Archive batch - groups records archived together
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveBatch {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub batch_number: String,
    pub retention_policy_id: Option<Uuid>,
    pub entity_type: String,
    pub status: String,
    pub total_records: i32,
    pub archived_records: i32,
    pub failed_records: i32,
    pub criteria: serde_json::Value,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Archive audit entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveAudit {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub operation: String,
    pub entity_type: String,
    pub record_id: Option<Uuid>,
    pub batch_id: Option<Uuid>,
    pub legal_hold_id: Option<Uuid>,
    pub retention_policy_id: Option<Uuid>,
    pub result: String,
    pub details: Option<String>,
    pub performed_by: Option<Uuid>,
    pub performed_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

/// Data archiving dashboard summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataArchivingDashboard {
    pub total_policies: i64,
    pub active_policies: i64,
    pub total_legal_holds: i64,
    pub active_legal_holds: i64,
    pub total_archived_records: i64,
    pub total_purged_records: i64,
    pub total_restored_records: i64,
    pub policies_by_entity_type: serde_json::Value,
    pub recent_audit_entries: Vec<ArchiveAudit>,
}

// ============================================================================
// Payroll Management (Oracle Fusion Global Payroll)
// ============================================================================

/// Payroll definition - represents a pay group.
/// Oracle Fusion: Payroll > Payroll Definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PayrollDefinition {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    /// How often this payroll runs: "weekly", "biweekly", "semimonthly", "monthly"
    pub pay_frequency: String,
    /// Default currency code for payroll runs
    pub currency_code: String,
    /// GL account code for salary expense posting
    pub salary_expense_account: Option<String>,
    /// GL account code for employer liabilities
    pub liability_account: Option<String>,
    /// GL account code for employer tax expense
    pub employer_tax_account: Option<String>,
    /// GL account code for bank / cash disbursement
    pub payment_account: Option<String>,
    /// Whether this payroll definition is active
    pub is_active: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

/// Payroll element - an earning or deduction component.
/// Oracle Fusion: Payroll > Element Definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PayrollElement {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    /// "earning" or "deduction"
    pub element_type: String,
    /// "salary", "hourly", "overtime", "bonus", "commission", "benefit", "tax", "retirement", "garnishment", "other"
    pub category: String,
    /// How the value is determined: "flat", "percentage", "`hourly_rate`", "formula"
    pub calculation_method: String,
    /// Default rate / value depending on `calculation_method`
    pub default_value: Option<String>,
    /// Whether this element is recurring every pay period
    pub is_recurring: bool,
    /// Whether employer also contributes (e.g. employer match on 401k)
    pub has_employer_contribution: bool,
    /// Percentage for employer contribution (if applicable)
    pub employer_contribution_rate: Option<String>,
    /// GL account override for this element
    pub gl_account_code: Option<String>,
    /// Whether this element is pretax (deducted before tax calc)
    pub is_pretax: bool,
    pub is_active: bool,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

/// Employee element assignment - links an element to an employee with a value.
/// Oracle Fusion: Payroll > Element Entries
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PayrollElementEntry {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub employee_id: Uuid,
    pub element_id: Uuid,
    pub element_code: String,
    pub element_name: String,
    pub element_type: String,
    pub entry_value: String,
    /// How many periods this entry spans (None = indefinite)
    pub remaining_periods: Option<i32>,
    pub is_active: bool,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

/// Payroll run - a single execution of payroll for a period.
/// Oracle Fusion: Payroll > Payroll Runs / Quick Pay
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PayrollRun {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub payroll_id: Uuid,
    pub run_number: String,
    /// "open", "calculated", "confirmed", "paid", "reversed"
    pub status: String,
    /// Period being paid
    pub period_start: chrono::NaiveDate,
    pub period_end: chrono::NaiveDate,
    /// When payroll should be deposited
    pub pay_date: chrono::NaiveDate,
    pub total_gross: String,
    pub total_deductions: String,
    pub total_net: String,
    pub total_employer_cost: String,
    pub employee_count: i32,
    pub confirmed_by: Option<Uuid>,
    pub confirmed_at: Option<DateTime<Utc>>,
    pub paid_by: Option<Uuid>,
    pub paid_at: Option<DateTime<Utc>>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

/// Pay slip - per-employee payroll result within a run.
/// Oracle Fusion: Payroll > Pay Slips / Payment History
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaySlip {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub payroll_run_id: Uuid,
    pub employee_id: Uuid,
    pub employee_name: Option<String>,
    pub gross_earnings: String,
    pub total_deductions: String,
    pub net_pay: String,
    pub employer_cost: String,
    pub currency_code: String,
    pub payment_method: Option<String>,
    pub bank_account_last4: Option<String>,
    pub lines: Vec<PaySlipLine>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

/// Individual line on a pay slip (one earning or deduction).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaySlipLine {
    pub id: Uuid,
    pub pay_slip_id: Uuid,
    pub element_code: String,
    pub element_name: String,
    pub element_type: String,
    pub category: String,
    /// Number of hours (for hourly earnings), units, or quantity
    pub hours_or_units: Option<String>,
    /// Rate per unit / hour
    pub rate: Option<String>,
    /// Computed amount for this line
    pub amount: String,
    /// Whether this is pretax
    pub is_pretax: bool,
    /// Whether this is an employer-paid portion
    pub is_employer: bool,
    pub gl_account_code: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Payroll summary for dashboard.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PayrollDashboard {
    pub total_gross: String,
    pub total_deductions: String,
    pub total_net: String,
    pub total_employer_cost: String,
    pub employee_count: i32,
    pub payroll_runs_this_period: i32,
    pub recent_runs: Vec<PayrollRun>,
    pub top_earnings_by_category: serde_json::Value,
    pub top_deductions_by_category: serde_json::Value,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Compensation Management (Oracle Fusion Cloud HCM Compensation Workbench)
// ═══════════════════════════════════════════════════════════════════════════════

/// Compensation plan component type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompensationComponent {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub plan_id: Uuid,
    pub component_name: String,
    pub component_type: String,
    pub description: Option<String>,
    pub is_recurring: bool,
    pub frequency: Option<String>,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Compensation plan definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompensationPlan {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub plan_code: String,
    pub plan_name: String,
    pub description: Option<String>,
    pub plan_type: String,
    pub status: String,
    pub effective_start_date: Option<chrono::NaiveDate>,
    pub effective_end_date: Option<chrono::NaiveDate>,
    pub eligibility_criteria: serde_json::Value,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Compensation cycle (annual review cycle)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompensationCycle {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub cycle_name: String,
    pub description: Option<String>,
    pub cycle_type: String,
    pub status: String,
    pub start_date: chrono::NaiveDate,
    pub end_date: chrono::NaiveDate,
    pub allocation_start_date: Option<chrono::NaiveDate>,
    pub allocation_end_date: Option<chrono::NaiveDate>,
    pub review_start_date: Option<chrono::NaiveDate>,
    pub review_end_date: Option<chrono::NaiveDate>,
    pub total_budget: String,
    pub total_allocated: String,
    pub total_approved: String,
    pub total_employees: i32,
    pub currency_code: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Compensation budget pool for manager allocation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompensationBudgetPool {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub cycle_id: Uuid,
    pub pool_name: String,
    pub pool_type: String,
    pub manager_id: Option<Uuid>,
    pub manager_name: Option<String>,
    pub department_id: Option<Uuid>,
    pub department_name: Option<String>,
    pub total_budget: String,
    pub allocated_amount: String,
    pub approved_amount: String,
    pub remaining_budget: String,
    pub currency_code: String,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Compensation worksheet line (per-employee allocation)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompensationWorksheetLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub worksheet_id: Uuid,
    pub employee_id: Uuid,
    pub employee_name: Option<String>,
    pub job_title: Option<String>,
    pub department_name: Option<String>,
    pub current_base_salary: String,
    pub proposed_base_salary: String,
    pub salary_change_amount: String,
    pub salary_change_percent: String,
    pub merit_amount: String,
    pub bonus_amount: String,
    pub equity_amount: String,
    pub total_compensation: String,
    pub performance_rating: Option<String>,
    pub compa_ratio: String,
    pub status: String,
    pub manager_comments: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Compensation worksheet (manager's worksheet)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompensationWorksheet {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub cycle_id: Uuid,
    pub pool_id: Option<Uuid>,
    pub manager_id: Uuid,
    pub manager_name: Option<String>,
    pub status: String,
    pub total_employees: i32,
    pub total_current_salary: String,
    pub total_proposed_salary: String,
    pub total_merit: String,
    pub total_bonus: String,
    pub total_equity: String,
    pub total_compensation_change: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Compensation statement (employee view)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompensationStatement {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub cycle_id: Uuid,
    pub employee_id: Uuid,
    pub employee_name: Option<String>,
    pub statement_date: chrono::NaiveDate,
    pub base_salary: String,
    pub merit_increase: String,
    pub bonus: String,
    pub equity: String,
    pub benefits_value: String,
    pub total_compensation: String,
    pub total_direct_compensation: String,
    pub total_indirect_compensation: String,
    pub change_from_previous: String,
    pub change_percent: String,
    pub currency_code: String,
    pub components: serde_json::Value,
    pub status: String,
    pub published_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Compensation dashboard summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompensationDashboard {
    pub active_plans: i32,
    pub active_cycles: i32,
    pub total_budget: String,
    pub total_allocated: String,
    pub total_approved: String,
    pub total_employees_in_cycle: i32,
    pub pending_worksheets: i32,
    pub completed_worksheets: i32,
    pub average_salary_increase_percent: String,
    pub budget_utilization_percent: String,
}

// ============================================================================
// Recruiting Management (Oracle Fusion HCM > Recruiting)
// ============================================================================

/// Job Requisition
/// Oracle Fusion: HCM > Recruiting > Job Requisitions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobRequisition {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub requisition_number: String,
    pub title: String,
    pub description: Option<String>,
    pub department: Option<String>,
    pub location: Option<String>,
    pub employment_type: String,
    pub position_type: String,
    pub vacancies: i32,
    pub priority: String,
    pub salary_min: Option<String>,
    pub salary_max: Option<String>,
    pub currency: String,
    pub required_skills: serde_json::Value,
    pub qualifications: Option<String>,
    pub experience_years_min: Option<i32>,
    pub experience_years_max: Option<i32>,
    pub education_level: Option<String>,
    pub hiring_manager_id: Option<Uuid>,
    pub recruiter_id: Option<Uuid>,
    pub target_start_date: Option<chrono::NaiveDate>,
    pub status: String,
    pub posted_date: Option<DateTime<Utc>>,
    pub closed_date: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Candidate
/// Oracle Fusion: HCM > Recruiting > Candidates
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Candidate {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub candidate_number: Option<String>,
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub country: Option<String>,
    pub postal_code: Option<String>,
    pub linkedin_url: Option<String>,
    pub source: Option<String>,
    pub source_detail: Option<String>,
    pub resume_url: Option<String>,
    pub cover_letter_url: Option<String>,
    pub current_employer: Option<String>,
    pub current_title: Option<String>,
    pub years_of_experience: Option<i32>,
    pub education_level: Option<String>,
    pub skills: serde_json::Value,
    pub notes: Option<String>,
    pub status: String,
    pub tags: serde_json::Value,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Job Application
/// Oracle Fusion: HCM > Recruiting > Job Applications
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobApplication {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub application_number: Option<String>,
    pub requisition_id: Uuid,
    pub candidate_id: Uuid,
    pub status: String,
    pub match_score: String,
    pub screening_notes: Option<String>,
    pub rejection_reason: Option<String>,
    pub applied_at: DateTime<Utc>,
    pub last_status_change: DateTime<Utc>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Interview
/// Oracle Fusion: HCM > Recruiting > Interviews
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Interview {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub application_id: Uuid,
    pub interview_type: String,
    pub round: i32,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub duration_minutes: i32,
    pub location: Option<String>,
    pub meeting_link: Option<String>,
    pub interviewer_ids: serde_json::Value,
    pub interviewer_names: serde_json::Value,
    pub status: String,
    pub feedback: Option<String>,
    pub rating: Option<i32>,
    pub recommendation: Option<String>,
    pub completed_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Job Offer
/// Oracle Fusion: HCM > Recruiting > Job Offers
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobOffer {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub application_id: Uuid,
    pub offer_number: Option<String>,
    pub job_title: String,
    pub department: Option<String>,
    pub location: Option<String>,
    pub employment_type: String,
    pub start_date: Option<chrono::NaiveDate>,
    pub salary_offered: Option<String>,
    pub salary_currency: String,
    pub salary_frequency: String,
    pub signing_bonus: Option<String>,
    pub benefits_summary: Option<String>,
    pub terms_and_conditions: Option<String>,
    pub status: String,
    pub offer_date: Option<DateTime<Utc>>,
    pub response_deadline: Option<DateTime<Utc>>,
    pub responded_at: Option<DateTime<Utc>>,
    pub response_notes: Option<String>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Recruiting Dashboard
/// Oracle Fusion: HCM > Recruiting > Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecruitingDashboard {
    pub total_requisitions: i32,
    pub open_requisitions: i32,
    pub total_candidates: i32,
    pub total_applications: i32,
    pub applications_this_month: i32,
    pub interviews_this_month: i32,
    pub offers_pending: i32,
    pub hires_this_month: i32,
    pub requisitions_by_status: serde_json::Value,
    pub applications_by_status: serde_json::Value,
    pub top_departments: serde_json::Value,
    pub recent_applications: serde_json::Value,
}

// ============================================================================
// Goal Management (Oracle Fusion HCM > Goal Management)
// ============================================================================

/// Goal Library Category: groups library templates.
/// Oracle Fusion equivalent: Goal Library > Categories
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoalLibraryCategory {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub display_order: i32,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Goal Library Template: predefined goal template.
/// Oracle Fusion equivalent: Goal Library > Goal Templates
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoalLibraryTemplate {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub category_id: Option<Uuid>,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub goal_type: String,
    pub success_criteria: Option<String>,
    pub target_metric: Option<String>,
    pub target_value: Option<String>,
    pub uom: Option<String>,
    pub suggested_weight: Option<String>,
    pub estimated_duration_days: Option<i32>,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Goal Plan: a performance or development period that contains goals.
/// Oracle Fusion equivalent: Goal Management > Goal Plans
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoalPlan {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub plan_type: String,
    pub review_period_start: chrono::NaiveDate,
    pub review_period_end: chrono::NaiveDate,
    pub goal_creation_deadline: Option<chrono::NaiveDate>,
    pub status: String,
    pub allow_self_goals: bool,
    pub allow_team_goals: bool,
    pub max_weight_sum: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Goal: individual, team, or organizational goal with progress tracking.
/// Oracle Fusion equivalent: Goal Management > Goals
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Goal {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub plan_id: Option<Uuid>,
    pub parent_goal_id: Option<Uuid>,
    pub library_template_id: Option<Uuid>,
    pub code: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub goal_type: String,
    pub category: Option<String>,
    pub owner_id: Uuid,
    pub owner_type: String,
    pub assigned_by: Option<Uuid>,
    pub success_criteria: Option<String>,
    pub target_metric: Option<String>,
    pub target_value: Option<String>,
    pub actual_value: Option<String>,
    pub uom: Option<String>,
    pub progress_pct: Option<String>,
    pub weight: Option<String>,
    pub status: String,
    pub priority: String,
    pub start_date: Option<chrono::NaiveDate>,
    pub target_date: Option<chrono::NaiveDate>,
    pub completed_date: Option<chrono::NaiveDate>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Goal Alignment: explicit link between two goals showing how they relate.
/// Oracle Fusion equivalent: Goal Management > Alignments
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoalAlignment {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub source_goal_id: Uuid,
    pub aligned_to_goal_id: Uuid,
    pub alignment_type: String,
    pub description: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// Goal Note: comment or feedback on a goal.
/// Oracle Fusion equivalent: Goal Management > Notes / Check-ins
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoalNote {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub goal_id: Uuid,
    pub author_id: Uuid,
    pub note_type: String,
    pub content: String,
    pub visibility: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Goal Management Dashboard Summary.
/// Oracle Fusion equivalent: Goal Management > Summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoalManagementSummary {
    pub total_goals: i32,
    pub goals_not_started: i32,
    pub goals_in_progress: i32,
    pub goals_on_track: i32,
    pub goals_at_risk: i32,
    pub goals_completed: i32,
    pub goals_cancelled: i32,
    pub avg_progress_pct: Option<String>,
    pub total_plans: i32,
    pub active_plans: i32,
    pub total_alignments: i32,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Landed Cost Management (Oracle Fusion SCM > Landed Cost Management)
// ═══════════════════════════════════════════════════════════════════════════════

/// Landed Cost Template
/// Oracle Fusion: SCM > Landed Cost Management > Cost Templates
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LandedCostTemplate {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Landed Cost Component (e.g., Freight, Insurance, Customs Duty)
/// Oracle Fusion: SCM > Landed Cost Management > Cost Components
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LandedCostComponent {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub template_id: Option<Uuid>,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub cost_type: String,
    pub allocation_basis: String,
    pub default_rate: Option<String>,
    pub rate_uom: Option<String>,
    pub expense_account: Option<String>,
    pub is_taxable: bool,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Landed Cost Charge Header
/// Oracle Fusion: SCM > Landed Cost Management > Charges
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LandedCostCharge {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub charge_number: String,
    pub template_id: Option<Uuid>,
    pub receipt_id: Option<Uuid>,
    pub purchase_order_id: Option<Uuid>,
    pub supplier_id: Option<Uuid>,
    pub supplier_name: Option<String>,
    pub charge_type: String,
    pub charge_date: Option<chrono::NaiveDate>,
    pub total_amount: String,
    pub currency: String,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Landed Cost Charge Line
/// Oracle Fusion: SCM > Landed Cost Management > Charge Lines
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LandedCostChargeLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub charge_id: Uuid,
    pub component_id: Option<Uuid>,
    pub line_number: i32,
    pub receipt_line_id: Option<Uuid>,
    pub item_id: Option<Uuid>,
    pub item_code: Option<String>,
    pub item_description: Option<String>,
    pub charge_amount: String,
    pub allocated_amount: String,
    pub allocation_basis: String,
    pub allocation_qty: Option<String>,
    pub allocation_value: Option<String>,
    pub expense_account: Option<String>,
    pub notes: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Landed Cost Allocation
/// Oracle Fusion: SCM > Landed Cost Management > Allocations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LandedCostAllocation {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub charge_id: Uuid,
    pub charge_line_id: Uuid,
    pub receipt_id: Option<Uuid>,
    pub receipt_line_id: Option<Uuid>,
    pub item_id: Option<Uuid>,
    pub item_code: Option<String>,
    pub allocated_amount: String,
    pub allocation_basis: String,
    pub allocation_basis_value: Option<String>,
    pub total_basis_value: Option<String>,
    pub allocation_pct: Option<String>,
    pub unit_landed_cost: Option<String>,
    pub original_unit_cost: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Landed Cost Simulation
/// Oracle Fusion: SCM > Landed Cost Management > Simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LandedCostSimulation {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub simulation_number: String,
    pub template_id: Option<Uuid>,
    pub purchase_order_id: Option<Uuid>,
    pub item_id: Option<Uuid>,
    pub item_code: Option<String>,
    pub item_description: Option<String>,
    pub estimated_quantity: String,
    pub unit_price: String,
    pub currency: String,
    pub estimated_charges: serde_json::Value,
    pub estimated_landed_cost: String,
    pub estimated_landed_cost_per_unit: String,
    pub variance_vs_actual: Option<String>,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Landed Cost Dashboard Summary
/// Oracle Fusion: SCM > Landed Cost Management > Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LandedCostDashboard {
    pub total_charges: i32,
    pub pending_charges: i32,
    pub allocated_charges: i32,
    pub total_charge_amount: String,
    pub total_allocated_amount: String,
    pub total_simulations: i32,
    pub charges_by_type: serde_json::Value,
    pub recent_charges: serde_json::Value,
    pub top_cost_components: serde_json::Value,
}

// ============================================================================
// Succession Planning
// Oracle Fusion: HCM > Succession Management > Succession Plans, Talent Pools,
//   Talent Reviews, Career Paths
// ============================================================================

/// Succession Plan
/// Oracle Fusion: HCM > Succession Management > Succession Plans
/// A plan for a key position identifying backup candidates and their readiness.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuccessionPlan {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub plan_type: String, // position, role, key_person
    pub position_id: Option<Uuid>,
    pub position_title: Option<String>,
    pub job_id: Option<Uuid>,
    pub department_id: Option<Uuid>,
    pub current_incumbent_id: Option<Uuid>,
    pub current_incumbent_name: Option<String>,
    pub risk_level: String, // low, medium, high, critical
    pub urgency: String,    // immediate, short_term, medium_term, long_term
    pub status: String,     // draft, active, completed, cancelled
    pub effective_date: Option<chrono::NaiveDate>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Succession Plan Candidate
/// Oracle Fusion: HCM > Succession Management > Plan Candidates
/// A candidate within a succession plan with readiness assessment.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuccessionCandidate {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub plan_id: Uuid,
    pub person_id: Uuid,
    pub person_name: Option<String>,
    pub employee_number: Option<String>,
    pub readiness: String, // ready_now, ready_1_2_years, ready_3_5_years, not_ready
    pub ranking: Option<i32>,
    pub performance_rating: Option<String>, // 1-5 scale or labels
    pub potential_rating: Option<String>,   // 1-5 scale or labels
    pub flight_risk: Option<String>,        // low, medium, high
    pub development_notes: Option<String>,
    pub recommended_actions: Option<String>,
    pub status: String, // proposed, approved, rejected, development
    pub metadata: serde_json::Value,
    pub added_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Talent Pool
/// Oracle Fusion: HCM > Succession Management > Talent Pools
/// A named group of high-potential employees tracked for development.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TalentPool {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub pool_type: String, // leadership, technical, high_potential, diversity, custom
    pub owner_id: Option<Uuid>,
    pub max_members: Option<i32>,
    pub status: String, // draft, active, archived
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Talent Pool Member
/// Oracle Fusion: HCM > Succession Management > Pool Members
/// A member of a talent pool with their assessment info.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TalentPoolMember {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub pool_id: Uuid,
    pub person_id: Uuid,
    pub person_name: Option<String>,
    pub performance_rating: Option<String>,
    pub potential_rating: Option<String>,
    pub readiness: String, // ready_now, ready_1_2_years, ready_3_5_years, not_ready
    pub development_plan: Option<String>,
    pub notes: Option<String>,
    pub added_date: Option<chrono::NaiveDate>,
    pub review_date: Option<chrono::NaiveDate>,
    pub status: String, // active, on_hold, removed, graduated
    pub metadata: serde_json::Value,
    pub added_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Talent Review
/// Oracle Fusion: HCM > Succession Management > Talent Review Meetings
/// A formal assessment session where managers evaluate talent.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TalentReview {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub review_type: String, // calibration, performance_potential, nine_box, leadership
    pub facilitator_id: Option<Uuid>,
    pub department_id: Option<Uuid>,
    pub review_date: Option<chrono::NaiveDate>,
    pub status: String, // scheduled, in_progress, completed, cancelled
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Talent Review Assessment
/// Oracle Fusion: HCM > Succession Management > Review Assessments
/// An individual assessment within a talent review meeting.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TalentReviewAssessment {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub review_id: Uuid,
    pub person_id: Uuid,
    pub person_name: Option<String>,
    pub performance_rating: Option<String>,
    pub potential_rating: Option<String>,
    pub nine_box_position: Option<String>, // star, workhorse, puzzle, solid_citizen, etc.
    pub strengths: Option<String>,
    pub weaknesses: Option<String>,
    pub career_aspiration: Option<String>,
    pub development_needs: Option<String>,
    pub succession_readiness: Option<String>,
    pub assessor_id: Option<Uuid>,
    pub notes: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Career Path
/// Oracle Fusion: HCM > Succession Management > Career Paths
/// A defined progression path between jobs/roles.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CareerPath {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub path_type: String, // linear, branching, lattice, dual_track
    pub from_job_id: Option<Uuid>,
    pub from_job_title: Option<String>,
    pub to_job_id: Option<Uuid>,
    pub to_job_title: Option<String>,
    pub typical_duration_months: Option<i32>,
    pub required_competencies: Option<String>,
    pub required_certifications: Option<String>,
    pub development_activities: Option<String>,
    pub status: String, // draft, active, archived
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Succession Planning Dashboard Summary
/// Oracle Fusion: HCM > Succession Management > Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuccessionDashboard {
    pub total_succession_plans: i32,
    pub active_plans: i32,
    pub plans_by_risk: serde_json::Value,
    pub plans_by_urgency: serde_json::Value,
    pub total_candidates: i32,
    pub candidates_by_readiness: serde_json::Value,
    pub total_talent_pools: i32,
    pub total_pool_members: i32,
    pub total_reviews: i32,
    pub total_career_paths: i32,
    pub coverage_pct: Option<String>,
}

// ============================================================================
// Learning Management
// Oracle Fusion: HCM > Learning > Courses, Specializations, Certifications,
//   Learning Paths, Enrollments, Completions, Assignments
// ============================================================================

/// Learning Item
/// Oracle Fusion: HCM > Learning > Learning Items
/// A learning object such as a course, certification, or specialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LearningItem {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub title: String,
    pub description: Option<String>,
    pub item_type: String, // course, certification, specialization, video, assessment, blended
    pub format: String,    // online, classroom, virtual_classroom, self_paced, blended
    pub category: Option<String>,
    pub provider: Option<String>,
    pub duration_hours: Option<f64>,
    pub currency_code: Option<String>,
    pub cost: Option<String>,
    pub credits: Option<String>,
    pub credit_type: Option<String>, // ceu, cpe, pdu, college_credit, custom
    pub validity_months: Option<i32>, // how long the certification remains valid
    pub recertification_required: bool,
    pub max_enrollments: Option<i32>,
    pub status: String, // draft, active, inactive, archived
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Learning Category
/// Oracle Fusion: HCM > Learning > Catalog Categories
/// A hierarchical category for organizing learning items.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LearningCategory {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub parent_category_id: Option<Uuid>,
    pub display_order: i32,
    pub status: String, // active, inactive
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Learning Enrollment
/// Oracle Fusion: HCM > Learning > Enrollments
/// A person's enrollment in a learning item.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LearningEnrollment {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub learning_item_id: Uuid,
    pub person_id: Uuid,
    pub person_name: Option<String>,
    pub enrollment_type: String, // self, manager, mandatory, auto_assigned
    pub enrolled_by: Option<Uuid>,
    pub status: String, // enrolled, in_progress, completed, failed, withdrawn, expired
    pub progress_pct: Option<String>,
    pub score: Option<String>,
    pub enrollment_date: Option<chrono::NaiveDate>,
    pub completion_date: Option<chrono::NaiveDate>,
    pub due_date: Option<chrono::NaiveDate>,
    pub certification_expiry: Option<chrono::NaiveDate>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Learning Path
/// Oracle Fusion: HCM > Learning > Learning Paths / Curricula
/// A sequence of learning items forming a curriculum.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LearningPath {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub path_type: String, // sequential, elective, milestone, tiered
    pub target_role: Option<String>,
    pub target_job_id: Option<Uuid>,
    pub estimated_duration_hours: Option<f64>,
    pub total_items: i32,
    pub status: String, // draft, active, inactive, archived
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Learning Path Item
/// Oracle Fusion: HCM > Learning > Path Steps
/// A single step/item within a learning path.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LearningPathItem {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub learning_path_id: Uuid,
    pub learning_item_id: Uuid,
    pub sequence_number: i32,
    pub is_required: bool,
    pub milestone_name: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Learning Assignment
/// Oracle Fusion: HCM > Learning > Mandatory Assignments
/// A mandatory learning requirement assigned to a person or group.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LearningAssignment {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub learning_item_id: Option<Uuid>,
    pub learning_path_id: Option<Uuid>,
    pub title: String,
    pub description: Option<String>,
    pub assignment_type: String, // individual, organization, department, job, position
    pub target_id: Option<Uuid>,
    pub assigned_by: Option<Uuid>,
    pub priority: String, // low, medium, high, critical
    pub due_date: Option<chrono::NaiveDate>,
    pub status: String, // active, completed, cancelled
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Learning Dashboard Summary
/// Oracle Fusion: HCM > Learning > Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LearningDashboard {
    pub total_learning_items: i32,
    pub active_items: i32,
    pub items_by_type: serde_json::Value,
    pub total_enrollments: i32,
    pub enrollments_by_status: serde_json::Value,
    pub completion_rate: Option<String>,
    pub total_learning_paths: i32,
    pub total_active_assignments: i32,
    pub overdue_enrollments: i32,
    pub avg_score: Option<String>,
}
