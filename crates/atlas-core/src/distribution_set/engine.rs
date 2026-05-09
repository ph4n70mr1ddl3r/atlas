//! Distribution Set Engine
//!
//! Manages the full lifecycle of distribution sets in Accounts Payable:
//! - Create distribution sets with percentage or amount-based distributions
//! - Add/remove distribution lines with GL account combinations
//! - Validate that percentages sum to 100% and amounts balance
//! - Apply distribution sets to invoices
//! - Track usage and audit trail
//! - Dashboard and reporting
//!
//! Oracle Fusion Cloud ERP equivalent: Financials > Payables > Setup > Distribution Sets

use super::*;
use atlas_shared::AtlasError;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

const VALID_DISTRIBUTION_TYPES: &[&str] = &["percentage", "amount"];
const VALID_STATUSES: &[&str] = &["active", "inactive"];
pub struct DistributionSetEngine {
    repository: Arc<dyn DistributionSetRepository>,
}

impl DistributionSetEngine {
    pub fn new(repository: Arc<dyn DistributionSetRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Distribution Set CRUD
    // ========================================================================

    /// Create a new distribution set
    pub async fn create_set(
        &self,
        org_id: Uuid,
        set_code: &str,
        set_name: &str,
        description: Option<&str>,
        distribution_type: &str,
        currency_code: &str,
        is_default: bool,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<DistributionSet> {
        if set_code.is_empty() {
            return Err(AtlasError::ValidationFailed("Set code is required".into()));
        }
        if set_name.is_empty() {
            return Err(AtlasError::ValidationFailed("Set name is required".into()));
        }
        if !VALID_DISTRIBUTION_TYPES.contains(&distribution_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid distribution_type '{}'. Must be one of: {}",
                distribution_type, VALID_DISTRIBUTION_TYPES.join(", ")
            )));
        }
        if currency_code.is_empty() {
            return Err(AtlasError::ValidationFailed("Currency code is required".into()));
        }

        // Validate effective dates
        if let (Some(from), Some(to)) = (effective_from, effective_to) {
            if from > to {
                return Err(AtlasError::ValidationFailed(
                    "effective_from must be before effective_to".into(),
                ));
            }
        }

        // Check for duplicate code
        if let Some(existing) = self.repository.get_set_by_code(org_id, set_code).await? {
            return Err(AtlasError::Conflict(format!(
                "Distribution set code '{}' already exists", set_code
            )));
        }

