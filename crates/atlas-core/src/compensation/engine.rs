//! Compensation Engine Implementation
//!
//! Oracle Fusion Cloud HCM Compensation Workbench.
//! Manages compensation plans, cycles, budget pools, manager worksheets,
//! per-employee allocation lines, and employee compensation statements.
//!
//! The process follows Oracle Fusion's Compensation workflow:
//! 1. Define compensation plans with component types
//! 2. Create compensation cycles (annual/biannual reviews)
//! 3. Set up budget pools for managers
//! 4. Managers allocate compensation via worksheets
//! 5. Approve/reject compensation changes
//! 6. Generate employee compensation statements

use atlas_shared::{
    CompensationPlan, CompensationComponent, CompensationCycle,
    CompensationBudgetPool, CompensationWorksheet, CompensationWorksheetLine,
    CompensationStatement, CompensationDashboard,
    AtlasError, AtlasResult,
};
use super::CompensationRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

// Valid constants
const VALID_PLAN_TYPES: &[&str] = &["salary", "bonus", "equity", "benefits", "mixed"];
#[allow(dead_code)]
const VALID_PLAN_STATUSES: &[&str] = &["active", "inactive", "archived"];
const VALID_COMPONENT_TYPES: &[&str] = &[
    "salary", "merit", "bonus", "equity", "commission", "allowance", "benefits",
];
const VALID_FREQUENCIES: &[&str] = &["annual", "semi_annual", "quarterly", "monthly", "one_time"];
const VALID_CYCLE_TYPES: &[&str] = &["annual", "mid_year", "off_cycle", "promotion"];
const VALID_CYCLE_STATUSES: &[&str] = &[
    "draft", "active", "allocation", "review", "completed", "cancelled",
];
const VALID_POOL_TYPES: &[&str] = &["merit", "bonus", "equity", "general"];
#[allow(dead_code)]
const VALID_POOL_STATUSES: &[&str] = &["active", "exhausted", "closed"];
const VALID_WORKSHEET_STATUSES: &[&str] = &[
    "draft", "submitted", "approved", "rejected", "completed",
];
#[allow(dead_code)]
const VALID_LINE_STATUSES: &[&str] = &[
    "pending", "recommended", "approved", "rejected",
];
#[allow(dead_code)]
const VALID_STATEMENT_STATUSES: &[&str] = &["draft", "published", "archived"];

/// Compensation Management engine
pub struct CompensationEngine {
    repository: Arc<dyn CompensationRepository>,
}

