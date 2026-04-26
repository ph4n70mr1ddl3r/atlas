//! Payroll Engine Implementation
//!
//! Manages payroll definitions, elements (earnings & deductions),
//! employee element entries, payroll run lifecycle, pay slip generation,
//! and payroll calculations (gross-to-net).
//!
//! Oracle Fusion Cloud HCM equivalent: Global Payroll

use atlas_shared::{
    PayrollDefinition, PayrollElement, PayrollElementEntry,
    PayrollRun, PaySlip, PaySlipLine, PayrollDashboard,
    AtlasError, AtlasResult,
};
use super::PayrollRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Valid pay frequencies
const VALID_PAY_FREQUENCIES: &[&str] = &[
    "weekly", "biweekly", "semimonthly", "monthly",
];

/// Valid element types
const VALID_ELEMENT_TYPES: &[&str] = &[
    "earning", "deduction",
];

/// Valid element categories
const VALID_ELEMENT_CATEGORIES: &[&str] = &[
    "salary", "hourly", "overtime", "bonus", "commission",
    "benefit", "tax", "retirement", "garnishment", "other",
];

/// Valid calculation methods
const VALID_CALC_METHODS: &[&str] = &[
    "flat", "percentage", "hourly_rate", "formula",
];

/// Valid payroll run statuses
const VALID_RUN_STATUSES: &[&str] = &[
    "open", "calculated", "confirmed", "paid", "reversed",
];

/// Payroll engine for managing payroll processing
pub struct PayrollEngine {
    repository: Arc<dyn PayrollRepository>,
}

