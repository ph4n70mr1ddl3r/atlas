//! Interest Invoice Engine
//!
//! Manages interest rate schedule lifecycle, overdue invoice tracking,
//! interest calculation, and interest invoice generation with full lifecycle
//! (draft → posted → reversed).
//!
//! Oracle Fusion Cloud ERP equivalent: Receivables > Late Charges

use atlas_shared::{
    InterestRateSchedule, OverdueInvoice, InterestCalculationRun,
    InterestCalculationLine, InterestInvoice, InterestInvoiceLine,
    InterestInvoiceDashboard,
    AtlasError, AtlasResult,
};
use super::InterestInvoiceRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Valid compounding frequencies
const VALID_COMPOUNDING_FREQUENCIES: &[&str] = &["daily", "monthly", "quarterly", "annual"];

/// Valid charge types
const VALID_CHARGE_TYPES: &[&str] = &["interest", "penalty", "mixed"];

/// Valid schedule statuses
const VALID_SCHEDULE_STATUSES: &[&str] = &["active", "inactive"];

/// Valid overdue invoice statuses
const VALID_OVERDUE_STATUSES: &[&str] = &["open", "paid", "closed", "disputed"];

/// Valid calculation run statuses
#[allow(dead_code)]
const VALID_RUN_STATUSES: &[&str] = &["draft", "calculated", "invoiced", "posted", "cancelled"];

/// Valid interest invoice statuses
const VALID_INVOICE_STATUSES: &[&str] = &["draft", "posted", "reversed", "cancelled"];

/// Calculate interest amount using simple daily interest formula:
/// interest = outstanding_amount * (annual_rate / 100) * (overdue_days / 365)
pub fn calculate_simple_interest(
    outstanding_amount: f64,
    annual_rate: f64,
    overdue_days: i32,
) -> f64 {
    outstanding_amount * (annual_rate / 100.0) * (overdue_days as f64 / 365.0)
}

/// Interest Invoice engine
pub struct InterestInvoiceEngine {
    repository: Arc<dyn InterestInvoiceRepository>,
}

