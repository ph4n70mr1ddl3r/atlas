//! Document Sequencing Engine
//!
//! Manages document sequence lifecycle, number generation with formatting,
//! sequence assignment resolution, periodic reset, and audit trail.
//!
//! Oracle Fusion equivalent: General Ledger > Setup > Document Sequencing

use atlas_shared::{
    DocumentSequence, DocumentSequenceAssignment, DocumentSequenceAudit,
    DocumentSequenceDashboardSummary,
    AtlasError, AtlasResult,
};
use super::DocumentSequencingRepository;
use chrono::Datelike;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Valid sequence types
const VALID_SEQUENCE_TYPES: &[&str] = &["gapless", "gap_permitted", "manual"];

/// Valid document types that can have sequences
const VALID_DOCUMENT_TYPES: &[&str] = &[
    "invoice", "credit_memo", "debit_memo",
    "purchase_order", "purchase_requisition",
    "journal_entry", "journal_batch",
    "payment", "receipt", "settlement",
    "sales_order", "quote",
    "expense_report", "timesheet",
    "fixed_asset", "lease",
    "tax_report", "custom",
];

/// Valid reset frequencies
const VALID_RESET_FREQUENCIES: &[&str] = &["daily", "monthly", "quarterly", "annually", "never"];

/// Valid assignment methods
const VALID_METHODS: &[&str] = &["automatic", "manual"];

/// Document Sequencing engine
pub struct DocumentSequencingEngine {
    repository: Arc<dyn DocumentSequencingRepository>,
}

