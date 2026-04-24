//! Data Archiving and Retention Management Engine
//!
//! Business logic for managing data lifecycle: retention policies,
//! archival, purging, legal holds, and restoration.

use atlas_shared::{
    ArchivedRecord, ArchiveAudit, ArchiveBatch,
    CreateLegalHoldRequest, CreateRetentionPolicyRequest,
    DataArchivingDashboard, LegalHold, LegalHoldItem, RetentionPolicy,
    AtlasError, AtlasResult,
};
use super::DataArchivingRepository;
use std::sync::Arc;
use uuid::Uuid;
use tracing::info;

/// Valid action types for retention policies
const VALID_ACTION_TYPES: &[&str] = &["archive", "purge", "archive_then_purge"];

/// Valid statuses for retention policies
#[allow(dead_code)] // Used for future validation endpoints
const VALID_POLICY_STATUSES: &[&str] = &["active", "inactive"];

/// Valid statuses for legal holds
#[allow(dead_code)] // Used for future validation endpoints
const VALID_HOLD_STATUSES: &[&str] = &["active", "released", "expired"];

/// Valid statuses for archived records
#[allow(dead_code)] // Used for future validation endpoints
const VALID_ARCHIVE_STATUSES: &[&str] = &["archived", "restored", "purged"];

/// Data Archiving Engine
pub struct DataArchivingEngine {
    repository: Arc<dyn DataArchivingRepository>,
}

