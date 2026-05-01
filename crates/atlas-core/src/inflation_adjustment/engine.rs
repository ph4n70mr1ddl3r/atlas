//! Inflation Adjustment Engine
//!
//! Core inflation adjustment operations:
//! - Inflation index management (CPI, GDP deflator, custom)
//! - Index rate maintenance (periodic rates and cumulative factors)
//! - Adjustment run creation and execution
//! - Account restatement calculations
//! - Workflow: draft -> submitted -> approved -> completed
//!
//! Oracle Fusion Cloud ERP equivalent: Financials > General Ledger > Inflation Adjustment

use atlas_shared::{
    InflationIndex, InflationIndexRate, InflationAdjustmentRun, InflationAdjustmentLine,
    InflationDashboardSummary,
    AtlasError, AtlasResult,
};
use super::InflationAdjustmentRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Valid index types
#[allow(dead_code)]
const VALID_INDEX_TYPES: &[&str] = &["cpi", "gdp_deflator", "custom"];

/// Valid adjustment methods
#[allow(dead_code)]
const VALID_ADJUSTMENT_METHODS: &[&str] = &["historical", "current"];

/// Valid index statuses
#[allow(dead_code)]
const VALID_INDEX_STATUSES: &[&str] = &["active", "inactive"];

/// Valid run statuses
#[allow(dead_code)]
const VALID_RUN_STATUSES: &[&str] = &[
    "draft", "submitted", "approved", "completed", "reversed",
];

/// Valid account types for inflation
#[allow(dead_code)]
const VALID_ACCOUNT_TYPES: &[&str] = &["monetary", "non_monetary"];

/// Valid balance types
#[allow(dead_code)]
const VALID_BALANCE_TYPES: &[&str] = &["debit", "credit"];

/// Inflation Adjustment Engine
pub struct InflationAdjustmentEngine {
    repository: Arc<dyn InflationAdjustmentRepository>,
}

