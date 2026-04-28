//! Joint Venture Management Engine
//!
//! Manages joint venture agreements, partner ownership, AFEs,
//! cost and revenue distributions, and Joint Interest Billing (JIB).
//!
//! Oracle Fusion Cloud equivalent: Financials > Joint Venture Management

use atlas_shared::{
    JointVenture, JointVenturePartner, JointVentureAfe,
    JvCostDistribution, JvCostDistributionLine,
    JvRevenueDistribution, JvRevenueDistributionLine,
    JvBilling, JvBillingLine, JvDashboard,
    AtlasError, AtlasResult,
};
use super::JointVentureRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

// ============================================================================
// Valid enum constants
// ============================================================================

const VALID_VENTURE_STATUSES: &[&str] = &["draft", "active", "on_hold", "closed"];
const VALID_ACCOUNTING_METHODS: &[&str] = &["proportional", "equity", "cost_method"];
const VALID_BILLING_CYCLES: &[&str] = &["monthly", "quarterly", "semi_annual", "annual"];
const VALID_PARTNER_TYPES: &[&str] = &["operator", "non_operator", "carried_interest"];
const VALID_PARTNER_STATUSES: &[&str] = &["active", "withdrawn", "suspended"];
const VALID_ROLES: &[&str] = &["operator", "partner", "carried"];
const VALID_AFE_STATUSES: &[&str] = &["draft", "submitted", "approved", "rejected", "closed"];
const VALID_COST_TYPES: &[&str] = &["operating", "capital", "aba", "overhead"];
const VALID_DISTRIBUTION_STATUSES: &[&str] = &["draft", "posted", "reversed"];
const VALID_REVENUE_TYPES: &[&str] = &["sales", "royalty", "bonus", "other"];
const VALID_BILLING_TYPES: &[&str] = &["jib", "revenue", "adjustment"];
const VALID_BILLING_STATUSES: &[&str] = &["draft", "submitted", "approved", "paid", "disputed", "cancelled"];

/// Joint Venture Management Engine
pub struct JointVentureEngine {
    repository: Arc<dyn JointVentureRepository>,
}

