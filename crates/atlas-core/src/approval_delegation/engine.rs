//! Approval Delegation Engine Implementation
//!
//! Oracle Fusion BPM Worklist > Rules > Configure Delegation

use atlas_shared::{
    ApprovalDelegationRule, CreateDelegationRuleRequest, DelegationHistoryEntry,
    DelegationDashboard, AtlasError, AtlasResult,
};
use super::ApprovalDelegationRepository;
use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;
use tracing::info;

/// Helper: returns true if the Option<Value> is None or contains an empty JSON array.
fn array_is_empty(val: &Option<serde_json::Value>) -> bool {
    val.as_ref()
        .and_then(|v| v.as_array())
        .is_none_or(|a| a.is_empty())
}

/// Engine for managing approval delegation rules
pub struct ApprovalDelegationEngine {
    repository: Arc<dyn ApprovalDelegationRepository>,
}

impl ApprovalDelegationEngine {
    pub fn new(repository: Arc<dyn ApprovalDelegationRepository>) -> Self {
        Self { repository }
    }

    /// Create a new delegation rule
    pub async fn create_rule(
        &self,
        org_id: Uuid,
        delegator_id: Uuid,
        request: CreateDelegationRuleRequest,
    ) -> AtlasResult<ApprovalDelegationRule> {
        info!("Creating delegation rule '{}' for user {}", request.rule_name, delegator_id);

        // Validate delegate user ID
        let delegate_to_id = Uuid::parse_str(&request.delegate_to_id)
            .map_err(|_| AtlasError::ValidationFailed("Invalid delegate_to_id".to_string()))?;

        // Can't delegate to yourself
        if delegator_id == delegate_to_id {
            return Err(AtlasError::ValidationFailed(
                "Cannot delegate approvals to yourself".to_string(),
            ));
        }

        // Validate date range
        if request.start_date >= request.end_date {
            return Err(AtlasError::ValidationFailed(
                "Start date must be before end date".to_string(),
            ));
        }

        // Reject rules that are already expired (end_date in the past)
        if request.end_date < Utc::now().date_naive() {
            return Err(AtlasError::ValidationFailed(
                "End date must not be in the past".to_string(),
            ));
        }

        // Validate delegation_type
        let valid_types = ["all", "by_category", "by_role", "by_entity"];
        if !valid_types.contains(&request.delegation_type.as_str()) {
            return Err(AtlasError::ValidationFailed(
                format!("Invalid delegation_type '{}'. Must be one of: {}", request.delegation_type, valid_types.join(", "))
            ));
        }

        // Validate that type-specific fields are populated
        match request.delegation_type.as_str() {
            "by_category"
                if array_is_empty(&request.categories) => {
                    return Err(AtlasError::ValidationFailed(
                        "categories must be a non-empty array for delegation_type 'by_category'".to_string(),
                    ));
                }
            "by_role"
                if array_is_empty(&request.roles) => {
                    return Err(AtlasError::ValidationFailed(
                        "roles must be a non-empty array for delegation_type 'by_role'".to_string(),
                    ));
                }
            "by_entity"
                if array_is_empty(&request.entity_types) => {
                    return Err(AtlasError::ValidationFailed(
                        "entity_types must be a non-empty array for delegation_type 'by_entity'".to_string(),
                    ));
                }
            _ => {}
        }

        // Determine initial status based on dates
        let today = chrono::Utc::now().date_naive();
        let auto_activate = request.auto_activate.unwrap_or(true);
        let auto_expire = request.auto_expire.unwrap_or(true);
        let status = if auto_activate && request.start_date <= today {
            "active"
        } else {
            "scheduled"
        };

        let rule = self.repository.create_rule(
            org_id,
            delegator_id,
            delegate_to_id,
            &request.rule_name,
            request.description.as_deref(),
            &request.delegation_type,
            request.categories.clone().unwrap_or(serde_json::json!([])),
            request.roles.clone().unwrap_or(serde_json::json!([])),
            request.entity_types.clone().unwrap_or(serde_json::json!([])),
            request.start_date,
            request.end_date,
            auto_activate,
            auto_expire,
            status,
            if status == "active" { Some(Utc::now()) } else { None },
        ).await?;

        info!("Created delegation rule {} with status {}", rule.id, rule.status);
        Ok(rule)
    }

    /// Get a delegation rule by ID
    pub async fn get_rule(&self, id: Uuid) -> AtlasResult<Option<ApprovalDelegationRule>> {
        self.repository.get_rule(id).await
    }

