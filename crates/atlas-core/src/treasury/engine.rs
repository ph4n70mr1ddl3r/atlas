//! Treasury Management Engine
//!
//! Manages treasury counterparties, deal lifecycle (investments, borrowings,
//! FX deals), interest calculations, settlement processing, and maturity tracking.
//!
//! Supports deal types:
//! - Investment: money market placement, earns interest
//! - Borrowing: loan / line of credit, accrues interest expense
//! - FX Spot: immediate foreign exchange
//! - FX Forward: forward foreign exchange contract
//!
//! Deal lifecycle: draft → authorized → settled → matured
//!
//! Oracle Fusion Cloud ERP equivalent: Treasury Management

use atlas_shared::{
    TreasuryCounterparty, TreasuryDeal, TreasurySettlement, TreasuryDashboardSummary,
    AtlasError, AtlasResult,
};
use super::TreasuryRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Valid counterparty types
const VALID_COUNTERPARTY_TYPES: &[&str] = &["bank", "financial_institution", "internal"];

/// Valid deal types
const VALID_DEAL_TYPES: &[&str] = &["investment", "borrowing", "fx_spot", "fx_forward"];

/// Valid deal statuses
const VALID_DEAL_STATUSES: &[&str] = &["draft", "authorized", "settled", "matured", "cancelled"];

/// Valid interest bases
const VALID_INTEREST_BASES: &[&str] = &["actual_360", "actual_365", "30_360"];

/// Valid settlement types
const VALID_SETTLEMENT_TYPES: &[&str] = &["full", "partial", "early"];

/// Treasury Management engine
pub struct TreasuryEngine {
    repository: Arc<dyn TreasuryRepository>,
}

