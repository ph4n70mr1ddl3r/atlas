//! Quality Management Engine
//!
//! Oracle Fusion Cloud Quality Management.
//! Manages inspection plans, quality inspections, non-conformance reports (NCRs),
//! corrective & preventive actions (CAPA), and quality holds.
//!
//! Quality lifecycle:
//! - Inspection Plans: draft → active → inactive
//! - Inspections: planned → in_progress → completed
//! - NCRs: open → under_investigation → corrective_action → resolved/closed
//! - CAPA: open → in_progress → completed → verified
//! - Holds: active → released
//!
//! Oracle Fusion Cloud ERP equivalent: Quality Management

use atlas_shared::{
    QualityInspectionPlan, QualityInspectionPlanCriterion,
    QualityInspection, QualityInspectionResult,
    NonConformanceReport, CorrectiveAction,
    QualityHold, QualityDashboardSummary,
    AtlasError, AtlasResult,
};
use super::QualityManagementRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

// ========================================================================
// Valid constants
// ========================================================================

const VALID_PLAN_TYPES: &[&str] = &[
    "receiving", "in_process", "final", "audit", "supplier",
];

const VALID_INSPECTION_TRIGGERS: &[&str] = &[
    "every_receipt", "first_article", "percentage_sample",
    "periodic", "on_demand", "supplier_certified",
];

const VALID_SAMPLING_METHODS: &[&str] = &[
    "full", "random", "stratified", "aql", "custom",
];

const VALID_FREQUENCIES: &[&str] = &[
    "per_lot", "per_shipment", "per_order", "daily", "weekly", "monthly", "one_time",
];

const VALID_MEASUREMENT_TYPES: &[&str] = &[
    "pass_fail", "numeric", "text", "visual", "multi_choice",
];

const VALID_CRITICALITIES: &[&str] = &[
    "critical", "major", "minor", "informational",
];

const VALID_INSPECTION_STATUSES: &[&str] = &[
    "planned", "in_progress", "completed", "cancelled",
];

const VALID_VERDICTS: &[&str] = &[
    "pass", "fail", "conditional_pass", "pending",
];

const VALID_RESULT_STATUSES: &[&str] = &[
    "pass", "fail", "conditional", "not_evaluated",
];

const VALID_NCR_TYPES: &[&str] = &[
    "defect", "damage", "wrong_item", "quantity_variance",
    "documentation", "packaging", "labeling", "specification",
    "performance", "other",
];

const VALID_SEVERITIES: &[&str] = &[
    "critical", "major", "minor", "low",
];

const VALID_NCR_ORIGINS: &[&str] = &[
    "inspection", "customer_complaint", "internal_audit",
    "supplier_audit", "process_monitoring", "other",
];

const VALID_NCR_STATUSES: &[&str] = &[
    "open", "under_investigation", "corrective_action",
    "resolved", "closed",
];

const VALID_RESOLUTION_TYPES: &[&str] = &[
    "rework", "scrap", "return_to_supplier", "use_as_is",
    "sort", "repair", "concession",
];

const VALID_ACTION_TYPES: &[&str] = &[
    "corrective", "preventive", "both",
];

// TODO: Use VALID_ACTION_STATUSES for status filtering in list_corrective_actions
#[allow(dead_code)]
const VALID_ACTION_STATUSES: &[&str] = &[
    "open", "in_progress", "completed", "verified", "cancelled",
];

const VALID_PRIORITIES: &[&str] = &[
    "critical", "high", "medium", "low",
];

const VALID_HOLD_TYPES: &[&str] = &[
    "item", "lot", "supplier", "purchase_order",
];

const VALID_HOLD_STATUSES: &[&str] = &[
    "active", "released", "expired",
];

/// Quality Management Engine
pub struct QualityManagementEngine {
    repository: Arc<dyn QualityManagementRepository>,
}

