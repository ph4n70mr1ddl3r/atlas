//! Advanced Pricing Repository
//!
//! PostgreSQL storage for price lists, price list lines, price tiers,
//! discount rules, charge definitions, pricing strategies, and calculation logs.

use atlas_shared::{
    PriceList, PriceListLine, PriceTier, DiscountRule, ChargeDefinition,
    PricingStrategy, PriceCalculationLog,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

/// Repository trait for advanced pricing data storage
#[async_trait]
pub trait PricingRepository: Send + Sync {
    // Price Lists
    async fn create_price_list(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        currency_code: &str,
        list_type: &str,
        pricing_basis: &str,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PriceList>;

    async fn get_price_list(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<PriceList>>;
    async fn get_price_list_by_id(&self, id: Uuid) -> AtlasResult<Option<PriceList>>;
    async fn list_price_lists(&self, org_id: Uuid, list_type: Option<&str>, status: Option<&str>) -> AtlasResult<Vec<PriceList>>;
    async fn update_price_list_status(&self, id: Uuid, status: &str) -> AtlasResult<PriceList>;
    async fn delete_price_list(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // Price List Lines
    async fn create_price_list_line(
        &self,
        org_id: Uuid,
        price_list_id: Uuid,
        line_number: i32,
        item_id: Option<Uuid>,
        item_code: Option<&str>,
        item_description: Option<&str>,
        pricing_unit_of_measure: &str,
        list_price: &str,
        unit_price: &str,
        cost_price: &str,
        margin_percent: &str,
        minimum_quantity: &str,
        maximum_quantity: Option<&str>,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PriceListLine>;

    async fn get_price_list_line(&self, id: Uuid) -> AtlasResult<Option<PriceListLine>>;
    async fn list_price_list_lines(&self, price_list_id: Uuid) -> AtlasResult<Vec<PriceListLine>>;
    async fn find_price_list_line_by_item(&self, price_list_id: Uuid, item_code: &str) -> AtlasResult<Option<PriceListLine>>;
    async fn delete_price_list_line(&self, id: Uuid) -> AtlasResult<()>;

    // Price Tiers
    async fn create_price_tier(
        &self,
        org_id: Uuid,
        price_list_line_id: Uuid,
        tier_number: i32,
        from_quantity: &str,
        to_quantity: Option<&str>,
        price: &str,
        discount_percent: &str,
        price_type: &str,
    ) -> AtlasResult<PriceTier>;

    async fn list_price_tiers(&self, price_list_line_id: Uuid) -> AtlasResult<Vec<PriceTier>>;
    async fn delete_price_tiers_by_line(&self, price_list_line_id: Uuid) -> AtlasResult<()>;

    // Discount Rules
    async fn create_discount_rule(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        discount_type: &str,
        discount_value: &str,
        application_method: &str,
        stacking_rule: &str,
        priority: i32,
        condition: serde_json::Value,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        max_usage: Option<i32>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<DiscountRule>;

    async fn get_discount_rule(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<DiscountRule>>;
    async fn get_discount_rule_by_id(&self, id: Uuid) -> AtlasResult<Option<DiscountRule>>;
    async fn list_discount_rules(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<DiscountRule>>;
    async fn increment_discount_usage(&self, id: Uuid) -> AtlasResult<()>;
    async fn delete_discount_rule(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // Charge Definitions
    async fn create_charge_definition(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        charge_type: &str,
        charge_category: &str,
        calculation_method: &str,
        charge_amount: &str,
        charge_percent: &str,
        minimum_charge: &str,
        maximum_charge: Option<&str>,
        taxable: bool,
        condition: serde_json::Value,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ChargeDefinition>;

    async fn get_charge_definition(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<ChargeDefinition>>;
    async fn list_charge_definitions(&self, org_id: Uuid, charge_type: Option<&str>) -> AtlasResult<Vec<ChargeDefinition>>;
    async fn delete_charge_definition(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // Pricing Strategies
    async fn create_pricing_strategy(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        strategy_type: &str,
        priority: i32,
        condition: serde_json::Value,
        price_list_id: Option<Uuid>,
        markup_percent: &str,
        markdown_percent: &str,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PricingStrategy>;

    async fn get_pricing_strategy(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<PricingStrategy>>;
    async fn list_pricing_strategies(&self, org_id: Uuid) -> AtlasResult<Vec<PricingStrategy>>;

    // Price Calculation Logs
    async fn create_calculation_log(
        &self,
        org_id: Uuid,
        entity_type: &str,
        entity_id: Uuid,
        line_id: Option<Uuid>,
        item_id: Option<Uuid>,
        item_code: Option<&str>,
        requested_quantity: Option<&str>,
        unit_list_price: &str,
        unit_selling_price: &str,
        discount_amount: &str,
        discount_rule_id: Option<Uuid>,
        charge_amount: &str,
        charge_definition_id: Option<Uuid>,
        strategy_id: Option<Uuid>,
        price_list_id: Option<Uuid>,
        calculation_steps: serde_json::Value,
        currency_code: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PriceCalculationLog>;

    async fn list_calculation_logs(&self, org_id: Uuid, entity_type: Option<&str>, entity_id: Option<Uuid>) -> AtlasResult<Vec<PriceCalculationLog>>;
}

/// PostgreSQL implementation
pub struct PostgresPricingRepository {
    pool: PgPool,
}

impl PostgresPricingRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn row_to_price_list(&self, row: &sqlx::postgres::PgRow) -> PriceList {
        PriceList {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            code: row.get("code"),
            name: row.get("name"),
            description: row.get("description"),
            currency_code: row.get("currency_code"),
            list_type: row.get("list_type"),
            pricing_basis: row.get("pricing_basis"),
            effective_from: row.get("effective_from"),
            effective_to: row.get("effective_to"),
            status: row.get("status"),
            is_active: row.get("is_active"),
            metadata: row.get("metadata"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_price_list_line(&self, row: &sqlx::postgres::PgRow) -> PriceListLine {
        let list_price: serde_json::Value = row.try_get("list_price").unwrap_or(serde_json::json!("0"));
        let unit_price: serde_json::Value = row.try_get("unit_price").unwrap_or(serde_json::json!("0"));
        let cost_price: serde_json::Value = row.try_get("cost_price").unwrap_or(serde_json::json!("0"));
        let margin: serde_json::Value = row.try_get("margin_percent").unwrap_or(serde_json::json!("0"));
        let min_qty: serde_json::Value = row.try_get("minimum_quantity").unwrap_or(serde_json::json!("1"));

        PriceListLine {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            price_list_id: row.get("price_list_id"),
            line_number: row.get("line_number"),
            item_id: row.get("item_id"),
            item_code: row.get("item_code"),
            item_description: row.get("item_description"),
            pricing_unit_of_measure: row.get("pricing_unit_of_measure"),
            list_price: list_price.to_string(),
            unit_price: unit_price.to_string(),
            cost_price: cost_price.to_string(),
            margin_percent: margin.to_string(),
            minimum_quantity: min_qty.to_string(),
            maximum_quantity: row.get("maximum_quantity"),
            effective_from: row.get("effective_from"),
            effective_to: row.get("effective_to"),
            is_active: row.get("is_active"),
            metadata: row.get("metadata"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_price_tier(&self, row: &sqlx::postgres::PgRow) -> PriceTier {
        let from_qty: serde_json::Value = row.try_get("from_quantity").unwrap_or(serde_json::json!("0"));
        let price: serde_json::Value = row.try_get("price").unwrap_or(serde_json::json!("0"));
        let disc_pct: serde_json::Value = row.try_get("discount_percent").unwrap_or(serde_json::json!("0"));

        PriceTier {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            price_list_line_id: row.get("price_list_line_id"),
            tier_number: row.get("tier_number"),
            from_quantity: from_qty.to_string(),
            to_quantity: row.get("to_quantity"),
            price: price.to_string(),
            discount_percent: disc_pct.to_string(),
            price_type: row.get("price_type"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_discount_rule(&self, row: &sqlx::postgres::PgRow) -> DiscountRule {
        let disc_val: serde_json::Value = row.try_get("discount_value").unwrap_or(serde_json::json!("0"));

        DiscountRule {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            code: row.get("code"),
            name: row.get("name"),
            description: row.get("description"),
            discount_type: row.get("discount_type"),
            discount_value: disc_val.to_string(),
            application_method: row.get("application_method"),
            stacking_rule: row.get("stacking_rule"),
            priority: row.get("priority"),
            condition: row.get("condition"),
            effective_from: row.get("effective_from"),
            effective_to: row.get("effective_to"),
            status: row.get("status"),
            is_active: row.get("is_active"),
            usage_count: row.get("usage_count"),
            max_usage: row.get("max_usage"),
            metadata: row.get("metadata"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_charge_definition(&self, row: &sqlx::postgres::PgRow) -> ChargeDefinition {
        let charge_amt: serde_json::Value = row.try_get("charge_amount").unwrap_or(serde_json::json!("0"));
        let charge_pct: serde_json::Value = row.try_get("charge_percent").unwrap_or(serde_json::json!("0"));
        let min_charge: serde_json::Value = row.try_get("minimum_charge").unwrap_or(serde_json::json!("0"));

        ChargeDefinition {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            code: row.get("code"),
            name: row.get("name"),
            description: row.get("description"),
            charge_type: row.get("charge_type"),
            charge_category: row.get("charge_category"),
            calculation_method: row.get("calculation_method"),
            charge_amount: charge_amt.to_string(),
            charge_percent: charge_pct.to_string(),
            minimum_charge: min_charge.to_string(),
            maximum_charge: row.get("maximum_charge"),
            taxable: row.get("taxable"),
            condition: row.get("condition"),
            effective_from: row.get("effective_from"),
            effective_to: row.get("effective_to"),
            is_active: row.get("is_active"),
            metadata: row.get("metadata"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_pricing_strategy(&self, row: &sqlx::postgres::PgRow) -> PricingStrategy {
        let markup: serde_json::Value = row.try_get("markup_percent").unwrap_or(serde_json::json!("0"));
        let markdown: serde_json::Value = row.try_get("markdown_percent").unwrap_or(serde_json::json!("0"));

        PricingStrategy {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            code: row.get("code"),
            name: row.get("name"),
            description: row.get("description"),
            strategy_type: row.get("strategy_type"),
            priority: row.get("priority"),
            condition: row.get("condition"),
            price_list_id: row.get("price_list_id"),
            markup_percent: markup.to_string(),
            markdown_percent: markdown.to_string(),
            effective_from: row.get("effective_from"),
            effective_to: row.get("effective_to"),
            is_active: row.get("is_active"),
            metadata: row.get("metadata"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_calculation_log(&self, row: &sqlx::postgres::PgRow) -> PriceCalculationLog {
        let list_price: serde_json::Value = row.try_get("unit_list_price").unwrap_or(serde_json::json!("0"));
        let sell_price: serde_json::Value = row.try_get("unit_selling_price").unwrap_or(serde_json::json!("0"));
        let disc_amt: serde_json::Value = row.try_get("discount_amount").unwrap_or(serde_json::json!("0"));
        let charge_amt: serde_json::Value = row.try_get("charge_amount").unwrap_or(serde_json::json!("0"));

        PriceCalculationLog {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            calculation_date: row.get("calculation_date"),
            entity_type: row.get("entity_type"),
            entity_id: row.get("entity_id"),
            line_id: row.get("line_id"),
            item_id: row.get("item_id"),
            item_code: row.get("item_code"),
            requested_quantity: row.get("requested_quantity"),
            unit_list_price: list_price.to_string(),
            unit_selling_price: sell_price.to_string(),
            discount_amount: disc_amt.to_string(),
            discount_rule_id: row.get("discount_rule_id"),
            charge_amount: charge_amt.to_string(),
            charge_definition_id: row.get("charge_definition_id"),
            strategy_id: row.get("strategy_id"),
            price_list_id: row.get("price_list_id"),
            calculation_steps: row.get("calculation_steps"),
            currency_code: row.get("currency_code"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
        }
    }
}

#[async_trait]
impl PricingRepository for PostgresPricingRepository {
    // ========================================================================
    // Price Lists
    // ========================================================================

    async fn create_price_list(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        currency_code: &str,
        list_type: &str,
        pricing_basis: &str,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PriceList> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.price_lists
                (organization_id, code, name, description, currency_code,
                 list_type, pricing_basis, effective_from, effective_to, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (organization_id, code) DO UPDATE
                SET name = $3, description = $4, currency_code = $5,
                    list_type = $6, pricing_basis = $7,
                    effective_from = $8, effective_to = $9, updated_at = now()
            RETURNING *
            "#,
        )
        .bind(org_id).bind(code).bind(name).bind(description).bind(currency_code)
        .bind(list_type).bind(pricing_basis)
        .bind(effective_from).bind(effective_to).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_price_list(&row))
    }

    async fn get_price_list(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<PriceList>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.price_lists WHERE organization_id = $1 AND code = $2 AND is_active = true"
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_price_list(&r)))
    }

    async fn get_price_list_by_id(&self, id: Uuid) -> AtlasResult<Option<PriceList>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.price_lists WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_price_list(&r)))
    }

    async fn list_price_lists(&self, org_id: Uuid, list_type: Option<&str>, status: Option<&str>) -> AtlasResult<Vec<PriceList>> {
        let rows = match (list_type, status) {
            (Some(lt), Some(s)) => sqlx::query(
                "SELECT * FROM _atlas.price_lists WHERE organization_id = $1 AND list_type = $2 AND status = $3 AND is_active = true ORDER BY code"
            )
            .bind(org_id).bind(lt).bind(s)
            .fetch_all(&self.pool).await,
            (Some(lt), None) => sqlx::query(
                "SELECT * FROM _atlas.price_lists WHERE organization_id = $1 AND list_type = $2 AND is_active = true ORDER BY code"
            )
            .bind(org_id).bind(lt)
            .fetch_all(&self.pool).await,
            (None, Some(s)) => sqlx::query(
                "SELECT * FROM _atlas.price_lists WHERE organization_id = $1 AND status = $2 AND is_active = true ORDER BY code"
            )
            .bind(org_id).bind(s)
            .fetch_all(&self.pool).await,
            (None, None) => sqlx::query(
                "SELECT * FROM _atlas.price_lists WHERE organization_id = $1 AND is_active = true ORDER BY code"
            )
            .bind(org_id)
            .fetch_all(&self.pool).await,
        }
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_price_list(r)).collect())
    }

    async fn update_price_list_status(&self, id: Uuid, status: &str) -> AtlasResult<PriceList> {
        let row = sqlx::query(
            "UPDATE _atlas.price_lists SET status = $2, updated_at = now() WHERE id = $1 RETURNING *"
        )
        .bind(id).bind(status)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_price_list(&row))
    }

    async fn delete_price_list(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.price_lists SET is_active = false, updated_at = now() WHERE organization_id = $1 AND code = $2"
        )
        .bind(org_id).bind(code)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Price List Lines
    // ========================================================================

    async fn create_price_list_line(
        &self,
        org_id: Uuid,
        price_list_id: Uuid,
        line_number: i32,
        item_id: Option<Uuid>,
        item_code: Option<&str>,
        item_description: Option<&str>,
        pricing_unit_of_measure: &str,
        list_price: &str,
        unit_price: &str,
        cost_price: &str,
        margin_percent: &str,
        minimum_quantity: &str,
        maximum_quantity: Option<&str>,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PriceListLine> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.price_list_lines
                (organization_id, price_list_id, line_number, item_id, item_code,
                 item_description, pricing_unit_of_measure, list_price, unit_price,
                 cost_price, margin_percent, minimum_quantity, maximum_quantity,
                 effective_from, effective_to, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7,
                    $8::numeric, $9::numeric, $10::numeric, $11::numeric,
                    $12::numeric, $13::numeric, $14, $15, $16)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(price_list_id).bind(line_number)
        .bind(item_id).bind(item_code).bind(item_description)
        .bind(pricing_unit_of_measure)
        .bind(list_price).bind(unit_price).bind(cost_price).bind(margin_percent)
        .bind(minimum_quantity).bind(maximum_quantity)
        .bind(effective_from).bind(effective_to).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_price_list_line(&row))
    }

    async fn get_price_list_line(&self, id: Uuid) -> AtlasResult<Option<PriceListLine>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.price_list_lines WHERE id = $1 AND is_active = true"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_price_list_line(&r)))
    }

    async fn list_price_list_lines(&self, price_list_id: Uuid) -> AtlasResult<Vec<PriceListLine>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.price_list_lines WHERE price_list_id = $1 AND is_active = true ORDER BY line_number"
        )
        .bind(price_list_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_price_list_line(r)).collect())
    }

    async fn find_price_list_line_by_item(&self, price_list_id: Uuid, item_code: &str) -> AtlasResult<Option<PriceListLine>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.price_list_lines WHERE price_list_id = $1 AND item_code = $2 AND is_active = true"
        )
        .bind(price_list_id).bind(item_code)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_price_list_line(&r)))
    }

    async fn delete_price_list_line(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.price_list_lines SET is_active = false, updated_at = now() WHERE id = $1"
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Price Tiers
    // ========================================================================

    async fn create_price_tier(
        &self,
        org_id: Uuid,
        price_list_line_id: Uuid,
        tier_number: i32,
        from_quantity: &str,
        to_quantity: Option<&str>,
        price: &str,
        discount_percent: &str,
        price_type: &str,
    ) -> AtlasResult<PriceTier> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.price_tiers
                (organization_id, price_list_line_id, tier_number,
                 from_quantity, to_quantity, price, discount_percent, price_type)
            VALUES ($1, $2, $3, $4::numeric, $5::numeric, $6::numeric, $7::numeric, $8)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(price_list_line_id).bind(tier_number)
        .bind(from_quantity).bind(to_quantity).bind(price)
        .bind(discount_percent).bind(price_type)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_price_tier(&row))
    }

    async fn list_price_tiers(&self, price_list_line_id: Uuid) -> AtlasResult<Vec<PriceTier>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.price_tiers WHERE price_list_line_id = $1 ORDER BY tier_number"
        )
        .bind(price_list_line_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_price_tier(r)).collect())
    }

    async fn delete_price_tiers_by_line(&self, price_list_line_id: Uuid) -> AtlasResult<()> {
        sqlx::query(
            "DELETE FROM _atlas.price_tiers WHERE price_list_line_id = $1"
        )
        .bind(price_list_line_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Discount Rules
    // ========================================================================

    async fn create_discount_rule(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        discount_type: &str,
        discount_value: &str,
        application_method: &str,
        stacking_rule: &str,
        priority: i32,
        condition: serde_json::Value,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        max_usage: Option<i32>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<DiscountRule> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.discount_rules
                (organization_id, code, name, description, discount_type,
                 discount_value, application_method, stacking_rule, priority,
                 condition, effective_from, effective_to, max_usage, created_by)
            VALUES ($1, $2, $3, $4, $5, $6::numeric, $7, $8, $9, $10, $11, $12, $13, $14)
            ON CONFLICT (organization_id, code) DO UPDATE
                SET name = $3, description = $4, discount_type = $5,
                    discount_value = $6::numeric, application_method = $7,
                    stacking_rule = $8, priority = $9, condition = $10,
                    effective_from = $11, effective_to = $12,
                    max_usage = $13, updated_at = now()
            RETURNING *
            "#,
        )
        .bind(org_id).bind(code).bind(name).bind(description)
        .bind(discount_type).bind(discount_value)
        .bind(application_method).bind(stacking_rule).bind(priority)
        .bind(condition).bind(effective_from).bind(effective_to)
        .bind(max_usage).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_discount_rule(&row))
    }

    async fn get_discount_rule(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<DiscountRule>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.discount_rules WHERE organization_id = $1 AND code = $2 AND is_active = true"
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_discount_rule(&r)))
    }

    async fn get_discount_rule_by_id(&self, id: Uuid) -> AtlasResult<Option<DiscountRule>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.discount_rules WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_discount_rule(&r)))
    }

    async fn list_discount_rules(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<DiscountRule>> {
        let rows = match status {
            Some(s) => sqlx::query(
                "SELECT * FROM _atlas.discount_rules WHERE organization_id = $1 AND status = $2 AND is_active = true ORDER BY priority, code"
            )
            .bind(org_id).bind(s)
            .fetch_all(&self.pool).await,
            None => sqlx::query(
                "SELECT * FROM _atlas.discount_rules WHERE organization_id = $1 AND is_active = true ORDER BY priority, code"
            )
            .bind(org_id)
            .fetch_all(&self.pool).await,
        }
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_discount_rule(r)).collect())
    }

    async fn increment_discount_usage(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.discount_rules SET usage_count = usage_count + 1, updated_at = now() WHERE id = $1"
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn delete_discount_rule(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.discount_rules SET is_active = false, updated_at = now() WHERE organization_id = $1 AND code = $2"
        )
        .bind(org_id).bind(code)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Charge Definitions
    // ========================================================================

    async fn create_charge_definition(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        charge_type: &str,
        charge_category: &str,
        calculation_method: &str,
        charge_amount: &str,
        charge_percent: &str,
        minimum_charge: &str,
        maximum_charge: Option<&str>,
        taxable: bool,
        condition: serde_json::Value,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ChargeDefinition> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.charge_definitions
                (organization_id, code, name, description, charge_type,
                 charge_category, calculation_method, charge_amount, charge_percent,
                 minimum_charge, maximum_charge, taxable, condition,
                 effective_from, effective_to, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7,
                    $8::numeric, $9::numeric, $10::numeric, $11::numeric,
                    $12, $13, $14, $15, $16)
            ON CONFLICT (organization_id, code) DO UPDATE
                SET name = $3, description = $4, charge_type = $5,
                    charge_category = $6, calculation_method = $7,
                    charge_amount = $8::numeric, charge_percent = $9::numeric,
                    minimum_charge = $10::numeric, maximum_charge = $11::numeric,
                    taxable = $12, condition = $13,
                    effective_from = $14, effective_to = $15, updated_at = now()
            RETURNING *
            "#,
        )
        .bind(org_id).bind(code).bind(name).bind(description)
        .bind(charge_type).bind(charge_category).bind(calculation_method)
        .bind(charge_amount).bind(charge_percent)
        .bind(minimum_charge).bind(maximum_charge)
        .bind(taxable).bind(condition)
        .bind(effective_from).bind(effective_to).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_charge_definition(&row))
    }

    async fn get_charge_definition(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<ChargeDefinition>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.charge_definitions WHERE organization_id = $1 AND code = $2 AND is_active = true"
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_charge_definition(&r)))
    }

    async fn list_charge_definitions(&self, org_id: Uuid, charge_type: Option<&str>) -> AtlasResult<Vec<ChargeDefinition>> {
        let rows = match charge_type {
            Some(ct) => sqlx::query(
                "SELECT * FROM _atlas.charge_definitions WHERE organization_id = $1 AND charge_type = $2 AND is_active = true ORDER BY code"
            )
            .bind(org_id).bind(ct)
            .fetch_all(&self.pool).await,
            None => sqlx::query(
                "SELECT * FROM _atlas.charge_definitions WHERE organization_id = $1 AND is_active = true ORDER BY code"
            )
            .bind(org_id)
            .fetch_all(&self.pool).await,
        }
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_charge_definition(r)).collect())
    }

    async fn delete_charge_definition(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.charge_definitions SET is_active = false, updated_at = now() WHERE organization_id = $1 AND code = $2"
        )
        .bind(org_id).bind(code)
        .execute(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Pricing Strategies
    // ========================================================================

    async fn create_pricing_strategy(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        strategy_type: &str,
        priority: i32,
        condition: serde_json::Value,
        price_list_id: Option<Uuid>,
        markup_percent: &str,
        markdown_percent: &str,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PricingStrategy> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.pricing_strategies
                (organization_id, code, name, description, strategy_type,
                 priority, condition, price_list_id,
                 markup_percent, markdown_percent,
                 effective_from, effective_to, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8,
                    $9::numeric, $10::numeric, $11, $12, $13)
            ON CONFLICT (organization_id, code) DO UPDATE
                SET name = $3, description = $4, strategy_type = $5,
                    priority = $6, condition = $7, price_list_id = $8,
                    markup_percent = $9::numeric, markdown_percent = $10::numeric,
                    effective_from = $11, effective_to = $12, updated_at = now()
            RETURNING *
            "#,
        )
        .bind(org_id).bind(code).bind(name).bind(description)
        .bind(strategy_type).bind(priority).bind(condition)
        .bind(price_list_id)
        .bind(markup_percent).bind(markdown_percent)
        .bind(effective_from).bind(effective_to).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_pricing_strategy(&row))
    }

    async fn get_pricing_strategy(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<PricingStrategy>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.pricing_strategies WHERE organization_id = $1 AND code = $2 AND is_active = true"
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_pricing_strategy(&r)))
    }

    async fn list_pricing_strategies(&self, org_id: Uuid) -> AtlasResult<Vec<PricingStrategy>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.pricing_strategies WHERE organization_id = $1 AND is_active = true ORDER BY priority, code"
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_pricing_strategy(r)).collect())
    }

    // ========================================================================
    // Price Calculation Logs
    // ========================================================================

    async fn create_calculation_log(
        &self,
        org_id: Uuid,
        entity_type: &str,
        entity_id: Uuid,
        line_id: Option<Uuid>,
        item_id: Option<Uuid>,
        item_code: Option<&str>,
        requested_quantity: Option<&str>,
        unit_list_price: &str,
        unit_selling_price: &str,
        discount_amount: &str,
        discount_rule_id: Option<Uuid>,
        charge_amount: &str,
        charge_definition_id: Option<Uuid>,
        strategy_id: Option<Uuid>,
        price_list_id: Option<Uuid>,
        calculation_steps: serde_json::Value,
        currency_code: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PriceCalculationLog> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.price_calculation_logs
                (organization_id, entity_type, entity_id, line_id,
                 item_id, item_code, requested_quantity,
                 unit_list_price, unit_selling_price, discount_amount,
                 discount_rule_id, charge_amount, charge_definition_id,
                 strategy_id, price_list_id, calculation_steps,
                 currency_code, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7::numeric,
                    $8::numeric, $9::numeric, $10::numeric,
                    $11, $12::numeric, $13,
                    $14, $15, $16, $17, $18)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(entity_type).bind(entity_id).bind(line_id)
        .bind(item_id).bind(item_code).bind(requested_quantity)
        .bind(unit_list_price).bind(unit_selling_price).bind(discount_amount)
        .bind(discount_rule_id).bind(charge_amount).bind(charge_definition_id)
        .bind(strategy_id).bind(price_list_id).bind(calculation_steps)
        .bind(currency_code).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_calculation_log(&row))
    }

    async fn list_calculation_logs(&self, org_id: Uuid, entity_type: Option<&str>, entity_id: Option<Uuid>) -> AtlasResult<Vec<PriceCalculationLog>> {
        let rows = match (entity_type, entity_id) {
            (Some(et), Some(eid)) => sqlx::query(
                "SELECT * FROM _atlas.price_calculation_logs WHERE organization_id = $1 AND entity_type = $2 AND entity_id = $3 ORDER BY calculation_date DESC"
            )
            .bind(org_id).bind(et).bind(eid)
            .fetch_all(&self.pool).await,
            (Some(et), None) => sqlx::query(
                "SELECT * FROM _atlas.price_calculation_logs WHERE organization_id = $1 AND entity_type = $2 ORDER BY calculation_date DESC"
            )
            .bind(org_id).bind(et)
            .fetch_all(&self.pool).await,
            _ => sqlx::query(
                "SELECT * FROM _atlas.price_calculation_logs WHERE organization_id = $1 ORDER BY calculation_date DESC LIMIT 100"
            )
            .bind(org_id)
            .fetch_all(&self.pool).await,
        }
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_calculation_log(r)).collect())
    }
}
