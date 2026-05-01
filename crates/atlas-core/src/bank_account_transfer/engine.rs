//! Bank Account Transfer Engine
//!
//! Core bank transfer operations:
//! - Transfer type management
//! - Transfer creation with multi-currency support
//! - Approval workflow (draft -> submitted -> approved -> in_transit -> completed)
//! - Cancellation and reversal
//! - Dashboard summary
//!
//! Oracle Fusion Cloud ERP equivalent: Financials > Cash Management > Bank Transfers

use atlas_shared::{
    BankTransferType, BankAccountTransfer, BankTransferDashboardSummary,
    AtlasError, AtlasResult,
};
use super::BankAccountTransferRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Valid settlement methods
#[allow(dead_code)]
const VALID_SETTLEMENT_METHODS: &[&str] = &["immediate", "scheduled", "batch"];

/// Valid transfer statuses
#[allow(dead_code)]
const VALID_TRANSFER_STATUSES: &[&str] = &[
    "draft", "submitted", "approved", "in_transit", "completed",
    "cancelled", "reversed", "failed",
];

/// Valid priorities
#[allow(dead_code)]
const VALID_PRIORITIES: &[&str] = &["low", "normal", "high", "urgent"];

/// Bank Account Transfer Engine
pub struct BankAccountTransferEngine {
    repository: Arc<dyn BankAccountTransferRepository>,
}