impl JointVentureEngine {
    pub fn new(repository: Arc<dyn JointVentureRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Joint Venture CRUD
    // ========================================================================

    /// Create a new joint venture
    #[allow(clippy::too_many_arguments)]
    pub async fn create_venture(
        &self,
        org_id: Uuid,
        venture_number: &str,
        name: &str,
        description: Option<&str>,
        operator_id: Option<Uuid>,
        operator_name: Option<&str>,
        currency_code: &str,
        start_date: Option<chrono::NaiveDate>,
        end_date: Option<chrono::NaiveDate>,
        accounting_method: &str,
        billing_cycle: &str,
        cost_cap_amount: Option<&str>,
        cost_cap_currency: Option<&str>,
        gl_revenue_account: Option<&str>,
        gl_cost_account: Option<&str>,
        gl_billing_account: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<JointVenture> {
        if venture_number.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Venture number is required".to_string(),
            ));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Venture name is required".to_string(),
            ));
        }
        validate_enum("accounting_method", accounting_method, VALID_ACCOUNTING_METHODS)?;
        validate_enum("billing_cycle", billing_cycle, VALID_BILLING_CYCLES)?;
        if let (Some(start), Some(end)) = (start_date, end_date) {
            if start > end {
                return Err(AtlasError::ValidationFailed(
                    "Start date must be before end date".to_string(),
                ));
            }
        }
        if let Some(cap) = cost_cap_amount {
            let cap_val: f64 = cap.parse().map_err(|_| AtlasError::ValidationFailed(
                "Cost cap amount must be a valid number".to_string(),
            ))?;
            if cap_val < 0.0 {
                return Err(AtlasError::ValidationFailed(
                    "Cost cap amount must be non-negative".to_string(),
                ));
            }
        }

        info!("Creating joint venture '{}' ({}) for org {}", venture_number, name, org_id);

        self.repository.create_venture(
            org_id, venture_number, name, description,
            operator_id, operator_name,
            currency_code, start_date, end_date,
            accounting_method, billing_cycle,
            cost_cap_amount, cost_cap_currency,
            gl_revenue_account, gl_cost_account, gl_billing_account,
            created_by,
        ).await
    }

    /// Get a joint venture by ID
    pub async fn get_venture(&self, id: Uuid) -> AtlasResult<Option<JointVenture>> {
        self.repository.get_venture(id).await
    }

    /// Get a joint venture by number
    pub async fn get_venture_by_number(&self, org_id: Uuid, venture_number: &str) -> AtlasResult<Option<JointVenture>> {
        self.repository.get_venture_by_number(org_id, venture_number).await
    }

    /// List joint ventures with optional status filter
    pub async fn list_ventures(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<JointVenture>> {
        if let Some(s) = status {
            validate_enum("status", s, VALID_VENTURE_STATUSES)?;
        }
        self.repository.list_ventures(org_id, status).await
    }

    /// Activate a joint venture
    pub async fn activate_venture(&self, id: Uuid) -> AtlasResult<JointVenture> {
        let venture = self.repository.get_venture(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Joint venture {} not found", id)
            ))?;

        if venture.status != "draft" && venture.status != "on_hold" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot activate venture in '{}' status. Must be 'draft' or 'on_hold'.", venture.status)
            ));
        }

        info!("Activating joint venture {}", id);
        self.repository.update_venture_status(id, "active").await
    }

    /// Put a venture on hold
    pub async fn hold_venture(&self, id: Uuid) -> AtlasResult<JointVenture> {
        let venture = self.repository.get_venture(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Joint venture {} not found", id)
            ))?;

        if venture.status != "active" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot hold venture in '{}' status. Must be 'active'.", venture.status)
            ));
        }

        info!("Placing joint venture {} on hold", id);
        self.repository.update_venture_status(id, "on_hold").await
    }

    /// Close a joint venture
    pub async fn close_venture(&self, id: Uuid) -> AtlasResult<JointVenture> {
        let venture = self.repository.get_venture(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Joint venture {} not found", id)
            ))?;

        if venture.status == "closed" {
            return Err(AtlasError::WorkflowError(
                "Venture is already closed".to_string(),
            ));
        }

        info!("Closing joint venture {}", id);
        self.repository.update_venture_status(id, "closed").await
    }

    // ========================================================================
    // Partner Management
    // ========================================================================

    /// Add a partner to a joint venture
    #[allow(clippy::too_many_arguments)]
    pub async fn add_partner(
        &self,
        org_id: Uuid,
        venture_id: Uuid,
        partner_id: Uuid,
        partner_name: &str,
        partner_type: &str,
        ownership_percentage: &str,
        revenue_interest_pct: Option<&str>,
        cost_bearing_pct: Option<&str>,
        role: &str,
        billing_contact: Option<&str>,
        billing_email: Option<&str>,
        billing_address: Option<&str>,
        effective_from: chrono::NaiveDate,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<JointVenturePartner> {
        // Validate venture exists and is not closed
        let venture = self.repository.get_venture(venture_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Joint venture {} not found", venture_id)
            ))?;

        if venture.status == "closed" {
            return Err(AtlasError::WorkflowError(
                "Cannot add partners to a closed venture".to_string(),
            ));
        }

        if partner_name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Partner name is required".to_string(),
            ));
        }
        validate_enum("partner_type", partner_type, VALID_PARTNER_TYPES)?;
        validate_enum("role", role, VALID_ROLES)?;

        let ownership: f64 = ownership_percentage.parse().map_err(|_| AtlasError::ValidationFailed(
            "Ownership percentage must be a valid number".to_string(),
        ))?;
        if ownership < 0.0 || ownership > 100.0 {
            return Err(AtlasError::ValidationFailed(
                "Ownership percentage must be between 0 and 100".to_string(),
            ));
        }

        // Validate total ownership doesn't exceed 100%
        let existing_partners = self.repository.list_partners_by_venture(venture_id).await?;
        let mut total_ownership: f64 = ownership;
        for p in &existing_partners {
            if p.status == "active" {
                total_ownership += p.ownership_percentage.parse().unwrap_or(0.0);
            }
        }
        if total_ownership > 100.0 {
            return Err(AtlasError::ValidationFailed(
                format!("Total ownership would be {:.2}% which exceeds 100%", total_ownership)
            ));
        }

        if let (Some(from), Some(to)) = (Some(effective_from), effective_to) {
            if from > to {
                return Err(AtlasError::ValidationFailed(
                    "Effective from must be before effective to".to_string(),
                ));
            }
        }

        info!("Adding partner '{}' to venture {} with {:.2}% ownership", partner_name, venture_id, ownership);

        self.repository.create_partner(
            org_id, venture_id, partner_id, partner_name,
            partner_type, ownership_percentage,
            revenue_interest_pct, cost_bearing_pct,
            role, billing_contact, billing_email, billing_address,
            effective_from, effective_to, created_by,
        ).await
    }

    /// Get a partner by ID
    pub async fn get_partner(&self, id: Uuid) -> AtlasResult<Option<JointVenturePartner>> {
        self.repository.get_partner(id).await
    }

    /// List all partners for a venture
    pub async fn list_partners(&self, venture_id: Uuid) -> AtlasResult<Vec<JointVenturePartner>> {
        self.repository.list_partners_by_venture(venture_id).await
    }

    /// List active partners as of a given date
    pub async fn list_active_partners(&self, venture_id: Uuid, on_date: chrono::NaiveDate) -> AtlasResult<Vec<JointVenturePartner>> {
        self.repository.list_active_partners(venture_id, on_date).await
    }

    /// Withdraw a partner
    pub async fn withdraw_partner(&self, id: Uuid) -> AtlasResult<JointVenturePartner> {
        let partner = self.repository.get_partner(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Partner {} not found", id)
            ))?;

        if partner.status != "active" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot withdraw partner in '{}' status", partner.status)
            ));
        }

        info!("Withdrawing partner {} from venture {}", id, partner.venture_id);
        self.repository.update_partner_status(id, "withdrawn").await
    }

    /// Remove a partner
    pub async fn remove_partner(&self, id: Uuid) -> AtlasResult<()> {
        info!("Removing partner {}", id);
        self.repository.delete_partner(id).await
    }

    // ========================================================================
    // AFE Management
    // ========================================================================

    /// Create an AFE (Authorization for Expenditure)
    #[allow(clippy::too_many_arguments)]
    pub async fn create_afe(
        &self,
        org_id: Uuid,
        venture_id: Uuid,
        afe_number: &str,
        title: &str,
        description: Option<&str>,
        estimated_cost: &str,
        currency_code: &str,
        cost_center: Option<&str>,
        work_area: Option<&str>,
        well_name: Option<&str>,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<JointVentureAfe> {
        // Validate venture exists
        let venture = self.repository.get_venture(venture_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Joint venture {} not found", venture_id)
            ))?;

        if venture.status == "closed" {
            return Err(AtlasError::WorkflowError(
                "Cannot create AFEs for a closed venture".to_string(),
            ));
        }

        if afe_number.is_empty() {
            return Err(AtlasError::ValidationFailed("AFE number is required".to_string()));
        }
        if title.is_empty() {
            return Err(AtlasError::ValidationFailed("AFE title is required".to_string()));
        }

        let cost: f64 = estimated_cost.parse().map_err(|_| AtlasError::ValidationFailed(
            "Estimated cost must be a valid number".to_string(),
        ))?;
        if cost < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Estimated cost must be non-negative".to_string(),
            ));
        }

        info!("Creating AFE '{}' for venture {} (estimated: {})", afe_number, venture_id, estimated_cost);

        self.repository.create_afe(
            org_id, venture_id, afe_number, title, description,
            estimated_cost, currency_code,
            cost_center, work_area, well_name,
            effective_from, effective_to, created_by,
        ).await
    }

    /// Get an AFE by ID
    pub async fn get_afe(&self, id: Uuid) -> AtlasResult<Option<JointVentureAfe>> {
        self.repository.get_afe(id).await
    }

    /// Get an AFE by number
    pub async fn get_afe_by_number(&self, org_id: Uuid, afe_number: &str) -> AtlasResult<Option<JointVentureAfe>> {
        self.repository.get_afe_by_number(org_id, afe_number).await
    }

    /// List AFEs for a venture
    pub async fn list_afes(&self, venture_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<JointVentureAfe>> {
        if let Some(s) = status {
            validate_enum("status", s, VALID_AFE_STATUSES)?;
        }
        self.repository.list_afes_by_venture(venture_id, status).await
    }

    /// Submit an AFE for approval
    pub async fn submit_afe(&self, id: Uuid) -> AtlasResult<JointVentureAfe> {
        let afe = self.repository.get_afe(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("AFE {} not found", id)
            ))?;

        if afe.status != "draft" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot submit AFE in '{}' status. Must be 'draft'.", afe.status)
            ));
        }

        info!("Submitting AFE {} for approval", id);
        self.repository.update_afe_status(id, "submitted", None, None).await
    }

    /// Approve an AFE
    pub async fn approve_afe(&self, id: Uuid, approved_by: Uuid) -> AtlasResult<JointVentureAfe> {
        let afe = self.repository.get_afe(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("AFE {} not found", id)
            ))?;

        if afe.status != "submitted" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot approve AFE in '{}' status. Must be 'submitted'.", afe.status)
            ));
        }

        info!("Approving AFE {} by {}", id, approved_by);
        self.repository.update_afe_status(id, "approved", Some(approved_by), None).await
    }

    /// Reject an AFE
    pub async fn reject_afe(&self, id: Uuid, reason: &str) -> AtlasResult<JointVentureAfe> {
        let afe = self.repository.get_afe(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("AFE {} not found", id)
            ))?;

        if afe.status != "submitted" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot reject AFE in '{}' status. Must be 'submitted'.", afe.status)
            ));
        }

        if reason.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Rejection reason is required".to_string(),
            ));
        }

        info!("Rejecting AFE {}: {}", id, reason);
        self.repository.update_afe_status(id, "rejected", None, Some(reason)).await
    }

    /// Close an AFE
    pub async fn close_afe(&self, id: Uuid) -> AtlasResult<JointVentureAfe> {
        let afe = self.repository.get_afe(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("AFE {} not found", id)
            ))?;

        if afe.status != "approved" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot close AFE in '{}' status. Must be 'approved'.", afe.status)
            ));
        }

        info!("Closing AFE {}", id);
        self.repository.update_afe_status(id, "closed", None, None).await
    }

    // ========================================================================
    // Cost Distribution
    // ========================================================================

    /// Create a cost distribution and auto-distribute across partners
    #[allow(clippy::too_many_arguments)]
    pub async fn create_cost_distribution(
        &self,
        org_id: Uuid,
        venture_id: Uuid,
        distribution_number: &str,
        afe_id: Option<Uuid>,
        description: Option<&str>,
        total_amount: &str,
        currency_code: &str,
        cost_type: &str,
        distribution_date: chrono::NaiveDate,
        source_type: Option<&str>,
        source_id: Option<Uuid>,
        source_number: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<(JvCostDistribution, Vec<JvCostDistributionLine>)> {
        // Validate venture is active
        let venture = self.repository.get_venture(venture_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Joint venture {} not found", venture_id)
            ))?;

        if venture.status != "active" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot distribute costs for venture in '{}' status. Must be 'active'.", venture.status)
            ));
        }

        if distribution_number.is_empty() {
            return Err(AtlasError::ValidationFailed("Distribution number is required".to_string()));
        }

        validate_enum("cost_type", cost_type, VALID_COST_TYPES)?;

        let amount: f64 = total_amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "Total amount must be a valid number".to_string(),
        ))?;
        if amount < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Total amount must be non-negative".to_string(),
            ));
        }

        // Validate AFE if specified
        if let Some(afe_id) = afe_id {
            let afe = self.repository.get_afe(afe_id).await?
                .ok_or_else(|| AtlasError::EntityNotFound(
                    format!("AFE {} not found", afe_id)
                ))?;
            if afe.status != "approved" {
                return Err(AtlasError::WorkflowError(
                    format!("Cannot distribute against AFE in '{}' status. Must be 'approved'.", afe.status)
                ));
            }
        }

        // Create the distribution header
        let distribution = self.repository.create_cost_distribution(
            org_id, venture_id, distribution_number,
            afe_id, description, total_amount, currency_code,
            cost_type, distribution_date,
            source_type, source_id, source_number, created_by,
        ).await?;

        // Get active partners as of distribution date
        let partners = self.repository.list_active_partners(venture_id, distribution_date).await?;

        if partners.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "No active partners found for the distribution date".to_string(),
            ));
        }

        // Distribute cost across partners based on cost_bearing_pct
        let mut lines = Vec::new();
        for partner in &partners {
            let bearing_pct: f64 = partner.cost_bearing_pct
                .as_deref()
                .map(|p| p.parse().unwrap_or(0.0))
                .unwrap_or_else(|| partner.ownership_percentage.parse().unwrap_or(0.0));

            let distributed = (amount * bearing_pct) / 100.0;

            let line = self.repository.create_cost_distribution_line(
                org_id, distribution.id, partner.partner_id,
                Some(&partner.partner_name),
                &partner.ownership_percentage,
                &format!("{:.4}", bearing_pct),
                &format!("{:.4}", distributed),
                venture.gl_cost_account.as_deref(),
                Some(&format!("Cost distribution for {} ({:.2}%)", partner.partner_name, bearing_pct)),
            ).await?;

            lines.push(line);
        }

        info!("Created cost distribution {} for venture {} with {} partner lines", distribution_number, venture_id, lines.len());

        Ok((distribution, lines))
    }

    /// Get a cost distribution
    pub async fn get_cost_distribution(&self, id: Uuid) -> AtlasResult<Option<JvCostDistribution>> {
        self.repository.get_cost_distribution(id).await
    }

    /// List cost distributions for a venture
    pub async fn list_cost_distributions(&self, venture_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<JvCostDistribution>> {
        if let Some(s) = status {
            validate_enum("status", s, VALID_DISTRIBUTION_STATUSES)?;
        }
        self.repository.list_cost_distributions(venture_id, status).await
    }

    /// Post a cost distribution to GL
    pub async fn post_cost_distribution(&self, id: Uuid) -> AtlasResult<JvCostDistribution> {
        let dist = self.repository.get_cost_distribution(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Cost distribution {} not found", id)
            ))?;

        if dist.status != "draft" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot post distribution in '{}' status. Must be 'draft'.", dist.status)
            ));
        }

        info!("Posting cost distribution {} to GL", id);
        self.repository.update_cost_distribution_status(id, "posted").await
    }

    /// Reverse a posted cost distribution
    pub async fn reverse_cost_distribution(&self, id: Uuid) -> AtlasResult<JvCostDistribution> {
        let dist = self.repository.get_cost_distribution(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Cost distribution {} not found", id)
            ))?;

        if dist.status != "posted" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot reverse distribution in '{}' status. Must be 'posted'.", dist.status)
            ));
        }

        info!("Reversing cost distribution {}", id);
        self.repository.update_cost_distribution_status(id, "reversed").await
    }

    /// List cost distribution lines
    pub async fn list_cost_distribution_lines(&self, distribution_id: Uuid) -> AtlasResult<Vec<JvCostDistributionLine>> {
        self.repository.list_cost_distribution_lines(distribution_id).await
    }

    // ========================================================================
    // Revenue Distribution
    // ========================================================================

    /// Create a revenue distribution and auto-distribute across partners
    #[allow(clippy::too_many_arguments)]
    pub async fn create_revenue_distribution(
        &self,
        org_id: Uuid,
        venture_id: Uuid,
        distribution_number: &str,
        description: Option<&str>,
        total_amount: &str,
        currency_code: &str,
        revenue_type: &str,
        distribution_date: chrono::NaiveDate,
        source_type: Option<&str>,
        source_id: Option<Uuid>,
        source_number: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<(JvRevenueDistribution, Vec<JvRevenueDistributionLine>)> {
        // Validate venture is active
        let venture = self.repository.get_venture(venture_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Joint venture {} not found", venture_id)
            ))?;

        if venture.status != "active" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot distribute revenue for venture in '{}' status", venture.status)
            ));
        }

        if distribution_number.is_empty() {
            return Err(AtlasError::ValidationFailed("Distribution number is required".to_string()));
        }

        validate_enum("revenue_type", revenue_type, VALID_REVENUE_TYPES)?;

        let amount: f64 = total_amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "Total amount must be a valid number".to_string(),
        ))?;
        if amount < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Total amount must be non-negative".to_string(),
            ));
        }

        // Create the distribution header
        let distribution = self.repository.create_revenue_distribution(
            org_id, venture_id, distribution_number,
            description, total_amount, currency_code,
            revenue_type, distribution_date,
            source_type, source_id, source_number, created_by,
        ).await?;

        // Get active partners
        let partners = self.repository.list_active_partners(venture_id, distribution_date).await?;

        if partners.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "No active partners found for the distribution date".to_string(),
            ));
        }

        // Distribute revenue based on revenue_interest_pct
        let mut lines = Vec::new();
        for partner in &partners {
            let rev_pct: f64 = partner.revenue_interest_pct
                .as_deref()
                .map(|p| p.parse().unwrap_or(0.0))
                .unwrap_or_else(|| partner.ownership_percentage.parse().unwrap_or(0.0));

            let distributed = (amount * rev_pct) / 100.0;

            let line = self.repository.create_revenue_distribution_line(
                org_id, distribution.id, partner.partner_id,
                Some(&partner.partner_name),
                &format!("{:.4}", rev_pct),
                &format!("{:.4}", distributed),
                venture.gl_revenue_account.as_deref(),
                Some(&format!("Revenue distribution for {} ({:.2}%)", partner.partner_name, rev_pct)),
            ).await?;

            lines.push(line);
        }

        info!("Created revenue distribution {} for venture {} with {} partner lines", distribution_number, venture_id, lines.len());

        Ok((distribution, lines))
    }

    /// Get a revenue distribution
    pub async fn get_revenue_distribution(&self, id: Uuid) -> AtlasResult<Option<JvRevenueDistribution>> {
        self.repository.get_revenue_distribution(id).await
    }

    /// List revenue distributions for a venture
    pub async fn list_revenue_distributions(&self, venture_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<JvRevenueDistribution>> {
        if let Some(s) = status {
            validate_enum("status", s, VALID_DISTRIBUTION_STATUSES)?;
        }
        self.repository.list_revenue_distributions(venture_id, status).await
    }

    /// Post a revenue distribution
    pub async fn post_revenue_distribution(&self, id: Uuid) -> AtlasResult<JvRevenueDistribution> {
        let dist = self.repository.get_revenue_distribution(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Revenue distribution {} not found", id)
            ))?;

        if dist.status != "draft" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot post distribution in '{}' status. Must be 'draft'.", dist.status)
            ));
        }

        info!("Posting revenue distribution {} to GL", id);
        self.repository.update_revenue_distribution_status(id, "posted").await
    }

    /// Reverse a posted revenue distribution
    pub async fn reverse_revenue_distribution(&self, id: Uuid) -> AtlasResult<JvRevenueDistribution> {
        let dist = self.repository.get_revenue_distribution(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Revenue distribution {} not found", id)
            ))?;

        if dist.status != "posted" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot reverse distribution in '{}' status. Must be 'posted'.", dist.status)
            ));
        }

        info!("Reversing revenue distribution {}", id);
        self.repository.update_revenue_distribution_status(id, "reversed").await
    }

    /// List revenue distribution lines
    pub async fn list_revenue_distribution_lines(&self, distribution_id: Uuid) -> AtlasResult<Vec<JvRevenueDistributionLine>> {
        self.repository.list_revenue_distribution_lines(distribution_id).await
    }

    // ========================================================================
    // Joint Interest Billing (JIB)
    // ========================================================================

    /// Create a billing for a partner
    #[allow(clippy::too_many_arguments)]
    pub async fn create_billing(
        &self,
        org_id: Uuid,
        venture_id: Uuid,
        billing_number: &str,
        partner_id: Uuid,
        partner_name: Option<&str>,
        billing_type: &str,
        total_amount: &str,
        tax_amount: &str,
        currency_code: &str,
        billing_period_start: chrono::NaiveDate,
        billing_period_end: chrono::NaiveDate,
        due_date: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<JvBilling> {
        if billing_number.is_empty() {
            return Err(AtlasError::ValidationFailed("Billing number is required".to_string()));
        }
        validate_enum("billing_type", billing_type, VALID_BILLING_TYPES)?;

        let total: f64 = total_amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "Total amount must be a valid number".to_string(),
        ))?;
        let tax: f64 = tax_amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "Tax amount must be a valid number".to_string(),
        ))?;

        if total < 0.0 {
            return Err(AtlasError::ValidationFailed("Total amount must be non-negative".to_string()));
        }
        if tax < 0.0 {
            return Err(AtlasError::ValidationFailed("Tax amount must be non-negative".to_string()));
        }

        if billing_period_start > billing_period_end {
            return Err(AtlasError::ValidationFailed(
                "Billing period start must be before end".to_string(),
            ));
        }

        let total_with_tax = format!("{:.4}", total + tax);

        info!("Creating billing {} for partner {} in venture {}", billing_number, partner_id, venture_id);

        self.repository.create_billing(
            org_id, venture_id, billing_number,
            partner_id, partner_name, billing_type,
            total_amount, tax_amount, &total_with_tax,
            currency_code, billing_period_start, billing_period_end,
            due_date, created_by,
        ).await
    }

    /// Get a billing by ID
    pub async fn get_billing(&self, id: Uuid) -> AtlasResult<Option<JvBilling>> {
        self.repository.get_billing(id).await
    }

    /// Get a billing by number
    pub async fn get_billing_by_number(&self, org_id: Uuid, billing_number: &str) -> AtlasResult<Option<JvBilling>> {
        self.repository.get_billing_by_number(org_id, billing_number).await
    }

    /// List billings for a venture
    pub async fn list_billings(&self, venture_id: Uuid, status: Option<&str>, billing_type: Option<&str>) -> AtlasResult<Vec<JvBilling>> {
        if let Some(s) = status {
            validate_enum("status", s, VALID_BILLING_STATUSES)?;
        }
        if let Some(bt) = billing_type {
            validate_enum("billing_type", bt, VALID_BILLING_TYPES)?;
        }
        self.repository.list_billings(venture_id, status, billing_type).await
    }

    /// Submit a billing for approval
    pub async fn submit_billing(&self, id: Uuid) -> AtlasResult<JvBilling> {
        let billing = self.repository.get_billing(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Billing {} not found", id)
            ))?;

        if billing.status != "draft" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot submit billing in '{}' status. Must be 'draft'.", billing.status)
            ));
        }

        info!("Submitting billing {} for approval", id);
        self.repository.update_billing_status(id, "submitted", None, None, None).await
    }

    /// Approve a billing
    pub async fn approve_billing(&self, id: Uuid, approved_by: Uuid) -> AtlasResult<JvBilling> {
        let billing = self.repository.get_billing(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Billing {} not found", id)
            ))?;

        if billing.status != "submitted" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot approve billing in '{}' status. Must be 'submitted'.", billing.status)
            ));
        }

        info!("Approving billing {} by {}", id, approved_by);
        self.repository.update_billing_status(id, "approved", Some(approved_by), None, None).await
    }

    /// Record payment for a billing
    pub async fn pay_billing(&self, id: Uuid, payment_reference: &str) -> AtlasResult<JvBilling> {
        let billing = self.repository.get_billing(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Billing {} not found", id)
            ))?;

        if billing.status != "approved" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot pay billing in '{}' status. Must be 'approved'.", billing.status)
            ));
        }

        if payment_reference.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Payment reference is required".to_string(),
            ));
        }

        info!("Recording payment for billing {} (ref: {})", id, payment_reference);
        self.repository.update_billing_status(id, "paid", None, Some(payment_reference), None).await
    }

    /// Dispute a billing
    pub async fn dispute_billing(&self, id: Uuid, reason: &str) -> AtlasResult<JvBilling> {
        let billing = self.repository.get_billing(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Billing {} not found", id)
            ))?;

        if billing.status != "approved" && billing.status != "submitted" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot dispute billing in '{}' status", billing.status)
            ));
        }

        if reason.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Dispute reason is required".to_string(),
            ));
        }

        info!("Disputing billing {}: {}", id, reason);
        self.repository.update_billing_status(id, "disputed", None, None, Some(reason)).await
    }

    /// Cancel a draft billing
    pub async fn cancel_billing(&self, id: Uuid) -> AtlasResult<JvBilling> {
        let billing = self.repository.get_billing(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Billing {} not found", id)
            ))?;

        if billing.status != "draft" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot cancel billing in '{}' status. Must be 'draft'.", billing.status)
            ));
        }

        info!("Cancelling billing {}", id);
        self.repository.update_billing_status(id, "cancelled", None, None, None).await
    }

    /// Add a line to a billing
    pub async fn add_billing_line(
        &self,
        org_id: Uuid,
        billing_id: Uuid,
        line_number: i32,
        cost_distribution_id: Option<Uuid>,
        revenue_distribution_id: Option<Uuid>,
        description: Option<&str>,
        cost_type: Option<&str>,
        amount: &str,
        ownership_pct: Option<&str>,
    ) -> AtlasResult<JvBillingLine> {
        let billing = self.repository.get_billing(billing_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Billing {} not found", billing_id)
            ))?;

        if billing.status != "draft" {
            return Err(AtlasError::WorkflowError(
                "Can only add lines to a draft billing".to_string(),
            ));
        }

        let amount_val: f64 = amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "Amount must be a valid number".to_string(),
        ))?;
        if amount_val < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Amount must be non-negative".to_string(),
            ));
        }

        self.repository.create_billing_line(
            org_id, billing_id, line_number,
            cost_distribution_id, revenue_distribution_id,
            description, cost_type, amount, ownership_pct,
        ).await
    }

    /// List billing lines
    pub async fn list_billing_lines(&self, billing_id: Uuid) -> AtlasResult<Vec<JvBillingLine>> {
        self.repository.list_billing_lines(billing_id).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get the joint venture dashboard summary
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<JvDashboard> {
        self.repository.get_dashboard(org_id).await
    }
}