impl PayrollEngine {
    pub fn new(repository: Arc<dyn PayrollRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Payroll Definition Management
    // ========================================================================

    /// Create a new payroll definition (pay group)
    pub async fn create_payroll(
        &self,
        org_id: Uuid,
        name: &str,
        description: Option<&str>,
        pay_frequency: &str,
        currency_code: &str,
        salary_expense_account: Option<&str>,
        liability_account: Option<&str>,
        employer_tax_account: Option<&str>,
        payment_account: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PayrollDefinition> {
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Payroll name is required".to_string(),
            ));
        }
        if !VALID_PAY_FREQUENCIES.contains(&pay_frequency) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid pay_frequency '{}'. Must be one of: {}",
                pay_frequency, VALID_PAY_FREQUENCIES.join(", ")
            )));
        }
        if currency_code.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Currency code is required".to_string(),
            ));
        }

        // Check unique name within org
        if self.repository.get_payroll_by_name(org_id, name).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Payroll definition '{}' already exists", name
            )));
        }

        info!("Creating payroll definition '{}' for org {}", name, org_id);

        self.repository.create_payroll(
            org_id, name, description, pay_frequency, currency_code,
            salary_expense_account, liability_account,
            employer_tax_account, payment_account, created_by,
        ).await
    }

    /// Get a payroll definition by ID
    pub async fn get_payroll(&self, id: Uuid) -> AtlasResult<Option<PayrollDefinition>> {
        self.repository.get_payroll(id).await
    }

    /// List all payroll definitions for an organization
    pub async fn list_payrolls(&self, org_id: Uuid) -> AtlasResult<Vec<PayrollDefinition>> {
        self.repository.list_payrolls(org_id).await
    }

    /// Deactivate a payroll definition
    pub async fn delete_payroll(&self, id: Uuid) -> AtlasResult<()> {
        info!("Deactivating payroll definition {}", id);
        self.repository.delete_payroll(id).await
    }

    // ========================================================================
    // Element Management
    // ========================================================================

    /// Create a new payroll element (earning or deduction type)
    pub async fn create_element(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        element_type: &str,
        category: &str,
        calculation_method: &str,
        default_value: Option<&str>,
        is_recurring: bool,
        has_employer_contribution: bool,
        employer_contribution_rate: Option<&str>,
        gl_account_code: Option<&str>,
        is_pretax: bool,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PayrollElement> {
        if code.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Element code is required".to_string(),
            ));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Element name is required".to_string(),
            ));
        }
        if !VALID_ELEMENT_TYPES.contains(&element_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid element_type '{}'. Must be one of: {}",
                element_type, VALID_ELEMENT_TYPES.join(", ")
            )));
        }
        if !VALID_ELEMENT_CATEGORIES.contains(&category) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid category '{}'. Must be one of: {}",
                category, VALID_ELEMENT_CATEGORIES.join(", ")
            )));
        }
        if !VALID_CALC_METHODS.contains(&calculation_method) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid calculation_method '{}'. Must be one of: {}",
                calculation_method, VALID_CALC_METHODS.join(", ")
            )));
        }

        // Validate calculation method compatibility
        if calculation_method == "percentage" {
            if let Some(val) = default_value {
                let pct: f64 = val.parse().map_err(|_| AtlasError::ValidationFailed(
                    "Percentage default_value must be a number".to_string(),
                ))?;
                if !(0.0..=100.0).contains(&pct) {
                    return Err(AtlasError::ValidationFailed(
                        "Percentage must be between 0 and 100".to_string(),
                    ));
                }
            }
        }

        // Validate employer contribution
        if has_employer_contribution {
            if employer_contribution_rate.is_none() {
                return Err(AtlasError::ValidationFailed(
                    "Employer contribution rate is required when has_employer_contribution is true".to_string(),
                ));
            }
            if let Some(rate) = employer_contribution_rate {
                let r: f64 = rate.parse().map_err(|_| AtlasError::ValidationFailed(
                    "Employer contribution rate must be a number".to_string(),
                ))?;
                if r < 0.0 {
                    return Err(AtlasError::ValidationFailed(
                        "Employer contribution rate cannot be negative".to_string(),
                    ));
                }
            }
        }

        // Validate date range
        if let (Some(from), Some(to)) = (effective_from, effective_to) {
            if from > to {
                return Err(AtlasError::ValidationFailed(
                    "effective_from must be before effective_to".to_string(),
                ));
            }
        }

        // Check unique code
        if self.repository.get_element_by_code(org_id, code).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Element code '{}' already exists", code
            )));
        }

        // Only deductions can be pretax
        if is_pretax && element_type != "deduction" {
            return Err(AtlasError::ValidationFailed(
                "Only deductions can be marked as pretax".to_string(),
            ));
        }

        info!("Creating payroll element '{}' ({}) for org {}", code, element_type, org_id);

        self.repository.create_element(
            org_id, code, name, description, element_type, category,
            calculation_method, default_value, is_recurring,
            has_employer_contribution, employer_contribution_rate,
            gl_account_code, is_pretax, effective_from, effective_to,
            created_by,
        ).await
    }

    /// Get an element by ID
    pub async fn get_element(&self, id: Uuid) -> AtlasResult<Option<PayrollElement>> {
        self.repository.get_element(id).await
    }

    /// List elements, optionally filtered by type
    pub async fn list_elements(&self, org_id: Uuid, element_type: Option<&str>) -> AtlasResult<Vec<PayrollElement>> {
        if let Some(et) = element_type {
            if !VALID_ELEMENT_TYPES.contains(&et) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid element_type filter '{}'. Must be one of: {}",
                    et, VALID_ELEMENT_TYPES.join(", ")
                )));
            }
        }
        self.repository.list_elements(org_id, element_type).await
    }

    /// Deactivate an element
    pub async fn delete_element(&self, id: Uuid) -> AtlasResult<()> {
        info!("Deactivating payroll element {}", id);
        self.repository.delete_element(id).await
    }

    // ========================================================================
    // Element Entry Management (Employee Assignments)
    // ========================================================================

    /// Assign a payroll element to an employee
    pub async fn assign_element(
        &self,
        org_id: Uuid,
        employee_id: Uuid,
        element_id: Uuid,
        entry_value: &str,
        remaining_periods: Option<i32>,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PayrollElementEntry> {
        // Validate element exists
        let element = self.repository.get_element(element_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Element {} not found", element_id)
            ))?;

        if !element.is_active {
            return Err(AtlasError::ValidationFailed(
                "Cannot assign inactive element".to_string(),
            ));
        }

        // Validate value is numeric
        let _: f64 = entry_value.parse().map_err(|_| AtlasError::ValidationFailed(
            "Entry value must be a valid number".to_string(),
        ))?;

        // Validate value is non-negative for earnings
        if element.element_type == "earning" {
            let val: f64 = entry_value.parse().unwrap_or(0.0);
            if val < 0.0 {
                return Err(AtlasError::ValidationFailed(
                    "Earning value cannot be negative".to_string(),
                ));
            }
        }

        // Validate date range
        if let (Some(from), Some(to)) = (effective_from, effective_to) {
            if from > to {
                return Err(AtlasError::ValidationFailed(
                    "effective_from must be before effective_to".to_string(),
                ));
            }
        }

        info!("Assigning element '{}' to employee {}", element.code, employee_id);

        self.repository.create_entry(
            org_id, employee_id, element_id,
            &element.code, &element.name, &element.element_type,
            entry_value, remaining_periods,
            effective_from, effective_to, created_by,
        ).await
    }

    /// Get all element entries for an employee
    pub async fn get_employee_entries(&self, employee_id: Uuid) -> AtlasResult<Vec<PayrollElementEntry>> {
        self.repository.get_entries_by_employee(employee_id).await
    }

    /// Remove an element entry
    pub async fn remove_entry(&self, id: Uuid) -> AtlasResult<()> {
        info!("Removing element entry {}", id);
        self.repository.delete_entry(id).await
    }

    // ========================================================================
    // Payroll Run Lifecycle
    // ========================================================================

    /// Create a new payroll run for a pay period
    pub async fn create_run(
        &self,
        org_id: Uuid,
        payroll_id: Uuid,
        period_start: chrono::NaiveDate,
        period_end: chrono::NaiveDate,
        pay_date: chrono::NaiveDate,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PayrollRun> {
        // Validate payroll exists
        let payroll = self.repository.get_payroll(payroll_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Payroll definition {} not found", payroll_id)
            ))?;

        if !payroll.is_active {
            return Err(AtlasError::ValidationFailed(
                "Cannot create run for inactive payroll".to_string(),
            ));
        }

        // Validate dates
        if period_start >= period_end {
            return Err(AtlasError::ValidationFailed(
                "Period start must be before period end".to_string(),
            ));
        }
        if pay_date < period_end {
            return Err(AtlasError::ValidationFailed(
                "Pay date must be on or after period end".to_string(),
            ));
        }

        // Generate run number
        let run_number = format!("PR-{}", chrono::Utc::now().format("%Y%m%d%H%M%S%f"));

        info!("Creating payroll run {} for payroll {}", run_number, payroll.name);

        self.repository.create_run(
            org_id, payroll_id, &run_number,
            period_start, period_end, pay_date, created_by,
        ).await
    }

    /// Get a payroll run by ID
    pub async fn get_run(&self, id: Uuid) -> AtlasResult<Option<PayrollRun>> {
        self.repository.get_run(id).await
    }

    /// List payroll runs
    pub async fn list_runs(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<PayrollRun>> {
        if let Some(s) = status {
            if !VALID_RUN_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}",
                    s, VALID_RUN_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_runs(org_id, status).await
    }

    /// Calculate payroll for a run.
    ///
    /// This processes all employees assigned to the payroll's pay group,
    /// computes earnings, applies pretax deductions, computes taxes,
    /// applies post-tax deductions, and generates pay slips.
    ///
    /// For this implementation we accept employee IDs + their entries
    /// explicitly since the employee directory is in atlas-hcm.
    pub async fn calculate_run(
        &self,
        run_id: Uuid,
        employee_data: &[EmployeePayrollInput],
    ) -> AtlasResult<PayrollRun> {
        let run = self.repository.get_run(run_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Payroll run {} not found", run_id)
            ))?;

        if run.status != "open" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot calculate run in '{}' status. Must be 'open'.", run.status
            )));
        }

        if employee_data.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "No employees to process".to_string(),
            ));
        }

        let mut total_gross = 0.0f64;
        let mut total_deductions = 0.0f64;
        let mut total_net = 0.0f64;
        let mut total_employer_cost = 0.0f64;

        for emp in employee_data {
            let (gross, deductions, net, employer_cost, lines) =
                self.calculate_employee_pay(&emp.entries, emp.annual_salary)?;

            total_gross += gross;
            total_deductions += deductions;
            total_net += net;
            total_employer_cost += employer_cost;

            // Create pay slip
            let slip = self.repository.create_pay_slip(
                run.organization_id,
                run_id,
                emp.employee_id,
                emp.employee_name.as_deref(),
                &format!("{:.2}", gross),
                &format!("{:.2}", deductions),
                &format!("{:.2}", net),
                &format!("{:.2}", employer_cost),
                "USD",
                emp.payment_method.as_deref(),
                emp.bank_account_last4.as_deref(),
            ).await?;

            // Create pay slip lines
            for line in lines {
                self.repository.create_pay_slip_line(
                    slip.id,
                    &line.element_code,
                    &line.element_name,
                    &line.element_type,
                    &line.category,
                    line.hours_or_units.as_deref(),
                    line.rate.as_deref(),
                    &line.amount,
                    line.is_pretax,
                    line.is_employer,
                    line.gl_account_code.as_deref(),
                ).await?;
            }
        }

        // Update run totals
        self.repository.update_run_totals(
            run_id,
            &format!("{:.2}", total_gross),
            &format!("{:.2}", total_deductions),
            &format!("{:.2}", total_net),
            &format!("{:.2}", total_employer_cost),
            employee_data.len() as i32,
        ).await?;

        let run = self.repository.update_run_status(run_id, "calculated", None).await?;

        info!(
            "Payroll run {} calculated: {} employees, gross={:.2}, net={:.2}",
            run.run_number, employee_data.len(), total_gross, total_net
        );

        Ok(run)
    }

    /// Confirm a calculated payroll run
    pub async fn confirm_run(&self, run_id: Uuid, confirmed_by: Uuid) -> AtlasResult<PayrollRun> {
        let run = self.repository.get_run(run_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Payroll run {} not found", run_id)
            ))?;

        if run.status != "calculated" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot confirm run in '{}' status. Must be 'calculated'.", run.status
            )));
        }

        info!("Confirming payroll run {} by {}", run.run_number, confirmed_by);
        self.repository.update_run_status(run_id, "confirmed", Some(confirmed_by)).await
    }

    /// Mark a confirmed run as paid
    pub async fn mark_paid(&self, run_id: Uuid, paid_by: Uuid) -> AtlasResult<PayrollRun> {
        let run = self.repository.get_run(run_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Payroll run {} not found", run_id)
            ))?;

        if run.status != "confirmed" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot mark run as paid in '{}' status. Must be 'confirmed'.", run.status
            )));
        }

        info!("Marking payroll run {} as paid by {}", run.run_number, paid_by);
        self.repository.update_run_status(run_id, "paid", Some(paid_by)).await
    }

    /// Reverse a payroll run
    pub async fn reverse_run(&self, run_id: Uuid) -> AtlasResult<PayrollRun> {
        let run = self.repository.get_run(run_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Payroll run {} not found", run_id)
            ))?;

        if run.status != "paid" && run.status != "confirmed" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot reverse run in '{}' status. Must be 'paid' or 'confirmed'.", run.status
            )));
        }

        info!("Reversing payroll run {}", run.run_number);
        self.repository.update_run_status(run_id, "reversed", None).await
    }

    // ========================================================================
    // Pay Slip Queries
    // ========================================================================

    /// Get pay slips for a payroll run
    pub async fn get_run_pay_slips(&self, run_id: Uuid) -> AtlasResult<Vec<PaySlip>> {
        self.repository.list_pay_slips_by_run(run_id).await
    }

    /// Get pay slips for an employee
    pub async fn get_employee_pay_slips(&self, employee_id: Uuid) -> AtlasResult<Vec<PaySlip>> {
        self.repository.list_pay_slips_by_employee(employee_id).await
    }

    /// Get a single pay slip with its lines
    pub async fn get_pay_slip(&self, id: Uuid) -> AtlasResult<Option<PaySlip>> {
        let mut slip = self.repository.get_pay_slip(id).await?;
        if let Some(ref mut s) = slip {
            s.lines = self.repository.list_pay_slip_lines(id).await?;
        }
        Ok(slip)
    }

    /// Get payroll dashboard summary
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<PayrollDashboard> {
        self.repository.get_dashboard(org_id).await
    }

    // ========================================================================
    // Pay Calculation Engine (Gross-to-Net)
    // ========================================================================

    /// Calculate pay for a single employee given their element entries.
    ///
    /// Processing order:
    /// 1. Sum all earnings → gross pay
    /// 2. Apply pretax deductions → taxable income
    /// 3. Apply tax deductions (percentage of taxable income)
    /// 4. Apply post-tax deductions
    /// 5. Net pay = gross - all deductions
    ///
    /// Returns (gross, total_deductions, net_pay, employer_cost, lines).
    pub fn calculate_employee_pay(
        &self,
        entries: &[ElementEntryInput],
        annual_salary: Option<f64>,
    ) -> AtlasResult<(f64, f64, f64, f64, Vec<PaySlipLine>)> {
        let mut lines: Vec<PaySlipLine> = Vec::new();
        let mut gross = 0.0f64;
        let mut pretax_deductions = 0.0f64;
        let mut tax_deductions = 0.0f64;
        let mut posttax_deductions = 0.0f64;
        let mut employer_cost = 0.0f64;

        // 1. Calculate earnings
        for entry in entries.iter().filter(|e| e.element_type == "earning") {
            let amount = match entry.calculation_method.as_str() {
                "flat" => entry.entry_value,
                "percentage" => {
                    let base = annual_salary.unwrap_or(0.0) / 12.0;
                    entry.entry_value / 100.0 * base
                }
                "hourly_rate" => entry.hours_or_units.unwrap_or(0.0) * entry.entry_value,
                _ => entry.entry_value,
            };

            gross += amount;

            lines.push(PaySlipLine {
                id: Uuid::new_v4(),
                pay_slip_id: Uuid::nil(), // Will be set when persisted
                element_code: entry.element_code.clone(),
                element_name: entry.element_name.clone(),
                element_type: "earning".to_string(),
                category: entry.category.clone(),
                hours_or_units: entry.hours_or_units.map(|h| format!("{:.2}", h)),
                rate: Some(format!("{:.2}", entry.entry_value)),
                amount: format!("{:.2}", amount),
                is_pretax: false,
                is_employer: false,
                gl_account_code: entry.gl_account_code.clone(),
                created_at: chrono::Utc::now(),
            });
        }

        // Default gross from salary if no explicit salary earning
        if annual_salary.is_some() && !entries.iter().any(|e| e.element_type == "earning" && e.category == "salary") {
            let salary_monthly = annual_salary.unwrap_or(0.0) / 12.0;
            gross += salary_monthly;
            lines.push(PaySlipLine {
                id: Uuid::new_v4(),
                pay_slip_id: Uuid::nil(),
                element_code: "SALARY".to_string(),
                element_name: "Base Salary".to_string(),
                element_type: "earning".to_string(),
                category: "salary".to_string(),
                hours_or_units: None,
                rate: Some(format!("{:.2}", salary_monthly)),
                amount: format!("{:.2}", salary_monthly),
                is_pretax: false,
                is_employer: false,
                gl_account_code: None,
                created_at: chrono::Utc::now(),
            });
        }

        // 2. Calculate pretax deductions (reduce taxable income)
        for entry in entries.iter().filter(|e| {
            e.element_type == "deduction" && e.is_pretax && e.category != "tax"
        }) {
            let amount = self.calculate_deduction_amount(entry, gross);
            pretax_deductions += amount;

            lines.push(PaySlipLine {
                id: Uuid::new_v4(),
                pay_slip_id: Uuid::nil(),
                element_code: entry.element_code.clone(),
                element_name: entry.element_name.clone(),
                element_type: "deduction".to_string(),
                category: entry.category.clone(),
                hours_or_units: None,
                rate: Some(format!("{:.2}", entry.entry_value)),
                amount: format!("{:.2}", amount),
                is_pretax: true,
                is_employer: false,
                gl_account_code: entry.gl_account_code.clone(),
                created_at: chrono::Utc::now(),
            });

            // Employer contribution
            if let Some(rate) = entry.employer_contribution_rate {
                let emp_amount = rate / 100.0 * gross;
                employer_cost += emp_amount;
                lines.push(PaySlipLine {
                    id: Uuid::new_v4(),
                    pay_slip_id: Uuid::nil(),
                    element_code: format!("{}_ER", entry.element_code),
                    element_name: format!("{} (Employer)", entry.element_name),
                    element_type: "deduction".to_string(),
                    category: entry.category.clone(),
                    hours_or_units: None,
                    rate: Some(format!("{:.2}", rate)),
                    amount: format!("{:.2}", emp_amount),
                    is_pretax: false,
                    is_employer: true,
                    gl_account_code: entry.gl_account_code.clone(),
                    created_at: chrono::Utc::now(),
                });
            }
        }

        // 3. Calculate taxable income after pretax deductions
        let taxable_income = (gross - pretax_deductions).max(0.0);

        // 4. Calculate tax deductions (applied to taxable income)
        for entry in entries.iter().filter(|e| {
            e.element_type == "deduction" && e.category == "tax"
        }) {
            let amount = match entry.calculation_method.as_str() {
                "percentage" => entry.entry_value / 100.0 * taxable_income,
                "flat" => entry.entry_value,
                _ => entry.entry_value,
            };
            tax_deductions += amount;

            lines.push(PaySlipLine {
                id: Uuid::new_v4(),
                pay_slip_id: Uuid::nil(),
                element_code: entry.element_code.clone(),
                element_name: entry.element_name.clone(),
                element_type: "deduction".to_string(),
                category: "tax".to_string(),
                hours_or_units: None,
                rate: Some(format!("{:.2}", entry.entry_value)),
                amount: format!("{:.2}", amount),
                is_pretax: false,
                is_employer: false,
                gl_account_code: entry.gl_account_code.clone(),
                created_at: chrono::Utc::now(),
            });

            // Employer tax
            if let Some(rate) = entry.employer_contribution_rate {
                let emp_amount = rate / 100.0 * taxable_income;
                employer_cost += emp_amount;
                lines.push(PaySlipLine {
                    id: Uuid::new_v4(),
                    pay_slip_id: Uuid::nil(),
                    element_code: format!("{}_ER", entry.element_code),
                    element_name: format!("{} (Employer)", entry.element_name),
                    element_type: "deduction".to_string(),
                    category: "tax".to_string(),
                    hours_or_units: None,
                    rate: Some(format!("{:.2}", rate)),
                    amount: format!("{:.2}", emp_amount),
                    is_pretax: false,
                    is_employer: true,
                    gl_account_code: None,
                    created_at: chrono::Utc::now(),
                });
            }
        }

        // 5. Calculate post-tax deductions
        for entry in entries.iter().filter(|e| {
            e.element_type == "deduction" && !e.is_pretax && e.category != "tax"
        }) {
            let amount = self.calculate_deduction_amount(entry, gross);
            posttax_deductions += amount;

            lines.push(PaySlipLine {
                id: Uuid::new_v4(),
                pay_slip_id: Uuid::nil(),
                element_code: entry.element_code.clone(),
                element_name: entry.element_name.clone(),
                element_type: "deduction".to_string(),
                category: entry.category.clone(),
                hours_or_units: None,
                rate: Some(format!("{:.2}", entry.entry_value)),
                amount: format!("{:.2}", amount),
                is_pretax: false,
                is_employer: false,
                gl_account_code: entry.gl_account_code.clone(),
                created_at: chrono::Utc::now(),
            });

            if let Some(rate) = entry.employer_contribution_rate {
                let emp_amount = rate / 100.0 * gross;
                employer_cost += emp_amount;
                lines.push(PaySlipLine {
                    id: Uuid::new_v4(),
                    pay_slip_id: Uuid::nil(),
                    element_code: format!("{}_ER", entry.element_code),
                    element_name: format!("{} (Employer)", entry.element_name),
                    element_type: "deduction".to_string(),
                    category: entry.category.clone(),
                    hours_or_units: None,
                    rate: Some(format!("{:.2}", rate)),
                    amount: format!("{:.2}", emp_amount),
                    is_pretax: false,
                    is_employer: true,
                    gl_account_code: None,
                    created_at: chrono::Utc::now(),
                });
            }
        }

        let total_deductions = pretax_deductions + tax_deductions + posttax_deductions;
        let net_pay = (gross - total_deductions).max(0.0);

        Ok((gross, total_deductions, net_pay, employer_cost, lines))
    }

    /// Calculate a single deduction amount based on calculation method
    fn calculate_deduction_amount(&self, entry: &ElementEntryInput, gross: f64) -> f64 {
        match entry.calculation_method.as_str() {
            "percentage" => entry.entry_value / 100.0 * gross,
            "flat" => entry.entry_value,
            _ => entry.entry_value,
        }
    }
}

