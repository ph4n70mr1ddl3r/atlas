//! Sales Commission Engine Implementation
//!
//! Manages sales representatives, commission plans with tiered rates,
//! plan assignments, quotas, commission transactions, and payout processing.
//!
//! Oracle Fusion Cloud ERP equivalent: Incentive Compensation

use atlas_shared::{
    SalesRepresentative, CommissionPlan, CommissionRateTier, PlanAssignment,
    SalesQuota, CommissionTransaction, CommissionPayout, CommissionPayoutLine,
    CommissionDashboardSummary, CommissionTopPerformer,
    AtlasError, AtlasResult,
};
use chrono::Datelike;
use super::SalesCommissionRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Valid plan types
const VALID_PLAN_TYPES: &[&str] = &[
    "revenue", "margin", "quantity", "flat_bonus",
];

/// Valid bases for commission calculation
const VALID_BASES: &[&str] = &[
    "revenue", "gross_margin", "net_margin", "quantity",
];

/// Valid calculation methods
const VALID_CALC_METHODS: &[&str] = &[
    "percentage", "tiered", "flat_rate", "graduated",
];

/// Valid quota types
const VALID_QUOTA_TYPES: &[&str] = &[
    "revenue", "units", "margin", "activities",
];

/// Sales Commission engine
pub struct SalesCommissionEngine {
    repository: Arc<dyn SalesCommissionRepository>,
}

impl SalesCommissionEngine {
    pub fn new(repository: Arc<dyn SalesCommissionRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Sales Representative Management
    // ========================================================================

    /// Create a new sales representative
    pub async fn create_rep(
        &self,
        org_id: Uuid,
        rep_code: &str,
        employee_id: Option<Uuid>,
        first_name: &str,
        last_name: &str,
        email: Option<&str>,
        territory_code: Option<&str>,
        territory_name: Option<&str>,
        manager_id: Option<Uuid>,
        manager_name: Option<&str>,
        hire_date: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<SalesRepresentative> {
        let code_upper = rep_code.to_uppercase();
        if code_upper.is_empty() || code_upper.len() > 50 {
            return Err(AtlasError::ValidationFailed(
                "Rep code must be 1-50 characters".to_string(),
            ));
        }
        if first_name.is_empty() || last_name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "First name and last name are required".to_string(),
            ));
        }

        info!("Creating sales rep '{}' ({}) for org {}", code_upper, first_name, org_id);

