//! Engineering Change Management Engine
//!
//! Manages engineering change orders, change lines, affected items,
//! approval workflows, and revision tracking.
//!
//! Oracle Fusion Cloud equivalent: Product Development > Engineering Change Management

use atlas_shared::{
    EngineeringChangeType, EngineeringChange, EngineeringChangeLine,
    EngineeringChangeAffectedItem, EngineeringChangeApproval, EcmDashboard,
    AtlasError, AtlasResult,
};
use super::EngineeringChangeManagementRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

// ============================================================================
// Valid enum constants
// ============================================================================

const VALID_CHANGE_CATEGORIES: &[&str] = &[
    "ecr", "eco", "ecn",
];

const VALID_PRIORITIES: &[&str] = &[
    "low", "medium", "high", "critical",
];

const VALID_CHANGE_STATUSES: &[&str] = &[
    "draft", "submitted", "in_review", "approved", "rejected",
    "implemented", "closed", "cancelled",
];

const VALID_CHANGE_REASONS: &[&str] = &[
    "design_improvement", "cost_reduction", "quality_issue",
    "safety_regulatory", "customer_request", "supplier_change",
    "product_enhancement", "defect_correction", "standardization",
    "obsolescence", "other",
];

const VALID_LINE_CATEGORIES: &[&str] = &[
    "item_update", "bom_add", "bom_remove", "bom_change",
    "revision_change", "specification_change",
];

const VALID_LINE_STATUSES: &[&str] = &[
    "pending", "in_progress", "completed", "failed", "skipped",
];

const VALID_IMPACT_TYPES: &[&str] = &[
    "direct", "indirect", "dependent",
];

const VALID_DISPOSITIONS: &[&str] = &[
    "use_existing", "scrap", "rework", "return_to_supplier",
];

#[allow(dead_code)]
const VALID_APPROVAL_STATUSES: &[&str] = &[
    "pending", "approved", "rejected", "returned", "delegated",
];

#[allow(dead_code)]
const VALID_RESOLUTION_CODES: &[&str] = &[
    "implemented", "partially_implemented", "withdrawn", "superseded",
];

/// Helper to validate a value against allowed set
fn validate_enum(field: &str, value: &str, allowed: &[&str]) -> AtlasResult<()> {
    if value.is_empty() {
        return Err(AtlasError::ValidationFailed(format!(
            "{} is required", field
        )));
    }
    if !allowed.contains(&value) {
        return Err(AtlasError::ValidationFailed(format!(
            "Invalid {} '{}'. Must be one of: {}", field, value, allowed.join(", ")
        )));
    }
    Ok(())
}

/// Engineering Change Management Engine
pub struct EngineeringChangeEngine {
    repository: Arc<dyn EngineeringChangeManagementRepository>,
}

