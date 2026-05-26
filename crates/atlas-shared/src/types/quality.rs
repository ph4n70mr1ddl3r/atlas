use serde::{Deserialize, Serialize};
use uuid::Uuid;
// ============================================================================
// Quality Management
// Oracle Fusion Cloud: Quality Management
//
// Quality inspection plans, inspections, non-conformance reports (NCRs),
// corrective & preventive actions (CAPA), quality holds, and dashboards.
// ============================================================================

/// Quality Inspection Plan - defines what to inspect, how, and when
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityInspectionPlan {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub plan_code: String,
    pub name: String,
    pub description: Option<String>,
    pub plan_type: String,
    pub item_id: Option<Uuid>,
    pub item_code: Option<String>,
    pub supplier_id: Option<Uuid>,
    pub supplier_name: Option<String>,
    pub inspection_trigger: String,
    pub sampling_method: String,
    pub sample_size_percent: String,
    pub accept_number: Option<i32>,
    pub reject_number: Option<i32>,
    pub frequency: String,
    pub is_active: bool,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub total_criteria: i32,
    pub total_inspections: i32,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Quality Inspection Plan Criterion - a specific check within a plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityInspectionPlanCriterion {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub plan_id: Uuid,
    pub criterion_number: i32,
    pub name: String,
    pub description: Option<String>,
    pub characteristic: String,
    pub measurement_type: String,
    pub target_value: String,
    pub lower_spec_limit: String,
    pub upper_spec_limit: String,
    pub unit_of_measure: Option<String>,
    pub is_mandatory: bool,
    pub weight: String,
    pub criticality: String,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Quality Inspection - an instance of executing an inspection plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityInspection {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub inspection_number: String,
    pub plan_id: Uuid,
    pub source_type: String,
    pub source_id: Option<Uuid>,
    pub source_number: Option<String>,
    pub item_id: Option<Uuid>,
    pub item_code: Option<String>,
    pub item_description: Option<String>,
    pub lot_number: Option<String>,
    pub quantity_inspected: String,
    pub quantity_accepted: String,
    pub quantity_rejected: String,
    pub unit_of_measure: Option<String>,
    pub status: String,
    pub verdict: String,
    pub overall_score: String,
    pub notes: Option<String>,
    pub inspector_id: Option<Uuid>,
    pub inspector_name: Option<String>,
    pub inspection_date: chrono::NaiveDate,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Quality Inspection Result - individual criterion result within an inspection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityInspectionResult {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub inspection_id: Uuid,
    pub criterion_id: Option<Uuid>,
    pub criterion_name: String,
    pub characteristic: String,
    pub measurement_type: String,
    pub observed_value: String,
    pub target_value: String,
    pub lower_spec_limit: String,
    pub upper_spec_limit: String,
    pub unit_of_measure: Option<String>,
    pub result_status: String,
    pub deviation: String,
    pub notes: Option<String>,
    pub evaluated_by: Option<Uuid>,
    pub evaluated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Non-Conformance Report (NCR)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NonConformanceReport {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub ncr_number: String,
    pub title: String,
    pub description: Option<String>,
    pub ncr_type: String,
    pub severity: String,
    pub origin: String,
    pub source_type: Option<String>,
    pub source_id: Option<Uuid>,
    pub source_number: Option<String>,
    pub item_id: Option<Uuid>,
    pub item_code: Option<String>,
    pub supplier_id: Option<Uuid>,
    pub supplier_name: Option<String>,
    pub detected_date: chrono::NaiveDate,
    pub detected_by: Option<String>,
    pub responsible_party: Option<String>,
    pub status: String,
    pub resolution_description: Option<String>,
    pub resolution_type: Option<String>,
    pub resolved_by: Option<String>,
    pub resolved_at: Option<chrono::DateTime<chrono::Utc>>,
    pub total_corrective_actions: i32,
    pub open_corrective_actions: i32,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Corrective & Preventive Action (CAPA)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrectiveAction {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub ncr_id: Uuid,
    pub action_number: String,
    pub action_type: String,
    pub title: String,
    pub description: Option<String>,
    pub root_cause: Option<String>,
    pub corrective_action_desc: Option<String>,
    pub preventive_action_desc: Option<String>,
    pub assigned_to: Option<String>,
    pub due_date: Option<chrono::NaiveDate>,
    pub status: String,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub effectiveness_rating: Option<i32>,
    pub priority: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Quality Hold - prevents items/lots from being used until resolved
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityHold {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub hold_number: String,
    pub reason: String,
    pub description: Option<String>,
    pub item_id: Option<Uuid>,
    pub item_code: Option<String>,
    pub lot_number: Option<String>,
    pub supplier_id: Option<Uuid>,
    pub supplier_name: Option<String>,
    pub source_type: Option<String>,
    pub source_id: Option<Uuid>,
    pub source_number: Option<String>,
    pub hold_type: String,
    pub status: String,
    pub released_by: Option<Uuid>,
    pub released_at: Option<chrono::DateTime<chrono::Utc>>,
    pub release_notes: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Quality Management Dashboard Summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityDashboardSummary {
    pub total_active_plans: i32,
    pub total_pending_inspections: i32,
    pub total_passed_inspections: i32,
    pub total_failed_inspections: i32,
    pub inspection_pass_rate_percent: String,
    pub total_open_ncrs: i32,
    pub total_ncrs: i32,
    pub critical_ncrs: i32,
    pub total_open_corrective_actions: i32,
    pub total_completed_corrective_actions: i32,
    pub corrective_action_completion_rate_percent: String,
    pub total_active_holds: i32,
    pub inspections_by_verdict: serde_json::Value,
    pub ncrs_by_severity: serde_json::Value,
    pub ncrs_by_type: serde_json::Value,
}

