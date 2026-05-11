//! Finance Charge Management Engine
//!
//! Orchestrates finance charge term management, assessment runs,
//! charge calculation, invoice generation, and lifecycle management.
//!
//! Oracle Fusion Cloud ERP equivalent: Financials > Receivables > Finance Charges

use atlas_shared::{AtlasError, AtlasResult};
use super::repository::{
    FinanceChargeRepository,
    FinanceChargeTerm, FinanceChargeRun, FinanceChargeLine, FinanceChargeInvoice,
    FinanceChargeActivity, FinanceChargeSummary,
    FinanceChargeTermCreateParams, FinanceChargeRunCreateParams,
    FinanceChargeLineCreateParams, FinanceChargeInvoiceCreateParams,
};
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

// Valid charge types
const VALID_CHARGE_TYPES: &[&str] = &["percentage", "flat_fee", "tiered"];

// Valid calculation bases
const VALID_CALCULATION_BASES: &[&str] = &["daily", "monthly", "annual"];

// Valid run statuses
const VALID_RUN_STATUSES: &[&str] = &[
    "draft", "submitted", "approved", "applied", "cancelled",
];

// Valid line statuses
const VALID_LINE_STATUSES: &[&str] = &[
    "pending", "charged", "waived", "cancelled",
];

// Valid invoice statuses
const VALID_INVOICE_STATUSES: &[&str] = &[
    "open", "paid", "cancelled", "reversed",
];

/// Finance Charge Management Engine
pub struct FinanceChargeEngine {
    repo: Arc<dyn FinanceChargeRepository>,
}

impl FinanceChargeEngine {
    pub fn new(repo: Arc<dyn FinanceChargeRepository>) -> Self {
        Self { repo }
    }

    // ========================================================================
    // Validation
    // ========================================================================