impl DataArchivingEngine {
    pub fn new(repository: Arc<dyn DataArchivingRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Retention Policies
    // ========================================================================

    /// Create a new retention policy.
    pub async fn create_policy(
        &self,
        org_id: Uuid,
        request: CreateRetentionPolicyRequest,
        created_by: Option<Uuid>,
    ) -> AtlasResult<RetentionPolicy> {
        if request.policy_code.is_empty() {
            return Err(AtlasError::ValidationFailed("policy_code is required".into()));
        }
        if request.name.is_empty() {
            return Err(AtlasError::ValidationFailed("name is required".into()));
        }
        if request.entity_type.is_empty() {
            return Err(AtlasError::ValidationFailed("entity_type is required".into()));
        }
        if request.retention_days < 0 {
            return Err(AtlasError::ValidationFailed(
                "retention_days must be non-negative".into(),
            ));
        }
        if !VALID_ACTION_TYPES.contains(&request.action_type.as_str()) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid action_type '{}'. Must be one of: {}",
                request.action_type,
                VALID_ACTION_TYPES.join(", ")
            )));
        }
        if request.action_type == "archive_then_purge" {
            if let Some(pad) = request.purge_after_days {
                if pad <= 0 {
                    return Err(AtlasError::ValidationFailed(
                        "purge_after_days must be greater than 0".into(),
                    ));
                }
            }
        }

        // Uniqueness check
        if self.repository.get_policy_by_code(org_id, &request.policy_code).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Retention policy with code '{}' already exists", request.policy_code
            )));
        }

        info!(
            "Creating retention policy {} for entity {} ({} days, {})",
            request.policy_code, request.entity_type,
            request.retention_days, request.action_type
        );

        self.repository.create_policy(
            org_id,
            &request.policy_code,
            &request.name,
            request.description.as_deref(),
            &request.entity_type,
            request.retention_days,
            &request.action_type,
            request.purge_after_days,
            request.condition_expression.as_deref(),
            created_by,
        ).await
    }

    /// Get a retention policy by ID.
    pub async fn get_policy(&self, id: Uuid) -> AtlasResult<Option<RetentionPolicy>> {
        self.repository.get_policy(id).await
    }

    /// Get a retention policy by code.
    pub async fn get_policy_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<RetentionPolicy>> {
        self.repository.get_policy_by_code(org_id, code).await
    }

    /// List retention policies with optional filters.
    pub async fn list_policies(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        entity_type: Option<&str>,
    ) -> AtlasResult<Vec<RetentionPolicy>> {
        self.repository.list_policies(org_id, status, entity_type).await
    }

    /// Activate a retention policy.
    pub async fn activate_policy(&self, id: Uuid) -> AtlasResult<RetentionPolicy> {
        let policy = self.get_policy(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Policy {} not found", id)))?;

        if policy.status == "active" {
            return Err(AtlasError::WorkflowError("Policy is already active".into()));
        }

        info!("Activated retention policy {}", policy.policy_code);
        self.repository.update_policy_status(id, "active").await
    }

    /// Deactivate a retention policy.
    pub async fn deactivate_policy(&self, id: Uuid) -> AtlasResult<RetentionPolicy> {
        let policy = self.get_policy(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Policy {} not found", id)))?;

        if policy.status == "inactive" {
            return Err(AtlasError::WorkflowError("Policy is already inactive".into()));
        }

        info!("Deactivated retention policy {}", policy.policy_code);
        self.repository.update_policy_status(id, "inactive").await
    }

    /// Delete a retention policy.
    pub async fn delete_policy(&self, id: Uuid) -> AtlasResult<()> {
        info!("Deleted retention policy {}", id);
        self.repository.delete_policy(id).await
    }

    // ========================================================================
    // Legal Holds
    // ========================================================================

    /// Create a new legal hold.
    pub async fn create_legal_hold(
        &self,
        org_id: Uuid,
        request: CreateLegalHoldRequest,
        created_by: Option<Uuid>,
    ) -> AtlasResult<LegalHold> {
        if request.hold_number.is_empty() {
            return Err(AtlasError::ValidationFailed("hold_number is required".into()));
        }
        if request.name.is_empty() {
            return Err(AtlasError::ValidationFailed("name is required".into()));
        }

        // Uniqueness check
        if self.repository.get_legal_hold_by_number(org_id, &request.hold_number).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Legal hold with number '{}' already exists", request.hold_number
            )));
        }

        let effective_from = request.effective_from
            .unwrap_or_else(|| chrono::Utc::now().date_naive());

        if let (Some(from), Some(to)) = (Some(effective_from), request.effective_to) {
            if to < from {
                return Err(AtlasError::ValidationFailed(
                    "effective_to must be after effective_from".into(),
                ));
            }
        }

        info!("Creating legal hold {} ({})", request.hold_number, request.name);

        self.repository.create_legal_hold(
            org_id,
            &request.hold_number,
            &request.name,
            request.description.as_deref(),
            request.reason.as_deref(),
            request.case_reference.as_deref(),
            created_by,
            effective_from,
            request.effective_to,
        ).await
    }

    /// Get a legal hold by ID.
    pub async fn get_legal_hold(&self, id: Uuid) -> AtlasResult<Option<LegalHold>> {
        self.repository.get_legal_hold(id).await
    }

    /// List legal holds with optional status filter.
    pub async fn list_legal_holds(
        &self,
        org_id: Uuid,
        status: Option<&str>,
    ) -> AtlasResult<Vec<LegalHold>> {
        self.repository.list_legal_holds(org_id, status).await
    }

    /// Release a legal hold.
    pub async fn release_legal_hold(
        &self,
        id: Uuid,
        released_by: Uuid,
        reason: Option<&str>,
    ) -> AtlasResult<LegalHold> {
        let hold = self.get_legal_hold(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Legal hold {} not found", id)))?;

        if hold.status != "active" {
            return Err(AtlasError::WorkflowError(
                format!("Legal hold is '{}' (not active), cannot release", hold.status)
            ));
        }

        info!("Releasing legal hold {}", hold.hold_number);
        self.repository.release_legal_hold(id, released_by, reason).await
    }

    /// Delete a legal hold.
    pub async fn delete_legal_hold(&self, id: Uuid) -> AtlasResult<()> {
        info!("Deleted legal hold {}", id);
        self.repository.delete_legal_hold(id).await
    }

    // ========================================================================
    // Legal Hold Items
    // ========================================================================

    /// Add records to a legal hold.
    pub async fn add_legal_hold_items(
        &self,
        org_id: Uuid,
        legal_hold_id: Uuid,
        items: Vec<(String, Uuid)>,  // (entity_type, record_id)
    ) -> AtlasResult<Vec<LegalHoldItem>> {
        let hold = self.get_legal_hold(legal_hold_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Legal hold {} not found", legal_hold_id)))?;

        if hold.status != "active" {
            return Err(AtlasError::WorkflowError(
                "Cannot add items to a non-active legal hold".into(),
            ));
        }

        let mut results = Vec::new();
        for (entity_type, record_id) in items {
            // Check if already under hold
            if self.repository.is_record_under_hold(org_id, &entity_type, record_id).await? {
                continue; // Skip duplicates
            }

            let item = self.repository.add_legal_hold_item(
                org_id, legal_hold_id, &entity_type, record_id,
            ).await?;

            // Audit
            self.repository.create_audit_entry(
                org_id, "legal_hold", &entity_type, Some(record_id),
                None, Some(legal_hold_id), None, "success",
                Some(&format!("Added to legal hold {}", hold.hold_number)),
                None,
            ).await?;

            results.push(item);
        }

        info!("Added {} items to legal hold {}", results.len(), hold.hold_number);
        Ok(results)
    }

    /// List items under a legal hold.
    pub async fn list_legal_hold_items(
        &self,
        legal_hold_id: Uuid,
    ) -> AtlasResult<Vec<LegalHoldItem>> {
        self.repository.list_legal_hold_items(legal_hold_id).await
    }

    /// Remove a single item from a legal hold.
    pub async fn remove_legal_hold_item(&self, id: Uuid) -> AtlasResult<()> {
        self.repository.remove_legal_hold_item(id).await
    }

    /// Check if a record is under any active legal hold.
    pub async fn is_record_under_hold(
        &self,
        org_id: Uuid,
        entity_type: &str,
        record_id: Uuid,
    ) -> AtlasResult<bool> {
        self.repository.is_record_under_hold(org_id, entity_type, record_id).await
    }

    // ========================================================================
    // Archive Operations
    // ========================================================================

    /// Execute an archive batch: archives records matching a retention policy.
    pub async fn execute_archive(
        &self,
        org_id: Uuid,
        policy_id: Uuid,
        batch_number: &str,
        performed_by: Option<Uuid>,
    ) -> AtlasResult<ArchiveBatch> {
        let policy = self.get_policy(policy_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Policy {} not found", policy_id)))?;

        if policy.status != "active" {
            return Err(AtlasError::WorkflowError(
                "Cannot execute archive with inactive policy".into(),
            ));
        }

        if policy.action_type == "purge" {
            return Err(AtlasError::ValidationFailed(
                "Policy action_type is 'purge'; use execute_purge instead".into(),
            ));
        }

        // Create batch
        let batch = self.repository.create_archive_batch(
            org_id,
            batch_number,
            Some(policy_id),
            &policy.entity_type,
            performed_by,
        ).await?;

        // Update batch status to in_progress
        self.repository.update_archive_batch_status(batch.id, "in_progress").await?;

        // Find qualifying records: records older than retention_days that aren't under legal hold
        let cutoff_date = chrono::Utc::now() - chrono::Duration::days(policy.retention_days as i64);
        let records = self.repository.find_qualifying_records(
            org_id, &policy.entity_type, cutoff_date,
        ).await?;

        let total = records.len() as i32;
        let mut archived = 0i32;
        let mut failed = 0i32;

        for (record_id, record_data, created_at, updated_at) in &records {
            // Check legal hold
            if self.is_record_under_hold(org_id, &policy.entity_type, *record_id).await? {
                self.repository.create_audit_entry(
                    org_id, "archive", &policy.entity_type, Some(*record_id),
                    Some(batch.id), None, Some(policy_id), "skipped",
                    Some("Record under legal hold"), performed_by,
                ).await?;
                failed += 1;
                continue;
            }

            // Check if already archived
            if self.repository.is_record_archived(org_id, &policy.entity_type, *record_id).await? {
                self.repository.create_audit_entry(
                    org_id, "archive", &policy.entity_type, Some(*record_id),
                    Some(batch.id), None, Some(policy_id), "skipped",
                    Some("Record already archived"), performed_by,
                ).await?;
                failed += 1;
                continue;
            }

            // Archive the record
            match self.repository.create_archived_record(
                org_id,
                &policy.entity_type,
                *record_id,
                record_data,
                Some(policy_id),
                Some(batch.id),
                *created_at,
                *updated_at,
                performed_by,
            ).await {
                Ok(_) => {
                    self.repository.create_audit_entry(
                        org_id, "archive", &policy.entity_type, Some(*record_id),
                        Some(batch.id), None, Some(policy_id), "success",
                        None, performed_by,
                    ).await?;
                    archived += 1;
                }
                Err(e) => {
                    self.repository.create_audit_entry(
                        org_id, "archive", &policy.entity_type, Some(*record_id),
                        Some(batch.id), None, Some(policy_id), "failed",
                        Some(&e.to_string()), performed_by,
                    ).await?;
                    failed += 1;
                }
            }
        }

        // Update batch counts
        self.repository.update_archive_batch_counts(batch.id, total, archived, failed).await?;
        self.repository.update_archive_batch_status(batch.id, "completed").await?;

        info!(
            "Archive batch {} completed: {}/{} archived, {} failed",
            batch_number, archived, total, failed
        );

        // Fetch updated batch
        self.repository.get_archive_batch(batch.id).await
            .map(|opt| opt.unwrap())
    }

    /// Restore an archived record.
    pub async fn restore_archived_record(
        &self,
        org_id: Uuid,
        archived_record_id: Uuid,
        performed_by: Option<Uuid>,
    ) -> AtlasResult<ArchivedRecord> {
        let record = self.repository.get_archived_record(archived_record_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Archived record {} not found", archived_record_id)
            ))?;

        if record.status != "archived" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot restore record with status '{}'", record.status)
            ));
        }

        let updated = self.repository.restore_archived_record(archived_record_id, performed_by).await?;

        self.repository.create_audit_entry(
            org_id, "restore", &record.entity_type,
            Some(record.original_record_id),
            record.archive_batch_id, None,
            record.retention_policy_id, "success",
            None, performed_by,
        ).await?;

        info!(
            "Restored archived record {} ({}/{})",
            archived_record_id, record.entity_type, record.original_record_id
        );

        Ok(updated)
    }

    /// Purge an archived record permanently (marks as purged).
    pub async fn purge_archived_record(
        &self,
        org_id: Uuid,
        archived_record_id: Uuid,
        performed_by: Option<Uuid>,
    ) -> AtlasResult<ArchivedRecord> {
        let record = self.repository.get_archived_record(archived_record_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Archived record {} not found", archived_record_id)
            ))?;

        if record.status != "archived" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot purge record with status '{}'. Only 'archived' records can be purged.", record.status)
            ));
        }

        // Check legal hold
        if self.is_record_under_hold(org_id, &record.entity_type, record.original_record_id).await? {
            return Err(AtlasError::WorkflowError(
                "Cannot purge: record is under an active legal hold".into(),
            ));
        }

        let updated = self.repository.purge_archived_record(archived_record_id, performed_by).await?;

        self.repository.create_audit_entry(
            org_id, "purge", &record.entity_type,
            Some(record.original_record_id),
            record.archive_batch_id, None,
            record.retention_policy_id, "success",
            None, performed_by,
        ).await?;

        info!(
            "Purged archived record {} ({}/{})",
            archived_record_id, record.entity_type, record.original_record_id
        );

        Ok(updated)
    }

    // ========================================================================
    // Query Operations
    // ========================================================================

    /// Get an archived record by ID.
    pub async fn get_archived_record(&self, id: Uuid) -> AtlasResult<Option<ArchivedRecord>> {
        self.repository.get_archived_record(id).await
    }

    /// List archived records with filters.
    pub async fn list_archived_records(
        &self,
        org_id: Uuid,
        entity_type: Option<&str>,
        status: Option<&str>,
        limit: Option<i32>,
    ) -> AtlasResult<Vec<ArchivedRecord>> {
        self.repository.list_archived_records(org_id, entity_type, status, limit).await
    }

    /// List archive batches.
    pub async fn list_archive_batches(
        &self,
        org_id: Uuid,
        status: Option<&str>,
    ) -> AtlasResult<Vec<ArchiveBatch>> {
        self.repository.list_archive_batches(org_id, status).await
    }

    /// Get an archive batch by ID.
    pub async fn get_archive_batch(&self, id: Uuid) -> AtlasResult<Option<ArchiveBatch>> {
        self.repository.get_archive_batch(id).await
    }

    /// List archive audit entries.
    pub async fn list_audit_entries(
        &self,
        org_id: Uuid,
        operation: Option<&str>,
        entity_type: Option<&str>,
        limit: Option<i32>,
    ) -> AtlasResult<Vec<ArchiveAudit>> {
        self.repository.list_audit_entries(org_id, operation, entity_type, limit).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get data archiving dashboard summary.
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<DataArchivingDashboard> {
        self.repository.get_dashboard(org_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_action_types() {
        assert!(VALID_ACTION_TYPES.contains(&"archive"));
        assert!(VALID_ACTION_TYPES.contains(&"purge"));
        assert!(VALID_ACTION_TYPES.contains(&"archive_then_purge"));
        assert_eq!(VALID_ACTION_TYPES.len(), 3);
    }

    #[test]
    fn test_valid_policy_statuses() {
        assert!(VALID_POLICY_STATUSES.contains(&"active"));
        assert!(VALID_POLICY_STATUSES.contains(&"inactive"));
    }

    #[test]
    fn test_valid_hold_statuses() {
        assert!(VALID_HOLD_STATUSES.contains(&"active"));
        assert!(VALID_HOLD_STATUSES.contains(&"released"));
        assert!(VALID_HOLD_STATUSES.contains(&"expired"));
    }

    #[test]
    fn test_valid_archive_statuses() {
        assert!(VALID_ARCHIVE_STATUSES.contains(&"archived"));
        assert!(VALID_ARCHIVE_STATUSES.contains(&"restored"));
        assert!(VALID_ARCHIVE_STATUSES.contains(&"purged"));
    }
}
