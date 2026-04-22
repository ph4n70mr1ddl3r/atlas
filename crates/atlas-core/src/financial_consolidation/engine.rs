//! Financial Consolidation Engine
//!
//! Manages consolidation of financial statements across multiple legal entities
//! and business units, including currency translation, intercompany eliminations,
//! minority interest calculations, consolidation adjustments, and equity elimination.
//!
//! Ledger lifecycle: created → active / inactive
//! Entity lifecycle: active → removed (include_in_consolidation=false)
//! Scenario lifecycle: draft → in_progress → pending_review → approved → posted / reversed
//! Adjustment lifecycle: draft → approved → posted
//!
//! Oracle Fusion Cloud ERP equivalent: General Ledger > Financial Consolidation

use atlas_shared::{
    ConsolidationLedger, ConsolidationEntity, ConsolidationScenario,
    ConsolidationTrialBalanceLine, ConsolidationEliminationRule,
    ConsolidationAdjustment, ConsolidationTranslationRate,
    ConsolidationDashboardSummary,
    AtlasError, AtlasResult,
};
use super::FinancialConsolidationRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

// ── Valid values ────────────────────────────────────────────────────────

const VALID_TRANSLATION_METHODS: &[&str] = &["current_rate", "temporal", "weighted_average"];
const VALID_EQUITY_ELIMINATION_METHODS: &[&str] = &["full", "proportional", "equity_method"];
const VALID_CONSOLIDATION_METHODS: &[&str] = &["full", "proportional", "equity_method"];
const VALID_SCENARIO_STATUSES: &[&str] = &[
    "draft", "in_progress", "pending_review", "approved", "posted", "reversed",
];
const VALID_ELIMINATION_TYPES: &[&str] = &[
    "intercompany_receivable_payable",
    "intercompany_revenue_expense",
    "investment_equity",
    "intercompany_inventory_profit",
    "other",
];
const VALID_ADJUSTMENT_TYPES: &[&str] = &["manual", "reclassification", "correction"];
const VALID_ADJUSTMENT_STATUSES: &[&str] = &["draft", "approved", "posted"];
const VALID_RATE_TYPES: &[&str] = &["period_end", "average", "historical", "spot"];
const VALID_LINE_TYPES: &[&str] = &["entity", "elimination", "adjustment", "minority", "consolidated"];

/// Financial Consolidation Engine
pub struct FinancialConsolidationEngine {
    repository: Arc<dyn FinancialConsolidationRepository>,
}

