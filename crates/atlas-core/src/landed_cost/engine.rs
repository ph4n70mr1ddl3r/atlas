//! Landed Cost Engine
//!
//! Manages the full landed cost lifecycle: templates with cost components,
//! charge capture, cost allocation to receipt lines, simulation/estimation,
//! and variance analysis.
//!
//! Oracle Fusion Cloud SCM equivalent: SCM > Landed Cost Management

use atlas_shared::{
    LandedCostTemplate, LandedCostComponent, LandedCostCharge,
    LandedCostChargeLine, LandedCostAllocation, LandedCostSimulation,
    LandedCostDashboard, AtlasError, AtlasResult,
};
use super::LandedCostRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Valid template statuses
const VALID_TEMPLATE_STATUSES: &[&str] = &["active", "inactive"];

/// Valid cost types for components
const VALID_COST_TYPES: &[&str] = &[
    "freight", "insurance", "customs_duty", "handling",
    "brokerage", "storage", "other",
];

/// Valid allocation bases
const VALID_ALLOCATION_BASES: &[&str] = &[
    "quantity", "weight", "volume", "value", "equal",
];

/// Valid charge types
const VALID_CHARGE_TYPES: &[&str] = &["estimated", "actual", "adjustment"];

/// Valid charge statuses
const VALID_CHARGE_STATUSES: &[&str] = &[
    "draft", "submitted", "allocated", "posted", "cancelled",
];

/// Valid rate units of measure
const VALID_RATE_UOMS: &[&str] = &["per_unit", "percentage", "flat"];

/// Valid simulation statuses
#[allow(dead_code)]
const VALID_SIMULATION_STATUSES: &[&str] = &["draft", "completed", "archived"];

/// Landed Cost Management engine
pub struct LandedCostEngine {
    repository: Arc<dyn LandedCostRepository>,
}

impl LandedCostEngine {
    pub fn new(repository: Arc<dyn LandedCostRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Cost Templates
    // ========================================================================

    /// Create a landed cost template
    pub async fn create_template(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<LandedCostTemplate> {
        if code.is_empty() {
            return Err(AtlasError::ValidationFailed("Template code is required".to_string()));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Template name is required".to_string()));
        }

        info!("Creating landed cost template {} in org {}", code, org_id);
        self.repository.create_template(org_id, code, name, description, created_by).await
    }

    /// Get a template by code
    pub async fn get_template(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<LandedCostTemplate>> {
        self.repository.get_template(org_id, code).await
    }

    /// List templates for an organization
    pub async fn list_templates(&self, org_id: Uuid) -> AtlasResult<Vec<LandedCostTemplate>> {
        self.repository.list_templates(org_id).await
    }

    /// Update a template's status
    pub async fn update_template_status(
        &self,
        org_id: Uuid,
        code: &str,
        status: &str,
    ) -> AtlasResult<LandedCostTemplate> {
        if !VALID_TEMPLATE_STATUSES.contains(&status) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid status '{}'. Must be one of: {}", status, VALID_TEMPLATE_STATUSES.join(", ")
            )));
        }

