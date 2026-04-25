//! Service Request Engine Implementation
//!
//! Manages service request categories, request lifecycle,
//! assignments, communications/updates, SLA tracking,
//! and resolution.
//!
//! Oracle Fusion CX Service equivalent: Service Requests

use atlas_shared::{
    ServiceCategory, ServiceRequest, ServiceRequestUpdate,
    ServiceRequestAssignment, ServiceRequestDashboard,
    AtlasError, AtlasResult,
};
use super::ServiceRequestRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Valid priorities for service requests
#[allow(dead_code)]
const VALID_PRIORITIES: &[&str] = &[
    "low", "medium", "high", "critical",
];

/// Valid statuses for service requests
#[allow(dead_code)]
const VALID_STATUSES: &[&str] = &[
    "open", "in_progress", "pending_customer", "resolved", "closed", "cancelled",
];

/// Valid request types
#[allow(dead_code)]
const VALID_REQUEST_TYPES: &[&str] = &[
    "incident", "service_request", "problem", "change_request",
];

/// Valid channels through which a request can arrive
#[allow(dead_code)]
const VALID_CHANNELS: &[&str] = &[
    "phone", "email", "web", "chat", "social_media", "walk_in", "api",
];

/// Valid update types for communications
#[allow(dead_code)]
const VALID_UPDATE_TYPES: &[&str] = &[
    "comment", "email", "phone_call", "note", "system",
];

/// Valid assignment types
#[allow(dead_code)]
const VALID_ASSIGNMENT_TYPES: &[&str] = &[
    "initial", "transfer", "escalation", "reassignment",
];

/// Valid resolution codes
#[allow(dead_code)]
const VALID_RESOLUTION_CODES: &[&str] = &[
    "resolved", "workaround", "no_fault_found", "duplicate",
    "out_of_scope", "cancelled_by_customer", "escalated",
];

/// Service Request engine for managing service cases
pub struct ServiceRequestEngine {
    repository: Arc<dyn ServiceRequestRepository>,
}

