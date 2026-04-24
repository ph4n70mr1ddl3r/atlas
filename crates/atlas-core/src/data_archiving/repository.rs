//! Data Archiving and Retention Management Repository
//!
//! PostgreSQL storage for retention policies, legal holds, archived records,
//! archive batches, and audit trail.

use atlas_shared::{
    ArchivedRecord, ArchiveAudit, ArchiveBatch,
    DataArchivingDashboard, LegalHold, LegalHoldItem, RetentionPolicy,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

/// Repository trait for data archiving storage
#[async_trait]
pub trait DataArchivingRepository: Send + Sync {
    // Retention Policies
    #[allow(clippy::too_many_arguments)]
    async fn create_policy(
        &self,
        org_id: Uuid,
        policy_code: &str,
        name: &str,
        description: Option<&str>,
        entity_type: &str,
        retention_days: i32,
        action_type: &str,
        purge_after_days: Option<i32>,
        condition_expression: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<RetentionPolicy>;

    async fn get_policy(&self, id: Uuid) -> AtlasResult<Option<RetentionPolicy>>;
    async fn get_policy_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<RetentionPolicy>>;
    async fn list_policies(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        entity_type: Option<&str>,
    ) -> AtlasResult<Vec<RetentionPolicy>>;
    async fn update_policy_status(&self, id: Uuid, status: &str) -> AtlasResult<RetentionPolicy>;
    async fn delete_policy(&self, id: Uuid) -> AtlasResult<()>;

    // Legal Holds
    #[allow(clippy::too_many_arguments)]
    async fn create_legal_hold(
        &self,
        org_id: Uuid,
        hold_number: &str,
        name: &str,
        description: Option<&str>,
        reason: Option<&str>,
        case_reference: Option<&str>,
        authorized_by: Option<Uuid>,
        effective_from: chrono::NaiveDate,
        effective_to: Option<chrono::NaiveDate>,
    ) -> AtlasResult<LegalHold>;

    async fn get_legal_hold(&self, id: Uuid) -> AtlasResult<Option<LegalHold>>;
    async fn get_legal_hold_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<LegalHold>>;
    async fn list_legal_holds(
        &self,
        org_id: Uuid,
        status: Option<&str>,
    ) -> AtlasResult<Vec<LegalHold>>;
    async fn release_legal_hold(
        &self,
        id: Uuid,
        released_by: Uuid,
        reason: Option<&str>,
    ) -> AtlasResult<LegalHold>;
    async fn delete_legal_hold(&self, id: Uuid) -> AtlasResult<()>;

    // Legal Hold Items
    async fn add_legal_hold_item(
        &self,
        org_id: Uuid,
        legal_hold_id: Uuid,
        entity_type: &str,
        record_id: Uuid,
    ) -> AtlasResult<LegalHoldItem>;
    async fn list_legal_hold_items(&self, legal_hold_id: Uuid) -> AtlasResult<Vec<LegalHoldItem>>;
    async fn remove_legal_hold_item(&self, id: Uuid) -> AtlasResult<()>;
    async fn is_record_under_hold(
        &self,
        org_id: Uuid,
        entity_type: &str,
        record_id: Uuid,
    ) -> AtlasResult<bool>;

    // Archived Records
    #[allow(clippy::too_many_arguments)]
    async fn create_archived_record(
        &self,
        org_id: Uuid,
        entity_type: &str,
        original_record_id: Uuid,
        original_data: &serde_json::Value,
        retention_policy_id: Option<Uuid>,
        archive_batch_id: Option<Uuid>,
        original_created_at: Option<chrono::DateTime<chrono::Utc>>,
        original_updated_at: Option<chrono::DateTime<chrono::Utc>>,
        archived_by: Option<Uuid>,
    ) -> AtlasResult<ArchivedRecord>;
    async fn get_archived_record(&self, id: Uuid) -> AtlasResult<Option<ArchivedRecord>>;
    async fn is_record_archived(
        &self,
        org_id: Uuid,
        entity_type: &str,
        record_id: Uuid,
    ) -> AtlasResult<bool>;
    async fn list_archived_records(
        &self,
        org_id: Uuid,
        entity_type: Option<&str>,
        status: Option<&str>,
        limit: Option<i32>,
    ) -> AtlasResult<Vec<ArchivedRecord>>;
    async fn restore_archived_record(
        &self,
        id: Uuid,
        restored_by: Option<Uuid>,
    ) -> AtlasResult<ArchivedRecord>;
    async fn purge_archived_record(
        &self,
        id: Uuid,
        purged_by: Option<Uuid>,
    ) -> AtlasResult<ArchivedRecord>;

    // Archive Batches
    #[allow(clippy::too_many_arguments)]
    async fn create_archive_batch(
        &self,
        org_id: Uuid,
        batch_number: &str,
        retention_policy_id: Option<Uuid>,
        entity_type: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ArchiveBatch>;
    async fn get_archive_batch(&self, id: Uuid) -> AtlasResult<Option<ArchiveBatch>>;
    async fn list_archive_batches(
        &self,
        org_id: Uuid,
        status: Option<&str>,
    ) -> AtlasResult<Vec<ArchiveBatch>>;
    async fn update_archive_batch_status(&self, id: Uuid, status: &str) -> AtlasResult<()>;
    async fn update_archive_batch_counts(
        &self,
        id: Uuid,
        total: i32,
        archived: i32,
        failed: i32,
    ) -> AtlasResult<()>;

    // Find records that qualify for archival
    async fn find_qualifying_records(
        &self,
        org_id: Uuid,
        entity_type: &str,
        cutoff: chrono::DateTime<chrono::Utc>,
    ) -> AtlasResult<Vec<(Uuid, serde_json::Value, Option<chrono::DateTime<chrono::Utc>>, Option<chrono::DateTime<chrono::Utc>>)>>;

    // Audit
    #[allow(clippy::too_many_arguments)]
    async fn create_audit_entry(
        &self,
        org_id: Uuid,
        operation: &str,
        entity_type: &str,
        record_id: Option<Uuid>,
        batch_id: Option<Uuid>,
        legal_hold_id: Option<Uuid>,
        retention_policy_id: Option<Uuid>,
        result: &str,
        details: Option<&str>,
        performed_by: Option<Uuid>,
    ) -> AtlasResult<ArchiveAudit>;
    async fn list_audit_entries(
        &self,
        org_id: Uuid,
        operation: Option<&str>,
        entity_type: Option<&str>,
        limit: Option<i32>,
    ) -> AtlasResult<Vec<ArchiveAudit>>;

    // Dashboard
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<DataArchivingDashboard>;
}

/// PostgreSQL implementation
pub struct PostgresDataArchivingRepository {
    pool: PgPool,
}

impl PostgresDataArchivingRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn row_to_policy(row: &sqlx::postgres::PgRow) -> RetentionPolicy {
    RetentionPolicy {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        policy_code: row.get("policy_code"),
        name: row.get("name"),
        description: row.get("description"),
        entity_type: row.get("entity_type"),
        retention_days: row.get("retention_days"),
        action_type: row.get("action_type"),
        purge_after_days: row.get("purge_after_days"),
        condition_expression: row.get("condition_expression"),
        status: row.get("status"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::Value::Null),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_legal_hold(row: &sqlx::postgres::PgRow) -> LegalHold {
    LegalHold {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        hold_number: row.get("hold_number"),
        name: row.get("name"),
        description: row.get("description"),
        status: row.get("status"),
        reason: row.get("reason"),
        case_reference: row.get("case_reference"),
        authorized_by: row.get("authorized_by"),
        released_at: row.get("released_at"),
        released_by: row.get("released_by"),
        release_reason: row.get("release_reason"),
        effective_from: row.get("effective_from"),
        effective_to: row.get("effective_to"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::Value::Null),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_legal_hold_item(row: &sqlx::postgres::PgRow) -> LegalHoldItem {
    LegalHoldItem {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        legal_hold_id: row.get("legal_hold_id"),
        entity_type: row.get("entity_type"),
        record_id: row.get("record_id"),
        created_at: row.get("created_at"),
    }
}

fn row_to_archived_record(row: &sqlx::postgres::PgRow) -> ArchivedRecord {
    ArchivedRecord {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        entity_type: row.get("entity_type"),
        original_record_id: row.get("original_record_id"),
        original_data: row.get("original_data"),
        retention_policy_id: row.get("retention_policy_id"),
        archive_batch_id: row.get("archive_batch_id"),
        status: row.get("status"),
        original_created_at: row.get("original_created_at"),
        original_updated_at: row.get("original_updated_at"),
        archived_at: row.get("archived_at"),
        archived_by: row.get("archived_by"),
        restored_at: row.get("restored_at"),
        restored_by: row.get("restored_by"),
        purged_at: row.get("purged_at"),
        purged_by: row.get("purged_by"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::Value::Null),
        created_at: row.get("created_at"),
    }
}

fn row_to_archive_batch(row: &sqlx::postgres::PgRow) -> ArchiveBatch {
    ArchiveBatch {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        batch_number: row.get("batch_number"),
        retention_policy_id: row.get("retention_policy_id"),
        entity_type: row.get("entity_type"),
        status: row.get("status"),
        total_records: row.get("total_records"),
        archived_records: row.get("archived_records"),
        failed_records: row.get("failed_records"),
        criteria: row.try_get("criteria").unwrap_or(serde_json::Value::Null),
        started_at: row.get("started_at"),
        completed_at: row.get("completed_at"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::Value::Null),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_audit(row: &sqlx::postgres::PgRow) -> ArchiveAudit {
    ArchiveAudit {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        operation: row.get("operation"),
        entity_type: row.get("entity_type"),
        record_id: row.get("record_id"),
        batch_id: row.get("batch_id"),
        legal_hold_id: row.get("legal_hold_id"),
        retention_policy_id: row.get("retention_policy_id"),
        result: row.get("result"),
        details: row.get("details"),
        performed_by: row.get("performed_by"),
        performed_at: row.get("performed_at"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::Value::Null),
    }
}

#[async_trait]
impl DataArchivingRepository for PostgresDataArchivingRepository {
    // ========================================================================
    // Retention Policies
    // ========================================================================

    async fn create_policy(
        &self,
        org_id: Uuid,
        policy_code: &str,
        name: &str,
        description: Option<&str>,
        entity_type: &str,
        retention_days: i32,
        action_type: &str,
        purge_after_days: Option<i32>,
        condition_expression: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<RetentionPolicy> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.retention_policies
                (organization_id, policy_code, name, description,
                 entity_type, retention_days, action_type, purge_after_days,
                 condition_expression, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
            RETURNING *"#,
        )
        .bind(org_id).bind(policy_code).bind(name).bind(description)
        .bind(entity_type).bind(retention_days).bind(action_type)
        .bind(purge_after_days).bind(condition_expression).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_policy(&row))
    }

    async fn get_policy(&self, id: Uuid) -> AtlasResult<Option<RetentionPolicy>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.retention_policies WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| row_to_policy(&r)))
    }

    async fn get_policy_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<RetentionPolicy>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.retention_policies WHERE organization_id = $1 AND policy_code = $2"
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| row_to_policy(&r)))
    }

    async fn list_policies(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        entity_type: Option<&str>,
    ) -> AtlasResult<Vec<RetentionPolicy>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.retention_policies
            WHERE organization_id = $1
              AND ($2::text IS NULL OR status = $2)
              AND ($3::text IS NULL OR entity_type = $3)
            ORDER BY policy_code"#,
        )
        .bind(org_id).bind(status).bind(entity_type)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(row_to_policy).collect())
    }

    async fn update_policy_status(&self, id: Uuid, status: &str) -> AtlasResult<RetentionPolicy> {
        let row = sqlx::query(
            r#"UPDATE _atlas.retention_policies
            SET status = $2, updated_at = now()
            WHERE id = $1
            RETURNING *"#,
        )
        .bind(id).bind(status)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_policy(&row))
    }

    async fn delete_policy(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.retention_policies WHERE id = $1")
            .bind(id)
            .execute(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Legal Holds
    // ========================================================================

    async fn create_legal_hold(
        &self,
        org_id: Uuid,
        hold_number: &str,
        name: &str,
        description: Option<&str>,
        reason: Option<&str>,
        case_reference: Option<&str>,
        authorized_by: Option<Uuid>,
        effective_from: chrono::NaiveDate,
        effective_to: Option<chrono::NaiveDate>,
    ) -> AtlasResult<LegalHold> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.legal_holds
                (organization_id, hold_number, name, description,
                 reason, case_reference, authorized_by,
                 effective_from, effective_to, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
            RETURNING *"#,
        )
        .bind(org_id).bind(hold_number).bind(name).bind(description)
        .bind(reason).bind(case_reference).bind(authorized_by)
        .bind(effective_from).bind(effective_to).bind(authorized_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_legal_hold(&row))
    }

    async fn get_legal_hold(&self, id: Uuid) -> AtlasResult<Option<LegalHold>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.legal_holds WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| row_to_legal_hold(&r)))
    }

    async fn get_legal_hold_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<LegalHold>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.legal_holds WHERE organization_id = $1 AND hold_number = $2"
        )
        .bind(org_id).bind(number)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| row_to_legal_hold(&r)))
    }

    async fn list_legal_holds(
        &self,
        org_id: Uuid,
        status: Option<&str>,
    ) -> AtlasResult<Vec<LegalHold>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.legal_holds
            WHERE organization_id = $1
              AND ($2::text IS NULL OR status = $2)
            ORDER BY created_at DESC"#,
        )
        .bind(org_id).bind(status)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(row_to_legal_hold).collect())
    }

    async fn release_legal_hold(
        &self,
        id: Uuid,
        released_by: Uuid,
        reason: Option<&str>,
    ) -> AtlasResult<LegalHold> {
        let row = sqlx::query(
            r#"UPDATE _atlas.legal_holds
            SET status = 'released', released_at = now(),
                released_by = $2, release_reason = $3,
                updated_at = now()
            WHERE id = $1
            RETURNING *"#,
        )
        .bind(id).bind(released_by).bind(reason)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_legal_hold(&row))
    }

    async fn delete_legal_hold(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.legal_holds WHERE id = $1")
            .bind(id)
            .execute(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Legal Hold Items
    // ========================================================================

    async fn add_legal_hold_item(
        &self,
        org_id: Uuid,
        legal_hold_id: Uuid,
        entity_type: &str,
        record_id: Uuid,
    ) -> AtlasResult<LegalHoldItem> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.legal_hold_items
                (organization_id, legal_hold_id, entity_type, record_id)
            VALUES ($1, $2, $3, $4)
            RETURNING *"#,
        )
        .bind(org_id).bind(legal_hold_id).bind(entity_type).bind(record_id)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_legal_hold_item(&row))
    }

    async fn list_legal_hold_items(&self, legal_hold_id: Uuid) -> AtlasResult<Vec<LegalHoldItem>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.legal_hold_items WHERE legal_hold_id = $1 ORDER BY created_at"
        )
        .bind(legal_hold_id)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(row_to_legal_hold_item).collect())
    }

    async fn remove_legal_hold_item(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.legal_hold_items WHERE id = $1")
            .bind(id)
            .execute(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn is_record_under_hold(
        &self,
        org_id: Uuid,
        entity_type: &str,
        record_id: Uuid,
    ) -> AtlasResult<bool> {
        let row = sqlx::query(
            r#"SELECT COUNT(*) as cnt FROM _atlas.legal_hold_items lhi
            JOIN _atlas.legal_holds lh ON lh.id = lhi.legal_hold_id
            WHERE lhi.organization_id = $1
              AND lhi.entity_type = $2
              AND lhi.record_id = $3
              AND lh.status = 'active'"#,
        )
        .bind(org_id).bind(entity_type).bind(record_id)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let cnt: i64 = row.get("cnt");
        Ok(cnt > 0)
    }

    // ========================================================================
    // Archived Records
    // ========================================================================

    async fn create_archived_record(
        &self,
        org_id: Uuid,
        entity_type: &str,
        original_record_id: Uuid,
        original_data: &serde_json::Value,
        retention_policy_id: Option<Uuid>,
        archive_batch_id: Option<Uuid>,
        original_created_at: Option<chrono::DateTime<chrono::Utc>>,
        original_updated_at: Option<chrono::DateTime<chrono::Utc>>,
        archived_by: Option<Uuid>,
    ) -> AtlasResult<ArchivedRecord> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.archived_records
                (organization_id, entity_type, original_record_id, original_data,
                 retention_policy_id, archive_batch_id,
                 original_created_at, original_updated_at, archived_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
            RETURNING *"#,
        )
        .bind(org_id).bind(entity_type).bind(original_record_id).bind(original_data)
        .bind(retention_policy_id).bind(archive_batch_id)
        .bind(original_created_at).bind(original_updated_at).bind(archived_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_archived_record(&row))
    }

    async fn get_archived_record(&self, id: Uuid) -> AtlasResult<Option<ArchivedRecord>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.archived_records WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| row_to_archived_record(&r)))
    }

    async fn is_record_archived(
        &self,
        org_id: Uuid,
        entity_type: &str,
        record_id: Uuid,
    ) -> AtlasResult<bool> {
        let row = sqlx::query(
            r#"SELECT COUNT(*) as cnt FROM _atlas.archived_records
            WHERE organization_id = $1
              AND entity_type = $2
              AND original_record_id = $3
              AND status = 'archived'"#,
        )
        .bind(org_id).bind(entity_type).bind(record_id)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let cnt: i64 = row.get("cnt");
        Ok(cnt > 0)
    }

    async fn list_archived_records(
        &self,
        org_id: Uuid,
        entity_type: Option<&str>,
        status: Option<&str>,
        limit: Option<i32>,
    ) -> AtlasResult<Vec<ArchivedRecord>> {
        let limit_val = limit.unwrap_or(100);
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.archived_records
            WHERE organization_id = $1
              AND ($2::text IS NULL OR entity_type = $2)
              AND ($3::text IS NULL OR status = $3)
            ORDER BY archived_at DESC
            LIMIT $4"#,
        )
        .bind(org_id).bind(entity_type).bind(status).bind(limit_val)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(row_to_archived_record).collect())
    }

    async fn restore_archived_record(
        &self,
        id: Uuid,
        restored_by: Option<Uuid>,
    ) -> AtlasResult<ArchivedRecord> {
        let row = sqlx::query(
            r#"UPDATE _atlas.archived_records
            SET status = 'restored', restored_at = now(), restored_by = $2
            WHERE id = $1
            RETURNING *"#,
        )
        .bind(id).bind(restored_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_archived_record(&row))
    }

    async fn purge_archived_record(
        &self,
        id: Uuid,
        purged_by: Option<Uuid>,
    ) -> AtlasResult<ArchivedRecord> {
        let row = sqlx::query(
            r#"UPDATE _atlas.archived_records
            SET status = 'purged', purged_at = now(), purged_by = $2
            WHERE id = $1
            RETURNING *"#,
        )
        .bind(id).bind(purged_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_archived_record(&row))
    }

    // ========================================================================
    // Archive Batches
    // ========================================================================

    async fn create_archive_batch(
        &self,
        org_id: Uuid,
        batch_number: &str,
        retention_policy_id: Option<Uuid>,
        entity_type: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ArchiveBatch> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.archive_batches
                (organization_id, batch_number, retention_policy_id,
                 entity_type, created_by)
            VALUES ($1,$2,$3,$4,$5)
            RETURNING *"#,
        )
        .bind(org_id).bind(batch_number).bind(retention_policy_id)
        .bind(entity_type).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_archive_batch(&row))
    }

    async fn get_archive_batch(&self, id: Uuid) -> AtlasResult<Option<ArchiveBatch>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.archive_batches WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| row_to_archive_batch(&r)))
    }

    async fn list_archive_batches(
        &self,
        org_id: Uuid,
        status: Option<&str>,
    ) -> AtlasResult<Vec<ArchiveBatch>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.archive_batches
            WHERE organization_id = $1
              AND ($2::text IS NULL OR status = $2)
            ORDER BY created_at DESC"#,
        )
        .bind(org_id).bind(status)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(row_to_archive_batch).collect())
    }

    async fn update_archive_batch_status(&self, id: Uuid, status: &str) -> AtlasResult<()> {
        let start_col = if status == "in_progress" { ", started_at = now()" } else if status == "completed" { ", completed_at = now()" } else { "" };
        let query = format!(
            "UPDATE _atlas.archive_batches SET status = $2{}, updated_at = now() WHERE id = $1",
            start_col
        );
        sqlx::query(&query)
            .bind(id).bind(status)
            .execute(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn update_archive_batch_counts(
        &self,
        id: Uuid,
        total: i32,
        archived: i32,
        failed: i32,
    ) -> AtlasResult<()> {
        sqlx::query(
            r#"UPDATE _atlas.archive_batches
            SET total_records = $2, archived_records = $3, failed_records = $4,
                updated_at = now()
            WHERE id = $1"#,
        )
        .bind(id).bind(total).bind(archived).bind(failed)
        .execute(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn find_qualifying_records(
        &self,
        org_id: Uuid,
        entity_type: &str,
        cutoff: chrono::DateTime<chrono::Utc>,
    ) -> AtlasResult<Vec<(Uuid, serde_json::Value, Option<chrono::DateTime<chrono::Utc>>, Option<chrono::DateTime<chrono::Utc>>)>> {
        // Query records from the entity's table that are older than the cutoff.
        // The entity_type maps to a table name. For test_items, we query directly.
        // In production, this would use the schema engine to find the right table.
        let table_name = entity_type;

        let query_str = format!(
            r#"SELECT id, to_jsonb(t) as data, created_at, updated_at
            FROM {} t
            WHERE (t.organization_id = $1 OR t.organization_id IS NULL)
              AND t.created_at < $2
            ORDER BY t.created_at ASC
            LIMIT 1000"#,
            table_name
        );

        let rows = sqlx::query(&query_str)
            .bind(org_id).bind(cutoff)
            .fetch_all(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let mut results = Vec::new();
        for row in &rows {
            let id: Uuid = row.get("id");
            let data: serde_json::Value = row.get("data");
            let created_at: Option<chrono::DateTime<chrono::Utc>> = row.try_get("created_at").ok();
            let updated_at: Option<chrono::DateTime<chrono::Utc>> = row.try_get("updated_at").ok();
            results.push((id, data, created_at, updated_at));
        }

        Ok(results)
    }

    // ========================================================================
    // Audit
    // ========================================================================

    async fn create_audit_entry(
        &self,
        org_id: Uuid,
        operation: &str,
        entity_type: &str,
        record_id: Option<Uuid>,
        batch_id: Option<Uuid>,
        legal_hold_id: Option<Uuid>,
        retention_policy_id: Option<Uuid>,
        result: &str,
        details: Option<&str>,
        performed_by: Option<Uuid>,
    ) -> AtlasResult<ArchiveAudit> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.archive_audit
                (organization_id, operation, entity_type, record_id,
                 batch_id, legal_hold_id, retention_policy_id,
                 result, details, performed_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
            RETURNING *"#,
        )
        .bind(org_id).bind(operation).bind(entity_type).bind(record_id)
        .bind(batch_id).bind(legal_hold_id).bind(retention_policy_id)
        .bind(result).bind(details).bind(performed_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_audit(&row))
    }

    async fn list_audit_entries(
        &self,
        org_id: Uuid,
        operation: Option<&str>,
        entity_type: Option<&str>,
        limit: Option<i32>,
    ) -> AtlasResult<Vec<ArchiveAudit>> {
        let limit_val = limit.unwrap_or(100);
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.archive_audit
            WHERE organization_id = $1
              AND ($2::text IS NULL OR operation = $2)
              AND ($3::text IS NULL OR entity_type = $3)
            ORDER BY performed_at DESC
            LIMIT $4"#,
        )
        .bind(org_id).bind(operation).bind(entity_type).bind(limit_val)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(row_to_audit).collect())
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<DataArchivingDashboard> {
        // Policy stats
        let policy_row = sqlx::query(
            r#"SELECT
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE status = 'active') as active
            FROM _atlas.retention_policies WHERE organization_id = $1"#,
        )
        .bind(org_id)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let total_policies: i64 = policy_row.try_get("total").unwrap_or(0);
        let active_policies: i64 = policy_row.try_get("active").unwrap_or(0);

        // Legal hold stats
        let hold_row = sqlx::query(
            r#"SELECT
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE status = 'active') as active
            FROM _atlas.legal_holds WHERE organization_id = $1"#,
        )
        .bind(org_id)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let total_legal_holds: i64 = hold_row.try_get("total").unwrap_or(0);
        let active_legal_holds: i64 = hold_row.try_get("active").unwrap_or(0);

        // Archived record stats
        let archive_row = sqlx::query(
            r#"SELECT
                COUNT(*) FILTER (WHERE status = 'archived') as archived,
                COUNT(*) FILTER (WHERE status = 'purged') as purged,
                COUNT(*) FILTER (WHERE status = 'restored') as restored
            FROM _atlas.archived_records WHERE organization_id = $1"#,
        )
        .bind(org_id)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let total_archived_records: i64 = archive_row.try_get("archived").unwrap_or(0);
        let total_purged_records: i64 = archive_row.try_get("purged").unwrap_or(0);
        let total_restored_records: i64 = archive_row.try_get("restored").unwrap_or(0);

        // Policies by entity type
        let et_rows = sqlx::query(
            r#"SELECT entity_type, COUNT(*) as cnt
            FROM _atlas.retention_policies
            WHERE organization_id = $1 AND status = 'active'
            GROUP BY entity_type"#,
        )
        .bind(org_id)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let mut by_et = serde_json::Map::new();
        for r in &et_rows {
            let et: String = r.get("entity_type");
            let cnt: i64 = r.get("cnt");
            by_et.insert(et, serde_json::Value::Number(cnt.into()));
        }

        // Recent audit entries
        let recent_audits = self.list_audit_entries(org_id, None, None, Some(10)).await?;

        Ok(DataArchivingDashboard {
            total_policies,
            active_policies,
            total_legal_holds,
            active_legal_holds,
            total_archived_records,
            total_purged_records,
            total_restored_records,
            policies_by_entity_type: serde_json::Value::Object(by_et),
            recent_audit_entries: recent_audits,
        })
    }
}