    fn validate_charge_type(charge_type: &str) -> AtlasResult<()> {
        if !VALID_CHARGE_TYPES.contains(&charge_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid charge_type '{}'. Must be one of: {}", charge_type, VALID_CHARGE_TYPES.join(", ")
            )));
        }
        Ok(())
    }

    fn validate_calculation_basis(basis: &str) -> AtlasResult<()> {
        if !VALID_CALCULATION_BASES.contains(&basis) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid calculation_basis '{}'. Must be one of: {}", basis, VALID_CALCULATION_BASES.join(", ")
            )));
        }
        Ok(())
    }

    fn validate_run_status(status: &str) -> AtlasResult<()> {
        if !VALID_RUN_STATUSES.contains(&status) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid run status '{}'. Must be one of: {}", status, VALID_RUN_STATUSES.join(", ")
            )));
        }
        Ok(())
    }

    #[allow(dead_code)]
    fn validate_line_status(status: &str) -> AtlasResult<()> {
        if !VALID_LINE_STATUSES.contains(&status) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid line status '{}'. Must be one of: {}", status, VALID_LINE_STATUSES.join(", ")
            )));
        }
        Ok(())
    }

    fn validate_invoice_status(status: &str) -> AtlasResult<()> {
        if !VALID_INVOICE_STATUSES.contains(&status) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid invoice status '{}'. Must be one of: {}", status, VALID_INVOICE_STATUSES.join(", ")
            )));
        }
        Ok(())
    }

    /// Validate run status transition
    pub fn validate_run_transition(current: &str, target: &str) -> AtlasResult<()> {
        match (current, target) {
            ("draft", "submitted") => Ok(()),
            ("draft", "cancelled") => Ok(()),
            ("submitted", "approved") => Ok(()),
            ("submitted", "cancelled") => Ok(()),
            ("approved", "applied") => Ok(()),
            ("approved", "cancelled") => Ok(()),
            _ => Err(AtlasError::WorkflowError(format!(
                "Invalid run status transition from '{current}' to '{target}'. \
                 Valid transitions: draft→submitted, draft→cancelled, \
                 submitted→approved, submitted→cancelled, \
                 approved→applied, approved→cancelled"
            ))),
        }
    }

    // ========================================================================
    // Finance Charge Terms CRUD
    // ========================================================================

    /// Create a new finance charge term definition
    pub async fn create_term(
        &self,
        org_id: Uuid,
        term_code: &str,
        term_name: &str,
        description: Option<&str>,
        charge_type: &str,
        charge_rate: Option<f64>,
        minimum_charge: Option<f64>,
        maximum_charge: Option<f64>,
        grace_period_days: i32,
        currency_code: &str,
        calculation_basis: &str,
        include_tax: bool,
        compound_charges: bool,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        auto_assess: bool,
        revenue_account_code: Option<&str>,
        receivable_account_code: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<FinanceChargeTerm> {
        info!("Creating finance charge term '{}' for org {}", term_code, org_id);

        if term_code.is_empty() || term_name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Term code and name are required".to_string(),
            ));
        }
        Self::validate_charge_type(charge_type)?;
        Self::validate_calculation_basis(calculation_basis)?;

        if charge_type == "percentage" && charge_rate.is_none() {
            return Err(AtlasError::ValidationFailed(
                "Charge rate is required for percentage charge type".to_string(),
            ));
        }
        if let Some(rate) = charge_rate {
            if rate < 0.0 {
                return Err(AtlasError::ValidationFailed(
                    "Charge rate cannot be negative".to_string(),
                ));
            }
        }
        if let (Some(min), Some(max)) = (minimum_charge, maximum_charge) {
            if max < min {
                return Err(AtlasError::ValidationFailed(
                    "Maximum charge cannot be less than minimum charge".to_string(),
                ));
            }
        }
        if grace_period_days < 0 {
            return Err(AtlasError::ValidationFailed(
                "Grace period days cannot be negative".to_string(),
            ));
        }

        let params = FinanceChargeTermCreateParams {
            org_id,
            term_code: term_code.to_string(),
            term_name: term_name.to_string(),
            description: description.map(std::string::ToString::to_string),
            charge_type: charge_type.to_string(),
            charge_rate,
            minimum_charge,
            maximum_charge,
            grace_period_days,
            currency_code: currency_code.to_string(),
            calculation_basis: calculation_basis.to_string(),
            include_tax,
            compound_charges,
            effective_from,
            effective_to,
            is_active: true,
            auto_assess,
            revenue_account_code: revenue_account_code.map(std::string::ToString::to_string),
            receivable_account_code: receivable_account_code.map(std::string::ToString::to_string),
            created_by,
        };

        let term = self.repo.create_term(&params).await?;

        let _ = self.repo.create_activity(
            org_id, "term", term.id,
            "created",
            Some(&format!("Finance charge term '{term_code}' created")),
            None, None, created_by, None,
        ).await;

        Ok(term)
    }

    /// Get a term by ID
    pub async fn get_term(&self, id: Uuid) -> AtlasResult<Option<FinanceChargeTerm>> {
        self.repo.get_term(id).await
    }

    /// Get a term by code
    pub async fn get_term_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<FinanceChargeTerm>> {
        self.repo.get_term_by_code(org_id, code).await
    }

    /// List terms
    pub async fn list_terms(&self, org_id: Uuid, is_active: Option<bool>) -> AtlasResult<Vec<FinanceChargeTerm>> {
        self.repo.list_terms(org_id, is_active).await
    }

    /// Activate/deactivate a term
    pub async fn update_term_status(&self, id: Uuid, is_active: bool) -> AtlasResult<FinanceChargeTerm> {
        info!("Setting term {} active={}", id, is_active);
        self.repo.update_term_status(id, is_active).await
    }

    /// Add a tier to a tiered charge term
    pub async fn add_tier(
        &self,
        org_id: Uuid,
        term_id: Uuid,
        from_days_overdue: i32,
        to_days_overdue: Option<i32>,
        charge_rate: f64,
        flat_fee: Option<f64>,
    ) -> AtlasResult<super::repository::FinanceChargeTier> {
        info!("Adding tier to term {} for org {}", term_id, org_id);

        if from_days_overdue < 0 {
            return Err(AtlasError::ValidationFailed(
                "From days overdue cannot be negative".to_string(),
            ));
        }
        if let Some(to) = to_days_overdue {
            if to <= from_days_overdue {
                return Err(AtlasError::ValidationFailed(
                    "To days overdue must be greater than from days overdue".to_string(),
                ));
            }
        }
        if charge_rate < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Charge rate cannot be negative".to_string(),
            ));
        }

        self.repo.create_tier(org_id, term_id, from_days_overdue, to_days_overdue, charge_rate, flat_fee).await
    }

    /// List tiers for a term
    pub async fn list_tiers(&self, term_id: Uuid) -> AtlasResult<Vec<super::repository::FinanceChargeTier>> {
        self.repo.list_tiers(term_id).await
    }

    // ========================================================================
    // Assessment Runs
    // ========================================================================

    /// Create a new finance charge assessment run
    pub async fn create_run(
        &self,
        org_id: Uuid,
        run_date: chrono::NaiveDate,
        gl_date: chrono::NaiveDate,
        term_id: Option<Uuid>,
        term_code: Option<&str>,
        currency_code: &str,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<FinanceChargeRun> {
        info!("Creating finance charge run for org {} on {}", org_id, run_date);

        // Resolve term
        let resolved_term_code = if let Some(code) = term_code {
            Some(code.to_string())
        } else if let Some(tid) = term_id {
            let term = self.repo.get_term(tid).await?
                .ok_or_else(|| AtlasError::EntityNotFound("Finance charge term not found".to_string()))?;
            Some(term.term_code)
        } else {
            None
        };

        let params = FinanceChargeRunCreateParams {
            org_id,
            run_date,
            gl_date,
            term_id,
            term_code: resolved_term_code,
            currency_code: currency_code.to_string(),
            notes: notes.map(std::string::ToString::to_string),
            created_by,
        };

        let run = self.repo.create_run(&params).await?;

        let _ = self.repo.create_activity(
            org_id, "run", run.id,
            "created",
            Some(&format!("Finance charge run '{}' created", run.run_number)),
            None, Some("draft"), created_by, None,
        ).await;

        Ok(run)
    }

    /// Get a run by ID
    pub async fn get_run(&self, id: Uuid) -> AtlasResult<Option<FinanceChargeRun>> {
        self.repo.get_run(id).await
    }

    /// Get a run by number
    pub async fn get_run_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<FinanceChargeRun>> {
        self.repo.get_run_by_number(org_id, number).await
    }

    /// List runs
    pub async fn list_runs(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<FinanceChargeRun>> {
        self.repo.list_runs(org_id, status).await
    }

    /// Delete a draft run
    pub async fn delete_run(&self, org_id: Uuid, run_number: &str) -> AtlasResult<()> {
        info!("Deleting finance charge run '{}' for org {}", run_number, org_id);

        let run = self.repo.get_run_by_number(org_id, run_number).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Run not found".to_string()))?;

        if run.status != "draft" {
            return Err(AtlasError::ValidationFailed(
                "Only draft runs can be deleted".to_string(),
            ));
        }

        // Delete lines first
        self.repo.delete_lines_for_run(run.id).await?;
        self.repo.delete_run(org_id, run_number).await
    }

    /// Transition a run to a new status
    pub async fn transition_run(
        &self,
        id: Uuid,
        new_status: &str,
        performed_by: Option<Uuid>,
    ) -> AtlasResult<FinanceChargeRun> {
        info!("Transitioning finance charge run {} to '{}'", id, new_status);
        Self::validate_run_status(new_status)?;

        let current = self.repo.get_run(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Finance charge run not found".to_string()))?;

        Self::validate_run_transition(&current.status, new_status)?;

        let run = self.repo.update_run_status(id, new_status).await?;

        let _ = self.repo.create_activity(
            current.organization_id, "run", id,
            &format!("status_change_{new_status}"),
            Some(&format!("Status changed from '{}' to '{}'", current.status, new_status)),
            Some(&current.status), Some(new_status),
            performed_by, None,
        ).await;

        Ok(run)
    }

    // ========================================================================
    // Charge Lines
    // ========================================================================

    /// Add a charge line to a run (manually or from assessment)
    pub async fn add_charge_line(
        &self,
        org_id: Uuid,
        run_id: Uuid,
        customer_id: Option<Uuid>,
        customer_number: Option<&str>,
        customer_name: Option<&str>,
        invoice_id: Option<Uuid>,
        invoice_number: Option<&str>,
        invoice_date: Option<chrono::NaiveDate>,
        invoice_due_date: Option<chrono::NaiveDate>,
        days_overdue: i32,
        invoice_amount: f64,
        outstanding_amount: f64,
        charge_type: &str,
        charge_rate: f64,
        charge_amount: f64,
        currency_code: &str,
        term_id: Option<Uuid>,
        term_code: Option<&str>,
    ) -> AtlasResult<FinanceChargeLine> {
        info!("Adding charge line to run {} for org {}", run_id, org_id);

        // Verify run is in draft status
        let run = self.repo.get_run(run_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Finance charge run not found".to_string()))?;

        if run.status != "draft" {
            return Err(AtlasError::ValidationFailed(
                "Lines can only be added to draft runs".to_string(),
            ));
        }

        if charge_amount < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Charge amount cannot be negative".to_string(),
            ));
        }
        if days_overdue < 0 {
            return Err(AtlasError::ValidationFailed(
                "Days overdue cannot be negative".to_string(),
            ));
        }
        if outstanding_amount < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Outstanding amount cannot be negative".to_string(),
            ));
        }

        // Get next line number
        let existing_lines = self.repo.list_lines(run_id).await?;
        let line_number = existing_lines.len() as i32 + 1;

        let params = FinanceChargeLineCreateParams {
            org_id,
            run_id,
            line_number,
            customer_id,
            customer_number: customer_number.map(std::string::ToString::to_string),
            customer_name: customer_name.map(std::string::ToString::to_string),
            invoice_id,
            invoice_number: invoice_number.map(std::string::ToString::to_string),
            invoice_date,
            invoice_due_date,
            days_overdue,
            invoice_amount,
            outstanding_amount,
            charge_type: charge_type.to_string(),
            charge_rate,
            charge_amount,
            currency_code: currency_code.to_string(),
            term_id,
            term_code: term_code.map(std::string::ToString::to_string),
        };

        let line = self.repo.create_line(&params).await?;

        // Update run totals
        let all_lines = self.repo.list_lines(run_id).await?;
        let total_invoices = all_lines.len() as i32;
        let total_charges: f64 = all_lines.iter().map(|l| l.charge_amount).sum();
        self.repo.update_run_totals(run_id, total_invoices, total_charges).await?;

        Ok(line)
    }

    /// List lines for a run
    pub async fn list_lines(&self, run_id: Uuid) -> AtlasResult<Vec<FinanceChargeLine>> {
        self.repo.list_lines(run_id).await
    }

    /// Waive a charge line
    pub async fn waive_line(&self, id: Uuid, reason: &str) -> AtlasResult<FinanceChargeLine> {
        info!("Waiving finance charge line {}", id);
        if reason.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Waiver reason is required".to_string(),
            ));
        }
        self.repo.update_line_status(id, "waived", Some(reason)).await
    }

    /// Cancel a charge line
    pub async fn cancel_line(&self, id: Uuid) -> AtlasResult<FinanceChargeLine> {
        info!("Cancelling finance charge line {}", id);
        self.repo.update_line_status(id, "cancelled", None).await
    }

    // ========================================================================
    // Charge Invoice Generation
    // ========================================================================

    /// Generate charge invoices for all pending lines in an approved run.
    /// Groups lines by customer and creates one invoice per customer.
    pub async fn generate_invoices(&self, run_id: Uuid, created_by: Option<Uuid>) -> AtlasResult<Vec<FinanceChargeInvoice>> {
        info!("Generating charge invoices for run {}", run_id);

        let run = self.repo.get_run(run_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Finance charge run not found".to_string()))?;

        if run.status != "approved" {
            return Err(AtlasError::ValidationFailed(
                "Invoices can only be generated for approved runs".to_string(),
            ));
        }

        let lines = self.repo.list_lines(run_id).await?;
        let pending_lines: Vec<_> = lines.iter()
            .filter(|l| l.status == "pending")
            .collect();

        if pending_lines.is_empty() {
            return Ok(vec![]);
        }

        // Group by customer identifier (customer_id if present, otherwise customer_name)
        let mut customer_groups: std::collections::HashMap<String, Vec<&FinanceChargeLine>> =
            std::collections::HashMap::new();
        for line in &pending_lines {
            let key = line.customer_id
                .map(|id| id.to_string())
                .or_else(|| line.customer_name.clone())
                .unwrap_or_else(|| "unknown".to_string());
            customer_groups.entry(key).or_default().push(line);
        }

        let mut invoices = Vec::new();

        for (_customer_key, group) in customer_groups {
            let first = group[0];
            let total_charge: f64 = group.iter().map(|l| l.charge_amount).sum();

            let seq = self.repo.get_next_invoice_number(run.organization_id).await.unwrap_or(1);
            let invoice_number = format!("FCI-{seq:06}");

            let params = FinanceChargeInvoiceCreateParams {
                org_id: run.organization_id,
                charge_invoice_number: invoice_number,
                customer_id: first.customer_id,
                customer_number: first.customer_number.clone(),
                customer_name: first.customer_name.clone(),
                invoice_date: run.run_date,
                gl_date: run.gl_date,
                due_date: run.run_date + chrono::Duration::days(30),
                currency_code: first.currency_code.clone(),
                total_charge_amount: total_charge,
                run_id: Some(run_id),
                revenue_account_code: None,
                receivable_account_code: None,
                notes: Some(format!("Finance charges from run {}", run.run_number)),
                created_by,
            };

            let invoice = self.repo.create_invoice(&params).await?;

            // Update each line with the invoice reference
            for line in group {
                let _ = self.repo.update_line_charge_invoice(
                    line.id, invoice.id, Some(&invoice.charge_invoice_number),
                ).await;
            }

            let _ = self.repo.create_activity(
                run.organization_id, "invoice", invoice.id,
                "created",
                Some(&format!("Charge invoice '{}' created for customer {:?}", invoice.charge_invoice_number, first.customer_name)),
                None, Some("open"), created_by, None,
            ).await;

            invoices.push(invoice);
        }

        // Auto-transition run to applied
        let run = self.repo.update_run_status(run_id, "applied").await?;

        let _ = self.repo.create_activity(
            run.organization_id, "run", run_id,
            "status_change_applied",
            Some(&format!("Run '{}' transitioned to applied after invoice generation", run.run_number)),
            Some("approved"), Some("applied"), created_by, None,
        ).await;

        Ok(invoices)
    }

    /// Get a charge invoice by ID
    pub async fn get_invoice(&self, id: Uuid) -> AtlasResult<Option<FinanceChargeInvoice>> {
        self.repo.get_invoice(id).await
    }

    /// Get a charge invoice by number
    pub async fn get_invoice_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<FinanceChargeInvoice>> {
        self.repo.get_invoice_by_number(org_id, number).await
    }

    /// List charge invoices
    pub async fn list_invoices(&self, org_id: Uuid, status: Option<&str>, customer_id: Option<Uuid>) -> AtlasResult<Vec<FinanceChargeInvoice>> {
        self.repo.list_invoices(org_id, status, customer_id).await
    }

    /// Transition an invoice status
    pub async fn transition_invoice(&self, id: Uuid, new_status: &str) -> AtlasResult<FinanceChargeInvoice> {
        info!("Transitioning charge invoice {} to '{}'", id, new_status);
        Self::validate_invoice_status(new_status)?;

        let current = self.repo.get_invoice(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Charge invoice not found".to_string()))?;

        match (&current.status as &str, new_status) {
            ("open", "paid" | "cancelled" | "reversed") => {}
            _ => {
                return Err(AtlasError::WorkflowError(format!(
                    "Invalid invoice transition from '{}' to '{}'", current.status, new_status
                )));
            }
        }

        let invoice = self.repo.update_invoice_status(id, new_status).await?;

        let _ = self.repo.create_activity(
            current.organization_id, "invoice", id,
            &format!("status_change_{new_status}"),
            Some(&format!("Invoice '{}' status changed to '{}'", current.charge_invoice_number, new_status)),
            Some(&current.status), Some(new_status), None, None,
        ).await;

        Ok(invoice)
    }

    // ========================================================================
    // Activities
    // ========================================================================

    /// List activities for an entity
    pub async fn list_activities(&self, entity_type: &str, entity_id: Uuid) -> AtlasResult<Vec<FinanceChargeActivity>> {
        self.repo.list_activities(entity_type, entity_id).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get finance charge dashboard summary
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<FinanceChargeSummary> {
        self.repo.get_dashboard(org_id).await
    }

    // ========================================================================
    // Charge Calculation Helpers
    // ========================================================================

    /// Calculate finance charge for a percentage-based term
    #[must_use] 
    pub fn calculate_percentage_charge(
        outstanding_amount: f64,
        annual_rate: f64,
        days_overdue: i32,
        calculation_basis: &str,
    ) -> f64 {
        let rate_decimal = annual_rate / 100.0;
        match calculation_basis {
            "daily" => outstanding_amount * rate_decimal / 365.0 * f64::from(days_overdue),
            "monthly" => outstanding_amount * rate_decimal / 12.0 * (f64::from(days_overdue) / 30.0),
            "annual" => outstanding_amount * rate_decimal,
            _ => 0.0,
        }
    }

    /// Apply min/max charge constraints
    #[must_use] 
    pub const fn apply_charge_limits(
        charge_amount: f64,
        minimum: Option<f64>,
        maximum: Option<f64>,
    ) -> f64 {
        let mut amount = charge_amount;
        if let Some(min) = minimum {
            amount = amount.max(min);
        }
        if let Some(max) = maximum {
            amount = amount.min(max);
        }
        amount
    }

    // ========================================================================
    // Exported validation functions
    // ========================================================================

    #[must_use] 
    pub const fn valid_charge_types() -> &'static [&'static str] { VALID_CHARGE_TYPES }
    #[must_use] 
    pub const fn valid_calculation_bases() -> &'static [&'static str] { VALID_CALCULATION_BASES }
    #[must_use] 
    pub const fn valid_run_statuses() -> &'static [&'static str] { VALID_RUN_STATUSES }
    #[must_use] 
    pub const fn valid_line_statuses() -> &'static [&'static str] { VALID_LINE_STATUSES }
    #[must_use] 
    pub const fn valid_invoice_statuses() -> &'static [&'static str] { VALID_INVOICE_STATUSES }
}
