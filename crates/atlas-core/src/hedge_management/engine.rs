//! Hedge Management Engine
//!
//! Manages derivative instrument lifecycle, hedge relationship designation,
//! effectiveness testing (IFRS 9 / ASC 815), and hedge documentation.
//!
//! Oracle Fusion Cloud ERP equivalent: Treasury > Hedge Management

use atlas_shared::{AtlasError, AtlasResult};
use super::repository::{
    HedgeManagementRepository, DerivativeCreateParams, HedgeRelationshipCreateParams,
    EffectivenessTestCreateParams, DocumentationCreateParams,
};
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

// Valid instrument types
const VALID_INSTRUMENT_TYPES: &[&str] = &[
    "forward", "future", "swap", "option", "cap", "floor", "collar",
];

// Valid underlying types
const VALID_UNDERLYING_TYPES: &[&str] = &[
    "interest_rate", "fx", "commodity", "credit", "equity",
];

// Valid derivative statuses
const VALID_DERIVATIVE_STATUSES: &[&str] = &[
    "draft", "active", "matured", "settled", "cancelled",
];

// Valid option types
const VALID_OPTION_TYPES: &[&str] = &[
    "none", "call", "put", "straddle",
];

// Valid hedge types
const VALID_HEDGE_TYPES: &[&str] = &[
    "fair_value", "cash_flow", "net_investment",
];

// Valid hedge statuses
const VALID_HEDGE_STATUSES: &[&str] = &[
    "draft", "designated", "active", "de-designated", "terminated",
];

// Valid hedged risks
const VALID_HEDGED_RISKS: &[&str] = &[
    "interest_rate", "fx", "commodity", "credit", "equity", "other",
];

// Valid effectiveness methods
const VALID_EFFECTIVENESS_METHODS: &[&str] = &[
    "dollar_offset", "regression", "variance_reduction", "scenario",
];

// Valid effectiveness results
#[allow(dead_code)]
const VALID_EFFECTIVENESS_RESULTS: &[&str] = &[
    "effective", "ineffective", "pending",
];

// Valid test types
const VALID_TEST_TYPES: &[&str] = &[
    "prospective", "retrospective", "ongoing",
];

// Valid test statuses
#[allow(dead_code)]
const VALID_TEST_STATUSES: &[&str] = &[
    "draft", "completed", "failed",
];

// Valid documentation statuses
#[allow(dead_code)]
const VALID_DOC_STATUSES: &[&str] = &[
    "draft", "approved", "rejected",
];

/// Calculate effectiveness using the dollar-offset method.
/// Returns (ratio, is_effective) where effectiveness requires 0.80 <= ratio <= 1.25.
///
/// # Panics / Precision
///
/// Uses `f64` arithmetic which is acceptable for the ratio calculation itself.
/// Callers should pass values that are already rounded to the desired
/// precision (e.g. 2 decimal places for currency amounts).
pub fn calculate_dollar_offset_effectiveness(
    derivative_fair_value_change: f64,
    hedged_item_fair_value_change: f64,
) -> (f64, bool) {
    if hedged_item_fair_value_change == 0.0 {
        return (0.0, false);
    }
    let ratio = (derivative_fair_value_change / hedged_item_fair_value_change).abs();
    // Round to 6 decimal places to avoid floating-point noise at the boundaries
    let ratio = (ratio * 1_000_000.0).round() / 1_000_000.0;
    let is_effective = (0.80..=1.25).contains(&ratio);
    (ratio, is_effective)
}

/// Hedge Management engine
pub struct HedgeManagementEngine {
    repository: Arc<dyn HedgeManagementRepository>,
}

