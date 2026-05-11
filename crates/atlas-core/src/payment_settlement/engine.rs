//! Payment Settlement Engine
//!
//! Orchestrates settlement batch creation, lifecycle transitions, line management,
//! activity tracking, validation, and dashboard summary.
//!
//! Oracle Fusion Cloud ERP equivalent: Financials > Payables > Settlement

use atlas_shared::{AtlasError, AtlasResult};
use super::repository::{
    PaymentSettlementRepository,
    SettlementBatch, SettlementLine, SettlementSummary,
    SettlementBatchCreateParams, SettlementLineCreateParams,
};
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

// Valid statuses
const VALID_BATCH_STATUSES: &[&str] = &[
    "draft", "submitted", "approved", "settled", "cancelled",
];

const VALID_SETTLEMENT_METHODS: &[&str] = &[
    "check", "electronic", "wire", "ach", "sepa", "manual",
];

const VALID_SETTLEMENT_TYPES: &[&str] = &[
    "full", "partial", "prepayment", "write_off",
];

/// Payment Settlement Engine
pub struct PaymentSettlementEngine {
    repo: Arc<dyn PaymentSettlementRepository>,
}

impl PaymentSettlementEngine {
    pub fn new(repo: Arc<dyn PaymentSettlementRepository>) -> Self {
        Self { repo }
    }

    // ========================================================================
    // Validation
    // ========================================================================

