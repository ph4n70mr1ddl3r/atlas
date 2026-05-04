//! Project Resource Management Engine
//!
//! Manages resource profiles, requests, assignments, and utilization tracking
//! for project staffing in Oracle Fusion style.
//!
//! Oracle Fusion Cloud equivalent: Project Management > Resource Management

use atlas_shared::{
    ResourceProfile, ResourceRequest, ResourceAssignment, UtilizationEntry,
    ResourceDashboard, AtlasError, AtlasResult,
};
use super::ProjectResourceManagementRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

// ============================================================================
// Valid enum constants
// ============================================================================

#[allow(dead_code)]
const VALID_RESOURCE_TYPES: &[&str] = &["employee", "contractor"];

#[allow(dead_code)]
const VALID_AVAILABILITY_STATUSES: &[&str] = &[
    "available", "partially_available", "fully_allocated", "on_leave",
];

#[allow(dead_code)]
const VALID_REQUEST_PRIORITIES: &[&str] = &[
    "low", "medium", "high", "critical",
];

#[allow(dead_code)]
const VALID_REQUEST_STATUSES: &[&str] = &[
    "draft", "submitted", "fulfilled", "partially_fulfilled", "cancelled",
];

#[allow(dead_code)]
const VALID_RESOURCE_TYPE_PREFERENCES: &[&str] = &[
    "any", "employee_only", "contractor_only",
];

#[allow(dead_code)]
const VALID_ASSIGNMENT_STATUSES: &[&str] = &[
    "planned", "active", "completed", "cancelled",
];

