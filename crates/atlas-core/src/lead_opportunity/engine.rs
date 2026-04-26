//! Lead and Opportunity Engine
//!
//! Oracle Fusion Cloud CX Sales.
//! Manages sales leads, opportunity pipeline, sales activities,
//! lead scoring, lead-to-opportunity conversion, and pipeline analytics.
//!
//! The process follows Oracle Fusion CX Sales workflow:
//! 1. Define lead sources and opportunity stages
//! 2. Create leads with contact info and qualification data
//! 3. Score and qualify leads
//! 4. Convert qualified leads to opportunities
//! 5. Track opportunity pipeline stages
//! 6. Manage sales activities
//! 7. Close opportunities as won/lost
//! 8. Analyze pipeline via dashboard

use atlas_shared::{
    AtlasError, AtlasResult,
};
use super::LeadOpportunityRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Valid lead statuses
const VALID_LEAD_STATUSES: &[&str] = &[
    "new", "contacted", "qualified", "unqualified", "converted", "archived",
];

/// Valid lead ratings
const VALID_LEAD_RATINGS: &[&str] = &[
    "hot", "warm", "cold",
];

/// Valid opportunity statuses
const VALID_OPP_STATUSES: &[&str] = &[
    "open", "won", "lost",
];

/// Valid activity types
const VALID_ACTIVITY_TYPES: &[&str] = &[
    "call", "meeting", "email", "task", "demo", "presentation", "other",
];

/// Valid activity statuses
#[allow(dead_code)]
const VALID_ACTIVITY_STATUSES: &[&str] = &[
    "planned", "in_progress", "completed", "cancelled",
];

/// Valid activity priorities
const VALID_ACTIVITY_PRIORITIES: &[&str] = &[
    "low", "medium", "high", "critical",
];

/// Lead and Opportunity engine
pub struct LeadOpportunityEngine {
    repository: Arc<dyn LeadOpportunityRepository>,
}

