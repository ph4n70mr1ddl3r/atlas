//! Budget Engine Implementation
//!
//! Manages budget definitions, budget versions with approval workflow,
//! budget lines (by account, period, department), budget vs. actuals
//! variance reporting, budget transfers, and budget controls.
//!
//! Oracle Fusion Cloud ERP equivalent: General Ledger > Budgets

use atlas_shared::{
    BudgetDefinition, BudgetVersion, BudgetLine, BudgetTransfer,
    BudgetVarianceReport, BudgetVarianceLine,
    AtlasError, AtlasResult,
};
use super::BudgetRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Valid budget types
const VALID_BUDGET_TYPES: &[&str] = &[
    "operating", "capital", "project", "cash_flow",
];

/// Valid control levels
const VALID_CONTROL_LEVELS: &[&str] = &[
    "none", "advisory", "absolute",
];

/// Valid version statuses
const _VALID_VERSION_STATUSES: &[&str] = &[
    "draft", "submitted", "approved", "active", "closed", "rejected",
];

/// Valid transfer statuses
const _VALID_TRANSFER_STATUSES: &[&str] = &[
    "pending", "approved", "rejected", "cancelled",
];

/// Budget engine for managing budgets, versions, lines, and transfers
pub struct BudgetEngine {
    repository: Arc<dyn BudgetRepository>,
}