impl ServiceRequestEngine {
    pub fn new(repository: Arc<dyn ServiceRequestRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Category Management
    // ========================================================================

    /// Create a new service category
    pub async fn create_category(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        parent_category_id: Option<Uuid>,
        default_priority: Option<&str>,
        default_sla_hours: Option<i32>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ServiceCategory> {
        let code_upper = code.to_uppercase();
        if code_upper.is_empty() || code_upper.len() > 50 {
            return Err(AtlasError::ValidationFailed(
                "Category code must be 1-50 characters".to_string(),
            ));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Category name is required".to_string(),
            ));
        }
        if let Some(ref pri) = default_priority {
            if !VALID_PRIORITIES.contains(&&*pri) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid priority '{}'. Must be one of: {}", pri, VALID_PRIORITIES.join(", ")
                )));
            }
        }

        info!("Creating service category '{}' ({}) for org {}", code_upper, name, org_id);

        self.repository.create_category(
            org_id, &code_upper, name, description,
            parent_category_id, default_priority, default_sla_hours,
            created_by,
        ).await
    }

    /// Get a service category by code
    pub async fn get_category(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<ServiceCategory>> {
        self.repository.get_category(org_id, &code.to_uppercase()).await
    }

    /// List service categories
    pub async fn list_categories(&self, org_id: Uuid) -> AtlasResult<Vec<ServiceCategory>> {
        self.repository.list_categories(org_id).await
    }

    /// Deactivate a service category
    pub async fn delete_category(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        info!("Deactivating service category '{}' for org {}", code, org_id);
        self.repository.delete_category(org_id, &code.to_uppercase()).await
    }

    // ========================================================================
    // Service Request Management
    // ========================================================================

    /// Create a new service request
    pub async fn create_request(
        &self,
        org_id: Uuid,
        request_number: &str,
        title: &str,
        description: Option<&str>,
        category_id: Option<Uuid>,
        priority: &str,
        request_type: &str,
        channel: &str,
        customer_id: Option<Uuid>,
        customer_name: Option<&str>,
        contact_id: Option<Uuid>,
        contact_name: Option<&str>,
        assigned_to: Option<Uuid>,
        assigned_to_name: Option<&str>,
        assigned_group: Option<&str>,
        product_id: Option<Uuid>,
        product_name: Option<&str>,
        serial_number: Option<&str>,
        parent_request_id: Option<Uuid>,
        related_object_type: Option<&str>,
        related_object_id: Option<Uuid>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ServiceRequest> {
        if title.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Service request title is required".to_string(),
            ));
        }
        if !VALID_PRIORITIES.contains(&priority) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid priority '{}'. Must be one of: {}", priority, VALID_PRIORITIES.join(", ")
            )));
        }
        if !VALID_REQUEST_TYPES.contains(&request_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid request_type '{}'. Must be one of: {}", request_type, VALID_REQUEST_TYPES.join(", ")
            )));
        }
        if !VALID_CHANNELS.contains(&channel) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid channel '{}'. Must be one of: {}", channel, VALID_CHANNELS.join(", ")
            )));
        }

        // Determine SLA due date from category
        let mut sla_due_date: Option<chrono::NaiveDate> = None;
        let mut category_name: Option<String> = None;
        if let Some(cat_id) = category_id {
            if let Some(cat) = self.repository.get_category_by_id(cat_id).await? {
                category_name = Some(cat.name.clone());
                if let Some(hours) = cat.default_sla_hours {
                    sla_due_date = Some((chrono::Utc::now().date_naive() + chrono::Duration::hours(hours as i64)));
                    // Note: TimeDelta + NaiveDate yields NaiveDate directly
                }
                // Use category default priority if not explicitly set
            }
        }

        info!("Creating service request '{}' ({}) for org {}", request_number, title, org_id);

        self.repository.create_request(
            org_id, request_number, title, description,
            category_id, category_name.as_deref(), priority,
            "open", request_type, channel,
            customer_id, customer_name, contact_id, contact_name,
            assigned_to, assigned_to_name, assigned_group,
            product_id, product_name, serial_number,
            sla_due_date, parent_request_id,
            related_object_type, related_object_id,
            created_by,
        ).await
    }

    /// Get a service request by ID
    pub async fn get_request(&self, id: Uuid) -> AtlasResult<Option<ServiceRequest>> {
        self.repository.get_request(id).await
    }

    /// Get a service request by number
    pub async fn get_request_by_number(&self, org_id: Uuid, request_number: &str) -> AtlasResult<Option<ServiceRequest>> {
        self.repository.get_request_by_number(org_id, request_number).await
    }

    /// List service requests with optional filters
    pub async fn list_requests(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        priority: Option<&str>,
        customer_id: Option<Uuid>,
        assigned_to: Option<Uuid>,
        category_id: Option<Uuid>,
    ) -> AtlasResult<Vec<ServiceRequest>> {
        self.repository.list_requests(org_id, status, priority, customer_id, assigned_to, category_id).await
    }

    /// Update a service request's status
    pub async fn update_status(
        &self,
        id: Uuid,
        new_status: &str,
    ) -> AtlasResult<ServiceRequest> {
        if !VALID_STATUSES.contains(&new_status) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid status '{}'. Must be one of: {}", new_status, VALID_STATUSES.join(", ")
            )));
        }

        let request = self.repository.get_request(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Service request {} not found", id)
            ))?;

        // Validate state transitions
        let allowed = match request.status.as_str() {
            "open" => vec!["in_progress", "cancelled"],
            "in_progress" => vec!["pending_customer", "resolved", "cancelled"],
            "pending_customer" => vec!["in_progress", "resolved", "cancelled"],
            "resolved" => vec!["closed", "in_progress"], // can reopen from resolved
            "closed" => vec!["in_progress"], // can reopen
            "cancelled" => vec!["open"], // can reopen cancelled
            _ => vec![],
        };

        if !allowed.contains(&new_status) {
            return Err(AtlasError::WorkflowError(
                format!("Cannot transition from '{}' to '{}'. Allowed: {:?}", request.status, new_status, allowed)
            ));
        }

        info!("Updating service request {} status from '{}' to '{}'", id, request.status, new_status);

        let now = chrono::Utc::now();
        let resolved_at = if new_status == "resolved" && request.resolved_at.is_none() {
            Some(now)
        } else {
            request.resolved_at
        };
        let closed_at = if new_status == "closed" && request.closed_at.is_none() {
            Some(now)
        } else {
            request.closed_at
        };

        self.repository.update_request_status(id, new_status, resolved_at, closed_at).await
    }

    /// Resolve a service request
    pub async fn resolve_request(
        &self,
        id: Uuid,
        resolution: &str,
        resolution_code: &str,
    ) -> AtlasResult<ServiceRequest> {
        if resolution.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Resolution description is required".to_string(),
            ));
        }
        if !VALID_RESOLUTION_CODES.contains(&resolution_code) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid resolution_code '{}'. Must be one of: {}",
                resolution_code, VALID_RESOLUTION_CODES.join(", ")
            )));
        }

        let request = self.repository.get_request(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Service request {} not found", id)
            ))?;

        if request.status == "resolved" || request.status == "closed" || request.status == "cancelled" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot resolve request in '{}' status", request.status)
            ));
        }

        info!("Resolving service request {} with code '{}'", id, resolution_code);

        let now = chrono::Utc::now();
        self.repository.update_request_resolution(id, resolution, resolution_code, now).await
    }

    /// Assign a service request
    pub async fn assign_request(
        &self,
        org_id: Uuid,
        request_id: Uuid,
        assigned_to: Option<Uuid>,
        assigned_to_name: Option<&str>,
        assigned_group: Option<&str>,
        assignment_type: &str,
        assigned_by: Option<Uuid>,
        assigned_by_name: Option<&str>,
    ) -> AtlasResult<ServiceRequestAssignment> {
        if !VALID_ASSIGNMENT_TYPES.contains(&assignment_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid assignment_type '{}'. Must be one of: {}", assignment_type, VALID_ASSIGNMENT_TYPES.join(", ")
            )));
        }

        let request = self.repository.get_request(request_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Service request {} not found", request_id)
            ))?;

        if request.status == "closed" || request.status == "cancelled" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot assign request in '{}' status", request.status)
            ));
        }

        info!("Assigning service request {} to {:?} ({})", request_id, assigned_to, assignment_type);

        let assignment = self.repository.create_assignment(
            org_id, request_id, assigned_to, assigned_to_name,
            assigned_group, assigned_by, assigned_by_name, assignment_type,
        ).await?;

        // Update the request itself
        self.repository.update_request_assignment(
            request_id, assigned_to, assigned_to_name, assigned_group,
        ).await?;

        // If the request was open and now assigned, move to in_progress
        if request.status == "open" && assigned_to.is_some() {
            let _ = self.repository.update_request_status(
                request_id, "in_progress", None, None,
            ).await;
        }

        Ok(assignment)
    }

    /// List assignments for a request
    pub async fn list_assignments(&self, request_id: Uuid) -> AtlasResult<Vec<ServiceRequestAssignment>> {
        self.repository.list_assignments(request_id).await
    }

    /// Add a communication/update to a request
    pub async fn add_update(
        &self,
        org_id: Uuid,
        request_id: Uuid,
        update_type: &str,
        author_id: Option<Uuid>,
        author_name: Option<&str>,
        subject: Option<&str>,
        body: &str,
        is_internal: bool,
    ) -> AtlasResult<ServiceRequestUpdate> {
        if !VALID_UPDATE_TYPES.contains(&update_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid update_type '{}'. Must be one of: {}", update_type, VALID_UPDATE_TYPES.join(", ")
            )));
        }
        if body.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Update body is required".to_string(),
            ));
        }

        // Verify request exists
        let request = self.repository.get_request(request_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Service request {} not found", request_id)
            ))?;

        if request.status == "closed" || request.status == "cancelled" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot add update to request in '{}' status", request.status)
            ));
        }

        info!("Adding {} update to service request {}", update_type, request_id);

        self.repository.create_update(
            org_id, request_id, update_type, author_id, author_name,
            subject, body, is_internal,
        ).await
    }

    /// List updates for a request
    pub async fn list_updates(
        &self,
        request_id: Uuid,
        include_internal: bool,
    ) -> AtlasResult<Vec<ServiceRequestUpdate>> {
        self.repository.list_updates(request_id, include_internal).await
    }

    /// Get dashboard summary
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<ServiceRequestDashboard> {
        self.repository.get_dashboard(org_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_priorities() {
        assert!(VALID_PRIORITIES.contains(&"low"));
        assert!(VALID_PRIORITIES.contains(&"medium"));
        assert!(VALID_PRIORITIES.contains(&"high"));
        assert!(VALID_PRIORITIES.contains(&"critical"));
    }

    #[test]
    fn test_valid_statuses() {
        assert!(VALID_STATUSES.contains(&"open"));
        assert!(VALID_STATUSES.contains(&"in_progress"));
        assert!(VALID_STATUSES.contains(&"pending_customer"));
        assert!(VALID_STATUSES.contains(&"resolved"));
        assert!(VALID_STATUSES.contains(&"closed"));
        assert!(VALID_STATUSES.contains(&"cancelled"));
    }

    #[test]
    fn test_valid_request_types() {
        assert!(VALID_REQUEST_TYPES.contains(&"incident"));
        assert!(VALID_REQUEST_TYPES.contains(&"service_request"));
        assert!(VALID_REQUEST_TYPES.contains(&"problem"));
        assert!(VALID_REQUEST_TYPES.contains(&"change_request"));
    }

    #[test]
    fn test_valid_channels() {
        assert!(VALID_CHANNELS.contains(&"phone"));
        assert!(VALID_CHANNELS.contains(&"email"));
        assert!(VALID_CHANNELS.contains(&"web"));
        assert!(VALID_CHANNELS.contains(&"chat"));
        assert!(VALID_CHANNELS.contains(&"social_media"));
    }

    #[test]
    fn test_valid_update_types() {
        assert!(VALID_UPDATE_TYPES.contains(&"comment"));
        assert!(VALID_UPDATE_TYPES.contains(&"email"));
        assert!(VALID_UPDATE_TYPES.contains(&"phone_call"));
        assert!(VALID_UPDATE_TYPES.contains(&"note"));
        assert!(VALID_UPDATE_TYPES.contains(&"system"));
    }

    #[test]
    fn test_valid_resolution_codes() {
        assert!(VALID_RESOLUTION_CODES.contains(&"resolved"));
        assert!(VALID_RESOLUTION_CODES.contains(&"workaround"));
        assert!(VALID_RESOLUTION_CODES.contains(&"no_fault_found"));
        assert!(VALID_RESOLUTION_CODES.contains(&"duplicate"));
    }

    #[test]
    fn test_valid_assignment_types() {
        assert!(VALID_ASSIGNMENT_TYPES.contains(&"initial"));
        assert!(VALID_ASSIGNMENT_TYPES.contains(&"transfer"));
        assert!(VALID_ASSIGNMENT_TYPES.contains(&"escalation"));
        assert!(VALID_ASSIGNMENT_TYPES.contains(&"reassignment"));
    }
}