#[allow(dead_code)]
const VALID_UTILIZATION_STATUSES: &[&str] = &[
    "submitted", "approved", "rejected",
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

/// Project Resource Management Engine
pub struct ProjectResourceManagementEngine {
    repository: Arc<dyn ProjectResourceManagementRepository>,
}

impl ProjectResourceManagementEngine {
    pub fn new(repository: Arc<dyn ProjectResourceManagementRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Resource Profiles
    // ========================================================================

    /// Create a new resource profile
    #[allow(clippy::too_many_arguments)]
    pub async fn create_profile(
        &self,
        org_id: Uuid,
        resource_number: &str,
        name: &str,
        email: &str,
        resource_type: &str,
        department: &str,
        job_title: &str,
        skills: Option<&str>,
        certifications: Option<&str>,
        availability_status: Option<&str>,
        available_hours_per_week: Option<f64>,
        cost_rate: Option<f64>,
        cost_rate_currency: Option<&str>,
        bill_rate: Option<f64>,
        bill_rate_currency: Option<&str>,
        location: Option<&str>,
        manager_id: Option<Uuid>,
        manager_name: Option<&str>,
        hire_date: Option<chrono::NaiveDate>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ResourceProfile> {
        if resource_number.is_empty() {
            return Err(AtlasError::ValidationFailed("Resource number is required".to_string()));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Resource name is required".to_string()));
        }
        validate_enum("resource_type", resource_type, VALID_RESOURCE_TYPES)?;

        if let Some(status) = availability_status {
            validate_enum("availability_status", status, VALID_AVAILABILITY_STATUSES)?;
        }

        if let Some(hw) = available_hours_per_week {
            if !(0.0..=168.0).contains(&hw) {
                return Err(AtlasError::ValidationFailed(
                    "Available hours per week must be between 0 and 168".to_string(),
                ));
            }
        }

        if let Some(cr) = cost_rate {
            if cr < 0.0 {
                return Err(AtlasError::ValidationFailed(
                    "Cost rate cannot be negative".to_string(),
                ));
            }
        }

        if let Some(br) = bill_rate {
            if br < 0.0 {
                return Err(AtlasError::ValidationFailed(
                    "Bill rate cannot be negative".to_string(),
                ));
            }
        }

        if self.repository.get_profile_by_number(org_id, resource_number).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Resource profile '{}' already exists", resource_number
            )));
        }

        info!("Creating resource profile '{}' ({}) for org {} [type={}, cost_rate={:?}, bill_rate={:?}]",
              resource_number, name, org_id, resource_type, cost_rate, bill_rate);

        self.repository.create_profile(
            org_id, resource_number, name, email,
            resource_type, department, job_title,
            skills.unwrap_or(""), certifications.unwrap_or(""),
            availability_status.unwrap_or("available"),
            available_hours_per_week.unwrap_or(40.0),
            cost_rate.unwrap_or(0.0),
            cost_rate_currency.unwrap_or("USD"),
            bill_rate.unwrap_or(0.0),
            bill_rate_currency.unwrap_or("USD"),
            location.unwrap_or(""),
            manager_id,
            manager_name.unwrap_or(""),
            hire_date,
            notes.unwrap_or(""),
            created_by,
        ).await
    }

    /// Get a profile by ID
    pub async fn get_profile(&self, id: Uuid) -> AtlasResult<Option<ResourceProfile>> {
        self.repository.get_profile(id).await
    }

    /// Get a profile by number
    pub async fn get_profile_by_number(&self, org_id: Uuid, resource_number: &str) -> AtlasResult<Option<ResourceProfile>> {
        self.repository.get_profile_by_number(org_id, resource_number).await
    }

    /// List profiles with optional filters
    pub async fn list_profiles(
        &self,
        org_id: Uuid,
        availability_status: Option<&str>,
        resource_type: Option<&str>,
        department: Option<&str>,
    ) -> AtlasResult<Vec<ResourceProfile>> {
        self.repository.list_profiles(org_id, availability_status, resource_type, department).await
    }

    /// Update availability status
    pub async fn update_availability(&self, id: Uuid, status: &str) -> AtlasResult<ResourceProfile> {
        validate_enum("availability_status", status, VALID_AVAILABILITY_STATUSES)?;
        info!("Updating resource {} availability to {}", id, status);
        self.repository.update_availability_status(id, status).await
    }

    /// Delete a profile by number
    pub async fn delete_profile(&self, org_id: Uuid, resource_number: &str) -> AtlasResult<()> {
        info!("Deleting resource profile '{}' for org {}", resource_number, org_id);
        self.repository.delete_profile(org_id, resource_number).await
    }

    // ========================================================================
    // Resource Requests
    // ========================================================================

    /// Create a new resource request
    #[allow(clippy::too_many_arguments)]
    pub async fn create_request(
        &self,
        org_id: Uuid,
        request_number: &str,
        project_id: Option<Uuid>,
        project_name: Option<&str>,
        project_number: Option<&str>,
        requested_role: &str,
        required_skills: Option<&str>,
        priority: &str,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
        hours_per_week: f64,
        total_planned_hours: f64,
        max_cost_rate: Option<f64>,
        currency_code: Option<&str>,
        resource_type_preference: Option<&str>,
        location_requirement: Option<&str>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ResourceRequest> {
        if request_number.is_empty() {
            return Err(AtlasError::ValidationFailed("Request number is required".to_string()));
        }
        validate_enum("priority", priority, VALID_REQUEST_PRIORITIES)?;

        if let Some(rtp) = resource_type_preference {
            validate_enum("resource_type_preference", rtp, VALID_RESOURCE_TYPE_PREFERENCES)?;
        }

        if start_date >= end_date {
            return Err(AtlasError::ValidationFailed(
                "Start date must be before end date".to_string(),
            ));
        }

        if hours_per_week <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Hours per week must be positive".to_string(),
            ));
        }

        if let Some(mcr) = max_cost_rate {
            if mcr < 0.0 {
                return Err(AtlasError::ValidationFailed(
                    "Max cost rate cannot be negative".to_string(),
                ));
            }
        }

        if self.repository.get_request_by_number(org_id, request_number).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Resource request '{}' already exists", request_number
            )));
        }

        info!("Creating resource request '{}' for org {} [role={}, priority={}]",
              request_number, org_id, requested_role, priority);

        self.repository.create_request(
            org_id, request_number, project_id,
            project_name.unwrap_or(""), project_number.unwrap_or(""),
            requested_role, required_skills.unwrap_or(""), priority,
            start_date, end_date, hours_per_week, total_planned_hours,
            max_cost_rate, currency_code.unwrap_or("USD"),
            resource_type_preference.unwrap_or("any"),
            location_requirement.unwrap_or(""), notes.unwrap_or(""), created_by,
        ).await
    }

    /// Get a request by ID
    pub async fn get_request(&self, id: Uuid) -> AtlasResult<Option<ResourceRequest>> {
        self.repository.get_request(id).await
    }

    /// List requests with optional filters
    pub async fn list_requests(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        priority: Option<&str>,
        project_id: Option<Uuid>,
    ) -> AtlasResult<Vec<ResourceRequest>> {
        self.repository.list_requests(org_id, status, priority, project_id).await
    }

    /// Submit a draft request
    pub async fn submit_request(&self, id: Uuid) -> AtlasResult<ResourceRequest> {
        info!("Submitting resource request {}", id);
        self.repository.update_request_status(id, "submitted").await
    }

    /// Fulfill a request
    pub async fn fulfill_request(&self, id: Uuid, fulfilled_by: Uuid) -> AtlasResult<ResourceRequest> {
        info!("Fulfilling resource request {} by {}", id, fulfilled_by);
        self.repository.fulfill_request(id, fulfilled_by).await
    }

    /// Cancel a request
    pub async fn cancel_request(&self, id: Uuid) -> AtlasResult<ResourceRequest> {
        info!("Cancelling resource request {}", id);
        self.repository.update_request_status(id, "cancelled").await
    }

    /// Delete a request by number (only drafts)
    pub async fn delete_request(&self, org_id: Uuid, request_number: &str) -> AtlasResult<()> {
        info!("Deleting resource request '{}' for org {}", request_number, org_id);
        self.repository.delete_request(org_id, request_number).await
    }

    // ========================================================================
    // Resource Assignments
    // ========================================================================

    /// Create a new resource assignment
    #[allow(clippy::too_many_arguments)]
    pub async fn create_assignment(
        &self,
        org_id: Uuid,
        assignment_number: &str,
        resource_id: Uuid,
        resource_name: Option<&str>,
        resource_email: Option<&str>,
        project_id: Option<Uuid>,
        project_name: Option<&str>,
        project_number: Option<&str>,
        request_id: Option<Uuid>,
        role: &str,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
        planned_hours: f64,
        cost_rate: Option<f64>,
        bill_rate: Option<f64>,
        currency_code: Option<&str>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ResourceAssignment> {
        if assignment_number.is_empty() {
            return Err(AtlasError::ValidationFailed("Assignment number is required".to_string()));
        }

        if start_date >= end_date {
            return Err(AtlasError::ValidationFailed(
                "Start date must be before end date".to_string(),
            ));
        }

        if planned_hours < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Planned hours cannot be negative".to_string(),
            ));
        }

        if let Some(cr) = cost_rate {
            if cr < 0.0 {
                return Err(AtlasError::ValidationFailed(
                    "Cost rate cannot be negative".to_string(),
                ));
            }
        }

        if let Some(br) = bill_rate {
            if br < 0.0 {
                return Err(AtlasError::ValidationFailed(
                    "Bill rate cannot be negative".to_string(),
                ));
            }
        }

        // Verify resource exists
        let resource = self.repository.get_profile(resource_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Resource profile {} not found", resource_id
            )))?;

        if self.repository.get_assignment_by_number(org_id, assignment_number).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Resource assignment '{}' already exists", assignment_number
            )));
        }

        info!("Creating assignment '{}' for resource {} on project [role={}]",
              assignment_number, resource.name, role);

        self.repository.create_assignment(
            org_id, assignment_number, resource_id,
            resource_name.unwrap_or(&resource.name),
            resource_email.unwrap_or(&resource.email),
            project_id, project_name.unwrap_or(""), project_number.unwrap_or(""),
            request_id, role,
            start_date, end_date, planned_hours,
            cost_rate.unwrap_or(resource.cost_rate),
            bill_rate.unwrap_or(resource.bill_rate),
            currency_code.unwrap_or(&resource.cost_rate_currency),
            notes.unwrap_or(""), created_by,
        ).await
    }

    /// Get an assignment by ID
    pub async fn get_assignment(&self, id: Uuid) -> AtlasResult<Option<ResourceAssignment>> {
        self.repository.get_assignment(id).await
    }

    /// List assignments with optional filters
    pub async fn list_assignments(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        resource_id: Option<Uuid>,
        project_id: Option<Uuid>,
    ) -> AtlasResult<Vec<ResourceAssignment>> {
        self.repository.list_assignments(org_id, status, resource_id, project_id).await
    }

    /// Activate a planned assignment
    pub async fn activate_assignment(&self, id: Uuid) -> AtlasResult<ResourceAssignment> {
        info!("Activating resource assignment {}", id);
        self.repository.update_assignment_status(id, "active").await
    }

    /// Complete an assignment
    pub async fn complete_assignment(&self, id: Uuid) -> AtlasResult<ResourceAssignment> {
        info!("Completing resource assignment {}", id);
        self.repository.update_assignment_status(id, "completed").await
    }

    /// Cancel an assignment
    pub async fn cancel_assignment(&self, id: Uuid) -> AtlasResult<ResourceAssignment> {
        info!("Cancelling resource assignment {}", id);
        self.repository.update_assignment_status(id, "cancelled").await
    }

    /// Delete an assignment by number
    pub async fn delete_assignment(&self, org_id: Uuid, assignment_number: &str) -> AtlasResult<()> {
        info!("Deleting resource assignment '{}' for org {}", assignment_number, org_id);
        self.repository.delete_assignment(org_id, assignment_number).await
    }

    // ========================================================================
    // Utilization Entries
    // ========================================================================

    /// Record a utilization entry
    pub async fn create_utilization_entry(
        &self,
        org_id: Uuid,
        assignment_id: Uuid,
        resource_id: Uuid,
        entry_date: chrono::NaiveDate,
        hours_worked: f64,
        description: Option<&str>,
        billable: Option<bool>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<UtilizationEntry> {
        if hours_worked <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Hours worked must be positive".to_string(),
            ));
        }
        if hours_worked > 24.0 {
            return Err(AtlasError::ValidationFailed(
                "Hours worked cannot exceed 24 in a single entry".to_string(),
            ));
        }

        // Verify assignment exists
        let assignment = self.repository.get_assignment(assignment_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Resource assignment {} not found", assignment_id
            )))?;

        if assignment.status == "cancelled" {
            return Err(AtlasError::ValidationFailed(
                "Cannot add utilization to a cancelled assignment".to_string(),
            ));
        }

        info!("Creating utilization entry for assignment {} [hours={}, date={}]",
              assignment_id, hours_worked, entry_date);

        let entry = self.repository.create_utilization_entry(
            org_id, assignment_id, resource_id,
            entry_date, hours_worked,
            description.unwrap_or(""), billable.unwrap_or(true),
            notes.unwrap_or(""), created_by,
        ).await?;

        // Update assignment actual hours and remaining hours
        let new_actual = assignment.actual_hours + hours_worked;
        let remaining = (assignment.planned_hours - new_actual).max(0.0);
        let utilization = if assignment.planned_hours > 0.0 {
            (new_actual / assignment.planned_hours * 100.0).min(999.99)
        } else {
            0.0
        };
        self.repository.update_assignment_hours(
            assignment_id, new_actual, remaining, utilization,
        ).await.ok();

        Ok(entry)
    }

    /// Get a utilization entry by ID
    pub async fn get_utilization_entry(&self, id: Uuid) -> AtlasResult<Option<UtilizationEntry>> {
        self.repository.get_utilization_entry(id).await
    }

    /// List utilization entries with optional filters
    pub async fn list_utilization_entries(
        &self,
        org_id: Uuid,
        assignment_id: Option<Uuid>,
        resource_id: Option<Uuid>,
        status: Option<&str>,
    ) -> AtlasResult<Vec<UtilizationEntry>> {
        self.repository.list_utilization_entries(org_id, assignment_id, resource_id, status).await
    }

    /// Approve a utilization entry
    pub async fn approve_utilization_entry(&self, id: Uuid, approved_by: Uuid) -> AtlasResult<UtilizationEntry> {
        info!("Approving utilization entry {} by {}", id, approved_by);
        self.repository.approve_utilization_entry(id, approved_by).await
    }

    /// Reject a utilization entry
    pub async fn reject_utilization_entry(&self, id: Uuid) -> AtlasResult<UtilizationEntry> {
        info!("Rejecting utilization entry {}", id);
        self.repository.update_utilization_status(id, "rejected").await
    }

    /// Delete a utilization entry
    pub async fn delete_utilization_entry(&self, id: Uuid) -> AtlasResult<()> {
        info!("Deleting utilization entry {}", id);
        self.repository.delete_utilization_entry(id).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get the resource management dashboard
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<ResourceDashboard> {
        self.repository.get_dashboard(org_id).await
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_resource_types() {
        assert!(VALID_RESOURCE_TYPES.contains(&"employee"));
        assert!(VALID_RESOURCE_TYPES.contains(&"contractor"));
        assert!(!VALID_RESOURCE_TYPES.contains(&"vendor"));
    }

    #[test]
    fn test_valid_availability_statuses() {
        assert!(VALID_AVAILABILITY_STATUSES.contains(&"available"));
        assert!(VALID_AVAILABILITY_STATUSES.contains(&"partially_available"));
        assert!(VALID_AVAILABILITY_STATUSES.contains(&"fully_allocated"));
        assert!(VALID_AVAILABILITY_STATUSES.contains(&"on_leave"));
    }

    #[test]
    fn test_valid_request_priorities() {
        assert!(VALID_REQUEST_PRIORITIES.contains(&"low"));
        assert!(VALID_REQUEST_PRIORITIES.contains(&"medium"));
        assert!(VALID_REQUEST_PRIORITIES.contains(&"high"));
        assert!(VALID_REQUEST_PRIORITIES.contains(&"critical"));
    }

    #[test]
    fn test_valid_request_statuses() {
        assert!(VALID_REQUEST_STATUSES.contains(&"draft"));
        assert!(VALID_REQUEST_STATUSES.contains(&"submitted"));
        assert!(VALID_REQUEST_STATUSES.contains(&"fulfilled"));
        assert!(VALID_REQUEST_STATUSES.contains(&"partially_fulfilled"));
        assert!(VALID_REQUEST_STATUSES.contains(&"cancelled"));
    }

    #[test]
    fn test_valid_assignment_statuses() {
        assert!(VALID_ASSIGNMENT_STATUSES.contains(&"planned"));
        assert!(VALID_ASSIGNMENT_STATUSES.contains(&"active"));
        assert!(VALID_ASSIGNMENT_STATUSES.contains(&"completed"));
        assert!(VALID_ASSIGNMENT_STATUSES.contains(&"cancelled"));
    }

    #[test]
    fn test_valid_utilization_statuses() {
        assert!(VALID_UTILIZATION_STATUSES.contains(&"submitted"));
        assert!(VALID_UTILIZATION_STATUSES.contains(&"approved"));
        assert!(VALID_UTILIZATION_STATUSES.contains(&"rejected"));
    }

    #[test]
    fn test_validate_enum_valid() {
        assert!(validate_enum("resource_type", "employee", VALID_RESOURCE_TYPES).is_ok());
    }

    #[test]
    fn test_validate_enum_invalid() {
        let result = validate_enum("resource_type", "invalid", VALID_RESOURCE_TYPES);
        assert!(result.is_err());
        match result {
            Err(AtlasError::ValidationFailed(msg)) => {
                assert!(msg.contains("resource_type"));
                assert!(msg.contains("invalid"));
            }
            _ => panic!("Expected ValidationFailed error"),
        }
    }

    #[test]
    fn test_validate_enum_empty() {
        let result = validate_enum("resource_type", "", VALID_RESOURCE_TYPES);
        assert!(result.is_err());
        match result {
            Err(AtlasError::ValidationFailed(msg)) => {
                assert!(msg.contains("required"));
            }
            _ => panic!("Expected ValidationFailed error"),
        }
    }
}