/// Input for an employee being processed in a payroll run.
#[derive(Debug, Clone)]
pub struct EmployeePayrollInput {
    pub employee_id: Uuid,
    pub employee_name: Option<String>,
    pub annual_salary: Option<f64>,
    pub payment_method: Option<String>,
    pub bank_account_last4: Option<String>,
    pub entries: Vec<ElementEntryInput>,
}

/// Simplified element entry input for payroll calculation.
#[derive(Debug, Clone)]
pub struct ElementEntryInput {
    pub element_code: String,
    pub element_name: String,
    pub element_type: String,
    pub category: String,
    pub calculation_method: String,
    pub entry_value: f64,
    pub hours_or_units: Option<f64>,
    pub is_pretax: bool,
    pub employer_contribution_rate: Option<f64>,
    pub gl_account_code: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_pay_frequencies() {
        assert!(VALID_PAY_FREQUENCIES.contains(&"weekly"));
        assert!(VALID_PAY_FREQUENCIES.contains(&"biweekly"));
        assert!(VALID_PAY_FREQUENCIES.contains(&"semimonthly"));
        assert!(VALID_PAY_FREQUENCIES.contains(&"monthly"));
        assert!(!VALID_PAY_FREQUENCIES.contains(&"daily"));
        assert!(!VALID_PAY_FREQUENCIES.contains(&"yearly"));
    }

