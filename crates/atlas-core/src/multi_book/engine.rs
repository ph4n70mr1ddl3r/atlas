//! Multi-Book Accounting Engine Implementation
//!
//! Manages accounting books, account mappings, book journal entries,
//! automatic journal propagation between books, and multi-GAAP compliance.
//!
//! Oracle Fusion Cloud ERP equivalent: General Ledger > Multi-Book Accounting

use atlas_shared::{
    AccountingBook, AccountMapping, BookJournalEntry, BookJournalLine,
    PropagationLog, MultiBookSummary,
    AtlasError, AtlasResult,
};
use super::MultiBookAccountingRepository;
use std::sync::Arc;
use tracing::info;

/// Journal entry line data passed between gateway and engine.
///
/// Replaces a complex 6-tuple to keep clippy happy and improve readability.
pub struct JournalLineData {
    pub account_code: String,
    pub account_name: Option<String>,
    pub debit_amount: String,
    pub credit_amount: String,
    pub description: Option<String>,
    pub tax_code: Option<String>,
}
use uuid::Uuid;

/// Valid book types
#[allow(dead_code)]
const VALID_BOOK_TYPES: &[&str] = &["primary", "secondary"];

/// Valid mapping levels
#[allow(dead_code)]
const VALID_MAPPING_LEVELS: &[&str] = &["journal", "subledger"];

/// Valid book statuses
#[allow(dead_code)]
const VALID_BOOK_STATUSES: &[&str] = &["draft", "active", "inactive", "suspended"];

/// Valid journal entry statuses
#[allow(dead_code)]
const VALID_ENTRY_STATUSES: &[&str] = &["draft", "posted", "propagated", "reversed"];

/// Valid propagation log statuses
#[allow(dead_code)]
const VALID_PROPAGATION_STATUSES: &[&str] = &["pending", "completed", "failed", "skipped"];

/// Multi-Book Accounting Engine
pub struct MultiBookAccountingEngine {
    repository: Arc<dyn MultiBookAccountingRepository>,
}

