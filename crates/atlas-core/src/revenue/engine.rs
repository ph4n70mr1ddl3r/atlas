//! Revenue Recognition Engine (ASC 606 / IFRS 15)
//!
//! Implements the five-step model for revenue recognition:
//! 1. Identify the contract
//! 2. Identify performance obligations
//! 3. Determine the transaction price
//! 4. Allocate the transaction price to performance obligations
//! 5. Recognize revenue when/as obligations are satisfied
//!
//! Supports:
//! - Revenue policies (recognition methods, allocation bases)
//! - Revenue contracts (customer agreements)
//! - Performance obligations (distinct goods/services)
//! - Revenue recognition schedules (planned recognition events)
//! - Contract modifications (amendments)
//!
//! Oracle Fusion Cloud ERP equivalent: Financials > Revenue Management

use atlas_shared::{
    RevenuePolicy, RevenueContract, PerformanceObligation,
    RevenueScheduleLine, RevenueModification,
    AtlasError, AtlasResult,
};
use super::RevenueRepository;
use chrono::Datelike;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Valid recognition methods
const VALID_RECOGNITION_METHODS: &[&str] = &[
    "over_time", "point_in_time",
];

/// Valid over-time methods
const VALID_OVER_TIME_METHODS: &[&str] = &[
    "output", "input", "straight_line",
];

/// Valid allocation bases
const VALID_ALLOCATION_BASES: &[&str] = &[
    "standalone_selling_price", "residual", "equal",
];

/// Valid contract statuses
const VALID_CONTRACT_STATUSES: &[&str] = &[
    "draft", "active", "completed", "cancelled", "modified",
];

/// Valid obligation statuses
const VALID_OBLIGATION_STATUSES: &[&str] = &[
    "pending", "in_progress", "satisfied", "partially_satisfied", "cancelled",
];

/// Valid satisfaction methods
const VALID_SATISFACTION_METHODS: &[&str] = &[
    "over_time", "point_in_time",
];

/// Valid schedule line statuses
const VALID_SCHEDULE_STATUSES: &[&str] = &[
    "planned", "recognized", "reversed", "cancelled",
];

/// Valid modification types
const VALID_MODIFICATION_TYPES: &[&str] = &[
    "price_change", "scope_change", "term_extension",
    "termination", "add_obligation", "remove_obligation",
];

/// Valid modification statuses
const VALID_MODIFICATION_STATUSES: &[&str] = &[
    "draft", "active", "cancelled",
];

/// Revenue Recognition Engine
pub struct RevenueEngine {
    repository: Arc<dyn RevenueRepository>,
}