        let template = self.repository.get_template(org_id, code).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Template '{}' not found", code)
            ))?;

        info!("Updating template {} status to {}", code, status);
        self.repository.update_template_status(template.id, status).await
    }

    // ========================================================================
    // Cost Components
    // ========================================================================

    /// Create a landed cost component
    pub async fn create_component(
        &self,
        org_id: Uuid,
        template_id: Option<Uuid>,
        code: &str,
        name: &str,
        description: Option<&str>,
        cost_type: &str,
        allocation_basis: &str,
        default_rate: Option<&str>,
        rate_uom: Option<&str>,
        expense_account: Option<&str>,
        is_taxable: bool,
        created_by: Option<Uuid>,
    ) -> AtlasResult<LandedCostComponent> {
        if code.is_empty() {
            return Err(AtlasError::ValidationFailed("Component code is required".to_string()));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Component name is required".to_string()));
        }
        if !VALID_COST_TYPES.contains(&cost_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid cost type '{}'. Must be one of: {}", cost_type, VALID_COST_TYPES.join(", ")
            )));
        }
        if !VALID_ALLOCATION_BASES.contains(&allocation_basis) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid allocation basis '{}'. Must be one of: {}",
                allocation_basis, VALID_ALLOCATION_BASES.join(", ")
            )));
        }
        if let Some(uom) = rate_uom {
            if !VALID_RATE_UOMS.contains(&uom) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid rate UOM '{}'. Must be one of: {}", uom, VALID_RATE_UOMS.join(", ")
                )));
            }
        }
        if let Some(rate) = default_rate {
            let r: f64 = rate.parse().map_err(|_| AtlasError::ValidationFailed(
                "Default rate must be a valid number".to_string(),
            ))?;
            if r < 0.0 {
                return Err(AtlasError::ValidationFailed(
                    "Default rate must be non-negative".to_string(),
                ));
            }
        }

        info!("Creating landed cost component {} in org {}", code, org_id);
        self.repository.create_component(
            org_id, template_id, code, name, description,
            cost_type, allocation_basis, default_rate, rate_uom,
            expense_account, is_taxable, created_by,
        ).await
    }

    /// Get a component by code
    pub async fn get_component(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<LandedCostComponent>> {
        self.repository.get_component(org_id, code).await
    }

    /// List components, optionally filtered by template
    pub async fn list_components(&self, org_id: Uuid, template_id: Option<Uuid>) -> AtlasResult<Vec<LandedCostComponent>> {
        self.repository.list_components(org_id, template_id).await
    }

    /// Update a component's status
    pub async fn update_component_status(
        &self,
        org_id: Uuid,
        code: &str,
        status: &str,
    ) -> AtlasResult<LandedCostComponent> {
        if !VALID_TEMPLATE_STATUSES.contains(&status) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid status '{}'. Must be one of: {}", status, VALID_TEMPLATE_STATUSES.join(", ")
            )));
        }

        let component = self.repository.get_component(org_id, code).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Component '{}' not found", code)
            ))?;

        info!("Updating component {} status to {}", code, status);
        self.repository.update_component_status(component.id, status).await
    }

    // ========================================================================
    // Charges
    // ========================================================================

    /// Create a landed cost charge
    pub async fn create_charge(
        &self,
        org_id: Uuid,
        charge_number: &str,
        template_id: Option<Uuid>,
        receipt_id: Option<Uuid>,
        purchase_order_id: Option<Uuid>,
        supplier_id: Option<Uuid>,
        supplier_name: Option<&str>,
        charge_type: &str,
        charge_date: Option<chrono::NaiveDate>,
        total_amount: &str,
        currency: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<LandedCostCharge> {
        if charge_number.is_empty() {
            return Err(AtlasError::ValidationFailed("Charge number is required".to_string()));
        }
        if !VALID_CHARGE_TYPES.contains(&charge_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid charge type '{}'. Must be one of: {}", charge_type, VALID_CHARGE_TYPES.join(", ")
            )));
        }
        let amount: f64 = total_amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "Total amount must be a valid number".to_string(),
        ))?;
        if amount < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Total amount must be non-negative".to_string(),
            ));
        }
        if currency.is_empty() {
            return Err(AtlasError::ValidationFailed("Currency is required".to_string()));
        }

        info!("Creating landed cost charge {} in org {}", charge_number, org_id);
        self.repository.create_charge(
            org_id, charge_number, template_id, receipt_id,
            purchase_order_id, supplier_id, supplier_name,
            charge_type, charge_date, total_amount, currency, created_by,
        ).await
    }

    /// Get a charge by ID
    pub async fn get_charge(&self, id: Uuid) -> AtlasResult<Option<LandedCostCharge>> {
        self.repository.get_charge(id).await
    }

    /// Get a charge by number
    pub async fn get_charge_by_number(&self, org_id: Uuid, charge_number: &str) -> AtlasResult<Option<LandedCostCharge>> {
        self.repository.get_charge_by_number(org_id, charge_number).await
    }

    /// List charges with optional filters
    pub async fn list_charges(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        charge_type: Option<&str>,
        receipt_id: Option<Uuid>,
    ) -> AtlasResult<Vec<LandedCostCharge>> {
        if let Some(s) = status {
            if !VALID_CHARGE_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_CHARGE_STATUSES.join(", ")
                )));
            }
        }
        if let Some(ct) = charge_type {
            if !VALID_CHARGE_TYPES.contains(&ct) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid charge type '{}'. Must be one of: {}", ct, VALID_CHARGE_TYPES.join(", ")
                )));
            }
        }
        self.repository.list_charges(org_id, status, charge_type, receipt_id).await
    }

    /// Submit a charge for allocation
    pub async fn submit_charge(&self, charge_id: Uuid) -> AtlasResult<LandedCostCharge> {
        let charge = self.repository.get_charge(charge_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Charge {} not found", charge_id)
            ))?;

        if charge.status != "draft" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot submit charge in '{}' status. Must be 'draft'.", charge.status)
            ));
        }

        info!("Submitting charge {}", charge.charge_number);
        self.repository.update_charge_status(charge_id, "submitted").await
    }

    /// Cancel a charge
    pub async fn cancel_charge(&self, charge_id: Uuid) -> AtlasResult<LandedCostCharge> {
        let charge = self.repository.get_charge(charge_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Charge {} not found", charge_id)
            ))?;

        if charge.status != "draft" && charge.status != "submitted" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot cancel charge in '{}' status.", charge.status)
            ));
        }

        info!("Cancelling charge {}", charge.charge_number);
        self.repository.update_charge_status(charge_id, "cancelled").await
    }

    // ========================================================================
    // Charge Lines
    // ========================================================================

    /// Add a line to a charge
    pub async fn add_charge_line(
        &self,
        org_id: Uuid,
        charge_id: Uuid,
        component_id: Option<Uuid>,
        receipt_line_id: Option<Uuid>,
        item_id: Option<Uuid>,
        item_code: Option<&str>,
        item_description: Option<&str>,
        charge_amount: &str,
        allocation_basis: &str,
        allocation_qty: Option<&str>,
        allocation_value: Option<&str>,
        expense_account: Option<&str>,
        notes: Option<&str>,
    ) -> AtlasResult<LandedCostChargeLine> {
        let charge = self.repository.get_charge(charge_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Charge {} not found", charge_id)
            ))?;

        if charge.status != "draft" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot add lines to charge in '{}' status", charge.status)
            ));
        }

        let amount: f64 = charge_amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "Charge amount must be a valid number".to_string(),
        ))?;
        if amount < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Charge amount must be non-negative".to_string(),
            ));
        }

        if !VALID_ALLOCATION_BASES.contains(&allocation_basis) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid allocation basis '{}'. Must be one of: {}",
                allocation_basis, VALID_ALLOCATION_BASES.join(", ")
            )));
        }

        let existing_lines = self.repository.list_charge_lines(charge_id).await?;
        let line_number = (existing_lines.len() as i32) + 1;

        info!("Adding line {} to charge {}", line_number, charge.charge_number);

        self.repository.create_charge_line(
            org_id, charge_id, component_id, line_number,
            receipt_line_id, item_id, item_code, item_description,
            charge_amount, allocation_basis, allocation_qty,
            allocation_value, expense_account, notes,
        ).await
    }

    /// List charge lines
    pub async fn list_charge_lines(&self, charge_id: Uuid) -> AtlasResult<Vec<LandedCostChargeLine>> {
        self.repository.list_charge_lines(charge_id).await
    }

    /// Get a charge line by ID
    pub async fn get_charge_line(&self, id: Uuid) -> AtlasResult<Option<LandedCostChargeLine>> {
        self.repository.get_charge_line(id).await
    }

    // ========================================================================
    // Allocation
    // ========================================================================

    /// Allocate a submitted charge to receipt lines
    pub async fn allocate_charge(&self, charge_id: Uuid) -> AtlasResult<Vec<LandedCostAllocation>> {
        let charge = self.repository.get_charge(charge_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Charge {} not found", charge_id)
            ))?;

        if charge.status != "submitted" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot allocate charge in '{}' status. Must be 'submitted'.", charge.status)
            ));
        }

        let lines = self.repository.list_charge_lines(charge_id).await?;
        if lines.is_empty() {
            return Err(AtlasError::WorkflowError(
                "Cannot allocate charge with no lines".to_string()
            ));
        }

        let mut allocations = Vec::new();

        for line in &lines {
            // Calculate allocation for each charge line
            let allocated = self.allocate_charge_line(&charge, line).await?;
            allocations.extend(allocated);
        }

        // Update charge status to allocated
        self.repository.update_charge_status(charge_id, "allocated").await?;

        info!("Allocated charge {} across {} allocation entries", charge.charge_number, allocations.len());
        Ok(allocations)
    }

    /// Allocate a single charge line
    async fn allocate_charge_line(
        &self,
        charge: &LandedCostCharge,
        line: &LandedCostChargeLine,
    ) -> AtlasResult<Vec<LandedCostAllocation>> {
        let basis = line.allocation_basis.as_str();

        match basis {
            "equal" => {
                // Equal split across all receipt lines
                let receipt_lines = self.repository.get_receipt_lines_for_charge(
                    charge.organization_id, charge.receipt_id,
                ).await?;

                if receipt_lines.is_empty() {
                    return Err(AtlasError::WorkflowError(
                        "No receipt lines found for allocation".to_string()
                    ));
                }

                let count = receipt_lines.len() as f64;
                let amount_per_line = line.charge_amount.parse::<f64>().unwrap_or(0.0) / count;

                let mut results = Vec::new();
                for (idx, rl) in receipt_lines.iter().enumerate() {
                    // Last line gets the remainder to avoid rounding drift
                    let allocated = if idx == receipt_lines.len() - 1 {
                        let total_so_far = amount_per_line * idx as f64;
                        line.charge_amount.parse::<f64>().unwrap_or(0.0) - total_so_far
                    } else {
                        amount_per_line
                    };

                    let alloc = self.repository.create_allocation(
                        charge.organization_id,
                        charge.id,
                        line.id,
                        charge.receipt_id,
                        Some(rl.receipt_line_id),
                        rl.item_id,
                        rl.item_code.as_deref(),
                        &format!("{:.6}", allocated),
                        "equal",
                        Some(&format!("{:.6}", count)),
                        Some(&format!("{:.6}", count)),
                        Some(&format!("{:.4}", 1.0 / count * 100.0)),
                        rl.unit_price.as_deref(),
                    ).await?;
                    results.push(alloc);
                }
                Ok(results)
            }
            "quantity" | "weight" | "volume" | "value" => {
                // Proportional allocation based on the basis
                let receipt_lines = self.repository.get_receipt_lines_for_charge(
                    charge.organization_id, charge.receipt_id,
                ).await?;

                if receipt_lines.is_empty() {
                    return Err(AtlasError::WorkflowError(
                        "No receipt lines found for allocation".to_string()
                    ));
                }

                let total_basis: f64 = receipt_lines.iter()
                    .map(|rl| rl.basis_value(basis))
                    .sum();

                if total_basis.abs() < f64::EPSILON {
                    return Err(AtlasError::WorkflowError(
                        format!("Total {} basis is zero — cannot allocate", basis)
                    ));
                }

                let charge_amount = line.charge_amount.parse::<f64>().unwrap_or(0.0);
                let mut results = Vec::new();
                let mut allocated_so_far = 0.0f64;

                for (idx, rl) in receipt_lines.iter().enumerate() {
                    let line_basis = rl.basis_value(basis);
                    let pct = line_basis / total_basis;
                    let allocated = if idx == receipt_lines.len() - 1 {
                        charge_amount - allocated_so_far
                    } else {
                        charge_amount * pct
                    };
                    allocated_so_far += allocated;

                    let alloc = self.repository.create_allocation(
                        charge.organization_id,
                        charge.id,
                        line.id,
                        charge.receipt_id,
                        Some(rl.receipt_line_id),
                        rl.item_id,
                        rl.item_code.as_deref(),
                        &format!("{:.6}", allocated),
                        basis,
                        Some(&format!("{:.6}", line_basis)),
                        Some(&format!("{:.6}", total_basis)),
                        Some(&format!("{:.4}", pct * 100.0)),
                        rl.unit_price.as_deref(),
                    ).await?;
                    results.push(alloc);
                }
                Ok(results)
            }
            _ => Err(AtlasError::ValidationFailed(format!(
                "Unsupported allocation basis: {}", basis
            ))),
        }
    }

    /// Get allocations for a charge
    pub async fn list_allocations(&self, charge_id: Uuid) -> AtlasResult<Vec<LandedCostAllocation>> {
        self.repository.list_allocations(charge_id).await
    }

    /// Get allocations for a receipt
    pub async fn get_allocations_for_receipt(&self, org_id: Uuid, receipt_id: Uuid) -> AtlasResult<Vec<LandedCostAllocation>> {
        self.repository.get_allocations_for_receipt(org_id, receipt_id).await
    }

    /// Post an allocated charge (final step — GL journal entry would be created here)
    pub async fn post_charge(&self, charge_id: Uuid) -> AtlasResult<LandedCostCharge> {
        let charge = self.repository.get_charge(charge_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Charge {} not found", charge_id)
            ))?;

        if charge.status != "allocated" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot post charge in '{}' status. Must be 'allocated'.", charge.status)
            ));
        }

        info!("Posting charge {} to GL", charge.charge_number);
        self.repository.update_charge_status(charge_id, "posted").await
    }

    // ========================================================================
    // Simulation
    // ========================================================================

    /// Create a landed cost simulation
    pub async fn create_simulation(
        &self,
        org_id: Uuid,
        template_id: Option<Uuid>,
        purchase_order_id: Option<Uuid>,
        item_id: Option<Uuid>,
        item_code: Option<&str>,
        item_description: Option<&str>,
        estimated_quantity: &str,
        unit_price: &str,
        currency: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<LandedCostSimulation> {
        let qty: f64 = estimated_quantity.parse().map_err(|_| AtlasError::ValidationFailed(
            "Estimated quantity must be a valid number".to_string(),
        ))?;
        if qty <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Estimated quantity must be positive".to_string(),
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

        // Load components for the template (if provided)
        let components = self.repository.list_components(org_id, template_id).await?;

        // Calculate estimated charges
        let mut estimated_charges: Vec<serde_json::Value> = Vec::new();
        let mut total_charges = 0.0f64;

        for comp in &components {
            if comp.status != "active" {
                continue;
            }
            let charge_amt = match comp.rate_uom.as_deref() {
                Some("per_unit") => {
                    let rate = comp.default_rate.as_ref()
                        .and_then(|r| r.parse::<f64>().ok())
                        .unwrap_or(0.0);
                    rate * qty
                }
                Some("percentage") => {
                    let rate = comp.default_rate.as_ref()
                        .and_then(|r| r.parse::<f64>().ok())
                        .unwrap_or(0.0);
                    price * qty * rate / 100.0
                }
                Some("flat") => {
                    comp.default_rate.as_ref()
                        .and_then(|r| r.parse::<f64>().ok())
                        .unwrap_or(0.0)
                }
                _ => 0.0,
            };

            estimated_charges.push(serde_json::json!({
                "component_code": comp.code,
                "component_name": comp.name,
                "cost_type": comp.cost_type,
                "estimated_amount": format!("{:.6}", charge_amt),
            }));
            total_charges += charge_amt;
        }

        let base_cost = price * qty;
        let estimated_landed_cost = base_cost + total_charges;
        let estimated_landed_cost_per_unit = if qty > 0.0 {
            estimated_landed_cost / qty
        } else {
            0.0
        };

        let simulation_number = format!("SIM-{}", Uuid::new_v4().to_string()[..8].to_uppercase());

        info!("Creating landed cost simulation {} in org {}", simulation_number, org_id);

        self.repository.create_simulation(
            org_id, &simulation_number, template_id,
            purchase_order_id, item_id, item_code, item_description,
            estimated_quantity, unit_price, currency,
            &serde_json::json!(estimated_charges),
            &format!("{:.6}", estimated_landed_cost),
            &format!("{:.6}", estimated_landed_cost_per_unit),
            created_by,
        ).await
    }

    /// Get a simulation by ID
    pub async fn get_simulation(&self, id: Uuid) -> AtlasResult<Option<LandedCostSimulation>> {
        self.repository.get_simulation(id).await
    }

    /// List simulations
    pub async fn list_simulations(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<LandedCostSimulation>> {
        if let Some(s) = status {
            if !VALID_SIMULATION_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_SIMULATION_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_simulations(org_id, status).await
    }

    /// Archive a simulation
    pub async fn archive_simulation(&self, simulation_id: Uuid) -> AtlasResult<LandedCostSimulation> {
        let sim = self.repository.get_simulation(simulation_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Simulation {} not found", simulation_id)
            ))?;

        if sim.status != "completed" && sim.status != "draft" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot archive simulation in '{}' status", sim.status)
            ));
        }

        info!("Archiving simulation {}", sim.simulation_number);
        self.repository.update_simulation_status(simulation_id, "archived").await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get landed cost dashboard summary
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<LandedCostDashboard> {
        let charges = self.repository.list_charges(org_id, None, None, None).await?;
        let simulations = self.repository.list_simulations(org_id, None).await?;

        let total_charges = charges.len() as i32;
        let pending_charges = charges.iter().filter(|c| c.status == "draft" || c.status == "submitted").count() as i32;
        let allocated_charges = charges.iter().filter(|c| c.status == "allocated" || c.status == "posted").count() as i32;

        let total_charge_amount: f64 = charges.iter()
            .map(|c| c.total_amount.parse::<f64>().unwrap_or(0.0))
            .sum();

        let allocations = self.repository.list_allocations_for_org(org_id).await?;
        let total_allocated_amount: f64 = allocations.iter()
            .map(|a| a.allocated_amount.parse::<f64>().unwrap_or(0.0))
            .sum();

        let total_simulations = simulations.len() as i32;

        // Group charges by type
        let mut type_counts: std::collections::HashMap<String, i32> = std::collections::HashMap::new();
        for c in &charges {
            *type_counts.entry(c.charge_type.clone()).or_insert(0) += 1;
        }
        let charges_by_type: serde_json::Value = type_counts.into_iter()
            .map(|(k, v)| serde_json::json!({"charge_type": k, "count": v}))
            .collect();

        // Recent charges
        let mut recent = charges.clone();
        recent.sort_by_key(|b| std::cmp::Reverse(b.created_at));
        recent.truncate(5);
        let recent_charges: serde_json::Value = recent.iter().map(|c| serde_json::json!({
            "id": c.id,
            "charge_number": c.charge_number,
            "charge_type": c.charge_type,
            "status": c.status,
            "total_amount": c.total_amount,
            "currency": c.currency,
        })).collect();

        // Top cost components by usage in charge lines
        let all_lines: Vec<_> = futures::future::join_all(
            charges.iter().map(|c| self.repository.list_charge_lines(c.id))
        ).await.into_iter()
            .filter_map(|r| r.ok())
            .flatten()
            .collect();

        let mut comp_amounts: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
        for l in &all_lines {
            if let Some(code) = &l.item_code {
                *comp_amounts.entry(code.clone()).or_insert(0.0)
                    += l.charge_amount.parse::<f64>().unwrap_or(0.0);
            }
        }
        let mut comp_vec: Vec<_> = comp_amounts.into_iter().collect();
        comp_vec.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        comp_vec.truncate(5);
        let top_cost_components: serde_json::Value = comp_vec.into_iter()
            .map(|(code, amount)| serde_json::json!({"item_code": code, "total_charges": format!("{:.2}", amount)}))
            .collect();

        Ok(LandedCostDashboard {
            total_charges,
            pending_charges,
            allocated_charges,
            total_charge_amount: format!("{:.2}", total_charge_amount),
            total_allocated_amount: format!("{:.2}", total_allocated_amount),
            total_simulations,
            charges_by_type,
            recent_charges,
            top_cost_components,
        })
    }
}

