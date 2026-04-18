//! Project Costing Engine Implementation
//!
//! Manages cost transactions against projects/tasks, burden schedule management,
//! overhead allocation (burdening), cost adjustments, and GL cost distributions.
//!
//! Oracle Fusion Cloud ERP equivalent: Project Management > Project Costing

use atlas_shared::{
    ProjectCostTransaction, BurdenSchedule, BurdenScheduleLine,
    ProjectCostAdjustment, ProjectCostDistribution, ProjectCostingSummary,
    AtlasError, AtlasResult,
};
use super::ProjectCostingRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Valid cost types
const VALID_COST_TYPES: &[&str] = &["labor", "material", "expense", "equipment", "other"];

/// Valid transaction statuses
const VALID_TRANSACTION_STATUSES: &[&str] = &[
    "draft", "approved", "distributed", "adjusted", "reversed", "capitalized",
];

/// Valid adjustment types
const VALID_ADJUSTMENT_TYPES: &[&str] = &[
    "increase", "decrease", "transfer", "reversal",
];

/// Valid adjustment statuses
const VALID_ADJUSTMENT_STATUSES: &[&str] = &[
    "pending", "approved", "rejected", "processed",
];

/// Valid schedule statuses
#[allow(dead_code)]
const VALID_SCHEDULE_STATUSES: &[&str] = &["draft", "active", "inactive"];

/// Project Costing engine for managing project costs with burdening
pub struct ProjectCostingEngine {
    repository: Arc<dyn ProjectCostingRepository>,
}

impl ProjectCostingEngine {
    pub fn new(repository: Arc<dyn ProjectCostingRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Cost Transactions
    // ========================================================================

    /// Create a new cost transaction against a project/task.
    /// Automatically calculates burdened cost using the applicable burden schedule.
    pub async fn create_cost_transaction(
        &self,
        org_id: Uuid,
        project_id: Uuid,
        project_number: Option<&str>,
        task_id: Option<Uuid>,
        task_number: Option<&str>,
        cost_type: &str,
        raw_cost_amount: &str,
        currency_code: &str,
        transaction_date: chrono::NaiveDate,
        gl_date: Option<chrono::NaiveDate>,
        description: Option<&str>,
        supplier_id: Option<Uuid>,
        supplier_name: Option<&str>,
        employee_id: Option<Uuid>,
        employee_name: Option<&str>,
        expenditure_category: Option<&str>,
        quantity: Option<&str>,
        unit_of_measure: Option<&str>,
        unit_rate: Option<&str>,
        is_billable: bool,
        is_capitalizable: bool,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ProjectCostTransaction> {
        if !VALID_COST_TYPES.contains(&cost_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid cost type '{}'. Must be one of: {}",
                cost_type, VALID_COST_TYPES.join(", ")
            )));
        }

