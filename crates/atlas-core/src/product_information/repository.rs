//! Product Information Management Repository
//!
//! PostgreSQL storage for product items, categories, cross-references,
//! new item requests, and item templates.

use atlas_shared::{
    ProductItem, PimCategory, PimCategoryAssignment, PimCrossReference,
    PimNewItemRequest, PimItemTemplate, PimDashboard,
    AtlasError, AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Repository trait for Product Information Management data storage
#[async_trait]
pub trait ProductInformationRepository: Send + Sync {
    // Product Items
    async fn create_item(
        &self,
        org_id: Uuid,
        item_number: &str,
        item_name: &str,
        description: Option<&str>,
        long_description: Option<&str>,
        item_type: &str,
        status: &str,
        lifecycle_phase: &str,
        primary_uom_code: &str,
        secondary_uom_code: Option<&str>,
        weight: Option<&str>,
        weight_uom: Option<&str>,
        volume: Option<&str>,
        volume_uom: Option<&str>,
        hazmat_flag: bool,
        lot_control_flag: bool,
        serial_control_flag: bool,
        shelf_life_days: Option<i32>,
        min_order_quantity: Option<&str>,
        max_order_quantity: Option<&str>,
        lead_time_days: Option<i32>,
        list_price: Option<&str>,
        cost_price: Option<&str>,
        currency_code: &str,
        inventory_item_flag: bool,
        purchasable_flag: bool,
        sellable_flag: bool,
        stock_enabled_flag: bool,
        invoice_enabled_flag: bool,
        default_buyer_id: Option<Uuid>,
        default_supplier_id: Option<Uuid>,
        template_id: Option<Uuid>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ProductItem>;

    async fn get_item(&self, id: Uuid) -> AtlasResult<Option<ProductItem>>;
    async fn get_item_by_number(&self, org_id: Uuid, item_number: &str) -> AtlasResult<Option<ProductItem>>;
    async fn list_items(&self, org_id: Uuid, status: Option<&str>, item_type: Option<&str>, category_id: Option<Uuid>) -> AtlasResult<Vec<ProductItem>>;
    async fn update_item_status(&self, id: Uuid, status: &str, lifecycle_phase: Option<&str>) -> AtlasResult<ProductItem>;
    async fn update_item_lifecycle(&self, id: Uuid, lifecycle_phase: &str) -> AtlasResult<ProductItem>;
    async fn delete_item(&self, id: Uuid) -> AtlasResult<()>;

    // Item Categories
    async fn create_category(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        parent_category_id: Option<Uuid>,
        level_number: i32,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PimCategory>;

    async fn get_category(&self, id: Uuid) -> AtlasResult<Option<PimCategory>>;
    async fn get_category_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<PimCategory>>;
    async fn list_categories(&self, org_id: Uuid, parent_id: Option<Uuid>) -> AtlasResult<Vec<PimCategory>>;
    async fn delete_category(&self, id: Uuid) -> AtlasResult<()>;

    // Item Category Assignments
    async fn assign_item_category(
        &self,
        org_id: Uuid,
        item_id: Uuid,
        category_id: Uuid,
        is_primary: bool,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PimCategoryAssignment>;

    async fn get_primary_category_assignment(&self, item_id: Uuid) -> AtlasResult<Option<PimCategoryAssignment>>;
    async fn list_item_categories(&self, item_id: Uuid) -> AtlasResult<Vec<PimCategoryAssignment>>;
    async fn remove_item_category(&self, assignment_id: Uuid) -> AtlasResult<()>;

    // Item Cross-References
    async fn create_cross_reference(
        &self,
        org_id: Uuid,
        item_id: Uuid,
        cross_reference_type: &str,
        cross_reference_value: &str,
        description: Option<&str>,
        source_system: Option<&str>,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PimCrossReference>;

    async fn get_cross_reference_by_value(&self, org_id: Uuid, xref_type: &str, value: &str) -> AtlasResult<Option<PimCrossReference>>;
    async fn list_cross_references(&self, item_id: Uuid) -> AtlasResult<Vec<PimCrossReference>>;
    async fn list_all_cross_references(&self, org_id: Uuid, xref_type: Option<&str>) -> AtlasResult<Vec<PimCrossReference>>;
    async fn delete_cross_reference(&self, id: Uuid) -> AtlasResult<()>;

    // Item Templates
    async fn create_template(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        item_type: &str,
        default_uom_code: Option<&str>,
        default_category_id: Option<Uuid>,
        default_inventory_flag: bool,
        default_purchasable_flag: bool,
        default_sellable_flag: bool,
        default_stock_enabled_flag: bool,
        attribute_defaults: serde_json::Value,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PimItemTemplate>;

    async fn get_template(&self, id: Uuid) -> AtlasResult<Option<PimItemTemplate>>;
    async fn get_template_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<PimItemTemplate>>;
    async fn list_templates(&self, org_id: Uuid) -> AtlasResult<Vec<PimItemTemplate>>;
    async fn delete_template(&self, id: Uuid) -> AtlasResult<()>;

    // New Item Requests
    async fn create_new_item_request(
        &self,
        org_id: Uuid,
        request_number: &str,
        title: &str,
        description: Option<&str>,
        item_type: &str,
        priority: &str,
        status: &str,
        requested_item_number: Option<&str>,
        requested_item_name: Option<&str>,
        requested_category_id: Option<Uuid>,
        justification: Option<&str>,
        target_launch_date: Option<chrono::NaiveDate>,
        estimated_cost: Option<&str>,
        currency_code: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PimNewItemRequest>;

    async fn get_new_item_request(&self, id: Uuid) -> AtlasResult<Option<PimNewItemRequest>>;
    async fn list_new_item_requests(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<PimNewItemRequest>>;
    async fn update_nir_status(
        &self,
        id: Uuid,
        status: &str,
        approved_by: Option<Uuid>,
        approved_at: Option<DateTime<Utc>>,
        rejection_reason: Option<&str>,
    ) -> AtlasResult<PimNewItemRequest>;
    async fn update_nir_implemented(
        &self,
        id: Uuid,
        implemented_item_id: Uuid,
        implemented_at: Option<DateTime<Utc>>,
    ) -> AtlasResult<()>;

    // Dashboard
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<PimDashboard>;
}

/// PostgreSQL implementation
pub struct PostgresProductInformationRepository {
    pool: PgPool,
}

impl PostgresProductInformationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn row_to_item(&self, row: &sqlx::postgres::PgRow) -> ProductItem {
        ProductItem {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            item_number: row.get("item_number"),
            item_name: row.get("item_name"),
            description: row.get("description"),
            long_description: row.get("long_description"),
            item_type: row.get("item_type"),
            status: row.get("status"),
            lifecycle_phase: row.get("lifecycle_phase"),
            primary_uom_code: row.get("primary_uom_code"),
            secondary_uom_code: row.get("secondary_uom_code"),
            weight: row.try_get("weight").ok().map(|v: serde_json::Value| v.to_string()),
            weight_uom: row.get("weight_uom"),
            volume: row.try_get("volume").ok().map(|v: serde_json::Value| v.to_string()),
            volume_uom: row.get("volume_uom"),
            hazmat_flag: row.get("hazmat_flag"),
            lot_control_flag: row.get("lot_control_flag"),
            serial_control_flag: row.get("serial_control_flag"),
            shelf_life_days: row.get("shelf_life_days"),
            min_order_quantity: row.try_get("min_order_quantity").ok().map(|v: serde_json::Value| v.to_string()),
            max_order_quantity: row.try_get("max_order_quantity").ok().map(|v: serde_json::Value| v.to_string()),
            lead_time_days: row.get("lead_time_days"),
            list_price: row.try_get("list_price").ok().map(|v: serde_json::Value| v.to_string()),
            cost_price: row.try_get("cost_price").ok().map(|v: serde_json::Value| v.to_string()),
            currency_code: row.get("currency_code"),
            inventory_item_flag: row.get("inventory_item_flag"),
            purchasable_flag: row.get("purchasable_flag"),
            sellable_flag: row.get("sellable_flag"),
            stock_enabled_flag: row.get("stock_enabled_flag"),
            invoice_enabled_flag: row.get("invoice_enabled_flag"),
            default_buyer_id: row.get("default_buyer_id"),
            default_supplier_id: row.get("default_supplier_id"),
            template_id: row.get("template_id"),
            thumbnail_url: row.get("thumbnail_url"),
            image_url: row.get("image_url"),
            metadata: row.get("metadata"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_category(&self, row: &sqlx::postgres::PgRow) -> PimCategory {
        PimCategory {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            code: row.get("code"),
            name: row.get("name"),
            description: row.get("description"),
            parent_category_id: row.get("parent_category_id"),
            level_number: row.get("level_number"),
            item_count: row.try_get("item_count").unwrap_or(0),
            is_active: row.get("is_active"),
            metadata: row.get("metadata"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_category_assignment(&self, row: &sqlx::postgres::PgRow) -> PimCategoryAssignment {
        PimCategoryAssignment {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            item_id: row.get("item_id"),
            category_id: row.get("category_id"),
            is_primary: row.get("is_primary"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
        }
    }

    fn row_to_cross_reference(&self, row: &sqlx::postgres::PgRow) -> PimCrossReference {
        PimCrossReference {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            item_id: row.get("item_id"),
            cross_reference_type: row.get("cross_reference_type"),
            cross_reference_value: row.get("cross_reference_value"),
            description: row.get("description"),
            source_system: row.get("source_system"),
            effective_from: row.get("effective_from"),
            effective_to: row.get("effective_to"),
            is_active: row.get("is_active"),
            metadata: row.get("metadata"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_template(&self, row: &sqlx::postgres::PgRow) -> PimItemTemplate {
        PimItemTemplate {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            code: row.get("code"),
            name: row.get("name"),
            description: row.get("description"),
            item_type: row.get("item_type"),
            default_uom_code: row.get("default_uom_code"),
            default_category_id: row.get("default_category_id"),
            default_inventory_flag: row.get("default_inventory_flag"),
            default_purchasable_flag: row.get("default_purchasable_flag"),
            default_sellable_flag: row.get("default_sellable_flag"),
            default_stock_enabled_flag: row.get("default_stock_enabled_flag"),
            attribute_defaults: row.get("attribute_defaults"),
            is_active: row.get("is_active"),
            metadata: row.get("metadata"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn row_to_nir(&self, row: &sqlx::postgres::PgRow) -> PimNewItemRequest {
        PimNewItemRequest {
            id: row.get("id"),
            organization_id: row.get("organization_id"),
            request_number: row.get("request_number"),
            title: row.get("title"),
            description: row.get("description"),
            item_type: row.get("item_type"),
            priority: row.get("priority"),
            status: row.get("status"),
            requested_item_number: row.get("requested_item_number"),
            requested_item_name: row.get("requested_item_name"),
            requested_category_id: row.get("requested_category_id"),
            justification: row.get("justification"),
            target_launch_date: row.get("target_launch_date"),
            estimated_cost: row.try_get("estimated_cost").ok().map(|v: serde_json::Value| v.to_string()),
            currency_code: row.get("currency_code"),
            requested_by: row.get("requested_by"),
            approved_by: row.get("approved_by"),
            approved_at: row.get("approved_at"),
            rejection_reason: row.get("rejection_reason"),
            implemented_item_id: row.get("implemented_item_id"),
            implemented_at: row.get("implemented_at"),
            metadata: row.get("metadata"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}

#[async_trait]
impl ProductInformationRepository for PostgresProductInformationRepository {
    // ========================================================================
    // Product Items
    // ========================================================================

    async fn create_item(
        &self,
        org_id: Uuid,
        item_number: &str,
        item_name: &str,
        description: Option<&str>,
        long_description: Option<&str>,
        item_type: &str,
        status: &str,
        lifecycle_phase: &str,
        primary_uom_code: &str,
        secondary_uom_code: Option<&str>,
        weight: Option<&str>,
        weight_uom: Option<&str>,
        volume: Option<&str>,
        volume_uom: Option<&str>,
        hazmat_flag: bool,
        lot_control_flag: bool,
        serial_control_flag: bool,
        shelf_life_days: Option<i32>,
        min_order_quantity: Option<&str>,
        max_order_quantity: Option<&str>,
        lead_time_days: Option<i32>,
        list_price: Option<&str>,
        cost_price: Option<&str>,
        currency_code: &str,
        inventory_item_flag: bool,
        purchasable_flag: bool,
        sellable_flag: bool,
        stock_enabled_flag: bool,
        invoice_enabled_flag: bool,
        default_buyer_id: Option<Uuid>,
        default_supplier_id: Option<Uuid>,
        template_id: Option<Uuid>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ProductItem> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.pim_items (
                organization_id, item_number, item_name, description, long_description,
                item_type, status, lifecycle_phase,
                primary_uom_code, secondary_uom_code,
                weight, weight_uom, volume, volume_uom,
                hazmat_flag, lot_control_flag, serial_control_flag, shelf_life_days,
                min_order_quantity, max_order_quantity, lead_time_days,
                list_price, cost_price, currency_code,
                inventory_item_flag, purchasable_flag, sellable_flag,
                stock_enabled_flag, invoice_enabled_flag,
                default_buyer_id, default_supplier_id, template_id,
                created_by
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8,
                $9, $10, $11, $12, $13, $14,
                $15, $16, $17, $18, $19, $20, $21,
                $22, $23, $24, $25, $26, $27, $28, $29,
                $30, $31, $32, $33
            )
            RETURNING *
            "#
        )
        .bind(org_id).bind(item_number).bind(item_name)
        .bind(description).bind(long_description)
        .bind(item_type).bind(status).bind(lifecycle_phase)
        .bind(primary_uom_code).bind(secondary_uom_code)
        .bind(weight.map(|v| v.parse::<f64>().ok()).flatten())
        .bind(weight_uom)
        .bind(volume.map(|v| v.parse::<f64>().ok()).flatten())
        .bind(volume_uom)
        .bind(hazmat_flag).bind(lot_control_flag).bind(serial_control_flag)
        .bind(shelf_life_days)
        .bind(min_order_quantity.map(|v| v.parse::<f64>().ok()).flatten())
        .bind(max_order_quantity.map(|v| v.parse::<f64>().ok()).flatten())
        .bind(lead_time_days)
        .bind(list_price.map(|v| v.parse::<f64>().ok()).flatten())
        .bind(cost_price.map(|v| v.parse::<f64>().ok()).flatten())
        .bind(currency_code)
        .bind(inventory_item_flag).bind(purchasable_flag).bind(sellable_flag)
        .bind(stock_enabled_flag).bind(invoice_enabled_flag)
        .bind(default_buyer_id).bind(default_supplier_id).bind(template_id)
        .bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_item(&row))
    }

    async fn get_item(&self, id: Uuid) -> AtlasResult<Option<ProductItem>> {
        let row = sqlx::query("SELECT * FROM _atlas.pim_items WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_item(&r)))
    }

    async fn get_item_by_number(&self, org_id: Uuid, item_number: &str) -> AtlasResult<Option<ProductItem>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.pim_items WHERE organization_id = $1 AND item_number = $2"
        )
        .bind(org_id).bind(item_number)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_item(&r)))
    }

    async fn list_items(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        item_type: Option<&str>,
        category_id: Option<Uuid>,
    ) -> AtlasResult<Vec<ProductItem>> {
        let rows = if category_id.is_some() {
            sqlx::query(
                r#"
                SELECT i.* FROM _atlas.pim_items i
                JOIN _atlas.pim_item_category_assignments ica ON i.id = ica.item_id
                WHERE i.organization_id = $1
                  AND ($2::text IS NULL OR i.status = $2)
                  AND ($3::text IS NULL OR i.item_type = $3)
                  AND ica.category_id = $4
                ORDER BY i.item_number
                "#
            )
            .bind(org_id).bind(status).bind(item_type).bind(category_id)
            .fetch_all(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?
        } else {
            sqlx::query(
                r#"
                SELECT * FROM _atlas.pim_items
                WHERE organization_id = $1
                  AND ($2::text IS NULL OR status = $2)
                  AND ($3::text IS NULL OR item_type = $3)
                ORDER BY item_number
                "#
            )
            .bind(org_id).bind(status).bind(item_type)
            .fetch_all(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?
        };
        Ok(rows.iter().map(|r| self.row_to_item(r)).collect())
    }

    async fn update_item_status(&self, id: Uuid, status: &str, lifecycle_phase: Option<&str>) -> AtlasResult<ProductItem> {
        let row = if let Some(phase) = lifecycle_phase {
            sqlx::query(
                r#"
                UPDATE _atlas.pim_items
                SET status = $2, lifecycle_phase = $3, updated_at = now()
                WHERE id = $1
                RETURNING *
                "#
            )
            .bind(id).bind(status).bind(phase)
            .fetch_one(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?
        } else {
            sqlx::query(
                r#"
                UPDATE _atlas.pim_items
                SET status = $2, updated_at = now()
                WHERE id = $1
                RETURNING *
                "#
            )
            .bind(id).bind(status)
            .fetch_one(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?
        };
        Ok(self.row_to_item(&row))
    }

    async fn update_item_lifecycle(&self, id: Uuid, lifecycle_phase: &str) -> AtlasResult<ProductItem> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.pim_items
            SET lifecycle_phase = $2, updated_at = now()
            WHERE id = $1
            RETURNING *
            "#
        )
        .bind(id).bind(lifecycle_phase)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_item(&row))
    }

    async fn delete_item(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.pim_items WHERE id = $1")
            .bind(id).execute(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Item Categories
    // ========================================================================

    async fn create_category(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        parent_category_id: Option<Uuid>,
        level_number: i32,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PimCategory> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.pim_categories (
                organization_id, code, name, description,
                parent_category_id, level_number, created_by
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#
        )
        .bind(org_id).bind(code).bind(name).bind(description)
        .bind(parent_category_id).bind(level_number).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_category(&row))
    }

    async fn get_category(&self, id: Uuid) -> AtlasResult<Option<PimCategory>> {
        let row = sqlx::query("SELECT * FROM _atlas.pim_categories WHERE id = $1 AND is_active = true")
            .bind(id)
            .fetch_optional(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_category(&r)))
    }

    async fn get_category_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<PimCategory>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.pim_categories WHERE organization_id = $1 AND code = $2 AND is_active = true"
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_category(&r)))
    }

    async fn list_categories(&self, org_id: Uuid, parent_id: Option<Uuid>) -> AtlasResult<Vec<PimCategory>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM _atlas.pim_categories
            WHERE organization_id = $1 AND is_active = true
              AND ($2::uuid IS NULL AND parent_category_id IS NULL OR parent_category_id = $2)
            ORDER BY code
            "#
        )
        .bind(org_id).bind(parent_id)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_category(r)).collect())
    }

    async fn delete_category(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.pim_categories WHERE id = $1")
            .bind(id).execute(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Item Category Assignments
    // ========================================================================

    async fn assign_item_category(
        &self,
        org_id: Uuid,
        item_id: Uuid,
        category_id: Uuid,
        is_primary: bool,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PimCategoryAssignment> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.pim_item_category_assignments (
                organization_id, item_id, category_id, is_primary, created_by
            )
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#
        )
        .bind(org_id).bind(item_id).bind(category_id).bind(is_primary).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_category_assignment(&row))
    }

    async fn get_primary_category_assignment(&self, item_id: Uuid) -> AtlasResult<Option<PimCategoryAssignment>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.pim_item_category_assignments WHERE item_id = $1 AND is_primary = true"
        )
        .bind(item_id)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_category_assignment(&r)))
    }

    async fn list_item_categories(&self, item_id: Uuid) -> AtlasResult<Vec<PimCategoryAssignment>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.pim_item_category_assignments WHERE item_id = $1 ORDER BY is_primary DESC, created_at"
        )
        .bind(item_id)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_category_assignment(r)).collect())
    }

    async fn remove_item_category(&self, assignment_id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.pim_item_category_assignments WHERE id = $1")
            .bind(assignment_id).execute(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Item Cross-References
    // ========================================================================

    async fn create_cross_reference(
        &self,
        org_id: Uuid,
        item_id: Uuid,
        cross_reference_type: &str,
        cross_reference_value: &str,
        description: Option<&str>,
        source_system: Option<&str>,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PimCrossReference> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.pim_item_cross_references (
                organization_id, item_id, cross_reference_type, cross_reference_value,
                description, source_system, effective_from, effective_to, created_by
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#
        )
        .bind(org_id).bind(item_id).bind(cross_reference_type).bind(cross_reference_value)
        .bind(description).bind(source_system)
        .bind(effective_from).bind(effective_to).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_cross_reference(&row))
    }

    async fn get_cross_reference_by_value(&self, org_id: Uuid, xref_type: &str, value: &str) -> AtlasResult<Option<PimCrossReference>> {
        let row = sqlx::query(
            r#"
            SELECT * FROM _atlas.pim_item_cross_references
            WHERE organization_id = $1 AND cross_reference_type = $2 AND cross_reference_value = $3 AND is_active = true
            "#
        )
        .bind(org_id).bind(xref_type).bind(value)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_cross_reference(&r)))
    }

    async fn list_cross_references(&self, item_id: Uuid) -> AtlasResult<Vec<PimCrossReference>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.pim_item_cross_references WHERE item_id = $1 AND is_active = true ORDER BY cross_reference_type, cross_reference_value"
        )
        .bind(item_id)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_cross_reference(r)).collect())
    }

    async fn list_all_cross_references(&self, org_id: Uuid, xref_type: Option<&str>) -> AtlasResult<Vec<PimCrossReference>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM _atlas.pim_item_cross_references
            WHERE organization_id = $1 AND is_active = true
              AND ($2::text IS NULL OR cross_reference_type = $2)
            ORDER BY cross_reference_type, cross_reference_value
            "#
        )
        .bind(org_id).bind(xref_type)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_cross_reference(r)).collect())
    }

    async fn delete_cross_reference(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.pim_item_cross_references WHERE id = $1")
            .bind(id).execute(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Item Templates
    // ========================================================================

    async fn create_template(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        item_type: &str,
        default_uom_code: Option<&str>,
        default_category_id: Option<Uuid>,
        default_inventory_flag: bool,
        default_purchasable_flag: bool,
        default_sellable_flag: bool,
        default_stock_enabled_flag: bool,
        attribute_defaults: serde_json::Value,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PimItemTemplate> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.pim_item_templates (
                organization_id, code, name, description, item_type,
                default_uom_code, default_category_id,
                default_inventory_flag, default_purchasable_flag,
                default_sellable_flag, default_stock_enabled_flag,
                attribute_defaults, created_by
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING *
            "#
        )
        .bind(org_id).bind(code).bind(name).bind(description).bind(item_type)
        .bind(default_uom_code).bind(default_category_id)
        .bind(default_inventory_flag).bind(default_purchasable_flag)
        .bind(default_sellable_flag).bind(default_stock_enabled_flag)
        .bind(&attribute_defaults).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_template(&row))
    }

    async fn get_template(&self, id: Uuid) -> AtlasResult<Option<PimItemTemplate>> {
        let row = sqlx::query("SELECT * FROM _atlas.pim_item_templates WHERE id = $1 AND is_active = true")
            .bind(id)
            .fetch_optional(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_template(&r)))
    }

    async fn get_template_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<PimItemTemplate>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.pim_item_templates WHERE organization_id = $1 AND code = $2 AND is_active = true"
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_template(&r)))
    }

    async fn list_templates(&self, org_id: Uuid) -> AtlasResult<Vec<PimItemTemplate>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.pim_item_templates WHERE organization_id = $1 AND is_active = true ORDER BY code"
        )
        .bind(org_id)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_template(r)).collect())
    }

    async fn delete_template(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.pim_item_templates WHERE id = $1")
            .bind(id).execute(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // New Item Requests
    // ========================================================================

    async fn create_new_item_request(
        &self,
        org_id: Uuid,
        request_number: &str,
        title: &str,
        description: Option<&str>,
        item_type: &str,
        priority: &str,
        status: &str,
        requested_item_number: Option<&str>,
        requested_item_name: Option<&str>,
        requested_category_id: Option<Uuid>,
        justification: Option<&str>,
        target_launch_date: Option<chrono::NaiveDate>,
        estimated_cost: Option<&str>,
        currency_code: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PimNewItemRequest> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.pim_new_item_requests (
                organization_id, request_number, title, description,
                item_type, priority, status,
                requested_item_number, requested_item_name,
                requested_category_id, justification,
                target_launch_date, estimated_cost, currency_code,
                created_by
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            RETURNING *
            "#
        )
        .bind(org_id).bind(request_number).bind(title).bind(description)
        .bind(item_type).bind(priority).bind(status)
        .bind(requested_item_number).bind(requested_item_name)
        .bind(requested_category_id).bind(justification)
        .bind(target_launch_date)
        .bind(estimated_cost.map(|v| v.parse::<f64>().ok()).flatten())
        .bind(currency_code).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        Ok(self.row_to_nir(&row))
    }

    async fn get_new_item_request(&self, id: Uuid) -> AtlasResult<Option<PimNewItemRequest>> {
        let row = sqlx::query("SELECT * FROM _atlas.pim_new_item_requests WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| self.row_to_nir(&r)))
    }

    async fn list_new_item_requests(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<PimNewItemRequest>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM _atlas.pim_new_item_requests
            WHERE organization_id = $1 AND ($2::text IS NULL OR status = $2)
            ORDER BY created_at DESC
            "#
        )
        .bind(org_id).bind(status)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(|r| self.row_to_nir(r)).collect())
    }

    async fn update_nir_status(
        &self,
        id: Uuid,
        status: &str,
        approved_by: Option<Uuid>,
        approved_at: Option<DateTime<Utc>>,
        rejection_reason: Option<&str>,
    ) -> AtlasResult<PimNewItemRequest> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.pim_new_item_requests
            SET status = $2, approved_by = COALESCE($3, approved_by),
                approved_at = COALESCE($4, approved_at),
                rejection_reason = COALESCE($5, rejection_reason),
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#
        )
        .bind(id).bind(status).bind(approved_by)
        .bind(approved_at).bind(rejection_reason)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(self.row_to_nir(&row))
    }

    async fn update_nir_implemented(
        &self,
        id: Uuid,
        implemented_item_id: Uuid,
        implemented_at: Option<DateTime<Utc>>,
    ) -> AtlasResult<()> {
        sqlx::query(
            r#"
            UPDATE _atlas.pim_new_item_requests
            SET status = 'implemented', implemented_item_id = $2,
                implemented_at = COALESCE($3, now()), updated_at = now()
            WHERE id = $1
            "#
        )
        .bind(id).bind(implemented_item_id).bind(implemented_at)
        .execute(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<PimDashboard> {
        let row = sqlx::query(
            r#"
            SELECT
                COUNT(*) as total_items,
                COUNT(*) FILTER (WHERE status = 'active') as active_items,
                COUNT(*) FILTER (WHERE status = 'draft') as draft_items,
                COUNT(*) FILTER (WHERE status = 'obsolete') as obsolete_items,
                (SELECT COUNT(*) FROM _atlas.pim_categories WHERE organization_id = $1 AND is_active = true) as total_categories,
                (SELECT COUNT(*) FROM _atlas.pim_new_item_requests WHERE organization_id = $1 AND status = 'submitted') as pending_nir_count,
                (SELECT COUNT(*) FROM _atlas.pim_new_item_requests WHERE organization_id = $1 AND status = 'approved') as approved_nir_count,
                (SELECT COUNT(*) FROM _atlas.pim_item_cross_references WHERE organization_id = $1 AND is_active = true) as cross_reference_count,
                (SELECT COUNT(*) FROM _atlas.pim_items WHERE organization_id = $1 AND created_at > now() - interval '30 days') as recently_created_items
            FROM _atlas.pim_items
            WHERE organization_id = $1
            "#
        )
        .bind(org_id)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        // Items by type breakdown
        let type_rows = sqlx::query(
            r#"
            SELECT item_type, COUNT(*) as count
            FROM _atlas.pim_items
            WHERE organization_id = $1
            GROUP BY item_type
            "#
        )
        .bind(org_id)
        .fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let items_by_type: serde_json::Value = serde_json::to_value(
            type_rows.iter()
                .map(|r| {
                    let item_type: String = r.get("item_type");
                    let count: i64 = r.get("count");
                    (item_type, count)
                })
                .collect::<std::collections::HashMap<String, i64>>()
        ).unwrap_or(serde_json::json!({}));

        Ok(PimDashboard {
            total_items: row.get::<i64, _>("total_items") as i32,
            active_items: row.get::<i64, _>("active_items") as i32,
            draft_items: row.get::<i64, _>("draft_items") as i32,
            obsolete_items: row.get::<i64, _>("obsolete_items") as i32,
            total_categories: row.get::<i64, _>("total_categories") as i32,
            pending_nir_count: row.get::<i64, _>("pending_nir_count") as i32,
            approved_nir_count: row.get::<i64, _>("approved_nir_count") as i32,
            cross_reference_count: row.get::<i64, _>("cross_reference_count") as i32,
            recently_created_items: row.get::<i64, _>("recently_created_items") as i32,
            items_by_type,
        })
    }
}
