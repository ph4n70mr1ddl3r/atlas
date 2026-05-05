//! Cash Concentration Engine
//!
//! Manages cash pool lifecycle, participant management, sweep rules,
//! and sweep execution for automated cash concentration.
//!
//! Oracle Fusion Cloud ERP equivalent: Treasury > Cash Pooling

use atlas_shared::{AtlasError, AtlasResult};
use super::repository::{
    CashConcentrationRepository, PoolCreateParams, ParticipantCreateParams,
    SweepRuleCreateParams, SweepRunCreateParams, SweepRunLineCreateParams,
};
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

// Valid pool types
const VALID_POOL_TYPES: &[&str] = &["physical", "notional"];

// Valid pool statuses
const VALID_POOL_STATUSES: &[&str] = &["draft", "active", "suspended", "closed"];

// Valid sweep frequencies
const VALID_SWEEP_FREQUENCIES: &[&str] = &["daily", "weekly", "monthly", "on_demand"];

// Valid participant types
const VALID_PARTICIPANT_TYPES: &[&str] = &["source", "concentration", "both"];

// Valid sweep directions
const VALID_SWEEP_DIRECTIONS: &[&str] = &[
    "to_concentration", "from_concentration", "two_way",
];

// Valid participant statuses
const VALID_PARTICIPANT_STATUSES: &[&str] = &["active", "suspended", "removed"];

// Valid sweep types
const VALID_SWEEP_TYPES: &[&str] = &[
    "zero_balance", "target_balance", "threshold", "excess_balance",
];

// Valid run types
const VALID_RUN_TYPES: &[&str] = &["scheduled", "manual", "automatic"];

// Valid run statuses
const VALID_RUN_STATUSES: &[&str] = &[
    "pending", "in_progress", "completed", "partially_completed", "failed", "cancelled",
];

// Valid line statuses
#[allow(dead_code)]
const VALID_LINE_STATUSES: &[&str] = &["pending", "completed", "failed", "skipped"];

/// Calculate the sweep amount for a zero-balance sweep.
/// Returns the amount to sweep from the source account to the concentration account.
pub fn calculate_zero_balance_sweep(current_balance: f64, minimum_balance: f64) -> f64 {
    let excess = current_balance - minimum_balance;
    if excess > 0.0 { excess } else { 0.0 }
}

/// Calculate the sweep amount for a target-balance sweep.
/// Returns the amount needed to bring the account to its target balance.
pub fn calculate_target_balance_sweep(current_balance: f64, target_balance: f64) -> f64 {
    let excess = current_balance - target_balance;
    if excess > 0.0 { excess } else { 0.0 }
}

/// Calculate the sweep amount for a threshold sweep.
/// Only sweeps if the balance exceeds the threshold.
pub fn calculate_threshold_sweep(current_balance: f64, threshold: f64, minimum_balance: f64) -> f64 {
    if current_balance > threshold {
        let excess = current_balance - minimum_balance;
        if excess > 0.0 { excess } else { 0.0 }
    } else {
        0.0
    }
}

/// Cash Concentration Engine
pub struct CashConcentrationEngine {
    repository: Arc<dyn CashConcentrationRepository>,
}