        self.repository.create_rep(
            org_id, &code_upper, employee_id,
            first_name, last_name, email,
            territory_code, territory_name,
            manager_id, manager_name, hire_date,
            created_by,
        ).await
    }

    /// Get a rep by code
    pub async fn get_rep(&self, org_id: Uuid, rep_code: &str) -> AtlasResult<Option<SalesRepresentative>> {
        self.repository.get_rep(org_id, &rep_code.to_uppercase()).await
    }

    /// List reps
    pub async fn list_reps(&self, org_id: Uuid, active_only: bool) -> AtlasResult<Vec<SalesRepresentative>> {
        self.repository.list_reps(org_id, active_only).await
    }

    /// Delete (soft-delete) a rep
    pub async fn delete_rep(&self, org_id: Uuid, rep_code: &str) -> AtlasResult<()> {
        info!("Deleting sales rep '{}' for org {}", rep_code, org_id);
        self.repository.delete_rep(org_id, &rep_code.to_uppercase()).await
    }

    // ========================================================================
    // Commission Plan Management
    // ========================================================================

    /// Create a new commission plan
    pub async fn create_plan(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        plan_type: &str,
        basis: &str,
        calculation_method: &str,
        default_rate: &str,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CommissionPlan> {
        let code_upper = code.to_uppercase();
        if code_upper.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Plan code is required".to_string(),
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
        if !VALID_BASES.contains(&basis) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid basis '{}'. Must be one of: {}", basis, VALID_BASES.join(", ")
            )));
        }
        if !VALID_CALC_METHODS.contains(&calculation_method) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid calculation_method '{}'. Must be one of: {}", calculation_method, VALID_CALC_METHODS.join(", ")
            )));
        }

        let rate: f64 = default_rate.parse().map_err(|_| AtlasError::ValidationFailed(
            "default_rate must be a valid number".to_string(),
        ))?;
        if rate < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "default_rate cannot be negative".to_string(),
            ));
        }

        info!("Creating commission plan '{}' ({}) for org {}", code_upper, name, org_id);

        self.repository.create_plan(
            org_id, &code_upper, name, description,
            plan_type, basis, calculation_method, default_rate,
            effective_from, effective_to, created_by,
        ).await
    }

    /// Get a plan by code
    pub async fn get_plan(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<CommissionPlan>> {
        self.repository.get_plan(org_id, &code.to_uppercase()).await
    }

    /// List plans
    pub async fn list_plans(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<CommissionPlan>> {
        self.repository.list_plans(org_id, status).await
    }

    /// Activate a commission plan
    pub async fn activate_plan(&self, id: Uuid) -> AtlasResult<CommissionPlan> {
        let plan = self.repository.get_plan_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Plan {} not found", id)))?;

        if plan.status != "draft" && plan.status != "inactive" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot activate plan in '{}' status", plan.status)
            ));
        }

        info!("Activating commission plan {}", plan.code);
        self.repository.update_plan_status(id, "active").await
    }

    /// Deactivate a commission plan
    pub async fn deactivate_plan(&self, id: Uuid) -> AtlasResult<CommissionPlan> {
        let plan = self.repository.get_plan_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Plan {} not found", id)))?;

        if plan.status != "active" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot deactivate plan in '{}' status", plan.status)
            ));
        }

        info!("Deactivating commission plan {}", plan.code);
        self.repository.update_plan_status(id, "inactive").await
    }

    /// Delete (soft-delete) a plan
    pub async fn delete_plan(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        info!("Deleting commission plan '{}' for org {}", code, org_id);
        self.repository.delete_plan(org_id, &code.to_uppercase()).await
    }

    // ========================================================================
    // Commission Rate Tiers
    // ========================================================================

    /// Add a rate tier to a commission plan
    pub async fn add_rate_tier(
        &self,
        org_id: Uuid,
        plan_id: Uuid,
        from_amount: &str,
        to_amount: Option<&str>,
        rate_percent: &str,
        flat_amount: Option<&str>,
    ) -> AtlasResult<CommissionRateTier> {
        let plan = self.repository.get_plan_by_id(plan_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Plan {} not found", plan_id)))?;

        let from: f64 = from_amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "from_amount must be a valid number".to_string(),
        ))?;
        let rate: f64 = rate_percent.parse().map_err(|_| AtlasError::ValidationFailed(
            "rate_percent must be a valid number".to_string(),
        ))?;
        if from < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "from_amount cannot be negative".to_string(),
            ));
        }
        if rate < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "rate_percent cannot be negative".to_string(),
            ));
        }

        let existing_tiers = self.repository.list_rate_tiers(plan_id).await?;
        let tier_number = (existing_tiers.len() + 1) as i32;

        info!("Adding rate tier {} to plan {}", tier_number, plan.code);

        self.repository.create_rate_tier(
            org_id, plan_id, tier_number,
            from_amount, to_amount, rate_percent, flat_amount,
        ).await
    }

    /// List rate tiers for a plan
    pub async fn list_rate_tiers(&self, plan_id: Uuid) -> AtlasResult<Vec<CommissionRateTier>> {
        self.repository.list_rate_tiers(plan_id).await
    }

    // ========================================================================
    // Plan Assignments
    // ========================================================================

    /// Assign a plan to a rep
    pub async fn assign_plan(
        &self,
        org_id: Uuid,
        rep_id: Uuid,
        plan_id: Uuid,
        effective_from: chrono::NaiveDate,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PlanAssignment> {
        // Verify rep exists
        self.repository.get_rep_by_id(rep_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Rep {} not found", rep_id)))?;

        // Verify plan exists
        self.repository.get_plan_by_id(plan_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Plan {} not found", plan_id)))?;

        info!("Assigning plan {} to rep {}", plan_id, rep_id);

        self.repository.create_assignment(
            org_id, rep_id, plan_id, effective_from, effective_to, created_by,
        ).await
    }

    /// List assignments
    pub async fn list_assignments(&self, org_id: Uuid, rep_id: Option<Uuid>) -> AtlasResult<Vec<PlanAssignment>> {
        self.repository.list_assignments(org_id, rep_id).await
    }

    // ========================================================================
    // Quotas
    // ========================================================================

    /// Create a sales quota
    pub async fn create_quota(
        &self,
        org_id: Uuid,
        rep_id: Uuid,
        plan_id: Option<Uuid>,
        quota_number: &str,
        period_name: &str,
        period_start_date: chrono::NaiveDate,
        period_end_date: chrono::NaiveDate,
        quota_type: &str,
        target_amount: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<SalesQuota> {
        // Verify rep exists
        self.repository.get_rep_by_id(rep_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Rep {} not found", rep_id)))?;

        if !VALID_QUOTA_TYPES.contains(&quota_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid quota_type '{}'. Must be one of: {}", quota_type, VALID_QUOTA_TYPES.join(", ")
            )));
        }

        let target: f64 = target_amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "target_amount must be a valid number".to_string(),
        ))?;
        if target <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "target_amount must be positive".to_string(),
            ));
        }

        if period_start_date >= period_end_date {
            return Err(AtlasError::ValidationFailed(
                "period_start_date must be before period_end_date".to_string(),
            ));
        }

        info!("Creating quota '{}' for rep {}", quota_number, rep_id);

        self.repository.create_quota(
            org_id, rep_id, plan_id, quota_number,
            period_name, period_start_date, period_end_date,
            quota_type, target_amount, created_by,
        ).await
    }

    /// Get a quota by ID
    pub async fn get_quota(&self, id: Uuid) -> AtlasResult<Option<SalesQuota>> {
        self.repository.get_quota(id).await
    }

    /// List quotas
    pub async fn list_quotas(&self, org_id: Uuid, rep_id: Option<Uuid>, status: Option<&str>) -> AtlasResult<Vec<SalesQuota>> {
        self.repository.list_quotas(org_id, rep_id, status).await
    }

    // ========================================================================
    // Commission Transactions
    // ========================================================================

    /// Credit a commission transaction to a rep
    pub async fn credit_transaction(
        &self,
        org_id: Uuid,
        rep_id: Uuid,
        plan_id: Option<Uuid>,
        quota_id: Option<Uuid>,
        source_type: &str,
        source_id: Option<Uuid>,
        source_number: Option<&str>,
        transaction_date: chrono::NaiveDate,
        sale_amount: &str,
        currency_code: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CommissionTransaction> {
        // Verify rep exists
        let rep = self.repository.get_rep_by_id(rep_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Rep {} not found", rep_id)))?;

        let sale: f64 = sale_amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "sale_amount must be a valid number".to_string(),
        ))?;
        if sale <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "sale_amount must be positive".to_string(),
            ));
        }

        // Determine the commission plan (explicit or from active assignment)
        let plan = if let Some(pid) = plan_id {
            self.repository.get_plan_by_id(pid).await?
                .ok_or_else(|| AtlasError::EntityNotFound(format!("Plan {} not found", pid)))?
        } else {
            // Find active assignment for this rep
            let assignments = self.repository.list_assignments(org_id, Some(rep_id)).await?;
            let today = chrono::Utc::now().date_naive();
            let active_assignment = assignments.into_iter()
                .filter(|a| a.status == "active")
                .filter(|a| a.effective_from <= today)
                .filter(|a| a.effective_to.is_none_or(|t| t >= today))
                .find(|_| true);

            if let Some(assignment) = active_assignment {
                self.repository.get_plan_by_id(assignment.plan_id).await?
                    .ok_or_else(|| AtlasError::EntityNotFound(
                        format!("Assigned plan {} not found", assignment.plan_id)
                    ))?
            } else {
                return Err(AtlasError::ValidationFailed(
                    "No active plan assignment found for this rep. Specify a plan_id.".to_string(),
                ));
            }
        };

        // Calculate commission based on plan's method and tiers
        let (commission_rate, commission_amount) = match plan.calculation_method.as_str() {
            "percentage" => {
                let rate: f64 = plan.default_rate.parse().unwrap_or(0.0);
                let commission = sale * rate / 100.0;
                (format!("{:.4}", rate), format!("{:.4}", commission))
            }
            "flat_rate" => {
                let flat: f64 = plan.default_rate.parse().unwrap_or(0.0);
                (format!("{:.4}", flat), format!("{:.4}", flat))
            }
            "tiered" | "graduated" => {
                let tiers = self.repository.list_rate_tiers(plan.id).await?;
                if tiers.is_empty() {
                    // Fall back to default rate
                    let rate: f64 = plan.default_rate.parse().unwrap_or(0.0);
                    let commission = sale * rate / 100.0;
                    (format!("{:.4}", rate), format!("{:.4}", commission))
                } else {
                    // Find applicable tier
                    let mut applicable_rate = 0.0_f64;
                    for tier in &tiers {
                        let from: f64 = tier.from_amount.parse().unwrap_or(0.0);
                        let to_opt: Option<f64> = tier.to_amount.as_ref()
                            .and_then(|t| t.parse().ok());

                        let in_range = sale >= from && to_opt.is_none_or(|t| sale <= t);
                        if in_range {
                            applicable_rate = tier.rate_percent.parse().unwrap_or(0.0);
                            break;
                        }
                    }
                    if applicable_rate == 0.0 {
                        // Use last tier's rate if sale exceeds all tiers
                        if let Some(last) = tiers.last() {
                            applicable_rate = last.rate_percent.parse().unwrap_or(0.0);
                        }
                    }
                    let commission = sale * applicable_rate / 100.0;
                    (format!("{:.4}", applicable_rate), format!("{:.4}", commission))
                }
            }
            _ => {
                let rate: f64 = plan.default_rate.parse().unwrap_or(0.0);
                let commission = sale * rate / 100.0;
                (format!("{:.4}", rate), format!("{:.4}", commission))
            }
        };

        // Generate transaction number
        let tx_number = format!("CTX-{}", uuid::Uuid::new_v4().to_string()[..8].to_uppercase());

        info!("Crediting commission tx {} to rep {} (amount={})", tx_number, rep.rep_code, commission_amount);

        let tx = self.repository.create_transaction(
            org_id, rep_id, Some(plan.id), quota_id,
            &tx_number,
            source_type, source_id, source_number,
            transaction_date,
            sale_amount, sale_amount, // basis = sale amount for now
            &commission_rate, &commission_amount,
            currency_code, created_by,
        ).await?;

        // Update quota achievement if linked
        if let Some(qid) = quota_id {
            if let Ok(Some(quota)) = self.repository.get_quota(qid).await {
                let current_achieved: f64 = quota.achieved_amount.parse().unwrap_or(0.0);
                let new_achieved = current_achieved + sale;
                let target: f64 = quota.target_amount.parse().unwrap_or(1.0);
                let pct = if target > 0.0 { (new_achieved / target) * 100.0 } else { 0.0 };

                self.repository.update_quota_achievement(
                    qid, &format!("{:.4}", new_achieved), &format!("{:.4}", pct),
                ).await?;
            }
        }

        Ok(tx)
    }

    /// Get a transaction by ID
    pub async fn get_transaction(&self, id: Uuid) -> AtlasResult<Option<CommissionTransaction>> {
        self.repository.get_transaction(id).await
    }

    /// List transactions
    pub async fn list_transactions(
        &self,
        org_id: Uuid,
        rep_id: Option<Uuid>,
        status: Option<&str>,
    ) -> AtlasResult<Vec<CommissionTransaction>> {
        self.repository.list_transactions(org_id, rep_id, status).await
    }

    // ========================================================================
    // Payout Processing
    // ========================================================================

    /// Process commission payouts for a period
    ///
    /// Collects all "credited" transactions in the period, groups by rep,
    /// creates payout lines, and creates a payout batch.
    pub async fn process_payout(
        &self,
        org_id: Uuid,
        period_name: &str,
        period_start_date: chrono::NaiveDate,
        period_end_date: chrono::NaiveDate,
        currency_code: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CommissionPayout> {
        if period_start_date >= period_end_date {
            return Err(AtlasError::ValidationFailed(
                "period_start_date must be before period_end_date".to_string(),
            ));
        }

        // Get all credited transactions for this period
        let all_txns = self.repository.list_transactions(org_id, None, Some("credited")).await?;
        let eligible_txns: Vec<&CommissionTransaction> = all_txns.iter()
            .filter(|t| t.transaction_date >= period_start_date && t.transaction_date <= period_end_date)
            .collect();

        if eligible_txns.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "No credited transactions found for this period".to_string(),
            ));
        }

        let payout_number = format!("CPY-{}", Uuid::new_v4().to_string()[..8].to_uppercase());

        info!("Processing payout {} for period {} ({} transactions)", payout_number, period_name, eligible_txns.len());

        // Create payout
        let payout = self.repository.create_payout(
            org_id, &payout_number, period_name,
            period_start_date, period_end_date,
            currency_code, created_by,
        ).await?;

        // Group transactions by rep
        use std::collections::HashMap;
        let mut rep_totals: HashMap<Uuid, (String, f64, i32, Option<Uuid>, Option<String>)> = HashMap::new();

        for tx in &eligible_txns {
            let rep = self.repository.get_rep_by_id(tx.rep_id).await?;
            let rep_name = rep.map(|r| format!("{} {}", r.first_name, r.last_name)).unwrap_or_default();

            let entry = rep_totals.entry(tx.rep_id).or_insert_with(|| {
                (rep_name, 0.0, 0, tx.plan_id, None)
            });
            let commission: f64 = tx.commission_amount.parse().unwrap_or(0.0);
            entry.1 += commission;
            entry.2 += 1;

            // Move transaction to "included" status
            self.repository.update_transaction_status(tx.id, "included", Some(payout.id)).await?;
        }

        // Create payout lines
        let mut total_payout: f64 = 0.0;
        for (rep_id, (rep_name, gross, tx_count, plan_id, _)) in &rep_totals {
            let net = *gross; // No adjustments for now
            total_payout += net;

            let plan_code = if let Some(pid) = plan_id {
                self.repository.get_plan_by_id(*pid).await?.map(|p| p.code)
            } else {
                None
            };

            self.repository.create_payout_line(
                org_id, payout.id, *rep_id, rep_name,
                *plan_id, plan_code.as_deref(),
                &format!("{:.4}", gross),
                "0.0000",
                &format!("{:.4}", net),
                currency_code,
                *tx_count,
            ).await?;
        }

        // Update payout totals
        self.repository.update_payout_totals(
            payout.id,
            &format!("{:.4}", total_payout),
            rep_totals.len() as i32,
            eligible_txns.len() as i32,
        ).await?;

        // Refresh payout
        self.repository.get_payout(payout.id).await?
            .ok_or_else(|| AtlasError::Internal("Failed to retrieve created payout".to_string()))
    }

    /// Get a payout by ID
    pub async fn get_payout(&self, id: Uuid) -> AtlasResult<Option<CommissionPayout>> {
        self.repository.get_payout(id).await
    }

    /// List payouts
    pub async fn list_payouts(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<CommissionPayout>> {
        self.repository.list_payouts(org_id, status).await
    }

    /// Get payout lines
    pub async fn list_payout_lines(&self, payout_id: Uuid) -> AtlasResult<Vec<CommissionPayoutLine>> {
        self.repository.list_payout_lines(payout_id).await
    }

    /// Approve a payout
    pub async fn approve_payout(&self, id: Uuid, approved_by: Option<Uuid>) -> AtlasResult<CommissionPayout> {
        let payout = self.repository.get_payout(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Payout {} not found", id)))?;

        if payout.status != "draft" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot approve payout in '{}' status. Must be 'draft'.", payout.status)
            ));
        }

        info!("Approving payout {}", payout.payout_number);
        self.repository.update_payout_status(id, "approved", approved_by, None).await
    }

    /// Reject a payout
    pub async fn reject_payout(&self, id: Uuid, rejected_reason: Option<&str>) -> AtlasResult<CommissionPayout> {
        let payout = self.repository.get_payout(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Payout {} not found", id)))?;

        if payout.status != "draft" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot reject payout in '{}' status. Must be 'draft'.", payout.status)
            ));
        }

        info!("Rejecting payout {}", payout.payout_number);
        self.repository.update_payout_status(id, "rejected", None, rejected_reason).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get a commission dashboard summary
    pub async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<CommissionDashboardSummary> {
        let reps = self.repository.list_reps(org_id, false).await?;
        let plans = self.repository.list_plans(org_id, None).await?;
        let quotas = self.repository.list_quotas(org_id, None, None).await?;
        let txns = self.repository.list_transactions(org_id, None, None).await?;
        let payouts = self.repository.list_payouts(org_id, None).await?;

        let active_reps = reps.iter().filter(|r| r.is_active).count() as i32;
        let active_plans = plans.iter().filter(|p| p.status == "active").count() as i32;

        let pending_payouts = payouts.iter().filter(|p| p.status == "draft").count() as i32;

        // Calculate total commission this month
        let now = chrono::Utc::now();
        let today = now.date_naive();
        let this_month_start = chrono::NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap_or_default();
        let total_commission_this_month: f64 = txns.iter()
            .filter(|t| t.transaction_date >= this_month_start)
            .map(|t| t.commission_amount.parse::<f64>().unwrap_or(0.0))
            .sum();

        // Average quota achievement
        let total_quota_achievement: f64 = if quotas.is_empty() {
            0.0
        } else {
            quotas.iter()
                .map(|q| q.achievement_percent.parse::<f64>().unwrap_or(0.0))
                .sum::<f64>() / quotas.len() as f64
        };

        // Payouts by status
        let mut pb_status = serde_json::Map::new();
        for p in &payouts {
            let count = pb_status.entry(p.status.clone())
                .or_insert(serde_json::Value::Number(0.into()));
            *count = serde_json::Value::Number((count.as_u64().unwrap_or(0) + 1).into());
        }

        // Top performers by commission this month
        use std::collections::HashMap;
        let mut rep_commissions: HashMap<Uuid, (String, f64, f64)> = HashMap::new();
        for tx in &txns {
            if tx.transaction_date >= this_month_start {
                let entry = rep_commissions.entry(tx.rep_id).or_insert((String::new(), 0.0, 0.0));
                let commission: f64 = tx.commission_amount.parse().unwrap_or(0.0);
                let sale: f64 = tx.sale_amount.parse().unwrap_or(0.0);
                entry.1 += commission;
                entry.2 += sale;
            }
        }

        // Enrich rep names
        for (rep_id, (name, _, _)) in rep_commissions.iter_mut() {
            if let Some(rep) = self.repository.get_rep_by_id(*rep_id).await.ok().flatten() {
                *name = format!("{} {}", rep.first_name, rep.last_name);
            }
        }

        let mut top: Vec<CommissionTopPerformer> = rep_commissions.into_iter()
            .map(|(rep_id, (name, total_comm, _))| CommissionTopPerformer {
                rep_id,
                rep_name: name,
                total_commission: format!("{:.4}", total_comm),
                quota_achievement: "0".to_string(),
                rank: 0,
            })
            .collect();
        top.sort_by(|a, b| {
            let a_val: f64 = a.total_commission.parse().unwrap_or(0.0);
            let b_val: f64 = b.total_commission.parse().unwrap_or(0.0);
            b_val.partial_cmp(&a_val).unwrap_or(std::cmp::Ordering::Equal)
        });
        for (i, performer) in top.iter_mut().enumerate() {
            performer.rank = (i + 1) as i32;
        }
        top.truncate(5);

        Ok(CommissionDashboardSummary {
            total_reps: reps.len() as i32,
            active_reps,
            total_plans: plans.len() as i32,
            active_plans,
            total_quotas: quotas.len() as i32,
            total_transactions: txns.len() as i32,
            total_pending_payouts: pending_payouts,
            total_commission_this_month: format!("{:.4}", total_commission_this_month),
            total_quota_achievement_percent: format!("{:.2}", total_quota_achievement),
            payouts_by_status: serde_json::Value::Object(pb_status),
            top_performers: top,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_plan_types() {
        assert!(VALID_PLAN_TYPES.contains(&"revenue"));
        assert!(VALID_PLAN_TYPES.contains(&"margin"));
        assert!(VALID_PLAN_TYPES.contains(&"quantity"));
        assert!(VALID_PLAN_TYPES.contains(&"flat_bonus"));
    }

    #[test]
    fn test_valid_bases() {
        assert!(VALID_BASES.contains(&"revenue"));
        assert!(VALID_BASES.contains(&"gross_margin"));
        assert!(VALID_BASES.contains(&"net_margin"));
        assert!(VALID_BASES.contains(&"quantity"));
    }

    #[test]
    fn test_valid_calc_methods() {
        assert!(VALID_CALC_METHODS.contains(&"percentage"));
        assert!(VALID_CALC_METHODS.contains(&"tiered"));
        assert!(VALID_CALC_METHODS.contains(&"flat_rate"));
        assert!(VALID_CALC_METHODS.contains(&"graduated"));
    }

    #[test]
    fn test_valid_quota_types() {
        assert!(VALID_QUOTA_TYPES.contains(&"revenue"));
        assert!(VALID_QUOTA_TYPES.contains(&"units"));
        assert!(VALID_QUOTA_TYPES.contains(&"margin"));
        assert!(VALID_QUOTA_TYPES.contains(&"activities"));
    }
}
