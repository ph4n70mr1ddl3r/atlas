//! Goal Management Engine
//!
//! Manages goal library templates, goal plans, goals with cascading
//! hierarchy, progress tracking, alignments, and notes.
//!
//! Oracle Fusion equivalent: HCM > Goal Management

use atlas_shared::{
    GoalLibraryCategory, GoalLibraryTemplate, GoalPlan, Goal,
    GoalAlignment, GoalNote, GoalManagementSummary,
    AtlasError, AtlasResult,
};
use super::GoalManagementRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

// Valid enum values
const VALID_GOAL_TYPES: &[&str] = &["individual", "team", "organization"];
const VALID_GOAL_STATUSES: &[&str] = &[
    "not_started", "in_progress", "on_track", "at_risk", "completed", "cancelled",
];
const VALID_PRIORITIES: &[&str] = &["low", "medium", "high", "critical"];
const VALID_PLAN_TYPES: &[&str] = &["performance", "development", "stretch"];
const VALID_PLAN_STATUSES: &[&str] = &["draft", "active", "closed"];
const VALID_ALIGNMENT_TYPES: &[&str] = &["supports", "depends_on", "cascaded_from"];
const VALID_NOTE_TYPES: &[&str] = &["comment", "feedback", "status_change", "check_in"];
const VALID_VISIBILITIES: &[&str] = &["private", "manager", "public"];
const VALID_OWNER_TYPES: &[&str] = &["employee", "team", "department", "organization"];

/// Goal Management Engine
pub struct GoalManagementEngine {
    repository: Arc<dyn GoalManagementRepository>,
}

