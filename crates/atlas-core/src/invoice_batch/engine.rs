//! AP Invoice Batch Engine
//!
//! Orchestrates invoice batch creation, lifecycle transitions, validation,
//! activity tracking, and dashboard summary.
//!
//! Oracle Fusion Cloud ERP equivalent: Financials > Payables > Invoice Batches

use atlas_shared::{AtlasError, AtlasResult};
use super::repository::{
    InvoiceBatchRepository, InvoiceBatch,
};
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

// Valid statuses
const VALID_BATCH_STATUSES: &[&str] = &[
    "draft", "submitted", "approved", "posted", "cancelled",
];

const VALID_SOURCES: &[&str] = &[
    "manual", "import", "edi", "project", "expense", "other",
];

const _VALID_CURRENCIES: &[&str] = &[
    "USD", "EUR", "GBP", "JPY", "CAD", "AUD", "CHF", "CNY", "INR", "BRL", "MXN",
];

/// AP Invoice Batch Engine
pub struct InvoiceBatchEngine {
    repo: Arc<dyn InvoiceBatchRepository>,
}

impl InvoiceBatchEngine {
    pub fn new(repo: Arc<dyn InvoiceBatchRepository>) -> Self {
        Self { repo }
    }

    // ========================================================================
    // Validation
    // ========================================================================

    fn validate_source(source: &str) -> AtlasResult<()> {
        if !VALID_SOURCES.contains(&source) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid source '{}'. Must be one of: {}", source, VALID_SOURCES.join(", ")
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
    /// draft → submitted → approved → posted
    /// draft → cancelled
    /// submitted → cancelled
    /// approved → cancelled
    pub fn validate_status_transition(current: &str, target: &str) -> AtlasResult<()> {
        match (current, target) {
            ("draft", "submitted") => Ok(()),
            ("draft", "cancelled") => Ok(()),
            ("submitted", "approved") => Ok(()),
            ("submitted", "cancelled") => Ok(()),
            ("approved", "posted") => Ok(()),
            ("approved", "cancelled") => Ok(()),
            _ => Err(AtlasError::WorkflowError(format!(
                "Invalid status transition from '{current}' to '{target}'. \
                 Valid: draft→submitted, draft→cancelled, submitted→approved, \
                 submitted→cancelled, approved→posted, approved→cancelled"
            ))),
        }
    }

    fn generate_batch_number() -> String {
        format!("IB-{}", chrono::Utc::now().format("%Y%m%d-%H%M%S%-3f"))
    }

    // ========================================================================
    // Batch CRUD
    // ========================================================================

    /// Create a new invoice batch
    pub async fn create_batch(
        &self,
        org_id: Uuid,
        batch_name: &str,
        description: Option<&str>,
        currency_code: &str,
        exchange_rate_type: Option<&str>,
        exchange_rate: Option<f64>,
        gl_date: Option<chrono::NaiveDate>,
        accounting_period: Option<&str>,
        source: &str,
        control_total: Option<f64>,
        control_count: Option<i32>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<InvoiceBatch> {
        info!("Creating invoice batch '{}' for org {}", batch_name, org_id);

        Self::validate_source(source)?;

        if batch_name.trim().is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Batch name is required".to_string(),
            ));
        }

        if let Some(ct) = control_total {
            if ct < 0.0 {
                return Err(AtlasError::ValidationFailed(
                    "Control total cannot be negative".to_string(),
                ));
            }
        }

        if let Some(cc) = control_count {
            if cc < 0 {
                return Err(AtlasError::ValidationFailed(
                    "Control count cannot be negative".to_string(),
                ));
            }
        }

        let batch_number = Self::generate_batch_number();

        let batch = self.repo.create_batch(
            org_id,
            &batch_number,
            batch_name,
            description,
            currency_code,
            exchange_rate_type,
            exchange_rate,
            gl_date,
            accounting_period,
            source,
            control_total,
            control_count,
            created_by,
        ).await?;

        // Log creation activity
        self.repo.add_activity(
            batch.id,
            "created",
            Some(&format!("Batch '{batch_name}' created")),
            None,
            Some("draft"),
            created_by,
            None,
        ).await.ok();

