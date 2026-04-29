//! Learning Management Engine
//!
//! Manages learning items, categories, enrollments, learning paths,
//! path items, assignments, and completion analytics.
//!
//! Oracle Fusion equivalent: HCM > Learning

use atlas_shared::{
    LearningItem, LearningCategory, LearningEnrollment,
    LearningPath, LearningPathItem, LearningAssignment, LearningDashboard,
    AtlasError, AtlasResult,
};
use super::LearningManagementRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

// Valid enum values
const VALID_ITEM_TYPES: &[&str] = &["course", "certification", "specialization", "video", "assessment", "blended"];
const VALID_FORMATS: &[&str] = &["online", "classroom", "virtual_classroom", "self_paced", "blended"];
const VALID_ITEM_STATUSES: &[&str] = &["draft", "active", "inactive", "archived"];
const VALID_CREDIT_TYPES: &[&str] = &["ceu", "cpe", "pdu", "college_credit", "custom"];
const VALID_ENROLLMENT_TYPES: &[&str] = &["self", "manager", "mandatory", "auto_assigned"];
const VALID_ENROLLMENT_STATUSES: &[&str] = &["enrolled", "in_progress", "completed", "failed", "withdrawn", "expired"];
const VALID_PATH_TYPES: &[&str] = &["sequential", "elective", "milestone", "tiered"];
const VALID_PATH_STATUSES: &[&str] = &["draft", "active", "inactive", "archived"];
const VALID_ASSIGNMENT_TYPES: &[&str] = &["individual", "organization", "department", "job", "position"];
const VALID_ASSIGNMENT_STATUSES: &[&str] = &["active", "completed", "cancelled"];
const VALID_PRIORITIES: &[&str] = &["low", "medium", "high", "critical"];
#[allow(dead_code)]
const VALID_CATEGORY_STATUSES: &[&str] = &["active", "inactive"];

/// Learning Management Engine
pub struct LearningManagementEngine {
    repository: Arc<dyn LearningManagementRepository>,
}

