//! Cash Management Repository
//!
//! PostgreSQL storage for cash positions, forecast templates, forecast sources,
//! cash forecasts, and forecast lines.

use atlas_shared::{
    CashPosition,
    CashForecastTemplate, CashForecastSource,
    CashForecast, CashForecastLine,
    AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

/// Repository trait for cash management data storage
#[async_trait]
pub trait CashManagementRepository: Send + Sync {
    // ========================================================================
    // Cash Positions
    // ========================================================================

    async fn upsert_cash_position(
        &self,
        org_id: Uuid,
        bank_account_id: Uuid,
        account_number: &str,
        account_name: &str,
        currency_code: &str,
        book_balance: &str,
        available_balance: &str,
        float_amount: &str,
        one_day_float: &str,
        two_day_float: &str,
        position_date: chrono::NaiveDate,
        average_balance: Option<&str>,
        prior_day_balance: Option<&str>,
        projected_inflows: &str,
        projected_outflows: &str,
        projected_net: &str,
        is_reconciled: bool,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CashPosition>;

    async fn get_cash_position(
        &self,
        org_id: Uuid,
        bank_account_id: Uuid,
        position_date: chrono::NaiveDate,
    ) -> AtlasResult<Option<CashPosition>>;

    async fn get_cash_position_by_id(&self, id: Uuid) -> AtlasResult<Option<CashPosition>>;
    async fn list_cash_positions(&self, org_id: Uuid, position_date: Option<chrono::NaiveDate>) -> AtlasResult<Vec<CashPosition>>;

    // ========================================================================
    // Forecast Templates
    // ========================================================================

    async fn create_forecast_template(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        bucket_type: &str,
        number_of_periods: i32,
        start_offset_days: i32,
        is_default: bool,
        columns: serde_json::Value,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CashForecastTemplate>;

    async fn get_forecast_template(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<CashForecastTemplate>>;
    async fn get_forecast_template_by_id(&self, id: Uuid) -> AtlasResult<Option<CashForecastTemplate>>;
    async fn list_forecast_templates(&self, org_id: Uuid) -> AtlasResult<Vec<CashForecastTemplate>>;
    async fn delete_forecast_template(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // ========================================================================
    // Forecast Sources
    // ========================================================================

    async fn create_forecast_source(
        &self,
        org_id: Uuid,
        template_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        source_type: &str,
        cash_flow_direction: &str,
        is_actual: bool,
        display_order: i32,
        lead_time_days: i32,
        payment_terms_reference: Option<&str>,
        account_code_filter: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CashForecastSource>;

    async fn get_forecast_source(&self, org_id: Uuid, template_id: Uuid, code: &str) -> AtlasResult<Option<CashForecastSource>>;
    async fn list_forecast_sources(&self, template_id: Uuid) -> AtlasResult<Vec<CashForecastSource>>;
    async fn delete_forecast_source(&self, org_id: Uuid, template_id: Uuid, code: &str) -> AtlasResult<()>;

    // ========================================================================
    // Cash Forecasts
    // ========================================================================

    async fn create_forecast(
        &self,
        org_id: Uuid,
        forecast_number: &str,
        template_id: Uuid,
        template_name: &str,
        name: &str,
        description: Option<&str>,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
        opening_balance: &str,
        total_inflows: &str,
        total_outflows: &str,
        net_cash_flow: &str,
        closing_balance: &str,
        minimum_balance: &str,
        maximum_balance: &str,
        deficit_count: i32,
        surplus_count: i32,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CashForecast>;

    async fn get_forecast(&self, id: Uuid) -> AtlasResult<Option<CashForecast>>;
    async fn get_forecast_by_number(&self, org_id: Uuid, forecast_number: &str) -> AtlasResult<Option<CashForecast>>;
    async fn list_forecasts(&self, org_id: Uuid, template_id: Option<Uuid>, status: Option<&str>) -> AtlasResult<Vec<CashForecast>>;
    async fn update_forecast_status(
        &self,
        id: Uuid,
        status: &str,
        approved_by: Option<Uuid>,
    ) -> AtlasResult<CashForecast>;
    async fn supersede_previous_forecasts(&self, template_id: Uuid, new_forecast_id: Uuid) -> AtlasResult<()>;

    // ========================================================================
    // Forecast Lines
    // ========================================================================

    async fn create_forecast_line(
        &self,
        org_id: Uuid,
        forecast_id: Uuid,
        source_id: Uuid,
        source_name: &str,
        source_type: &str,
        cash_flow_direction: &str,
        period_start_date: chrono::NaiveDate,
        period_end_date: chrono::NaiveDate,
        period_label: &str,
        period_sequence: i32,
        amount: &str,
        cumulative_amount: &str,
        is_actual: bool,
        currency_code: &str,
        transaction_count: i32,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CashForecastLine>;

    async fn list_forecast_lines(&self, forecast_id: Uuid) -> AtlasResult<Vec<CashForecastLine>>;
    async fn list_forecast_lines_by_period(
        &self,
        forecast_id: Uuid,
        period_start_date: chrono::NaiveDate,
        period_end_date: chrono::NaiveDate,
    ) -> AtlasResult<Vec<CashForecastLine>>;
}

/// PostgreSQL implementation
pub struct PostgresCashManagementRepository {
    pool: PgPool,
}

impl PostgresCashManagementRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn get_num(row: &sqlx::postgres::PgRow, col: &str) -> String {
    let v: serde_json::Value = row.try_get(col).unwrap_or(serde_json::json!("0"));
    v.to_string()
}

fn row_to_cash_position(row: &sqlx::postgres::PgRow) -> CashPosition {
    CashPosition {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        bank_account_id: row.get("bank_account_id"),
        account_number: row.get("account_number"),
        account_name: row.get("account_name"),
        currency_code: row.get("currency_code"),
        book_balance: get_num(row, "book_balance"),
        available_balance: get_num(row, "available_balance"),
        float_amount: get_num(row, "float_amount"),
        one_day_float: get_num(row, "one_day_float"),
        two_day_float: get_num(row, "two_day_float"),
        position_date: row.get("position_date"),
        average_balance: row.try_get("average_balance").unwrap_or(None),
        prior_day_balance: row.try_get("prior_day_balance").unwrap_or(None),
        projected_inflows: get_num(row, "projected_inflows"),
        projected_outflows: get_num(row, "projected_outflows"),
        projected_net: get_num(row, "projected_net"),
        is_reconciled: row.get("is_reconciled"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_template(row: &sqlx::postgres::PgRow) -> CashForecastTemplate {
    CashForecastTemplate {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        code: row.get("code"),
        name: row.get("name"),
        description: row.get("description"),
        bucket_type: row.get("bucket_type"),
        number_of_periods: row.get("number_of_periods"),
        start_offset_days: row.get("start_offset_days"),
        is_default: row.get("is_default"),
        is_active: row.get("is_active"),
        columns: row.try_get("columns").unwrap_or(serde_json::json!([])),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_source(row: &sqlx::postgres::PgRow) -> CashForecastSource {
    CashForecastSource {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        template_id: row.get("template_id"),
        code: row.get("code"),
        name: row.get("name"),
        description: row.get("description"),
        source_type: row.get("source_type"),
        cash_flow_direction: row.get("cash_flow_direction"),
        is_actual: row.get("is_actual"),
        display_order: row.get("display_order"),
        is_active: row.get("is_active"),
        lead_time_days: row.get("lead_time_days"),
        payment_terms_reference: row.get("payment_terms_reference"),
        account_code_filter: row.get("account_code_filter"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_forecast(row: &sqlx::postgres::PgRow) -> CashForecast {
    CashForecast {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        forecast_number: row.get("forecast_number"),
        template_id: row.get("template_id"),
        template_name: row.get("template_name"),
        name: row.get("name"),
        description: row.get("description"),
        start_date: row.get("start_date"),
        end_date: row.get("end_date"),
        opening_balance: get_num(row, "opening_balance"),
        total_inflows: get_num(row, "total_inflows"),
        total_outflows: get_num(row, "total_outflows"),
        net_cash_flow: get_num(row, "net_cash_flow"),
        closing_balance: get_num(row, "closing_balance"),
        minimum_balance: get_num(row, "minimum_balance"),
        maximum_balance: get_num(row, "maximum_balance"),
        deficit_count: row.get("deficit_count"),
        surplus_count: row.get("surplus_count"),
        status: row.get("status"),
        is_latest: row.get("is_latest"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        approved_by: row.get("approved_by"),
        approved_at: row.get("approved_at"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_forecast_line(row: &sqlx::postgres::PgRow) -> CashForecastLine {
    CashForecastLine {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        forecast_id: row.get("forecast_id"),
        source_id: row.get("source_id"),
        source_name: row.get("source_name"),
        source_type: row.get("source_type"),
        cash_flow_direction: row.get("cash_flow_direction"),
        period_start_date: row.get("period_start_date"),
        period_end_date: row.get("period_end_date"),
        period_label: row.get("period_label"),
        period_sequence: row.get("period_sequence"),
        amount: get_num(row, "amount"),
        cumulative_amount: get_num(row, "cumulative_amount"),
        is_actual: row.get("is_actual"),
        currency_code: row.get("currency_code"),
        transaction_count: row.get("transaction_count"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

#[async_trait]
impl CashManagementRepository for PostgresCashManagementRepository {
    // ========================================================================
    // Cash Positions
    // ========================================================================

    async fn upsert_cash_position(
        &self,
        org_id: Uuid,
        bank_account_id: Uuid,
        account_number: &str,
        account_name: &str,
        currency_code: &str,
        book_balance: &str,
        available_balance: &str,
        float_amount: &str,
        one_day_float: &str,
        two_day_float: &str,
        position_date: chrono::NaiveDate,
        average_balance: Option<&str>,
        prior_day_balance: Option<&str>,
        projected_inflows: &str,
        projected_outflows: &str,
        projected_net: &str,
        is_reconciled: bool,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CashPosition> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.cash_positions
                (organization_id, bank_account_id, account_number, account_name,
                 currency_code, book_balance, available_balance,
                 float_amount, one_day_float, two_day_float,
                 position_date, average_balance, prior_day_balance,
                 projected_inflows, projected_outflows, projected_net,
                 is_reconciled, created_by)
            VALUES ($1, $2, $3, $4, $5, $6::numeric, $7::numeric,
                    $8::numeric, $9::numeric, $10::numeric,
                    $11, $12::numeric, $13::numeric,
                    $14::numeric, $15::numeric, $16::numeric,
                    $17, $18)
            ON CONFLICT (organization_id, bank_account_id, position_date) DO UPDATE
                SET account_number = $3, account_name = $4, currency_code = $5,
                    book_balance = $6::numeric, available_balance = $7::numeric,
                    float_amount = $8::numeric, one_day_float = $9::numeric,
                    two_day_float = $10::numeric,
                    average_balance = $12::numeric, prior_day_balance = $13::numeric,
                    projected_inflows = $14::numeric, projected_outflows = $15::numeric,
                    projected_net = $16::numeric, is_reconciled = $17, updated_at = now()
            RETURNING *
            "#,
        )
        .bind(org_id).bind(bank_account_id).bind(account_number).bind(account_name)
        .bind(currency_code).bind(book_balance).bind(available_balance)
        .bind(float_amount).bind(one_day_float).bind(two_day_float)
        .bind(position_date).bind(average_balance).bind(prior_day_balance)
        .bind(projected_inflows).bind(projected_outflows).bind(projected_net)
        .bind(is_reconciled).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_cash_position(&row))
    }

    async fn get_cash_position(
        &self,
        org_id: Uuid,
        bank_account_id: Uuid,
        position_date: chrono::NaiveDate,
    ) -> AtlasResult<Option<CashPosition>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.cash_positions WHERE organization_id = $1 AND bank_account_id = $2 AND position_date = $3"
        )
        .bind(org_id).bind(bank_account_id).bind(position_date)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_cash_position(&r)))
    }

    async fn get_cash_position_by_id(&self, id: Uuid) -> AtlasResult<Option<CashPosition>> {
        let row = sqlx::query("SELECT * FROM _atlas.cash_positions WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_cash_position(&r)))
    }

    async fn list_cash_positions(
        &self,
        org_id: Uuid,
        position_date: Option<chrono::NaiveDate>,
    ) -> AtlasResult<Vec<CashPosition>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM _atlas.cash_positions
            WHERE organization_id = $1
              AND ($2::date IS NULL OR position_date = $2)
            ORDER BY position_date DESC, account_name
            "#,
        )
        .bind(org_id).bind(position_date)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_cash_position).collect())
    }

    // ========================================================================
    // Forecast Templates
    // ========================================================================

    async fn create_forecast_template(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        bucket_type: &str,
        number_of_periods: i32,
        start_offset_days: i32,
        is_default: bool,
        columns: serde_json::Value,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CashForecastTemplate> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.cash_forecast_templates
                (organization_id, code, name, description, bucket_type,
                 number_of_periods, start_offset_days, is_default, columns, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (organization_id, code) DO UPDATE
                SET name = $3, description = $4, bucket_type = $5,
                    number_of_periods = $6, start_offset_days = $7,
                    is_default = $8, columns = $9, is_active = true, updated_at = now()
            RETURNING *
            "#,
        )
        .bind(org_id).bind(code).bind(name).bind(description).bind(bucket_type)
        .bind(number_of_periods).bind(start_offset_days).bind(is_default)
        .bind(columns).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_template(&row))
    }

    async fn get_forecast_template(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<CashForecastTemplate>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.cash_forecast_templates WHERE organization_id = $1 AND code = $2 AND is_active = true"
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_template(&r)))
    }

    async fn get_forecast_template_by_id(&self, id: Uuid) -> AtlasResult<Option<CashForecastTemplate>> {
        let row = sqlx::query("SELECT * FROM _atlas.cash_forecast_templates WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_template(&r)))
    }

    async fn list_forecast_templates(&self, org_id: Uuid) -> AtlasResult<Vec<CashForecastTemplate>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.cash_forecast_templates WHERE organization_id = $1 AND is_active = true ORDER BY name"
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_template).collect())
    }

    async fn delete_forecast_template(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.cash_forecast_templates SET is_active = false, updated_at = now() WHERE organization_id = $1 AND code = $2"
        )
        .bind(org_id).bind(code)
        .execute(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Forecast Sources
    // ========================================================================

    async fn create_forecast_source(
        &self,
        org_id: Uuid,
        template_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        source_type: &str,
        cash_flow_direction: &str,
        is_actual: bool,
        display_order: i32,
        lead_time_days: i32,
        payment_terms_reference: Option<&str>,
        account_code_filter: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CashForecastSource> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.cash_forecast_sources
                (organization_id, template_id, code, name, description,
                 source_type, cash_flow_direction, is_actual, display_order,
                 lead_time_days, payment_terms_reference, account_code_filter, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            ON CONFLICT (template_id, code) DO UPDATE
                SET name = $4, description = $5, source_type = $6,
                    cash_flow_direction = $7, is_actual = $8, display_order = $9,
                    lead_time_days = $10, payment_terms_reference = $11,
                    account_code_filter = $12, is_active = true, updated_at = now()
            RETURNING *
            "#,
        )
        .bind(org_id).bind(template_id).bind(code).bind(name).bind(description)
        .bind(source_type).bind(cash_flow_direction).bind(is_actual).bind(display_order)
        .bind(lead_time_days).bind(payment_terms_reference).bind(account_code_filter)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_source(&row))
    }

    async fn get_forecast_source(&self, org_id: Uuid, template_id: Uuid, code: &str) -> AtlasResult<Option<CashForecastSource>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.cash_forecast_sources WHERE organization_id = $1 AND template_id = $2 AND code = $3 AND is_active = true"
        )
        .bind(org_id).bind(template_id).bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_source(&r)))
    }

    async fn list_forecast_sources(&self, template_id: Uuid) -> AtlasResult<Vec<CashForecastSource>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.cash_forecast_sources WHERE template_id = $1 AND is_active = true ORDER BY display_order, code"
        )
        .bind(template_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_source).collect())
    }

    async fn delete_forecast_source(&self, org_id: Uuid, template_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.cash_forecast_sources SET is_active = false, updated_at = now() WHERE organization_id = $1 AND template_id = $2 AND code = $3"
        )
        .bind(org_id).bind(template_id).bind(code)
        .execute(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Cash Forecasts
    // ========================================================================

    async fn create_forecast(
        &self,
        org_id: Uuid,
        forecast_number: &str,
        template_id: Uuid,
        template_name: &str,
        name: &str,
        description: Option<&str>,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
        opening_balance: &str,
        total_inflows: &str,
        total_outflows: &str,
        net_cash_flow: &str,
        closing_balance: &str,
        minimum_balance: &str,
        maximum_balance: &str,
        deficit_count: i32,
        surplus_count: i32,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CashForecast> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.cash_forecasts
                (organization_id, forecast_number, template_id, template_name,
                 name, description, start_date, end_date,
                 opening_balance, total_inflows, total_outflows, net_cash_flow,
                 closing_balance, minimum_balance, maximum_balance,
                 deficit_count, surplus_count, status, is_latest, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8,
                    $9::numeric, $10::numeric, $11::numeric, $12::numeric,
                    $13::numeric, $14::numeric, $15::numeric,
                    $16, $17, 'generated', true, $18)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(forecast_number).bind(template_id).bind(template_name)
        .bind(name).bind(description).bind(start_date).bind(end_date)
        .bind(opening_balance).bind(total_inflows).bind(total_outflows).bind(net_cash_flow)
        .bind(closing_balance).bind(minimum_balance).bind(maximum_balance)
        .bind(deficit_count).bind(surplus_count).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_forecast(&row))
    }

    async fn get_forecast(&self, id: Uuid) -> AtlasResult<Option<CashForecast>> {
        let row = sqlx::query("SELECT * FROM _atlas.cash_forecasts WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_forecast(&r)))
    }

    async fn get_forecast_by_number(&self, org_id: Uuid, forecast_number: &str) -> AtlasResult<Option<CashForecast>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.cash_forecasts WHERE organization_id = $1 AND forecast_number = $2"
        )
        .bind(org_id).bind(forecast_number)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_forecast(&r)))
    }

    async fn list_forecasts(
        &self,
        org_id: Uuid,
        template_id: Option<Uuid>,
        status: Option<&str>,
    ) -> AtlasResult<Vec<CashForecast>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM _atlas.cash_forecasts
            WHERE organization_id = $1
              AND ($2::uuid IS NULL OR template_id = $2)
              AND ($3::text IS NULL OR status = $3)
            ORDER BY created_at DESC
            "#,
        )
        .bind(org_id).bind(template_id).bind(status)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_forecast).collect())
    }

    async fn update_forecast_status(
        &self,
        id: Uuid,
        status: &str,
        approved_by: Option<Uuid>,
    ) -> AtlasResult<CashForecast> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.cash_forecasts
            SET status = $2,
                approved_by = CASE WHEN $2 = 'approved' THEN $3 ELSE approved_by END,
                approved_at = CASE WHEN $2 = 'approved' THEN now() ELSE approved_at END,
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(status).bind(approved_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_forecast(&row))
    }

    async fn supersede_previous_forecasts(&self, template_id: Uuid, new_forecast_id: Uuid) -> AtlasResult<()> {
        sqlx::query(
            r#"
            UPDATE _atlas.cash_forecasts
            SET is_latest = false, status = CASE WHEN status = 'generated' THEN 'superseded' ELSE status END,
                updated_at = now()
            WHERE template_id = $1 AND id != $2 AND is_latest = true
            "#,
        )
        .bind(template_id).bind(new_forecast_id)
        .execute(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Forecast Lines
    // ========================================================================

    async fn create_forecast_line(
        &self,
        org_id: Uuid,
        forecast_id: Uuid,
        source_id: Uuid,
        source_name: &str,
        source_type: &str,
        cash_flow_direction: &str,
        period_start_date: chrono::NaiveDate,
        period_end_date: chrono::NaiveDate,
        period_label: &str,
        period_sequence: i32,
        amount: &str,
        cumulative_amount: &str,
        is_actual: bool,
        currency_code: &str,
        transaction_count: i32,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CashForecastLine> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.cash_forecast_lines
                (organization_id, forecast_id, source_id, source_name, source_type,
                 cash_flow_direction, period_start_date, period_end_date,
                 period_label, period_sequence, amount, cumulative_amount,
                 is_actual, currency_code, transaction_count, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                    $11::numeric, $12::numeric, $13, $14, $15, $16)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(forecast_id).bind(source_id).bind(source_name).bind(source_type)
        .bind(cash_flow_direction).bind(period_start_date).bind(period_end_date)
        .bind(period_label).bind(period_sequence).bind(amount).bind(cumulative_amount)
        .bind(is_actual).bind(currency_code).bind(transaction_count).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_forecast_line(&row))
    }

    async fn list_forecast_lines(&self, forecast_id: Uuid) -> AtlasResult<Vec<CashForecastLine>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.cash_forecast_lines WHERE forecast_id = $1 ORDER BY period_sequence, display_order"
        )
        .bind(forecast_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_forecast_line).collect())
    }

    async fn list_forecast_lines_by_period(
        &self,
        forecast_id: Uuid,
        period_start_date: chrono::NaiveDate,
        period_end_date: chrono::NaiveDate,
    ) -> AtlasResult<Vec<CashForecastLine>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM _atlas.cash_forecast_lines
            WHERE forecast_id = $1
              AND period_start_date >= $2
              AND period_end_date <= $3
            ORDER BY period_sequence
            "#,
        )
        .bind(forecast_id).bind(period_start_date).bind(period_end_date)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_forecast_line).collect())
    }
}