        Ok(batch)
    }

    /// Get a batch by ID
    pub async fn get_batch(&self, org_id: Uuid, id: Uuid) -> AtlasResult<InvoiceBatch> {
        self.repo.get_batch(org_id, id).await
    }

    /// Get a batch by batch number
    pub async fn get_batch_by_number(&self, org_id: Uuid, batch_number: &str) -> AtlasResult<InvoiceBatch> {
        self.repo.get_batch_by_number(org_id, batch_number).await
    }

    /// List batches with optional status filter
    pub async fn list_batches(
        &self,
        org_id: Uuid,
        status: Option<&str>,
    ) -> AtlasResult<Vec<InvoiceBatch>> {
        if let Some(s) = status {
            Self::validate_batch_status(s)?;
        }
        self.repo.list_batches(org_id, status).await
    }

    /// Delete a draft batch by number
    pub async fn delete_batch(&self, org_id: Uuid, batch_number: &str) -> AtlasResult<()> {
        info!("Deleting invoice batch '{}' for org {}", batch_number, org_id);
        self.repo.delete_batch(org_id, batch_number).await
    }

    // ========================================================================
    // Lifecycle Transitions
    // ========================================================================

    /// Submit a draft batch for approval
    pub async fn submit_batch(
        &self,
        org_id: Uuid,
        id: Uuid,
        submitted_by: Uuid,
    ) -> AtlasResult<InvoiceBatch> {
        info!("Submitting invoice batch {} for approval", id);

        let batch = self.repo.get_batch(org_id, id).await?;
        Self::validate_status_transition(&batch.status, "submitted")?;

        // Validate batch has invoices
        if batch.total_invoice_count == 0 {
            return Err(AtlasError::ValidationFailed(
                "Cannot submit batch with no invoices".to_string(),
            ));
        }

        // Validate against control totals if set
        let mut validation_errors: Vec<String> = vec![];
        if let Some(ct) = batch.control_total {
            if (batch.total_amount - ct).abs() > 0.01 {
                validation_errors.push(format!(
                    "Batch total ({:.2}) does not match control total ({:.2})",
                    batch.total_amount, ct
                ));
            }
        }
        if let Some(cc) = batch.control_count {
            if batch.total_invoice_count != cc {
                validation_errors.push(format!(
                    "Invoice count ({}) does not match control count ({})",
                    batch.total_invoice_count, cc
                ));
            }
        }

        if !validation_errors.is_empty() {
            return Err(AtlasError::ValidationFailed(
                format!("Batch validation failed: {}", validation_errors.join("; "))
            ));
        }

        let old_status = batch.status.clone();
        let updated = self.repo.set_submitted(id, submitted_by).await?;

        self.repo.add_activity(
            id,
            "submitted",
            Some("Batch submitted for approval"),
            Some(&old_status),
            Some("submitted"),
            Some(submitted_by),
            None,
        ).await.ok();

        Ok(updated)
    }

    /// Approve a submitted batch
    pub async fn approve_batch(
        &self,
        org_id: Uuid,
        id: Uuid,
        approved_by: Uuid,
    ) -> AtlasResult<InvoiceBatch> {
        info!("Approving invoice batch {}", id);

        let batch = self.repo.get_batch(org_id, id).await?;
        Self::validate_status_transition(&batch.status, "approved")?;

        let old_status = batch.status.clone();
        let updated = self.repo.set_approved(id, approved_by).await?;

        self.repo.add_activity(
            id,
            "approved",
            Some("Batch approved"),
            Some(&old_status),
            Some("approved"),
            Some(approved_by),
            None,
        ).await.ok();

        Ok(updated)
    }

    /// Post an approved batch to the GL
    pub async fn post_batch(
        &self,
        org_id: Uuid,
        id: Uuid,
        posted_by: Uuid,
    ) -> AtlasResult<InvoiceBatch> {
        info!("Posting invoice batch {} to GL", id);

        let batch = self.repo.get_batch(org_id, id).await?;
        Self::validate_status_transition(&batch.status, "posted")?;

        let old_status = batch.status.clone();
        let updated = self.repo.set_posted(id, posted_by).await?;

        self.repo.add_activity(
            id,
            "posted",
            Some(&format!("Batch posted to GL ({} invoices, {:.2} total)", batch.total_invoice_count, batch.total_amount)),
            Some(&old_status),
            Some("posted"),
            Some(posted_by),
            None,
        ).await.ok();

        Ok(updated)
    }

    /// Cancel a batch (from draft, submitted, or approved)
    pub async fn cancel_batch(
        &self,
        org_id: Uuid,
        id: Uuid,
        cancelled_by: Uuid,
        reason: Option<&str>,
    ) -> AtlasResult<InvoiceBatch> {
        info!("Cancelling invoice batch {}", id);

        let batch = self.repo.get_batch(org_id, id).await?;
        Self::validate_status_transition(&batch.status, "cancelled")?;

        let old_status = batch.status.clone();
        let updated = self.repo.set_cancelled(id, cancelled_by, reason).await?;

        self.repo.add_activity(
            id,
            "cancelled",
            Some(&format!("Batch cancelled{}", reason.map(|r| format!(": {r}")).unwrap_or_default())),
            Some(&old_status),
            Some("cancelled"),
            Some(cancelled_by),
            reason.map(|r| serde_json::json!({"reason": r})),
        ).await.ok();

        Ok(updated)
    }

    // ========================================================================
    // Totals Management
    // ========================================================================

    /// Add an invoice to batch totals
    pub async fn add_invoice_to_totals(
        &self,
        batch_id: Uuid,
        invoice_amount: f64,
        tax_amount: f64,
    ) -> AtlasResult<InvoiceBatch> {
        let batch = self.repo.recalculate_totals(batch_id).await?;

        let new_count = batch.total_invoice_count + 1;
        let new_invoice_amount = batch.total_invoice_amount + invoice_amount;
        let new_tax_amount = batch.total_tax_amount + tax_amount;
        let new_total = new_invoice_amount + new_tax_amount;

        self.repo.update_totals(
            batch_id,
            new_count,
            new_invoice_amount,
            new_tax_amount,
            new_total,
        ).await?;

        self.repo.add_activity(
            batch_id,
            "invoice_added",
            Some(&format!("Invoice added (amount: {invoice_amount:.2}, tax: {tax_amount:.2})")),
            None,
            None,
            None,
            Some(serde_json::json!({"invoice_amount": invoice_amount, "tax_amount": tax_amount})),
        ).await.ok();

        self.repo.recalculate_totals(batch_id).await
    }

    /// Remove an invoice from batch totals
    pub async fn remove_invoice_from_totals(
        &self,
        batch_id: Uuid,
        invoice_amount: f64,
        tax_amount: f64,
    ) -> AtlasResult<InvoiceBatch> {
        let batch = self.repo.recalculate_totals(batch_id).await?;

        let new_count = (batch.total_invoice_count - 1).max(0);
        let new_invoice_amount = batch.total_invoice_amount - invoice_amount;
        let new_tax_amount = batch.total_tax_amount - tax_amount;
        let new_total = new_invoice_amount + new_tax_amount;

        self.repo.update_totals(
            batch_id,
            new_count,
            new_invoice_amount,
            new_tax_amount,
            new_total,
        ).await?;

        self.repo.add_activity(
            batch_id,
            "invoice_removed",
            Some(&format!("Invoice removed (amount: {invoice_amount:.2})")),
            None,
            None,
            None,
            Some(serde_json::json!({"invoice_amount": invoice_amount, "tax_amount": tax_amount})),
        ).await.ok();

        self.repo.recalculate_totals(batch_id).await
    }

    // ========================================================================
    // Activities & Dashboard
    // ========================================================================

    /// List all activities for a batch
    pub async fn list_activities(&self, batch_id: Uuid) -> AtlasResult<Vec<super::repository::InvoiceBatchActivity>> {
        self.repo.list_activities(batch_id).await
    }

    /// Get dashboard summary
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<super::repository::InvoiceBatchSummary> {
        self.repo.get_dashboard(org_id).await
    }

    /// Validate a batch (check control totals)
    pub async fn validate_batch(&self, org_id: Uuid, id: Uuid) -> AtlasResult<InvoiceBatch> {
        let batch = self.repo.get_batch(org_id, id).await?;

        let mut errors: Vec<String> = vec![];

        if let Some(ct) = batch.control_total {
            if (batch.total_amount - ct).abs() > 0.01 {
                errors.push(format!(
                    "Total ({:.2}) != control total ({:.2})",
                    batch.total_amount, ct
                ));
            }
        }

        if let Some(cc) = batch.control_count {
            if batch.total_invoice_count != cc {
                errors.push(format!(
                    "Count ({}) != control count ({})",
                    batch.total_invoice_count, cc
                ));
            }
        }

        let vs = if errors.is_empty() { "valid" } else { "invalid" };
        let ve = serde_json::to_value(&errors).unwrap_or(serde_json::json!([]));

        self.repo.update_status(id, &batch.status, Some(vs), Some(ve)).await
    }
}