impl RevenueEngine {
    pub fn new(repository: Arc<dyn RevenueRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Revenue Policies
    // ========================================================================

    /// Create a new revenue recognition policy
    pub async fn create_policy(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        recognition_method: &str,
        over_time_method: Option<&str>,
        allocation_basis: &str,
        default_selling_price: Option<&str>,
        constrain_variable_consideration: bool,
        constraint_threshold_percent: Option<&str>,
        revenue_account_code: Option<&str>,
        deferred_revenue_account_code: Option<&str>,
        contra_revenue_account_code: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<RevenuePolicy> {
        if code.is_empty() || name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Policy code and name are required".to_string(),
            ));
        }
        if !VALID_RECOGNITION_METHODS.contains(&recognition_method) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid recognition method '{}'. Must be one of: {}",
                recognition_method, VALID_RECOGNITION_METHODS.join(", ")
            )));
        }
        if let Some(otm) = over_time_method {
            if !VALID_OVER_TIME_METHODS.contains(&otm) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid over-time method '{}'. Must be one of: {}",
                    otm, VALID_OVER_TIME_METHODS.join(", ")
                )));
            }
        }
        if !VALID_ALLOCATION_BASES.contains(&allocation_basis) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid allocation basis '{}'. Must be one of: {}",
                allocation_basis, VALID_ALLOCATION_BASES.join(", ")
            )));
        }
        if let Some(ref pct) = constraint_threshold_percent {
            let val: f64 = pct.parse().map_err(|_| AtlasError::ValidationFailed(
                "Constraint threshold must be a valid number".to_string(),
            ))?;
            if !(0.0..=100.0).contains(&val) {
                return Err(AtlasError::ValidationFailed(
                    "Constraint threshold must be between 0 and 100".to_string(),
                ));
            }
        }

        info!("Creating revenue policy '{}' for org {}", code, org_id);

        self.repository.create_policy(
            org_id, code, name, description,
            recognition_method, over_time_method, allocation_basis,
            default_selling_price, constrain_variable_consideration,
            constraint_threshold_percent,
            revenue_account_code, deferred_revenue_account_code,
            contra_revenue_account_code, created_by,
        ).await
    }

    /// Get a revenue policy by code
    pub async fn get_policy(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<RevenuePolicy>> {
        self.repository.get_policy(org_id, code).await
    }

    /// List all revenue policies for an organization
    pub async fn list_policies(&self, org_id: Uuid) -> AtlasResult<Vec<RevenuePolicy>> {
        self.repository.list_policies(org_id).await
    }

    /// Delete (deactivate) a revenue policy
    pub async fn delete_policy(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        info!("Deactivating revenue policy '{}' in org {}", code, org_id);
        self.repository.delete_policy(org_id, code).await
    }

    // ========================================================================
    // Revenue Contracts
    // ========================================================================

    /// Create a new revenue contract (Step 1: Identify the contract)
    pub async fn create_contract(
        &self,
        org_id: Uuid,
        source_type: Option<&str>,
        source_id: Option<Uuid>,
        source_number: Option<&str>,
        customer_id: Uuid,
        customer_number: Option<&str>,
        customer_name: Option<&str>,
        contract_date: Option<chrono::NaiveDate>,
        start_date: Option<chrono::NaiveDate>,
        end_date: Option<chrono::NaiveDate>,
        total_transaction_price: &str,
        currency_code: &str,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<RevenueContract> {
        let price: f64 = total_transaction_price.parse().map_err(|_| AtlasError::ValidationFailed(
            "Transaction price must be a valid number".to_string(),
        ))?;
        if price < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Transaction price must be non-negative".to_string(),
            ));
        }
        if currency_code.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Currency code is required".to_string(),
            ));
        }

        let contract_number = format!("RC-{}", Uuid::new_v4().to_string()[..8].to_uppercase());

        info!("Creating revenue contract {} for customer {} in org {}",
            contract_number, customer_id, org_id);

        self.repository.create_contract(
            org_id, &contract_number,
            source_type, source_id, source_number,
            customer_id, customer_number, customer_name,
            contract_date, start_date, end_date,
            total_transaction_price, currency_code, notes, created_by,
        ).await
    }

    /// Get a revenue contract by ID
    pub async fn get_contract(&self, id: Uuid) -> AtlasResult<Option<RevenueContract>> {
        self.repository.get_contract(id).await
    }

    /// Get a revenue contract by number
    pub async fn get_contract_by_number(&self, org_id: Uuid, contract_number: &str) -> AtlasResult<Option<RevenueContract>> {
        self.repository.get_contract_by_number(org_id, contract_number).await
    }

    /// List revenue contracts with optional filters
    pub async fn list_contracts(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        customer_id: Option<Uuid>,
    ) -> AtlasResult<Vec<RevenueContract>> {
        if let Some(s) = status {
            if !VALID_CONTRACT_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_CONTRACT_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_contracts(org_id, status, customer_id).await
    }

    /// Activate a contract (mark Step 1 complete)
    pub async fn activate_contract(&self, contract_id: Uuid) -> AtlasResult<RevenueContract> {
        let contract = self.repository.get_contract(contract_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Revenue contract {} not found", contract_id)
            ))?;

        if contract.status != "draft" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot activate contract in '{}' status. Must be 'draft'.", contract.status)
            ));
        }

        info!("Activating revenue contract {}", contract.contract_number);

        self.repository.update_contract_status(
            contract_id,
            Some("active"),
            Some(true), // step1_contract_identified
            None, None, None, None,
            None, None, None, None, None,
        ).await
    }

    /// Cancel a contract
    pub async fn cancel_contract(&self, contract_id: Uuid, reason: Option<&str>) -> AtlasResult<RevenueContract> {
        let contract = self.repository.get_contract(contract_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Revenue contract {} not found", contract_id)
            ))?;

        if contract.status == "completed" || contract.status == "cancelled" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot cancel contract in '{}' status", contract.status)
            ));
        }

        info!("Cancelling revenue contract {}", contract.contract_number);

        self.repository.update_contract_status(
            contract_id,
            Some("cancelled"),
            None, None, None, None, None,
            None, None, None, None, Some(reason),
        ).await
    }

    // ========================================================================
    // Performance Obligations (Step 2: Identify Performance Obligations)
    // ========================================================================

    /// Add a performance obligation to a contract
    pub async fn create_obligation(
        &self,
        org_id: Uuid,
        contract_id: Uuid,
        description: Option<&str>,
        product_id: Option<Uuid>,
        product_name: Option<&str>,
        source_line_id: Option<Uuid>,
        revenue_policy_id: Option<Uuid>,
        recognition_method: Option<&str>,
        over_time_method: Option<&str>,
        standalone_selling_price: &str,
        satisfaction_method: &str,
        recognition_start_date: Option<chrono::NaiveDate>,
        recognition_end_date: Option<chrono::NaiveDate>,
        revenue_account_code: Option<&str>,
        deferred_revenue_account_code: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PerformanceObligation> {
        let contract = self.repository.get_contract(contract_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Revenue contract {} not found", contract_id)
            ))?;

        if contract.status == "cancelled" || contract.status == "completed" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot add obligations to contract in '{}' status", contract.status)
            ));
        }

        let ssp: f64 = standalone_selling_price.parse().map_err(|_| AtlasError::ValidationFailed(
            "Standalone selling price must be a valid number".to_string(),
        ))?;
        if ssp < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Standalone selling price must be non-negative".to_string(),
            ));
        }

        if let Some(rm) = recognition_method {
            if !VALID_RECOGNITION_METHODS.contains(&rm) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid recognition method '{}'. Must be one of: {}",
                    rm, VALID_RECOGNITION_METHODS.join(", ")
                )));
            }
        }
        if !VALID_SATISFACTION_METHODS.contains(&satisfaction_method) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid satisfaction method '{}'. Must be one of: {}",
                satisfaction_method, VALID_SATISFACTION_METHODS.join(", ")
            )));
        }

        // Get next line number
        let obligations = self.repository.list_obligations(contract_id).await?;
        let line_number = (obligations.len() as i32) + 1;

        info!("Adding performance obligation {} to contract {}",
            line_number, contract.contract_number);

        let obligation = self.repository.create_obligation(
            org_id, contract_id, line_number,
            description, product_id, product_name, source_line_id,
            revenue_policy_id, recognition_method, over_time_method,
            standalone_selling_price, "0", // allocated_transaction_price (set during allocation)
            satisfaction_method,
            recognition_start_date, recognition_end_date,
            revenue_account_code, deferred_revenue_account_code,
            created_by,
        ).await?;

        // Mark step 2 as complete (obligations identified)
        self.repository.update_contract_status(
            contract_id,
            Some(&contract.status),
            None,
            Some(true), // step2_obligations_identified
            None, None, None,
            None, None, None, None, None,
        ).await?;

        Ok(obligation)
    }

    /// Get a performance obligation by ID
    pub async fn get_obligation(&self, id: Uuid) -> AtlasResult<Option<PerformanceObligation>> {
        self.repository.get_obligation(id).await
    }

    /// List performance obligations for a contract
    pub async fn list_obligations(&self, contract_id: Uuid) -> AtlasResult<Vec<PerformanceObligation>> {
        self.repository.list_obligations(contract_id).await
    }

    // ========================================================================
    // Transaction Price Allocation (Step 4: Allocate Transaction Price)
    // ========================================================================

    /// Allocate the transaction price across all performance obligations
    /// using the standalone selling price (SSP) method.
    /// This implements ASC 606 Step 4.
    pub async fn allocate_transaction_price(&self, contract_id: Uuid) -> AtlasResult<Vec<PerformanceObligation>> {
        let contract = self.repository.get_contract(contract_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Revenue contract {} not found", contract_id)
            ))?;

        if contract.status == "cancelled" {
            return Err(AtlasError::WorkflowError(
                "Cannot allocate price for a cancelled contract".to_string()
            ));
        }

        let obligations = self.repository.list_obligations(contract_id).await?;
        if obligations.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Cannot allocate transaction price: no performance obligations defined".to_string(),
            ));
        }

        let total_price: f64 = contract.total_transaction_price.parse().unwrap_or(0.0);
        let total_ssp: f64 = obligations.iter()
            .map(|o| o.standalone_selling_price.parse::<f64>().unwrap_or(0.0))
            .sum();

        if total_ssp <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Total standalone selling price must be positive".to_string(),
            ));
        }

        info!("Allocating transaction price {} across {} obligations (total SSP: {})",
            total_price, obligations.len(), total_ssp);

        let mut updated_obligations = Vec::new();
        for obligation in &obligations {
            let ssp: f64 = obligation.standalone_selling_price.parse().unwrap_or(0.0);
            let allocated = (ssp / total_ssp) * total_price;
            let allocated_str = format!("{:.2}", allocated);

            let updated = self.repository.update_obligation_allocation(
                obligation.id,
                &allocated_str,
                &format!("{:.2}", allocated), // deferred_revenue = allocated initially
            ).await?;

            updated_obligations.push(updated);
        }

        // Mark step 3 (price determined) and step 4 (price allocated)
        let total_allocated: f64 = updated_obligations.iter()
            .map(|o| o.allocated_transaction_price.parse::<f64>().unwrap_or(0.0))
            .sum();

        self.repository.update_contract_status(
            contract_id,
            Some(&contract.status),
            None,
            None,
            Some(true), // step3_price_determined
            Some(true), // step4_price_allocated
            None,
            Some(&format!("{:.2}", total_allocated)),
            Some(&format!("{:.2}", total_allocated)),
            Some("0"),
            None, None,
        ).await?;

        Ok(updated_obligations)
    }

    // ========================================================================
    // Revenue Recognition Scheduling (Step 5: Recognize Revenue)
    // ========================================================================

    /// Generate a straight-line recognition schedule for a performance obligation.
    /// Creates evenly-distributed schedule lines from recognition_start_date to
    /// recognition_end_date.
    pub async fn generate_straight_line_schedule(
        &self,
        obligation_id: Uuid,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
    ) -> AtlasResult<Vec<RevenueScheduleLine>> {
        let obligation = self.repository.get_obligation(obligation_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Performance obligation {} not found", obligation_id)
            ))?;

        if obligation.status == "cancelled" || obligation.status == "satisfied" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot generate schedule for obligation in '{}' status", obligation.status)
            ));
        }

        if start_date >= end_date {
            return Err(AtlasError::ValidationFailed(
                "Start date must be before end date".to_string(),
            ));
        }

        let total_amount: f64 = obligation.allocated_transaction_price.parse().unwrap_or(0.0);
        if total_amount <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Obligation must have an allocated transaction price before scheduling".to_string(),
            ));
        }

        // Calculate monthly schedule
        let total_months = Self::months_between(start_date, end_date);
        if total_months == 0 {
            return Err(AtlasError::ValidationFailed(
                "Date range must span at least one month".to_string(),
            ));
        }

        let per_month = total_amount / (total_months as f64);
        // Adjust last month for rounding
        let per_month_rounded = format!("{:.2}", per_month);

        info!("Generating {}-month straight-line schedule for obligation {} ({} per month)",
            total_months, obligation_id, per_month_rounded);

        let mut lines = Vec::new();
        let mut current = start_date;
        let mut total_scheduled = 0.0_f64;

        for i in 1..=total_months {
            let is_last = i == total_months;
            let amount = if is_last {
                // Last month gets the remainder to avoid rounding errors
                format!("{:.2}", total_amount - total_scheduled)
            } else {
                per_month_rounded.clone()
            };
            let amount_f64: f64 = amount.parse().unwrap();
            total_scheduled += amount_f64;

            let line = self.repository.create_schedule_line(
                obligation.organization_id,
                obligation_id,
                obligation.contract_id,
                i,
                current,
                &amount,
                &format!("{:.4}", amount_f64 / total_amount * 100.0),
                "straight_line",
                obligation.created_by,
            ).await?;

            lines.push(line);

            // Advance to next month
            current = Self::add_months(current, 1);
        }

        // Mark step 5 as complete
        self.repository.update_contract_status(
            obligation.contract_id,
            None as Option<&str>,
            None, None, None, None,
            Some(true), // step5_recognition_scheduled
            None, None, None, None, None,
        ).await?;

        // Update obligation status to in_progress
        self.repository.update_obligation_status(
            obligation_id,
            "in_progress",
            Some(&start_date.to_string()),
            None,
        ).await?;

        Ok(lines)
    }

    /// Create a point-in-time recognition schedule (single recognition date)
    pub async fn schedule_point_in_time(
        &self,
        obligation_id: Uuid,
        recognition_date: chrono::NaiveDate,
    ) -> AtlasResult<RevenueScheduleLine> {
        let obligation = self.repository.get_obligation(obligation_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Performance obligation {} not found", obligation_id)
            ))?;

        if obligation.status == "cancelled" || obligation.status == "satisfied" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot schedule for obligation in '{}' status", obligation.status)
            ));
        }

        let total_amount: f64 = obligation.allocated_transaction_price.parse().unwrap_or(0.0);
        if total_amount <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Obligation must have an allocated transaction price".to_string(),
            ));
        }

        info!("Creating point-in-time schedule for obligation {} on {}",
            obligation_id, recognition_date);

        let line = self.repository.create_schedule_line(
            obligation.organization_id,
            obligation_id,
            obligation.contract_id,
            1,
            recognition_date,
            &format!("{:.2}", total_amount),
            "100.0000",
            "point_in_time",
            obligation.created_by,
        ).await?;

        // Mark step 5 and update obligation
        self.repository.update_contract_status(
            obligation.contract_id,
            None as Option<&str>,
            None, None, None, None,
            Some(true),
            None, None, None, None, None,
        ).await?;

        self.repository.update_obligation_status(
            obligation_id,
            "pending", // Still pending until recognition date arrives
            Some(&recognition_date.to_string()),
            None,
        ).await?;

        Ok(line)
    }

    /// Recognize revenue for a specific schedule line (post to GL)
    pub async fn recognize_revenue(&self, line_id: Uuid) -> AtlasResult<RevenueScheduleLine> {
        let line = self.repository.get_schedule_line(line_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Revenue schedule line {} not found", line_id)
            ))?;

        if line.status != "planned" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot recognize line in '{}' status. Must be 'planned'.", line.status)
            ));
        }

        let amount: f64 = line.amount.parse().unwrap_or(0.0);
        info!("Recognizing revenue of {} for schedule line {}", amount, line_id);

        // Update the schedule line
        let updated_line = self.repository.update_schedule_line_status(
            line_id,
            "recognized",
            Some(&line.amount),
            None,
        ).await?;

        // Update the parent obligation's recognized/deferred amounts
        let obligation = self.repository.get_obligation(line.obligation_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Performance obligation {} not found", line.obligation_id)
            ))?;

        let prev_recognized: f64 = obligation.total_recognized_revenue.parse().unwrap_or(0.0);
        let new_recognized = prev_recognized + amount;
        let prev_deferred: f64 = obligation.deferred_revenue.parse().unwrap_or(0.0);
        let new_deferred = (prev_deferred - amount).max(0.0);

        let allocated: f64 = obligation.allocated_transaction_price.parse().unwrap_or(0.0);
        let pct_complete = if allocated > 0.0 {
            format!("{:.2}", new_recognized / allocated * 100.0)
        } else {
            "0.00".to_string()
        };

        let new_status = if new_deferred < 0.01 {
            "satisfied"
        } else {
            "partially_satisfied"
        };

        self.repository.update_obligation_recognition(
            obligation.id,
            &format!("{:.2}", new_recognized),
            &format!("{:.2}", new_deferred),
            &pct_complete,
            new_status,
        ).await?;

        // Update contract totals
        let contract = self.repository.get_contract(line.contract_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Revenue contract {} not found", line.contract_id)
            ))?;

        let contract_recognized: f64 = contract.total_recognized_revenue.parse().unwrap_or(0.0) + amount;
        let contract_deferred: f64 = contract.total_deferred_revenue.parse().unwrap_or(0.0);
        let new_contract_deferred = (contract_deferred - amount).max(0.0);

        self.repository.update_contract_status(
            contract.id,
            None as Option<&str>,
            None, None, None, None, None,
            None,
            Some(&format!("{:.2}", contract_recognized)),
            Some(&format!("{:.2}", new_contract_deferred)),
            None, None,
        ).await?;

        Ok(updated_line)
    }

    /// Reverse a previously recognized revenue line
    pub async fn reverse_recognition(&self, line_id: Uuid, reason: &str) -> AtlasResult<RevenueScheduleLine> {
        let line = self.repository.get_schedule_line(line_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Revenue schedule line {} not found", line_id)
            ))?;

        if line.status != "recognized" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot reverse line in '{}' status. Must be 'recognized'.", line.status)
            ));
        }
        if reason.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Reversal reason is required".to_string(),
            ));
        }

        let amount: f64 = line.recognized_amount.parse().unwrap_or(0.0);
        info!("Reversing revenue of {} for schedule line {} ({})", amount, line_id, reason);

        self.repository.update_schedule_line_status(
            line_id,
            "reversed",
            None,
            Some(reason),
        ).await
    }

    /// List schedule lines for an obligation
    pub async fn list_schedule_lines(&self, obligation_id: Uuid) -> AtlasResult<Vec<RevenueScheduleLine>> {
        self.repository.list_schedule_lines(obligation_id).await
    }

    /// List all schedule lines for a contract (across all obligations)
    pub async fn list_contract_schedule_lines(&self, contract_id: Uuid) -> AtlasResult<Vec<RevenueScheduleLine>> {
        self.repository.list_schedule_lines_by_contract(contract_id).await
    }

    // ========================================================================
    // Contract Modifications
    // ========================================================================

    /// Create a contract modification
    pub async fn create_modification(
        &self,
        org_id: Uuid,
        contract_id: Uuid,
        modification_type: &str,
        description: Option<&str>,
        previous_transaction_price: &str,
        new_transaction_price: &str,
        previous_end_date: Option<chrono::NaiveDate>,
        new_end_date: Option<chrono::NaiveDate>,
        effective_date: chrono::NaiveDate,
        created_by: Option<Uuid>,
    ) -> AtlasResult<RevenueModification> {
        let contract = self.repository.get_contract(contract_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Revenue contract {} not found", contract_id)
            ))?;

        if contract.status != "active" && contract.status != "modified" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot modify contract in '{}' status", contract.status)
            ));
        }

        if !VALID_MODIFICATION_TYPES.contains(&modification_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid modification type '{}'. Must be one of: {}",
                modification_type, VALID_MODIFICATION_TYPES.join(", ")
            )));
        }

        // Get next modification number
        let modifications = self.repository.list_modifications(contract_id).await?;
        let modification_number = (modifications.len() as i32) + 1;

        info!("Creating modification #{} for contract {} ({})",
            modification_number, contract.contract_number, modification_type);

        let modification = self.repository.create_modification(
            org_id, contract_id, modification_number,
            modification_type, description,
            previous_transaction_price, new_transaction_price,
            previous_end_date, new_end_date,
            effective_date, created_by,
        ).await?;

        // Update contract status and price
        let new_price: f64 = new_transaction_price.parse().unwrap_or(0.0);
        self.repository.update_contract_status(
            contract_id,
            Some("modified"),
            None, None, None, None, None,
            None, None, None,
            Some(&format!("{:.2}", new_price)),
            None,
        ).await?;

        Ok(modification)
    }

    /// List modifications for a contract
    pub async fn list_modifications(&self, contract_id: Uuid) -> AtlasResult<Vec<RevenueModification>> {
        self.repository.list_modifications(contract_id).await
    }

    // ========================================================================
    // Helpers
    // ========================================================================

    /// Calculate the number of whole months between two dates
    fn months_between(start: chrono::NaiveDate, end: chrono::NaiveDate) -> i32 {
        let years = end.year() - start.year();
        let months = end.month() as i32 - start.month() as i32;
        let total = years * 12 + months;
        if total <= 0 { 1 } else { total }
    }

    /// Add months to a date (clamped to valid day)
    fn add_months(date: chrono::NaiveDate, months: i32) -> chrono::NaiveDate {
        let mut year = date.year();
        let mut month = date.month() as i32 + months;
        while month > 12 {
            month -= 12;
            year += 1;
        }
        while month < 1 {
            month += 12;
            year -= 1;
        }
        let day = date.day().min(days_in_month(year, month as u32));
        chrono::NaiveDate::from_ymd_opt(year, month as u32, day).unwrap_or(date)
    }
}