    #[test]
    fn test_valid_element_types() {
        assert!(VALID_ELEMENT_TYPES.contains(&"earning"));
        assert!(VALID_ELEMENT_TYPES.contains(&"deduction"));
        assert!(!VALID_ELEMENT_TYPES.contains(&"tax"));
        assert!(!VALID_ELEMENT_TYPES.contains(&"benefit"));
    }

    #[test]
    fn test_valid_element_categories() {
        assert!(VALID_ELEMENT_CATEGORIES.contains(&"salary"));
        assert!(VALID_ELEMENT_CATEGORIES.contains(&"hourly"));
        assert!(VALID_ELEMENT_CATEGORIES.contains(&"overtime"));
        assert!(VALID_ELEMENT_CATEGORIES.contains(&"bonus"));
        assert!(VALID_ELEMENT_CATEGORIES.contains(&"commission"));
        assert!(VALID_ELEMENT_CATEGORIES.contains(&"benefit"));
        assert!(VALID_ELEMENT_CATEGORIES.contains(&"tax"));
        assert!(VALID_ELEMENT_CATEGORIES.contains(&"retirement"));
        assert!(VALID_ELEMENT_CATEGORIES.contains(&"garnishment"));
        assert!(VALID_ELEMENT_CATEGORIES.contains(&"other"));
        assert!(!VALID_ELEMENT_CATEGORIES.contains(&"invalid"));
    }

