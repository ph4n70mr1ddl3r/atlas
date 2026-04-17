//! Approval Engine Implementation
//!
//! Multi-level approval system inspired by Oracle Fusion.
//! Supports sequential approval chains, role-based approvers,
//! timed escalation, and delegation.

use atlas_shared::{
    ApprovalRequest, ApprovalStep, ApprovalLevel,
    AtlasError, AtlasResult,
};
use super::ApprovalRepository;
use std::sync::Arc;
use uuid::Uuid;
use tracing::info;

/// Approval engine for managing multi-level approvals
pub struct ApprovalEngine {
    repository: Arc<dyn ApprovalRepository>,
}

impl ApprovalEngine {
    pub fn new(repository: Arc<dyn ApprovalRepository>) -> Self {
        Self { repository }
    }

    /// Create a new approval request from a chain definition
    /// This initializes all the approval steps based on the chain levels
    #[allow(clippy::too_many_arguments)]
    pub async fn create_request(
        &self,
        org_id: Uuid,
        chain_id: Uuid,
        entity_type: &str,
        entity_id: Uuid,
        requested_by: Uuid,
        title: Option<&str>,
        description: Option<&str>,
    ) -> AtlasResult<ApprovalRequest> {
        info!("Creating approval request for {} {}", entity_type, entity_id);

        // Load the chain definition
        let chain = self.repository.get_chain(chain_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Approval chain {}", chain_id)))?;

        // Parse chain definition to get levels
        let levels: Vec<ApprovalLevel> = serde_json::from_value(chain.chain_definition.clone())
            .map_err(|e| AtlasError::ConfigError(format!("Invalid chain definition: {}", e)))?;

        if levels.is_empty() {
            return Err(AtlasError::ConfigError("Approval chain has no levels".to_string()));
        }

        let total_levels = levels.len() as i32;

        // Create the approval request
        let request = self.repository.create_request(
            org_id,
            chain_id,
            entity_type,
            entity_id,
            total_levels,
            requested_by,
            title,
            description,
        ).await?;

        // Create the approval steps
        for level in &levels {
            let approver_user_id = level.user_ids.as_ref()
                .and_then(|ids| ids.first().copied());

            self.repository.create_step(
                org_id,
                request.id,
                level.level,
                &level.approver_type,
                level.roles.first().map(|s| s.as_str()),
                approver_user_id,
                level.auto_approve_after_hours,
            ).await?;
        }

        // Reload the request with steps
        let request = self.repository.get_request(request.id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Created approval request".to_string()))?;

        info!("Created approval request {} with {} levels", request.id, total_levels);
        Ok(request)
    }

    /// Approve a step in the approval chain
    /// If this is the last step, the entire request is marked as approved
    pub async fn approve_step(
        &self,
        _org_id: Uuid,
        step_id: Uuid,
        approved_by: Uuid,
        comment: Option<&str>,
    ) -> AtlasResult<ApprovalRequest> {
        info!("Approving step {} by {}", step_id, approved_by);

        let step = self.repository.get_step(step_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Approval step {}", step_id)))?;

        if step.status != "pending" {
            return Err(AtlasError::WorkflowError(
                format!("Step {} is not pending (status: {})", step_id, step.status)
            ));
        }

        // Mark the step as approved
        self.repository.approve_step(step_id, approved_by, comment).await?;

        // Get the parent request
        let mut request = self.repository.get_request(step.approval_request_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Approval request".to_string()))?;

        // Check if this was the current level (not already advanced)
        if step.level >= request.current_level {
            request.current_level += 1;

            if request.current_level > request.total_levels {
                // All levels approved - mark request as completed
                self.repository.complete_request(request.id, approved_by, "approved").await?;
                request.status = "approved".to_string();
                request.completed_at = Some(chrono::Utc::now());
                info!("Approval request {} fully approved", request.id);
            } else {
                // Advance to next level
                self.repository.advance_request_level(request.id, request.current_level).await?;
                info!("Approval request {} advanced to level {}", request.id, request.current_level);
            }
        }

        // Reload
        self.repository.get_request(request.id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Approval request".to_string()))
    }

    /// Reject an approval step - this rejects the entire request
    pub async fn reject_step(
        &self,
        _org_id: Uuid,
        step_id: Uuid,
        rejected_by: Uuid,
        comment: Option<&str>,
    ) -> AtlasResult<ApprovalRequest> {
        info!("Rejecting step {} by {}", step_id, rejected_by);

        let step = self.repository.get_step(step_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Approval step {}", step_id)))?;

        if step.status != "pending" {
            return Err(AtlasError::WorkflowError(
                format!("Step {} is not pending", step_id)
            ));
        }

        // Mark the step as rejected
        self.repository.reject_step(step_id, rejected_by, comment).await?;

        // Mark the entire request as rejected
        self.repository.complete_request(step.approval_request_id, rejected_by, "rejected").await?;

        self.repository.get_request(step.approval_request_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Approval request".to_string()))
    }

    /// Delegate an approval step to another user
    pub async fn delegate_step(
        &self,
        _org_id: Uuid,
        step_id: Uuid,
        delegated_by: Uuid,
        delegated_to: Uuid,
    ) -> AtlasResult<ApprovalStep> {
        info!("Delegating step {} from {} to {}", step_id, delegated_by, delegated_to);

        self.repository.delegate_step(step_id, delegated_by, delegated_to).await?;

        self.repository.get_step(step_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Step {}", step_id)))
    }

    /// Check for pending approvals that need escalation
    /// and escalate or auto-approve them
    pub async fn process_escalations(&self) -> AtlasResult<Vec<Uuid>> {
        info!("Processing approval escalations");

        let expired_steps = self.repository.find_expired_steps().await?;
        let mut escalated = Vec::new();

        for step in expired_steps {
            if let Some(auto_hours) = step.auto_approve_after_hours {
                // Auto-approve expired step
                info!("Auto-approving expired step {} (auto_approve_after_hours: {})", 
                    step.id, auto_hours);
                self.repository.auto_approve_step(step.id).await?;
                escalated.push(step.approval_request_id);
            }
        }

        Ok(escalated)
    }

    /// Get pending approval steps for a specific user
    pub async fn get_pending_for_user(
        &self,
        org_id: Uuid,
        user_id: Uuid,
    ) -> AtlasResult<Vec<ApprovalStep>> {
        self.repository.get_pending_steps_for_user(org_id, user_id).await
    }

    /// Get pending approval steps where the user's role matches
    pub async fn get_pending_for_role(
        &self,
        org_id: Uuid,
        role: &str,
    ) -> AtlasResult<Vec<ApprovalStep>> {
        self.repository.get_pending_steps_for_role(org_id, role).await
    }

    /// Get all approval requests for an entity
    pub async fn get_requests_for_entity(
        &self,
        entity_type: &str,
        entity_id: Uuid,
    ) -> AtlasResult<Vec<ApprovalRequest>> {
        self.repository.get_requests_for_entity(entity_type, entity_id).await
    }

    /// Cancel an approval request
    pub async fn cancel_request(&self, request_id: Uuid) -> AtlasResult<()> {
        self.repository.cancel_request(request_id).await
    }
}