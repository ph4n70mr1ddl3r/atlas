//! Rebate Management Engine
//!
//! Manages rebate agreements, tiers, transactions, accruals, and settlements
//! for both supplier (payable) and customer (receivable) rebate programs.
//!
//! Oracle Fusion Cloud equivalent: Trade Management > Rebates

use atlas_shared::{
    RebateAgreement, RebateTier, RebateTransaction, RebateAccrual,
    RebateSettlement, RebateSettlementLine, RebateDashboard,
    AtlasError, AtlasResult,
};
use super::RebateManagementRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

// ============================================================================
// Valid enum constants
// ============================================================================

#[allow(dead_code)]
const VALID_REBATE_TYPES: &[&str] = &["supplier_rebate", "customer_rebate"];

#[allow(dead_code)]
const VALID_DIRECTIONS: &[&str] = &["receivable", "payable"];

#[allow(dead_code)]
const VALID_PARTNER_TYPES: &[&str] = &["supplier", "customer"];

#[allow(dead_code)]
const VALID_AGREEMENT_STATUSES: &[&str] = &[
    "draft", "active", "on_hold", "expired", "terminated",
];

#[allow(dead_code)]
const VALID_CALC_METHODS: &[&str] = &[
    "flat_rate", "tiered", "cumulative",
];

#[allow(dead_code)]
const VALID_SETTLEMENT_FREQUENCIES: &[&str] = &[
    "monthly", "quarterly", "annually", "at_end",
];

#[allow(dead_code)]
const VALID_RATE_TYPES: &[&str] = &[
    "percentage", "fixed_per_unit", "fixed_amount",
];

#[allow(dead_code)]
const VALID_TXN_STATUSES: &[&str] = &[
    "eligible", "accrued", "settled", "excluded", "disputed",
];

#[allow(dead_code)]
const VALID_SOURCE_TYPES: &[&str] = &[
    "sales_order", "purchase_order", "invoice", "manual",
];

#[allow(dead_code)]
const VALID_ACCRUAL_STATUSES: &[&str] = &[
    "draft", "posted", "reversed", "settled",
];

#[allow(dead_code)]
const VALID_SETTLEMENT_STATUSES: &[&str] = &[
    "pending", "approved", "paid", "cancelled", "disputed",
];

#[allow(dead_code)]
const VALID_SETTLEMENT_TYPES: &[&str] = &[
    "payment", "credit_memo", "offset",
];

#[allow(dead_code)]
const VALID_PAYMENT_METHODS: &[&str] = &[
    "check", "wire", "ach", "credit_note",
];

/// Helper to validate a value against allowed set
fn validate_enum(field: &str, value: &str, allowed: &[&str]) -> AtlasResult<()> {
    if value.is_empty() {
        return Err(AtlasError::ValidationFailed(format!(
            "{} is required", field
        )));
    }
    if !allowed.contains(&value) {
        return Err(AtlasError::ValidationFailed(format!(
            "Invalid {} '{}'. Must be one of: {}", field, value, allowed.join(", ")
        )));
    }
    Ok(())
}

/// Rebate Management Engine
pub struct RebateManagementEngine {
    repository: Arc<dyn RebateManagementRepository>,
}