    #[test]
    fn test_valid_calculation_methods() {
        assert!(VALID_CALC_METHODS.contains(&"flat"));
        assert!(VALID_CALC_METHODS.contains(&"percentage"));
        assert!(VALID_CALC_METHODS.contains(&"hourly_rate"));
        assert!(VALID_CALC_METHODS.contains(&"formula"));
        assert!(!VALID_CALC_METHODS.contains(&"random"));
    }

    #[test]
    fn test_valid_run_statuses() {
        assert!(VALID_RUN_STATUSES.contains(&"open"));
        assert!(VALID_RUN_STATUSES.contains(&"calculated"));
        assert!(VALID_RUN_STATUSES.contains(&"confirmed"));
        assert!(VALID_RUN_STATUSES.contains(&"paid"));
        assert!(VALID_RUN_STATUSES.contains(&"reversed"));
        assert!(!VALID_RUN_STATUSES.contains(&"cancelled"));
    }

    // ========================================================================
    // Pay Calculation Tests
    // ========================================================================

    /// Helper: create a PayrollEngine with a mock repo for unit tests
    fn test_engine() -> PayrollEngine {
        PayrollEngine::new(Arc::new(crate::mock_repos::MockPayrollRepository))
    }

    #[test]
    fn test_calculate_employee_pay_salary_only() {
        let engine = test_engine();

        let (gross, deductions, net, employer_cost, lines) =
            engine.calculate_employee_pay(&[], Some(120_000.0)).unwrap();

        // Monthly salary = 120000 / 12 = 10000
        assert!((gross - 10_000.0).abs() < 0.01);
        assert!((deductions - 0.0).abs() < 0.01);
        assert!((net - 10_000.0).abs() < 0.01);
        assert!((employer_cost - 0.0).abs() < 0.01);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].element_code, "SALARY");
        assert_eq!(lines[0].element_type, "earning");
    }

    #[test]
    fn test_calculate_employee_pay_with_tax_deduction() {
        let engine = test_engine();

        let entries = vec![
            ElementEntryInput {
                element_code: "FED_TAX".to_string(),
                element_name: "Federal Income Tax".to_string(),
                element_type: "deduction".to_string(),
                category: "tax".to_string(),
                calculation_method: "percentage".to_string(),
                entry_value: 22.0, // 22%
                hours_or_units: None,
                is_pretax: false,
                employer_contribution_rate: None,
                gl_account_code: None,
            },
        ];

        let (gross, deductions, net, employer_cost, lines) =
            engine.calculate_employee_pay(&entries, Some(120_000.0)).unwrap();

        // gross = 10000, tax = 22% of 10000 = 2200, net = 7800
        assert!((gross - 10_000.0).abs() < 0.01);
        assert!((deductions - 2_200.0).abs() < 0.01);
        assert!((net - 7_800.0).abs() < 0.01);
        assert!((employer_cost - 0.0).abs() < 0.01);
        assert_eq!(lines.len(), 2); // salary + tax

        let tax_line = lines.iter().find(|l| l.element_code == "FED_TAX").unwrap();
        assert_eq!(tax_line.element_type, "deduction");
        assert_eq!(tax_line.category, "tax");
        let tax_amount: f64 = tax_line.amount.parse().unwrap();
        assert!((tax_amount - 2_200.0).abs() < 0.01);
    }

    #[test]
    fn test_calculate_employee_pay_pretax_deduction_reduces_taxable_income() {
        let engine = test_engine();

        let entries = vec![
            // Pretax retirement deduction: 5% of gross
            ElementEntryInput {
                element_code: "401K".to_string(),
                element_name: "401(k) Contribution".to_string(),
                element_type: "deduction".to_string(),
                category: "retirement".to_string(),
                calculation_method: "percentage".to_string(),
                entry_value: 5.0,
                hours_or_units: None,
                is_pretax: true,
                employer_contribution_rate: Some(3.0), // 3% employer match
                gl_account_code: None,
            },
            // Tax: 22% of taxable income (after pretax)
            ElementEntryInput {
                element_code: "FED_TAX".to_string(),
                element_name: "Federal Income Tax".to_string(),
                element_type: "deduction".to_string(),
                category: "tax".to_string(),
                calculation_method: "percentage".to_string(),
                entry_value: 22.0,
                hours_or_units: None,
                is_pretax: false,
                employer_contribution_rate: Some(7.65), // FICA employer
                gl_account_code: None,
            },
        ];

        let (gross, deductions, net, employer_cost, lines) =
            engine.calculate_employee_pay(&entries, Some(120_000.0)).unwrap();

        // gross = 10000
        // pretax 401k = 5% of 10000 = 500
        // taxable = 10000 - 500 = 9500
        // fed tax = 22% of 9500 = 2090
        // total deductions = 500 + 2090 = 2590
        // net = 10000 - 2590 = 7410
        // employer cost = 3% of 10000 (401k match) + 7.65% of 9500 (FICA) = 300 + 726.75 = 1026.75
        assert!((gross - 10_000.0).abs() < 0.01);
        assert!((deductions - 2_590.0).abs() < 0.01);
        assert!((net - 7_410.0).abs() < 0.01);
        assert!((employer_cost - 1_026.75).abs() < 0.01);

        // Lines: salary, 401k, 401k_ER, fed_tax, fed_tax_ER
        assert_eq!(lines.len(), 5);

        let k401_line = lines.iter().find(|l| l.element_code == "401K").unwrap();
        assert!(k401_line.is_pretax);
        let k401_amount: f64 = k401_line.amount.parse().unwrap();
        assert!((k401_amount - 500.0).abs() < 0.01);
    }

    #[test]
    fn test_calculate_employee_pay_hourly_earning() {
        let engine = test_engine();

        let entries = vec![
            ElementEntryInput {
                element_code: "OVERTIME".to_string(),
                element_name: "Overtime Pay".to_string(),
                element_type: "earning".to_string(),
                category: "overtime".to_string(),
                calculation_method: "hourly_rate".to_string(),
                entry_value: 50.0, // $50/hour overtime rate
                hours_or_units: Some(10.0), // 10 hours overtime
                is_pretax: false,
                employer_contribution_rate: None,
                gl_account_code: None,
            },
        ];

        let (gross, deductions, net, employer_cost, lines) =
            engine.calculate_employee_pay(&entries, Some(120_000.0)).unwrap();

        // salary = 10000, overtime = 10 * 50 = 500, gross = 10500
        assert!((gross - 10_500.0).abs() < 0.01);
        assert!((deductions - 0.0).abs() < 0.01);
        assert!((net - 10_500.0).abs() < 0.01);
        assert!((employer_cost - 0.0).abs() < 0.01);

        let ot_line = lines.iter().find(|l| l.element_code == "OVERTIME").unwrap();
        let ot_amount: f64 = ot_line.amount.parse().unwrap();
        assert!((ot_amount - 500.0).abs() < 0.01);
        assert_eq!(ot_line.category, "overtime");
    }

    #[test]
    fn test_calculate_employee_pay_flat_deduction() {
        let engine = test_engine();

        let entries = vec![
            ElementEntryInput {
                element_code: "HEALTH_INS".to_string(),
                element_name: "Health Insurance".to_string(),
                element_type: "deduction".to_string(),
                category: "benefit".to_string(),
                calculation_method: "flat".to_string(),
                entry_value: 250.0, // $250 flat per pay period
                hours_or_units: None,
                is_pretax: false,
                employer_contribution_rate: None,
                gl_account_code: None,
            },
        ];

        let (gross, deductions, net, _employer_cost, lines) =
            engine.calculate_employee_pay(&entries, Some(120_000.0)).unwrap();

        // gross = 10000, deduction = 250 flat, net = 9750
        assert!((gross - 10_000.0).abs() < 0.01);
        assert!((deductions - 250.0).abs() < 0.01);
        assert!((net - 9_750.0).abs() < 0.01);

        let ins_line = lines.iter().find(|l| l.element_code == "HEALTH_INS").unwrap();
        let ins_amount: f64 = ins_line.amount.parse().unwrap();
        assert!((ins_amount - 250.0).abs() < 0.01);
        assert!(!ins_line.is_pretax);
    }

    #[test]
    fn test_calculate_employee_pay_no_salary() {
        let engine = test_engine();

        // No salary, no entries
        let (gross, deductions, net, employer_cost, lines) =
            engine.calculate_employee_pay(&[], None).unwrap();

        assert!((gross - 0.0).abs() < 0.01);
        assert!((deductions - 0.0).abs() < 0.01);
        assert!((net - 0.0).abs() < 0.01);
        assert!((employer_cost - 0.0).abs() < 0.01);
        assert!(lines.is_empty());
    }

    #[test]
    fn test_calculate_employee_pay_complex_scenario() {
        let engine = test_engine();

        let entries = vec![
            // Overtime earning
            ElementEntryInput {
                element_code: "OVERTIME".to_string(),
                element_name: "Overtime Pay".to_string(),
                element_type: "earning".to_string(),
                category: "overtime".to_string(),
                calculation_method: "hourly_rate".to_string(),
                entry_value: 45.0,
                hours_or_units: Some(8.0),
                is_pretax: false,
                employer_contribution_rate: None,
                gl_account_code: None,
            },
            // Pretax 401k: 6%
            ElementEntryInput {
                element_code: "401K".to_string(),
                element_name: "401(k)".to_string(),
                element_type: "deduction".to_string(),
                category: "retirement".to_string(),
                calculation_method: "percentage".to_string(),
                entry_value: 6.0,
                hours_or_units: None,
                is_pretax: true,
                employer_contribution_rate: Some(3.0),
                gl_account_code: None,
            },
            // Federal tax: 22%
            ElementEntryInput {
                element_code: "FED_TAX".to_string(),
                element_name: "Federal Tax".to_string(),
                element_type: "deduction".to_string(),
                category: "tax".to_string(),
                calculation_method: "percentage".to_string(),
                entry_value: 22.0,
                hours_or_units: None,
                is_pretax: false,
                employer_contribution_rate: Some(7.65),
                gl_account_code: None,
            },
            // State tax: 5%
            ElementEntryInput {
                element_code: "STATE_TAX".to_string(),
                element_name: "State Tax".to_string(),
                element_type: "deduction".to_string(),
                category: "tax".to_string(),
                calculation_method: "percentage".to_string(),
                entry_value: 5.0,
                hours_or_units: None,
                is_pretax: false,
                employer_contribution_rate: None,
                gl_account_code: None,
            },
            // Post-tax health insurance: $200 flat
            ElementEntryInput {
                element_code: "HEALTH".to_string(),
                element_name: "Health Insurance".to_string(),
                element_type: "deduction".to_string(),
                category: "benefit".to_string(),
                calculation_method: "flat".to_string(),
                entry_value: 200.0,
                hours_or_units: None,
                is_pretax: false,
                employer_contribution_rate: None,
                gl_account_code: None,
            },
        ];

        let (gross, deductions, net, employer_cost, lines) =
            engine.calculate_employee_pay(&entries, Some(120_000.0)).unwrap();

        // gross = salary(10000) + overtime(8*45=360) = 10360
        // pretax 401k = 6% of 10360 = 621.60
        // taxable = 10360 - 621.60 = 9738.40
        // fed tax = 22% of 9738.40 = 2142.448
        // state tax = 5% of 9738.40 = 486.92
        // posttax health = 200 flat
        // total deductions = 621.60 + 2142.448 + 486.92 + 200 = 3450.968
        // net = 10360 - 3450.968 = 6909.032

        assert!((gross - 10_360.0).abs() < 0.01);
        assert!((deductions - 3_450.97).abs() < 0.5);
        assert!((net - 6_909.03).abs() < 0.5);

        // employer cost = 3% of 10360 (401k) + 7.65% of 9738.40 (FICA) = 310.80 + 744.99 = 1055.79
        assert!((employer_cost - 1_055.79).abs() < 0.5);

        // Lines: salary, overtime, 401k, 401k_ER, fed_tax, fed_tax_ER, state_tax, health
        assert_eq!(lines.len(), 8);

        // Verify earning lines
        let earning_lines: Vec<_> = lines.iter().filter(|l| l.element_type == "earning").collect();
        assert_eq!(earning_lines.len(), 2); // salary + overtime

        // Verify deduction lines (employee)
        let emp_deduction_lines: Vec<_> = lines.iter()
            .filter(|l| l.element_type == "deduction" && !l.is_employer)
            .collect();
        assert_eq!(emp_deduction_lines.len(), 4); // 401k, fed_tax, state_tax, health

        // Verify employer lines
        let er_lines: Vec<_> = lines.iter().filter(|l| l.is_employer).collect();
        assert_eq!(er_lines.len(), 2); // 401k_ER, fed_tax_ER
    }

    #[test]
    fn test_calculate_employee_pay_net_cannot_go_negative() {
        let engine = test_engine();

        let entries = vec![
            // 100% tax would make net = 0, but let's go beyond
            ElementEntryInput {
                element_code: "MEGA_TAX".to_string(),
                element_name: "Mega Tax".to_string(),
                element_type: "deduction".to_string(),
                category: "tax".to_string(),
                calculation_method: "percentage".to_string(),
                entry_value: 100.0,
                hours_or_units: None,
                is_pretax: false,
                employer_contribution_rate: None,
                gl_account_code: None,
            },
            ElementEntryInput {
                element_code: "EXTRA_DED".to_string(),
                element_name: "Extra Deduction".to_string(),
                element_type: "deduction".to_string(),
                category: "other".to_string(),
                calculation_method: "flat".to_string(),
                entry_value: 5000.0,
                hours_or_units: None,
                is_pretax: false,
                employer_contribution_rate: None,
                gl_account_code: None,
            },
        ];

        let (gross, deductions, net, _employer_cost, _lines) =
            engine.calculate_employee_pay(&entries, Some(120_000.0)).unwrap();

        // gross = 10000, mega_tax = 10000, extra = 5000, total_ded = 15000
        // net = max(0, 10000 - 15000) = 0
        assert!((gross - 10_000.0).abs() < 0.01);
        assert!((deductions - 15_000.0).abs() < 0.01);
        assert!((net - 0.0).abs() < 0.01); // Clamped at 0
    }

    #[test]
    fn test_employer_contribution_on_posttax_deduction() {
        let engine = test_engine();

        let entries = vec![
            ElementEntryInput {
                element_code: "LIFE_INS".to_string(),
                element_name: "Life Insurance".to_string(),
                element_type: "deduction".to_string(),
                category: "benefit".to_string(),
                calculation_method: "flat".to_string(),
                entry_value: 50.0,
                hours_or_units: None,
                is_pretax: false,
                employer_contribution_rate: Some(100.0), // Employer pays 100% match
                gl_account_code: None,
            },
        ];

        let (gross, deductions, net, employer_cost, _lines) =
            engine.calculate_employee_pay(&entries, Some(120_000.0)).unwrap();

        // gross = 10000, employee deduction = 50 flat, net = 9950
        // employer cost = 100% of 10000 = 10000
        assert!((gross - 10_000.0).abs() < 0.01);
        assert!((deductions - 50.0).abs() < 0.01);
        assert!((net - 9_950.0).abs() < 0.01);
        assert!((employer_cost - 10_000.0).abs() < 0.01);
    }

    #[test]
    fn test_pay_slip_line_format() {
        let engine = test_engine();

        let entries = vec![ElementEntryInput {
            element_code: "OVERTIME".to_string(),
            element_name: "Overtime Pay".to_string(),
            element_type: "earning".to_string(),
            category: "overtime".to_string(),
            calculation_method: "hourly_rate".to_string(),
            entry_value: 75.0,
            hours_or_units: Some(5.0),
            is_pretax: false,
            employer_contribution_rate: None,
            gl_account_code: Some("5100".to_string()),
        }];

        let (_, _, _, _, lines) = engine.calculate_employee_pay(&entries, Some(60_000.0)).unwrap();

        let ot_line = lines.iter().find(|l| l.element_code == "OVERTIME").unwrap();
        assert_eq!(ot_line.category, "overtime");
        assert_eq!(ot_line.gl_account_code, Some("5100".to_string()));
        let amount: f64 = ot_line.amount.parse().unwrap();
        assert!((amount - 375.0).abs() < 0.01); // 5 * 75 = 375
        let hours: f64 = ot_line.hours_or_units.as_ref().unwrap().parse().unwrap();
        assert!((hours - 5.0).abs() < 0.01);
    }
}