/// Helper: get number of days in a month
fn days_in_month(year: i32, month: u32) -> u32 {
    // month+1 might overflow (December -> 13), handle via checked arithmetic
    if month == 12 {
        // December: compare Jan 1 of next year with Dec 1
        let dec1 = chrono::NaiveDate::from_ymd_opt(year, 12, 1).unwrap();
        let jan1 = chrono::NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap();
        (jan1 - dec1).num_days() as u32
    } else {
        let first = chrono::NaiveDate::from_ymd_opt(year, month, 1).unwrap();
        let next = chrono::NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap();
        (next - first).num_days() as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_recognition_methods() {
        assert!(VALID_RECOGNITION_METHODS.contains(&"over_time"));
        assert!(VALID_RECOGNITION_METHODS.contains(&"point_in_time"));
        assert_eq!(VALID_RECOGNITION_METHODS.len(), 2);
    }

    #[test]
    fn test_valid_over_time_methods() {
        assert!(VALID_OVER_TIME_METHODS.contains(&"output"));
        assert!(VALID_OVER_TIME_METHODS.contains(&"input"));
        assert!(VALID_OVER_TIME_METHODS.contains(&"straight_line"));
    }

    #[test]
    fn test_valid_allocation_bases() {
        assert!(VALID_ALLOCATION_BASES.contains(&"standalone_selling_price"));
        assert!(VALID_ALLOCATION_BASES.contains(&"residual"));
        assert!(VALID_ALLOCATION_BASES.contains(&"equal"));
    }

    #[test]
    fn test_valid_contract_statuses() {
        assert!(VALID_CONTRACT_STATUSES.contains(&"draft"));
        assert!(VALID_CONTRACT_STATUSES.contains(&"active"));
        assert!(VALID_CONTRACT_STATUSES.contains(&"completed"));
        assert!(VALID_CONTRACT_STATUSES.contains(&"cancelled"));
        assert!(VALID_CONTRACT_STATUSES.contains(&"modified"));
    }

    #[test]
    fn test_valid_obligation_statuses() {
        assert!(VALID_OBLIGATION_STATUSES.contains(&"pending"));
        assert!(VALID_OBLIGATION_STATUSES.contains(&"in_progress"));
        assert!(VALID_OBLIGATION_STATUSES.contains(&"satisfied"));
        assert!(VALID_OBLIGATION_STATUSES.contains(&"partially_satisfied"));
        assert!(VALID_OBLIGATION_STATUSES.contains(&"cancelled"));
    }

    #[test]
    fn test_valid_schedule_statuses() {
        assert!(VALID_SCHEDULE_STATUSES.contains(&"planned"));
        assert!(VALID_SCHEDULE_STATUSES.contains(&"recognized"));
        assert!(VALID_SCHEDULE_STATUSES.contains(&"reversed"));
        assert!(VALID_SCHEDULE_STATUSES.contains(&"cancelled"));
    }

    #[test]
    fn test_valid_modification_types() {
        assert!(VALID_MODIFICATION_TYPES.contains(&"price_change"));
        assert!(VALID_MODIFICATION_TYPES.contains(&"scope_change"));
        assert!(VALID_MODIFICATION_TYPES.contains(&"term_extension"));
        assert!(VALID_MODIFICATION_TYPES.contains(&"termination"));
        assert!(VALID_MODIFICATION_TYPES.contains(&"add_obligation"));
        assert!(VALID_MODIFICATION_TYPES.contains(&"remove_obligation"));
    }

    #[test]
    fn test_months_between() {
        let jan1 = chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let jun1 = chrono::NaiveDate::from_ymd_opt(2024, 6, 1).unwrap();
        assert_eq!(RevenueEngine::months_between(jan1, jun1), 5);

        let dec1 = chrono::NaiveDate::from_ymd_opt(2024, 12, 1).unwrap();
        let jan1_2025 = chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        assert_eq!(RevenueEngine::months_between(dec1, jan1_2025), 1);

        // Same month returns 1
        let same = chrono::NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();
        assert_eq!(RevenueEngine::months_between(same, same), 1);
    }

    #[test]
    fn test_add_months() {
        let jan31 = chrono::NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();
        let feb = RevenueEngine::add_months(jan31, 1);
        // Jan 31 + 1 month should clamp to Feb 29 (2024 is leap year)
        assert_eq!(feb, chrono::NaiveDate::from_ymd_opt(2024, 2, 29).unwrap());

        let mar15 = chrono::NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
        let jun15 = RevenueEngine::add_months(mar15, 3);
        assert_eq!(jun15, chrono::NaiveDate::from_ymd_opt(2024, 6, 15).unwrap());

        let nov = RevenueEngine::add_months(
            chrono::NaiveDate::from_ymd_opt(2024, 11, 15).unwrap(), 3
        );
        assert_eq!(nov, chrono::NaiveDate::from_ymd_opt(2025, 2, 15).unwrap());
    }

    #[test]
    fn test_months_between_year_boundary() {
        let oct = chrono::NaiveDate::from_ymd_opt(2024, 10, 1).unwrap();
        let mar = chrono::NaiveDate::from_ymd_opt(2025, 3, 1).unwrap();
        assert_eq!(RevenueEngine::months_between(oct, mar), 5);
    }

    #[test]
    fn test_days_in_month() {
        assert_eq!(days_in_month(2024, 1), 31);
        assert_eq!(days_in_month(2024, 2), 29); // leap year
        assert_eq!(days_in_month(2023, 2), 28);
        assert_eq!(days_in_month(2024, 4), 30);
        assert_eq!(days_in_month(2024, 12), 31);
    }
}
