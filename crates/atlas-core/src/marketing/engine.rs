//! Marketing Campaign Engine
//!
//! Oracle Fusion Cloud CX Marketing.
//! Manages campaign types, marketing campaigns, campaign members,
//! responses, and ROI analytics.
//!
//! The process follows Oracle Fusion CX Marketing workflow:
//! 1. Define campaign types (email, event, webinar, digital, social, etc.)
//! 2. Create campaigns with budget, channel, and timeline
//! 3. Add campaign members (contacts and leads)
//! 4. Activate campaigns
//! 5. Track responses and conversions
//! 6. Complete campaigns and analyze ROI

use atlas_shared::{AtlasError, AtlasResult};
use super::MarketingRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Valid campaign statuses
const VALID_CAMPAIGN_STATUSES: &[&str] = &[
    "draft", "active", "paused", "completed", "cancelled",
];

/// Valid channels
const VALID_CHANNELS: &[&str] = &[
    "email", "event", "webinar", "digital", "social", "print", "phone", "other",
];

/// Valid member statuses
const VALID_MEMBER_STATUSES: &[&str] = &[
    "invited", "responded", "converted", "bounced", "unsubscribed", "removed",
];

/// Valid response types
const VALID_RESPONSE_TYPES: &[&str] = &[
    "opened", "clicked", "registered", "attended", "downloaded",
    "submitted_form", "replied", "purchased", "referred", "other",
];

/// Marketing Campaign Engine
pub struct MarketingEngine {
    repository: Arc<dyn MarketingRepository>,
}

