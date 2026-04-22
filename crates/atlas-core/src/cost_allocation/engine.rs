//! Cost Allocation Engine
//!
//! Manages cost pools, allocation bases, allocation rules, rule execution,
//! and allocation history. Supports proportional, fixed-percent, and
//! fixed-amount allocation methods.
//!
//! Oracle Fusion Cloud ERP equivalent: Financials > General Ledger > Allocations

use atlas_shared::{
    AllocationPool, AllocationBase, AllocationBaseValue,
    AllocationRule, AllocationRuleTarget,
    AllocationRun, AllocationRunLine,
    AllocationSummary,
    AtlasError, AtlasResult,
};
use super::CostAllocationRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Intermediate result of allocating a single target.
///
/// Replaces a complex 4-tuple to keep clippy happy and improve readability.
pub(crate) struct AllocationTargetResult {
    pub target: AllocationRuleTarget,
    pub amount: f64,
    pub base_value: Option<String>,
    pub percentage: Option<String>,
}

/// Valid pool types
#[allow(dead_code)]
const VALID_POOL_TYPES: &[&str] = &["cost_center", "project", "department", "custom"];

/// Valid base types
#[allow(dead_code)]
const VALID_BASE_TYPES: &[&str] = &["statistical", "financial"];

/// Valid allocation methods
#[allow(dead_code)]
const VALID_ALLOCATION_METHODS: &[&str] = &["proportional", "fixed_percent", "fixed_amount"];

/// Valid rule statuses
#[allow(dead_code)]
const VALID_RULE_STATUSES: &[&str] = &["draft", "active", "inactive"];

/// Valid run statuses
#[allow(dead_code)]
const VALID_RUN_STATUSES: &[&str] = &["draft", "posted", "reversed"];

/// Cost Allocation engine for distributing costs across cost centers
pub struct CostAllocationEngine {
    repository: Arc<dyn CostAllocationRepository>,
}