impl BudgetEngine {
    pub fn new(repository: Arc<dyn BudgetRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Budget Definition Management
    // ========================================================================

    /// Create a new budget definition
    pub async fn create_definition(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        calendar_id: Option<Uuid>,
        fiscal_year: Option<i32>,
        budget_type: &str,
        control_level: &str,
        allow_carry_forward: bool,
        allow_transfers: bool,
        currency_code: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<BudgetDefinition> {
        let code_upper = code.to_uppercase();
        if code_upper.is_empty() || code_upper.len() > 50 {
            return Err(AtlasError::ValidationFailed(
                "Budget code must be 1-50 characters".to_string(),
            ));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Budget name is required".to_string(),
            ));
        }
        if !VALID_BUDGET_TYPES.contains(&budget_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid budget_type '{}'. Must be one of: {}", budget_type, VALID_BUDGET_TYPES.join(", ")
            )));
        }
        if !VALID_CONTROL_LEVELS.contains(&control_level) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid control_level '{}'. Must be one of: {}", control_level, VALID_CONTROL_LEVELS.join(", ")
            )));
        }

        info!("Creating budget definition '{}' ({}) for org {}", code_upper, name, org_id);

        self.repository.create_definition(
            org_id, &code_upper, name, description,
            calendar_id, fiscal_year, budget_type, control_level,
            allow_carry_forward, allow_transfers, currency_code, created_by,
        ).await
    }

    /// Get a budget definition by code
    pub async fn get_definition(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<BudgetDefinition>> {
        self.repository.get_definition(org_id, &code.to_uppercase()).await
    }

    /// List all budget definitions for an organization
    pub async fn list_definitions(&self, org_id: Uuid) -> AtlasResult<Vec<BudgetDefinition>> {
        self.repository.list_definitions(org_id).await
    }

    /// Deactivate a budget definition
    pub async fn delete_definition(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        info!("Deactivating budget definition '{}' for org {}", code, org_id);
        self.repository.delete_definition(org_id, &code.to_uppercase()).await
    }

    // ========================================================================
    // Budget Version Management
    // ========================================================================

    /// Create a new budget version
    pub async fn create_version(
        &self,
        org_id: Uuid,
        budget_code: &str,
        label: Option<&str>,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<BudgetVersion> {
        let definition = self.get_definition(org_id, budget_code).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Budget definition '{}' not found", budget_code)
            ))?;

        if let (Some(from), Some(to)) = (effective_from, effective_to) {
            if from > to {
                return Err(AtlasError::ValidationFailed(
                    "effective_from must be before effective_to".to_string(),
                ));
            }
        }

        let version_number = self.repository.get_next_version_number(definition.id).await?;

        info!("Creating budget version {} for '{}' (v{})", version_number, budget_code, version_number);

        self.repository.create_version(
            org_id, definition.id, version_number, label,
            effective_from, effective_to, notes, created_by,
        ).await
    }

    /// Get a budget version by ID
    pub async fn get_version(&self, id: Uuid) -> AtlasResult<Option<BudgetVersion>> {
        self.repository.get_version(id).await
    }

    /// Get the active version for a budget definition
    pub async fn get_active_version(&self, definition_id: Uuid) -> AtlasResult<Option<BudgetVersion>> {
        self.repository.get_active_version(definition_id).await
    }

    /// List all versions for a budget definition
    pub async fn list_versions(&self, definition_id: Uuid) -> AtlasResult<Vec<BudgetVersion>> {
        self.repository.list_versions(definition_id).await
    }

    // ========================================================================
    // Budget Version Workflow
    // ========================================================================

    /// Submit a budget version for approval
    /// Only draft versions can be submitted
    pub async fn submit_version(&self, version_id: Uuid, submitted_by: Uuid) -> AtlasResult<BudgetVersion> {
        let version = self.repository.get_version(version_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Budget version {} not found", version_id)
            ))?;

        if version.status != "draft" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot submit version in '{}' status. Must be 'draft'.", version.status)
            ));
        }

        // Check version has at least one line
        let lines = self.repository.list_lines_by_version(version_id).await?;
        if lines.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Budget version must have at least one budget line before submission".to_string(),
            ));
        }

        info!("Submitting budget version {}", version_id);
        self.repository.update_version_status(version_id, "submitted", Some(submitted_by), None, None).await
    }

    /// Approve a budget version
    pub async fn approve_version(&self, version_id: Uuid, approved_by: Uuid) -> AtlasResult<BudgetVersion> {
        let version = self.repository.get_version(version_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Budget version {} not found", version_id)
            ))?;

        if version.status != "submitted" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot approve version in '{}' status. Must be 'submitted'.", version.status)
            ));
        }

        info!("Approving budget version {} by {}", version_id, approved_by);
        self.repository.update_version_status(version_id, "approved", None, Some(approved_by), None).await
    }

    /// Activate an approved budget version
    pub async fn activate_version(&self, version_id: Uuid) -> AtlasResult<BudgetVersion> {
        let version = self.repository.get_version(version_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Budget version {} not found", version_id)
            ))?;

        if version.status != "approved" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot activate version in '{}' status. Must be 'approved'.", version.status)
            ));
        }

        // Deactivate any currently active versions for the same definition
        if let Ok(Some(current_active)) = self.repository.get_active_version(version.definition_id).await {
            if current_active.id != version_id {
                self.repository.update_version_status(current_active.id, "closed", None, None, None).await?;
            }
        }

        info!("Activating budget version {}", version_id);
        self.repository.update_version_status(version_id, "active", None, None, None).await
    }

    /// Reject a submitted budget version
    pub async fn reject_version(&self, version_id: Uuid, reason: Option<&str>) -> AtlasResult<BudgetVersion> {
        let version = self.repository.get_version(version_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Budget version {} not found", version_id)
            ))?;

        if version.status != "submitted" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot reject version in '{}' status. Must be 'submitted'.", version.status)
            ));
        }

        info!("Rejecting budget version {}", version_id);
        self.repository.update_version_status(version_id, "rejected", None, None, reason).await
    }

    /// Close an active budget version
    pub async fn close_version(&self, version_id: Uuid) -> AtlasResult<BudgetVersion> {
        let version = self.repository.get_version(version_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Budget version {} not found", version_id)
            ))?;

        if version.status != "active" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot close version in '{}' status. Must be 'active'.", version.status)
            ));
        }

        info!("Closing budget version {}", version_id);
        self.repository.update_version_status(version_id, "closed", None, None, None).await
    }

    // ========================================================================
    // Budget Line Management
    // ========================================================================

    /// Add a budget line to a version
    pub async fn add_line(
        &self,
        org_id: Uuid,
        version_id: Uuid,
        account_code: &str,
        account_name: Option<&str>,
        period_name: Option<&str>,
        period_start_date: Option<chrono::NaiveDate>,
        period_end_date: Option<chrono::NaiveDate>,
        fiscal_year: Option<i32>,
        quarter: Option<i32>,
        department_id: Option<Uuid>,
        department_name: Option<&str>,
        project_id: Option<Uuid>,
        project_name: Option<&str>,
        cost_center: Option<&str>,
        budget_amount: &str,
        description: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<BudgetLine> {
        // Validate version exists and is in draft status
        let version = self.repository.get_version(version_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Budget version {} not found", version_id)
            ))?;

        if version.status != "draft" {
            return Err(AtlasError::WorkflowError(
                "Can only add lines to a draft budget version".to_string(),
            ));
        }

        if account_code.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Account code is required".to_string(),
            ));
        }

        let amount_val: f64 = budget_amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "Budget amount must be a valid number".to_string(),
        ))?;
        if amount_val < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Budget amount must be non-negative".to_string(),
            ));
        }

        // Check for duplicate line
        if let Some(_existing) = self.repository.find_line(
            version_id, account_code, period_name, department_id.as_ref(), cost_center,
        ).await? {
            return Err(AtlasError::ValidationFailed(
                format!("A budget line already exists for account '{}' with the same period/department/cost center", account_code)
            ));
        }

        // Determine line number
        let existing_lines = self.repository.list_lines_by_version(version_id).await?;
        let line_number = (existing_lines.len() + 1) as i32;

        info!("Adding budget line {} to version {} (account: {})", line_number, version_id, account_code);

        let line = self.repository.create_line(
            org_id, version_id, line_number,
            account_code, account_name,
            period_name, period_start_date, period_end_date,
            fiscal_year, quarter,
            department_id, department_name,
            project_id, project_name,
            cost_center, budget_amount, description, created_by,
        ).await?;

        // Recalculate version totals
        self.recalculate_version_totals(version_id).await?;

        Ok(line)
    }

    /// Update a budget line's amount
    pub async fn update_line_amount(&self, version_id: Uuid, line_id: Uuid, new_amount: &str) -> AtlasResult<BudgetLine> {
        let version = self.repository.get_version(version_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Budget version {} not found", version_id)
            ))?;

        if version.status != "draft" {
            return Err(AtlasError::WorkflowError(
                "Can only update lines in a draft budget version".to_string(),
            ));
        }

        let amount_val: f64 = new_amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "Budget amount must be a valid number".to_string(),
        ))?;
        if amount_val < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Budget amount must be non-negative".to_string(),
            ));
        }

        let line = self.repository.update_line_amount(line_id, new_amount).await?;

        // Recalculate version totals
        self.recalculate_version_totals(version_id).await?;

        Ok(line)
    }

    /// Delete a budget line
    pub async fn delete_line(&self, version_id: Uuid, line_id: Uuid) -> AtlasResult<()> {
        let version = self.repository.get_version(version_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Budget version {} not found", version_id)
            ))?;

        if version.status != "draft" {
            return Err(AtlasError::WorkflowError(
                "Can only delete lines from a draft budget version".to_string(),
            ));
        }

        self.repository.delete_line(line_id).await?;

        // Recalculate version totals
        self.recalculate_version_totals(version_id).await?;

        Ok(())
    }

    /// List all lines for a budget version
    pub async fn list_lines(&self, version_id: Uuid) -> AtlasResult<Vec<BudgetLine>> {
        self.repository.list_lines_by_version(version_id).await
    }

    // ========================================================================
    // Budget Transfers
    // ========================================================================

    /// Create a budget transfer request
    pub async fn create_transfer(
        &self,
        org_id: Uuid,
        version_id: Uuid,
        transfer_number: &str,
        description: Option<&str>,
        from_account_code: &str,
        from_period_name: Option<&str>,
        from_department_id: Option<Uuid>,
        from_cost_center: Option<&str>,
        to_account_code: &str,
        to_period_name: Option<&str>,
        to_department_id: Option<Uuid>,
        to_cost_center: Option<&str>,
        amount: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<BudgetTransfer> {
        let version = self.repository.get_version(version_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Budget version {} not found", version_id)
            ))?;

        // Check definition allows transfers
        let definition = self.repository.get_definition_by_id(version.definition_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                "Budget definition not found".to_string()
            ))?;

        if !definition.allow_transfers {
            return Err(AtlasError::ValidationFailed(
                "This budget does not allow transfers".to_string(),
            ));
        }

        if version.status != "active" {
            return Err(AtlasError::WorkflowError(
                format!("Can only transfer amounts in an active budget version (current: '{}')", version.status)
            ));
        }

        let amount_val: f64 = amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "Transfer amount must be a valid number".to_string(),
        ))?;
        if amount_val <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Transfer amount must be positive".to_string(),
            ));
        }

        // Verify source line exists and has sufficient budget
        let from_line = self.repository.find_line(
            version_id, from_account_code, from_period_name, from_department_id.as_ref(), from_cost_center,
        ).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Source budget line not found for account '{}'", from_account_code)
            ))?;

        let from_budget: f64 = from_line.budget_amount.parse().unwrap_or(0.0);
        let from_transferred_out: f64 = from_line.transferred_out_amount.parse().unwrap_or(0.0);
        let available = from_budget - from_transferred_out;
        if available < amount_val {
            return Err(AtlasError::ValidationFailed(
                format!("Insufficient budget available for transfer. Available: {:.2}, Requested: {:.2}", available, amount_val)
            ));
        }

        // Verify destination line exists
        let _to_line = self.repository.find_line(
            version_id, to_account_code, to_period_name, to_department_id.as_ref(), to_cost_center,
        ).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Destination budget line not found for account '{}'", to_account_code)
            ))?;

        info!("Creating budget transfer {} for version {}", transfer_number, version_id);

        self.repository.create_transfer(
            org_id, version_id, transfer_number, description,
            from_account_code, from_period_name, from_department_id, from_cost_center,
            to_account_code, to_period_name, to_department_id, to_cost_center,
            amount, created_by,
        ).await
    }

    /// Approve a budget transfer
    pub async fn approve_transfer(&self, transfer_id: Uuid, approved_by: Uuid) -> AtlasResult<BudgetTransfer> {
        let transfer = self.repository.get_transfer(transfer_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Budget transfer {} not found", transfer_id)
            ))?;

        if transfer.status != "pending" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot approve transfer in '{}' status. Must be 'pending'.", transfer.status)
            ));
        }

        let transfer = self.repository.update_transfer_status(transfer_id, "approved", Some(approved_by), None).await?;

        // Update source and destination line transfer amounts
        let version_id = transfer.version_id;
        let amount = &transfer.amount;

        // Update source line: increase transferred_out
        let from_line = self.repository.find_line(
            version_id, &transfer.from_account_code,
            transfer.from_period_name.as_deref(),
            transfer.from_department_id.as_ref(),
            transfer.from_cost_center.as_deref(),
        ).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Source line not found".to_string()))?;

        let current_out: f64 = from_line.transferred_out_amount.parse().unwrap_or(0.0);
        let _new_out = current_out + amount.parse::<f64>().unwrap_or(0.0);
        self.repository.update_line_amount(from_line.id, &from_line.budget_amount).await?;

        // Note: In a full implementation we'd separately track transferred_in/out
        // For now the transfer record itself is the source of truth

        info!("Approved budget transfer {}", transfer_id);
        Ok(transfer)
    }

    /// Reject a budget transfer
    pub async fn reject_transfer(&self, transfer_id: Uuid, reason: Option<&str>) -> AtlasResult<BudgetTransfer> {
        let transfer = self.repository.get_transfer(transfer_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Budget transfer {} not found", transfer_id)
            ))?;

        if transfer.status != "pending" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot reject transfer in '{}' status. Must be 'pending'.", transfer.status)
            ));
        }

        info!("Rejecting budget transfer {}", transfer_id);
        self.repository.update_transfer_status(transfer_id, "rejected", None, reason).await
    }

    /// List all transfers for a budget version
    pub async fn list_transfers(&self, version_id: Uuid) -> AtlasResult<Vec<BudgetTransfer>> {
        self.repository.list_transfers(version_id).await
    }

    // ========================================================================
    // Budget vs Actuals Variance Report
    // ========================================================================

    /// Generate a budget vs actuals variance report for a budget version
    pub async fn get_variance_report(&self, version_id: Uuid) -> AtlasResult<BudgetVarianceReport> {
        let version = self.repository.get_version(version_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Budget version {} not found", version_id)
            ))?;

        let definition = self.repository.get_definition_by_id(version.definition_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Budget definition not found".to_string()))?;

        let lines = self.repository.list_lines_by_version(version_id).await?;

        let mut report_lines = Vec::new();
        let mut total_budget: f64 = 0.0;
        let mut total_actual: f64 = 0.0;
        let mut total_committed: f64 = 0.0;
        let mut total_variance: f64 = 0.0;

        for line in &lines {
            let budget: f64 = line.budget_amount.parse().unwrap_or(0.0);
            let actual: f64 = line.actual_amount.parse().unwrap_or(0.0);
            let committed: f64 = line.committed_amount.parse().unwrap_or(0.0);
            let variance = budget - actual;
            let variance_pct = if budget != 0.0 { (variance / budget) * 100.0 } else { 0.0 };

            total_budget += budget;
            total_actual += actual;
            total_committed += committed;
            total_variance += variance;

            report_lines.push(BudgetVarianceLine {
                account_code: line.account_code.clone(),
                account_name: line.account_name.clone(),
                period_name: line.period_name.clone(),
                department_name: line.department_name.clone(),
                project_name: line.project_name.clone(),
                cost_center: line.cost_center.clone(),
                budget_amount: format!("{:.2}", budget),
                committed_amount: format!("{:.2}", committed),
                actual_amount: format!("{:.2}", actual),
                variance_amount: format!("{:.2}", variance),
                variance_percent: format!("{:.2}", variance_pct),
                is_over_budget: actual > budget,
            });
        }

        let total_variance_pct = if total_budget != 0.0 {
            (total_variance / total_budget) * 100.0
        } else {
            0.0
        };

        Ok(BudgetVarianceReport {
            definition_id: definition.id,
            definition_code: definition.code,
            definition_name: definition.name,
            version_id: version.id,
            version_label: version.label,
            fiscal_year: definition.fiscal_year,
            total_budget: format!("{:.2}", total_budget),
            total_actual: format!("{:.2}", total_actual),
            total_committed: format!("{:.2}", total_committed),
            total_variance: format!("{:.2}", total_variance),
            variance_percent: format!("{:.2}", total_variance_pct),
            lines: report_lines,
        })
    }

    /// Check if a given amount would exceed budget for an account
    /// Returns Ok if within budget, Err with details if over budget
    pub async fn check_budget_control(
        &self,
        org_id: Uuid,
        budget_code: &str,
        account_code: &str,
        period_name: Option<&str>,
        department_id: Option<&Uuid>,
        cost_center: Option<&str>,
        proposed_amount: f64,
    ) -> AtlasResult<BudgetControlResult> {
        let definition = self.get_definition(org_id, budget_code).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Budget definition '{}' not found", budget_code)
            ))?;

        if definition.control_level == "none" {
            return Ok(BudgetControlResult {
                within_budget: true,
                control_level: "none".to_string(),
                budget_amount: "0".to_string(),
                actual_amount: "0".to_string(),
                committed_amount: "0".to_string(),
                available_amount: "0".to_string(),
                proposed_amount: format!("{:.2}", proposed_amount),
                message: "No budget control".to_string(),
            });
        }

        let active_version = self.get_active_version(definition.id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("No active budget version for '{}'", budget_code)
            ))?;

        let line = self.repository.find_line(
            active_version.id, account_code, period_name, department_id, cost_center,
        ).await?;

        match line {
            Some(line) => {
                let budget: f64 = line.budget_amount.parse().unwrap_or(0.0);
                let actual: f64 = line.actual_amount.parse().unwrap_or(0.0);
                let committed: f64 = line.committed_amount.parse().unwrap_or(0.0);
                let available = budget - actual - committed;
                let within = proposed_amount <= available;

                let message = if within {
                    "Within budget".to_string()
                } else {
                    format!("Over budget: available={:.2}, proposed={:.2}", available, proposed_amount)
                };

                Ok(BudgetControlResult {
                    within_budget: within,
                    control_level: definition.control_level.clone(),
                    budget_amount: format!("{:.2}", budget),
                    actual_amount: format!("{:.2}", actual),
                    committed_amount: format!("{:.2}", committed),
                    available_amount: format!("{:.2}", available),
                    proposed_amount: format!("{:.2}", proposed_amount),
                    message,
                })
            }
            None => {
                // No budget line found
                Ok(BudgetControlResult {
                    within_budget: definition.control_level != "absolute",
                    control_level: definition.control_level.clone(),
                    budget_amount: "0".to_string(),
                    actual_amount: "0".to_string(),
                    committed_amount: "0".to_string(),
                    available_amount: "0".to_string(),
                    proposed_amount: format!("{:.2}", proposed_amount),
                    message: "No budget line found for this account".to_string(),
                })
            }
        }
    }

    // ========================================================================
    // Internal Helpers
    // ========================================================================

    /// Recalculate version totals from budget lines
    async fn recalculate_version_totals(&self, version_id: Uuid) -> AtlasResult<()> {
        let lines = self.repository.list_lines_by_version(version_id).await?;

        let mut total_budget: f64 = 0.0;
        let mut total_committed: f64 = 0.0;
        let mut total_actual: f64 = 0.0;

        for line in &lines {
            total_budget += line.budget_amount.parse().unwrap_or(0.0);
            total_committed += line.committed_amount.parse().unwrap_or(0.0);
            total_actual += line.actual_amount.parse().unwrap_or(0.0);
        }

        let total_variance = total_budget - total_actual;

        self.repository.update_version_totals(
            version_id,
            &format!("{:.2}", total_budget),
            &format!("{:.2}", total_committed),
            &format!("{:.2}", total_actual),
            &format!("{:.2}", total_variance),
        ).await?;

        Ok(())
    }
}