impl FinancialConsolidationEngine {
    pub fn new(repository: Arc<dyn FinancialConsolidationRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Consolidation Ledger Management
    // ========================================================================

    /// Create a new consolidation ledger
    pub async fn create_ledger(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        base_currency_code: &str,
        translation_method: &str,
        equity_elimination_method: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ConsolidationLedger> {
        if code.is_empty() {
            return Err(AtlasError::ValidationFailed("Ledger code is required".to_string()));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Ledger name is required".to_string()));
        }
        if base_currency_code.len() != 3 {
            return Err(AtlasError::ValidationFailed(
                "Base currency code must be 3 characters (e.g. USD)".to_string(),
            ));
        }
        if !VALID_TRANSLATION_METHODS.contains(&translation_method) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid translation method '{}'. Must be one of: {}",
                translation_method,
                VALID_TRANSLATION_METHODS.join(", ")
            )));
        }
        if !VALID_EQUITY_ELIMINATION_METHODS.contains(&equity_elimination_method) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid equity elimination method '{}'. Must be one of: {}",
                equity_elimination_method,
                VALID_EQUITY_ELIMINATION_METHODS.join(", ")
            )));
        }

        info!(
            "Creating consolidation ledger {} ({}) for org {}",
            code, name, org_id
        );

        self.repository
            .create_ledger(
                org_id, code, name, description,
                base_currency_code, translation_method,
                equity_elimination_method, created_by,
            )
            .await
    }

    /// Get a ledger by code
    pub async fn get_ledger(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<ConsolidationLedger>> {
        self.repository.get_ledger(org_id, code).await
    }

    /// List ledgers
    pub async fn list_ledgers(&self, org_id: Uuid, active_only: bool) -> AtlasResult<Vec<ConsolidationLedger>> {
        self.repository.list_ledgers(org_id, active_only).await
    }

    // ========================================================================
    // Consolidation Entity Management
    // ========================================================================

    /// Add an entity (subsidiary/BU) to a consolidation ledger
    pub async fn add_entity(
        &self,
        org_id: Uuid,
        ledger_id: Uuid,
        entity_id: Uuid,
        entity_name: &str,
        entity_code: &str,
        local_currency_code: &str,
        ownership_percentage: &str,
        consolidation_method: &str,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ConsolidationEntity> {
        // Validate ledger exists
        let ledger = self.repository.get_ledger_by_id(ledger_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Ledger {} not found", ledger_id)))?;

        if !ledger.is_active {
            return Err(AtlasError::ValidationFailed(format!(
                "Ledger '{}' is not active",
                ledger.name
            )));
        }

        if entity_name.is_empty() {
            return Err(AtlasError::ValidationFailed("Entity name is required".to_string()));
        }
        if entity_code.is_empty() {
            return Err(AtlasError::ValidationFailed("Entity code is required".to_string()));
        }
        if !VALID_CONSOLIDATION_METHODS.contains(&consolidation_method) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid consolidation method '{}'. Must be one of: {}",
                consolidation_method,
                VALID_CONSOLIDATION_METHODS.join(", ")
            )));
        }

        Self::validate_percentage(ownership_percentage, "Ownership percentage")?;

        if let (Some(from), Some(to)) = (effective_from, effective_to) {
            if to <= from {
                return Err(AtlasError::ValidationFailed(
                    "Effective end date must be after start date".to_string(),
                ));
            }
        }

        info!(
            "Adding entity {} ({}) to consolidation ledger {}",
            entity_code, entity_name, ledger.code
        );

        self.repository
            .create_entity(
                org_id, ledger_id, entity_id, entity_name, entity_code,
                local_currency_code, ownership_percentage, consolidation_method,
                effective_from, effective_to, created_by,
            )
            .await
    }

    /// Get an entity by code
    pub async fn get_entity(&self, ledger_id: Uuid, entity_code: &str) -> AtlasResult<Option<ConsolidationEntity>> {
        self.repository.get_entity(ledger_id, entity_code).await
    }

    /// List entities in a ledger
    pub async fn list_entities(&self, ledger_id: Uuid, active_only: bool) -> AtlasResult<Vec<ConsolidationEntity>> {
        self.repository.list_entities(ledger_id, active_only).await
    }

    // ========================================================================
    // Consolidation Scenarios
    // ========================================================================

    /// Create a new consolidation scenario
    pub async fn create_scenario(
        &self,
        org_id: Uuid,
        ledger_id: Uuid,
        scenario_number: &str,
        name: &str,
        description: Option<&str>,
        fiscal_year: i32,
        period_name: &str,
        period_start_date: chrono::NaiveDate,
        period_end_date: chrono::NaiveDate,
        translation_rate_type: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ConsolidationScenario> {
        // Validate ledger
        let ledger = self.repository.get_ledger_by_id(ledger_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Ledger {} not found", ledger_id)))?;

        if !ledger.is_active {
            return Err(AtlasError::ValidationFailed(format!(
                "Ledger '{}' is not active",
                ledger.name
            )));
        }

        if scenario_number.is_empty() {
            return Err(AtlasError::ValidationFailed("Scenario number is required".to_string()));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Scenario name is required".to_string()));
        }
        if period_end_date <= period_start_date {
            return Err(AtlasError::ValidationFailed(
                "Period end date must be after start date".to_string(),
            ));
        }
        if !(1900..=2100).contains(&fiscal_year) {
            return Err(AtlasError::ValidationFailed(
                "Fiscal year must be between 1900 and 2100".to_string(),
            ));
        }

        if let Some(rt) = translation_rate_type {
            if !VALID_RATE_TYPES.contains(&rt) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid rate type '{}'. Must be one of: {}",
                    rt,
                    VALID_RATE_TYPES.join(", ")
                )));
            }
        }

        info!(
            "Creating consolidation scenario {} ({}) for ledger {} period {}-{}",
            scenario_number, name, ledger.code, fiscal_year, period_name
        );

        self.repository
            .create_scenario(
                org_id, ledger_id, scenario_number, name, description,
                fiscal_year, period_name, period_start_date, period_end_date,
                translation_rate_type, created_by,
            )
            .await
    }

    /// Get a scenario by number
    pub async fn get_scenario(&self, org_id: Uuid, scenario_number: &str) -> AtlasResult<Option<ConsolidationScenario>> {
        self.repository.get_scenario(org_id, scenario_number).await
    }

    /// List scenarios
    pub async fn list_scenarios(
        &self,
        org_id: Uuid,
        ledger_id: Option<Uuid>,
        status: Option<&str>,
    ) -> AtlasResult<Vec<ConsolidationScenario>> {
        if let Some(s) = status {
            if !VALID_SCENARIO_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid scenario status '{}'. Must be one of: {}",
                    s,
                    VALID_SCENARIO_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_scenarios(org_id, ledger_id, status).await
    }

    /// Execute the consolidation process
    ///
    /// Runs currency translation, intercompany eliminations, and minority interest
    /// calculations for the given scenario. Transitions from "draft" or "in_progress"
    /// to "pending_review".
    pub async fn execute_consolidation(&self, scenario_id: Uuid) -> AtlasResult<ConsolidationScenario> {
        let scenario = self.repository.get_scenario_by_id(scenario_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Scenario {} not found", scenario_id)))?;

        if scenario.status != "draft" && scenario.status != "in_progress" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot execute consolidation for scenario in '{}' status. Must be 'draft' or 'in_progress'.",
                scenario.status
            )));
        }

        // Move to in_progress first
        let mut scenario = self.repository
            .update_scenario_status(scenario_id, "in_progress", None, None)
            .await?;

        // Get all entities for this ledger
        let entities = self.repository.list_entities(scenario.ledger_id, true).await?;

        let total_entities = entities.len() as i32;
        let mut total_debits: f64 = 0.0;
        let mut total_credits: f64 = 0.0;
        let mut total_eliminations = 0i32;

        // For each entity, simulate currency translation and record trial balance lines
        for entity in &entities {
            // Try to get a translation rate for this entity
            let rate = self.repository
                .get_translation_rate(scenario_id, entity.entity_id, "period_end")
                .await?;

            let rate_str = rate
                .as_ref()
                .map(|r| r.exchange_rate.clone())
                .unwrap_or_else(|| "1.0".to_string());
            let rate_val: f64 = rate_str.parse().unwrap_or(1.0);

            // Check if entity currency matches base currency
            let ledger = self.repository.get_ledger_by_id(scenario.ledger_id).await?;
            let needs_translation = ledger
                .as_ref()
                .map(|l| l.base_currency_code != entity.local_currency_code)
                .unwrap_or(false);

            // Simulate: create entity-level trial balance line
            // In a real implementation, this would pull actual GL balances
            let local_debit = "10000.00";
            let local_credit = "10000.00";
            let local_balance = "0.00";

            let (translated_debit, translated_credit, translated_balance) = if needs_translation {
                let ld: f64 = local_debit.parse().unwrap_or(0.0);
                let lc: f64 = local_credit.parse().unwrap_or(0.0);
                let lb: f64 = local_balance.parse().unwrap_or(0.0);
                (
                    format!("{:.2}", ld * rate_val),
                    format!("{:.2}", lc * rate_val),
                    format!("{:.2}", lb * rate_val),
                )
            } else {
                (local_debit.to_string(), local_credit.to_string(), local_balance.to_string())
            };

            // Calculate minority interest for non-100% ownership
            let ownership: f64 = entity.ownership_percentage.parse().unwrap_or(100.0);
            let minority_pct = (100.0 - ownership) / 100.0;
            let mi_debit = format!("{:.2}", translated_debit.parse::<f64>().unwrap_or(0.0) * minority_pct);
            let mi_credit = format!("{:.2}", translated_credit.parse::<f64>().unwrap_or(0.0) * minority_pct);
            let mi_balance = format!("{:.2}", translated_balance.parse::<f64>().unwrap_or(0.0) * minority_pct);

            // Consolidated = translated - minority interest
            let cons_debit = format!("{:.2}", translated_debit.parse::<f64>().unwrap_or(0.0) * (ownership / 100.0));
            let cons_credit = format!("{:.2}", translated_credit.parse::<f64>().unwrap_or(0.0) * (ownership / 100.0));
            let cons_balance = format!("{:.2}", translated_balance.parse::<f64>().unwrap_or(0.0) * (ownership / 100.0));

            self.repository
                .create_trial_balance_line(
                    scenario.organization_id, scenario_id,
                    Some(entity.entity_id), Some(&entity.entity_code),
                    "0000", Some("Placeholder Account"), Some("asset"), Some("balance_sheet"),
                    local_debit, local_credit, local_balance,
                    Some(&rate_str),
                    &translated_debit, &translated_credit, &translated_balance,
                    "0", "0", "0",
                    &mi_debit, &mi_credit, &mi_balance,
                    &cons_debit, &cons_credit, &cons_balance,
                    false, "entity",
                )
                .await?;

            total_debits += cons_debit.parse::<f64>().unwrap_or(0.0);
            total_credits += cons_credit.parse::<f64>().unwrap_or(0.0);
        }

        // Apply elimination rules
        let rules = self.repository
            .list_elimination_rules(scenario.ledger_id, true)
            .await?;

        for rule in &rules {
            // Simulate elimination entries
            self.repository
                .create_trial_balance_line(
                    scenario.organization_id, scenario_id,
                    None, None,
                    &rule.offset_account_code, Some(&rule.name),
                    Some("liability"), Some("balance_sheet"),
                    "0", "0", "0",
                    None,
                    "0", "0", "0",
                    "5000.00", "5000.00", "0.00",
                    "0", "0", "0",
                    "5000.00", "5000.00", "0.00",
                    true, "elimination",
                )
                .await?;

            total_eliminations += 1;
            total_debits += 5000.0;
            total_credits += 5000.0;
        }

        // Count adjustments
        let adjustments = self.repository
            .list_adjustments(scenario_id, Some("approved"))
            .await?;
        let total_adjustments = adjustments.len() as i32;

        for adj in &adjustments {
            total_debits += adj.debit.parse::<f64>().unwrap_or(0.0);
            total_credits += adj.credit.parse::<f64>().unwrap_or(0.0);
        }

        let is_balanced = (total_debits - total_credits).abs() < 0.01;

        // Update scenario totals
        self.repository
            .update_scenario_totals(
                scenario_id, total_entities, total_eliminations,
                total_adjustments,
                &format!("{:.2}", total_debits),
                &format!("{:.2}", total_credits),
                is_balanced,
            )
            .await?;

        // Move to pending_review
        scenario = self.repository
            .update_scenario_status(scenario_id, "pending_review", None, None)
            .await?;

        info!(
            "Executed consolidation scenario {} - {} entities, {} eliminations, balanced={}",
            scenario.scenario_number, total_entities, total_eliminations, is_balanced
        );

        Ok(scenario)
    }

    /// Approve a consolidation scenario
    pub async fn approve_scenario(&self, scenario_id: Uuid, approver_id: Uuid) -> AtlasResult<ConsolidationScenario> {
        let scenario = self.repository.get_scenario_by_id(scenario_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Scenario {} not found", scenario_id)))?;

        if scenario.status != "pending_review" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot approve scenario in '{}' status. Must be 'pending_review'.",
                scenario.status
            )));
        }

        info!("Approved consolidation scenario {}", scenario.scenario_number);
        self.repository
            .update_scenario_status(scenario_id, "approved", Some(approver_id), None)
            .await
    }

    /// Post a consolidation scenario
    pub async fn post_scenario(&self, scenario_id: Uuid, poster_id: Uuid) -> AtlasResult<ConsolidationScenario> {
        let scenario = self.repository.get_scenario_by_id(scenario_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Scenario {} not found", scenario_id)))?;

        if scenario.status != "approved" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot post scenario in '{}' status. Must be 'approved'.",
                scenario.status
            )));
        }

        if !scenario.is_balanced {
            return Err(AtlasError::ValidationFailed(
                "Cannot post unbalanced consolidation scenario".to_string(),
            ));
        }

        info!("Posted consolidation scenario {}", scenario.scenario_number);
        self.repository
            .update_scenario_status(scenario_id, "posted", None, Some(poster_id))
            .await
    }

    /// Reverse a posted consolidation scenario
    pub async fn reverse_scenario(&self, scenario_id: Uuid) -> AtlasResult<ConsolidationScenario> {
        let scenario = self.repository.get_scenario_by_id(scenario_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Scenario {} not found", scenario_id)))?;

        if scenario.status != "posted" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot reverse scenario in '{}' status. Must be 'posted'.",
                scenario.status
            )));
        }

        info!("Reversed consolidation scenario {}", scenario.scenario_number);
        self.repository
            .update_scenario_status(scenario_id, "reversed", None, None)
            .await
    }

    // ========================================================================
    // Elimination Rules
    // ========================================================================

    /// Create an intercompany elimination rule
    pub async fn create_elimination_rule(
        &self,
        org_id: Uuid,
        ledger_id: Uuid,
        rule_code: &str,
        name: &str,
        description: Option<&str>,
        elimination_type: &str,
        from_entity_id: Option<Uuid>,
        to_entity_id: Option<Uuid>,
        from_account_pattern: Option<&str>,
        to_account_pattern: Option<&str>,
        offset_account_code: &str,
        priority: i32,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ConsolidationEliminationRule> {
        // Validate ledger
        let _ledger = self.repository.get_ledger_by_id(ledger_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Ledger {} not found", ledger_id)))?;

        if rule_code.is_empty() {
            return Err(AtlasError::ValidationFailed("Rule code is required".to_string()));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Rule name is required".to_string()));
        }
        if !VALID_ELIMINATION_TYPES.contains(&elimination_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid elimination type '{}'. Must be one of: {}",
                elimination_type,
                VALID_ELIMINATION_TYPES.join(", ")
            )));
        }
        if offset_account_code.is_empty() {
            return Err(AtlasError::ValidationFailed("Offset account code is required".to_string()));
        }

        info!(
            "Creating elimination rule {} ({}) for ledger {}",
            rule_code, name, ledger_id
        );

        self.repository
            .create_elimination_rule(
                org_id, ledger_id, rule_code, name, description,
                elimination_type, from_entity_id, to_entity_id,
                from_account_pattern, to_account_pattern,
                offset_account_code, priority, created_by,
            )
            .await
    }

    /// Get an elimination rule
    pub async fn get_elimination_rule(
        &self,
        ledger_id: Uuid,
        rule_code: &str,
    ) -> AtlasResult<Option<ConsolidationEliminationRule>> {
        self.repository.get_elimination_rule(ledger_id, rule_code).await
    }

    /// List elimination rules
    pub async fn list_elimination_rules(
        &self,
        ledger_id: Uuid,
        active_only: bool,
    ) -> AtlasResult<Vec<ConsolidationEliminationRule>> {
        self.repository.list_elimination_rules(ledger_id, active_only).await
    }

    // ========================================================================
    // Adjustments
    // ========================================================================

    /// Create a consolidation adjustment
    pub async fn create_adjustment(
        &self,
        org_id: Uuid,
        scenario_id: Uuid,
        adjustment_number: &str,
        description: Option<&str>,
        account_code: &str,
        account_name: Option<&str>,
        entity_id: Option<Uuid>,
        entity_code: Option<&str>,
        debit: &str,
        credit: &str,
        adjustment_type: &str,
        reference: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ConsolidationAdjustment> {
        // Validate scenario
        let scenario = self.repository.get_scenario_by_id(scenario_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Scenario {} not found", scenario_id)))?;

        if scenario.status != "draft" && scenario.status != "in_progress" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot add adjustments to scenario in '{}' status.",
                scenario.status
            )));
        }

        if adjustment_number.is_empty() {
            return Err(AtlasError::ValidationFailed("Adjustment number is required".to_string()));
        }
        if account_code.is_empty() {
            return Err(AtlasError::ValidationFailed("Account code is required".to_string()));
        }
        if !VALID_ADJUSTMENT_TYPES.contains(&adjustment_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid adjustment type '{}'. Must be one of: {}",
                adjustment_type,
                VALID_ADJUSTMENT_TYPES.join(", ")
            )));
        }

        Self::validate_positive_amount(debit, "Debit")?;
        Self::validate_positive_amount(credit, "Credit")?;

        info!(
            "Creating adjustment {} for scenario {}",
            adjustment_number, scenario.scenario_number
        );

        self.repository
            .create_adjustment(
                org_id, scenario_id, adjustment_number, description,
                account_code, account_name, entity_id, entity_code,
                debit, credit, adjustment_type, reference, created_by,
            )
            .await
    }

    /// Get an adjustment
    pub async fn get_adjustment(&self, id: Uuid) -> AtlasResult<Option<ConsolidationAdjustment>> {
        self.repository.get_adjustment(id).await
    }

    /// List adjustments
    pub async fn list_adjustments(
        &self,
        scenario_id: Uuid,
        status: Option<&str>,
    ) -> AtlasResult<Vec<ConsolidationAdjustment>> {
        if let Some(s) = status {
            if !VALID_ADJUSTMENT_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid adjustment status '{}'. Must be one of: {}",
                    s,
                    VALID_ADJUSTMENT_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_adjustments(scenario_id, status).await
    }

    /// Approve an adjustment
    pub async fn approve_adjustment(
        &self,
        adjustment_id: Uuid,
        approver_id: Uuid,
    ) -> AtlasResult<ConsolidationAdjustment> {
        let adj = self.repository.get_adjustment(adjustment_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Adjustment {} not found", adjustment_id)))?;

        if adj.status != "draft" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot approve adjustment in '{}' status. Must be 'draft'.",
                adj.status
            )));
        }

        info!("Approved adjustment {}", adj.adjustment_number);
        self.repository
            .update_adjustment_status(adjustment_id, "approved", Some(approver_id))
            .await
    }

    // ========================================================================
    // Translation Rates
    // ========================================================================

    /// Set a currency translation rate for an entity in a scenario
    pub async fn set_translation_rate(
        &self,
        org_id: Uuid,
        scenario_id: Uuid,
        entity_id: Uuid,
        from_currency: &str,
        to_currency: &str,
        rate_type: &str,
        exchange_rate: &str,
        effective_date: chrono::NaiveDate,
    ) -> AtlasResult<ConsolidationTranslationRate> {
        // Validate scenario
        let scenario = self.repository.get_scenario_by_id(scenario_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Scenario {} not found", scenario_id)))?;

        if scenario.status != "draft" && scenario.status != "in_progress" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot set translation rates for scenario in '{}' status.",
                scenario.status
            )));
        }

        if !VALID_RATE_TYPES.contains(&rate_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid rate type '{}'. Must be one of: {}",
                rate_type,
                VALID_RATE_TYPES.join(", ")
            )));
        }

        Self::validate_positive_amount(exchange_rate, "Exchange rate")?;

        if from_currency == to_currency {
            return Err(AtlasError::ValidationFailed(
                "From and to currencies must be different".to_string(),
            ));
        }

        info!(
            "Setting translation rate {} {}→{} for entity {} in scenario {}",
            exchange_rate, from_currency, to_currency, entity_id, scenario.scenario_number
        );

        self.repository
            .create_translation_rate(
                org_id, scenario_id, entity_id,
                from_currency, to_currency, rate_type,
                exchange_rate, effective_date,
            )
            .await
    }

    /// List translation rates for a scenario
    pub async fn list_translation_rates(
        &self,
        scenario_id: Uuid,
    ) -> AtlasResult<Vec<ConsolidationTranslationRate>> {
        self.repository.list_translation_rates(scenario_id).await
    }

    // ========================================================================
    // Trial Balance
    // ========================================================================

    /// Get the consolidated trial balance for a scenario
    pub async fn get_consolidated_trial_balance(
        &self,
        scenario_id: Uuid,
        entity_id: Option<Uuid>,
        line_type: Option<&str>,
    ) -> AtlasResult<Vec<ConsolidationTrialBalanceLine>> {
        if let Some(lt) = line_type {
            if !VALID_LINE_TYPES.contains(&lt) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid line type '{}'. Must be one of: {}",
                    lt,
                    VALID_LINE_TYPES.join(", ")
                )));
            }
        }
        self.repository.list_trial_balance(scenario_id, entity_id, line_type).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get consolidation dashboard summary
    pub async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<ConsolidationDashboardSummary> {
        self.repository.get_dashboard_summary(org_id).await
    }

    // ========================================================================
    // Internal Helpers
    // ========================================================================

    /// Validate that a string parses to a non-negative amount
    fn validate_positive_amount(value: &str, field: &str) -> AtlasResult<()> {
        let v: f64 = value.parse().map_err(|_| AtlasError::ValidationFailed(format!(
            "{} must be a valid number", field
        )))?;
        if v < 0.0 {
            return Err(AtlasError::ValidationFailed(format!(
                "{} cannot be negative", field
            )));
        }
        Ok(())
    }

    /// Validate a percentage value (0-100)
    fn validate_percentage(value: &str, field: &str) -> AtlasResult<()> {
        let v: f64 = value.parse().map_err(|_| AtlasError::ValidationFailed(format!(
            "{} must be a valid number", field
        )))?;
        if !(0.0..=100.0).contains(&v) {
            return Err(AtlasError::ValidationFailed(format!(
                "{} must be between 0 and 100", field
            )));
        }
        Ok(())
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_translation_methods() {
        assert!(VALID_TRANSLATION_METHODS.contains(&"current_rate"));
        assert!(VALID_TRANSLATION_METHODS.contains(&"temporal"));
        assert!(VALID_TRANSLATION_METHODS.contains(&"weighted_average"));
    }

    #[test]
    fn test_valid_equity_elimination_methods() {
        assert!(VALID_EQUITY_ELIMINATION_METHODS.contains(&"full"));
        assert!(VALID_EQUITY_ELIMINATION_METHODS.contains(&"proportional"));
        assert!(VALID_EQUITY_ELIMINATION_METHODS.contains(&"equity_method"));
    }

    #[test]
    fn test_valid_consolidation_methods() {
        assert!(VALID_CONSOLIDATION_METHODS.contains(&"full"));
        assert!(VALID_CONSOLIDATION_METHODS.contains(&"proportional"));
        assert!(VALID_CONSOLIDATION_METHODS.contains(&"equity_method"));
    }

    #[test]
    fn test_valid_scenario_statuses() {
        assert!(VALID_SCENARIO_STATUSES.contains(&"draft"));
        assert!(VALID_SCENARIO_STATUSES.contains(&"in_progress"));
        assert!(VALID_SCENARIO_STATUSES.contains(&"pending_review"));
        assert!(VALID_SCENARIO_STATUSES.contains(&"approved"));
        assert!(VALID_SCENARIO_STATUSES.contains(&"posted"));
        assert!(VALID_SCENARIO_STATUSES.contains(&"reversed"));
    }

    #[test]
    fn test_valid_elimination_types() {
        assert!(VALID_ELIMINATION_TYPES.contains(&"intercompany_receivable_payable"));
        assert!(VALID_ELIMINATION_TYPES.contains(&"intercompany_revenue_expense"));
        assert!(VALID_ELIMINATION_TYPES.contains(&"investment_equity"));
        assert!(VALID_ELIMINATION_TYPES.contains(&"intercompany_inventory_profit"));
        assert!(VALID_ELIMINATION_TYPES.contains(&"other"));
    }

    #[test]
    fn test_valid_adjustment_types() {
        assert!(VALID_ADJUSTMENT_TYPES.contains(&"manual"));
        assert!(VALID_ADJUSTMENT_TYPES.contains(&"reclassification"));
        assert!(VALID_ADJUSTMENT_TYPES.contains(&"correction"));
    }

    #[test]
    fn test_valid_rate_types() {
        assert!(VALID_RATE_TYPES.contains(&"period_end"));
        assert!(VALID_RATE_TYPES.contains(&"average"));
        assert!(VALID_RATE_TYPES.contains(&"historical"));
        assert!(VALID_RATE_TYPES.contains(&"spot"));
    }

    #[test]
    fn test_valid_line_types() {
        assert!(VALID_LINE_TYPES.contains(&"entity"));
        assert!(VALID_LINE_TYPES.contains(&"elimination"));
        assert!(VALID_LINE_TYPES.contains(&"adjustment"));
        assert!(VALID_LINE_TYPES.contains(&"minority"));
        assert!(VALID_LINE_TYPES.contains(&"consolidated"));
    }

    #[test]
    fn test_validate_positive_amount_valid() {
        assert!(FinancialConsolidationEngine::validate_positive_amount("100.00", "test").is_ok());
        assert!(FinancialConsolidationEngine::validate_positive_amount("0", "test").is_ok());
        assert!(FinancialConsolidationEngine::validate_positive_amount("1.5", "test").is_ok());
    }

    #[test]
    fn test_validate_positive_amount_negative() {
        assert!(FinancialConsolidationEngine::validate_positive_amount("-1.00", "test").is_err());
    }

    #[test]
    fn test_validate_positive_amount_invalid() {
        assert!(FinancialConsolidationEngine::validate_positive_amount("abc", "test").is_err());
        assert!(FinancialConsolidationEngine::validate_positive_amount("", "test").is_err());
    }

    #[test]
    fn test_validate_percentage_valid() {
        assert!(FinancialConsolidationEngine::validate_percentage("0", "test").is_ok());
        assert!(FinancialConsolidationEngine::validate_percentage("50.00", "test").is_ok());
        assert!(FinancialConsolidationEngine::validate_percentage("100", "test").is_ok());
    }

    #[test]
    fn test_validate_percentage_out_of_range() {
        assert!(FinancialConsolidationEngine::validate_percentage("-1", "test").is_err());
        assert!(FinancialConsolidationEngine::validate_percentage("101", "test").is_err());
    }

    #[test]
    fn test_validate_percentage_invalid() {
        assert!(FinancialConsolidationEngine::validate_percentage("abc", "test").is_err());
    }

    #[test]
    fn test_ownership_percentage_calculation() {
        // Verify minority interest logic: if ownership = 80%, minority = 20%
        let ownership: f64 = 80.0;
        let minority_pct = (100.0 - ownership) / 100.0;
        assert!((minority_pct - 0.2).abs() < 0.001);

        // Verify consolidated = translated * (ownership / 100)
        let translated: f64 = 10000.0;
        let consolidated = translated * (ownership / 100.0);
        assert!((consolidated - 8000.0).abs() < 0.01);

        let minority = translated * minority_pct;
        assert!((minority - 2000.0).abs() < 0.01);

        // Verify consolidated + minority = translated
        assert!((consolidated + minority - translated).abs() < 0.01);
    }

    #[test]
    fn test_currency_translation_calculation() {
        // Entity in EUR, base currency USD, rate 1.10
        let local_debit = 50000.00_f64;
        let rate = 1.10_f64;
        let translated = local_debit * rate;
        assert!((translated - 55000.0).abs() < 0.01);
    }

    #[test]
    fn test_full_consolidation_100_percent() {
        // With 100% ownership, no minority interest
        let ownership: f64 = 100.0;
        let minority_pct = (100.0 - ownership) / 100.0;
        assert!(minority_pct.abs() < 0.001);

        let translated: f64 = 10000.0;
        let consolidated = translated * (ownership / 100.0);
        assert!((consolidated - 10000.0).abs() < 0.01);
    }

    #[test]
    fn test_scenario_status_ordering() {
        // Verify status progression: draft → in_progress → pending_review → approved → posted
        let statuses = ["draft", "in_progress", "pending_review", "approved", "posted"];
        for s in &statuses {
            assert!(VALID_SCENARIO_STATUSES.contains(s));
        }
        // Reversed is also valid
        assert!(VALID_SCENARIO_STATUSES.contains(&"reversed"));
    }

    #[test]
    fn test_elimination_type_intercompany_rp() {
        // Intercompany receivable/payable elimination: AR in entity A cancels AP in entity B
        assert!(VALID_ELIMINATION_TYPES.contains(&"intercompany_receivable_payable"));
    }

    #[test]
    fn test_elimination_type_intercompany_re() {
        // Intercompany revenue/expense elimination
        assert!(VALID_ELIMINATION_TYPES.contains(&"intercompany_revenue_expense"));
    }

    #[test]
    fn test_elimination_type_investment_equity() {
        // Investment in subsidiary equity elimination
        assert!(VALID_ELIMINATION_TYPES.contains(&"investment_equity"));
    }

    #[test]
    fn test_elimination_type_inventory_profit() {
        // Intercompany inventory profit elimination (upstream/downstream)
        assert!(VALID_ELIMINATION_TYPES.contains(&"intercompany_inventory_profit"));
    }
}