        let raw_cost: f64 = raw_cost_amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "Raw cost amount must be a valid number".to_string(),
        ))?;
        if raw_cost < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Raw cost amount must be non-negative".to_string(),
            ));
        }

        // Look up applicable burden rate
        let (burden_rate, _burden_schedule_id) = self.get_burden_rate(org_id, cost_type, expenditure_category).await?;
        let burden_amount = raw_cost * burden_rate / 100.0;
        let burdened_cost = raw_cost + burden_amount;

        let transaction_number = format!("PJC-{}", Uuid::new_v4().to_string()[..8].to_uppercase());

        info!("Creating cost transaction {} for project {} ({}): raw={:.2}, burden={:.2}, burdened={:.2}",
              transaction_number, project_id, cost_type, raw_cost, burden_amount, burdened_cost);

        self.repository.create_cost_transaction(
            org_id, &transaction_number,
            project_id, project_number,
            task_id, task_number,
            cost_type,
            &format!("{:.2}", raw_cost),
            &format!("{:.2}", burdened_cost),
            &format!("{:.2}", burden_amount),
            currency_code,
            transaction_date, gl_date, description,
            supplier_id, supplier_name,
            employee_id, employee_name,
            expenditure_category,
            quantity, unit_of_measure, unit_rate,
            is_billable, is_capitalizable,
            None, None, None,
            created_by,
        ).await
    }

    /// Get a cost transaction by ID
    pub async fn get_cost_transaction(&self, id: Uuid) -> AtlasResult<Option<ProjectCostTransaction>> {
        self.repository.get_cost_transaction(id).await
    }

    /// List cost transactions with optional filters
    pub async fn list_cost_transactions(
        &self,
        org_id: Uuid,
        project_id: Option<Uuid>,
        cost_type: Option<&str>,
        status: Option<&str>,
    ) -> AtlasResult<Vec<ProjectCostTransaction>> {
        if let Some(ct) = cost_type {
            if !VALID_COST_TYPES.contains(&ct) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid cost type '{}'. Must be one of: {}", ct, VALID_COST_TYPES.join(", ")
                )));
            }
        }
        if let Some(s) = status {
            if !VALID_TRANSACTION_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_TRANSACTION_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_cost_transactions(org_id, project_id, cost_type, status).await
    }

    /// Approve a cost transaction
    pub async fn approve_cost_transaction(
        &self,
        id: Uuid,
        approved_by: Option<Uuid>,
    ) -> AtlasResult<ProjectCostTransaction> {
        let txn = self.repository.get_cost_transaction(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Cost transaction {} not found", id)
            ))?;

        if txn.status != "draft" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot approve transaction in '{}' status. Must be 'draft'.", txn.status)
            ));
        }

        info!("Approving cost transaction {}", txn.transaction_number);
        self.repository.update_cost_transaction_status(id, "approved", approved_by).await
    }

    /// Reverse a cost transaction
    pub async fn reverse_cost_transaction(
        &self,
        org_id: Uuid,
        id: Uuid,
        reason: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ProjectCostTransaction> {
        let txn = self.repository.get_cost_transaction(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Cost transaction {} not found", id)
            ))?;

        if txn.status == "reversed" {
            return Err(AtlasError::WorkflowError(
                "Transaction is already reversed".to_string()
            ));
        }

        let raw: f64 = txn.raw_cost_amount.parse().unwrap_or(0.0);
        let burdened: f64 = txn.burdened_cost_amount.parse().unwrap_or(0.0);
        let burden: f64 = txn.burden_amount.parse().unwrap_or(0.0);

        // Create reversal transaction
        let reversal_number = format!("PJC-REV-{}", Uuid::new_v4().to_string()[..8].to_uppercase());
        let reversal = self.repository.create_cost_transaction(
            org_id, &reversal_number,
            txn.project_id, txn.project_number.as_deref(),
            txn.task_id, txn.task_number.as_deref(),
            &txn.cost_type,
            &format!("{:.2}", -raw),
            &format!("{:.2}", -burdened),
            &format!("{:.2}", -burden),
            &txn.currency_code,
            chrono::Utc::now().date_naive(),
            None,
            Some(&format!("Reversal of {}", txn.transaction_number)),
            txn.supplier_id, txn.supplier_name.as_deref(),
            txn.employee_id, txn.employee_name.as_deref(),
            txn.expenditure_category.as_deref(),
            txn.quantity.as_deref(), txn.unit_of_measure.as_deref(), txn.unit_rate.as_deref(),
            txn.is_billable, txn.is_capitalizable,
            Some(id),
            Some("reversal"),
            reason,
            created_by,
        ).await?;

        // Mark original as reversed
        self.repository.update_cost_transaction_status(id, "reversed", None).await?;

        // Approve the reversal automatically
        self.repository.update_cost_transaction_status(reversal.id, "approved", created_by).await
    }

    // ========================================================================
    // Burden Schedules
    // ========================================================================

    /// Create a new burden schedule
    pub async fn create_burden_schedule(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        effective_from: chrono::NaiveDate,
        effective_to: Option<chrono::NaiveDate>,
        is_default: bool,
        created_by: Option<Uuid>,
    ) -> AtlasResult<BurdenSchedule> {
        if code.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Burden schedule code is required".to_string(),
            ));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Burden schedule name is required".to_string(),
            ));
        }

        info!("Creating burden schedule {} ({}) for org {}", code, name, org_id);

        self.repository.create_burden_schedule(
            org_id, code, name, description, "draft",
            effective_from, effective_to, is_default, created_by,
        ).await
    }

    /// Get a burden schedule by code
    pub async fn get_burden_schedule(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<BurdenSchedule>> {
        self.repository.get_burden_schedule(org_id, code).await
    }

    /// List all burden schedules
    pub async fn list_burden_schedules(&self, org_id: Uuid) -> AtlasResult<Vec<BurdenSchedule>> {
        self.repository.list_burden_schedules(org_id).await
    }

    /// Activate a burden schedule
    pub async fn activate_burden_schedule(&self, id: Uuid) -> AtlasResult<BurdenSchedule> {
        let schedule = self.repository.get_burden_schedule_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Burden schedule {} not found", id)
            ))?;

        if schedule.status != "draft" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot activate schedule in '{}' status. Must be 'draft'.", schedule.status)
            ));
        }

        info!("Activating burden schedule {} ({})", schedule.code, schedule.name);
        self.repository.update_burden_schedule_status(id, "active").await
    }

    /// Add a burden schedule line
    pub async fn add_burden_schedule_line(
        &self,
        org_id: Uuid,
        schedule_id: Uuid,
        cost_type: &str,
        expenditure_category: Option<&str>,
        burden_rate_percent: &str,
        burden_account_code: Option<&str>,
    ) -> AtlasResult<BurdenScheduleLine> {
        if !VALID_COST_TYPES.contains(&cost_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid cost type '{}'. Must be one of: {}",
                cost_type, VALID_COST_TYPES.join(", ")
            )));
        }

        let rate: f64 = burden_rate_percent.parse().map_err(|_| AtlasError::ValidationFailed(
            "Burden rate percent must be a valid number".to_string(),
        ))?;
        if rate < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Burden rate percent must be non-negative".to_string(),
            ));
        }

        let schedule = self.repository.get_burden_schedule_by_id(schedule_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Burden schedule {} not found", schedule_id)
            ))?;

        let lines = self.repository.list_burden_schedule_lines(schedule_id).await?;
        let next_line_number = lines.len() as i32 + 1;

        info!("Adding burden line to schedule {}: {} @ {}%", schedule.code, cost_type, rate);

        self.repository.create_burden_schedule_line(
            org_id, schedule_id, next_line_number,
            cost_type, expenditure_category,
            burden_rate_percent, burden_account_code,
        ).await
    }

    /// List lines for a burden schedule
    pub async fn list_burden_schedule_lines(&self, schedule_id: Uuid) -> AtlasResult<Vec<BurdenScheduleLine>> {
        self.repository.list_burden_schedule_lines(schedule_id).await
    }

    // ========================================================================
    // Cost Adjustments
    // ========================================================================

    /// Create a cost adjustment
    pub async fn create_cost_adjustment(
        &self,
        org_id: Uuid,
        original_transaction_id: Uuid,
        adjustment_type: &str,
        adjustment_amount: &str,
        reason: &str,
        description: Option<&str>,
        effective_date: chrono::NaiveDate,
        transfer_to_project_id: Option<Uuid>,
        transfer_to_task_id: Option<Uuid>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ProjectCostAdjustment> {
        if !VALID_ADJUSTMENT_TYPES.contains(&adjustment_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid adjustment type '{}'. Must be one of: {}",
                adjustment_type, VALID_ADJUSTMENT_TYPES.join(", ")
            )));
        }

        if reason.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Adjustment reason is required".to_string(),
            ));
        }

        let original = self.repository.get_cost_transaction(original_transaction_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Original cost transaction {} not found", original_transaction_id)
            ))?;

        if original.status != "approved" && original.status != "distributed" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot adjust transaction in '{}' status. Must be 'approved' or 'distributed'.", original.status)
            ));
        }

        let adj_amount: f64 = adjustment_amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "Adjustment amount must be a valid number".to_string(),
        ))?;
        if adj_amount <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Adjustment amount must be positive".to_string(),
            ));
        }

        let original_raw: f64 = original.raw_cost_amount.parse().unwrap_or(0.0);
        let original_burdened: f64 = original.burdened_cost_amount.parse().unwrap_or(0.0);
        let original_burden: f64 = original.burden_amount.parse().unwrap_or(0.0);
        let burden_rate = if original_raw != 0.0 { original_burden / original_raw * 100.0 } else { 0.0 };

        let (new_raw, new_burdened) = match adjustment_type {
            "increase" => {
                let adj_burden = adj_amount * burden_rate / 100.0;
                (original_raw + adj_amount, original_burdened + adj_amount + adj_burden)
            }
            "decrease" => {
                let adj_burden = adj_amount * burden_rate / 100.0;
                let new = original_raw - adj_amount;
                if new < 0.0 {
                    return Err(AtlasError::ValidationFailed(
                        format!("Decrease of {:.2} would result in negative cost (current: {:.2})", adj_amount, original_raw)
                    ));
                }
                (new, original_burdened - adj_amount - adj_burden)
            }
            "reversal" => (0.0, 0.0),
            "transfer" => (original_raw, original_burdened), // amount stays same, just moves
            _ => return Err(AtlasError::ValidationFailed("Invalid adjustment type".to_string())),
        };

        let adjustment_number = format!("ADJ-{}", Uuid::new_v4().to_string()[..8].to_uppercase());

        info!("Creating cost adjustment {} ({}) for transaction {}: amount={:.2}",
              adjustment_number, adjustment_type, original.transaction_number, adj_amount);

        self.repository.create_cost_adjustment(
            org_id, &adjustment_number,
            original_transaction_id,
            adjustment_type,
            adjustment_amount,
            &format!("{:.2}", new_raw),
            &format!("{:.2}", new_burdened),
            reason, description,
            effective_date,
            transfer_to_project_id, transfer_to_task_id,
            created_by,
        ).await
    }

    /// Approve a cost adjustment and apply it
    pub async fn approve_cost_adjustment(
        &self,
        id: Uuid,
        approved_by: Option<Uuid>,
    ) -> AtlasResult<ProjectCostAdjustment> {
        let adjustment = self.repository.get_cost_adjustment(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Cost adjustment {} not found", id)
            ))?;

        if adjustment.status != "pending" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot approve adjustment in '{}' status.", adjustment.status)
            ));
        }

        // Mark original transaction as adjusted
        self.repository.update_cost_transaction_status(
            adjustment.original_transaction_id, "adjusted", None,
        ).await?;

        // Update adjustment status
        info!("Approving cost adjustment {}", adjustment.adjustment_number);
        self.repository.update_cost_adjustment_status(
            id, "approved", approved_by, None,
        ).await
    }

    /// List cost adjustments
    pub async fn list_cost_adjustments(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<ProjectCostAdjustment>> {
        if let Some(s) = status {
            if !VALID_ADJUSTMENT_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_ADJUSTMENT_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_cost_adjustments(org_id, status).await
    }

    // ========================================================================
    // Cost Distributions
    // ========================================================================

    /// Generate GL distributions for an approved cost transaction
    pub async fn distribute_cost_transaction(
        &self,
        org_id: Uuid,
        transaction_id: Uuid,
        raw_cost_account: &str,
        burden_account: &str,
        ap_ar_account: &str,
    ) -> AtlasResult<Vec<ProjectCostDistribution>> {
        let txn = self.repository.get_cost_transaction(transaction_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Cost transaction {} not found", transaction_id)
            ))?;

        if txn.status != "approved" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot distribute transaction in '{}' status. Must be 'approved'.", txn.status)
            ));
        }

        let raw: f64 = txn.raw_cost_amount.parse().unwrap_or(0.0);
        let burden: f64 = txn.burden_amount.parse().unwrap_or(0.0);
        let gl_date = txn.gl_date.unwrap_or(txn.transaction_date);

        let mut distributions = Vec::new();
        let mut line_num = 0;

        // Raw cost distribution: debit project cost, credit AP/AR
        if raw.abs() > 0.001 {
            line_num += 1;
            let dist = self.repository.create_cost_distribution(
                org_id, transaction_id, line_num,
                raw_cost_account, ap_ar_account,
                &format!("{:.2}", raw.abs()),
                "raw_cost", gl_date,
            ).await?;
            distributions.push(dist);
        }

        // Burden distribution: debit burden account, credit AP/AR
        if burden.abs() > 0.001 {
            line_num += 1;
            let dist = self.repository.create_cost_distribution(
                org_id, transaction_id, line_num,
                burden_account, ap_ar_account,
                &format!("{:.2}", burden.abs()),
                "burden", gl_date,
            ).await?;
            distributions.push(dist);
        }

        // Update transaction status to distributed
        self.repository.update_cost_transaction_status(transaction_id, "distributed", None).await?;

        info!("Generated {} distribution lines for transaction {}", distributions.len(), txn.transaction_number);
        Ok(distributions)
    }

    /// List distributions for a cost transaction
    pub async fn list_cost_distributions(&self, transaction_id: Uuid) -> AtlasResult<Vec<ProjectCostDistribution>> {
        self.repository.list_cost_distributions(transaction_id).await
    }

    /// Get all unposted distributions
    pub async fn get_unposted_distributions(&self, org_id: Uuid) -> AtlasResult<Vec<ProjectCostDistribution>> {
        self.repository.list_unposted_distributions(org_id).await
    }

    /// Mark distributions as posted (batch)
    pub async fn post_distributions(&self, org_id: Uuid, gl_batch_id: Option<Uuid>) -> AtlasResult<i32> {
        let unposted = self.repository.list_unposted_distributions(org_id).await?;
        let mut count = 0;
        for dist in &unposted {
            self.repository.mark_distribution_posted(dist.id, gl_batch_id).await?;
            count += 1;
        }
        info!("Posted {} project cost distributions for org {}", count, org_id);
        Ok(count)
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get project costing dashboard summary
    pub async fn get_costing_summary(&self, org_id: Uuid) -> AtlasResult<ProjectCostingSummary> {
        self.repository.get_costing_summary(org_id).await
    }

    // ========================================================================
    // Burden Calculation
    // ========================================================================

    /// Get the applicable burden rate for a cost type and optional expenditure category
    async fn get_burden_rate(&self, org_id: Uuid, cost_type: &str, expenditure_category: Option<&str>) -> AtlasResult<(f64, Option<Uuid>)> {
        // Try default schedule first
        if let Some(schedule) = self.repository.get_default_burden_schedule(org_id).await? {
            if let Some(line) = self.repository.get_applicable_burden_rate(schedule.id, cost_type, expenditure_category).await? {
                let rate: f64 = line.burden_rate_percent.parse().unwrap_or(0.0);
                return Ok((rate, Some(schedule.id)));
            }
        }

        // No burden schedule found - zero burden
        Ok((0.0, None))
    }

    /// Calculate burden for a given raw cost amount
    pub fn calculate_burden(&self, raw_cost: f64, burden_rate_percent: f64) -> (f64, f64) {
        let burden = raw_cost * burden_rate_percent / 100.0;
        let burdened = raw_cost + burden;
        (burden, burdened)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_cost_types() {
        assert!(VALID_COST_TYPES.contains(&"labor"));
        assert!(VALID_COST_TYPES.contains(&"material"));
        assert!(VALID_COST_TYPES.contains(&"expense"));
        assert!(VALID_COST_TYPES.contains(&"equipment"));
        assert!(VALID_COST_TYPES.contains(&"other"));
    }

    #[test]
    fn test_valid_adjustment_types() {
        assert!(VALID_ADJUSTMENT_TYPES.contains(&"increase"));
        assert!(VALID_ADJUSTMENT_TYPES.contains(&"decrease"));
        assert!(VALID_ADJUSTMENT_TYPES.contains(&"transfer"));
        assert!(VALID_ADJUSTMENT_TYPES.contains(&"reversal"));
    }

    #[test]
    fn test_calculate_burden() {
        let engine = ProjectCostingEngine::new(Arc::new(crate::MockProjectCostingRepository));

        // 25% burden on $1000 raw cost
        let (burden, burdened) = engine.calculate_burden(1000.0, 25.0);
        assert!((burden - 250.0).abs() < 0.01, "Expected burden 250.0, got {}", burden);
        assert!((burdened - 1250.0).abs() < 0.01, "Expected burdened 1250.0, got {}", burdened);
    }

    #[test]
    fn test_calculate_burden_zero_rate() {
        let engine = ProjectCostingEngine::new(Arc::new(crate::MockProjectCostingRepository));
        let (burden, burdened) = engine.calculate_burden(1000.0, 0.0);
        assert_eq!(burden, 0.0);
        assert_eq!(burdened, 1000.0);
    }

    #[test]
    fn test_calculate_burden_zero_cost() {
        let engine = ProjectCostingEngine::new(Arc::new(crate::MockProjectCostingRepository));
        let (burden, burdened) = engine.calculate_burden(0.0, 25.0);
        assert_eq!(burden, 0.0);
        assert_eq!(burdened, 0.0);
    }
}
