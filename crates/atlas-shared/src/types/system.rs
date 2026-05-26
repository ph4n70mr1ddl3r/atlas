use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::types::*;
// ============================================================================
// Schema Metadata
// ============================================================================

/// Schema version for caching and hot-reload
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaVersion {
    pub entity: String,
    pub version: i64,
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// Import Job Types
// ============================================================================

/// Import job status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ImportJobStatus {
    Pending,
    Validating,
    Importing,
    Completed,
    Failed,
    Cancelled,
}

/// Import job tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportJob {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub user_id: Uuid,
    pub entity_type: String,
    pub format: String,
    pub status: String,
    pub total_rows: i32,
    pub processed_rows: i32,
    pub imported_rows: i32,
    pub failed_rows: i32,
    pub skipped_rows: i32,
    pub original_filename: Option<String>,
    pub file_size_bytes: Option<i64>,
    pub field_mapping: serde_json::Value,
    pub upsert_mode: bool,
    pub skip_validation: bool,
    pub stop_on_error: bool,
    pub validation_errors: serde_json::Value,
    pub import_errors: serde_json::Value,
    pub duplicate_action: String,
    pub duplicates_found: i32,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

// ============================================================================
// Duplicate Detection
// ============================================================================

/// Duplicate detection rule
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateRule {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub entity_type: String,
    pub description: Option<String>,
    pub match_criteria: serde_json::Value,
    pub filter_condition: serde_json::Value,
    pub on_duplicate: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Duplicate match criterion
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchCriterion {
    pub field: String,
    pub match_type: String, // "exact", "fuzzy", "case_insensitive"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub threshold: Option<f64>,
}

/// Detected duplicate
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectedDuplicate {
    pub rule_name: String,
    pub entity_type: String,
    pub existing_record_id: Uuid,
    pub match_field: String,
    pub match_type: String,
    pub existing_value: serde_json::Value,
    pub new_value: serde_json::Value,
}

// ============================================================================
// Document Sequencing
// Oracle Fusion: General Ledger > Setup > Document Sequencing
// ============================================================================

/// A document sequence definition.
///
/// Controls automatic numbering of business documents (invoices, POs, journals, etc.)
/// Supports gapless (regulatory compliance) and gap-permitted (operational) modes.
/// Oracle Fusion: General Ledger > Setup > Sequences > Document Sequences
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentSequence {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub sequence_type: String, // gapless, gap_permitted, manual
    pub document_type: String, // invoice, purchase_order, journal_entry, payment, receipt, etc.
    pub initial_value: i64,
    pub current_value: i64,
    pub increment_by: i32,
    pub max_value: Option<i64>,
    pub cycle_flag: bool,
    pub prefix: Option<String>,
    pub suffix: Option<String>,
    pub pad_length: i32,
    pub pad_character: String,
    pub reset_frequency: Option<String>, // daily, monthly, quarterly, annually, never
    pub last_reset_date: Option<chrono::NaiveDate>,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub status: String, // active, inactive
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A sequence assignment that maps a document sequence to a specific
/// document category + business unit + ledger combination.
/// Oracle Fusion: General Ledger > Setup > Sequences > Sequence Assignments
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentSequenceAssignment {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub sequence_id: Uuid,
    pub sequence_code: String,
    pub document_category: String, // e.g. "accounts_payable_invoice", "gl_journal"
    pub business_unit_id: Option<Uuid>,
    pub ledger_id: Option<Uuid>,
    pub method: String, // automatic, manual
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub priority: i32,
    pub status: String, // active, inactive
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// An audit record for a generated document number.
/// Tracks every number assignment for compliance and traceability.
/// Oracle Fusion: General Ledger > Sequences > Sequence Audit
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentSequenceAudit {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub sequence_id: Uuid,
    pub sequence_code: String,
    pub generated_number: String,
    pub numeric_value: i64,
    pub document_category: String,
    pub document_id: Option<Uuid>,
    pub document_number: Option<String>,
    pub business_unit_id: Option<Uuid>,
    pub generated_at: DateTime<Utc>,
    pub generated_by: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// Summary statistics for document sequences.
/// Oracle Fusion: General Ledger > Sequences > Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentSequenceDashboardSummary {
    pub total_sequences: i32,
    pub active_sequences: i32,
    pub gapless_sequences: i32,
    pub gap_permitted_sequences: i32,
    pub total_numbers_generated: i64,
    pub total_assignments: i32,
    pub recent_audits: Vec<DocumentSequenceAudit>,
    pub sequences_by_type: serde_json::Value,
    pub sequences_by_document_type: serde_json::Value,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Descriptive Flexfields (Oracle Fusion DFF)
// ═══════════════════════════════════════════════════════════════════════════════
//
// Descriptive Flexfields allow administrators to add custom configurable
// fields to any entity at runtime. They support:
// - Global segments (always visible on the entity)
// - Context-sensitive segments (visible based on a context value)
// - Value sets with validation rules (table-validated, independent, dependent, etc.)
// - Required / optional segments with defaults
//
// Oracle Fusion equivalent: Application Extensions > Flexfields > Descriptive
// ═══════════════════════════════════════════════════════════════════════════════

/// A value set defines the list of valid values for a flexfield segment.
/// Value sets enforce validation at data entry time.
///
/// Oracle Fusion: Setup and Maintenance > Manage Value Sets
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlexfieldValueSet {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    /// Type of validation: "none", "independent", "dependent", "table", "`format_only`"
    pub validation_type: String,
    /// Data type for values: "string", "number", "date", "datetime"
    pub data_type: String,
    /// Maximum length for string values
    pub max_length: i32,
    /// Minimum length for string values
    pub min_length: i32,
    /// For `format_only`: regex or format pattern
    pub format_mask: Option<String>,
    /// For table validation: table name, value column, meaning column, WHERE clause
    pub table_validation: Option<serde_json::Value>,
    /// For independent: list of valid values stored as JSON
    pub independent_values: Option<serde_json::Value>,
    /// For dependent: reference to parent value set code
    pub parent_value_set_code: Option<String>,
    pub is_active: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A value set value entry (for independent and dependent value sets).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlexfieldValueSetEntry {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub value_set_id: Uuid,
    pub value: String,
    pub meaning: Option<String>,
    pub description: Option<String>,
    /// For dependent value sets: the parent value this depends on
    pub parent_value: Option<String>,
    pub is_enabled: bool,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub sort_order: i32,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A descriptive flexfield definition attached to an entity.
///
/// Oracle Fusion: Application Extensions > Flexfields > Descriptive Flexfields
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DescriptiveFlexfield {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    /// The entity/table this flexfield is attached to (e.g., "`purchase_orders`")
    pub entity_name: String,
    /// The column on the entity table where the context value is stored (default: "`dff_context`")
    pub context_column: String,
    /// Default context code used when no context is specified
    pub default_context_code: Option<String>,
    /// Whether the flexfield is active
    pub is_active: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A context within a flexfield. Contexts group segments and allow
/// context-sensitive custom fields (different fields appear based on context).
///
/// Oracle Fusion: Each DFF can have multiple contexts (e.g., "US", "EU" for
/// region-specific fields, or "IT", "Hardware" for category-specific fields)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlexfieldContext {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub flexfield_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    /// Whether this is a global context (segments apply to all records)
    pub is_global: bool,
    pub is_enabled: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A segment (custom field) within a flexfield context.
/// Segments are ordered and can be required or optional.
///
/// Oracle Fusion: Each context has 1-N segments with display ordering
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlexfieldSegment {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub flexfield_id: Uuid,
    pub context_id: Uuid,
    pub segment_code: String,
    pub name: String,
    pub description: Option<String>,
    /// Display order within the context
    pub display_order: i32,
    /// The column used to store this segment's value (e.g., "attribute1", "attribute2")
    pub column_name: String,
    /// Data type: "string", "number", "date", "datetime"
    pub data_type: String,
    /// Whether this segment is required
    pub is_required: bool,
    /// Whether this segment is read-only
    pub is_read_only: bool,
    /// Whether this segment is visible on forms
    pub is_visible: bool,
    /// Default value for the segment
    pub default_value: Option<String>,
    /// Reference to the value set for validation (by ID)
    pub value_set_id: Option<Uuid>,
    /// Reference to the value set for validation (by code, denormalized)
    pub value_set_code: Option<String>,
    /// Help text shown to users
    pub help_text: Option<String>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Flexfield data values stored for a specific entity record.
/// The context value plus all segment values for that context.
///
/// Oracle Fusion: DFF values are stored in attribute columns on the entity row
/// or in a dedicated flexfield values table.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlexfieldData {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub flexfield_id: Uuid,
    pub entity_name: String,
    pub entity_id: Uuid,
    /// The context code selected for this record
    pub context_code: String,
    /// Segment values as a JSON object: {"`segment_code"`: "value", ...}
    pub segment_values: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Dashboard summary for descriptive flexfields.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlexfieldDashboardSummary {
    pub total_flexfields: i32,
    pub active_flexfields: i32,
    pub total_contexts: i32,
    pub total_segments: i32,
    pub total_value_sets: i32,
    pub flexfields_by_entity: serde_json::Value,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Cross-Validation Rules (Oracle Fusion GL > Chart of Accounts > Cross-Validation)
// ═══════════════════════════════════════════════════════════════════════════════
//
// Cross-Validation Rules prevent users from creating invalid combinations
// of account segment values. For example, you can prevent creating an
// account like "1000.Cash.4000.Marketing" if that combination doesn't
// make business sense.
//
// Each rule defines a pattern of segment values that must (or must not)
// co-occur. Patterns use exact values or wildcards ("%" = any value,
// "T" = typed shorthand, etc.).
//
// Oracle Fusion equivalent: General Ledger > Setup > Chart of Accounts >
//   Cross-Validation Rules
// ═══════════════════════════════════════════════════════════════════════════════

/// A Cross-Validation Rule (CVR) definition.
/// Each rule has a name, an enabled flag, effective dates, an error message,
/// and a type ("allow" or "deny").
///
/// Oracle Fusion: General Ledger > Setup > Chart of Accounts > Cross-Validation Rules
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrossValidationRule {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Unique code for the rule, e.g. "`CVR_CASH_MARKETING`"
    pub code: String,
    /// Human-readable name
    pub name: String,
    /// Description of the rule
    pub description: Option<String>,
    /// Rule type: "deny" blocks matching combinations, "allow" permits them
    pub rule_type: String,
    /// Error message shown when the rule is violated
    pub error_message: String,
    /// Whether this rule is enabled
    pub is_enabled: bool,
    /// Priority order (lower = evaluated first)
    pub priority: i32,
    /// Segment names in the chart of accounts (e.g. ["company", "department", "account"])
    pub segment_names: Vec<String>,
    /// Effective from date
    pub effective_from: Option<chrono::NaiveDate>,
    /// Effective to date
    pub effective_to: Option<chrono::NaiveDate>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A validation line within a Cross-Validation Rule.
/// Each line specifies a pattern for the account segment combination.
/// Patterns use exact values or "%" as a wildcard meaning "any value".
///
/// Example ("deny" rule with 2 lines):
///   Line 1 (from): {"patterns": ["1000", "%", "%"]}  -- company=1000
///   Line 2 (to):   {"patterns": ["%", "%", "5000"]}  -- account=5000
///   Meaning: Deny any combination where company=1000 AND account=5000
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrossValidationRuleLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub rule_id: Uuid,
    /// Line type: "from" (left side) or "to" (right side)
    pub line_type: String,
    /// Pattern values for each segment. "%" = any value, exact string = exact match
    pub patterns: Vec<String>,
    /// Display order
    pub display_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Result of a cross-validation check.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrossValidationResult {
    /// Whether the combination is valid (passes all rules)
    pub is_valid: bool,
    /// List of violated rule codes
    pub violated_rules: Vec<String>,
    /// List of error messages from violated rules
    pub error_messages: Vec<String>,
}

/// Dashboard summary for Cross-Validation Rules.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrossValidationDashboardSummary {
    pub total_rules: i32,
    pub enabled_rules: i32,
    pub deny_rules: i32,
    pub allow_rules: i32,
    pub total_lines: i32,
    pub rules_by_type: serde_json::Value,
}

// ============================================================================
// Scheduled Processes (Oracle Fusion Enterprise Scheduler Service)
// ============================================================================
// Oracle Fusion: Navigator > Tools > Scheduled Processes
// Allows users to submit, schedule, and monitor batch processes, reports,
// data imports/exports, and custom jobs.

/// Process template definition
/// Oracle Fusion: Scheduled Processes > Manage Process Templates
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduledProcessTemplate {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub process_type: String,
    pub executor_type: String,
    pub executor_config: serde_json::Value,
    pub parameters: serde_json::Value,
    pub default_parameters: serde_json::Value,
    pub timeout_minutes: i32,
    pub max_retries: i32,
    pub retry_delay_minutes: i32,
    pub requires_approval: bool,
    pub approval_chain_id: Option<Uuid>,
    pub is_active: bool,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Submitted process instance
/// Oracle Fusion: Scheduled Processes > Submit New Process / Monitor
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduledProcess {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub template_id: Option<Uuid>,
    pub template_code: Option<String>,
    pub process_name: String,
    pub process_type: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: String,
    pub submitted_by: Uuid,
    pub submitted_at: DateTime<Utc>,
    pub scheduled_start_at: Option<DateTime<Utc>>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub cancelled_by: Option<Uuid>,
    pub cancel_reason: Option<String>,
    pub last_heartbeat_at: Option<DateTime<Utc>>,
    pub retry_count: i32,
    pub max_retries: i32,
    pub timeout_minutes: i32,
    pub progress_percent: i32,
    pub parameters: serde_json::Value,
    pub result_summary: Option<String>,
    pub output_file_url: Option<String>,
    pub output_format: String,
    pub log_output: Option<String>,
    pub error_message: Option<String>,
    pub parent_process_id: Option<Uuid>,
    pub recurrence_id: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Recurrence schedule for recurring process submissions
/// Oracle Fusion: Scheduled Processes > Schedule > Recurrence
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduledProcessRecurrence {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub template_id: Uuid,
    pub template_code: Option<String>,
    pub parameters: serde_json::Value,
    pub recurrence_type: String,
    pub recurrence_config: serde_json::Value,
    pub start_date: chrono::NaiveDate,
    pub end_date: Option<chrono::NaiveDate>,
    pub next_run_at: Option<DateTime<Utc>>,
    pub last_run_at: Option<DateTime<Utc>>,
    pub run_count: i32,
    pub max_runs: Option<i32>,
    pub is_active: bool,
    pub submitted_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Process execution log entry
/// Oracle Fusion: Scheduled Processes > Process Details > Log
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduledProcessLog {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub process_id: Uuid,
    pub log_level: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
    pub step_name: Option<String>,
    pub duration_ms: Option<i32>,
    pub created_at: DateTime<Utc>,
}

/// Scheduled processes dashboard summary
/// Oracle Fusion: Scheduled Processes > Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduledProcessDashboardSummary {
    pub total_processes: i32,
    pub pending_processes: i32,
    pub running_processes: i32,
    pub completed_processes: i32,
    pub failed_processes: i32,
    pub cancelled_processes: i32,
    pub scheduled_processes: i32,
    pub active_recurrences: i32,
    pub recent_processes: Vec<ScheduledProcess>,
    pub processes_by_type: serde_json::Value,
}

// ═══════════════════════════════════════════════════════════════════════════
// Segregation of Duties (SoD)
// Oracle Fusion: Advanced Access Control > Segregation of Duties
// ═══════════════════════════════════════════════════════════════════════════

/// An `SoD` rule defines a pair (or set) of incompatible duties/roles.
/// For example: "Create Vendor" and "Approve Vendor Payments" must not be
/// held by the same person.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SodRule {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    /// The set of duties that conflict with each other.
    /// A violation occurs when a single user holds duties from
    /// *both* `first_duties` AND `second_duties`.
    pub first_duties: Vec<String>,
    pub second_duties: Vec<String>,
    /// "preventive" = block violating role assignments,
    /// "detective" = report but allow
    pub enforcement_mode: String,
    /// Risk level: "high", "medium", "low"
    pub risk_level: String,
    pub is_active: bool,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// An `SoD` violation detected for a specific user against a rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SodViolation {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub rule_id: Uuid,
    pub rule_code: String,
    pub user_id: Uuid,
    /// The duties from the first set the user holds
    pub first_matched_duties: Vec<String>,
    /// The duties from the second set the user holds
    pub second_matched_duties: Vec<String>,
    pub violation_status: String, // "open", "mitigated", "exception", "resolved"
    pub detected_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolved_by: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A mitigating control applied to an `SoD` violation.
/// In Oracle Fusion, this is a documented compensating control that
/// reduces the risk of the conflict to an acceptable level.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SodMitigatingControl {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub violation_id: Uuid,
    pub control_name: String,
    pub control_description: String,
    /// Who is responsible for executing this control
    pub control_owner_id: Option<Uuid>,
    /// Frequency: "daily", "weekly", "monthly", "quarterly"
    pub review_frequency: String,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub status: String, // "active", "expired", "revoked"
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Role assignment entry tracked for `SoD` analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SodRoleAssignment {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub user_id: Uuid,
    pub role_name: String,
    /// The duty/privilege this role grants
    pub duty_code: String,
    pub assigned_by: Option<Uuid>,
    pub assigned_at: DateTime<Utc>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Result of checking a proposed role assignment for conflicts.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SodConflictCheckResult {
    pub has_conflicts: bool,
    pub conflicts: Vec<SodConflictDetail>,
    pub would_be_blocked: bool,
}

/// Details of a single conflict found during a check.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SodConflictDetail {
    pub rule_id: Uuid,
    pub rule_code: String,
    pub rule_name: String,
    pub risk_level: String,
    pub enforcement_mode: String,
    pub conflicting_duty: String,
    pub existing_duties_causing_conflict: Vec<String>,
}

/// Dashboard summary for `SoD` compliance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SodDashboardSummary {
    pub total_rules: i32,
    pub active_rules: i32,
    pub total_violations: i32,
    pub open_violations: i32,
    pub mitigated_violations: i32,
    pub exception_violations: i32,
    pub violations_by_risk_level: serde_json::Value,
    pub recent_violations: Vec<SodViolation>,
    pub rules_summary: serde_json::Value,
}

// ============================================================================
// General Ledger Allocations (Oracle Fusion GL Allocations)
// ============================================================================

/// GL Allocation pool definition
/// Oracle Fusion: General Ledger > Allocations > Allocation Pools
/// Represents a pool of costs/amounts to be distributed to targets.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlAllocationPool {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Unique code for this pool (e.g., "`RENT_POOL`", "`IT_OVERHEAD`")
    pub code: String,
    /// Human-readable name
    pub name: String,
    pub description: Option<String>,
    /// Pool type: '`cost_center`', '`account_range`', 'manual'
    pub pool_type: String,
    /// Source account code or range for the pool
    pub source_account_code: Option<String>,
    pub source_account_range_from: Option<String>,
    pub source_account_range_to: Option<String>,
    /// Source cost center or department
    pub source_department_id: Option<Uuid>,
    /// Source project
    pub source_project_id: Option<Uuid>,
    /// Currency code
    pub currency_code: String,
    /// Whether this pool is active
    pub is_active: bool,
    /// Start/end dates for effective period
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create/update GL allocation pool request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlAllocationPoolRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(default = "default_pool_type")]
    pub pool_type: String,
    pub source_account_code: Option<String>,
    pub source_account_range_from: Option<String>,
    pub source_account_range_to: Option<String>,
    pub source_department_id: Option<Uuid>,
    pub source_project_id: Option<Uuid>,
    #[serde(default = "default_currency_usd")]
    pub currency_code: String,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
}

pub fn default_pool_type() -> String { "cost_center".to_string() }

/// GL Allocation basis definition
/// Oracle Fusion: General Ledger > Allocations > Allocation Bases
/// Defines how amounts are distributed (e.g., headcount, revenue, square footage).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlAllocationBasis {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Unique code (e.g., "HEADCOUNT", "REVENUE", "SQFT")
    pub code: String,
    /// Human-readable name
    pub name: String,
    pub description: Option<String>,
    /// Basis type: 'statistical', 'financial', 'percentage'
    pub basis_type: String,
    /// Unit of measure (e.g., 'people', 'USD', '`sq_ft`')
    pub unit_of_measure: Option<String>,
    /// Whether the basis amounts are entered manually or sourced from GL
    pub is_manual: bool,
    /// GL account code to source basis amounts from (if not manual)
    pub source_account_code: Option<String>,
    /// Whether this basis is active
    pub is_active: bool,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create/update GL allocation basis request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlAllocationBasisRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(default = "default_basis_type")]
    pub basis_type: String,
    pub unit_of_measure: Option<String>,
    #[serde(default)]
    pub is_manual: bool,
    pub source_account_code: Option<String>,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
}

pub fn default_basis_type() -> String { "statistical".to_string() }

/// GL Allocation basis detail (individual target's share of the basis)
/// Oracle Fusion: General Ledger > Allocations > Basis Details
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlAllocationBasisDetail {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// The basis this detail belongs to
    pub basis_id: Uuid,
    /// Target dimension: department, cost center, project, or account
    pub target_department_id: Option<Uuid>,
    pub target_department_name: Option<String>,
    pub target_cost_center: Option<String>,
    pub target_project_id: Option<Uuid>,
    pub target_project_name: Option<String>,
    pub target_account_code: Option<String>,
    /// The basis amount (headcount count, revenue amount, square footage, etc.)
    pub basis_amount: String,
    /// The percentage this detail represents of the total basis
    pub percentage: String,
    /// Effective period
    pub period_name: Option<String>,
    pub period_start_date: Option<chrono::NaiveDate>,
    pub period_end_date: Option<chrono::NaiveDate>,
    pub is_active: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create/update GL allocation basis detail request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlAllocationBasisDetailRequest {
    pub target_department_id: Option<Uuid>,
    pub target_department_name: Option<String>,
    pub target_cost_center: Option<String>,
    pub target_project_id: Option<Uuid>,
    pub target_project_name: Option<String>,
    pub target_account_code: Option<String>,
    pub basis_amount: String,
    pub period_name: Option<String>,
    pub period_start_date: Option<chrono::NaiveDate>,
    pub period_end_date: Option<chrono::NaiveDate>,
}

/// GL Allocation rule definition
/// Oracle Fusion: General Ledger > Allocations > Allocation Rules
/// Maps a pool to targets using a basis.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlAllocationRule {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Unique code (e.g., "`RENT_ALLOC`", "`IT_OVERHEAD_ALLOC`")
    pub code: String,
    /// Human-readable name
    pub name: String,
    pub description: Option<String>,
    /// The pool to allocate from
    pub pool_id: Uuid,
    pub pool_code: String,
    /// The basis to use for distribution
    pub basis_id: Uuid,
    pub basis_code: String,
    /// Allocation method: 'proportional', '`fixed_percentage`', '`step_down`'
    pub allocation_method: String,
    /// Offset method: 'none', '`same_account`', '`specified_account`'
    pub offset_method: String,
    /// Offset account for the credit side of the allocation
    pub offset_account_code: Option<String>,
    /// Journal batch name prefix for generated entries
    pub journal_batch_prefix: Option<String>,
    /// Whether allocations round differences to the largest target
    pub round_to_largest: bool,
    /// Minimum allocation threshold (amounts below this are not allocated)
    pub minimum_threshold: Option<String>,
    /// Whether this rule is active
    pub is_active: bool,
    /// Effective dates
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    /// Target lines for this allocation rule
    pub target_lines: Vec<GlAllocationTargetLine>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// GL Allocation target line
/// Oracle Fusion: General Ledger > Allocations > Target Lines
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlAllocationTargetLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub rule_id: Uuid,
    pub line_number: i32,
    pub target_department_id: Option<Uuid>,
    pub target_department_name: Option<String>,
    pub target_cost_center: Option<String>,
    pub target_project_id: Option<Uuid>,
    pub target_project_name: Option<String>,
    pub target_account_code: String,
    pub target_account_name: Option<String>,
    pub fixed_percentage: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create/update GL allocation rule request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlAllocationRuleRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub pool_code: String,
    pub basis_code: String,
    #[serde(default = "default_gl_allocation_method")]
    pub allocation_method: String,
    #[serde(default = "default_gl_offset_method")]
    pub offset_method: String,
    pub offset_account_code: Option<String>,
    pub journal_batch_prefix: Option<String>,
    #[serde(default)]
    pub round_to_largest: bool,
    pub minimum_threshold: Option<String>,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub target_lines: Option<Vec<GlAllocationTargetLineRequest>>,
}

pub fn default_gl_allocation_method() -> String { "proportional".to_string() }
pub fn default_gl_offset_method() -> String { "same_account".to_string() }

/// Create/update GL allocation target line request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlAllocationTargetLineRequest {
    pub target_department_id: Option<Uuid>,
    pub target_department_name: Option<String>,
    pub target_cost_center: Option<String>,
    pub target_project_id: Option<Uuid>,
    pub target_project_name: Option<String>,
    pub target_account_code: String,
    pub target_account_name: Option<String>,
    pub fixed_percentage: Option<String>,
    pub is_active: Option<bool>,
}

/// GL Allocation run (generated journal entry batch)
/// Oracle Fusion: General Ledger > Allocations > Allocation Runs
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlAllocationRun {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub run_number: String,
    pub rule_id: Uuid,
    pub rule_code: String,
    pub rule_name: String,
    pub period_name: String,
    pub period_start_date: chrono::NaiveDate,
    pub period_end_date: chrono::NaiveDate,
    pub pool_amount: String,
    pub allocation_method: String,
    pub total_allocated: String,
    pub total_offset: String,
    pub rounding_difference: String,
    pub target_count: i32,
    pub journal_batch_id: Option<Uuid>,
    pub journal_batch_name: Option<String>,
    pub status: String,
    pub run_date: chrono::NaiveDate,
    pub posted_at: Option<DateTime<Utc>>,
    pub reversed_at: Option<DateTime<Utc>>,
    pub posted_by: Option<Uuid>,
    pub results: Vec<GlAllocationRunLine>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Individual line in a GL allocation run
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlAllocationRunLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub run_id: Uuid,
    pub line_number: i32,
    pub target_department_id: Option<Uuid>,
    pub target_department_name: Option<String>,
    pub target_cost_center: Option<String>,
    pub target_project_id: Option<Uuid>,
    pub target_project_name: Option<String>,
    pub target_account_code: String,
    pub target_account_name: Option<String>,
    pub source_account_code: Option<String>,
    pub basis_amount: String,
    pub basis_percentage: String,
    pub allocated_amount: String,
    pub offset_amount: String,
    pub line_type: String,
    pub journal_line_id: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// GL Allocation run request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlAllocationRunRequest {
    pub rule_code: String,
    pub period_name: String,
    pub period_start_date: chrono::NaiveDate,
    pub period_end_date: chrono::NaiveDate,
    pub run_date: Option<chrono::NaiveDate>,
    pub pool_amount_override: Option<String>,
}

/// GL Allocation dashboard summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlAllocationDashboardSummary {
    pub total_pools: i32,
    pub active_pools: i32,
    pub total_bases: i32,
    pub active_bases: i32,
    pub total_rules: i32,
    pub active_rules: i32,
    pub total_runs: i32,
    pub posted_runs: i32,
    pub draft_runs: i32,
    pub total_allocated_amount: String,
    pub pools_by_type: serde_json::Value,
    pub rules_by_method: serde_json::Value,
}

// ============================================================================
// AutoInvoice (Oracle Fusion Receivables AutoInvoice)
// ============================================================================

/// `AutoInvoice` grouping rule definition.
/// Oracle Fusion: Receivables > `AutoInvoice` > Grouping Rules
/// Controls how imported transaction lines are grouped into invoices.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoInvoiceGroupingRule {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    /// Transaction types this rule applies to (e.g., ["invoice", "`credit_memo`"])
    pub transaction_types: serde_json::Value,
    /// Fields to group by (e.g., ["`bill_to_customer_id`", "`currency_code`"])
    pub group_by_fields: serde_json::Value,
    /// Line ordering fields (e.g., ["`line_number`", "`item_code`"])
    pub line_order_by: serde_json::Value,
    /// Whether this is the default grouping rule
    pub is_default: bool,
    pub is_active: bool,
    pub priority: i32,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// `AutoInvoice` validation rule.
/// Oracle Fusion: Receivables > `AutoInvoice` > Validation Rules
/// Validates transaction lines before invoice creation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoInvoiceValidationRule {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    /// The field being validated
    pub field_name: String,
    /// Validation type: "required", "format", "reference", "range", "custom"
    pub validation_type: String,
    /// Expression or value for the validation
    pub validation_expression: Option<String>,
    /// Error message when validation fails
    pub error_message: String,
    /// Whether to reject the entire line on failure (vs. flag as warning)
    pub is_fatal: bool,
    /// Transaction types this rule applies to
    pub transaction_types: serde_json::Value,
    pub is_active: bool,
    pub priority: i32,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// `AutoInvoice` import batch (header for a batch of transaction lines being imported).
/// Oracle Fusion: Receivables > `AutoInvoice` > Import
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoInvoiceBatch {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub batch_number: String,
    pub batch_source: String,
    pub description: Option<String>,
    /// 'pending', 'validating', 'validated', 'processing', 'completed', 'failed', 'cancelled'
    pub status: String,
    pub total_lines: i32,
    pub valid_lines: i32,
    pub invalid_lines: i32,
    pub invoices_created: i32,
    pub invoices_total_amount: String,
    pub grouping_rule_id: Option<Uuid>,
    pub validation_errors: serde_json::Value,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// `AutoInvoice` transaction line (raw line being imported).
/// Oracle Fusion: Receivables > `AutoInvoice` > Interface Lines
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoInvoiceLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub batch_id: Uuid,
    pub line_number: i32,
    /// Source system identifier
    pub source_line_id: Option<String>,
    /// Transaction type: "invoice", "`credit_memo`", "`debit_memo`", "`on_account_credit`"
    pub transaction_type: String,
    pub customer_id: Option<Uuid>,
    pub customer_number: Option<String>,
    pub customer_name: Option<String>,
    pub bill_to_customer_id: Option<Uuid>,
    pub bill_to_site_id: Option<Uuid>,
    pub ship_to_customer_id: Option<Uuid>,
    pub ship_to_site_id: Option<Uuid>,
    pub item_code: Option<String>,
    pub item_description: Option<String>,
    pub quantity: Option<String>,
    pub unit_of_measure: Option<String>,
    pub unit_price: String,
    pub line_amount: String,
    pub currency_code: String,
    pub exchange_rate: Option<String>,
    pub transaction_date: chrono::NaiveDate,
    pub gl_date: chrono::NaiveDate,
    pub due_date: Option<chrono::NaiveDate>,
    pub revenue_account_code: Option<String>,
    pub receivable_account_code: Option<String>,
    pub tax_code: Option<String>,
    pub tax_amount: Option<String>,
    pub sales_rep_id: Option<Uuid>,
    pub sales_rep_name: Option<String>,
    pub memo_line: Option<String>,
    pub reference_number: Option<String>,
    pub sales_order_number: Option<String>,
    pub sales_order_line: Option<String>,
    /// 'pending', 'valid', 'invalid', 'grouped', 'error'
    pub status: String,
    pub validation_errors: serde_json::Value,
    /// Assigned invoice ID after grouping
    pub invoice_id: Option<Uuid>,
    pub invoice_line_number: Option<i32>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// `AutoInvoice` result - the generated AR invoice.
/// Oracle Fusion: Receivables > Transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoInvoiceResult {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub batch_id: Uuid,
    pub invoice_number: String,
    pub transaction_type: String,
    pub customer_id: Option<Uuid>,
    pub bill_to_customer_id: Option<Uuid>,
    pub bill_to_site_id: Option<Uuid>,
    pub ship_to_customer_id: Option<Uuid>,
    pub ship_to_site_id: Option<Uuid>,
    pub currency_code: String,
    pub exchange_rate: Option<String>,
    pub transaction_date: chrono::NaiveDate,
    pub gl_date: chrono::NaiveDate,
    pub due_date: Option<chrono::NaiveDate>,
    pub subtotal: String,
    pub tax_amount: String,
    pub total_amount: String,
    pub line_count: i32,
    pub receivable_account_code: Option<String>,
    pub sales_rep_id: Option<Uuid>,
    pub sales_order_number: Option<String>,
    pub reference_number: Option<String>,
    /// 'draft', 'complete', 'posted', 'cancelled'
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// `AutoInvoice` result line
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoInvoiceResultLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub invoice_id: Uuid,
    pub line_number: i32,
    pub source_line_id: Option<String>,
    pub item_code: Option<String>,
    pub item_description: Option<String>,
    pub quantity: Option<String>,
    pub unit_of_measure: Option<String>,
    pub unit_price: String,
    pub line_amount: String,
    pub tax_code: Option<String>,
    pub tax_amount: Option<String>,
    pub revenue_account_code: Option<String>,
    pub sales_order_number: Option<String>,
    pub sales_order_line: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// `AutoInvoice` import request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoInvoiceImportRequest {
    pub batch_source: String,
    pub description: Option<String>,
    pub lines: Vec<AutoInvoiceLineRequest>,
    pub grouping_rule_id: Option<Uuid>,
}

/// Single line in an `AutoInvoice` import
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoInvoiceLineRequest {
    pub source_line_id: Option<String>,
    pub transaction_type: Option<String>,
    pub customer_id: Option<Uuid>,
    pub customer_number: Option<String>,
    pub customer_name: Option<String>,
    pub bill_to_customer_id: Option<Uuid>,
    pub bill_to_site_id: Option<Uuid>,
    pub ship_to_customer_id: Option<Uuid>,
    pub ship_to_site_id: Option<Uuid>,
    pub item_code: Option<String>,
    pub item_description: Option<String>,
    pub quantity: Option<String>,
    pub unit_of_measure: Option<String>,
    pub unit_price: Option<String>,
    pub line_amount: Option<String>,
    #[serde(default = "default_currency_usd")]
    pub currency_code: String,
    pub exchange_rate: Option<String>,
    pub transaction_date: Option<chrono::NaiveDate>,
    pub gl_date: Option<chrono::NaiveDate>,
    pub due_date: Option<chrono::NaiveDate>,
    pub revenue_account_code: Option<String>,
    pub receivable_account_code: Option<String>,
    pub tax_code: Option<String>,
    pub tax_amount: Option<String>,
    pub sales_rep_id: Option<Uuid>,
    pub sales_rep_name: Option<String>,
    pub memo_line: Option<String>,
    pub reference_number: Option<String>,
    pub sales_order_number: Option<String>,
    pub sales_order_line: Option<String>,
}

/// `AutoInvoice` processing summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoInvoiceSummary {
    pub total_batches: i32,
    pub pending_batches: i32,
    pub completed_batches: i32,
    pub failed_batches: i32,
    pub total_lines_imported: i32,
    pub total_invoices_created: i32,
    pub total_invoice_amount: String,
}

/// Validation error for an `AutoInvoice` line
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoInvoiceValidationError {
    pub line_number: i32,
    pub field_name: String,
    pub validation_rule: String,
    pub error_message: String,
    pub is_fatal: bool,
}

// ============================================================================
// KPI & Embedded Analytics (Oracle Fusion OTBI-inspired)
// ============================================================================

/// KPI definition
/// Oracle Fusion: Analytics > KPI Library
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KpiDefinition {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub category: String,
    pub unit_of_measure: String,
    pub direction: String,        // "higher_is_better", "lower_is_better", "target_range"
    pub target_value: String,
    pub warning_threshold: Option<String>,
    pub critical_threshold: Option<String>,
    pub data_source_query: Option<String>,
    pub evaluation_frequency: String, // "manual", "hourly", "daily", "weekly", "monthly"
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// KPI data point (time-series value)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KpiDataPoint {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub kpi_id: Uuid,
    pub value: String,
    pub recorded_at: DateTime<Utc>,
    pub period_start: Option<chrono::NaiveDate>,
    pub period_end: Option<chrono::NaiveDate>,
    pub status: String, // "on_track", "warning", "critical", "no_target"
    pub notes: Option<String>,
    pub recorded_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

/// Dashboard definition
/// Oracle Fusion: Analytics > Dashboards
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Dashboard {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub owner_id: Option<Uuid>,
    pub is_shared: bool,
    pub is_default: bool,
    pub layout_config: serde_json::Value,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Dashboard widget (links a KPI to a dashboard with display config)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardWidget {
    pub id: Uuid,
    pub dashboard_id: Uuid,
    pub kpi_id: Option<Uuid>,
    pub widget_type: String, // "kpi_card", "chart", "table", "gauge", "trend"
    pub title: String,
    pub position_row: i32,
    pub position_col: i32,
    pub width: i32,
    pub height: i32,
    pub display_config: serde_json::Value,
    pub is_visible: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// KPI Analytics Dashboard summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KpiDashboardSummary {
    pub total_kpis: i32,
    pub active_kpis: i32,
    pub on_track: i32,
    pub warning: i32,
    pub critical: i32,
    pub no_data: i32,
    pub total_dashboards: i32,
    pub kpis_by_category: serde_json::Value,
    pub recent_values: serde_json::Value,
}

// ============================================================================
// Contract Lifecycle Management (Oracle Fusion Enterprise Contracts)
// ============================================================================

/// CLM Contract Type
/// Oracle Fusion: Enterprise Contracts > Contract Types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClmContractType {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub contract_category: String,
    pub default_duration_days: Option<i32>,
    pub requires_approval: bool,
    pub is_auto_renew: bool,
    pub risk_scoring_enabled: bool,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// CLM Clause (reusable contract clause)
/// Oracle Fusion: Enterprise Contracts > Clause Library
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClmClause {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub title: String,
    pub body: String,
    pub clause_type: String,
    pub clause_category: String,
    pub applicability: String,
    pub is_locked: bool,
    pub version: i32,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// CLM Template
/// Oracle Fusion: Enterprise Contracts > Contract Templates
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClmTemplate {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub contract_type_id: Option<Uuid>,
    pub default_currency: String,
    pub default_duration_days: Option<i32>,
    pub terms_and_conditions: Option<String>,
    pub is_standard: bool,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// CLM Template Clause
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClmTemplateClause {
    pub id: Uuid,
    pub template_id: Uuid,
    pub clause_id: Uuid,
    pub section: Option<String>,
    pub display_order: i32,
    pub is_required: bool,
    pub created_at: DateTime<Utc>,
}

/// CLM Contract
/// Oracle Fusion: Enterprise Contracts > Contracts
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClmContract {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub contract_number: String,
    pub title: String,
    pub description: Option<String>,
    pub contract_type_id: Option<Uuid>,
    pub template_id: Option<Uuid>,
    pub contract_category: String,
    pub currency: String,
    pub total_value: String,
    pub start_date: Option<chrono::NaiveDate>,
    pub end_date: Option<chrono::NaiveDate>,
    pub status: String,
    pub priority: String,
    pub risk_score: Option<i32>,
    pub risk_level: Option<String>,
    pub parent_contract_id: Option<Uuid>,
    pub renewal_type: String,
    pub auto_renew_months: Option<i32>,
    pub renewal_notice_days: i32,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub approved_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// CLM Contract Party
/// Oracle Fusion: Enterprise Contracts > Contract Parties
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClmContractParty {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub contract_id: Uuid,
    pub party_type: String,
    pub party_role: String,
    pub party_name: String,
    pub contact_name: Option<String>,
    pub contact_email: Option<String>,
    pub contact_phone: Option<String>,
    pub entity_reference: Option<String>,
    pub is_primary: bool,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// CLM Contract Clause (instance in a contract)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClmContractClause {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub contract_id: Uuid,
    pub clause_id: Option<Uuid>,
    pub section: Option<String>,
    pub title: String,
    pub body: String,
    pub clause_type: String,
    pub display_order: i32,
    pub is_modified: bool,
    pub original_body: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// CLM Contract Milestone
/// Oracle Fusion: Enterprise Contracts > Milestones
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClmMilestone {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub contract_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub milestone_type: String,
    pub due_date: Option<chrono::NaiveDate>,
    pub completed_date: Option<chrono::NaiveDate>,
    pub amount: Option<String>,
    pub currency: String,
    pub status: String,
    pub responsible_party_id: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// CLM Contract Deliverable
/// Oracle Fusion: Enterprise Contracts > Deliverables
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClmDeliverable {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub contract_id: Uuid,
    pub milestone_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub deliverable_type: String,
    pub quantity: String,
    pub unit_of_measure: String,
    pub due_date: Option<chrono::NaiveDate>,
    pub completed_date: Option<chrono::NaiveDate>,
    pub acceptance_date: Option<chrono::NaiveDate>,
    pub amount: Option<String>,
    pub currency: String,
    pub status: String,
    pub accepted_by: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// CLM Contract Amendment
/// Oracle Fusion: Enterprise Contracts > Amendments
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClmAmendment {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub contract_id: Uuid,
    pub amendment_number: String,
    pub title: String,
    pub description: Option<String>,
    pub amendment_type: String,
    pub previous_value: Option<String>,
    pub new_value: Option<String>,
    pub effective_date: Option<chrono::NaiveDate>,
    pub status: String,
    pub approved_by: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// CLM Contract Risk Assessment
/// Oracle Fusion: Enterprise Contracts > Risk Management
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClmRisk {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub contract_id: Uuid,
    pub risk_category: String,
    pub risk_description: String,
    pub probability: String,
    pub impact: String,
    pub mitigation_strategy: Option<String>,
    pub residual_risk: Option<String>,
    pub owner_id: Option<Uuid>,
    pub status: String,
    pub metadata: serde_json::Value,
    pub assessed_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// CLM Dashboard Summary
/// Oracle Fusion: Enterprise Contracts > Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClmDashboard {
    pub total_contracts: i32,
    pub active_contracts: i32,
    pub draft_contracts: i32,
    pub expiring_contracts: i32,
    pub total_contract_value: String,
    pub contracts_by_category: serde_json::Value,
    pub contracts_by_status: serde_json::Value,
    pub high_risk_contracts: i32,
    pub pending_milestones: i32,
    pub pending_deliverables: i32,
    pub pending_amendments: i32,
    pub recent_contracts: serde_json::Value,
}

// ============================================================================
// Advanced Financial Controls (Oracle Fusion Advanced Controls)
// ============================================================================

/// A monitoring rule that detects anomalies or policy violations.
/// Oracle Fusion equivalent: Advanced Controls > Continuous Monitoring > Monitor Rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlMonitorRule {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    /// Control category: "transaction", "access", "`master_data`", "`period_close`", "`master_record`"
    pub category: String,
    /// Risk level: "critical", "high", "medium", "low"
    pub risk_level: String,
    /// The control type: "threshold", "pattern", "frequency", "segregation", "approval", "custom"
    pub control_type: String,
    /// Conditions defining when to trigger the rule (JSON)
    pub conditions: serde_json::Value,
    /// Threshold value for threshold-based controls
    pub threshold_value: Option<String>,
    /// The entity/table being monitored
    pub target_entity: String,
    /// Fields to check
    pub target_fields: serde_json::Value,
    /// Actions to take on violation: "alert", "block", "escalate", "review"
    pub actions: serde_json::Value,
    /// Whether to auto-resolve when conditions are no longer met
    pub auto_resolve: bool,
    /// Schedule for periodic checks: "realtime", "daily", "weekly", "monthly"
    pub check_schedule: String,
    pub is_active: bool,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub last_check_at: Option<chrono::DateTime<Utc>>,
    pub last_violation_at: Option<chrono::DateTime<Utc>>,
    pub total_violations: i32,
    pub total_resolved: i32,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

/// A specific violation instance detected by a control rule.
/// Oracle Fusion equivalent: Advanced Controls > Violations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlViolation {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub rule_id: Uuid,
    pub rule_code: Option<String>,
    pub rule_name: Option<String>,
    pub violation_number: String,
    /// The entity type that triggered the violation
    pub entity_type: String,
    /// The specific record ID
    pub entity_id: Option<Uuid>,
    /// Description of the violation
    pub description: String,
    /// Detailed findings (JSON)
    pub findings: serde_json::Value,
    /// Risk level of this specific violation
    pub risk_level: String,
    /// "open", "`under_review`", "resolved", "`false_positive`", "escalated", "waived"
    pub status: String,
    /// Who is assigned to review
    pub assigned_to: Option<Uuid>,
    pub assigned_to_name: Option<String>,
    /// Resolution details
    pub resolution_notes: Option<String>,
    pub resolved_by: Option<Uuid>,
    pub resolved_at: Option<chrono::DateTime<Utc>>,
    /// For escalated violations
    pub escalated_to: Option<Uuid>,
    pub escalated_at: Option<chrono::DateTime<Utc>>,
    /// Related transaction references
    pub related_entities: serde_json::Value,
    pub detected_at: chrono::DateTime<Utc>,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

/// Dashboard summary for Advanced Financial Controls
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialControlsDashboardSummary {
    pub total_rules: i32,
    pub active_rules: i32,
    pub total_violations: i32,
    pub open_violations: i32,
    pub resolved_violations: i32,
    pub escalated_violations: i32,
    pub false_positive_violations: i32,
    pub critical_violations: i32,
    pub high_violations: i32,
    pub medium_violations: i32,
    pub low_violations: i32,
    pub violations_by_category: serde_json::Value,
    pub violations_by_rule: serde_json::Value,
    pub avg_resolution_time_hours: Option<f64>,
}


// ============================================================================
// Distribution Set Types (Oracle Fusion: Payables > Distribution Sets)
// ============================================================================

/// Distribution Set header
/// Oracle Fusion equivalent: Financials > Payables > Setup > Distribution Sets
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DistributionSet {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub set_code: String,
    pub set_name: String,
    pub description: Option<String>,
    pub distribution_type: String,
    pub currency_code: String,
    pub total_percentage: String,
    pub total_amount: Option<String>,
    pub status: String,
    pub is_default: bool,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub usage_count: i32,
    pub last_used_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// Distribution Set Line
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DistributionSetLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub distribution_set_id: Uuid,
    pub line_number: i32,
    pub account_combination: String,
    pub account_description: Option<String>,
    pub segment1: Option<String>,
    pub segment2: Option<String>,
    pub segment3: Option<String>,
    pub segment4: Option<String>,
    pub segment5: Option<String>,
    pub percentage: String,
    pub amount: Option<String>,
    pub description: Option<String>,
    pub cost_center: Option<String>,
    pub department: Option<String>,
    pub project_code: Option<String>,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// Distribution Set Usage Log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DistributionSetUsage {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub distribution_set_id: Uuid,
    pub set_code: String,
    pub target_entity_type: String,
    pub target_entity_id: Uuid,
    pub target_entity_number: Option<String>,
    pub applied_by: Option<Uuid>,
    pub applied_at: DateTime<Utc>,
    pub line_count: i32,
    pub total_amount: Option<String>,
    pub metadata: serde_json::Value,
}

/// Distribution Set Dashboard summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DistributionSetDashboard {
    pub total_sets: i32,
    pub active_sets: i32,
    pub inactive_sets: i32,
    pub default_sets: i32,
    pub total_usages: i32,
    pub percentage_sets: i32,
    pub amount_sets: i32,
    pub avg_usage_per_set: Option<String>,
}