impl MarketingEngine {
    pub fn new(repository: Arc<dyn MarketingRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Campaign Type Management
    // ========================================================================

    /// Create a campaign type
    pub async fn create_campaign_type(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        channel: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::CampaignType> {
        let code = code.to_uppercase();
        if code.is_empty() || code.len() > 50 {
            return Err(AtlasError::ValidationFailed(
                "Campaign type code must be 1-50 characters".to_string(),
            ));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Campaign type name is required".to_string()));
        }
        if !VALID_CHANNELS.contains(&channel) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid channel '{}'. Must be one of: {}", channel, VALID_CHANNELS.join(", ")
            )));
        }
        if self.repository.get_campaign_type_by_code(org_id, &code).await?.is_some() {
            return Err(AtlasError::Conflict(format!("Campaign type '{}' already exists", code)));
        }
        info!("Creating campaign type '{}' for org {}", code, org_id);
        self.repository.create_campaign_type(org_id, &code, name, description, channel, created_by).await
    }

    /// List campaign types
    pub async fn list_campaign_types(&self, org_id: Uuid) -> AtlasResult<Vec<atlas_shared::CampaignType>> {
        self.repository.list_campaign_types(org_id).await
    }

    /// Delete a campaign type
    pub async fn delete_campaign_type(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        info!("Deleting campaign type '{}' for org {}", code, org_id);
        self.repository.delete_campaign_type(org_id, code).await
    }

    // ========================================================================
    // Marketing Campaign Management
    // ========================================================================

    /// Create a new marketing campaign
    #[allow(clippy::too_many_arguments)]
    pub async fn create_campaign(
        &self,
        org_id: Uuid,
        campaign_number: &str,
        name: &str,
        description: Option<&str>,
        campaign_type_id: Option<Uuid>,
        campaign_type_name: Option<&str>,
        channel: &str,
        budget: &str,
        currency_code: &str,
        start_date: Option<chrono::NaiveDate>,
        end_date: Option<chrono::NaiveDate>,
        owner_id: Option<Uuid>,
        owner_name: Option<&str>,
        expected_responses: i32,
        expected_revenue: &str,
        parent_campaign_id: Option<Uuid>,
        parent_campaign_name: Option<&str>,
        tags: serde_json::Value,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::MarketingCampaign> {
        if campaign_number.is_empty() {
            return Err(AtlasError::ValidationFailed("Campaign number is required".to_string()));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Campaign name is required".to_string()));
        }
        if !VALID_CHANNELS.contains(&channel) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid channel '{}'. Must be one of: {}", channel, VALID_CHANNELS.join(", ")
            )));
        }
        let budget_val: f64 = budget.parse().unwrap_or(0.0);
        if budget_val < 0.0 {
            return Err(AtlasError::ValidationFailed("Budget cannot be negative".to_string()));
        }

        // Check uniqueness
        if self.repository.get_campaign_by_number(org_id, campaign_number).await?.is_some() {
            return Err(AtlasError::Conflict(format!("Campaign '{}' already exists", campaign_number)));
        }

        info!("Creating marketing campaign '{}' for org {}", campaign_number, org_id);

        self.repository.create_campaign(
            org_id, campaign_number, name, description,
            campaign_type_id, campaign_type_name, channel, budget, currency_code,
            start_date, end_date, owner_id, owner_name,
            expected_responses, expected_revenue,
            parent_campaign_id, parent_campaign_name, tags, notes, created_by,
        ).await
    }

    /// Get a campaign by ID
    pub async fn get_campaign(&self, id: Uuid) -> AtlasResult<Option<atlas_shared::MarketingCampaign>> {
        self.repository.get_campaign(id).await
    }

    /// List campaigns with optional filters
    pub async fn list_campaigns(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        channel: Option<&str>,
        owner_id: Option<Uuid>,
    ) -> AtlasResult<Vec<atlas_shared::MarketingCampaign>> {
        if let Some(s) = status {
            if !VALID_CAMPAIGN_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid campaign status '{}'. Must be one of: {}", s, VALID_CAMPAIGN_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_campaigns(org_id, status, channel, owner_id).await
    }

    /// Activate a campaign (draft/paused -> active)
    pub async fn activate_campaign(&self, id: Uuid) -> AtlasResult<atlas_shared::MarketingCampaign> {
        let campaign = self.repository.get_campaign(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Campaign {} not found", id)))?;
        if campaign.status != "draft" && campaign.status != "paused" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot activate campaign in '{}' status. Must be 'draft' or 'paused'.", campaign.status
            )));
        }
        info!("Activating campaign {}", campaign.campaign_number);
        self.repository.activate_campaign(id).await
    }

    /// Pause a campaign (active -> paused)
    pub async fn pause_campaign(&self, id: Uuid) -> AtlasResult<atlas_shared::MarketingCampaign> {
        let campaign = self.repository.get_campaign(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Campaign {} not found", id)))?;
        if campaign.status != "active" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot pause campaign in '{}' status. Must be 'active'.", campaign.status
            )));
        }
        info!("Pausing campaign {}", campaign.campaign_number);
        self.repository.update_campaign_status(id, "paused").await
    }

    /// Complete a campaign (active/paused -> completed)
    pub async fn complete_campaign(&self, id: Uuid) -> AtlasResult<atlas_shared::MarketingCampaign> {
        let campaign = self.repository.get_campaign(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Campaign {} not found", id)))?;
        if campaign.status != "active" && campaign.status != "paused" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot complete campaign in '{}' status. Must be 'active' or 'paused'.", campaign.status
            )));
        }
        info!("Completing campaign {}", campaign.campaign_number);
        self.repository.complete_campaign(id).await
    }

    /// Cancel a campaign (any -> cancelled)
    pub async fn cancel_campaign(&self, id: Uuid) -> AtlasResult<atlas_shared::MarketingCampaign> {
        let campaign = self.repository.get_campaign(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Campaign {} not found", id)))?;
        if campaign.status == "completed" || campaign.status == "cancelled" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot cancel campaign in '{}' status.", campaign.status
            )));
        }
        info!("Cancelling campaign {}", campaign.campaign_number);
        self.repository.cancel_campaign(id).await
    }

    /// Delete a campaign
    pub async fn delete_campaign(&self, org_id: Uuid, campaign_number: &str) -> AtlasResult<()> {
        self.repository.delete_campaign(org_id, campaign_number).await
    }

    // ========================================================================
    // Campaign Members
    // ========================================================================

    /// Add a member to a campaign
    pub async fn add_campaign_member(
        &self,
        org_id: Uuid,
        campaign_id: Uuid,
        contact_id: Option<Uuid>,
        contact_name: Option<&str>,
        contact_email: Option<&str>,
        lead_id: Option<Uuid>,
        lead_number: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::CampaignMember> {
        // Verify campaign exists
        let campaign = self.repository.get_campaign(campaign_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Campaign {} not found", campaign_id)))?;
        if campaign.organization_id != org_id {
            return Err(AtlasError::EntityNotFound(format!("Campaign {} not found", campaign_id)));
        }

        info!("Adding member to campaign {}", campaign.campaign_number);
        self.repository.add_campaign_member(
            org_id, campaign_id, contact_id, contact_name, contact_email,
            lead_id, lead_number, created_by,
        ).await
    }

    /// List campaign members
    pub async fn list_campaign_members(
        &self,
        campaign_id: Uuid,
        status: Option<&str>,
    ) -> AtlasResult<Vec<atlas_shared::CampaignMember>> {
        if let Some(s) = status {
            if !VALID_MEMBER_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid member status '{}'. Must be one of: {}", s, VALID_MEMBER_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_campaign_members(campaign_id, status).await
    }

    /// Update a member's status (mark as responded, converted, etc.)
    pub async fn update_member_status(
        &self,
        id: Uuid,
        status: &str,
        response: Option<&str>,
    ) -> AtlasResult<atlas_shared::CampaignMember> {
        if !VALID_MEMBER_STATUSES.contains(&status) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid member status '{}'. Must be one of: {}", status, VALID_MEMBER_STATUSES.join(", ")
            )));
        }
        let member = self.repository.get_campaign_member(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Campaign member {} not found", id)))?;
        info!("Updating member {} status to {} in campaign {}", id, status, member.campaign_id);
        self.repository.update_member_status(id, status, response).await
    }

    /// Remove a campaign member
    pub async fn delete_campaign_member(&self, id: Uuid) -> AtlasResult<()> {
        self.repository.delete_campaign_member(id).await
    }

    // ========================================================================
    // Campaign Responses
    // ========================================================================

    /// Record a campaign response
    #[allow(clippy::too_many_arguments)]
    pub async fn create_response(
        &self,
        org_id: Uuid,
        campaign_id: Uuid,
        member_id: Option<Uuid>,
        response_type: &str,
        contact_id: Option<Uuid>,
        contact_name: Option<&str>,
        contact_email: Option<&str>,
        lead_id: Option<Uuid>,
        description: Option<&str>,
        value: &str,
        currency_code: &str,
        source_url: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::CampaignResponse> {
        if !VALID_RESPONSE_TYPES.contains(&response_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid response type '{}'. Must be one of: {}", response_type, VALID_RESPONSE_TYPES.join(", ")
            )));
        }

        // Verify campaign exists
        let campaign = self.repository.get_campaign(campaign_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Campaign {} not found", campaign_id)))?;
        if campaign.organization_id != org_id {
            return Err(AtlasError::EntityNotFound(format!("Campaign {} not found", campaign_id)));
        }

        info!("Recording {} response for campaign {}", response_type, campaign.campaign_number);

        let response = self.repository.create_response(
            org_id, campaign_id, member_id, response_type,
            contact_id, contact_name, contact_email, lead_id,
            description, value, currency_code, source_url, created_by,
        ).await?;

        // Update campaign actuals: increment actual_responses by 1
        let current = self.repository.get_campaign(campaign_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Campaign {} not found", campaign_id)))?;
        let new_responses = current.actual_responses + 1;
        let value_f64: f64 = value.parse().unwrap_or(0.0);
        let current_revenue: f64 = current.actual_revenue.parse().unwrap_or(0.0);
        let new_revenue = current_revenue + value_f64;
        self.repository.update_campaign_actuals(
            campaign_id,
            &current.actual_cost,
            new_responses,
            &format!("{:.2}", new_revenue),
            current.converted_leads,
            current.converted_opportunities,
            current.converted_won,
        ).await?;

        Ok(response)
    }

    /// List campaign responses
    pub async fn list_responses(
        &self,
        campaign_id: Uuid,
        response_type: Option<&str>,
    ) -> AtlasResult<Vec<atlas_shared::CampaignResponse>> {
        self.repository.list_responses(campaign_id, response_type).await
    }

    /// Delete a campaign response
    pub async fn delete_response(&self, id: Uuid) -> AtlasResult<()> {
        self.repository.delete_response(id).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get the marketing dashboard
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<atlas_shared::MarketingDashboard> {
        self.repository.get_dashboard(org_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_campaign_statuses() {
        assert!(VALID_CAMPAIGN_STATUSES.contains(&"draft"));
        assert!(VALID_CAMPAIGN_STATUSES.contains(&"active"));
        assert!(VALID_CAMPAIGN_STATUSES.contains(&"paused"));
        assert!(VALID_CAMPAIGN_STATUSES.contains(&"completed"));
        assert!(VALID_CAMPAIGN_STATUSES.contains(&"cancelled"));
        assert!(!VALID_CAMPAIGN_STATUSES.contains(&"deleted"));
    }

    #[test]
    fn test_valid_channels() {
        assert!(VALID_CHANNELS.contains(&"email"));
        assert!(VALID_CHANNELS.contains(&"event"));
        assert!(VALID_CHANNELS.contains(&"webinar"));
        assert!(VALID_CHANNELS.contains(&"digital"));
        assert!(VALID_CHANNELS.contains(&"social"));
        assert!(VALID_CHANNELS.contains(&"print"));
        assert!(VALID_CHANNELS.contains(&"phone"));
        assert!(VALID_CHANNELS.contains(&"other"));
        assert!(!VALID_CHANNELS.contains(&"fax"));
    }

    #[test]
    fn test_valid_member_statuses() {
        assert!(VALID_MEMBER_STATUSES.contains(&"invited"));
        assert!(VALID_MEMBER_STATUSES.contains(&"responded"));
        assert!(VALID_MEMBER_STATUSES.contains(&"converted"));
        assert!(VALID_MEMBER_STATUSES.contains(&"bounced"));
        assert!(VALID_MEMBER_STATUSES.contains(&"unsubscribed"));
        assert!(VALID_MEMBER_STATUSES.contains(&"removed"));
    }

    #[test]
    fn test_valid_response_types() {
        assert!(VALID_RESPONSE_TYPES.contains(&"opened"));
        assert!(VALID_RESPONSE_TYPES.contains(&"clicked"));
        assert!(VALID_RESPONSE_TYPES.contains(&"registered"));
        assert!(VALID_RESPONSE_TYPES.contains(&"attended"));
        assert!(VALID_RESPONSE_TYPES.contains(&"downloaded"));
        assert!(VALID_RESPONSE_TYPES.contains(&"submitted_form"));
        assert!(VALID_RESPONSE_TYPES.contains(&"replied"));
        assert!(VALID_RESPONSE_TYPES.contains(&"purchased"));
        assert!(VALID_RESPONSE_TYPES.contains(&"referred"));
        assert!(VALID_RESPONSE_TYPES.contains(&"other"));
    }

    #[test]
    fn test_roi_calculation() {
        let cost = 10000.0_f64;
        let revenue = 50000.0_f64;
        let roi: f64 = if cost > 0.0 { ((revenue - cost) / cost) * 100.0 } else { 0.0 };
        assert!((roi - 400.0).abs() < 0.01);

        // Zero cost
        let cost = 0.0_f64;
        let roi: f64 = if cost > 0.0 { ((revenue - cost) / cost) * 100.0 } else { 0.0 };
        assert!((roi - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_campaign_status_transitions() {
        // draft -> active (valid)
        // paused -> active (valid)
        // active -> paused (valid)
        // active -> completed (valid)
        // paused -> completed (valid)
        // draft -> cancelled (valid, not completed/cancelled)
        // completed -> cancelled (invalid)
        // cancelled -> active (invalid, not draft/paused)
        let valid_from_draft = vec!["active", "cancelled"];
        let valid_from_active = vec!["paused", "completed", "cancelled"];
        let valid_from_paused = vec!["active", "completed", "cancelled"];
        assert!(valid_from_draft.contains(&"active"));
        assert!(valid_from_active.contains(&"paused"));
        assert!(valid_from_paused.contains(&"active"));
    }
}
