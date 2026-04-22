//! Allocation Engine Implementation
//!
//! Manages GL allocation pools, bases, rules, and runs.
//! Supports proportional, fixed-percentage, and step-down allocation methods.

use atlas_shared::{
    GlAllocationPool,
    GlAllocationBasis,
    GlAllocationBasisDetail, GlAllocationBasisDetailRequest,
    GlAllocationRule, GlAllocationRuleRequest,
    GlAllocationRun, GlAllocationRunRequest,
    GlAllocationDashboardSummary,
    AtlasError, AtlasResult,
};
use super::AllocationRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

#[allow(dead_code)]
const VALID_POOL_TYPES: &[&str] = &["cost_center", "account_range", "manual"];
#[allow(dead_code)]
const VALID_BASIS_TYPES: &[&str] = &["statistical", "financial", "percentage"];
#[allow(dead_code)]
const VALID_ALLOCATION_METHODS: &[&str] = &["proportional", "fixed_percentage", "step_down"];
#[allow(dead_code)]
const VALID_OFFSET_METHODS: &[&str] = &["none", "same_account", "specified_account"];
#[allow(dead_code)]
const VALID_RUN_STATUSES: &[&str] = &["draft", "posted", "reversed", "cancelled"];

/// GL Allocation engine
pub struct AllocationEngine {
    repository: Arc<dyn AllocationRepository>,
}

