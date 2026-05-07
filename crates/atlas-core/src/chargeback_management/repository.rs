//! Chargeback Management Repository
//!
//! PostgreSQL storage for chargebacks, chargeback lines, and activity audit trail.

use atlas_shared::{AtlasError, AtlasResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use uuid::Uuid;

// ============================================================================
// Data Types
// ============================================================================

/// Chargeback header record
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Chargeback {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub chargeback_number: String,
    pub customer_id: Option<Uuid>,
    pub customer_number: Option<String>,
    pub customer_name: Option<String>,
    pub receipt_id: Option<Uuid>,
    pub receipt_number: Option<String>,
    pub invoice_id: Option<Uuid>,
    pub invoice_number: Option<String>,
    pub chargeback_date: chrono::NaiveDate,
    pub gl_date: chrono::NaiveDate,
    pub currency_code: String,
    pub exchange_rate_type: Option<String>,
    pub exchange_rate: Option<f64>,
    pub amount: f64,
    pub tax_amount: f64,
    pub total_amount: f64,
    pub open_amount: f64,
    pub reason_code: String,
    pub reason_description: Option<String>,
    pub category: Option<String>,
    pub status: String,
    pub priority: String,
    pub assigned_to: Option<String>,
    pub assigned_team: Option<String>,
    pub due_date: Option<chrono::NaiveDate>,
    pub resolution_date: Option<chrono::NaiveDate>,
    pub resolution_notes: Option<String>,
    pub resolved_by: Option<Uuid>,
    pub reference: Option<String>,
    pub customer_reference: Option<String>,
    pub sales_rep: Option<String>,
    pub notes: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Chargeback line record
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ChargebackLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub chargeback_id: Uuid,
    pub line_number: i32,
    pub line_type: String,
    pub description: Option<String>,
    pub quantity: i32,
    pub unit_price: f64,
    pub amount: f64,
    pub tax_amount: f64,
    pub total_amount: f64,
    pub reason_code: Option<String>,
    pub reason_description: Option<String>,
    pub item_number: Option<String>,
    pub item_description: Option<String>,
    pub gl_account_code: Option<String>,
    pub gl_account_name: Option<String>,
    pub reference: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Chargeback activity log record
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ChargebackActivity {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub chargeback_id: Uuid,
    pub activity_type: String,
    pub description: Option<String>,
    pub old_status: Option<String>,
    pub new_status: Option<String>,
    pub performed_by: Option<Uuid>,
    pub performed_by_name: Option<String>,
    pub notes: Option<String>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Chargeback dashboard summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChargebackSummary {
    pub total_chargebacks: i64,
    pub open_count: i64,
    pub under_review_count: i64,
    pub accepted_count: i64,
    pub rejected_count: i64,
    pub written_off_count: i64,
    pub total_amount: f64,
    pub open_amount: f64,
    pub by_reason: serde_json::Value,
    pub by_category: serde_json::Value,
    pub by_priority: serde_json::Value,
}

// ============================================================================
// Create Parameters
// ============================================================================

/// Parameters for creating a chargeback
pub struct ChargebackCreateParams {
    pub org_id: Uuid,
    pub customer_id: Option<Uuid>,
    pub customer_number: Option<String>,
    pub customer_name: Option<String>,
    pub receipt_id: Option<Uuid>,
    pub receipt_number: Option<String>,
    pub invoice_id: Option<Uuid>,
    pub invoice_number: Option<String>,
    pub chargeback_date: chrono::NaiveDate,
    pub gl_date: chrono::NaiveDate,
    pub currency_code: String,
    pub exchange_rate_type: Option<String>,
    pub exchange_rate: Option<f64>,
    pub amount: f64,
    pub tax_amount: f64,
    pub total_amount: f64,
    pub open_amount: f64,
    pub reason_code: String,
    pub reason_description: Option<String>,
    pub category: Option<String>,
    pub priority: String,
    pub assigned_to: Option<String>,
    pub assigned_team: Option<String>,
    pub due_date: Option<chrono::NaiveDate>,
    pub reference: Option<String>,
    pub customer_reference: Option<String>,
    pub sales_rep: Option<String>,
    pub notes: Option<String>,
    pub created_by: Option<Uuid>,
}

/// Parameters for creating a chargeback line
pub struct ChargebackLineCreateParams {
    pub org_id: Uuid,
    pub chargeback_id: Uuid,
    pub line_type: String,
    pub description: Option<String>,
    pub quantity: i32,
    pub unit_price: f64,
    pub amount: f64,
    pub tax_amount: f64,
    pub total_amount: f64,
    pub reason_code: Option<String>,
    pub reason_description: Option<String>,
    pub item_number: Option<String>,
    pub item_description: Option<String>,
    pub gl_account_code: Option<String>,
    pub gl_account_name: Option<String>,
    pub reference: Option<String>,
}

// ============================================================================
// Repository Trait
// ============================================================================

/// Repository trait for chargeback management data storage
#[async_trait]
pub trait ChargebackManagementRepository: Send + Sync {
    // Chargebacks
    async fn create_chargeback(&self, params: &ChargebackCreateParams) -> AtlasResult<Chargeback>;
    async fn get_chargeback(&self, id: Uuid) -> AtlasResult<Option<Chargeback>>;
    async fn get_chargeback_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<Chargeback>>;
    async fn list_chargebacks(
        &self, org_id: Uuid, status: Option<&str>, customer_id: Option<Uuid>,
        reason_code: Option<&str>, category: Option<&str>, priority: Option<&str>,
    ) -> AtlasResult<Vec<Chargeback>>;
    async fn update_chargeback_status(
        &self, id: Uuid, status: &str, resolution_date: Option<chrono::NaiveDate>,
        resolution_notes: Option<&str>, resolved_by: Option<Uuid>,
    ) -> AtlasResult<Chargeback>;
    async fn assign_chargeback(&self, id: Uuid, assigned_to: Option<&str>, assigned_team: Option<&str>) -> AtlasResult<Chargeback>;
    async fn update_notes(&self, id: Uuid, notes: Option<&str>) -> AtlasResult<Chargeback>;
    async fn update_chargeback_totals(&self, id: Uuid, amount: f64, tax_amount: f64, total_amount: f64, open_amount: f64) -> AtlasResult<()>;
    async fn delete_chargeback(&self, org_id: Uuid, number: &str) -> AtlasResult<()>;
    async fn get_next_chargeback_number(&self, org_id: Uuid) -> AtlasResult<i32>;

    // Lines
    async fn create_chargeback_line(&self, params: &ChargebackLineCreateParams) -> AtlasResult<ChargebackLine>;
    async fn list_chargeback_lines(&self, chargeback_id: Uuid) -> AtlasResult<Vec<ChargebackLine>>;
    async fn delete_chargeback_line(&self, chargeback_id: Uuid, line_id: Uuid) -> AtlasResult<()>;
    async fn get_next_line_number(&self, chargeback_id: Uuid) -> AtlasResult<i32>;

    // Activities
    async fn create_activity(
        &self, org_id: Uuid, chargeback_id: Uuid, activity_type: &str,
        description: Option<&str>, old_status: Option<&str>, new_status: Option<&str>,
        performed_by: Option<Uuid>, performed_by_name: Option<&str>, notes: Option<&str>,
    ) -> AtlasResult<ChargebackActivity>;
    async fn list_activities(&self, chargeback_id: Uuid) -> AtlasResult<Vec<ChargebackActivity>>;

    // Dashboard
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<ChargebackSummary>;
}

// ============================================================================
// PostgreSQL Implementation
// ============================================================================

/// PostgreSQL-backed chargeback management repository
pub struct PostgresChargebackManagementRepository {
    pool: PgPool,
}

impl PostgresChargebackManagementRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ChargebackManagementRepository for PostgresChargebackManagementRepository {
    async fn create_chargeback(&self, params: &ChargebackCreateParams) -> AtlasResult<Chargeback> {
        let seq = self.get_next_chargeback_number(params.org_id).await.unwrap_or(1);
        let chargeback_number = format!("CB-{:06}", seq);

        let row = sqlx::query_as::<_, Chargeback>(
            r#"INSERT INTO _atlas.chargebacks
               (organization_id, chargeback_number, customer_id, customer_number, customer_name,
                receipt_id, receipt_number, invoice_id, invoice_number,
                chargeback_date, gl_date, currency_code, exchange_rate_type, exchange_rate,
                amount, tax_amount, total_amount, open_amount,
                reason_code, reason_description, category,
                status, priority, assigned_to, assigned_team, due_date,
                reference, customer_reference, sales_rep, notes, created_by)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,'open',$22,$23,$24,$25,$26,$27,$28,$29,$30)
               RETURNING *"#,
        )
        .bind(params.org_id)
        .bind(&chargeback_number)
        .bind(params.customer_id)
        .bind(&params.customer_number)
        .bind(&params.customer_name)
        .bind(params.receipt_id)
        .bind(&params.receipt_number)
        .bind(params.invoice_id)
        .bind(&params.invoice_number)
        .bind(params.chargeback_date)
        .bind(params.gl_date)
        .bind(&params.currency_code)
        .bind(&params.exchange_rate_type)
        .bind(params.exchange_rate)
        .bind(params.amount)
        .bind(params.tax_amount)
        .bind(params.total_amount)
        .bind(params.open_amount)
        .bind(&params.reason_code)
        .bind(&params.reason_description)
        .bind(&params.category)
        .bind(&params.priority)
        .bind(&params.assigned_to)
        .bind(&params.assigned_team)
        .bind(params.due_date)
        .bind(&params.reference)
        .bind(&params.customer_reference)
        .bind(&params.sales_rep)
        .bind(&params.notes)
        .bind(params.created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row)
    }

    async fn get_chargeback(&self, id: Uuid) -> AtlasResult<Option<Chargeback>> {
        let row = sqlx::query_as::<_, Chargeback>(
            "SELECT * FROM _atlas.chargebacks WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row)
    }

    async fn get_chargeback_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<Chargeback>> {
        let row = sqlx::query_as::<_, Chargeback>(
            "SELECT * FROM _atlas.chargebacks WHERE organization_id = $1 AND chargeback_number = $2",
        )
        .bind(org_id)
        .bind(number)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row)
    }

    async fn list_chargebacks(
        &self, org_id: Uuid, status: Option<&str>, customer_id: Option<Uuid>,
        reason_code: Option<&str>, category: Option<&str>, priority: Option<&str>,
    ) -> AtlasResult<Vec<Chargeback>> {
        let rows = sqlx::query_as::<_, Chargeback>(
            r#"SELECT * FROM _atlas.chargebacks
               WHERE organization_id = $1
               AND ($2::text IS NULL OR status = $2)
               AND ($3::uuid IS NULL OR customer_id = $3)
               AND ($4::text IS NULL OR reason_code = $4)
               AND ($5::text IS NULL OR category = $5)
               AND ($6::text IS NULL OR priority = $6)
               ORDER BY chargeback_date DESC, chargeback_number"#,
        )
        .bind(org_id)
        .bind(status)
        .bind(customer_id)
        .bind(reason_code)
        .bind(category)
        .bind(priority)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows)
    }

    async fn update_chargeback_status(
        &self, id: Uuid, status: &str, resolution_date: Option<chrono::NaiveDate>,
        resolution_notes: Option<&str>, resolved_by: Option<Uuid>,
    ) -> AtlasResult<Chargeback> {
        let row = sqlx::query_as::<_, Chargeback>(
            r#"UPDATE _atlas.chargebacks
               SET status = $2, resolution_date = COALESCE($3, resolution_date),
                   resolution_notes = COALESCE($4, resolution_notes),
                   resolved_by = COALESCE($5, resolved_by),
                   updated_at = now()
               WHERE id = $1 RETURNING *"#,
        )
        .bind(id)
        .bind(status)
        .bind(resolution_date)
        .bind(resolution_notes)
        .bind(resolved_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row)
    }

    async fn assign_chargeback(&self, id: Uuid, assigned_to: Option<&str>, assigned_team: Option<&str>) -> AtlasResult<Chargeback> {
        let row = sqlx::query_as::<_, Chargeback>(
            r#"UPDATE _atlas.chargebacks
               SET assigned_to = $2, assigned_team = $3, updated_at = now()
               WHERE id = $1 RETURNING *"#,
        )
        .bind(id)
        .bind(assigned_to)
        .bind(assigned_team)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row)
    }

    async fn update_notes(&self, id: Uuid, notes: Option<&str>) -> AtlasResult<Chargeback> {
        let row = sqlx::query_as::<_, Chargeback>(
            r#"UPDATE _atlas.chargebacks SET notes = $2, updated_at = now() WHERE id = $1 RETURNING *"#,
        )
        .bind(id)
        .bind(notes)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row)
    }

    async fn update_chargeback_totals(&self, id: Uuid, amount: f64, tax_amount: f64, total_amount: f64, open_amount: f64) -> AtlasResult<()> {
        sqlx::query(
            r#"UPDATE _atlas.chargebacks
               SET amount = $2, tax_amount = $3, total_amount = $4, open_amount = $5, updated_at = now()
               WHERE id = $1"#,
        )
        .bind(id)
        .bind(amount)
        .bind(tax_amount)
        .bind(total_amount)
        .bind(open_amount)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn delete_chargeback(&self, org_id: Uuid, number: &str) -> AtlasResult<()> {
        let result = sqlx::query(
            "DELETE FROM _atlas.chargebacks WHERE organization_id = $1 AND chargeback_number = $2",
        )
        .bind(org_id)
        .bind(number)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound("Chargeback not found".to_string()));
        }
        Ok(())
    }

    async fn get_next_chargeback_number(&self, org_id: Uuid) -> AtlasResult<i32> {
        let row = sqlx::query(
            "SELECT COALESCE(MAX(CAST(SUBSTRING(chargeback_number FROM 'CB-(\\d+)') AS INTEGER)), 0) + 1 FROM _atlas.chargebacks WHERE organization_id = $1",
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        let seq: i32 = row.try_get(0).unwrap_or(1);
        Ok(seq)
    }

    // Lines
    async fn create_chargeback_line(&self, params: &ChargebackLineCreateParams) -> AtlasResult<ChargebackLine> {
        let line_number = self.get_next_line_number(params.chargeback_id).await.unwrap_or(1);

        let row = sqlx::query_as::<_, ChargebackLine>(
            r#"INSERT INTO _atlas.chargeback_lines
               (organization_id, chargeback_id, line_number, line_type, description,
                quantity, unit_price, amount, tax_amount, total_amount,
                reason_code, reason_description,
                item_number, item_description, gl_account_code, gl_account_name, reference)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17)
               RETURNING *"#,
        )
        .bind(params.org_id)
        .bind(params.chargeback_id)
        .bind(line_number)
        .bind(&params.line_type)
        .bind(&params.description)
        .bind(params.quantity)
        .bind(params.unit_price)
        .bind(params.amount)
        .bind(params.tax_amount)
        .bind(params.total_amount)
        .bind(&params.reason_code)
        .bind(&params.reason_description)
        .bind(&params.item_number)
        .bind(&params.item_description)
        .bind(&params.gl_account_code)
        .bind(&params.gl_account_name)
        .bind(&params.reference)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row)
    }

    async fn list_chargeback_lines(&self, chargeback_id: Uuid) -> AtlasResult<Vec<ChargebackLine>> {
        let rows = sqlx::query_as::<_, ChargebackLine>(
            "SELECT * FROM _atlas.chargeback_lines WHERE chargeback_id = $1 ORDER BY line_number",
        )
        .bind(chargeback_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows)
    }

    async fn delete_chargeback_line(&self, chargeback_id: Uuid, line_id: Uuid) -> AtlasResult<()> {
        let result = sqlx::query(
            "DELETE FROM _atlas.chargeback_lines WHERE chargeback_id = $1 AND id = $2",
        )
        .bind(chargeback_id)
        .bind(line_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        if result.rows_affected() == 0 {
            return Err(AtlasError::EntityNotFound("Chargeback line not found".to_string()));
        }
        Ok(())
    }

    async fn get_next_line_number(&self, chargeback_id: Uuid) -> AtlasResult<i32> {
        let row = sqlx::query(
            "SELECT COALESCE(MAX(line_number), 0) + 1 FROM _atlas.chargeback_lines WHERE chargeback_id = $1",
        )
        .bind(chargeback_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        let seq: i32 = row.try_get(0).unwrap_or(1);
        Ok(seq)
    }

    // Activities
    async fn create_activity(
        &self, org_id: Uuid, chargeback_id: Uuid, activity_type: &str,
        description: Option<&str>, old_status: Option<&str>, new_status: Option<&str>,
        performed_by: Option<Uuid>, performed_by_name: Option<&str>, notes: Option<&str>,
    ) -> AtlasResult<ChargebackActivity> {
        let row = sqlx::query_as::<_, ChargebackActivity>(
            r#"INSERT INTO _atlas.chargeback_activities
               (organization_id, chargeback_id, activity_type, description,
                old_status, new_status, performed_by, performed_by_name, notes)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
               RETURNING *"#,
        )
        .bind(org_id)
        .bind(chargeback_id)
        .bind(activity_type)
        .bind(description)
        .bind(old_status)
        .bind(new_status)
        .bind(performed_by)
        .bind(performed_by_name)
        .bind(notes)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row)
    }

    async fn list_activities(&self, chargeback_id: Uuid) -> AtlasResult<Vec<ChargebackActivity>> {
        let rows = sqlx::query_as::<_, ChargebackActivity>(
            "SELECT * FROM _atlas.chargeback_activities WHERE chargeback_id = $1 ORDER BY created_at",
        )
        .bind(chargeback_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows)
    }

    // Dashboard
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<ChargebackSummary> {
        let row = sqlx::query(
            r#"SELECT
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE status = 'open') as open_cnt,
                COUNT(*) FILTER (WHERE status = 'under_review') as review_cnt,
                COUNT(*) FILTER (WHERE status = 'accepted') as accepted_cnt,
                COUNT(*) FILTER (WHERE status = 'rejected') as rejected_cnt,
                COUNT(*) FILTER (WHERE status = 'written_off') as written_off_cnt,
                COALESCE(SUM(total_amount), 0) as total_amt,
                COALESCE(SUM(open_amount), 0) as open_amt
               FROM _atlas.chargebacks WHERE organization_id = $1"#,
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let by_reason = self.get_grouped_counts(org_id, "reason_code").await?;
        let by_category = self.get_grouped_counts(org_id, "category").await?;
        let by_priority = self.get_grouped_counts(org_id, "priority").await?;

        Ok(ChargebackSummary {
            total_chargebacks: row.try_get("total").unwrap_or(0),
            open_count: row.try_get("open_cnt").unwrap_or(0),
            under_review_count: row.try_get("review_cnt").unwrap_or(0),
            accepted_count: row.try_get("accepted_cnt").unwrap_or(0),
            rejected_count: row.try_get("rejected_cnt").unwrap_or(0),
            written_off_count: row.try_get("written_off_cnt").unwrap_or(0),
            total_amount: row.try_get("total_amt").unwrap_or(0.0),
            open_amount: row.try_get("open_amt").unwrap_or(0.0),
            by_reason,
            by_category,
            by_priority,
        })
    }
}

impl PostgresChargebackManagementRepository {
    async fn get_grouped_counts(&self, org_id: Uuid, column: &str) -> AtlasResult<serde_json::Value> {
        let query = format!(
            "SELECT {} as key, COUNT(*) as cnt, COALESCE(SUM(total_amount), 0) as total FROM _atlas.chargebacks WHERE organization_id = $1 GROUP BY {}",
            column, column
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
