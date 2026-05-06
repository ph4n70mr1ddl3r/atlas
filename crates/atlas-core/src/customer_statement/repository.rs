//! Customer Statement Repository
//!
//! PostgreSQL storage for customer statements and statement lines.

use atlas_shared::{
    CustomerStatement, CustomerStatementLine, CustomerStatementSummary,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

/// Repository trait for customer statement data storage
#[async_trait]
pub trait CustomerStatementRepository: Send + Sync {
    async fn create_statement(
        &self,
        org_id: Uuid,
        statement_number: &str,
        customer_id: Uuid,
        customer_number: Option<&str>,
        customer_name: Option<&str>,
        statement_date: chrono::NaiveDate,
        billing_period_from: chrono::NaiveDate,
        billing_period_to: chrono::NaiveDate,
        billing_cycle: &str,
        opening_balance: &str,
        total_charges: &str,
        total_payments: &str,
        total_credits: &str,
        total_adjustments: &str,
        closing_balance: &str,
        amount_due: &str,
        aging_current: &str,
        aging_1_30: &str,
        aging_31_60: &str,
        aging_61_90: &str,
        aging_91_120: &str,
        aging_121_plus: &str,
        currency_code: &str,
        delivery_method: Option<&str>,
        delivery_email: Option<&str>,
        previous_statement_id: Option<Uuid>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CustomerStatement>;

    async fn get_statement(&self, id: Uuid) -> AtlasResult<Option<CustomerStatement>>;
    async fn get_statement_by_number(&self, org_id: Uuid, statement_number: &str) -> AtlasResult<Option<CustomerStatement>>;
    async fn list_statements(
        &self,
        org_id: Uuid,
        customer_id: Option<Uuid>,
        status: Option<&str>,
        billing_cycle: Option<&str>,
    ) -> AtlasResult<Vec<CustomerStatement>>;
    async fn update_statement_status(&self, id: Uuid, status: &str) -> AtlasResult<CustomerStatement>;
    async fn update_statement_notes(&self, id: Uuid, notes: Option<&str>) -> AtlasResult<()>;
    async fn get_next_statement_number(&self, org_id: Uuid) -> AtlasResult<i32>;

    async fn create_statement_line(
        &self,
        org_id: Uuid,
        statement_id: Uuid,
        line_type: &str,
        transaction_id: Option<Uuid>,
        transaction_number: Option<&str>,
        transaction_date: Option<chrono::NaiveDate>,
        due_date: Option<chrono::NaiveDate>,
        original_amount: Option<&str>,
        amount: &str,
        description: Option<&str>,
        reference_type: Option<&str>,
        reference_id: Option<Uuid>,
        display_order: i32,
        metadata: serde_json::Value,
    ) -> AtlasResult<CustomerStatementLine>;

    async fn list_statement_lines(&self, statement_id: Uuid) -> AtlasResult<Vec<CustomerStatementLine>>;
    async fn delete_statement_line(&self, line_id: Uuid) -> AtlasResult<()>;
    async fn get_next_line_order(&self, statement_id: Uuid) -> AtlasResult<i32>;
    async fn get_statement_summary(&self, org_id: Uuid) -> AtlasResult<CustomerStatementSummary>;
}

// Helper functions
fn get_numeric_text(row: &sqlx::postgres::PgRow, col: &str) -> String {
    row.try_get::<f64, _>(col)
        .map(|v| format!("{:.2}", v))
        .unwrap_or_else(|_| "0.00".to_string())
}

fn row_to_statement(row: &sqlx::postgres::PgRow) -> CustomerStatement {
    CustomerStatement {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        statement_number: row.get("statement_number"),
        customer_id: row.get("customer_id"),
        customer_number: row.get("customer_number"),
        customer_name: row.get("customer_name"),
        statement_date: row.get("statement_date"),
        billing_period_from: row.get("billing_period_from"),
        billing_period_to: row.get("billing_period_to"),
        billing_cycle: row.get("billing_cycle"),
        opening_balance: get_numeric_text(row, "opening_balance"),
        total_charges: get_numeric_text(row, "total_charges"),
        total_payments: get_numeric_text(row, "total_payments"),
        total_credits: get_numeric_text(row, "total_credits"),
        total_adjustments: get_numeric_text(row, "total_adjustments"),
        closing_balance: get_numeric_text(row, "closing_balance"),
        amount_due: get_numeric_text(row, "amount_due"),
        aging_current: get_numeric_text(row, "aging_current"),
        aging_1_30: get_numeric_text(row, "aging_1_30"),
        aging_31_60: get_numeric_text(row, "aging_31_60"),
        aging_61_90: get_numeric_text(row, "aging_61_90"),
        aging_91_120: get_numeric_text(row, "aging_91_120"),
        aging_121_plus: get_numeric_text(row, "aging_121_plus"),
        currency_code: row.get("currency_code"),
        delivery_method: row.get("delivery_method"),
        delivery_email: row.get("delivery_email"),
        status: row.get("status"),
        generated_at: row.get("generated_at"),
        sent_at: row.get("sent_at"),
        viewed_at: row.get("viewed_at"),
        notes: row.get("notes"),
        previous_statement_id: row.get("previous_statement_id"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_line(row: &sqlx::postgres::PgRow) -> CustomerStatementLine {
    CustomerStatementLine {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        statement_id: row.get("statement_id"),
        line_type: row.get("line_type"),
        transaction_id: row.get("transaction_id"),
        transaction_number: row.get("transaction_number"),
        transaction_date: row.get("transaction_date"),
        due_date: row.get("due_date"),
        original_amount: Some(get_numeric_text(row, "original_amount")),
        amount: get_numeric_text(row, "amount"),
        running_balance: Some(get_numeric_text(row, "running_balance")),
        description: row.get("description"),
        reference_type: row.get("reference_type"),
        reference_id: row.get("reference_id"),
        display_order: row.try_get("display_order").unwrap_or(0),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.get("created_at"),
    }
}

/// PostgreSQL implementation
pub struct PostgresCustomerStatementRepository {
    pool: PgPool,
}

impl PostgresCustomerStatementRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CustomerStatementRepository for PostgresCustomerStatementRepository {
    async fn create_statement(
        &self,
        org_id: Uuid,
        statement_number: &str,
        customer_id: Uuid,
        customer_number: Option<&str>,
        customer_name: Option<&str>,
        statement_date: chrono::NaiveDate,
        billing_period_from: chrono::NaiveDate,
        billing_period_to: chrono::NaiveDate,
        billing_cycle: &str,
        opening_balance: &str,
        total_charges: &str,
        total_payments: &str,
        total_credits: &str,
        total_adjustments: &str,
        closing_balance: &str,
        amount_due: &str,
        aging_current: &str,
        aging_1_30: &str,
        aging_31_60: &str,
        aging_61_90: &str,
        aging_91_120: &str,
        aging_121_plus: &str,
        currency_code: &str,
        delivery_method: Option<&str>,
        delivery_email: Option<&str>,
        previous_statement_id: Option<Uuid>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CustomerStatement> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.customer_statements
                (organization_id, statement_number, customer_id, customer_number, customer_name,
                 statement_date, billing_period_from, billing_period_to, billing_cycle,
                 opening_balance, total_charges, total_payments, total_credits, total_adjustments,
                 closing_balance, amount_due,
                 aging_current, aging_1_30, aging_31_60, aging_61_90, aging_91_120, aging_121_plus,
                 currency_code, delivery_method, delivery_email,
                 previous_statement_id, notes, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,
                    $10::numeric,$11::numeric,$12::numeric,$13::numeric,$14::numeric,
                    $15::numeric,$16::numeric,
                    $17::numeric,$18::numeric,$19::numeric,$20::numeric,$21::numeric,$22::numeric,
                    $23,$24,$25,$26,$27,$28)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(statement_number).bind(customer_id).bind(customer_number).bind(customer_name)
        .bind(statement_date).bind(billing_period_from).bind(billing_period_to).bind(billing_cycle)
        .bind(opening_balance).bind(total_charges).bind(total_payments).bind(total_credits).bind(total_adjustments)
        .bind(closing_balance).bind(amount_due)
        .bind(aging_current).bind(aging_1_30).bind(aging_31_60).bind(aging_61_90).bind(aging_91_120).bind(aging_121_plus)
        .bind(currency_code).bind(delivery_method).bind(delivery_email)
        .bind(previous_statement_id).bind(notes).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_statement(&row))
    }

    async fn get_statement(&self, id: Uuid) -> AtlasResult<Option<CustomerStatement>> {
        let row = sqlx::query("SELECT * FROM _atlas.customer_statements WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_statement(&r)))
    }

    async fn get_statement_by_number(&self, org_id: Uuid, statement_number: &str) -> AtlasResult<Option<CustomerStatement>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.customer_statements WHERE organization_id = $1 AND statement_number = $2"
        )
        .bind(org_id).bind(statement_number)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_statement(&r)))
    }

    async fn list_statements(
        &self,
        org_id: Uuid,
        customer_id: Option<Uuid>,
        status: Option<&str>,
        billing_cycle: Option<&str>,
    ) -> AtlasResult<Vec<CustomerStatement>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM _atlas.customer_statements
            WHERE organization_id = $1
              AND ($2::uuid IS NULL OR customer_id = $2)
              AND ($3::text IS NULL OR status = $3)
              AND ($4::text IS NULL OR billing_cycle = $4)
            ORDER BY statement_date DESC, statement_number DESC
            "#,
        )
        .bind(org_id).bind(customer_id).bind(status).bind(billing_cycle)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_statement).collect())
    }

    async fn update_statement_status(&self, id: Uuid, status: &str) -> AtlasResult<CustomerStatement> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.customer_statements
            SET status = $2,
                generated_at = CASE WHEN $2 = 'generated' AND generated_at IS NULL THEN now() ELSE generated_at END,
                sent_at = CASE WHEN $2 = 'sent' THEN now() ELSE sent_at END,
                viewed_at = CASE WHEN $2 = 'viewed' AND viewed_at IS NULL THEN now() ELSE viewed_at END,
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(status)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_statement(&row))
    }

    async fn update_statement_notes(&self, id: Uuid, notes: Option<&str>) -> AtlasResult<()> {
        sqlx::query("UPDATE _atlas.customer_statements SET notes = $2, updated_at = now() WHERE id = $1")
            .bind(id).bind(notes)
            .execute(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn get_next_statement_number(&self, org_id: Uuid) -> AtlasResult<i32> {
        let row = sqlx::query(
            "SELECT COALESCE(MAX(CAST(SUBSTRING(statement_number FROM 4) AS INT)), 0) as max_num FROM _atlas.customer_statements WHERE organization_id = $1"
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        let max_num: i64 = row.try_get("max_num").unwrap_or(0);
        Ok((max_num + 1) as i32)
    }

    // ========================================================================
    // Statement Lines
    // ========================================================================

    async fn create_statement_line(
        &self,
        org_id: Uuid,
        statement_id: Uuid,
        line_type: &str,
        transaction_id: Option<Uuid>,
        transaction_number: Option<&str>,
        transaction_date: Option<chrono::NaiveDate>,
        due_date: Option<chrono::NaiveDate>,
        original_amount: Option<&str>,
        amount: &str,
        description: Option<&str>,
        reference_type: Option<&str>,
        reference_id: Option<Uuid>,
        display_order: i32,
        metadata: serde_json::Value,
    ) -> AtlasResult<CustomerStatementLine> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.customer_statement_lines
                (organization_id, statement_id, line_type,
                 transaction_id, transaction_number, transaction_date, due_date,
                 original_amount, amount, description,
                 reference_type, reference_id, display_order, metadata)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8::numeric,$9::numeric,$10,$11,$12,$13,$14)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(statement_id).bind(line_type)
        .bind(transaction_id).bind(transaction_number).bind(transaction_date).bind(due_date)
        .bind(original_amount).bind(amount).bind(description)
        .bind(reference_type).bind(reference_id).bind(display_order).bind(metadata)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_line(&row))
    }

    async fn list_statement_lines(&self, statement_id: Uuid) -> AtlasResult<Vec<CustomerStatementLine>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.customer_statement_lines WHERE statement_id = $1 ORDER BY display_order, created_at"
        )
        .bind(statement_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_line).collect())
    }

    async fn delete_statement_line(&self, line_id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.customer_statement_lines WHERE id = $1")
            .bind(line_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn get_next_line_order(&self, statement_id: Uuid) -> AtlasResult<i32> {
        let row = sqlx::query(
            "SELECT COALESCE(MAX(display_order), 0) + 1 as next_order FROM _atlas.customer_statement_lines WHERE statement_id = $1"
        )
        .bind(statement_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        let next: i32 = row.try_get("next_order").unwrap_or(1);
        Ok(next)
    }

    async fn get_statement_summary(&self, org_id: Uuid) -> AtlasResult<CustomerStatementSummary> {
        let row = sqlx::query(
            r#"SELECT
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE status = 'draft') as draft_cnt,
                COUNT(*) FILTER (WHERE status = 'sent') as sent_cnt
            FROM _atlas.customer_statements WHERE organization_id = $1"#,
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let total: i64 = row.try_get("total").unwrap_or(0);
        let draft: i64 = row.try_get("draft_cnt").unwrap_or(0);
        let sent: i64 = row.try_get("sent_cnt").unwrap_or(0);

        let amt_row = sqlx::query(
            "SELECT COALESCE(SUM(closing_balance), 0) as outstanding FROM _atlas.customer_statements WHERE organization_id = $1 AND status NOT IN ('cancelled')"
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let outstanding: f64 = amt_row.try_get("outstanding").unwrap_or(0.0);

        // By billing cycle
        let cycle_rows = sqlx::query(
            "SELECT billing_cycle, COUNT(*) as count FROM _atlas.customer_statements WHERE organization_id = $1 GROUP BY billing_cycle"
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let mut by_cycle = serde_json::Map::new();
        for r in &cycle_rows {
            let bc: String = r.try_get("billing_cycle").unwrap_or_default();
            let count: i64 = r.try_get("count").unwrap_or(0);
            by_cycle.insert(bc, serde_json::json!(count));
        }

        // By currency
        let curr_rows = sqlx::query(
            "SELECT currency_code, COUNT(*) as count FROM _atlas.customer_statements WHERE organization_id = $1 GROUP BY currency_code"
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

        Ok(CustomerStatementSummary {
            total_statements: total as i32,
            draft_count: draft as i32,
            sent_count: sent as i32,
            total_amount_outstanding: format!("{:.2}", outstanding),
            by_billing_cycle: serde_json::Value::Object(by_cycle),
            by_currency: serde_json::Value::Object(by_currency),
        })
    }
}
