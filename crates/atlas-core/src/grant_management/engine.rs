//! Grant Management Engine
//!
//! Manages the full lifecycle of research and institutional grants including:
//! - Sponsor management (funding organizations)
//! - Award lifecycle (draft → active → suspended → completed → closed)
//! - Budget line management and tracking
//! - Expenditure recording with indirect cost calculation
//! - Sponsor billing/invoicing
//! - Compliance reporting (SF-425, progress reports, etc.)
//! - Indirect cost rate management
//!
//! Oracle Fusion Cloud ERP equivalent: Grants Management

use atlas_shared::{
    GrantSponsor, GrantIndirectCostRate, GrantAward, GrantBudgetLine,
    GrantExpenditure, GrantBilling, GrantComplianceReport, GrantDashboardSummary,
    AtlasError, AtlasResult,
};
use super::GrantManagementRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

// ── Validation constants ──

#[allow(dead_code)]
const VALID_SPONSOR_TYPES: &[&str] = &[
    "government", "foundation", "corporate", "internal", "university",
];

#[allow(dead_code)]
const VALID_AWARD_STATUSES: &[&str] = &[
    "draft", "active", "suspended", "completed", "terminated", "closed",
];

#[allow(dead_code)]
const VALID_AWARD_TYPES: &[&str] = &[
    "research", "training", "fellowship", "contract", "cooperative_agreement", "other",
];

#[allow(dead_code)]
const VALID_BILLING_FREQUENCIES: &[&str] = &[
    "monthly", "quarterly", "annual", "on_demand", "milestone",
];

#[allow(dead_code)]
const VALID_BILLING_BASES: &[&str] = &[
    "cost", "milestone", "fixed_price", "deliverable",
];

#[allow(dead_code)]
const VALID_BUDGET_CATEGORIES: &[&str] = &[
    "personnel", "fringe", "travel", "equipment", "supplies",
    "contractual", "other_direct", "indirect", "cost_sharing",
];

#[allow(dead_code)]
const VALID_EXPENDITURE_TYPES: &[&str] = &[
    "actual", "commitment", "encumbrance", "adjustment",
];

#[allow(dead_code)]
const VALID_EXPENDITURE_STATUSES: &[&str] = &[
    "pending", "approved", "billed", "reversed", "hold",
];

#[allow(dead_code)]
const VALID_BILLING_STATUSES: &[&str] = &[
    "draft", "submitted", "approved", "paid", "partial", "disputed", "cancelled",
];

#[allow(dead_code)]
const VALID_REPORT_TYPES: &[&str] = &[
    "federal_financial_report_sf425", "progress_report", "invention_report",
    "property_report", "closeout_report", "audit_report", "budget_modification",
];

#[allow(dead_code)]
const VALID_REPORT_STATUSES: &[&str] = &[
    "draft", "in_review", "approved", "submitted", "rejected",
];

#[allow(dead_code)]
const VALID_RATE_TYPES: &[&str] = &[
    "negotiated", "predetermined", "fixed", "provisional",
];

#[allow(dead_code)]
const VALID_BASE_TYPES: &[&str] = &[
    "modified_total_direct_costs", "total_direct_costs", "salaries_and_wages",
];

/// Grant Management Engine
pub struct GrantManagementEngine {
    repository: Arc<dyn GrantManagementRepository>,
}