impl LeadOpportunityEngine {
    pub fn new(repository: Arc<dyn LeadOpportunityRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Lead Source Management
    // ========================================================================

    /// Create a lead source
    pub async fn create_lead_source(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::LeadSource> {
        let code = code.to_uppercase();
        if code.is_empty() || code.len() > 50 {
            return Err(AtlasError::ValidationFailed(
                "Lead source code must be 1-50 characters".to_string(),
            ));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Lead source name is required".to_string()));
        }
        if self.repository.get_lead_source_by_code(org_id, &code).await?.is_some() {
            return Err(AtlasError::Conflict(format!("Lead source '{}' already exists", code)));
        }
        info!("Creating lead source '{}' for org {}", code, org_id);
        self.repository.create_lead_source(org_id, &code, name, description, created_by).await
    }

    /// List lead sources
    pub async fn list_lead_sources(&self, org_id: Uuid) -> AtlasResult<Vec<atlas_shared::LeadSource>> {
        self.repository.list_lead_sources(org_id).await
    }

    /// Delete a lead source
    pub async fn delete_lead_source(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        info!("Deleting lead source '{}' for org {}", code, org_id);
        self.repository.delete_lead_source(org_id, code).await
    }

    // ========================================================================
    // Lead Rating Models
    // ========================================================================

    /// Create a lead rating/scoring model
    pub async fn create_lead_rating_model(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        scoring_criteria: serde_json::Value,
        created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::LeadRatingModel> {
        let code = code.to_uppercase();
        if code.is_empty() {
            return Err(AtlasError::ValidationFailed("Rating model code is required".to_string()));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Rating model name is required".to_string()));
        }
        if self.repository.get_lead_rating_model_by_code(org_id, &code).await?.is_some() {
            return Err(AtlasError::Conflict(format!("Lead rating model '{}' already exists", code)));
        }
        info!("Creating lead rating model '{}' for org {}", code, org_id);
        self.repository.create_lead_rating_model(org_id, &code, name, description, scoring_criteria, created_by).await
    }

    /// List lead rating models
    pub async fn list_lead_rating_models(&self, org_id: Uuid) -> AtlasResult<Vec<atlas_shared::LeadRatingModel>> {
        self.repository.list_lead_rating_models(org_id).await
    }

    /// Delete a lead rating model
    pub async fn delete_lead_rating_model(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        self.repository.delete_lead_rating_model(org_id, code).await
    }

    // ========================================================================
    // Sales Lead Management
    // ========================================================================

    /// Create a new sales lead
    pub async fn create_lead(
        &self,
        org_id: Uuid,
        lead_number: &str,
        first_name: Option<&str>,
        last_name: Option<&str>,
        company: Option<&str>,
        title: Option<&str>,
        email: Option<&str>,
        phone: Option<&str>,
        website: Option<&str>,
        industry: Option<&str>,
        lead_source_id: Option<Uuid>,
        lead_source_name: Option<&str>,
        lead_rating_model_id: Option<Uuid>,
        estimated_value: &str,
        currency_code: &str,
        owner_id: Option<Uuid>,
        owner_name: Option<&str>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::SalesLead> {
        if lead_number.is_empty() {
            return Err(AtlasError::ValidationFailed("Lead number is required".to_string()));
        }

        // Validate lead source if provided
        let source_name = if let Some(src_id) = lead_source_id {
            let _src = self.repository.get_lead_source_by_code(org_id, &src_id.to_string()).await?;
            // Allow direct name pass-through
            lead_source_name.map(|s| s.to_string())
        } else {
            lead_source_name.map(|s| s.to_string())
        };

        // Validate estimated value
        let value: f64 = estimated_value.parse().unwrap_or(0.0);
        if value < 0.0 {
            return Err(AtlasError::ValidationFailed("Estimated value cannot be negative".to_string()));
        }

        // Check uniqueness
        if self.repository.get_lead_by_number(org_id, lead_number).await?.is_some() {
            return Err(AtlasError::Conflict(format!("Lead '{}' already exists", lead_number)));
        }

        info!("Creating sales lead '{}' for org {}", lead_number, org_id);

        self.repository.create_lead(
            org_id, lead_number, first_name, last_name, company, title,
            email, phone, website, industry, lead_source_id,
            source_name.as_deref(), lead_rating_model_id,
            estimated_value, currency_code, owner_id, owner_name, notes, created_by,
        ).await
    }

    /// Get a lead by ID
    pub async fn get_lead(&self, id: Uuid) -> AtlasResult<Option<atlas_shared::SalesLead>> {
        self.repository.get_lead(id).await
    }

    /// Get a lead by number
    pub async fn get_lead_by_number(&self, org_id: Uuid, lead_number: &str) -> AtlasResult<Option<atlas_shared::SalesLead>> {
        self.repository.get_lead_by_number(org_id, lead_number).await
    }

    /// List leads with optional filters
    pub async fn list_leads(&self, org_id: Uuid, status: Option<&str>, owner_id: Option<Uuid>) -> AtlasResult<Vec<atlas_shared::SalesLead>> {
        if let Some(s) = status {
            if !VALID_LEAD_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid lead status '{}'. Must be one of: {}", s, VALID_LEAD_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_leads(org_id, status, owner_id).await
    }

    /// Update lead status
    pub async fn update_lead_status(&self, id: Uuid, status: &str) -> AtlasResult<atlas_shared::SalesLead> {
        if !VALID_LEAD_STATUSES.contains(&status) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid lead status '{}'. Must be one of: {}", status, VALID_LEAD_STATUSES.join(", ")
            )));
        }
        let lead = self.repository.get_lead(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Lead {} not found", id)))?;
        info!("Updating lead {} status to {}", lead.lead_number, status);
        self.repository.update_lead_status(id, status).await
    }

    /// Update lead score and rating
    pub async fn update_lead_score(&self, id: Uuid, score: &str, rating: &str) -> AtlasResult<atlas_shared::SalesLead> {
        if !VALID_LEAD_RATINGS.contains(&rating) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid lead rating '{}'. Must be one of: {}", rating, VALID_LEAD_RATINGS.join(", ")
            )));
        }
        let score_val: f64 = score.parse().map_err(|_| {
            AtlasError::ValidationFailed("Lead score must be a number".to_string())
        })?;
        if !(0.0..=100.0).contains(&score_val) {
            return Err(AtlasError::ValidationFailed("Lead score must be between 0 and 100".to_string()));
        }
        self.repository.update_lead_score(id, score, rating).await
    }

    /// Convert a lead to an opportunity (and optionally a customer)
    pub async fn convert_lead(
        &self,
        id: Uuid,
        customer_id: Option<Uuid>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<(atlas_shared::SalesLead, atlas_shared::SalesOpportunity)> {
        let lead = self.repository.get_lead(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Lead {} not found", id)))?;

        if lead.status == "converted" {
            return Err(AtlasError::WorkflowError("Lead is already converted".to_string()));
        }

        info!("Converting lead {} to opportunity", lead.lead_number);

        // Create opportunity from lead data
        let opp_number = format!("OPP-{}", &lead.lead_number.replace("LD-", ""));
        let opp_name = format!("{} - {}",
            lead.company.as_deref().unwrap_or("New Opportunity"),
            lead.lead_number,
        );

        let opportunity = self.repository.create_opportunity(
            lead.organization_id,
            &opp_number,
            &opp_name,
            None,
            customer_id,
            None,
            Some(id),
            None,
            None,
            &lead.estimated_value,
            &lead.currency_code,
            "25", // initial probability for new opportunity
            None,
            lead.owner_id,
            lead.owner_name.as_deref(),
            None,
            None,
            created_by,
        ).await?;

        // Mark lead as converted
        let converted_lead = self.repository.convert_lead(id, Some(opportunity.id), customer_id).await?;

        Ok((converted_lead, opportunity))
    }

    /// Delete a lead
    pub async fn delete_lead(&self, org_id: Uuid, lead_number: &str) -> AtlasResult<()> {
        self.repository.delete_lead(org_id, lead_number).await
    }

    // ========================================================================
    // Opportunity Stages
    // ========================================================================

    /// Create an opportunity pipeline stage
    pub async fn create_opportunity_stage(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        probability: &str,
        display_order: i32,
        is_won: bool,
        is_lost: bool,
        created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::OpportunityStage> {
        let code = code.to_uppercase();
        if code.is_empty() {
            return Err(AtlasError::ValidationFailed("Stage code is required".to_string()));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Stage name is required".to_string()));
        }
        let prob: f64 = probability.parse().unwrap_or(0.0);
        if !(0.0..=100.0).contains(&prob) {
            return Err(AtlasError::ValidationFailed("Probability must be 0-100".to_string()));
        }
        if self.repository.get_opportunity_stage_by_code(org_id, &code).await?.is_some() {
            return Err(AtlasError::Conflict(format!("Opportunity stage '{}' already exists", code)));
        }
        info!("Creating opportunity stage '{}' for org {}", code, org_id);
        self.repository.create_opportunity_stage(
            org_id, &code, name, description, probability,
            display_order, is_won, is_lost, created_by,
        ).await
    }

    /// List opportunity stages
    pub async fn list_opportunity_stages(&self, org_id: Uuid) -> AtlasResult<Vec<atlas_shared::OpportunityStage>> {
        self.repository.list_opportunity_stages(org_id).await
    }

    /// Delete an opportunity stage
    pub async fn delete_opportunity_stage(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        self.repository.delete_opportunity_stage(org_id, code).await
    }

    // ========================================================================
    // Sales Opportunity Management
    // ========================================================================

    /// Create a new opportunity
    pub async fn create_opportunity(
        &self,
        org_id: Uuid,
        opportunity_number: &str,
        name: &str,
        description: Option<&str>,
        customer_id: Option<Uuid>,
        customer_name: Option<&str>,
        lead_id: Option<Uuid>,
        stage_id: Option<Uuid>,
        amount: &str,
        currency_code: &str,
        probability: &str,
        expected_close_date: Option<chrono::NaiveDate>,
        owner_id: Option<Uuid>,
        owner_name: Option<&str>,
        contact_id: Option<Uuid>,
        contact_name: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::SalesOpportunity> {
        if opportunity_number.is_empty() {
            return Err(AtlasError::ValidationFailed("Opportunity number is required".to_string()));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Opportunity name is required".to_string()));
        }

        let amt: f64 = amount.parse().unwrap_or(0.0);
        if amt < 0.0 {
            return Err(AtlasError::ValidationFailed("Amount cannot be negative".to_string()));
        }

        let prob: f64 = probability.parse().unwrap_or(0.0);
        if !(0.0..=100.0).contains(&prob) {
            return Err(AtlasError::ValidationFailed("Probability must be 0-100".to_string()));
        }

        // Resolve stage name from stage_id
        let stage_name = if let Some(sid) = stage_id {
            self.repository.get_opportunity_stage(sid).await?.map(|s| s.name.clone())
        } else {
            None
        };

        // Check uniqueness
        if self.repository.get_opportunity_by_number(org_id, opportunity_number).await?.is_some() {
            return Err(AtlasError::Conflict(format!("Opportunity '{}' already exists", opportunity_number)));
        }

        info!("Creating opportunity '{}' for org {}", opportunity_number, org_id);

        self.repository.create_opportunity(
            org_id, opportunity_number, name, description,
            customer_id, customer_name, lead_id, stage_id,
            stage_name.as_deref(), amount, currency_code, probability,
            expected_close_date, owner_id, owner_name,
            contact_id, contact_name, created_by,
        ).await
    }

    /// Get an opportunity by ID
    pub async fn get_opportunity(&self, id: Uuid) -> AtlasResult<Option<atlas_shared::SalesOpportunity>> {
        self.repository.get_opportunity(id).await
    }

    /// List opportunities with optional filters
    pub async fn list_opportunities(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        owner_id: Option<Uuid>,
        stage_id: Option<Uuid>,
    ) -> AtlasResult<Vec<atlas_shared::SalesOpportunity>> {
        if let Some(s) = status {
            if !VALID_OPP_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid opportunity status '{}'. Must be one of: {}", s, VALID_OPP_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_opportunities(org_id, status, owner_id, stage_id).await
    }

    /// Update opportunity stage (pipeline progression)
    pub async fn update_opportunity_stage(
        &self,
        id: Uuid,
        stage_id: Option<Uuid>,
        changed_by: Option<Uuid>,
        changed_by_name: Option<&str>,
        notes: Option<&str>,
    ) -> AtlasResult<atlas_shared::SalesOpportunity> {
        let opp = self.repository.get_opportunity(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Opportunity {} not found", id)))?;

        if opp.status != "open" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot change stage of '{}' opportunity", opp.status
            )));
        }

        let (stage_name, probability, weighted) = if let Some(sid) = stage_id {
            let stage = self.repository.get_opportunity_stage(sid).await?
                .ok_or_else(|| AtlasError::EntityNotFound(format!("Stage {} not found", sid)))?;
            let amt: f64 = opp.amount.parse().unwrap_or(0.0);
            let prob: f64 = stage.probability.parse().unwrap_or(0.0);
            (Some(stage.name.clone()), stage.probability.clone(), format!("{:.2}", amt * prob / 100.0))
        } else {
            (None, opp.probability.clone(), opp.weighted_amount.clone())
        };

        let old_stage = opp.stage_name.clone();
        let updated = self.repository.update_opportunity_stage(
            id, stage_id, stage_name.as_deref(), &probability, &weighted,
        ).await?;

        // Record stage history
        let _ = self.repository.add_stage_history(
            opp.organization_id, id,
            old_stage.as_deref(), stage_name.as_deref().unwrap_or("Unknown"),
            changed_by, changed_by_name, notes,
        ).await;

        info!("Opportunity {} moved to stage {}", opp.opportunity_number, stage_name.as_deref().unwrap_or("Unknown"));
        Ok(updated)
    }

    /// Close opportunity as won
    pub async fn close_opportunity_won(&self, id: Uuid) -> AtlasResult<atlas_shared::SalesOpportunity> {
        let opp = self.repository.get_opportunity(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Opportunity {} not found", id)))?;
        if opp.status != "open" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot close '{}' opportunity as won. Must be 'open'.", opp.status
            )));
        }
        info!("Closing opportunity {} as WON", opp.opportunity_number);
        self.repository.close_opportunity_won(id).await
    }

    /// Close opportunity as lost
    pub async fn close_opportunity_lost(&self, id: Uuid, lost_reason: Option<&str>) -> AtlasResult<atlas_shared::SalesOpportunity> {
        let opp = self.repository.get_opportunity(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Opportunity {} not found", id)))?;
        if opp.status != "open" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot close '{}' opportunity as lost. Must be 'open'.", opp.status
            )));
        }
        info!("Closing opportunity {} as LOST: {}", opp.opportunity_number, lost_reason.unwrap_or("N/A"));
        self.repository.close_opportunity_lost(id, lost_reason).await
    }

    /// Delete an opportunity
    pub async fn delete_opportunity(&self, org_id: Uuid, opp_number: &str) -> AtlasResult<()> {
        self.repository.delete_opportunity(org_id, opp_number).await
    }

    /// List stage history for an opportunity
    pub async fn list_stage_history(&self, opportunity_id: Uuid) -> AtlasResult<Vec<atlas_shared::OpportunityStageHistory>> {
        self.repository.list_stage_history(opportunity_id).await
    }

    // ========================================================================
    // Opportunity Lines
    // ========================================================================

    /// Add a line item to an opportunity
    pub async fn add_opportunity_line(
        &self,
        org_id: Uuid,
        opportunity_id: Uuid,
        product_name: &str,
        product_code: Option<&str>,
        description: Option<&str>,
        quantity: &str,
        unit_price: &str,
        discount_percent: &str,
    ) -> AtlasResult<atlas_shared::OpportunityLine> {
        if product_name.is_empty() {
            return Err(AtlasError::ValidationFailed("Product name is required".to_string()));
        }

        // Verify opportunity exists
        self.repository.get_opportunity(opportunity_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Opportunity {} not found", opportunity_id)))?;

        let qty: f64 = quantity.parse().unwrap_or(1.0);
        let price: f64 = unit_price.parse().unwrap_or(0.0);
        let disc: f64 = discount_percent.parse().unwrap_or(0.0);
        let line_amount = qty * price * (1.0 - disc / 100.0);

        // Get next line number
        let existing = self.repository.list_opportunity_lines(opportunity_id).await?;
        let line_number = (existing.len() as i32) + 1;

        info!("Adding line {} to opportunity {}", product_name, opportunity_id);

        self.repository.add_opportunity_line(
            org_id, opportunity_id, line_number, product_name, product_code,
            description, quantity, unit_price, &format!("{:.2}", line_amount), discount_percent,
        ).await
    }

    /// List opportunity lines
    pub async fn list_opportunity_lines(&self, opportunity_id: Uuid) -> AtlasResult<Vec<atlas_shared::OpportunityLine>> {
        self.repository.list_opportunity_lines(opportunity_id).await
    }

    /// Delete an opportunity line
    pub async fn delete_opportunity_line(&self, id: Uuid) -> AtlasResult<()> {
        self.repository.delete_opportunity_line(id).await
    }

    // ========================================================================
    // Sales Activities
    // ========================================================================

    /// Create a sales activity
    pub async fn create_activity(
        &self,
        org_id: Uuid,
        subject: &str,
        description: Option<&str>,
        activity_type: &str,
        priority: &str,
        lead_id: Option<Uuid>,
        opportunity_id: Option<Uuid>,
        contact_id: Option<Uuid>,
        contact_name: Option<&str>,
        owner_id: Option<Uuid>,
        owner_name: Option<&str>,
        start_at: Option<chrono::DateTime<chrono::Utc>>,
        end_at: Option<chrono::DateTime<chrono::Utc>>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::SalesActivity> {
        if subject.is_empty() {
            return Err(AtlasError::ValidationFailed("Activity subject is required".to_string()));
        }
        if !VALID_ACTIVITY_TYPES.contains(&activity_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid activity_type '{}'. Must be one of: {}", activity_type, VALID_ACTIVITY_TYPES.join(", ")
            )));
        }
        if !VALID_ACTIVITY_PRIORITIES.contains(&priority) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid priority '{}'. Must be one of: {}", priority, VALID_ACTIVITY_PRIORITIES.join(", ")
            )));
        }

        self.repository.create_activity(
            org_id, subject, description, activity_type, priority,
            lead_id, opportunity_id, contact_id, contact_name,
            owner_id, owner_name, start_at, end_at, created_by,
        ).await
    }

    /// Get an activity
    pub async fn get_activity(&self, id: Uuid) -> AtlasResult<Option<atlas_shared::SalesActivity>> {
        self.repository.get_activity(id).await
    }

    /// List activities
    pub async fn list_activities(
        &self,
        org_id: Uuid,
        lead_id: Option<Uuid>,
        opportunity_id: Option<Uuid>,
    ) -> AtlasResult<Vec<atlas_shared::SalesActivity>> {
        self.repository.list_activities(org_id, lead_id, opportunity_id).await
    }

    /// Complete an activity
    pub async fn complete_activity(&self, id: Uuid, outcome: Option<&str>) -> AtlasResult<atlas_shared::SalesActivity> {
        let activity = self.repository.get_activity(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Activity {} not found", id)))?;
        if activity.status != "planned" && activity.status != "in_progress" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot complete activity in '{}' status", activity.status
            )));
        }
        info!("Completing activity: {}", activity.subject);
        self.repository.complete_activity(id, outcome).await
    }

    /// Cancel an activity
    pub async fn cancel_activity(&self, id: Uuid) -> AtlasResult<atlas_shared::SalesActivity> {
        let activity = self.repository.get_activity(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Activity {} not found", id)))?;
        if activity.status == "completed" || activity.status == "cancelled" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot cancel activity in '{}' status", activity.status
            )));
        }
        self.repository.cancel_activity(id).await
    }

    /// Delete an activity
    pub async fn delete_activity(&self, id: Uuid) -> AtlasResult<()> {
        self.repository.delete_activity(id).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get the sales pipeline dashboard
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<atlas_shared::SalesPipelineDashboard> {
        self.repository.get_dashboard(org_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_lead_statuses() {
        assert!(VALID_LEAD_STATUSES.contains(&"new"));
        assert!(VALID_LEAD_STATUSES.contains(&"contacted"));
        assert!(VALID_LEAD_STATUSES.contains(&"qualified"));
        assert!(VALID_LEAD_STATUSES.contains(&"unqualified"));
        assert!(VALID_LEAD_STATUSES.contains(&"converted"));
        assert!(VALID_LEAD_STATUSES.contains(&"archived"));
        assert!(!VALID_LEAD_STATUSES.contains(&"deleted"));
    }

    #[test]
    fn test_valid_lead_ratings() {
        assert!(VALID_LEAD_RATINGS.contains(&"hot"));
        assert!(VALID_LEAD_RATINGS.contains(&"warm"));
        assert!(VALID_LEAD_RATINGS.contains(&"cold"));
        assert!(!VALID_LEAD_RATINGS.contains(&"frozen"));
    }

    #[test]
    fn test_valid_opp_statuses() {
        assert!(VALID_OPP_STATUSES.contains(&"open"));
        assert!(VALID_OPP_STATUSES.contains(&"won"));
        assert!(VALID_OPP_STATUSES.contains(&"lost"));
        assert!(!VALID_OPP_STATUSES.contains(&"pending"));
    }

    #[test]
    fn test_valid_activity_types() {
        assert!(VALID_ACTIVITY_TYPES.contains(&"call"));
        assert!(VALID_ACTIVITY_TYPES.contains(&"meeting"));
        assert!(VALID_ACTIVITY_TYPES.contains(&"email"));
        assert!(VALID_ACTIVITY_TYPES.contains(&"task"));
        assert!(VALID_ACTIVITY_TYPES.contains(&"demo"));
        assert!(VALID_ACTIVITY_TYPES.contains(&"presentation"));
        assert!(VALID_ACTIVITY_TYPES.contains(&"other"));
        assert!(!VALID_ACTIVITY_TYPES.contains(&"fax"));
    }

    #[test]
    fn test_valid_activity_priorities() {
        assert!(VALID_ACTIVITY_PRIORITIES.contains(&"low"));
        assert!(VALID_ACTIVITY_PRIORITIES.contains(&"medium"));
        assert!(VALID_ACTIVITY_PRIORITIES.contains(&"high"));
        assert!(VALID_ACTIVITY_PRIORITIES.contains(&"critical"));
        assert!(!VALID_ACTIVITY_PRIORITIES.contains(&"urgent"));
    }

    #[test]
    fn test_weighted_amount_calculation() {
        let amount: f64 = 100000.0;
        let probability: f64 = 25.0;
        let weighted = amount * probability / 100.0;
        assert!((weighted - 25000.0).abs() < 0.01);

        let probability: f64 = 75.0;
        let weighted = amount * probability / 100.0;
        assert!((weighted - 75000.0).abs() < 0.01);
    }

    #[test]
    fn test_line_amount_with_discount() {
        let qty: f64 = 10.0;
        let price: f64 = 100.0;
        let disc: f64 = 15.0;
        let line_amount = qty * price * (1.0 - disc / 100.0);
        assert!((line_amount - 850.0).abs() < 0.01);
    }

    #[test]
    fn test_win_rate_calculation() {
        let won = 30;
        let total = 100;
        let win_rate = (won as f64 / total as f64) * 100.0;
        assert!((win_rate - 30.0).abs() < 0.01);

        // Zero total
        let total = 0;
        let win_rate = if total > 0 { (won as f64 / total as f64) * 100.0 } else { 0.0 };
        assert!((win_rate - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_score_range() {
        assert!((0.0..=100.0).contains(&0.0));
        assert!((0.0..=100.0).contains(&50.0));
        assert!((0.0..=100.0).contains(&100.0));
        assert!(!(0.0..=100.0).contains(&-1.0));
        assert!(!(0.0..=100.0).contains(&101.0));
    }
}
