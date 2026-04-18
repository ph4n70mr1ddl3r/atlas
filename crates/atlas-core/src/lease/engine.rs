//! Lease Accounting Engine Implementation (ASC 842 / IFRS 16)
//!
//! Manages lease contracts, right-of-use (ROU) assets, lease liabilities,
//! amortization schedules, lease payments with escalation, modifications,
//! impairment, and termination.
//!
//! Supports both operating and finance lease classification per ASC 842.
//! Calculates present value of lease payments using the incremental borrowing rate.
//! Generates amortization schedules with interest and principal breakdowns.
//!
//! Oracle Fusion Cloud ERP equivalent: Lease Management

use chrono::Datelike;
use atlas_shared::{
    LeaseContract, LeasePayment, LeaseModification, LeaseTermination,
    LeaseDashboardSummary,
    AtlasError, AtlasResult,
};
use super::LeaseAccountingRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Valid lease classifications
const VALID_CLASSIFICATIONS: &[&str] = &["operating", "finance"];

/// Valid lease statuses
const VALID_STATUSES: &[&str] = &[
    "draft", "active", "modified", "impaired", "terminated", "expired",
];

/// Valid payment frequencies
const VALID_FREQUENCIES: &[&str] = &["monthly", "quarterly", "annually"];

/// Valid modification types
const VALID_MODIFICATION_TYPES: &[&str] = &[
    "term_extension", "scope_change", "payment_change", "rate_change",
    "reclassification",
];

/// Valid termination types
const VALID_TERMINATION_TYPES: &[&str] = &[
    "early", "end_of_term", "mutual_agreement", "default",
];

/// Lease Accounting engine for managing leases per ASC 842 / IFRS 16
pub struct LeaseAccountingEngine {
    repository: Arc<dyn LeaseAccountingRepository>,
}