impl InterestInvoiceEngine {
    pub fn new(repository: Arc<dyn InterestInvoiceRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Interest Rate Schedule Management
    // ========================================================================

    /// Create a new interest rate schedule
    pub async fn create_schedule(
        &self,
        org_id: Uuid,
        schedule_code: &str,
        name: &str,
        description: Option<&str>,
        annual_rate: &str,
        compounding_frequency: &str,
        charge_type: &str,
        grace_period_days: i32,
        minimum_charge: &str,
        maximum_charge: Option<&str>,
        currency_code: &str,
        effective_from: chrono::NaiveDate,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<InterestRateSchedule> {
        if schedule_code.is_empty() {
            return Err(AtlasError::ValidationFailed("Schedule code is required".to_string()));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Schedule name is required".to_string()));
        }
        if !VALID_COMPOUNDING_FREQUENCIES.contains(&compounding_frequency) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid compounding frequency '{}'. Must be one of: {}",
                compounding_frequency,
                VALID_COMPOUNDING_FREQUENCIES.join(", ")
            )));
        }
        if !VALID_CHARGE_TYPES.contains(&charge_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid charge type '{}'. Must be one of: {}",
                charge_type,
                VALID_CHARGE_TYPES.join(", ")
            )));
        }

        let rate: f64 = annual_rate.parse().map_err(|_| AtlasError::ValidationFailed(
            "Annual rate must be a valid number".to_string(),
        ))?;
        if rate <= 0.0 || rate > 100.0 {
            return Err(AtlasError::ValidationFailed(
                "Annual rate must be between 0 and 100".to_string(),
            ));
        }

        if grace_period_days < 0 {
            return Err(AtlasError::ValidationFailed(
                "Grace period days cannot be negative".to_string(),
            ));
        }

        if let (Some(from), Some(to)) = (effective_to, Some(effective_from)) {
            if to < from {
                return Err(AtlasError::ValidationFailed(
                    "Effective to date must be after effective from date".to_string(),
                ));
            }
        }

        info!("Creating interest rate schedule {} ({}) for org {}", schedule_code, name, org_id);

        self.repository.create_schedule(
            org_id, schedule_code, name, description, annual_rate,
            compounding_frequency, charge_type, grace_period_days, minimum_charge,
            maximum_charge, currency_code, effective_from, effective_to, created_by,
        ).await
    }

    /// Get a schedule by code
    pub async fn get_schedule(&self, org_id: Uuid, schedule_code: &str) -> AtlasResult<Option<InterestRateSchedule>> {
        self.repository.get_schedule(org_id, schedule_code).await
    }

    /// Get a schedule by ID
    pub async fn get_schedule_by_id(&self, id: Uuid) -> AtlasResult<Option<InterestRateSchedule>> {
        self.repository.get_schedule_by_id(id).await
    }

    /// List schedules with optional status filter
    pub async fn list_schedules(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<InterestRateSchedule>> {
        if let Some(s) = status {
            if !VALID_SCHEDULE_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}",
                    s,
                    VALID_SCHEDULE_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_schedules(org_id, status).await
    }

    /// Deactivate a schedule
    pub async fn deactivate_schedule(&self, id: Uuid) -> AtlasResult<InterestRateSchedule> {
        let schedule = self.repository.get_schedule_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Schedule {} not found", id)))?;

        if schedule.status != "active" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot deactivate schedule in '{}' status. Must be 'active'.",
                schedule.status
            )));
        }

        info!("Deactivated interest rate schedule {}", schedule.schedule_code);
        self.repository.update_schedule_status(id, "inactive").await
    }

    /// Activate a schedule
    pub async fn activate_schedule(&self, id: Uuid) -> AtlasResult<InterestRateSchedule> {
        let schedule = self.repository.get_schedule_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Schedule {} not found", id)))?;

        if schedule.status != "inactive" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot activate schedule in '{}' status. Must be 'inactive'.",
                schedule.status
            )));
        }

        info!("Activated interest rate schedule {}", schedule.schedule_code);
        self.repository.update_schedule_status(id, "active").await
    }

    /// Delete a schedule (only if inactive)
    pub async fn delete_schedule(&self, org_id: Uuid, schedule_code: &str) -> AtlasResult<()> {
        let schedule = self.repository.get_schedule(org_id, schedule_code).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Schedule {} not found", schedule_code)))?;

        if schedule.status != "inactive" {
            return Err(AtlasError::WorkflowError(
                "Cannot delete schedule that is not in 'inactive' status".to_string(),
            ));
        }

        info!("Deleted interest rate schedule {}", schedule_code);
        self.repository.delete_schedule(org_id, schedule_code).await
    }

    // ========================================================================
    // Overdue Invoice Management
    // ========================================================================

    /// Register an overdue invoice for interest tracking
    pub async fn register_overdue_invoice(
        &self,
        org_id: Uuid,
        invoice_number: &str,
        customer_id: Uuid,
        customer_name: Option<&str>,
        original_amount: &str,
        outstanding_amount: &str,
        due_date: chrono::NaiveDate,
        overdue_days: i32,
        currency_code: &str,
    ) -> AtlasResult<OverdueInvoice> {
        if invoice_number.is_empty() {
            return Err(AtlasError::ValidationFailed("Invoice number is required".to_string()));
        }

        let outstanding: f64 = outstanding_amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "Outstanding amount must be a valid number".to_string(),
        ))?;
        if outstanding <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Outstanding amount must be greater than zero".to_string(),
            ));
        }

        if overdue_days <= 0 {
            return Err(AtlasError::ValidationFailed(
                "Overdue days must be greater than zero".to_string(),
            ));
        }

        info!("Registering overdue invoice {} ({} days overdue) for org {}", invoice_number, overdue_days, org_id);

        self.repository.register_overdue_invoice(
            org_id, invoice_number, customer_id, customer_name,
            original_amount, outstanding_amount, due_date, overdue_days, currency_code,
        ).await
    }

    /// Get an overdue invoice by number
    pub async fn get_overdue_invoice(&self, org_id: Uuid, invoice_number: &str) -> AtlasResult<Option<OverdueInvoice>> {
        self.repository.get_overdue_invoice(org_id, invoice_number).await
    }

    /// List overdue invoices
    pub async fn list_overdue_invoices(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<OverdueInvoice>> {
        if let Some(s) = status {
            if !VALID_OVERDUE_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}",
                    s,
                    VALID_OVERDUE_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_overdue_invoices(org_id, status).await
    }

    /// Close an overdue invoice (e.g., when fully paid)
    pub async fn close_overdue_invoice(&self, id: Uuid) -> AtlasResult<OverdueInvoice> {
        let inv = self.repository.get_overdue_invoice_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Overdue invoice {} not found", id)))?;

        // This is fine - we handle the method not existing on the trait directly
        // by using list and filtering
        if inv.status != "open" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot close invoice in '{}' status. Must be 'open'.",
                inv.status
            )));
        }

        info!("Closed overdue invoice {}", inv.invoice_number);
        self.repository.update_overdue_invoice_status(id, "closed").await
    }

    // ========================================================================
    // Interest Calculation
    // ========================================================================

    /// Calculate interest on all open overdue invoices for an organization
    pub async fn calculate_interest(
        &self,
        org_id: Uuid,
        description: Option<&str>,
        schedule_id: Option<Uuid>,
        calculation_date: chrono::NaiveDate,
        generated_by: Option<Uuid>,
    ) -> AtlasResult<InterestCalculationRun> {
        // Get the schedule to use for calculation
        let schedule = if let Some(sid) = schedule_id {
            self.repository.get_schedule_by_id(sid).await?
                .ok_or_else(|| AtlasError::EntityNotFound(format!("Schedule {} not found", sid)))?
        } else {
            // Find the first active schedule
            let schedules = self.repository.list_schedules(org_id, Some("active")).await?;
            schedules.into_iter().next()
                .ok_or_else(|| AtlasError::ValidationFailed(
                    "No active interest rate schedule found. Create one first.".to_string(),
                ))?
        };

        if schedule.status != "active" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot calculate interest with schedule in '{}' status. Must be 'active'.",
                schedule.status
            )));
        }

        // Check effective dates
        if calculation_date < schedule.effective_from {
            return Err(AtlasError::ValidationFailed(format!(
                "Calculation date {} is before schedule effective date {}",
                calculation_date, schedule.effective_from
            )));
        }
        if let Some(to) = schedule.effective_to {
            if calculation_date > to {
                return Err(AtlasError::ValidationFailed(format!(
                    "Calculation date {} is after schedule effective end date {}",
                    calculation_date, to
                )));
            }
        }

        let annual_rate: f64 = schedule.annual_rate.parse().unwrap_or(0.0);

        // Get open overdue invoices
        let overdue_invoices = self.repository.list_overdue_invoices(org_id, Some("open")).await?;

        // Filter by grace period
        let eligible_invoices: Vec<&OverdueInvoice> = overdue_invoices.iter()
            .filter(|inv| inv.overdue_days > schedule.grace_period_days)
            .collect();

        if eligible_invoices.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "No eligible overdue invoices found for interest calculation.".to_string(),
            ));
        }

        // Get next run number
        let next_run = self.repository.get_latest_run_number(org_id).await? + 1;

        // Create the calculation run
        let run = self.repository.create_calculation_run(
            org_id, &next_run.to_string(), description,
            calculation_date, Some(schedule.id), &schedule.currency_code, generated_by,
        ).await?;

        // Calculate interest for each invoice
        let mut total_interest = 0.0_f64;
        let mut lines_count = 0;

        for inv in &eligible_invoices {
            let effective_overdue_days = inv.overdue_days - schedule.grace_period_days;
            let outstanding: f64 = inv.outstanding_amount.parse().unwrap_or(0.0);

            let interest = calculate_simple_interest(outstanding, annual_rate, effective_overdue_days);

            // Apply minimum charge
            let minimum: f64 = schedule.minimum_charge.parse().unwrap_or(0.0);
            let interest = interest.max(minimum);

            // Apply maximum charge if set
            let interest = if let Some(max_str) = &schedule.maximum_charge {
                let max: f64 = max_str.parse().unwrap_or(f64::MAX);
                interest.min(max)
            } else {
                interest
            };

            if interest > 0.0 {
                self.repository.create_calculation_line(
                    org_id, run.id, Some(inv.id),
                    &inv.invoice_number, inv.customer_id,
                    inv.customer_name.as_deref(),
                    &format!("{:.2}", outstanding),
                    effective_overdue_days,
                    &format!("{:.6}", annual_rate),
                    &format!("{:.2}", interest),
                    &schedule.currency_code,
                ).await?;

                total_interest += interest;
                lines_count += 1;

                // Update overdue invoice tracking
                let prev_total: f64 = inv.total_interest_charged.parse().unwrap_or(0.0);
                self.repository.update_overdue_interest(
                    inv.id,
                    &format!("{:.2}", prev_total + interest),
                    calculation_date,
                ).await?;
            }
        }

        // Update run totals
        let run = self.repository.update_calculation_run_totals(
            run.id, lines_count, &format!("{:.2}", total_interest),
        ).await?;

        // Mark run as calculated
        let run = self.repository.update_calculation_run_status(
            run.id, "calculated", None,
        ).await?;

        info!(
            "Calculated interest run #{} for org {}: {} invoices, total interest {:.2}",
            next_run, org_id, lines_count, total_interest
        );

        Ok(run)
    }

    /// Get a calculation run by ID
    pub async fn get_calculation_run(&self, id: Uuid) -> AtlasResult<Option<InterestCalculationRun>> {
        self.repository.get_calculation_run(id).await
    }

    /// List calculation runs
    pub async fn list_calculation_runs(&self, org_id: Uuid) -> AtlasResult<Vec<InterestCalculationRun>> {
        self.repository.list_calculation_runs(org_id).await
    }

    /// List calculation lines for a run
    pub async fn list_calculation_lines(&self, run_id: Uuid) -> AtlasResult<Vec<InterestCalculationLine>> {
        self.repository.list_calculation_lines(run_id).await
    }

    /// Cancel a draft calculation run
    pub async fn cancel_calculation_run(&self, id: Uuid) -> AtlasResult<InterestCalculationRun> {
        let run = self.repository.get_calculation_run(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Run {} not found", id)))?;

        if run.status != "calculated" && run.status != "draft" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot cancel run in '{}' status.",
                run.status
            )));
        }

        info!("Cancelled interest calculation run #{}", run.run_number);
        self.repository.update_calculation_run_status(id, "cancelled", None).await
    }

    // ========================================================================
    // Interest Invoice Generation
    // ========================================================================

    /// Generate interest invoices from a calculation run.
    /// Creates one interest invoice per customer from the calculation lines.
    pub async fn generate_interest_invoices(
        &self,
        org_id: Uuid,
        run_id: Uuid,
        invoice_date: chrono::NaiveDate,
        due_date: Option<chrono::NaiveDate>,
        gl_account_code: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<Vec<InterestInvoice>> {
        let run = self.repository.get_calculation_run(run_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Run {} not found", run_id)))?;

        if run.status != "calculated" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot generate invoices from run in '{}' status. Must be 'calculated'.",
                run.status
            )));
        }

        let lines = self.repository.list_calculation_lines(run_id).await?;
        if lines.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "No calculation lines found for this run.".to_string(),
            ));
        }

        // Group lines by customer
        let mut customer_lines: std::collections::HashMap<Uuid, Vec<&InterestCalculationLine>> =
            std::collections::HashMap::new();
        for line in &lines {
            customer_lines.entry(line.customer_id)
                .or_default()
                .push(line);
        }

        let mut invoices = Vec::new();
        let mut next_inv_num = self.repository.get_latest_invoice_number(org_id).await? + 1;

        for (customer_id, cust_lines) in &customer_lines {
            let total_interest: f64 = cust_lines.iter()
                .map(|l| l.interest_amount.parse::<f64>().unwrap_or(0.0))
                .sum();

            let customer_name = cust_lines.first().and_then(|l| l.customer_name.clone());

            let inv_num = format!("{}", next_inv_num);
            let currency = cust_lines.first().map(|l| l.currency_code.clone()).unwrap_or("USD".to_string());

            let invoice = self.repository.create_interest_invoice(
                org_id, &inv_num, *customer_id, customer_name.as_deref(),
                Some(run_id), invoice_date, due_date,
                &format!("{:.2}", total_interest), &currency,
                cust_lines.len() as i32, gl_account_code,
                None, None, created_by,
            ).await?;

            // Create invoice lines
            for (idx, calc_line) in cust_lines.iter().enumerate() {
                self.repository.create_interest_invoice_line(
                    org_id, invoice.id, Some(calc_line.id),
                    idx as i32 + 1, "interest",
                    Some(&format!("Interest on invoice {} ({} days overdue @ {}%)",
                        calc_line.invoice_number,
                        calc_line.overdue_days,
                        calc_line.annual_rate_used)),
                    Some(&calc_line.invoice_number),
                    Some(calc_line.overdue_days),
                    Some(&calc_line.outstanding_amount),
                    Some(&calc_line.annual_rate_used),
                    &calc_line.interest_amount,
                    &calc_line.currency_code,
                    gl_account_code,
                ).await?;

                // Link calculation line to the invoice
                self.repository.update_calculation_line_status(
                    calc_line.id, "invoiced", Some(invoice.id),
                ).await?;
            }

            invoices.push(invoice);
            next_inv_num += 1;
        }

        // Update run status to invoiced
        self.repository.update_calculation_run_status(run_id, "invoiced", None).await?;

        info!(
            "Generated {} interest invoice(s) from run #{} for org {}",
            invoices.len(), run.run_number, org_id
        );

        Ok(invoices)
    }

    /// Get an interest invoice by number
    pub async fn get_interest_invoice(&self, org_id: Uuid, invoice_number: &str) -> AtlasResult<Option<InterestInvoice>> {
        self.repository.get_interest_invoice(org_id, invoice_number).await
    }

    /// Get an interest invoice by ID
    pub async fn get_interest_invoice_by_id(&self, id: Uuid) -> AtlasResult<Option<InterestInvoice>> {
        self.repository.get_interest_invoice_by_id(id).await
    }

    /// List interest invoices
    pub async fn list_interest_invoices(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<InterestInvoice>> {
        if let Some(s) = status {
            if !VALID_INVOICE_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}",
                    s,
                    VALID_INVOICE_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_interest_invoices(org_id, status).await
    }

    /// Post an interest invoice
    pub async fn post_interest_invoice(&self, id: Uuid) -> AtlasResult<InterestInvoice> {
        let inv = self.repository.get_interest_invoice_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Invoice {} not found", id)))?;

        if inv.status != "draft" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot post invoice in '{}' status. Must be 'draft'.",
                inv.status
            )));
        }

        info!("Posted interest invoice {}", inv.invoice_number);
        self.repository.update_interest_invoice_status(
            id, "posted", Some(chrono::Utc::now()), None, None,
        ).await
    }

    /// Reverse a posted interest invoice
    pub async fn reverse_interest_invoice(&self, id: Uuid) -> AtlasResult<InterestInvoice> {
        let inv = self.repository.get_interest_invoice_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Invoice {} not found", id)))?;

        if inv.status != "posted" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot reverse invoice in '{}' status. Must be 'posted'.",
                inv.status
            )));
        }

        info!("Reversed interest invoice {}", inv.invoice_number);
        self.repository.update_interest_invoice_status(
            id, "reversed", None, Some(chrono::Utc::now()), Some(Uuid::new_v4()),
        ).await
    }

    /// Cancel a draft interest invoice
    pub async fn cancel_interest_invoice(&self, id: Uuid) -> AtlasResult<InterestInvoice> {
        let inv = self.repository.get_interest_invoice_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Invoice {} not found", id)))?;

        if inv.status != "draft" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot cancel invoice in '{}' status. Must be 'draft'.",
                inv.status
            )));
        }

        info!("Cancelled interest invoice {}", inv.invoice_number);
        self.repository.update_interest_invoice_status(
            id, "cancelled", None, None, None,
        ).await
    }

    /// List interest invoice lines
    pub async fn list_interest_invoice_lines(&self, interest_invoice_id: Uuid) -> AtlasResult<Vec<InterestInvoiceLine>> {
        self.repository.list_interest_invoice_lines(interest_invoice_id).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get interest invoice dashboard summary
    pub async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<InterestInvoiceDashboard> {
        self.repository.get_dashboard_summary(org_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_compounding_frequencies() {
        assert!(VALID_COMPOUNDING_FREQUENCIES.contains(&"daily"));
        assert!(VALID_COMPOUNDING_FREQUENCIES.contains(&"monthly"));
        assert!(VALID_COMPOUNDING_FREQUENCIES.contains(&"quarterly"));
        assert!(VALID_COMPOUNDING_FREQUENCIES.contains(&"annual"));
    }

    #[test]
    fn test_valid_charge_types() {
        assert!(VALID_CHARGE_TYPES.contains(&"interest"));
        assert!(VALID_CHARGE_TYPES.contains(&"penalty"));
        assert!(VALID_CHARGE_TYPES.contains(&"mixed"));
    }

    #[test]
    fn test_valid_schedule_statuses() {
        assert!(VALID_SCHEDULE_STATUSES.contains(&"active"));
        assert!(VALID_SCHEDULE_STATUSES.contains(&"inactive"));
    }

    #[test]
    fn test_valid_invoice_statuses() {
        assert!(VALID_INVOICE_STATUSES.contains(&"draft"));
        assert!(VALID_INVOICE_STATUSES.contains(&"posted"));
        assert!(VALID_INVOICE_STATUSES.contains(&"reversed"));
        assert!(VALID_INVOICE_STATUSES.contains(&"cancelled"));
    }

    #[test]
    fn test_calculate_simple_interest_basic() {
        // $10,000 at 12% annual for 30 days
        let interest = calculate_simple_interest(10000.0, 12.0, 30);
        let expected = 10000.0 * 0.12 * (30.0 / 365.0);
        assert!((interest - expected).abs() < 0.01);
    }

    #[test]
    fn test_calculate_simple_interest_zero_rate() {
        let interest = calculate_simple_interest(10000.0, 0.0, 30);
        assert_eq!(interest, 0.0);
    }

    #[test]
    fn test_calculate_simple_interest_full_year() {
        // $10,000 at 10% for 365 days = $1,000
        let interest = calculate_simple_interest(10000.0, 10.0, 365);
        assert!((interest - 1000.0).abs() < 0.01);
    }

    #[test]
    fn test_calculate_simple_interest_zero_days() {
        let interest = calculate_simple_interest(10000.0, 12.0, 0);
        assert_eq!(interest, 0.0);
    }

    #[test]
    fn test_calculate_simple_interest_large_amount() {
        // $1,000,000 at 8% for 90 days
        let interest = calculate_simple_interest(1000000.0, 8.0, 90);
        let expected = 1000000.0 * 0.08 * (90.0 / 365.0);
        assert!((interest - expected).abs() < 0.01);
        // Approx $19,726
        assert!(interest > 19000.0 && interest < 20000.0);
    }
}