impl CashConcentrationEngine {
    pub fn new(repository: Arc<dyn CashConcentrationRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Cash Pools
    // ========================================================================

    /// Create a new cash pool
    pub async fn create_pool(
        &self,
        org_id: Uuid,
        pool_code: &str,
        pool_name: &str,
        pool_type: &str,
        concentration_account_id: Option<Uuid>,
        concentration_account_name: Option<&str>,
        currency_code: &str,
        sweep_frequency: Option<&str>,
        sweep_time: Option<&str>,
        minimum_transfer_amount: Option<&str>,
        maximum_transfer_amount: Option<&str>,
        target_balance: Option<&str>,
        interest_allocation_method: Option<&str>,
        interest_rate: Option<&str>,
        effective_date: Option<chrono::NaiveDate>,
        termination_date: Option<chrono::NaiveDate>,
        description: Option<&str>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::CashPool> {
        if pool_code.is_empty() {
            return Err(AtlasError::ValidationFailed("Pool code is required".to_string()));
        }
        if pool_name.is_empty() {
            return Err(AtlasError::ValidationFailed("Pool name is required".to_string()));
        }
        if !VALID_POOL_TYPES.contains(&pool_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid pool type '{}'. Must be one of: {}",
                pool_type, VALID_POOL_TYPES.join(", ")
            )));
        }
        if let Some(freq) = sweep_frequency {
            if !VALID_SWEEP_FREQUENCIES.contains(&freq) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid sweep frequency '{}'. Must be one of: {}",
                    freq, VALID_SWEEP_FREQUENCIES.join(", ")
                )));
            }
        }
        if let Some(max_amt) = maximum_transfer_amount {
            let max_val: f64 = max_amt.parse().map_err(|_| AtlasError::ValidationFailed(
                "Maximum transfer amount must be a valid number".to_string(),
            ))?;
            if max_val < 0.0 {
                return Err(AtlasError::ValidationFailed(
                    "Maximum transfer amount cannot be negative".to_string(),
                ));
            }
        }

        info!("Creating cash pool {} for org {}", pool_code, org_id);

        self.repository.create_pool(&PoolCreateParams {
            org_id,
            pool_code: pool_code.to_string(),
            pool_name: pool_name.to_string(),
            pool_type: pool_type.to_string(),
            concentration_account_id,
            concentration_account_name: concentration_account_name.map(|s| s.to_string()),
            currency_code: currency_code.to_string(),
            sweep_frequency: sweep_frequency.map(|s| s.to_string()),
            sweep_time: sweep_time.map(|s| s.to_string()),
            minimum_transfer_amount: minimum_transfer_amount.map(|s| s.to_string()),
            maximum_transfer_amount: maximum_transfer_amount.map(|s| s.to_string()),
            target_balance: target_balance.map(|s| s.to_string()),
            interest_allocation_method: interest_allocation_method.map(|s| s.to_string()),
            interest_rate: interest_rate.map(|s| s.to_string()),
            effective_date,
            termination_date,
            description: description.map(|s| s.to_string()),
            notes: notes.map(|s| s.to_string()),
            created_by,
        }).await
    }

    /// Get a cash pool by code
    pub async fn get_pool(&self, org_id: Uuid, pool_code: &str) -> AtlasResult<Option<atlas_shared::CashPool>> {
        self.repository.get_pool(org_id, pool_code).await
    }

    /// Get a cash pool by ID
    pub async fn get_pool_by_id(&self, id: Uuid) -> AtlasResult<Option<atlas_shared::CashPool>> {
        self.repository.get_pool_by_id(id).await
    }

    /// List cash pools with optional status filter
    pub async fn list_pools(&self, org_id: Uuid, status: Option<&str>, pool_type: Option<&str>) -> AtlasResult<Vec<atlas_shared::CashPool>> {
        if let Some(s) = status {
            if !VALID_POOL_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}",
                    s, VALID_POOL_STATUSES.join(", ")
                )));
            }
        }
        if let Some(t) = pool_type {
            if !VALID_POOL_TYPES.contains(&t) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid pool type '{}'. Must be one of: {}",
                    t, VALID_POOL_TYPES.join(", ")
                )));
            }
        }
        self.repository.list_pools(org_id, status, pool_type).await
    }

    /// Activate a cash pool
    pub async fn activate_pool(&self, id: Uuid) -> AtlasResult<atlas_shared::CashPool> {
        let pool = self.repository.get_pool_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Cash pool {} not found", id)))?;

        if pool.status != "draft" && pool.status != "suspended" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot activate pool in '{}' status. Must be 'draft' or 'suspended'.",
                pool.status
            )));
        }

        info!("Activated cash pool {}", pool.pool_code);
        self.repository.update_pool_status(id, "active").await
    }

    /// Suspend a cash pool
    pub async fn suspend_pool(&self, id: Uuid) -> AtlasResult<atlas_shared::CashPool> {
        let pool = self.repository.get_pool_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Cash pool {} not found", id)))?;

        if pool.status != "active" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot suspend pool in '{}' status. Must be 'active'.",
                pool.status
            )));
        }

        info!("Suspended cash pool {}", pool.pool_code);
        self.repository.update_pool_status(id, "suspended").await
    }

    /// Close a cash pool
    pub async fn close_pool(&self, id: Uuid) -> AtlasResult<atlas_shared::CashPool> {
        let pool = self.repository.get_pool_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Cash pool {} not found", id)))?;

        if pool.status == "closed" {
            return Err(AtlasError::WorkflowError("Pool is already closed".to_string()));
        }

        info!("Closed cash pool {}", pool.pool_code);
        self.repository.update_pool_status(id, "closed").await
    }

    /// Delete a cash pool (only if draft)
    pub async fn delete_pool(&self, org_id: Uuid, pool_code: &str) -> AtlasResult<()> {
        let pool = self.repository.get_pool(org_id, pool_code).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Cash pool {} not found", pool_code)))?;

        if pool.status != "draft" {
            return Err(AtlasError::WorkflowError(
                "Cannot delete pool that is not in 'draft' status".to_string(),
            ));
        }

        info!("Deleted cash pool {}", pool_code);
        self.repository.delete_pool(org_id, pool_code).await
    }

    // ========================================================================
    // Pool Participants
    // ========================================================================

    /// Add a participant to a cash pool
    pub async fn add_participant(
        &self,
        org_id: Uuid,
        pool_id: Uuid,
        participant_code: &str,
        bank_account_id: Option<Uuid>,
        bank_account_name: Option<&str>,
        bank_name: Option<&str>,
        account_number: Option<&str>,
        participant_type: &str,
        sweep_direction: &str,
        priority: Option<i32>,
        minimum_balance: Option<&str>,
        maximum_balance: Option<&str>,
        threshold_amount: Option<&str>,
        current_balance: Option<&str>,
        entity_id: Option<Uuid>,
        entity_name: Option<&str>,
        effective_date: Option<chrono::NaiveDate>,
        description: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::CashPoolParticipant> {
        // Validate pool exists
        let pool = self.repository.get_pool_by_id(pool_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Pool {} not found", pool_id)))?;

        if pool.status == "closed" {
            return Err(AtlasError::WorkflowError("Cannot add participants to a closed pool".to_string()));
        }

        if !VALID_PARTICIPANT_TYPES.contains(&participant_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid participant type '{}'. Must be one of: {}",
                participant_type, VALID_PARTICIPANT_TYPES.join(", ")
            )));
        }
        if !VALID_SWEEP_DIRECTIONS.contains(&sweep_direction) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid sweep direction '{}'. Must be one of: {}",
                sweep_direction, VALID_SWEEP_DIRECTIONS.join(", ")
            )));
        }

        info!("Adding participant {} to pool {}", participant_code, pool.pool_code);

        self.repository.create_participant(&ParticipantCreateParams {
            org_id,
            pool_id,
            participant_code: participant_code.to_string(),
            bank_account_id,
            bank_account_name: bank_account_name.map(|s| s.to_string()),
            bank_name: bank_name.map(|s| s.to_string()),
            account_number: account_number.map(|s| s.to_string()),
            participant_type: participant_type.to_string(),
            sweep_direction: sweep_direction.to_string(),
            priority,
            minimum_balance: minimum_balance.map(|s| s.to_string()),
            maximum_balance: maximum_balance.map(|s| s.to_string()),
            threshold_amount: threshold_amount.map(|s| s.to_string()),
            current_balance: current_balance.map(|s| s.to_string()),
            entity_id,
            entity_name: entity_name.map(|s| s.to_string()),
            effective_date,
            description: description.map(|s| s.to_string()),
            created_by,
        }).await
    }

    /// List participants for a pool
    pub async fn list_participants(&self, pool_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<atlas_shared::CashPoolParticipant>> {
        if let Some(s) = status {
            if !VALID_PARTICIPANT_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid participant status '{}'. Must be one of: {}",
                    s, VALID_PARTICIPANT_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_participants(pool_id, status).await
    }

    /// Remove a participant from a pool
    pub async fn remove_participant(&self, pool_id: Uuid, participant_code: &str) -> AtlasResult<()> {
        let participant = self.repository.get_participant(pool_id, participant_code).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Participant {} not found in pool", participant_code
            )))?;

        if participant.status == "removed" {
            return Err(AtlasError::WorkflowError("Participant is already removed".to_string()));
        }

        info!("Removed participant {} from pool", participant_code);
        self.repository.update_participant_status(participant.id, "removed").await?;
        Ok(())
    }

    /// Update participant balance
    pub async fn update_participant_balance(
        &self,
        participant_id: Uuid,
        new_balance: &str,
    ) -> AtlasResult<atlas_shared::CashPoolParticipant> {
        let _bal: f64 = new_balance.parse().map_err(|_| AtlasError::ValidationFailed(
            "Balance must be a valid number".to_string(),
        ))?;

        self.repository.update_participant_balance(participant_id, new_balance).await
    }

    // ========================================================================
    // Sweep Rules
    // ========================================================================

    /// Create a sweep rule
    pub async fn create_sweep_rule(
        &self,
        org_id: Uuid,
        pool_id: Uuid,
        rule_code: &str,
        rule_name: &str,
        sweep_type: &str,
        participant_id: Option<Uuid>,
        direction: &str,
        trigger_condition: Option<&str>,
        threshold_amount: Option<&str>,
        target_balance: Option<&str>,
        minimum_transfer: Option<&str>,
        maximum_transfer: Option<&str>,
        priority: Option<i32>,
        effective_date: Option<chrono::NaiveDate>,
        description: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::CashPoolSweepRule> {
        // Validate pool exists
        let pool = self.repository.get_pool_by_id(pool_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Pool {} not found", pool_id)))?;

        if pool.status == "closed" {
            return Err(AtlasError::WorkflowError("Cannot add rules to a closed pool".to_string()));
        }

        if !VALID_SWEEP_TYPES.contains(&sweep_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid sweep type '{}'. Must be one of: {}",
                sweep_type, VALID_SWEEP_TYPES.join(", ")
            )));
        }
        if !VALID_SWEEP_DIRECTIONS.contains(&direction) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid direction '{}'. Must be one of: {}",
                direction, VALID_SWEEP_DIRECTIONS.join(", ")
            )));
        }

        info!("Creating sweep rule {} for pool {}", rule_code, pool.pool_code);

        self.repository.create_sweep_rule(&SweepRuleCreateParams {
            org_id,
            pool_id,
            rule_code: rule_code.to_string(),
            rule_name: rule_name.to_string(),
            sweep_type: sweep_type.to_string(),
            participant_id,
            direction: direction.to_string(),
            trigger_condition: trigger_condition.map(|s| s.to_string()),
            threshold_amount: threshold_amount.map(|s| s.to_string()),
            target_balance: target_balance.map(|s| s.to_string()),
            minimum_transfer: minimum_transfer.map(|s| s.to_string()),
            maximum_transfer: maximum_transfer.map(|s| s.to_string()),
            priority,
            effective_date,
            description: description.map(|s| s.to_string()),
            created_by,
        }).await
    }

    /// List sweep rules for a pool
    pub async fn list_sweep_rules(&self, pool_id: Uuid) -> AtlasResult<Vec<atlas_shared::CashPoolSweepRule>> {
        self.repository.list_sweep_rules(pool_id).await
    }

    /// Delete a sweep rule
    pub async fn delete_sweep_rule(&self, pool_id: Uuid, rule_code: &str) -> AtlasResult<()> {
        info!("Deleting sweep rule {} from pool", rule_code);
        self.repository.delete_sweep_rule(pool_id, rule_code).await
    }

    // ========================================================================
    // Sweep Runs
    // ========================================================================

    /// Execute a sweep run
    pub async fn execute_sweep(
        &self,
        org_id: Uuid,
        pool_id: Uuid,
        run_type: &str,
        run_date: chrono::NaiveDate,
        notes: Option<&str>,
        initiated_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::CashPoolSweepRun> {
        if !VALID_RUN_TYPES.contains(&run_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid run type '{}'. Must be one of: {}",
                run_type, VALID_RUN_TYPES.join(", ")
            )));
        }

        let pool = self.repository.get_pool_by_id(pool_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Pool {} not found", pool_id)))?;

        if pool.status != "active" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot execute sweep on pool in '{}' status. Must be 'active'.",
                pool.status
            )));
        }

        // Generate run number
        let next_num = self.repository.get_latest_run_number(org_id).await? + 1;
        let run_number = format!("SWEEP-{:04}", next_num);

        info!("Executing sweep run {} for pool {}", run_number, pool.pool_code);

        // Create the run
        let run = self.repository.create_sweep_run(&SweepRunCreateParams {
            org_id,
            pool_id,
            run_number: run_number.clone(),
            run_date,
            run_type: run_type.to_string(),
            initiated_by,
            notes: notes.map(|s| s.to_string()),
        }).await?;

        // Get active participants
        let participants = self.repository.list_participants(pool_id, Some("active")).await?;

        // Get active rules
        let rules = self.repository.list_sweep_rules(pool_id).await?;
        let active_rules: Vec<_> = rules.iter().filter(|r| r.is_active.unwrap_or(true)).collect();

        let mut total_swept = 0.0_f64;
        let mut total_txns = 0i32;
        let mut successful = 0i32;
        let mut failed = 0i32;

        // Process each participant
        for participant in &participants {
            // Find applicable rule for this participant
            let rule = active_rules.iter().find(|r| {
                r.participant_id == Some(participant.id) || r.participant_id.is_none()
            });

            let current_bal: f64 = participant.current_balance
                .as_ref()
                .and_then(|b| b.parse().ok())
                .unwrap_or(0.0);

            let min_bal: f64 = participant.minimum_balance
                .as_ref()
                .and_then(|b| b.parse().ok())
                .unwrap_or(0.0);

            let sweep_amount = if let Some(r) = rule {
                match r.sweep_type.as_str() {
                    "zero_balance" => calculate_zero_balance_sweep(current_bal, min_bal),
                    "target_balance" => {
                        let target: f64 = r.target_balance
                            .as_ref()
                            .and_then(|t| t.parse().ok())
                            .unwrap_or(min_bal);
                        calculate_target_balance_sweep(current_bal, target)
                    },
                    "threshold" => {
                        let threshold: f64 = r.threshold_amount
                            .as_ref()
                            .and_then(|t| t.parse().ok())
                            .unwrap_or(0.0);
                        calculate_threshold_sweep(current_bal, threshold, min_bal)
                    },
                    "excess_balance" => {
                        let max_bal: f64 = participant.maximum_balance
                            .as_ref()
                            .and_then(|b| b.parse().ok())
                            .unwrap_or(f64::MAX);
                        if current_bal > max_bal {
                            current_bal - max_bal
                        } else {
                            0.0
                        }
                    },
                    _ => 0.0,
                }
            } else {
                // Default: zero-balance sweep
                calculate_zero_balance_sweep(current_bal, min_bal)
            };

            // Apply minimum transfer amount from pool config
            let pool_min: f64 = pool.minimum_transfer_amount
                .as_ref()
                .and_then(|m| m.parse().ok())
                .unwrap_or(0.0);
            if sweep_amount > 0.0 && sweep_amount < pool_min {
                continue; // Skip: below minimum transfer
            }

            // Apply maximum transfer amount
            let pool_max: f64 = pool.maximum_transfer_amount
                .as_ref()
                .and_then(|m| m.parse().ok())
                .unwrap_or(f64::MAX);
            let sweep_amount = if sweep_amount > pool_max { pool_max } else { sweep_amount };

            total_txns += 1;

            if sweep_amount > 0.0 {
                let post_balance = current_bal - sweep_amount;

                self.repository.create_sweep_run_line(&SweepRunLineCreateParams {
                    organization_id: org_id,
                    sweep_run_id: run.id,
                    pool_id,
                    participant_id: participant.id,
                    participant_code: Some(participant.participant_code.clone()),
                    bank_account_name: participant.bank_account_name.clone(),
                    sweep_rule_id: rule.map(|r| r.id),
                    direction: "debit".to_string(),
                    pre_sweep_balance: Some(format!("{:.2}", current_bal)),
                    sweep_amount: format!("{:.2}", sweep_amount),
                    post_sweep_balance: Some(format!("{:.2}", post_balance)),
                    status: "completed".to_string(),
                }).await?;

                // Update participant balance
                self.repository.update_participant_balance(
                    participant.id, &format!("{:.2}", post_balance),
                ).await?;

                total_swept += sweep_amount;
                successful += 1;
            } else {
                // Skip: nothing to sweep
                self.repository.create_sweep_run_line(&SweepRunLineCreateParams {
                    organization_id: org_id,
                    sweep_run_id: run.id,
                    pool_id,
                    participant_id: participant.id,
                    participant_code: Some(participant.participant_code.clone()),
                    bank_account_name: participant.bank_account_name.clone(),
                    sweep_rule_id: rule.map(|r| r.id),
                    direction: "debit".to_string(),
                    pre_sweep_balance: Some(format!("{:.2}", current_bal)),
                    sweep_amount: "0.00".to_string(),
                    post_sweep_balance: Some(format!("{:.2}", current_bal)),
                    status: "skipped".to_string(),
                }).await?;
            }
        }

        // Update run with results
        let run_status = if failed > 0 && successful > 0 {
            "partially_completed"
        } else if failed > 0 {
            "failed"
        } else {
            "completed"
        };

        let run = self.repository.update_sweep_run_status(
            run.id,
            run_status,
            Some(&format!("{:.2}", total_swept)),
            Some(total_txns),
            Some(successful),
            Some(failed),
        ).await?;

        Ok(run)
    }

    /// Get a sweep run
    pub async fn get_sweep_run(&self, id: Uuid) -> AtlasResult<Option<atlas_shared::CashPoolSweepRun>> {
        self.repository.get_sweep_run(id).await
    }

    /// List sweep runs for a pool
    pub async fn list_sweep_runs(&self, pool_id: Uuid) -> AtlasResult<Vec<atlas_shared::CashPoolSweepRun>> {
        self.repository.list_sweep_runs(pool_id).await
    }

    /// List sweep run lines for a run
    pub async fn list_sweep_run_lines(&self, sweep_run_id: Uuid) -> AtlasResult<Vec<atlas_shared::CashPoolSweepRunLine>> {
        self.repository.list_sweep_run_lines(sweep_run_id).await
    }

    /// Cancel a sweep run (only if pending)
    pub async fn cancel_sweep_run(&self, id: Uuid) -> AtlasResult<atlas_shared::CashPoolSweepRun> {
        let run = self.repository.get_sweep_run(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Sweep run {} not found", id)))?;

        if run.status != "pending" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot cancel sweep run in '{}' status. Must be 'pending'.",
                run.status
            )));
        }

        info!("Cancelled sweep run {}", run.run_number);
        self.repository.update_sweep_run_status(id, "cancelled", None, None, None, None).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get cash concentration dashboard summary
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<atlas_shared::CashPoolDashboard> {
        self.repository.get_dashboard(org_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_pool_types() {
        assert!(VALID_POOL_TYPES.contains(&"physical"));
        assert!(VALID_POOL_TYPES.contains(&"notional"));
    }

    #[test]
    fn test_valid_pool_statuses() {
        assert!(VALID_POOL_STATUSES.contains(&"draft"));
        assert!(VALID_POOL_STATUSES.contains(&"active"));
        assert!(VALID_POOL_STATUSES.contains(&"suspended"));
        assert!(VALID_POOL_STATUSES.contains(&"closed"));
    }

    #[test]
    fn test_valid_sweep_types() {
        assert!(VALID_SWEEP_TYPES.contains(&"zero_balance"));
        assert!(VALID_SWEEP_TYPES.contains(&"target_balance"));
        assert!(VALID_SWEEP_TYPES.contains(&"threshold"));
        assert!(VALID_SWEEP_TYPES.contains(&"excess_balance"));
    }

    #[test]
    fn test_valid_participant_types() {
        assert!(VALID_PARTICIPANT_TYPES.contains(&"source"));
        assert!(VALID_PARTICIPANT_TYPES.contains(&"concentration"));
        assert!(VALID_PARTICIPANT_TYPES.contains(&"both"));
    }

    #[test]
    fn test_zero_balance_sweep() {
        // Account with 50000, min balance 10000 => sweep 40000
        let amount = calculate_zero_balance_sweep(50000.0, 10000.0);
        assert!((amount - 40000.0).abs() < 0.01);

        // Account below minimum => sweep 0
        let amount = calculate_zero_balance_sweep(5000.0, 10000.0);
        assert!((amount).abs() < 0.01);
    }

    #[test]
    fn test_target_balance_sweep() {
        // Account with 75000, target 25000 => sweep 50000
        let amount = calculate_target_balance_sweep(75000.0, 25000.0);
        assert!((amount - 50000.0).abs() < 0.01);

        // Account at target => sweep 0
        let amount = calculate_target_balance_sweep(25000.0, 25000.0);
        assert!((amount).abs() < 0.01);

        // Account below target => sweep 0
        let amount = calculate_target_balance_sweep(10000.0, 25000.0);
        assert!((amount).abs() < 0.01);
    }

    #[test]
    fn test_threshold_sweep() {
        // Balance 80000 > threshold 50000, min 10000 => sweep 70000
        let amount = calculate_threshold_sweep(80000.0, 50000.0, 10000.0);
        assert!((amount - 70000.0).abs() < 0.01);

        // Balance 30000 < threshold 50000 => no sweep
        let amount = calculate_threshold_sweep(30000.0, 50000.0, 10000.0);
        assert!((amount).abs() < 0.01);
    }

    #[test]
    fn test_zero_balance_sweep_exact_minimum() {
        let amount = calculate_zero_balance_sweep(10000.0, 10000.0);
        assert!((amount).abs() < 0.01);
    }

    #[test]
    fn test_zero_balance_sweep_zero_balance() {
        let amount = calculate_zero_balance_sweep(0.0, 0.0);
        assert!((amount).abs() < 0.01);
    }
}