impl LeaseAccountingEngine {
    pub fn new(repository: Arc<dyn LeaseAccountingRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Lease Contract Management
    // ========================================================================

    /// Create a new lease contract
    pub async fn create_lease(
        &self,
        org_id: Uuid,
        title: &str,
        description: Option<&str>,
        classification: &str,
        lessor_id: Option<Uuid>,
        lessor_name: Option<&str>,
        asset_description: Option<&str>,
        location: Option<&str>,
        department_id: Option<Uuid>,
        department_name: Option<&str>,
        commencement_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
        lease_term_months: i32,
        purchase_option_exists: bool,
        purchase_option_likely: bool,
        renewal_option_exists: bool,
        renewal_option_months: Option<i32>,
        renewal_option_likely: bool,
        discount_rate: &str,
        currency_code: &str,
        payment_frequency: &str,
        annual_payment_amount: &str,
        escalation_rate: Option<&str>,
        escalation_frequency_months: Option<i32>,
        residual_guarantee_amount: Option<&str>,
        rou_asset_account_code: Option<&str>,
        rou_depreciation_account_code: Option<&str>,
        lease_liability_account_code: Option<&str>,
        lease_expense_account_code: Option<&str>,
        interest_expense_account_code: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<LeaseContract> {
        if title.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Lease title is required".to_string(),
            ));
        }
        if !VALID_CLASSIFICATIONS.contains(&classification) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid classification '{}'. Must be one of: {}",
                classification, VALID_CLASSIFICATIONS.join(", ")
            )));
        }
        if !VALID_FREQUENCIES.contains(&payment_frequency) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid payment frequency '{}'. Must be one of: {}",
                payment_frequency, VALID_FREQUENCIES.join(", ")
            )));
        }
        if commencement_date >= end_date {
            return Err(AtlasError::ValidationFailed(
                "Commencement date must be before end date".to_string(),
            ));
        }
        if lease_term_months <= 0 {
            return Err(AtlasError::ValidationFailed(
                "Lease term must be positive".to_string(),
            ));
        }

        let rate: f64 = discount_rate.parse().map_err(|_| AtlasError::ValidationFailed(
            "Discount rate must be a valid number".to_string(),
        ))?;
        if rate <= 0.0 || rate > 1.0 {
            return Err(AtlasError::ValidationFailed(
                "Discount rate must be between 0 and 1 (e.g., 0.05 for 5%)".to_string(),
            ));
        }

        let annual_payment: f64 = annual_payment_amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "Annual payment amount must be a valid number".to_string(),
        ))?;
        if annual_payment <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Annual payment amount must be positive".to_string(),
            ));
        }

        let esc_rate: f64 = escalation_rate
            .map(|r| r.parse::<f64>())
            .transpose()
            .map_err(|_| AtlasError::ValidationFailed(
                "Escalation rate must be a valid number".to_string(),
            ))?
            .unwrap_or(0.0);

        // Calculate number of payments based on frequency
        let periods_per_year = match payment_frequency {
            "monthly" => 12,
            "quarterly" => 4,
            "annually" => 1,
            _ => return Err(AtlasError::ValidationFailed("Invalid payment frequency".to_string())),
        };
        let total_periods = periods_per_year * lease_term_months / 12;
        let payment_per_period = annual_payment / periods_per_year as f64;

        // Calculate present value of lease payments (lease liability)
        let present_value = self.calculate_present_value(
            payment_per_period,
            total_periods,
            rate / periods_per_year as f64,
        );

        let residual: f64 = residual_guarantee_amount
            .map(|r| r.parse::<f64>())
            .transpose()
            .map_err(|_| AtlasError::ValidationFailed(
                "Residual guarantee amount must be a valid number".to_string(),
            ))?
            .unwrap_or(0.0);

        // Add PV of residual guarantee
        let pv_residual = if residual > 0.0 {
            residual / (1.0 + rate / periods_per_year as f64).powi(total_periods)
        } else {
            0.0
        };

        let total_liability = present_value + pv_residual;

        // Total lease payments (undiscounted)
        let total_lease_payments = self.calculate_total_lease_payments(
            payment_per_period, total_periods, esc_rate,
            escalation_frequency_months.map(|m| {
                let years = m as f64 / 12.0;
                (years * periods_per_year as f64).round() as i32
            }),
        );

        // ROU asset value = initial lease liability + any initial direct costs + prepayments
        // For simplicity, ROU asset = lease liability
        let rou_asset = total_liability;

        let lease_number = format!("LSE-{}", Uuid::new_v4().to_string()[..8].to_uppercase());

        info!("Creating lease {} ({}) for org {}: liability={:.2}, rou={:.2}",
              lease_number, title, org_id, total_liability, rou_asset);

        self.repository.create_lease(
            org_id, &lease_number, title, description,
            classification,
            lessor_id, lessor_name,
            asset_description, location,
            department_id, department_name,
            commencement_date, end_date, lease_term_months,
            purchase_option_exists, purchase_option_likely,
            renewal_option_exists, renewal_option_months, renewal_option_likely,
            discount_rate, currency_code, payment_frequency,
            &format!("{:.2}", annual_payment),
            escalation_rate, escalation_frequency_months,
            &format!("{:.2}", total_lease_payments),
            &format!("{:.2}", total_liability),
            &format!("{:.2}", rou_asset),
            residual_guarantee_amount,
            &format!("{:.2}", total_liability),
            &format!("{:.2}", rou_asset),
            "0.00",
            "0.00",
            0,
            rou_asset_account_code,
            rou_depreciation_account_code,
            lease_liability_account_code,
            lease_expense_account_code,
            interest_expense_account_code,
            created_by,
        ).await
    }

    /// Get a lease contract by ID
    pub async fn get_lease(&self, id: Uuid) -> AtlasResult<Option<LeaseContract>> {
        self.repository.get_lease(id).await
    }

    /// Get a lease contract by number
    pub async fn get_lease_by_number(&self, org_id: Uuid, lease_number: &str) -> AtlasResult<Option<LeaseContract>> {
        self.repository.get_lease_by_number(org_id, lease_number).await
    }

    /// List lease contracts with optional filters
    pub async fn list_leases(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        classification: Option<&str>,
    ) -> AtlasResult<Vec<LeaseContract>> {
        if let Some(s) = status {
            if !VALID_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_STATUSES.join(", ")
                )));
            }
        }
        if let Some(c) = classification {
            if !VALID_CLASSIFICATIONS.contains(&c) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid classification '{}'. Must be one of: {}", c, VALID_CLASSIFICATIONS.join(", ")
                )));
            }
        }
        self.repository.list_leases(org_id, status, classification).await
    }

    // ========================================================================
    // Lease Activation
    // ========================================================================

    /// Activate a draft lease and generate amortization schedule
    pub async fn activate_lease(&self, lease_id: Uuid, activated_by: Option<Uuid>) -> AtlasResult<LeaseContract> {
        let lease = self.repository.get_lease(lease_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Lease {} not found", lease_id)
            ))?;

        if lease.status != "draft" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot activate lease in '{}' status. Must be 'draft'.", lease.status)
            ));
        }

        // Generate amortization schedule
        self.generate_amortization_schedule(&lease).await?;

        // Update lease status
        let updated = self.repository.update_lease_status(lease_id, "active", None, None).await?;

        info!("Activated lease {} ({})", lease.lease_number, lease.title);
        Ok(updated)
    }

    // ========================================================================
    // Amortization Schedule
    // ========================================================================

    /// Generate the full amortization schedule for a lease
    async fn generate_amortization_schedule(&self, lease: &LeaseContract) -> AtlasResult<Vec<LeasePayment>> {
        let rate: f64 = lease.discount_rate.parse().unwrap_or(0.05);
        let periods_per_year = match lease.payment_frequency.as_str() {
            "monthly" => 12,
            "quarterly" => 4,
            "annually" => 1,
            _ => 12,
        };
        let total_periods = periods_per_year * lease.lease_term_months / 12;
        let period_rate = rate / periods_per_year as f64;

        let annual_payment: f64 = lease.total_lease_payments.parse()
            .unwrap_or(0.0) / (lease.lease_term_months as f64 / 12.0);
        let base_payment = annual_payment / periods_per_year as f64;

        let esc_rate: f64 = lease.escalation_rate
            .as_ref()
            .and_then(|r| r.parse().ok())
            .unwrap_or(0.0);
        let esc_freq_periods: i32 = lease.escalation_frequency_months
            .map(|m| {
                let years = m as f64 / 12.0;
                (years * periods_per_year as f64).round() as i32
            })
            .unwrap_or(total_periods + 1); // No escalation if not set

        let mut remaining_liability: f64 = lease.initial_lease_liability.parse().unwrap_or(0.0);
        let rou_value: f64 = lease.initial_rou_asset_value.parse().unwrap_or(0.0);
        let rou_per_period = if total_periods > 0 { rou_value / total_periods as f64 } else { 0.0 };
        let mut accumulated_dep = 0.0;

        // Straight-line lease expense (for operating leases)
        let total_lease_payments_val: f64 = lease.total_lease_payments.parse().unwrap_or(0.0);
        let straight_line_expense = if total_periods > 0 {
            total_lease_payments_val / total_periods as f64
        } else {
            0.0
        };

        let mut payments = Vec::new();

        for period in 1..=total_periods {
            // Apply escalation
            let escalation_multiplier = if esc_freq_periods > 0 && period > 1 && (period - 1) % esc_freq_periods == 0 {
                1.0 + esc_rate
            } else {
                1.0
            };

            let current_payment = base_payment * escalation_multiplier;

            // Interest on remaining liability
            let interest = remaining_liability * period_rate;
            let principal = (current_payment - interest).min(remaining_liability);

            remaining_liability = (remaining_liability - principal).max(0.0);

            // ROU depreciation
            let depreciation = rou_per_period.min(rou_value - accumulated_dep).max(0.0);
            accumulated_dep += depreciation;

            // Calculate payment date
            let payment_date = self.calculate_payment_date(
                lease.commencement_date, period, lease.payment_frequency.as_str(),
            );

            let payment = self.repository.create_payment(
                lease.organization_id, lease.id, period, payment_date,
                &format!("{:.2}", current_payment),
                &format!("{:.2}", interest),
                &format!("{:.2}", principal),
                &format!("{:.2}", remaining_liability),
                &format!("{:.2}", rou_value),
                &format!("{:.2}", depreciation),
                &format!("{:.2}", accumulated_dep),
                &format!("{:.2}", straight_line_expense),
                false, None, None, "scheduled",
            ).await?;

            payments.push(payment);
        }

        info!("Generated {} payment schedule entries for lease {}", payments.len(), lease.lease_number);
        Ok(payments)
    }

    /// Calculate payment date for a given period
    fn calculate_payment_date(
        &self,
        commencement_date: chrono::NaiveDate,
        period_number: i32,
        frequency: &str,
    ) -> chrono::NaiveDate {
        let months_to_add = match frequency {
            "monthly" => period_number,
            "quarterly" => period_number * 3,
            "annually" => period_number * 12,
            _ => period_number,
        };

        let year = commencement_date.year();
        let month = commencement_date.month() as i32;
        let day = commencement_date.day();

        let total_months = (year * 12 + month - 1) + months_to_add;
        let new_year = total_months / 12;
        let new_month = total_months % 12 + 1;

        chrono::NaiveDate::from_ymd_opt(new_year, new_month as u32, day)
            .unwrap_or_else(|| chrono::NaiveDate::from_ymd_opt(new_year, new_month as u32, 28).unwrap())
    }

    // ========================================================================
    // Payment Processing
    // ========================================================================

    /// Process a lease payment for a specific period
    pub async fn process_payment(
        &self,
        lease_id: Uuid,
        period_number: i32,
        payment_reference: Option<&str>,
    ) -> AtlasResult<LeasePayment> {
        let lease = self.repository.get_lease(lease_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Lease {} not found", lease_id)
            ))?;

        if lease.status != "active" && lease.status != "modified" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot process payment for lease in '{}' status.", lease.status)
            ));
        }

        let payment = self.repository.get_payment_by_period(lease_id, period_number).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Payment for period {} not found", period_number)
            ))?;

        if payment.status != "scheduled" {
            return Err(AtlasError::WorkflowError(
                format!("Payment for period {} is '{}' (expected 'scheduled')", period_number, payment.status)
            ));
        }

        // Mark payment as paid
        let payment = self.repository.update_payment_status(
            payment.id, "paid", true, payment_reference, None,
        ).await?;

        // Update lease balances
        let principal: f64 = payment.principal_amount.parse().unwrap_or(0.0);
        let depreciation: f64 = payment.rou_depreciation.parse().unwrap_or(0.0);

        let current_liability: f64 = lease.current_lease_liability.parse().unwrap_or(0.0);
        let current_rou: f64 = lease.current_rou_asset_value.parse().unwrap_or(0.0);
        let current_accum_dep: f64 = lease.accumulated_rou_depreciation.parse().unwrap_or(0.0);
        let current_payments: f64 = lease.total_payments_made.parse().unwrap_or(0.0);

        let payment_amount: f64 = payment.payment_amount.parse().unwrap_or(0.0);

        self.repository.update_lease_balances(
            lease_id,
            &format!("{:.2}", current_liability - principal),
            &format!("{:.2}", current_rou),
            &format!("{:.2}", current_accum_dep + depreciation),
            &format!("{:.2}", current_payments + payment_amount),
            lease.periods_elapsed + 1,
        ).await?;

        info!("Processed payment period {} for lease {}", period_number, lease.lease_number);
        Ok(payment)
    }

    /// Get payment schedule for a lease
    pub async fn list_payments(&self, lease_id: Uuid) -> AtlasResult<Vec<LeasePayment>> {
        self.repository.list_payments(lease_id).await
    }

    // ========================================================================
    // Lease Modifications
    // ========================================================================

    /// Create a lease modification
    pub async fn create_modification(
        &self,
        org_id: Uuid,
        lease_id: Uuid,
        modification_type: &str,
        description: Option<&str>,
        effective_date: chrono::NaiveDate,
        new_term_months: Option<i32>,
        new_end_date: Option<chrono::NaiveDate>,
        new_discount_rate: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<LeaseModification> {
        let lease = self.repository.get_lease(lease_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Lease {} not found", lease_id)
            ))?;

        if lease.status != "active" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot modify lease in '{}' status. Must be 'active'.", lease.status)
            ));
        }

        if !VALID_MODIFICATION_TYPES.contains(&modification_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid modification type '{}'. Must be one of: {}",
                modification_type, VALID_MODIFICATION_TYPES.join(", ")
            )));
        }

        // Calculate liability adjustment (simplified - recalculate with new terms)
        let liability_adjustment = "0.00".to_string();
        let rou_adjustment = "0.00".to_string();

        // Get next modification number
        let mod_number = self.repository.get_next_modification_number(lease_id).await?;

        info!("Creating modification {} for lease {}", mod_number, lease.lease_number);

        let modification = self.repository.create_modification(
            org_id, lease_id, mod_number, modification_type, description,
            effective_date,
            Some(lease.lease_term_months), new_term_months,
            Some(lease.end_date), new_end_date,
            Some(lease.discount_rate.as_str()), new_discount_rate,
            &liability_adjustment, &rou_adjustment,
            "pending", created_by,
        ).await?;

        // Update lease status to modified
        self.repository.update_lease_status(lease_id, "modified", None, None).await?;

        Ok(modification)
    }

    /// List modifications for a lease
    pub async fn list_modifications(&self, lease_id: Uuid) -> AtlasResult<Vec<LeaseModification>> {
        self.repository.list_modifications(lease_id).await
    }

    // ========================================================================
    // Lease Impairment
    // ========================================================================

    /// Record an impairment on the ROU asset
    pub async fn record_impairment(
        &self,
        lease_id: Uuid,
        impairment_amount: &str,
        impairment_date: chrono::NaiveDate,
    ) -> AtlasResult<LeaseContract> {
        let lease = self.repository.get_lease(lease_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Lease {} not found", lease_id)
            ))?;

        if lease.status != "active" && lease.status != "modified" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot impair lease in '{}' status.", lease.status)
            ));
        }

        let impairment: f64 = impairment_amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "Impairment amount must be a valid number".to_string(),
        ))?;
        if impairment <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Impairment amount must be positive".to_string(),
            ));
        }

        let current_rou: f64 = lease.current_rou_asset_value.parse().unwrap_or(0.0);
        let accum_dep: f64 = lease.accumulated_rou_depreciation.parse().unwrap_or(0.0);
        let net_rou = current_rou - accum_dep;

        if impairment > net_rou {
            return Err(AtlasError::ValidationFailed(
                format!("Impairment ({:.2}) exceeds net ROU asset value ({:.2})", impairment, net_rou)
            ));
        }

        info!("Recording impairment of {:.2} on lease {}", impairment, lease.lease_number);

        self.repository.update_lease_status(
            lease_id, "impaired",
            Some(&format!("{:.2}", impairment)),
            Some(impairment_date),
        ).await
    }

    // ========================================================================
    // Lease Termination
    // ========================================================================

    /// Terminate a lease
    pub async fn terminate_lease(
        &self,
        org_id: Uuid,
        lease_id: Uuid,
        termination_type: &str,
        termination_date: chrono::NaiveDate,
        termination_penalty: &str,
        reason: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<LeaseTermination> {
        let lease = self.repository.get_lease(lease_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Lease {} not found", lease_id)
            ))?;

        if lease.status != "active" && lease.status != "modified" && lease.status != "impaired" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot terminate lease in '{}' status.", lease.status)
            ));
        }

        if !VALID_TERMINATION_TYPES.contains(&termination_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid termination type '{}'. Must be one of: {}",
                termination_type, VALID_TERMINATION_TYPES.join(", ")
            )));
        }

        let remaining_liability: f64 = lease.current_lease_liability.parse().unwrap_or(0.0);
        let current_rou: f64 = lease.current_rou_asset_value.parse().unwrap_or(0.0);
        let accum_dep: f64 = lease.accumulated_rou_depreciation.parse().unwrap_or(0.0);
        let remaining_rou = current_rou - accum_dep;
        let penalty: f64 = termination_penalty.parse().unwrap_or(0.0);

        // Gain/loss = remaining liability - remaining ROU - penalty
        let gain_loss = remaining_liability - remaining_rou - penalty;
        let gain_loss_type = if gain_loss > 0.0 { "gain" } else if gain_loss < 0.0 { "loss" } else { "none" };

        let termination = self.repository.create_termination(
            org_id, lease_id, termination_type, termination_date, reason,
            &format!("{:.2}", remaining_liability),
            &format!("{:.2}", remaining_rou),
            &format!("{:.2}", penalty),
            &format!("{:.2}", gain_loss.abs()),
            if gain_loss_type == "none" { None } else { Some(gain_loss_type) },
            None, "pending", created_by,
        ).await?;

        // Update lease status to terminated
        self.repository.update_lease_status(lease_id, "terminated", None, None).await?;

        info!("Terminated lease {} ({}): gain/loss={:.2}", lease.lease_number, termination_type, gain_loss);
        Ok(termination)
    }

    /// List terminations for a lease
    pub async fn list_terminations(&self, lease_id: Uuid) -> AtlasResult<Vec<LeaseTermination>> {
        self.repository.list_terminations(lease_id).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get lease accounting dashboard summary
    pub async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<LeaseDashboardSummary> {
        self.repository.get_dashboard_summary(org_id).await
    }

    // ========================================================================
    // Financial Calculations
    // ========================================================================

    /// Calculate present value of an annuity
    /// PV = PMT × [1 - (1 + r)^(-n)] / r
    pub fn calculate_present_value(&self, payment: f64, periods: i32, rate_per_period: f64) -> f64 {
        if rate_per_period <= 0.0 {
            return payment * periods as f64;
        }
        if periods <= 0 {
            return 0.0;
        }
        payment * (1.0 - (1.0 + rate_per_period).powi(-periods)) / rate_per_period
    }

    /// Calculate total lease payments with optional escalation
    fn calculate_total_lease_payments(
        &self,
        base_payment: f64,
        total_periods: i32,
        escalation_rate: f64,
        escalation_freq_periods: Option<i32>,
    ) -> f64 {
        let mut total = 0.0;
        let mut current_payment = base_payment;
        let esc_freq = escalation_freq_periods.unwrap_or(total_periods + 1);

        for period in 1..=total_periods {
            if esc_freq > 0 && period > 1 && (period - 1) % esc_freq == 0 {
                current_payment *= 1.0 + escalation_rate;
            }
            total += current_payment;
        }
        total
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_classifications() {
        assert!(VALID_CLASSIFICATIONS.contains(&"operating"));
        assert!(VALID_CLASSIFICATIONS.contains(&"finance"));
    }

    #[test]
    fn test_valid_statuses() {
        assert!(VALID_STATUSES.contains(&"draft"));
        assert!(VALID_STATUSES.contains(&"active"));
        assert!(VALID_STATUSES.contains(&"terminated"));
    }

    #[test]
    fn test_valid_payment_frequencies() {
        assert!(VALID_FREQUENCIES.contains(&"monthly"));
        assert!(VALID_FREQUENCIES.contains(&"quarterly"));
        assert!(VALID_FREQUENCIES.contains(&"annually"));
    }

    #[test]
    fn test_present_value_calculation() {
        let engine = LeaseAccountingEngine::new(Arc::new(crate::MockLeaseAccountingRepository));

        // Monthly payment of 1000 for 36 months at 5% annual (0.4167% monthly)
        let pv = engine.calculate_present_value(1000.0, 36, 0.05 / 12.0);
        // Expected: ~33,378.34
        assert!((pv - 33378.34).abs() < 100.0, "PV was {:.2}", pv);
    }

    #[test]
    fn test_present_value_zero_rate() {
        let engine = LeaseAccountingEngine::new(Arc::new(crate::MockLeaseAccountingRepository));
        let pv = engine.calculate_present_value(1000.0, 12, 0.0);
        assert_eq!(pv, 12000.0);
    }

    #[test]
    fn test_present_value_zero_periods() {
        let engine = LeaseAccountingEngine::new(Arc::new(crate::MockLeaseAccountingRepository));
        let pv = engine.calculate_present_value(1000.0, 0, 0.05);
        assert_eq!(pv, 0.0);
    }

    #[test]
    fn test_payment_date_monthly() {
        let engine = LeaseAccountingEngine::new(Arc::new(crate::MockLeaseAccountingRepository));
        let date = chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let p1 = engine.calculate_payment_date(date, 1, "monthly");
        assert_eq!(p1, chrono::NaiveDate::from_ymd_opt(2024, 2, 1).unwrap());
        let p12 = engine.calculate_payment_date(date, 12, "monthly");
        assert_eq!(p12, chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap());
    }

    #[test]
    fn test_payment_date_quarterly() {
        let engine = LeaseAccountingEngine::new(Arc::new(crate::MockLeaseAccountingRepository));
        let date = chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let p1 = engine.calculate_payment_date(date, 1, "quarterly");
        assert_eq!(p1, chrono::NaiveDate::from_ymd_opt(2024, 4, 1).unwrap());
    }

    #[test]
    fn test_total_lease_payments_no_escalation() {
        let engine = LeaseAccountingEngine::new(Arc::new(crate::MockLeaseAccountingRepository));
        let total = engine.calculate_total_lease_payments(1000.0, 12, 0.0, None);
        assert_eq!(total, 12000.0);
    }

    #[test]
    fn test_total_lease_payments_with_escalation() {
        let engine = LeaseAccountingEngine::new(Arc::new(crate::MockLeaseAccountingRepository));
        // 3% escalation every 12 periods, 24 total periods
        let total = engine.calculate_total_lease_payments(1000.0, 24, 0.03, Some(12));
        // First 12: 1000 each = 12000
        // Last 12: 1030 each = 12360
        let expected = 12000.0 + 12360.0;
        assert!((total - expected).abs() < 0.01, "Expected {}, got {}", expected, total);
    }
}