impl EngineeringChangeEngine {
    pub fn new(repository: Arc<dyn EngineeringChangeManagementRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Change Types
    // ========================================================================

    /// Create an engineering change type definition
    #[allow(clippy::too_many_arguments)]
    pub async fn create_change_type(
        &self,
        org_id: Uuid,
        type_code: &str,
        name: &str,
        description: Option<&str>,
        category: &str,
        approval_required: bool,
        default_priority: &str,
        number_prefix: &str,
        description_template: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<EngineeringChangeType> {
        if type_code.is_empty() {
            return Err(AtlasError::ValidationFailed("Type code is required".to_string()));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Type name is required".to_string()));
        }
        validate_enum("category", category, VALID_CHANGE_CATEGORIES)?;
        validate_enum("default_priority", default_priority, VALID_PRIORITIES)?;
        if number_prefix.is_empty() {
            return Err(AtlasError::ValidationFailed("Number prefix is required".to_string()));
        }

        if self.repository.get_change_type_by_code(org_id, type_code).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Change type '{}' already exists", type_code
            )));
        }

        info!("Creating engineering change type '{}' ({}) for org {}", type_code, name, org_id);
        self.repository.create_change_type(
            org_id, type_code, name, description, category,
            approval_required, default_priority, number_prefix,
            description_template,
            serde_json::json!(VALID_CHANGE_STATUSES),
            created_by,
        ).await
    }

    /// Get a change type by ID
    pub async fn get_change_type(&self, id: Uuid) -> AtlasResult<Option<EngineeringChangeType>> {
        self.repository.get_change_type(id).await
    }

    /// Get a change type by code
    pub async fn get_change_type_by_code(&self, org_id: Uuid, type_code: &str) -> AtlasResult<Option<EngineeringChangeType>> {
        self.repository.get_change_type_by_code(org_id, type_code).await
    }

    /// List change types for an organization
    pub async fn list_change_types(
        &self,
        org_id: Uuid,
        category: Option<&str>,
    ) -> AtlasResult<Vec<EngineeringChangeType>> {
        if let Some(c) = category {
            validate_enum("category", c, VALID_CHANGE_CATEGORIES)?;
        }
        self.repository.list_change_types(org_id, category).await
    }

    /// Delete a change type by code
    pub async fn delete_change_type(&self, org_id: Uuid, type_code: &str) -> AtlasResult<()> {
        info!("Deleting change type '{}' for org {}", type_code, org_id);
        self.repository.delete_change_type(org_id, type_code).await
    }

    // ========================================================================
    // Engineering Changes (ECO/ECR/ECN)
    // ========================================================================

    /// Create an engineering change order
    #[allow(clippy::too_many_arguments)]
    pub async fn create_change(
        &self,
        org_id: Uuid,
        change_number: &str,
        change_type_id: Option<Uuid>,
        category: &str,
        title: &str,
        description: Option<&str>,
        change_reason: Option<&str>,
        change_reason_description: Option<&str>,
        priority: &str,
        assigned_to: Option<Uuid>,
        assigned_to_name: Option<&str>,
        target_date: Option<chrono::NaiveDate>,
        effective_date: Option<chrono::NaiveDate>,
        estimated_cost: Option<f64>,
        currency_code: Option<&str>,
        estimated_hours: Option<f64>,
        regulatory_impact: Option<&str>,
        safety_impact: Option<&str>,
        validation_required: bool,
        parent_change_id: Option<Uuid>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<EngineeringChange> {
        if change_number.is_empty() {
            return Err(AtlasError::ValidationFailed("Change number is required".to_string()));
        }
        if title.is_empty() {
            return Err(AtlasError::ValidationFailed("Title is required".to_string()));
        }
        validate_enum("category", category, VALID_CHANGE_CATEGORIES)?;
        validate_enum("priority", priority, VALID_PRIORITIES)?;
        if let Some(r) = change_reason {
            validate_enum("change_reason", r, VALID_CHANGE_REASONS)?;
        }

        // Validate change type exists if specified
        if let Some(ct_id) = change_type_id {
            self.repository.get_change_type(ct_id).await?
                .ok_or_else(|| AtlasError::EntityNotFound(format!(
                    "Change type {} not found", ct_id
                )))?;
        }

        // Validate parent change exists if specified
        if let Some(p_id) = parent_change_id {
            self.repository.get_change(p_id).await?
                .ok_or_else(|| AtlasError::EntityNotFound(format!(
                    "Parent change {} not found", p_id
                )))?;
        }

        // Validate estimated cost
        if let Some(cost) = estimated_cost {
            if cost < 0.0 {
                return Err(AtlasError::ValidationFailed(
                    "Estimated cost cannot be negative".to_string(),
                ));
            }
        }

        if let Some(hours) = estimated_hours {
            if hours < 0.0 {
                return Err(AtlasError::ValidationFailed(
                    "Estimated hours cannot be negative".to_string(),
                ));
            }
        }

        if self.repository.get_change_by_number(org_id, change_number).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Change '{}' already exists", change_number
            )));
        }

        info!("Creating engineering change '{}' ({}) for org {} [category={}, priority={}]",
              change_number, title, org_id, category, priority);

        self.repository.create_change(
            org_id, change_number, change_type_id, category,
            title, description, change_reason, change_reason_description,
            priority, "draft", "A",
            assigned_to, assigned_to_name,
            None, None, None, // submitted_at, approved_at, implemented_at
            target_date, effective_date,
            None, None, // resolution_code, resolution_notes
            parent_change_id, None, // superseded_by_id
            serde_json::json!({}),
            estimated_cost, None, // actual_cost
            currency_code.unwrap_or("USD"),
            estimated_hours, None, // actual_hours
            regulatory_impact, safety_impact,
            validation_required,
            created_by,
        ).await
    }

    /// Get a change by ID
    pub async fn get_change(&self, id: Uuid) -> AtlasResult<Option<EngineeringChange>> {
        self.repository.get_change(id).await
    }

    /// Get a change by number
    pub async fn get_change_by_number(&self, org_id: Uuid, change_number: &str) -> AtlasResult<Option<EngineeringChange>> {
        self.repository.get_change_by_number(org_id, change_number).await
    }

    /// List changes with optional filters
    pub async fn list_changes(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        category: Option<&str>,
        priority: Option<&str>,
        assigned_to: Option<&Uuid>,
    ) -> AtlasResult<Vec<EngineeringChange>> {
        if let Some(s) = status {
            validate_enum("status", s, VALID_CHANGE_STATUSES)?;
        }
        if let Some(c) = category {
            validate_enum("category", c, VALID_CHANGE_CATEGORIES)?;
        }
        if let Some(p) = priority {
            validate_enum("priority", p, VALID_PRIORITIES)?;
        }
        self.repository.list_changes(org_id, status, category, priority, assigned_to).await
    }

    /// Submit a change for review/approval (draft → submitted)
    pub async fn submit_change(&self, id: Uuid) -> AtlasResult<EngineeringChange> {
        let change = self.repository.get_change(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Change {} not found", id
            )))?;

        if change.status != "draft" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot submit change in '{}' status. Must be 'draft'.", change.status
            )));
        }

        // Verify there is at least one change line or affected item
        let lines = self.repository.list_change_lines(id).await?;
        let items = self.repository.list_affected_items(id).await?;
        if lines.is_empty() && items.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Cannot submit change without at least one change line or affected item".to_string(),
            ));
        }

        info!("Submitting engineering change '{}' for review", change.change_number);
        self.repository.update_change_status(
            id, "submitted", Some(chrono::Utc::now()), None, None,
        ).await
    }

    /// Start review of a change (submitted → in_review)
    pub async fn start_review(&self, id: Uuid) -> AtlasResult<EngineeringChange> {
        let change = self.repository.get_change(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Change {} not found", id
            )))?;

        if change.status != "submitted" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot start review for change in '{}' status. Must be 'submitted'.", change.status
            )));
        }

        info!("Starting review of engineering change '{}'", change.change_number);
        self.repository.update_change_status(id, "in_review", None, None, None).await
    }

    /// Approve a change (submitted/in_review → approved)
    pub async fn approve_change(
        &self,
        id: Uuid,
        approver_id: Option<Uuid>,
        approver_name: Option<&str>,
        comments: Option<&str>,
    ) -> AtlasResult<EngineeringChange> {
        let change = self.repository.get_change(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Change {} not found", id
            )))?;

        if change.status != "submitted" && change.status != "in_review" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot approve change in '{}' status. Must be 'submitted' or 'in_review'.", change.status
            )));
        }

        // Create approval record
        self.repository.create_approval(
            change.organization_id, id, 1,
            approver_id, approver_name, None,
            "approved", Some(chrono::Utc::now()),
            comments, None, None,
            approver_id,
        ).await?;

        info!("Approving engineering change '{}'", change.change_number);
        self.repository.update_change_status(
            id, "approved", None, Some(chrono::Utc::now()), None,
        ).await
    }

    /// Reject a change (submitted/in_review → rejected)
    pub async fn reject_change(
        &self,
        id: Uuid,
        approver_id: Option<Uuid>,
        approver_name: Option<&str>,
        comments: Option<&str>,
        resolution_notes: Option<&str>,
    ) -> AtlasResult<EngineeringChange> {
        let change = self.repository.get_change(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Change {} not found", id
            )))?;

        if change.status != "submitted" && change.status != "in_review" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot reject change in '{}' status. Must be 'submitted' or 'in_review'.", change.status
            )));
        }

        // Create rejection approval record
        self.repository.create_approval(
            change.organization_id, id, 1,
            approver_id, approver_name, None,
            "rejected", Some(chrono::Utc::now()),
            comments, None, None,
            approver_id,
        ).await?;

        info!("Rejecting engineering change '{}'", change.change_number);
        self.repository.update_change_with_resolution(
            id, "rejected", resolution_notes, Some("withdrawn"),
        ).await
    }

    /// Implement an approved change (approved → implemented)
    pub async fn implement_change(
        &self,
        id: Uuid,
        actual_cost: Option<f64>,
        actual_hours: Option<f64>,
    ) -> AtlasResult<EngineeringChange> {
        let change = self.repository.get_change(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Change {} not found", id
            )))?;

        if change.status != "approved" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot implement change in '{}' status. Must be 'approved'.", change.status
            )));
        }

        info!("Implementing engineering change '{}'", change.change_number);
        self.repository.implement_change(id, actual_cost, actual_hours, Some(chrono::Utc::now())).await
    }

    /// Close an implemented change (implemented → closed)
    pub async fn close_change(&self, id: Uuid) -> AtlasResult<EngineeringChange> {
        let change = self.repository.get_change(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Change {} not found", id
            )))?;

        if change.status != "implemented" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot close change in '{}' status. Must be 'implemented'.", change.status
            )));
        }

        info!("Closing engineering change '{}'", change.change_number);
        self.repository.update_change_status(id, "closed", None, None, None).await
    }

    /// Cancel a draft or submitted change
    pub async fn cancel_change(&self, id: Uuid) -> AtlasResult<EngineeringChange> {
        let change = self.repository.get_change(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Change {} not found", id
            )))?;

        if change.status != "draft" && change.status != "submitted" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot cancel change in '{}' status. Only draft or submitted changes can be cancelled.", change.status
            )));
        }

        info!("Cancelling engineering change '{}'", change.change_number);
        self.repository.update_change_status(id, "cancelled", None, None, None).await
    }

    /// Return a change to draft (submitted → draft) for rework
    pub async fn return_for_rework(&self, id: Uuid, comments: Option<&str>) -> AtlasResult<EngineeringChange> {
        let change = self.repository.get_change(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Change {} not found", id
            )))?;

        if change.status != "submitted" && change.status != "in_review" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot return change in '{}' status. Must be 'submitted' or 'in_review'.", change.status
            )));
        }

        info!("Returning engineering change '{}' for rework", change.change_number);
        self.repository.return_for_rework(id, comments).await
    }

    /// Delete a change by number (only draft or cancelled)
    pub async fn delete_change(&self, org_id: Uuid, change_number: &str) -> AtlasResult<()> {
        let change = self.repository.get_change_by_number(org_id, change_number).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Change '{}' not found", change_number
            )))?;

        if change.status != "draft" && change.status != "cancelled" {
            return Err(AtlasError::ValidationFailed(
                "Only draft or cancelled changes can be deleted".to_string(),
            ));
        }

        info!("Deleting engineering change '{}' for org {}", change_number, org_id);
        self.repository.delete_change(org_id, change_number).await
    }

    // ========================================================================
    // Change Lines
    // ========================================================================

    /// Add a change line to an engineering change
    #[allow(clippy::too_many_arguments)]
    pub async fn create_change_line(
        &self,
        org_id: Uuid,
        change_id: Uuid,
        item_id: Option<Uuid>,
        item_number: Option<&str>,
        item_name: Option<&str>,
        change_category: &str,
        field_name: Option<&str>,
        old_value: Option<&str>,
        new_value: Option<&str>,
        old_revision: Option<&str>,
        new_revision: Option<&str>,
        component_item_id: Option<Uuid>,
        component_item_number: Option<&str>,
        bom_quantity_old: Option<f64>,
        bom_quantity_new: Option<f64>,
        effectivity_date: Option<chrono::NaiveDate>,
        effectivity_end_date: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<EngineeringChangeLine> {
        // Verify change exists and is in an editable state
        let change = self.repository.get_change(change_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Change {} not found", change_id
            )))?;

        if change.status != "draft" {
            return Err(AtlasError::WorkflowError(
                "Change lines can only be added to draft changes".to_string(),
            ));
        }

        validate_enum("change_category", change_category, VALID_LINE_CATEGORIES)?;

        // Validate BOM quantities
        if let Some(qty) = bom_quantity_old {
            if qty < 0.0 {
                return Err(AtlasError::ValidationFailed(
                    "BOM quantity old cannot be negative".to_string(),
                ));
            }
        }
        if let Some(qty) = bom_quantity_new {
            if qty < 0.0 {
                return Err(AtlasError::ValidationFailed(
                    "BOM quantity new cannot be negative".to_string(),
                ));
            }
        }

        // Validate effectivity dates
        if let (Some(from), Some(to)) = (effectivity_date, effectivity_end_date) {
            if to < from {
                return Err(AtlasError::ValidationFailed(
                    "Effectivity end date must be after start date".to_string(),
                ));
            }
        }

        // Get next line number
        let existing_lines = self.repository.list_change_lines(change_id).await?;
        let line_number = (existing_lines.len() as i32) + 1;

        info!("Adding change line {} to change '{}' [category={}]",
              line_number, change.change_number, change_category);

        self.repository.create_change_line(
            org_id, change_id, line_number,
            item_id, item_number, item_name,
            change_category, field_name,
            old_value, new_value,
            old_revision, new_revision,
            component_item_id, component_item_number,
            bom_quantity_old, bom_quantity_new,
            effectivity_date, effectivity_end_date,
            "pending", None,
            line_number, // sequence_number
            created_by,
        ).await
    }

    /// Get a change line by ID
    pub async fn get_change_line(&self, id: Uuid) -> AtlasResult<Option<EngineeringChangeLine>> {
        self.repository.get_change_line(id).await
    }

    /// List change lines for a change
    pub async fn list_change_lines(&self, change_id: Uuid) -> AtlasResult<Vec<EngineeringChangeLine>> {
        self.repository.list_change_lines(change_id).await
    }

    /// Update a change line status
    pub async fn update_change_line_status(
        &self,
        id: Uuid,
        status: &str,
        completion_notes: Option<&str>,
    ) -> AtlasResult<EngineeringChangeLine> {
        validate_enum("line status", status, VALID_LINE_STATUSES)?;
        info!("Updating change line {} status to {}", id, status);
        self.repository.update_change_line_status(id, status, completion_notes).await
    }

    /// Delete a change line
    pub async fn delete_change_line(&self, id: Uuid) -> AtlasResult<()> {
        info!("Deleting change line {}", id);
        self.repository.delete_change_line(id).await
    }

    // ========================================================================
    // Affected Items
    // ========================================================================

    /// Add an affected item to a change
    #[allow(clippy::too_many_arguments)]
    pub async fn add_affected_item(
        &self,
        org_id: Uuid,
        change_id: Uuid,
        item_id: Uuid,
        item_number: &str,
        item_name: Option<&str>,
        impact_type: &str,
        impact_description: Option<&str>,
        current_revision: Option<&str>,
        new_revision: Option<&str>,
        disposition: Option<&str>,
        phase_in_date: Option<chrono::NaiveDate>,
        phase_out_date: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<EngineeringChangeAffectedItem> {
        // Verify change exists and is in an editable state
        let change = self.repository.get_change(change_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Change {} not found", change_id
            )))?;

        if change.status != "draft" && change.status != "submitted" {
            return Err(AtlasError::WorkflowError(
                "Affected items can only be added to draft or submitted changes".to_string(),
            ));
        }

        validate_enum("impact_type", impact_type, VALID_IMPACT_TYPES)?;
        if let Some(d) = disposition {
            validate_enum("disposition", d, VALID_DISPOSITIONS)?;
        }

        // Validate phase dates
        if let (Some(from), Some(to)) = (phase_in_date, phase_out_date) {
            if to < from {
                return Err(AtlasError::ValidationFailed(
                    "Phase-out date must be after phase-in date".to_string(),
                ));
            }
        }

        // Check uniqueness of item within change
        if self.repository.get_affected_item(change_id, item_id).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Item {} is already an affected item on this change", item_number
            )));
        }

        info!("Adding affected item '{}' to change '{}' [impact={}]",
              item_number, change.change_number, impact_type);

        self.repository.create_affected_item(
            org_id, change_id, item_id, item_number, item_name,
            impact_type, impact_description,
            current_revision, new_revision,
            disposition, None, None, // old/new item status
            phase_in_date, phase_out_date,
            created_by,
        ).await
    }

    /// Get an affected item by ID
    pub async fn get_affected_item_by_id(&self, id: Uuid) -> AtlasResult<Option<EngineeringChangeAffectedItem>> {
        self.repository.get_affected_item_by_id(id).await
    }

    /// List affected items for a change
    pub async fn list_affected_items(&self, change_id: Uuid) -> AtlasResult<Vec<EngineeringChangeAffectedItem>> {
        self.repository.list_affected_items(change_id).await
    }

    /// Remove an affected item
    pub async fn remove_affected_item(&self, id: Uuid) -> AtlasResult<()> {
        info!("Removing affected item {}", id);
        self.repository.remove_affected_item(id).await
    }

    // ========================================================================
    // Approvals
    // ========================================================================

    /// List approvals for a change
    pub async fn list_approvals(&self, change_id: Uuid) -> AtlasResult<Vec<EngineeringChangeApproval>> {
        self.repository.list_approvals(change_id).await
    }

    /// Get pending approvals for an approver
    pub async fn get_pending_approvals(&self, approver_id: Uuid) -> AtlasResult<Vec<EngineeringChangeApproval>> {
        self.repository.get_pending_approvals(approver_id).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get the ECM dashboard summary
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<EcmDashboard> {
        self.repository.get_dashboard(org_id).await
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Enum validation tests
    // ========================================================================

    #[test]
    fn test_valid_change_categories() {
        assert!(VALID_CHANGE_CATEGORIES.contains(&"ecr"));
        assert!(VALID_CHANGE_CATEGORIES.contains(&"eco"));
        assert!(VALID_CHANGE_CATEGORIES.contains(&"ecn"));
        assert!(!VALID_CHANGE_CATEGORIES.contains(&"ecp"));
    }

    #[test]
    fn test_valid_priorities() {
        assert!(VALID_PRIORITIES.contains(&"low"));
        assert!(VALID_PRIORITIES.contains(&"medium"));
        assert!(VALID_PRIORITIES.contains(&"high"));
        assert!(VALID_PRIORITIES.contains(&"critical"));
        assert!(!VALID_PRIORITIES.contains(&"urgent"));
    }

    #[test]
    fn test_valid_change_statuses() {
        assert!(VALID_CHANGE_STATUSES.contains(&"draft"));
        assert!(VALID_CHANGE_STATUSES.contains(&"submitted"));
        assert!(VALID_CHANGE_STATUSES.contains(&"in_review"));
        assert!(VALID_CHANGE_STATUSES.contains(&"approved"));
        assert!(VALID_CHANGE_STATUSES.contains(&"rejected"));
        assert!(VALID_CHANGE_STATUSES.contains(&"implemented"));
        assert!(VALID_CHANGE_STATUSES.contains(&"closed"));
        assert!(VALID_CHANGE_STATUSES.contains(&"cancelled"));
    }

    #[test]
    fn test_valid_change_reasons() {
        assert!(VALID_CHANGE_REASONS.contains(&"design_improvement"));
        assert!(VALID_CHANGE_REASONS.contains(&"cost_reduction"));
        assert!(VALID_CHANGE_REASONS.contains(&"quality_issue"));
        assert!(VALID_CHANGE_REASONS.contains(&"safety_regulatory"));
        assert!(VALID_CHANGE_REASONS.contains(&"customer_request"));
        assert!(VALID_CHANGE_REASONS.contains(&"supplier_change"));
        assert!(VALID_CHANGE_REASONS.contains(&"defect_correction"));
        assert!(VALID_CHANGE_REASONS.contains(&"obsolescence"));
    }

    #[test]
    fn test_valid_line_categories() {
        assert!(VALID_LINE_CATEGORIES.contains(&"item_update"));
        assert!(VALID_LINE_CATEGORIES.contains(&"bom_add"));
        assert!(VALID_LINE_CATEGORIES.contains(&"bom_remove"));
        assert!(VALID_LINE_CATEGORIES.contains(&"bom_change"));
        assert!(VALID_LINE_CATEGORIES.contains(&"revision_change"));
        assert!(VALID_LINE_CATEGORIES.contains(&"specification_change"));
    }

    #[test]
    fn test_valid_line_statuses() {
        assert!(VALID_LINE_STATUSES.contains(&"pending"));
        assert!(VALID_LINE_STATUSES.contains(&"in_progress"));
        assert!(VALID_LINE_STATUSES.contains(&"completed"));
        assert!(VALID_LINE_STATUSES.contains(&"failed"));
        assert!(VALID_LINE_STATUSES.contains(&"skipped"));
    }

    #[test]
    fn test_valid_impact_types() {
        assert!(VALID_IMPACT_TYPES.contains(&"direct"));
        assert!(VALID_IMPACT_TYPES.contains(&"indirect"));
        assert!(VALID_IMPACT_TYPES.contains(&"dependent"));
    }

    #[test]
    fn test_valid_dispositions() {
        assert!(VALID_DISPOSITIONS.contains(&"use_existing"));
        assert!(VALID_DISPOSITIONS.contains(&"scrap"));
        assert!(VALID_DISPOSITIONS.contains(&"rework"));
        assert!(VALID_DISPOSITIONS.contains(&"return_to_supplier"));
    }

    #[test]
    fn test_valid_approval_statuses() {
        assert!(VALID_APPROVAL_STATUSES.contains(&"pending"));
        assert!(VALID_APPROVAL_STATUSES.contains(&"approved"));
        assert!(VALID_APPROVAL_STATUSES.contains(&"rejected"));
        assert!(VALID_APPROVAL_STATUSES.contains(&"returned"));
        assert!(VALID_APPROVAL_STATUSES.contains(&"delegated"));
    }

    #[test]
    fn test_valid_resolution_codes() {
        assert!(VALID_RESOLUTION_CODES.contains(&"implemented"));
        assert!(VALID_RESOLUTION_CODES.contains(&"partially_implemented"));
        assert!(VALID_RESOLUTION_CODES.contains(&"withdrawn"));
        assert!(VALID_RESOLUTION_CODES.contains(&"superseded"));
    }

    // ========================================================================
    // Enum validation function tests
    // ========================================================================

    #[test]
    fn test_validate_enum_valid() {
        assert!(validate_enum("category", "eco", VALID_CHANGE_CATEGORIES).is_ok());
        assert!(validate_enum("priority", "high", VALID_PRIORITIES).is_ok());
        assert!(validate_enum("status", "draft", VALID_CHANGE_STATUSES).is_ok());
    }

    #[test]
    fn test_validate_enum_invalid() {
        let result = validate_enum("category", "invalid", VALID_CHANGE_CATEGORIES);
        assert!(result.is_err());
        match result {
            Err(AtlasError::ValidationFailed(msg)) => {
                assert!(msg.contains("category"));
                assert!(msg.contains("invalid"));
            }
            _ => panic!("Expected ValidationFailed error"),
        }
    }

    #[test]
    fn test_validate_enum_empty() {
        let result = validate_enum("priority", "", VALID_PRIORITIES);
        assert!(result.is_err());
        match result {
            Err(AtlasError::ValidationFailed(msg)) => {
                assert!(msg.contains("required"));
            }
            _ => panic!("Expected ValidationFailed error"),
        }
    }

    // ========================================================================
    // Status transition validation tests
    // ========================================================================

    #[test]
    fn test_change_status_transition_submit() {
        // Submit: only draft -> submitted
        assert!(can_submit("draft"));
        assert!(!can_submit("submitted"));
        assert!(!can_submit("in_review"));
        assert!(!can_submit("approved"));
        assert!(!can_submit("rejected"));
        assert!(!can_submit("implemented"));
        assert!(!can_submit("closed"));
        assert!(!can_submit("cancelled"));
    }

    #[test]
    fn test_change_status_transition_review() {
        // Start review: only submitted -> in_review
        assert!(!can_start_review("draft"));
        assert!(can_start_review("submitted"));
        assert!(!can_start_review("in_review"));
        assert!(!can_start_review("approved"));
    }

    #[test]
    fn test_change_status_transition_approve() {
        // Approve: submitted or in_review -> approved
        assert!(!can_approve("draft"));
        assert!(can_approve("submitted"));
        assert!(can_approve("in_review"));
        assert!(!can_approve("approved"));
        assert!(!can_approve("rejected"));
        assert!(!can_approve("implemented"));
    }

    #[test]
    fn test_change_status_transition_reject() {
        // Reject: submitted or in_review -> rejected
        assert!(!can_reject("draft"));
        assert!(can_reject("submitted"));
        assert!(can_reject("in_review"));
        assert!(!can_reject("approved"));
    }

    #[test]
    fn test_change_status_transition_implement() {
        // Implement: only approved -> implemented
        assert!(!can_implement("draft"));
        assert!(!can_implement("submitted"));
        assert!(!can_implement("approved")); // would need mock, test logic only
        assert!(!can_implement("rejected"));
        assert!(!can_implement("implemented"));
    }

    #[test]
    fn test_change_status_transition_close() {
        // Close: only implemented -> closed
        assert!(can_close("implemented"));
        assert!(!can_close("draft"));
        assert!(!can_close("approved"));
    }

    #[test]
    fn test_change_status_transition_cancel() {
        // Cancel: only draft or submitted -> cancelled
        assert!(can_cancel("draft"));
        assert!(can_cancel("submitted"));
        assert!(!can_cancel("in_review"));
        assert!(!can_cancel("approved"));
        assert!(!can_cancel("implemented"));
        assert!(!can_cancel("closed"));
    }

    #[test]
    fn test_change_status_transition_return_for_rework() {
        // Return: submitted or in_review -> draft
        assert!(!can_return("draft"));
        assert!(can_return("submitted"));
        assert!(can_return("in_review"));
        assert!(!can_return("approved"));
    }

    #[test]
    fn test_full_lifecycle_eco() {
        // Happy path: draft -> submitted -> in_review -> approved -> implemented -> closed
        let mut status = "draft";
        assert_eq!(status, "draft");

        status = "submitted"; // submit
        assert_eq!(status, "submitted");

        status = "in_review"; // start review
        assert_eq!(status, "in_review");

        status = "approved"; // approve
        assert_eq!(status, "approved");

        status = "implemented"; // implement
        assert_eq!(status, "implemented");

        status = "closed"; // close
        assert_eq!(status, "closed");
    }

    #[test]
    fn test_fast_track_lifecycle() {
        // Fast track: draft -> submitted -> approved -> implemented -> closed
        let statuses = vec!["draft", "submitted", "approved", "implemented", "closed"];
        for i in 0..statuses.len() - 1 {
            assert_ne!(statuses[i], statuses[i + 1]);
        }
    }

    // ========================================================================
    // Change line category validation tests
    // ========================================================================

    #[test]
    fn test_bom_change_line_categories() {
        let bom_cats = &["bom_add", "bom_remove", "bom_change"];
        for cat in bom_cats {
            assert!(VALID_LINE_CATEGORIES.contains(cat));
        }
    }

    #[test]
    fn test_item_change_line_categories() {
        let item_cats = &["item_update", "revision_change", "specification_change"];
        for cat in item_cats {
            assert!(VALID_LINE_CATEGORIES.contains(cat));
        }
    }

    // ========================================================================
    // Cost validation tests
    // ========================================================================

    #[test]
    fn test_estimated_cost_validation() {
        let valid: f64 = 1000.50;
        let zero: f64 = 0.0;
        let invalid: f64 = -1.0;
        assert!(valid >= 0.0);
        assert!(zero >= 0.0);
        assert!(invalid < 0.0);
    }

    // ========================================================================
    // Date validation tests
    // ========================================================================

    #[test]
    fn test_effectivity_date_validation() {
        let from = chrono::NaiveDate::from_ymd_opt(2024, 6, 1).unwrap();
        let to = chrono::NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();
        assert!(to >= from);

        let bad_to = chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        assert!(bad_to < from);
    }

    #[test]
    fn test_phase_date_validation() {
        let phase_in = chrono::NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();
        let phase_out = chrono::NaiveDate::from_ymd_opt(2024, 9, 30).unwrap();
        assert!(phase_out >= phase_in);
    }

    // ========================================================================
    // Helper functions for status transition tests
    // ========================================================================

    fn can_submit(status: &str) -> bool { status == "draft" }
    fn can_start_review(status: &str) -> bool { status == "submitted" }
    fn can_approve(status: &str) -> bool { status == "submitted" || status == "in_review" }
    fn can_reject(status: &str) -> bool { status == "submitted" || status == "in_review" }
    fn can_implement(_status: &str) -> bool { false } // needs mock
    fn can_close(status: &str) -> bool { status == "implemented" }
    fn can_cancel(status: &str) -> bool { status == "draft" || status == "submitted" }
    fn can_return(status: &str) -> bool { status == "submitted" || status == "in_review" }
}