impl RebateManagementEngine {
    pub fn new(repository: Arc<dyn RebateManagementRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Rebate Agreements
    // ========================================================================

    /// Create a new rebate agreement
    #[allow(clippy::too_many_arguments)]
    pub async fn create_agreement(
        &self,
        org_id: Uuid,
        agreement_number: &str,
        name: &str,
        description: Option<&str>,
        rebate_type: &str,
        direction: &str,
        partner_type: &str,
        partner_id: Option<Uuid>,
        partner_name: Option<&str>,
        partner_number: Option<&str>,
        product_category: Option<&str>,
        product_id: Option<Uuid>,
        product_name: Option<&str>,
        uom: Option<&str>,
        currency_code: &str,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
        calculation_method: &str,
        accrual_account: Option<&str>,
        liability_account: Option<&str>,
        expense_account: Option<&str>,
        payment_terms: Option<&str>,
        settlement_frequency: Option<&str>,
        minimum_amount: Option<f64>,
        maximum_amount: Option<f64>,
        auto_accrue: Option<bool>,
        requires_approval: Option<bool>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<RebateAgreement> {
        if agreement_number.is_empty() {
            return Err(AtlasError::ValidationFailed("Agreement number is required".to_string()));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Agreement name is required".to_string()));
        }
        validate_enum("rebate_type", rebate_type, VALID_REBATE_TYPES)?;
        validate_enum("direction", direction, VALID_DIRECTIONS)?;
        validate_enum("partner_type", partner_type, VALID_PARTNER_TYPES)?;
        validate_enum("calculation_method", calculation_method, VALID_CALC_METHODS)?;
        if let Some(sf) = settlement_frequency {
            validate_enum("settlement_frequency", sf, VALID_SETTLEMENT_FREQUENCIES)?;
        }
        if start_date >= end_date {
            return Err(AtlasError::ValidationFailed(
                "Start date must be before end date".to_string(),
            ));
        }
        if currency_code.is_empty() {
            return Err(AtlasError::ValidationFailed("Currency code is required".to_string()));
        }

        if self.repository.get_agreement_by_number(org_id, agreement_number).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Rebate agreement '{}' already exists", agreement_number
            )));
        }

        info!("Creating rebate agreement '{}' ({}) for org {} [type={}, direction={}]",
              agreement_number, name, org_id, rebate_type, direction);

        self.repository.create_agreement(
            org_id, agreement_number, name, description,
            rebate_type, direction, partner_type,
            partner_id, partner_name, partner_number,
            product_category, product_id, product_name,
            uom, currency_code, start_date, end_date,
            calculation_method,
            accrual_account, liability_account, expense_account,
            payment_terms, settlement_frequency,
            minimum_amount.unwrap_or(0.0),
            maximum_amount,
            auto_accrue.unwrap_or(true),
            requires_approval.unwrap_or(true),
            notes,
            created_by,
        ).await
    }

    /// Get an agreement by ID
    pub async fn get_agreement(&self, id: Uuid) -> AtlasResult<Option<RebateAgreement>> {
        self.repository.get_agreement(id).await
    }

    /// Get an agreement by number
    pub async fn get_agreement_by_number(&self, org_id: Uuid, agreement_number: &str) -> AtlasResult<Option<RebateAgreement>> {
        self.repository.get_agreement_by_number(org_id, agreement_number).await
    }

    /// List agreements with optional filters
    pub async fn list_agreements(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        rebate_type: Option<&str>,
        partner_type: Option<&str>,
    ) -> AtlasResult<Vec<RebateAgreement>> {
        self.repository.list_agreements(org_id, status, rebate_type, partner_type).await
    }

    /// Activate a rebate agreement
    pub async fn activate_agreement(&self, id: Uuid) -> AtlasResult<RebateAgreement> {
        info!("Activating rebate agreement {}", id);
        self.repository.update_agreement_status(id, "active").await
    }

    /// Put agreement on hold
    pub async fn hold_agreement(&self, id: Uuid) -> AtlasResult<RebateAgreement> {
        info!("Putting rebate agreement {} on hold", id);
        self.repository.update_agreement_status(id, "on_hold").await
    }

    /// Terminate an agreement
    pub async fn terminate_agreement(&self, id: Uuid) -> AtlasResult<RebateAgreement> {
        info!("Terminating rebate agreement {}", id);
        self.repository.update_agreement_status(id, "terminated").await
    }

    /// Delete an agreement by number (only drafts)
    pub async fn delete_agreement(&self, org_id: Uuid, agreement_number: &str) -> AtlasResult<()> {
        info!("Deleting rebate agreement '{}' for org {}", agreement_number, org_id);
        self.repository.delete_agreement(org_id, agreement_number).await
    }

    // ========================================================================
    // Rebate Tiers
    // ========================================================================

    /// Add a tier to a rebate agreement
    pub async fn create_tier(
        &self,
        org_id: Uuid,
        agreement_id: Uuid,
        tier_number: i32,
        from_value: f64,
        to_value: Option<f64>,
        rebate_rate: f64,
        rate_type: &str,
        description: Option<&str>,
    ) -> AtlasResult<RebateTier> {
        validate_enum("rate_type", rate_type, VALID_RATE_TYPES)?;

        if rebate_rate < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Rebate rate cannot be negative".to_string(),
            ));
        }
        if from_value < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "From value cannot be negative".to_string(),
            ));
        }
        if let Some(tv) = to_value {
            if tv <= from_value {
                return Err(AtlasError::ValidationFailed(
                    "To value must be greater than from value".to_string(),
                ));
            }
        }

        // Verify agreement exists
        self.repository.get_agreement(agreement_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Rebate agreement {} not found", agreement_id
            )))?;

        info!("Creating tier {} for agreement {} [from={}, to={:?}, rate={}, type={}]",
              tier_number, agreement_id, from_value, to_value, rebate_rate, rate_type);

        self.repository.create_tier(
            org_id, agreement_id, tier_number,
            from_value, to_value, rebate_rate, rate_type, description,
        ).await
    }

    /// List tiers for an agreement
    pub async fn list_tiers(&self, agreement_id: Uuid) -> AtlasResult<Vec<RebateTier>> {
        self.repository.list_tiers(agreement_id).await
    }

    /// Delete a tier
    pub async fn delete_tier(&self, id: Uuid) -> AtlasResult<()> {
        self.repository.delete_tier(id).await
    }

    // ========================================================================
    // Rebate Transactions
    // ========================================================================

    /// Record a qualifying transaction against a rebate agreement
    #[allow(clippy::too_many_arguments)]
    pub async fn create_transaction(
        &self,
        org_id: Uuid,
        agreement_id: Uuid,
        transaction_number: &str,
        source_type: Option<&str>,
        source_id: Option<Uuid>,
        source_number: Option<&str>,
        transaction_date: chrono::NaiveDate,
        product_id: Option<Uuid>,
        product_name: Option<&str>,
        quantity: f64,
        unit_price: f64,
        transaction_amount: f64,
        currency_code: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<RebateTransaction> {
        if transaction_number.is_empty() {
            return Err(AtlasError::ValidationFailed("Transaction number is required".to_string()));
        }
        if let Some(st) = source_type {
            validate_enum("source_type", st, VALID_SOURCE_TYPES)?;
        }
        if transaction_amount < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Transaction amount cannot be negative".to_string(),
            ));
        }

        // Verify agreement exists and is active
        let agreement = self.repository.get_agreement(agreement_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Rebate agreement {} not found", agreement_id
            )))?;

        if agreement.status != "active" {
            return Err(AtlasError::ValidationFailed(format!(
                "Agreement is not active (status: {})", agreement.status
            )));
        }

        if self.repository.get_transaction_by_number(org_id, transaction_number).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Rebate transaction '{}' already exists", transaction_number
            )));
        }

        // Calculate applicable rate and rebate amount based on tiers
        let (applicable_rate, tier_id, rebate_amount) = self.calculate_rebate(
            agreement_id, transaction_amount, quantity, &agreement.calculation_method,
        ).await?;

        info!("Creating rebate transaction '{}' for agreement {} [amount={}, rate={:.4}, rebate={:.2}]",
              transaction_number, agreement_id, transaction_amount, applicable_rate, rebate_amount);

        self.repository.create_transaction(
            org_id, agreement_id, transaction_number,
            source_type, source_id, source_number,
            transaction_date, product_id, product_name,
            quantity, unit_price, transaction_amount,
            currency_code.unwrap_or(&agreement.currency_code),
            applicable_rate, rebate_amount,
            tier_id, created_by,
        ).await
    }

    /// Get a transaction by ID
    pub async fn get_transaction(&self, id: Uuid) -> AtlasResult<Option<RebateTransaction>> {
        self.repository.get_transaction(id).await
    }

    /// List transactions for an agreement
    pub async fn list_transactions(
        &self,
        agreement_id: Uuid,
        status: Option<&str>,
    ) -> AtlasResult<Vec<RebateTransaction>> {
        self.repository.list_transactions(agreement_id, status).await
    }

    /// Update transaction status
    pub async fn update_transaction_status(&self, id: Uuid, status: &str, reason: Option<&str>) -> AtlasResult<RebateTransaction> {
        validate_enum("transaction status", status, VALID_TXN_STATUSES)?;
        info!("Updating rebate transaction {} status to {}", id, status);
        self.repository.update_transaction_status(id, status, reason).await
    }

    /// Delete a transaction by number
    pub async fn delete_transaction(&self, org_id: Uuid, transaction_number: &str) -> AtlasResult<()> {
        info!("Deleting rebate transaction '{}' for org {}", transaction_number, org_id);
        self.repository.delete_transaction(org_id, transaction_number).await
    }

    // ========================================================================
    // Rebate Accruals
    // ========================================================================

    /// Create a rebate accrual
    #[allow(clippy::too_many_arguments)]
    pub async fn create_accrual(
        &self,
        org_id: Uuid,
        agreement_id: Uuid,
        accrual_number: &str,
        accrual_date: chrono::NaiveDate,
        accrual_period: Option<&str>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<RebateAccrual> {
        if accrual_number.is_empty() {
            return Err(AtlasError::ValidationFailed("Accrual number is required".to_string()));
        }

        // Verify agreement exists
        let agreement = self.repository.get_agreement(agreement_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Rebate agreement {} not found", agreement_id
            )))?;

        // Sum up eligible transactions for this agreement
        let transactions = self.repository.list_transactions(agreement_id, Some("eligible")).await?;
        let accumulated_amount: f64 = transactions.iter().map(|t| t.transaction_amount).sum();
        let accumulated_quantity: f64 = transactions.iter().map(|t| t.quantity).sum();

        // Calculate applicable rate
        let (applicable_rate, tier_id, accrued_amount) = self.calculate_rebate(
            agreement_id, accumulated_amount, accumulated_quantity, &agreement.calculation_method,
        ).await?;

        if self.repository.get_accrual_by_number(org_id, accrual_number).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Rebate accrual '{}' already exists", accrual_number
            )));
        }

        info!("Creating rebate accrual '{}' for agreement {} [accumulated={}, accrued={:.2}]",
              accrual_number, agreement_id, accumulated_amount, accrued_amount);

        let accrual = self.repository.create_accrual(
            org_id, agreement_id, accrual_number,
            accrual_date, accrual_period,
            accumulated_quantity, accumulated_amount,
            tier_id, applicable_rate, accrued_amount,
            &agreement.currency_code,
            notes, created_by,
        ).await?;

        // Mark transactions as accrued
        for txn in &transactions {
            self.repository.update_transaction_status(txn.id, "accrued", None).await.ok();
        }

        Ok(accrual)
    }

    /// Get an accrual by ID
    pub async fn get_accrual(&self, id: Uuid) -> AtlasResult<Option<RebateAccrual>> {
        self.repository.get_accrual(id).await
    }

    /// List accruals for an agreement
    pub async fn list_accruals(
        &self,
        agreement_id: Uuid,
        status: Option<&str>,
    ) -> AtlasResult<Vec<RebateAccrual>> {
        self.repository.list_accruals(agreement_id, status).await
    }

    /// Post an accrual to GL
    pub async fn post_accrual(&self, id: Uuid) -> AtlasResult<RebateAccrual> {
        info!("Posting rebate accrual {}", id);
        self.repository.update_accrual_status(id, "posted").await
    }

    /// Reverse an accrual
    pub async fn reverse_accrual(&self, id: Uuid) -> AtlasResult<RebateAccrual> {
        info!("Reversing rebate accrual {}", id);
        self.repository.update_accrual_status(id, "reversed").await
    }

    /// Delete an accrual by number
    pub async fn delete_accrual(&self, org_id: Uuid, accrual_number: &str) -> AtlasResult<()> {
        info!("Deleting rebate accrual '{}' for org {}", accrual_number, org_id);
        self.repository.delete_accrual(org_id, accrual_number).await
    }

    // ========================================================================
    // Rebate Settlements
    // ========================================================================

    /// Create a rebate settlement
    #[allow(clippy::too_many_arguments)]
    pub async fn create_settlement(
        &self,
        org_id: Uuid,
        agreement_id: Uuid,
        settlement_number: &str,
        settlement_date: chrono::NaiveDate,
        settlement_period_from: Option<chrono::NaiveDate>,
        settlement_period_to: Option<chrono::NaiveDate>,
        settlement_type: &str,
        payment_method: Option<&str>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<RebateSettlement> {
        if settlement_number.is_empty() {
            return Err(AtlasError::ValidationFailed("Settlement number is required".to_string()));
        }
        validate_enum("settlement_type", settlement_type, VALID_SETTLEMENT_TYPES)?;
        if let Some(pm) = payment_method {
            validate_enum("payment_method", pm, VALID_PAYMENT_METHODS)?;
        }

        // Verify agreement exists
        let agreement = self.repository.get_agreement(agreement_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Rebate agreement {} not found", agreement_id
            )))?;

        // Sum up accrued transactions
        let transactions = self.repository.list_transactions(agreement_id, Some("accrued")).await?;
        let total_qualifying_amount: f64 = transactions.iter().map(|t| t.transaction_amount).sum();
        let total_qualifying_quantity: f64 = transactions.iter().map(|t| t.quantity).sum();

        let (applicable_rate, tier_id, settlement_amount) = self.calculate_rebate(
            agreement_id, total_qualifying_amount, total_qualifying_quantity, &agreement.calculation_method,
        ).await?;

        if self.repository.get_settlement_by_number(org_id, settlement_number).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Rebate settlement '{}' already exists", settlement_number
            )));
        }

        info!("Creating rebate settlement '{}' for agreement {} [qualifying={}, settlement={:.2}]",
              settlement_number, agreement_id, total_qualifying_amount, settlement_amount);

        let settlement = self.repository.create_settlement(
            org_id, agreement_id, settlement_number,
            settlement_date, settlement_period_from, settlement_period_to,
            total_qualifying_amount, total_qualifying_quantity,
            tier_id, applicable_rate, settlement_amount,
            &agreement.currency_code,
            settlement_type, payment_method,
            notes, created_by,
        ).await?;

        // Mark transactions as settled and create settlement lines
        for txn in &transactions {
            self.repository.update_transaction_status(txn.id, "settled", None).await.ok();
            self.repository.create_settlement_line(settlement.id, txn.id, txn.rebate_amount).await.ok();
        }

        Ok(settlement)
    }

    /// Get a settlement by ID
    pub async fn get_settlement(&self, id: Uuid) -> AtlasResult<Option<RebateSettlement>> {
        self.repository.get_settlement(id).await
    }

    /// List settlements for an agreement
    pub async fn list_settlements(
        &self,
        agreement_id: Uuid,
        status: Option<&str>,
    ) -> AtlasResult<Vec<RebateSettlement>> {
        self.repository.list_settlements(agreement_id, status).await
    }

    /// Approve a settlement
    pub async fn approve_settlement(&self, id: Uuid, approved_by: Uuid) -> AtlasResult<RebateSettlement> {
        info!("Approving rebate settlement {} by {}", id, approved_by);
        self.repository.approve_settlement(id, approved_by).await
    }

    /// Cancel a settlement
    pub async fn cancel_settlement(&self, id: Uuid) -> AtlasResult<RebateSettlement> {
        info!("Cancelling rebate settlement {}", id);
        self.repository.update_settlement_status(id, "cancelled").await
    }

    /// Mark a settlement as paid
    pub async fn pay_settlement(&self, id: Uuid) -> AtlasResult<RebateSettlement> {
        info!("Recording payment for rebate settlement {}", id);
        self.repository.pay_settlement(id).await
    }

    /// Delete a settlement by number
    pub async fn delete_settlement(&self, org_id: Uuid, settlement_number: &str) -> AtlasResult<()> {
        info!("Deleting rebate settlement '{}' for org {}", settlement_number, org_id);
        self.repository.delete_settlement(org_id, settlement_number).await
    }

    // ========================================================================
    // Settlement Lines
    // ========================================================================

    /// List settlement lines
    pub async fn list_settlement_lines(&self, settlement_id: Uuid) -> AtlasResult<Vec<RebateSettlementLine>> {
        self.repository.list_settlement_lines(settlement_id).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get the rebate management dashboard
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<RebateDashboard> {
        self.repository.get_dashboard(org_id).await
    }

    // ========================================================================
    // Internal helpers
    // ========================================================================

    /// Calculate the applicable rebate rate and amount based on tiers
    async fn calculate_rebate(
        &self,
        agreement_id: Uuid,
        amount: f64,
        quantity: f64,
        calc_method: &str,
    ) -> AtlasResult<(f64, Option<Uuid>, f64)> {
        let tiers = self.repository.list_tiers(agreement_id).await?;

        if tiers.is_empty() {
            // No tiers defined — zero rebate
            return Ok((0.0, None, 0.0));
        }

        // Determine the basis for tier evaluation
        let basis = match calc_method {
            "cumulative" => amount,
            _ => amount, // Default: use amount for tier matching
        };

        // Find the applicable tier
        let applicable_tier = tiers.iter().find(|t| {
            let above_from = basis >= t.from_value;
            let below_to = t.to_value.is_none() || basis < t.to_value.unwrap();
            above_from && below_to
        });

        match applicable_tier {
            Some(tier) => {
                let rebate_amount = match tier.rate_type.as_str() {
                    "percentage" => amount * (tier.rebate_rate / 100.0),
                    "fixed_per_unit" => quantity * tier.rebate_rate,
                    "fixed_amount" => tier.rebate_rate,
                    _ => 0.0,
                };
                Ok((tier.rebate_rate, Some(tier.id), rebate_amount))
            }
            None => Ok((0.0, None, 0.0)),
        }
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_rebate_types() {
        assert!(VALID_REBATE_TYPES.contains(&"supplier_rebate"));
        assert!(VALID_REBATE_TYPES.contains(&"customer_rebate"));
        assert!(!VALID_REBATE_TYPES.contains(&"partner_rebate"));
    }

    #[test]
    fn test_valid_directions() {
        assert!(VALID_DIRECTIONS.contains(&"receivable"));
        assert!(VALID_DIRECTIONS.contains(&"payable"));
        assert!(!VALID_DIRECTIONS.contains(&"neutral"));
    }

    #[test]
    fn test_valid_partner_types() {
        assert!(VALID_PARTNER_TYPES.contains(&"supplier"));
        assert!(VALID_PARTNER_TYPES.contains(&"customer"));
    }

    #[test]
    fn test_valid_agreement_statuses() {
        assert!(VALID_AGREEMENT_STATUSES.contains(&"draft"));
        assert!(VALID_AGREEMENT_STATUSES.contains(&"active"));
        assert!(VALID_AGREEMENT_STATUSES.contains(&"on_hold"));
        assert!(VALID_AGREEMENT_STATUSES.contains(&"expired"));
        assert!(VALID_AGREEMENT_STATUSES.contains(&"terminated"));
    }

    #[test]
    fn test_valid_calc_methods() {
        assert!(VALID_CALC_METHODS.contains(&"flat_rate"));
        assert!(VALID_CALC_METHODS.contains(&"tiered"));
        assert!(VALID_CALC_METHODS.contains(&"cumulative"));
    }

    #[test]
    fn test_valid_rate_types() {
        assert!(VALID_RATE_TYPES.contains(&"percentage"));
        assert!(VALID_RATE_TYPES.contains(&"fixed_per_unit"));
        assert!(VALID_RATE_TYPES.contains(&"fixed_amount"));
    }

    #[test]
    fn test_valid_settlement_types() {
        assert!(VALID_SETTLEMENT_TYPES.contains(&"payment"));
        assert!(VALID_SETTLEMENT_TYPES.contains(&"credit_memo"));
        assert!(VALID_SETTLEMENT_TYPES.contains(&"offset"));
    }

    #[test]
    fn test_validate_enum_valid() {
        assert!(validate_enum("rebate_type", "supplier_rebate", VALID_REBATE_TYPES).is_ok());
    }

    #[test]
    fn test_validate_enum_invalid() {
        let result = validate_enum("rebate_type", "invalid", VALID_REBATE_TYPES);
        assert!(result.is_err());
        match result {
            Err(AtlasError::ValidationFailed(msg)) => {
                assert!(msg.contains("rebate_type"));
                assert!(msg.contains("invalid"));
            }
            _ => panic!("Expected ValidationFailed error"),
        }
    }

    #[test]
    fn test_validate_enum_empty() {
        let result = validate_enum("rebate_type", "", VALID_REBATE_TYPES);
        assert!(result.is_err());
        match result {
            Err(AtlasError::ValidationFailed(msg)) => {
                assert!(msg.contains("required"));
            }
            _ => panic!("Expected ValidationFailed error"),
        }
    }
}
