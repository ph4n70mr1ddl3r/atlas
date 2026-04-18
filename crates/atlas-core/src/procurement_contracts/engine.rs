//! Procurement Contracts Engine
//!
//! Manages the full lifecycle of procurement contracts:
//! - Contract type definitions
//! - Contract creation and lifecycle (draft → pending_approval → active → expired/closed/terminated)
//! - Contract lines with quantity and amount tracking
//! - Milestones and deliverables
//! - Contract renewals
//! - Spend tracking against contracts
//!
//! Oracle Fusion Cloud ERP equivalent: SCM > Procurement > Contracts

use atlas_shared::{
    ContractType, ProcurementContract, ContractLine, ContractMilestone,
    ContractRenewal, ContractSpend, ContractDashboardSummary,
    AtlasError, AtlasResult,
};
use super::ProcurementContractRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Valid contract classifications
const VALID_CLASSIFICATIONS: &[&str] = &[
    "blanket", "purchase_agreement", "service", "lease", "other",
];

/// Valid contract statuses
const VALID_STATUSES: &[&str] = &[
    "draft", "pending_approval", "active", "expired", "terminated", "closed",
];

/// Valid milestone types
const VALID_MILESTONE_TYPES: &[&str] = &[
    "delivery", "payment", "review", "acceptance", "custom",
];

/// Valid milestone statuses
const VALID_MILESTONE_STATUSES: &[&str] = &[
    "pending", "in_progress", "completed", "overdue", "cancelled",
];

/// Valid renewal types
const VALID_RENEWAL_TYPES: &[&str] = &[
    "automatic", "manual", "negotiated",
];

/// Valid price types
const VALID_PRICE_TYPES: &[&str] = &[
    "fixed", "variable", "tiered",
];

/// Procurement Contracts Engine
pub struct ProcurementContractEngine {
    repository: Arc<dyn ProcurementContractRepository>,
}

