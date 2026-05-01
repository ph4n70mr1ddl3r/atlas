//! Asset Depreciation Repository
//!
//! PostgreSQL storage for depreciation history and asset updates.

use atlas_shared::{AssetDepreciationHistory, FixedAsset, AtlasError, AtlasResult};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

/// Repository trait for asset depreciation data storage
#[async_trait]
pub trait AssetDepreciationRepository: Send + Sync {
    async fn get_asset(&self, id: Uuid) -> AtlasResult<Option<FixedAsset>>;
    async fn create_depreciation_history(
        &self,
        org_id: Uuid,
        asset_id: Uuid,
        fiscal_year: i32,
        period_number: i32,
        period_name: Option<&str>,
        depreciation_date: chrono::NaiveDate,
        depreciation_amount: &str,
        accumulated_depreciation: &str,
        net_book_value: &str,
        depreciation_method: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AssetDepreciationHistory>;
    async fn list_depreciation_history(&self, asset_id: Uuid) -> AtlasResult<Vec<AssetDepreciationHistory>>;
    async fn update_asset_depreciation(
        &self,
        asset_id: Uuid,
        accumulated_depreciation: &str,
        net_book_value: &str,
        last_depreciation_amount: &str,
        periods_depreciated: i32,
        last_depreciation_date: chrono::NaiveDate,
    ) -> AtlasResult<()>;
}

/// PostgreSQL implementation
pub struct PostgresAssetDepreciationRepository {
    pool: PgPool,
}

impl PostgresAssetDepreciationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

use sqlx::Row;

fn get_num(row: &sqlx::postgres::PgRow, col: &str) -> String {
    let v: f64 = row.try_get(col).unwrap_or(0.0);
    format!("{:.2}", v)
}

fn row_to_depreciation_history(row: &sqlx::postgres::PgRow) -> AssetDepreciationHistory {
    AssetDepreciationHistory {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        asset_id: row.get("asset_id"),
        fiscal_year: row.get("fiscal_year"),
        period_number: row.get("period_number"),
        period_name: row.get("period_name"),
        depreciation_date: row.get("depreciation_date"),
        depreciation_amount: get_num(row, "depreciation_amount"),
        accumulated_depreciation: get_num(row, "accumulated_depreciation"),
        net_book_value: get_num(row, "net_book_value"),
        depreciation_method: row.get("depreciation_method"),
        journal_entry_id: row.get("journal_entry_id"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
    }
}

#[async_trait]
impl AssetDepreciationRepository for PostgresAssetDepreciationRepository {
    async fn get_asset(&self, id: Uuid) -> AtlasResult<Option<FixedAsset>> {
        // Delegate to the fixed_assets table
        let row = sqlx::query("SELECT * FROM _atlas.fixed_assets WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        // We need a simpler approach - just return a FixedAsset with basic fields
        // The full row mapping is in the fixed_assets repository; here we map minimally
        Ok(row.map(|r| FixedAsset {
            id: r.get("id"),
            organization_id: r.get("organization_id"),
            asset_number: r.get("asset_number"),
            asset_name: r.get("asset_name"),
            description: r.get("description"),
            category_id: r.get("category_id"),
            category_code: r.get("category_code"),
            book_id: r.get("book_id"),
            book_code: r.get("book_code"),
            asset_type: r.get("asset_type"),
            status: r.get("status"),
            original_cost: get_num(&r, "original_cost"),
            current_cost: get_num(&r, "current_cost"),
            salvage_value: get_num(&r, "salvage_value"),
            salvage_value_percent: r.try_get::<Option<String>, _>("salvage_value_percent").ok().flatten().unwrap_or_else(|| "0".to_string()),
            depreciation_method: r.get("depreciation_method"),
            useful_life_months: r.get("useful_life_months"),
            declining_balance_rate: r.try_get::<Option<String>, _>("declining_balance_rate").ok().flatten(),
            depreciable_basis: get_num(&r, "depreciable_basis"),
            accumulated_depreciation: get_num(&r, "accumulated_depreciation"),
            net_book_value: get_num(&r, "net_book_value"),
            depreciation_per_period: get_num(&r, "depreciation_per_period"),
            periods_depreciated: r.get("periods_depreciated"),
            last_depreciation_date: r.get("last_depreciation_date"),
            last_depreciation_amount: get_num(&r, "last_depreciation_amount"),
            acquisition_date: r.get("acquisition_date"),
            in_service_date: r.get("in_service_date"),
            disposal_date: r.get("disposal_date"),
            retirement_date: r.get("retirement_date"),
            location: r.get("location"),
            department_id: r.get("department_id"),
            department_name: r.get("department_name"),
            custodian_id: r.get("custodian_id"),
            custodian_name: r.get("custodian_name"),
            serial_number: r.get("serial_number"),
            tag_number: r.get("tag_number"),
            manufacturer: r.get("manufacturer"),
            model: r.get("model"),
            warranty_expiry: r.get("warranty_expiry"),
            insurance_policy_number: r.get("insurance_policy_number"),
            insurance_expiry: r.get("insurance_expiry"),
            lease_number: r.get("lease_number"),
            lease_expiry: r.get("lease_expiry"),
            asset_account_code: r.get("asset_account_code"),
            accum_depr_account_code: r.get("accum_depr_account_code"),
            depr_expense_account_code: r.get("depr_expense_account_code"),
            gain_loss_account_code: r.get("gain_loss_account_code"),
            metadata: r.try_get("metadata").unwrap_or(serde_json::json!({})),
            created_by: r.get("created_by"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }))
    }

    async fn create_depreciation_history(
        &self,
        org_id: Uuid,
        asset_id: Uuid,
        fiscal_year: i32,
        period_number: i32,
        period_name: Option<&str>,
        depreciation_date: chrono::NaiveDate,
        depreciation_amount: &str,
        accumulated_depreciation: &str,
        net_book_value: &str,
        depreciation_method: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AssetDepreciationHistory> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.asset_depreciation_history
                (organization_id, asset_id, fiscal_year, period_number,
                 period_name, depreciation_date, depreciation_amount,
                 accumulated_depreciation, net_book_value,
                 depreciation_method, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7::double precision,
                    $8::double precision, $9::double precision, $10, $11)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(asset_id).bind(fiscal_year).bind(period_number)
        .bind(period_name).bind(depreciation_date).bind(depreciation_amount)
        .bind(accumulated_depreciation).bind(net_book_value)
        .bind(depreciation_method).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(row_to_depreciation_history(&row))
    }

    async fn list_depreciation_history(&self, asset_id: Uuid) -> AtlasResult<Vec<AssetDepreciationHistory>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.asset_depreciation_history WHERE asset_id = $1 ORDER BY fiscal_year, period_number"
        )
        .bind(asset_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_depreciation_history).collect())
    }

    async fn update_asset_depreciation(
        &self,
        asset_id: Uuid,
        accumulated_depreciation: &str,
        net_book_value: &str,
        last_depreciation_amount: &str,
        periods_depreciated: i32,
        last_depreciation_date: chrono::NaiveDate,
    ) -> AtlasResult<()> {
        sqlx::query(
            r#"
            UPDATE _atlas.fixed_assets
            SET accumulated_depreciation = $2::double precision,
                net_book_value = $3::double precision,
                last_depreciation_amount = $4::double precision,
                periods_depreciated = $5,
                last_depreciation_date = $6,
                updated_at = now()
            WHERE id = $1
            "#,
        )
        .bind(asset_id).bind(accumulated_depreciation).bind(net_book_value)
        .bind(last_depreciation_amount).bind(periods_depreciated).bind(last_depreciation_date)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}