impl TreasuryEngine {
    pub fn new(repository: Arc<dyn TreasuryRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Counterparty Management
    // ========================================================================

    /// Create a new treasury counterparty (bank or financial institution)
    pub async fn create_counterparty(
        &self,
        org_id: Uuid,
        counterparty_code: &str,
        name: &str,
        counterparty_type: &str,
        country_code: Option<&str>,
        credit_rating: Option<&str>,
        credit_limit: Option<&str>,
        settlement_currency: Option<&str>,
        contact_name: Option<&str>,
        contact_email: Option<&str>,
        contact_phone: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<TreasuryCounterparty> {
        if counterparty_code.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Counterparty code is required".to_string(),
            ));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Counterparty name is required".to_string(),
            ));
        }
        if !VALID_COUNTERPARTY_TYPES.contains(&counterparty_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid counterparty type '{}'. Must be one of: {}",
                counterparty_type, VALID_COUNTERPARTY_TYPES.join(", ")
            )));
        }

        info!("Creating treasury counterparty {} ({}) for org {}", counterparty_code, name, org_id);

        self.repository.create_counterparty(
            org_id, counterparty_code, name, counterparty_type,
            country_code, credit_rating, credit_limit, settlement_currency,
            contact_name, contact_email, contact_phone, created_by,
        ).await
    }

    /// Get a counterparty by code
    pub async fn get_counterparty(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<TreasuryCounterparty>> {
        self.repository.get_counterparty(org_id, code).await
    }

    /// List counterparties
    pub async fn list_counterparties(&self, org_id: Uuid, active_only: bool) -> AtlasResult<Vec<TreasuryCounterparty>> {
        self.repository.list_counterparties(org_id, active_only).await
    }

    /// Delete a counterparty
    pub async fn delete_counterparty(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        self.repository.delete_counterparty(org_id, code).await
    }

    // ========================================================================
    // Deal Management
    // ========================================================================

    /// Create a new treasury deal
    pub async fn create_deal(
        &self,
        org_id: Uuid,
        deal_type: &str,
        description: Option<&str>,
        counterparty_id: Uuid,
        counterparty_name: Option<&str>,
        currency_code: &str,
        principal_amount: &str,
        interest_rate: Option<&str>,
        interest_basis: Option<&str>,
        start_date: chrono::NaiveDate,
        maturity_date: chrono::NaiveDate,
        fx_buy_currency: Option<&str>,
        fx_buy_amount: Option<&str>,
        fx_sell_currency: Option<&str>,
        fx_sell_amount: Option<&str>,
        fx_rate: Option<&str>,
        gl_account_code: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<TreasuryDeal> {
        if !VALID_DEAL_TYPES.contains(&deal_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid deal type '{}'. Must be one of: {}",
                deal_type, VALID_DEAL_TYPES.join(", ")
            )));
        }

        // Validate counterparty exists
        let cp = self.repository.get_counterparty_by_id(counterparty_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Counterparty {} not found", counterparty_id)
            ))?;

        if !cp.is_active {
            return Err(AtlasError::ValidationFailed(
                format!("Counterparty '{}' is not active", cp.name)
            ));
        }

        if start_date >= maturity_date {
            return Err(AtlasError::ValidationFailed(
                "Start date must be before maturity date".to_string(),
            ));
        }

        let term_days = (maturity_date - start_date).num_days() as i32;
        if term_days <= 0 {
            return Err(AtlasError::ValidationFailed(
                "Deal term must be positive".to_string(),
            ));
        }

        // Validate interest fields for investment/borrowing
        if deal_type == "investment" || deal_type == "borrowing" {
            if interest_rate.is_none() {
                return Err(AtlasError::ValidationFailed(
                    format!("Interest rate is required for {} deals", deal_type)
                ));
            }
            let rate: f64 = interest_rate.unwrap().parse().map_err(|_| AtlasError::ValidationFailed(
                "Interest rate must be a valid number".to_string(),
            ))?;
            if rate < 0.0 {
                return Err(AtlasError::ValidationFailed(
                    "Interest rate cannot be negative".to_string(),
                ));
            }
        }

        // Validate principal for non-FX deals
        if deal_type == "investment" || deal_type == "borrowing" {
            let principal: f64 = principal_amount.parse().map_err(|_| AtlasError::ValidationFailed(
                "Principal amount must be a valid number".to_string(),
            ))?;
            if principal <= 0.0 {
                return Err(AtlasError::ValidationFailed(
                    "Principal amount must be positive".to_string(),
                ));
            }
        }

        // Validate FX fields for FX deals
        if deal_type == "fx_spot" || deal_type == "fx_forward" {
            if fx_buy_currency.is_none() || fx_sell_currency.is_none() {
                return Err(AtlasError::ValidationFailed(
                    "FX buy and sell currencies are required for FX deals".to_string(),
                ));
            }
            if fx_buy_amount.is_none() || fx_sell_amount.is_none() {
                return Err(AtlasError::ValidationFailed(
                    "FX buy and sell amounts are required for FX deals".to_string(),
                ));
            }
            if fx_rate.is_none() {
                return Err(AtlasError::ValidationFailed(
                    "FX rate is required for FX deals".to_string(),
                ));
            }
            let rate: f64 = fx_rate.unwrap().parse().map_err(|_| AtlasError::ValidationFailed(
                "FX rate must be a valid number".to_string(),
            ))?;
            if rate <= 0.0 {
                return Err(AtlasError::ValidationFailed(
                    "FX rate must be positive".to_string(),
                ));
            }
        }

        if let Some(basis) = interest_basis {
            if !VALID_INTEREST_BASES.contains(&basis) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid interest basis '{}'. Must be one of: {}",
                    basis, VALID_INTEREST_BASES.join(", ")
                )));
            }
        }

        let deal_number = format!("TRD-{}", Uuid::new_v4().to_string()[..8].to_uppercase());
        let default_basis = interest_basis.unwrap_or("actual_360");

        info!("Creating treasury deal {} ({}) for org {}", deal_number, deal_type, org_id);

        self.repository.create_deal(
            org_id, &deal_number, deal_type, description,
            counterparty_id, counterparty_name.or(Some(cp.name.as_str())),
            currency_code, principal_amount,
            interest_rate, Some(default_basis),
            start_date, maturity_date, term_days,
            fx_buy_currency, fx_buy_amount,
            fx_sell_currency, fx_sell_amount, fx_rate,
            gl_account_code, created_by,
        ).await
    }

    /// Get a deal by ID
    pub async fn get_deal(&self, id: Uuid) -> AtlasResult<Option<TreasuryDeal>> {
        self.repository.get_deal(id).await
    }

    /// Get a deal by number
    pub async fn get_deal_by_number(&self, org_id: Uuid, deal_number: &str) -> AtlasResult<Option<TreasuryDeal>> {
        self.repository.get_deal_by_number(org_id, deal_number).await
    }

    /// List deals with optional filters
    pub async fn list_deals(
        &self,
        org_id: Uuid,
        deal_type: Option<&str>,
        status: Option<&str>,
    ) -> AtlasResult<Vec<TreasuryDeal>> {
        if let Some(dt) = deal_type {
            if !VALID_DEAL_TYPES.contains(&dt) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid deal type '{}'. Must be one of: {}", dt, VALID_DEAL_TYPES.join(", ")
                )));
            }
        }
        if let Some(s) = status {
            if !VALID_DEAL_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_DEAL_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_deals(org_id, deal_type, status).await
    }

    // ========================================================================
    // Deal Lifecycle
    // ========================================================================

    /// Authorize a draft deal
    pub async fn authorize_deal(
        &self,
        deal_id: Uuid,
        authorized_by: Option<Uuid>,
    ) -> AtlasResult<TreasuryDeal> {
        let deal = self.repository.get_deal(deal_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Deal {} not found", deal_id)
            ))?;

        if deal.status != "draft" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot authorize deal in '{}' status. Must be 'draft'.", deal.status)
            ));
        }

        // Calculate initial accrued interest
        let interest = self.calculate_interest(&deal);
        self.repository.update_deal_interest(
            deal_id,
            &format!("{:.2}", interest),
            None,
        ).await?;

        info!("Authorized treasury deal {} ({})", deal.deal_number, deal.deal_type);
        self.repository.update_deal_status(deal_id, "authorized", authorized_by, None, None).await
    }

    /// Settle a deal (mark as settled with payment)
    pub async fn settle_deal(
        &self,
        deal_id: Uuid,
        settlement_type: &str,
        payment_reference: Option<&str>,
        settled_by: Option<Uuid>,
    ) -> AtlasResult<TreasurySettlement> {
        if !VALID_SETTLEMENT_TYPES.contains(&settlement_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid settlement type '{}'. Must be one of: {}",
                settlement_type, VALID_SETTLEMENT_TYPES.join(", ")
            )));
        }

        let deal = self.repository.get_deal(deal_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Deal {} not found", deal_id)
            ))?;

        if deal.status != "authorized" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot settle deal in '{}' status. Must be 'authorized'.", deal.status)
            ));
        }

        // Calculate final interest
        let interest = self.calculate_interest(&deal);
        let principal: f64 = deal.principal_amount.parse().unwrap_or(0.0);
        let total = principal + interest;

        let settlement_number = format!("STL-{}", Uuid::new_v4().to_string()[..8].to_uppercase());

        let settlement = self.repository.create_settlement(
            deal.organization_id, deal_id, &settlement_number, settlement_type,
            chrono::Utc::now().date_naive(),
            &format!("{:.2}", principal),
            &format!("{:.2}", interest),
            &format!("{:.2}", total),
            payment_reference, settled_by,
        ).await?;

        // Update deal
        self.repository.update_deal_interest(
            deal_id, &format!("{:.2}", interest), Some(&format!("{:.2}", total)),
        ).await?;

        self.repository.update_deal_status(
            deal_id, "settled", None, Some(chrono::Utc::now()), None,
        ).await?;

        info!("Settled treasury deal {} for {:.2}", deal.deal_number, total);
        Ok(settlement)
    }

    /// Mature a deal
    pub async fn mature_deal(&self, deal_id: Uuid) -> AtlasResult<TreasuryDeal> {
        let deal = self.repository.get_deal(deal_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Deal {} not found", deal_id)
            ))?;

        if deal.status != "settled" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot mature deal in '{}' status. Must be 'settled'.", deal.status)
            ));
        }

        info!("Matured treasury deal {}", deal.deal_number);
        self.repository.update_deal_status(deal_id, "matured", None, None, Some(chrono::Utc::now())).await
    }

    /// Cancel a deal
    pub async fn cancel_deal(&self, deal_id: Uuid) -> AtlasResult<TreasuryDeal> {
        let deal = self.repository.get_deal(deal_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Deal {} not found", deal_id)
            ))?;

        if deal.status != "draft" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot cancel deal in '{}' status. Only 'draft' deals can be cancelled.", deal.status)
            ));
        }

        info!("Cancelled treasury deal {}", deal.deal_number);
        self.repository.update_deal_status(deal_id, "cancelled", None, None, None).await
    }

    // ========================================================================
    // Settlement Management
    // ========================================================================

    /// List settlements for a deal
    pub async fn list_settlements(&self, deal_id: Uuid) -> AtlasResult<Vec<TreasurySettlement>> {
        self.repository.list_settlements(deal_id).await
    }

    // ========================================================================
    // Interest Calculation
    // ========================================================================

    /// Calculate accrued interest for a deal based on its interest basis
    pub fn calculate_interest(&self, deal: &TreasuryDeal) -> f64 {
        let rate: f64 = deal.interest_rate.as_deref()
            .and_then(|r| r.parse().ok())
            .unwrap_or(0.0);
        let principal: f64 = deal.principal_amount.parse().unwrap_or(0.0);

        if rate <= 0.0 || principal <= 0.0 {
            return 0.0;
        }

        let basis_days = match deal.interest_basis.as_deref() {
            Some("actual_365") => 365.0,
            Some("30_360") => 360.0,
            _ => 360.0, // actual_360 is default
        };

        let term_days = deal.term_days as f64;
        // Simple interest: principal * rate * (term / basis_days)
        principal * rate * (term_days / basis_days)
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get treasury dashboard summary
    pub async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<TreasuryDashboardSummary> {
        self.repository.get_dashboard_summary(org_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_deal_types() {
        assert!(VALID_DEAL_TYPES.contains(&"investment"));
        assert!(VALID_DEAL_TYPES.contains(&"borrowing"));
        assert!(VALID_DEAL_TYPES.contains(&"fx_spot"));
        assert!(VALID_DEAL_TYPES.contains(&"fx_forward"));
    }

    #[test]
    fn test_valid_deal_statuses() {
        assert!(VALID_DEAL_STATUSES.contains(&"draft"));
        assert!(VALID_DEAL_STATUSES.contains(&"authorized"));
        assert!(VALID_DEAL_STATUSES.contains(&"settled"));
        assert!(VALID_DEAL_STATUSES.contains(&"matured"));
        assert!(VALID_DEAL_STATUSES.contains(&"cancelled"));
    }

    #[test]
    fn test_valid_counterparty_types() {
        assert!(VALID_COUNTERPARTY_TYPES.contains(&"bank"));
        assert!(VALID_COUNTERPARTY_TYPES.contains(&"financial_institution"));
        assert!(VALID_COUNTERPARTY_TYPES.contains(&"internal"));
    }

    #[test]
    fn test_valid_interest_bases() {
        assert!(VALID_INTEREST_BASES.contains(&"actual_360"));
        assert!(VALID_INTEREST_BASES.contains(&"actual_365"));
        assert!(VALID_INTEREST_BASES.contains(&"30_360"));
    }

    #[test]
    fn test_interest_calculation_actual_360() {
        let engine = TreasuryEngine::new(Arc::new(crate::MockTreasuryRepository));
        let deal = TreasuryDeal {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            deal_number: "TRD-TEST01".to_string(),
            deal_type: "investment".to_string(),
            description: None,
            counterparty_id: Uuid::new_v4(),
            counterparty_name: None,
            currency_code: "USD".to_string(),
            principal_amount: "1000000.00".to_string(),
            interest_rate: Some("0.05".to_string()), // 5%
            interest_basis: Some("actual_360".to_string()),
            start_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            maturity_date: chrono::NaiveDate::from_ymd_opt(2024, 4, 1).unwrap(),
            term_days: 91,
            fx_buy_currency: None, fx_buy_amount: None,
            fx_sell_currency: None, fx_sell_amount: None, fx_rate: None,
            accrued_interest: "0".to_string(),
            settlement_amount: None,
            gl_account_code: None,
            status: "draft".to_string(),
            authorized_by: None, authorized_at: None,
            settled_at: None, matured_at: None,
            metadata: serde_json::json!({}),
            created_by: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let interest = engine.calculate_interest(&deal);
        // 1,000,000 * 0.05 * (91/360) ≈ 12,638.89
        let expected = 1_000_000.0 * 0.05 * (91.0 / 360.0);
        assert!((interest - expected).abs() < 1.0, "Expected {:.2}, got {:.2}", expected, interest);
    }

    #[test]
    fn test_interest_calculation_actual_365() {
        let engine = TreasuryEngine::new(Arc::new(crate::MockTreasuryRepository));
        let deal = TreasuryDeal {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            deal_number: "TRD-TEST02".to_string(),
            deal_type: "borrowing".to_string(),
            description: None,
            counterparty_id: Uuid::new_v4(),
            counterparty_name: None,
            currency_code: "USD".to_string(),
            principal_amount: "500000.00".to_string(),
            interest_rate: Some("0.035".to_string()), // 3.5%
            interest_basis: Some("actual_365".to_string()),
            start_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            maturity_date: chrono::NaiveDate::from_ymd_opt(2024, 7, 1).unwrap(),
            term_days: 182,
            fx_buy_currency: None, fx_buy_amount: None,
            fx_sell_currency: None, fx_sell_amount: None, fx_rate: None,
            accrued_interest: "0".to_string(),
            settlement_amount: None,
            gl_account_code: None,
            status: "draft".to_string(),
            authorized_by: None, authorized_at: None,
            settled_at: None, matured_at: None,
            metadata: serde_json::json!({}),
            created_by: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let interest = engine.calculate_interest(&deal);
        // 500,000 * 0.035 * (182/365) ≈ 8,726.03
        let expected = 500_000.0 * 0.035 * (182.0 / 365.0);
        assert!((interest - expected).abs() < 1.0, "Expected {:.2}, got {:.2}", expected, interest);
    }

    #[test]
    fn test_interest_calculation_zero_rate() {
        let engine = TreasuryEngine::new(Arc::new(crate::MockTreasuryRepository));
        let deal = TreasuryDeal {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            deal_number: "TRD-TEST03".to_string(),
            deal_type: "investment".to_string(),
            description: None,
            counterparty_id: Uuid::new_v4(),
            counterparty_name: None,
            currency_code: "USD".to_string(),
            principal_amount: "1000000.00".to_string(),
            interest_rate: Some("0".to_string()),
            interest_basis: Some("actual_360".to_string()),
            start_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            maturity_date: chrono::NaiveDate::from_ymd_opt(2024, 4, 1).unwrap(),
            term_days: 91,
            fx_buy_currency: None, fx_buy_amount: None,
            fx_sell_currency: None, fx_sell_amount: None, fx_rate: None,
            accrued_interest: "0".to_string(),
            settlement_amount: None,
            gl_account_code: None,
            status: "draft".to_string(),
            authorized_by: None, authorized_at: None,
            settled_at: None, matured_at: None,
            metadata: serde_json::json!({}),
            created_by: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let interest = engine.calculate_interest(&deal);
        assert_eq!(interest, 0.0);
    }

    #[test]
    fn test_interest_calculation_30_360() {
        let engine = TreasuryEngine::new(Arc::new(crate::MockTreasuryRepository));
        let deal = TreasuryDeal {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            deal_number: "TRD-TEST04".to_string(),
            deal_type: "investment".to_string(),
            description: None,
            counterparty_id: Uuid::new_v4(),
            counterparty_name: None,
            currency_code: "USD".to_string(),
            principal_amount: "2000000.00".to_string(),
            interest_rate: Some("0.04".to_string()), // 4%
            interest_basis: Some("30_360".to_string()),
            start_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            maturity_date: chrono::NaiveDate::from_ymd_opt(2024, 7, 1).unwrap(),
            term_days: 180,
            fx_buy_currency: None, fx_buy_amount: None,
            fx_sell_currency: None, fx_sell_amount: None, fx_rate: None,
            accrued_interest: "0".to_string(),
            settlement_amount: None,
            gl_account_code: None,
            status: "draft".to_string(),
            authorized_by: None, authorized_at: None,
            settled_at: None, matured_at: None,
            metadata: serde_json::json!({}),
            created_by: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let interest = engine.calculate_interest(&deal);
        // 2,000,000 * 0.04 * (180/360) = 40,000
        let expected = 2_000_000.0 * 0.04 * (180.0 / 360.0);
        assert!((interest - expected).abs() < 1.0, "Expected {:.2}, got {:.2}", expected, interest);
    }
}
