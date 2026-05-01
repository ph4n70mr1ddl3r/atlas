//! Netting Engine
//!
//! Core netting operations:
//! - Netting agreement management (create/query agreements)
//! - Netting batch creation and transaction selection
//! - Net difference calculation (payables - receivables)
//! - Batch approval workflow
//! - Settlement processing
//!
//! Oracle Fusion Cloud ERP equivalent: Financials > Netting

use atlas_shared::{
    NettingAgreement, NettingBatch, NettingTransactionLine,
    NettingSettlementSummary, NettingDashboardSummary,
    AtlasError, AtlasResult,
};
use super::NettingRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Valid netting directions
#[allow(dead_code)]
const VALID_NETTING_DIRECTIONS: &[&str] = &[
    "payables_to_receivables", "receivables_to_payables", "bi_directional",
];

/// Valid settlement methods
#[allow(dead_code)]
const VALID_SETTLEMENT_METHODS: &[&str] = &[
    "automatic", "manual",
];

/// Valid agreement statuses
#[allow(dead_code)]
const VALID_AGREEMENT_STATUSES: &[&str] = &[
    "draft", "active", "inactive", "terminated",
];

/// Valid batch statuses
#[allow(dead_code)]
const VALID_BATCH_STATUSES: &[&str] = &[
    "draft", "submitted", "approved", "settled", "cancelled", "reversed",
];

/// Valid settlement directions
#[allow(dead_code)]
const VALID_SETTLEMENT_DIRECTIONS: &[&str] = &[
    "pay", "receive", "zero",
];

/// Valid line source types
#[allow(dead_code)]
const VALID_LINE_SOURCE_TYPES: &[&str] = &[
    "payable", "receivable",
];

/// Valid line statuses
#[allow(dead_code)]
const VALID_LINE_STATUSES: &[&str] = &[
    "selected", "netted", "cancelled",
];

/// Netting Engine
pub struct NettingEngine {
    repository: Arc<dyn NettingRepository>,
}