impl QualityManagementEngine {
    pub fn new(repository: Arc<dyn QualityManagementRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Inspection Plans
    // ========================================================================

    /// Create a new quality inspection plan
    pub async fn create_plan(
        &self,
        org_id: Uuid,
        plan_code: &str,
        name: &str,
        description: Option<&str>,
        plan_type: &str,
        item_id: Option<Uuid>,
        item_code: Option<&str>,
        supplier_id: Option<Uuid>,
        supplier_name: Option<&str>,
        inspection_trigger: &str,
        sampling_method: &str,
        sample_size_percent: Option<&str>,
        accept_number: Option<i32>,
        reject_number: Option<i32>,
        frequency: &str,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<QualityInspectionPlan> {
        if plan_code.is_empty() {
            return Err(AtlasError::ValidationFailed("Plan code is required".to_string()));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Plan name is required".to_string()));
        }
        if !VALID_PLAN_TYPES.contains(&plan_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid plan type '{}'. Must be one of: {}",
                plan_type, VALID_PLAN_TYPES.join(", ")
            )));
        }
        if !VALID_INSPECTION_TRIGGERS.contains(&inspection_trigger) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid inspection trigger '{}'. Must be one of: {}",
                inspection_trigger, VALID_INSPECTION_TRIGGERS.join(", ")
            )));
        }
        if !VALID_SAMPLING_METHODS.contains(&sampling_method) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid sampling method '{}'. Must be one of: {}",
                sampling_method, VALID_SAMPLING_METHODS.join(", ")
            )));
        }
        if !VALID_FREQUENCIES.contains(&frequency) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid frequency '{}'. Must be one of: {}",
                frequency, VALID_FREQUENCIES.join(", ")
            )));
        }
        if let Some(pct_str) = sample_size_percent {
            let pct: f64 = pct_str.parse().map_err(|_| AtlasError::ValidationFailed(
                "Sample size percent must be a valid number".to_string(),
            ))?;
            if pct <= 0.0 || pct > 100.0 {
                return Err(AtlasError::ValidationFailed(
                    "Sample size percent must be between 0 and 100".to_string(),
                ));
            }
        }
        if let (Some(accept), Some(reject)) = (accept_number, reject_number) {
            if reject <= accept {
                return Err(AtlasError::ValidationFailed(
                    "Reject number must be greater than accept number".to_string(),
                ));
            }
        }

        info!("Creating inspection plan {} ({}) for org {}", plan_code, name, org_id);

        self.repository
            .create_plan(
                org_id, plan_code, name, description, plan_type,
                item_id, item_code, supplier_id, supplier_name,
                inspection_trigger, sampling_method, sample_size_percent,
                accept_number, reject_number, frequency,
                effective_from, effective_to, created_by,
            )
            .await
    }

    /// Get a plan by code
    pub async fn get_plan(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<QualityInspectionPlan>> {
        self.repository.get_plan(org_id, code).await
    }

    /// List inspection plans
    pub async fn list_plans(&self, org_id: Uuid, active_only: bool) -> AtlasResult<Vec<QualityInspectionPlan>> {
        self.repository.list_plans(org_id, active_only).await
    }

    /// Delete a plan
    pub async fn delete_plan(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        self.repository.delete_plan(org_id, code).await
    }

    // ========================================================================
    // Plan Criteria
    // ========================================================================

    /// Add a criterion to an inspection plan
    pub async fn create_criterion(
        &self,
        org_id: Uuid,
        plan_id: Uuid,
        criterion_number: i32,
        name: &str,
        description: Option<&str>,
        characteristic: &str,
        measurement_type: &str,
        target_value: Option<&str>,
        lower_spec_limit: Option<&str>,
        upper_spec_limit: Option<&str>,
        unit_of_measure: Option<&str>,
        is_mandatory: bool,
        weight: &str,
        criticality: &str,
    ) -> AtlasResult<QualityInspectionPlanCriterion> {
        // Validate plan exists
        let _plan = self
            .repository
            .get_plan_by_id(plan_id)
            .await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Inspection plan {} not found", plan_id)))?;

        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Criterion name is required".to_string()));
        }
        if !VALID_MEASUREMENT_TYPES.contains(&measurement_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid measurement type '{}'. Must be one of: {}",
                measurement_type, VALID_MEASUREMENT_TYPES.join(", ")
            )));
        }
        let w: f64 = weight.parse().map_err(|_| AtlasError::ValidationFailed(
            "Weight must be a valid number".to_string(),
        ))?;
        if w < 0.0 {
            return Err(AtlasError::ValidationFailed("Weight cannot be negative".to_string()));
        }
        if !VALID_CRITICALITIES.contains(&criticality) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid criticality '{}'. Must be one of: {}",
                criticality, VALID_CRITICALITIES.join(", ")
            )));
        }

        info!("Adding criterion {} to plan {}", criterion_number, plan_id);

        self.repository
            .create_criterion(
                org_id, plan_id, criterion_number, name, description,
                characteristic, measurement_type, target_value,
                lower_spec_limit, upper_spec_limit, unit_of_measure,
                is_mandatory, weight, criticality,
            )
            .await
    }

    /// List criteria for a plan
    pub async fn list_criteria(&self, plan_id: Uuid) -> AtlasResult<Vec<QualityInspectionPlanCriterion>> {
        self.repository.list_criteria(plan_id).await
    }

    /// Delete a criterion
    pub async fn delete_criterion(&self, id: Uuid) -> AtlasResult<()> {
        self.repository.delete_criterion(id).await
    }

    // ========================================================================
    // Inspections
    // ========================================================================

    /// Create a new inspection from a plan
    pub async fn create_inspection(
        &self,
        org_id: Uuid,
        plan_id: Uuid,
        source_type: &str,
        source_id: Option<Uuid>,
        source_number: Option<&str>,
        item_id: Option<Uuid>,
        item_code: Option<&str>,
        item_description: Option<&str>,
        lot_number: Option<&str>,
        quantity_inspected: &str,
        quantity_accepted: &str,
        quantity_rejected: &str,
        unit_of_measure: Option<&str>,
        inspector_id: Option<Uuid>,
        inspector_name: Option<&str>,
        inspection_date: chrono::NaiveDate,
        created_by: Option<Uuid>,
    ) -> AtlasResult<QualityInspection> {
        // Validate plan exists
        let _plan = self
            .repository
            .get_plan_by_id(plan_id)
            .await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Inspection plan {} not found", plan_id)))?;

        let qi: f64 = quantity_inspected.parse().map_err(|_| AtlasError::ValidationFailed(
            "Quantity inspected must be a valid number".to_string(),
        ))?;
        let qa: f64 = quantity_accepted.parse().unwrap_or(0.0);
        let qr: f64 = quantity_rejected.parse().unwrap_or(0.0);

        if qi < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Quantity inspected cannot be negative".to_string(),
            ));
        }
        if qa + qr > qi {
            return Err(AtlasError::ValidationFailed(
                "Sum of accepted and rejected quantities cannot exceed inspected quantity".to_string(),
            ));
        }

        let inspection_number = format!("QI-{}", Uuid::new_v4().to_string()[..8].to_uppercase());

        info!("Creating inspection {} from plan {}", inspection_number, plan_id);

        self.repository
            .create_inspection(
                org_id, &inspection_number, plan_id,
                source_type, source_id, source_number,
                item_id, item_code, item_description,
                lot_number, quantity_inspected, quantity_accepted, quantity_rejected,
                unit_of_measure, inspector_id, inspector_name, inspection_date,
                created_by,
            )
            .await
    }

    /// Get an inspection by ID
    pub async fn get_inspection(&self, id: Uuid) -> AtlasResult<Option<QualityInspection>> {
        self.repository.get_inspection(id).await
    }

    /// List inspections with optional filters
    pub async fn list_inspections(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        plan_id: Option<Uuid>,
    ) -> AtlasResult<Vec<QualityInspection>> {
        if let Some(s) = status {
            if !VALID_INSPECTION_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}",
                    s, VALID_INSPECTION_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_inspections(org_id, status, plan_id, None).await
    }

    /// Start an inspection (move from planned to in_progress)
    pub async fn start_inspection(&self, id: Uuid) -> AtlasResult<QualityInspection> {
        let inspection = self
            .repository
            .get_inspection(id)
            .await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Inspection {} not found", id)))?;

        if inspection.status != "planned" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot start inspection in '{}' status. Must be 'planned'.",
                inspection.status
            )));
        }

        info!("Starting inspection {}", inspection.inspection_number);
        self.repository.update_inspection_status(id, "in_progress", None).await
    }

    /// Complete an inspection with a verdict
    pub async fn complete_inspection(
        &self,
        id: Uuid,
        verdict: &str,
        notes: Option<&str>,
    ) -> AtlasResult<QualityInspection> {
        let inspection = self
            .repository
            .get_inspection(id)
            .await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Inspection {} not found", id)))?;

        if inspection.status != "in_progress" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot complete inspection in '{}' status. Must be 'in_progress'.",
                inspection.status
            )));
        }

        if !VALID_VERDICTS.contains(&verdict) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid verdict '{}'. Must be one of: {}",
                verdict, VALID_VERDICTS.join(", ")
            )));
        }

        // Calculate overall score from results
        let results = self.repository.list_results(id).await?;
        let score = self.calculate_inspection_score(&results);

        info!(
            "Completing inspection {} with verdict '{}' (score: {:.1}%)",
            inspection.inspection_number, verdict, score
        );

        self.repository
            .update_inspection_verdict(id, verdict, Some(&format!("{:.2}", score)), notes)
            .await?;

        self.repository
            .update_inspection_status(id, "completed", Some(chrono::Utc::now()))
            .await
    }

    /// Cancel an inspection
    pub async fn cancel_inspection(&self, id: Uuid) -> AtlasResult<QualityInspection> {
        let inspection = self
            .repository
            .get_inspection(id)
            .await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Inspection {} not found", id)))?;

        if inspection.status == "completed" || inspection.status == "cancelled" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot cancel inspection in '{}' status.",
                inspection.status
            )));
        }

        info!("Cancelled inspection {}", inspection.inspection_number);
        self.repository.update_inspection_status(id, "cancelled", None).await
    }

    // ========================================================================
    // Inspection Results
    // ========================================================================

    /// Record an inspection result
    pub async fn create_result(
        &self,
        org_id: Uuid,
        inspection_id: Uuid,
        criterion_id: Option<Uuid>,
        criterion_name: &str,
        characteristic: &str,
        measurement_type: &str,
        observed_value: Option<&str>,
        target_value: Option<&str>,
        lower_spec_limit: Option<&str>,
        upper_spec_limit: Option<&str>,
        unit_of_measure: Option<&str>,
        result_status: &str,
        deviation: Option<&str>,
        notes: Option<&str>,
        evaluated_by: Option<Uuid>,
    ) -> AtlasResult<QualityInspectionResult> {
        // Validate inspection exists and is in progress
        let inspection = self
            .repository
            .get_inspection(inspection_id)
            .await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Inspection {} not found", inspection_id)))?;

        if inspection.status != "in_progress" && inspection.status != "planned" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot record results for inspection in '{}' status.",
                inspection.status
            )));
        }

        if criterion_name.is_empty() {
            return Err(AtlasError::ValidationFailed("Criterion name is required".to_string()));
        }
        if !VALID_RESULT_STATUSES.contains(&result_status) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid result status '{}'. Must be one of: {}",
                result_status, VALID_RESULT_STATUSES.join(", ")
            )));
        }

        // Auto-calculate deviation if numeric with specs
        let computed_deviation = self.calculate_deviation(
            observed_value, target_value, lower_spec_limit, upper_spec_limit,
        );

        let final_deviation = deviation.or(computed_deviation.as_deref());

        self.repository
            .create_result(
                org_id, inspection_id, criterion_id,
                criterion_name, characteristic, measurement_type,
                observed_value, target_value, lower_spec_limit,
                upper_spec_limit, unit_of_measure, result_status,
                final_deviation, notes, evaluated_by,
            )
            .await
    }

    /// List results for an inspection
    pub async fn list_results(&self, inspection_id: Uuid) -> AtlasResult<Vec<QualityInspectionResult>> {
        self.repository.list_results(inspection_id).await
    }

    // ========================================================================
    // Non-Conformance Reports
    // ========================================================================

    /// Create a new Non-Conformance Report
    pub async fn create_ncr(
        &self,
        org_id: Uuid,
        title: &str,
        description: Option<&str>,
        ncr_type: &str,
        severity: &str,
        origin: &str,
        source_type: Option<&str>,
        source_id: Option<Uuid>,
        source_number: Option<&str>,
        item_id: Option<Uuid>,
        item_code: Option<&str>,
        supplier_id: Option<Uuid>,
        supplier_name: Option<&str>,
        detected_date: chrono::NaiveDate,
        detected_by: Option<&str>,
        responsible_party: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<NonConformanceReport> {
        if title.is_empty() {
            return Err(AtlasError::ValidationFailed("NCR title is required".to_string()));
        }
        if !VALID_NCR_TYPES.contains(&ncr_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid NCR type '{}'. Must be one of: {}",
                ncr_type, VALID_NCR_TYPES.join(", ")
            )));
        }
        if !VALID_SEVERITIES.contains(&severity) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid severity '{}'. Must be one of: {}",
                severity, VALID_SEVERITIES.join(", ")
            )));
        }
        if !VALID_NCR_ORIGINS.contains(&origin) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid origin '{}'. Must be one of: {}",
                origin, VALID_NCR_ORIGINS.join(", ")
            )));
        }

        let ncr_number = format!("NCR-{}", Uuid::new_v4().to_string()[..8].to_uppercase());

        info!("Creating NCR {} ({}) - {} severity", ncr_number, title, severity);

        self.repository
            .create_ncr(
                org_id, &ncr_number, title, description,
                ncr_type, severity, origin,
                source_type, source_id, source_number,
                item_id, item_code, supplier_id, supplier_name,
                detected_date, detected_by, responsible_party, created_by,
            )
            .await
    }

    /// Get an NCR by ID
    pub async fn get_ncr(&self, id: Uuid) -> AtlasResult<Option<NonConformanceReport>> {
        self.repository.get_ncr(id).await
    }

    /// List NCRs with optional filters
    pub async fn list_ncrs(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        severity: Option<&str>,
    ) -> AtlasResult<Vec<NonConformanceReport>> {
        if let Some(s) = status {
            if !VALID_NCR_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid NCR status '{}'. Must be one of: {}",
                    s, VALID_NCR_STATUSES.join(", ")
                )));
            }
        }
        if let Some(sev) = severity {
            if !VALID_SEVERITIES.contains(&sev) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid severity '{}'. Must be one of: {}",
                    sev, VALID_SEVERITIES.join(", ")
                )));
            }
        }
        self.repository.list_ncrs(org_id, status, severity, None).await
    }

    /// Move NCR to under_investigation
    pub async fn investigate_ncr(&self, id: Uuid) -> AtlasResult<NonConformanceReport> {
        let ncr = self
            .repository
            .get_ncr(id)
            .await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("NCR {} not found", id)))?;

        if ncr.status != "open" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot investigate NCR in '{}' status. Must be 'open'.",
                ncr.status
            )));
        }

        info!("Investigating NCR {}", ncr.ncr_number);
        self.repository.update_ncr_status(id, "under_investigation", None).await
    }

    /// Move NCR to corrective_action phase
    pub async fn start_corrective_action_phase(&self, id: Uuid) -> AtlasResult<NonConformanceReport> {
        let ncr = self
            .repository
            .get_ncr(id)
            .await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("NCR {} not found", id)))?;

        if ncr.status != "under_investigation" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot start corrective action for NCR in '{}' status. Must be 'under_investigation'.",
                ncr.status
            )));
        }

        info!("Starting corrective action phase for NCR {}", ncr.ncr_number);
        self.repository.update_ncr_status(id, "corrective_action", None).await
    }

    /// Resolve an NCR
    pub async fn resolve_ncr(
        &self,
        id: Uuid,
        resolution_description: &str,
        resolution_type: &str,
        resolved_by: Option<&str>,
    ) -> AtlasResult<NonConformanceReport> {
        let ncr = self
            .repository
            .get_ncr(id)
            .await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("NCR {} not found", id)))?;

        if ncr.status != "corrective_action" && ncr.status != "under_investigation" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot resolve NCR in '{}' status. Must be 'corrective_action' or 'under_investigation'.",
                ncr.status
            )));
        }

        if resolution_description.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Resolution description is required".to_string(),
            ));
        }

        if !VALID_RESOLUTION_TYPES.contains(&resolution_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid resolution type '{}'. Must be one of: {}",
                resolution_type, VALID_RESOLUTION_TYPES.join(", ")
            )));
        }

        info!("Resolving NCR {} via {}", ncr.ncr_number, resolution_type);

        self.repository
            .update_ncr_resolution(id, resolution_description, resolution_type, resolved_by)
            .await?;

        self.repository
            .update_ncr_status(id, "resolved", Some(chrono::Utc::now()))
            .await
    }

    /// Close an NCR
    pub async fn close_ncr(&self, id: Uuid) -> AtlasResult<NonConformanceReport> {
        let ncr = self
            .repository
            .get_ncr(id)
            .await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("NCR {} not found", id)))?;

        if ncr.status != "resolved" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot close NCR in '{}' status. Must be 'resolved'.",
                ncr.status
            )));
        }

        // Check all corrective actions are completed or verified
        let actions = self.repository.list_corrective_actions(id).await?;
        let all_closed = actions.iter().all(|a| a.status == "completed" || a.status == "verified" || a.status == "cancelled");
        if !actions.is_empty() && !all_closed {
            return Err(AtlasError::WorkflowError(
                "Cannot close NCR with open corrective actions".to_string(),
            ));
        }

        info!("Closing NCR {}", ncr.ncr_number);
        self.repository.update_ncr_status(id, "closed", None).await
    }

    // ========================================================================
    // Corrective & Preventive Actions
    // ========================================================================

    /// Create a corrective/preventive action for an NCR
    pub async fn create_corrective_action(
        &self,
        org_id: Uuid,
        ncr_id: Uuid,
        action_type: &str,
        title: &str,
        description: Option<&str>,
        root_cause: Option<&str>,
        corrective_action_desc: Option<&str>,
        preventive_action_desc: Option<&str>,
        assigned_to: Option<&str>,
        due_date: Option<chrono::NaiveDate>,
        priority: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CorrectiveAction> {
        // Validate NCR exists
        let ncr = self
            .repository
            .get_ncr(ncr_id)
            .await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("NCR {} not found", ncr_id)))?;

        if ncr.status == "closed" {
            return Err(AtlasError::WorkflowError(
                "Cannot add corrective actions to a closed NCR".to_string(),
            ));
        }

        if title.is_empty() {
            return Err(AtlasError::ValidationFailed("Action title is required".to_string()));
        }
        if !VALID_ACTION_TYPES.contains(&action_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid action type '{}'. Must be one of: {}",
                action_type, VALID_ACTION_TYPES.join(", ")
            )));
        }
        if !VALID_PRIORITIES.contains(&priority) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid priority '{}'. Must be one of: {}",
                priority, VALID_PRIORITIES.join(", ")
            )));
        }

        // Auto-assign action number
        let existing = self.repository.list_corrective_actions(ncr_id).await?;
        let action_number = format!("CAPA-{}-{}", ncr.ncr_number, existing.len() + 1);

        info!("Creating {} action {} for NCR {}", action_type, action_number, ncr.ncr_number);

        self.repository
            .create_corrective_action(
                org_id, ncr_id, &action_number, action_type,
                title, description, root_cause,
                corrective_action_desc, preventive_action_desc,
                assigned_to, due_date, priority, created_by,
            )
            .await
    }

    /// Get a corrective action
    pub async fn get_corrective_action(&self, id: Uuid) -> AtlasResult<Option<CorrectiveAction>> {
        self.repository.get_corrective_action(id).await
    }

    /// List corrective actions for an NCR
    pub async fn list_corrective_actions(&self, ncr_id: Uuid) -> AtlasResult<Vec<CorrectiveAction>> {
        self.repository.list_corrective_actions(ncr_id).await
    }

    /// Start working on a corrective action
    pub async fn start_corrective_action(&self, id: Uuid) -> AtlasResult<CorrectiveAction> {
        let action = self
            .repository
            .get_corrective_action(id)
            .await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Corrective action {} not found", id)))?;

        if action.status != "open" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot start action in '{}' status. Must be 'open'.",
                action.status
            )));
        }

        info!("Starting corrective action {}", action.action_number);
        self.repository.update_corrective_action_status(id, "in_progress", None, None).await
    }

    /// Complete a corrective action
    pub async fn complete_corrective_action(
        &self,
        id: Uuid,
        effectiveness_rating: Option<i32>,
    ) -> AtlasResult<CorrectiveAction> {
        let action = self
            .repository
            .get_corrective_action(id)
            .await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Corrective action {} not found", id)))?;

        if action.status != "in_progress" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot complete action in '{}' status. Must be 'in_progress'.",
                action.status
            )));
        }

        if let Some(rating) = effectiveness_rating {
            if !(1..=5).contains(&rating) {
                return Err(AtlasError::ValidationFailed(
                    "Effectiveness rating must be between 1 and 5".to_string(),
                ));
            }
        }

        info!("Completing corrective action {}", action.action_number);
        self.repository
            .update_corrective_action_status(id, "completed", Some(chrono::Utc::now()), effectiveness_rating)
            .await
    }

    /// Verify a corrective action
    pub async fn verify_corrective_action(&self, id: Uuid) -> AtlasResult<CorrectiveAction> {
        let action = self
            .repository
            .get_corrective_action(id)
            .await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Corrective action {} not found", id)))?;

        if action.status != "completed" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot verify action in '{}' status. Must be 'completed'.",
                action.status
            )));
        }

        info!("Verifying corrective action {}", action.action_number);
        self.repository.update_corrective_action_status(id, "verified", None, None).await
    }

    // ========================================================================
    // Quality Holds
    // ========================================================================

    /// Create a quality hold
    pub async fn create_hold(
        &self,
        org_id: Uuid,
        reason: &str,
        description: Option<&str>,
        item_id: Option<Uuid>,
        item_code: Option<&str>,
        lot_number: Option<&str>,
        supplier_id: Option<Uuid>,
        supplier_name: Option<&str>,
        source_type: Option<&str>,
        source_id: Option<Uuid>,
        source_number: Option<&str>,
        hold_type: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<QualityHold> {
        if reason.is_empty() {
            return Err(AtlasError::ValidationFailed("Hold reason is required".to_string()));
        }
        if !VALID_HOLD_TYPES.contains(&hold_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid hold type '{}'. Must be one of: {}",
                hold_type, VALID_HOLD_TYPES.join(", ")
            )));
        }

        let hold_number = format!("QH-{}", Uuid::new_v4().to_string()[..8].to_uppercase());

        info!("Creating quality hold {} - {}", hold_number, reason);

        self.repository
            .create_hold(
                org_id, &hold_number, reason, description,
                item_id, item_code, lot_number,
                supplier_id, supplier_name,
                source_type, source_id, source_number,
                hold_type, created_by,
            )
            .await
    }

    /// Get a hold by ID
    pub async fn get_hold(&self, id: Uuid) -> AtlasResult<Option<QualityHold>> {
        self.repository.get_hold(id).await
    }

    /// List holds with optional filters
    pub async fn list_holds(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        item_id: Option<Uuid>,
    ) -> AtlasResult<Vec<QualityHold>> {
        if let Some(s) = status {
            if !VALID_HOLD_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid hold status '{}'. Must be one of: {}",
                    s, VALID_HOLD_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_holds(org_id, status, item_id).await
    }

    /// Release a quality hold
    pub async fn release_hold(
        &self,
        id: Uuid,
        released_by: Option<Uuid>,
        release_notes: Option<&str>,
    ) -> AtlasResult<QualityHold> {
        let hold = self
            .repository
            .get_hold(id)
            .await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Hold {} not found", id)))?;

        if hold.status != "active" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot release hold in '{}' status. Must be 'active'.",
                hold.status
            )));
        }

        info!("Releasing quality hold {}", hold.hold_number);
        self.repository.update_hold_status(id, "released", released_by, release_notes).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get quality dashboard summary
    pub async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<QualityDashboardSummary> {
        self.repository.get_dashboard_summary(org_id).await
    }

    // ========================================================================
    // Internal helpers
    // ========================================================================

    /// Calculate overall inspection score from individual results
    fn calculate_inspection_score(&self, results: &[QualityInspectionResult]) -> f64 {
        if results.is_empty() {
            return 0.0;
        }
        let passed = results.iter().filter(|r| r.result_status == "pass").count() as f64;
        (passed / results.len() as f64) * 100.0
    }

    /// Calculate deviation from specification limits
    fn calculate_deviation(
        &self,
        observed: Option<&str>,
        target: Option<&str>,
        lower_limit: Option<&str>,
        upper_limit: Option<&str>,
    ) -> Option<String> {
        let obs: f64 = observed?.parse().ok()?;
        let tgt: f64 = target?.parse().ok()?;

        let deviation = obs - tgt;

        // Check if within spec
        let within_lower = lower_limit
            .and_then(|l| l.parse::<f64>().ok())
            .is_none_or(|l| obs >= l);
        let within_upper = upper_limit
            .and_then(|u| u.parse::<f64>().ok())
            .is_none_or(|u| obs <= u);

        if within_lower && within_upper {
            Some(format!("{:.4}", deviation))
        } else {
            // Out of spec: include indicator for clarity
            Some(format!("{:.4} (OUT_OF_SPEC)", deviation))
        }
    }
}

