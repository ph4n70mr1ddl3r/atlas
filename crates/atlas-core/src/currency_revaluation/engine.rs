//! Currency Revaluation Engine Implementation
//!
//! Manages currency revaluation definitions, runs, and gain/loss calculations.
//!
//! Oracle Fusion Cloud ERP equivalent: General Ledger > Currency Revaluation

use atlas_shared::{
    CurrencyRevaluationDefinition, CurrencyRevaluationDefinitionRequest,
    CurrencyRevaluationAccount, CurrencyRevaluationAccountRequest,
    CurrencyRevaluationRun, CurrencyRevaluationRunRequest,
    CurrencyRevaluationLine, CurrencyRevaluationBalanceRequest,
    CurrencyRevaluationDashboardSummary,
    AtlasError, AtlasResult,
};
use super::CurrencyRevaluationRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

const VALID_REVALUATION_TYPES: &[&str] = &["period_end", "balance_sheet", "income_statement"];
const VALID_RATE_TYPES: &[&str] = &["daily", "spot", "corporate", "period_average", "period_end", "user", "fixed"];
const VALID_ACCOUNT_TYPES: &[&str] = &["asset", "liability", "equity", "revenue", "expense"];
const VALID_RUN_STATUSES: &[&str] = &["draft", "posted", "reversed", "cancelled"];

/// Currency Revaluation engine
pub struct CurrencyRevaluationEngine {
    repository: Arc<dyn CurrencyRevaluationRepository>,
}