impl InflationAdjustmentEngine {
    pub fn new(repository: Arc<dyn InflationAdjustmentRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Inflation Index Management
    // ========================================================================

    /// Create a new inflation index
    pub async fn create_index(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        country_code: &str,
        currency_code: &str,
        index_type: &str,
        is_hyperinflationary: bool,
        hyperinflationary_start_date: Option<chrono::NaiveDate>,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<InflationIndex> {
        if code.is_empty() || name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Index code and name are required".to_string(),
            ));
        }
        if country_code.len() != 3 {
            return Err(AtlasError::ValidationFailed(
                "Country code must be 3 characters (ISO 3166-1 alpha-3)".to_string(),
            ));
        }
        if !VALID_INDEX_TYPES.contains(&index_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid index_type '{}'. Must be one of: {}", index_type, VALID_INDEX_TYPES.join(", ")
            )));
        }

        // Check uniqueness
        if self.repository.get_index_by_code(org_id, code).await?.is_some() {
            return Err(AtlasError::Conflict(
                format!("Index code '{}' already exists", code)
            ));
        }

        info!("Creating inflation index '{}' for country {}", code, country_code);

        self.repository.create_index(
            org_id, code, name, description, country_code, currency_code,
            index_type, is_hyperinflationary, hyperinflationary_start_date,
            effective_from, effective_to, created_by,
        ).await
    }

    /// Get index by ID
    pub async fn get_index(&self, id: Uuid) -> AtlasResult<Option<InflationIndex>> {
        self.repository.get_index(id).await
    }

    /// Get index by code
    pub async fn get_index_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<InflationIndex>> {
        self.repository.get_index_by_code(org_id, code).await
    }

    /// List indices
    pub async fn list_indices(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<InflationIndex>> {
        if let Some(s) = status {
            if !VALID_INDEX_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_INDEX_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_indices(org_id, status).await
    }

    // ========================================================================
    // Index Rate Management
    // ========================================================================

    /// Add an index rate
    pub async fn add_index_rate(
        &self,
        org_id: Uuid,
        index_id: Uuid,
        period_start: chrono::NaiveDate,
        period_end: chrono::NaiveDate,
        index_value: &str,
        cumulative_factor: &str,
        period_factor: &str,
        source: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<InflationIndexRate> {
        let _index = self.repository.get_index(index_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Index {} not found", index_id)))?;

        let value: f64 = index_value.parse().map_err(|_| AtlasError::ValidationFailed(
            "Index value must be a valid number".to_string(),
        ))?;
        if value < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Index value must be non-negative".to_string(),
            ));
        }

        let cum_factor: f64 = cumulative_factor.parse().map_err(|_| AtlasError::ValidationFailed(
            "Cumulative factor must be a valid number".to_string(),
        ))?;
        if cum_factor <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Cumulative factor must be positive".to_string(),
            ));
        }

        let per_factor: f64 = period_factor.parse().map_err(|_| AtlasError::ValidationFailed(
            "Period factor must be a valid number".to_string(),
        ))?;
        if per_factor <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Period factor must be positive".to_string(),
            ));
        }

        if period_start >= period_end {
            return Err(AtlasError::ValidationFailed(
                "Period start must be before period end".to_string(),
            ));
        }

        info!("Adding index rate for period {} to {}", period_start, period_end);

        self.repository.create_index_rate(
            org_id, index_id, period_start, period_end,
            index_value, cumulative_factor, period_factor,
            source, created_by,
        ).await
    }

    /// List index rates
    pub async fn list_index_rates(&self, index_id: Uuid) -> AtlasResult<Vec<InflationIndexRate>> {
        self.repository.list_index_rates(index_id).await
    }

    // ========================================================================
    // Adjustment Run Management
    // ========================================================================

    /// Create an inflation adjustment run
    pub async fn create_run(
        &self,
        org_id: Uuid,
        name: Option<&str>,
        description: Option<&str>,
        index_id: Uuid,
        ledger_id: Option<Uuid>,
        from_period: chrono::NaiveDate,
        to_period: chrono::NaiveDate,
        adjustment_method: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<InflationAdjustmentRun> {
        let _index = self.repository.get_index(index_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Index {} not found", index_id)))?;

        if !VALID_ADJUSTMENT_METHODS.contains(&adjustment_method) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid adjustment_method '{}'. Must be one of: {}", adjustment_method, VALID_ADJUSTMENT_METHODS.join(", ")
            )));
        }

        if from_period >= to_period {
            return Err(AtlasError::ValidationFailed(
                "From period must be before to period".to_string(),
            ));
        }

        let run_number = format!("INF-{}", Uuid::new_v4().to_string()[..8].to_uppercase());

        info!("Creating inflation adjustment run {}", run_number);

        self.repository.create_run(
            org_id, &run_number, name, description, index_id,
            ledger_id, from_period, to_period, adjustment_method, created_by,
        ).await
    }

    /// Get run by ID
    pub async fn get_run(&self, id: Uuid) -> AtlasResult<Option<InflationAdjustmentRun>> {
        self.repository.get_run(id).await
    }

    /// List runs
    pub async fn list_runs(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<InflationAdjustmentRun>> {
        if let Some(s) = status {
            if !VALID_RUN_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_RUN_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_runs(org_id, status).await
    }

    /// Add an adjustment line to a run
    pub async fn add_adjustment_line(
        &self,
        org_id: Uuid,
        run_id: Uuid,
        account_code: &str,
        account_name: Option<&str>,
        account_type: &str,
        balance_type: &str,
        original_balance: &str,
        inflation_factor: &str,
        acquisition_date: Option<chrono::NaiveDate>,
        gain_loss_account: Option<&str>,
        currency_code: Option<&str>,
    ) -> AtlasResult<InflationAdjustmentLine> {
        let run = self.repository.get_run(run_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Run {} not found", run_id)))?;

        if run.status != "draft" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot add lines to run in '{}' status. Must be 'draft'.", run.status)
            ));
        }

        if !VALID_ACCOUNT_TYPES.contains(&account_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid account_type '{}'. Must be one of: {}", account_type, VALID_ACCOUNT_TYPES.join(", ")
            )));
        }

        if !VALID_BALANCE_TYPES.contains(&balance_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid balance_type '{}'. Must be one of: {}", balance_type, VALID_BALANCE_TYPES.join(", ")
            )));
        }

        let original: f64 = original_balance.parse().map_err(|_| AtlasError::ValidationFailed(
            "Original balance must be a valid number".to_string(),
        ))?;
        let factor: f64 = inflation_factor.parse().map_err(|_| AtlasError::ValidationFailed(
            "Inflation factor must be a valid number".to_string(),
        ))?;

        if factor <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Inflation factor must be positive".to_string(),
            ));
        }

        let restated = original * factor;
        let adjustment = restated - original;
        // For monetary accounts, gain/loss is the adjustment; for non-monetary it's revalued
        let gain_loss = if account_type == "monetary" { adjustment } else { 0.0 };

        let lines = self.repository.list_run_lines(run_id).await?;
        let line_number = (lines.len() as i32) + 1;

        info!("Adding adjustment line for account {} (factor: {}, adj: {:.2})",
            account_code, factor, adjustment);

        self.repository.create_adjustment_line(
            org_id, run_id, line_number, account_code, account_name,
            account_type, balance_type, original_balance,
            &format!("{:.2}", restated), &format!("{:.2}", adjustment),
            inflation_factor, acquisition_date,
            &format!("{:.2}", gain_loss), gain_loss_account, currency_code,
        ).await
    }

    /// List run lines
    pub async fn list_run_lines(&self, run_id: Uuid) -> AtlasResult<Vec<InflationAdjustmentLine>> {
        self.repository.list_run_lines(run_id).await
    }

    /// Calculate restated amount
    pub fn calculate_restated_amount(original_balance: f64, inflation_factor: f64) -> f64 {
        original_balance * inflation_factor
    }

    /// Calculate monetary gain/loss (for IAS 29 monetary items)
    pub fn calculate_monetary_gain_loss(original_balance: f64, inflation_factor: f64) -> f64 {
        // Gain/loss = restated - original (monetary items get purchasing power gain/loss)
        (original_balance * inflation_factor) - original_balance
    }

    // ========================================================================
    // Run Workflow
    // ========================================================================

    /// Submit a draft run for approval
    pub async fn submit_run(&self, run_id: Uuid, submitted_by: Option<Uuid>) -> AtlasResult<InflationAdjustmentRun> {
        let run = self.repository.get_run(run_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Run {} not found", run_id)))?;

        if run.status != "draft" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot submit run in '{}' status. Must be 'draft'.", run.status)
            ));
        }

        let lines = self.repository.list_run_lines(run_id).await?;
        if lines.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Cannot submit a run with no adjustment lines".to_string(),
            ));
        }

        // Calculate totals
        let total_debit: f64 = lines.iter()
            .filter(|l| l.balance_type.as_deref() == Some("debit"))
            .map(|l| l.adjustment_amount.parse::<f64>().unwrap_or(0.0))
            .sum();
        let total_credit: f64 = lines.iter()
            .filter(|l| l.balance_type.as_deref() == Some("credit"))
            .map(|l| l.adjustment_amount.parse::<f64>().unwrap_or(0.0))
            .sum();
        let total_gain_loss: f64 = lines.iter()
            .map(|l| l.gain_loss_amount.parse::<f64>().unwrap_or(0.0))
            .sum();

        self.repository.update_run_totals(
            run_id,
            &format!("{:.2}", total_debit),
            &format!("{:.2}", total_credit),
            &format!("{:.2}", total_gain_loss),
            lines.len() as i32,
        ).await?;

        info!("Submitting inflation adjustment run {}", run.run_number);

        self.repository.update_run_status(run_id, "submitted", submitted_by, None).await
    }

    /// Approve a submitted run
    pub async fn approve_run(&self, run_id: Uuid, approved_by: Option<Uuid>) -> AtlasResult<InflationAdjustmentRun> {
        let run = self.repository.get_run(run_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Run {} not found", run_id)))?;

        if run.status != "submitted" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot approve run in '{}' status. Must be 'submitted'.", run.status)
            ));
        }

        info!("Approving inflation adjustment run {}", run.run_number);

        self.repository.update_run_status(run_id, "approved", None, approved_by).await
    }

    /// Complete an approved run
    pub async fn complete_run(&self, run_id: Uuid) -> AtlasResult<InflationAdjustmentRun> {
        let run = self.repository.get_run(run_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Run {} not found", run_id)))?;

        if run.status != "approved" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot complete run in '{}' status. Must be 'approved'.", run.status)
            ));
        }

        info!("Completing inflation adjustment run {}", run.run_number);

        self.repository.update_run_status(run_id, "completed", None, None).await
    }

    /// Get dashboard summary
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<InflationDashboardSummary> {
        self.repository.get_dashboard_summary(org_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_index_types() {
        assert!(VALID_INDEX_TYPES.contains(&"cpi"));
        assert!(VALID_INDEX_TYPES.contains(&"gdp_deflator"));
        assert!(VALID_INDEX_TYPES.contains(&"custom"));
        assert_eq!(VALID_INDEX_TYPES.len(), 3);
    }

    #[test]
    fn test_valid_adjustment_methods() {
        assert!(VALID_ADJUSTMENT_METHODS.contains(&"historical"));
        assert!(VALID_ADJUSTMENT_METHODS.contains(&"current"));
        assert_eq!(VALID_ADJUSTMENT_METHODS.len(), 2);
    }

    #[test]
    fn test_valid_index_statuses() {
        assert!(VALID_INDEX_STATUSES.contains(&"active"));
        assert!(VALID_INDEX_STATUSES.contains(&"inactive"));
        assert_eq!(VALID_INDEX_STATUSES.len(), 2);
    }

    #[test]
    fn test_valid_run_statuses() {
        assert!(VALID_RUN_STATUSES.contains(&"draft"));
        assert!(VALID_RUN_STATUSES.contains(&"submitted"));
        assert!(VALID_RUN_STATUSES.contains(&"approved"));
        assert!(VALID_RUN_STATUSES.contains(&"completed"));
        assert!(VALID_RUN_STATUSES.contains(&"reversed"));
        assert_eq!(VALID_RUN_STATUSES.len(), 5);
    }

    #[test]
    fn test_valid_account_types() {
        assert!(VALID_ACCOUNT_TYPES.contains(&"monetary"));
        assert!(VALID_ACCOUNT_TYPES.contains(&"non_monetary"));
        assert_eq!(VALID_ACCOUNT_TYPES.len(), 2);
    }

    #[test]
    fn test_valid_balance_types() {
        assert!(VALID_BALANCE_TYPES.contains(&"debit"));
        assert!(VALID_BALANCE_TYPES.contains(&"credit"));
        assert_eq!(VALID_BALANCE_TYPES.len(), 2);
    }

    #[test]
    fn test_calculate_restated_amount() {
        // 1000 * 1.5 = 1500
        let restated = InflationAdjustmentEngine::calculate_restated_amount(1000.0, 1.5);
        assert!((restated - 1500.0).abs() < 0.01);

        // Factor of 1.0 means no change
        let restated = InflationAdjustmentEngine::calculate_restated_amount(500.0, 1.0);
        assert!((restated - 500.0).abs() < 0.01);
    }

    #[test]
    fn test_calculate_monetary_gain_loss_positive() {
        // When inflation factor > 1, monetary assets gain purchasing power
        let gain_loss = InflationAdjustmentEngine::calculate_monetary_gain_loss(10000.0, 1.25);
        assert!((gain_loss - 2500.0).abs() < 0.01);
    }

    #[test]
    fn test_calculate_monetary_gain_loss_no_change() {
        // Factor of 1.0 means no gain/loss
        let gain_loss = InflationAdjustmentEngine::calculate_monetary_gain_loss(10000.0, 1.0);
        assert!(gain_loss.abs() < 0.01);
    }

    #[test]
    fn test_calculate_monetary_gain_loss_negative_factor() {
        // Factor < 1 means deflation
        let gain_loss = InflationAdjustmentEngine::calculate_monetary_gain_loss(10000.0, 0.9);
        assert!((gain_loss - (-1000.0)).abs() < 0.01);
    }

    #[test]
    fn test_calculate_restated_amount_large_factor() {
        // Hyperinflation: 100x factor
        let restated = InflationAdjustmentEngine::calculate_restated_amount(100.0, 100.0);
        assert!((restated - 10000.0).abs() < 0.01);
    }

    #[test]
    fn test_calculate_monetary_gain_loss_ias29_example() {
        // IAS 29 example: Balance 100,000, factor 2.5
        // Restated = 250,000, gain/loss = 150,000
        let gain_loss = InflationAdjustmentEngine::calculate_monetary_gain_loss(100000.0, 2.5);
        assert!((gain_loss - 150000.0).abs() < 0.01);
    }
}