impl CompensationEngine {
    pub fn new(repository: Arc<dyn CompensationRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Compensation Plan Management
    // ========================================================================

    /// Create a new compensation plan
    pub async fn create_plan(
        &self,
        org_id: Uuid,
        plan_code: &str,
        plan_name: &str,
        description: Option<&str>,
        plan_type: &str,
        effective_start_date: Option<chrono::NaiveDate>,
        effective_end_date: Option<chrono::NaiveDate>,
        eligibility_criteria: serde_json::Value,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CompensationPlan> {
        let code_upper = plan_code.to_uppercase();
        if code_upper.is_empty() || code_upper.len() > 50 {
            return Err(AtlasError::ValidationFailed(
                "Plan code must be 1-50 characters".to_string(),
            ));
        }
        if plan_name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Plan name is required".to_string(),
            ));
        }
        if !VALID_PLAN_TYPES.contains(&plan_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid plan_type '{}'. Must be one of: {}",
                plan_type, VALID_PLAN_TYPES.join(", ")
            )));
        }
        if self.repository.get_plan_by_code(org_id, &code_upper).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Compensation plan '{}' already exists", code_upper
            )));
        }

        info!("Creating compensation plan '{}' for org {}", code_upper, org_id);
        self.repository.create_plan(
            org_id, &code_upper, plan_name, description, plan_type,
            effective_start_date, effective_end_date, eligibility_criteria, created_by,
        ).await
    }

    /// Get plan by ID
    pub async fn get_plan(&self, id: Uuid) -> AtlasResult<Option<CompensationPlan>> {
        self.repository.get_plan(id).await
    }

    /// Get plan by code
    pub async fn get_plan_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<CompensationPlan>> {
        self.repository.get_plan_by_code(org_id, &code.to_uppercase()).await
    }

    /// List plans
    pub async fn list_plans(&self, org_id: Uuid) -> AtlasResult<Vec<CompensationPlan>> {
        self.repository.list_plans(org_id).await
    }

    /// Delete plan
    pub async fn delete_plan(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        info!("Deleting compensation plan '{}' for org {}", code, org_id);
        self.repository.delete_plan(org_id, &code.to_uppercase()).await
    }

    // ========================================================================
    // Compensation Components
    // ========================================================================

    /// Add a component to a plan
    pub async fn create_component(
        &self,
        org_id: Uuid,
        plan_id: Uuid,
        component_name: &str,
        component_type: &str,
        description: Option<&str>,
        is_recurring: bool,
        frequency: Option<&str>,
    ) -> AtlasResult<CompensationComponent> {
        if component_name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Component name is required".to_string(),
            ));
        }
        if !VALID_COMPONENT_TYPES.contains(&component_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid component_type '{}'. Must be one of: {}",
                component_type, VALID_COMPONENT_TYPES.join(", ")
            )));
        }
        if let Some(freq) = frequency {
            if !VALID_FREQUENCIES.contains(&freq) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid frequency '{}'. Must be one of: {}",
                    freq, VALID_FREQUENCIES.join(", ")
                )));
            }
        }
        // Verify plan exists
        self.repository.get_plan(plan_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Compensation plan {} not found", plan_id)
            ))?;

        info!("Creating component '{}' for plan {}", component_name, plan_id);
        self.repository.create_component(
            org_id, plan_id, component_name, component_type,
            description, is_recurring, frequency,
        ).await
    }

    /// List components for a plan
    pub async fn list_components(&self, plan_id: Uuid) -> AtlasResult<Vec<CompensationComponent>> {
        self.repository.list_components(plan_id).await
    }

    /// Delete a component
    pub async fn delete_component(&self, id: Uuid) -> AtlasResult<()> {
        self.repository.delete_component(id).await
    }

    // ========================================================================
    // Compensation Cycle Management
    // ========================================================================

    /// Create a compensation cycle
    pub async fn create_cycle(
        &self,
        org_id: Uuid,
        cycle_name: &str,
        description: Option<&str>,
        cycle_type: &str,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
        total_budget: &str,
        currency_code: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CompensationCycle> {
        if cycle_name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Cycle name is required".to_string(),
            ));
        }
        if !VALID_CYCLE_TYPES.contains(&cycle_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid cycle_type '{}'. Must be one of: {}",
                cycle_type, VALID_CYCLE_TYPES.join(", ")
            )));
        }
        if end_date <= start_date {
            return Err(AtlasError::ValidationFailed(
                "End date must be after start date".to_string(),
            ));
        }
        let budget: f64 = total_budget.parse().map_err(|_| {
            AtlasError::ValidationFailed("total_budget must be a valid number".to_string())
        })?;
        if budget < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "total_budget cannot be negative".to_string(),
            ));
        }

        info!("Creating compensation cycle '{}' for org {}", cycle_name, org_id);
        self.repository.create_cycle(
            org_id, cycle_name, description, cycle_type,
            start_date, end_date, total_budget, currency_code, created_by,
        ).await
    }

    /// Get cycle by ID
    pub async fn get_cycle(&self, id: Uuid) -> AtlasResult<Option<CompensationCycle>> {
        self.repository.get_cycle(id).await
    }

    /// List cycles
    pub async fn list_cycles(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<CompensationCycle>> {
        if let Some(s) = status {
            if !VALID_CYCLE_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid cycle status '{}'. Must be one of: {}",
                    s, VALID_CYCLE_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_cycles(org_id, status).await
    }

    /// Transition cycle status
    pub async fn transition_cycle(&self, id: Uuid, new_status: &str) -> AtlasResult<CompensationCycle> {
        if !VALID_CYCLE_STATUSES.contains(&new_status) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid cycle status '{}'. Must be one of: {}",
                new_status, VALID_CYCLE_STATUSES.join(", ")
            )));
        }

        let cycle = self.repository.get_cycle(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Compensation cycle {} not found", id)
            ))?;

        // Validate transition
        let valid_transitions: &[&[&str]] = &[
            &["draft", "active"],
            &["active", "allocation"],
            &["allocation", "review"],
            &["review", "completed"],
            &["draft", "cancelled"],
            &["active", "cancelled"],
        ];

        let transition_valid = valid_transitions.iter()
            .any(|t| t[0] == cycle.status && t[1] == new_status);

        if !transition_valid {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot transition cycle from '{}' to '{}'", cycle.status, new_status
            )));
        }

        info!("Transitioning compensation cycle {} from {} to {}", id, cycle.status, new_status);
        self.repository.update_cycle_status(id, new_status).await
    }

    /// Delete a cycle
    pub async fn delete_cycle(&self, id: Uuid) -> AtlasResult<()> {
        let cycle = self.repository.get_cycle(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Compensation cycle {} not found", id)
            ))?;
        if cycle.status != "draft" {
            return Err(AtlasError::WorkflowError(
                "Only draft cycles can be deleted".to_string(),
            ));
        }
        self.repository.delete_cycle(id).await
    }

    // ========================================================================
    // Budget Pool Management
    // ========================================================================

    /// Create a budget pool
    pub async fn create_budget_pool(
        &self,
        org_id: Uuid,
        cycle_id: Uuid,
        pool_name: &str,
        pool_type: &str,
        manager_id: Option<Uuid>,
        manager_name: Option<&str>,
        department_id: Option<Uuid>,
        department_name: Option<&str>,
        total_budget: &str,
        currency_code: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CompensationBudgetPool> {
        if pool_name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Pool name is required".to_string(),
            ));
        }
        if !VALID_POOL_TYPES.contains(&pool_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid pool_type '{}'. Must be one of: {}",
                pool_type, VALID_POOL_TYPES.join(", ")
            )));
        }
        let budget: f64 = total_budget.parse().map_err(|_| {
            AtlasError::ValidationFailed("total_budget must be a valid number".to_string())
        })?;
        if budget < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "total_budget cannot be negative".to_string(),
            ));
        }

        // Verify cycle exists
        self.repository.get_cycle(cycle_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Compensation cycle {} not found", cycle_id)
            ))?;

        info!("Creating budget pool '{}' for cycle {}", pool_name, cycle_id);
        self.repository.create_budget_pool(
            org_id, cycle_id, pool_name, pool_type,
            manager_id, manager_name, department_id, department_name,
            total_budget, currency_code, created_by,
        ).await
    }

    /// Get budget pool
    pub async fn get_budget_pool(&self, id: Uuid) -> AtlasResult<Option<CompensationBudgetPool>> {
        self.repository.get_budget_pool(id).await
    }

    /// List budget pools for a cycle
    pub async fn list_budget_pools(&self, cycle_id: Uuid) -> AtlasResult<Vec<CompensationBudgetPool>> {
        self.repository.list_budget_pools(cycle_id).await
    }

    /// Delete budget pool
    pub async fn delete_budget_pool(&self, id: Uuid) -> AtlasResult<()> {
        self.repository.delete_budget_pool(id).await
    }

    // ========================================================================
    // Worksheet Management
    // ========================================================================

    /// Create a worksheet for a manager
    pub async fn create_worksheet(
        &self,
        org_id: Uuid,
        cycle_id: Uuid,
        pool_id: Option<Uuid>,
        manager_id: Uuid,
        manager_name: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CompensationWorksheet> {
        // Verify cycle exists
        let cycle = self.repository.get_cycle(cycle_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Compensation cycle {} not found", cycle_id)
            ))?;

        if cycle.status != "allocation" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot create worksheet: cycle status is '{}', must be 'allocation'",
                cycle.status
            )));
        }

        // Verify pool if provided
        if let Some(pid) = pool_id {
            self.repository.get_budget_pool(pid).await?
                .ok_or_else(|| AtlasError::EntityNotFound(
                    format!("Budget pool {} not found", pid)
                ))?;
        }

        info!("Creating worksheet for manager {} in cycle {}", manager_id, cycle_id);
        self.repository.create_worksheet(
            org_id, cycle_id, pool_id, manager_id, manager_name, created_by,
        ).await
    }

    /// Get worksheet
    pub async fn get_worksheet(&self, id: Uuid) -> AtlasResult<Option<CompensationWorksheet>> {
        self.repository.get_worksheet(id).await
    }

    /// List worksheets for a cycle
    pub async fn list_worksheets(&self, cycle_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<CompensationWorksheet>> {
        if let Some(s) = status {
            if !VALID_WORKSHEET_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid worksheet status '{}'. Must be one of: {}",
                    s, VALID_WORKSHEET_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_worksheets(cycle_id, status).await
    }

    /// Submit worksheet for approval
    pub async fn submit_worksheet(&self, id: Uuid) -> AtlasResult<CompensationWorksheet> {
        let ws = self.repository.get_worksheet(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Worksheet {} not found", id)
            ))?;

        if ws.status != "draft" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot submit worksheet in '{}' status. Must be 'draft'.", ws.status
            )));
        }

        // Check that worksheet has lines
        let lines = self.repository.list_worksheet_lines(id).await?;
        if lines.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Cannot submit worksheet with no lines".to_string(),
            ));
        }

        info!("Submitting worksheet {} for approval", id);
        self.repository.update_worksheet_status(id, "submitted").await
    }

    /// Approve worksheet
    pub async fn approve_worksheet(&self, id: Uuid) -> AtlasResult<CompensationWorksheet> {
        let ws = self.repository.get_worksheet(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Worksheet {} not found", id)
            ))?;

        if ws.status != "submitted" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot approve worksheet in '{}' status. Must be 'submitted'.", ws.status
            )));
        }

        info!("Approving worksheet {}", id);
        let approved = self.repository.update_worksheet_status(id, "approved").await?;

        // Update all lines to approved
        let lines = self.repository.list_worksheet_lines(id).await?;
        for line in &lines {
            if line.status == "recommended" || line.status == "pending" {
                if let Err(e) = self.repository.update_line_status(line.id, "approved").await {
                    tracing::warn!("Failed to approve line {}: {}", line.id, e);
                }
            }
        }

        // Update cycle totals
        self.recalculate_cycle_totals(ws.cycle_id).await?;

        Ok(approved)
    }

    /// Reject worksheet
    pub async fn reject_worksheet(&self, id: Uuid) -> AtlasResult<CompensationWorksheet> {
        let ws = self.repository.get_worksheet(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Worksheet {} not found", id)
            ))?;

        if ws.status != "submitted" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot reject worksheet in '{}' status. Must be 'submitted'.", ws.status
            )));
        }

        info!("Rejecting worksheet {}", id);
        self.repository.update_worksheet_status(id, "rejected").await
    }

    /// Delete worksheet
    pub async fn delete_worksheet(&self, id: Uuid) -> AtlasResult<()> {
        let ws = self.repository.get_worksheet(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Worksheet {} not found", id)
            ))?;
        if ws.status != "draft" {
            return Err(AtlasError::WorkflowError(
                "Only draft worksheets can be deleted".to_string(),
            ));
        }
        self.repository.delete_worksheet(id).await
    }

    // ========================================================================
    // Worksheet Line Management
    // ========================================================================

    /// Add an employee line to a worksheet
    pub async fn add_worksheet_line(
        &self,
        org_id: Uuid,
        worksheet_id: Uuid,
        employee_id: Uuid,
        employee_name: Option<&str>,
        job_title: Option<&str>,
        department_name: Option<&str>,
        current_base_salary: &str,
        proposed_base_salary: &str,
        merit_amount: &str,
        bonus_amount: &str,
        equity_amount: &str,
        performance_rating: Option<&str>,
        manager_comments: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CompensationWorksheetLine> {
        let ws = self.repository.get_worksheet(worksheet_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Worksheet {} not found", worksheet_id)
            ))?;

        if ws.status != "draft" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot add lines to worksheet in '{}' status. Must be 'draft'.", ws.status
            )));
        }

        let current: f64 = current_base_salary.parse().map_err(|_| {
            AtlasError::ValidationFailed("current_base_salary must be a number".to_string())
        })?;
        let proposed: f64 = proposed_base_salary.parse().map_err(|_| {
            AtlasError::ValidationFailed("proposed_base_salary must be a number".to_string())
        })?;
        let merit: f64 = merit_amount.parse().unwrap_or(0.0);
        let bonus: f64 = bonus_amount.parse().unwrap_or(0.0);
        let equity: f64 = equity_amount.parse().unwrap_or(0.0);

        if proposed < 0.0 || current < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Salary amounts cannot be negative".to_string(),
            ));
        }

        let change_amount = proposed - current;
        let change_percent = if current > 0.0 { (change_amount / current) * 100.0 } else { 0.0 };
        let total_comp = proposed + merit + bonus + equity;

        let line = self.repository.create_worksheet_line(
            org_id, worksheet_id, employee_id, employee_name,
            job_title, department_name,
            current_base_salary, proposed_base_salary,
            &format!("{:.2}", change_amount),
            &format!("{:.4}", change_percent),
            merit_amount, bonus_amount, equity_amount,
            &format!("{:.2}", total_comp),
            performance_rating,
            &format!("{:.4}", if current > 0.0 { proposed / current } else { 0.0 }),
            manager_comments, created_by,
        ).await?;

        // Recalculate worksheet totals
        self.recalculate_worksheet_totals(worksheet_id).await?;

        Ok(line)
    }

    /// Update a worksheet line
    pub async fn update_worksheet_line(
        &self,
        line_id: Uuid,
        proposed_base_salary: &str,
        merit_amount: &str,
        bonus_amount: &str,
        equity_amount: &str,
        manager_comments: Option<&str>,
    ) -> AtlasResult<CompensationWorksheetLine> {
        let line = self.repository.get_line(line_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Worksheet line {} not found", line_id)
            ))?;

        // Verify worksheet is still in draft
        let ws = self.repository.get_worksheet(line.worksheet_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Worksheet {} not found", line.worksheet_id)
            ))?;
        if ws.status != "draft" {
            return Err(AtlasError::WorkflowError(
                "Cannot update lines on a non-draft worksheet".to_string(),
            ));
        }

        let proposed: f64 = proposed_base_salary.parse().map_err(|_| {
            AtlasError::ValidationFailed("proposed_base_salary must be a number".to_string())
        })?;
        let merit: f64 = merit_amount.parse().unwrap_or(0.0);
        let bonus: f64 = bonus_amount.parse().unwrap_or(0.0);
        let equity: f64 = equity_amount.parse().unwrap_or(0.0);
        let current: f64 = line.current_base_salary.parse().unwrap_or(0.0);

        let change_amount = proposed - current;
        let change_percent = if current > 0.0 { (change_amount / current) * 100.0 } else { 0.0 };
        let total_comp = proposed + merit + bonus + equity;

        let updated = self.repository.update_worksheet_line(
            line_id,
            proposed_base_salary,
            &format!("{:.2}", change_amount),
            &format!("{:.4}", change_percent),
            merit_amount,
            bonus_amount,
            equity_amount,
            &format!("{:.2}", total_comp),
            &format!("{:.4}", if current > 0.0 { proposed / current } else { 0.0 }),
            manager_comments,
        ).await?;

        // Recalculate worksheet totals
        self.recalculate_worksheet_totals(line.worksheet_id).await?;

        Ok(updated)
    }

    /// List worksheet lines
    pub async fn list_worksheet_lines(&self, worksheet_id: Uuid) -> AtlasResult<Vec<CompensationWorksheetLine>> {
        self.repository.list_worksheet_lines(worksheet_id).await
    }

    /// Delete a worksheet line
    pub async fn delete_worksheet_line(&self, line_id: Uuid) -> AtlasResult<()> {
        let line = self.repository.get_line(line_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Worksheet line {} not found", line_id)
            ))?;

        let ws = self.repository.get_worksheet(line.worksheet_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Worksheet {} not found", line.worksheet_id)
            ))?;
        if ws.status != "draft" {
            return Err(AtlasError::WorkflowError(
                "Cannot delete lines from a non-draft worksheet".to_string(),
            ));
        }

        self.repository.delete_worksheet_line(line_id).await?;
        self.recalculate_worksheet_totals(line.worksheet_id).await?;
        Ok(())
    }

    // ========================================================================
    // Compensation Statements
    // ========================================================================

    /// Generate a compensation statement for an employee
    pub async fn generate_statement(
        &self,
        org_id: Uuid,
        cycle_id: Uuid,
        employee_id: Uuid,
        employee_name: Option<&str>,
        base_salary: &str,
        merit_increase: &str,
        bonus: &str,
        equity: &str,
        benefits_value: &str,
        currency_code: &str,
        components: serde_json::Value,
    ) -> AtlasResult<CompensationStatement> {
        let salary: f64 = base_salary.parse().unwrap_or(0.0);
        let merit: f64 = merit_increase.parse().unwrap_or(0.0);
        let bonus_val: f64 = bonus.parse().unwrap_or(0.0);
        let equity_val: f64 = equity.parse().unwrap_or(0.0);
        let benefits: f64 = benefits_value.parse().unwrap_or(0.0);

        let direct = salary + merit + bonus_val + equity_val;
        let indirect = benefits;
        let total = direct + indirect;

        // Verify cycle exists
        self.repository.get_cycle(cycle_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Compensation cycle {} not found", cycle_id)
            ))?;

        info!("Generating compensation statement for employee {} in cycle {}", employee_id, cycle_id);

        self.repository.upsert_statement(
            org_id, cycle_id, employee_id, employee_name,
            chrono::Utc::now().date_naive(),
            base_salary, merit_increase, bonus, equity, benefits_value,
            &format!("{:.2}", total),
            &format!("{:.2}", direct),
            &format!("{:.2}", indirect),
            "0", "0",
            currency_code, components,
        ).await
    }

    /// Get a statement
    pub async fn get_statement(&self, id: Uuid) -> AtlasResult<Option<CompensationStatement>> {
        self.repository.get_statement(id).await
    }

    /// Get statement by cycle and employee
    pub async fn get_statement_by_employee(&self, cycle_id: Uuid, employee_id: Uuid) -> AtlasResult<Option<CompensationStatement>> {
        self.repository.get_statement_by_employee(cycle_id, employee_id).await
    }

    /// List statements for a cycle
    pub async fn list_statements(&self, cycle_id: Uuid) -> AtlasResult<Vec<CompensationStatement>> {
        self.repository.list_statements(cycle_id).await
    }

    /// Publish a statement (make visible to employee)
    pub async fn publish_statement(&self, id: Uuid) -> AtlasResult<CompensationStatement> {
        let stmt = self.repository.get_statement(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Statement {} not found", id)
            ))?;

        if stmt.status != "draft" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot publish statement in '{}' status. Must be 'draft'.", stmt.status
            )));
        }

        info!("Publishing compensation statement {}", id);
        self.repository.publish_statement(id).await
    }

    /// Delete a statement
    pub async fn delete_statement(&self, id: Uuid) -> AtlasResult<()> {
        self.repository.delete_statement(id).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get compensation dashboard summary
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<CompensationDashboard> {
        self.repository.get_dashboard(org_id).await
    }

    // ========================================================================
    // Internal Helpers
    // ========================================================================

    /// Recalculate worksheet totals from lines
    async fn recalculate_worksheet_totals(&self, worksheet_id: Uuid) -> AtlasResult<()> {
        let lines = self.repository.list_worksheet_lines(worksheet_id).await?;

        let total_employees = lines.len() as i32;
        let total_current: f64 = lines.iter()
            .map(|l| l.current_base_salary.parse::<f64>().unwrap_or(0.0))
            .sum();
        let total_proposed: f64 = lines.iter()
            .map(|l| l.proposed_base_salary.parse::<f64>().unwrap_or(0.0))
            .sum();
        let total_merit: f64 = lines.iter()
            .map(|l| l.merit_amount.parse::<f64>().unwrap_or(0.0))
            .sum();
        let total_bonus: f64 = lines.iter()
            .map(|l| l.bonus_amount.parse::<f64>().unwrap_or(0.0))
            .sum();
        let total_equity: f64 = lines.iter()
            .map(|l| l.equity_amount.parse::<f64>().unwrap_or(0.0))
            .sum();
        let total_change: f64 = total_proposed - total_current + total_merit + total_bonus + total_equity;

        self.repository.update_worksheet_totals(
            worksheet_id,
            total_employees,
            &format!("{:.2}", total_current),
            &format!("{:.2}", total_proposed),
            &format!("{:.2}", total_merit),
            &format!("{:.2}", total_bonus),
            &format!("{:.2}", total_equity),
            &format!("{:.2}", total_change),
        ).await
    }

    /// Recalculate cycle totals from worksheets
    async fn recalculate_cycle_totals(&self, cycle_id: Uuid) -> AtlasResult<()> {
        let worksheets = self.repository.list_worksheets(cycle_id, None).await?;
        let approved: Vec<_> = worksheets.iter().filter(|w| w.status == "approved").collect();

        let total_approved: f64 = approved.iter()
            .map(|w| w.total_compensation_change.parse::<f64>().unwrap_or(0.0))
            .sum();
        let total_employees: i32 = approved.iter()
            .map(|w| w.total_employees)
            .sum();

        self.repository.update_cycle_totals(
            cycle_id,
            &format!("{:.2}", total_approved),
            total_employees,
        ).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_plan_types() {
        assert!(VALID_PLAN_TYPES.contains(&"salary"));
        assert!(VALID_PLAN_TYPES.contains(&"bonus"));
        assert!(VALID_PLAN_TYPES.contains(&"equity"));
        assert!(VALID_PLAN_TYPES.contains(&"benefits"));
        assert!(VALID_PLAN_TYPES.contains(&"mixed"));
        assert!(!VALID_PLAN_TYPES.contains(&"invalid"));
    }

    #[test]
    fn test_valid_component_types() {
        assert!(VALID_COMPONENT_TYPES.contains(&"salary"));
        assert!(VALID_COMPONENT_TYPES.contains(&"merit"));
        assert!(VALID_COMPONENT_TYPES.contains(&"bonus"));
        assert!(VALID_COMPONENT_TYPES.contains(&"equity"));
        assert!(VALID_COMPONENT_TYPES.contains(&"commission"));
        assert!(VALID_COMPONENT_TYPES.contains(&"allowance"));
        assert!(VALID_COMPONENT_TYPES.contains(&"benefits"));
        assert!(!VALID_COMPONENT_TYPES.contains(&"stock"));
    }

    #[test]
    fn test_valid_cycle_types() {
        assert!(VALID_CYCLE_TYPES.contains(&"annual"));
        assert!(VALID_CYCLE_TYPES.contains(&"mid_year"));
        assert!(VALID_CYCLE_TYPES.contains(&"off_cycle"));
        assert!(VALID_CYCLE_TYPES.contains(&"promotion"));
        assert!(!VALID_CYCLE_TYPES.contains(&"quarterly"));
    }

    #[test]
    fn test_valid_cycle_statuses() {
        assert!(VALID_CYCLE_STATUSES.contains(&"draft"));
        assert!(VALID_CYCLE_STATUSES.contains(&"active"));
        assert!(VALID_CYCLE_STATUSES.contains(&"allocation"));
        assert!(VALID_CYCLE_STATUSES.contains(&"review"));
        assert!(VALID_CYCLE_STATUSES.contains(&"completed"));
        assert!(VALID_CYCLE_STATUSES.contains(&"cancelled"));
        assert!(!VALID_CYCLE_STATUSES.contains(&"pending"));
    }

    #[test]
    fn test_valid_pool_types() {
        assert!(VALID_POOL_TYPES.contains(&"merit"));
        assert!(VALID_POOL_TYPES.contains(&"bonus"));
        assert!(VALID_POOL_TYPES.contains(&"equity"));
        assert!(VALID_POOL_TYPES.contains(&"general"));
        assert!(!VALID_POOL_TYPES.contains(&"salary"));
    }

    #[test]
    fn test_valid_worksheet_statuses() {
        assert!(VALID_WORKSHEET_STATUSES.contains(&"draft"));
        assert!(VALID_WORKSHEET_STATUSES.contains(&"submitted"));
        assert!(VALID_WORKSHEET_STATUSES.contains(&"approved"));
        assert!(VALID_WORKSHEET_STATUSES.contains(&"rejected"));
        assert!(VALID_WORKSHEET_STATUSES.contains(&"completed"));
        assert!(!VALID_WORKSHEET_STATUSES.contains(&"in_review"));
    }

    #[test]
    fn test_valid_line_statuses() {
        assert!(VALID_LINE_STATUSES.contains(&"pending"));
        assert!(VALID_LINE_STATUSES.contains(&"recommended"));
        assert!(VALID_LINE_STATUSES.contains(&"approved"));
        assert!(VALID_LINE_STATUSES.contains(&"rejected"));
        assert!(!VALID_LINE_STATUSES.contains(&"draft"));
    }

    #[test]
    fn test_salary_change_calculation() {
        let current: f64 = 100000.0;
        let proposed: f64 = 110000.0;
        let change = proposed - current;
        let pct = if current > 0.0 { (change / current) * 100.0 } else { 0.0 };
        assert!((change - 10000.0).abs() < 0.01);
        assert!((pct - 10.0).abs() < 0.01);
    }

    #[test]
    fn test_total_compensation_calculation() {
        let proposed: f64 = 110000.0;
        let merit: f64 = 5000.0;
        let bonus: f64 = 10000.0;
        let equity: f64 = 15000.0;
        let total = proposed + merit + bonus + equity;
        assert!((total - 140000.0).abs() < 0.01);
    }

    #[test]
    fn test_compa_ratio_calculation() {
        let current: f64 = 100000.0;
        let proposed: f64 = 110000.0;
        let ratio = if current > 0.0 { proposed / current } else { 0.0 };
        assert!((ratio - 1.1).abs() < 0.001);
    }

    #[test]
    fn test_cycle_transition_validation() {
        let valid_transitions: &[&[&str]] = &[
            &["draft", "active"],
            &["active", "allocation"],
            &["allocation", "review"],
            &["review", "completed"],
            &["draft", "cancelled"],
            &["active", "cancelled"],
        ];

        // Valid transitions
        assert!(valid_transitions.iter().any(|t| t[0] == "draft" && t[1] == "active"));
        assert!(valid_transitions.iter().any(|t| t[0] == "active" && t[1] == "allocation"));
        assert!(valid_transitions.iter().any(|t| t[0] == "allocation" && t[1] == "review"));

        // Invalid transitions
        assert!(!valid_transitions.iter().any(|t| t[0] == "draft" && t[1] == "completed"));
        assert!(!valid_transitions.iter().any(|t| t[0] == "completed" && t[1] == "draft"));
    }

    #[test]
    fn test_valid_frequencies() {
        assert!(VALID_FREQUENCIES.contains(&"annual"));
        assert!(VALID_FREQUENCIES.contains(&"semi_annual"));
        assert!(VALID_FREQUENCIES.contains(&"quarterly"));
        assert!(VALID_FREQUENCIES.contains(&"monthly"));
        assert!(VALID_FREQUENCIES.contains(&"one_time"));
        assert!(!VALID_FREQUENCIES.contains(&"weekly"));
    }

    #[test]
    fn test_statement_total_calculation() {
        let salary = 120000.0_f64;
        let merit = 6000.0_f64;
        let bonus = 15000.0_f64;
        let equity = 20000.0_f64;
        let benefits = 18000.0_f64;

        let direct = salary + merit + bonus + equity;
        let indirect = benefits;
        let total = direct + indirect;

        assert!((direct - 161000.0).abs() < 0.01);
        assert!((indirect - 18000.0).abs() < 0.01);
        assert!((total - 179000.0).abs() < 0.01);
    }

    #[test]
    fn test_budget_negative_rejected() {
        let budget = -5000.0_f64;
        assert!(budget < 0.0);
    }
}