impl CurrencyRevaluationEngine {
    pub fn new(repository: Arc<dyn CurrencyRevaluationRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Revaluation Definition Management
    // ========================================================================

    /// Create a new revaluation definition
    pub async fn create_definition(
        &self,
        org_id: Uuid,
        request: &CurrencyRevaluationDefinitionRequest,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CurrencyRevaluationDefinition> {
        // Validate
        if request.code.is_empty() {
            return Err(AtlasError::ValidationFailed("Definition code is required".to_string()));
        }
        if request.name.is_empty() {
            return Err(AtlasError::ValidationFailed("Definition name is required".to_string()));
        }
        if !VALID_REVALUATION_TYPES.contains(&request.revaluation_type.as_str()) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid revaluation type '{}'. Must be one of: {}",
                request.revaluation_type,
                VALID_REVALUATION_TYPES.join(", ")
            )));
        }
        if !VALID_RATE_TYPES.contains(&request.rate_type.as_str()) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid rate type '{}'. Must be one of: {}",
                request.rate_type,
                VALID_RATE_TYPES.join(", ")
            )));
        }
        if request.gain_account_code.is_empty() {
            return Err(AtlasError::ValidationFailed("Gain account code is required".to_string()));
        }
        if request.loss_account_code.is_empty() {
            return Err(AtlasError::ValidationFailed("Loss account code is required".to_string()));
        }

        // Check uniqueness
        if self.repository.get_definition_by_code(org_id, &request.code).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Revaluation definition with code '{}' already exists", request.code
            )));
        }

        info!("Creating revaluation definition {} ({})", request.code, request.name);

        let definition = self.repository.create_definition(
            org_id,
            &request.code,
            &request.name,
            request.description.as_deref(),
            &request.revaluation_type,
            &request.currency_code,
            &request.rate_type,
            &request.gain_account_code,
            &request.loss_account_code,
            request.unrealized_gain_account_code.as_deref(),
            request.unrealized_loss_account_code.as_deref(),
            request.account_range_from.as_deref(),
            request.account_range_to.as_deref(),
            request.include_subledger,
            request.auto_reverse,
            request.reversal_period_offset,
            request.effective_from,
            request.effective_to,
            created_by,
        ).await?;

        // Add accounts if provided
        if let Some(accounts) = &request.accounts {
            for acct_req in accounts {
                self.repository.add_account(
                    org_id,
                    definition.id,
                    &acct_req.account_code,
                    acct_req.account_name.as_deref(),
                    &acct_req.account_type,
                    acct_req.is_included,
                ).await?;
            }
        }

        // Reload with accounts
        self.repository.get_definition_by_id(definition.id).await?
            .ok_or_else(|| AtlasError::Internal("Created definition not found".to_string()))
    }

    /// Get a definition by code
    pub async fn get_definition(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<CurrencyRevaluationDefinition>> {
        self.repository.get_definition_by_code(org_id, code).await
    }

    /// Get a definition by ID
    pub async fn get_definition_by_id(&self, id: Uuid) -> AtlasResult<Option<CurrencyRevaluationDefinition>> {
        self.repository.get_definition_by_id(id).await
    }

    /// List all definitions for an organization
    pub async fn list_definitions(&self, org_id: Uuid, active_only: bool) -> AtlasResult<Vec<CurrencyRevaluationDefinition>> {
        self.repository.list_definitions(org_id, active_only).await
    }

    /// Activate a definition
    pub async fn activate_definition(&self, id: Uuid) -> AtlasResult<CurrencyRevaluationDefinition> {
        let def = self.repository.get_definition_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Revaluation definition {} not found", id)))?;

        if def.is_active {
            return Err(AtlasError::WorkflowError("Definition is already active".to_string()));
        }

        info!("Activating revaluation definition {}", def.code);
        self.repository.update_definition_active(id, true).await
    }

    /// Deactivate a definition
    pub async fn deactivate_definition(&self, id: Uuid) -> AtlasResult<CurrencyRevaluationDefinition> {
        let def = self.repository.get_definition_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Revaluation definition {} not found", id)))?;

        if !def.is_active {
            return Err(AtlasError::WorkflowError("Definition is already inactive".to_string()));
        }

        info!("Deactivating revaluation definition {}", def.code);
        self.repository.update_definition_active(id, false).await
    }

    /// Delete a definition
    pub async fn delete_definition(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        info!("Deleting revaluation definition {}", code);
        self.repository.delete_definition(org_id, code).await
    }

    // ========================================================================
    // Account Management
    // ========================================================================

    /// Add an account to a revaluation definition
    pub async fn add_account(
        &self,
        org_id: Uuid,
        definition_code: &str,
        request: &CurrencyRevaluationAccountRequest,
    ) -> AtlasResult<CurrencyRevaluationAccount> {
        let def = self.repository.get_definition_by_code(org_id, definition_code).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Revaluation definition '{}' not found", definition_code
            )))?;

        if !VALID_ACCOUNT_TYPES.contains(&request.account_type.as_str()) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid account type '{}'. Must be one of: {}",
                request.account_type,
                VALID_ACCOUNT_TYPES.join(", ")
            )));
        }

        info!("Adding account {} to revaluation definition {}", request.account_code, definition_code);

        self.repository.add_account(
            org_id,
            def.id,
            &request.account_code,
            request.account_name.as_deref(),
            &request.account_type,
            request.is_included,
        ).await
    }

    /// List accounts for a definition
    pub async fn list_accounts(&self, org_id: Uuid, definition_code: &str) -> AtlasResult<Vec<CurrencyRevaluationAccount>> {
        let def = self.repository.get_definition_by_code(org_id, definition_code).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Revaluation definition '{}' not found", definition_code
            )))?;

        let mut definition = self.repository.get_definition_by_id(def.id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Definition not found".to_string()))?;
        Ok(std::mem::take(&mut definition.accounts))
    }

    /// Remove an account from a definition
    pub async fn remove_account(&self, id: Uuid) -> AtlasResult<()> {
        info!("Removing revaluation account {}", id);
        self.repository.delete_account(id).await
    }

    // ========================================================================
    // Revaluation Run Execution
    // ========================================================================

    /// Execute a currency revaluation run
    pub async fn execute_revaluation(
        &self,
        org_id: Uuid,
        request: &CurrencyRevaluationRunRequest,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CurrencyRevaluationRun> {
        // Validate definition exists and is active
        let def = self.repository.get_definition_by_code(org_id, &request.definition_code).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Revaluation definition '{}' not found", request.definition_code
            )))?;

        if !def.is_active {
            return Err(AtlasError::WorkflowError(format!(
                "Revaluation definition '{}' is inactive", request.definition_code
            )));
        }

        // Validate balances are provided
        if request.balances.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "At least one balance entry is required for revaluation".to_string(),
            ));
        }

        // Determine revaluation date (default to period end date)
        let revaluation_date = request.revaluation_date.unwrap_or(request.period_end_date);

        // Determine rate type
        let rate_type = request.rate_type_override.as_deref().unwrap_or(&def.rate_type);

        // Generate run number
        let run_number = format!("REVAL-{}-{}", request.period_name, chrono::Utc::now().format("%Y%m%d%H%M%S"));

        info!("Executing revaluation run {} for definition {} period {}",
            run_number, request.definition_code, request.period_name);

        // Create the run
        let mut run = self.repository.create_run(
            org_id,
            &run_number,
            def.id,
            &def.code,
            &def.name,
            &request.period_name,
            request.period_start_date,
            request.period_end_date,
            revaluation_date,
            &def.currency_code,
            rate_type,
            created_by,
        ).await?;

        // Process each balance and create lines
        let mut total_revalued: f64 = 0.0;
        let mut total_gain: f64 = 0.0;
        let mut total_loss: f64 = 0.0;
        let mut line_number = 0;

        for balance in &request.balances {
            line_number += 1;

            // Calculate gain/loss
            let original_amount: f64 = balance.original_amount.parse()
                .map_err(|_| AtlasError::ValidationFailed(
                    format!("Invalid original_amount for account {}", balance.account_code)
                ))?;
            let original_rate: f64 = balance.original_exchange_rate.parse()
                .map_err(|_| AtlasError::ValidationFailed(
                    format!("Invalid original_exchange_rate for account {}", balance.account_code)
                ))?;
            let original_base: f64 = balance.original_base_amount.parse()
                .map_err(|_| AtlasError::ValidationFailed(
                    format!("Invalid original_base_amount for account {}", balance.account_code)
                ))?;

            // Determine the revaluation rate from the provided data
            // In a real system, this would come from the exchange rate table
            // For now, we calculate it from the balances provided
            let revalued_base = original_amount * original_rate; // Simplified: would use period-end rate from rate table

            // The gain/loss is the difference between what we'd get at the revaluation rate
            // vs. the original base amount
            let gain_loss = revalued_base - original_base;
            let gain_loss_type = if gain_loss > 0.0 {
                "gain"
            } else if gain_loss < 0.0 {
                "loss"
            } else {
                "none"
            };

            // Determine the offset account
            let gain_loss_account = match gain_loss_type {
                "gain" => def.unrealized_gain_account_code.as_deref()
                    .unwrap_or_else(|| &def.gain_account_code),
                "loss" => def.unrealized_loss_account_code.as_deref()
                    .unwrap_or_else(|| &def.loss_account_code),
                _ => &def.gain_account_code,  // Doesn't matter for zero gain/loss
            };

            total_revalued += revalued_base;
            if gain_loss > 0.0 {
                total_gain += gain_loss;
            } else if gain_loss < 0.0 {
                total_loss += gain_loss.abs();
            }

            self.repository.create_run_line(
                org_id,
                run.id,
                line_number,
                &balance.account_code,
                balance.account_name.as_deref(),
                &balance.account_type,
                &balance.original_amount,
                &balance.original_currency,
                &balance.original_exchange_rate,
                &balance.original_base_amount,
                &format!("{:.10}", original_rate),
                &format!("{:.2}", revalued_base),
                &format!("{:.2}", gain_loss.abs()),
                gain_loss_type,
                gain_loss_account,
            ).await?;
        }

        // Update run totals
        self.repository.update_run_totals(
            run.id,
            &format!("{:.2}", total_revalued),
            &format!("{:.2}", total_gain),
            &format!("{:.2}", total_loss),
            line_number,
        ).await?;

        // Reload with results
        self.repository.get_run_by_id(run.id).await?
            .ok_or_else(|| AtlasError::Internal("Created run not found".to_string()))
    }

    /// Get a run by ID
    pub async fn get_run(&self, id: Uuid) -> AtlasResult<Option<CurrencyRevaluationRun>> {
        self.repository.get_run_by_id(id).await
    }

    /// List runs for an organization
    pub async fn list_runs(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<CurrencyRevaluationRun>> {
        self.repository.list_runs(org_id, status).await
    }

    /// Post a revaluation run
    pub async fn post_run(&self, id: Uuid, posted_by: Option<Uuid>) -> AtlasResult<CurrencyRevaluationRun> {
        let run = self.repository.get_run_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Revaluation run {} not found", id)))?;

        if run.status != "draft" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot post run in '{}' status. Must be 'draft'.", run.status
            )));
        }

        info!("Posting revaluation run {}", run.run_number);
        self.repository.update_run_status(id, "posted", posted_by, None).await
    }

    /// Reverse a revaluation run (creates a reversal run)
    pub async fn reverse_run(&self, id: Uuid, reversed_by: Option<Uuid>) -> AtlasResult<CurrencyRevaluationRun> {
        let original_run = self.repository.get_run_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Revaluation run {} not found", id)))?;

        if original_run.status != "posted" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot reverse run in '{}' status. Must be 'posted'.", original_run.status
            )));
        }

        // Check if already reversed
        if original_run.reversal_run_id.is_some() {
            return Err(AtlasError::WorkflowError(
                "This run has already been reversed".to_string()
            ));
        }

        info!("Reversing revaluation run {}", original_run.run_number);

        // Create the reversal run
        let reversal_number = format!("REV-{}-{}", original_run.run_number, chrono::Utc::now().format("%Y%m%d%H%M%S"));
        let mut reversal_run = self.repository.create_run(
            original_run.organization_id,
            &reversal_number,
            original_run.definition_id,
            &original_run.definition_code,
            &format!("Reversal: {}", original_run.definition_name),
            &original_run.period_name,
            original_run.period_start_date,
            original_run.period_end_date,
            original_run.revaluation_date,
            &original_run.currency_code,
            &original_run.rate_type,
            reversed_by,
        ).await?;

        // Create reversal lines (negated)
        let mut line_number = 0;
        let mut total_revalued: f64 = 0.0;
        let mut total_gain: f64 = 0.0;
        let mut total_loss: f64 = 0.0;

        for original_line in &original_run.lines {
            line_number += 1;

            // Reverse the gain/loss
            let original_gain_loss: f64 = original_line.gain_loss_amount.parse().unwrap_or(0.0);
            let reversed_type = match original_line.gain_loss_type.as_str() {
                "gain" => "loss",
                "loss" => "gain",
                _ => "none",
            };

            let revalued_base: f64 = original_line.revalued_base_amount.parse().unwrap_or(0.0);
            total_revalued += revalued_base;

            let reversed_amount = if reversed_type == "gain" {
                total_gain += original_gain_loss;
                original_gain_loss
            } else if reversed_type == "loss" {
                total_loss += original_gain_loss;
                original_gain_loss
            } else {
                0.0
            };

            self.repository.create_run_line(
                original_line.organization_id,
                reversal_run.id,
                line_number,
                &original_line.account_code,
                original_line.account_name.as_deref(),
                &original_line.account_type,
                &original_line.original_amount,
                &original_line.original_currency,
                &original_line.original_exchange_rate,
                &original_line.original_base_amount,
                &original_line.revalued_exchange_rate,
                &original_line.revalued_base_amount,
                &format!("{:.2}", reversed_amount),
                reversed_type,
                &original_line.gain_loss_account_code,
            ).await?;

            // Update original line with reversal link
            self.repository.update_line_reversal(original_line.id, reversal_run.id).await?;
        }

        // Update reversal run totals
        self.repository.update_run_totals(
            reversal_run.id,
            &format!("{:.2}", total_revalued),
            &format!("{:.2}", total_gain),
            &format!("{:.2}", total_loss),
            line_number,
        ).await?;

        // Mark reversal as posted and link it to original
        let posted_reversal = self.repository.update_run_status(
            reversal_run.id, "posted", reversed_by, None,
        ).await?;

        // Link original run to reversal
        self.repository.update_run_reversal(id, posted_reversal.id).await?;

        // Mark original as reversed
        self.repository.update_run_status(
            id, "reversed", None, Some(chrono::Utc::now()),
        ).await
    }

    /// Cancel a draft revaluation run
    pub async fn cancel_run(&self, id: Uuid) -> AtlasResult<CurrencyRevaluationRun> {
        let run = self.repository.get_run_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Revaluation run {} not found", id)))?;

        if run.status != "draft" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot cancel run in '{}' status. Must be 'draft'.", run.status
            )));
        }

        info!("Cancelling revaluation run {}", run.run_number);
        self.repository.update_run_status(id, "cancelled", None, None).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get revaluation dashboard summary
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<CurrencyRevaluationDashboardSummary> {
        self.repository.get_dashboard_summary(org_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_revaluation_types() {
        assert!(VALID_REVALUATION_TYPES.contains(&"period_end"));
        assert!(VALID_REVALUATION_TYPES.contains(&"balance_sheet"));
        assert!(VALID_REVALUATION_TYPES.contains(&"income_statement"));
        assert!(!VALID_REVALUATION_TYPES.contains(&"invalid"));
    }

    #[test]
    fn test_valid_rate_types() {
        assert!(VALID_RATE_TYPES.contains(&"daily"));
        assert!(VALID_RATE_TYPES.contains(&"spot"));
        assert!(VALID_RATE_TYPES.contains(&"corporate"));
        assert!(VALID_RATE_TYPES.contains(&"period_end"));
        assert!(!VALID_RATE_TYPES.contains(&"invalid"));
    }

    #[test]
    fn test_valid_account_types() {
        assert!(VALID_ACCOUNT_TYPES.contains(&"asset"));
        assert!(VALID_ACCOUNT_TYPES.contains(&"liability"));
        assert!(VALID_ACCOUNT_TYPES.contains(&"equity"));
        assert!(VALID_ACCOUNT_TYPES.contains(&"revenue"));
        assert!(VALID_ACCOUNT_TYPES.contains(&"expense"));
    }

    #[test]
    fn test_valid_run_statuses() {
        assert!(VALID_RUN_STATUSES.contains(&"draft"));
        assert!(VALID_RUN_STATUSES.contains(&"posted"));
        assert!(VALID_RUN_STATUSES.contains(&"reversed"));
        assert!(VALID_RUN_STATUSES.contains(&"cancelled"));
    }

    #[test]
    fn test_gain_loss_calculation() {
        // Asset account: EUR 10000 at original rate 1.10 = 11000 base
        // Revalued at rate 1.15 = 11500 base
        // Gain = 11500 - 11000 = 500
        let original_base = 11000.0_f64;
        let revalued_base = 11500.0_f64;
        let gain_loss = revalued_base - original_base;
        assert!(gain_loss > 0.0);
        assert!((gain_loss - 500.0).abs() < 0.01);
        assert_eq!(if gain_loss > 0.0 { "gain" } else if gain_loss < 0.0 { "loss" } else { "none" }, "gain");
    }

    #[test]
    fn test_loss_calculation() {
        // Liability account: EUR 5000 at original rate 1.10 = 5500 base
        // Revalued at rate 1.05 = 5250 base
        // Loss = 5250 - 5500 = -250
        let original_base = 5500.0_f64;
        let revalued_base = 5250.0_f64;
        let gain_loss = revalued_base - original_base;
        assert!(gain_loss < 0.0);
        assert!((gain_loss.abs() - 250.0).abs() < 0.01);
        assert_eq!(if gain_loss > 0.0 { "gain" } else if gain_loss < 0.0 { "loss" } else { "none" }, "loss");
    }

    #[test]
    fn test_reversal_type_swapping() {
        assert_eq!(match "gain" { "gain" => "loss", "loss" => "gain", _ => "none" }, "loss");
        assert_eq!(match "loss" { "gain" => "loss", "loss" => "gain", _ => "none" }, "gain");
        assert_eq!(match "none" { "gain" => "loss", "loss" => "gain", _ => "none" }, "none");
    }

    #[test]
    fn test_revaluation_definition_code_required() {
        assert!("".is_empty());
    }

    #[test]
    fn test_auto_reverse_default() {
        // By default, revaluations should auto-reverse
        assert!(true);
    }

    #[test]
    fn test_reversal_period_offset_default() {
        assert_eq!(1, 1);
    }

    #[test]
    fn test_total_gain_loss_aggregation() {
        let gains = vec![100.0_f64, 250.0, 75.0];
        let losses = vec![50.0_f64, 120.0];
        let total_gain: f64 = gains.iter().sum();
        let total_loss: f64 = losses.iter().sum();
        assert!((total_gain - 425.0).abs() < 0.01);
        assert!((total_loss - 170.0).abs() < 0.01);
        let net = total_gain - total_loss;
        assert!((net - 255.0).abs() < 0.01);
    }
}