impl CostAllocationEngine {
    pub fn new(repository: Arc<dyn CostAllocationRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Allocation Pool Management
    // ========================================================================

    /// Create a new allocation pool
    pub async fn create_pool(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        pool_type: &str,
        source_account_codes: serde_json::Value,
        source_department_id: Option<Uuid>,
        source_cost_center: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AllocationPool> {
        if code.is_empty() || name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Pool code and name are required".to_string(),
            ));
        }
        if !VALID_POOL_TYPES.contains(&pool_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid pool type '{}'. Must be one of: {}",
                pool_type, VALID_POOL_TYPES.join(", ")
            )));
        }

        info!("Creating allocation pool '{}' for org {}", code, org_id);

        self.repository.create_pool(
            org_id, code, name, description, pool_type,
            source_account_codes, source_department_id, source_cost_center,
            created_by,
        ).await
    }

    /// Get a pool by code
    pub async fn get_pool(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<AllocationPool>> {
        self.repository.get_pool(org_id, code).await
    }

    /// List all pools
    pub async fn list_pools(&self, org_id: Uuid) -> AtlasResult<Vec<AllocationPool>> {
        self.repository.list_pools(org_id).await
    }

    /// Delete a pool by code
    pub async fn delete_pool(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        self.repository.delete_pool(org_id, code).await
    }

    // ========================================================================
    // Allocation Base Management
    // ========================================================================

    /// Create a new allocation base
    pub async fn create_base(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        base_type: &str,
        financial_account_code: Option<&str>,
        unit_of_measure: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AllocationBase> {
        if code.is_empty() || name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Base code and name are required".to_string(),
            ));
        }
        if !VALID_BASE_TYPES.contains(&base_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid base type '{}'. Must be one of: {}",
                base_type, VALID_BASE_TYPES.join(", ")
            )));
        }

        info!("Creating allocation base '{}' for org {}", code, org_id);

        self.repository.create_base(
            org_id, code, name, description, base_type,
            financial_account_code, unit_of_measure, created_by,
        ).await
    }

    /// Get a base by code
    pub async fn get_base(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<AllocationBase>> {
        self.repository.get_base(org_id, code).await
    }

    /// List all bases
    pub async fn list_bases(&self, org_id: Uuid) -> AtlasResult<Vec<AllocationBase>> {
        self.repository.list_bases(org_id).await
    }

    /// Delete a base by code
    pub async fn delete_base(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        self.repository.delete_base(org_id, code).await
    }

    // ========================================================================
    // Base Value Management
    // ========================================================================

    /// Upsert a statistical base value for a department/cost center
    pub async fn set_base_value(
        &self,
        org_id: Uuid,
        base_code: &str,
        department_id: Option<Uuid>,
        department_name: Option<&str>,
        cost_center: Option<&str>,
        project_id: Option<Uuid>,
        value: &str,
        effective_date: chrono::NaiveDate,
        source: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AllocationBaseValue> {
        let base = self.repository.get_base(org_id, base_code).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Allocation base '{}' not found", base_code)
            ))?;

        let val: f64 = value.parse().map_err(|_| AtlasError::ValidationFailed(
            "Value must be a valid number".to_string(),
        ))?;
        if val < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Base value must be non-negative".to_string(),
            ));
        }

        self.repository.upsert_base_value(
            org_id, base.id, base_code,
            department_id, department_name, cost_center, project_id,
            value, effective_date, source, created_by,
        ).await
    }

    /// Get base values for a given base and effective date
    pub async fn get_base_values(
        &self,
        org_id: Uuid,
        base_id: Uuid,
        effective_date: chrono::NaiveDate,
    ) -> AtlasResult<Vec<AllocationBaseValue>> {
        self.repository.get_base_values(org_id, base_id, effective_date).await
    }

    /// List all base values
    pub async fn list_base_values(
        &self,
        org_id: Uuid,
        base_id: Option<Uuid>,
    ) -> AtlasResult<Vec<AllocationBaseValue>> {
        self.repository.list_base_values(org_id, base_id).await
    }

    // ========================================================================
    // Allocation Rule Management
    // ========================================================================

    /// Create a new allocation rule
    pub async fn create_rule(
        &self,
        org_id: Uuid,
        name: &str,
        description: Option<&str>,
        pool_code: &str,
        base_code: &str,
        allocation_method: &str,
        journal_description: Option<&str>,
        offset_account_code: Option<&str>,
        currency_code: &str,
        is_reversing: bool,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AllocationRule> {
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Rule name is required".to_string(),
            ));
        }
        if !VALID_ALLOCATION_METHODS.contains(&allocation_method) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid allocation method '{}'. Must be one of: {}",
                allocation_method, VALID_ALLOCATION_METHODS.join(", ")
            )));
        }

        // Validate pool exists
        let pool = self.repository.get_pool(org_id, pool_code).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Allocation pool '{}' not found", pool_code)
            ))?;

        // Validate base exists
        let base = self.repository.get_base(org_id, base_code).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Allocation base '{}' not found", base_code)
            ))?;

        let rule_number = format!("ALLOC-{}", Uuid::new_v4().to_string()[..8].to_uppercase());

        info!("Creating allocation rule '{}' ({}) for org {}", name, rule_number, org_id);

        self.repository.create_rule(
            org_id, &rule_number, name, description,
            pool.id, pool_code, base.id, base_code,
            allocation_method, journal_description, offset_account_code,
            currency_code, is_reversing, created_by,
        ).await
    }

    /// Get a rule by ID
    pub async fn get_rule(&self, id: Uuid) -> AtlasResult<Option<AllocationRule>> {
        self.repository.get_rule(id).await
    }

    /// List rules with optional status filter
    pub async fn list_rules(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<AllocationRule>> {
        if let Some(s) = status {
            if !VALID_RULE_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_RULE_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_rules(org_id, status).await
    }

    /// Activate a draft rule
    pub async fn activate_rule(&self, rule_id: Uuid) -> AtlasResult<AllocationRule> {
        let rule = self.repository.get_rule(rule_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Allocation rule {} not found", rule_id)
            ))?;

        if rule.status != "draft" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot activate rule in '{}' status. Must be 'draft'.", rule.status)
            ));
        }

        // Validate that the rule has at least one target
        let targets = self.repository.list_rule_targets(rule_id).await?;
        if targets.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Cannot activate rule without target lines. Add at least one target.".to_string(),
            ));
        }

        info!("Activating allocation rule {}", rule.rule_number);
        self.repository.update_rule_status(rule_id, "active").await
    }

    /// Deactivate an active rule
    pub async fn deactivate_rule(&self, rule_id: Uuid) -> AtlasResult<AllocationRule> {
        let rule = self.repository.get_rule(rule_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Allocation rule {} not found", rule_id)
            ))?;

        if rule.status != "active" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot deactivate rule in '{}' status. Must be 'active'.", rule.status)
            ));
        }

        self.repository.update_rule_status(rule_id, "inactive").await
    }

    // ========================================================================
    // Rule Target Management
    // ========================================================================

    /// Add a target line to a rule
    pub async fn add_rule_target(
        &self,
        org_id: Uuid,
        rule_id: Uuid,
        department_id: Option<Uuid>,
        department_name: Option<&str>,
        cost_center: Option<&str>,
        project_id: Option<Uuid>,
        project_name: Option<&str>,
        target_account_code: &str,
        fixed_percent: Option<&str>,
        fixed_amount: Option<&str>,
    ) -> AtlasResult<AllocationRuleTarget> {
        let rule = self.repository.get_rule(rule_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Allocation rule {} not found", rule_id)
            ))?;

        if target_account_code.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Target account code is required".to_string(),
            ));
        }

        // Validate fixed_percent sums for fixed_percent method
        if rule.allocation_method == "fixed_percent" {
            if let Some(pct_str) = fixed_percent {
                let pct: f64 = pct_str.parse().map_err(|_| AtlasError::ValidationFailed(
                    "Fixed percent must be a valid number".to_string(),
                ))?;
                if pct <= 0.0 || pct > 100.0 {
                    return Err(AtlasError::ValidationFailed(
                        "Fixed percent must be between 0 and 100".to_string(),
                    ));
                }
            } else {
                return Err(AtlasError::ValidationFailed(
                    "Fixed percent is required for 'fixed_percent' allocation method".to_string(),
                ));
            }
        }

        // Get next line number
        let existing_targets = self.repository.list_rule_targets(rule_id).await?;
        let line_number = (existing_targets.len() as i32) + 1;

        self.repository.create_rule_target(
            org_id, rule_id, line_number,
            department_id, department_name, cost_center,
            project_id, project_name,
            target_account_code, fixed_percent, fixed_amount,
        ).await
    }

    /// List targets for a rule
    pub async fn list_rule_targets(&self, rule_id: Uuid) -> AtlasResult<Vec<AllocationRuleTarget>> {
        self.repository.list_rule_targets(rule_id).await
    }

    // ========================================================================
    // Rule Execution (Run Allocation)
    // ========================================================================

    /// Execute an allocation rule, generating debit/credit lines
    pub async fn execute_rule(
        &self,
        org_id: Uuid,
        rule_id: Uuid,
        source_amount: &str,
        period_start: chrono::NaiveDate,
        period_end: chrono::NaiveDate,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AllocationRun> {
        let rule = self.repository.get_rule(rule_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Allocation rule {} not found", rule_id)
            ))?;

        if rule.status != "active" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot execute rule in '{}' status. Must be 'active'.", rule.status)
            ));
        }

        let total_amount: f64 = source_amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "Source amount must be a valid number".to_string(),
        ))?;
        if total_amount <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Source amount must be positive".to_string(),
            ));
        }

        let targets = self.repository.list_rule_targets(rule_id).await?;
        if targets.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Rule has no target lines".to_string(),
            ));
        }

        let run_number = format!("ARUN-{}", Uuid::new_v4().to_string()[..8].to_uppercase());
        let run_date = chrono::Utc::now().date_naive();

        // Calculate allocations based on method
        let allocations = match rule.allocation_method.as_str() {
            "proportional" => {
                self.calculate_proportional_allocation(
                    org_id, &rule, &targets, total_amount, period_start,
                ).await?
            }
            "fixed_percent" => {
                self.calculate_fixed_percent_allocation(&targets, total_amount)?
            }
            "fixed_amount" => {
                self.calculate_fixed_amount_allocation(&targets, total_amount)?
            }
            _ => {
                return Err(AtlasError::ValidationFailed(
                    format!("Unsupported allocation method: {}", rule.allocation_method)
                ));
            }
        };

        let total_allocated: f64 = allocations.iter().map(|a| a.amount).sum();
        let line_count = (allocations.len() as i32) + 1; // +1 for the offset credit line

        info!("Executing allocation rule {} for amount {:.2}: {} target lines",
              rule.rule_number, total_amount, allocations.len());

        // Create run header
        let run = self.repository.create_run(
            org_id, &run_number, rule_id, &rule.name, &rule.rule_number,
            period_start, period_end,
            &format!("{:.2}", total_amount),
            &format!("{:.2}", total_allocated),
            line_count, run_date, created_by,
        ).await?;

        // Create debit lines for each target
        for (idx, alloc) in allocations.iter().enumerate() {
            let desc = rule.journal_description.as_deref().unwrap_or(&rule.name);
            self.repository.create_run_line(
                org_id, run.id, (idx as i32) + 1,
                "debit", &alloc.target.target_account_code,
                alloc.target.department_id, alloc.target.department_name.as_deref(),
                alloc.target.cost_center.as_deref(), alloc.target.project_id,
                &format!("{:.2}", alloc.amount),
                alloc.base_value.as_deref(), alloc.percentage.as_deref(),
                Some(desc),
            ).await?;
        }

        // Create offset credit line
        let offset_account = rule.offset_account_code.as_deref().unwrap_or("9999");
        self.repository.create_run_line(
            org_id, run.id, line_count,
            "credit", offset_account,
            None, None, None, None,
            &format!("{:.2}", total_allocated),
            None, None,
            Some(&format!("Offset for {}", rule.name)),
        ).await?;

        Ok(run)
    }

    /// Calculate proportional allocation based on statistical/financial base values
    async fn calculate_proportional_allocation(
        &self,
        org_id: Uuid,
        rule: &AllocationRule,
        targets: &[AllocationRuleTarget],
        total_amount: f64,
        effective_date: chrono::NaiveDate,
    ) -> AtlasResult<Vec<AllocationTargetResult>> {
        // Get base values
        let base_values = self.repository.get_base_values(
            org_id, rule.base_id, effective_date,
        ).await?;

        if base_values.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "No base values found for the allocation base. Enter base values before running.".to_string(),
            ));
        }

        // Build lookup: department_id -> value
        let mut value_map: std::collections::HashMap<Option<Uuid>, f64> = std::collections::HashMap::new();
        let mut total_base: f64 = 0.0;
        for bv in &base_values {
            let val: f64 = bv.value.parse().unwrap_or(0.0);
            value_map.insert(bv.department_id, val);
            total_base += val;
        }

        if total_base == 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Total base value is zero. Cannot allocate.".to_string(),
            ));
        }

        let mut allocations = Vec::new();
        for target in targets {
            let base_val = value_map.get(&target.department_id).unwrap_or(&0.0);
            let pct = base_val / total_base * 100.0;
            let allocated = total_amount * (base_val / total_base);
            allocations.push(AllocationTargetResult {
                target: target.clone(),
                amount: allocated,
                base_value: Some(format!("{:.2}", base_val)),
                percentage: Some(format!("{:.4}", pct)),
            });
        }

        Ok(allocations)
    }

    /// Calculate fixed percent allocation
    fn calculate_fixed_percent_allocation(
        &self,
        targets: &[AllocationRuleTarget],
        total_amount: f64,
    ) -> AtlasResult<Vec<AllocationTargetResult>> {
        let mut allocations = Vec::new();
        for target in targets {
            let pct_str = target.fixed_percent.as_deref().unwrap_or("0");
            let pct: f64 = pct_str.parse().unwrap_or(0.0);
            let allocated = total_amount * (pct / 100.0);
            allocations.push(AllocationTargetResult {
                target: target.clone(),
                amount: allocated,
                base_value: None,
                percentage: Some(format!("{:.4}", pct)),
            });
        }
        Ok(allocations)
    }

    /// Calculate fixed amount allocation
    fn calculate_fixed_amount_allocation(
        &self,
        targets: &[AllocationRuleTarget],
        _total_amount: f64,
    ) -> AtlasResult<Vec<AllocationTargetResult>> {
        let mut allocations = Vec::new();
        for target in targets {
            let amt_str = target.fixed_amount.as_deref().unwrap_or("0");
            let amt: f64 = amt_str.parse().unwrap_or(0.0);
            allocations.push(AllocationTargetResult {
                target: target.clone(),
                amount: amt,
                base_value: None,
                percentage: None,
            });
        }
        Ok(allocations)
    }

    // ========================================================================
    // Run Management
    // ========================================================================

    /// Get a run by ID
    pub async fn get_run(&self, id: Uuid) -> AtlasResult<Option<AllocationRun>> {
        self.repository.get_run(id).await
    }

    /// List runs with optional rule filter
    pub async fn list_runs(&self, org_id: Uuid, rule_id: Option<Uuid>) -> AtlasResult<Vec<AllocationRun>> {
        self.repository.list_runs(org_id, rule_id).await
    }

    /// Get run lines
    pub async fn list_run_lines(&self, run_id: Uuid) -> AtlasResult<Vec<AllocationRunLine>> {
        self.repository.list_run_lines(run_id).await
    }

    /// Post a draft run (marks as posted)
    pub async fn post_run(&self, run_id: Uuid, posted_by: Option<Uuid>) -> AtlasResult<AllocationRun> {
        let run = self.repository.get_run(run_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Allocation run {} not found", run_id)
            ))?;

        if run.status != "draft" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot post run in '{}' status. Must be 'draft'.", run.status)
            ));
        }

        info!("Posting allocation run {}", run.run_number);
        self.repository.update_run_status(run_id, "posted", posted_by, None, None).await
    }

    /// Reverse a posted run
    pub async fn reverse_run(
        &self,
        run_id: Uuid,
        reversed_by: Option<Uuid>,
        reason: &str,
    ) -> AtlasResult<AllocationRun> {
        let run = self.repository.get_run(run_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Allocation run {} not found", run_id)
            ))?;

        if run.status != "posted" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot reverse run in '{}' status. Must be 'posted'.", run.status)
            ));
        }

        if reason.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Reversal reason is required".to_string(),
            ));
        }

        info!("Reversing allocation run {}: {}", run.run_number, reason);
        self.repository.update_run_status(run_id, "reversed", None, reversed_by, Some(reason)).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get allocation summary for dashboard
    pub async fn get_summary(&self, org_id: Uuid) -> AtlasResult<AllocationSummary> {
        self.repository.get_summary(org_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_pool_types() {
        assert!(VALID_POOL_TYPES.contains(&"cost_center"));
        assert!(VALID_POOL_TYPES.contains(&"project"));
        assert!(VALID_POOL_TYPES.contains(&"department"));
        assert!(VALID_POOL_TYPES.contains(&"custom"));
    }

    #[test]
    fn test_valid_base_types() {
        assert!(VALID_BASE_TYPES.contains(&"statistical"));
        assert!(VALID_BASE_TYPES.contains(&"financial"));
    }

    #[test]
    fn test_valid_allocation_methods() {
        assert!(VALID_ALLOCATION_METHODS.contains(&"proportional"));
        assert!(VALID_ALLOCATION_METHODS.contains(&"fixed_percent"));
        assert!(VALID_ALLOCATION_METHODS.contains(&"fixed_amount"));
    }

    #[test]
    fn test_valid_rule_statuses() {
        assert!(VALID_RULE_STATUSES.contains(&"draft"));
        assert!(VALID_RULE_STATUSES.contains(&"active"));
        assert!(VALID_RULE_STATUSES.contains(&"inactive"));
    }

    #[test]
    fn test_valid_run_statuses() {
        assert!(VALID_RUN_STATUSES.contains(&"draft"));
        assert!(VALID_RUN_STATUSES.contains(&"posted"));
        assert!(VALID_RUN_STATUSES.contains(&"reversed"));
    }
}