impl ProcurementContractEngine {
    pub fn new(repository: Arc<dyn ProcurementContractRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Contract Types
    // ========================================================================

    /// Create a new contract type
    pub async fn create_contract_type(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        contract_classification: &str,
        requires_approval: bool,
        default_duration_days: Option<i32>,
        allow_amount_commitment: bool,
        allow_quantity_commitment: bool,
        allow_line_additions: bool,
        allow_price_adjustment: bool,
        allow_renewal: bool,
        allow_termination: bool,
        max_renewals: Option<i32>,
        default_payment_terms_code: Option<&str>,
        default_currency_code: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ContractType> {
        if code.is_empty() || name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Contract type code and name are required".to_string(),
            ));
        }
        if !VALID_CLASSIFICATIONS.contains(&contract_classification) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid classification '{}'. Must be one of: {}",
                contract_classification, VALID_CLASSIFICATIONS.join(", ")
            )));
        }

        info!("Creating contract type '{}' for org {}", code, org_id);

        self.repository.create_contract_type(
            org_id, code, name, description, contract_classification,
            requires_approval, default_duration_days,
            allow_amount_commitment, allow_quantity_commitment,
            allow_line_additions, allow_price_adjustment,
            allow_renewal, allow_termination, max_renewals,
            default_payment_terms_code, default_currency_code,
            created_by,
        ).await
    }

    /// Get a contract type by code
    pub async fn get_contract_type(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<ContractType>> {
        self.repository.get_contract_type(org_id, code).await
    }

    /// List all contract types
    pub async fn list_contract_types(&self, org_id: Uuid) -> AtlasResult<Vec<ContractType>> {
        self.repository.list_contract_types(org_id).await
    }

    /// Delete a contract type
    pub async fn delete_contract_type(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        info!("Deactivating contract type '{}' in org {}", code, org_id);
        self.repository.delete_contract_type(org_id, code).await
    }

    // ========================================================================
    // Contracts
    // ========================================================================

    /// Create a new procurement contract
    pub async fn create_contract(
        &self,
        org_id: Uuid,
        title: &str,
        description: Option<&str>,
        contract_type_code: Option<&str>,
        contract_classification: &str,
        supplier_id: Uuid,
        supplier_number: Option<&str>,
        supplier_name: Option<&str>,
        supplier_contact: Option<&str>,
        buyer_id: Option<Uuid>,
        buyer_name: Option<&str>,
        start_date: Option<chrono::NaiveDate>,
        end_date: Option<chrono::NaiveDate>,
        total_committed_amount: &str,
        currency_code: &str,
        payment_terms_code: Option<&str>,
        price_type: &str,
        max_renewals: Option<i32>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ProcurementContract> {
        if title.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Contract title is required".to_string(),
            ));
        }
        if !VALID_CLASSIFICATIONS.contains(&contract_classification) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid classification '{}'. Must be one of: {}",
                contract_classification, VALID_CLASSIFICATIONS.join(", ")
            )));
        }
        if !VALID_PRICE_TYPES.contains(&price_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid price type '{}'. Must be one of: {}",
                price_type, VALID_PRICE_TYPES.join(", ")
            )));
        }
        if currency_code.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Currency code is required".to_string(),
            ));
        }
        let committed: f64 = total_committed_amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "Total committed amount must be a valid number".to_string(),
        ))?;
        if committed < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Total committed amount must be non-negative".to_string(),
            ));
        }
        // Validate date range if both provided
        if let (Some(sd), Some(ed)) = (start_date, end_date) {
            if sd >= ed {
                return Err(AtlasError::ValidationFailed(
                    "Start date must be before end date".to_string(),
                ));
            }
        }

        let contract_number = format!("PC-{}", Uuid::new_v4().to_string()[..8].to_uppercase());

        info!("Creating procurement contract {} for supplier {} in org {}",
            contract_number, supplier_id, org_id);

        self.repository.create_contract(
            org_id, &contract_number, title, description,
            contract_type_code, contract_classification,
            supplier_id, supplier_number, supplier_name, supplier_contact,
            buyer_id, buyer_name, start_date, end_date,
            total_committed_amount, currency_code,
            payment_terms_code, price_type, max_renewals, notes,
            created_by,
        ).await
    }

    /// Get a contract by ID
    pub async fn get_contract(&self, id: Uuid) -> AtlasResult<Option<ProcurementContract>> {
        self.repository.get_contract(id).await
    }

    /// Get a contract by number
    pub async fn get_contract_by_number(&self, org_id: Uuid, contract_number: &str) -> AtlasResult<Option<ProcurementContract>> {
        self.repository.get_contract_by_number(org_id, contract_number).await
    }

    /// List contracts with optional filters
    pub async fn list_contracts(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        supplier_id: Option<Uuid>,
    ) -> AtlasResult<Vec<ProcurementContract>> {
        if let Some(s) = status {
            if !VALID_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_contracts(org_id, status, supplier_id).await
    }

    /// Submit a contract for approval
    pub async fn submit_contract(&self, contract_id: Uuid) -> AtlasResult<ProcurementContract> {
        let contract = self.repository.get_contract(contract_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Contract {} not found", contract_id)
            ))?;

        if contract.status != "draft" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot submit contract in '{}' status. Must be 'draft'.", contract.status
            )));
        }

        info!("Submitting contract {} for approval", contract.contract_number);

        // Check if the contract type requires approval
        let needs_approval = if let Some(ref type_code) = contract.contract_type_code {
            let ct = self.repository.get_contract_type(contract.organization_id, type_code).await?;
            ct.map(|t| t.requires_approval).unwrap_or(true)
        } else {
            true // Default: require approval
        };

        let new_status = if needs_approval { "pending_approval" } else { "active" };
        self.repository.update_contract_status(
            contract_id, new_status, None, None, None, None,
        ).await
    }

    /// Approve a contract (activate it)
    pub async fn approve_contract(&self, contract_id: Uuid, approved_by: Uuid) -> AtlasResult<ProcurementContract> {
        let contract = self.repository.get_contract(contract_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Contract {} not found", contract_id)
            ))?;

        if contract.status != "pending_approval" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot approve contract in '{}' status. Must be 'pending_approval'.", contract.status
            )));
        }

        info!("Approving contract {} by {}", contract.contract_number, approved_by);

        self.repository.update_contract_status(
            contract_id, "active", Some(approved_by), None, None, None,
        ).await
    }

    /// Reject a contract
    pub async fn reject_contract(&self, contract_id: Uuid, reason: &str) -> AtlasResult<ProcurementContract> {
        if reason.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Rejection reason is required".to_string(),
            ));
        }

        let contract = self.repository.get_contract(contract_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Contract {} not found", contract_id)
            ))?;

        if contract.status != "pending_approval" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot reject contract in '{}' status.", contract.status
            )));
        }

        info!("Rejecting contract {} - reason: {}", contract.contract_number, reason);

        self.repository.update_contract_status(
            contract_id, "draft", None, Some(reason), None, None,
        ).await
    }

    /// Terminate an active contract
    pub async fn terminate_contract(
        &self,
        contract_id: Uuid,
        terminated_by: Uuid,
        reason: &str,
    ) -> AtlasResult<ProcurementContract> {
        if reason.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Termination reason is required".to_string(),
            ));
        }

        let contract = self.repository.get_contract(contract_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Contract {} not found", contract_id)
            ))?;

        if contract.status != "active" && contract.status != "expired" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot terminate contract in '{}' status. Must be 'active' or 'expired'.", contract.status
            )));
        }

        info!("Terminating contract {} - reason: {}", contract.contract_number, reason);

        self.repository.update_contract_status(
            contract_id, "terminated", None, None, Some(terminated_by), Some(reason),
        ).await
    }

    /// Close a contract
    pub async fn close_contract(&self, contract_id: Uuid) -> AtlasResult<ProcurementContract> {
        let contract = self.repository.get_contract(contract_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Contract {} not found", contract_id)
            ))?;

        if contract.status != "active" && contract.status != "expired" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot close contract in '{}' status.", contract.status
            )));
        }

        info!("Closing contract {}", contract.contract_number);

        self.repository.update_contract_status(
            contract_id, "closed", None, None, None, None,
        ).await
    }

    // ========================================================================
    // Contract Lines
    // ========================================================================

    /// Add a line to a contract
    pub async fn add_contract_line(
        &self,
        org_id: Uuid,
        contract_id: Uuid,
        item_description: &str,
        item_code: Option<&str>,
        category: Option<&str>,
        uom: Option<&str>,
        quantity_committed: Option<&str>,
        unit_price: &str,
        delivery_date: Option<chrono::NaiveDate>,
        supplier_part_number: Option<&str>,
        account_code: Option<&str>,
        cost_center: Option<&str>,
        project_id: Option<Uuid>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ContractLine> {
        let contract = self.repository.get_contract(contract_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Contract {} not found", contract_id)
            ))?;

        if contract.status != "draft" && contract.status != "pending_approval" {
            // Check if type allows line additions after activation
            let allow = if let Some(ref type_code) = contract.contract_type_code {
                let ct = self.repository.get_contract_type(org_id, type_code).await?;
                ct.map(|t| t.allow_line_additions).unwrap_or(false)
            } else {
                false
            };
            if !allow {
                return Err(AtlasError::WorkflowError(format!(
                    "Cannot add lines to contract in '{}' status", contract.status
                )));
            }
        }

        if item_description.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Item description is required".to_string(),
            ));
        }

        let price: f64 = unit_price.parse().map_err(|_| AtlasError::ValidationFailed(
            "Unit price must be a valid number".to_string(),
        ))?;
        if price < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Unit price must be non-negative".to_string(),
            ));
        }

        let qty: f64 = quantity_committed
            .map(|q| q.parse::<f64>().unwrap_or(0.0))
            .unwrap_or(0.0);

        let line_amount = if qty > 0.0 {
            format!("{:.2}", qty * price)
        } else {
            "0".to_string()
        };

        // Get next line number
        let existing_lines = self.repository.list_contract_lines(contract_id).await?;
        let line_number = (existing_lines.len() as i32) + 1;

        info!("Adding line {} to contract {}", line_number, contract.contract_number);

        let line = self.repository.create_contract_line(
            org_id, contract_id, line_number,
            item_description, item_code, category, uom,
            quantity_committed, "0",
            unit_price, &line_amount, "0",
            delivery_date, supplier_part_number,
            account_code, cost_center, project_id, notes,
            created_by,
        ).await?;

        // Recalculate total committed amount
        let all_lines = self.repository.list_contract_lines(contract_id).await?;
        let total_committed: f64 = all_lines.iter()
            .map(|l| l.line_amount.parse::<f64>().unwrap_or(0.0))
            .sum();

        self.repository.update_contract_totals(
            contract_id,
            Some(&format!("{:.2}", total_committed)),
            None, None,
            Some(all_lines.len() as i32),
            None,
        ).await?;

        Ok(line)
    }

    /// List contract lines
    pub async fn list_contract_lines(&self, contract_id: Uuid) -> AtlasResult<Vec<ContractLine>> {
        self.repository.list_contract_lines(contract_id).await
    }

    /// Delete a contract line
    pub async fn delete_contract_line(&self, line_id: Uuid) -> AtlasResult<()> {
        let line = self.repository.get_contract_line(line_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Contract line {} not found", line_id)
            ))?;

        let contract = self.repository.get_contract(line.contract_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Contract {} not found", line.contract_id)
            ))?;

        if contract.status != "draft" && contract.status != "pending_approval" {
            return Err(AtlasError::WorkflowError(
                "Cannot delete lines from an active/closed contract".to_string(),
            ));
        }

        info!("Deleting line {} from contract {}", line.line_number, contract.contract_number);

        self.repository.delete_contract_line(line_id).await?;

        // Recalculate totals
        let remaining = self.repository.list_contract_lines(line.contract_id).await?;
        let total_committed: f64 = remaining.iter()
            .map(|l| l.line_amount.parse::<f64>().unwrap_or(0.0))
            .sum();

        self.repository.update_contract_totals(
            line.contract_id,
            Some(&format!("{:.2}", total_committed)),
            None, None,
            Some(remaining.len() as i32),
            None,
        ).await?;

        Ok(())
    }

    // ========================================================================
    // Milestones
    // ========================================================================

    /// Add a milestone to a contract
    pub async fn add_milestone(
        &self,
        org_id: Uuid,
        contract_id: Uuid,
        contract_line_id: Option<Uuid>,
        name: &str,
        description: Option<&str>,
        milestone_type: &str,
        target_date: chrono::NaiveDate,
        amount: &str,
        percent_of_total: &str,
        deliverable: Option<&str>,
        is_billable: bool,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ContractMilestone> {
        let contract = self.repository.get_contract(contract_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Contract {} not found", contract_id)
            ))?;

        if contract.status == "terminated" || contract.status == "closed" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot add milestones to contract in '{}' status", contract.status)
            ));
        }

        if !VALID_MILESTONE_TYPES.contains(&milestone_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid milestone type '{}'. Must be one of: {}",
                milestone_type, VALID_MILESTONE_TYPES.join(", ")
            )));
        }

        if name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Milestone name is required".to_string(),
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

        // Get next milestone number
        let existing = self.repository.list_milestones(contract_id).await?;
        let milestone_number = (existing.len() as i32) + 1;

        info!("Adding milestone {} '{}' to contract {}",
            milestone_number, name, contract.contract_number);

        let milestone = self.repository.create_milestone(
            org_id, contract_id, contract_line_id,
            milestone_number, name, description,
            milestone_type, target_date,
            amount, percent_of_total,
            deliverable, is_billable, created_by,
        ).await?;

        // Update milestone count
        let all_milestones = self.repository.list_milestones(contract_id).await?;
        self.repository.update_contract_totals(
            contract_id, None, None, None, None,
            Some(all_milestones.len() as i32),
        ).await?;

        Ok(milestone)
    }

    /// List milestones for a contract
    pub async fn list_milestones(&self, contract_id: Uuid) -> AtlasResult<Vec<ContractMilestone>> {
        self.repository.list_milestones(contract_id).await
    }

    /// Update milestone status
    pub async fn update_milestone_status(
        &self,
        milestone_id: Uuid,
        status: &str,
        actual_date: Option<chrono::NaiveDate>,
    ) -> AtlasResult<ContractMilestone> {
        if !VALID_MILESTONE_STATUSES.contains(&status) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid milestone status '{}'. Must be one of: {}",
                status, VALID_MILESTONE_STATUSES.join(", ")
            )));
        }

        let milestone = self.repository.get_milestone(milestone_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Milestone {} not found", milestone_id)
            ))?;

        info!("Updating milestone {} status to {}", milestone.name, status);

        self.repository.update_milestone_status(milestone_id, status, actual_date).await
    }

    // ========================================================================
    // Renewals
    // ========================================================================

    /// Renew a contract
    pub async fn renew_contract(
        &self,
        contract_id: Uuid,
        new_end_date: chrono::NaiveDate,
        renewal_type: &str,
        terms_changed: Option<&str>,
        renewed_by: Option<Uuid>,
        notes: Option<&str>,
    ) -> AtlasResult<ContractRenewal> {
        if !VALID_RENEWAL_TYPES.contains(&renewal_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid renewal type '{}'. Must be one of: {}",
                renewal_type, VALID_RENEWAL_TYPES.join(", ")
            )));
        }

        let contract = self.repository.get_contract(contract_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Contract {} not found", contract_id)
            ))?;

        if contract.status != "active" && contract.status != "expired" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot renew contract in '{}' status", contract.status)
            ));
        }

        // Check renewal limits
        if let Some(max) = contract.max_renewals {
            if contract.renewal_count >= max {
                return Err(AtlasError::ValidationFailed(
                    format!("Contract has reached maximum renewals ({})", max)
                ));
            }
        }

        let previous_end_date = contract.end_date
            .ok_or_else(|| AtlasError::ValidationFailed(
                "Contract must have an end date to be renewed".to_string(),
            ))?;

        if new_end_date <= previous_end_date {
            return Err(AtlasError::ValidationFailed(
                "New end date must be after current end date".to_string(),
            ));
        }

        let renewal_number = contract.renewal_count + 1;

        info!("Renewing contract {} (renewal #{}) to {}",
            contract.contract_number, renewal_number, new_end_date);

        let renewal = self.repository.create_renewal(
            contract.organization_id, contract_id, renewal_number,
            previous_end_date, new_end_date,
            renewal_type, terms_changed, renewed_by, notes,
        ).await?;

        // Update contract dates and renewal count
        self.repository.update_contract_dates(
            contract_id, None, Some(new_end_date),
        ).await?;

        self.repository.update_contract_status(
            contract_id, "active", None, None, None, None,
        ).await?;

        self.repository.increment_renewal_count(contract_id).await?;

        Ok(renewal)
    }

    /// List renewals for a contract
    pub async fn list_renewals(&self, contract_id: Uuid) -> AtlasResult<Vec<ContractRenewal>> {
        self.repository.list_renewals(contract_id).await
    }

    // ========================================================================
    // Spend Tracking
    // ========================================================================

    /// Record spend against a contract
    pub async fn record_spend(
        &self,
        org_id: Uuid,
        contract_id: Uuid,
        contract_line_id: Option<Uuid>,
        source_type: &str,
        source_id: Option<Uuid>,
        source_number: Option<&str>,
        transaction_date: chrono::NaiveDate,
        amount: &str,
        quantity: Option<&str>,
        description: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ContractSpend> {
        let contract = self.repository.get_contract(contract_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Contract {} not found", contract_id)
            ))?;

        if contract.status != "active" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot record spend against contract in '{}' status", contract.status)
            ));
        }

        let amount_val: f64 = amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "Amount must be a valid number".to_string(),
        ))?;
        if amount_val <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Spend amount must be positive".to_string(),
            ));
        }

        info!("Recording spend of {} against contract {}", amount, contract.contract_number);

        let spend = self.repository.create_spend_entry(
            org_id, contract_id, contract_line_id,
            source_type, source_id, source_number,
            transaction_date, amount, quantity,
            description, created_by,
        ).await?;

        // Update contract released amount
        let all_spend = self.repository.list_spend_entries(contract_id).await?;
        let total_released: f64 = all_spend.iter()
            .map(|s| s.amount.parse::<f64>().unwrap_or(0.0))
            .sum();

        self.repository.update_contract_totals(
            contract_id, None,
            Some(&format!("{:.2}", total_released)),
            None, None, None,
        ).await?;

        Ok(spend)
    }

    /// List spend entries for a contract
    pub async fn list_spend_entries(&self, contract_id: Uuid) -> AtlasResult<Vec<ContractSpend>> {
        self.repository.list_spend_entries(contract_id).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get dashboard summary for procurement contracts
    pub async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<ContractDashboardSummary> {
        let all_contracts = self.repository.list_contracts(org_id, None, None).await?;

        let total_count = all_contracts.len() as i32;
        let active_count = all_contracts.iter().filter(|c| c.status == "active").count() as i32;

        let today = chrono::Utc::now().date_naive();
        let thirty_days = today + chrono::Duration::days(30);
        let expiring_count = all_contracts.iter()
            .filter(|c| {
                c.status == "active" &&
                c.end_date.map(|d| d <= thirty_days).unwrap_or(false)
            })
            .count() as i32;

        let mut total_committed = 0.0_f64;
        let mut total_released = 0.0_f64;

        for c in &all_contracts {
            if c.status == "active" {
                total_committed += c.total_committed_amount.parse::<f64>().unwrap_or(0.0);
                total_released += c.total_released_amount.parse::<f64>().unwrap_or(0.0);
            }
        }

        let utilization = if total_committed > 0.0 {
            format!("{:.1}", total_released / total_committed * 100.0)
        } else {
            "0.0".to_string()
        };

        // Group by status
        let mut by_status: std::collections::HashMap<String, i32> = std::collections::HashMap::new();
        for c in &all_contracts {
            *by_status.entry(c.status.clone()).or_insert(0) += 1;
        }
        let contracts_by_status: serde_json::Value = by_status.into_iter()
            .map(|(k, v)| serde_json::json!({"status": k, "count": v}))
            .collect();

        // Group by type
        let mut by_type: std::collections::HashMap<String, i32> = std::collections::HashMap::new();
        for c in &all_contracts {
            let key = c.contract_type_code.clone().unwrap_or_else(|| "untyped".to_string());
            *by_type.entry(key).or_insert(0) += 1;
        }
        let contracts_by_type: serde_json::Value = by_type.into_iter()
            .map(|(k, v)| serde_json::json!({"type": k, "count": v}))
            .collect();

        // Top suppliers by committed amount
        let mut by_supplier: std::collections::HashMap<String, (String, f64)> = std::collections::HashMap::new();
        for c in &all_contracts {
            if c.status == "active" {
                let entry = by_supplier.entry(c.supplier_id.to_string()).or_insert((
                    c.supplier_name.clone().unwrap_or_else(|| c.supplier_id.to_string()),
                    0.0,
                ));
                entry.1 += c.total_committed_amount.parse::<f64>().unwrap_or(0.0);
            }
        }
        let mut suppliers: Vec<_> = by_supplier.into_iter().map(|(id, (name, amt))| {
            serde_json::json!({"supplier_id": id, "supplier_name": name, "committed": format!("{:.2}", amt)})
        }).collect();
        suppliers.sort_by(|a, b| {
            let va: f64 = a["committed"].as_str().unwrap_or("0").parse().unwrap_or(0.0);
            let vb: f64 = b["committed"].as_str().unwrap_or("0").parse().unwrap_or(0.0);
            vb.partial_cmp(&va).unwrap_or(std::cmp::Ordering::Equal)
        });
        suppliers.truncate(10);

        Ok(ContractDashboardSummary {
            total_contracts: total_count,
            active_contracts: active_count,
            expiring_contracts_count: expiring_count,
            total_committed_amount: format!("{:.2}", total_committed),
            total_released_amount: format!("{:.2}", total_released),
            utilization_percent: utilization,
            contracts_by_status,
            contracts_by_type,
            top_suppliers: serde_json::Value::Array(suppliers),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_classifications() {
        assert!(VALID_CLASSIFICATIONS.contains(&"blanket"));
        assert!(VALID_CLASSIFICATIONS.contains(&"purchase_agreement"));
        assert!(VALID_CLASSIFICATIONS.contains(&"service"));
        assert!(VALID_CLASSIFICATIONS.contains(&"lease"));
        assert!(VALID_CLASSIFICATIONS.contains(&"other"));
    }

    #[test]
    fn test_valid_statuses() {
        assert!(VALID_STATUSES.contains(&"draft"));
        assert!(VALID_STATUSES.contains(&"pending_approval"));
        assert!(VALID_STATUSES.contains(&"active"));
        assert!(VALID_STATUSES.contains(&"expired"));
        assert!(VALID_STATUSES.contains(&"terminated"));
        assert!(VALID_STATUSES.contains(&"closed"));
    }

    #[test]
    fn test_valid_milestone_types() {
        assert!(VALID_MILESTONE_TYPES.contains(&"delivery"));
        assert!(VALID_MILESTONE_TYPES.contains(&"payment"));
        assert!(VALID_MILESTONE_TYPES.contains(&"review"));
        assert!(VALID_MILESTONE_TYPES.contains(&"acceptance"));
        assert!(VALID_MILESTONE_TYPES.contains(&"custom"));
    }

    #[test]
    fn test_valid_milestone_statuses() {
        assert!(VALID_MILESTONE_STATUSES.contains(&"pending"));
        assert!(VALID_MILESTONE_STATUSES.contains(&"in_progress"));
        assert!(VALID_MILESTONE_STATUSES.contains(&"completed"));
        assert!(VALID_MILESTONE_STATUSES.contains(&"overdue"));
        assert!(VALID_MILESTONE_STATUSES.contains(&"cancelled"));
    }

    #[test]
    fn test_valid_renewal_types() {
        assert!(VALID_RENEWAL_TYPES.contains(&"automatic"));
        assert!(VALID_RENEWAL_TYPES.contains(&"manual"));
        assert!(VALID_RENEWAL_TYPES.contains(&"negotiated"));
    }

    #[test]
    fn test_valid_price_types() {
        assert!(VALID_PRICE_TYPES.contains(&"fixed"));
        assert!(VALID_PRICE_TYPES.contains(&"variable"));
        assert!(VALID_PRICE_TYPES.contains(&"tiered"));
    }
}