impl BankAccountTransferEngine {
    pub fn new(repository: Arc<dyn BankAccountTransferRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Transfer Type Management
    // ========================================================================

    /// Create a transfer type
    pub async fn create_transfer_type(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        settlement_method: &str,
        requires_approval: bool,
        approval_threshold: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<BankTransferType> {
        if code.is_empty() || name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Transfer type code and name are required".to_string(),
            ));
        }
        if !VALID_SETTLEMENT_METHODS.contains(&settlement_method) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid settlement_method '{}'. Must be one of: {}", settlement_method, VALID_SETTLEMENT_METHODS.join(", ")
            )));
        }
        if let Some(threshold) = approval_threshold {
            let t: f64 = threshold.parse().map_err(|_| AtlasError::ValidationFailed(
                "Approval threshold must be a valid number".to_string(),
            ))?;
            if t < 0.0 {
                return Err(AtlasError::ValidationFailed(
                    "Approval threshold must be non-negative".to_string(),
                ));
            }
        }

        info!("Creating bank transfer type '{}'", code);
        self.repository.create_transfer_type(
            org_id, code, name, description, settlement_method,
            requires_approval, approval_threshold, created_by,
        ).await
    }

    /// List transfer types
    pub async fn list_transfer_types(&self, org_id: Uuid) -> AtlasResult<Vec<BankTransferType>> {
        self.repository.list_transfer_types(org_id).await
    }

    // ========================================================================
    // Transfer Management
    // ========================================================================

    /// Create a bank account transfer
    pub async fn create_transfer(
        &self,
        org_id: Uuid,
        transfer_type_id: Option<Uuid>,
        from_bank_account_id: Uuid,
        from_bank_account_number: Option<&str>,
        from_bank_name: Option<&str>,
        to_bank_account_id: Uuid,
        to_bank_account_number: Option<&str>,
        to_bank_name: Option<&str>,
        amount: &str,
        currency_code: &str,
        exchange_rate: Option<&str>,
        from_currency: Option<&str>,
        to_currency: Option<&str>,
        transfer_date: chrono::NaiveDate,
        value_date: Option<chrono::NaiveDate>,
        reference_number: Option<&str>,
        description: Option<&str>,
        purpose: Option<&str>,
        priority: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<BankAccountTransfer> {
        if from_bank_account_id == to_bank_account_id {
            return Err(AtlasError::ValidationFailed(
                "Source and destination bank accounts must be different".to_string(),
            ));
        }

        let amt: f64 = amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "Amount must be a valid number".to_string(),
        ))?;
        if amt <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Transfer amount must be positive".to_string(),
            ));
        }

        if !VALID_PRIORITIES.contains(&priority) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid priority '{}'. Must be one of: {}", priority, VALID_PRIORITIES.join(", ")
            )));
        }

        let transfer_number = format!("TRF-{}", Uuid::new_v4().to_string()[..8].to_uppercase());

        // Calculate transferred amount if cross-currency
        let transferred_amount = if let (Some(rate), Some(_to_curr)) = (exchange_rate, to_currency) {
            let rate: f64 = rate.parse().unwrap_or(1.0);
            Some(format!("{:.2}", amt * rate))
        } else {
            None
        };

        info!("Creating bank transfer {} for {:.2} {}", transfer_number, amt, currency_code);

        self.repository.create_transfer(
            org_id, &transfer_number, transfer_type_id,
            from_bank_account_id, from_bank_account_number, from_bank_name,
            to_bank_account_id, to_bank_account_number, to_bank_name,
            amount, currency_code, exchange_rate, from_currency, to_currency,
            transferred_amount.as_deref(),
            transfer_date, value_date, None, // settlement_date
            reference_number, description, purpose,
            "draft", priority, created_by,
        ).await
    }

    /// Get transfer by ID
    pub async fn get_transfer(&self, id: Uuid) -> AtlasResult<Option<BankAccountTransfer>> {
        self.repository.get_transfer(id).await
    }

    /// List transfers
    pub async fn list_transfers(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<BankAccountTransfer>> {
        if let Some(s) = status {
            if !VALID_TRANSFER_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_TRANSFER_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_transfers(org_id, status).await
    }

    // ========================================================================
    // Transfer Workflow
    // ========================================================================

    /// Submit a draft transfer
    pub async fn submit_transfer(&self, transfer_id: Uuid, submitted_by: Option<Uuid>) -> AtlasResult<BankAccountTransfer> {
        let transfer = self.repository.get_transfer(transfer_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Transfer {} not found", transfer_id)))?;

        if transfer.status != "draft" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot submit transfer in '{}' status. Must be 'draft'.", transfer.status)
            ));
        }

        info!("Submitting bank transfer {}", transfer.transfer_number);
        self.repository.update_transfer_status(
            transfer_id, "submitted", submitted_by, None, None, None, None,
        ).await
    }

    /// Approve a submitted transfer
    pub async fn approve_transfer(&self, transfer_id: Uuid, approved_by: Option<Uuid>) -> AtlasResult<BankAccountTransfer> {
        let transfer = self.repository.get_transfer(transfer_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Transfer {} not found", transfer_id)))?;

        if transfer.status != "submitted" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot approve transfer in '{}' status. Must be 'submitted'.", transfer.status)
            ));
        }

        info!("Approving bank transfer {}", transfer.transfer_number);
        self.repository.update_transfer_status(
            transfer_id, "approved", None, approved_by, None, None, None,
        ).await
    }

    /// Mark transfer as in transit
    pub async fn mark_in_transit(&self, transfer_id: Uuid) -> AtlasResult<BankAccountTransfer> {
        let transfer = self.repository.get_transfer(transfer_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Transfer {} not found", transfer_id)))?;

        if transfer.status != "approved" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot mark in-transit from '{}' status. Must be 'approved'.", transfer.status)
            ));
        }

        info!("Bank transfer {} in transit", transfer.transfer_number);
        self.repository.update_transfer_status(
            transfer_id, "in_transit", None, None, None, None, None,
        ).await
    }

    /// Complete a transfer
    pub async fn complete_transfer(&self, transfer_id: Uuid, completed_by: Option<Uuid>) -> AtlasResult<BankAccountTransfer> {
        let transfer = self.repository.get_transfer(transfer_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Transfer {} not found", transfer_id)))?;

        if transfer.status != "in_transit" && transfer.status != "approved" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot complete transfer in '{}' status. Must be 'in_transit' or 'approved'.", transfer.status)
            ));
        }

        info!("Completing bank transfer {}", transfer.transfer_number);
        self.repository.update_transfer_status(
            transfer_id, "completed", None, None, completed_by, None, None,
        ).await
    }

    /// Cancel a transfer
    pub async fn cancel_transfer(&self, transfer_id: Uuid, cancelled_by: Option<Uuid>, reason: Option<&str>) -> AtlasResult<BankAccountTransfer> {
        let transfer = self.repository.get_transfer(transfer_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Transfer {} not found", transfer_id)))?;

        if transfer.status != "draft" && transfer.status != "submitted" && transfer.status != "approved" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot cancel transfer in '{}' status. Must be 'draft', 'submitted', or 'approved'.", transfer.status)
            ));
        }

        info!("Cancelling bank transfer {}", transfer.transfer_number);
        self.repository.update_transfer_status(
            transfer_id, "cancelled", None, None, None, cancelled_by, reason,
        ).await
    }

    /// Calculate cross-currency transfer amount
    pub fn calculate_cross_currency_amount(amount: f64, exchange_rate: f64) -> f64 {
        amount * exchange_rate
    }

    /// Get dashboard summary
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<BankTransferDashboardSummary> {
        self.repository.get_dashboard_summary(org_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_settlement_methods() {
        assert!(VALID_SETTLEMENT_METHODS.contains(&"immediate"));
        assert!(VALID_SETTLEMENT_METHODS.contains(&"scheduled"));
        assert!(VALID_SETTLEMENT_METHODS.contains(&"batch"));
        assert_eq!(VALID_SETTLEMENT_METHODS.len(), 3);
    }

    #[test]
    fn test_valid_transfer_statuses() {
        assert!(VALID_TRANSFER_STATUSES.contains(&"draft"));
        assert!(VALID_TRANSFER_STATUSES.contains(&"submitted"));
        assert!(VALID_TRANSFER_STATUSES.contains(&"approved"));
        assert!(VALID_TRANSFER_STATUSES.contains(&"in_transit"));
        assert!(VALID_TRANSFER_STATUSES.contains(&"completed"));
        assert!(VALID_TRANSFER_STATUSES.contains(&"cancelled"));
        assert!(VALID_TRANSFER_STATUSES.contains(&"reversed"));
        assert!(VALID_TRANSFER_STATUSES.contains(&"failed"));
        assert_eq!(VALID_TRANSFER_STATUSES.len(), 8);
    }

    #[test]
    fn test_valid_priorities() {
        assert!(VALID_PRIORITIES.contains(&"low"));
        assert!(VALID_PRIORITIES.contains(&"normal"));
        assert!(VALID_PRIORITIES.contains(&"high"));
        assert!(VALID_PRIORITIES.contains(&"urgent"));
        assert_eq!(VALID_PRIORITIES.len(), 4);
    }

    #[test]
    fn test_calculate_cross_currency_same() {
        // Same currency: rate 1.0
        let result = BankAccountTransferEngine::calculate_cross_currency_amount(1000.0, 1.0);
        assert!((result - 1000.0).abs() < 0.01);
    }

    #[test]
    fn test_calculate_cross_currency_usd_eur() {
        // USD to EUR: rate 0.92
        let result = BankAccountTransferEngine::calculate_cross_currency_amount(10000.0, 0.92);
        assert!((result - 9200.0).abs() < 0.01);
    }

    #[test]
    fn test_calculate_cross_currency_eur_usd() {
        // EUR to USD: rate 1.087
        let result = BankAccountTransferEngine::calculate_cross_currency_amount(5000.0, 1.087);
        assert!((result - 5435.0).abs() < 0.01);
    }

    #[test]
    fn test_calculate_cross_currency_large_amount() {
        let result = BankAccountTransferEngine::calculate_cross_currency_amount(1000000.0, 0.75);
        assert!((result - 750000.0).abs() < 0.01);
    }
}
