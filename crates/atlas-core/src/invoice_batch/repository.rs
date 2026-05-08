//! AP Invoice Batch Repository
//!
//! PostgreSQL storage for AP invoice batches and batch activity audit trail.

use atlas_shared::{AtlasError, AtlasResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use uuid::Uuid;

// ============================================================================
// Data Types
// ============================================================================

/// AP Invoice Batch header record
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct InvoiceBatch {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub batch_number: String,
    pub batch_name: String,
    pub description: Option<String>,
    pub total_invoice_count: i32,
    pub total_invoice_amount: f64,
    pub total_tax_amount: f64,
    pub total_amount: f64,
    pub control_total: Option<f64>,
    pub control_count: Option<i32>,
    pub currency_code: String,
    pub exchange_rate_type: Option<String>,
    pub exchange_rate: Option<f64>,
    pub gl_date: Option<chrono::NaiveDate>,
    pub accounting_period: Option<String>,
    pub source: String,
    pub status: String,
    pub validation_status: String,
    pub validation_errors: serde_json::Value,
    pub submitted_by: Option<Uuid>,
    pub submitted_at: Option<chrono::DateTime<chrono::Utc>>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<chrono::DateTime<chrono::Utc>>,
    pub posted_by: Option<Uuid>,
    pub posted_at: Option<chrono::DateTime<chrono::Utc>>,
    pub cancelled_by: Option<Uuid>,
    pub cancelled_at: Option<chrono::DateTime<chrono::Utc>>,
    pub cancel_reason: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Invoice batch activity log record
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct InvoiceBatchActivity {
    pub id: Uuid,
    pub batch_id: Uuid,
    pub activity_type: String,
    pub description: Option<String>,
    pub old_status: Option<String>,
    pub new_status: Option<String>,
    pub performed_by: Option<Uuid>,
    pub performed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub metadata: serde_json::Value,
}

/// Dashboard summary for invoice batches
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InvoiceBatchSummary {
    pub total_batches: i64,
    pub draft_count: i64,
    pub submitted_count: i64,
    pub approved_count: i64,
    pub posted_count: i64,
    pub cancelled_count: i64,
    pub total_invoice_amount: f64,
    pub total_invoices: i64,
    pub by_source: serde_json::Value,
}

// ============================================================================
// Trait
// ============================================================================

#[async_trait]
pub trait InvoiceBatchRepository: Send + Sync {
    async fn create_batch(
        &self,
        org_id: Uuid,
        batch_number: &str,
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
    ) -> AtlasResult<InvoiceBatch>;

    async fn get_batch(&self, org_id: Uuid, id: Uuid) -> AtlasResult<InvoiceBatch>;
    async fn get_batch_by_number(&self, org_id: Uuid, batch_number: &str) -> AtlasResult<InvoiceBatch>;
    async fn list_batches(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<InvoiceBatch>>;
    async fn delete_batch(&self, org_id: Uuid, batch_number: &str) -> AtlasResult<()>;

    async fn update_status(
        &self,
        id: Uuid,
        status: &str,
        validation_status: Option<&str>,
        validation_errors: Option<serde_json::Value>,
    ) -> AtlasResult<InvoiceBatch>;

    async fn update_totals(
        &self,
        id: Uuid,
        invoice_count: i32,
        invoice_amount: f64,
        tax_amount: f64,
        total_amount: f64,
    ) -> AtlasResult<()>;

    async fn set_submitted(&self, id: Uuid, submitted_by: Uuid) -> AtlasResult<InvoiceBatch>;
    async fn set_approved(&self, id: Uuid, approved_by: Uuid) -> AtlasResult<InvoiceBatch>;
    async fn set_posted(&self, id: Uuid, posted_by: Uuid) -> AtlasResult<InvoiceBatch>;
    async fn set_cancelled(&self, id: Uuid, cancelled_by: Uuid, reason: Option<&str>) -> AtlasResult<InvoiceBatch>;

    async fn add_activity(
        &self,
        batch_id: Uuid,
        activity_type: &str,
        description: Option<&str>,
        old_status: Option<&str>,
        new_status: Option<&str>,
        performed_by: Option<Uuid>,
        metadata: Option<serde_json::Value>,
    ) -> AtlasResult<InvoiceBatchActivity>;

    async fn list_activities(&self, batch_id: Uuid) -> AtlasResult<Vec<InvoiceBatchActivity>>;

    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<InvoiceBatchSummary>;

    async fn recalculate_totals(&self, batch_id: Uuid) -> AtlasResult<InvoiceBatch>;
}

// ============================================================================
// PostgreSQL Implementation
// ============================================================================

pub struct PostgresInvoiceBatchRepository {
    pool: PgPool,
}

impl PostgresInvoiceBatchRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl InvoiceBatchRepository for PostgresInvoiceBatchRepository {
    async fn create_batch(
        &self,
        org_id: Uuid,
        batch_number: &str,
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
        let batch = sqlx::query_as::<_, InvoiceBatch>(
            r#"
            INSERT INTO _atlas.ap_invoice_batches
                (organization_id, batch_number, batch_name, description,
                 currency_code, exchange_rate_type, exchange_rate,
                 gl_date, accounting_period, source,
                 control_total, control_count, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING *
            "#,
        )
        .bind(org_id)
        .bind(batch_number)
        .bind(batch_name)
        .bind(description)
        .bind(currency_code)
        .bind(exchange_rate_type)
        .bind(exchange_rate)
        .bind(gl_date)
        .bind(accounting_period)
        .bind(source)
        .bind(control_total)
        .bind(control_count)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(format!("Failed to create invoice batch: {}", e)))?;

        Ok(batch)
    }

    async fn get_batch(&self, org_id: Uuid, id: Uuid) -> AtlasResult<InvoiceBatch> {
        sqlx::query_as::<_, InvoiceBatch>(
            "SELECT * FROM _atlas.ap_invoice_batches WHERE id = $1 AND organization_id = $2"
        )
        .bind(id)
        .bind(org_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(format!("Failed to get invoice batch: {}", e)))?
        .ok_or_else(|| AtlasError::EntityNotFound(format!("Invoice batch {} not found", id)))
    }

    async fn get_batch_by_number(&self, org_id: Uuid, batch_number: &str) -> AtlasResult<InvoiceBatch> {
        sqlx::query_as::<_, InvoiceBatch>(
            "SELECT * FROM _atlas.ap_invoice_batches WHERE batch_number = $1 AND organization_id = $2"
        )
        .bind(batch_number)
        .bind(org_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(format!("Failed to get invoice batch by number: {}", e)))?
        .ok_or_else(|| AtlasError::EntityNotFound(format!("Invoice batch '{}' not found", batch_number)))
    }

    async fn list_batches(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<InvoiceBatch>> {
        let batches = if let Some(s) = status {
            sqlx::query_as::<_, InvoiceBatch>(
                "SELECT * FROM _atlas.ap_invoice_batches WHERE organization_id = $1 AND status = $2 ORDER BY created_at DESC"
            )
            .bind(org_id)
            .bind(s)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as::<_, InvoiceBatch>(
                "SELECT * FROM _atlas.ap_invoice_batches WHERE organization_id = $1 ORDER BY created_at DESC"
            )
            .bind(org_id)
            .fetch_all(&self.pool)
            .await
        }
        .map_err(|e| AtlasError::DatabaseError(format!("Failed to list invoice batches: {}", e)))?;

        Ok(batches)
    }

    async fn delete_batch(&self, org_id: Uuid, batch_number: &str) -> AtlasResult<()> {
        let result = sqlx::query(
            "DELETE FROM _atlas.ap_invoice_batches WHERE batch_number = $1 AND organization_id = $2 AND status = 'draft'"
        )
        .bind(batch_number)
        .bind(org_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(format!("Failed to delete invoice batch: {}", e)))?;

        if result.rows_affected() == 0 {
            return Err(AtlasError::ValidationFailed(
                "Batch not found or not in draft status".to_string(),
            ));
        }
        Ok(())
    }

    async fn update_status(
        &self,
        id: Uuid,
        status: &str,
        validation_status: Option<&str>,
        validation_errors: Option<serde_json::Value>,
    ) -> AtlasResult<InvoiceBatch> {
        let batch = if let Some(vs) = validation_status {
            sqlx::query_as::<_, InvoiceBatch>(
                r#"UPDATE _atlas.ap_invoice_batches
                   SET status = $2, validation_status = $3, validation_errors = COALESCE($4, validation_errors),
                       updated_at = now()
                   WHERE id = $1
                   RETURNING *"#,
            )
            .bind(id)
            .bind(status)
            .bind(vs)
            .bind(validation_errors)
            .fetch_one(&self.pool)
            .await
        } else {
            sqlx::query_as::<_, InvoiceBatch>(
                r#"UPDATE _atlas.ap_invoice_batches SET status = $2, updated_at = now()
                   WHERE id = $1 RETURNING *"#,
            )
            .bind(id)
            .bind(status)
            .fetch_one(&self.pool)
            .await
        }
        .map_err(|e| AtlasError::DatabaseError(format!("Failed to update batch status: {}", e)))?;

        Ok(batch)
    }

    async fn update_totals(
        &self,
        id: Uuid,
        invoice_count: i32,
        invoice_amount: f64,
        tax_amount: f64,
        total_amount: f64,
    ) -> AtlasResult<()> {
        sqlx::query(
            r#"UPDATE _atlas.ap_invoice_batches
               SET total_invoice_count = $2, total_invoice_amount = $3,
                   total_tax_amount = $4, total_amount = $5, updated_at = now()
               WHERE id = $1"#,
        )
        .bind(id)
        .bind(invoice_count)
        .bind(invoice_amount)
        .bind(tax_amount)
        .bind(total_amount)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(format!("Failed to update batch totals: {}", e)))?;
        Ok(())
    }

    async fn set_submitted(&self, id: Uuid, submitted_by: Uuid) -> AtlasResult<InvoiceBatch> {
        sqlx::query_as::<_, InvoiceBatch>(
            r#"UPDATE _atlas.ap_invoice_batches
               SET status = 'submitted', submitted_by = $2, submitted_at = now(),
                   updated_at = now()
               WHERE id = $1 RETURNING *"#,
        )
        .bind(id)
        .bind(submitted_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(format!("Failed to submit batch: {}", e)))
    }

    async fn set_approved(&self, id: Uuid, approved_by: Uuid) -> AtlasResult<InvoiceBatch> {
        sqlx::query_as::<_, InvoiceBatch>(
            r#"UPDATE _atlas.ap_invoice_batches
               SET status = 'approved', approved_by = $2, approved_at = now(),
                   updated_at = now()
               WHERE id = $1 RETURNING *"#,
        )
        .bind(id)
        .bind(approved_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(format!("Failed to approve batch: {}", e)))
    }

    async fn set_posted(&self, id: Uuid, posted_by: Uuid) -> AtlasResult<InvoiceBatch> {
        sqlx::query_as::<_, InvoiceBatch>(
            r#"UPDATE _atlas.ap_invoice_batches
               SET status = 'posted', posted_by = $2, posted_at = now(),
                   updated_at = now()
               WHERE id = $1 RETURNING *"#,
        )
        .bind(id)
        .bind(posted_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(format!("Failed to post batch: {}", e)))
    }

    async fn set_cancelled(&self, id: Uuid, cancelled_by: Uuid, reason: Option<&str>) -> AtlasResult<InvoiceBatch> {
        sqlx::query_as::<_, InvoiceBatch>(
            r#"UPDATE _atlas.ap_invoice_batches
               SET status = 'cancelled', cancelled_by = $2, cancelled_at = now(),
                   cancel_reason = $3, updated_at = now()
               WHERE id = $1 RETURNING *"#,
        )
        .bind(id)
        .bind(cancelled_by)
        .bind(reason)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(format!("Failed to cancel batch: {}", e)))
    }

    async fn add_activity(
        &self,
        batch_id: Uuid,
        activity_type: &str,
        description: Option<&str>,
        old_status: Option<&str>,
        new_status: Option<&str>,
        performed_by: Option<Uuid>,
        metadata: Option<serde_json::Value>,
    ) -> AtlasResult<InvoiceBatchActivity> {
        sqlx::query_as::<_, InvoiceBatchActivity>(
            r#"INSERT INTO _atlas.ap_invoice_batch_activities
                (batch_id, activity_type, description, old_status, new_status, performed_by, metadata)
               VALUES ($1, $2, $3, $4, $5, $6, $7)
               RETURNING *"#,
        )
        .bind(batch_id)
        .bind(activity_type)
        .bind(description)
        .bind(old_status)
        .bind(new_status)
        .bind(performed_by)
        .bind(metadata.unwrap_or(serde_json::json!({})))
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(format!("Failed to add batch activity: {}", e)))
    }

    async fn list_activities(&self, batch_id: Uuid) -> AtlasResult<Vec<InvoiceBatchActivity>> {
        sqlx::query_as::<_, InvoiceBatchActivity>(
            "SELECT * FROM _atlas.ap_invoice_batch_activities WHERE batch_id = $1 ORDER BY performed_at ASC"
        )
        .bind(batch_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(format!("Failed to list batch activities: {}", e)))
    }

    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<InvoiceBatchSummary> {
        let row = sqlx::query(
            r#"SELECT
                COUNT(*) as total_batches,
                COUNT(*) FILTER (WHERE status = 'draft') as draft_count,
                COUNT(*) FILTER (WHERE status = 'submitted') as submitted_count,
                COUNT(*) FILTER (WHERE status = 'approved') as approved_count,
                COUNT(*) FILTER (WHERE status = 'posted') as posted_count,
                COUNT(*) FILTER (WHERE status = 'cancelled') as cancelled_count,
                COALESCE(SUM(total_invoice_amount), 0) as total_invoice_amount,
                COALESCE(SUM(total_invoice_count), 0) as total_invoices,
                COALESCE(
                    json_object_agg(
                        source, cnt
                    ) FILTER (WHERE source IS NOT NULL),
                    '{}'
                ) as by_source
            FROM (
                SELECT status, source, total_invoice_amount, total_invoice_count,
                       COUNT(*) OVER (PARTITION BY source) as cnt
                FROM _atlas.ap_invoice_batches
                WHERE organization_id = $1
            ) sub"#,
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(format!("Failed to get dashboard: {}", e)))?;

        let by_source = if let Ok(val) = row.try_get::<serde_json::Value, _>("by_source") {
            val
        } else {
            serde_json::json!({})
        };

        Ok(InvoiceBatchSummary {
            total_batches: row.try_get("total_batches").unwrap_or(0),
            draft_count: row.try_get("draft_count").unwrap_or(0),
            submitted_count: row.try_get("submitted_count").unwrap_or(0),
            approved_count: row.try_get("approved_count").unwrap_or(0),
            posted_count: row.try_get("posted_count").unwrap_or(0),
            cancelled_count: row.try_get("cancelled_count").unwrap_or(0),
            total_invoice_amount: row.try_get("total_invoice_amount").unwrap_or(0.0),
            total_invoices: row.try_get("total_invoices").unwrap_or(0),
            by_source,
        })
    }

    async fn recalculate_totals(&self, batch_id: Uuid) -> AtlasResult<InvoiceBatch> {
        // Recalculate from the ap_invoices linked to this batch
        // For now, return the current batch (full implementation would join with ap_invoices)
        sqlx::query_as::<_, InvoiceBatch>(
            "SELECT * FROM _atlas.ap_invoice_batches WHERE id = $1"
        )
        .bind(batch_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(format!("Failed to recalculate totals: {}", e)))
    }
}