impl DocumentSequencingEngine {
    pub fn new(repository: Arc<dyn DocumentSequencingRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Sequence Management
    // ========================================================================

    /// Create a new document sequence
    pub async fn create_sequence(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        sequence_type: &str,
        document_type: &str,
        initial_value: i64,
        increment_by: i32,
        max_value: Option<i64>,
        cycle_flag: bool,
        prefix: Option<&str>,
        suffix: Option<&str>,
        pad_length: i32,
        pad_character: &str,
        reset_frequency: Option<&str>,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<DocumentSequence> {
        // Validate inputs
        if code.is_empty() {
            return Err(AtlasError::ValidationFailed("Sequence code is required".to_string()));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Sequence name is required".to_string()));
        }
        if !VALID_SEQUENCE_TYPES.contains(&sequence_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid sequence type '{}'. Must be one of: {}",
                sequence_type, VALID_SEQUENCE_TYPES.join(", ")
            )));
        }
        if !VALID_DOCUMENT_TYPES.contains(&document_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid document type '{}'. Must be one of: {}",
                document_type, VALID_DOCUMENT_TYPES.join(", ")
            )));
        }
        if initial_value < 0 {
            return Err(AtlasError::ValidationFailed("Initial value must be >= 0".to_string()));
        }
        if increment_by < 1 {
            return Err(AtlasError::ValidationFailed("Increment by must be >= 1".to_string()));
        }
        if let Some(max) = max_value {
            if max < initial_value {
                return Err(AtlasError::ValidationFailed(
                    "Max value must be >= initial value".to_string(),
                ));
            }
        }
        if pad_length < 0 {
            return Err(AtlasError::ValidationFailed("Pad length must be >= 0".to_string()));
        }
        if pad_character.len() != 1 {
            return Err(AtlasError::ValidationFailed("Pad character must be exactly 1 character".to_string()));
        }
        if let Some(freq) = reset_frequency {
            if !VALID_RESET_FREQUENCIES.contains(&freq) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid reset frequency '{}'. Must be one of: {}",
                    freq, VALID_RESET_FREQUENCIES.join(", ")
                )));
            }
        }
        if let (Some(from), Some(to)) = (effective_from, effective_to) {
            if to < from {
                return Err(AtlasError::ValidationFailed(
                    "Effective to date must be after effective from date".to_string(),
                ));
            }
        }
        if cycle_flag && max_value.is_none() {
            return Err(AtlasError::ValidationFailed(
                "Cycle flag requires a max value to be set".to_string(),
            ));
        }

        // Check uniqueness
        if self.repository.get_sequence(org_id, code).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Document sequence with code '{}' already exists", code
            )));
        }

        info!("Creating document sequence {} ({}) for org {}", code, name, org_id);

        self.repository.create_sequence(
            org_id, code, name, description, sequence_type, document_type,
            initial_value, increment_by, max_value, cycle_flag,
            prefix, suffix, pad_length, pad_character, reset_frequency,
            effective_from, effective_to, created_by,
        ).await
    }

    /// Get a sequence by code
    pub async fn get_sequence(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<DocumentSequence>> {
        self.repository.get_sequence(org_id, code).await
    }

    /// Get a sequence by ID
    pub async fn get_sequence_by_id(&self, id: Uuid) -> AtlasResult<Option<DocumentSequence>> {
        self.repository.get_sequence_by_id(id).await
    }

    /// List sequences with optional filters
    pub async fn list_sequences(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        document_type: Option<&str>,
    ) -> AtlasResult<Vec<DocumentSequence>> {
        self.repository.list_sequences(org_id, status, document_type).await
    }

    /// Activate a sequence
    pub async fn activate_sequence(&self, id: Uuid) -> AtlasResult<DocumentSequence> {
        let seq = self.get_sequence_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Sequence {} not found", id)))?;

        if seq.status == "active" {
            return Err(AtlasError::WorkflowError("Sequence is already active".to_string()));
        }

        info!("Activated document sequence {}", seq.code);
        self.repository.update_sequence_status(id, "active").await
    }

    /// Deactivate a sequence
    pub async fn deactivate_sequence(&self, id: Uuid) -> AtlasResult<DocumentSequence> {
        let seq = self.get_sequence_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Sequence {} not found", id)))?;

        if seq.status == "inactive" {
            return Err(AtlasError::WorkflowError("Sequence is already inactive".to_string()));
        }

        info!("Deactivated document sequence {}", seq.code);
        self.repository.update_sequence_status(id, "inactive").await
    }

    /// Delete a sequence (only if no assignments exist)
    pub async fn delete_sequence(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        let seq = self.repository.get_sequence(org_id, code).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Sequence '{}' not found", code)))?;

        // Check for assignments
        let assignments = self.repository.list_assignments(org_id, Some(seq.id)).await?;
        if !assignments.is_empty() {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot delete sequence '{}' - it has {} assignment(s). Remove assignments first.",
                code, assignments.len()
            )));
        }

        info!("Deleted document sequence {}", code);
        self.repository.delete_sequence(org_id, code).await
    }

    // ========================================================================
    // Number Generation
    // ========================================================================

    /// Generate the next document number for a given document category.
    /// Automatically resolves the sequence assignment and handles reset logic.
    pub async fn generate_number(
        &self,
        org_id: Uuid,
        document_category: &str,
        business_unit_id: Option<Uuid>,
        ledger_id: Option<Uuid>,
        document_id: Option<Uuid>,
        document_number: Option<&str>,
        generated_by: Option<Uuid>,
    ) -> AtlasResult<DocumentSequenceAudit> {
        // Find the applicable assignment
        let assignment = self.repository.find_assignment(
            org_id, document_category, business_unit_id, ledger_id,
        ).await?.ok_or_else(|| AtlasError::EntityNotFound(format!(
            "No active sequence assignment found for document category '{}'{}",
            document_category,
            business_unit_id.map(|id| format!(" and business unit {}", id)).unwrap_or_default()
        )))?;

        // Get the sequence
        let mut sequence = self.repository.get_sequence_by_id(assignment.sequence_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Sequence {} not found", assignment.sequence_id
            )))?;

        // Check sequence is active
        if sequence.status != "active" {
            return Err(AtlasError::WorkflowError(format!(
                "Sequence '{}' is not active (status: {})", sequence.code, sequence.status
            )));
        }

        // Check effective dates
        let today = chrono::Utc::now().date_naive();
        if let Some(from) = sequence.effective_from {
            if today < from {
                return Err(AtlasError::ValidationFailed(format!(
                    "Sequence '{}' is not yet effective (effective from {})",
                    sequence.code, from
                )));
            }
        }
        if let Some(to) = sequence.effective_to {
            if today > to {
                return Err(AtlasError::ValidationFailed(format!(
                    "Sequence '{}' has expired (effective to {})",
                    sequence.code, to
                )));
            }
        }

        // Check and perform reset if needed
        sequence = self.check_and_reset(sequence).await?;

        // Get next value
        let next_value = if sequence.sequence_type == "manual" {
            // Manual sequences don't auto-increment; the caller provides the number
            return Err(AtlasError::WorkflowError(
                "Manual sequences do not support automatic number generation".to_string(),
            ));
        } else {
            // Atomic increment for both gapless and gap_permitted
            let incremented = self.repository.increment_sequence_value(sequence.id, sequence.increment_by).await?;
            incremented.current_value
        };

        // Check max value
        if let Some(max) = sequence.max_value {
            if next_value > max {
                if sequence.cycle_flag {
                    // Reset and get new value
                    sequence = self.repository.reset_sequence(sequence.id, today).await?;
                    let incremented = self.repository.increment_sequence_value(sequence.id, sequence.increment_by).await?;
                    // Use the newly cycled value
                    let cycled_value = incremented.current_value;

                    let formatted = Self::format_number(
                        &sequence, cycled_value,
                    );

                    info!(
                        "Generated cycled document number '{}' for sequence {} (category: {})",
                        formatted, sequence.code, document_category
                    );

                    return self.repository.create_audit_entry(
                        org_id, sequence.id, &sequence.code,
                        &formatted, cycled_value,
                        document_category, document_id, document_number,
                        business_unit_id, generated_by,
                        serde_json::json!({ "cycled": true }),
                    ).await;
                } else {
                    return Err(AtlasError::WorkflowError(format!(
                        "Sequence '{}' has reached its maximum value ({})",
                        sequence.code, max
                    )));
                }
            }
        }

        // Format the number
        let formatted = Self::format_number(&sequence, next_value);

        info!(
            "Generated document number '{}' for sequence {} (category: {})",
            formatted, sequence.code, document_category
        );

        // Create audit entry
        self.repository.create_audit_entry(
            org_id, sequence.id, &sequence.code,
            &formatted, next_value,
            document_category, document_id, document_number,
            business_unit_id, generated_by,
            serde_json::json!({}),
        ).await
    }

    /// Generate a number directly from a specific sequence (bypassing assignment resolution)
    pub async fn generate_number_direct(
        &self,
        org_id: Uuid,
        sequence_code: &str,
        document_category: &str,
        document_id: Option<Uuid>,
        document_number: Option<&str>,
        business_unit_id: Option<Uuid>,
        generated_by: Option<Uuid>,
    ) -> AtlasResult<DocumentSequenceAudit> {
        let sequence = self.repository.get_sequence(org_id, sequence_code).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Sequence '{}' not found", sequence_code
            )))?;

        if sequence.status != "active" {
            return Err(AtlasError::WorkflowError(format!(
                "Sequence '{}' is not active", sequence_code
            )));
        }

        if sequence.sequence_type == "manual" {
            return Err(AtlasError::WorkflowError(
                "Manual sequences do not support automatic number generation".to_string(),
            ));
        }

        // Check and perform reset
        let sequence = self.check_and_reset(sequence).await?;

        // Atomic increment
        let incremented = self.repository.increment_sequence_value(sequence.id, sequence.increment_by).await?;
        let next_value = incremented.current_value;

        // Check max
        if let Some(max) = sequence.max_value {
            if next_value > max && !sequence.cycle_flag {
                return Err(AtlasError::WorkflowError(format!(
                    "Sequence '{}' has reached its maximum value ({})",
                    sequence_code, max
                )));
            }
        }

        let formatted = Self::format_number(&sequence, next_value);

        info!("Generated direct document number '{}' from sequence {}", formatted, sequence_code);

        self.repository.create_audit_entry(
            org_id, sequence.id, sequence_code,
            &formatted, next_value,
            document_category, document_id, document_number,
            business_unit_id, generated_by,
            serde_json::json!({}),
        ).await
    }

    // ========================================================================
    // Assignment Management
    // ========================================================================

    /// Create a sequence assignment
    pub async fn create_assignment(
        &self,
        org_id: Uuid,
        sequence_code: &str,
        document_category: &str,
        business_unit_id: Option<Uuid>,
        ledger_id: Option<Uuid>,
        method: &str,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        priority: i32,
        created_by: Option<Uuid>,
    ) -> AtlasResult<DocumentSequenceAssignment> {
        if !VALID_METHODS.contains(&method) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid method '{}'. Must be one of: {}",
                method, VALID_METHODS.join(", ")
            )));
        }

        let sequence = self.repository.get_sequence(org_id, sequence_code).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Sequence '{}' not found", sequence_code
            )))?;

        if let (Some(from), Some(to)) = (effective_from, effective_to) {
            if to < from {
                return Err(AtlasError::ValidationFailed(
                    "Effective to date must be after effective from date".to_string(),
                ));
            }
        }

        info!(
            "Creating sequence assignment: {} -> {} (category: {})",
            sequence_code, document_category, document_category
        );

        self.repository.create_assignment(
            org_id, sequence.id, sequence_code,
            document_category, business_unit_id, ledger_id,
            method, effective_from, effective_to, priority, created_by,
        ).await
    }

    /// Get an assignment by ID
    pub async fn get_assignment(&self, id: Uuid) -> AtlasResult<Option<DocumentSequenceAssignment>> {
        self.repository.get_assignment(id).await
    }

    /// List assignments
    pub async fn list_assignments(
        &self,
        org_id: Uuid,
        sequence_id: Option<Uuid>,
    ) -> AtlasResult<Vec<DocumentSequenceAssignment>> {
        self.repository.list_assignments(org_id, sequence_id).await
    }

    /// Deactivate an assignment
    pub async fn deactivate_assignment(&self, id: Uuid) -> AtlasResult<DocumentSequenceAssignment> {
        let assignment = self.repository.get_assignment(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Assignment {} not found", id)))?;

        if assignment.status == "inactive" {
            return Err(AtlasError::WorkflowError("Assignment is already inactive".to_string()));
        }

        info!("Deactivated sequence assignment for category {}", assignment.document_category);
        self.repository.update_assignment_status(id, "inactive").await
    }

    /// Delete an assignment
    pub async fn delete_assignment(&self, id: Uuid) -> AtlasResult<()> {
        info!("Deleted sequence assignment {}", id);
        self.repository.delete_assignment(id).await
    }

    // ========================================================================
    // Audit Trail
    // ========================================================================

    /// List audit entries
    pub async fn list_audit_entries(
        &self,
        org_id: Uuid,
        sequence_id: Option<Uuid>,
        limit: Option<i32>,
    ) -> AtlasResult<Vec<DocumentSequenceAudit>> {
        self.repository.list_audit_entries(org_id, sequence_id, limit).await
    }

    /// Get audit entry for a specific document
    pub async fn get_audit_by_document(&self, document_id: Uuid) -> AtlasResult<Option<DocumentSequenceAudit>> {
        self.repository.get_audit_by_document(document_id).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get dashboard summary
    pub async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<DocumentSequenceDashboardSummary> {
        self.repository.get_dashboard_summary(org_id).await
    }

    // ========================================================================
    // Internal Helpers
    // ========================================================================

    /// Format a numeric value using the sequence's prefix, suffix, and padding
    pub fn format_number(sequence: &DocumentSequence, value: i64) -> String {
        let numeric_str = if sequence.pad_length > 0 {
            format!("{:0>width$}", value, width = sequence.pad_length as usize)
        } else {
            value.to_string()
        };

        let prefix = sequence.prefix.as_deref().unwrap_or("");
        let suffix = sequence.suffix.as_deref().unwrap_or("");

        format!("{}{}{}", prefix, numeric_str, suffix)
    }

    /// Check if a sequence needs to be reset based on its reset_frequency
    /// and perform the reset if needed.
    async fn check_and_reset(&self, mut sequence: DocumentSequence) -> AtlasResult<DocumentSequence> {
        let frequency = match &sequence.reset_frequency {
            Some(f) if f != "never" => f.clone(),
            _ => return Ok(sequence),
        };

        let today = chrono::Utc::now().date_naive();
        let needs_reset = match sequence.last_reset_date {
            None => true,
            Some(last_reset) => {
                match frequency.as_str() {
                    "daily" => today > last_reset,
                    "monthly" => {
                        let today_month = today.format("%Y-%m").to_string();
                        let last_month = last_reset.format("%Y-%m").to_string();
                        today_month != last_month
                    }
                    "quarterly" => {
                        let today_q = today.format("%Y").to_string() + &format!("-Q{}", (today.month() - 1) / 3 + 1);
                        let last_q = last_reset.format("%Y").to_string() + &format!("-Q{}", (last_reset.month() - 1) / 3 + 1);
                        today_q != last_q
                    }
                    "annually" => today.format("%Y").to_string() != last_reset.format("%Y").to_string(),
                    _ => false,
                }
            }
        };

        if needs_reset {
            info!("Resetting sequence {} (frequency: {})", sequence.code, frequency);
            sequence = self.repository.reset_sequence(sequence.id, today).await?;
        }

        Ok(sequence)
    }

    /// Check if a value would need reset based on the frequency and last reset date.
    /// Used for testing; the actual reset happens in check_and_reset.
    pub fn needs_reset(sequence: &DocumentSequence, today: chrono::NaiveDate) -> bool {
        let frequency = match &sequence.reset_frequency {
            Some(f) if f != "never" => f.clone(),
            _ => return false,
        };

        match sequence.last_reset_date {
            None => true,
            Some(last_reset) => {
                match frequency.as_str() {
                    "daily" => today > last_reset,
                    "monthly" => {
                        today.format("%Y-%m").to_string() != last_reset.format("%Y-%m").to_string()
                    }
                    "quarterly" => {
                        let today_q = today.format("%Y").to_string() + &format!("-Q{}", (today.month() - 1) / 3 + 1);
                        let last_q = last_reset.format("%Y").to_string() + &format!("-Q{}", (last_reset.month() - 1) / 3 + 1);
                        today_q != last_q
                    }
                    "annually" => today.format("%Y").to_string() != last_reset.format("%Y").to_string(),
                    _ => false,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to create a default DocumentSequence for testing
    fn make_sequence(
        code: &str,
        sequence_type: &str,
        initial_value: i64,
        increment_by: i32,
        pad_length: i32,
        pad_character: &str,
        prefix: Option<&str>,
        suffix: Option<&str>,
        max_value: Option<i64>,
        cycle_flag: bool,
        reset_frequency: Option<&str>,
        last_reset_date: Option<chrono::NaiveDate>,
    ) -> DocumentSequence {
        DocumentSequence {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            code: code.to_string(),
            name: format!("{} sequence", code),
            description: None,
            sequence_type: sequence_type.to_string(),
            document_type: "invoice".to_string(),
            initial_value,
            current_value: initial_value - (increment_by.max(1) as i64),
            increment_by,
            max_value,
            cycle_flag,
            prefix: prefix.map(String::from),
            suffix: suffix.map(String::from),
            pad_length,
            pad_character: pad_character.to_string(),
            reset_frequency: reset_frequency.map(String::from),
            last_reset_date,
            effective_from: None,
            effective_to: None,
            status: "active".to_string(),
            created_by: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn test_valid_sequence_types() {
        assert!(VALID_SEQUENCE_TYPES.contains(&"gapless"));
        assert!(VALID_SEQUENCE_TYPES.contains(&"gap_permitted"));
        assert!(VALID_SEQUENCE_TYPES.contains(&"manual"));
        assert_eq!(VALID_SEQUENCE_TYPES.len(), 3);
    }

    #[test]
    fn test_valid_document_types() {
        assert!(VALID_DOCUMENT_TYPES.contains(&"invoice"));
        assert!(VALID_DOCUMENT_TYPES.contains(&"purchase_order"));
        assert!(VALID_DOCUMENT_TYPES.contains(&"journal_entry"));
        assert!(VALID_DOCUMENT_TYPES.contains(&"payment"));
        assert!(VALID_DOCUMENT_TYPES.contains(&"receipt"));
        assert!(VALID_DOCUMENT_TYPES.contains(&"sales_order"));
    }

    #[test]
    fn test_valid_reset_frequencies() {
        assert!(VALID_RESET_FREQUENCIES.contains(&"daily"));
        assert!(VALID_RESET_FREQUENCIES.contains(&"monthly"));
        assert!(VALID_RESET_FREQUENCIES.contains(&"quarterly"));
        assert!(VALID_RESET_FREQUENCIES.contains(&"annually"));
        assert!(VALID_RESET_FREQUENCIES.contains(&"never"));
    }

    #[test]
    fn test_valid_methods() {
        assert!(VALID_METHODS.contains(&"automatic"));
        assert!(VALID_METHODS.contains(&"manual"));
    }

    #[test]
    fn test_format_number_simple() {
        let seq = make_sequence("INV-SEQ", "gapless", 1, 1, 0, "0", None, None, None, false, None, None);
        assert_eq!(DocumentSequencingEngine::format_number(&seq, 1), "1");
        assert_eq!(DocumentSequencingEngine::format_number(&seq, 42), "42");
        assert_eq!(DocumentSequencingEngine::format_number(&seq, 1000), "1000");
    }

    #[test]
    fn test_format_number_with_padding() {
        let seq = make_sequence("INV-SEQ", "gapless", 1, 1, 6, "0", None, None, None, false, None, None);
        assert_eq!(DocumentSequencingEngine::format_number(&seq, 1), "000001");
        assert_eq!(DocumentSequencingEngine::format_number(&seq, 42), "000042");
        assert_eq!(DocumentSequencingEngine::format_number(&seq, 123456), "123456");
        assert_eq!(DocumentSequencingEngine::format_number(&seq, 9999999), "9999999"); // exceeds pad length
    }

    #[test]
    fn test_format_number_with_prefix() {
        let seq = make_sequence("INV-SEQ", "gapless", 1, 1, 6, "0", Some("INV-"), None, None, false, None, None);
        assert_eq!(DocumentSequencingEngine::format_number(&seq, 1), "INV-000001");
        assert_eq!(DocumentSequencingEngine::format_number(&seq, 123), "INV-000123");
    }

    #[test]
    fn test_format_number_with_suffix() {
        let seq = make_sequence("PO-SEQ", "gap_permitted", 1, 1, 5, "0", None, Some("-2024"), None, false, None, None);
        assert_eq!(DocumentSequencingEngine::format_number(&seq, 1), "00001-2024");
        assert_eq!(DocumentSequencingEngine::format_number(&seq, 99), "00099-2024");
    }

    #[test]
    fn test_format_number_with_prefix_and_suffix() {
        let seq = make_sequence("JE-SEQ", "gapless", 1, 1, 8, "0", Some("JE"), Some("-GL"), None, false, None, None);
        assert_eq!(DocumentSequencingEngine::format_number(&seq, 1), "JE00000001-GL");
        assert_eq!(DocumentSequencingEngine::format_number(&seq, 500), "JE00000500-GL");
    }

    #[test]
    fn test_format_number_no_padding_with_prefix() {
        let seq = make_sequence("PAY-SEQ", "gap_permitted", 1000, 1, 0, "0", Some("PAY-"), None, None, false, None, None);
        assert_eq!(DocumentSequencingEngine::format_number(&seq, 1000), "PAY-1000");
        assert_eq!(DocumentSequencingEngine::format_number(&seq, 9999), "PAY-9999");
    }

    #[test]
    fn test_format_number_with_increment_by_5() {
        let seq = make_sequence("BATCH-SEQ", "gap_permitted", 0, 5, 4, "0", None, None, None, false, None, None);
        assert_eq!(DocumentSequencingEngine::format_number(&seq, 5), "0005");
        assert_eq!(DocumentSequencingEngine::format_number(&seq, 50), "0050");
        assert_eq!(DocumentSequencingEngine::format_number(&seq, 100), "0100");
    }

    #[test]
    fn test_needs_reset_no_frequency() {
        let seq = make_sequence("SEQ", "gapless", 1, 1, 0, "0", None, None, None, false, None, None);
        let today = chrono::NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        assert!(!DocumentSequencingEngine::needs_reset(&seq, today));
    }

    #[test]
    fn test_needs_reset_never_frequency() {
        let seq = make_sequence("SEQ", "gapless", 1, 1, 0, "0", None, None, None, false, Some("never"), None);
        let today = chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        assert!(!DocumentSequencingEngine::needs_reset(&seq, today));
    }

    #[test]
    fn test_needs_reset_daily_same_day() {
        let today = chrono::NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let seq = make_sequence("SEQ", "gapless", 1, 1, 0, "0", None, None, None, false, Some("daily"), Some(today));
        assert!(!DocumentSequencingEngine::needs_reset(&seq, today));
    }

    #[test]
    fn test_needs_reset_daily_next_day() {
        let yesterday = chrono::NaiveDate::from_ymd_opt(2024, 6, 14).unwrap();
        let today = chrono::NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let seq = make_sequence("SEQ", "gapless", 1, 1, 0, "0", None, None, None, false, Some("daily"), Some(yesterday));
        assert!(DocumentSequencingEngine::needs_reset(&seq, today));
    }

    #[test]
    fn test_needs_reset_monthly_same_month() {
        let last_reset = chrono::NaiveDate::from_ymd_opt(2024, 6, 1).unwrap();
        let today = chrono::NaiveDate::from_ymd_opt(2024, 6, 28).unwrap();
        let seq = make_sequence("SEQ", "gapless", 1, 1, 0, "0", None, None, None, false, Some("monthly"), Some(last_reset));
        assert!(!DocumentSequencingEngine::needs_reset(&seq, today));
    }

    #[test]
    fn test_needs_reset_monthly_next_month() {
        let last_reset = chrono::NaiveDate::from_ymd_opt(2024, 5, 31).unwrap();
        let today = chrono::NaiveDate::from_ymd_opt(2024, 6, 1).unwrap();
        let seq = make_sequence("SEQ", "gapless", 1, 1, 0, "0", None, None, None, false, Some("monthly"), Some(last_reset));
        assert!(DocumentSequencingEngine::needs_reset(&seq, today));
    }

    #[test]
    fn test_needs_reset_quarterly_same_quarter() {
        let last_reset = chrono::NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let today = chrono::NaiveDate::from_ymd_opt(2024, 3, 31).unwrap();
        let seq = make_sequence("SEQ", "gapless", 1, 1, 0, "0", None, None, None, false, Some("quarterly"), Some(last_reset));
        assert!(!DocumentSequencingEngine::needs_reset(&seq, today));
    }

    #[test]
    fn test_needs_reset_quarterly_next_quarter() {
        let last_reset = chrono::NaiveDate::from_ymd_opt(2024, 3, 31).unwrap();
        let today = chrono::NaiveDate::from_ymd_opt(2024, 4, 1).unwrap();
        let seq = make_sequence("SEQ", "gapless", 1, 1, 0, "0", None, None, None, false, Some("quarterly"), Some(last_reset));
        assert!(DocumentSequencingEngine::needs_reset(&seq, today));
    }

    #[test]
    fn test_needs_reset_annually_same_year() {
        let last_reset = chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let today = chrono::NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();
        let seq = make_sequence("SEQ", "gapless", 1, 1, 0, "0", None, None, None, false, Some("annually"), Some(last_reset));
        assert!(!DocumentSequencingEngine::needs_reset(&seq, today));
    }

    #[test]
    fn test_needs_reset_annually_next_year() {
        let last_reset = chrono::NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();
        let today = chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        let seq = make_sequence("SEQ", "gapless", 1, 1, 0, "0", None, None, None, false, Some("annually"), Some(last_reset));
        assert!(DocumentSequencingEngine::needs_reset(&seq, today));
    }

    #[test]
    fn test_needs_reset_no_last_reset_date() {
        let seq = make_sequence("SEQ", "gapless", 1, 1, 0, "0", None, None, None, false, Some("monthly"), None);
        let today = chrono::NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        assert!(DocumentSequencingEngine::needs_reset(&seq, today));
    }

    #[test]
    fn test_format_number_large_value() {
        let seq = make_sequence("SEQ", "gapless", 1, 1, 10, "0", Some("DOC-"), None, None, false, None, None);
        assert_eq!(DocumentSequencingEngine::format_number(&seq, 1234567890), "DOC-1234567890");
    }

    #[test]
    fn test_format_number_initial_value_non_one() {
        let seq = make_sequence("SEQ", "gap_permitted", 1000, 1, 6, "0", Some("PO-"), None, None, false, None, None);
        assert_eq!(DocumentSequencingEngine::format_number(&seq, 1000), "PO-001000");
        assert_eq!(DocumentSequencingEngine::format_number(&seq, 1001), "PO-001001");
    }

    #[test]
    fn test_quarterly_boundary_q1_to_q2() {
        let last_reset = chrono::NaiveDate::from_ymd_opt(2024, 3, 31).unwrap();
        let today = chrono::NaiveDate::from_ymd_opt(2024, 4, 1).unwrap();
        let seq = make_sequence("SEQ", "gapless", 1, 1, 0, "0", None, None, None, false, Some("quarterly"), Some(last_reset));
        assert!(DocumentSequencingEngine::needs_reset(&seq, today));
    }

    #[test]
    fn test_quarterly_boundary_q2_to_q3() {
        let last_reset = chrono::NaiveDate::from_ymd_opt(2024, 5, 15).unwrap();
        let today = chrono::NaiveDate::from_ymd_opt(2024, 7, 1).unwrap();
        let seq = make_sequence("SEQ", "gapless", 1, 1, 0, "0", None, None, None, false, Some("quarterly"), Some(last_reset));
        assert!(DocumentSequencingEngine::needs_reset(&seq, today));
    }

    #[test]
    fn test_quarterly_boundary_q3_to_q4() {
        let last_reset = chrono::NaiveDate::from_ymd_opt(2024, 9, 30).unwrap();
        let today = chrono::NaiveDate::from_ymd_opt(2024, 10, 1).unwrap();
        let seq = make_sequence("SEQ", "gapless", 1, 1, 0, "0", None, None, None, false, Some("quarterly"), Some(last_reset));
        assert!(DocumentSequencingEngine::needs_reset(&seq, today));
    }

    #[test]
    fn test_quarterly_boundary_q4_to_q1() {
        let last_reset = chrono::NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();
        let today = chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        let seq = make_sequence("SEQ", "gapless", 1, 1, 0, "0", None, None, None, false, Some("quarterly"), Some(last_reset));
        assert!(DocumentSequencingEngine::needs_reset(&seq, today));
    }
}