// ============================================================================
// Validation helpers
// ============================================================================

fn validate_enum(field: &str, value: &str, valid: &[&str]) -> AtlasResult<()> {
    if !valid.contains(&value) {
        Err(AtlasError::ValidationFailed(format!(
            "Invalid {} '{}'. Must be one of: {}", field, value, valid.join(", ")
        )))
    } else {
        Ok(())
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_venture_statuses() {
        assert!(VALID_VENTURE_STATUSES.contains(&"draft"));
        assert!(VALID_VENTURE_STATUSES.contains(&"active"));
        assert!(VALID_VENTURE_STATUSES.contains(&"on_hold"));
        assert!(VALID_VENTURE_STATUSES.contains(&"closed"));
    }

    #[test]
    fn test_valid_accounting_methods() {
        assert!(VALID_ACCOUNTING_METHODS.contains(&"proportional"));
        assert!(VALID_ACCOUNTING_METHODS.contains(&"equity"));
        assert!(VALID_ACCOUNTING_METHODS.contains(&"cost_method"));
    }

    #[test]
    fn test_valid_billing_cycles() {
        assert!(VALID_BILLING_CYCLES.contains(&"monthly"));
        assert!(VALID_BILLING_CYCLES.contains(&"quarterly"));
        assert!(VALID_BILLING_CYCLES.contains(&"semi_annual"));
        assert!(VALID_BILLING_CYCLES.contains(&"annual"));
    }

    #[test]
    fn test_valid_partner_types() {
        assert!(VALID_PARTNER_TYPES.contains(&"operator"));
        assert!(VALID_PARTNER_TYPES.contains(&"non_operator"));
        assert!(VALID_PARTNER_TYPES.contains(&"carried_interest"));
    }

    #[test]
    fn test_valid_roles() {
        assert!(VALID_ROLES.contains(&"operator"));
        assert!(VALID_ROLES.contains(&"partner"));
        assert!(VALID_ROLES.contains(&"carried"));
    }

    #[test]
    fn test_valid_afe_statuses() {
        assert!(VALID_AFE_STATUSES.contains(&"draft"));
        assert!(VALID_AFE_STATUSES.contains(&"submitted"));
        assert!(VALID_AFE_STATUSES.contains(&"approved"));
        assert!(VALID_AFE_STATUSES.contains(&"rejected"));
        assert!(VALID_AFE_STATUSES.contains(&"closed"));
    }

    #[test]
    fn test_valid_cost_types() {
        assert!(VALID_COST_TYPES.contains(&"operating"));
        assert!(VALID_COST_TYPES.contains(&"capital"));
        assert!(VALID_COST_TYPES.contains(&"aba"));
        assert!(VALID_COST_TYPES.contains(&"overhead"));
    }

    #[test]
    fn test_valid_distribution_statuses() {
        assert!(VALID_DISTRIBUTION_STATUSES.contains(&"draft"));
        assert!(VALID_DISTRIBUTION_STATUSES.contains(&"posted"));
        assert!(VALID_DISTRIBUTION_STATUSES.contains(&"reversed"));
    }

    #[test]
    fn test_valid_revenue_types() {
        assert!(VALID_REVENUE_TYPES.contains(&"sales"));
        assert!(VALID_REVENUE_TYPES.contains(&"royalty"));
        assert!(VALID_REVENUE_TYPES.contains(&"bonus"));
        assert!(VALID_REVENUE_TYPES.contains(&"other"));
    }

    #[test]
    fn test_valid_billing_types() {
        assert!(VALID_BILLING_TYPES.contains(&"jib"));
        assert!(VALID_BILLING_TYPES.contains(&"revenue"));
        assert!(VALID_BILLING_TYPES.contains(&"adjustment"));
    }

    #[test]
    fn test_valid_billing_statuses() {
        assert!(VALID_BILLING_STATUSES.contains(&"draft"));
        assert!(VALID_BILLING_STATUSES.contains(&"submitted"));
        assert!(VALID_BILLING_STATUSES.contains(&"approved"));
        assert!(VALID_BILLING_STATUSES.contains(&"paid"));
        assert!(VALID_BILLING_STATUSES.contains(&"disputed"));
        assert!(VALID_BILLING_STATUSES.contains(&"cancelled"));
    }

    #[test]
    fn test_validate_enum_valid() {
        assert!(validate_enum("test", "valid", &["valid", "also_valid"]).is_ok());
    }

    #[test]
    fn test_validate_enum_invalid() {
        let result = validate_enum("test", "invalid", &["valid", "also_valid"]);
        assert!(result.is_err());
        match result {
            Err(AtlasError::ValidationFailed(msg)) => {
                assert!(msg.contains("test"));
                assert!(msg.contains("invalid"));
            }
            _ => panic!("Expected ValidationFailed error"),
        }
    }

    #[test]
    fn test_validate_enum_empty() {
        let result = validate_enum("field", "", &["value1", "value2"]);
        assert!(result.is_err());
    }

    // ========================================================================
    // Integration-style tests with Mock Repository
    // ========================================================================

    use crate::mock_repos::MockJointVentureRepository;
    use chrono::NaiveDate;

    fn create_engine() -> JointVentureEngine {
        JointVentureEngine::new(Arc::new(MockJointVentureRepository))
    }

    fn test_org_id() -> Uuid {
        Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap()
    }

    fn test_user_id() -> Uuid {
        Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap()
    }

    #[tokio::test]
    async fn test_create_venture_validation_empty_number() {
        let engine = create_engine();
        let result = engine.create_venture(
            test_org_id(), "", "Test Venture", None,
            None, None, "USD", None, None,
            "proportional", "monthly", None, None,
            None, None, None, None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("number")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_venture_validation_empty_name() {
        let engine = create_engine();
        let result = engine.create_venture(
            test_org_id(), "JV-001", "", None,
            None, None, "USD", None, None,
            "proportional", "monthly", None, None,
            None, None, None, None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("name")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_venture_validation_bad_accounting_method() {
        let engine = create_engine();
        let result = engine.create_venture(
            test_org_id(), "JV-001", "Test Venture", None,
            None, None, "USD", None, None,
            "invalid_method", "monthly", None, None,
            None, None, None, None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("accounting_method")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_venture_validation_bad_billing_cycle() {
        let engine = create_engine();
        let result = engine.create_venture(
            test_org_id(), "JV-001", "Test Venture", None,
            None, None, "USD", None, None,
            "proportional", "weekly", None, None,
            None, None, None, None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("billing_cycle")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_venture_validation_date_range() {
        let engine = create_engine();
        let start = NaiveDate::from_ymd_opt(2025, 12, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        let result = engine.create_venture(
            test_org_id(), "JV-001", "Test Venture", None,
            None, None, "USD", Some(start), Some(end),
            "proportional", "monthly", None, None,
            None, None, None, None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("date")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_venture_validation_negative_cost_cap() {
        let engine = create_engine();
        let result = engine.create_venture(
            test_org_id(), "JV-001", "Test Venture", None,
            None, None, "USD", None, None,
            "proportional", "monthly", Some("-100"), None,
            None, None, None, None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("non-negative")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_venture_success() {
        let engine = create_engine();
        let result = engine.create_venture(
            test_org_id(), "JV-001", "Alpha Joint Venture", Some("Oil & gas partnership"),
            Some(test_user_id()), Some("Operator Corp"), "USD",
            Some(NaiveDate::from_ymd_opt(2025, 1, 1).unwrap()),
            Some(NaiveDate::from_ymd_opt(2030, 12, 31).unwrap()),
            "proportional", "monthly", Some("1000000"), Some("USD"),
            Some("4000"), Some("5000"), Some("6000"),
            Some(test_user_id()),
        ).await;
        assert!(result.is_ok());
        let venture = result.unwrap();
        assert_eq!(venture.venture_number, "JV-001");
        assert_eq!(venture.name, "Alpha Joint Venture");
        assert_eq!(venture.status, "draft");
        assert_eq!(venture.accounting_method, "proportional");
        assert_eq!(venture.billing_cycle, "monthly");
    }

    #[tokio::test]
    async fn test_activate_venture_not_found() {
        let engine = create_engine();
        let result = engine.activate_venture(Uuid::new_v4()).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::EntityNotFound(msg) => assert!(msg.contains("not found")),
            _ => panic!("Expected EntityNotFound"),
        }
    }

    // add_partner and create_afe require a venture to exist first
    // (mock returns None), so these test the EntityNotFound path

    #[tokio::test]
    async fn test_add_partner_venture_not_found() {
        let engine = create_engine();
        let result = engine.add_partner(
            test_org_id(), Uuid::new_v4(), Uuid::new_v4(), "",
            "operator", "50.00", None, None,
            "operator", None, None, None,
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(), None, None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::EntityNotFound(msg) => assert!(msg.contains("not found")),
            _ => panic!("Expected EntityNotFound"),
        }
    }

    #[tokio::test]
    async fn test_add_partner_invalid_type_venture_not_found() {
        let engine = create_engine();
        let result = engine.add_partner(
            test_org_id(), Uuid::new_v4(), Uuid::new_v4(), "Partner A",
            "invalid_type", "50.00", None, None,
            "operator", None, None, None,
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(), None, None,
        ).await;
        assert!(result.is_err());
        // Venture check comes first, so EntityNotFound
        match result.unwrap_err() {
            AtlasError::EntityNotFound(msg) => assert!(msg.contains("not found")),
            _ => panic!("Expected EntityNotFound"),
        }
    }

    #[tokio::test]
    async fn test_add_partner_ownership_over_100_venture_not_found() {
        let engine = create_engine();
        let result = engine.add_partner(
            test_org_id(), Uuid::new_v4(), Uuid::new_v4(), "Partner A",
            "non_operator", "150.00", None, None,
            "partner", None, None, None,
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(), None, None,
        ).await;
        assert!(result.is_err());
        // Venture check comes first
        match result.unwrap_err() {
            AtlasError::EntityNotFound(msg) => assert!(msg.contains("not found")),
            _ => panic!("Expected EntityNotFound"),
        }
    }

    #[tokio::test]
    async fn test_create_afe_venture_not_found() {
        let engine = create_engine();
        let result = engine.create_afe(
            test_org_id(), Uuid::new_v4(), "", "AFE Title", None,
            "100000", "USD", None, None, None,
            None, None, None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::EntityNotFound(msg) => assert!(msg.contains("not found")),
            _ => panic!("Expected EntityNotFound"),
        }
    }

    #[tokio::test]
    async fn test_create_afe_negative_cost_venture_not_found() {
        let engine = create_engine();
        let result = engine.create_afe(
            test_org_id(), Uuid::new_v4(), "AFE-001", "AFE Title", None,
            "-500", "USD", None, None, None,
            None, None, None,
        ).await;
        assert!(result.is_err());
        // Venture check comes first
        match result.unwrap_err() {
            AtlasError::EntityNotFound(msg) => assert!(msg.contains("not found")),
            _ => panic!("Expected EntityNotFound"),
        }
    }

    #[tokio::test]
    async fn test_create_billing_negative_amount() {
        let engine = create_engine();
        let result = engine.create_billing(
            test_org_id(), Uuid::new_v4(), "BIL-001",
            Uuid::new_v4(), Some("Partner"), "jib",
            "-100", "0", "USD",
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2025, 1, 31).unwrap(),
            None, None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("non-negative")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_billing_invalid_type() {
        let engine = create_engine();
        let result = engine.create_billing(
            test_org_id(), Uuid::new_v4(), "BIL-001",
            Uuid::new_v4(), Some("Partner"), "invalid",
            "100", "0", "USD",
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2025, 1, 31).unwrap(),
            None, None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("billing_type")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_create_billing_date_range_invalid() {
        let engine = create_engine();
        let result = engine.create_billing(
            test_org_id(), Uuid::new_v4(), "BIL-001",
            Uuid::new_v4(), Some("Partner"), "jib",
            "100", "0", "USD",
            NaiveDate::from_ymd_opt(2025, 2, 1).unwrap(),
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            None, None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("start must be before end")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_list_ventures_invalid_status() {
        let engine = create_engine();
        let result = engine.list_ventures(test_org_id(), Some("invalid")).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("status")),
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[tokio::test]
    async fn test_list_afes_invalid_status() {
        let engine = create_engine();
        let result = engine.list_afes(Uuid::new_v4(), Some("invalid")).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_cost_distributions_invalid_status() {
        let engine = create_engine();
        let result = engine.list_cost_distributions(Uuid::new_v4(), Some("invalid")).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_revenue_distributions_invalid_status() {
        let engine = create_engine();
        let result = engine.list_revenue_distributions(Uuid::new_v4(), Some("invalid")).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_billings_invalid_status() {
        let engine = create_engine();
        let result = engine.list_billings(Uuid::new_v4(), Some("invalid"), None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_billings_invalid_type() {
        let engine = create_engine();
        let result = engine.list_billings(Uuid::new_v4(), None, Some("invalid")).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_venture() {
        let engine = create_engine();
        // Mock returns None for non-existent
        let result = engine.get_venture(Uuid::new_v4()).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_get_venture_by_number() {
        let engine = create_engine();
        let result = engine.get_venture_by_number(test_org_id(), "JV-001").await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_list_ventures_no_filter() {
        let engine = create_engine();
        let result = engine.list_ventures(test_org_id(), None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_partner() {
        let engine = create_engine();
        let result = engine.get_partner(Uuid::new_v4()).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_get_billing() {
        let engine = create_engine();
        let result = engine.get_billing(Uuid::new_v4()).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_get_billing_by_number() {
        let engine = create_engine();
        let result = engine.get_billing_by_number(test_org_id(), "BIL-001").await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_get_afe() {
        let engine = create_engine();
        let result = engine.get_afe(Uuid::new_v4()).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_get_afe_by_number() {
        let engine = create_engine();
        let result = engine.get_afe_by_number(test_org_id(), "AFE-001").await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_get_cost_distribution() {
        let engine = create_engine();
        let result = engine.get_cost_distribution(Uuid::new_v4()).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_get_revenue_distribution() {
        let engine = create_engine();
        let result = engine.get_revenue_distribution(Uuid::new_v4()).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_activate_venture_wrong_status() {
        let engine = create_engine();
        // Create a venture first (it will be in draft status, which can be activated)
        // But mock returns None for get_venture, so it'll be EntityNotFound
        let result = engine.activate_venture(Uuid::new_v4()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_hold_venture_not_found() {
        let engine = create_engine();
        let result = engine.hold_venture(Uuid::new_v4()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_close_venture_not_found() {
        let engine = create_engine();
        let result = engine.close_venture(Uuid::new_v4()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_withdraw_partner_not_found() {
        let engine = create_engine();
        let result = engine.withdraw_partner(Uuid::new_v4()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_submit_afe_not_found() {
        let engine = create_engine();
        let result = engine.submit_afe(Uuid::new_v4()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_approve_afe_not_found() {
        let engine = create_engine();
        let result = engine.approve_afe(Uuid::new_v4(), test_user_id()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_reject_afe_not_found() {
        let engine = create_engine();
        let result = engine.reject_afe(Uuid::new_v4(), "Out of budget").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_reject_afe_empty_reason() {
        let engine = create_engine();
        // Need to create an AFE first to test empty reason
        // Since mock returns None for get_afe, this will be EntityNotFound
        let result = engine.reject_afe(Uuid::new_v4(), "").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_close_afe_not_found() {
        let engine = create_engine();
        let result = engine.close_afe(Uuid::new_v4()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_post_cost_distribution_not_found() {
        let engine = create_engine();
        let result = engine.post_cost_distribution(Uuid::new_v4()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_reverse_cost_distribution_not_found() {
        let engine = create_engine();
        let result = engine.reverse_cost_distribution(Uuid::new_v4()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_post_revenue_distribution_not_found() {
        let engine = create_engine();
        let result = engine.post_revenue_distribution(Uuid::new_v4()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_reverse_revenue_distribution_not_found() {
        let engine = create_engine();
        let result = engine.reverse_revenue_distribution(Uuid::new_v4()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_submit_billing_not_found() {
        let engine = create_engine();
        let result = engine.submit_billing(Uuid::new_v4()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_approve_billing_not_found() {
        let engine = create_engine();
        let result = engine.approve_billing(Uuid::new_v4(), test_user_id()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_pay_billing_not_found() {
        let engine = create_engine();
        let result = engine.pay_billing(Uuid::new_v4(), "PAY-001").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_dispute_billing_not_found() {
        let engine = create_engine();
        let result = engine.dispute_billing(Uuid::new_v4(), "Incorrect charges").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_cancel_billing_not_found() {
        let engine = create_engine();
        let result = engine.cancel_billing(Uuid::new_v4()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_dashboard() {
        let engine = create_engine();
        let result = engine.get_dashboard(test_org_id()).await;
        assert!(result.is_ok());
    }
}