impl NettingEngine {
    pub fn new(repository: Arc<dyn NettingRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Netting Agreement Management
    // ========================================================================

    /// Create a new netting agreement
    pub async fn create_agreement(
        &self,
        org_id: Uuid,
        agreement_number: &str,
        name: &str,
        description: Option<&str>,
        partner_id: Uuid,
        partner_number: Option<&str>,
        partner_name: Option<&str>,
        currency_code: &str,
        netting_direction: &str,
        settlement_method: &str,
        minimum_netting_amount: &str,
        maximum_netting_amount: Option<&str>,
        auto_select_transactions: bool,
        selection_criteria: serde_json::Value,
        netting_clearing_account: Option<&str>,
        ap_clearing_account: Option<&str>,
        ar_clearing_account: Option<&str>,
        approval_required: bool,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<NettingAgreement> {
        if agreement_number.is_empty() || name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Agreement number and name are required".to_string(),
            ));
        }
        if currency_code.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Currency code is required".to_string(),
            ));
        }
        if !VALID_NETTING_DIRECTIONS.contains(&netting_direction) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid netting_direction '{}'. Must be one of: {}",
                netting_direction, VALID_NETTING_DIRECTIONS.join(", ")
            )));
        }
        if !VALID_SETTLEMENT_METHODS.contains(&settlement_method) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid settlement_method '{}'. Must be one of: {}",
                settlement_method, VALID_SETTLEMENT_METHODS.join(", ")
            )));
        }
        let min_amt: f64 = minimum_netting_amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "Minimum netting amount must be a valid number".to_string(),
        ))?;
        if min_amt < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Minimum netting amount must be non-negative".to_string(),
            ));
        }
        if let Some(max_str) = maximum_netting_amount {
            let max_amt: f64 = max_str.parse().map_err(|_| AtlasError::ValidationFailed(
                "Maximum netting amount must be a valid number".to_string(),
            ))?;
            if max_amt < 0.0 {
                return Err(AtlasError::ValidationFailed(
                    "Maximum netting amount must be non-negative".to_string(),
                ));
            }
            if max_amt < min_amt {
                return Err(AtlasError::ValidationFailed(
                    "Maximum netting amount must be >= minimum".to_string(),
                ));
            }
        }

        // Check uniqueness
        if let Some(_) = self.repository.get_agreement_by_number(org_id, agreement_number).await? {
            return Err(AtlasError::Conflict(
                format!("Agreement number '{}' already exists", agreement_number)
            ));
        }

        info!("Creating netting agreement {} for partner {}", agreement_number, partner_id);

        self.repository.create_agreement(
            org_id, agreement_number, name, description,
            partner_id, partner_number, partner_name,
            currency_code, netting_direction, settlement_method,
            minimum_netting_amount, maximum_netting_amount,
            auto_select_transactions, selection_criteria,
            netting_clearing_account, ap_clearing_account, ar_clearing_account,
            approval_required, effective_from, effective_to, created_by,
        ).await
    }

    /// Get agreement by ID
    pub async fn get_agreement(&self, id: Uuid) -> AtlasResult<Option<NettingAgreement>> {
        self.repository.get_agreement(id).await
    }

    /// Get agreement by number
    pub async fn get_agreement_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<NettingAgreement>> {
        self.repository.get_agreement_by_number(org_id, number).await
    }

    /// List agreements
    pub async fn list_agreements(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<NettingAgreement>> {
        if let Some(s) = status {
            if !VALID_AGREEMENT_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_AGREEMENT_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_agreements(org_id, status).await
    }

    /// Activate a draft agreement
    pub async fn activate_agreement(&self, id: Uuid) -> AtlasResult<NettingAgreement> {
        let agreement = self.repository.get_agreement(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Agreement {} not found", id)))?;

        if agreement.status != "draft" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot activate agreement in '{}' status. Must be 'draft'.", agreement.status)
            ));
        }

        info!("Activating netting agreement {}", agreement.agreement_number);
        self.repository.update_agreement_status(id, "active").await
    }

    // ========================================================================
    // Netting Batch Management
    // ========================================================================

    /// Create a new netting batch
    pub async fn create_batch(
        &self,
        org_id: Uuid,
        agreement_id: Uuid,
        netting_date: chrono::NaiveDate,
        gl_date: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<NettingBatch> {
        let agreement = self.repository.get_agreement(agreement_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Netting agreement {} not found", agreement_id)
            ))?;

        if agreement.status != "active" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot create batch for agreement in '{}' status. Must be 'active'.", agreement.status)
            ));
        }

        let batch_number = format!("NET-{}", Uuid::new_v4().to_string()[..8].to_uppercase());

        info!("Creating netting batch {} for agreement {}", batch_number, agreement.agreement_number);

        self.repository.create_batch(
            org_id, &batch_number, agreement_id,
            netting_date, gl_date,
            agreement.partner_id, agreement.partner_name.as_deref(),
            &agreement.currency_code,
            created_by,
        ).await
    }

    /// Get a netting batch by ID
    pub async fn get_batch(&self, id: Uuid) -> AtlasResult<Option<NettingBatch>> {
        self.repository.get_batch(id).await
    }

    /// List batches for an agreement
    pub async fn list_batches(&self, org_id: Uuid, agreement_id: Option<Uuid>, status: Option<&str>) -> AtlasResult<Vec<NettingBatch>> {
        if let Some(s) = status {
            if !VALID_BATCH_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_BATCH_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_batches(org_id, agreement_id, status).await
    }

    // ========================================================================
    // Transaction Selection
    // ========================================================================

    /// Add a transaction line to a netting batch
    pub async fn add_transaction_line(
        &self,
        org_id: Uuid,
        batch_id: Uuid,
        source_type: &str,
        source_id: Uuid,
        source_number: Option<&str>,
        source_date: Option<chrono::NaiveDate>,
        original_amount: &str,
        netting_amount: &str,
        currency_code: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<NettingTransactionLine> {
        let batch = self.repository.get_batch(batch_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Netting batch {} not found", batch_id)
            ))?;

        if batch.status != "draft" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot add lines to batch in '{}' status. Must be 'draft'.", batch.status)
            ));
        }

        if !VALID_LINE_SOURCE_TYPES.contains(&source_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid source_type '{}'. Must be one of: {}", source_type, VALID_LINE_SOURCE_TYPES.join(", ")
            )));
        }

        let original: f64 = original_amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "Original amount must be a valid number".to_string(),
        ))?;
        let netting: f64 = netting_amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "Netting amount must be a valid number".to_string(),
        ))?;

        if netting <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Netting amount must be positive".to_string(),
            ));
        }
        if netting > original {
            return Err(AtlasError::ValidationFailed(
                "Netting amount cannot exceed original amount".to_string(),
            ));
        }

        let remaining = original - netting;
        let lines = self.repository.list_batch_lines(batch_id).await?;
        let line_number = (lines.len() as i32) + 1;

        info!("Adding {} line to batch {}: {} of {}",
            source_type, batch.batch_number, netting_amount, original_amount);

        self.repository.create_transaction_line(
            org_id, batch_id, line_number,
            source_type, source_id, source_number, source_date,
            original_amount, netting_amount, &format!("{:.2}", remaining),
            currency_code, created_by,
        ).await
    }

    /// List transaction lines for a batch
    pub async fn list_batch_lines(&self, batch_id: Uuid) -> AtlasResult<Vec<NettingTransactionLine>> {
        self.repository.list_batch_lines(batch_id).await
    }

    /// Calculate net difference for a batch
    pub fn calculate_net_difference(payables: f64, receivables: f64) -> (f64, String) {
        let diff = payables - receivables;
        let direction = if diff.abs() < 0.01 {
            "zero".to_string()
        } else if diff > 0.0 {
            "pay".to_string()
        } else {
            "receive".to_string()
        };
        (diff, direction)
    }

    // ========================================================================
    // Batch Workflow
    // ========================================================================

    /// Submit a draft batch for approval
    pub async fn submit_batch(&self, batch_id: Uuid, submitted_by: Option<Uuid>) -> AtlasResult<NettingBatch> {
        let batch = self.repository.get_batch(batch_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Netting batch {} not found", batch_id)
            ))?;

        if batch.status != "draft" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot submit batch in '{}' status. Must be 'draft'.", batch.status)
            ));
        }

        // Validate batch has at least one payable and one receivable
        let lines = self.repository.list_batch_lines(batch_id).await?;
        let has_payables = lines.iter().any(|l| l.source_type == "payable");
        let has_receivables = lines.iter().any(|l| l.source_type == "receivable");

        if !has_payables || !has_receivables {
            return Err(AtlasError::ValidationFailed(
                "Netting batch must contain at least one payable and one receivable transaction".to_string(),
            ));
        }

        // Calculate totals
        let total_payables: f64 = lines.iter()
            .filter(|l| l.source_type == "payable")
            .map(|l| l.netting_amount.parse::<f64>().unwrap_or(0.0))
            .sum();
        let total_receivables: f64 = lines.iter()
            .filter(|l| l.source_type == "receivable")
            .map(|l| l.netting_amount.parse::<f64>().unwrap_or(0.0))
            .sum();
        let (net_diff, settlement_dir) = Self::calculate_net_difference(total_payables, total_receivables);

        // Update totals
        self.repository.update_batch_totals(
            batch_id,
            &format!("{:.2}", total_payables),
            &format!("{:.2}", total_receivables),
            &format!("{:.2}", net_diff),
            &settlement_dir,
            lines.iter().filter(|l| l.source_type == "payable").count() as i32,
            lines.iter().filter(|l| l.source_type == "receivable").count() as i32,
        ).await?;

        info!("Submitting netting batch {} (payables: {:.2}, receivables: {:.2}, net: {:.2} {})",
            batch.batch_number, total_payables, total_receivables, net_diff, settlement_dir);

        self.repository.update_batch_status(
            batch_id, "submitted", submitted_by, None, None,
        ).await
    }

    /// Approve a submitted batch
    pub async fn approve_batch(&self, batch_id: Uuid, approved_by: Option<Uuid>) -> AtlasResult<NettingBatch> {
        let batch = self.repository.get_batch(batch_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Netting batch {} not found", batch_id)
            ))?;

        if batch.status != "submitted" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot approve batch in '{}' status. Must be 'submitted'.", batch.status)
            ));
        }

        info!("Approving netting batch {}", batch.batch_number);

        self.repository.update_batch_status(
            batch_id, "approved", None, approved_by, None,
        ).await
    }

    /// Settle an approved batch (mark transactions as netted)
    pub async fn settle_batch(&self, batch_id: Uuid) -> AtlasResult<NettingBatch> {
        let batch = self.repository.get_batch(batch_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Netting batch {} not found", batch_id)
            ))?;

        if batch.status != "approved" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot settle batch in '{}' status. Must be 'approved'.", batch.status)
            ));
        }

        // Mark all transaction lines as netted
        let lines = self.repository.list_batch_lines(batch_id).await?;
        for line in &lines {
            self.repository.update_line_status(line.id, "netted").await?;
        }

        info!("Settling netting batch {} (direction: {})", batch.batch_number, batch.settlement_direction);

        self.repository.update_batch_status(
            batch_id, "settled", None, None, None,
        ).await
    }

    /// Cancel a draft or submitted batch
    pub async fn cancel_batch(&self, batch_id: Uuid, reason: Option<&str>) -> AtlasResult<NettingBatch> {
        let batch = self.repository.get_batch(batch_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Netting batch {} not found", batch_id)
            ))?;

        if batch.status != "draft" && batch.status != "submitted" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot cancel batch in '{}' status. Must be 'draft' or 'submitted'.", batch.status)
            ));
        }

        info!("Cancelling netting batch {}", batch.batch_number);

        self.repository.update_batch_status(
            batch_id, "cancelled", None, None, reason,
        ).await
    }

    /// Get settlement summary for a batch
    pub async fn get_settlement_summary(&self, batch_id: Uuid) -> AtlasResult<NettingSettlementSummary> {
        let batch = self.repository.get_batch(batch_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Netting batch {} not found", batch_id)
            ))?;

        Ok(NettingSettlementSummary {
            batch_id: batch.id,
            batch_number: batch.batch_number,
            partner_name: batch.partner_name,
            total_payables: batch.total_payables_amount,
            total_receivables: batch.total_receivables_amount,
            net_difference: batch.net_difference,
            settlement_direction: batch.settlement_direction,
            payable_transaction_count: batch.payable_transaction_count,
            receivable_transaction_count: batch.receivable_transaction_count,
        })
    }

    /// Get netting dashboard summary
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<NettingDashboardSummary> {
        self.repository.get_dashboard_summary(org_id).await
    }

    /// Check if a partner is eligible for netting
    pub fn check_netting_eligibility(
        payables_amount: f64,
        receivables_amount: f64,
        minimum_amount: f64,
    ) -> bool {
        let diff = (payables_amount - receivables_amount).abs();
        diff >= minimum_amount && payables_amount > 0.0 && receivables_amount > 0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_netting_directions() {
        assert!(VALID_NETTING_DIRECTIONS.contains(&"payables_to_receivables"));
        assert!(VALID_NETTING_DIRECTIONS.contains(&"receivables_to_payables"));
        assert!(VALID_NETTING_DIRECTIONS.contains(&"bi_directional"));
        assert_eq!(VALID_NETTING_DIRECTIONS.len(), 3);
    }

    #[test]
    fn test_valid_settlement_methods() {
        assert!(VALID_SETTLEMENT_METHODS.contains(&"automatic"));
        assert!(VALID_SETTLEMENT_METHODS.contains(&"manual"));
        assert_eq!(VALID_SETTLEMENT_METHODS.len(), 2);
    }

    #[test]
    fn test_valid_agreement_statuses() {
        assert!(VALID_AGREEMENT_STATUSES.contains(&"draft"));
        assert!(VALID_AGREEMENT_STATUSES.contains(&"active"));
        assert!(VALID_AGREEMENT_STATUSES.contains(&"inactive"));
        assert!(VALID_AGREEMENT_STATUSES.contains(&"terminated"));
        assert_eq!(VALID_AGREEMENT_STATUSES.len(), 4);
    }

    #[test]
    fn test_valid_batch_statuses() {
        assert!(VALID_BATCH_STATUSES.contains(&"draft"));
        assert!(VALID_BATCH_STATUSES.contains(&"submitted"));
        assert!(VALID_BATCH_STATUSES.contains(&"approved"));
        assert!(VALID_BATCH_STATUSES.contains(&"settled"));
        assert!(VALID_BATCH_STATUSES.contains(&"cancelled"));
        assert!(VALID_BATCH_STATUSES.contains(&"reversed"));
        assert_eq!(VALID_BATCH_STATUSES.len(), 6);
    }

    #[test]
    fn test_valid_settlement_directions() {
        assert!(VALID_SETTLEMENT_DIRECTIONS.contains(&"pay"));
        assert!(VALID_SETTLEMENT_DIRECTIONS.contains(&"receive"));
        assert!(VALID_SETTLEMENT_DIRECTIONS.contains(&"zero"));
        assert_eq!(VALID_SETTLEMENT_DIRECTIONS.len(), 3);
    }

    #[test]
    fn test_valid_line_source_types() {
        assert!(VALID_LINE_SOURCE_TYPES.contains(&"payable"));
        assert!(VALID_LINE_SOURCE_TYPES.contains(&"receivable"));
        assert_eq!(VALID_LINE_SOURCE_TYPES.len(), 2);
    }

    #[test]
    fn test_valid_line_statuses() {
        assert!(VALID_LINE_STATUSES.contains(&"selected"));
        assert!(VALID_LINE_STATUSES.contains(&"netted"));
        assert!(VALID_LINE_STATUSES.contains(&"cancelled"));
        assert_eq!(VALID_LINE_STATUSES.len(), 3);
    }

    #[test]
    fn test_calculate_net_difference_pay() {
        let (diff, direction) = NettingEngine::calculate_net_difference(10000.0, 7500.0);
        assert!((diff - 2500.0).abs() < 0.01);
        assert_eq!(direction, "pay");
    }

    #[test]
    fn test_calculate_net_difference_receive() {
        let (diff, direction) = NettingEngine::calculate_net_difference(5000.0, 8000.0);
        assert!((diff - (-3000.0)).abs() < 0.01);
        assert_eq!(direction, "receive");
    }

    #[test]
    fn test_calculate_net_difference_zero() {
        let (diff, direction) = NettingEngine::calculate_net_difference(5000.0, 5000.0);
        assert!(diff.abs() < 0.01);
        assert_eq!(direction, "zero");
    }

    #[test]
    fn test_calculate_net_difference_near_zero() {
        let (diff, direction) = NettingEngine::calculate_net_difference(5000.0, 5000.005);
        assert!(diff.abs() < 0.01);
        assert_eq!(direction, "zero");
    }

    #[test]
    fn test_check_netting_eligibility() {
        // Both sides have amounts, above minimum
        assert!(NettingEngine::check_netting_eligibility(10000.0, 8000.0, 100.0));
        // Below minimum
        assert!(!NettingEngine::check_netting_eligibility(100.0, 50.0, 100.0));
        // Exact minimum
        assert!(NettingEngine::check_netting_eligibility(200.0, 100.0, 100.0));
        // No receivables
        assert!(!NettingEngine::check_netting_eligibility(10000.0, 0.0, 100.0));
        // No payables
        assert!(!NettingEngine::check_netting_eligibility(0.0, 5000.0, 100.0));
        // Both zero
        assert!(!NettingEngine::check_netting_eligibility(0.0, 0.0, 100.0));
    }

    #[test]
    fn test_check_netting_eligibility_zero_minimum() {
        // With zero minimum, any non-zero both-sides amount is eligible
        assert!(NettingEngine::check_netting_eligibility(50.0, 30.0, 0.0));
    }
}