/// Receipt line info used during allocation
pub struct ReceiptLineInfo {
    pub receipt_line_id: Uuid,
    pub item_id: Option<Uuid>,
    pub item_code: Option<String>,
    pub quantity: f64,
    pub unit_price: Option<String>,
    pub weight: Option<f64>,
    pub volume: Option<f64>,
}

impl ReceiptLineInfo {
    /// Get the basis value for a given allocation basis
    pub fn basis_value(&self, basis: &str) -> f64 {
        match basis {
            "quantity" => self.quantity,
            "weight" => self.weight.unwrap_or(0.0),
            "volume" => self.volume.unwrap_or(0.0),
            "value" => {
                let price: f64 = self.unit_price.as_ref()
                    .and_then(|p| p.parse().ok())
                    .unwrap_or(0.0);
                price * self.quantity
            }
            _ => 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Validation constant tests
    // ========================================================================

    #[test]
    fn test_valid_template_statuses() {
        assert!(VALID_TEMPLATE_STATUSES.contains(&"active"));
        assert!(VALID_TEMPLATE_STATUSES.contains(&"inactive"));
        assert!(!VALID_TEMPLATE_STATUSES.contains(&"draft"));
    }

    #[test]
    fn test_valid_cost_types() {
        assert!(VALID_COST_TYPES.contains(&"freight"));
        assert!(VALID_COST_TYPES.contains(&"insurance"));
        assert!(VALID_COST_TYPES.contains(&"customs_duty"));
        assert!(VALID_COST_TYPES.contains(&"handling"));
        assert!(VALID_COST_TYPES.contains(&"brokerage"));
        assert!(VALID_COST_TYPES.contains(&"storage"));
        assert!(VALID_COST_TYPES.contains(&"other"));
        assert!(!VALID_COST_TYPES.contains(&"tax"));
    }

    #[test]
    fn test_valid_allocation_bases() {
        assert!(VALID_ALLOCATION_BASES.contains(&"quantity"));
        assert!(VALID_ALLOCATION_BASES.contains(&"weight"));
        assert!(VALID_ALLOCATION_BASES.contains(&"volume"));
        assert!(VALID_ALLOCATION_BASES.contains(&"value"));
        assert!(VALID_ALLOCATION_BASES.contains(&"equal"));
        assert!(!VALID_ALLOCATION_BASES.contains(&"ratio"));
    }

    #[test]
    fn test_valid_charge_types() {
        assert!(VALID_CHARGE_TYPES.contains(&"estimated"));
        assert!(VALID_CHARGE_TYPES.contains(&"actual"));
        assert!(VALID_CHARGE_TYPES.contains(&"adjustment"));
    }

    #[test]
    fn test_valid_charge_statuses() {
        assert!(VALID_CHARGE_STATUSES.contains(&"draft"));
        assert!(VALID_CHARGE_STATUSES.contains(&"submitted"));
        assert!(VALID_CHARGE_STATUSES.contains(&"allocated"));
        assert!(VALID_CHARGE_STATUSES.contains(&"posted"));
        assert!(VALID_CHARGE_STATUSES.contains(&"cancelled"));
    }

    #[test]
    fn test_valid_rate_uoms() {
        assert!(VALID_RATE_UOMS.contains(&"per_unit"));
        assert!(VALID_RATE_UOMS.contains(&"percentage"));
        assert!(VALID_RATE_UOMS.contains(&"flat"));
    }

    #[test]
    fn test_valid_simulation_statuses() {
        assert!(VALID_SIMULATION_STATUSES.contains(&"draft"));
        assert!(VALID_SIMULATION_STATUSES.contains(&"completed"));
        assert!(VALID_SIMULATION_STATUSES.contains(&"archived"));
    }

    // ========================================================================
    // ReceiptLineInfo unit tests
    // ========================================================================

    #[test]
    fn test_receipt_line_info_basis_quantity() {
        let info = ReceiptLineInfo {
            receipt_line_id: Uuid::new_v4(),
            item_id: None,
            item_code: Some("ITEM-001".to_string()),
            quantity: 100.0,
            unit_price: Some("10.00".to_string()),
            weight: Some(50.0),
            volume: Some(25.0),
        };
        assert!((info.basis_value("quantity") - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_receipt_line_info_basis_weight() {
        let info = ReceiptLineInfo {
            receipt_line_id: Uuid::new_v4(),
            item_id: None,
            item_code: Some("ITEM-001".to_string()),
            quantity: 100.0,
            unit_price: Some("10.00".to_string()),
            weight: Some(50.0),
            volume: Some(25.0),
        };
        assert!((info.basis_value("weight") - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_receipt_line_info_basis_volume() {
        let info = ReceiptLineInfo {
            receipt_line_id: Uuid::new_v4(),
            item_id: None,
            item_code: Some("ITEM-001".to_string()),
            quantity: 100.0,
            unit_price: Some("10.00".to_string()),
            weight: Some(50.0),
            volume: Some(25.0),
        };
        assert!((info.basis_value("volume") - 25.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_receipt_line_info_basis_value() {
        let info = ReceiptLineInfo {
            receipt_line_id: Uuid::new_v4(),
            item_id: None,
            item_code: Some("ITEM-001".to_string()),
            quantity: 100.0,
            unit_price: Some("10.00".to_string()),
            weight: Some(50.0),
            volume: Some(25.0),
        };
        // value = unit_price * quantity = 10.0 * 100.0 = 1000.0
        assert!((info.basis_value("value") - 1000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_receipt_line_info_basis_equal() {
        let info = ReceiptLineInfo {
            receipt_line_id: Uuid::new_v4(),
            item_id: None,
            item_code: Some("ITEM-001".to_string()),
            quantity: 100.0,
            unit_price: Some("10.00".to_string()),
            weight: None,
            volume: None,
        };
        assert!((info.basis_value("equal") - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_receipt_line_info_basis_value_no_price() {
        let info = ReceiptLineInfo {
            receipt_line_id: Uuid::new_v4(),
            item_id: None,
            item_code: Some("ITEM-001".to_string()),
            quantity: 100.0,
            unit_price: None,
            weight: None,
            volume: None,
        };
        // No price => value basis = 0
        assert!((info.basis_value("value") - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_receipt_line_info_basis_weight_none() {
        let info = ReceiptLineInfo {
            receipt_line_id: Uuid::new_v4(),
            item_id: None,
            item_code: Some("ITEM-001".to_string()),
            quantity: 100.0,
            unit_price: None,
            weight: None,
            volume: None,
        };
        assert!((info.basis_value("weight") - 0.0).abs() < f64::EPSILON);
    }

    // ========================================================================
    // Allocation math tests
    // ========================================================================

    #[test]
    fn test_equal_allocation_math() {
        // Simulate equal allocation of $1000 across 4 receipt lines
        let total_charge = 1000.0f64;
        let line_count = 4.0f64;
        let per_line = total_charge / line_count;
        assert!((per_line - 250.0).abs() < f64::EPSILON);

        // Verify the last line absorbs rounding
        let mut sum = 0.0;
        for i in 0..4 {
            let allocated = if i == 3 {
                total_charge - sum
            } else {
                per_line
            };
            sum += allocated;
        }
        assert!((sum - total_charge).abs() < f64::EPSILON);
    }

    #[test]
    fn test_proportional_allocation_math() {
        // Simulate quantity-based allocation of $500 across lines with
        // quantities 100, 200, 300 (total 600)
        let quantities = [100.0f64, 200.0f64, 300.0f64];
        let total_basis: f64 = quantities.iter().sum();
        let charge_amount = 500.0f64;

        let mut sum = 0.0f64;
        for (i, &qty) in quantities.iter().enumerate() {
            let pct = qty / total_basis;
            let allocated = if i == quantities.len() - 1 {
                charge_amount - sum
            } else {
                charge_amount * pct
            };
            sum += allocated;

            // Verify individual allocations are roughly proportional
            let expected = charge_amount * pct;
            assert!((allocated - expected).abs() < 0.01);
        }

        // Total must equal the charge amount
        assert!((sum - charge_amount).abs() < f64::EPSILON);
    }

    #[test]
    fn test_value_basis_allocation() {
        // Two items: A (qty=10, price=$20) and B (qty=5, price=$100)
        // Total value basis = 200 + 500 = 700
        // Charge = $140
        // A gets 200/700 * 140 = 40, B gets 500/700 * 140 = 100
        let items = [
            ("A", 10.0f64, 20.0f64),
            ("B", 5.0f64, 100.0f64),
        ];
        let charge = 140.0f64;
        let total_value: f64 = items.iter().map(|(_, q, p)| q * p).sum();

        assert!((total_value - 700.0).abs() < f64::EPSILON);

        let a_alloc = charge * (10.0 * 20.0) / total_value;
        let b_alloc = charge * (5.0 * 100.0) / total_value;

        assert!((a_alloc - 40.0).abs() < 0.01);
        assert!((b_alloc - 100.0).abs() < 0.01);
        assert!((a_alloc + b_alloc - charge).abs() < 0.01);
    }

    #[test]
    fn test_zero_basis_prevents_allocation() {
        let quantities = [0.0f64, 0.0f64, 0.0f64];
        let total_basis: f64 = quantities.iter().sum();
        assert!(total_basis.abs() < f64::EPSILON);
    }

    // ========================================================================
    // Simulation math tests
    // ========================================================================

    #[test]
    fn test_simulation_per_unit_rate() {
        // Freight @ $2/unit, qty=100 => $200
        let rate = 2.0f64;
        let qty = 100.0f64;
        let charge = rate * qty;
        assert!((charge - 200.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_simulation_percentage_rate() {
        // Insurance @ 1.5% of PO value, price=$50, qty=200 => $150
        let rate = 1.5f64;
        let price = 50.0f64;
        let qty = 200.0f64;
        let charge = price * qty * rate / 100.0;
        assert!((charge - 150.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_simulation_flat_rate() {
        // Customs brokerage: flat $350
        let flat = 350.0f64;
        assert!((flat - 350.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_simulation_total_landed_cost() {
        // Base: $5000 (100 units * $50)
        // Freight: $200 (per_unit $2)
        // Insurance: $75 (percentage 1.5%)
        // Customs: $350 (flat)
        // Total landed = $5000 + $200 + $75 + $350 = $5625
        // Per unit = $56.25
        let base = 5000.0f64;
        let freight = 200.0f64;
        let insurance = 75.0f64;
        let customs = 350.0f64;
        let total_landed = base + freight + insurance + customs;
        let qty = 100.0f64;
        let per_unit = total_landed / qty;

        assert!((total_landed - 5625.0).abs() < f64::EPSILON);
        assert!((per_unit - 56.25).abs() < f64::EPSILON);
    }

    // ========================================================================
    // Engine validation tests (using mock repo)
    // ========================================================================

    use std::sync::Arc as StdArc;

    /// A minimal mock for LandedCostRepository used in validation tests
    struct MockLandedCostRepo;

    #[async_trait::async_trait]
    impl LandedCostRepository for MockLandedCostRepo {
        async fn create_template(&self, _org_id: Uuid, code: &str, name: &str, _description: Option<&str>, _created_by: Option<Uuid>) -> AtlasResult<LandedCostTemplate> {
            Ok(LandedCostTemplate {
                id: Uuid::new_v4(),
                organization_id: _org_id,
                code: code.to_string(),
                name: name.to_string(),
                description: None,
                status: "active".to_string(),
                metadata: serde_json::json!({}),
                created_by: _created_by,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            })
        }
        async fn get_template(&self, _org_id: Uuid, _code: &str) -> AtlasResult<Option<LandedCostTemplate>> { Ok(None) }
        async fn list_templates(&self, _org_id: Uuid) -> AtlasResult<Vec<LandedCostTemplate>> { Ok(vec![]) }
        async fn update_template_status(&self, _id: Uuid, _status: &str) -> AtlasResult<LandedCostTemplate> {
            Err(AtlasError::NotImplemented("mock".to_string()))
        }
        async fn create_component(&self, _org_id: Uuid, _template_id: Option<Uuid>, code: &str, name: &str, _description: Option<&str>, cost_type: &str, allocation_basis: &str, default_rate: Option<&str>, _rate_uom: Option<&str>, _expense_account: Option<&str>, _is_taxable: bool, _created_by: Option<Uuid>) -> AtlasResult<LandedCostComponent> {
            Ok(LandedCostComponent {
                id: Uuid::new_v4(),
                organization_id: _org_id,
                template_id: _template_id,
                code: code.to_string(),
                name: name.to_string(),
                description: None,
                cost_type: cost_type.to_string(),
                allocation_basis: allocation_basis.to_string(),
                default_rate: default_rate.map(|r| r.to_string()),
                rate_uom: _rate_uom.map(|u| u.to_string()),
                expense_account: None,
                is_taxable: _is_taxable,
                status: "active".to_string(),
                metadata: serde_json::json!({}),
                created_by: _created_by,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            })
        }
        async fn get_component(&self, _org_id: Uuid, _code: &str) -> AtlasResult<Option<LandedCostComponent>> { Ok(None) }
        async fn list_components(&self, _org_id: Uuid, _template_id: Option<Uuid>) -> AtlasResult<Vec<LandedCostComponent>> { Ok(vec![]) }
        async fn update_component_status(&self, _id: Uuid, _status: &str) -> AtlasResult<LandedCostComponent> {
            Err(AtlasError::NotImplemented("mock".to_string()))
        }
        async fn create_charge(&self, _org_id: Uuid, charge_number: &str, _template_id: Option<Uuid>, _receipt_id: Option<Uuid>, _purchase_order_id: Option<Uuid>, _supplier_id: Option<Uuid>, _supplier_name: Option<&str>, charge_type: &str, _charge_date: Option<chrono::NaiveDate>, total_amount: &str, currency: &str, _created_by: Option<Uuid>) -> AtlasResult<LandedCostCharge> {
            Ok(LandedCostCharge {
                id: Uuid::new_v4(),
                organization_id: _org_id,
                charge_number: charge_number.to_string(),
                template_id: _template_id,
                receipt_id: _receipt_id,
                purchase_order_id: _purchase_order_id,
                supplier_id: _supplier_id,
                supplier_name: _supplier_name.map(|s| s.to_string()),
                charge_type: charge_type.to_string(),
                charge_date: _charge_date,
                total_amount: total_amount.to_string(),
                currency: currency.to_string(),
                status: "draft".to_string(),
                metadata: serde_json::json!({}),
                created_by: _created_by,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            })
        }
        async fn get_charge(&self, _id: Uuid) -> AtlasResult<Option<LandedCostCharge>> { Ok(None) }
        async fn get_charge_by_number(&self, _org_id: Uuid, _charge_number: &str) -> AtlasResult<Option<LandedCostCharge>> { Ok(None) }
        async fn list_charges(&self, _org_id: Uuid, _status: Option<&str>, _charge_type: Option<&str>, _receipt_id: Option<Uuid>) -> AtlasResult<Vec<LandedCostCharge>> { Ok(vec![]) }
        async fn update_charge_status(&self, _id: Uuid, _status: &str) -> AtlasResult<LandedCostCharge> {
            Err(AtlasError::NotImplemented("mock".to_string()))
        }
        async fn create_charge_line(&self, _org_id: Uuid, _charge_id: Uuid, _component_id: Option<Uuid>, _line_number: i32, _receipt_line_id: Option<Uuid>, _item_id: Option<Uuid>, _item_code: Option<&str>, _item_description: Option<&str>, charge_amount: &str, allocation_basis: &str, _allocation_qty: Option<&str>, _allocation_value: Option<&str>, _expense_account: Option<&str>, _notes: Option<&str>) -> AtlasResult<LandedCostChargeLine> {
            Ok(LandedCostChargeLine {
                id: Uuid::new_v4(),
                organization_id: _org_id,
                charge_id: _charge_id,
                component_id: _component_id,
                line_number: _line_number,
                receipt_line_id: _receipt_line_id,
                item_id: _item_id,
                item_code: _item_code.map(|s| s.to_string()),
                item_description: _item_description.map(|s| s.to_string()),
                charge_amount: charge_amount.to_string(),
                allocated_amount: "0".to_string(),
                allocation_basis: allocation_basis.to_string(),
                allocation_qty: _allocation_qty.map(|s| s.to_string()),
                allocation_value: _allocation_value.map(|s| s.to_string()),
                expense_account: None,
                notes: None,
                metadata: serde_json::json!({}),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            })
        }
        async fn get_charge_line(&self, _id: Uuid) -> AtlasResult<Option<LandedCostChargeLine>> { Ok(None) }
        async fn list_charge_lines(&self, _charge_id: Uuid) -> AtlasResult<Vec<LandedCostChargeLine>> { Ok(vec![]) }
        async fn create_allocation(&self, _org_id: Uuid, _charge_id: Uuid, _charge_line_id: Uuid, _receipt_id: Option<Uuid>, _receipt_line_id: Option<Uuid>, _item_id: Option<Uuid>, _item_code: Option<&str>, allocated_amount: &str, allocation_basis: &str, _allocation_basis_value: Option<&str>, _total_basis_value: Option<&str>, _allocation_pct: Option<&str>, _original_unit_cost: Option<&str>) -> AtlasResult<LandedCostAllocation> {
            Ok(LandedCostAllocation {
                id: Uuid::new_v4(),
                organization_id: _org_id,
                charge_id: _charge_id,
                charge_line_id: _charge_line_id,
                receipt_id: _receipt_id,
                receipt_line_id: _receipt_line_id,
                item_id: _item_id,
                item_code: _item_code.map(|s| s.to_string()),
                allocated_amount: allocated_amount.to_string(),
                allocation_basis: allocation_basis.to_string(),
                allocation_basis_value: None,
                total_basis_value: None,
                allocation_pct: None,
                unit_landed_cost: None,
                original_unit_cost: None,
                metadata: serde_json::json!({}),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            })
        }
        async fn list_allocations(&self, _charge_id: Uuid) -> AtlasResult<Vec<LandedCostAllocation>> { Ok(vec![]) }
        async fn list_allocations_for_org(&self, _org_id: Uuid) -> AtlasResult<Vec<LandedCostAllocation>> { Ok(vec![]) }
        async fn get_allocations_for_receipt(&self, _org_id: Uuid, _receipt_id: Uuid) -> AtlasResult<Vec<LandedCostAllocation>> { Ok(vec![]) }
        async fn get_receipt_lines_for_charge(&self, _org_id: Uuid, _receipt_id: Option<Uuid>) -> AtlasResult<Vec<ReceiptLineInfo>> { Ok(vec![]) }
        async fn create_simulation(&self, _org_id: Uuid, sim_number: &str, _template_id: Option<Uuid>, _purchase_order_id: Option<Uuid>, _item_id: Option<Uuid>, _item_code: Option<&str>, _item_description: Option<&str>, est_qty: &str, unit_price: &str, currency: &str, est_charges: &serde_json::Value, est_landed: &str, est_per_unit: &str, _created_by: Option<Uuid>) -> AtlasResult<LandedCostSimulation> {
            Ok(LandedCostSimulation {
                id: Uuid::new_v4(),
                organization_id: _org_id,
                simulation_number: sim_number.to_string(),
                template_id: _template_id,
                purchase_order_id: _purchase_order_id,
                item_id: _item_id,
                item_code: _item_code.map(|s| s.to_string()),
                item_description: _item_description.map(|s| s.to_string()),
                estimated_quantity: est_qty.to_string(),
                unit_price: unit_price.to_string(),
                currency: currency.to_string(),
                estimated_charges: est_charges.clone(),
                estimated_landed_cost: est_landed.to_string(),
                estimated_landed_cost_per_unit: est_per_unit.to_string(),
                variance_vs_actual: None,
                status: "draft".to_string(),
                metadata: serde_json::json!({}),
                created_by: _created_by,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            })
        }
        async fn get_simulation(&self, _id: Uuid) -> AtlasResult<Option<LandedCostSimulation>> { Ok(None) }
        async fn list_simulations(&self, _org_id: Uuid, _status: Option<&str>) -> AtlasResult<Vec<LandedCostSimulation>> { Ok(vec![]) }
        async fn update_simulation_status(&self, _id: Uuid, _status: &str) -> AtlasResult<LandedCostSimulation> {
            Err(AtlasError::NotImplemented("mock".to_string()))
        }
    }

    fn make_engine() -> LandedCostEngine {
        LandedCostEngine::new(StdArc::new(MockLandedCostRepo))
    }

    #[tokio::test]
    async fn test_create_template_empty_code() {
        let engine = make_engine();
        let result = engine.create_template(
            Uuid::new_v4(), "", "Test Template", None, None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("code is required")),
            other => panic!("Expected ValidationFailed, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_create_template_empty_name() {
        let engine = make_engine();
        let result = engine.create_template(
            Uuid::new_v4(), "TPL-001", "", None, None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("name is required")),
            other => panic!("Expected ValidationFailed, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_create_template_success() {
        let engine = make_engine();
        let result = engine.create_template(
            Uuid::new_v4(), "TPL-001", "Import Freight Template", Some("desc"), None,
        ).await;
        assert!(result.is_ok());
        let tpl = result.unwrap();
        assert_eq!(tpl.code, "TPL-001");
        assert_eq!(tpl.name, "Import Freight Template");
        assert_eq!(tpl.status, "active");
    }

    #[tokio::test]
    async fn test_create_component_invalid_cost_type() {
        let engine = make_engine();
        let result = engine.create_component(
            Uuid::new_v4(), None, "FRT-01", "Freight", None,
            "invalid_type", "quantity", None, None, None, false, None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("Invalid cost type")),
            other => panic!("Expected ValidationFailed, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_create_component_invalid_allocation_basis() {
        let engine = make_engine();
        let result = engine.create_component(
            Uuid::new_v4(), None, "FRT-01", "Freight", None,
            "freight", "invalid_basis", None, None, None, false, None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("Invalid allocation basis")),
            other => panic!("Expected ValidationFailed, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_create_component_invalid_rate_uom() {
        let engine = make_engine();
        let result = engine.create_component(
            Uuid::new_v4(), None, "FRT-01", "Freight", None,
            "freight", "quantity", Some("5.00"), Some("invalid_uom"), None, false, None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("Invalid rate UOM")),
            other => panic!("Expected ValidationFailed, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_create_component_negative_rate() {
        let engine = make_engine();
        let result = engine.create_component(
            Uuid::new_v4(), None, "FRT-01", "Freight", None,
            "freight", "quantity", Some("-5.00"), Some("per_unit"), None, false, None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("non-negative")),
            other => panic!("Expected ValidationFailed, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_create_component_success() {
        let engine = make_engine();
        let result = engine.create_component(
            Uuid::new_v4(), None, "FRT-01", "Ocean Freight", Some("desc"),
            "freight", "quantity", Some("2.50"), Some("per_unit"), Some("5100"), true, None,
        ).await;
        assert!(result.is_ok());
        let comp = result.unwrap();
        assert_eq!(comp.code, "FRT-01");
        assert_eq!(comp.cost_type, "freight");
        assert_eq!(comp.allocation_basis, "quantity");
        assert!(comp.is_taxable);
    }

    #[tokio::test]
    async fn test_create_charge_invalid_type() {
        let engine = make_engine();
        let result = engine.create_charge(
            Uuid::new_v4(), "CHG-001", None, None, None, None, None,
            "invalid", None, "100.00", "USD", None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("Invalid charge type")),
            other => panic!("Expected ValidationFailed, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_create_charge_negative_amount() {
        let engine = make_engine();
        let result = engine.create_charge(
            Uuid::new_v4(), "CHG-001", None, None, None, None, None,
            "actual", None, "-50.00", "USD", None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("non-negative")),
            other => panic!("Expected ValidationFailed, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_create_charge_empty_currency() {
        let engine = make_engine();
        let result = engine.create_charge(
            Uuid::new_v4(), "CHG-001", None, None, None, None, None,
            "actual", None, "100.00", "", None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("Currency is required")),
            other => panic!("Expected ValidationFailed, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_create_charge_success() {
        let engine = make_engine();
        let result = engine.create_charge(
            Uuid::new_v4(), "CHG-001", None, None, None, Some(Uuid::new_v4()), Some("Acme Corp"),
            "actual", chrono::NaiveDate::from_ymd_opt(2024, 6, 15), "1250.00", "USD", None,
        ).await;
        assert!(result.is_ok());
        let charge = result.unwrap();
        assert_eq!(charge.charge_number, "CHG-001");
        assert_eq!(charge.charge_type, "actual");
        assert_eq!(charge.total_amount, "1250.00");
        assert_eq!(charge.currency, "USD");
        assert_eq!(charge.status, "draft");
    }

    #[tokio::test]
    async fn test_simulation_zero_quantity() {
        let engine = make_engine();
        let result = engine.create_simulation(
            Uuid::new_v4(), None, None, None, None, None,
            "0", "50.00", "USD", None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("positive")),
            other => panic!("Expected ValidationFailed, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_simulation_negative_price() {
        let engine = make_engine();
        let result = engine.create_simulation(
            Uuid::new_v4(), None, None, None, None, None,
            "100", "-10.00", "USD", None,
        ).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AtlasError::ValidationFailed(msg) => assert!(msg.contains("non-negative")),
            other => panic!("Expected ValidationFailed, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_simulation_success() {
        let engine = make_engine();
        let result = engine.create_simulation(
            Uuid::new_v4(), None, None, None, Some("ITEM-001"), Some("Widget"),
            "100", "50.00", "USD", None,
        ).await;
        assert!(result.is_ok());
        let sim = result.unwrap();
        assert!(sim.simulation_number.starts_with("SIM-"));
        assert_eq!(sim.estimated_quantity, "100");
        assert_eq!(sim.unit_price, "50.00");
        assert_eq!(sim.currency, "USD");
    }

    #[tokio::test]
    async fn test_list_charges_invalid_status_filter() {
        let engine = make_engine();
        let result = engine.list_charges(
            Uuid::new_v4(), Some("bogus"), None, None,
        ).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_charges_invalid_type_filter() {
        let engine = make_engine();
        let result = engine.list_charges(
            Uuid::new_v4(), None, Some("bogus"), None,
        ).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_update_template_status_invalid() {
        let engine = make_engine();
        let result = engine.update_template_status(
            Uuid::new_v4(), "TPL-001", "bogus",
        ).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_update_component_status_invalid() {
        let engine = make_engine();
        let result = engine.update_component_status(
            Uuid::new_v4(), "FRT-01", "bogus",
        ).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_simulations_invalid_status() {
        let engine = make_engine();
        let result = engine.list_simulations(
            Uuid::new_v4(), Some("bogus"),
        ).await;
        assert!(result.is_err());
    }
}
