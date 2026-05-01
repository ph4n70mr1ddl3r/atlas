//! Inflation Adjustment Repository
//!
//! Storage interface for inflation adjustment data.

use atlas_shared::{
    InflationIndex, InflationIndexRate, InflationAdjustmentRun, InflationAdjustmentLine,
    InflationDashboardSummary,
    AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

/// Repository trait for inflation adjustment data storage
#[async_trait]
pub trait InflationAdjustmentRepository: Send + Sync {
    // Indices
    async fn create_index(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        country_code: &str,
        currency_code: &str,
        index_type: &str,
        is_hyperinflationary: bool,
        hyperinflationary_start_date: Option<chrono::NaiveDate>,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<InflationIndex>;

    async fn get_index(&self, id: Uuid) -> AtlasResult<Option<InflationIndex>>;
    async fn get_index_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<InflationIndex>>;
    async fn list_indices(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<InflationIndex>>;

    // Index Rates
    async fn create_index_rate(
        &self,
        org_id: Uuid,
        index_id: Uuid,
        period_start: chrono::NaiveDate,
        period_end: chrono::NaiveDate,
        index_value: &str,
        cumulative_factor: &str,
        period_factor: &str,
        source: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<InflationIndexRate>;

    async fn list_index_rates(&self, index_id: Uuid) -> AtlasResult<Vec<InflationIndexRate>>;

    // Runs
    async fn create_run(
        &self,
        org_id: Uuid,
        run_number: &str,
        name: Option<&str>,
        description: Option<&str>,
        index_id: Uuid,
        ledger_id: Option<Uuid>,
        from_period: chrono::NaiveDate,
        to_period: chrono::NaiveDate,
        adjustment_method: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<InflationAdjustmentRun>;

    async fn get_run(&self, id: Uuid) -> AtlasResult<Option<InflationAdjustmentRun>>;
    async fn list_runs(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<InflationAdjustmentRun>>;
    async fn update_run_status(
        &self,
        id: Uuid,
        status: &str,
        submitted_by: Option<Uuid>,
        approved_by: Option<Uuid>,
    ) -> AtlasResult<InflationAdjustmentRun>;
    async fn update_run_totals(
        &self,
        id: Uuid,
        total_debit: &str,
        total_credit: &str,
        total_gain_loss: &str,
        account_count: i32,
    ) -> AtlasResult<()>;

    // Lines
    async fn create_adjustment_line(
        &self,
        org_id: Uuid,
        run_id: Uuid,
        line_number: i32,
        account_code: &str,
        account_name: Option<&str>,
        account_type: &str,
        balance_type: &str,
        original_balance: &str,
        restated_balance: &str,
        adjustment_amount: &str,
        inflation_factor: &str,
        acquisition_date: Option<chrono::NaiveDate>,
        gain_loss_amount: &str,
        gain_loss_account: Option<&str>,
        currency_code: Option<&str>,
    ) -> AtlasResult<InflationAdjustmentLine>;

    async fn list_run_lines(&self, run_id: Uuid) -> AtlasResult<Vec<InflationAdjustmentLine>>;

    // Dashboard
    async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<InflationDashboardSummary>;
}

/// PostgreSQL implementation
pub struct PostgresInflationAdjustmentRepository {
    pool: PgPool,
}

impl PostgresInflationAdjustmentRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

use sqlx::Row;

fn row_to_index(row: &sqlx::postgres::PgRow) -> InflationIndex {
    InflationIndex {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        code: row.get("code"),
        name: row.get("name"),
        description: row.get("description"),
        country_code: row.get("country_code"),
        currency_code: row.get("currency_code"),
        index_type: row.get("index_type"),
        is_hyperinflationary: row.get("is_hyperinflationary"),
        hyperinflationary_start_date: row.get("hyperinflationary_start_date"),
        effective_from: row.get("effective_from"),
        effective_to: row.get("effective_to"),
        status: row.get("status"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_index_rate(row: &sqlx::postgres::PgRow) -> InflationIndexRate {
    InflationIndexRate {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        index_id: row.get("index_id"),
        period_start: row.get("period_start"),
        period_end: row.get("period_end"),
        index_value: row.try_get("index_value").unwrap_or("0".to_string()),
        cumulative_factor: row.try_get("cumulative_factor").unwrap_or("1".to_string()),
        period_factor: row.try_get("period_factor").unwrap_or("1".to_string()),
        source: row.get("source"),
        status: row.get("status"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_run(row: &sqlx::postgres::PgRow) -> InflationAdjustmentRun {
    InflationAdjustmentRun {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        run_number: row.get("run_number"),
        name: row.get("name"),
        description: row.get("description"),
        index_id: row.get("index_id"),
        ledger_id: row.get("ledger_id"),
        from_period: row.get("from_period"),
        to_period: row.get("to_period"),
        adjustment_method: row.get("adjustment_method"),
        total_debit_adjustment: row.try_get("total_debit_adjustment").unwrap_or("0".to_string()),
        total_credit_adjustment: row.try_get("total_credit_adjustment").unwrap_or("0".to_string()),
        total_monetary_gain_loss: row.try_get("total_monetary_gain_loss").unwrap_or("0".to_string()),
        account_count: row.try_get("account_count").unwrap_or(0),
        status: row.get("status"),
        submitted_by: row.get("submitted_by"),
        submitted_at: row.get("submitted_at"),
        approved_by: row.get("approved_by"),
        approved_at: row.get("approved_at"),
        completed_at: row.get("completed_at"),
        journal_entry_id: row.get("journal_entry_id"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_adjustment_line(row: &sqlx::postgres::PgRow) -> InflationAdjustmentLine {
    InflationAdjustmentLine {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        run_id: row.get("run_id"),
        line_number: row.get("line_number"),
        account_code: row.get("account_code"),
        account_name: row.get("account_name"),
        account_type: row.get("account_type"),
        balance_type: row.get("balance_type"),
        original_balance: row.try_get("original_balance").unwrap_or("0".to_string()),
        restated_balance: row.try_get("restated_balance").unwrap_or("0".to_string()),
        adjustment_amount: row.try_get("adjustment_amount").unwrap_or("0".to_string()),
        inflation_factor: row.try_get("inflation_factor").unwrap_or("1".to_string()),
        acquisition_date: row.get("acquisition_date"),
        gain_loss_amount: row.try_get("gain_loss_amount").unwrap_or("0".to_string()),
        gain_loss_account: row.get("gain_loss_account"),
        currency_code: row.get("currency_code"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

#[async_trait]
impl InflationAdjustmentRepository for PostgresInflationAdjustmentRepository {
    async fn create_index(
        &self,
        org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        country_code: &str, currency_code: &str, index_type: &str,
        is_hyperinflationary: bool, hyperinflationary_start_date: Option<chrono::NaiveDate>,
        effective_from: Option<chrono::NaiveDate>, effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<InflationIndex> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.inflation_indices
                (organization_id, code, name, description, country_code, currency_code,
                 index_type, is_hyperinflationary, hyperinflationary_start_date,
                 effective_from, effective_to, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(code).bind(name).bind(description)
        .bind(country_code).bind(currency_code).bind(index_type)
        .bind(is_hyperinflationary).bind(hyperinflationary_start_date)
        .bind(effective_from).bind(effective_to).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_index(&row))
    }

    async fn get_index(&self, id: Uuid) -> AtlasResult<Option<InflationIndex>> {
        let row = sqlx::query("SELECT * FROM _atlas.inflation_indices WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_index(&r)))
    }

    async fn get_index_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<InflationIndex>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.inflation_indices WHERE organization_id = $1 AND code = $2"
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_index(&r)))
    }

    async fn list_indices(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<InflationIndex>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.inflation_indices
               WHERE organization_id = $1 AND ($2::text IS NULL OR status = $2)
               ORDER BY created_at DESC"#,
        )
        .bind(org_id).bind(status)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_index).collect())
    }

    async fn create_index_rate(
        &self,
        org_id: Uuid, index_id: Uuid,
        period_start: chrono::NaiveDate, period_end: chrono::NaiveDate,
        index_value: &str, cumulative_factor: &str, period_factor: &str,
        source: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<InflationIndexRate> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.inflation_index_rates
                (organization_id, index_id, period_start, period_end,
                 index_value, cumulative_factor, period_factor, source, created_by)
            VALUES ($1, $2, $3, $4, $5::decimal, $6::decimal, $7::decimal, $8, $9)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(index_id).bind(period_start).bind(period_end)
        .bind(index_value).bind(cumulative_factor).bind(period_factor)
        .bind(source).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_index_rate(&row))
    }

    async fn list_index_rates(&self, index_id: Uuid) -> AtlasResult<Vec<InflationIndexRate>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.inflation_index_rates WHERE index_id = $1 ORDER BY period_start"
        )
        .bind(index_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_index_rate).collect())
    }

    async fn create_run(
        &self,
        org_id: Uuid, run_number: &str, name: Option<&str>, description: Option<&str>,
        index_id: Uuid, ledger_id: Option<Uuid>,
        from_period: chrono::NaiveDate, to_period: chrono::NaiveDate,
        adjustment_method: &str, created_by: Option<Uuid>,
    ) -> AtlasResult<InflationAdjustmentRun> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.inflation_adjustment_runs
                (organization_id, run_number, name, description, index_id, ledger_id,
                 from_period, to_period, adjustment_method, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(run_number).bind(name).bind(description)
        .bind(index_id).bind(ledger_id).bind(from_period).bind(to_period)
        .bind(adjustment_method).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_run(&row))
    }

    async fn get_run(&self, id: Uuid) -> AtlasResult<Option<InflationAdjustmentRun>> {
        let row = sqlx::query("SELECT * FROM _atlas.inflation_adjustment_runs WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_run(&r)))
    }

    async fn list_runs(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<InflationAdjustmentRun>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.inflation_adjustment_runs
               WHERE organization_id = $1 AND ($2::text IS NULL OR status = $2)
               ORDER BY created_at DESC"#,
        )
        .bind(org_id).bind(status)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_run).collect())
    }

    async fn update_run_status(
        &self, id: Uuid, status: &str,
        submitted_by: Option<Uuid>, approved_by: Option<Uuid>,
    ) -> AtlasResult<InflationAdjustmentRun> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.inflation_adjustment_runs
            SET status = $2,
                submitted_by = COALESCE($3, submitted_by),
                submitted_at = CASE WHEN $2 = 'submitted' THEN now() ELSE submitted_at END,
                approved_by = COALESCE($4, approved_by),
                approved_at = CASE WHEN $2 = 'approved' THEN now() ELSE approved_at END,
                completed_at = CASE WHEN $2 = 'completed' THEN now() ELSE completed_at END,
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(status).bind(submitted_by).bind(approved_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_run(&row))
    }

    async fn update_run_totals(
        &self, id: Uuid, total_debit: &str, total_credit: &str,
        total_gain_loss: &str, account_count: i32,
    ) -> AtlasResult<()> {
        sqlx::query(
            r#"
            UPDATE _atlas.inflation_adjustment_runs
            SET total_debit_adjustment = $2::decimal, total_credit_adjustment = $3::decimal,
                total_monetary_gain_loss = $4::decimal, account_count = $5, updated_at = now()
            WHERE id = $1
            "#,
        )
        .bind(id).bind(total_debit).bind(total_credit)
        .bind(total_gain_loss).bind(account_count)
        .execute(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn create_adjustment_line(
        &self,
        org_id: Uuid, run_id: Uuid, line_number: i32,
        account_code: &str, account_name: Option<&str>,
        account_type: &str, balance_type: &str,
        original_balance: &str, restated_balance: &str,
        adjustment_amount: &str, inflation_factor: &str,
        acquisition_date: Option<chrono::NaiveDate>,
        gain_loss_amount: &str, gain_loss_account: Option<&str>,
        currency_code: Option<&str>,
    ) -> AtlasResult<InflationAdjustmentLine> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.inflation_adjustment_lines
                (organization_id, run_id, line_number, account_code, account_name,
                 account_type, balance_type, original_balance, restated_balance,
                 adjustment_amount, inflation_factor, acquisition_date,
                 gain_loss_amount, gain_loss_account, currency_code)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8::decimal, $9::decimal,
                    $10::decimal, $11::decimal, $12, $13::decimal, $14, $15)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(run_id).bind(line_number)
        .bind(account_code).bind(account_name)
        .bind(account_type).bind(balance_type)
        .bind(original_balance).bind(restated_balance)
        .bind(adjustment_amount).bind(inflation_factor)
        .bind(acquisition_date).bind(gain_loss_amount)
        .bind(gain_loss_account).bind(currency_code)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_adjustment_line(&row))
    }

    async fn list_run_lines(&self, run_id: Uuid) -> AtlasResult<Vec<InflationAdjustmentLine>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.inflation_adjustment_lines WHERE run_id = $1 ORDER BY line_number"
        )
        .bind(run_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_adjustment_line).collect())
    }

    async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<InflationDashboardSummary> {
        let index_row = sqlx::query(
            r#"SELECT
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE is_hyperinflationary) as hyper
            FROM _atlas.inflation_indices WHERE organization_id = $1"#,
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        let run_row = sqlx::query(
            r#"SELECT
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE status = 'draft') as draft,
                COUNT(*) FILTER (WHERE status = 'completed') as completed,
                COALESCE(SUM(total_debit_adjustment + total_credit_adjustment), 0) as total_adj,
                COALESCE(SUM(ABS(total_monetary_gain_loss)), 0) as total_gl
            FROM _atlas.inflation_adjustment_runs WHERE organization_id = $1"#,
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(InflationDashboardSummary {
            total_indices: index_row.try_get::<i64, _>("total").unwrap_or(0) as i32,
            hyperinflationary_indices: index_row.try_get::<i64, _>("hyper").unwrap_or(0) as i32,
            total_runs: run_row.try_get::<i64, _>("total").unwrap_or(0) as i32,
            draft_runs: run_row.try_get::<i64, _>("draft").unwrap_or(0) as i32,
            completed_runs: run_row.try_get::<i64, _>("completed").unwrap_or(0) as i32,
            total_adjustments: format!("{:.2}", run_row.try_get::<f64, _>("total_adj").unwrap_or(0.0)),
            total_gain_loss: format!("{:.2}", run_row.try_get::<f64, _>("total_gl").unwrap_or(0.0)),
        })
    }
}