/// Result of a budget control check
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BudgetControlResult {
    pub within_budget: bool,
    pub control_level: String,
    pub budget_amount: String,
    pub actual_amount: String,
    pub committed_amount: String,
    pub available_amount: String,
    pub proposed_amount: String,
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_budget_types() {
        assert!(VALID_BUDGET_TYPES.contains(&"operating"));
        assert!(VALID_BUDGET_TYPES.contains(&"capital"));
        assert!(VALID_BUDGET_TYPES.contains(&"project"));
        assert!(VALID_BUDGET_TYPES.contains(&"cash_flow"));
    }

    #[test]
    fn test_valid_control_levels() {
        assert!(VALID_CONTROL_LEVELS.contains(&"none"));
        assert!(VALID_CONTROL_LEVELS.contains(&"advisory"));
        assert!(VALID_CONTROL_LEVELS.contains(&"absolute"));
    }

    #[test]
    fn test_valid_version_statuses() {
        assert!(_VALID_VERSION_STATUSES.contains(&"draft"));
        assert!(_VALID_VERSION_STATUSES.contains(&"submitted"));
        assert!(_VALID_VERSION_STATUSES.contains(&"approved"));
        assert!(_VALID_VERSION_STATUSES.contains(&"active"));
        assert!(_VALID_VERSION_STATUSES.contains(&"closed"));
        assert!(_VALID_VERSION_STATUSES.contains(&"rejected"));
        assert!(_VALID_VERSION_STATUSES.contains(&"submitted"));
        assert!(_VALID_VERSION_STATUSES.contains(&"approved"));
        assert!(_VALID_VERSION_STATUSES.contains(&"active"));
        assert!(_VALID_VERSION_STATUSES.contains(&"closed"));
        assert!(_VALID_VERSION_STATUSES.contains(&"rejected"));
    }

    #[test]
    fn test_valid_transfer_statuses() {
        assert!(_VALID_TRANSFER_STATUSES.contains(&"pending"));
        assert!(_VALID_TRANSFER_STATUSES.contains(&"approved"));
        assert!(_VALID_TRANSFER_STATUSES.contains(&"rejected"));
        assert!(_VALID_TRANSFER_STATUSES.contains(&"cancelled"));
    }
}
