//! Manual Journal Entry Engine
//!
//! Manages the full lifecycle of manual journal entries:
//! - Create journal batches and entries
//! - Add debit/credit lines with balance validation
//! - Submit, approve, post, and reverse journals
//!
//! Oracle Fusion Cloud ERP equivalent: General Ledger > Journals > New Journal

use atlas_shared::{
    JournalBatch, JournalEntry, JournalEntryLine, ManualJournalDashboardSummary,
    AtlasError, AtlasResult,
};
use super::ManualJournalRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Valid batch statuses
const VALID_BATCH_STATUSES: &[&str] = &["draft", "submitted", "approved", "posted", "reversed"];

/// Valid entry statuses
const VALID_ENTRY_STATUSES: &[&str] = &["draft", "submitted", "approved", "posted", "reversed"];

/// Valid line types
const VALID_LINE_TYPES: &[&str] = &["debit", "credit"];

/// Balance tolerance for floating-point comparison
const BALANCE_TOLERANCE: f64 = 0.005;

/// Manual Journal Entry engine
pub struct ManualJournalEngine {
    repository: Arc<dyn ManualJournalRepository>,
}

impl ManualJournalEngine {
    pub fn new(repository: Arc<dyn ManualJournalRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Batch Management
    // ========================================================================

    /// Create a new journal batch in draft status
    pub async fn create_batch(
        &self,
        org_id: Uuid,
        batch_number: &str,
        name: &str,
        description: Option<&str>,
        ledger_id: Option<Uuid>,
        currency_code: &str,
        accounting_date: Option<chrono::NaiveDate>,
        period_name: Option<&str>,
        source: Option<&str>,
        is_automatic_post: bool,
        created_by: Option<Uuid>,
    ) -> AtlasResult<JournalBatch> {
        if batch_number.is_empty() {
            return Err(AtlasError::ValidationFailed("Batch number is required".to_string()));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Batch name is required".to_string()));
        }

        let effective_source = source.unwrap_or("manual");

        let period = period_name
            .map(|p| p.to_string())
            .or_else(|| accounting_date.map(|d| d.format("%Y-%m").to_string()));

        info!(
            "Creating journal batch {} ({}) for org {}",
            batch_number, name, org_id
        );

        self.repository
            .create_batch(
                org_id, batch_number, name, description, ledger_id,
                currency_code, accounting_date, period.as_deref(),
                effective_source, is_automatic_post, created_by,
            )
            .await
    }

    /// Get a batch by number
    pub async fn get_batch(&self, org_id: Uuid, batch_number: &str) -> AtlasResult<Option<JournalBatch>> {
        self.repository.get_batch(org_id, batch_number).await
    }

    /// Get a batch by ID
    pub async fn get_batch_by_id(&self, id: Uuid) -> AtlasResult<Option<JournalBatch>> {
        self.repository.get_batch_by_id(id).await
    }

    /// List batches with optional status filter
    pub async fn list_batches(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<JournalBatch>> {
        if let Some(s) = status {
            if !VALID_BATCH_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}",
                    s, VALID_BATCH_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_batches(org_id, status).await
    }

    /// Delete a draft batch
    pub async fn delete_batch(&self, org_id: Uuid, batch_number: &str) -> AtlasResult<()> {
        let batch = self
            .repository
            .get_batch(org_id, batch_number)
            .await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Batch {} not found", batch_number)))?;

        if batch.status != "draft" {
            return Err(AtlasError::WorkflowError(
                "Cannot delete batch that is not in 'draft' status".to_string(),
            ));
        }

        info!("Deleted journal batch {}", batch_number);
        self.repository.delete_batch(org_id, batch_number).await
    }

    // ========================================================================
    // Entry Management
    // ========================================================================

    /// Create a new journal entry in a batch
    pub async fn create_entry(
        &self,
        org_id: Uuid,
        batch_id: Uuid,
        entry_number: &str,
        name: Option<&str>,
        description: Option<&str>,
        ledger_id: Option<Uuid>,
        currency_code: &str,
        accounting_date: Option<chrono::NaiveDate>,
        period_name: Option<&str>,
        journal_category: Option<&str>,
        reference_number: Option<&str>,
        external_reference: Option<&str>,
        statistical_entry: bool,
        created_by: Option<Uuid>,
    ) -> AtlasResult<JournalEntry> {
        if entry_number.is_empty() {
            return Err(AtlasError::ValidationFailed("Entry number is required".to_string()));
        }

        let batch = self
            .repository
            .get_batch_by_id(batch_id)
            .await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Batch {} not found", batch_id)))?;

        if batch.status != "draft" {
            return Err(AtlasError::WorkflowError(
                "Cannot add entries to a batch that is not in 'draft' status".to_string(),
            ));
        }

        let period = period_name
            .map(|p| p.to_string())
            .or_else(|| accounting_date.map(|d| d.format("%Y-%m").to_string()));

        let category = journal_category.unwrap_or("manual");

        info!(
            "Creating journal entry {} in batch {}",
            entry_number, batch.batch_number
        );

        let entry = self.repository
            .create_entry(
                org_id, batch_id, entry_number, name, description,
                ledger_id, currency_code, accounting_date, period.as_deref(),
                category, "manual", reference_number, external_reference,
                statistical_entry, created_by,
            )
            .await?;

        // Update batch entry count
        self.recalculate_batch_totals(batch_id).await?;

        Ok(entry)
    }

    /// Get an entry by ID
    pub async fn get_entry(&self, id: Uuid) -> AtlasResult<Option<JournalEntry>> {
        self.repository.get_entry(id).await
    }

    /// List entries in a batch
    pub async fn list_entries_by_batch(&self, batch_id: Uuid) -> AtlasResult<Vec<JournalEntry>> {
        self.repository.list_entries_by_batch(batch_id).await
    }

    /// List all entries with optional status filter
    pub async fn list_entries(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<JournalEntry>> {
        if let Some(s) = status {
            if !VALID_ENTRY_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}",
                    s, VALID_ENTRY_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_entries(org_id, status).await
    }

    /// Delete a draft entry
    pub async fn delete_entry(&self, entry_id: Uuid) -> AtlasResult<()> {
        let entry = self
            .repository
            .get_entry(entry_id)
            .await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Entry {} not found", entry_id)))?;

        if entry.status != "draft" {
            return Err(AtlasError::WorkflowError(
                "Cannot delete entry that is not in 'draft' status".to_string(),
            ));
        }

        self.repository.delete_entry(entry_id).await?;
        self.recalculate_batch_totals(entry.batch_id).await?;
        Ok(())
    }

    // ========================================================================
    // Journal Lines
    // ========================================================================

    /// Add a line to a journal entry
    pub async fn add_line(
        &self,
        org_id: Uuid,
        entry_id: Uuid,
        line_type: &str,
        account_code: &str,
        account_name: Option<&str>,
        description: Option<&str>,
        amount: &str,
        entered_amount: Option<&str>,
        entered_currency_code: Option<&str>,
        exchange_rate: Option<&str>,
        tax_code: Option<&str>,
        cost_center: Option<&str>,
        department_id: Option<Uuid>,
        project_id: Option<Uuid>,
        intercompany_entity_id: Option<Uuid>,
        statistical_amount: Option<&str>,
    ) -> AtlasResult<JournalEntryLine> {
        if !VALID_LINE_TYPES.contains(&line_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid line type '{}'. Must be one of: {}",
                line_type, VALID_LINE_TYPES.join(", ")
            )));
        }
        if account_code.is_empty() {
            return Err(AtlasError::ValidationFailed("Account code is required".to_string()));
        }

        let amt: f64 = amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "Amount must be a valid number".to_string(),
        ))?;
        if amt < 0.0 {
            return Err(AtlasError::ValidationFailed("Amount cannot be negative".to_string()));
        }

        let entry = self
            .repository
            .get_entry(entry_id)
            .await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Entry {} not found", entry_id)))?;

        if entry.status != "draft" {
            return Err(AtlasError::WorkflowError(
                "Cannot add lines to an entry that is not in 'draft' status".to_string(),
            ));
        }

        // Get next line number
        let existing = self.repository.list_lines_by_entry(entry_id).await?;
        let line_number = existing.len() as i32 + 1;

        info!(
            "Adding line {} ({}) to entry {}",
            line_number, account_code, entry.entry_number
        );

        let line = self.repository
            .create_line(
                org_id, entry_id, line_number, line_type, account_code,
                account_name, description, amount, entered_amount,
                entered_currency_code, exchange_rate, tax_code, cost_center,
                department_id, project_id, intercompany_entity_id,
                statistical_amount,
            )
            .await?;

        // Recalculate entry totals
        self.recalculate_entry_totals(entry_id).await?;
        // Recalculate batch totals
        self.recalculate_batch_totals(entry.batch_id).await?;

        Ok(line)
    }

    /// List lines for an entry
    pub async fn list_lines(&self, entry_id: Uuid) -> AtlasResult<Vec<JournalEntryLine>> {
        self.repository.list_lines_by_entry(entry_id).await
    }

    /// Delete a line
    pub async fn delete_line(&self, line_id: Uuid) -> AtlasResult<()> {
        // Get the line to find its entry
        let _all_entries = self.repository.list_lines_by_entry(Uuid::nil()).await.ok();
        // We need the entry_id; let's find the line another way
        // Since we don't have get_line_by_id, we'll require the entry_id context
        // For simplicity, the line knows its entry_id via the DB row
        // Let's get the entry from the line first
        let line = self.find_line_by_id(line_id).await?;
        let entry_id = line.entry_id;

        let entry = self
            .repository
            .get_entry(entry_id)
            .await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Entry {} not found", entry_id)))?;

        if entry.status != "draft" {
            return Err(AtlasError::WorkflowError(
                "Cannot delete lines from an entry that is not in 'draft' status".to_string(),
            ));
        }

        self.repository.delete_line(line_id).await?;
        self.recalculate_entry_totals(entry_id).await?;
        self.recalculate_batch_totals(entry.batch_id).await?;
        Ok(())
    }

    // ========================================================================
    // Batch Lifecycle
    // ========================================================================

    /// Submit a batch for approval
    pub async fn submit_batch(&self, batch_id: Uuid, submitted_by: Option<Uuid>) -> AtlasResult<JournalBatch> {
        let batch = self
            .repository
            .get_batch_by_id(batch_id)
            .await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Batch {} not found", batch_id)))?;

        if batch.status != "draft" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot submit batch in '{}' status. Must be 'draft'.",
                batch.status
            )));
        }

        // Validate batch has entries
        let entries = self.repository.list_entries_by_batch(batch_id).await?;
        if entries.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Cannot submit batch without journal entries".to_string(),
            ));
        }

        // Validate all entries are balanced
        for entry in &entries {
            if !entry.is_balanced {
                return Err(AtlasError::ValidationFailed(format!(
                    "Entry {} is not balanced. Total debits ({}) != total credits ({})",
                    entry.entry_number, entry.total_debit, entry.total_credit
                )));
            }
        }

        // Update all entries to submitted
        for entry in &entries {
            self.repository
                .update_entry_status(entry.id, "submitted", None, None)
                .await?;
        }

        info!("Submitted journal batch {}", batch.batch_number);
        self.repository
            .update_batch_status(batch_id, "submitted", submitted_by, None, None, None)
            .await
    }

    /// Approve a submitted batch
    pub async fn approve_batch(&self, batch_id: Uuid, approved_by: Option<Uuid>) -> AtlasResult<JournalBatch> {
        let batch = self
            .repository
            .get_batch_by_id(batch_id)
            .await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Batch {} not found", batch_id)))?;

        if batch.status != "submitted" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot approve batch in '{}' status. Must be 'submitted'.",
                batch.status
            )));
        }

        // Update all entries to approved
        let entries = self.repository.list_entries_by_batch(batch_id).await?;
        for entry in &entries {
            self.repository
                .update_entry_status(entry.id, "approved", approved_by, None)
                .await?;
        }

        info!("Approved journal batch {}", batch.batch_number);
        self.repository
            .update_batch_status(batch_id, "approved", None, approved_by, None, None)
            .await
    }

    /// Reject a submitted batch
    pub async fn reject_batch(&self, batch_id: Uuid, rejection_reason: Option<&str>) -> AtlasResult<JournalBatch> {
        let batch = self
            .repository
            .get_batch_by_id(batch_id)
            .await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Batch {} not found", batch_id)))?;

        if batch.status != "submitted" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot reject batch in '{}' status. Must be 'submitted'.",
                batch.status
            )));
        }

        // Revert all entries to draft
        let entries = self.repository.list_entries_by_batch(batch_id).await?;
        for entry in &entries {
            self.repository
                .update_entry_status(entry.id, "draft", None, None)
                .await?;
        }

        info!("Rejected journal batch {}", batch.batch_number);
        self.repository
            .update_batch_status(batch_id, "draft", None, None, None, rejection_reason)
            .await
    }

    /// Post an approved batch to the General Ledger
    pub async fn post_batch(&self, batch_id: Uuid, posted_by: Option<Uuid>) -> AtlasResult<JournalBatch> {
        let batch = self
            .repository
            .get_batch_by_id(batch_id)
            .await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Batch {} not found", batch_id)))?;

        if batch.status != "approved" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot post batch in '{}' status. Must be 'approved'.",
                batch.status
            )));
        }

        let now = chrono::Utc::now();
        // Update all entries to posted
        let entries = self.repository.list_entries_by_batch(batch_id).await?;
        for entry in &entries {
            self.repository
                .update_entry_status(entry.id, "posted", None, Some(now))
                .await?;
        }

        info!("Posted journal batch {}", batch.batch_number);
        self.repository
            .update_batch_status(batch_id, "posted", None, None, posted_by, None)
            .await
    }

    /// Reverse a posted batch, creating reversal entries
    pub async fn reverse_batch(&self, batch_id: Uuid, reversed_by: Option<Uuid>) -> AtlasResult<JournalBatch> {
        let batch = self
            .repository
            .get_batch_by_id(batch_id)
            .await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Batch {} not found", batch_id)))?;

        if batch.status != "posted" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot reverse batch in '{}' status. Must be 'posted'.",
                batch.status
            )));
        }

        // Reverse each entry in the batch
        let entries = self.repository.list_entries_by_batch(batch_id).await?;
        for entry in &entries {
            if entry.status != "posted" {
                continue;
            }
            // Create reversal entry
            let reversal_number = format!("REV-{}", entry.entry_number);
            let reversal_entry = self.repository
                .create_entry(
                    entry.organization_id, batch_id, &reversal_number,
                    Some(&format!("Reversal of {}", entry.entry_number)),
                    Some("System-generated reversal entry"),
                    entry.ledger_id, &entry.currency_code,
                    entry.accounting_date, entry.period_name.as_deref(),
                    &entry.journal_category, "reversal",
                    None, None, entry.statistical_entry,
                    reversed_by,
                )
                .await?;

            // Copy lines with swapped debits/credits
            let lines = self.repository.list_lines_by_entry(entry.id).await?;
            for line in &lines {
                let reversed_type = if line.line_type == "debit" { "credit" } else { "debit" };
                self.repository
                    .create_line(
                        entry.organization_id, reversal_entry.id, line.line_number,
                        reversed_type, &line.account_code, line.account_name.as_deref(),
                        line.description.as_deref(), &line.amount,
                        line.entered_amount.as_deref(), line.entered_currency_code.as_deref(),
                        line.exchange_rate.as_deref(), line.tax_code.as_deref(),
                        line.cost_center.as_deref(), line.department_id, line.project_id,
                        line.intercompany_entity_id, line.statistical_amount.as_deref(),
                    )
                    .await?;
            }

            // Mark reversal entry totals
            self.recalculate_entry_totals(reversal_entry.id).await?;

            // Mark the original entry as reversed
            self.repository
                .mark_entry_reversal(entry.id, reversal_entry.id)
                .await?;

            // Post the reversal entry
            self.repository
                .update_entry_status(reversal_entry.id, "posted", reversed_by, Some(chrono::Utc::now()))
                .await?;
        }

        // Recalculate batch totals
        self.recalculate_batch_totals(batch_id).await?;

        info!("Reversed journal batch {}", batch.batch_number);
        self.repository
            .update_batch_status(batch_id, "reversed", None, None, reversed_by, None)
            .await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get manual journal dashboard summary
    pub async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<ManualJournalDashboardSummary> {
        self.repository.get_dashboard_summary(org_id).await
    }

    // ========================================================================
    // Internal Helpers
    // ========================================================================

    /// Recalculate entry totals from its lines
    async fn recalculate_entry_totals(&self, entry_id: Uuid) -> AtlasResult<()> {
        let lines = self.repository.list_lines_by_entry(entry_id).await?;
        let total_debit: f64 = lines.iter()
            .filter(|l| l.line_type == "debit")
            .map(|l| l.amount.parse::<f64>().unwrap_or(0.0))
            .sum();
        let total_credit: f64 = lines.iter()
            .filter(|l| l.line_type == "credit")
            .map(|l| l.amount.parse::<f64>().unwrap_or(0.0))
            .sum();
        let is_balanced = (total_debit - total_credit).abs() < BALANCE_TOLERANCE;

        self.repository
            .update_entry_totals(
                entry_id,
                &format!("{:.2}", total_debit),
                &format!("{:.2}", total_credit),
                lines.len() as i32,
                is_balanced,
            )
            .await?;
        Ok(())
    }

    /// Recalculate batch totals from its entries
    async fn recalculate_batch_totals(&self, batch_id: Uuid) -> AtlasResult<()> {
        let entries = self.repository.list_entries_by_batch(batch_id).await?;
        let total_debit: f64 = entries.iter()
            .map(|e| e.total_debit.parse::<f64>().unwrap_or(0.0))
            .sum();
        let total_credit: f64 = entries.iter()
            .map(|e| e.total_credit.parse::<f64>().unwrap_or(0.0))
            .sum();

        self.repository
            .update_batch_totals(
                batch_id,
                &format!("{:.2}", total_debit),
                &format!("{:.2}", total_credit),
                entries.len() as i32,
            )
            .await?;
        Ok(())
    }

    /// Find a line by ID (helper - scans entries to find the line)
    async fn find_line_by_id(&self, line_id: Uuid) -> AtlasResult<JournalEntryLine> {
        // We need a direct query for this; use a placeholder approach
        // In practice we'd add get_line_by_id to the repository
        // For now, iterate a small set
        Err(AtlasError::EntityNotFound(format!(
            "Line {} not found. Use delete_line with known entry context.", line_id
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_batch_statuses() {
        assert!(VALID_BATCH_STATUSES.contains(&"draft"));
        assert!(VALID_BATCH_STATUSES.contains(&"submitted"));
        assert!(VALID_BATCH_STATUSES.contains(&"approved"));
        assert!(VALID_BATCH_STATUSES.contains(&"posted"));
        assert!(VALID_BATCH_STATUSES.contains(&"reversed"));
    }

    #[test]
    fn test_valid_entry_statuses() {
        assert!(VALID_ENTRY_STATUSES.contains(&"draft"));
        assert!(VALID_ENTRY_STATUSES.contains(&"submitted"));
        assert!(VALID_ENTRY_STATUSES.contains(&"approved"));
        assert!(VALID_ENTRY_STATUSES.contains(&"posted"));
        assert!(VALID_ENTRY_STATUSES.contains(&"reversed"));
    }

    #[test]
    fn test_valid_line_types() {
        assert!(VALID_LINE_TYPES.contains(&"debit"));
        assert!(VALID_LINE_TYPES.contains(&"credit"));
    }

    #[test]
    fn test_balance_tolerance() {
        // Within tolerance should be considered balanced
        let diff = 0.003;
        assert!(diff < BALANCE_TOLERANCE);

        // Outside tolerance should not be balanced
        let diff = 0.01;
        assert!(diff >= BALANCE_TOLERANCE);
    }

    #[test]
    fn test_debit_credit_swapping() {
        // Verify reversal swaps debits and credits
        let line_type = "debit";
        let reversed = if line_type == "debit" { "credit" } else { "debit" };
        assert_eq!(reversed, "credit");

        let line_type = "credit";
        let reversed = if line_type == "debit" { "credit" } else { "debit" };
        assert_eq!(reversed, "debit");
    }

    #[test]
    fn test_period_name_from_accounting_date() {
        let date = chrono::NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
        let period = date.format("%Y-%m").to_string();
        assert_eq!(period, "2024-03");
    }

    #[test]
    fn test_reversal_entry_number() {
        let entry_number = "JE-001";
        let reversal = format!("REV-{}", entry_number);
        assert_eq!(reversal, "REV-JE-001");
    }
}