        info!("Distribution Set Engine: Creating set '{}' ({})", set_code, set_name);
        self.repository.create_set(
            org_id, set_code, set_name, description,
            distribution_type, currency_code, is_default,
            effective_from, effective_to, created_by,
        ).await
    }

    /// Get a distribution set by ID
    pub async fn get_set(&self, id: Uuid) -> AtlasResult<Option<DistributionSet>> {
        self.repository.get_set(id).await
    }

    /// List distribution sets with optional filters
    pub async fn list_sets(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        distribution_type: Option<&str>,
    ) -> AtlasResult<Vec<DistributionSet>> {
        self.repository.list_sets(org_id, status, distribution_type).await
    }

    /// Activate a distribution set
    pub async fn activate_set(&self, id: Uuid) -> AtlasResult<DistributionSet> {
        let set = self.repository.get_set(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Distribution set not found".into()))?;

        if set.status == "active" {
            return Err(AtlasError::ValidationFailed("Set is already active".into()));
        }

        // Validate that active sets have lines
        let lines = self.repository.list_lines(id).await?;
        if lines.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Cannot activate a distribution set with no lines".into(),
            ));
        }

        // For percentage type, validate total is 100%
        if set.distribution_type == "percentage" {
            let total_pct: f64 = lines.iter()
                .map(|l| l.percentage.parse::<f64>().unwrap_or(0.0))
                .sum();
            if (total_pct - 100.0).abs() > 0.01 {
                return Err(AtlasError::ValidationFailed(format!(
                    "Percentage-type set must have lines summing to 100%. Current total: {:.2}%",
                    total_pct
                )));
            }
        }

        info!("Distribution Set Engine: Activating set {}", id);
        self.repository.update_set_status(id, "active").await
    }

    /// Deactivate a distribution set
    pub async fn deactivate_set(&self, id: Uuid) -> AtlasResult<DistributionSet> {
        let set = self.repository.get_set(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Distribution set not found".into()))?;

        if set.status == "inactive" {
            return Err(AtlasError::ValidationFailed("Set is already inactive".into()));
        }

        info!("Distribution Set Engine: Deactivating set {}", id);
        self.repository.update_set_status(id, "inactive").await
    }

    /// Delete a distribution set (only allowed for inactive sets)
    pub async fn delete_set(&self, id: Uuid) -> AtlasResult<()> {
        let set = self.repository.get_set(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Distribution set not found".into()))?;

        if set.status == "active" {
            return Err(AtlasError::ValidationFailed(
                "Cannot delete an active distribution set. Deactivate it first.".into(),
            ));
        }

        info!("Distribution Set Engine: Deleting set {}", id);
        self.repository.delete_set(id).await
    }

    // ========================================================================
    // Distribution Set Lines
    // ========================================================================

    /// Add a line to a distribution set
    pub async fn add_line(
        &self,
        org_id: Uuid,
        distribution_set_id: Uuid,
        account_combination: &str,
        account_description: Option<&str>,
        segment1: Option<&str>,
        segment2: Option<&str>,
        segment3: Option<&str>,
        segment4: Option<&str>,
        segment5: Option<&str>,
        percentage: &str,
        amount: Option<&str>,
        description: Option<&str>,
        cost_center: Option<&str>,
        department: Option<&str>,
        project_code: Option<&str>,
    ) -> AtlasResult<DistributionSetLine> {
        let set = self.repository.get_set(distribution_set_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Distribution set not found".into()))?;

        if set.status != "active" && set.status != "inactive" {
            return Err(AtlasError::ValidationFailed(
                "Cannot add lines to a set in its current status".into(),
            ));
        }

        if account_combination.is_empty() {
            return Err(AtlasError::ValidationFailed("Account combination is required".into()));
        }

        // Validate percentage
        let pct: f64 = percentage.parse().map_err(|_| AtlasError::ValidationFailed(
            "Percentage must be a valid number".into(),
        ))?;
        if pct < 0.0 || pct > 100.0 {
            return Err(AtlasError::ValidationFailed(
                "Percentage must be between 0 and 100".into(),
            ));
        }

        // Get existing lines to determine next line number and validate totals
        let existing_lines = self.repository.list_lines(distribution_set_id).await?;
        let line_number = (existing_lines.len() as i32) + 1;

        // Validate total won't exceed 100%
        if set.distribution_type == "percentage" {
            let current_total: f64 = existing_lines.iter()
                .map(|l| l.percentage.parse::<f64>().unwrap_or(0.0))
                .sum();
            let new_total = current_total + pct;
            if new_total > 100.01 {
                return Err(AtlasError::ValidationFailed(format!(
                    "Adding {:.2}% would exceed 100%. Current total: {:.2}%",
                    pct, current_total
                )));
            }
        }

        // Validate amount
        if let Some(amt_str) = amount {
            let amt: f64 = amt_str.parse().map_err(|_| AtlasError::ValidationFailed(
                "Amount must be a valid number".into(),
            ))?;
            if amt < 0.0 {
                return Err(AtlasError::ValidationFailed(
                    "Amount cannot be negative".into(),
                ));
            }
        }

        let line = self.repository.add_line(
            org_id, distribution_set_id, line_number,
            account_combination, account_description,
            segment1, segment2, segment3, segment4, segment5,
            percentage, amount, description,
            cost_center, department, project_code,
        ).await?;

        // Recalculate total percentage on the set
        let all_lines = self.repository.list_lines(distribution_set_id).await?;
        let total_pct: f64 = all_lines.iter()
            .map(|l| l.percentage.parse::<f64>().unwrap_or(0.0))
            .sum();
        self.repository.update_set_total_percentage(
            distribution_set_id,
            &format!("{:.4}", total_pct),
        ).await?;

        Ok(line)
    }

    /// List lines for a distribution set
    pub async fn list_lines(&self, distribution_set_id: Uuid) -> AtlasResult<Vec<DistributionSetLine>> {
        self.repository.list_lines(distribution_set_id).await
    }

    /// Remove a line from a distribution set
    pub async fn remove_line(&self, line_id: Uuid) -> AtlasResult<()> {
        let line = self.repository.get_line(line_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Distribution set line not found".into()))?;

        self.repository.delete_line(line_id).await?;

        // Recalculate total percentage
        let all_lines = self.repository.list_lines(line.distribution_set_id).await?;
        let total_pct: f64 = all_lines.iter()
            .map(|l| l.percentage.parse::<f64>().unwrap_or(0.0))
            .sum();
        self.repository.update_set_total_percentage(
            line.distribution_set_id,
            &format!("{:.4}", total_pct),
        ).await?;

        Ok(())
    }

    // ========================================================================
    // Apply to Invoice
    // ========================================================================

    /// Apply a distribution set to an invoice, generating distribution lines
    pub async fn apply_to_invoice(
        &self,
        org_id: Uuid,
        distribution_set_id: Uuid,
        invoice_id: Uuid,
        invoice_number: Option<&str>,
        invoice_amount: &str,
        applied_by: Option<Uuid>,
    ) -> AtlasResult<Vec<DistributionSetLine>> {
        let set = self.repository.get_set(distribution_set_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Distribution set not found".into()))?;

        if set.status != "active" {
            return Err(AtlasError::ValidationFailed(
                "Can only apply active distribution sets".into(),
            ));
        }

        let total: f64 = invoice_amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "Invoice amount must be a valid number".into(),
        ))?;
        if total <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Invoice amount must be positive".into(),
            ));
        }

        let lines = self.repository.list_lines(distribution_set_id).await?;
        if lines.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Distribution set has no lines".into(),
            ));
        }

        // Log the usage
        self.repository.log_usage(
            org_id, distribution_set_id, &set.set_code,
            "ap_invoice", invoice_id, invoice_number,
            applied_by, lines.len() as i32, Some(invoice_amount),
        ).await?;

        // Update usage count
        self.repository.update_set_usage(distribution_set_id).await?;

        info!(
            "Distribution Set Engine: Applied set '{}' to invoice {} ({} lines)",
            set.set_code,
            invoice_number.unwrap_or("N/A"),
            lines.len()
        );

        Ok(lines)
    }

    // ========================================================================
    // Usage History
    // ========================================================================

    /// List usage history
    pub async fn list_usage(
        &self,
        org_id: Uuid,
        distribution_set_id: Option<Uuid>,
    ) -> AtlasResult<Vec<DistributionSetUsage>> {
        self.repository.list_usage(org_id, distribution_set_id).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get distribution set dashboard summary
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<DistributionSetDashboard> {
        self.repository.get_dashboard(org_id).await
    }

    // ========================================================================
    // Calculation Helpers
    // ========================================================================

    /// Calculate distributed amounts from percentage lines
    pub fn calculate_distribution_amounts(
        total_amount: f64,
        lines: &[(Uuid, f64)], // (line_id, percentage)
    ) -> Vec<(Uuid, f64)> {
        let mut results: Vec<(Uuid, f64)> = lines.iter()
            .map(|(id, pct)| (*id, total_amount * (pct / 100.0)))
            .collect();

        // Ensure rounding doesn't lose pennies
        let sum: f64 = results.iter().map(|(_, amt)| *amt).sum();
        let diff = total_amount - sum;
        if !results.is_empty() && diff.abs() > 0.0 {
            results[0].1 += diff;
        }

        results
    }
}