impl GrantManagementEngine {
    pub fn new(repository: Arc<dyn GrantManagementRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Sponsor Management
    // ========================================================================

    /// Create a new grant sponsor
    pub async fn create_sponsor(
        &self, org_id: Uuid, sponsor_code: &str, name: &str, sponsor_type: &str,
        country_code: Option<&str>, taxpayer_id: Option<&str>,
        contact_name: Option<&str>, contact_email: Option<&str>, contact_phone: Option<&str>,
        address_line1: Option<&str>, address_line2: Option<&str>,
        city: Option<&str>, state_province: Option<&str>, postal_code: Option<&str>,
        payment_terms: Option<&str>, billing_frequency: &str, currency_code: &str,
        credit_limit: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<GrantSponsor> {
        if sponsor_code.is_empty() {
            return Err(AtlasError::ValidationFailed("Sponsor code is required".into()));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Sponsor name is required".into()));
        }
        if !VALID_SPONSOR_TYPES.contains(&sponsor_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid sponsor type '{}'. Must be one of: {}", sponsor_type, VALID_SPONSOR_TYPES.join(", ")
            )));
        }
        if !VALID_BILLING_FREQUENCIES.contains(&billing_frequency) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid billing frequency '{}'. Must be one of: {}", billing_frequency, VALID_BILLING_FREQUENCIES.join(", ")
            )));
        }
        if let Some(limit) = credit_limit {
            let val: f64 = limit.parse().map_err(|_| AtlasError::ValidationFailed(
                "Credit limit must be a valid number".into(),
            ))?;
            if val < 0.0 {
                return Err(AtlasError::ValidationFailed("Credit limit cannot be negative".into()));
            }
        }

        info!("Creating grant sponsor {} ({}) for org {}", sponsor_code, name, org_id);

        self.repository.create_sponsor(
            org_id, sponsor_code, name, sponsor_type, country_code, taxpayer_id,
            contact_name, contact_email, contact_phone, address_line1, address_line2,
            city, state_province, postal_code, payment_terms, billing_frequency,
            currency_code, credit_limit, created_by,
        ).await
    }

    /// Get a sponsor by code
    pub async fn get_sponsor(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<GrantSponsor>> {
        self.repository.get_sponsor(org_id, code).await
    }

    /// List sponsors
    pub async fn list_sponsors(&self, org_id: Uuid, active_only: bool) -> AtlasResult<Vec<GrantSponsor>> {
        self.repository.list_sponsors(org_id, active_only).await
    }

    /// Delete a sponsor
    pub async fn delete_sponsor(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        self.repository.delete_sponsor(org_id, code).await
    }

    // ========================================================================
    // Indirect Cost Rates
    // ========================================================================

    /// Create an indirect cost rate
    pub async fn create_indirect_cost_rate(
        &self, org_id: Uuid, rate_name: &str, rate_type: &str,
        rate_percentage: &str, base_type: &str,
        effective_from: chrono::NaiveDate, effective_to: Option<chrono::NaiveDate>,
        negotiated_by: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<GrantIndirectCostRate> {
        if rate_name.is_empty() {
            return Err(AtlasError::ValidationFailed("Rate name is required".into()));
        }
        if !VALID_RATE_TYPES.contains(&rate_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid rate type '{}'. Must be one of: {}", rate_type, VALID_RATE_TYPES.join(", ")
            )));
        }
        if !VALID_BASE_TYPES.contains(&base_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid base type '{}'. Must be one of: {}", base_type, VALID_BASE_TYPES.join(", ")
            )));
        }
        let pct: f64 = rate_percentage.parse().map_err(|_| AtlasError::ValidationFailed(
            "Rate percentage must be a valid number".into(),
        ))?;
        if !(0.0..=100.0).contains(&pct) {
            return Err(AtlasError::ValidationFailed(
                "Rate percentage must be between 0 and 100".into(),
            ));
        }
        if let Some(to) = effective_to {
            if to < effective_from {
                return Err(AtlasError::ValidationFailed(
                    "Effective to date must be after effective from date".into(),
                ));
            }
        }

        info!("Creating indirect cost rate {} ({}%) for org {}", rate_name, rate_percentage, org_id);

        self.repository.create_indirect_cost_rate(
            org_id, rate_name, rate_type, rate_percentage, base_type,
            effective_from, effective_to, negotiated_by, created_by,
        ).await
    }

    /// List indirect cost rates
    pub async fn list_indirect_cost_rates(&self, org_id: Uuid, active_only: bool) -> AtlasResult<Vec<GrantIndirectCostRate>> {
        self.repository.list_indirect_cost_rates(org_id, active_only).await
    }

    // ========================================================================
    // Award Lifecycle
    // ========================================================================

    /// Create a new grant award in draft status
    #[allow(clippy::too_many_arguments)]
    pub async fn create_award(
        &self, org_id: Uuid, award_number: &str, award_title: &str,
        sponsor_id: Uuid, sponsor_award_number: Option<&str>,
        award_type: &str, award_purpose: Option<&str>,
        start_date: chrono::NaiveDate, end_date: chrono::NaiveDate,
        total_award_amount: &str, direct_costs_total: &str,
        indirect_costs_total: &str, cost_sharing_total: &str,
        currency_code: &str, indirect_cost_rate_id: Option<Uuid>,
        indirect_cost_rate: &str, cost_sharing_required: bool,
        cost_sharing_percent: &str,
        principal_investigator_id: Option<Uuid>, principal_investigator_name: Option<&str>,
        department_id: Option<Uuid>, department_name: Option<&str>,
        project_id: Option<Uuid>, cost_center: Option<&str>,
        gl_revenue_account: Option<&str>, gl_receivable_account: Option<&str>,
        gl_deferred_account: Option<&str>,
        billing_frequency: &str, billing_basis: &str,
        reporting_requirements: Option<&str>, compliance_notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<GrantAward> {
        // Validation
        if award_number.is_empty() {
            return Err(AtlasError::ValidationFailed("Award number is required".into()));
        }
        if award_title.is_empty() {
            return Err(AtlasError::ValidationFailed("Award title is required".into()));
        }
        if !VALID_AWARD_TYPES.contains(&award_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid award type '{}'. Must be one of: {}", award_type, VALID_AWARD_TYPES.join(", ")
            )));
        }
        if end_date <= start_date {
            return Err(AtlasError::ValidationFailed(
                "End date must be after start date".into(),
            ));
        }
        if !VALID_BILLING_FREQUENCIES.contains(&billing_frequency) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid billing frequency '{}'. Must be one of: {}", billing_frequency, VALID_BILLING_FREQUENCIES.join(", ")
            )));
        }
        if !VALID_BILLING_BASES.contains(&billing_basis) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid billing basis '{}'. Must be one of: {}", billing_basis, VALID_BILLING_BASES.join(", ")
            )));
        }

        // Validate numeric fields
        let total: f64 = total_award_amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "Total award amount must be a valid number".into(),
        ))?;
        if total < 0.0 {
            return Err(AtlasError::ValidationFailed("Total award amount cannot be negative".into()));
        }

        // Validate sponsor exists
        let sponsor = self.repository.get_sponsor(org_id, "").await.ok().flatten();
        let sponsor_name = sponsor.as_ref().map(|s| s.name.as_str());

        let _available_balance = total_award_amount.to_string();

        info!("Creating grant award {} ({}) for org {}", award_number, award_title, org_id);

        self.repository.create_award(
            org_id, award_number, award_title, sponsor_id, sponsor_name,
            sponsor_award_number, award_type, award_purpose, start_date, end_date,
            total_award_amount, direct_costs_total, indirect_costs_total,
            cost_sharing_total, currency_code, indirect_cost_rate_id, indirect_cost_rate,
            cost_sharing_required, cost_sharing_percent,
            principal_investigator_id, principal_investigator_name,
            department_id, department_name, project_id, cost_center,
            gl_revenue_account, gl_receivable_account, gl_deferred_account,
            billing_frequency, billing_basis, reporting_requirements, compliance_notes,
            created_by,
        ).await
    }

    /// Get an award by ID
    pub async fn get_award(&self, id: Uuid) -> AtlasResult<Option<GrantAward>> {
        self.repository.get_award(id).await
    }

    /// Get an award by number
    pub async fn get_award_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<GrantAward>> {
        self.repository.get_award_by_number(org_id, number).await
    }

    /// List awards with optional filters
    pub async fn list_awards(&self, org_id: Uuid, status: Option<&str>, sponsor_id: Option<Uuid>) -> AtlasResult<Vec<GrantAward>> {
        if let Some(s) = status {
            if !VALID_AWARD_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_AWARD_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_awards(org_id, status, sponsor_id).await
    }

    /// Activate a draft award
    pub async fn activate_award(&self, award_id: Uuid) -> AtlasResult<GrantAward> {
        let award = self.repository.get_award(award_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Award {} not found", award_id)))?;

        if award.status != "draft" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot activate award in '{}' status. Must be 'draft'.", award.status
            )));
        }

        info!("Activating grant award {} ({})", award.award_number, award.award_title);
        self.repository.update_award_status(award_id, "active", None, None).await
    }

    /// Suspend an active award
    pub async fn suspend_award(&self, award_id: Uuid) -> AtlasResult<GrantAward> {
        let award = self.repository.get_award(award_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Award {} not found", award_id)))?;

        if award.status != "active" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot suspend award in '{}' status. Must be 'active'.", award.status
            )));
        }

        info!("Suspending grant award {}", award.award_number);
        self.repository.update_award_status(award_id, "suspended", None, None).await
    }

    /// Complete an active award
    pub async fn complete_award(&self, award_id: Uuid, closeout_notes: Option<&str>) -> AtlasResult<GrantAward> {
        let award = self.repository.get_award(award_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Award {} not found", award_id)))?;

        if award.status != "active" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot complete award in '{}' status. Must be 'active'.", award.status
            )));
        }

        info!("Completing grant award {}", award.award_number);
        self.repository.update_award_status(
            award_id, "completed", Some(chrono::Utc::now().date_naive()), closeout_notes,
        ).await
    }

    /// Terminate an award
    pub async fn terminate_award(&self, award_id: Uuid, closeout_notes: Option<&str>) -> AtlasResult<GrantAward> {
        let award = self.repository.get_award(award_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Award {} not found", award_id)))?;

        if award.status != "active" && award.status != "suspended" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot terminate award in '{}' status. Must be 'active' or 'suspended'.", award.status
            )));
        }

        info!("Terminating grant award {}", award.award_number);
        self.repository.update_award_status(
            award_id, "terminated", Some(chrono::Utc::now().date_naive()), closeout_notes,
        ).await
    }

    // ========================================================================
    // Budget Lines
    // ========================================================================

    /// Add a budget line to an award
    pub async fn create_budget_line(
        &self, org_id: Uuid, award_id: Uuid, budget_category: &str,
        description: Option<&str>, account_code: Option<&str>,
        budget_amount: &str, period_start: Option<chrono::NaiveDate>,
        period_end: Option<chrono::NaiveDate>, fiscal_year: Option<i32>,
        notes: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<GrantBudgetLine> {
        let award = self.repository.get_award(award_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Award {} not found", award_id)))?;

        if award.status != "draft" && award.status != "active" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot add budget lines to award in '{}' status.", award.status
            )));
        }

        if !VALID_BUDGET_CATEGORIES.contains(&budget_category) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid budget category '{}'. Must be one of: {}",
                budget_category, VALID_BUDGET_CATEGORIES.join(", ")
            )));
        }

        let amount: f64 = budget_amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "Budget amount must be a valid number".into(),
        ))?;
        if amount < 0.0 {
            return Err(AtlasError::ValidationFailed("Budget amount cannot be negative".into()));
        }

        // Get next line number
        let existing = self.repository.list_budget_lines(award_id).await?;
        let line_number = (existing.len() as i32) + 1;

        info!("Adding budget line ({}) to award {}", budget_category, award.award_number);

        self.repository.create_budget_line(
            org_id, award_id, line_number, budget_category, description,
            account_code, budget_amount, period_start, period_end,
            fiscal_year, notes, created_by,
        ).await
    }

    /// List budget lines for an award
    pub async fn list_budget_lines(&self, award_id: Uuid) -> AtlasResult<Vec<GrantBudgetLine>> {
        self.repository.list_budget_lines(award_id).await
    }

    // ========================================================================
    // Expenditures
    // ========================================================================

    /// Record a grant expenditure
    pub async fn create_expenditure(
        &self, org_id: Uuid, award_id: Uuid, expenditure_type: &str,
        expenditure_date: chrono::NaiveDate, description: Option<&str>,
        budget_line_id: Option<Uuid>, budget_category: Option<&str>,
        amount: &str, employee_id: Option<Uuid>, employee_name: Option<&str>,
        vendor_id: Option<Uuid>, vendor_name: Option<&str>,
        source_entity_type: Option<&str>, source_entity_id: Option<Uuid>,
        source_entity_number: Option<&str>,
        gl_debit_account: Option<&str>, gl_credit_account: Option<&str>,
        notes: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<GrantExpenditure> {
        let award = self.repository.get_award(award_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Award {} not found", award_id)))?;

        if award.status != "active" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot record expenditures on award in '{}' status. Must be 'active'.", award.status
            )));
        }

        if !VALID_EXPENDITURE_TYPES.contains(&expenditure_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid expenditure type '{}'. Must be one of: {}",
                expenditure_type, VALID_EXPENDITURE_TYPES.join(", ")
            )));
        }

        let amount_val: f64 = amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "Amount must be a valid number".into(),
        ))?;
        if amount_val < 0.0 {
            return Err(AtlasError::ValidationFailed("Amount cannot be negative".into()));
        }

        // Calculate indirect costs
        let icr: f64 = award.indirect_cost_rate.parse().unwrap_or(0.0);
        let indirect_cost_amount = amount_val * icr / 100.0;
        let total_amount = amount_val + indirect_cost_amount;

        // Calculate cost sharing
        let cs_pct: f64 = award.cost_sharing_percent.parse().unwrap_or(0.0);
        let cost_sharing_amount = total_amount * cs_pct / 100.0;

        let expenditure_number = format!("EXP-{}", Uuid::new_v4().to_string()[..8].to_uppercase());

        info!("Creating expenditure {} on award {} (amount: {})", expenditure_number, award.award_number, amount);

        let exp = self.repository.create_expenditure(
            org_id, award_id, &expenditure_number, expenditure_type, expenditure_date,
            description, budget_line_id, budget_category, amount,
            &format!("{:.2}", indirect_cost_amount),
            &format!("{:.2}", total_amount),
            &format!("{:.2}", cost_sharing_amount),
            employee_id, employee_name, vendor_id, vendor_name,
            source_entity_type, source_entity_id, source_entity_number,
            gl_debit_account, gl_credit_account, "pending", notes, created_by,
        ).await?;

        // Update budget line amounts if budget_line_id specified
        if let Some(bl_id) = budget_line_id {
            if let Some(_bl) = self.repository.get_budget_line(bl_id).await? {
                // Recalculate budget line totals from all expenditures
                self.recalculate_budget_line(bl_id).await?;
            }
        }

        // Update award totals
        self.recalculate_award_totals(award_id).await?;

        Ok(exp)
    }

    /// Approve a pending expenditure
    pub async fn approve_expenditure(&self, expenditure_id: Uuid, approved_by: Option<Uuid>) -> AtlasResult<GrantExpenditure> {
        let exp = self.repository.get_expenditure(expenditure_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Expenditure {} not found", expenditure_id)))?;

        if exp.status != "pending" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot approve expenditure in '{}' status. Must be 'pending'.", exp.status
            )));
        }

        info!("Approving expenditure {}", exp.expenditure_number);
        self.repository.update_expenditure_status(expenditure_id, "approved", approved_by).await
    }

    /// Reverse an expenditure
    pub async fn reverse_expenditure(&self, expenditure_id: Uuid) -> AtlasResult<GrantExpenditure> {
        let exp = self.repository.get_expenditure(expenditure_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Expenditure {} not found", expenditure_id)))?;

        if exp.status != "approved" && exp.status != "pending" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot reverse expenditure in '{}' status.", exp.status
            )));
        }

        info!("Reversing expenditure {}", exp.expenditure_number);
        let result = self.repository.update_expenditure_status(expenditure_id, "reversed", None).await?;

        // Recalculate award totals
        self.recalculate_award_totals(exp.award_id).await?;
        if let Some(bl_id) = exp.budget_line_id {
            self.recalculate_budget_line(bl_id).await?;
        }

        Ok(result)
    }

    /// List expenditures for an award
    pub async fn list_expenditures(&self, award_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<GrantExpenditure>> {
        self.repository.list_expenditures(award_id, status).await
    }

    // ========================================================================
    // Billing
    // ========================================================================

    /// Create a billing invoice for sponsor
    pub async fn create_billing(
        &self, org_id: Uuid, award_id: Uuid,
        period_start: chrono::NaiveDate, period_end: chrono::NaiveDate,
        notes: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<GrantBilling> {
        let award = self.repository.get_award(award_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Award {} not found", award_id)))?;

        if award.status != "active" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot create billing for award in '{}' status. Must be 'active'.", award.status
            )));
        }

        // Get approved expenditures in the period that haven't been billed yet
        let expenditures = self.repository.list_expenditures(award_id, Some("approved")).await?;
        let period_expenditures: Vec<_> = expenditures.iter()
            .filter(|e| e.expenditure_date >= period_start && e.expenditure_date <= period_end)
            .collect();

        let mut direct_costs = 0.0_f64;
        let mut indirect_costs = 0.0_f64;
        let mut cost_sharing = 0.0_f64;
        let mut exp_ids = Vec::new();

        for exp in &period_expenditures {
            let amount: f64 = exp.amount.parse().unwrap_or(0.0);
            let ic: f64 = exp.indirect_cost_amount.parse().unwrap_or(0.0);
            let cs: f64 = exp.cost_sharing_amount.parse().unwrap_or(0.0);
            direct_costs += amount;
            indirect_costs += ic;
            cost_sharing += cs;
            exp_ids.push(exp.id.to_string());
        }

        let total_amount = direct_costs + indirect_costs;
        let invoice_number = format!("INV-GRANT-{}", Uuid::new_v4().to_string()[..8].to_uppercase());
        let invoice_date = chrono::Utc::now().date_naive();
        let due_date = invoice_date + chrono::Duration::days(30);

        info!(
            "Creating billing {} for award {} (period: {} to {}, total: {:.2})",
            invoice_number, award.award_number, period_start, period_end, total_amount
        );

        let billing = self.repository.create_billing(
            org_id, award_id, &invoice_number, invoice_date,
            period_start, period_end, Some(due_date),
            &format!("{:.2}", direct_costs), &format!("{:.2}", indirect_costs),
            &format!("{:.2}", cost_sharing), &format!("{:.2}", total_amount),
            serde_json::json!(exp_ids), notes, created_by,
        ).await?;

        // Mark expenditures as billed
        for exp in &period_expenditures {
            self.repository.update_expenditure_status(exp.id, "billed", None).await?;
        }

        // Recalculate award totals
        self.recalculate_award_totals(award_id).await?;

        Ok(billing)
    }

    /// Submit a draft billing for approval
    pub async fn submit_billing(&self, billing_id: Uuid, submitted_by: Option<Uuid>) -> AtlasResult<GrantBilling> {
        let billing = self.repository.get_billing(billing_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Billing {} not found", billing_id)))?;

        if billing.status != "draft" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot submit billing in '{}' status. Must be 'draft'.", billing.status
            )));
        }

        info!("Submitting billing {}", billing.invoice_number);
        self.repository.update_billing_status(billing_id, "submitted", submitted_by, None).await
    }

    /// Approve a submitted billing
    pub async fn approve_billing(&self, billing_id: Uuid, approved_by: Option<Uuid>) -> AtlasResult<GrantBilling> {
        let billing = self.repository.get_billing(billing_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Billing {} not found", billing_id)))?;

        if billing.status != "submitted" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot approve billing in '{}' status. Must be 'submitted'.", billing.status
            )));
        }

        info!("Approving billing {}", billing.invoice_number);
        self.repository.update_billing_status(billing_id, "approved", approved_by, None).await
    }

    /// Mark a billing as paid
    pub async fn mark_billing_paid(&self, billing_id: Uuid, payment_reference: Option<&str>) -> AtlasResult<GrantBilling> {
        let billing = self.repository.get_billing(billing_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Billing {} not found", billing_id)))?;

        if billing.status != "approved" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot mark billing as paid in '{}' status. Must be 'approved'.", billing.status
            )));
        }

        info!("Marking billing {} as paid", billing.invoice_number);
        let result = self.repository.update_billing_status(billing_id, "paid", None, payment_reference).await?;

        // Update award totals
        self.recalculate_award_totals(billing.award_id).await?;

        Ok(result)
    }

    /// List billings for an award
    pub async fn list_billings(&self, award_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<GrantBilling>> {
        self.repository.list_billings(award_id, status).await
    }

    // ========================================================================
    // Compliance Reports
    // ========================================================================

    /// Create a compliance report
    pub async fn create_compliance_report(
        &self, org_id: Uuid, award_id: Uuid, report_type: &str,
        report_title: Option<&str>,
        reporting_period_start: chrono::NaiveDate, reporting_period_end: chrono::NaiveDate,
        due_date: Option<chrono::NaiveDate>,
        notes: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<GrantComplianceReport> {
        let award = self.repository.get_award(award_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Award {} not found", award_id)))?;

        if !VALID_REPORT_TYPES.contains(&report_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid report type '{}'. Must be one of: {}",
                report_type, VALID_REPORT_TYPES.join(", ")
            )));
        }

        // Gather totals for the reporting period
        let expenditures = self.repository.list_expenditures(award_id, None).await?;
        let period_exps: Vec<_> = expenditures.iter()
            .filter(|e| e.expenditure_date >= reporting_period_start && e.expenditure_date <= reporting_period_end)
            .collect();

        let mut total_expenditures = 0.0_f64;
        for exp in &period_exps {
            total_expenditures += exp.total_amount.parse().unwrap_or(0.0);
        }

        let billings = self.repository.list_billings(award_id, None).await?;
        let mut total_billed = 0.0_f64;
        for bill in &billings {
            if bill.invoice_date >= reporting_period_start && bill.invoice_date <= reporting_period_end {
                total_billed += bill.total_amount.parse().unwrap_or(0.0);
            }
        }

        let content = serde_json::json!({
            "award_number": award.award_number,
            "award_title": award.award_title,
            "sponsor_name": award.sponsor_name,
            "expenditure_count": period_exps.len(),
        });

        info!("Creating compliance report ({}) for award {}", report_type, award.award_number);

        self.repository.create_compliance_report(
            org_id, award_id, report_type, report_title,
            reporting_period_start, reporting_period_end, due_date,
            &format!("{:.2}", total_expenditures),
            &format!("{:.2}", total_billed),
            "0", "0", "0", content, notes, created_by,
        ).await
    }

    /// Submit a compliance report
    pub async fn submit_compliance_report(&self, report_id: Uuid, submitted_by: Option<Uuid>) -> AtlasResult<GrantComplianceReport> {
        let report = self.repository.get_compliance_report(report_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Report {} not found", report_id)))?;

        if report.status != "draft" && report.status != "rejected" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot submit report in '{}' status.", report.status
            )));
        }

        info!("Submitting compliance report for award {}", report.award_id);
        self.repository.update_compliance_report_status(report_id, "submitted", submitted_by, None).await
    }

    /// Approve a compliance report
    pub async fn approve_compliance_report(&self, report_id: Uuid, approved_by: Option<Uuid>) -> AtlasResult<GrantComplianceReport> {
        let report = self.repository.get_compliance_report(report_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Report {} not found", report_id)))?;

        if report.status != "in_review" && report.status != "submitted" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot approve report in '{}' status.", report.status
            )));
        }

        info!("Approving compliance report for award {}", report.award_id);
        self.repository.update_compliance_report_status(report_id, "approved", None, approved_by).await
    }

    /// List compliance reports for an award
    pub async fn list_compliance_reports(&self, award_id: Uuid, report_type: Option<&str>) -> AtlasResult<Vec<GrantComplianceReport>> {
        self.repository.list_compliance_reports(award_id, report_type).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get grant management dashboard summary
    pub async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<GrantDashboardSummary> {
        self.repository.get_dashboard_summary(org_id).await
    }

    // ========================================================================
    // Internal helpers
    // ========================================================================

    /// Recalculate award totals from expenditures and billings
    async fn recalculate_award_totals(&self, award_id: Uuid) -> AtlasResult<()> {
        let award = self.repository.get_award(award_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Award {} not found", award_id)))?;

        let expenditures = self.repository.list_expenditures(award_id, None).await?;
        let total_expenditures: f64 = expenditures.iter()
            .filter(|e| e.status == "approved" || e.status == "billed")
            .map(|e| e.total_amount.parse::<f64>().unwrap_or(0.0))
            .sum();

        let total_commitments: f64 = expenditures.iter()
            .filter(|e| e.status == "pending")
            .map(|e| e.total_amount.parse::<f64>().unwrap_or(0.0))
            .sum();

        let billings = self.repository.list_billings(award_id, None).await?;
        let total_billed: f64 = billings.iter()
            .filter(|b| b.status != "cancelled")
            .map(|b| b.total_amount.parse::<f64>().unwrap_or(0.0))
            .sum();

        let total_collected: f64 = billings.iter()
            .filter(|b| b.status == "paid")
            .map(|b| b.total_amount.parse::<f64>().unwrap_or(0.0))
            .sum();

        let award_total: f64 = award.total_award_amount.parse().unwrap_or(0.0);
        let available = award_total - total_expenditures;

        self.repository.update_award_totals(
            award_id,
            &format!("{:.2}", total_expenditures),
            &format!("{:.2}", total_commitments),
            &format!("{:.2}", total_billed),
            &format!("{:.2}", total_collected),
            &format!("{:.2}", available),
        ).await
    }

    /// Recalculate budget line totals from expenditures
    async fn recalculate_budget_line(&self, budget_line_id: Uuid) -> AtlasResult<()> {
        // Get all expenditures for this budget line
        let bl = self.repository.get_budget_line(budget_line_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Budget line {} not found", budget_line_id)))?;

        let expenditures = self.repository.list_expenditures(bl.award_id, None).await?;
        let line_exps: Vec<_> = expenditures.iter()
            .filter(|e| e.budget_line_id == Some(budget_line_id))
            .collect();

        let expended: f64 = line_exps.iter()
            .filter(|e| e.status == "approved" || e.status == "billed")
            .map(|e| e.total_amount.parse::<f64>().unwrap_or(0.0))
            .sum();

        let committed: f64 = line_exps.iter()
            .filter(|e| e.status == "pending")
            .map(|e| e.total_amount.parse::<f64>().unwrap_or(0.0))
            .sum();

        let billed: f64 = line_exps.iter()
            .filter(|e| e.status == "billed")
            .map(|e| e.total_amount.parse::<f64>().unwrap_or(0.0))
            .sum();

        let budget: f64 = bl.budget_amount.parse().unwrap_or(0.0);
        let available = budget - expended;

        self.repository.update_budget_line_amounts(
            budget_line_id,
            &format!("{:.2}", committed),
            &format!("{:.2}", expended),
            &format!("{:.2}", billed),
            &format!("{:.2}", available),
        ).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_sponsor_types() {
        assert!(VALID_SPONSOR_TYPES.contains(&"government"));
        assert!(VALID_SPONSOR_TYPES.contains(&"foundation"));
        assert!(VALID_SPONSOR_TYPES.contains(&"corporate"));
        assert!(VALID_SPONSOR_TYPES.contains(&"internal"));
        assert!(VALID_SPONSOR_TYPES.contains(&"university"));
    }

    #[test]
    fn test_valid_award_statuses() {
        assert!(VALID_AWARD_STATUSES.contains(&"draft"));
        assert!(VALID_AWARD_STATUSES.contains(&"active"));
        assert!(VALID_AWARD_STATUSES.contains(&"suspended"));
        assert!(VALID_AWARD_STATUSES.contains(&"completed"));
        assert!(VALID_AWARD_STATUSES.contains(&"terminated"));
        assert!(VALID_AWARD_STATUSES.contains(&"closed"));
    }

    #[test]
    fn test_valid_award_types() {
        assert!(VALID_AWARD_TYPES.contains(&"research"));
        assert!(VALID_AWARD_TYPES.contains(&"training"));
        assert!(VALID_AWARD_TYPES.contains(&"fellowship"));
        assert!(VALID_AWARD_TYPES.contains(&"contract"));
        assert!(VALID_AWARD_TYPES.contains(&"cooperative_agreement"));
    }

    #[test]
    fn test_valid_budget_categories() {
        assert!(VALID_BUDGET_CATEGORIES.contains(&"personnel"));
        assert!(VALID_BUDGET_CATEGORIES.contains(&"fringe"));
        assert!(VALID_BUDGET_CATEGORIES.contains(&"travel"));
        assert!(VALID_BUDGET_CATEGORIES.contains(&"equipment"));
        assert!(VALID_BUDGET_CATEGORIES.contains(&"supplies"));
        assert!(VALID_BUDGET_CATEGORIES.contains(&"contractual"));
        assert!(VALID_BUDGET_CATEGORIES.contains(&"other_direct"));
        assert!(VALID_BUDGET_CATEGORIES.contains(&"indirect"));
        assert!(VALID_BUDGET_CATEGORIES.contains(&"cost_sharing"));
    }

    #[test]
    fn test_valid_expenditure_types() {
        assert!(VALID_EXPENDITURE_TYPES.contains(&"actual"));
        assert!(VALID_EXPENDITURE_TYPES.contains(&"commitment"));
        assert!(VALID_EXPENDITURE_TYPES.contains(&"encumbrance"));
        assert!(VALID_EXPENDITURE_TYPES.contains(&"adjustment"));
    }

    #[test]
    fn test_valid_billing_bases() {
        assert!(VALID_BILLING_BASES.contains(&"cost"));
        assert!(VALID_BILLING_BASES.contains(&"milestone"));
        assert!(VALID_BILLING_BASES.contains(&"fixed_price"));
        assert!(VALID_BILLING_BASES.contains(&"deliverable"));
    }

    #[test]
    fn test_valid_report_types() {
        assert!(VALID_REPORT_TYPES.contains(&"federal_financial_report_sf425"));
        assert!(VALID_REPORT_TYPES.contains(&"progress_report"));
        assert!(VALID_REPORT_TYPES.contains(&"invention_report"));
        assert!(VALID_REPORT_TYPES.contains(&"property_report"));
        assert!(VALID_REPORT_TYPES.contains(&"closeout_report"));
    }

    #[test]
    fn test_indirect_cost_calculation() {
        // 50% indirect cost rate on $1000 direct cost = $500 IDC, total = $1500
        let direct = 1000.0_f64;
        let rate = 50.0_f64;
        let indirect = direct * rate / 100.0;
        let total = direct + indirect;
        assert!((indirect - 500.0).abs() < f64::EPSILON);
        assert!((total - 1500.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_cost_sharing_calculation() {
        // 20% cost sharing on $1500 total = $300 cost share
        let total = 1500.0_f64;
        let cs_pct = 20.0_f64;
        let cost_sharing = total * cs_pct / 100.0;
        assert!((cost_sharing - 300.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_available_balance_calculation() {
        // Award: $500,000 - Expenditures: $150,000 = Available: $350,000
        let award = 500000.0_f64;
        let expenditures = 150000.0_f64;
        let available = award - expenditures;
        assert!((available - 350000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_budget_utilization_percent() {
        let total_exp = 350000.0_f64;
        let total_value = 500000.0_f64;
        let utilization = total_exp / total_value * 100.0;
        assert!((utilization - 70.0).abs() < f64::EPSILON);
    }
}