// ========================================================================
// Tests
// ========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Mock repository for unit testing the engine logic
    struct MockQualityRepository;

    #[async_trait::async_trait]
    impl QualityManagementRepository for MockQualityRepository {
        async fn create_plan(
            &self, _org_id: Uuid, plan_code: &str, name: &str, _description: Option<&str>,
            plan_type: &str, _item_id: Option<Uuid>, _item_code: Option<&str>,
            _supplier_id: Option<Uuid>, _supplier_name: Option<&str>,
            inspection_trigger: &str, sampling_method: &str,
            sample_size_percent: Option<&str>, accept_number: Option<i32>,
            reject_number: Option<i32>, frequency: &str,
            _effective_from: Option<chrono::NaiveDate>, _effective_to: Option<chrono::NaiveDate>,
            _created_by: Option<Uuid>,
        ) -> AtlasResult<QualityInspectionPlan> {
            Ok(QualityInspectionPlan {
                id: Uuid::new_v4(),
                organization_id: _org_id,
                plan_code: plan_code.to_string(),
                name: name.to_string(),
                description: None,
                plan_type: plan_type.to_string(),
                item_id: None, item_code: None,
                supplier_id: None, supplier_name: None,
                inspection_trigger: inspection_trigger.to_string(),
                sampling_method: sampling_method.to_string(),
                sample_size_percent: sample_size_percent.unwrap_or("100").to_string(),
                accept_number, reject_number,
                frequency: frequency.to_string(),
                is_active: true,
                effective_from: None, effective_to: None,
                total_criteria: 0, total_inspections: 0,
                metadata: serde_json::json!({}),
                created_by: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            })
        }
        async fn get_plan(&self, _org_id: Uuid, _code: &str) -> AtlasResult<Option<QualityInspectionPlan>> { Ok(None) }
        async fn get_plan_by_id(&self, _id: Uuid) -> AtlasResult<Option<QualityInspectionPlan>> {
            // Return a stub plan so inspection creation can proceed
            Ok(Some(QualityInspectionPlan {
                id: _id,
                organization_id: Uuid::new_v4(),
                plan_code: "MOCK".to_string(),
                name: "Mock Plan".to_string(),
                description: None,
                plan_type: "receiving".to_string(),
                item_id: None, item_code: None,
                supplier_id: None, supplier_name: None,
                inspection_trigger: "every_receipt".to_string(),
                sampling_method: "full".to_string(),
                sample_size_percent: "100".to_string(),
                accept_number: None, reject_number: None,
                frequency: "per_lot".to_string(),
                is_active: true,
                effective_from: None, effective_to: None,
                total_criteria: 0, total_inspections: 0,
                metadata: serde_json::json!({}),
                created_by: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            }))
        }
        async fn list_plans(&self, _org_id: Uuid, _active_only: bool) -> AtlasResult<Vec<QualityInspectionPlan>> { Ok(vec![]) }
        async fn delete_plan(&self, _org_id: Uuid, _code: &str) -> AtlasResult<()> { Ok(()) }

        async fn create_criterion(
            &self, _org_id: Uuid, _plan_id: Uuid, _criterion_number: i32,
            name: &str, _description: Option<&str>, _characteristic: &str,
            measurement_type: &str, _target_value: Option<&str>,
            _lower_spec_limit: Option<&str>, _upper_spec_limit: Option<&str>,
            _unit_of_measure: Option<&str>, _is_mandatory: bool,
            weight: &str, criticality: &str,
        ) -> AtlasResult<QualityInspectionPlanCriterion> {
            Ok(QualityInspectionPlanCriterion {
                id: Uuid::new_v4(), organization_id: _org_id, plan_id: _plan_id,
                criterion_number: _criterion_number, name: name.to_string(),
                description: None, characteristic: String::new(),
                measurement_type: measurement_type.to_string(),
                target_value: String::new(), lower_spec_limit: String::new(),
                upper_spec_limit: String::new(), unit_of_measure: None,
                is_mandatory: false, weight: weight.to_string(),
                criticality: criticality.to_string(), is_active: true,
                metadata: serde_json::json!({}),
                created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            })
        }
        async fn list_criteria(&self, _plan_id: Uuid) -> AtlasResult<Vec<QualityInspectionPlanCriterion>> { Ok(vec![]) }
        async fn delete_criterion(&self, _id: Uuid) -> AtlasResult<()> { Ok(()) }

        async fn create_inspection(
            &self, _org_id: Uuid, _inspection_number: &str, _plan_id: Uuid,
            _source_type: &str, _source_id: Option<Uuid>,
            _source_number: Option<&str>, _item_id: Option<Uuid>,
            _item_code: Option<&str>, _item_description: Option<&str>,
            _lot_number: Option<&str>, _quantity_inspected: &str,
            _quantity_accepted: &str, _quantity_rejected: &str,
            _unit_of_measure: Option<&str>, _inspector_id: Option<Uuid>,
            _inspector_name: Option<&str>, _inspection_date: chrono::NaiveDate,
            _created_by: Option<Uuid>,
        ) -> AtlasResult<QualityInspection> {
            Ok(QualityInspection {
                id: Uuid::new_v4(), organization_id: _org_id,
                inspection_number: _inspection_number.to_string(),
                plan_id: _plan_id,
                source_type: _source_type.to_string(),
                source_id: None, source_number: None,
                item_id: None, item_code: None, item_description: None,
                lot_number: None,
                quantity_inspected: _quantity_inspected.to_string(),
                quantity_accepted: _quantity_accepted.to_string(),
                quantity_rejected: _quantity_rejected.to_string(),
                unit_of_measure: None,
                status: "planned".to_string(),
                verdict: "pending".to_string(),
                overall_score: "0".to_string(),
                notes: None,
                inspector_id: None, inspector_name: None,
                inspection_date: _inspection_date,
                completed_at: None,
                metadata: serde_json::json!({}),
                created_by: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            })
        }
        async fn get_inspection(&self, _id: Uuid) -> AtlasResult<Option<QualityInspection>> { Ok(None) }
        async fn get_inspection_by_number(&self, _org_id: Uuid, _number: &str) -> AtlasResult<Option<QualityInspection>> { Ok(None) }
        async fn list_inspections(&self, _org_id: Uuid, _status: Option<&str>, _plan_id: Option<Uuid>, _limit: Option<i64>) -> AtlasResult<Vec<QualityInspection>> { Ok(vec![]) }
        async fn update_inspection_status(&self, _id: Uuid, _status: &str, _completed_at: Option<chrono::DateTime<chrono::Utc>>) -> AtlasResult<QualityInspection> {
            Err(AtlasError::EntityNotFound("Mock".to_string()))
        }
        async fn update_inspection_verdict(&self, _id: Uuid, _verdict: &str, _score: Option<&str>, _notes: Option<&str>) -> AtlasResult<QualityInspection> {
            Err(AtlasError::EntityNotFound("Mock".to_string()))
        }
        async fn create_result(
            &self, _org_id: Uuid, _inspection_id: Uuid, _criterion_id: Option<Uuid>,
            _criterion_name: &str, _characteristic: &str,
            _measurement_type: &str, _observed_value: Option<&str>,
            _target_value: Option<&str>, _lower_spec_limit: Option<&str>,
            _upper_spec_limit: Option<&str>, _unit_of_measure: Option<&str>,
            _result_status: &str, _deviation: Option<&str>,
            _notes: Option<&str>, _evaluated_by: Option<Uuid>,
        ) -> AtlasResult<QualityInspectionResult> {
            Ok(QualityInspectionResult {
                id: Uuid::new_v4(), organization_id: _org_id,
                inspection_id: _inspection_id, criterion_id: None,
                criterion_name: _criterion_name.to_string(),
                characteristic: String::new(),
                measurement_type: String::new(),
                observed_value: String::new(), target_value: String::new(),
                lower_spec_limit: String::new(), upper_spec_limit: String::new(),
                unit_of_measure: None,
                result_status: _result_status.to_string(),
                deviation: String::new(), notes: None,
                evaluated_by: None, evaluated_at: None,
                metadata: serde_json::json!({}),
                created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            })
        }
        async fn list_results(&self, _inspection_id: Uuid) -> AtlasResult<Vec<QualityInspectionResult>> { Ok(vec![]) }
        async fn update_result_status(&self, _id: Uuid, _status: &str, _deviation: Option<&str>) -> AtlasResult<QualityInspectionResult> {
            Err(AtlasError::EntityNotFound("Mock".to_string()))
        }

        async fn create_ncr(
            &self, _org_id: Uuid, ncr_number: &str, title: &str,
            _description: Option<&str>, ncr_type: &str, severity: &str,
            origin: &str, _source_type: Option<&str>, _source_id: Option<Uuid>,
            _source_number: Option<&str>, _item_id: Option<Uuid>,
            _item_code: Option<&str>, _supplier_id: Option<Uuid>,
            _supplier_name: Option<&str>, detected_date: chrono::NaiveDate,
            _detected_by: Option<&str>, _responsible_party: Option<&str>,
            _created_by: Option<Uuid>,
        ) -> AtlasResult<NonConformanceReport> {
            Ok(NonConformanceReport {
                id: Uuid::new_v4(), organization_id: _org_id,
                ncr_number: ncr_number.to_string(),
                title: title.to_string(), description: None,
                ncr_type: ncr_type.to_string(), severity: severity.to_string(),
                origin: origin.to_string(),
                source_type: None, source_id: None, source_number: None,
                item_id: None, item_code: None,
                supplier_id: None, supplier_name: None,
                detected_date, detected_by: None, responsible_party: None,
                status: "open".to_string(),
                resolution_description: None, resolution_type: None,
                resolved_by: None, resolved_at: None,
                total_corrective_actions: 0, open_corrective_actions: 0,
                metadata: serde_json::json!({}),
                created_by: None, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            })
        }
        async fn get_ncr(&self, _id: Uuid) -> AtlasResult<Option<NonConformanceReport>> { Ok(None) }
        async fn get_ncr_by_number(&self, _org_id: Uuid, _number: &str) -> AtlasResult<Option<NonConformanceReport>> { Ok(None) }
        async fn list_ncrs(&self, _org_id: Uuid, _status: Option<&str>, _severity: Option<&str>, _limit: Option<i64>) -> AtlasResult<Vec<NonConformanceReport>> { Ok(vec![]) }
        async fn update_ncr_status(&self, _id: Uuid, _status: &str, _resolved_at: Option<chrono::DateTime<chrono::Utc>>) -> AtlasResult<NonConformanceReport> {
            Err(AtlasError::EntityNotFound("Mock".to_string()))
        }
        async fn update_ncr_resolution(&self, _id: Uuid, _resolution_description: &str, _resolution_type: &str, _resolved_by: Option<&str>) -> AtlasResult<NonConformanceReport> {
            Err(AtlasError::EntityNotFound("Mock".to_string()))
        }

        async fn create_corrective_action(
            &self, _org_id: Uuid, _ncr_id: Uuid, action_number: &str,
            action_type: &str, title: &str, _description: Option<&str>,
            _root_cause: Option<&str>, _corrective_action_desc: Option<&str>,
            _preventive_action_desc: Option<&str>,
            _assigned_to: Option<&str>, _due_date: Option<chrono::NaiveDate>,
            priority: &str, _created_by: Option<Uuid>,
        ) -> AtlasResult<CorrectiveAction> {
            Ok(CorrectiveAction {
                id: Uuid::new_v4(), organization_id: _org_id,
                ncr_id: _ncr_id,
                action_number: action_number.to_string(),
                action_type: action_type.to_string(),
                title: title.to_string(), description: None,
                root_cause: None, corrective_action_desc: None,
                preventive_action_desc: None, assigned_to: None,
                due_date: None, status: "open".to_string(),
                completed_at: None, effectiveness_rating: None,
                priority: priority.to_string(),
                metadata: serde_json::json!({}),
                created_by: None, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            })
        }
        async fn get_corrective_action(&self, _id: Uuid) -> AtlasResult<Option<CorrectiveAction>> { Ok(None) }
        async fn list_corrective_actions(&self, _ncr_id: Uuid) -> AtlasResult<Vec<CorrectiveAction>> { Ok(vec![]) }
        async fn update_corrective_action_status(&self, _id: Uuid, _status: &str, _completed_at: Option<chrono::DateTime<chrono::Utc>>, _effectiveness_rating: Option<i32>) -> AtlasResult<CorrectiveAction> {
            Err(AtlasError::EntityNotFound("Mock".to_string()))
        }

        async fn create_hold(
            &self, _org_id: Uuid, hold_number: &str, reason: &str,
            _description: Option<&str>, _item_id: Option<Uuid>,
            _item_code: Option<&str>, _lot_number: Option<&str>,
            _supplier_id: Option<Uuid>, _supplier_name: Option<&str>,
            _source_type: Option<&str>, _source_id: Option<Uuid>,
            _source_number: Option<&str>, hold_type: &str,
            _created_by: Option<Uuid>,
        ) -> AtlasResult<QualityHold> {
            Ok(QualityHold {
                id: Uuid::new_v4(), organization_id: _org_id,
                hold_number: hold_number.to_string(),
                reason: reason.to_string(), description: None,
                item_id: None, item_code: None, lot_number: None,
                supplier_id: None, supplier_name: None,
                source_type: None, source_id: None, source_number: None,
                hold_type: hold_type.to_string(),
                status: "active".to_string(),
                released_by: None, released_at: None, release_notes: None,
                metadata: serde_json::json!({}),
                created_by: None, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            })
        }
        async fn get_hold(&self, _id: Uuid) -> AtlasResult<Option<QualityHold>> { Ok(None) }
        async fn list_holds(&self, _org_id: Uuid, _status: Option<&str>, _item_id: Option<Uuid>) -> AtlasResult<Vec<QualityHold>> { Ok(vec![]) }
        async fn update_hold_status(&self, _id: Uuid, _status: &str, _released_by: Option<Uuid>, _release_notes: Option<&str>) -> AtlasResult<QualityHold> {
            Err(AtlasError::EntityNotFound("Mock".to_string()))
        }

        async fn get_dashboard_summary(&self, _org_id: Uuid) -> AtlasResult<QualityDashboardSummary> {
            Ok(QualityDashboardSummary {
                total_active_plans: 0, total_pending_inspections: 0,
                total_passed_inspections: 0, total_failed_inspections: 0,
                inspection_pass_rate_percent: "0.0".to_string(),
                total_open_ncrs: 0, total_ncrs: 0, critical_ncrs: 0,
                total_open_corrective_actions: 0, total_completed_corrective_actions: 0,
                corrective_action_completion_rate_percent: "0.0".to_string(),
                total_active_holds: 0,
                inspections_by_verdict: serde_json::json!({}),
                ncrs_by_severity: serde_json::json!({}),
                ncrs_by_type: serde_json::json!({}),
            })
        }
    }

    fn make_engine() -> QualityManagementEngine {
        QualityManagementEngine::new(std::sync::Arc::new(MockQualityRepository))
    }

    #[tokio::test]
    async fn test_create_plan_validation() {
        let engine = make_engine();
        let org = Uuid::new_v4();

        // Empty code
        let r = engine.create_plan(
            org, "", "Test Plan", None, "receiving",
            None, None, None, None, "every_receipt", "full",
            None, None, None, "per_lot", None, None, None,
        ).await;
        assert!(r.is_err());
        match r.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("Plan code")),
            _ => panic!("Expected ValidationFailed"),
        }

        // Invalid plan type
        let r = engine.create_plan(
            org, "P-001", "Test Plan", None, "invalid_type",
            None, None, None, None, "every_receipt", "full",
            None, None, None, "per_lot", None, None, None,
        ).await;
        assert!(r.is_err());

        // Valid plan
        let r = engine.create_plan(
            org, "P-001", "Receiving Inspection", None, "receiving",
            None, None, None, None, "every_receipt", "full",
            None, None, None, "per_lot", None, None, None,
        ).await;
        assert!(r.is_ok());
        let plan = r.unwrap();
        assert_eq!(plan.plan_code, "P-001");
        assert_eq!(plan.plan_type, "receiving");
    }

    #[tokio::test]
    async fn test_create_plan_sample_percent_bounds() {
        let engine = make_engine();
        let org = Uuid::new_v4();

        // 0% - invalid
        let r = engine.create_plan(
            org, "P-002", "Plan", None, "receiving",
            None, None, None, None, "every_receipt", "random",
            Some("0"), None, None, "per_lot", None, None, None,
        ).await;
        assert!(r.is_err());

        // 101% - invalid
        let r = engine.create_plan(
            org, "P-002", "Plan", None, "receiving",
            None, None, None, None, "every_receipt", "random",
            Some("101"), None, None, "per_lot", None, None, None,
        ).await;
        assert!(r.is_err());

        // 50% - valid
        let r = engine.create_plan(
            org, "P-002", "Plan", None, "receiving",
            None, None, None, None, "every_receipt", "random",
            Some("50"), None, None, "per_lot", None, None, None,
        ).await;
        assert!(r.is_ok());
    }

    #[tokio::test]
    async fn test_create_inspection_quantity_validation() {
        let engine = make_engine();
        let org = Uuid::new_v4();
        let plan_id = Uuid::new_v4();

        // Negative quantity
        let r = engine.create_inspection(
            org, plan_id, "receiving", None, None,
            None, None, None, None,
            "-10", "0", "0", None,
            None, None, chrono::Utc::now().date_naive(), None,
        ).await;
        assert!(r.is_err());

        // Accepted + Rejected > Inspected
        let r = engine.create_inspection(
            org, plan_id, "receiving", None, None,
            None, None, None, None,
            "10", "8", "5", None,
            None, None, chrono::Utc::now().date_naive(), None,
        ).await;
        assert!(r.is_err());

        // Valid
        let r = engine.create_inspection(
            org, plan_id, "receiving", None, None,
            None, None, None, None,
            "10", "8", "2", None,
            None, None, chrono::Utc::now().date_naive(), None,
        ).await;
        assert!(r.is_ok());
        let insp = r.unwrap();
        assert!(insp.inspection_number.starts_with("QI-"));
    }

    #[tokio::test]
    async fn test_create_ncr_validation() {
        let engine = make_engine();
        let org = Uuid::new_v4();

        // Empty title
        let r = engine.create_ncr(
            org, "", None, "defect", "critical", "inspection",
            None, None, None, None, None, None, None,
            chrono::Utc::now().date_naive(), None, None, None,
        ).await;
        assert!(r.is_err());

        // Invalid type
        let r = engine.create_ncr(
            org, "Bad part", None, "invalid_type", "critical", "inspection",
            None, None, None, None, None, None, None,
            chrono::Utc::now().date_naive(), None, None, None,
        ).await;
        assert!(r.is_err());

        // Invalid severity
        let r = engine.create_ncr(
            org, "Bad part", None, "defect", "catastrophic", "inspection",
            None, None, None, None, None, None, None,
            chrono::Utc::now().date_naive(), None, None, None,
        ).await;
        assert!(r.is_err());

        // Valid
        let r = engine.create_ncr(
            org, "Scratched surface on received parts", None,
            "defect", "major", "inspection",
            None, None, None, None, None, None, None,
            chrono::Utc::now().date_naive(), None, None, None,
        ).await;
        assert!(r.is_ok());
        let ncr = r.unwrap();
        assert!(ncr.ncr_number.starts_with("NCR-"));
        assert_eq!(ncr.status, "open");
    }

    #[tokio::test]
    async fn test_create_hold_validation() {
        let engine = make_engine();
        let org = Uuid::new_v4();

        // Empty reason
        let r = engine.create_hold(
            org, "", None, None, None, None,
            None, None, None, None, None, "item", None,
        ).await;
        assert!(r.is_err());

        // Invalid hold type
        let r = engine.create_hold(
            org, "Bad quality", None, None, None, None,
            None, None, None, None, None, "invalid", None,
        ).await;
        assert!(r.is_err());

        // Valid
        let r = engine.create_hold(
            org, "Failed incoming inspection", None,
            None, Some("ITEM-001"), None,
            None, None, None, None, None, "item", None,
        ).await;
        assert!(r.is_ok());
        let hold = r.unwrap();
        assert!(hold.hold_number.starts_with("QH-"));
        assert_eq!(hold.status, "active");
    }

    #[test]
    fn test_inspection_score_calculation() {
        let engine = make_engine();
        let results = vec![
            QualityInspectionResult {
                id: Uuid::new_v4(), organization_id: Uuid::new_v4(),
                inspection_id: Uuid::new_v4(), criterion_id: None,
                criterion_name: "Dimension".to_string(),
                characteristic: String::new(), measurement_type: String::new(),
                observed_value: "10.05".to_string(), target_value: "10.00".to_string(),
                lower_spec_limit: "9.90".to_string(), upper_spec_limit: "10.10".to_string(),
                unit_of_measure: Some("mm".to_string()),
                result_status: "pass".to_string(),
                deviation: "0.05".to_string(), notes: None,
                evaluated_by: None, evaluated_at: None,
                metadata: serde_json::json!({}),
                created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            },
            QualityInspectionResult {
                id: Uuid::new_v4(), organization_id: Uuid::new_v4(),
                inspection_id: Uuid::new_v4(), criterion_id: None,
                criterion_name: "Weight".to_string(),
                characteristic: String::new(), measurement_type: String::new(),
                observed_value: "50.20".to_string(), target_value: "50.00".to_string(),
                lower_spec_limit: "49.50".to_string(), upper_spec_limit: "50.50".to_string(),
                unit_of_measure: Some("g".to_string()),
                result_status: "pass".to_string(),
                deviation: "0.20".to_string(), notes: None,
                evaluated_by: None, evaluated_at: None,
                metadata: serde_json::json!({}),
                created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            },
            QualityInspectionResult {
                id: Uuid::new_v4(), organization_id: Uuid::new_v4(),
                inspection_id: Uuid::new_v4(), criterion_id: None,
                criterion_name: "Visual".to_string(),
                characteristic: String::new(), measurement_type: String::new(),
                observed_value: String::new(), target_value: String::new(),
                lower_spec_limit: String::new(), upper_spec_limit: String::new(),
                unit_of_measure: None,
                result_status: "fail".to_string(),
                deviation: String::new(), notes: None,
                evaluated_by: None, evaluated_at: None,
                metadata: serde_json::json!({}),
                created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            },
        ];

        let score = engine.calculate_inspection_score(&results);
        assert!((score - 66.67).abs() < 0.1); // 2/3 * 100
    }

    #[test]
    fn test_inspection_score_empty() {
        let engine = make_engine();
        let score = engine.calculate_inspection_score(&[]);
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_deviation_calculation() {
        let engine = make_engine();

        // Deviation with target
        let dev = engine.calculate_deviation(
            Some("10.05"), Some("10.00"), Some("9.90"), Some("10.10"),
        );
        assert!(dev.is_some());
        let dev_val: f64 = dev.unwrap().parse().unwrap();
        assert!((dev_val - 0.05).abs() < 0.001);

        // No observed value
        let dev = engine.calculate_deviation(
            None, Some("10.00"), Some("9.90"), Some("10.10"),
        );
        assert!(dev.is_none());
    }

    #[test]
    fn test_valid_plan_types() {
        assert!(VALID_PLAN_TYPES.contains(&"receiving"));
        assert!(VALID_PLAN_TYPES.contains(&"in_process"));
        assert!(VALID_PLAN_TYPES.contains(&"final"));
        assert!(VALID_PLAN_TYPES.contains(&"audit"));
        assert!(VALID_PLAN_TYPES.contains(&"supplier"));
    }

    #[test]
    fn test_valid_severities() {
        assert!(VALID_SEVERITIES.contains(&"critical"));
        assert!(VALID_SEVERITIES.contains(&"major"));
        assert!(VALID_SEVERITIES.contains(&"minor"));
        assert!(VALID_SEVERITIES.contains(&"low"));
    }

    #[test]
    fn test_valid_ncr_types() {
        assert!(VALID_NCR_TYPES.contains(&"defect"));
        assert!(VALID_NCR_TYPES.contains(&"damage"));
        assert!(VALID_NCR_TYPES.contains(&"wrong_item"));
        assert!(VALID_NCR_TYPES.contains(&"specification"));
    }

    #[test]
    fn test_valid_verdicts() {
        assert!(VALID_VERDICTS.contains(&"pass"));
        assert!(VALID_VERDICTS.contains(&"fail"));
        assert!(VALID_VERDICTS.contains(&"conditional_pass"));
        assert!(VALID_VERDICTS.contains(&"pending"));
    }

    #[test]
    fn test_valid_hold_types() {
        assert!(VALID_HOLD_TYPES.contains(&"item"));
        assert!(VALID_HOLD_TYPES.contains(&"lot"));
        assert!(VALID_HOLD_TYPES.contains(&"supplier"));
        assert!(VALID_HOLD_TYPES.contains(&"purchase_order"));
    }

    #[test]
    fn test_valid_resolution_types() {
        assert!(VALID_RESOLUTION_TYPES.contains(&"rework"));
        assert!(VALID_RESOLUTION_TYPES.contains(&"scrap"));
        assert!(VALID_RESOLUTION_TYPES.contains(&"return_to_supplier"));
        assert!(VALID_RESOLUTION_TYPES.contains(&"use_as_is"));
    }

    #[tokio::test]
    async fn test_create_corrective_action_validation() {
        let engine = QualityManagementEngine::new(std::sync::Arc::new(ClosedNcrMockRepo));

        let org = Uuid::new_v4();
        let ncr_id = Uuid::new_v4();

        // This should fail because the NCR is "closed"
        let r = engine.create_corrective_action(
            org, ncr_id, "corrective", "Fix the problem",
            None, None, None, None, None, None,
            "high", None,
        ).await;
        assert!(r.is_err());
    }

    /// Specialized mock that returns a "closed" NCR
    struct ClosedNcrMockRepo;

    #[async_trait::async_trait]
    impl QualityManagementRepository for ClosedNcrMockRepo {
        async fn create_plan(&self, _org_id: Uuid, _plan_code: &str, _name: &str, _description: Option<&str>, _plan_type: &str, _item_id: Option<Uuid>, _item_code: Option<&str>, _supplier_id: Option<Uuid>, _supplier_name: Option<&str>, _inspection_trigger: &str, _sampling_method: &str, _sample_size_percent: Option<&str>, _accept_number: Option<i32>, _reject_number: Option<i32>, _frequency: &str, _effective_from: Option<chrono::NaiveDate>, _effective_to: Option<chrono::NaiveDate>, _created_by: Option<Uuid>) -> AtlasResult<QualityInspectionPlan> { Err(AtlasError::EntityNotFound("mock".into())) }
        async fn get_plan(&self, _org_id: Uuid, _code: &str) -> AtlasResult<Option<QualityInspectionPlan>> { Ok(None) }
        async fn get_plan_by_id(&self, _id: Uuid) -> AtlasResult<Option<QualityInspectionPlan>> { Ok(None) }
        async fn list_plans(&self, _org_id: Uuid, _active_only: bool) -> AtlasResult<Vec<QualityInspectionPlan>> { Ok(vec![]) }
        async fn delete_plan(&self, _org_id: Uuid, _code: &str) -> AtlasResult<()> { Ok(()) }
        async fn create_criterion(&self, _org_id: Uuid, _plan_id: Uuid, _criterion_number: i32, _name: &str, _description: Option<&str>, _characteristic: &str, _measurement_type: &str, _target_value: Option<&str>, _lower_spec_limit: Option<&str>, _upper_spec_limit: Option<&str>, _unit_of_measure: Option<&str>, _is_mandatory: bool, _weight: &str, _criticality: &str) -> AtlasResult<QualityInspectionPlanCriterion> { Err(AtlasError::EntityNotFound("mock".into())) }
        async fn list_criteria(&self, _plan_id: Uuid) -> AtlasResult<Vec<QualityInspectionPlanCriterion>> { Ok(vec![]) }
        async fn delete_criterion(&self, _id: Uuid) -> AtlasResult<()> { Ok(()) }
        async fn create_inspection(&self, _org_id: Uuid, _inspection_number: &str, _plan_id: Uuid, _source_type: &str, _source_id: Option<Uuid>, _source_number: Option<&str>, _item_id: Option<Uuid>, _item_code: Option<&str>, _item_description: Option<&str>, _lot_number: Option<&str>, _quantity_inspected: &str, _quantity_accepted: &str, _quantity_rejected: &str, _unit_of_measure: Option<&str>, _inspector_id: Option<Uuid>, _inspector_name: Option<&str>, _inspection_date: chrono::NaiveDate, _created_by: Option<Uuid>) -> AtlasResult<QualityInspection> { Err(AtlasError::EntityNotFound("mock".into())) }
        async fn get_inspection(&self, _id: Uuid) -> AtlasResult<Option<QualityInspection>> { Ok(None) }
        async fn get_inspection_by_number(&self, _org_id: Uuid, _number: &str) -> AtlasResult<Option<QualityInspection>> { Ok(None) }
        async fn list_inspections(&self, _org_id: Uuid, _status: Option<&str>, _plan_id: Option<Uuid>, _limit: Option<i64>) -> AtlasResult<Vec<QualityInspection>> { Ok(vec![]) }
        async fn update_inspection_status(&self, _id: Uuid, _status: &str, _completed_at: Option<chrono::DateTime<chrono::Utc>>) -> AtlasResult<QualityInspection> { Err(AtlasError::EntityNotFound("mock".into())) }
        async fn update_inspection_verdict(&self, _id: Uuid, _verdict: &str, _score: Option<&str>, _notes: Option<&str>) -> AtlasResult<QualityInspection> { Err(AtlasError::EntityNotFound("mock".into())) }
        async fn create_result(&self, _org_id: Uuid, _inspection_id: Uuid, _criterion_id: Option<Uuid>, _criterion_name: &str, _characteristic: &str, _measurement_type: &str, _observed_value: Option<&str>, _target_value: Option<&str>, _lower_spec_limit: Option<&str>, _upper_spec_limit: Option<&str>, _unit_of_measure: Option<&str>, _result_status: &str, _deviation: Option<&str>, _notes: Option<&str>, _evaluated_by: Option<Uuid>) -> AtlasResult<QualityInspectionResult> { Err(AtlasError::EntityNotFound("mock".into())) }
        async fn list_results(&self, _inspection_id: Uuid) -> AtlasResult<Vec<QualityInspectionResult>> { Ok(vec![]) }
        async fn update_result_status(&self, _id: Uuid, _status: &str, _deviation: Option<&str>) -> AtlasResult<QualityInspectionResult> { Err(AtlasError::EntityNotFound("mock".into())) }
        async fn create_ncr(&self, _org_id: Uuid, _ncr_number: &str, _title: &str, _description: Option<&str>, _ncr_type: &str, _severity: &str, _origin: &str, _source_type: Option<&str>, _source_id: Option<Uuid>, _source_number: Option<&str>, _item_id: Option<Uuid>, _item_code: Option<&str>, _supplier_id: Option<Uuid>, _supplier_name: Option<&str>, _detected_date: chrono::NaiveDate, _detected_by: Option<&str>, _responsible_party: Option<&str>, _created_by: Option<Uuid>) -> AtlasResult<NonConformanceReport> { Err(AtlasError::EntityNotFound("mock".into())) }
        async fn get_ncr(&self, _id: Uuid) -> AtlasResult<Option<NonConformanceReport>> {
            // Return a closed NCR
            Ok(Some(NonConformanceReport {
                id: _id, organization_id: Uuid::new_v4(),
                ncr_number: "NCR-CLOSED".to_string(),
                title: "Closed NCR".to_string(), description: None,
                ncr_type: "defect".to_string(), severity: "major".to_string(),
                origin: "inspection".to_string(),
                source_type: None, source_id: None, source_number: None,
                item_id: None, item_code: None,
                supplier_id: None, supplier_name: None,
                detected_date: chrono::Utc::now().date_naive(),
                detected_by: None, responsible_party: None,
                status: "closed".to_string(),
                resolution_description: None, resolution_type: None,
                resolved_by: None, resolved_at: None,
                total_corrective_actions: 0, open_corrective_actions: 0,
                metadata: serde_json::json!({}),
                created_by: None, created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
            }))
        }
        async fn get_ncr_by_number(&self, _org_id: Uuid, _number: &str) -> AtlasResult<Option<NonConformanceReport>> { Ok(None) }
        async fn list_ncrs(&self, _org_id: Uuid, _status: Option<&str>, _severity: Option<&str>, _limit: Option<i64>) -> AtlasResult<Vec<NonConformanceReport>> { Ok(vec![]) }
        async fn update_ncr_status(&self, _id: Uuid, _status: &str, _resolved_at: Option<chrono::DateTime<chrono::Utc>>) -> AtlasResult<NonConformanceReport> { Err(AtlasError::EntityNotFound("mock".into())) }
        async fn update_ncr_resolution(&self, _id: Uuid, _resolution_description: &str, _resolution_type: &str, _resolved_by: Option<&str>) -> AtlasResult<NonConformanceReport> { Err(AtlasError::EntityNotFound("mock".into())) }
        async fn create_corrective_action(&self, _org_id: Uuid, _ncr_id: Uuid, _action_number: &str, _action_type: &str, _title: &str, _description: Option<&str>, _root_cause: Option<&str>, _corrective_action_desc: Option<&str>, _preventive_action_desc: Option<&str>, _assigned_to: Option<&str>, _due_date: Option<chrono::NaiveDate>, _priority: &str, _created_by: Option<Uuid>) -> AtlasResult<CorrectiveAction> { Err(AtlasError::EntityNotFound("mock".into())) }
        async fn get_corrective_action(&self, _id: Uuid) -> AtlasResult<Option<CorrectiveAction>> { Ok(None) }
        async fn list_corrective_actions(&self, _ncr_id: Uuid) -> AtlasResult<Vec<CorrectiveAction>> { Ok(vec![]) }
        async fn update_corrective_action_status(&self, _id: Uuid, _status: &str, _completed_at: Option<chrono::DateTime<chrono::Utc>>, _effectiveness_rating: Option<i32>) -> AtlasResult<CorrectiveAction> { Err(AtlasError::EntityNotFound("mock".into())) }
        async fn create_hold(&self, _org_id: Uuid, _hold_number: &str, _reason: &str, _description: Option<&str>, _item_id: Option<Uuid>, _item_code: Option<&str>, _lot_number: Option<&str>, _supplier_id: Option<Uuid>, _supplier_name: Option<&str>, _source_type: Option<&str>, _source_id: Option<Uuid>, _source_number: Option<&str>, _hold_type: &str, _created_by: Option<Uuid>) -> AtlasResult<QualityHold> { Err(AtlasError::EntityNotFound("mock".into())) }
        async fn get_hold(&self, _id: Uuid) -> AtlasResult<Option<QualityHold>> { Ok(None) }
        async fn list_holds(&self, _org_id: Uuid, _status: Option<&str>, _item_id: Option<Uuid>) -> AtlasResult<Vec<QualityHold>> { Ok(vec![]) }
        async fn update_hold_status(&self, _id: Uuid, _status: &str, _released_by: Option<Uuid>, _release_notes: Option<&str>) -> AtlasResult<QualityHold> { Err(AtlasError::EntityNotFound("mock".into())) }
        async fn get_dashboard_summary(&self, _org_id: Uuid) -> AtlasResult<QualityDashboardSummary> { Err(AtlasError::EntityNotFound("mock".into())) }
    }

    #[tokio::test]
    async fn test_list_ncrs_status_validation() {
        let engine = make_engine();
        let org = Uuid::new_v4();

        // Invalid status filter
        let r = engine.list_ncrs(org, Some("nonexistent"), None).await;
        assert!(r.is_err());

        // Invalid severity filter
        let r = engine.list_ncrs(org, None, Some("catastrophic")).await;
        assert!(r.is_err());

        // Valid - no filter
        let r = engine.list_ncrs(org, None, None).await;
        assert!(r.is_ok());
        assert!(r.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_list_inspections_status_validation() {
        let engine = make_engine();
        let org = Uuid::new_v4();

        // Invalid status
        let r = engine.list_inspections(org, Some("unknown"), None).await;
        assert!(r.is_err());

        // Valid status
        let r = engine.list_inspections(org, Some("planned"), None).await;
        assert!(r.is_ok());
    }

    #[tokio::test]
    async fn test_list_holds_status_validation() {
        let engine = make_engine();
        let org = Uuid::new_v4();

        // Invalid status
        let r = engine.list_holds(org, Some("unknown"), None).await;
        assert!(r.is_err());

        // Valid status
        let r = engine.list_holds(org, Some("active"), None).await;
        assert!(r.is_ok());
    }

    #[tokio::test]
    async fn test_dashboard_summary() {
        let engine = make_engine();
        let org = Uuid::new_v4();

        let summary = engine.get_dashboard_summary(org).await.unwrap();
        assert_eq!(summary.total_active_plans, 0);
        assert_eq!(summary.total_pending_inspections, 0);
        assert_eq!(summary.total_active_holds, 0);
    }
}