    fn validate_settlement_method(method: &str) -> AtlasResult<()> {
        if !VALID_SETTLEMENT_METHODS.contains(&method) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid settlement_method '{}'. Must be one of: {}", method, VALID_SETTLEMENT_METHODS.join(", ")
            )));
        }
        Ok(())
    }

    fn validate_settlement_type(s_type: &str) -> AtlasResult<()> {
        if !VALID_SETTLEMENT_TYPES.contains(&s_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid settlement_type '{}'. Must be one of: {}", s_type, VALID_SETTLEMENT_TYPES.join(", ")
            )));
        }
        Ok(())
    }

    fn validate_batch_status(status: &str) -> AtlasResult<()> {
        if !VALID_BATCH_STATUSES.contains(&status) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid status '{}'. Must be one of: {}", status, VALID_BATCH_STATUSES.join(", ")
            )));
        }
        Ok(())
    }

    /// Validate batch status transition
    /// draft → submitted → approved → settled
    /// draft → cancelled
    /// submitted → cancelled
    /// approved → cancelled
    pub fn validate_status_transition(current: &str, target: &str) -> AtlasResult<()> {
        match (current, target) {
            ("draft", "submitted") => Ok(()),
            ("draft", "cancelled") => Ok(()),
            ("submitted", "approved") => Ok(()),
            ("submitted", "cancelled") => Ok(()),
            ("approved", "settled") => Ok(()),
            ("approved", "cancelled") => Ok(()),
            _ => Err(AtlasError::WorkflowError(format!(
                "Invalid status transition from '{current}' to '{target}'. \
                 Valid: draft→submitted, draft→cancelled, submitted→approved, \
                 submitted→cancelled, approved→settled, approved→cancelled"
            ))),
        }
    }

    // ========================================================================
    // Batch CRUD
    // ========================================================================

    /// Create a new settlement batch
    pub async fn create_batch(
        &self,
        org_id: Uuid,
        batch_name: &str,
        description: Option<&str>,
        bank_account_id: Option<Uuid>,
        bank_account_name: Option<&str>,
        currency_code: &str,
        exchange_rate_type: Option<&str>,
        exchange_rate: Option<f64>,
        settlement_date: chrono::NaiveDate,
        gl_date: chrono::NaiveDate,
        settlement_method: &str,
        settlement_type: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<SettlementBatch> {
        info!("Creating settlement batch '{}' for org {}", batch_name, org_id);

        Self::validate_settlement_method(settlement_method)?;
        Self::validate_settlement_type(settlement_type)?;

        if gl_date < settlement_date {
            return Err(AtlasError::ValidationFailed(
                "GL date cannot be before settlement date".to_string()
            ));
        }

        let params = SettlementBatchCreateParams {
            org_id,
            batch_name: batch_name.to_string(),
            description: description.map(std::string::ToString::to_string),
            bank_account_id,
            bank_account_name: bank_account_name.map(std::string::ToString::to_string),
            currency_code: currency_code.to_string(),
            exchange_rate_type: exchange_rate_type.map(std::string::ToString::to_string),
            exchange_rate,
            settlement_date,
            gl_date,
            settlement_method: settlement_method.to_string(),
            settlement_type: settlement_type.to_string(),
            created_by,
        };

        let batch = self.repo.create_batch(&params).await?;

        // Log activity
        let _ = self.repo.create_activity(
            org_id, batch.id, None,
            "created", Some("Settlement batch created"),
            None, Some("draft"), created_by, None, None,
        ).await;

        Ok(batch)
    }

    /// Get a batch by ID
    pub async fn get_batch(&self, id: Uuid) -> AtlasResult<Option<SettlementBatch>> {
        self.repo.get_batch(id).await
    }

    /// Get a batch by number
    pub async fn get_batch_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<SettlementBatch>> {
        self.repo.get_batch_by_number(org_id, number).await
    }

    /// List batches with optional status filter
    pub async fn list_batches(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<SettlementBatch>> {
        if let Some(s) = status {
            Self::validate_batch_status(s)?;
        }
        self.repo.list_batches(org_id, status).await
    }

    /// Delete a draft batch
    pub async fn delete_batch(&self, org_id: Uuid, number: &str) -> AtlasResult<()> {
        info!("Deleting settlement batch '{}' for org {}", number, org_id);

        let batch = self.repo.get_batch_by_number(org_id, number).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Settlement batch not found".to_string()))?;

        if batch.status != "draft" {
            return Err(AtlasError::ValidationFailed(
                "Only draft batches can be deleted".to_string()
            ));
        }

        self.repo.delete_batch(org_id, number).await
    }

    // ========================================================================
    // Lifecycle Transitions
    // ========================================================================

    /// Submit a draft batch for approval
    pub async fn submit_batch(
        &self,
        id: Uuid,
        submitted_by: Option<Uuid>,
    ) -> AtlasResult<SettlementBatch> {
        info!("Submitting settlement batch {}", id);

        let batch = self.repo.get_batch(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Settlement batch not found".to_string()))?;

        if batch.status != "draft" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot submit batch in '{}' status. Must be 'draft'.", batch.status
            )));
        }

        // Must have at least one pending line
        let lines = self.repo.list_lines(id).await?;
        let pending = lines.iter().filter(|l| l.status == "pending").count();
        if pending == 0 {
            return Err(AtlasError::ValidationFailed(
                "Batch must have at least one pending line before submission".to_string()
            ));
        }

        let _ = self.repo.update_batch_submission(id, submitted_by).await?;
        let batch = self.repo.update_batch_status(id, "submitted").await?;

        let _ = self.repo.create_activity(
            batch.organization_id, id, None,
            "submitted", Some("Batch submitted for approval"),
            Some("draft"), Some("submitted"), submitted_by, None, None,
        ).await;

        Ok(batch)
    }

    /// Approve a submitted batch
    pub async fn approve_batch(
        &self,
        id: Uuid,
        approved_by: Option<Uuid>,
    ) -> AtlasResult<SettlementBatch> {
        info!("Approving settlement batch {}", id);

        let batch = self.repo.get_batch(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Settlement batch not found".to_string()))?;

        Self::validate_status_transition(&batch.status, "approved")?;

        let _ = self.repo.update_batch_approval(id, approved_by).await?;
        let batch = self.repo.update_batch_status(id, "approved").await?;

        let _ = self.repo.create_activity(
            batch.organization_id, id, None,
            "approved", Some("Batch approved"),
            Some("submitted"), Some("approved"), approved_by, None, None,
        ).await;

        Ok(batch)
    }

    /// Settle an approved batch — marks all lines as settled
    pub async fn settle_batch(
        &self,
        id: Uuid,
        settled_by: Option<Uuid>,
    ) -> AtlasResult<SettlementBatch> {
        info!("Settling settlement batch {}", id);

        let batch = self.repo.get_batch(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Settlement batch not found".to_string()))?;

        Self::validate_status_transition(&batch.status, "settled")?;

        // Mark all pending lines as settled
        let lines = self.repo.list_lines(id).await?;
        for line in lines.iter().filter(|l| l.status == "pending") {
            let _ = self.repo.update_line_status(line.id, "settled", None).await;

            let _ = self.repo.create_activity(
                batch.organization_id, id, Some(line.id),
                "line_settled", Some(&format!("Invoice {} settled for {:.2}",
                    line.invoice_number.as_deref().unwrap_or("?"), line.amount_paid)),
                Some("pending"), Some("settled"), settled_by, None, None,
            ).await;
        }

        let _ = self.repo.update_batch_settlement(id, settled_by).await?;
        let batch = self.repo.update_batch_status(id, "settled").await?;

        let _ = self.repo.create_activity(
            batch.organization_id, id, None,
            "settled", Some("Batch settled"),
            Some("approved"), Some("settled"), settled_by, None, None,
        ).await;

        Ok(batch)
    }

    /// Cancel a batch (only from draft, submitted, or approved)
    pub async fn cancel_batch(
        &self,
        id: Uuid,
        cancelled_by: Option<Uuid>,
        reason: Option<&str>,
    ) -> AtlasResult<SettlementBatch> {
        info!("Cancelling settlement batch {}: {}", id, reason.unwrap_or("No reason"));

        let batch = self.repo.get_batch(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Settlement batch not found".to_string()))?;

        Self::validate_status_transition(&batch.status, "cancelled")?;

        // Cancel all pending lines
        let lines = self.repo.list_lines(id).await?;
        for line in lines.iter().filter(|l| l.status == "pending") {
            let _ = self.repo.update_line_status(line.id, "cancelled", None).await;
        }

        let _ = self.repo.update_batch_cancellation(id, cancelled_by, reason).await?;
        let batch = self.repo.update_batch_status(id, "cancelled").await?;

        let _ = self.repo.create_activity(
            batch.organization_id, id, None,
            "cancelled", Some(&format!("Batch cancelled: {}", reason.unwrap_or("No reason"))),
            None, Some("cancelled"), cancelled_by, None, None,
        ).await;

        Ok(batch)
    }

    // ========================================================================
    // Settlement Lines
    // ========================================================================

    /// Add a settlement line to a batch
    pub async fn add_line(
        &self,
        org_id: Uuid,
        batch_id: Uuid,
        invoice_id: Uuid,
        invoice_number: Option<&str>,
        invoice_date: Option<chrono::NaiveDate>,
        invoice_amount: f64,
        supplier_id: Option<Uuid>,
        supplier_number: Option<&str>,
        supplier_name: Option<&str>,
        supplier_site: Option<&str>,
        original_amount: f64,
        amount_due: f64,
        amount_paid: f64,
        discount_available: f64,
        discount_taken: f64,
        discount_date: Option<chrono::NaiveDate>,
        bank_charges: f64,
        adjustment_amount: f64,
        adjustment_reason: Option<&str>,
        line_settlement_type: &str,
        liability_account: Option<&str>,
        discount_account: Option<&str>,
        charges_account: Option<&str>,
    ) -> AtlasResult<SettlementLine> {
        info!("Adding settlement line for invoice {:?} to batch {}", invoice_number, batch_id);

        // Validate settlement type
        if !VALID_SETTLEMENT_TYPES.contains(&line_settlement_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid line settlement_type '{}'. Must be one of: {}",
                line_settlement_type, VALID_SETTLEMENT_TYPES.join(", ")
            )));
        }

        // Verify batch exists and is editable
        let batch = self.repo.get_batch(batch_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Settlement batch not found".to_string()))?;

        if batch.status != "draft" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot add lines to batch in '{}' status. Must be 'draft'.", batch.status
            )));
        }

        // Validate amounts
        if amount_paid <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Amount paid must be greater than zero".to_string()
            ));
        }

        if amount_paid > amount_due + 0.01 {
            return Err(AtlasError::ValidationFailed(
                "Amount paid cannot exceed amount due".to_string()
            ));
        }

        if discount_taken > discount_available + 0.01 {
            return Err(AtlasError::ValidationFailed(
                "Discount taken cannot exceed discount available".to_string()
            ));
        }

        if bank_charges < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Bank charges cannot be negative".to_string()
            ));
        }

        let params = SettlementLineCreateParams {
            org_id,
            batch_id,
            invoice_id,
            invoice_number: invoice_number.map(std::string::ToString::to_string),
            invoice_date,
            invoice_amount,
            supplier_id,
            supplier_number: supplier_number.map(std::string::ToString::to_string),
            supplier_name: supplier_name.map(std::string::ToString::to_string),
            supplier_site: supplier_site.map(std::string::ToString::to_string),
            original_amount,
            amount_due,
            amount_paid,
            discount_available,
            discount_taken,
            discount_date,
            bank_charges,
            adjustment_amount,
            adjustment_reason: adjustment_reason.map(std::string::ToString::to_string),
            settlement_type: line_settlement_type.to_string(),
            liability_account: liability_account.map(std::string::ToString::to_string),
            discount_account: discount_account.map(std::string::ToString::to_string),
            charges_account: charges_account.map(std::string::ToString::to_string),
        };

        let line = self.repo.create_line(&params).await?;

        // Recalculate batch totals
        self.recalculate_batch_totals(batch_id).await?;

        // Log activity
        let _ = self.repo.create_activity(
            org_id, batch_id, Some(line.id),
            "line_added", Some(&format!("Settlement line added for invoice {}",
                invoice_number.unwrap_or("?"))),
            None, None, None, None, None,
        ).await;

        Ok(line)
    }

    /// Get a line by ID
    pub async fn get_line(&self, id: Uuid) -> AtlasResult<Option<SettlementLine>> {
        self.repo.get_line(id).await
    }

    /// List lines for a batch
    pub async fn list_lines(&self, batch_id: Uuid) -> AtlasResult<Vec<SettlementLine>> {
        self.repo.list_lines(batch_id).await
    }

    /// Remove a line from a draft batch
    pub async fn remove_line(&self, batch_id: Uuid, line_id: Uuid) -> AtlasResult<()> {
        info!("Removing line {} from batch {}", line_id, batch_id);

        let batch = self.repo.get_batch(batch_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Settlement batch not found".to_string()))?;

        if batch.status != "draft" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot remove lines from batch in '{}' status", batch.status
            )));
        }

        self.repo.delete_line(batch_id, line_id).await?;

        // Recalculate totals
        self.recalculate_batch_totals(batch_id).await?;

        Ok(())
    }

    // ========================================================================
    // Activities
    // ========================================================================

    /// List activities for a batch
    pub async fn list_activities(&self, batch_id: Uuid) -> AtlasResult<Vec<SettlementActivity>> {
        self.repo.list_activities(batch_id).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get settlement dashboard summary
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<SettlementSummary> {
        self.repo.get_dashboard(org_id).await
    }

    // ========================================================================
    // Helpers
    // ========================================================================

    /// Recalculate batch totals from lines
    async fn recalculate_batch_totals(&self, batch_id: Uuid) -> AtlasResult<()> {
        let lines = self.repo.list_lines(batch_id).await?;
        let pending: Vec<&SettlementLine> = lines.iter().filter(|l| l.status == "pending").collect();

        let total_invoices = pending.len() as i32;
        let total_invoice_amount: f64 = pending.iter().map(|l| l.invoice_amount).sum();
        let total_discount_taken: f64 = pending.iter().map(|l| l.discount_taken).sum();
        let total_settled_amount: f64 = pending.iter().map(|l| l.amount_paid).sum();
        let total_charges: f64 = pending.iter().map(|l| l.bank_charges).sum();
        let total_net_payment: f64 = pending.iter().map(|l| l.net_settlement).sum();

        self.repo.update_batch_totals(
            batch_id,
            total_invoices,
            total_invoice_amount,
            total_discount_taken,
            total_settled_amount,
            total_charges,
            total_net_payment,
        ).await?;

        Ok(())
    }

    // ========================================================================
    // Exported validation functions for handler use
    // ========================================================================

    #[must_use] 
    pub const fn valid_settlement_methods() -> &'static [&'static str] { VALID_SETTLEMENT_METHODS }
    #[must_use] 
    pub const fn valid_settlement_types() -> &'static [&'static str] { VALID_SETTLEMENT_TYPES }
    #[must_use] 
    pub const fn valid_batch_statuses() -> &'static [&'static str] { VALID_BATCH_STATUSES }
}

// Re-export for convenience
use super::repository::SettlementActivity;
