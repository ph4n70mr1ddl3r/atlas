//! Document Sequencing Repository
//!
//! PostgreSQL storage for document sequences, assignments, and audit trail.

use atlas_shared::{
    DocumentSequence, DocumentSequenceAssignment, DocumentSequenceAudit,
    DocumentSequenceDashboardSummary,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

/// Repository trait for document sequencing data storage
#[async_trait]
pub trait DocumentSequencingRepository: Send + Sync {
    // Sequences
    async fn create_sequence(
        &self, org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        sequence_type: &str, document_type: &str,
        initial_value: i64, increment_by: i32, max_value: Option<i64>,
        cycle_flag: bool, prefix: Option<&str>, suffix: Option<&str>,
        pad_length: i32, pad_character: &str, reset_frequency: Option<&str>,
        effective_from: Option<chrono::NaiveDate>, effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<DocumentSequence>;

    async fn get_sequence(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<DocumentSequence>>;
    async fn get_sequence_by_id(&self, id: Uuid) -> AtlasResult<Option<DocumentSequence>>;
    async fn list_sequences(&self, org_id: Uuid, status: Option<&str>, document_type: Option<&str>) -> AtlasResult<Vec<DocumentSequence>>;
    async fn update_sequence_status(&self, id: Uuid, status: &str) -> AtlasResult<DocumentSequence>;
    async fn delete_sequence(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    /// Atomically increment and return the next value. Used for gapless sequences.
    async fn increment_sequence_value(&self, id: Uuid, increment_by: i32) -> AtlasResult<DocumentSequence>;

    /// Reset the current value back to initial_value and set last_reset_date.
    async fn reset_sequence(&self, id: Uuid, reset_date: chrono::NaiveDate) -> AtlasResult<DocumentSequence>;

    // Assignments
    async fn create_assignment(
        &self, org_id: Uuid, sequence_id: Uuid, sequence_code: &str,
        document_category: &str, business_unit_id: Option<Uuid>,
        ledger_id: Option<Uuid>, method: &str,
        effective_from: Option<chrono::NaiveDate>, effective_to: Option<chrono::NaiveDate>,
        priority: i32, created_by: Option<Uuid>,
    ) -> AtlasResult<DocumentSequenceAssignment>;

    async fn get_assignment(&self, id: Uuid) -> AtlasResult<Option<DocumentSequenceAssignment>>;
    async fn find_assignment(
        &self, org_id: Uuid, document_category: &str,
        business_unit_id: Option<Uuid>, ledger_id: Option<Uuid>,
    ) -> AtlasResult<Option<DocumentSequenceAssignment>>;
    async fn list_assignments(&self, org_id: Uuid, sequence_id: Option<Uuid>) -> AtlasResult<Vec<DocumentSequenceAssignment>>;
    async fn update_assignment_status(&self, id: Uuid, status: &str) -> AtlasResult<DocumentSequenceAssignment>;
    async fn delete_assignment(&self, id: Uuid) -> AtlasResult<()>;

    // Audit
    async fn create_audit_entry(
        &self, org_id: Uuid, sequence_id: Uuid, sequence_code: &str,
        generated_number: &str, numeric_value: i64,
        document_category: &str, document_id: Option<Uuid>,
        document_number: Option<&str>, business_unit_id: Option<Uuid>,
        generated_by: Option<Uuid>, metadata: serde_json::Value,
    ) -> AtlasResult<DocumentSequenceAudit>;

    async fn list_audit_entries(
        &self, org_id: Uuid, sequence_id: Option<Uuid>,
        limit: Option<i32>,
    ) -> AtlasResult<Vec<DocumentSequenceAudit>>;

    async fn get_audit_by_document(&self, document_id: Uuid) -> AtlasResult<Option<DocumentSequenceAudit>>;

    // Dashboard
    async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<DocumentSequenceDashboardSummary>;
}

/// PostgreSQL implementation
pub struct PostgresDocumentSequencingRepository {
    pool: PgPool,
}

impl PostgresDocumentSequencingRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn row_to_sequence(row: &sqlx::postgres::PgRow) -> DocumentSequence {
    DocumentSequence {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        code: row.get("code"),
        name: row.get("name"),
        description: row.get("description"),
        sequence_type: row.get("sequence_type"),
        document_type: row.get("document_type"),
        initial_value: row.get("initial_value"),
        current_value: row.get("current_value"),
        increment_by: row.get("increment_by"),
        max_value: row.get("max_value"),
        cycle_flag: row.get("cycle_flag"),
        prefix: row.get("prefix"),
        suffix: row.get("suffix"),
        pad_length: row.get("pad_length"),
        pad_character: row.get("pad_character"),
        reset_frequency: row.get("reset_frequency"),
        last_reset_date: row.get("last_reset_date"),
        effective_from: row.get("effective_from"),
        effective_to: row.get("effective_to"),
        status: row.get("status"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_assignment(row: &sqlx::postgres::PgRow) -> DocumentSequenceAssignment {
    DocumentSequenceAssignment {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        sequence_id: row.get("sequence_id"),
        sequence_code: row.get("sequence_code"),
        document_category: row.get("document_category"),
        business_unit_id: row.get("business_unit_id"),
        ledger_id: row.get("ledger_id"),
        method: row.get("method"),
        effective_from: row.get("effective_from"),
        effective_to: row.get("effective_to"),
        priority: row.get("priority"),
        status: row.get("status"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_audit(row: &sqlx::postgres::PgRow) -> DocumentSequenceAudit {
    DocumentSequenceAudit {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        sequence_id: row.get("sequence_id"),
        sequence_code: row.get("sequence_code"),
        generated_number: row.get("generated_number"),
        numeric_value: row.get("numeric_value"),
        document_category: row.get("document_category"),
        document_id: row.get("document_id"),
        document_number: row.get("document_number"),
        business_unit_id: row.get("business_unit_id"),
        generated_at: row.get("generated_at"),
        generated_by: row.get("generated_by"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::Value::Null),
        created_at: row.get("created_at"),
    }
}

#[async_trait]
impl DocumentSequencingRepository for PostgresDocumentSequencingRepository {
    async fn create_sequence(
        &self, org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        sequence_type: &str, document_type: &str,
        initial_value: i64, increment_by: i32, max_value: Option<i64>,
        cycle_flag: bool, prefix: Option<&str>, suffix: Option<&str>,
        pad_length: i32, pad_character: &str, reset_frequency: Option<&str>,
        effective_from: Option<chrono::NaiveDate>, effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<DocumentSequence> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.document_sequences
                (organization_id, code, name, description, sequence_type, document_type,
                 initial_value, current_value, increment_by, max_value, cycle_flag,
                 prefix, suffix, pad_length, pad_character, reset_frequency,
                 effective_from, effective_to, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19)
            RETURNING *"#,
        )
        .bind(org_id).bind(code).bind(name).bind(description)
        .bind(sequence_type).bind(document_type)
        .bind(initial_value).bind(initial_value - (increment_by.max(1) as i64)) // current_value starts before initial
        .bind(increment_by).bind(max_value).bind(cycle_flag)
        .bind(prefix).bind(suffix).bind(pad_length).bind(pad_character)
        .bind(reset_frequency).bind(effective_from).bind(effective_to)
        .bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_sequence(&row))
    }

    async fn get_sequence(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<DocumentSequence>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.document_sequences WHERE organization_id=$1 AND code=$2"
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_sequence(&r)))
    }

    async fn get_sequence_by_id(&self, id: Uuid) -> AtlasResult<Option<DocumentSequence>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.document_sequences WHERE id=$1"
        )
        .bind(id)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_sequence(&r)))
    }

    async fn list_sequences(&self, org_id: Uuid, status: Option<&str>, document_type: Option<&str>) -> AtlasResult<Vec<DocumentSequence>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.document_sequences
            WHERE organization_id=$1
              AND ($2::text IS NULL OR status=$2)
              AND ($3::text IS NULL OR document_type=$3)
            ORDER BY code"#,
        )
        .bind(org_id).bind(status).bind(document_type)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_sequence).collect())
    }

    async fn update_sequence_status(&self, id: Uuid, status: &str) -> AtlasResult<DocumentSequence> {
        let row = sqlx::query(
            r#"UPDATE _atlas.document_sequences SET status=$2, updated_at=now()
            WHERE id=$1 RETURNING *"#,
        )
        .bind(id).bind(status)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_sequence(&row))
    }

    async fn delete_sequence(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query(
            "DELETE FROM _atlas.document_sequences WHERE organization_id=$1 AND code=$2"
        )
        .bind(org_id).bind(code)
        .execute(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn increment_sequence_value(&self, id: Uuid, increment_by: i32) -> AtlasResult<DocumentSequence> {
        let row = sqlx::query(
            r#"UPDATE _atlas.document_sequences
            SET current_value = current_value + $2,
                updated_at = now()
            WHERE id=$1
            RETURNING *"#,
        )
        .bind(id).bind(increment_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_sequence(&row))
    }

    async fn reset_sequence(&self, id: Uuid, reset_date: chrono::NaiveDate) -> AtlasResult<DocumentSequence> {
        let row = sqlx::query(
            r#"UPDATE _atlas.document_sequences
            SET current_value = initial_value - increment_by,
                last_reset_date = $2,
                updated_at = now()
            WHERE id=$1
            RETURNING *"#,
        )
        .bind(id).bind(reset_date)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_sequence(&row))
    }

    async fn create_assignment(
        &self, org_id: Uuid, sequence_id: Uuid, sequence_code: &str,
        document_category: &str, business_unit_id: Option<Uuid>,
        ledger_id: Option<Uuid>, method: &str,
        effective_from: Option<chrono::NaiveDate>, effective_to: Option<chrono::NaiveDate>,
        priority: i32, created_by: Option<Uuid>,
    ) -> AtlasResult<DocumentSequenceAssignment> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.document_sequence_assignments
                (organization_id, sequence_id, sequence_code, document_category,
                 business_unit_id, ledger_id, method, effective_from, effective_to,
                 priority, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)
            RETURNING *"#,
        )
        .bind(org_id).bind(sequence_id).bind(sequence_code)
        .bind(document_category).bind(business_unit_id).bind(ledger_id)
        .bind(method).bind(effective_from).bind(effective_to)
        .bind(priority).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_assignment(&row))
    }

    async fn get_assignment(&self, id: Uuid) -> AtlasResult<Option<DocumentSequenceAssignment>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.document_sequence_assignments WHERE id=$1"
        )
        .bind(id)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_assignment(&r)))
    }

    async fn find_assignment(
        &self, org_id: Uuid, document_category: &str,
        business_unit_id: Option<Uuid>, ledger_id: Option<Uuid>,
    ) -> AtlasResult<Option<DocumentSequenceAssignment>> {
        let row = sqlx::query(
            r#"SELECT * FROM _atlas.document_sequence_assignments
            WHERE organization_id=$1
              AND document_category=$2
              AND status='active'
              AND (business_unit_id IS NULL OR business_unit_id=$3)
              AND (ledger_id IS NULL OR ledger_id=$4)
              AND (effective_from IS NULL OR effective_from <= CURRENT_DATE)
              AND (effective_to IS NULL OR effective_to >= CURRENT_DATE)
            ORDER BY priority DESC, business_unit_id IS NOT NULL, ledger_id IS NOT NULL
            LIMIT 1"#,
        )
        .bind(org_id).bind(document_category).bind(business_unit_id).bind(ledger_id)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_assignment(&r)))
    }

    async fn list_assignments(&self, org_id: Uuid, sequence_id: Option<Uuid>) -> AtlasResult<Vec<DocumentSequenceAssignment>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.document_sequence_assignments
            WHERE organization_id=$1 AND ($2::uuid IS NULL OR sequence_id=$2)
            ORDER BY document_category, priority DESC"#,
        )
        .bind(org_id).bind(sequence_id)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_assignment).collect())
    }

    async fn update_assignment_status(&self, id: Uuid, status: &str) -> AtlasResult<DocumentSequenceAssignment> {
        let row = sqlx::query(
            r#"UPDATE _atlas.document_sequence_assignments SET status=$2, updated_at=now()
            WHERE id=$1 RETURNING *"#,
        )
        .bind(id).bind(status)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_assignment(&row))
    }

    async fn delete_assignment(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.document_sequence_assignments WHERE id=$1")
            .bind(id).execute(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn create_audit_entry(
        &self, org_id: Uuid, sequence_id: Uuid, sequence_code: &str,
        generated_number: &str, numeric_value: i64,
        document_category: &str, document_id: Option<Uuid>,
        document_number: Option<&str>, business_unit_id: Option<Uuid>,
        generated_by: Option<Uuid>, metadata: serde_json::Value,
    ) -> AtlasResult<DocumentSequenceAudit> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.document_sequence_audit
                (organization_id, sequence_id, sequence_code, generated_number,
                 numeric_value, document_category, document_id, document_number,
                 business_unit_id, generated_by, metadata)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)
            RETURNING *"#,
        )
        .bind(org_id).bind(sequence_id).bind(sequence_code)
        .bind(generated_number).bind(numeric_value)
        .bind(document_category).bind(document_id).bind(document_number)
        .bind(business_unit_id).bind(generated_by).bind(metadata)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_audit(&row))
    }

    async fn list_audit_entries(
        &self, org_id: Uuid, sequence_id: Option<Uuid>, limit: Option<i32>,
    ) -> AtlasResult<Vec<DocumentSequenceAudit>> {
        let limit_val = limit.unwrap_or(100);
        let rows = if sequence_id.is_some() {
            sqlx::query(
                r#"SELECT * FROM _atlas.document_sequence_audit
                WHERE organization_id=$1 AND sequence_id=$2
                ORDER BY generated_at DESC LIMIT $3"#,
            )
            .bind(org_id).bind(sequence_id).bind(limit_val)
            .fetch_all(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?
        } else {
            sqlx::query(
                r#"SELECT * FROM _atlas.document_sequence_audit
                WHERE organization_id=$1
                ORDER BY generated_at DESC LIMIT $2"#,
            )
            .bind(org_id).bind(limit_val)
            .fetch_all(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?
        };
        Ok(rows.iter().map(row_to_audit).collect())
    }

    async fn get_audit_by_document(&self, document_id: Uuid) -> AtlasResult<Option<DocumentSequenceAudit>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.document_sequence_audit WHERE document_id=$1"
        )
        .bind(document_id)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_audit(&r)))
    }

    async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<DocumentSequenceDashboardSummary> {
        let row = sqlx::query(
            r#"SELECT
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE status = 'active') as active,
                COUNT(*) FILTER (WHERE sequence_type = 'gapless') as gapless,
                COUNT(*) FILTER (WHERE sequence_type = 'gap_permitted') as gap_permitted,
                COALESCE(SUM(current_value), 0) as total_generated
            FROM _atlas.document_sequences WHERE organization_id = $1"#,
        )
        .bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let total: i64 = row.try_get("total").unwrap_or(0);
        let active: i64 = row.try_get("active").unwrap_or(0);
        let gapless: i64 = row.try_get("gapless").unwrap_or(0);
        let gap_permitted: i64 = row.try_get("gap_permitted").unwrap_or(0);
        let total_generated: i64 = row.try_get("total_generated").unwrap_or(0);

        let assignment_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM _atlas.document_sequence_assignments WHERE organization_id=$1"
        )
        .bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let recent_rows = sqlx::query(
            r#"SELECT * FROM _atlas.document_sequence_audit
            WHERE organization_id=$1
            ORDER BY generated_at DESC LIMIT 10"#,
        )
        .bind(org_id).fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(DocumentSequenceDashboardSummary {
            total_sequences: total as i32,
            active_sequences: active as i32,
            gapless_sequences: gapless as i32,
            gap_permitted_sequences: gap_permitted as i32,
            total_numbers_generated: total_generated,
            total_assignments: assignment_count as i32,
            recent_audits: recent_rows.iter().map(row_to_audit).collect(),
            sequences_by_type: serde_json::json!({}),
            sequences_by_document_type: serde_json::json!({}),
        })
    }
}
