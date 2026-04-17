//! Fixed Asset Repository
//!
//! PostgreSQL storage for asset categories, books, fixed assets,
//! depreciation history, transfers, and retirements.

use atlas_shared::{
    AssetCategory, AssetBook, FixedAsset, AssetDepreciationHistory,
    AssetTransfer, AssetRetirement,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

/// Repository trait for fixed asset data storage
#[async_trait]
pub trait FixedAssetRepository: Send + Sync {
    // Asset Categories
    async fn create_category(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        default_depreciation_method: &str,
        default_useful_life_months: i32,
        default_salvage_value_percent: &str,
        default_asset_account_code: Option<&str>,
        default_accum_depr_account_code: Option<&str>,
        default_depr_expense_account_code: Option<&str>,
        default_gain_loss_account_code: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AssetCategory>;

    async fn get_category(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<AssetCategory>>;
    async fn get_category_by_id(&self, id: Uuid) -> AtlasResult<Option<AssetCategory>>;
    async fn list_categories(&self, org_id: Uuid) -> AtlasResult<Vec<AssetCategory>>;
    async fn delete_category(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // Asset Books
    async fn create_book(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        book_type: &str,
        auto_depreciation: bool,
        depreciation_calendar: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AssetBook>;

    async fn get_book(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<AssetBook>>;
    async fn get_book_by_id(&self, id: Uuid) -> AtlasResult<Option<AssetBook>>;
    async fn list_books(&self, org_id: Uuid) -> AtlasResult<Vec<AssetBook>>;
    async fn delete_book(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // Fixed Assets
    async fn create_asset(
        &self,
        org_id: Uuid,
        asset_number: &str,
        asset_name: &str,
        description: Option<&str>,
        category_id: Option<Uuid>,
        category_code: Option<&str>,
        book_id: Option<Uuid>,
        book_code: Option<&str>,
        asset_type: &str,
        original_cost: &str,
        salvage_value: &str,
        salvage_value_percent: &str,
        depreciation_method: &str,
        useful_life_months: i32,
        declining_balance_rate: Option<&str>,
        acquisition_date: Option<chrono::NaiveDate>,
        location: Option<&str>,
        department_id: Option<Uuid>,
        department_name: Option<&str>,
        custodian_id: Option<Uuid>,
        custodian_name: Option<&str>,
        serial_number: Option<&str>,
        tag_number: Option<&str>,
        manufacturer: Option<&str>,
        model: Option<&str>,
        warranty_expiry: Option<chrono::NaiveDate>,
        insurance_policy_number: Option<&str>,
        insurance_expiry: Option<chrono::NaiveDate>,
        lease_number: Option<&str>,
        lease_expiry: Option<chrono::NaiveDate>,
        asset_account_code: Option<&str>,
        accum_depr_account_code: Option<&str>,
        depr_expense_account_code: Option<&str>,
        gain_loss_account_code: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<FixedAsset>;

    async fn get_asset(&self, id: Uuid) -> AtlasResult<Option<FixedAsset>>;
    async fn get_asset_by_number(&self, org_id: Uuid, asset_number: &str) -> AtlasResult<Option<FixedAsset>>;
    async fn list_assets(&self, org_id: Uuid, status: Option<&str>, category_code: Option<&str>, book_code: Option<&str>) -> AtlasResult<Vec<FixedAsset>>;
    async fn update_asset_status(&self, id: Uuid, status: &str, in_service_date: Option<chrono::NaiveDate>, disposal_date: Option<chrono::NaiveDate>, retirement_date: Option<chrono::NaiveDate>) -> AtlasResult<FixedAsset>;
    async fn update_asset_depreciation(&self, id: Uuid, accumulated_depreciation: &str, net_book_value: &str, periods_depreciated: i32, last_depreciation_date: Option<chrono::NaiveDate>, last_depreciation_amount: &str) -> AtlasResult<FixedAsset>;
    async fn update_asset_assignment(&self, id: Uuid, department_id: Option<Uuid>, department_name: Option<&str>, location: Option<&str>, custodian_id: Option<Uuid>, custodian_name: Option<&str>) -> AtlasResult<FixedAsset>;
    async fn delete_asset(&self, org_id: Uuid, asset_number: &str) -> AtlasResult<()>;

    // Depreciation History
    async fn create_depreciation_entry(
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
    async fn get_depreciation_for_period(&self, asset_id: Uuid, fiscal_year: i32, period_number: i32) -> AtlasResult<Option<AssetDepreciationHistory>>;

    // Asset Transfers
    async fn create_transfer(
        &self,
        org_id: Uuid,
        transfer_number: &str,
        asset_id: Uuid,
        from_department_id: Option<Uuid>,
        from_department_name: Option<&str>,
        from_location: Option<&str>,
        from_custodian_id: Option<Uuid>,
        from_custodian_name: Option<&str>,
        to_department_id: Option<Uuid>,
        to_department_name: Option<&str>,
        to_location: Option<&str>,
        to_custodian_id: Option<Uuid>,
        to_custodian_name: Option<&str>,
        transfer_date: chrono::NaiveDate,
        reason: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AssetTransfer>;

    async fn get_transfer(&self, id: Uuid) -> AtlasResult<Option<AssetTransfer>>;
    async fn list_transfers(&self, org_id: Uuid, asset_id: Option<Uuid>) -> AtlasResult<Vec<AssetTransfer>>;
    async fn update_transfer_status(&self, id: Uuid, status: &str, approved_by: Option<Uuid>, rejected_reason: Option<&str>) -> AtlasResult<AssetTransfer>;

    // Asset Retirements
    async fn create_retirement(
        &self,
        org_id: Uuid,
        retirement_number: &str,
        asset_id: Uuid,
        retirement_type: &str,
        retirement_date: chrono::NaiveDate,
        proceeds: &str,
        removal_cost: &str,
        net_book_value_at_retirement: &str,
        accumulated_depreciation_at_retirement: &str,
        gain_loss_amount: &str,
        gain_loss_type: Option<&str>,
        gain_account_code: Option<&str>,
        loss_account_code: Option<&str>,
        cash_account_code: Option<&str>,
        asset_account_code: Option<&str>,
        accum_depr_account_code: Option<&str>,
        reference_number: Option<&str>,
        buyer_name: Option<&str>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AssetRetirement>;

    async fn get_retirement(&self, id: Uuid) -> AtlasResult<Option<AssetRetirement>>;
    async fn list_retirements(&self, org_id: Uuid, asset_id: Option<Uuid>) -> AtlasResult<Vec<AssetRetirement>>;
    async fn update_retirement_status(&self, id: Uuid, status: &str, approved_by: Option<Uuid>) -> AtlasResult<AssetRetirement>;
}

/// PostgreSQL implementation
pub struct PostgresFixedAssetRepository {
    pool: PgPool,
}

impl PostgresFixedAssetRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn row_to_category(&self, row: &sqlx::postgres::PgRow) -> AssetCategory {
        let salvage_pct: serde_json::Value = row.try_get("default_salvage_value_percent").unwrap_or(serde_json::json!("0"));
        AssetCategory {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            code: row.get("code"),
            name: row.get("name"),
            description: row.get("description"),
            default_depreciation_method: row.get("default_depreciation_method"),
            default_useful_life_months: row.get("default_useful_life_months"),
            default_salvage_value_percent: salvage_pct.to_string(),
            default_asset_account_code: row.get("default_asset_account_code"),
            default_accum_depr_account_code: row.get("default_accum_depr_account_code"),
            default_depr_expense_account_code: row.get("default_depr_expense_account_code"),
            default_gain_loss_account_code: row.get("default_gain_loss_account_code"),
            is_active: row.get("is_active"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_book(&self, row: &sqlx::postgres::PgRow) -> AssetBook {
        AssetBook {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            code: row.get("code"),
            name: row.get("name"),
            description: row.get("description"),
            book_type: row.get("book_type"),
            auto_depreciation: row.get("auto_depreciation"),
            depreciation_calendar: row.get("depreciation_calendar"),
            current_fiscal_year: row.get("current_fiscal_year"),
            last_depreciation_date: row.get("last_depreciation_date"),
            is_active: row.get("is_active"),
            metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_asset(&self, row: &sqlx::postgres::PgRow) -> FixedAsset {
        fn get_num(row: &sqlx::postgres::PgRow, col: &str) -> String {
            let v: serde_json::Value = row.try_get(col).unwrap_or(serde_json::json!("0"));
            v.to_string()
        }
        FixedAsset {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            asset_number: row.get("asset_number"),
            asset_name: row.get("asset_name"),
            description: row.get("description"),
            category_id: row.get("category_id"),
            category_code: row.get("category_code"),
            book_id: row.get("book_id"),
            book_code: row.get("book_code"),
            asset_type: row.get("asset_type"),
            status: row.get("status"),
            original_cost: get_num(row, "original_cost"),
            current_cost: get_num(row, "current_cost"),
            salvage_value: get_num(row, "salvage_value"),
            salvage_value_percent: get_num(row, "salvage_value_percent"),
            depreciation_method: row.get("depreciation_method"),
            useful_life_months: row.get("useful_life_months"),
            declining_balance_rate: row.try_get("declining_balance_rate").unwrap_or(None),
            depreciable_basis: get_num(row, "depreciable_basis"),
            accumulated_depreciation: get_num(row, "accumulated_depreciation"),
            net_book_value: get_num(row, "net_book_value"),
            depreciation_per_period: get_num(row, "depreciation_per_period"),
            periods_depreciated: row.get("periods_depreciated"),
            last_depreciation_date: row.get("last_depreciation_date"),
            last_depreciation_amount: get_num(row, "last_depreciation_amount"),
            acquisition_date: row.get("acquisition_date"),
            in_service_date: row.get("in_service_date"),
            disposal_date: row.get("disposal_date"),
            retirement_date: row.get("retirement_date"),
            location: row.get("location"),
            department_id: row.get("department_id"),
            department_name: row.get("department_name"),
            custodian_id: row.get("custodian_id"),
            custodian_name: row.get("custodian_name"),
            serial_number: row.get("serial_number"),
            tag_number: row.get("tag_number"),
            manufacturer: row.get("manufacturer"),
            model: row.get("model"),
            warranty_expiry: row.get("warranty_expiry"),
            insurance_policy_number: row.get("insurance_policy_number"),
            insurance_expiry: row.get("insurance_expiry"),
            lease_number: row.get("lease_number"),
            lease_expiry: row.get("lease_expiry"),
            asset_account_code: row.get("asset_account_code"),
            accum_depr_account_code: row.get("accum_depr_account_code"),
            depr_expense_account_code: row.get("depr_expense_account_code"),
            gain_loss_account_code: row.get("gain_loss_account_code"),
            metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_depr_history(&self, row: &sqlx::postgres::PgRow) -> AssetDepreciationHistory {
        fn get_num(row: &sqlx::postgres::PgRow, col: &str) -> String {
            let v: serde_json::Value = row.try_get(col).unwrap_or(serde_json::json!("0"));
            v.to_string()
        }
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

    fn row_to_transfer(&self, row: &sqlx::postgres::PgRow) -> AssetTransfer {
        AssetTransfer {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            transfer_number: row.get("transfer_number"),
            asset_id: row.get("asset_id"),
            from_department_id: row.get("from_department_id"),
            from_department_name: row.get("from_department_name"),
            from_location: row.get("from_location"),
            from_custodian_id: row.get("from_custodian_id"),
            from_custodian_name: row.get("from_custodian_name"),
            to_department_id: row.get("to_department_id"),
            to_department_name: row.get("to_department_name"),
            to_location: row.get("to_location"),
            to_custodian_id: row.get("to_custodian_id"),
            to_custodian_name: row.get("to_custodian_name"),
            transfer_date: row.get("transfer_date"),
            reason: row.get("reason"),
            status: row.get("status"),
            approved_by: row.get("approved_by"),
            approved_at: row.get("approved_at"),
            rejected_reason: row.get("rejected_reason"),
            metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_retirement(&self, row: &sqlx::postgres::PgRow) -> AssetRetirement {
        fn get_num(row: &sqlx::postgres::PgRow, col: &str) -> String {
            let v: serde_json::Value = row.try_get(col).unwrap_or(serde_json::json!("0"));
            v.to_string()
        }
        AssetRetirement {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            retirement_number: row.get("retirement_number"),
            asset_id: row.get("asset_id"),
            retirement_type: row.get("retirement_type"),
            retirement_date: row.get("retirement_date"),
            proceeds: get_num(row, "proceeds"),
            removal_cost: get_num(row, "removal_cost"),
            net_book_value_at_retirement: get_num(row, "net_book_value_at_retirement"),
            accumulated_depreciation_at_retirement: get_num(row, "accumulated_depreciation_at_retirement"),
            gain_loss_amount: get_num(row, "gain_loss_amount"),
            gain_loss_type: row.get("gain_loss_type"),
            gain_account_code: row.get("gain_account_code"),
            loss_account_code: row.get("loss_account_code"),
            cash_account_code: row.get("cash_account_code"),
            asset_account_code: row.get("asset_account_code"),
            accum_depr_account_code: row.get("accum_depr_account_code"),
            reference_number: row.get("reference_number"),
            buyer_name: row.get("buyer_name"),
            notes: row.get("notes"),
            status: row.get("status"),
            approved_by: row.get("approved_by"),
            approved_at: row.get("approved_at"),
            journal_entry_id: row.get("journal_entry_id"),
            metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}

#[async_trait]
impl FixedAssetRepository for PostgresFixedAssetRepository {
    // ========================================================================
    // Asset Categories
    // ========================================================================

    async fn create_category(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        default_depreciation_method: &str,
        default_useful_life_months: i32,
        default_salvage_value_percent: &str,
        default_asset_account_code: Option<&str>,
        default_accum_depr_account_code: Option<&str>,
        default_depr_expense_account_code: Option<&str>,
        default_gain_loss_account_code: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AssetCategory> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.asset_categories
                (organization_id, code, name, description,
                 default_depreciation_method, default_useful_life_months,
                 default_salvage_value_percent,
                 default_asset_account_code, default_accum_depr_account_code,
                 default_depr_expense_account_code, default_gain_loss_account_code,
                 created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7::numeric, $8, $9, $10, $11, $12)
            ON CONFLICT (organization_id, code) DO UPDATE
                SET name = $3, description = $4,
                    default_depreciation_method = $5, default_useful_life_months = $6,
                    default_salvage_value_percent = $7::numeric,
                    default_asset_account_code = $8, default_accum_depr_account_code = $9,
                    default_depr_expense_account_code = $10, default_gain_loss_account_code = $11,
                    is_active = true, updated_at = now()
            RETURNING *
            "#,
        )
        .bind(org_id).bind(code).bind(name).bind(description)
        .bind(default_depreciation_method).bind(default_useful_life_months)
        .bind(default_salvage_value_percent)
        .bind(default_asset_account_code).bind(default_accum_depr_account_code)
        .bind(default_depr_expense_account_code).bind(default_gain_loss_account_code)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_category(&row))
    }

    async fn get_category(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<AssetCategory>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.asset_categories WHERE organization_id = $1 AND code = $2 AND is_active = true"
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_category(&r)))
    }

    async fn get_category_by_id(&self, id: Uuid) -> AtlasResult<Option<AssetCategory>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.asset_categories WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_category(&r)))
    }

    async fn list_categories(&self, org_id: Uuid) -> AtlasResult<Vec<AssetCategory>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.asset_categories WHERE organization_id = $1 AND is_active = true ORDER BY code"
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_category(&r)).collect())
    }

    async fn delete_category(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.asset_categories SET is_active = false, updated_at = now() WHERE organization_id = $1 AND code = $2"
        )
        .bind(org_id).bind(code)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Asset Books
    // ========================================================================

    async fn create_book(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        book_type: &str,
        auto_depreciation: bool,
        depreciation_calendar: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AssetBook> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.asset_books
                (organization_id, code, name, description,
                 book_type, auto_depreciation, depreciation_calendar, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (organization_id, code) DO UPDATE
                SET name = $3, description = $4,
                    book_type = $5, auto_depreciation = $6,
                    depreciation_calendar = $7,
                    is_active = true, updated_at = now()
            RETURNING *
            "#,
        )
        .bind(org_id).bind(code).bind(name).bind(description)
        .bind(book_type).bind(auto_depreciation).bind(depreciation_calendar).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_book(&row))
    }

    async fn get_book(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<AssetBook>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.asset_books WHERE organization_id = $1 AND code = $2 AND is_active = true"
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_book(&r)))
    }

    async fn get_book_by_id(&self, id: Uuid) -> AtlasResult<Option<AssetBook>> {
        let row = sqlx::query("SELECT * FROM _atlas.asset_books WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_book(&r)))
    }

    async fn list_books(&self, org_id: Uuid) -> AtlasResult<Vec<AssetBook>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.asset_books WHERE organization_id = $1 AND is_active = true ORDER BY code"
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_book(&r)).collect())
    }

    async fn delete_book(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.asset_books SET is_active = false, updated_at = now() WHERE organization_id = $1 AND code = $2"
        )
        .bind(org_id).bind(code)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Fixed Assets
    // ========================================================================

    async fn create_asset(
        &self,
        org_id: Uuid,
        asset_number: &str,
        asset_name: &str,
        description: Option<&str>,
        category_id: Option<Uuid>,
        category_code: Option<&str>,
        book_id: Option<Uuid>,
        book_code: Option<&str>,
        asset_type: &str,
        original_cost: &str,
        salvage_value: &str,
        salvage_value_percent: &str,
        depreciation_method: &str,
        useful_life_months: i32,
        declining_balance_rate: Option<&str>,
        acquisition_date: Option<chrono::NaiveDate>,
        location: Option<&str>,
        department_id: Option<Uuid>,
        department_name: Option<&str>,
        custodian_id: Option<Uuid>,
        custodian_name: Option<&str>,
        serial_number: Option<&str>,
        tag_number: Option<&str>,
        manufacturer: Option<&str>,
        model: Option<&str>,
        warranty_expiry: Option<chrono::NaiveDate>,
        insurance_policy_number: Option<&str>,
        insurance_expiry: Option<chrono::NaiveDate>,
        lease_number: Option<&str>,
        lease_expiry: Option<chrono::NaiveDate>,
        asset_account_code: Option<&str>,
        accum_depr_account_code: Option<&str>,
        depr_expense_account_code: Option<&str>,
        gain_loss_account_code: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<FixedAsset> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.fixed_assets
                (organization_id, asset_number, asset_name, description,
                 category_id, category_code, book_id, book_code,
                 asset_type, status,
                 original_cost, current_cost, salvage_value, salvage_value_percent,
                 depreciation_method, useful_life_months, declining_balance_rate,
                 depreciable_basis, accumulated_depreciation, net_book_value,
                 depreciation_per_period,
                 acquisition_date,
                 location, department_id, department_name,
                 custodian_id, custodian_name,
                 serial_number, tag_number, manufacturer, model,
                 warranty_expiry, insurance_policy_number, insurance_expiry,
                 lease_number, lease_expiry,
                 asset_account_code, accum_depr_account_code,
                 depr_expense_account_code, gain_loss_account_code,
                 created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8,
                    $9, 'draft',
                    $10::numeric, $10::numeric, $11::numeric, $12::numeric,
                    $13, $14, $15::numeric,
                    ($10::numeric - $11::numeric), 0, $10::numeric,
                    CASE WHEN $14 > 0 THEN (($10::numeric - $11::numeric) / $14::numeric)
                         ELSE 0 END,
                    $16,
                    $17, $18, $19, $20, $21,
                    $22, $23, $24, $25,
                    $26, $27, $28,
                    $29, $30,
                    $31, $32, $33, $34,
                    $35)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(asset_number).bind(asset_name).bind(description)
        .bind(category_id).bind(category_code).bind(book_id).bind(book_code)
        .bind(asset_type)
        .bind(original_cost).bind(salvage_value).bind(salvage_value_percent)
        .bind(depreciation_method).bind(useful_life_months).bind(declining_balance_rate)
        .bind(acquisition_date)
        .bind(location).bind(department_id).bind(department_name)
        .bind(custodian_id).bind(custodian_name)
        .bind(serial_number).bind(tag_number).bind(manufacturer).bind(model)
        .bind(warranty_expiry).bind(insurance_policy_number).bind(insurance_expiry)
        .bind(lease_number).bind(lease_expiry)
        .bind(asset_account_code).bind(accum_depr_account_code)
        .bind(depr_expense_account_code).bind(gain_loss_account_code)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_asset(&row))
    }

    async fn get_asset(&self, id: Uuid) -> AtlasResult<Option<FixedAsset>> {
        let row = sqlx::query("SELECT * FROM _atlas.fixed_assets WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_asset(&r)))
    }

    async fn get_asset_by_number(&self, org_id: Uuid, asset_number: &str) -> AtlasResult<Option<FixedAsset>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.fixed_assets WHERE organization_id = $1 AND asset_number = $2"
        )
        .bind(org_id).bind(asset_number)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_asset(&r)))
    }

    async fn list_assets(&self, org_id: Uuid, status: Option<&str>, category_code: Option<&str>, book_code: Option<&str>) -> AtlasResult<Vec<FixedAsset>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM _atlas.fixed_assets
            WHERE organization_id = $1
              AND ($2::text IS NULL OR status = $2)
              AND ($3::text IS NULL OR category_code = $3)
              AND ($4::text IS NULL OR book_code = $4)
            ORDER BY asset_number
            "#,
        )
        .bind(org_id).bind(status).bind(category_code).bind(book_code)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_asset(&r)).collect())
    }

    async fn update_asset_status(
        &self,
        id: Uuid,
        status: &str,
        in_service_date: Option<chrono::NaiveDate>,
        disposal_date: Option<chrono::NaiveDate>,
        retirement_date: Option<chrono::NaiveDate>,
    ) -> AtlasResult<FixedAsset> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.fixed_assets
            SET status = $2,
                in_service_date = COALESCE($3, in_service_date),
                disposal_date = COALESCE($4, disposal_date),
                retirement_date = COALESCE($5, retirement_date),
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(status).bind(in_service_date).bind(disposal_date).bind(retirement_date)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_asset(&row))
    }

    async fn update_asset_depreciation(
        &self,
        id: Uuid,
        accumulated_depreciation: &str,
        net_book_value: &str,
        periods_depreciated: i32,
        last_depreciation_date: Option<chrono::NaiveDate>,
        last_depreciation_amount: &str,
    ) -> AtlasResult<FixedAsset> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.fixed_assets
            SET accumulated_depreciation = $2::numeric,
                net_book_value = $3::numeric,
                periods_depreciated = $4,
                last_depreciation_date = $5,
                last_depreciation_amount = $6::numeric,
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(accumulated_depreciation).bind(net_book_value)
        .bind(periods_depreciated).bind(last_depreciation_date).bind(last_depreciation_amount)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_asset(&row))
    }

    async fn update_asset_assignment(
        &self,
        id: Uuid,
        department_id: Option<Uuid>,
        department_name: Option<&str>,
        location: Option<&str>,
        custodian_id: Option<Uuid>,
        custodian_name: Option<&str>,
    ) -> AtlasResult<FixedAsset> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.fixed_assets
            SET department_id = $2, department_name = $3,
                location = $4, custodian_id = $5, custodian_name = $6,
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(department_id).bind(department_name).bind(location)
        .bind(custodian_id).bind(custodian_name)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_asset(&row))
    }

    async fn delete_asset(&self, org_id: Uuid, asset_number: &str) -> AtlasResult<()> {
        sqlx::query(
            "DELETE FROM _atlas.fixed_assets WHERE organization_id = $1 AND asset_number = $2 AND status = 'draft'"
        )
        .bind(org_id).bind(asset_number)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Depreciation History
    // ========================================================================

    async fn create_depreciation_entry(
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
            VALUES ($1, $2, $3, $4, $5, $6, $7::numeric, $8::numeric, $9::numeric, $10, $11)
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

        Ok(self.row_to_depr_history(&row))
    }

    async fn list_depreciation_history(&self, asset_id: Uuid) -> AtlasResult<Vec<AssetDepreciationHistory>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.asset_depreciation_history WHERE asset_id = $1 ORDER BY fiscal_year, period_number"
        )
        .bind(asset_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_depr_history(&r)).collect())
    }

    async fn get_depreciation_for_period(&self, asset_id: Uuid, fiscal_year: i32, period_number: i32) -> AtlasResult<Option<AssetDepreciationHistory>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.asset_depreciation_history WHERE asset_id = $1 AND fiscal_year = $2 AND period_number = $3"
        )
        .bind(asset_id).bind(fiscal_year).bind(period_number)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_depr_history(&r)))
    }

    // ========================================================================
    // Asset Transfers
    // ========================================================================

    async fn create_transfer(
        &self,
        org_id: Uuid,
        transfer_number: &str,
        asset_id: Uuid,
        from_department_id: Option<Uuid>,
        from_department_name: Option<&str>,
        from_location: Option<&str>,
        from_custodian_id: Option<Uuid>,
        from_custodian_name: Option<&str>,
        to_department_id: Option<Uuid>,
        to_department_name: Option<&str>,
        to_location: Option<&str>,
        to_custodian_id: Option<Uuid>,
        to_custodian_name: Option<&str>,
        transfer_date: chrono::NaiveDate,
        reason: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AssetTransfer> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.asset_transfers
                (organization_id, transfer_number, asset_id,
                 from_department_id, from_department_name, from_location,
                 from_custodian_id, from_custodian_name,
                 to_department_id, to_department_name, to_location,
                 to_custodian_id, to_custodian_name,
                 transfer_date, reason, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(transfer_number).bind(asset_id)
        .bind(from_department_id).bind(from_department_name).bind(from_location)
        .bind(from_custodian_id).bind(from_custodian_name)
        .bind(to_department_id).bind(to_department_name).bind(to_location)
        .bind(to_custodian_id).bind(to_custodian_name)
        .bind(transfer_date).bind(reason).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_transfer(&row))
    }

    async fn get_transfer(&self, id: Uuid) -> AtlasResult<Option<AssetTransfer>> {
        let row = sqlx::query("SELECT * FROM _atlas.asset_transfers WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_transfer(&r)))
    }

    async fn list_transfers(&self, org_id: Uuid, asset_id: Option<Uuid>) -> AtlasResult<Vec<AssetTransfer>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM _atlas.asset_transfers
            WHERE organization_id = $1 AND ($2::uuid IS NULL OR asset_id = $2)
            ORDER BY created_at DESC
            "#,
        )
        .bind(org_id).bind(asset_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_transfer(&r)).collect())
    }

    async fn update_transfer_status(
        &self,
        id: Uuid,
        status: &str,
        approved_by: Option<Uuid>,
        rejected_reason: Option<&str>,
    ) -> AtlasResult<AssetTransfer> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.asset_transfers
            SET status = $2,
                approved_by = COALESCE($3, approved_by),
                approved_at = CASE WHEN $3 IS NOT NULL AND approved_at IS NULL THEN now() ELSE approved_at END,
                rejected_reason = $4,
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(status).bind(approved_by).bind(rejected_reason)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_transfer(&row))
    }

    // ========================================================================
    // Asset Retirements
    // ========================================================================

    async fn create_retirement(
        &self,
        org_id: Uuid,
        retirement_number: &str,
        asset_id: Uuid,
        retirement_type: &str,
        retirement_date: chrono::NaiveDate,
        proceeds: &str,
        removal_cost: &str,
        net_book_value_at_retirement: &str,
        accumulated_depreciation_at_retirement: &str,
        gain_loss_amount: &str,
        gain_loss_type: Option<&str>,
        gain_account_code: Option<&str>,
        loss_account_code: Option<&str>,
        cash_account_code: Option<&str>,
        asset_account_code: Option<&str>,
        accum_depr_account_code: Option<&str>,
        reference_number: Option<&str>,
        buyer_name: Option<&str>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AssetRetirement> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.asset_retirements
                (organization_id, retirement_number, asset_id,
                 retirement_type, retirement_date,
                 proceeds, removal_cost,
                 net_book_value_at_retirement, accumulated_depreciation_at_retirement,
                 gain_loss_amount, gain_loss_type,
                 gain_account_code, loss_account_code, cash_account_code,
                 asset_account_code, accum_depr_account_code,
                 reference_number, buyer_name, notes,
                 created_by)
            VALUES ($1, $2, $3, $4, $5,
                    $6::numeric, $7::numeric,
                    $8::numeric, $9::numeric,
                    $10::numeric, $11,
                    $12, $13, $14, $15, $16,
                    $17, $18, $19, $20)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(retirement_number).bind(asset_id)
        .bind(retirement_type).bind(retirement_date)
        .bind(proceeds).bind(removal_cost)
        .bind(net_book_value_at_retirement).bind(accumulated_depreciation_at_retirement)
        .bind(gain_loss_amount).bind(gain_loss_type)
        .bind(gain_account_code).bind(loss_account_code).bind(cash_account_code)
        .bind(asset_account_code).bind(accum_depr_account_code)
        .bind(reference_number).bind(buyer_name).bind(notes)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_retirement(&row))
    }

    async fn get_retirement(&self, id: Uuid) -> AtlasResult<Option<AssetRetirement>> {
        let row = sqlx::query("SELECT * FROM _atlas.asset_retirements WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_retirement(&r)))
    }

    async fn list_retirements(&self, org_id: Uuid, asset_id: Option<Uuid>) -> AtlasResult<Vec<AssetRetirement>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM _atlas.asset_retirements
            WHERE organization_id = $1 AND ($2::uuid IS NULL OR asset_id = $2)
            ORDER BY created_at DESC
            "#,
        )
        .bind(org_id).bind(asset_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_retirement(&r)).collect())
    }

    async fn update_retirement_status(
        &self,
        id: Uuid,
        status: &str,
        approved_by: Option<Uuid>,
    ) -> AtlasResult<AssetRetirement> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.asset_retirements
            SET status = $2,
                approved_by = COALESCE($3, approved_by),
                approved_at = CASE WHEN $3 IS NOT NULL AND approved_at IS NULL THEN now() ELSE approved_at END,
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(status).bind(approved_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_retirement(&row))
    }
}
