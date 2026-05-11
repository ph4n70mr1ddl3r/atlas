//! Payment Settlement Repository
//!
//! `PostgreSQL` storage for settlement batches, settlement lines, and activity audit trail.

use atlas_shared::{AtlasError, AtlasResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use uuid::Uuid;

// ============================================================================
// Data Types
// ============================================================================

/// Settlement batch header record
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SettlementBatch {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub batch_number: String,
    pub batch_name: String,
    pub description: Option<String>,
    pub bank_account_id: Option<Uuid>,
    pub bank_account_name: Option<String>,
    pub bank_reference: Option<String>,
    pub currency_code: String,
    pub exchange_rate_type: Option<String>,
    pub exchange_rate: Option<f64>,
    pub settlement_date: chrono::NaiveDate,
    pub gl_date: chrono::NaiveDate,
    pub settlement_method: String,
    pub total_invoices: i32,
    pub total_invoice_amount: f64,
    pub total_discount_taken: f64,
    pub total_settled_amount: f64,
    pub total_charges: f64,
    pub total_net_payment: f64,
    pub status: String,
    pub settlement_type: String,
    pub submitted_by: Option<Uuid>,
    pub submitted_at: Option<chrono::DateTime<chrono::Utc>>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<chrono::DateTime<chrono::Utc>>,
    pub settled_by: Option<Uuid>,
    pub settled_at: Option<chrono::DateTime<chrono::Utc>>,
    pub cancelled_by: Option<Uuid>,
    pub cancelled_at: Option<chrono::DateTime<chrono::Utc>>,
    pub cancel_reason: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Settlement line record
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SettlementLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub batch_id: Uuid,
    pub line_number: i32,
    pub invoice_id: Uuid,
    pub invoice_number: Option<String>,
    pub invoice_date: Option<chrono::NaiveDate>,
    pub invoice_amount: f64,
    pub supplier_id: Option<Uuid>,
    pub supplier_number: Option<String>,
    pub supplier_name: Option<String>,
    pub supplier_site: Option<String>,
    pub original_amount: f64,
    pub amount_due: f64,
    pub amount_paid: f64,
    pub discount_available: f64,
    pub discount_taken: f64,
    pub discount_date: Option<chrono::NaiveDate>,
    pub discount_reason: Option<String>,
    pub bank_charges: f64,
    pub adjustment_amount: f64,
    pub adjustment_reason: Option<String>,
    pub net_settlement: f64,
    pub remaining_balance: f64,
    pub settlement_type: String,
    pub liability_account: Option<String>,
    pub discount_account: Option<String>,
    pub charges_account: Option<String>,
    pub status: String,
    pub error_message: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Settlement activity log record
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SettlementActivity {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub batch_id: Uuid,
    pub line_id: Option<Uuid>,
    pub activity_type: String,
    pub description: Option<String>,
    pub old_status: Option<String>,
    pub new_status: Option<String>,
    pub performed_by: Option<Uuid>,
    pub performed_by_name: Option<String>,
    pub details: serde_json::Value,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Settlement dashboard summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettlementSummary {
    pub total_batches: i64,
    pub draft_count: i64,
    pub submitted_count: i64,
    pub approved_count: i64,
    pub settled_count: i64,
    pub cancelled_count: i64,
    pub total_settled_amount: f64,
    pub total_discount_taken: f64,
    pub total_invoices_settled: i64,
    pub by_settlement_method: serde_json::Value,
    pub by_settlement_type: serde_json::Value,
}

// ============================================================================
// Create Parameters
// ============================================================================

pub struct SettlementBatchCreateParams {
    pub org_id: Uuid,
    pub batch_name: String,
    pub description: Option<String>,
    pub bank_account_id: Option<Uuid>,
    pub bank_account_name: Option<String>,
    pub currency_code: String,
    pub exchange_rate_type: Option<String>,
    pub exchange_rate: Option<f64>,
    pub settlement_date: chrono::NaiveDate,
    pub gl_date: chrono::NaiveDate,
    pub settlement_method: String,
    pub settlement_type: String,
    pub created_by: Option<Uuid>,
}

pub struct SettlementLineCreateParams {
    pub org_id: Uuid,
    pub batch_id: Uuid,
    pub invoice_id: Uuid,
    pub invoice_number: Option<String>,
    pub invoice_date: Option<chrono::NaiveDate>,
    pub invoice_amount: f64,
    pub supplier_id: Option<Uuid>,
    pub supplier_number: Option<String>,
    pub supplier_name: Option<String>,
    pub supplier_site: Option<String>,
    pub original_amount: f64,
    pub amount_due: f64,
    pub amount_paid: f64,
    pub discount_available: f64,
    pub discount_taken: f64,
    pub discount_date: Option<chrono::NaiveDate>,
    pub bank_charges: f64,
    pub adjustment_amount: f64,
    pub adjustment_reason: Option<String>,
    pub settlement_type: String,
    pub liability_account: Option<String>,
    pub discount_account: Option<String>,
    pub charges_account: Option<String>,
}

// ============================================================================
// Repository Trait
// ============================================================================

#[async_trait]
pub trait PaymentSettlementRepository: Send + Sync {
    // Batches
    async fn create_batch(&self, params: &SettlementBatchCreateParams) -> AtlasResult<SettlementBatch>;
    async fn get_batch(&self, id: Uuid) -> AtlasResult<Option<SettlementBatch>>;
    async fn get_batch_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<SettlementBatch>>;
    async fn list_batches(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<SettlementBatch>>;
    async fn update_batch_status(&self, id: Uuid, status: &str) -> AtlasResult<SettlementBatch>;
    async fn update_batch_submission(&self, id: Uuid, submitted_by: Option<Uuid>) -> AtlasResult<SettlementBatch>;
    async fn update_batch_approval(&self, id: Uuid, approved_by: Option<Uuid>) -> AtlasResult<SettlementBatch>;
    async fn update_batch_settlement(&self, id: Uuid, settled_by: Option<Uuid>) -> AtlasResult<SettlementBatch>;
    async fn update_batch_cancellation(&self, id: Uuid, cancelled_by: Option<Uuid>, reason: Option<&str>) -> AtlasResult<SettlementBatch>;
    async fn update_batch_totals(&self, id: Uuid, invoices: i32, invoice_amt: f64, discount: f64, settled: f64, charges: f64, net: f64) -> AtlasResult<()>;
    async fn delete_batch(&self, org_id: Uuid, number: &str) -> AtlasResult<()>;
    async fn get_next_batch_number(&self, org_id: Uuid) -> AtlasResult<i32>;

    // Lines
    async fn create_line(&self, params: &SettlementLineCreateParams) -> AtlasResult<SettlementLine>;
    async fn get_line(&self, id: Uuid) -> AtlasResult<Option<SettlementLine>>;
    async fn list_lines(&self, batch_id: Uuid) -> AtlasResult<Vec<SettlementLine>>;
    async fn update_line_status(&self, id: Uuid, status: &str, error_message: Option<&str>) -> AtlasResult<SettlementLine>;
    async fn delete_line(&self, batch_id: Uuid, line_id: Uuid) -> AtlasResult<()>;
    async fn get_next_line_number(&self, batch_id: Uuid) -> AtlasResult<i32>;

    // Activities
    async fn create_activity(
        &self, org_id: Uuid, batch_id: Uuid, line_id: Option<Uuid>,
        activity_type: &str, description: Option<&str>,
        old_status: Option<&str>, new_status: Option<&str>,
        performed_by: Option<Uuid>, performed_by_name: Option<&str>,
        details: Option<serde_json::Value>,
    ) -> AtlasResult<SettlementActivity>;
    async fn list_activities(&self, batch_id: Uuid) -> AtlasResult<Vec<SettlementActivity>>;

    // Dashboard
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<SettlementSummary>;
}

// ============================================================================
// PostgreSQL Implementation
// ============================================================================

pub struct PostgresPaymentSettlementRepository {
    pool: PgPool,
}

impl PostgresPaymentSettlementRepository {
    #[must_use] 
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PaymentSettlementRepository for PostgresPaymentSettlementRepository {
    async fn create_batch(&self, params: &SettlementBatchCreateParams) -> AtlasResult<SettlementBatch> {
        let seq = self.get_next_batch_number(params.org_id).await.unwrap_or(1);
        let batch_number = format!("STL-{seq:06}");

        let row = sqlx::query_as::<_, SettlementBatch>(
            r"INSERT INTO _atlas.settlement_batches
               (organization_id, batch_number, batch_name, description,
                bank_account_id, bank_account_name,
                currency_code, exchange_rate_type, exchange_rate,
                settlement_date, gl_date, settlement_method, settlement_type,
                status, created_by)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,'draft',$14)
               RETURNING *",
        )
        .bind(params.org_id)
        .bind(&batch_number)
        .bind(&params.batch_name)
        .bind(&params.description)
        .bind(params.bank_account_id)
        .bind(&params.bank_account_name)
        .bind(&params.currency_code)
        .bind(&params.exchange_rate_type)
        .bind(params.exchange_rate)
        .bind(params.settlement_date)
        .bind(params.gl_date)
        .bind(&params.settlement_method)
        .bind(&params.settlement_type)
        .bind(params.created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row)
    }

    async fn get_batch(&self, id: Uuid) -> AtlasResult<Option<SettlementBatch>> {
        let row = sqlx::query_as::<_, SettlementBatch>(
            "SELECT * FROM _atlas.settlement_batches WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row)
    }

    async fn get_batch_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<SettlementBatch>> {
        let row = sqlx::query_as::<_, SettlementBatch>(
            "SELECT * FROM _atlas.settlement_batches WHERE organization_id = $1 AND batch_number = $2",
        )
        .bind(org_id)
        .bind(number)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row)
    }

    async fn list_batches(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<SettlementBatch>> {
        let rows = sqlx::query_as::<_, SettlementBatch>(
            r"SELECT * FROM _atlas.settlement_batches
               WHERE organization_id = $1
               AND ($2::text IS NULL OR status = $2)
               ORDER BY settlement_date DESC, batch_number",
        )
        .bind(org_id)
        .bind(status)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows)
    }

    async fn update_batch_status(&self, id: Uuid, status: &str) -> AtlasResult<SettlementBatch> {
        let row = sqlx::query_as::<_, SettlementBatch>(
            r"UPDATE _atlas.settlement_batches SET status = $2, updated_at = now()
               WHERE id = $1 RETURNING *",
        )
        .bind(id)
        .bind(status)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row)
    }

    async fn update_batch_submission(&self, id: Uuid, submitted_by: Option<Uuid>) -> AtlasResult<SettlementBatch> {
        let row = sqlx::query_as::<_, SettlementBatch>(
            r"UPDATE _atlas.settlement_batches
               SET submitted_by = $2, submitted_at = now(), updated_at = now()
               WHERE id = $1 RETURNING *",
        )
        .bind(id)
        .bind(submitted_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row)
    }

    async fn update_batch_approval(&self, id: Uuid, approved_by: Option<Uuid>) -> AtlasResult<SettlementBatch> {
        let row = sqlx::query_as::<_, SettlementBatch>(
            r"UPDATE _atlas.settlement_batches
               SET approved_by = $2, approved_at = now(), updated_at = now()
               WHERE id = $1 RETURNING *",
        )
        .bind(id)
        .bind(approved_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row)
    }

    async fn update_batch_settlement(&self, id: Uuid, settled_by: Option<Uuid>) -> AtlasResult<SettlementBatch> {
        let row = sqlx::query_as::<_, SettlementBatch>(
            r"UPDATE _atlas.settlement_batches
               SET settled_by = $2, settled_at = now(), updated_at = now()
               WHERE id = $1 RETURNING *",
        )
        .bind(id)
        .bind(settled_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row)
    }

    async fn update_batch_cancellation(&self, id: Uuid, cancelled_by: Option<Uuid>, reason: Option<&str>) -> AtlasResult<SettlementBatch> {
        let row = sqlx::query_as::<_, SettlementBatch>(
            r"UPDATE _atlas.settlement_batches
               SET cancelled_by = $2, cancelled_at = now(), cancel_reason = $3, updated_at = now()
               WHERE id = $1 RETURNING *",
        )
        .bind(id)
        .bind(cancelled_by)
        .bind(reason)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row)
    }

    async fn update_batch_totals(&self, id: Uuid, invoices: i32, invoice_amt: f64, discount: f64, settled: f64, charges: f64, net: f64) -> AtlasResult<()> {
        sqlx::query(
            r"UPDATE _atlas.settlement_batches
               SET total_invoices = $2, total_invoice_amount = $3,
                   total_discount_taken = $4, total_settled_amount = $5,
                   total_charges = $6, total_net_payment = $7, updated_at = now()
               WHERE id = $1",
        )
        .bind(id)
        .bind(invoices)
        .bind(invoice_amt)
        .bind(discount)
        .bind(settled)
        .bind(charges)
        .bind(net)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn delete_batch(&self, org_id: Uuid, number: &str) -> AtlasResult<()> {
        let result = sqlx::query(
            "DELETE FROM _atlas.settlement_batches WHERE organization_id = $1 AND batch_number = $2",
        )
        .bind(org_id)
        .bind(number)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound("Settlement batch not found".to_string()));
        }
        Ok(())
    }

    async fn get_next_batch_number(&self, org_id: Uuid) -> AtlasResult<i32> {
        let row = sqlx::query(
            "SELECT COALESCE(MAX(CAST(SUBSTRING(batch_number FROM 'STL-(\\d+)') AS INTEGER)), 0) + 1 FROM _atlas.settlement_batches WHERE organization_id = $1",
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        let seq: i32 = row.try_get(0).unwrap_or(1);
        Ok(seq)
    }

    // Lines
    async fn create_line(&self, params: &SettlementLineCreateParams) -> AtlasResult<SettlementLine> {
        let line_number = self.get_next_line_number(params.batch_id).await.unwrap_or(1);
        let remaining_balance = params.amount_due - params.amount_paid;
        let net_settlement = params.amount_paid - params.discount_taken - params.bank_charges + params.adjustment_amount;

        let row = sqlx::query_as::<_, SettlementLine>(
            r"INSERT INTO _atlas.settlement_lines
               (organization_id, batch_id, line_number,
                invoice_id, invoice_number, invoice_date, invoice_amount,
                supplier_id, supplier_number, supplier_name, supplier_site,
                original_amount, amount_due, amount_paid,
                discount_available, discount_taken, discount_date,
                bank_charges, adjustment_amount, adjustment_reason,
                net_settlement, remaining_balance,
                settlement_type, liability_account, discount_account, charges_account,
                status)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22,$23,$24,$25,$26,'pending')
               RETURNING *",
        )
        .bind(params.org_id)
        .bind(params.batch_id)
        .bind(line_number)
        .bind(params.invoice_id)
        .bind(&params.invoice_number)
        .bind(params.invoice_date)
        .bind(params.invoice_amount)
        .bind(params.supplier_id)
        .bind(&params.supplier_number)
        .bind(&params.supplier_name)
        .bind(&params.supplier_site)
        .bind(params.original_amount)
        .bind(params.amount_due)
        .bind(params.amount_paid)
        .bind(params.discount_available)
        .bind(params.discount_taken)
        .bind(params.discount_date)
        .bind(params.bank_charges)
        .bind(params.adjustment_amount)
        .bind(&params.adjustment_reason)
        .bind(net_settlement)
        .bind(remaining_balance)
        .bind(&params.settlement_type)
        .bind(&params.liability_account)
        .bind(&params.discount_account)
        .bind(&params.charges_account)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row)
    }

    async fn get_line(&self, id: Uuid) -> AtlasResult<Option<SettlementLine>> {
        let row = sqlx::query_as::<_, SettlementLine>(
            "SELECT * FROM _atlas.settlement_lines WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row)
    }

    async fn list_lines(&self, batch_id: Uuid) -> AtlasResult<Vec<SettlementLine>> {
        let rows = sqlx::query_as::<_, SettlementLine>(
            "SELECT * FROM _atlas.settlement_lines WHERE batch_id = $1 ORDER BY line_number",
        )
        .bind(batch_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows)
    }

    async fn update_line_status(&self, id: Uuid, status: &str, error_message: Option<&str>) -> AtlasResult<SettlementLine> {
        let row = sqlx::query_as::<_, SettlementLine>(
            r"UPDATE _atlas.settlement_lines
               SET status = $2, error_message = COALESCE($3, error_message), updated_at = now()
               WHERE id = $1 RETURNING *",
        )
        .bind(id)
        .bind(status)
        .bind(error_message)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row)
    }

    async fn delete_line(&self, batch_id: Uuid, line_id: Uuid) -> AtlasResult<()> {
        let result = sqlx::query(
            "DELETE FROM _atlas.settlement_lines WHERE batch_id = $1 AND id = $2",
        )
        .bind(batch_id)
        .bind(line_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound("Settlement line not found".to_string()));
        }
        Ok(())
    }

    async fn get_next_line_number(&self, batch_id: Uuid) -> AtlasResult<i32> {
        let row = sqlx::query(
            "SELECT COALESCE(MAX(line_number), 0) + 1 FROM _atlas.settlement_lines WHERE batch_id = $1",
        )
        .bind(batch_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        let seq: i32 = row.try_get(0).unwrap_or(1);
        Ok(seq)
    }

    // Activities
    async fn create_activity(
        &self, org_id: Uuid, batch_id: Uuid, line_id: Option<Uuid>,
        activity_type: &str, description: Option<&str>,
        old_status: Option<&str>, new_status: Option<&str>,
        performed_by: Option<Uuid>, performed_by_name: Option<&str>,
        details: Option<serde_json::Value>,
    ) -> AtlasResult<SettlementActivity> {
        let row = sqlx::query_as::<_, SettlementActivity>(
            r"INSERT INTO _atlas.settlement_activities
               (organization_id, batch_id, line_id, activity_type, description,
                old_status, new_status, performed_by, performed_by_name, details)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
               RETURNING *",
        )
        .bind(org_id)
        .bind(batch_id)
        .bind(line_id)
        .bind(activity_type)
        .bind(description)
        .bind(old_status)
        .bind(new_status)
        .bind(performed_by)
        .bind(performed_by_name)
        .bind(details.unwrap_or(serde_json::json!({})))
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row)
    }

    async fn list_activities(&self, batch_id: Uuid) -> AtlasResult<Vec<SettlementActivity>> {
        let rows = sqlx::query_as::<_, SettlementActivity>(
            "SELECT * FROM _atlas.settlement_activities WHERE batch_id = $1 ORDER BY created_at",
        )
        .bind(batch_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows)
    }

    // Dashboard
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<SettlementSummary> {
        let row = sqlx::query(
            r"SELECT
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE status = 'draft') as draft_cnt,
                COUNT(*) FILTER (WHERE status = 'submitted') as submitted_cnt,
                COUNT(*) FILTER (WHERE status = 'approved') as approved_cnt,
                COUNT(*) FILTER (WHERE status = 'settled') as settled_cnt,
                COUNT(*) FILTER (WHERE status = 'cancelled') as cancelled_cnt,
                COALESCE(SUM(total_settled_amount), 0) as total_settled,
                COALESCE(SUM(total_discount_taken), 0) as total_discount,
                COALESCE(SUM(total_invoices), 0) as total_inv
               FROM _atlas.settlement_batches WHERE organization_id = $1",
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let by_method = self.get_grouped_counts(org_id, "settlement_method").await?;
        let by_type = self.get_grouped_counts(org_id, "settlement_type").await?;

        Ok(SettlementSummary {
            total_batches: row.try_get("total").unwrap_or(0),
            draft_count: row.try_get("draft_cnt").unwrap_or(0),
            submitted_count: row.try_get("submitted_cnt").unwrap_or(0),
            approved_count: row.try_get("approved_cnt").unwrap_or(0),
            settled_count: row.try_get("settled_cnt").unwrap_or(0),
            cancelled_count: row.try_get("cancelled_cnt").unwrap_or(0),
            total_settled_amount: row.try_get("total_settled").unwrap_or(0.0),
            total_discount_taken: row.try_get("total_discount").unwrap_or(0.0),
            total_invoices_settled: row.try_get("total_inv").unwrap_or(0),
            by_settlement_method: by_method,
            by_settlement_type: by_type,
        })
    }
}

impl PostgresPaymentSettlementRepository {
    async fn get_grouped_counts(&self, org_id: Uuid, column: &str) -> AtlasResult<serde_json::Value> {
        let query = format!(
            "SELECT {column} as key, COUNT(*) as cnt, COALESCE(SUM(total_net_payment), 0) as total FROM _atlas.settlement_batches WHERE organization_id = $1 GROUP BY {column}"
        );
        let rows = sqlx::query(&query)
            .bind(org_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let mut result = serde_json::Map::new();
        for row in rows {
            let key: String = row.try_get("key").unwrap_or_default();
            let cnt: i64 = row.try_get("cnt").unwrap_or(0);
            let total: f64 = row.try_get("total").unwrap_or(0.0);
            result.insert(key, serde_json::json!({"count": cnt, "amount": total}));
        }
        Ok(serde_json::Value::Object(result))
    }
}
