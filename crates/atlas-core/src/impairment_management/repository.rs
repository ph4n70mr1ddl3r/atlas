//! Impairment Management Repository
//!
//! Storage interface for impairment management data.

use atlas_shared::{
    ImpairmentIndicator, ImpairmentTest, ImpairmentCashFlow, ImpairmentTestAsset,
    ImpairmentDashboardSummary,
    AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

/// Repository trait for impairment management data storage
#[async_trait]
pub trait ImpairmentManagementRepository: Send + Sync {
    // Indicators
    async fn create_indicator(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        indicator_type: &str,
        severity: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ImpairmentIndicator>;

    async fn get_indicator(&self, id: Uuid) -> AtlasResult<Option<ImpairmentIndicator>>;
    async fn get_indicator_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<ImpairmentIndicator>>;
    async fn list_indicators(&self, org_id: Uuid, active_only: bool) -> AtlasResult<Vec<ImpairmentIndicator>>;

    // Tests
    async fn create_test(
        &self,
        org_id: Uuid,
        test_number: &str,
        name: &str,
        description: Option<&str>,
        test_type: &str,
        test_method: &str,
        test_date: chrono::NaiveDate,
        reporting_period: Option<&str>,
        indicator_id: Option<Uuid>,
        carrying_amount: &str,
        recoverable_amount: &str,
        impairment_loss: &str,
        impairment_account: Option<&str>,
        reversal_account: Option<&str>,
        asset_id: Option<Uuid>,
        cgu_id: Option<Uuid>,
        discount_rate: Option<&str>,
        growth_rate: Option<&str>,
        terminal_value: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ImpairmentTest>;

    async fn get_test(&self, id: Uuid) -> AtlasResult<Option<ImpairmentTest>>;
    async fn list_tests(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<ImpairmentTest>>;
    async fn update_test_status(
        &self,
        id: Uuid,
        status: &str,
        submitted_by: Option<Uuid>,
        approved_by: Option<Uuid>,
    ) -> AtlasResult<ImpairmentTest>;
    async fn update_test_recoverable(&self, id: Uuid, recoverable_amount: &str) -> AtlasResult<()>;
    async fn update_test_results(&self, id: Uuid, recoverable_amount: &str, impairment_loss: &str) -> AtlasResult<ImpairmentTest>;

    // Cash Flows
    async fn create_cash_flow(
        &self,
        org_id: Uuid,
        test_id: Uuid,
        period_year: i32,
        period_number: i32,
        description: Option<&str>,
        cash_inflow: &str,
        cash_outflow: &str,
        net_cash_flow: &str,
        discount_factor: &str,
        present_value: &str,
    ) -> AtlasResult<ImpairmentCashFlow>;

    async fn list_cash_flows(&self, test_id: Uuid) -> AtlasResult<Vec<ImpairmentCashFlow>>;

    // Test Assets
    async fn create_test_asset(
        &self,
        org_id: Uuid,
        test_id: Uuid,
        asset_id: Uuid,
        asset_number: Option<&str>,
        asset_name: Option<&str>,
        asset_category: Option<&str>,
        carrying_amount: &str,
        recoverable_amount: &str,
        impairment_loss: &str,
        status: &str,
        impairment_date: Option<chrono::NaiveDate>,
    ) -> AtlasResult<ImpairmentTestAsset>;

    async fn list_test_assets(&self, test_id: Uuid) -> AtlasResult<Vec<ImpairmentTestAsset>>;
    async fn update_test_asset(
        &self,
        id: Uuid,
        recoverable_amount: &str,
        impairment_loss: &str,
        status: &str,
    ) -> AtlasResult<ImpairmentTestAsset>;

    // Dashboard
    async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<ImpairmentDashboardSummary>;
}

/// PostgreSQL implementation
pub struct PostgresImpairmentManagementRepository {
    pool: PgPool,
}

impl PostgresImpairmentManagementRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

use sqlx::Row;

fn row_to_indicator(row: &sqlx::postgres::PgRow) -> ImpairmentIndicator {
    ImpairmentIndicator {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        code: row.get("code"),
        name: row.get("name"),
        description: row.get("description"),
        indicator_type: row.get("indicator_type"),
        severity: row.get("severity"),
        is_active: row.get("is_active"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_test(row: &sqlx::postgres::PgRow) -> ImpairmentTest {
    ImpairmentTest {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        test_number: row.get("test_number"),
        name: row.get("name"),
        description: row.get("description"),
        test_type: row.get("test_type"),
        test_method: row.get("test_method"),
        test_date: row.get("test_date"),
        reporting_period: row.get("reporting_period"),
        indicator_id: row.get("indicator_id"),
        carrying_amount: row.try_get("carrying_amount").unwrap_or("0".to_string()),
        recoverable_amount: row.try_get("recoverable_amount").unwrap_or("0".to_string()),
        impairment_loss: row.try_get("impairment_loss").unwrap_or("0".to_string()),
        reversal_amount: row.try_get("reversal_amount").unwrap_or(None),
        status: row.get("status"),
        impairment_account: row.get("impairment_account"),
        reversal_account: row.get("reversal_account"),
        asset_id: row.get("asset_id"),
        cgu_id: row.get("cgu_id"),
        discount_rate: row.try_get("discount_rate").unwrap_or(None),
        growth_rate: row.try_get("growth_rate").unwrap_or(None),
        terminal_value: row.try_get("terminal_value").unwrap_or(None),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        submitted_by: row.get("submitted_by"),
        submitted_at: row.get("submitted_at"),
        approved_by: row.get("approved_by"),
        approved_at: row.get("approved_at"),
        completed_at: row.get("completed_at"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_cash_flow(row: &sqlx::postgres::PgRow) -> ImpairmentCashFlow {
    ImpairmentCashFlow {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        test_id: row.get("test_id"),
        period_year: row.get("period_year"),
        period_number: row.get("period_number"),
        description: row.get("description"),
        cash_inflow: row.try_get("cash_inflow").unwrap_or("0".to_string()),
        cash_outflow: row.try_get("cash_outflow").unwrap_or("0".to_string()),
        net_cash_flow: row.try_get("net_cash_flow").unwrap_or("0".to_string()),
        discount_factor: row.try_get("discount_factor").unwrap_or("1".to_string()),
        present_value: row.try_get("present_value").unwrap_or("0".to_string()),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_test_asset(row: &sqlx::postgres::PgRow) -> ImpairmentTestAsset {
    ImpairmentTestAsset {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        test_id: row.get("test_id"),
        asset_id: row.get("asset_id"),
        asset_number: row.get("asset_number"),
        asset_name: row.get("asset_name"),
        asset_category: row.get("asset_category"),
        carrying_amount: row.try_get("carrying_amount").unwrap_or("0".to_string()),
        recoverable_amount: row.try_get("recoverable_amount").unwrap_or("0".to_string()),
        impairment_loss: row.try_get("impairment_loss").unwrap_or("0".to_string()),
        status: row.get("status"),
        impairment_date: row.get("impairment_date"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

#[async_trait]
impl ImpairmentManagementRepository for PostgresImpairmentManagementRepository {
    async fn create_indicator(
        &self,
        org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        indicator_type: &str, severity: &str, created_by: Option<Uuid>,
    ) -> AtlasResult<ImpairmentIndicator> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.impairment_indicators
                (organization_id, code, name, description, indicator_type, severity, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(code).bind(name).bind(description)
        .bind(indicator_type).bind(severity).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_indicator(&row))
    }

    async fn get_indicator(&self, id: Uuid) -> AtlasResult<Option<ImpairmentIndicator>> {
        let row = sqlx::query("SELECT * FROM _atlas.impairment_indicators WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_indicator(&r)))
    }

    async fn get_indicator_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<ImpairmentIndicator>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.impairment_indicators WHERE organization_id = $1 AND code = $2"
        ).bind(org_id).bind(code).fetch_optional(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_indicator(&r)))
    }

    async fn list_indicators(&self, org_id: Uuid, active_only: bool) -> AtlasResult<Vec<ImpairmentIndicator>> {
        let rows = if active_only {
            sqlx::query("SELECT * FROM _atlas.impairment_indicators WHERE organization_id = $1 AND is_active = true ORDER BY created_at DESC")
                .bind(org_id)
        } else {
            sqlx::query("SELECT * FROM _atlas.impairment_indicators WHERE organization_id = $1 ORDER BY created_at DESC")
                .bind(org_id)
        }
        .fetch_all(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_indicator).collect())
    }

    async fn create_test(
        &self,
        org_id: Uuid, test_number: &str, name: &str, description: Option<&str>,
        test_type: &str, test_method: &str, test_date: chrono::NaiveDate,
        reporting_period: Option<&str>, indicator_id: Option<Uuid>,
        carrying_amount: &str, recoverable_amount: &str, impairment_loss: &str,
        impairment_account: Option<&str>, reversal_account: Option<&str>,
        asset_id: Option<Uuid>, cgu_id: Option<Uuid>,
        discount_rate: Option<&str>, growth_rate: Option<&str>,
        terminal_value: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<ImpairmentTest> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.impairment_tests
                (organization_id, test_number, name, description, test_type, test_method,
                 test_date, reporting_period, indicator_id, carrying_amount, recoverable_amount,
                 impairment_loss, impairment_account, reversal_account, asset_id, cgu_id,
                 discount_rate, growth_rate, terminal_value, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9,
                    $10::decimal, $11::decimal, $12::decimal, $13, $14, $15, $16,
                    $17::decimal, $18::decimal, $19::decimal, $20)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(test_number).bind(name).bind(description)
        .bind(test_type).bind(test_method).bind(test_date).bind(reporting_period)
        .bind(indicator_id).bind(carrying_amount).bind(recoverable_amount)
        .bind(impairment_loss).bind(impairment_account).bind(reversal_account)
        .bind(asset_id).bind(cgu_id).bind(discount_rate).bind(growth_rate)
        .bind(terminal_value).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_test(&row))
    }

    async fn get_test(&self, id: Uuid) -> AtlasResult<Option<ImpairmentTest>> {
        let row = sqlx::query("SELECT * FROM _atlas.impairment_tests WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_test(&r)))
    }

    async fn list_tests(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<ImpairmentTest>> {
        let rows = sqlx::query(
            r#"SELECT * FROM _atlas.impairment_tests
               WHERE organization_id = $1 AND ($2::text IS NULL OR status = $2)
               ORDER BY test_date DESC, created_at DESC"#,
        ).bind(org_id).bind(status)
        .fetch_all(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_test).collect())
    }

    async fn update_test_status(
        &self, id: Uuid, status: &str,
        submitted_by: Option<Uuid>, approved_by: Option<Uuid>,
    ) -> AtlasResult<ImpairmentTest> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.impairment_tests
            SET status = $2,
                submitted_by = COALESCE($3, submitted_by),
                submitted_at = CASE WHEN $2 = 'submitted' THEN now() ELSE submitted_at END,
                approved_by = COALESCE($4, approved_by),
                approved_at = CASE WHEN $2 = 'approved' THEN now() ELSE approved_at END,
                completed_at = CASE WHEN $2 = 'completed' THEN now() ELSE completed_at END,
                updated_at = now()
            WHERE id = $1 RETURNING *
            "#,
        ).bind(id).bind(status).bind(submitted_by).bind(approved_by)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_test(&row))
    }

    async fn update_test_recoverable(&self, id: Uuid, recoverable_amount: &str) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.impairment_tests SET recoverable_amount = $2::decimal, updated_at = now() WHERE id = $1"
        ).bind(id).bind(recoverable_amount)
        .execute(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn update_test_results(&self, id: Uuid, recoverable_amount: &str, impairment_loss: &str) -> AtlasResult<ImpairmentTest> {
        let row = sqlx::query(
            r#"UPDATE _atlas.impairment_tests
               SET recoverable_amount = $2::decimal, impairment_loss = $3::decimal, updated_at = now()
               WHERE id = $1 RETURNING *"#,
        ).bind(id).bind(recoverable_amount).bind(impairment_loss)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_test(&row))
    }

    async fn create_cash_flow(
        &self,
        org_id: Uuid, test_id: Uuid, period_year: i32, period_number: i32,
        description: Option<&str>, cash_inflow: &str, cash_outflow: &str,
        net_cash_flow: &str, discount_factor: &str, present_value: &str,
    ) -> AtlasResult<ImpairmentCashFlow> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.impairment_cash_flows
                (organization_id, test_id, period_year, period_number, description,
                 cash_inflow, cash_outflow, net_cash_flow, discount_factor, present_value)
            VALUES ($1, $2, $3, $4, $5, $6::decimal, $7::decimal, $8::decimal, $9::decimal, $10::decimal)
            RETURNING *
            "#,
        ).bind(org_id).bind(test_id).bind(period_year).bind(period_number)
        .bind(description).bind(cash_inflow).bind(cash_outflow)
        .bind(net_cash_flow).bind(discount_factor).bind(present_value)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_cash_flow(&row))
    }

    async fn list_cash_flows(&self, test_id: Uuid) -> AtlasResult<Vec<ImpairmentCashFlow>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.impairment_cash_flows WHERE test_id = $1 ORDER BY period_year, period_number"
        ).bind(test_id).fetch_all(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_cash_flow).collect())
    }

    async fn create_test_asset(
        &self,
        org_id: Uuid, test_id: Uuid, asset_id: Uuid,
        asset_number: Option<&str>, asset_name: Option<&str>,
        asset_category: Option<&str>, carrying_amount: &str,
        recoverable_amount: &str, impairment_loss: &str,
        status: &str, impairment_date: Option<chrono::NaiveDate>,
    ) -> AtlasResult<ImpairmentTestAsset> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.impairment_test_assets
                (organization_id, test_id, asset_id, asset_number, asset_name, asset_category,
                 carrying_amount, recoverable_amount, impairment_loss, status, impairment_date)
            VALUES ($1, $2, $3, $4, $5, $6, $7::decimal, $8::decimal, $9::decimal, $10, $11)
            RETURNING *
            "#,
        ).bind(org_id).bind(test_id).bind(asset_id).bind(asset_number)
        .bind(asset_name).bind(asset_category).bind(carrying_amount)
        .bind(recoverable_amount).bind(impairment_loss).bind(status).bind(impairment_date)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_test_asset(&row))
    }

    async fn list_test_assets(&self, test_id: Uuid) -> AtlasResult<Vec<ImpairmentTestAsset>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.impairment_test_assets WHERE test_id = $1"
        ).bind(test_id).fetch_all(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_test_asset).collect())
    }

    async fn update_test_asset(
        &self, id: Uuid, recoverable_amount: &str, impairment_loss: &str, status: &str,
    ) -> AtlasResult<ImpairmentTestAsset> {
        let row = sqlx::query(
            r#"UPDATE _atlas.impairment_test_assets
               SET recoverable_amount = $2::decimal, impairment_loss = $3::decimal,
                   status = $4, updated_at = now()
               WHERE id = $1 RETURNING *"#,
        ).bind(id).bind(recoverable_amount).bind(impairment_loss).bind(status)
        .fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_test_asset(&row))
    }

    async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<ImpairmentDashboardSummary> {
        let ind_row = sqlx::query(
            r#"SELECT
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE is_active) as active
            FROM _atlas.impairment_indicators WHERE organization_id = $1"#,
        ).bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        let test_row = sqlx::query(
            r#"SELECT
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE status = 'draft') as pending,
                COUNT(*) FILTER (WHERE status = 'completed') as completed,
                COALESCE(SUM(impairment_loss), 0) as total_loss,
                COALESCE(SUM(COALESCE(reversal_amount::decimal, 0)), 0) as total_reversals,
                COUNT(DISTINCT asset_id) FILTER (WHERE status IN ('draft','submitted')) as under_review
            FROM _atlas.impairment_tests WHERE organization_id = $1"#,
        ).bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;

        Ok(ImpairmentDashboardSummary {
            total_indicators: ind_row.try_get::<i64, _>("total").unwrap_or(0) as i32,
            active_indicators: ind_row.try_get::<i64, _>("active").unwrap_or(0) as i32,
            total_tests: test_row.try_get::<i64, _>("total").unwrap_or(0) as i32,
            pending_tests: test_row.try_get::<i64, _>("pending").unwrap_or(0) as i32,
            completed_tests: test_row.try_get::<i64, _>("completed").unwrap_or(0) as i32,
            total_impairment_loss: format!("{:.2}", test_row.try_get::<f64, _>("total_loss").unwrap_or(0.0)),
            total_reversals: format!("{:.2}", test_row.try_get::<f64, _>("total_reversals").unwrap_or(0.0)),
            assets_under_review: test_row.try_get::<i64, _>("under_review").unwrap_or(0) as i32,
        })
    }
}