impl MultiBookAccountingEngine {
    pub fn new(repository: Arc<dyn MultiBookAccountingRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Accounting Book Management
    // ========================================================================

    /// Create a new accounting book
    pub async fn create_book(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        book_type: &str,
        chart_of_accounts_code: &str,
        calendar_code: &str,
        currency_code: &str,
        auto_propagation_enabled: bool,
        mapping_level: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AccountingBook> {
        let code_upper = code.to_uppercase();
        if code_upper.is_empty() || code_upper.len() > 50 {
            return Err(AtlasError::ValidationFailed(
                "Accounting book code must be 1-50 characters".to_string(),
            ));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Accounting book name is required".to_string(),
            ));
        }
        if !VALID_BOOK_TYPES.contains(&book_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid book_type '{}'. Must be one of: {}", book_type, VALID_BOOK_TYPES.join(", ")
            )));
        }
        if chart_of_accounts_code.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Chart of accounts code is required".to_string(),
            ));
        }
        if calendar_code.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Calendar code is required".to_string(),
            ));
        }
        if currency_code.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Currency code is required".to_string(),
            ));
        }
        if !VALID_MAPPING_LEVELS.contains(&mapping_level) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid mapping_level '{}'. Must be one of: {}", mapping_level, VALID_MAPPING_LEVELS.join(", ")
            )));
        }

        // Only one primary book allowed per organization
        if book_type == "primary" {
            let existing = self.repository.get_primary_book(org_id).await?;
            if existing.is_some() {
                return Err(AtlasError::ValidationFailed(
                    "Organization already has a primary accounting book. Only one primary book is allowed.".to_string(),
                ));
            }
        }

        info!("Creating accounting book '{}' ({}) for org {}", code_upper, name, org_id);

        self.repository.create_book(
            org_id, &code_upper, name, description, book_type,
            chart_of_accounts_code, calendar_code, currency_code,
            auto_propagation_enabled, mapping_level, created_by,
        ).await
    }

    /// Get an accounting book by code
    pub async fn get_book(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<AccountingBook>> {
        self.repository.get_book(org_id, &code.to_uppercase()).await
    }

    /// Get an accounting book by ID
    pub async fn get_book_by_id(&self, id: Uuid) -> AtlasResult<Option<AccountingBook>> {
        self.repository.get_book_by_id(id).await
    }

    /// List all accounting books for an organization
    pub async fn list_books(&self, org_id: Uuid) -> AtlasResult<Vec<AccountingBook>> {
        self.repository.list_books(org_id).await
    }

    /// Update book status
    pub async fn update_book_status(&self, id: Uuid, status: &str) -> AtlasResult<AccountingBook> {
        if !VALID_BOOK_STATUSES.contains(&status) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid book status '{}'. Must be one of: {}", status, VALID_BOOK_STATUSES.join(", ")
            )));
        }

        let book = self.repository.get_book_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Accounting book {} not found", id)
            ))?;

        // Cannot deactivate a primary book
        if book.book_type == "primary" && status == "inactive" {
            return Err(AtlasError::ValidationFailed(
                "Cannot deactivate a primary accounting book".to_string(),
            ));
        }

        info!("Updating accounting book {} status to {}", id, status);
        self.repository.update_book_status(id, status).await
    }

    /// Delete (deactivate) an accounting book
    pub async fn delete_book(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        let book = self.get_book(org_id, code).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Accounting book '{}' not found", code)
            ))?;

        if book.book_type == "primary" {
            return Err(AtlasError::ValidationFailed(
                "Cannot delete the primary accounting book".to_string(),
            ));
        }

        info!("Deactivating accounting book '{}' for org {}", code, org_id);
        self.repository.delete_book(org_id, &code.to_uppercase()).await
    }

    // ========================================================================
    // Account Mapping Rules
    // ========================================================================

    /// Create an account mapping rule between two books
    pub async fn create_account_mapping(
        &self,
        org_id: Uuid,
        source_book_id: Uuid,
        target_book_id: Uuid,
        source_account_code: &str,
        target_account_code: &str,
        segment_mappings: serde_json::Value,
        priority: i32,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AccountMapping> {
        if source_book_id == target_book_id {
            return Err(AtlasError::ValidationFailed(
                "Source and target books must be different".to_string(),
            ));
        }
        if source_account_code.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Source account code is required".to_string(),
            ));
        }
        if target_account_code.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Target account code is required".to_string(),
            ));
        }

        // Validate source and target books exist
        let source_book = self.repository.get_book_by_id(source_book_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Source accounting book {} not found", source_book_id)
            ))?;
        let _target_book = self.repository.get_book_by_id(target_book_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Target accounting book {} not found", target_book_id)
            ))?;

        if let (Some(from), Some(to)) = (effective_from, effective_to) {
            if from > to {
                return Err(AtlasError::ValidationFailed(
                    "effective_from must be before effective_to".to_string(),
                ));
            }
        }

        info!("Creating account mapping: {} -> {} from book {} to book {}",
            source_account_code, target_account_code, source_book.code, target_book_id);

        self.repository.create_account_mapping(
            org_id, source_book_id, target_book_id,
            source_account_code, target_account_code,
            segment_mappings, priority,
            effective_from, effective_to, created_by,
        ).await
    }

    /// Get mapping rules for a source-target book pair
    pub async fn list_account_mappings(
        &self,
        org_id: Uuid,
        source_book_id: Option<Uuid>,
        target_book_id: Option<Uuid>,
    ) -> AtlasResult<Vec<AccountMapping>> {
        self.repository.list_account_mappings(org_id, source_book_id, target_book_id).await
    }

    /// Delete an account mapping rule
    pub async fn delete_account_mapping(&self, id: Uuid) -> AtlasResult<()> {
        info!("Deleting account mapping rule {}", id);
        self.repository.delete_account_mapping(id).await
    }

    // ========================================================================
    // Book Journal Entries
    // ========================================================================

    /// Create a journal entry in a specific book
    pub async fn create_journal_entry(
        &self,
        org_id: Uuid,
        book_id: Uuid,
        header_description: Option<&str>,
        external_reference: Option<&str>,
        accounting_date: chrono::NaiveDate,
        period_name: Option<&str>,
        currency_code: &str,
        lines: &[JournalLineData],
        created_by: Option<Uuid>,
    ) -> AtlasResult<BookJournalEntry> {
        // Validate book exists and is active
        let book = self.repository.get_book_by_id(book_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Accounting book {} not found", book_id)
            ))?;

        if book.status != "active" {
            return Err(AtlasError::ValidationFailed(
                format!("Accounting book '{}' is not active (status: {})", book.code, book.status)
            ));
        }

        if lines.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Journal entry must have at least one line".to_string(),
            ));
        }

        // Validate balanced entry
        let total_debit: f64 = lines.iter()
            .map(|l| l.debit_amount.parse::<f64>().unwrap_or(0.0))
            .sum();
        let total_credit: f64 = lines.iter()
            .map(|l| l.credit_amount.parse::<f64>().unwrap_or(0.0))
            .sum();

        let diff = (total_debit - total_credit).abs();
        if diff > 0.005 {
            return Err(AtlasError::ValidationFailed(format!(
                "Journal entry is not balanced. Total debit: {:.2}, Total credit: {:.2}, Difference: {:.2}",
                total_debit, total_credit, diff
            )));
        }

        // Generate entry number
        let entry_number = format!("{}-JE-{}", book.code, chrono::Utc::now().timestamp() % 100000);

        info!("Creating journal entry {} in book {} for org {}", entry_number, book.code, org_id);

        let entry = self.repository.create_journal_entry(
            org_id, book_id, &entry_number, header_description,
            None, None, external_reference,
            accounting_date, period_name,
            &format!("{:.2}", total_debit),
            &format!("{:.2}", total_credit),
            "draft", false, currency_code, None,
            serde_json::json!({}),
            created_by,
        ).await?;

        // Create lines
        for (i, line) in lines.iter().enumerate() {
            self.repository.create_journal_line(
                org_id, entry.id, (i + 1) as i32,
                &line.account_code, line.account_name.as_deref(),
                &line.debit_amount, &line.credit_amount, line.description.as_deref(),
                line.tax_code.as_deref(), None,
                serde_json::json!({}),
            ).await?;
        }

        // Reload entry with lines
        self.repository.get_journal_entry_by_id(entry.id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                "Journal entry not found after creation".to_string()
            ))
    }

    /// Get a journal entry by ID (with lines)
    pub async fn get_journal_entry(&self, id: Uuid) -> AtlasResult<Option<BookJournalEntry>> {
        self.repository.get_journal_entry_by_id(id).await
    }

    /// List journal entries for a book
    pub async fn list_journal_entries(
        &self,
        org_id: Uuid,
        book_id: Uuid,
        status: Option<&str>,
    ) -> AtlasResult<Vec<BookJournalEntry>> {
        self.repository.list_journal_entries(org_id, book_id, status).await
    }

    /// Get journal lines for an entry
    pub async fn get_journal_lines(&self, entry_id: Uuid) -> AtlasResult<Vec<BookJournalLine>> {
        self.repository.list_journal_lines(entry_id).await
    }

    /// Post a journal entry (change status from draft to posted)
    pub async fn post_journal_entry(&self, id: Uuid, posted_by: Option<Uuid>) -> AtlasResult<BookJournalEntry> {
        let entry = self.repository.get_journal_entry_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Journal entry {} not found", id)
            ))?;

        if entry.status != "draft" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot post entry in '{}' status. Must be 'draft'.", entry.status)
            ));
        }

        info!("Posting journal entry {} in book {}", entry.entry_number, entry.book_id);

        let posted = self.repository.update_journal_entry_status(
            id, "posted", posted_by,
        ).await?;

        // Check if auto-propagation should be triggered
        let book = self.repository.get_book_by_id(entry.book_id).await?;
        if let Some(book) = book {
            if book.book_type == "primary" {
                self.propagate_to_secondary_books(&posted).await?;
            }
        }

        Ok(posted)
    }

    /// Reverse a posted journal entry
    pub async fn reverse_journal_entry(&self, id: Uuid, created_by: Option<Uuid>) -> AtlasResult<BookJournalEntry> {
        let entry = self.repository.get_journal_entry_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Journal entry {} not found", id)
            ))?;

        if entry.status != "posted" && entry.status != "propagated" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot reverse entry in '{}' status. Must be 'posted' or 'propagated'.", entry.status)
            ));
        }

        info!("Reversing journal entry {} in book {}", entry.entry_number, entry.book_id);

        // Create reversal entry
        let reversal_number = format!("{}-REV-{}", entry.entry_number, chrono::Utc::now().timestamp() % 10000);
        let lines = self.repository.list_journal_lines(id).await?;

        let reversal_entry = self.repository.create_journal_entry(
            entry.organization_id, entry.book_id,
            &reversal_number,
            Some(&format!("Reversal of {}", entry.entry_number)),
            None, None, None,
            entry.accounting_date, entry.period_name.as_deref(),
            &entry.total_credit, // swap debit/credit
            &entry.total_debit,
            "draft", false, &entry.currency_code, None,
            serde_json::json!({ "reversal_of": entry.id.to_string() }),
            created_by,
        ).await?;

        // Create reversal lines (swap debit/credit)
        for line in &lines {
            self.repository.create_journal_line(
                entry.organization_id, reversal_entry.id, line.line_number,
                &line.account_code, line.account_name.as_deref(),
                &line.credit_amount, &line.debit_amount,
                line.description.as_deref(), line.tax_code.as_deref(),
                Some(line.id),
                serde_json::json!({ "reversal_of": line.id.to_string() }),
            ).await?;
        }

        // Mark original as reversed
        self.repository.update_journal_entry_status(id, "reversed", None).await?;

        // Post the reversal immediately
        let posted_reversal = self.repository.update_journal_entry_status(
            reversal_entry.id, "posted", created_by,
        ).await?;

        Ok(posted_reversal)
    }

    // ========================================================================
    // Journal Propagation
    // ========================================================================

    /// Propagate a posted primary-book journal entry to all secondary books
    async fn propagate_to_secondary_books(&self, entry: &BookJournalEntry) -> AtlasResult<Vec<PropagationLog>> {
        let org_id = entry.organization_id;
        let books = self.repository.list_books(org_id).await?;
        let secondary_books: Vec<_> = books.iter()
            .filter(|b| b.book_type == "secondary" && b.is_enabled && b.auto_propagation_enabled)
            .collect();

        if secondary_books.is_empty() {
            return Ok(vec![]);
        }

        let lines = self.repository.list_journal_lines(entry.id).await?;
        let mut logs = Vec::new();

        for target_book in secondary_books {
            let log = self.propagate_entry_to_book(entry, &lines, target_book).await?;
            logs.push(log);
        }

        Ok(logs)
    }

    /// Propagate a single entry to a target secondary book
    async fn propagate_entry_to_book(
        &self,
        source_entry: &BookJournalEntry,
        source_lines: &[BookJournalLine],
        target_book: &AccountingBook,
    ) -> AtlasResult<PropagationLog> {
        info!("Propagating entry {} to book {}",
            source_entry.entry_number, target_book.code);

        let mut propagated_lines = 0;
        let mut unmapped_lines = 0;
        let mut target_line_data: Vec<JournalLineData> = Vec::new();

        for line in source_lines {
            // Look up account mapping
            let mapping = self.repository.find_account_mapping(
                source_entry.organization_id,
                source_entry.book_id,
                target_book.id,
                &line.account_code,
            ).await?;

            match mapping {
                Some(m) => {
                    target_line_data.push(JournalLineData {
                        account_code: m.target_account_code.clone(),
                        account_name: None,
                        debit_amount: line.debit_amount.clone(),
                        credit_amount: line.credit_amount.clone(),
                        description: line.description.clone(),
                        tax_code: line.tax_code.clone(),
                    });
                    propagated_lines += 1;
                }
                None => {
                    // No mapping found - skip this line
                    unmapped_lines += 1;
                    info!("No mapping found for account {} from book {} to book {}, skipping line",
                        line.account_code, source_entry.book_id, target_book.id);
                }
            }
        }

        if target_line_data.is_empty() {
            // No lines could be mapped
            let log = self.repository.create_propagation_log(
                source_entry.organization_id,
                source_entry.book_id,
                target_book.id,
                source_entry.id,
                None,
                "skipped",
                0,
                unmapped_lines,
                Some("No account mappings found for any lines"),
                serde_json::json!({}),
            ).await?;
            return Ok(log);
        }

        // Check if balanced after propagation
        let total_debit: f64 = target_line_data.iter()
            .map(|l| l.debit_amount.parse::<f64>().unwrap_or(0.0))
            .sum();
        let total_credit: f64 = target_line_data.iter()
            .map(|l| l.credit_amount.parse::<f64>().unwrap_or(0.0))
            .sum();

        // Create the propagated entry
        let propagated_number = format!("{}-PROP-{}", target_book.code, chrono::Utc::now().timestamp() % 100000);

        // Determine conversion rate if currencies differ
        let conversion_rate = if source_entry.currency_code != target_book.currency_code {
            Some("1.0".to_string()) // Simplified - would integrate with currency engine
        } else {
            None
        };

        let target_entry = self.repository.create_journal_entry(
            source_entry.organization_id,
            target_book.id,
            &propagated_number,
            Some(&format!("Propagated from {}", source_entry.entry_number)),
            Some(source_entry.book_id),
            Some(source_entry.id),
            None,
            source_entry.accounting_date,
            source_entry.period_name.as_deref(),
            &format!("{:.2}", total_debit),
            &format!("{:.2}", total_credit),
            "propagated",
            true,
            &target_book.currency_code,
            conversion_rate.as_deref(),
            serde_json::json!({}),
            source_entry.created_by,
        ).await?;

        // Create target lines
        for (i, line) in target_line_data.iter().enumerate() {
            // Find the source line for cross-reference
            let source_line = source_lines.get(i);
            self.repository.create_journal_line(
                source_entry.organization_id,
                target_entry.id,
                (i + 1) as i32,
                &line.account_code,
                line.account_name.as_deref(),
                &line.debit_amount,
                &line.credit_amount,
                line.description.as_deref(),
                line.tax_code.as_deref(),
                source_line.map(|l| l.id),
                serde_json::json!({}),
            ).await?;
        }

        // Create propagation log
        let log = self.repository.create_propagation_log(
            source_entry.organization_id,
            source_entry.book_id,
            target_book.id,
            source_entry.id,
            Some(target_entry.id),
            "completed",
            propagated_lines,
            unmapped_lines,
            None,
            serde_json::json!({}),
        ).await?;

        Ok(log)
    }

    /// Manually propagate a specific entry to a target book
    pub async fn propagate_entry(
        &self,
        entry_id: Uuid,
        target_book_id: Uuid,
    ) -> AtlasResult<PropagationLog> {
        let entry = self.repository.get_journal_entry_by_id(entry_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Journal entry {} not found", entry_id)
            ))?;

        if entry.status != "posted" {
            return Err(AtlasError::ValidationFailed(
                "Only posted journal entries can be propagated".to_string(),
            ));
        }

        let target_book = self.repository.get_book_by_id(target_book_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Target accounting book {} not found", target_book_id)
            ))?;

        let lines = self.repository.list_journal_lines(entry_id).await?;
        self.propagate_entry_to_book(&entry, &lines, &target_book).await
    }

    /// List propagation logs
    pub async fn list_propagation_logs(
        &self,
        org_id: Uuid,
        source_book_id: Option<Uuid>,
        target_book_id: Option<Uuid>,
    ) -> AtlasResult<Vec<PropagationLog>> {
        self.repository.list_propagation_logs(org_id, source_book_id, target_book_id).await
    }

    // ========================================================================
    // Dashboard Summary
    // ========================================================================

    /// Get multi-book accounting dashboard summary
    pub async fn get_summary(&self, org_id: Uuid) -> AtlasResult<MultiBookSummary> {
        let books = self.repository.list_books(org_id).await?;
        let primary = books.iter().find(|b| b.book_type == "primary");
        let secondary_count = books.iter().filter(|b| b.book_type == "secondary").count();

        let mappings = self.repository.list_account_mappings(org_id, None, None).await?;
        let logs = self.repository.list_propagation_logs(org_id, None, None).await?;

        let completed = logs.iter().filter(|l| l.status == "completed").count();
        let total = logs.len();
        let success_rate = if total > 0 {
            format!("{:.1}%", (completed as f64 / total as f64) * 100.0)
        } else {
            "N/A".to_string()
        };

        let mut unposted_by_book = serde_json::Map::new();
        let mut entry_counts = serde_json::Map::new();
        for book in &books {
            let entries = self.repository.list_journal_entries(org_id, book.id, None).await?;
            let unposted = entries.iter().filter(|e| e.status == "draft").count();
            unposted_by_book.insert(book.code.clone(), serde_json::json!(unposted));
            entry_counts.insert(book.code.clone(), serde_json::json!(entries.len()));
        }

        Ok(MultiBookSummary {
            book_count: books.len() as i32,
            primary_book_code: primary.map(|b| b.code.clone()),
            secondary_book_count: secondary_count as i32,
            mapping_rule_count: mappings.len() as i32,
            recent_propagation_count: total as i32,
            propagation_success_rate: success_rate,
            unposted_entries_by_book: serde_json::Value::Object(unposted_by_book),
            entry_counts_by_book: serde_json::Value::Object(entry_counts),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_book_types() {
        assert!(VALID_BOOK_TYPES.contains(&"primary"));
        assert!(VALID_BOOK_TYPES.contains(&"secondary"));
        assert!(!VALID_BOOK_TYPES.contains(&"tertiary"));
    }

    #[test]
    fn test_valid_mapping_levels() {
        assert!(VALID_MAPPING_LEVELS.contains(&"journal"));
        assert!(VALID_MAPPING_LEVELS.contains(&"subledger"));
    }

    #[test]
    fn test_valid_book_statuses() {
        assert!(VALID_BOOK_STATUSES.contains(&"draft"));
        assert!(VALID_BOOK_STATUSES.contains(&"active"));
        assert!(VALID_BOOK_STATUSES.contains(&"inactive"));
        assert!(VALID_BOOK_STATUSES.contains(&"suspended"));
    }

    #[test]
    fn test_valid_entry_statuses() {
        assert!(VALID_ENTRY_STATUSES.contains(&"draft"));
        assert!(VALID_ENTRY_STATUSES.contains(&"posted"));
        assert!(VALID_ENTRY_STATUSES.contains(&"propagated"));
        assert!(VALID_ENTRY_STATUSES.contains(&"reversed"));
    }

    #[test]
    fn test_valid_propagation_statuses() {
        assert!(VALID_PROPAGATION_STATUSES.contains(&"pending"));
        assert!(VALID_PROPAGATION_STATUSES.contains(&"completed"));
        assert!(VALID_PROPAGATION_STATUSES.contains(&"failed"));
        assert!(VALID_PROPAGATION_STATUSES.contains(&"skipped"));
    }
}
