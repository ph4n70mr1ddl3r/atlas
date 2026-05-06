//! Remittance Batch Engine
//!
//! Manages remittance batch lifecycle: creation, approval, formatting,
//! transmission, confirmation, settlement, and reversal.
//!
//! Oracle Fusion Cloud ERP equivalent: Receivables > Receipts > Remittance Batches

use atlas_shared::{
    RemittanceBatch, RemittanceBatchReceipt, RemittanceBatchSummary,
    AtlasError, AtlasResult,
};
use super::RemittanceBatchRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Valid remittance methods
const VALID_REMITTANCE_METHODS: &[&str] = &["standard", "factoring", "standard_with_recourse"];

/// Valid batch statuses (ordered lifecycle)
const VALID_STATUSES: &[&str] = &[
    "draft", "approved", "formatted", "transmitted",
    "confirmed", "settled", "reversed", "cancelled",
];

/// Valid receipt statuses in a batch
#[allow(dead_code)]
const VALID_RECEIPT_STATUSES: &[&str] = &["included", "excluded", "reversed"];

/// Validate that a status transition is allowed.
/// draft -> approved -> formatted -> transmitted -> confirmed -> settled
/// Any status -> cancelled (only if draft or approved)
/// settled -> reversed
pub fn validate_status_transition(current: &str, target: &str) -> AtlasResult<()> {
    match (current, target) {
        // Forward transitions: must advance by exactly one step
        (_, "approved") if current == "draft" => Ok(()),
        (_, "formatted") if current == "approved" => Ok(()),
        (_, "transmitted") if current == "formatted" => Ok(()),
        (_, "confirmed") if current == "transmitted" => Ok(()),
        (_, "settled") if current == "confirmed" => Ok(()),
        // Cancel: only from draft or approved
        (_, "cancelled") if current == "draft" || current == "approved" => Ok(()),
        // Reverse: only from settled
        (_, "reversed") if current == "settled" => Ok(()),
        _ => Err(AtlasError::WorkflowError(format!(
            "Invalid status transition from '{}' to '{}'. \
             Valid transitions: draft→approved→formatted→transmitted→confirmed→settled, \
             draft/approved→cancelled, settled→reversed",
            current, target
        ))),
    }
}

/// Calculate total amount and count from a list of included receipt amounts.
/// Returns (total_amount, receipt_count).
pub fn calculate_batch_totals(receipt_amounts: &[(f64, &str)]) -> (f64, i32) {
    let mut total = 0.0_f64;
    let mut count = 0_i32;
    for (amount, status) in receipt_amounts {
        if *status == "included" {
            total += amount;
            count += 1;
        }
    }
    (total, count)
}

/// Remittance Batch Engine
pub struct RemittanceBatchEngine {
    repository: Arc<dyn RemittanceBatchRepository>,
}

