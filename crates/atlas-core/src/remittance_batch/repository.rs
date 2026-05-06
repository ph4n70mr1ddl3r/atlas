//! Remittance Batch Repository
//!
//! PostgreSQL storage for remittance batches and batch receipts.

use atlas_shared::{
    RemittanceBatch, RemittanceBatchReceipt, RemittanceBatchSummary,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

/// Repository trait for remittance batch data storage
#[async_trait]
pub trait RemittanceBatchRepository: Send + Sync {
    async fn create_batch(
        &self,
        org_id: Uuid,
        batch_number: &str,
        batch_name: Option<&str>,
        bank_account_id: Option<Uuid>,
        bank_account_name: Option<&str>,
        bank_name: Option<&str>,
        remittance_method: &str,
        currency_code: &str,
        batch_date: chrono::NaiveDate,
        gl_date: Option<chrono::NaiveDate>,
        receipt_currency_code: Option<&str>,
        exchange_rate_type: Option<&str>,
        format_program: Option<&str>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<RemittanceBatch>;

    async fn get_batch(&self, id: Uuid) -> AtlasResult<Option<RemittanceBatch>>;
    async fn get_batch_by_number(&self, org_id: Uuid, batch_number: &str) -> AtlasResult<Option<RemittanceBatch>>;
    async fn list_batches(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        currency_code: Option<&str>,
        remittance_method: Option<&str>,
    ) -> AtlasResult<Vec<RemittanceBatch>>;
    async fn update_batch_status(&self, id: Uuid, status: &str) -> AtlasResult<RemittanceBatch>;
    async fn update_batch_notes(&self, id: Uuid, notes: Option<&str>) -> AtlasResult<()>;
    async fn update_reference_number(&self, id: Uuid, reference_number: &str) -> AtlasResult<()>;
    async fn update_batch_totals(&self, id: Uuid, total_amount: f64, receipt_count: i32) -> AtlasResult<()>;
    async fn update_advice_sent(&self, id: Uuid) -> AtlasResult<RemittanceBatch>;
    async fn get_next_batch_number(&self, org_id: Uuid) -> AtlasResult<i32>;

    async fn create_batch_receipt(
        &self,
        org_id: Uuid,
        batch_id: Uuid,
        receipt_id: Uuid,
        receipt_number: Option<&str>,
        customer_id: Option<Uuid>,
        customer_number: Option<&str>,
        customer_name: Option<&str>,
        receipt_date: Option<chrono::NaiveDate>,
        receipt_amount: &str,
        applied_amount: &str,
        receipt_method: Option<&str>,
        currency_code: &str,
        exchange_rate: Option<&str>,
        display_order: i32,
        metadata: serde_json::Value,
    ) -> AtlasResult<RemittanceBatchReceipt>;

    async fn list_batch_receipts(&self, batch_id: Uuid) -> AtlasResult<Vec<RemittanceBatchReceipt>>;
    async fn delete_batch_receipt(&self, batch_id: Uuid, receipt_id: Uuid) -> AtlasResult<()>;
    async fn get_batch_receipt_by_receipt_id(&self, batch_id: Uuid, receipt_id: Uuid) -> AtlasResult<Option<RemittanceBatchReceipt>>;
    async fn get_next_receipt_order(&self, batch_id: Uuid) -> AtlasResult<i32>;
    async fn get_batch_summary(&self, org_id: Uuid) -> AtlasResult<RemittanceBatchSummary>;
}

// Helper functions
fn get_numeric_text(row: &sqlx::postgres::PgRow, col: &str) -> String {
    row.try_get::<f64, _>(col)
        .map(|v| format!("{:.2}", v))
        .unwrap_or_else(|_| "0.00".to_string())
}

fn row_to_batch(row: &sqlx::postgres::PgRow) -> RemittanceBatch {
    RemittanceBatch {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        batch_number: row.get("batch_number"),
        batch_name: row.get("batch_name"),
        bank_account_id: row.get("bank_account_id"),
        bank_account_name: row.get("bank_account_name"),
        bank_name: row.get("bank_name"),
        remittance_method: row.get("remittance_method"),
        currency_code: row.get("currency_code"),
        batch_date: row.get("batch_date"),
        gl_date: row.get("gl_date"),
        receipt_currency_code: row.get("receipt_currency_code"),
        exchange_rate_type: row.get("exchange_rate_type"),
        status: row.get("status"),
        total_amount: get_numeric_text(row, "total_amount"),
        receipt_count: row.try_get("receipt_count").unwrap_or(0),
        format_program: row.get("format_program"),
        format_date: row.get("format_date"),
        transmission_date: row.get("transmission_date"),
        confirmation_date: row.get("confirmation_date"),
        settlement_date: row.get("settlement_date"),
        reversal_date: row.get("reversal_date"),
        reference_number: row.get("reference_number"),
        remittance_advice_sent: row.try_get("remittance_advice_sent").unwrap_or(false).into(),
        remittance_advice_date: row.get("remittance_advice_date"),
        notes: row.get("notes"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_receipt(row: &sqlx::postgres::PgRow) -> RemittanceBatchReceipt {
    RemittanceBatchReceipt {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        batch_id: row.get("batch_id"),
        receipt_id: row.get("receipt_id"),
        receipt_number: row.get("receipt_number"),
        customer_id: row.get("customer_id"),
        customer_number: row.get("customer_number"),
        customer_name: row.get("customer_name"),
        receipt_date: row.get("receipt_date"),
        receipt_amount: get_numeric_text(row, "receipt_amount"),
        applied_amount: get_numeric_text(row, "applied_amount"),
        receipt_method: row.get("receipt_method"),
        currency_code: row.get("currency_code"),
        exchange_rate: row.try_get::<f64, _>("exchange_rate").ok().map(|v| format!("{:.6}", v)),
        status: row.get("status"),
        display_order: row.try_get("display_order").unwrap_or(0),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

/// PostgreSQL implementation
pub struct PostgresRemittanceBatchRepository {
    pool: PgPool,
}

impl PostgresRemittanceBatchRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RemittanceBatchRepository for PostgresRemittanceBatchRepository {
    async fn create_batch(
        &self,
        org_id: Uuid,
        batch_number: &str,
        batch_name: Option<&str>,
        bank_account_id: Option<Uuid>,
        bank_account_name: Option<&str>,
        bank_name: Option<&str>,
        remittance_method: &str,
        currency_code: &str,
        batch_date: chrono::NaiveDate,
        gl_date: Option<chrono::NaiveDate>,
        receipt_currency_code: Option<&str>,
        exchange_rate_type: Option<&str>,
        format_program: Option<&str>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<RemittanceBatch> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.remittance_batches
                (organization_id, batch_number, batch_name,
                 bank_account_id, bank_account_name, bank_name,
                 remittance_method, currency_code, batch_date, gl_date,
                 receipt_currency_code, exchange_rate_type,
                 format_program, notes, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(batch_number).bind(batch_name)
        .bind(bank_account_id).bind(bank_account_name).bind(bank_name)
        .bind(remittance_method).bind(currency_code).bind(batch_date).bind(gl_date)
        .bind(receipt_currency_code).bind(exchange_rate_type)
        .bind(format_program).bind(notes).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_batch(&row))
    }

    async fn get_batch(&self, id: Uuid) -> AtlasResult<Option<RemittanceBatch>> {
        let row = sqlx::query("SELECT * FROM _atlas.remittance_batches WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_batch(&r)))
    }

    async fn get_batch_by_number(&self, org_id: Uuid, batch_number: &str) -> AtlasResult<Option<RemittanceBatch>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.remittance_batches WHERE organization_id = $1 AND batch_number = $2"
        )
        .bind(org_id).bind(batch_number)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_batch(&r)))
    }

    async fn list_batches(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        currency_code: Option<&str>,
        remittance_method: Option<&str>,
    ) -> AtlasResult<Vec<RemittanceBatch>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM _atlas.remittance_batches
            WHERE organization_id = $1
              AND ($2::text IS NULL OR status = $2)
              AND ($3::text IS NULL OR currency_code = $3)
              AND ($4::text IS NULL OR remittance_method = $4)
            ORDER BY batch_date DESC, batch_number DESC
            "#,
        )
        .bind(org_id).bind(status).bind(currency_code).bind(remittance_method)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_batch).collect())
    }

    async fn update_batch_status(&self, id: Uuid, status: &str) -> AtlasResult<RemittanceBatch> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.remittance_batches
            SET status = $2,
                format_date = CASE WHEN $2 = 'formatted' AND format_date IS NULL THEN now() ELSE format_date END,
                transmission_date = CASE WHEN $2 = 'transmitted' THEN now() ELSE transmission_date END,
                confirmation_date = CASE WHEN $2 = 'confirmed' AND confirmation_date IS NULL THEN now() ELSE confirmation_date END,
                settlement_date = CASE WHEN $2 = 'settled' THEN now() ELSE settlement_date END,
                reversal_date = CASE WHEN $2 = 'reversed' THEN now() ELSE reversal_date END,
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(status)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_batch(&row))
    }

    async fn update_batch_notes(&self, id: Uuid, notes: Option<&str>) -> AtlasResult<()> {
        sqlx::query("UPDATE _atlas.remittance_batches SET notes = $2, updated_at = now() WHERE id = $1")
            .bind(id).bind(notes)
            .execute(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn update_reference_number(&self, id: Uuid, reference_number: &str) -> AtlasResult<()> {
        sqlx::query("UPDATE _atlas.remittance_batches SET reference_number = $2, updated_at = now() WHERE id = $1")
            .bind(id).bind(reference_number)
            .execute(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn update_batch_totals(&self, id: Uuid, total_amount: f64, receipt_count: i32) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.remittance_batches SET total_amount = $2, receipt_count = $3, updated_at = now() WHERE id = $1"
        )
        .bind(id).bind(total_amount).bind(receipt_count)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn update_advice_sent(&self, id: Uuid) -> AtlasResult<RemittanceBatch> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.remittance_batches
            SET remittance_advice_sent = true,
                remittance_advice_date = now(),
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_batch(&row))
    }

    async fn get_next_batch_number(&self, org_id: Uuid) -> AtlasResult<i32> {
        let row = sqlx::query(
            "SELECT nextval('_atlas.remittance_batch_num_seq') as next_num"
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        let next_num: i64 = row.try_get("next_num").unwrap_or(1);
        Ok(next_num as i32)
    }

    // ========================================================================
    // Batch Receipts
    // ========================================================================

    async fn create_batch_receipt(
        &self,
        org_id: Uuid,
        batch_id: Uuid,
        receipt_id: Uuid,
        receipt_number: Option<&str>,
        customer_id: Option<Uuid>,
        customer_number: Option<&str>,
        customer_name: Option<&str>,
        receipt_date: Option<chrono::NaiveDate>,
        receipt_amount: &str,
        applied_amount: &str,
        receipt_method: Option<&str>,
        currency_code: &str,
        exchange_rate: Option<&str>,
        display_order: i32,
        metadata: serde_json::Value,
    ) -> AtlasResult<RemittanceBatchReceipt> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.remittance_batch_receipts
                (organization_id, batch_id, receipt_id, receipt_number,
                 customer_id, customer_number, customer_name,
                 receipt_date, receipt_amount, applied_amount,
                 receipt_method, currency_code, exchange_rate,
                 display_order, metadata)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9::numeric,$10::numeric,$11,$12,$13::numeric,$14,$15)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(batch_id).bind(receipt_id).bind(receipt_number)
        .bind(customer_id).bind(customer_number).bind(customer_name)
        .bind(receipt_date).bind(receipt_amount).bind(applied_amount)
        .bind(receipt_method).bind(currency_code).bind(exchange_rate)
        .bind(display_order).bind(metadata)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_receipt(&row))
    }

    async fn list_batch_receipts(&self, batch_id: Uuid) -> AtlasResult<Vec<RemittanceBatchReceipt>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.remittance_batch_receipts WHERE batch_id = $1 ORDER BY display_order, created_at"
        )
        .bind(batch_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_receipt).collect())
    }

    async fn delete_batch_receipt(&self, batch_id: Uuid, receipt_id: Uuid) -> AtlasResult<()> {
        sqlx::query(
            "DELETE FROM _atlas.remittance_batch_receipts WHERE batch_id = $1 AND receipt_id = $2"
        )
        .bind(batch_id).bind(receipt_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn get_batch_receipt_by_receipt_id(&self, batch_id: Uuid, receipt_id: Uuid) -> AtlasResult<Option<RemittanceBatchReceipt>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.remittance_batch_receipts WHERE batch_id = $1 AND receipt_id = $2"
        )
        .bind(batch_id).bind(receipt_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_receipt(&r)))
    }

    async fn get_next_receipt_order(&self, batch_id: Uuid) -> AtlasResult<i32> {
        let row = sqlx::query(
            "SELECT COALESCE(MAX(display_order), 0) + 1 as next_order FROM _atlas.remittance_batch_receipts WHERE batch_id = $1"
        )
        .bind(batch_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        let next: i32 = row.try_get("next_order").unwrap_or(1);
        Ok(next)
    }

    async fn get_batch_summary(&self, org_id: Uuid) -> AtlasResult<RemittanceBatchSummary> {
        let row = sqlx::query(
            r#"SELECT
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE status = 'draft') as draft_cnt,
                COUNT(*) FILTER (WHERE status = 'approved') as approved_cnt,
                COUNT(*) FILTER (WHERE status = 'settled') as settled_cnt,
                COALESCE(SUM(total_amount), 0) as total_amt,
                COALESCE(SUM(receipt_count), 0) as total_receipts
            FROM _atlas.remittance_batches WHERE organization_id = $1"#,
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let total: i64 = row.try_get("total").unwrap_or(0);
        let draft: i64 = row.try_get("draft_cnt").unwrap_or(0);
        let approved: i64 = row.try_get("approved_cnt").unwrap_or(0);
        let settled: i64 = row.try_get("settled_cnt").unwrap_or(0);
        let total_amt: f64 = row.try_get("total_amt").unwrap_or(0.0);
        let total_receipts: i64 = row.try_get("total_receipts").unwrap_or(0);

        // By status
        let status_rows = sqlx::query(
            "SELECT status, COUNT(*) as count FROM _atlas.remittance_batches WHERE organization_id = $1 GROUP BY status"
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let mut by_status = serde_json::Map::new();
        for r in &status_rows {
            let s: String = r.try_get("status").unwrap_or_default();
            let count: i64 = r.try_get("count").unwrap_or(0);
            by_status.insert(s, serde_json::json!(count));
        }

        // By currency
        let curr_rows = sqlx::query(
            "SELECT currency_code, COUNT(*) as count FROM _atlas.remittance_batches WHERE organization_id = $1 GROUP BY currency_code"
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let mut by_currency = serde_json::Map::new();
        for r in &curr_rows {
            let cc: String = r.try_get("currency_code").unwrap_or_default();
            let count: i64 = r.try_get("count").unwrap_or(0);
            by_currency.insert(cc, serde_json::json!(count));
        }

        Ok(RemittanceBatchSummary {
            total_batches: total as i32,
            draft_count: draft as i32,
            approved_count: approved as i32,
            settled_count: settled as i32,
            total_amount: format!("{:.2}", total_amt),
            total_receipts: total_receipts as i32,
            by_status: serde_json::Value::Object(by_status),
            by_currency: serde_json::Value::Object(by_currency),
        })
    }
}