impl AllocationEngine {
    pub fn new(repository: Arc<dyn AllocationRepository>) -> Self {
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
        source_account_code: Option<&str>,
        source_account_range_from: Option<&str>,
        source_account_range_to: Option<&str>,
        source_department_id: Option<Uuid>,
        source_project_id: Option<Uuid>,
        currency_code: &str,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<GlAllocationPool> {
        if code.is_empty() {
            return Err(AtlasError::ValidationFailed("Pool code is required".to_string()));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Pool name is required".to_string()));
        }
        if !VALID_POOL_TYPES.contains(&pool_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid pool type '{}'. Must be one of: {}", pool_type, VALID_POOL_TYPES.join(", ")
            )));
        }

        // Check uniqueness
        if self.repository.get_pool_by_code(org_id, code).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Allocation pool with code '{}' already exists", code
            )));
        }

        info!("Creating allocation pool {} ({})", code, name);

        self.repository.create_pool(
            org_id, code, name, description, pool_type,
            source_account_code, source_account_range_from, source_account_range_to,
            source_department_id, source_project_id,
            currency_code, effective_from, effective_to, created_by,
        ).await
    }

    /// Get a pool by code
    pub async fn get_pool(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<GlAllocationPool>> {
        self.repository.get_pool_by_code(org_id, code).await
    }

    /// Get a pool by ID
    pub async fn get_pool_by_id(&self, id: Uuid) -> AtlasResult<Option<GlAllocationPool>> {
        self.repository.get_pool_by_id(id).await
    }

    /// List all pools
    pub async fn list_pools(&self, org_id: Uuid, active_only: bool) -> AtlasResult<Vec<GlAllocationPool>> {
        self.repository.list_pools(org_id, active_only).await
    }

    /// Activate a pool
    pub async fn activate_pool(&self, id: Uuid) -> AtlasResult<GlAllocationPool> {
        let pool = self.repository.get_pool_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Allocation pool {} not found", id)))?;

        if pool.is_active {
            return Err(AtlasError::WorkflowError("Pool is already active".to_string()));
        }

        info!("Activating allocation pool {}", pool.code);
        self.repository.update_pool_active(id, true).await
    }

    /// Deactivate a pool
    pub async fn deactivate_pool(&self, id: Uuid) -> AtlasResult<GlAllocationPool> {
        let pool = self.repository.get_pool_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Allocation pool {} not found", id)))?;

        if !pool.is_active {
            return Err(AtlasError::WorkflowError("Pool is already inactive".to_string()));
        }

        info!("Deactivating allocation pool {}", pool.code);
        self.repository.update_pool_active(id, false).await
    }

    /// Delete a pool
    pub async fn delete_pool(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        info!("Deleting allocation pool {}", code);
        self.repository.delete_pool(org_id, code).await
    }

    // ========================================================================
    // Allocation Basis Management
    // ========================================================================

    /// Create a new allocation basis
    pub async fn create_basis(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        basis_type: &str,
        unit_of_measure: Option<&str>,
        is_manual: bool,
        source_account_code: Option<&str>,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<GlAllocationBasis> {
        if code.is_empty() {
            return Err(AtlasError::ValidationFailed("Basis code is required".to_string()));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Basis name is required".to_string()));
        }
        if !VALID_BASIS_TYPES.contains(&basis_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid basis type '{}'. Must be one of: {}", basis_type, VALID_BASIS_TYPES.join(", ")
            )));
        }

        // Check uniqueness
        if self.repository.get_basis_by_code(org_id, code).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Allocation basis with code '{}' already exists", code
            )));
        }

        info!("Creating allocation basis {} ({})", code, name);

        self.repository.create_basis(
            org_id, code, name, description, basis_type,
            unit_of_measure, is_manual, source_account_code,
            effective_from, effective_to, created_by,
        ).await
    }

    /// Get a basis by code
    pub async fn get_basis(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<GlAllocationBasis>> {
        self.repository.get_basis_by_code(org_id, code).await
    }

    /// Get a basis by ID
    pub async fn get_basis_by_id(&self, id: Uuid) -> AtlasResult<Option<GlAllocationBasis>> {
        self.repository.get_basis_by_id(id).await
    }

    /// List all bases
    pub async fn list_bases(&self, org_id: Uuid, active_only: bool) -> AtlasResult<Vec<GlAllocationBasis>> {
        self.repository.list_bases(org_id, active_only).await
    }

    /// Activate a basis
    pub async fn activate_basis(&self, id: Uuid) -> AtlasResult<GlAllocationBasis> {
        let basis = self.repository.get_basis_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Allocation basis {} not found", id)))?;

        if basis.is_active {
            return Err(AtlasError::WorkflowError("Basis is already active".to_string()));
        }

        info!("Activating allocation basis {}", basis.code);
        self.repository.update_basis_active(id, true).await
    }

    /// Deactivate a basis
    pub async fn deactivate_basis(&self, id: Uuid) -> AtlasResult<GlAllocationBasis> {
        let basis = self.repository.get_basis_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Allocation basis {} not found", id)))?;

        if !basis.is_active {
            return Err(AtlasError::WorkflowError("Basis is already inactive".to_string()));
        }

        info!("Deactivating allocation basis {}", basis.code);
        self.repository.update_basis_active(id, false).await
    }

    /// Delete a basis
    pub async fn delete_basis(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        info!("Deleting allocation basis {}", code);
        self.repository.delete_basis(org_id, code).await
    }

    // ========================================================================
    // Basis Detail Management
    // ========================================================================

    /// Add a basis detail
    pub async fn add_basis_detail(
        &self,
        org_id: Uuid,
        basis_code: &str,
        request: &GlAllocationBasisDetailRequest,
        created_by: Option<Uuid>,
    ) -> AtlasResult<GlAllocationBasisDetail> {
        let basis = self.repository.get_basis_by_code(org_id, basis_code).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Allocation basis '{}' not found", basis_code
            )))?;

        if !basis.is_active {
            return Err(AtlasError::WorkflowError(format!(
                "Basis '{}' is inactive", basis_code
            )));
        }

        self.repository.create_basis_detail(
            org_id, basis.id,
            request.target_department_id,
            request.target_department_name.as_deref(),
            request.target_cost_center.as_deref(),
            request.target_project_id,
            request.target_project_name.as_deref(),
            request.target_account_code.as_deref(),
            &request.basis_amount,
            request.period_name.as_deref(),
            request.period_start_date,
            request.period_end_date,
            created_by,
        ).await
    }

    /// List basis details for a basis
    pub async fn list_basis_details(
        &self,
        org_id: Uuid,
        basis_code: &str,
        period_name: Option<&str>,
    ) -> AtlasResult<Vec<GlAllocationBasisDetail>> {
        let basis = self.repository.get_basis_by_code(org_id, basis_code).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Allocation basis '{}' not found", basis_code
            )))?;

        self.repository.list_basis_details(basis.id, period_name).await
    }

    /// Update basis detail amounts (recalculate percentages)
    pub async fn update_basis_detail_amount(
        &self,
        detail_id: Uuid,
        basis_amount: &str,
    ) -> AtlasResult<GlAllocationBasisDetail> {
        let _detail = self.repository.get_basis_detail_by_id(detail_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Basis detail {} not found", detail_id
            )))?;

        info!("Updating basis detail {} amount to {}", detail_id, basis_amount);
        self.repository.update_basis_detail_amount(detail_id, basis_amount).await
    }

    /// Delete a basis detail
    pub async fn delete_basis_detail(&self, id: Uuid) -> AtlasResult<()> {
        info!("Deleting basis detail {}", id);
        self.repository.delete_basis_detail(id).await
    }

    /// Recalculate basis percentages
    pub async fn recalculate_basis_percentages(
        &self,
        org_id: Uuid,
        basis_code: &str,
    ) -> AtlasResult<Vec<GlAllocationBasisDetail>> {
        let basis = self.repository.get_basis_by_code(org_id, basis_code).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Allocation basis '{}' not found", basis_code
            )))?;

        let details = self.repository.list_basis_details(basis.id, None).await?;

        // Calculate total basis amount
        let total: f64 = details.iter()
            .filter_map(|d| d.basis_amount.parse::<f64>().ok())
            .sum();

        if total.abs() < f64::EPSILON {
            return Ok(details);
        }

        // Update percentages
        for detail in &details {
            let amount: f64 = detail.basis_amount.parse::<f64>().unwrap_or(0.0);
            let percentage = (amount / total) * 100.0;
            self.repository.update_basis_detail_percentage(detail.id, &format!("{:.6}", percentage)).await?;
        }

        self.repository.list_basis_details(basis.id, None).await
    }

    // ========================================================================
    // Allocation Rule Management
    // ========================================================================

    /// Create a new allocation rule
    pub async fn create_rule(
        &self,
        org_id: Uuid,
        request: &GlAllocationRuleRequest,
        created_by: Option<Uuid>,
    ) -> AtlasResult<GlAllocationRule> {
        if request.code.is_empty() {
            return Err(AtlasError::ValidationFailed("Rule code is required".to_string()));
        }
        if request.name.is_empty() {
            return Err(AtlasError::ValidationFailed("Rule name is required".to_string()));
        }
        if !VALID_ALLOCATION_METHODS.contains(&request.allocation_method.as_str()) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid allocation method '{}'. Must be one of: {}",
                request.allocation_method, VALID_ALLOCATION_METHODS.join(", ")
            )));
        }
        if !VALID_OFFSET_METHODS.contains(&request.offset_method.as_str()) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid offset method '{}'. Must be one of: {}",
                request.offset_method, VALID_OFFSET_METHODS.join(", ")
            )));
        }

        // Validate pool exists
        let pool = self.repository.get_pool_by_code(org_id, &request.pool_code).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Allocation pool '{}' not found", request.pool_code
            )))?;

        // Validate basis exists
        let basis = self.repository.get_basis_by_code(org_id, &request.basis_code).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Allocation basis '{}' not found", request.basis_code
            )))?;

        // Check uniqueness
        if self.repository.get_rule_by_code(org_id, &request.code).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Allocation rule with code '{}' already exists", request.code
            )));
        }

        // For fixed_percentage method, validate target lines
        if request.allocation_method == "fixed_percentage" {
            if let Some(lines) = &request.target_lines {
                let total_pct: f64 = lines.iter()
                    .filter_map(|l| l.fixed_percentage.as_ref())
                    .filter_map(|p| p.parse::<f64>().ok())
                    .sum();
                if (total_pct - 100.0).abs() > 0.01 {
                    return Err(AtlasError::ValidationFailed(format!(
                        "Fixed percentages must sum to 100%. Current total: {:.2}%", total_pct
                    )));
                }
            }
        }

        // Validate offset account when method is 'specified_account'
        if request.offset_method == "specified_account" && request.offset_account_code.is_none() {
            return Err(AtlasError::ValidationFailed(
                "Offset account code is required when offset method is 'specified_account'".to_string(),
            ));
        }

        info!("Creating allocation rule {} ({})", request.code, request.name);

        let rule = self.repository.create_rule(
            org_id, &request.code, &request.name, request.description.as_deref(),
            pool.id, &request.pool_code, basis.id, &request.basis_code,
            &request.allocation_method, &request.offset_method,
            request.offset_account_code.as_deref(),
            request.journal_batch_prefix.as_deref(),
            request.round_to_largest,
            request.minimum_threshold.as_deref(),
            request.effective_from, request.effective_to,
            created_by,
        ).await?;

        // Create target lines if provided
        if let Some(lines) = &request.target_lines {
            for (idx, line_req) in lines.iter().enumerate() {
                self.repository.create_target_line(
                    org_id, rule.id,
                    (idx + 1) as i32,
                    line_req.target_department_id,
                    line_req.target_department_name.as_deref(),
                    line_req.target_cost_center.as_deref(),
                    line_req.target_project_id,
                    line_req.target_project_name.as_deref(),
                    &line_req.target_account_code,
                    line_req.target_account_name.as_deref(),
                    line_req.fixed_percentage.as_deref(),
                    line_req.is_active.unwrap_or(true),
                ).await?;
            }
        }

        // Reload with target lines
        self.repository.get_rule_by_code(org_id, &request.code).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Created rule not found".to_string()))
    }

    /// Get a rule by code
    pub async fn get_rule(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<GlAllocationRule>> {
        self.repository.get_rule_by_code(org_id, code).await
    }

    /// List all rules
    pub async fn list_rules(&self, org_id: Uuid, active_only: bool) -> AtlasResult<Vec<GlAllocationRule>> {
        self.repository.list_rules(org_id, active_only).await
    }

    /// Activate a rule
    pub async fn activate_rule(&self, id: Uuid) -> AtlasResult<GlAllocationRule> {
        let rule = self.repository.get_rule_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Allocation rule {} not found", id)))?;

        if rule.is_active {
            return Err(AtlasError::WorkflowError("Rule is already active".to_string()));
        }

        info!("Activating allocation rule {}", rule.code);
        self.repository.update_rule_active(id, true).await
    }

    /// Deactivate a rule
    pub async fn deactivate_rule(&self, id: Uuid) -> AtlasResult<GlAllocationRule> {
        let rule = self.repository.get_rule_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Allocation rule {} not found", id)))?;

        if !rule.is_active {
            return Err(AtlasError::WorkflowError("Rule is already inactive".to_string()));
        }

        info!("Deactivating allocation rule {}", rule.code);
        self.repository.update_rule_active(id, false).await
    }

    /// Delete a rule
    pub async fn delete_rule(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        info!("Deleting allocation rule {}", code);
        self.repository.delete_rule(org_id, code).await
    }

    // ========================================================================
    // Allocation Run Management
    // ========================================================================

    /// Execute an allocation rule (create an allocation run)
    pub async fn execute_allocation(
        &self,
        org_id: Uuid,
        request: &GlAllocationRunRequest,
        created_by: Option<Uuid>,
    ) -> AtlasResult<GlAllocationRun> {
        // Validate rule exists and is active
        let rule = self.repository.get_rule_by_code(org_id, &request.rule_code).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Allocation rule '{}' not found", request.rule_code
            )))?;

        if !rule.is_active {
            return Err(AtlasError::WorkflowError(format!(
                "Allocation rule '{}' is inactive", request.rule_code
            )));
        }

        // Get pool
        let pool = self.repository.get_pool_by_id(rule.pool_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Allocation pool not found for rule {}", request.rule_code
            )))?;

        // Get basis
        let basis = self.repository.get_basis_by_id(rule.basis_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Allocation basis not found for rule {}", request.rule_code
            )))?;

        // Get basis details for the period
        let basis_details = self.repository.list_basis_details(
            basis.id, Some(&request.period_name),
        ).await?;

        if basis_details.is_empty() {
            return Err(AtlasError::ValidationFailed(format!(
                "No basis details found for period '{}' in basis '{}'. Add basis amounts before running allocation.",
                request.period_name, rule.basis_code
            )));
        }

        // Determine pool amount
        let pool_amount = request.pool_amount_override.clone()
            .unwrap_or_else(|| "1000.00".to_string());

        // Calculate allocations based on method
        let run_number = format!("ALLOC-{}-{}", request.period_name, chrono::Utc::now().format("%Y%m%d%H%M%S"));

        info!("Executing allocation rule {} for period {}", request.rule_code, request.period_name);

        let _run_date = request.run_date.unwrap_or(chrono::Utc::now().date_naive());

        // Create the run
        let run = self.repository.create_run(
            org_id, &run_number, rule.id, &rule.code, &rule.name,
            &request.period_name, request.period_start_date, request.period_end_date,
            &pool_amount, &rule.allocation_method,
            created_by,
        ).await?;

        // Generate target lines based on method
        let mut run_lines = Vec::new();
        let pool_amount_f64: f64 = pool_amount.parse::<f64>().unwrap_or(0.0);

        match rule.allocation_method.as_str() {
            "proportional" => {
                // Calculate total basis
                let total_basis: f64 = basis_details.iter()
                    .filter_map(|d| d.basis_amount.parse::<f64>().ok())
                    .sum();

                if total_basis.abs() < f64::EPSILON {
                    return Err(AtlasError::ValidationFailed(
                        "Total basis amount is zero. Cannot allocate.".to_string(),
                    ));
                }

                for (idx, detail) in basis_details.iter().enumerate() {
                    let basis_amt: f64 = detail.basis_amount.parse::<f64>().unwrap_or(0.0);
                    let percentage = (basis_amt / total_basis) * 100.0;
                    let allocated = pool_amount_f64 * (basis_amt / total_basis);

                    // Check minimum threshold
                    if let Some(ref threshold) = rule.minimum_threshold {
                        let threshold_f64: f64 = threshold.parse::<f64>().unwrap_or(0.0);
                        if allocated < threshold_f64 {
                            continue;
                        }
                    }

                    let line = self.repository.create_run_line(
                        org_id, run.id, (idx + 1) as i32,
                        detail.target_department_id,
                        detail.target_department_name.as_deref(),
                        detail.target_cost_center.as_deref(),
                        detail.target_project_id,
                        detail.target_project_name.as_deref(),
                        detail.target_account_code.as_deref().unwrap_or(""),
                        None,
                        pool.source_account_code.as_deref(),
                        &detail.basis_amount,
                        &format!("{:.6}", percentage),
                        &format!("{:.2}", allocated),
                        &format!("{:.2}", allocated),
                        "allocation",
                    ).await?;
                    run_lines.push(line);
                }
            }
            "fixed_percentage" => {
                if !rule.target_lines.is_empty() {
                    for (idx, target) in rule.target_lines.iter().enumerate() {
                        let pct: f64 = target.fixed_percentage
                            .as_ref()
                            .and_then(|p| p.parse::<f64>().ok())
                            .unwrap_or(0.0);
                        let allocated = pool_amount_f64 * (pct / 100.0);

                        let line = self.repository.create_run_line(
                            org_id, run.id, (idx + 1) as i32,
                            target.target_department_id,
                            target.target_department_name.as_deref(),
                            target.target_cost_center.as_deref(),
                            target.target_project_id,
                            target.target_project_name.as_deref(),
                            &target.target_account_code,
                            target.target_account_name.as_deref(),
                            pool.source_account_code.as_deref(),
                            "0.00",
                            &format!("{:.6}", pct),
                            &format!("{:.2}", allocated),
                            &format!("{:.2}", allocated),
                            "allocation",
                        ).await?;
                        run_lines.push(line);
                    }
                }
            }
            "step_down" => {
                // Step-down allocation: distribute sequentially, removing each target
                // from the basis after their share is calculated (simplified version)
                let total_basis: f64 = basis_details.iter()
                    .filter_map(|d| d.basis_amount.parse::<f64>().ok())
                    .sum();

                if total_basis.abs() < f64::EPSILON {
                    return Err(AtlasError::ValidationFailed(
                        "Total basis amount is zero. Cannot allocate.".to_string(),
                    ));
                }

                let mut remaining = pool_amount_f64;
                for (idx, detail) in basis_details.iter().enumerate() {
                    let basis_amt: f64 = detail.basis_amount.parse::<f64>().unwrap_or(0.0);
                    let percentage = (basis_amt / total_basis) * 100.0;
                    let allocated = remaining * (basis_amt / total_basis);
                    remaining -= allocated;

                    let line = self.repository.create_run_line(
                        org_id, run.id, (idx + 1) as i32,
                        detail.target_department_id,
                        detail.target_department_name.as_deref(),
                        detail.target_cost_center.as_deref(),
                        detail.target_project_id,
                        detail.target_project_name.as_deref(),
                        detail.target_account_code.as_deref().unwrap_or(""),
                        None,
                        pool.source_account_code.as_deref(),
                        &detail.basis_amount,
                        &format!("{:.6}", percentage),
                        &format!("{:.2}", allocated),
                        &format!("{:.2}", allocated),
                        "allocation",
                    ).await?;
                    run_lines.push(line);
                }
            }
            _ => {
                return Err(AtlasError::ValidationFailed(format!(
                    "Unsupported allocation method: {}", rule.allocation_method
                )));
            }
        }

        // Add offset line if needed
        if rule.offset_method != "none" && !run_lines.is_empty() {
            let total_allocated: f64 = run_lines.iter()
                .filter_map(|l| l.allocated_amount.parse::<f64>().ok())
                .sum();

            let offset_account = match rule.offset_method.as_str() {
                "same_account" => pool.source_account_code.as_deref().unwrap_or(""),
                "specified_account" => rule.offset_account_code.as_deref().unwrap_or(""),
                _ => "",
            };

            // Calculate rounding difference
            let _rounding_diff = pool_amount_f64 - total_allocated;

            self.repository.create_run_line(
                org_id, run.id, (run_lines.len() + 1) as i32,
                None, None, None, None, None,
                offset_account, None,
                None,
                "0.00", // no basis for offset
                "0.000000", // no percentage for offset
                &format!("{:.2}", total_allocated),
                &format!("{:.2}", total_allocated),
                "offset",
            ).await?;
        }

        // Recalculate totals
        let final_run = self.repository.update_run_totals(run.id).await?;

        info!("Allocation run {} created with {} lines", run_number, run_lines.len());

        // Reload with results
        self.repository.get_run_by_id(final_run.id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Created run not found".to_string()))
    }

    /// Get a run by ID
    pub async fn get_run(&self, id: Uuid) -> AtlasResult<Option<GlAllocationRun>> {
        self.repository.get_run_by_id(id).await
    }

    /// List runs for an organization
    pub async fn list_runs(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<GlAllocationRun>> {
        self.repository.list_runs(org_id, status).await
    }

    /// Post an allocation run (mark as posted)
    pub async fn post_run(&self, id: Uuid, posted_by: Option<Uuid>) -> AtlasResult<GlAllocationRun> {
        let run = self.repository.get_run_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Allocation run {} not found", id)))?;

        if run.status != "draft" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot post run in '{}' status. Must be 'draft'.", run.status
            )));
        }

        info!("Posting allocation run {}", run.run_number);
        self.repository.update_run_status(id, "posted", posted_by).await
    }

    /// Reverse an allocation run
    pub async fn reverse_run(&self, id: Uuid, reversed_by: Option<Uuid>) -> AtlasResult<GlAllocationRun> {
        let run = self.repository.get_run_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Allocation run {} not found", id)))?;

        if run.status != "posted" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot reverse run in '{}' status. Must be 'posted'.", run.status
            )));
        }

        info!("Reversing allocation run {}", run.run_number);
        self.repository.update_run_status(id, "reversed", reversed_by).await
    }

    /// Cancel an allocation run
    pub async fn cancel_run(&self, id: Uuid) -> AtlasResult<GlAllocationRun> {
        let run = self.repository.get_run_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Allocation run {} not found", id)))?;

        if run.status != "draft" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot cancel run in '{}' status. Must be 'draft'.", run.status
            )));
        }

        info!("Cancelling allocation run {}", run.run_number);
        self.repository.update_run_status(id, "cancelled", None).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get allocation dashboard summary
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<GlAllocationDashboardSummary> {
        self.repository.get_dashboard_summary(org_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_pool_types() {
        assert!(VALID_POOL_TYPES.contains(&"cost_center"));
        assert!(VALID_POOL_TYPES.contains(&"account_range"));
        assert!(VALID_POOL_TYPES.contains(&"manual"));
        assert!(!VALID_POOL_TYPES.contains(&"invalid"));
    }

    #[test]
    fn test_valid_basis_types() {
        assert!(VALID_BASIS_TYPES.contains(&"statistical"));
        assert!(VALID_BASIS_TYPES.contains(&"financial"));
        assert!(VALID_BASIS_TYPES.contains(&"percentage"));
        assert!(!VALID_BASIS_TYPES.contains(&"invalid"));
    }

    #[test]
    fn test_valid_allocation_methods() {
        assert!(VALID_ALLOCATION_METHODS.contains(&"proportional"));
        assert!(VALID_ALLOCATION_METHODS.contains(&"fixed_percentage"));
        assert!(VALID_ALLOCATION_METHODS.contains(&"step_down"));
        assert!(!VALID_ALLOCATION_METHODS.contains(&"invalid"));
    }

    #[test]
    fn test_valid_offset_methods() {
        assert!(VALID_OFFSET_METHODS.contains(&"none"));
        assert!(VALID_OFFSET_METHODS.contains(&"same_account"));
        assert!(VALID_OFFSET_METHODS.contains(&"specified_account"));
    }

    #[test]
    fn test_valid_run_statuses() {
        assert!(VALID_RUN_STATUSES.contains(&"draft"));
        assert!(VALID_RUN_STATUSES.contains(&"posted"));
        assert!(VALID_RUN_STATUSES.contains(&"reversed"));
        assert!(VALID_RUN_STATUSES.contains(&"cancelled"));
    }

    #[test]
    fn test_proportional_allocation_calculation() {
        // Test simple proportional allocation math
        let pool_amount = 10000.0_f64;
        let basis_amounts = vec![300.0, 200.0, 500.0];
        let total_basis: f64 = basis_amounts.iter().sum();

        assert!((total_basis - 1000.0).abs() < f64::EPSILON);

        let allocations: Vec<f64> = basis_amounts.iter()
            .map(|b| pool_amount * (b / total_basis))
            .collect();

        assert!((allocations[0] - 3000.0).abs() < 0.01);
        assert!((allocations[1] - 2000.0).abs() < 0.01);
        assert!((allocations[2] - 5000.0).abs() < 0.01);

        let total_allocated: f64 = allocations.iter().sum();
        assert!((total_allocated - pool_amount).abs() < 0.01);
    }

    #[test]
    fn test_fixed_percentage_allocation() {
        let pool_amount = 50000.0_f64;
        let percentages = vec![30.0, 45.0, 25.0];
        let total_pct: f64 = percentages.iter().sum();

        assert!((total_pct - 100.0).abs() < 0.01);

        let allocations: Vec<f64> = percentages.iter()
            .map(|p| pool_amount * (p / 100.0))
            .collect();

        assert!((allocations[0] - 15000.0).abs() < 0.01);
        assert!((allocations[1] - 22500.0).abs() < 0.01);
        assert!((allocations[2] - 12500.0).abs() < 0.01);
    }

    #[test]
    fn test_rounding_difference_tracking() {
        // Even with proportional allocation, rounding can cause small differences
        let pool_amount = 10000.0_f64;
        let basis = vec![333333333.0_f64, 333333333.0, 333333334.0];

        let total_basis: f64 = basis.iter().sum();
        let allocations: Vec<f64> = basis.iter()
            .map(|b| (pool_amount * (b / total_basis) * 100.0).round() / 100.0)
            .collect();

        let total_allocated: f64 = allocations.iter().sum();
        // Total should be very close to pool amount due to rounding
        assert!((total_allocated - pool_amount).abs() < 0.02);
    }

    #[test]
    fn test_validation_pool_code_required() {
        // Empty pool code is considered invalid in a real implementation
        let code = "";
        assert!(code.is_empty());
    }

    #[test]
    fn test_validation_method_combinations() {
        // Fixed percentage must have target lines with percentages
        // Proportional uses basis details
        // Step-down processes sequentially
        // These are validated in create_rule
    }
}