    /// List delegation rules for a delegator (user who set up the rules)
    pub async fn list_rules_for_delegator(
        &self,
        org_id: Uuid,
        delegator_id: Uuid,
        status: Option<&str>,
    ) -> AtlasResult<Vec<ApprovalDelegationRule>> {
        self.repository.list_rules_for_delegator(org_id, delegator_id, status).await
    }

    /// List all delegation rules for an organization
    pub async fn list_rules(
        &self,
        org_id: Uuid,
        status: Option<&str>,
    ) -> AtlasResult<Vec<ApprovalDelegationRule>> {
        self.repository.list_rules(org_id, status).await
    }

    /// Cancel a delegation rule
    pub async fn cancel_rule(
        &self,
        id: Uuid,
        cancelled_by: Uuid,
        reason: Option<&str>,
    ) -> AtlasResult<ApprovalDelegationRule> {
        info!("Cancelling delegation rule {}", id);

        let rule = self.repository.get_rule(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Delegation rule {}", id)))?;

        if rule.status == "cancelled" {
            return Err(AtlasError::WorkflowError("Rule is already cancelled".to_string()));
        }
        if rule.status == "expired" {
            return Err(AtlasError::WorkflowError("Cannot cancel an expired rule".to_string()));
        }

        self.repository.cancel_rule(id, cancelled_by, reason).await?;

        self.repository.get_rule(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Delegation rule {}", id)))
    }

    /// Activate a scheduled delegation rule
    pub async fn activate_rule(&self, id: Uuid) -> AtlasResult<ApprovalDelegationRule> {
        info!("Activating delegation rule {}", id);

        let rule = self.repository.get_rule(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Delegation rule {}", id)))?;

        if rule.status != "scheduled" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot activate rule in status '{}'. Must be 'scheduled'.", rule.status)
            ));
        }

        self.repository.activate_rule(id).await?;

        self.repository.get_rule(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Delegation rule {}", id)))
    }

    /// Process all scheduled rules that should be activated (auto-activate)
    /// and all active rules that should be expired (auto-expire)
    pub async fn process_scheduled_rules(&self) -> AtlasResult<(Vec<Uuid>, Vec<Uuid>)> {
        info!("Processing scheduled delegation rules");

        let activated = self.repository.activate_due_rules().await?;
        let expired = self.repository.expire_due_rules().await?;

        info!("Activated {} rules, expired {} rules", activated.len(), expired.len());
        Ok((activated, expired))
    }

    /// Find an active delegation for a given approver.
    /// Returns the delegate user ID if an active delegation rule exists.
    /// This is the key method called by the approval engine to check for delegations.
    pub async fn find_delegate_for_approver(
        &self,
        org_id: Uuid,
        approver_id: Uuid,
        entity_type: Option<&str>,
        approver_role: Option<&str>,
    ) -> AtlasResult<Option<Uuid>> {
        self.repository.find_active_delegate(org_id, approver_id, entity_type, approver_role).await
    }

    /// Record a delegation event in history
    pub async fn record_delegation(
        &self,
        org_id: Uuid,
        rule_id: Uuid,
        original_approver_id: Uuid,
        delegated_to_id: Uuid,
        approval_step_id: Option<Uuid>,
        approval_request_id: Option<Uuid>,
        entity_type: Option<&str>,
        entity_id: Option<Uuid>,
    ) -> AtlasResult<DelegationHistoryEntry> {
        info!("Recording delegation from {} to {} for rule {}",
            original_approver_id, delegated_to_id, rule_id);

        self.repository.record_delegation(
            org_id, rule_id, original_approver_id, delegated_to_id,
            approval_step_id, approval_request_id, entity_type, entity_id,
        ).await
    }

    /// List delegation history for a user (either as delegator or delegate)
    pub async fn list_delegation_history(
        &self,
        org_id: Uuid,
        user_id: Uuid,
        limit: Option<i64>,
    ) -> AtlasResult<Vec<DelegationHistoryEntry>> {
        let limit = limit.unwrap_or(50);
        self.repository.list_delegation_history(org_id, user_id, limit).await
    }

    /// Get delegation dashboard summary
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<DelegationDashboard> {
        self.repository.get_dashboard(org_id).await
    }

    /// Delete a delegation rule
    pub async fn delete_rule(&self, id: Uuid) -> AtlasResult<()> {
        self.repository.delete_rule(id).await
    }
}
