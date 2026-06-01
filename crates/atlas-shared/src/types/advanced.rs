use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
// ============================================================================
// Manufacturing Execution (Oracle Fusion SCM > Manufacturing)
// ============================================================================

/// Work Definition (BOM + Routing template)
/// Oracle Fusion equivalent: Manufacturing > Work Definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkDefinition {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub definition_number: String,
    pub description: Option<String>,
    pub item_id: Option<Uuid>,
    pub item_code: Option<String>,
    pub item_description: Option<String>,
    pub version: i32,
    pub status: String,
    pub production_type: String,
    pub planning_type: String,
    pub standard_lot_size: String,
    pub unit_of_measure: String,
    pub lead_time_days: i32,
    pub cost_type: String,
    pub standard_cost: String,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Work Definition Component (BOM line)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkDefinitionComponent {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub work_definition_id: Uuid,
    pub line_number: i32,
    pub component_item_id: Option<Uuid>,
    pub component_item_code: String,
    pub component_item_description: Option<String>,
    pub quantity_required: String,
    pub unit_of_measure: String,
    pub component_type: String,
    pub scrap_percent: String,
    pub yield_percent: String,
    pub supply_type: String,
    pub supply_subinventory: Option<String>,
    pub wip_supply_type: String,
    pub operation_sequence: Option<i32>,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Work Definition Operation (Routing step)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkDefinitionOperation {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub work_definition_id: Uuid,
    pub operation_sequence: i32,
    pub operation_name: String,
    pub operation_description: Option<String>,
    pub work_center_code: Option<String>,
    pub work_center_name: Option<String>,
    pub department_code: Option<String>,
    pub setup_hours: String,
    pub run_time_hours: String,
    pub run_time_unit: String,
    pub units_per_run: String,
    pub resource_code: Option<String>,
    pub resource_type: String,
    pub resource_count: i32,
    pub standard_labor_cost: String,
    pub standard_overhead_cost: String,
    pub standard_machine_cost: String,
    pub operation_type: String,
    pub backflush_enabled: bool,
    pub count_point_type: String,
    pub yield_percent: String,
    pub scrap_percent: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Work Order (Production Order)
/// Oracle Fusion equivalent: Manufacturing > Work Orders
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkOrder {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub work_order_number: String,
    pub description: Option<String>,
    pub work_definition_id: Option<Uuid>,
    pub item_id: Option<Uuid>,
    pub item_code: Option<String>,
    pub item_description: Option<String>,
    pub quantity_ordered: String,
    pub quantity_completed: String,
    pub quantity_scrapped: String,
    pub quantity_in_queue: String,
    pub quantity_running: String,
    pub quantity_rejected: String,
    pub unit_of_measure: String,
    pub scheduled_start_date: Option<chrono::NaiveDate>,
    pub scheduled_completion_date: Option<chrono::NaiveDate>,
    pub actual_start_date: Option<chrono::NaiveDate>,
    pub actual_completion_date: Option<chrono::NaiveDate>,
    pub due_date: Option<chrono::NaiveDate>,
    pub status: String,
    pub priority: String,
    pub production_line: Option<String>,
    pub work_center_code: Option<String>,
    pub warehouse_code: Option<String>,
    pub cost_type: String,
    pub estimated_material_cost: String,
    pub estimated_labor_cost: String,
    pub estimated_overhead_cost: String,
    pub estimated_total_cost: String,
    pub actual_material_cost: String,
    pub actual_labor_cost: String,
    pub actual_overhead_cost: String,
    pub actual_total_cost: String,
    pub source_type: Option<String>,
    pub source_document_number: Option<String>,
    pub source_document_line_id: Option<Uuid>,
    pub firm_planned: bool,
    pub company_id: Option<Uuid>,
    pub plant_code: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub released_at: Option<DateTime<Utc>>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub closed_at: Option<DateTime<Utc>>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub cancellation_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Work Order Operation (tracking per-operation progress)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkOrderOperation {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub work_order_id: Uuid,
    pub operation_sequence: i32,
    pub operation_name: String,
    pub work_center_code: Option<String>,
    pub work_center_name: Option<String>,
    pub department_code: Option<String>,
    pub quantity_in_queue: String,
    pub quantity_running: String,
    pub quantity_completed: String,
    pub quantity_rejected: String,
    pub quantity_scrapped: String,
    pub scheduled_start_date: Option<chrono::NaiveDate>,
    pub scheduled_completion_date: Option<chrono::NaiveDate>,
    pub actual_start_date: Option<chrono::NaiveDate>,
    pub actual_completion_date: Option<chrono::NaiveDate>,
    pub actual_setup_hours: String,
    pub actual_run_hours: String,
    pub resource_code: Option<String>,
    pub resource_type: String,
    pub status: String,
    pub actual_labor_cost: String,
    pub actual_overhead_cost: String,
    pub actual_machine_cost: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Work Order Material Requirement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkOrderMaterial {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub work_order_id: Uuid,
    pub operation_sequence: Option<i32>,
    pub component_item_id: Option<Uuid>,
    pub component_item_code: String,
    pub component_item_description: Option<String>,
    pub quantity_required: String,
    pub quantity_issued: String,
    pub quantity_returned: String,
    pub quantity_scrapped: String,
    pub unit_of_measure: String,
    pub supply_type: String,
    pub supply_subinventory: Option<String>,
    pub wip_supply_type: String,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create Work Definition Request
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreateWorkDefinitionRequest {
    pub definition_number: Option<String>,
    pub description: Option<String>,
    pub item_id: Option<Uuid>,
    pub item_code: Option<String>,
    pub item_description: Option<String>,
    pub production_type: Option<String>,
    pub planning_type: Option<String>,
    pub standard_lot_size: Option<String>,
    pub unit_of_measure: Option<String>,
    pub lead_time_days: Option<i32>,
    pub cost_type: Option<String>,
    pub standard_cost: Option<String>,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub created_by: Option<Uuid>,
}

/// Create Work Order Request
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreateWorkOrderRequest {
    pub work_definition_id: Option<Uuid>,
    pub description: Option<String>,
    pub item_id: Option<Uuid>,
    pub item_code: Option<String>,
    pub item_description: Option<String>,
    pub quantity_ordered: String,
    pub unit_of_measure: Option<String>,
    pub scheduled_start_date: Option<chrono::NaiveDate>,
    pub scheduled_completion_date: Option<chrono::NaiveDate>,
    pub due_date: Option<chrono::NaiveDate>,
    pub priority: Option<String>,
    pub production_line: Option<String>,
    pub work_center_code: Option<String>,
    pub warehouse_code: Option<String>,
    pub cost_type: Option<String>,
    pub source_type: Option<String>,
    pub source_document_number: Option<String>,
    pub firm_planned: Option<bool>,
    pub company_id: Option<Uuid>,
    pub plant_code: Option<String>,
    pub created_by: Option<Uuid>,
}

/// Report Production Completion Request
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReportCompletionRequest {
    pub operation_sequence: Option<i32>,
    pub quantity_completed: String,
    pub quantity_scrapped: String,
    pub actual_run_hours: Option<String>,
    pub actual_labor_cost: Option<String>,
    pub actual_overhead_cost: Option<String>,
    pub completed_by: Option<Uuid>,
}

/// Issue Materials Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueMaterialRequest {
    pub material_id: Uuid,
    pub quantity_issued: String,
}

/// Manufacturing Dashboard Summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManufacturingDashboard {
    pub total_work_orders: i32,
    pub open_work_orders: i32,
    pub in_progress_work_orders: i32,
    pub completed_work_orders: i32,
    pub cancelled_work_orders: i32,
    pub total_definitions: i32,
    pub active_definitions: i32,
    pub overdue_orders: i32,
    pub total_estimated_cost: String,
    pub total_actual_cost: String,
    pub cost_variance_pct: String,
    pub orders_by_status: serde_json::Value,
    pub orders_by_priority: serde_json::Value,
    pub completion_rate_pct: String,
    pub on_time_completion_pct: String,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Warehouse Management (Oracle Fusion Cloud Warehouse Management)
// ═══════════════════════════════════════════════════════════════════════════════

/// Warehouse definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Warehouse {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub location_code: Option<String>,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Warehouse zone (e.g., receiving, bulk storage, picking, packing, staging)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WarehouseZone {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub warehouse_id: Uuid,
    pub code: String,
    pub name: String,
    pub zone_type: String, // receiving, storage, picking, packing, staging, shipping
    pub description: Option<String>,
    pub aisle_count: Option<i32>,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Put-away rule configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PutAwayRule {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub warehouse_id: Uuid,
    pub rule_name: String,
    pub description: Option<String>,
    pub priority: i32,
    /// Item category filter (optional - null means all categories)
    pub item_category: Option<String>,
    /// Zone type to route to
    pub target_zone_type: String,
    /// Strategy: closest, `zone_rotation`, `fixed_location`
    pub strategy: String,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Warehouse task types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WarehouseTask {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub warehouse_id: Uuid,
    pub task_number: String,
    pub task_type: String, // pick, pack, put_away, load, receive
    pub status: String,    // pending, in_progress, completed, cancelled
    pub priority: String,  // low, medium, high, urgent
    pub source_document: Option<String>,
    pub source_document_id: Option<Uuid>,
    pub source_line_id: Option<Uuid>,
    pub item_id: Option<Uuid>,
    pub item_description: Option<String>,
    pub from_zone_id: Option<Uuid>,
    pub to_zone_id: Option<Uuid>,
    pub from_location: Option<String>,
    pub to_location: Option<String>,
    pub quantity: Option<String>,
    pub uom: Option<String>,
    pub assigned_to: Option<Uuid>,
    pub wave_id: Option<Uuid>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Pick wave for grouping picking tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PickWave {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub warehouse_id: Uuid,
    pub wave_number: String,
    pub status: String, // draft, released, in_progress, completed, cancelled
    pub priority: String,
    pub cut_off_date: Option<chrono::NaiveDate>,
    pub shipping_method: Option<String>,
    pub total_tasks: i32,
    pub completed_tasks: i32,
    pub released_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Warehouse dashboard summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WarehouseDashboard {
    pub total_warehouses: i64,
    pub active_warehouses: i64,
    pub total_zones: i64,
    pub total_pending_tasks: i64,
    pub total_in_progress_tasks: i64,
    pub total_completed_tasks_today: i64,
    pub total_active_waves: i64,
    pub tasks_by_type: serde_json::Value,
    pub tasks_by_priority: serde_json::Value,
    pub wave_completion_pct: String,
    pub recent_tasks: Vec<WarehouseTask>,
}

// ═══════════════════════════════════════════════════════════════════════
// Absence Management (Oracle Fusion Cloud HCM Absence Management)
// ═══════════════════════════════════════════════════════════════════════

/// Absence type definition (e.g., vacation, sick, parental)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AbsenceType {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub category: String,
    pub plan_type: String,
    pub requires_approval: bool,
    pub requires_documentation: bool,
    pub auto_approve_below_days: String,
    pub allow_negative_balance: bool,
    pub allow_half_day: bool,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Absence plan with accrual rules
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AbsencePlan {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub absence_type_id: Uuid,
    pub accrual_frequency: String,
    pub accrual_rate: String,
    pub accrual_unit: String,
    pub carry_over_max: Option<String>,
    pub carry_over_expiry_months: Option<i32>,
    pub max_balance: Option<String>,
    pub probation_period_days: i32,
    pub prorate_first_year: bool,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Employee absence balance for a given period
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AbsenceBalance {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub employee_id: Uuid,
    pub plan_id: Uuid,
    pub period_start: chrono::NaiveDate,
    pub period_end: chrono::NaiveDate,
    pub accrued: String,
    pub taken: String,
    pub adjusted: String,
    pub carried_over: String,
    pub remaining: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Individual absence entry (leave request/record)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AbsenceEntry {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub employee_id: Uuid,
    pub employee_name: Option<String>,
    pub absence_type_id: Uuid,
    pub plan_id: Option<Uuid>,
    pub entry_number: String,
    pub status: String,
    pub start_date: chrono::NaiveDate,
    pub end_date: chrono::NaiveDate,
    pub duration_days: String,
    pub duration_hours: Option<String>,
    pub is_half_day: bool,
    pub half_day_period: Option<String>,
    pub reason: Option<String>,
    pub comments: Option<String>,
    pub documentation_provided: bool,
    pub submitted_at: Option<DateTime<Utc>>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub rejected_reason: Option<String>,
    pub cancelled_reason: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Absence entry history (audit trail)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AbsenceEntryHistory {
    pub id: Uuid,
    pub entry_id: Uuid,
    pub action: String,
    pub from_status: Option<String>,
    pub to_status: Option<String>,
    pub performed_by: Option<Uuid>,
    pub comment: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Absence management dashboard summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AbsenceDashboard {
    pub total_types: i64,
    pub active_types: i64,
    pub total_plans: i64,
    pub active_plans: i64,
    pub pending_entries: i64,
    pub approved_entries_today: i64,
    pub entries_by_status: serde_json::Value,
    pub entries_by_type: serde_json::Value,
    pub recent_entries: Vec<AbsenceEntry>,
}

// ============================================================================
// Risk Management & Internal Controls (Oracle Fusion GRC / Advanced Controls)
// ============================================================================

/// Risk Category
/// Oracle Fusion: GRC > Risk Manager > Risk Categories
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RiskCategory {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub parent_category_id: Option<Uuid>,
    pub is_active: bool,
    pub sort_order: i32,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Risk Register Entry
/// Oracle Fusion: GRC > Risk Manager > Risk Register
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RiskEntry {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub risk_number: String,
    pub title: String,
    pub description: Option<String>,
    pub category_id: Option<Uuid>,
    pub risk_source: String,
    pub likelihood: i32,
    pub impact: i32,
    pub risk_score: i32,
    pub risk_level: String,
    pub status: String,
    pub owner_id: Option<Uuid>,
    pub owner_name: Option<String>,
    pub business_units: serde_json::Value,
    pub response_strategy: Option<String>,
    pub residual_likelihood: Option<i32>,
    pub residual_impact: Option<i32>,
    pub identified_date: chrono::NaiveDate,
    pub last_assessed_date: Option<chrono::NaiveDate>,
    pub next_review_date: Option<chrono::NaiveDate>,
    pub closed_date: Option<chrono::NaiveDate>,
    pub related_entity_type: Option<String>,
    pub related_entity_id: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Control Registry Entry
/// Oracle Fusion: GRC > Advanced Controls > Control Registry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ControlEntry {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub control_number: String,
    pub title: String,
    pub description: Option<String>,
    pub control_type: String,
    pub control_nature: String,
    pub frequency: String,
    pub objective: Option<String>,
    pub test_procedures: Option<String>,
    pub owner_id: Option<Uuid>,
    pub owner_name: Option<String>,
    pub is_key_control: bool,
    pub effectiveness: String,
    pub status: String,
    pub business_processes: serde_json::Value,
    pub regulatory_frameworks: serde_json::Value,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Risk-Control Mapping
/// Oracle Fusion: GRC > Risk Manager > Risk-Control Associations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RiskControlMapping {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub risk_id: Uuid,
    pub control_id: Uuid,
    pub mitigation_effectiveness: String,
    pub status: String,
    pub description: Option<String>,
    pub mapped_by: Option<Uuid>,
    pub mapped_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Control Test
/// Oracle Fusion: GRC > Advanced Controls > Control Testing & Certification
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ControlTest {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub control_id: Uuid,
    pub test_number: String,
    pub test_plan: String,
    pub test_period_start: chrono::NaiveDate,
    pub test_period_end: chrono::NaiveDate,
    pub tester_id: Option<Uuid>,
    pub tester_name: Option<String>,
    pub result: String,
    pub findings: Option<String>,
    pub deficiency_severity: Option<String>,
    pub evidence_document_ids: serde_json::Value,
    pub sample_size: Option<i32>,
    pub sample_exceptions: Option<i32>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub reviewer_id: Option<Uuid>,
    pub reviewer_name: Option<String>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub review_status: String,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Risk Issue / Remediation
/// Oracle Fusion: GRC > Issue Management > Remediation Tracker
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RiskIssue {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub issue_number: String,
    pub title: String,
    pub description: String,
    pub source: String,
    pub risk_id: Option<Uuid>,
    pub control_id: Option<Uuid>,
    pub control_test_id: Option<Uuid>,
    pub severity: String,
    pub priority: String,
    pub status: String,
    pub owner_id: Option<Uuid>,
    pub owner_name: Option<String>,
    pub remediation_plan: Option<String>,
    pub remediation_due_date: Option<chrono::NaiveDate>,
    pub remediation_completed_date: Option<chrono::NaiveDate>,
    pub root_cause: Option<String>,
    pub corrective_actions: Option<String>,
    pub identified_date: chrono::NaiveDate,
    pub resolved_date: Option<chrono::NaiveDate>,
    pub closed_date: Option<chrono::NaiveDate>,
    pub regulatory_reference: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Risk Management Dashboard Summary
/// Oracle Fusion: GRC > Risk Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RiskDashboard {
    pub total_risks: i32,
    pub open_risks: i32,
    pub mitigated_risks: i32,
    pub accepted_risks: i32,
    pub critical_risks: i32,
    pub high_risks: i32,
    pub medium_risks: i32,
    pub low_risks: i32,
    pub total_controls: i32,
    pub active_controls: i32,
    pub effective_controls: i32,
    pub ineffective_controls: i32,
    pub not_tested_controls: i32,
    pub total_tests: i32,
    pub passed_tests: i32,
    pub failed_tests: i32,
    pub open_issues: i32,
    pub critical_issues: i32,
    pub overdue_remediations: i32,
    pub risks_by_source: serde_json::Value,
    pub risks_by_level: serde_json::Value,
    pub control_effectiveness_summary: serde_json::Value,
}

// ============================================================================
// Enterprise Asset Management (eAM)
// Oracle Fusion Cloud: Maintenance Management / Enterprise Asset Management
// Physical asset maintenance, work orders, preventive maintenance,
// maintenance schedules, material & labor tracking.
// ============================================================================

/// Physical asset definition (distinct from Fixed Assets which is financial)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetDefinition {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub asset_number: String,
    pub name: String,
    pub description: String,
    pub asset_group: String, // equipment class (pump, motor, vehicle, hvac, etc.)
    pub asset_criticality: String, // low, medium, high, critical
    pub asset_status: String, // active, inactive, disposed, in_repair
    pub location_id: Option<Uuid>,
    pub location_name: String,
    pub parent_asset_id: Option<Uuid>,
    pub serial_number: String,
    pub manufacturer: String,
    pub model: String,
    pub install_date: Option<chrono::NaiveDate>,
    pub warranty_expiry: Option<chrono::NaiveDate>,
    pub last_maintenance_date: Option<chrono::NaiveDate>,
    pub next_maintenance_date: Option<chrono::NaiveDate>,
    pub meter_reading: Option<serde_json::Value>, // {type, value, unit, last_read}
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Maintenance work order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceWorkOrder {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub work_order_number: String,
    pub title: String,
    pub description: String,
    pub work_order_type: String, // corrective, preventive, emergency, inspection, project
    pub priority: String,        // low, normal, high, urgent
    pub status: String,          // draft, approved, in_progress, completed, closed, cancelled
    pub asset_id: Uuid,
    pub asset_number: String,
    pub asset_name: String,
    pub location_name: String,
    pub assigned_to: Option<Uuid>,
    pub assigned_to_name: String,
    pub scheduled_start: Option<chrono::NaiveDate>,
    pub scheduled_end: Option<chrono::NaiveDate>,
    pub actual_start: Option<chrono::DateTime<chrono::Utc>>,
    pub actual_end: Option<chrono::DateTime<chrono::Utc>>,
    pub estimated_hours: Option<serde_json::Value>,
    pub actual_hours: Option<serde_json::Value>,
    pub estimated_cost: String,
    pub actual_cost: String,
    pub downtime_hours: f64,
    pub failure_code: String,
    pub cause_code: String,
    pub resolution_code: String,
    pub materials: serde_json::Value, // [{item, quantity, unit_cost}]
    pub labor: serde_json::Value,     // [{person_id, name, hours, rate}]
    pub completion_notes: String,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<chrono::DateTime<chrono::Utc>>,
    pub closed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Preventive maintenance schedule / program
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreventiveMaintenanceSchedule {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub schedule_number: String,
    pub name: String,
    pub description: String,
    pub asset_id: Uuid,
    pub asset_number: String,
    pub asset_name: String,
    pub schedule_type: String, // time_based, meter_based, condition_based
    pub frequency: String,     // daily, weekly, monthly, quarterly, semi_annual, annual
    pub interval_value: i32,   // every N days/weeks/months/meters
    pub interval_unit: String, // days, weeks, months, hours, miles, cycles
    pub meter_type: String,    // hours, miles, km, cycles (for meter-based)
    pub meter_threshold: Option<serde_json::Value>,
    pub work_order_template: Option<serde_json::Value>,
    pub estimated_duration_hours: f64,
    pub estimated_cost: String,
    pub next_due_date: Option<chrono::NaiveDate>,
    pub last_completed_date: Option<chrono::NaiveDate>,
    pub last_completed_wo: String,
    pub auto_generate: bool, // auto-generate work orders
    pub lead_time_days: i32, // days before due to generate WO
    pub status: String,      // active, inactive, completed
    pub effective_start: Option<chrono::NaiveDate>,
    pub effective_end: Option<chrono::NaiveDate>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Asset maintenance dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceDashboard {
    pub total_assets: i32,
    pub active_assets: i32,
    pub assets_in_repair: i32,
    pub critical_assets: i32,
    pub total_work_orders: i32,
    pub open_work_orders: i32,
    pub in_progress_work_orders: i32,
    pub completed_work_orders: i32,
    pub overdue_work_orders: i32,
    pub emergency_work_orders: i32,
    pub preventive_work_orders: i32,
    pub corrective_work_orders: i32,
    pub total_schedules: i32,
    pub active_schedules: i32,
    pub overdue_schedules: i32,
    pub avg_completion_days: f64,
    pub total_maintenance_cost: String,
    pub total_downtime_hours: f64,
    pub mtbf_hours: f64, // mean time between failures
    pub mttr_hours: f64, // mean time to repair
    pub work_orders_by_priority: serde_json::Value,
    pub work_orders_by_type: serde_json::Value,
    pub assets_by_criticality: serde_json::Value,
    pub costs_by_month: serde_json::Value,
}

// ============================================================================
// Sustainability & ESG Management Types
// Oracle Fusion Cloud: Sustainability / Environmental Accounting & Reporting
// ============================================================================

/// Sustainability facility (site / building being tracked)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SustainabilityFacility {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub facility_code: String,
    pub name: String,
    pub description: Option<String>,
    pub country_code: Option<String>,
    pub region: Option<String>,
    pub city: Option<String>,
    pub address: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub facility_type: String,
    pub industry_sector: Option<String>,
    pub total_area_sqm: Option<f64>,
    pub employee_count: Option<i32>,
    pub operating_hours_per_year: i32,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Emission factor for converting activity data to `CO2e`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmissionFactor {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub factor_code: String,
    pub name: String,
    pub description: Option<String>,
    pub scope: String,
    pub category: String,
    pub activity_type: String,
    pub factor_value: f64,
    pub unit_of_measure: String,
    pub gas_type: String,
    pub factor_source: Option<String>,
    pub effective_from: chrono::NaiveDate,
    pub effective_to: Option<chrono::NaiveDate>,
    pub region_code: Option<String>,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Environmental activity log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentalActivity {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub activity_number: String,
    pub facility_id: Option<Uuid>,
    pub facility_code: Option<String>,
    pub activity_type: String,
    pub scope: String,
    pub category: Option<String>,
    pub quantity: f64,
    pub unit_of_measure: String,
    pub emission_factor_id: Option<Uuid>,
    pub co2e_kg: f64,
    pub co2_kg: Option<f64>,
    pub ch4_kg: Option<f64>,
    pub n2o_kg: Option<f64>,
    pub cost_amount: Option<f64>,
    pub cost_currency: Option<String>,
    pub activity_date: chrono::NaiveDate,
    pub reporting_period: Option<String>,
    pub source_type: Option<String>,
    pub source_reference: Option<String>,
    pub department_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub status: String,
    pub verified_by: Option<Uuid>,
    pub verified_at: Option<chrono::DateTime<chrono::Utc>>,
    pub notes: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// ESG metric definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsgMetric {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub metric_code: String,
    pub name: String,
    pub description: Option<String>,
    pub pillar: String,
    pub category: String,
    pub unit_of_measure: String,
    pub gri_standard: Option<String>,
    pub sasb_standard: Option<String>,
    pub tcfd_category: Option<String>,
    pub eu_taxonomy_code: Option<String>,
    pub target_value: Option<f64>,
    pub warning_threshold: Option<f64>,
    pub direction: String,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// ESG metric reading (actual value over time)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsgMetricReading {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub metric_id: Uuid,
    pub metric_value: f64,
    pub reading_date: chrono::NaiveDate,
    pub reporting_period: Option<String>,
    pub facility_id: Option<Uuid>,
    pub notes: Option<String>,
    pub source: Option<String>,
    pub verified_by: Option<Uuid>,
    pub verified_at: Option<chrono::DateTime<chrono::Utc>>,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Sustainability goal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SustainabilityGoal {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub goal_code: String,
    pub name: String,
    pub description: Option<String>,
    pub goal_type: String,
    pub scope: Option<String>,
    pub baseline_value: f64,
    pub baseline_year: i32,
    pub baseline_unit: String,
    pub target_value: f64,
    pub target_year: i32,
    pub target_unit: String,
    pub target_reduction_pct: Option<f64>,
    pub milestones: serde_json::Value,
    pub current_value: f64,
    pub progress_pct: f64,
    pub facility_id: Option<Uuid>,
    pub status: String,
    pub owner_id: Option<Uuid>,
    pub owner_name: Option<String>,
    pub framework: Option<String>,
    pub framework_reference: Option<String>,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Carbon offset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CarbonOffset {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub offset_number: String,
    pub name: String,
    pub description: Option<String>,
    pub project_name: String,
    pub project_type: String,
    pub project_location: Option<String>,
    pub registry: Option<String>,
    pub registry_id: Option<String>,
    pub certification_standard: Option<String>,
    pub quantity_tonnes: f64,
    pub remaining_tonnes: f64,
    pub unit_price: Option<f64>,
    pub total_cost: Option<f64>,
    pub currency_code: Option<String>,
    pub vintage_year: i32,
    pub retired_quantity: f64,
    pub retired_date: Option<chrono::NaiveDate>,
    pub effective_from: chrono::NaiveDate,
    pub effective_to: Option<chrono::NaiveDate>,
    pub status: String,
    pub supplier_name: Option<String>,
    pub supplier_id: Option<Uuid>,
    pub notes: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

// ============================================================================
// Engineering Change Management (ECM) Types
// Oracle Fusion Cloud: Product Development > Engineering Change Management
// ============================================================================

/// Engineering change type definition (ECR, ECO, ECN)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineeringChangeType {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub type_code: String,
    pub name: String,
    pub description: Option<String>,
    pub category: String,
    pub approval_required: bool,
    pub default_priority: String,
    pub number_prefix: String,
    pub statuses: serde_json::Value,
    pub description_template: Option<String>,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Engineering change order / request / notice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineeringChange {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub change_number: String,
    pub change_type_id: Option<Uuid>,
    pub category: String,
    pub title: String,
    pub description: Option<String>,
    pub change_reason: Option<String>,
    pub change_reason_description: Option<String>,
    pub priority: String,
    pub status: String,
    pub revision: String,
    pub assigned_to: Option<Uuid>,
    pub assigned_to_name: Option<String>,
    pub submitted_at: Option<chrono::DateTime<chrono::Utc>>,
    pub approved_at: Option<chrono::DateTime<chrono::Utc>>,
    pub implemented_at: Option<chrono::DateTime<chrono::Utc>>,
    pub target_date: Option<chrono::NaiveDate>,
    pub effective_date: Option<chrono::NaiveDate>,
    pub resolution_code: Option<String>,
    pub resolution_notes: Option<String>,
    pub parent_change_id: Option<Uuid>,
    pub superseded_by_id: Option<Uuid>,
    pub impact_analysis: serde_json::Value,
    pub estimated_cost: Option<f64>,
    pub actual_cost: Option<f64>,
    pub currency_code: String,
    pub estimated_hours: Option<f64>,
    pub actual_hours: Option<f64>,
    pub regulatory_impact: Option<String>,
    pub safety_impact: Option<String>,
    pub validation_required: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Change line within an engineering change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineeringChangeLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub change_id: Uuid,
    pub line_number: i32,
    pub item_id: Option<Uuid>,
    pub item_number: Option<String>,
    pub item_name: Option<String>,
    pub change_category: String,
    pub field_name: Option<String>,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub old_revision: Option<String>,
    pub new_revision: Option<String>,
    pub component_item_id: Option<Uuid>,
    pub component_item_number: Option<String>,
    pub bom_quantity_old: Option<f64>,
    pub bom_quantity_new: Option<f64>,
    pub effectivity_date: Option<chrono::NaiveDate>,
    pub effectivity_end_date: Option<chrono::NaiveDate>,
    pub status: String,
    pub completion_notes: Option<String>,
    pub sequence_number: i32,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Affected item linked to an engineering change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineeringChangeAffectedItem {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub change_id: Uuid,
    pub item_id: Uuid,
    pub item_number: String,
    pub item_name: Option<String>,
    pub impact_type: String,
    pub impact_description: Option<String>,
    pub current_revision: Option<String>,
    pub new_revision: Option<String>,
    pub disposition: Option<String>,
    pub old_item_status: Option<String>,
    pub new_item_status: Option<String>,
    pub phase_in_date: Option<chrono::NaiveDate>,
    pub phase_out_date: Option<chrono::NaiveDate>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Approval record for an engineering change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineeringChangeApproval {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub change_id: Uuid,
    pub approval_level: i32,
    pub approver_id: Option<Uuid>,
    pub approver_name: Option<String>,
    pub approver_role: Option<String>,
    pub status: String,
    pub action_date: Option<chrono::DateTime<chrono::Utc>>,
    pub comments: Option<String>,
    pub delegated_from_id: Option<Uuid>,
    pub approval_conditions: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// ECM dashboard summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcmDashboard {
    pub total_changes: i32,
    pub open_changes: i32,
    pub pending_approval: i32,
    pub approved_changes: i32,
    pub implemented_changes: i32,
    pub rejected_changes: i32,
    pub ecr_count: i32,
    pub eco_count: i32,
    pub ecn_count: i32,
    pub critical_open: i32,
    pub high_open: i32,
    pub medium_open: i32,
    pub low_open: i32,
    pub avg_days_to_implement: f64,
    pub avg_days_to_approve: f64,
    pub total_items_affected: i32,
    pub total_estimated_cost: f64,
    pub total_actual_cost: f64,
    pub changes_by_reason: serde_json::Value,
    pub changes_by_status: serde_json::Value,
    pub changes_trend: serde_json::Value,
}

/// Sustainability dashboard summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SustainabilityDashboard {
    pub total_facilities: i32,
    pub active_facilities: i32,
    pub total_emissions_co2e_tonnes: f64,
    pub scope1_emissions_tonnes: f64,
    pub scope2_emissions_tonnes: f64,
    pub scope3_emissions_tonnes: f64,
    pub total_energy_consumed_kwh: f64,
    pub renewable_energy_pct: f64,
    pub total_water_consumed_cubic_m: f64,
    pub total_waste_generated_tonnes: f64,
    pub waste_diverted_pct: f64,
    pub total_offsets_tonnes: f64,
    pub net_emissions_tonnes: f64,
    pub active_goals: i32,
    pub goals_on_track: i32,
    pub goals_achieved: i32,
    pub esg_metrics_count: i32,
    pub emissions_by_scope: serde_json::Value,
    pub emissions_by_category: serde_json::Value,
    pub emissions_trend: serde_json::Value,
    pub goals_by_status: serde_json::Value,
}

// ============================================================================
// Rebate Management (Oracle Fusion Cloud: Trade Management > Rebates)
// ============================================================================
// Manages supplier and customer rebate agreements with tiered pricing,
// volume tracking, accruals, and settlement processing.
//
// Key concepts:
// - Rebate Agreement: contract defining rebate terms with a partner
// - Rebate Tier: volume-based pricing thresholds
// - Rebate Transaction: qualifying purchase/sale linked to an agreement
// - Rebate Accrual: periodic recognition of estimated rebate income/expense
// - Rebate Settlement: actual payment or credit of accrued rebates
// ============================================================================

/// Rebate Agreement
/// Defines the terms of a rebate arrangement with a supplier or customer.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RebateAgreement {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub agreement_number: String,
    pub name: String,
    pub description: String,
    pub rebate_type: String,  // supplier_rebate, customer_rebate
    pub direction: String,    // receivable, payable
    pub partner_type: String, // supplier, customer
    pub partner_id: Option<Uuid>,
    pub partner_name: String,
    pub partner_number: String,
    pub product_category: String,
    pub product_id: Option<Uuid>,
    pub product_name: String,
    pub uom: String,
    pub currency_code: String,
    pub start_date: chrono::NaiveDate,
    pub end_date: chrono::NaiveDate,
    pub status: String,
    pub calculation_method: String, // flat_rate, tiered, cumulative
    pub accrual_account: String,
    pub liability_account: String,
    pub expense_account: String,
    pub payment_terms: String,
    pub settlement_frequency: String,
    pub minimum_amount: f64,
    pub maximum_amount: Option<f64>,
    pub auto_accrue: bool,
    pub requires_approval: bool,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<chrono::DateTime<chrono::Utc>>,
    pub notes: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Rebate Tier
/// Defines a volume threshold within a tiered rebate agreement.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RebateTier {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub agreement_id: Uuid,
    pub tier_number: i32,
    pub from_value: f64,
    pub to_value: Option<f64>,
    pub rebate_rate: f64,
    pub rate_type: String, // percentage, fixed_per_unit, fixed_amount
    pub description: String,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Rebate Transaction
/// A qualifying purchase/sale transaction linked to a rebate agreement.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RebateTransaction {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub agreement_id: Uuid,
    pub transaction_number: String,
    pub source_type: String,
    pub source_id: Option<Uuid>,
    pub source_number: String,
    pub transaction_date: chrono::NaiveDate,
    pub product_id: Option<Uuid>,
    pub product_name: String,
    pub quantity: f64,
    pub unit_price: f64,
    pub transaction_amount: f64,
    pub currency_code: String,
    pub applicable_rate: f64,
    pub rebate_amount: f64,
    pub status: String,
    pub tier_id: Option<Uuid>,
    pub excluded_reason: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Rebate Accrual
/// Periodic recognition of estimated rebate amounts.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RebateAccrual {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub agreement_id: Uuid,
    pub accrual_number: String,
    pub accrual_date: chrono::NaiveDate,
    pub accrual_period: String,
    pub accumulated_quantity: f64,
    pub accumulated_amount: f64,
    pub applicable_tier_id: Option<Uuid>,
    pub applicable_rate: f64,
    pub accrued_amount: f64,
    pub currency_code: String,
    pub gl_posted: bool,
    pub gl_journal_id: Option<Uuid>,
    pub status: String,
    pub notes: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Rebate Settlement
/// Actual payment or credit memo for accrued rebate amounts.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RebateSettlement {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub agreement_id: Uuid,
    pub settlement_number: String,
    pub settlement_date: chrono::NaiveDate,
    pub settlement_period_from: Option<chrono::NaiveDate>,
    pub settlement_period_to: Option<chrono::NaiveDate>,
    pub total_qualifying_amount: f64,
    pub total_qualifying_quantity: f64,
    pub applicable_tier_id: Option<Uuid>,
    pub applicable_rate: f64,
    pub settlement_amount: f64,
    pub currency_code: String,
    pub settlement_type: String,
    pub payment_method: String,
    pub payment_reference: String,
    pub status: String,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<chrono::DateTime<chrono::Utc>>,
    pub paid_at: Option<chrono::DateTime<chrono::Utc>>,
    pub notes: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Rebate Settlement Line
/// Links a settlement to specific rebate transactions.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RebateSettlementLine {
    pub id: Uuid,
    pub settlement_id: Uuid,
    pub transaction_id: Uuid,
    pub settlement_amount: f64,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Rebate Management Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RebateDashboard {
    pub organization_id: Uuid,
    pub total_agreements: i64,
    pub active_agreements: i64,
    pub total_transactions: i64,
    pub total_qualifying_amount: f64,
    pub total_accrued_amount: f64,
    pub total_settled_amount: f64,
    pub pending_settlements: i64,
    pub agreements_by_type: serde_json::Value,
    pub top_rebate_agreements: serde_json::Value,
    pub recent_settlements: serde_json::Value,
}