impl HedgeManagementEngine {
    pub fn new(repository: Arc<dyn HedgeManagementRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Derivative Instruments
    // ========================================================================

    /// Create a new derivative instrument
    pub async fn create_derivative(
        &self,
        org_id: Uuid,
        instrument_type: &str,
        underlying_type: &str,
        underlying_description: Option<&str>,
        currency_code: &str,
        counter_currency_code: Option<&str>,
        notional_amount: &str,
        strike_rate: Option<&str>,
        forward_rate: Option<&str>,
        spot_rate: Option<&str>,
        option_type: Option<&str>,
        premium_amount: Option<&str>,
        trade_date: Option<chrono::NaiveDate>,
        effective_date: Option<chrono::NaiveDate>,
        maturity_date: Option<chrono::NaiveDate>,
        settlement_date: Option<chrono::NaiveDate>,
        settlement_type: Option<&str>,
        counterparty_name: Option<&str>,
        counterparty_reference: Option<&str>,
        portfolio_code: Option<&str>,
        trading_book: Option<&str>,
        accounting_treatment: Option<&str>,
        risk_factor: Option<&str>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::DerivativeInstrument> {
        // Validation
        if !VALID_INSTRUMENT_TYPES.contains(&instrument_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid instrument type '{}'. Must be one of: {}",
                instrument_type,
                VALID_INSTRUMENT_TYPES.join(", ")
            )));
        }
        if !VALID_UNDERLYING_TYPES.contains(&underlying_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid underlying type '{}'. Must be one of: {}",
                underlying_type,
                VALID_UNDERLYING_TYPES.join(", ")
            )));
        }
        if let Some(ot) = option_type {
            if !VALID_OPTION_TYPES.contains(&ot) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid option type '{}'. Must be one of: {}",
                    ot,
                    VALID_OPTION_TYPES.join(", ")
                )));
            }
        }

        let notional: f64 = notional_amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "Notional amount must be a valid number".to_string(),
        ))?;
        if notional <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Notional amount must be greater than zero".to_string(),
            ));
        }

        // Generate instrument number
        let next_num = self.repository.get_latest_derivative_number(org_id).await? + 1;
        let instrument_number = format!("DERIV-{:04}", next_num);

        info!("Creating derivative instrument {} for org {}", instrument_number, org_id);

        self.repository.create_derivative(&DerivativeCreateParams {
            org_id,
            instrument_number,
            instrument_type: instrument_type.to_string(),
            underlying_type: underlying_type.to_string(),
            underlying_description: underlying_description.map(|s| s.to_string()),
            currency_code: currency_code.to_string(),
            counter_currency_code: counter_currency_code.map(|s| s.to_string()),
            notional_amount: notional_amount.to_string(),
            strike_rate: strike_rate.map(|s| s.to_string()),
            forward_rate: forward_rate.map(|s| s.to_string()),
            spot_rate: spot_rate.map(|s| s.to_string()),
            option_type: option_type.map(|s| s.to_string()),
            premium_amount: premium_amount.map(|s| s.to_string()),
            trade_date,
            effective_date,
            maturity_date,
            settlement_date,
            settlement_type: settlement_type.map(|s| s.to_string()),
            counterparty_name: counterparty_name.map(|s| s.to_string()),
            counterparty_reference: counterparty_reference.map(|s| s.to_string()),
            portfolio_code: portfolio_code.map(|s| s.to_string()),
            trading_book: trading_book.map(|s| s.to_string()),
            accounting_treatment: accounting_treatment.map(|s| s.to_string()),
            risk_factor: risk_factor.map(|s| s.to_string()),
            notes: notes.map(|s| s.to_string()),
            created_by,
        }).await
    }

    /// Get a derivative by instrument number
    pub async fn get_derivative(&self, org_id: Uuid, instrument_number: &str) -> AtlasResult<Option<atlas_shared::DerivativeInstrument>> {
        self.repository.get_derivative(org_id, instrument_number).await
    }

    /// Get a derivative by ID
    pub async fn get_derivative_by_id(&self, id: Uuid) -> AtlasResult<Option<atlas_shared::DerivativeInstrument>> {
        self.repository.get_derivative_by_id(id).await
    }

    /// List derivatives with optional filters
    pub async fn list_derivatives(&self, org_id: Uuid, status: Option<&str>, instrument_type: Option<&str>) -> AtlasResult<Vec<atlas_shared::DerivativeInstrument>> {
        if let Some(s) = status {
            if !VALID_DERIVATIVE_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}",
                    s, VALID_DERIVATIVE_STATUSES.join(", ")
                )));
            }
        }
        if let Some(t) = instrument_type {
            if !VALID_INSTRUMENT_TYPES.contains(&t) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid instrument type '{}'. Must be one of: {}",
                    t, VALID_INSTRUMENT_TYPES.join(", ")
                )));
            }
        }
        self.repository.list_derivatives(org_id, status, instrument_type).await
    }

    /// Activate a derivative
    pub async fn activate_derivative(&self, id: Uuid) -> AtlasResult<atlas_shared::DerivativeInstrument> {
        let deriv = self.repository.get_derivative_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Derivative {} not found", id)))?;

        if deriv.status != "draft" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot activate derivative in '{}' status. Must be 'draft'.",
                deriv.status
            )));
        }

        info!("Activated derivative instrument {}", deriv.instrument_number);
        self.repository.update_derivative_status(id, "active").await
    }

    /// Mature a derivative
    pub async fn mature_derivative(&self, id: Uuid) -> AtlasResult<atlas_shared::DerivativeInstrument> {
        let deriv = self.repository.get_derivative_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Derivative {} not found", id)))?;

        if deriv.status != "active" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot mature derivative in '{}' status. Must be 'active'.",
                deriv.status
            )));
        }

        info!("Matured derivative instrument {}", deriv.instrument_number);
        self.repository.update_derivative_status(id, "matured").await
    }

    /// Settle a derivative
    pub async fn settle_derivative(&self, id: Uuid) -> AtlasResult<atlas_shared::DerivativeInstrument> {
        let deriv = self.repository.get_derivative_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Derivative {} not found", id)))?;

        if deriv.status != "matured" && deriv.status != "active" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot settle derivative in '{}' status. Must be 'active' or 'matured'.",
                deriv.status
            )));
        }

        info!("Settled derivative instrument {}", deriv.instrument_number);
        self.repository.update_derivative_status(id, "settled").await
    }

    /// Cancel a derivative
    pub async fn cancel_derivative(&self, id: Uuid) -> AtlasResult<atlas_shared::DerivativeInstrument> {
        let deriv = self.repository.get_derivative_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Derivative {} not found", id)))?;

        if deriv.status != "draft" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot cancel derivative in '{}' status. Must be 'draft'.",
                deriv.status
            )));
        }

        info!("Cancelled derivative instrument {}", deriv.instrument_number);
        self.repository.update_derivative_status(id, "cancelled").await
    }

    /// Update derivative valuation (mark-to-market)
    pub async fn update_derivative_valuation(
        &self,
        id: Uuid,
        fair_value: &str,
        unrealized_gain_loss: &str,
        valuation_method: Option<&str>,
    ) -> AtlasResult<atlas_shared::DerivativeInstrument> {
        let deriv = self.repository.get_derivative_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Derivative {} not found", id)))?;

        if deriv.status != "active" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot value derivative in '{}' status. Must be 'active'.",
                deriv.status
            )));
        }

        let _fv: f64 = fair_value.parse().map_err(|_| AtlasError::ValidationFailed(
            "Fair value must be a valid number".to_string(),
        ))?;
        let _ugl: f64 = unrealized_gain_loss.parse().map_err(|_| AtlasError::ValidationFailed(
            "Unrealized gain/loss must be a valid number".to_string(),
        ))?;

        info!("Updated valuation for derivative {} (FV={}, UGL={})", deriv.instrument_number, fair_value, unrealized_gain_loss);
        self.repository.update_derivative_valuation(
            id, fair_value, unrealized_gain_loss, valuation_method, Some(chrono::Utc::now().date_naive()),
        ).await
    }

    /// Delete a derivative (only if draft or cancelled)
    pub async fn delete_derivative(&self, org_id: Uuid, instrument_number: &str) -> AtlasResult<()> {
        let deriv = self.repository.get_derivative(org_id, instrument_number).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Derivative {} not found", instrument_number)))?;

        if deriv.status != "draft" && deriv.status != "cancelled" {
            return Err(AtlasError::WorkflowError(
                "Cannot delete derivative that is not in 'draft' or 'cancelled' status".to_string(),
            ));
        }

        info!("Deleted derivative instrument {}", instrument_number);
        self.repository.delete_derivative(org_id, instrument_number).await
    }

    // ========================================================================
    // Hedge Relationships
    // ========================================================================

    /// Create a hedge relationship
    pub async fn create_hedge_relationship(
        &self,
        org_id: Uuid,
        hedge_type: &str,
        derivative_id: Option<Uuid>,
        derivative_number: Option<&str>,
        hedged_item_description: Option<&str>,
        hedged_item_id: Option<Uuid>,
        hedged_risk: &str,
        hedge_strategy: Option<&str>,
        hedged_item_reference: Option<&str>,
        hedged_item_currency: Option<&str>,
        hedged_amount: &str,
        hedge_ratio: Option<&str>,
        designated_start_date: Option<chrono::NaiveDate>,
        designated_end_date: Option<chrono::NaiveDate>,
        effectiveness_method: &str,
        critical_terms_match: Option<&str>,
        hedge_documentation_ref: Option<&str>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::HedgeRelationship> {
        if !VALID_HEDGE_TYPES.contains(&hedge_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid hedge type '{}'. Must be one of: {}",
                hedge_type, VALID_HEDGE_TYPES.join(", ")
            )));
        }
        if !VALID_HEDGED_RISKS.contains(&hedged_risk) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid hedged risk '{}'. Must be one of: {}",
                hedged_risk, VALID_HEDGED_RISKS.join(", ")
            )));
        }
        if !VALID_EFFECTIVENESS_METHODS.contains(&effectiveness_method) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid effectiveness method '{}'. Must be one of: {}",
                effectiveness_method, VALID_EFFECTIVENESS_METHODS.join(", ")
            )));
        }

        let hedged_amt: f64 = hedged_amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "Hedged amount must be a valid number".to_string(),
        ))?;
        if hedged_amt <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Hedged amount must be greater than zero".to_string(),
            ));
        }

        // Validate derivative exists if specified
        if let Some(did) = derivative_id {
            let deriv = self.repository.get_derivative_by_id(did).await?;
            if deriv.is_none() {
                return Err(AtlasError::EntityNotFound(format!("Derivative {} not found", did)));
            }
        }

        // Generate hedge ID
        let next_num = self.repository.get_latest_hedge_number(org_id).await? + 1;
        let hedge_id = format!("HEDGE-{:04}", next_num);

        info!("Creating hedge relationship {} for org {}", hedge_id, org_id);

        self.repository.create_hedge_relationship(&HedgeRelationshipCreateParams {
            org_id,
            hedge_id,
            hedge_type: hedge_type.to_string(),
            derivative_id,
            derivative_number: derivative_number.map(|s| s.to_string()),
            hedged_item_description: hedged_item_description.map(|s| s.to_string()),
            hedged_item_id,
            hedged_risk: hedged_risk.to_string(),
            hedge_strategy: hedge_strategy.map(|s| s.to_string()),
            hedged_item_reference: hedged_item_reference.map(|s| s.to_string()),
            hedged_item_currency: hedged_item_currency.map(|s| s.to_string()),
            hedged_amount: hedged_amount.to_string(),
            hedge_ratio: hedge_ratio.map(|s| s.to_string()),
            designated_start_date,
            designated_end_date,
            effectiveness_method: effectiveness_method.to_string(),
            critical_terms_match: critical_terms_match.map(|s| s.to_string()),
            hedge_documentation_ref: hedge_documentation_ref.map(|s| s.to_string()),
            notes: notes.map(|s| s.to_string()),
            created_by,
        }).await
    }

    /// Get a hedge relationship by hedge_id
    pub async fn get_hedge_relationship(&self, org_id: Uuid, hedge_id: &str) -> AtlasResult<Option<atlas_shared::HedgeRelationship>> {
        self.repository.get_hedge_relationship(org_id, hedge_id).await
    }

    /// Get a hedge relationship by ID
    pub async fn get_hedge_relationship_by_id(&self, id: Uuid) -> AtlasResult<Option<atlas_shared::HedgeRelationship>> {
        self.repository.get_hedge_relationship_by_id(id).await
    }

    /// List hedge relationships with optional filters
    pub async fn list_hedge_relationships(&self, org_id: Uuid, status: Option<&str>, hedge_type: Option<&str>) -> AtlasResult<Vec<atlas_shared::HedgeRelationship>> {
        if let Some(s) = status {
            if !VALID_HEDGE_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}",
                    s, VALID_HEDGE_STATUSES.join(", ")
                )));
            }
        }
        if let Some(t) = hedge_type {
            if !VALID_HEDGE_TYPES.contains(&t) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid hedge type '{}'. Must be one of: {}",
                    t, VALID_HEDGE_TYPES.join(", ")
                )));
            }
        }
        self.repository.list_hedge_relationships(org_id, status, hedge_type).await
    }

    /// Designate a hedge (move from draft to designated)
    pub async fn designate_hedge(&self, id: Uuid) -> AtlasResult<atlas_shared::HedgeRelationship> {
        let hedge = self.repository.get_hedge_relationship_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Hedge relationship {} not found", id)))?;

        if hedge.status != "draft" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot designate hedge in '{}' status. Must be 'draft'.",
                hedge.status
            )));
        }

        info!("Designated hedge relationship {}", hedge.hedge_id);
        self.repository.update_hedge_relationship_status(id, "designated").await
    }

    /// Activate a hedge relationship
    pub async fn activate_hedge(&self, id: Uuid) -> AtlasResult<atlas_shared::HedgeRelationship> {
        let hedge = self.repository.get_hedge_relationship_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Hedge relationship {} not found", id)))?;

        if hedge.status != "designated" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot activate hedge in '{}' status. Must be 'designated'.",
                hedge.status
            )));
        }

        info!("Activated hedge relationship {}", hedge.hedge_id);
        self.repository.update_hedge_relationship_status(id, "active").await
    }

    /// De-designate a hedge relationship
    pub async fn de_designate_hedge(&self, id: Uuid) -> AtlasResult<atlas_shared::HedgeRelationship> {
        let hedge = self.repository.get_hedge_relationship_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Hedge relationship {} not found", id)))?;

        if hedge.status != "active" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot de-designate hedge in '{}' status. Must be 'active'.",
                hedge.status
            )));
        }

        info!("De-designated hedge relationship {}", hedge.hedge_id);
        self.repository.update_hedge_relationship_status(id, "de-designated").await
    }

    /// Terminate a hedge relationship
    pub async fn terminate_hedge(&self, id: Uuid) -> AtlasResult<atlas_shared::HedgeRelationship> {
        let hedge = self.repository.get_hedge_relationship_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Hedge relationship {} not found", id)))?;

        if hedge.status != "active" && hedge.status != "de-designated" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot terminate hedge in '{}' status.",
                hedge.status
            )));
        }

        info!("Terminated hedge relationship {}", hedge.hedge_id);
        self.repository.update_hedge_relationship_status(id, "terminated").await
    }

    /// Delete a hedge relationship (only if draft)
    pub async fn delete_hedge_relationship(&self, org_id: Uuid, hedge_id: &str) -> AtlasResult<()> {
        let hedge = self.repository.get_hedge_relationship(org_id, hedge_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Hedge {} not found", hedge_id)))?;

        if hedge.status != "draft" && hedge.status != "terminated" {
            return Err(AtlasError::WorkflowError(
                "Cannot delete hedge that is not in 'draft' or 'terminated' status".to_string(),
            ));
        }

        info!("Deleted hedge relationship {}", hedge_id);
        self.repository.delete_hedge_relationship(org_id, hedge_id).await
    }

    // ========================================================================
    // Effectiveness Testing
    // ========================================================================

    /// Run an effectiveness test on a hedge relationship
    pub async fn run_effectiveness_test(
        &self,
        org_id: Uuid,
        hedge_relationship_id: Uuid,
        test_type: &str,
        test_date: chrono::NaiveDate,
        derivative_fair_value_change: &str,
        hedged_item_fair_value_change: &str,
        test_period_start: Option<chrono::NaiveDate>,
        test_period_end: Option<chrono::NaiveDate>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::HedgeEffectivenessTest> {
        if !VALID_TEST_TYPES.contains(&test_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid test type '{}'. Must be one of: {}",
                test_type, VALID_TEST_TYPES.join(", ")
            )));
        }

        let hedge = self.repository.get_hedge_relationship_by_id(hedge_relationship_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Hedge relationship {} not found", hedge_relationship_id
            )))?;

        if hedge.status != "active" && hedge.status != "designated" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot test effectiveness for hedge in '{}' status.",
                hedge.status
            )));
        }

        let deriv_change: f64 = derivative_fair_value_change.parse().map_err(|_| AtlasError::ValidationFailed(
            "Derivative fair value change must be a valid number".to_string(),
        ))?;
        let hedged_change: f64 = hedged_item_fair_value_change.parse().map_err(|_| AtlasError::ValidationFailed(
            "Hedged item fair value change must be a valid number".to_string(),
        ))?;

        // Calculate effectiveness using dollar-offset method
        let (ratio, is_effective) = calculate_dollar_offset_effectiveness(deriv_change, hedged_change);
        let effectiveness_result = if is_effective { "effective" } else { "ineffective" };
        let ineffective_amount = if is_effective { "0.00" } else {
            &format!("{:.2}", (deriv_change - hedged_change).abs())
        };

        info!(
            "Effectiveness test for hedge {}: ratio={:.4}, result={}",
            hedge.hedge_id, ratio, effectiveness_result
        );

        // Create test record
        let test = self.repository.create_effectiveness_test(&EffectivenessTestCreateParams {
            org_id,
            hedge_relationship_id,
            hedge_id: Some(hedge.hedge_id.clone()),
            test_type: test_type.to_string(),
            effectiveness_method: hedge.effectiveness_method.clone(),
            test_date,
            test_period_start,
            test_period_end,
            derivative_fair_value_change: Some(derivative_fair_value_change.to_string()),
            hedged_item_fair_value_change: Some(hedged_item_fair_value_change.to_string()),
            hedge_ratio_result: Some(format!("{:.4}", ratio)),
            ratio_lower_bound: Some("0.80".to_string()),
            ratio_upper_bound: Some("1.25".to_string()),
            effectiveness_result: effectiveness_result.to_string(),
            ineffective_amount: Some(ineffective_amount.to_string()),
            cumulative_gain_loss: None,
            regression_r_squared: None,
            notes: notes.map(|s| s.to_string()),
            created_by,
        }).await?;

        // Update hedge relationship with effectiveness result
        self.repository.update_hedge_effectiveness(
            hedge_relationship_id, test_date, effectiveness_result,
        ).await?;

        // Mark test as completed
        let test = self.repository.update_effectiveness_test_status(test.id, "completed").await?;

        Ok(test)
    }

    /// Get an effectiveness test by ID
    pub async fn get_effectiveness_test(&self, id: Uuid) -> AtlasResult<Option<atlas_shared::HedgeEffectivenessTest>> {
        self.repository.get_effectiveness_test(id).await
    }

    /// List effectiveness tests for a hedge relationship
    pub async fn list_effectiveness_tests(&self, hedge_relationship_id: Uuid) -> AtlasResult<Vec<atlas_shared::HedgeEffectivenessTest>> {
        self.repository.list_effectiveness_tests(hedge_relationship_id).await
    }

    // ========================================================================
    // Documentation
    // ========================================================================

    /// Create hedge documentation
    pub async fn create_documentation(
        &self,
        org_id: Uuid,
        hedge_relationship_id: Option<Uuid>,
        hedge_id: Option<&str>,
        hedge_type: &str,
        risk_management_objective: Option<&str>,
        hedging_strategy_description: Option<&str>,
        hedged_item_description: Option<&str>,
        hedged_risk_description: Option<&str>,
        derivative_description: Option<&str>,
        effectiveness_method_description: Option<&str>,
        assessment_frequency: Option<&str>,
        designation_date: Option<chrono::NaiveDate>,
        documentation_date: Option<chrono::NaiveDate>,
        prepared_by: Option<&str>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::HedgeDocumentation> {
        if !VALID_HEDGE_TYPES.contains(&hedge_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid hedge type '{}'. Must be one of: {}",
                hedge_type, VALID_HEDGE_TYPES.join(", ")
            )));
        }

        // Generate document number
        let next_num = self.repository.get_latest_test_number(org_id).await? + 1;
        let document_number = format!("HDOC-{:04}", next_num);

        info!("Creating hedge documentation {} for org {}", document_number, org_id);

        self.repository.create_hedge_documentation(&DocumentationCreateParams {
            org_id,
            hedge_relationship_id,
            hedge_id: hedge_id.map(|s| s.to_string()),
            document_number,
            hedge_type: hedge_type.to_string(),
            risk_management_objective: risk_management_objective.map(|s| s.to_string()),
            hedging_strategy_description: hedging_strategy_description.map(|s| s.to_string()),
            hedged_item_description: hedged_item_description.map(|s| s.to_string()),
            hedged_risk_description: hedged_risk_description.map(|s| s.to_string()),
            derivative_description: derivative_description.map(|s| s.to_string()),
            effectiveness_method_description: effectiveness_method_description.map(|s| s.to_string()),
            assessment_frequency: assessment_frequency.map(|s| s.to_string()),
            designation_date,
            documentation_date,
            prepared_by: prepared_by.map(|s| s.to_string()),
            notes: notes.map(|s| s.to_string()),
            created_by,
        }).await
    }

    /// Get documentation by document number
    pub async fn get_documentation(&self, org_id: Uuid, document_number: &str) -> AtlasResult<Option<atlas_shared::HedgeDocumentation>> {
        self.repository.get_hedge_documentation(org_id, document_number).await
    }

    /// List documentation with optional hedge relationship filter
    pub async fn list_documentation(&self, org_id: Uuid, hedge_relationship_id: Option<Uuid>) -> AtlasResult<Vec<atlas_shared::HedgeDocumentation>> {
        self.repository.list_hedge_documentation(org_id, hedge_relationship_id).await
    }

    /// Approve hedge documentation
    pub async fn approve_documentation(&self, id: Uuid, approved_by: Option<Uuid>) -> AtlasResult<atlas_shared::HedgeDocumentation> {
        let doc = self.repository.get_hedge_documentation_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Documentation {} not found", id)))?;

        if doc.status != "draft" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot approve documentation in '{}' status. Must be 'draft'.",
                doc.status
            )));
        }

        info!("Approved hedge documentation {}", doc.document_number);
        self.repository.update_documentation_status(id, "approved", approved_by).await
    }

    /// Reject hedge documentation
    pub async fn reject_documentation(&self, id: Uuid) -> AtlasResult<atlas_shared::HedgeDocumentation> {
        // Use a simpler approach - try update and let DB handle it
        info!("Rejecting hedge documentation {}", id);
        self.repository.update_documentation_status(id, "rejected", None).await
    }

    /// Delete documentation (only if draft or rejected)
    pub async fn delete_documentation(&self, org_id: Uuid, document_number: &str) -> AtlasResult<()> {
        let doc = self.repository.get_hedge_documentation(org_id, document_number).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Documentation {} not found", document_number)))?;

        if doc.status != "draft" && doc.status != "rejected" {
            return Err(AtlasError::WorkflowError(
                "Cannot delete documentation that is not in 'draft' or 'rejected' status".to_string(),
            ));
        }

        info!("Deleted hedge documentation {}", document_number);
        self.repository.delete_documentation(org_id, document_number).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get hedge management dashboard summary
    pub async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<atlas_shared::HedgeDashboard> {
        self.repository.get_dashboard_summary(org_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_instrument_types() {
        assert!(VALID_INSTRUMENT_TYPES.contains(&"forward"));
        assert!(VALID_INSTRUMENT_TYPES.contains(&"future"));
        assert!(VALID_INSTRUMENT_TYPES.contains(&"swap"));
        assert!(VALID_INSTRUMENT_TYPES.contains(&"option"));
        assert!(VALID_INSTRUMENT_TYPES.contains(&"cap"));
        assert!(VALID_INSTRUMENT_TYPES.contains(&"floor"));
        assert!(VALID_INSTRUMENT_TYPES.contains(&"collar"));
    }

    #[test]
    fn test_valid_hedge_types() {
        assert!(VALID_HEDGE_TYPES.contains(&"fair_value"));
        assert!(VALID_HEDGE_TYPES.contains(&"cash_flow"));
        assert!(VALID_HEDGE_TYPES.contains(&"net_investment"));
    }

    #[test]
    fn test_valid_effectiveness_methods() {
        assert!(VALID_EFFECTIVENESS_METHODS.contains(&"dollar_offset"));
        assert!(VALID_EFFECTIVENESS_METHODS.contains(&"regression"));
        assert!(VALID_EFFECTIVENESS_METHODS.contains(&"variance_reduction"));
        assert!(VALID_EFFECTIVENESS_METHODS.contains(&"scenario"));
    }

    #[test]
    fn test_dollar_offset_effective() {
        // 1:1 ratio - perfectly effective
        let (ratio, effective) = calculate_dollar_offset_effectiveness(10000.0, 10000.0);
        assert!(effective);
        assert!((ratio - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_dollar_offset_effective_lower_bound() {
        // Ratio of 0.85 - still effective
        let (ratio, effective) = calculate_dollar_offset_effectiveness(8500.0, 10000.0);
        assert!(effective);
        assert!((ratio - 0.85).abs() < 0.001);
    }

    #[test]
    fn test_dollar_offset_ineffective_low() {
        // Ratio of 0.50 - not effective
        let (ratio, effective) = calculate_dollar_offset_effectiveness(5000.0, 10000.0);
        assert!(!effective);
        assert!((ratio - 0.50).abs() < 0.001);
    }

    #[test]
    fn test_dollar_offset_ineffective_high() {
        // Ratio of 1.50 - not effective
        let (ratio, effective) = calculate_dollar_offset_effectiveness(15000.0, 10000.0);
        assert!(!effective);
        assert!((ratio - 1.50).abs() < 0.001);
    }

    #[test]
    fn test_dollar_offset_zero_hedged() {
        let (ratio, effective) = calculate_dollar_offset_effectiveness(10000.0, 0.0);
        assert!(!effective);
        assert_eq!(ratio, 0.0);
    }

    #[test]
    fn test_dollar_offset_negative_changes() {
        // Negative changes should still work (absolute value of ratio)
        let (ratio, effective) = calculate_dollar_offset_effectiveness(-10000.0, -10500.0);
        assert!(effective); // ratio = 10000/10500 ≈ 0.952
        assert!(ratio > 0.90 && ratio < 1.00);
    }

    #[test]
    fn test_valid_derivative_statuses() {
        assert!(VALID_DERIVATIVE_STATUSES.contains(&"draft"));
        assert!(VALID_DERIVATIVE_STATUSES.contains(&"active"));
        assert!(VALID_DERIVATIVE_STATUSES.contains(&"matured"));
        assert!(VALID_DERIVATIVE_STATUSES.contains(&"settled"));
        assert!(VALID_DERIVATIVE_STATUSES.contains(&"cancelled"));
    }

    #[test]
    fn test_valid_hedge_statuses() {
        assert!(VALID_HEDGE_STATUSES.contains(&"draft"));
        assert!(VALID_HEDGE_STATUSES.contains(&"designated"));
        assert!(VALID_HEDGE_STATUSES.contains(&"active"));
        assert!(VALID_HEDGE_STATUSES.contains(&"de-designated"));
        assert!(VALID_HEDGE_STATUSES.contains(&"terminated"));
    }

    #[test]
    fn test_valid_underlying_types() {
        assert!(VALID_UNDERLYING_TYPES.contains(&"interest_rate"));
        assert!(VALID_UNDERLYING_TYPES.contains(&"fx"));
        assert!(VALID_UNDERLYING_TYPES.contains(&"commodity"));
        assert!(VALID_UNDERLYING_TYPES.contains(&"credit"));
        assert!(VALID_UNDERLYING_TYPES.contains(&"equity"));
    }

    #[test]
    fn test_valid_test_types() {
        assert!(VALID_TEST_TYPES.contains(&"prospective"));
        assert!(VALID_TEST_TYPES.contains(&"retrospective"));
        assert!(VALID_TEST_TYPES.contains(&"ongoing"));
    }
}