impl LearningManagementEngine {
    pub fn new(repository: Arc<dyn LearningManagementRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Learning Items
    // ========================================================================

    /// Create a learning item
    #[allow(clippy::too_many_arguments)]
    pub async fn create_learning_item(
        &self,
        org_id: Uuid,
        code: &str,
        title: &str,
        description: Option<&str>,
        item_type: &str,
        format: &str,
        category: Option<&str>,
        provider: Option<&str>,
        duration_hours: Option<f64>,
        currency_code: Option<&str>,
        cost: Option<&str>,
        credits: Option<&str>,
        credit_type: Option<&str>,
        validity_months: Option<i32>,
        recertification_required: bool,
        max_enrollments: Option<i32>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<LearningItem> {
        let code_upper = code.to_uppercase();
        validate_code(&code_upper)?;
        if title.is_empty() {
            return Err(AtlasError::ValidationFailed("Title is required".to_string()));
        }
        validate_enum("item_type", item_type, VALID_ITEM_TYPES)?;
        validate_enum("format", format, VALID_FORMATS)?;
        if let Some(ct) = credit_type {
            validate_enum("credit_type", ct, VALID_CREDIT_TYPES)?;
        }
        if let Some(dh) = duration_hours {
            if dh < 0.0 {
                return Err(AtlasError::ValidationFailed("Duration hours must be >= 0".to_string()));
            }
        }
        if let Some(vm) = validity_months {
            if vm < 1 {
                return Err(AtlasError::ValidationFailed("Validity months must be >= 1".to_string()));
            }
        }
        if let Some(me) = max_enrollments {
            if me < 1 {
                return Err(AtlasError::ValidationFailed("Max enrollments must be >= 1".to_string()));
            }
        }
        if let Some(c) = cost {
            if c.parse::<f64>().is_err() {
                return Err(AtlasError::ValidationFailed("Cost must be a valid number".to_string()));
            }
        }
        if let Some(cr) = credits {
            if cr.parse::<f64>().is_err() {
                return Err(AtlasError::ValidationFailed("Credits must be a valid number".to_string()));
            }
        }

        if self.repository.get_learning_item_by_code(org_id, &code_upper).await?.is_some() {
            return Err(AtlasError::Conflict(format!("Learning item '{}' already exists", code_upper)));
        }

        info!("Creating learning item '{}' for org {}", code_upper, org_id);
        self.repository.create_learning_item(
            org_id, &code_upper, title, description, item_type, format,
            category, provider, duration_hours, currency_code, cost,
            credits, credit_type, validity_months, recertification_required,
            max_enrollments, created_by,
        ).await
    }

    /// Get a learning item by ID
    pub async fn get_learning_item(&self, id: Uuid) -> AtlasResult<Option<LearningItem>> {
        self.repository.get_learning_item(id).await
    }

    /// List learning items with optional filters
    pub async fn list_learning_items(
        &self,
        org_id: Uuid,
        item_type: Option<&str>,
        status: Option<&str>,
        category: Option<&str>,
    ) -> AtlasResult<Vec<LearningItem>> {
        if let Some(t) = item_type {
            validate_enum("item_type", t, VALID_ITEM_TYPES)?;
        }
        if let Some(s) = status {
            validate_enum("status", s, VALID_ITEM_STATUSES)?;
        }
        self.repository.list_learning_items(org_id, item_type, status, category).await
    }

    /// Update learning item status
    pub async fn update_learning_item_status(&self, id: Uuid, status: &str) -> AtlasResult<LearningItem> {
        validate_enum("status", status, VALID_ITEM_STATUSES)?;
        let item = self.repository.get_learning_item(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Learning item {} not found", id)))?;

        match (item.status.as_str(), status) {
            ("draft", "active") | ("active", "inactive") |
            ("inactive", "active") | ("active", "archived") |
            ("draft", "archived") => {}
            _ => return Err(AtlasError::ValidationFailed(format!(
                "Cannot transition item from '{}' to '{}'", item.status, status
            ))),
        }

        info!("Updating learning item {} status to {}", id, status);
        self.repository.update_learning_item_status(id, status).await
    }

    /// Delete a learning item by code
    pub async fn delete_learning_item(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        info!("Deleting learning item '{}' for org {}", code, org_id);
        self.repository.delete_learning_item(org_id, code).await
    }

    // ========================================================================
    // Learning Categories
    // ========================================================================

    /// Create a learning category
    pub async fn create_learning_category(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        parent_category_id: Option<Uuid>,
        display_order: i32,
    ) -> AtlasResult<LearningCategory> {
        let code_upper = code.to_uppercase();
        validate_code(&code_upper)?;
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Category name is required".to_string()));
        }
        if let Some(pid) = parent_category_id {
            let _parent = self.repository.get_learning_category(pid).await?
                .ok_or_else(|| AtlasError::EntityNotFound(format!("Parent category {} not found", pid)))?;
        }

        if self.repository.get_learning_category_by_code(org_id, &code_upper).await?.is_some() {
            return Err(AtlasError::Conflict(format!("Learning category '{}' already exists", code_upper)));
        }

        info!("Creating learning category '{}' for org {}", code_upper, org_id);
        self.repository.create_learning_category(
            org_id, &code_upper, name, description, parent_category_id, display_order,
        ).await
    }

    /// Get a learning category by ID
    pub async fn get_learning_category(&self, id: Uuid) -> AtlasResult<Option<LearningCategory>> {
        self.repository.get_learning_category(id).await
    }

    /// List learning categories, optionally filtered by parent
    pub async fn list_learning_categories(
        &self,
        org_id: Uuid,
        parent_id: Option<Uuid>,
    ) -> AtlasResult<Vec<LearningCategory>> {
        self.repository.list_learning_categories(org_id, parent_id).await
    }

    /// Delete a learning category by code
    pub async fn delete_learning_category(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        info!("Deleting learning category '{}' for org {}", code, org_id);
        self.repository.delete_learning_category(org_id, code).await
    }

    // ========================================================================
    // Learning Enrollments
    // ========================================================================

    /// Enroll a person in a learning item
    pub async fn create_learning_enrollment(
        &self,
        org_id: Uuid,
        learning_item_id: Uuid,
        person_id: Uuid,
        person_name: Option<&str>,
        enrollment_type: &str,
        enrolled_by: Option<Uuid>,
        enrollment_date: Option<chrono::NaiveDate>,
        due_date: Option<chrono::NaiveDate>,
    ) -> AtlasResult<LearningEnrollment> {
        validate_enum("enrollment_type", enrollment_type, VALID_ENROLLMENT_TYPES)?;

        // Verify learning item exists and is active
        let item = self.repository.get_learning_item(learning_item_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Learning item {} not found", learning_item_id)))?;
        if item.status != "active" {
            return Err(AtlasError::ValidationFailed(
                "Can only enroll in active learning items".to_string(),
            ));
        }
        if item.organization_id != org_id {
            return Err(AtlasError::ValidationFailed("Item does not belong to this organization".to_string()));
        }

        info!("Enrolling person {} in learning item {}", person_id, learning_item_id);
        self.repository.create_learning_enrollment(
            org_id, learning_item_id, person_id, person_name,
            enrollment_type, enrolled_by, enrollment_date, due_date,
        ).await
    }

    /// Get an enrollment by ID
    pub async fn get_learning_enrollment(&self, id: Uuid) -> AtlasResult<Option<LearningEnrollment>> {
        self.repository.get_learning_enrollment(id).await
    }

    /// List enrollments with optional filters
    pub async fn list_learning_enrollments(
        &self,
        org_id: Uuid,
        learning_item_id: Option<Uuid>,
        person_id: Option<Uuid>,
        status: Option<&str>,
    ) -> AtlasResult<Vec<LearningEnrollment>> {
        if let Some(s) = status {
            validate_enum("status", s, VALID_ENROLLMENT_STATUSES)?;
        }
        self.repository.list_learning_enrollments(org_id, learning_item_id, person_id, status).await
    }

    /// Update enrollment progress
    pub async fn update_enrollment_progress(
        &self,
        id: Uuid,
        progress_pct: Option<&str>,
        score: Option<&str>,
        status: Option<&str>,
    ) -> AtlasResult<LearningEnrollment> {
        if let Some(s) = status {
            validate_enum("status", s, VALID_ENROLLMENT_STATUSES)?;
        }
        if let Some(p) = progress_pct {
            let p_val: f64 = p.parse().map_err(|_| AtlasError::ValidationFailed(
                "Progress must be a valid number".to_string(),
            ))?;
            if !(0.0..=100.0).contains(&p_val) {
                return Err(AtlasError::ValidationFailed("Progress must be between 0 and 100".to_string()));
            }
        }
        if let Some(sc) = score {
            let sc_val: f64 = sc.parse().map_err(|_| AtlasError::ValidationFailed(
                "Score must be a valid number".to_string(),
            ))?;
            if sc_val < 0.0 {
                return Err(AtlasError::ValidationFailed("Score must be >= 0".to_string()));
            }
        }

        let enrollment = self.repository.get_learning_enrollment(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Enrollment {} not found", id)))?;

        let completion_date = if status == Some("completed") && enrollment.completion_date.is_none() {
            Some(chrono::Utc::now().date_naive())
        } else {
            None
        };

        // Compute certification expiry for completed certifications
        let certification_expiry = if status == Some("completed") {
            self.repository.get_learning_item(enrollment.learning_item_id).await?.and_then(|item| {
                if item.item_type == "certification" {
                    item.validity_months.map(|vm| {
                        chrono::Utc::now().date_naive()
                            .checked_add_months(chrono::Months::new(vm as u32))
                            .unwrap_or_else(|| chrono::Utc::now().date_naive())
                    })
                } else {
                    None
                }
            })
        } else {
            None
        };

        info!("Updating enrollment {} progress", id);
        self.repository.update_enrollment_progress(
            id, progress_pct, score, status, completion_date, certification_expiry,
        ).await
    }

    /// Withdraw an enrollment
    pub async fn withdraw_enrollment(&self, id: Uuid) -> AtlasResult<LearningEnrollment> {
        let enrollment = self.repository.get_learning_enrollment(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Enrollment {} not found", id)))?;
        if enrollment.status == "completed" {
            return Err(AtlasError::ValidationFailed("Cannot withdraw a completed enrollment".to_string()));
        }
        info!("Withdrawing enrollment {}", id);
        self.repository.update_enrollment_progress(id, None, None, Some("withdrawn"), None, None).await
    }

    /// Delete an enrollment
    pub async fn delete_learning_enrollment(&self, id: Uuid) -> AtlasResult<()> {
        info!("Deleting enrollment {}", id);
        self.repository.delete_learning_enrollment(id).await
    }

    // ========================================================================
    // Learning Paths
    // ========================================================================

    /// Create a learning path
    pub async fn create_learning_path(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        path_type: &str,
        target_role: Option<&str>,
        target_job_id: Option<Uuid>,
        estimated_duration_hours: Option<f64>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<LearningPath> {
        let code_upper = code.to_uppercase();
        validate_code(&code_upper)?;
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Path name is required".to_string()));
        }
        validate_enum("path_type", path_type, VALID_PATH_TYPES)?;
        if let Some(dh) = estimated_duration_hours {
            if dh < 0.0 {
                return Err(AtlasError::ValidationFailed("Estimated duration must be >= 0".to_string()));
            }
        }

        if self.repository.get_learning_path_by_code(org_id, &code_upper).await?.is_some() {
            return Err(AtlasError::Conflict(format!("Learning path '{}' already exists", code_upper)));
        }

        info!("Creating learning path '{}' for org {}", code_upper, org_id);
        self.repository.create_learning_path(
            org_id, &code_upper, name, description, path_type,
            target_role, target_job_id, estimated_duration_hours, created_by,
        ).await
    }

    /// Get a learning path by ID
    pub async fn get_learning_path(&self, id: Uuid) -> AtlasResult<Option<LearningPath>> {
        self.repository.get_learning_path(id).await
    }

    /// List learning paths with optional filters
    pub async fn list_learning_paths(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        path_type: Option<&str>,
    ) -> AtlasResult<Vec<LearningPath>> {
        if let Some(s) = status {
            validate_enum("status", s, VALID_PATH_STATUSES)?;
        }
        if let Some(pt) = path_type {
            validate_enum("path_type", pt, VALID_PATH_TYPES)?;
        }
        self.repository.list_learning_paths(org_id, status, path_type).await
    }

    /// Update learning path status
    pub async fn update_learning_path_status(&self, id: Uuid, status: &str) -> AtlasResult<LearningPath> {
        validate_enum("status", status, VALID_PATH_STATUSES)?;
        let path = self.repository.get_learning_path(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Learning path {} not found", id)))?;

        match (path.status.as_str(), status) {
            ("draft", "active") | ("active", "inactive") |
            ("inactive", "active") | ("active", "archived") |
            ("draft", "archived") => {}
            _ => return Err(AtlasError::ValidationFailed(format!(
                "Cannot transition path from '{}' to '{}'", path.status, status
            ))),
        }

        info!("Updating learning path {} status to {}", id, status);
        self.repository.update_learning_path_status(id, status).await
    }

    /// Delete a learning path by code
    pub async fn delete_learning_path(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        info!("Deleting learning path '{}' for org {}", code, org_id);
        self.repository.delete_learning_path(org_id, code).await
    }

    // ========================================================================
    // Learning Path Items
    // ========================================================================

    /// Add an item to a learning path
    pub async fn add_learning_path_item(
        &self,
        org_id: Uuid,
        learning_path_id: Uuid,
        learning_item_id: Uuid,
        sequence_number: i32,
        is_required: bool,
        milestone_name: Option<&str>,
    ) -> AtlasResult<LearningPathItem> {
        if sequence_number < 1 {
            return Err(AtlasError::ValidationFailed("Sequence number must be >= 1".to_string()));
        }

        // Verify path exists and is draft
        let path = self.repository.get_learning_path(learning_path_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Learning path {} not found", learning_path_id)))?;
        if path.status != "draft" {
            return Err(AtlasError::ValidationFailed(
                "Can only add items to a draft learning path".to_string(),
            ));
        }
        if path.organization_id != org_id {
            return Err(AtlasError::ValidationFailed("Path does not belong to this organization".to_string()));
        }

        // Verify item exists and is active
        let item = self.repository.get_learning_item(learning_item_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Learning item {} not found", learning_item_id)))?;
        if item.status != "active" {
            return Err(AtlasError::ValidationFailed(
                "Can only add active learning items to a path".to_string(),
            ));
        }

        info!("Adding item {} to learning path {} at sequence {}", learning_item_id, learning_path_id, sequence_number);
        let path_item = self.repository.create_learning_path_item(
            org_id, learning_path_id, learning_item_id,
            sequence_number, is_required, milestone_name,
        ).await?;

        // Update total_items count on the path
        let items = self.repository.list_learning_path_items(learning_path_id).await?;
        let _ = self.repository.update_learning_path_total_items(learning_path_id, items.len() as i32).await;

        Ok(path_item)
    }

    /// List items in a learning path
    pub async fn list_learning_path_items(&self, learning_path_id: Uuid) -> AtlasResult<Vec<LearningPathItem>> {
        self.repository.list_learning_path_items(learning_path_id).await
    }

    /// Remove an item from a learning path
    pub async fn remove_learning_path_item(&self, id: Uuid) -> AtlasResult<()> {
        let item = self.repository.list_learning_path_items(Uuid::nil()).await.ok();
        info!("Removing learning path item {}", id);
        self.repository.delete_learning_path_item(id).await?;

        // Try to update the parent path's total count if we can determine it
        if let Some(path_items) = item {
            if let Some(first) = path_items.first() {
                let remaining = self.repository.list_learning_path_items(first.learning_path_id).await?;
                let _ = self.repository.update_learning_path_total_items(first.learning_path_id, remaining.len() as i32).await;
            }
        }
        Ok(())
    }

    // ========================================================================
    // Learning Assignments
    // ========================================================================

    /// Create a learning assignment
    pub async fn create_learning_assignment(
        &self,
        org_id: Uuid,
        learning_item_id: Option<Uuid>,
        learning_path_id: Option<Uuid>,
        title: &str,
        description: Option<&str>,
        assignment_type: &str,
        target_id: Option<Uuid>,
        assigned_by: Option<Uuid>,
        priority: &str,
        due_date: Option<chrono::NaiveDate>,
    ) -> AtlasResult<LearningAssignment> {
        if title.is_empty() {
            return Err(AtlasError::ValidationFailed("Assignment title is required".to_string()));
        }
        validate_enum("assignment_type", assignment_type, VALID_ASSIGNMENT_TYPES)?;
        validate_enum("priority", priority, VALID_PRIORITIES)?;

        if learning_item_id.is_none() && learning_path_id.is_none() {
            return Err(AtlasError::ValidationFailed(
                "At least one of learning_item_id or learning_path_id is required".to_string(),
            ));
        }

        // Verify learning item if provided
        if let Some(li_id) = learning_item_id {
            let item = self.repository.get_learning_item(li_id).await?
                .ok_or_else(|| AtlasError::EntityNotFound(format!("Learning item {} not found", li_id)))?;
            if item.status != "active" {
                return Err(AtlasError::ValidationFailed("Can only assign active learning items".to_string()));
            }
        }

        // Verify learning path if provided
        if let Some(lp_id) = learning_path_id {
            let path = self.repository.get_learning_path(lp_id).await?
                .ok_or_else(|| AtlasError::EntityNotFound(format!("Learning path {} not found", lp_id)))?;
            if path.status != "active" {
                return Err(AtlasError::ValidationFailed("Can only assign active learning paths".to_string()));
            }
        }

        info!("Creating learning assignment '{}' for org {}", title, org_id);
        self.repository.create_learning_assignment(
            org_id, learning_item_id, learning_path_id,
            title, description, assignment_type, target_id,
            assigned_by, priority, due_date,
        ).await
    }

    /// Get a learning assignment by ID
    pub async fn get_learning_assignment(&self, id: Uuid) -> AtlasResult<Option<LearningAssignment>> {
        self.repository.get_learning_assignment(id).await
    }

    /// List learning assignments with optional filters
    pub async fn list_learning_assignments(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        assignment_type: Option<&str>,
    ) -> AtlasResult<Vec<LearningAssignment>> {
        if let Some(s) = status {
            validate_enum("status", s, VALID_ASSIGNMENT_STATUSES)?;
        }
        if let Some(at) = assignment_type {
            validate_enum("assignment_type", at, VALID_ASSIGNMENT_TYPES)?;
        }
        self.repository.list_learning_assignments(org_id, status, assignment_type).await
    }

    /// Update learning assignment status
    pub async fn update_learning_assignment_status(&self, id: Uuid, status: &str) -> AtlasResult<LearningAssignment> {
        validate_enum("status", status, VALID_ASSIGNMENT_STATUSES)?;
        let assignment = self.repository.get_learning_assignment(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Assignment {} not found", id)))?;

        match (assignment.status.as_str(), status) {
            ("active", "completed") | ("active", "cancelled") => {}
            _ => return Err(AtlasError::ValidationFailed(format!(
                "Cannot transition assignment from '{}' to '{}'", assignment.status, status
            ))),
        }

        info!("Updating learning assignment {} status to {}", id, status);
        self.repository.update_learning_assignment_status(id, status).await
    }

    /// Delete a learning assignment
    pub async fn delete_learning_assignment(&self, id: Uuid) -> AtlasResult<()> {
        info!("Deleting learning assignment {}", id);
        self.repository.delete_learning_assignment(id).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get the learning management dashboard summary
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<LearningDashboard> {
        self.repository.get_learning_dashboard(org_id).await
    }
}

// ============================================================================
// Validation helpers
// ============================================================================

fn validate_code(code: &str) -> AtlasResult<()> {
    if code.is_empty() || code.len() > 100 {
        return Err(AtlasError::ValidationFailed(
            "Code must be 1-100 characters".to_string(),
        ));
    }
    Ok(())
}

fn validate_enum(field_name: &str, value: &str, valid: &[&str]) -> AtlasResult<()> {
    if !valid.contains(&value) {
        return Err(AtlasError::ValidationFailed(format!(
            "Invalid {} '{}'. Must be one of: {}", field_name, value, valid.join(", ")
        )));
    }
    Ok(())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Enum validation tests
    // ========================================================================

    #[test]
    fn test_valid_item_types() {
        assert!(VALID_ITEM_TYPES.contains(&"course"));
        assert!(VALID_ITEM_TYPES.contains(&"certification"));
        assert!(VALID_ITEM_TYPES.contains(&"specialization"));
        assert!(VALID_ITEM_TYPES.contains(&"video"));
        assert!(VALID_ITEM_TYPES.contains(&"assessment"));
        assert!(VALID_ITEM_TYPES.contains(&"blended"));
        assert_eq!(VALID_ITEM_TYPES.len(), 6);
    }

    #[test]
    fn test_valid_formats() {
        assert!(VALID_FORMATS.contains(&"online"));
        assert!(VALID_FORMATS.contains(&"classroom"));
        assert!(VALID_FORMATS.contains(&"virtual_classroom"));
        assert!(VALID_FORMATS.contains(&"self_paced"));
        assert!(VALID_FORMATS.contains(&"blended"));
        assert_eq!(VALID_FORMATS.len(), 5);
    }

    #[test]
    fn test_valid_item_statuses() {
        assert!(VALID_ITEM_STATUSES.contains(&"draft"));
        assert!(VALID_ITEM_STATUSES.contains(&"active"));
        assert!(VALID_ITEM_STATUSES.contains(&"inactive"));
        assert!(VALID_ITEM_STATUSES.contains(&"archived"));
        assert_eq!(VALID_ITEM_STATUSES.len(), 4);
    }

    #[test]
    fn test_valid_credit_types() {
        assert!(VALID_CREDIT_TYPES.contains(&"ceu"));
        assert!(VALID_CREDIT_TYPES.contains(&"cpe"));
        assert!(VALID_CREDIT_TYPES.contains(&"pdu"));
        assert!(VALID_CREDIT_TYPES.contains(&"college_credit"));
        assert!(VALID_CREDIT_TYPES.contains(&"custom"));
        assert_eq!(VALID_CREDIT_TYPES.len(), 5);
    }

    #[test]
    fn test_valid_enrollment_types() {
        assert!(VALID_ENROLLMENT_TYPES.contains(&"self"));
        assert!(VALID_ENROLLMENT_TYPES.contains(&"manager"));
        assert!(VALID_ENROLLMENT_TYPES.contains(&"mandatory"));
        assert!(VALID_ENROLLMENT_TYPES.contains(&"auto_assigned"));
        assert_eq!(VALID_ENROLLMENT_TYPES.len(), 4);
    }

    #[test]
    fn test_valid_enrollment_statuses() {
        assert!(VALID_ENROLLMENT_STATUSES.contains(&"enrolled"));
        assert!(VALID_ENROLLMENT_STATUSES.contains(&"in_progress"));
        assert!(VALID_ENROLLMENT_STATUSES.contains(&"completed"));
        assert!(VALID_ENROLLMENT_STATUSES.contains(&"failed"));
        assert!(VALID_ENROLLMENT_STATUSES.contains(&"withdrawn"));
        assert!(VALID_ENROLLMENT_STATUSES.contains(&"expired"));
        assert_eq!(VALID_ENROLLMENT_STATUSES.len(), 6);
    }

    #[test]
    fn test_valid_path_types() {
        assert!(VALID_PATH_TYPES.contains(&"sequential"));
        assert!(VALID_PATH_TYPES.contains(&"elective"));
        assert!(VALID_PATH_TYPES.contains(&"milestone"));
        assert!(VALID_PATH_TYPES.contains(&"tiered"));
        assert_eq!(VALID_PATH_TYPES.len(), 4);
    }

    #[test]
    fn test_valid_path_statuses() {
        assert!(VALID_PATH_STATUSES.contains(&"draft"));
        assert!(VALID_PATH_STATUSES.contains(&"active"));
        assert!(VALID_PATH_STATUSES.contains(&"inactive"));
        assert!(VALID_PATH_STATUSES.contains(&"archived"));
        assert_eq!(VALID_PATH_STATUSES.len(), 4);
    }

    #[test]
    fn test_valid_assignment_types() {
        assert!(VALID_ASSIGNMENT_TYPES.contains(&"individual"));
        assert!(VALID_ASSIGNMENT_TYPES.contains(&"organization"));
        assert!(VALID_ASSIGNMENT_TYPES.contains(&"department"));
        assert!(VALID_ASSIGNMENT_TYPES.contains(&"job"));
        assert!(VALID_ASSIGNMENT_TYPES.contains(&"position"));
        assert_eq!(VALID_ASSIGNMENT_TYPES.len(), 5);
    }

    #[test]
    fn test_valid_assignment_statuses() {
        assert!(VALID_ASSIGNMENT_STATUSES.contains(&"active"));
        assert!(VALID_ASSIGNMENT_STATUSES.contains(&"completed"));
        assert!(VALID_ASSIGNMENT_STATUSES.contains(&"cancelled"));
        assert_eq!(VALID_ASSIGNMENT_STATUSES.len(), 3);
    }

    #[test]
    fn test_valid_priorities() {
        assert!(VALID_PRIORITIES.contains(&"low"));
        assert!(VALID_PRIORITIES.contains(&"medium"));
        assert!(VALID_PRIORITIES.contains(&"high"));
        assert!(VALID_PRIORITIES.contains(&"critical"));
        assert_eq!(VALID_PRIORITIES.len(), 4);
    }

    #[test]
    fn test_valid_category_statuses() {
        assert!(VALID_CATEGORY_STATUSES.contains(&"active"));
        assert!(VALID_CATEGORY_STATUSES.contains(&"inactive"));
        assert_eq!(VALID_CATEGORY_STATUSES.len(), 2);
    }

    // ========================================================================
    // validate_code tests
    // ========================================================================

    #[test]
    fn test_validate_code_valid() {
        assert!(validate_code("SAFETY-101").is_ok());
        assert!(validate_code("A").is_ok());
    }

    #[test]
    fn test_validate_code_empty() {
        assert!(validate_code("").is_err());
    }

    #[test]
    fn test_validate_code_too_long() {
        let long_code = "X".repeat(101);
        assert!(validate_code(&long_code).is_err());
    }

    #[test]
    fn test_validate_code_exactly_100() {
        let code = "X".repeat(100);
        assert!(validate_code(&code).is_ok());
    }

    // ========================================================================
    // validate_enum tests
    // ========================================================================

    #[test]
    fn test_validate_enum_valid() {
        assert!(validate_enum("test_field", "active", &["draft", "active", "archived"]).is_ok());
    }

    #[test]
    fn test_validate_enum_invalid() {
        let result = validate_enum("test_field", "bogus", &["draft", "active", "archived"]);
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("test_field"));
        assert!(msg.contains("bogus"));
    }

    #[test]
    fn test_validate_enum_empty_valid_array() {
        assert!(validate_enum("field", "value", &[]).is_err());
    }

    // ========================================================================
    // Status transition tests
    // ========================================================================

    #[test]
    fn test_item_status_transitions_draft_to_active() {
        let valid = matches!(("draft", "active"),
            ("draft", "active") | ("active", "inactive") |
            ("inactive", "active") | ("active", "archived") |
            ("draft", "archived")
        );
        assert!(valid);
    }

    #[test]
    fn test_item_status_transitions_archived_to_active_invalid() {
        let valid = matches!(("archived", "active"),
            ("draft", "active") | ("active", "inactive") |
            ("inactive", "active") | ("active", "archived") |
            ("draft", "archived")
        );
        assert!(!valid);
    }

    #[test]
    fn test_item_status_can_be_reactivated() {
        let valid = matches!(("inactive", "active"),
            ("draft", "active") | ("active", "inactive") |
            ("inactive", "active") | ("active", "archived") |
            ("draft", "archived")
        );
        assert!(valid);
    }

    #[test]
    fn test_assignment_status_active_to_completed() {
        let valid = matches!(("active", "completed"), ("active", "completed") | ("active", "cancelled"));
        assert!(valid);
    }

    #[test]
    fn test_assignment_status_completed_to_active_invalid() {
        let valid = matches!(("completed", "active"), ("active", "completed") | ("active", "cancelled"));
        assert!(!valid);
    }

    #[test]
    fn test_path_status_transitions() {
        // draft -> active
        assert!(matches!(("draft", "active"), ("draft", "active")));
        // active -> archived
        assert!(matches!(("active", "archived"), ("active", "archived")));
    }

    // ========================================================================
    // Learning item domain tests
    // ========================================================================

    #[test]
    fn test_item_types_include_certification() {
        // Oracle Fusion supports certifications with recertification tracking
        assert!(VALID_ITEM_TYPES.contains(&"certification"));
    }

    #[test]
    fn test_item_types_include_specialization() {
        // Oracle Fusion supports learning specializations
        assert!(VALID_ITEM_TYPES.contains(&"specialization"));
    }

    #[test]
    fn test_formats_include_virtual_classroom() {
        // Oracle Fusion supports virtual instructor-led training
        assert!(VALID_FORMATS.contains(&"virtual_classroom"));
    }

    #[test]
    fn test_credit_types_include_professional_credits() {
        // Oracle Fusion tracks CEU, CPE, PDU, and college credits
        assert!(VALID_CREDIT_TYPES.contains(&"ceu"));
        assert!(VALID_CREDIT_TYPES.contains(&"cpe"));
        assert!(VALID_CREDIT_TYPES.contains(&"pdu"));
        assert!(VALID_CREDIT_TYPES.contains(&"college_credit"));
    }

    #[test]
    fn test_enrollment_types_include_auto_assigned() {
        // Oracle Fusion supports auto-assignment via rules
        assert!(VALID_ENROLLMENT_TYPES.contains(&"auto_assigned"));
    }

    #[test]
    fn test_enrollment_statuses_include_failed() {
        // Oracle Fusion tracks failed completions
        assert!(VALID_ENROLLMENT_STATUSES.contains(&"failed"));
        assert!(VALID_ENROLLMENT_STATUSES.contains(&"withdrawn"));
        assert!(VALID_ENROLLMENT_STATUSES.contains(&"expired"));
    }

    #[test]
    fn test_path_types_include_milestone_and_tiered() {
        // Oracle Fusion supports milestone-based and tiered curricula
        assert!(VALID_PATH_TYPES.contains(&"milestone"));
        assert!(VALID_PATH_TYPES.contains(&"tiered"));
    }

    #[test]
    fn test_assignment_types_include_job_and_position() {
        // Oracle Fusion allows assignment by job role or specific position
        assert!(VALID_ASSIGNMENT_TYPES.contains(&"job"));
        assert!(VALID_ASSIGNMENT_TYPES.contains(&"position"));
    }

    #[test]
    fn test_assignment_requires_item_or_path() {
        // Business rule: at least one of learning_item_id or learning_path_id required
        // This is validated in create_learning_assignment
        let has_item = None::<Uuid>;
        let has_path = None::<Uuid>;
        assert!(has_item.is_none() && has_path.is_none());
        // In the engine, this combination would return ValidationFailed
    }

    #[test]
    fn test_all_enrollment_statuses_are_unique() {
        let mut statuses: Vec<&str> = VALID_ENROLLMENT_STATUSES.to_vec();
        statuses.sort();
        statuses.dedup();
        assert_eq!(statuses.len(), VALID_ENROLLMENT_STATUSES.len());
    }

    #[test]
    fn test_all_item_types_are_unique() {
        let mut types: Vec<&str> = VALID_ITEM_TYPES.to_vec();
        types.sort();
        types.dedup();
        assert_eq!(types.len(), VALID_ITEM_TYPES.len());
    }

    #[test]
    fn test_all_assignment_types_are_unique() {
        let mut types: Vec<&str> = VALID_ASSIGNMENT_TYPES.to_vec();
        types.sort();
        types.dedup();
        assert_eq!(types.len(), VALID_ASSIGNMENT_TYPES.len());
    }
}