impl RemittanceBatchEngine {
    pub fn new(repository: Arc<dyn RemittanceBatchRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Batch Creation
    // ========================================================================

    /// Create a new remittance batch
    pub async fn create_batch(
        &self,
        org_id: Uuid,
        batch_name: Option<&str>,
        bank_account_id: Option<Uuid>,
        bank_account_name: Option<&str>,
        bank_name: Option<&str>,
        remittance_method: &str,
        currency_code: &str,
        batch_date: chrono::NaiveDate,
        gl_date: Option<chrono::NaiveDate>,
        receipt_currency_code: Option<&str>,
        exchange_rate_type: Option<&str>,
        format_program: Option<&str>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<RemittanceBatch> {
        if !VALID_REMITTANCE_METHODS.contains(&remittance_method) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid remittance method '{}'. Must be one of: {}",
                remittance_method, VALID_REMITTANCE_METHODS.join(", ")
            )));
        }

        // Generate batch number
        let next_num = self.repository.get_next_batch_number(org_id).await?;
        let batch_number = format!("RB-{:05}", next_num);

        info!(
            "Creating remittance batch {} for org {}",
            batch_number, org_id
        );

        self.repository.create_batch(
            org_id, &batch_number, batch_name,
            bank_account_id, bank_account_name, bank_name,
            remittance_method, currency_code, batch_date, gl_date,
            receipt_currency_code, exchange_rate_type,
            format_program, notes, created_by,
        ).await
    }

    /// Get a batch by ID
    pub async fn get_batch(&self, id: Uuid) -> AtlasResult<Option<RemittanceBatch>> {
        self.repository.get_batch(id).await
    }

    /// Get a batch by number
    pub async fn get_batch_by_number(
        &self,
        org_id: Uuid,
        batch_number: &str,
    ) -> AtlasResult<Option<RemittanceBatch>> {
        self.repository.get_batch_by_number(org_id, batch_number).await
    }

    /// List batches with optional filters
    pub async fn list_batches(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        currency_code: Option<&str>,
        remittance_method: Option<&str>,
    ) -> AtlasResult<Vec<RemittanceBatch>> {
        if let Some(s) = status {
            if !VALID_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}",
                    s, VALID_STATUSES.join(", ")
                )));
            }
        }
        if let Some(rm) = remittance_method {
            if !VALID_REMITTANCE_METHODS.contains(&rm) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid remittance method '{}'. Must be one of: {}",
                    rm, VALID_REMITTANCE_METHODS.join(", ")
                )));
            }
        }
        self.repository.list_batches(org_id, status, currency_code, remittance_method).await
    }

    // ========================================================================
    // Batch Lifecycle
    // ========================================================================

    /// Approve a batch (draft → approved)
    pub async fn approve_batch(&self, id: Uuid) -> AtlasResult<RemittanceBatch> {
        let batch = self.get_batch_or_error(id).await?;
        validate_status_transition(&batch.status, "approved")?;
        info!("Approving remittance batch {}", batch.batch_number);
        self.repository.update_batch_status(id, "approved").await
    }

    /// Format a batch (approved → formatted)
    pub async fn format_batch(&self, id: Uuid) -> AtlasResult<RemittanceBatch> {
        let batch = self.get_batch_or_error(id).await?;
        validate_status_transition(&batch.status, "formatted")?;
        if batch.receipt_count == 0 {
            return Err(AtlasError::ValidationFailed(
                "Cannot format a batch with no receipts".to_string(),
            ));
        }
        info!("Formatting remittance batch {}", batch.batch_number);
        self.repository.update_batch_status(id, "formatted").await
    }

    /// Transmit a batch (formatted → transmitted)
    pub async fn transmit_batch(&self, id: Uuid, reference_number: Option<&str>) -> AtlasResult<RemittanceBatch> {
        let batch = self.get_batch_or_error(id).await?;
        validate_status_transition(&batch.status, "transmitted")?;
        info!("Transmitting remittance batch {}", batch.batch_number);
        if let Some(ref_num) = reference_number {
            self.repository.update_reference_number(id, ref_num).await?;
        }
        self.repository.update_batch_status(id, "transmitted").await
    }

    /// Confirm a batch (transmitted → confirmed)
    pub async fn confirm_batch(&self, id: Uuid) -> AtlasResult<RemittanceBatch> {
        let batch = self.get_batch_or_error(id).await?;
        validate_status_transition(&batch.status, "confirmed")?;
        info!("Confirming remittance batch {}", batch.batch_number);
        self.repository.update_batch_status(id, "confirmed").await
    }

    /// Settle a batch (confirmed → settled)
    pub async fn settle_batch(&self, id: Uuid) -> AtlasResult<RemittanceBatch> {
        let batch = self.get_batch_or_error(id).await?;
        validate_status_transition(&batch.status, "settled")?;
        info!("Settling remittance batch {}", batch.batch_number);
        self.repository.update_batch_status(id, "settled").await
    }

    /// Reverse a batch (settled → reversed)
    pub async fn reverse_batch(&self, id: Uuid, reason: Option<&str>) -> AtlasResult<RemittanceBatch> {
        let batch = self.get_batch_or_error(id).await?;
        validate_status_transition(&batch.status, "reversed")?;
        info!("Reversing remittance batch {}", batch.batch_number);
        if let Some(r) = reason {
            self.repository.update_batch_notes(id, Some(r)).await?;
        }
        self.repository.update_batch_status(id, "reversed").await
    }

    /// Cancel a batch (draft/approved → cancelled)
    pub async fn cancel_batch(&self, id: Uuid, reason: Option<&str>) -> AtlasResult<RemittanceBatch> {
        let batch = self.get_batch_or_error(id).await?;
        validate_status_transition(&batch.status, "cancelled")?;
        info!("Cancelling remittance batch {}", batch.batch_number);
        if let Some(r) = reason {
            self.repository.update_batch_notes(id, Some(r)).await?;
        }
        self.repository.update_batch_status(id, "cancelled").await
    }

    // ========================================================================
    // Batch Receipts
    // ========================================================================

    /// Add a receipt to a batch
    pub async fn add_receipt(
        &self,
        org_id: Uuid,
        batch_id: Uuid,
        receipt_id: Uuid,
        receipt_number: Option<&str>,
        customer_id: Option<Uuid>,
        customer_number: Option<&str>,
        customer_name: Option<&str>,
        receipt_date: Option<chrono::NaiveDate>,
        receipt_amount: &str,
        applied_amount: &str,
        receipt_method: Option<&str>,
        currency_code: &str,
        exchange_rate: Option<&str>,
        metadata: serde_json::Value,
    ) -> AtlasResult<RemittanceBatchReceipt> {
        let batch = self.get_batch_or_error(batch_id).await?;

        if batch.organization_id != org_id {
            return Err(AtlasError::Forbidden("Not authorized".to_string()));
        }

        if batch.status != "draft" {
            return Err(AtlasError::WorkflowError(
                "Cannot add receipts to a batch that is not in 'draft' status".to_string(),
            ));
        }

        let amt: f64 = receipt_amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "Receipt amount must be a valid number".to_string(),
        ))?;

        if amt <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Receipt amount must be positive".to_string(),
            ));
        }

        let applied: f64 = applied_amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "Applied amount must be a valid number".to_string(),
        ))?;

        if applied < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Applied amount cannot be negative".to_string(),
            ));
        }

        // Check for duplicate receipt
        let existing = self.repository.get_batch_receipt_by_receipt_id(batch_id, receipt_id).await?;
        if existing.is_some() {
            return Err(AtlasError::ValidationFailed(
                format!("Receipt {} is already in this batch", receipt_id)
            ));
        }

        let next_order = self.repository.get_next_receipt_order(batch_id).await?;

        info!(
            "Adding receipt to batch {} (amount: {})",
            batch.batch_number, receipt_amount
        );

        let result = self.repository.create_batch_receipt(
            org_id, batch_id, receipt_id, receipt_number,
            customer_id, customer_number, customer_name,
            receipt_date, receipt_amount, applied_amount,
            receipt_method, currency_code, exchange_rate,
            next_order, metadata,
        ).await?;

        // Recalculate batch totals
        self.recalculate_batch_totals(batch_id).await?;

        Ok(result)
    }

    /// Remove a receipt from a batch
    pub async fn remove_receipt(&self, batch_id: Uuid, receipt_id: Uuid) -> AtlasResult<()> {
        let batch = self.get_batch_or_error(batch_id).await?;

        if batch.status != "draft" {
            return Err(AtlasError::WorkflowError(
                "Cannot remove receipts from a batch that is not in 'draft' status".to_string(),
            ));
        }

        self.repository.delete_batch_receipt(batch_id, receipt_id).await?;

        // Recalculate batch totals
        self.recalculate_batch_totals(batch_id).await?;

        Ok(())
    }

    /// List receipts in a batch
    pub async fn list_batch_receipts(&self, batch_id: Uuid) -> AtlasResult<Vec<RemittanceBatchReceipt>> {
        self.repository.list_batch_receipts(batch_id).await
    }

    // ========================================================================
    // Remittance Advice
    // ========================================================================

    /// Mark remittance advice as sent
    pub async fn mark_advice_sent(&self, id: Uuid) -> AtlasResult<RemittanceBatch> {
        let batch = self.get_batch_or_error(id).await?;
        if batch.status != "confirmed" && batch.status != "settled" && batch.status != "transmitted" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot send remittance advice for batch in '{}' status. Must be transmitted, confirmed, or settled.",
                batch.status
            )));
        }
        info!("Marking remittance advice as sent for batch {}", batch.batch_number);
        self.repository.update_advice_sent(id).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get remittance batch summary dashboard
    pub async fn get_batch_summary(&self, org_id: Uuid) -> AtlasResult<RemittanceBatchSummary> {
        self.repository.get_batch_summary(org_id).await
    }

    // ========================================================================
    // Helpers
    // ========================================================================

    async fn get_batch_or_error(&self, id: Uuid) -> AtlasResult<RemittanceBatch> {
        self.repository.get_batch(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Remittance batch {} not found", id)
            ))
    }

    async fn recalculate_batch_totals(&self, batch_id: Uuid) -> AtlasResult<()> {
        let receipts = self.repository.list_batch_receipts(batch_id).await?;
        let mut total = 0.0_f64;
        let mut count = 0_i32;
        for r in &receipts {
            if r.status == "included" {
                let amt: f64 = r.receipt_amount.parse().unwrap_or(0.0);
                total += amt;
                count += 1;
            }
        }
        self.repository.update_batch_totals(batch_id, total, count).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_remittance_methods() {
        assert!(VALID_REMITTANCE_METHODS.contains(&"standard"));
        assert!(VALID_REMITTANCE_METHODS.contains(&"factoring"));
        assert!(VALID_REMITTANCE_METHODS.contains(&"standard_with_recourse"));
    }

    #[test]
    fn test_valid_statuses() {
        assert!(VALID_STATUSES.contains(&"draft"));
        assert!(VALID_STATUSES.contains(&"approved"));
        assert!(VALID_STATUSES.contains(&"formatted"));
        assert!(VALID_STATUSES.contains(&"transmitted"));
        assert!(VALID_STATUSES.contains(&"confirmed"));
        assert!(VALID_STATUSES.contains(&"settled"));
        assert!(VALID_STATUSES.contains(&"reversed"));
        assert!(VALID_STATUSES.contains(&"cancelled"));
    }

    // Status Transition Tests

    #[test]
    fn test_transition_draft_to_approved() {
        assert!(validate_status_transition("draft", "approved").is_ok());
    }

    #[test]
    fn test_transition_approved_to_formatted() {
        assert!(validate_status_transition("approved", "formatted").is_ok());
    }

    #[test]
    fn test_transition_formatted_to_transmitted() {
        assert!(validate_status_transition("formatted", "transmitted").is_ok());
    }

    #[test]
    fn test_transition_transmitted_to_confirmed() {
        assert!(validate_status_transition("transmitted", "confirmed").is_ok());
    }

    #[test]
    fn test_transition_confirmed_to_settled() {
        assert!(validate_status_transition("confirmed", "settled").is_ok());
    }

    #[test]
    fn test_transition_settled_to_reversed() {
        assert!(validate_status_transition("settled", "reversed").is_ok());
    }

    #[test]
    fn test_transition_draft_to_cancelled() {
        assert!(validate_status_transition("draft", "cancelled").is_ok());
    }

    #[test]
    fn test_transition_approved_to_cancelled() {
        assert!(validate_status_transition("approved", "cancelled").is_ok());
    }

    #[test]
    fn test_transition_invalid_skip() {
        // Cannot skip steps: draft -> formatted is invalid
        assert!(validate_status_transition("draft", "formatted").is_err());
    }

    #[test]
    fn test_transition_invalid_backwards() {
        // Cannot go backwards: approved -> draft
        assert!(validate_status_transition("approved", "draft").is_err());
    }

    #[test]
    fn test_transition_invalid_cancel_from_formatted() {
        // Cannot cancel from formatted
        assert!(validate_status_transition("formatted", "cancelled").is_err());
    }

    #[test]
    fn test_transition_invalid_reverse_from_draft() {
        // Cannot reverse from draft
        assert!(validate_status_transition("draft", "reversed").is_err());
    }

    #[test]
    fn test_transition_invalid_reverse_from_approved() {
        assert!(validate_status_transition("approved", "reversed").is_err());
    }

    #[test]
    fn test_transition_full_lifecycle() {
        assert!(validate_status_transition("draft", "approved").is_ok());
        assert!(validate_status_transition("approved", "formatted").is_ok());
        assert!(validate_status_transition("formatted", "transmitted").is_ok());
        assert!(validate_status_transition("transmitted", "confirmed").is_ok());
        assert!(validate_status_transition("confirmed", "settled").is_ok());
        assert!(validate_status_transition("settled", "reversed").is_ok());
    }

    // Batch Totals Tests

    #[test]
    fn test_calculate_batch_totals_included_only() {
        let receipts: Vec<(f64, &str)> = vec![
            (1000.0, "included"),
            (2500.0, "included"),
            (750.0, "included"),
        ];
        let (total, count) = calculate_batch_totals(&receipts);
        assert!((total - 4250.0).abs() < 0.01);
        assert_eq!(count, 3);
    }

    #[test]
    fn test_calculate_batch_totals_mixed_status() {
        let receipts: Vec<(f64, &str)> = vec![
            (1000.0, "included"),
            (500.0, "excluded"),
            (2500.0, "included"),
            (300.0, "reversed"),
        ];
        let (total, count) = calculate_batch_totals(&receipts);
        assert!((total - 3500.0).abs() < 0.01);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_calculate_batch_totals_empty() {
        let receipts: Vec<(f64, &str)> = vec![];
        let (total, count) = calculate_batch_totals(&receipts);
        assert!((total).abs() < 0.01);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_calculate_batch_totals_all_excluded() {
        let receipts: Vec<(f64, &str)> = vec![
            (100.0, "excluded"),
            (200.0, "reversed"),
        ];
        let (total, count) = calculate_batch_totals(&receipts);
        assert!((total).abs() < 0.01);
        assert_eq!(count, 0);
    }
}