impl GoalManagementEngine {
    pub fn new(repository: Arc<dyn GoalManagementRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Goal Library Categories
    // ========================================================================

    /// Create a library category
    pub async fn create_library_category(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        display_order: i32,
        created_by: Option<Uuid>,
    ) -> AtlasResult<GoalLibraryCategory> {
        let code_upper = code.to_uppercase();
        if code_upper.is_empty() || code_upper.len() > 100 {
            return Err(AtlasError::ValidationFailed(
                "Category code must be 1-100 characters".to_string(),
            ));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Category name is required".to_string(),
            ));
        }
        if self.repository.get_library_category_by_code(org_id, &code_upper).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Library category '{}' already exists", code_upper
            )));
        }
        info!("Creating goal library category '{}' for org {}", code_upper, org_id);
        self.repository.create_library_category(
            org_id, &code_upper, name, description, display_order, created_by,
        ).await
    }

    /// Get a library category by ID
    pub async fn get_library_category(&self, id: Uuid) -> AtlasResult<Option<GoalLibraryCategory>> {
        self.repository.get_library_category(id).await
    }

    /// List library categories for an organization
    pub async fn list_library_categories(&self, org_id: Uuid) -> AtlasResult<Vec<GoalLibraryCategory>> {
        self.repository.list_library_categories(org_id).await
    }

    /// Delete a library category by code
    pub async fn delete_library_category(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        info!("Deleting library category '{}' for org {}", code, org_id);
        self.repository.delete_library_category(org_id, code).await
    }

    // ========================================================================
    // Goal Library Templates
    // ========================================================================

    /// Create a library template
    #[allow(clippy::too_many_arguments)]
    pub async fn create_library_template(
        &self,
        org_id: Uuid,
        category_id: Option<Uuid>,
        code: &str,
        name: &str,
        description: Option<&str>,
        goal_type: &str,
        success_criteria: Option<&str>,
        target_metric: Option<&str>,
        target_value: Option<&str>,
        uom: Option<&str>,
        suggested_weight: Option<&str>,
        estimated_duration_days: Option<i32>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<GoalLibraryTemplate> {
        let code_upper = code.to_uppercase();
        if code_upper.is_empty() || code_upper.len() > 100 {
            return Err(AtlasError::ValidationFailed(
                "Template code must be 1-100 characters".to_string(),
            ));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Template name is required".to_string(),
            ));
        }
        if !VALID_GOAL_TYPES.contains(&goal_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid goal_type '{}'. Must be one of: {}", goal_type, VALID_GOAL_TYPES.join(", ")
            )));
        }
        if let Some(w) = suggested_weight {
            if w.parse::<f64>().is_err() {
                return Err(AtlasError::ValidationFailed(
                    "Suggested weight must be a valid number".to_string(),
                ));
            }
        }
        if let Some(v) = target_value {
            if v.parse::<f64>().is_err() {
                return Err(AtlasError::ValidationFailed(
                    "Target value must be a valid number".to_string(),
                ));
            }
        }
        if self.repository.get_library_template_by_code(org_id, &code_upper).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Library template '{}' already exists", code_upper
            )));
        }
        info!("Creating goal library template '{}' for org {}", code_upper, org_id);
        self.repository.create_library_template(
            org_id, category_id, &code_upper, name, description, goal_type,
            success_criteria, target_metric, target_value, uom,
            suggested_weight, estimated_duration_days, created_by,
        ).await
    }

    /// Get a library template by ID
    pub async fn get_library_template(&self, id: Uuid) -> AtlasResult<Option<GoalLibraryTemplate>> {
        self.repository.get_library_template(id).await
    }

    /// List library templates for an organization, optionally filtered by category
    pub async fn list_library_templates(
        &self,
        org_id: Uuid,
        category_id: Option<Uuid>,
        goal_type: Option<&str>,
    ) -> AtlasResult<Vec<GoalLibraryTemplate>> {
        self.repository.list_library_templates(org_id, category_id, goal_type).await
    }

    /// Delete a library template by code
    pub async fn delete_library_template(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        info!("Deleting library template '{}' for org {}", code, org_id);
        self.repository.delete_library_template(org_id, code).await
    }

    // ========================================================================
    // Goal Plans
    // ========================================================================

    /// Create a goal plan
    #[allow(clippy::too_many_arguments)]
    pub async fn create_goal_plan(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        plan_type: &str,
        review_period_start: chrono::NaiveDate,
        review_period_end: chrono::NaiveDate,
        goal_creation_deadline: Option<chrono::NaiveDate>,
        allow_self_goals: bool,
        allow_team_goals: bool,
        max_weight_sum: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<GoalPlan> {
        let code_upper = code.to_uppercase();
        if code_upper.is_empty() || code_upper.len() > 100 {
            return Err(AtlasError::ValidationFailed(
                "Plan code must be 1-100 characters".to_string(),
            ));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Plan name is required".to_string(),
            ));
        }
        if !VALID_PLAN_TYPES.contains(&plan_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid plan_type '{}'. Must be one of: {}", plan_type, VALID_PLAN_TYPES.join(", ")
            )));
        }
        if review_period_end <= review_period_start {
            return Err(AtlasError::ValidationFailed(
                "Review period end must be after start".to_string(),
            ));
        }
        if let Some(dl) = goal_creation_deadline {
            if dl > review_period_end {
                return Err(AtlasError::ValidationFailed(
                    "Goal creation deadline cannot be after review period end".to_string(),
                ));
            }
        }
        if self.repository.get_goal_plan_by_code(org_id, &code_upper).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Goal plan '{}' already exists", code_upper
            )));
        }
        info!("Creating goal plan '{}' for org {}", code_upper, org_id);
        self.repository.create_goal_plan(
            org_id, &code_upper, name, description, plan_type,
            review_period_start, review_period_end, goal_creation_deadline,
            allow_self_goals, allow_team_goals, max_weight_sum, created_by,
        ).await
    }

    /// Get a goal plan by ID
    pub async fn get_goal_plan(&self, id: Uuid) -> AtlasResult<Option<GoalPlan>> {
        self.repository.get_goal_plan(id).await
    }

    /// List goal plans for an organization
    pub async fn list_goal_plans(
        &self,
        org_id: Uuid,
        status: Option<&str>,
    ) -> AtlasResult<Vec<GoalPlan>> {
        if let Some(s) = status {
            if !VALID_PLAN_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid plan status '{}'. Must be one of: {}", s, VALID_PLAN_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_goal_plans(org_id, status).await
    }

    /// Update goal plan status
    pub async fn update_goal_plan_status(
        &self,
        id: Uuid,
        status: &str,
    ) -> AtlasResult<GoalPlan> {
        if !VALID_PLAN_STATUSES.contains(&status) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid plan status '{}'. Must be one of: {}", status, VALID_PLAN_STATUSES.join(", ")
            )));
        }
        let plan = self.repository.get_goal_plan(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Goal plan {} not found", id)))?;
        // Validate status transitions
        match (plan.status.as_str(), status) {
            ("draft", "active") | ("active", "closed") => {}
            _ => return Err(AtlasError::ValidationFailed(format!(
                "Cannot transition plan from '{}' to '{}'", plan.status, status
            ))),
        }
        info!("Updating goal plan {} status to {}", id, status);
        self.repository.update_goal_plan_status(id, status).await
    }

    /// Delete a goal plan by code
    pub async fn delete_goal_plan(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        info!("Deleting goal plan '{}' for org {}", code, org_id);
        self.repository.delete_goal_plan(org_id, code).await
    }

    // ========================================================================
    // Goals
    // ========================================================================

    /// Create a goal
    #[allow(clippy::too_many_arguments)]
    pub async fn create_goal(
        &self,
        org_id: Uuid,
        plan_id: Option<Uuid>,
        parent_goal_id: Option<Uuid>,
        library_template_id: Option<Uuid>,
        code: Option<&str>,
        name: &str,
        description: Option<&str>,
        goal_type: &str,
        category: Option<&str>,
        owner_id: Uuid,
        owner_type: &str,
        assigned_by: Option<Uuid>,
        success_criteria: Option<&str>,
        target_metric: Option<&str>,
        target_value: Option<&str>,
        uom: Option<&str>,
        weight: Option<&str>,
        priority: &str,
        start_date: Option<chrono::NaiveDate>,
        target_date: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<Goal> {
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Goal name is required".to_string(),
            ));
        }
        if !VALID_GOAL_TYPES.contains(&goal_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid goal_type '{}'. Must be one of: {}", goal_type, VALID_GOAL_TYPES.join(", ")
            )));
        }
        if !VALID_PRIORITIES.contains(&priority) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid priority '{}'. Must be one of: {}", priority, VALID_PRIORITIES.join(", ")
            )));
        }
        if !VALID_OWNER_TYPES.contains(&owner_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid owner_type '{}'. Must be one of: {}", owner_type, VALID_OWNER_TYPES.join(", ")
            )));
        }
        if let Some(w) = weight {
            let w_val: f64 = w.parse().map_err(|_| AtlasError::ValidationFailed(
                "Weight must be a valid number".to_string(),
            ))?;
            if w_val < 0.0 || w_val > 100.0 {
                return Err(AtlasError::ValidationFailed(
                    "Weight must be between 0 and 100".to_string(),
                ));
            }
        }
        if let Some(tv) = target_value {
            if tv.parse::<f64>().is_err() {
                return Err(AtlasError::ValidationFailed(
                    "Target value must be a valid number".to_string(),
                ));
            }
        }
        // Verify parent goal exists if provided
        if let Some(pid) = parent_goal_id {
            let _parent = self.repository.get_goal(pid).await?
                .ok_or_else(|| AtlasError::EntityNotFound(format!(
                    "Parent goal {} not found", pid
                )))?;
        }
        // Verify plan exists and is active if provided
        if let Some(plid) = plan_id {
            let plan = self.repository.get_goal_plan(plid).await?
                .ok_or_else(|| AtlasError::EntityNotFound(format!(
                    "Goal plan {} not found", plid
                )))?;
            if plan.status != "active" {
                return Err(AtlasError::ValidationFailed(
                    "Cannot add goals to a non-active plan".to_string(),
                ));
            }
        }

        info!("Creating goal '{}' for org {} (owner: {})", name, org_id, owner_id);
        self.repository.create_goal(
            org_id, plan_id, parent_goal_id, library_template_id,
            code, name, description, goal_type, category,
            owner_id, owner_type, assigned_by,
            success_criteria, target_metric, target_value, uom,
            weight, priority, start_date, target_date, created_by,
        ).await
    }

    /// Get a goal by ID
    pub async fn get_goal(&self, id: Uuid) -> AtlasResult<Option<Goal>> {
        self.repository.get_goal(id).await
    }

    /// List goals with optional filters
    pub async fn list_goals(
        &self,
        org_id: Uuid,
        plan_id: Option<Uuid>,
        owner_id: Option<Uuid>,
        goal_type: Option<&str>,
        status: Option<&str>,
        parent_goal_id: Option<Uuid>,
    ) -> AtlasResult<Vec<Goal>> {
        if let Some(gt) = goal_type {
            if !VALID_GOAL_TYPES.contains(&gt) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid goal_type '{}'. Must be one of: {}", gt, VALID_GOAL_TYPES.join(", ")
                )));
            }
        }
        if let Some(s) = status {
            if !VALID_GOAL_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_GOAL_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_goals(org_id, plan_id, owner_id, goal_type, status, parent_goal_id).await
    }

    /// Update goal progress
    pub async fn update_goal_progress(
        &self,
        id: Uuid,
        actual_value: Option<&str>,
        progress_pct: Option<&str>,
        status: Option<&str>,
    ) -> AtlasResult<Goal> {
        let goal = self.repository.get_goal(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Goal {} not found", id)))?;

        if let Some(s) = status {
            if !VALID_GOAL_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_GOAL_STATUSES.join(", ")
                )));
            }
        }
        if let Some(p) = progress_pct {
            let p_val: f64 = p.parse().map_err(|_| AtlasError::ValidationFailed(
                "Progress percentage must be a valid number".to_string(),
            ))?;
            if p_val < 0.0 || p_val > 100.0 {
                return Err(AtlasError::ValidationFailed(
                    "Progress must be between 0 and 100".to_string(),
                ));
            }
        }
        if let Some(av) = actual_value {
            if av.parse::<f64>().is_err() {
                return Err(AtlasError::ValidationFailed(
                    "Actual value must be a valid number".to_string(),
                ));
            }
        }

        let completed_date = if status == Some("completed") && goal.completed_date.is_none() {
            Some(chrono::Utc::now().date_naive())
        } else {
            None
        };

        info!("Updating goal {} progress", id);
        self.repository.update_goal_progress(id, actual_value, progress_pct, status, completed_date).await
    }

    /// Delete a goal
    pub async fn delete_goal(&self, id: Uuid) -> AtlasResult<()> {
        info!("Deleting goal {}", id);
        self.repository.delete_goal(id).await
    }

    // ========================================================================
    // Goal Alignments
    // ========================================================================

    /// Create a goal alignment
    pub async fn create_goal_alignment(
        &self,
        org_id: Uuid,
        source_goal_id: Uuid,
        aligned_to_goal_id: Uuid,
        alignment_type: &str,
        description: Option<&str>,
    ) -> AtlasResult<GoalAlignment> {
        if source_goal_id == aligned_to_goal_id {
            return Err(AtlasError::ValidationFailed(
                "A goal cannot align to itself".to_string(),
            ));
        }
        if !VALID_ALIGNMENT_TYPES.contains(&alignment_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid alignment_type '{}'. Must be one of: {}",
                alignment_type, VALID_ALIGNMENT_TYPES.join(", ")
            )));
        }
        // Verify both goals exist
        let _source = self.repository.get_goal(source_goal_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Source goal {} not found", source_goal_id)))?;
        let _target = self.repository.get_goal(aligned_to_goal_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Target goal {} not found", aligned_to_goal_id)))?;

        info!("Creating alignment: {} {} {}", source_goal_id, alignment_type, aligned_to_goal_id);
        self.repository.create_goal_alignment(
            org_id, source_goal_id, aligned_to_goal_id, alignment_type, description,
        ).await
    }

    /// List alignments for a goal
    pub async fn list_goal_alignments(&self, goal_id: Uuid) -> AtlasResult<Vec<GoalAlignment>> {
        self.repository.list_goal_alignments(goal_id).await
    }

    /// Delete a goal alignment
    pub async fn delete_goal_alignment(&self, id: Uuid) -> AtlasResult<()> {
        self.repository.delete_goal_alignment(id).await
    }

    // ========================================================================
    // Goal Notes
    // ========================================================================

    /// Create a goal note
    pub async fn create_goal_note(
        &self,
        org_id: Uuid,
        goal_id: Uuid,
        author_id: Uuid,
        note_type: &str,
        content: &str,
        visibility: &str,
    ) -> AtlasResult<GoalNote> {
        if content.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Note content is required".to_string(),
            ));
        }
        if !VALID_NOTE_TYPES.contains(&note_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid note_type '{}'. Must be one of: {}", note_type, VALID_NOTE_TYPES.join(", ")
            )));
        }
        if !VALID_VISIBILITIES.contains(&visibility) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid visibility '{}'. Must be one of: {}", visibility, VALID_VISIBILITIES.join(", ")
            )));
        }
        // Verify goal exists
        let _goal = self.repository.get_goal(goal_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Goal {} not found", goal_id)))?;

        info!("Adding {} note to goal {}", note_type, goal_id);
        self.repository.create_goal_note(
            org_id, goal_id, author_id, note_type, content, visibility,
        ).await
    }

    /// List notes for a goal
    pub async fn list_goal_notes(&self, goal_id: Uuid) -> AtlasResult<Vec<GoalNote>> {
        self.repository.list_goal_notes(goal_id).await
    }

    /// Delete a goal note
    pub async fn delete_goal_note(&self, id: Uuid) -> AtlasResult<()> {
        self.repository.delete_goal_note(id).await
    }

    // ========================================================================
    // Dashboard Summary
    // ========================================================================

    /// Get the goal management dashboard summary
    pub async fn get_summary(&self, org_id: Uuid) -> AtlasResult<GoalManagementSummary> {
        self.repository.get_summary(org_id).await
    }
}
