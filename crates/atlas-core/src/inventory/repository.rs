//! Inventory Management Repository
//!
//! PostgreSQL storage for inventory data: organizations, items,
//! subinventories, locators, on-hand balances, transactions,
//! cycle counts, and transaction reasons.

use atlas_shared::{
    InventoryOrganization, ItemCategory, Item, Subinventory, Locator,
    OnHandBalance, InventoryTransactionType, InventoryTransaction,
    CycleCountHeader, CycleCountLine, TransactionReason,
    AtlasResult,
};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

/// Repository trait for inventory data storage
#[async_trait]
pub trait InventoryRepository: Send + Sync {
    // Inventory Organizations
    async fn create_inventory_org(
        &self,
        org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        org_type: &str, location_code: Option<&str>, address: Option<serde_json::Value>,
        default_subinventory_code: Option<&str>,
        default_currency_code: &str,
        requires_approval_for_issues: bool,
        requires_approval_for_transfers: bool,
        enable_lot_control: bool,
        enable_serial_control: bool,
        enable_revision_control: bool,
        created_by: Option<Uuid>,
    ) -> AtlasResult<InventoryOrganization>;

    async fn get_inventory_org(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<InventoryOrganization>>;
    async fn list_inventory_orgs(&self, org_id: Uuid) -> AtlasResult<Vec<InventoryOrganization>>;
    async fn delete_inventory_org(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    // Item Categories
    async fn create_item_category(
        &self, org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        parent_category_id: Option<Uuid>, track_as_asset: bool, created_by: Option<Uuid>,
    ) -> AtlasResult<ItemCategory>;

    async fn get_item_category(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<ItemCategory>>;
    async fn list_item_categories(&self, org_id: Uuid) -> AtlasResult<Vec<ItemCategory>>;

    // Items
    async fn create_item(
        &self,
        org_id: Uuid, item_code: &str, name: &str, description: Option<&str>,
        long_description: Option<&str>, category_id: Option<Uuid>, category_code: Option<&str>,
        item_type: &str, uom: &str, secondary_uom: Option<&str>,
        weight: Option<&str>, weight_uom: Option<&str>,
        volume: Option<&str>, volume_uom: Option<&str>,
        list_price: &str, standard_cost: &str,
        min_order_quantity: Option<&str>, max_order_quantity: Option<&str>,
        lead_time_days: i32, shelf_life_days: Option<i32>,
        is_lot_controlled: bool, is_serial_controlled: bool, is_revision_controlled: bool,
        is_perishable: bool, is_hazardous: bool,
        is_purchasable: bool, is_sellable: bool, is_stockable: bool,
        inventory_asset_account_code: Option<&str>, expense_account_code: Option<&str>,
        cost_of_goods_sold_account: Option<&str>, revenue_account_code: Option<&str>,
        barcode: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<Item>;

    async fn get_item(&self, id: Uuid) -> AtlasResult<Option<Item>>;
    async fn get_item_by_code(&self, org_id: Uuid, item_code: &str) -> AtlasResult<Option<Item>>;
    async fn list_items(&self, org_id: Uuid, category_code: Option<&str>, item_type: Option<&str>) -> AtlasResult<Vec<Item>>;
    async fn update_item_status(&self, id: Uuid, is_active: bool) -> AtlasResult<Item>;

    // Subinventories
    async fn create_subinventory(
        &self, org_id: Uuid, inventory_org_id: Uuid, code: &str, name: &str,
        description: Option<&str>, subinventory_type: &str,
        asset_subinventory: bool, quantity_tracked: bool,
        location_code: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<Subinventory>;

    async fn get_subinventory(&self, id: Uuid) -> AtlasResult<Option<Subinventory>>;
    async fn list_subinventories(&self, inventory_org_id: Uuid) -> AtlasResult<Vec<Subinventory>>;

    // Locators
    async fn create_locator(
        &self, org_id: Uuid, subinventory_id: Uuid, code: &str,
        description: Option<&str>, picker_order: i32,
    ) -> AtlasResult<Locator>;

    async fn list_locators(&self, subinventory_id: Uuid) -> AtlasResult<Vec<Locator>>;

    // On-Hand Balances
    async fn get_on_hand_balance(
        &self, org_id: Uuid, inventory_org_id: Uuid, item_id: Uuid,
        subinventory_id: Uuid, locator_id: Option<Uuid>,
        lot_number: Option<&str>, serial_number: Option<&str>, revision: Option<&str>,
    ) -> AtlasResult<Option<OnHandBalance>>;

    async fn upsert_on_hand_balance(
        &self, org_id: Uuid, inventory_org_id: Uuid, item_id: Uuid,
        subinventory_id: Uuid, locator_id: Option<Uuid>,
        lot_number: Option<&str>, serial_number: Option<&str>, revision: Option<&str>,
        quantity_delta: &str, unit_cost: &str,
    ) -> AtlasResult<OnHandBalance>;

    async fn list_on_hand_balances(&self, org_id: Uuid, item_id: Option<Uuid>, inventory_org_id: Option<Uuid>) -> AtlasResult<Vec<OnHandBalance>>;

    // Transaction Types
    async fn create_transaction_type(
        &self, org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        transaction_action: &str, source_type: &str, is_system: bool,
        created_by: Option<Uuid>,
    ) -> AtlasResult<InventoryTransactionType>;

    async fn get_transaction_type(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<InventoryTransactionType>>;
    async fn list_transaction_types(&self, org_id: Uuid) -> AtlasResult<Vec<InventoryTransactionType>>;

    // Inventory Transactions
    async fn create_transaction(
        &self,
        org_id: Uuid, transaction_number: &str,
        transaction_type_id: Option<Uuid>, transaction_type_code: Option<&str>,
        transaction_action: &str, source_type: &str,
        source_id: Option<Uuid>, source_number: Option<&str>, source_line_id: Option<Uuid>,
        item_id: Uuid, item_code: Option<&str>, item_description: Option<&str>,
        from_inventory_org_id: Option<Uuid>, from_subinventory_id: Option<Uuid>, from_locator_id: Option<Uuid>,
        to_inventory_org_id: Option<Uuid>, to_subinventory_id: Option<Uuid>, to_locator_id: Option<Uuid>,
        quantity: &str, uom: &str, unit_cost: &str, total_cost: &str,
        lot_number: Option<&str>, serial_number: Option<&str>, revision: Option<&str>,
        transaction_date: chrono::DateTime<chrono::Utc>,
        reason_id: Option<Uuid>, reason_name: Option<&str>,
        notes: Option<&str>, status: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<InventoryTransaction>;

    async fn get_transaction(&self, id: Uuid) -> AtlasResult<Option<InventoryTransaction>>;
    async fn list_transactions(
        &self, org_id: Uuid, item_id: Option<Uuid>,
        transaction_action: Option<&str>, status: Option<&str>,
    ) -> AtlasResult<Vec<InventoryTransaction>>;
    async fn update_transaction_status(&self, id: Uuid, status: &str, approved_by: Option<Uuid>) -> AtlasResult<InventoryTransaction>;

    // Transaction Reasons
    async fn create_transaction_reason(
        &self, org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        applicable_actions: serde_json::Value, created_by: Option<Uuid>,
    ) -> AtlasResult<TransactionReason>;

    async fn list_transaction_reasons(&self, org_id: Uuid) -> AtlasResult<Vec<TransactionReason>>;

    // Cycle Counts
    async fn create_cycle_count(
        &self, org_id: Uuid, count_number: &str, name: &str, description: Option<&str>,
        inventory_org_id: Uuid, subinventory_id: Option<Uuid>, count_date: chrono::NaiveDate,
        count_method: &str, tolerance_percent: &str, created_by: Option<Uuid>,
    ) -> AtlasResult<CycleCountHeader>;

    async fn get_cycle_count(&self, id: Uuid) -> AtlasResult<Option<CycleCountHeader>>;
    async fn list_cycle_counts(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<CycleCountHeader>>;
    async fn update_cycle_count_status(&self, id: Uuid, status: &str) -> AtlasResult<CycleCountHeader>;
    async fn update_cycle_count_summary(
        &self, id: Uuid, total_items: i32, counted_items: i32,
        matched_items: i32, mismatched_items: i32,
    ) -> AtlasResult<CycleCountHeader>;

    // Cycle Count Lines
    async fn create_cycle_count_line(
        &self, org_id: Uuid, cycle_count_id: Uuid, line_number: i32,
        item_id: Uuid, item_code: Option<&str>, item_description: Option<&str>,
        subinventory_id: Option<Uuid>, locator_id: Option<Uuid>,
        lot_number: Option<&str>, revision: Option<&str>,
        system_quantity: &str,
    ) -> AtlasResult<CycleCountLine>;

    async fn list_cycle_count_lines(&self, cycle_count_id: Uuid) -> AtlasResult<Vec<CycleCountLine>>;
    async fn update_cycle_count_line_count(
        &self, id: Uuid, count_number: i32, count_quantity: &str, counted_by: Option<Uuid>,
    ) -> AtlasResult<CycleCountLine>;
    async fn approve_cycle_count_line(&self, id: Uuid, approved_quantity: &str) -> AtlasResult<CycleCountLine>;
}

/// PostgreSQL implementation
pub struct PostgresInventoryRepository {
    pool: PgPool,
}

impl PostgresInventoryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

// Row mappers

fn row_to_inventory_org(row: &sqlx::postgres::PgRow) -> InventoryOrganization {
    InventoryOrganization {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        code: row.get("code"),
        name: row.get("name"),
        description: row.get("description"),
        org_type: row.get("org_type"),
        location_code: row.get("location_code"),
        address: row.get("address"),
        is_active: row.get("is_active"),
        default_subinventory_code: row.get("default_subinventory_code"),
        default_currency_code: row.get("default_currency_code"),
        requires_approval_for_issues: row.get("requires_approval_for_issues"),
        requires_approval_for_transfers: row.get("requires_approval_for_transfers"),
        enable_lot_control: row.get("enable_lot_control"),
        enable_serial_control: row.get("enable_serial_control"),
        enable_revision_control: row.get("enable_revision_control"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_item_category(row: &sqlx::postgres::PgRow) -> ItemCategory {
    ItemCategory {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        code: row.get("code"),
        name: row.get("name"),
        description: row.get("description"),
        parent_category_id: row.get("parent_category_id"),
        track_as_asset: row.get("track_as_asset"),
        is_active: row.get("is_active"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_item(row: &sqlx::postgres::PgRow) -> Item {
    #[allow(dead_code)]
    fn get_num(row: &sqlx::postgres::PgRow, col: &str) -> String {
        let v: serde_json::Value = row.try_get(col).unwrap_or(serde_json::json!("0"));
        v.to_string()
    }
    Item {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        item_code: row.get("item_code"),
        name: row.get("name"),
        description: row.get("description"),
        long_description: row.get("long_description"),
        category_id: row.get("category_id"),
        category_code: row.get("category_code"),
        item_type: row.get("item_type"),
        uom: row.get("uom"),
        secondary_uom: row.get("secondary_uom"),
        weight: row.get("weight"),
        weight_uom: row.get("weight_uom"),
        volume: row.get("volume"),
        volume_uom: row.get("volume_uom"),
        list_price: row.get("list_price"),
        standard_cost: row.get("standard_cost"),
        min_order_quantity: row.get("min_order_quantity"),
        max_order_quantity: row.get("max_order_quantity"),
        lead_time_days: row.get("lead_time_days"),
        shelf_life_days: row.get("shelf_life_days"),
        is_lot_controlled: row.get("is_lot_controlled"),
        is_serial_controlled: row.get("is_serial_controlled"),
        is_revision_controlled: row.get("is_revision_controlled"),
        is_perishable: row.get("is_perishable"),
        is_hazardous: row.get("is_hazardous"),
        is_purchasable: row.get("is_purchasable"),
        is_sellable: row.get("is_sellable"),
        is_stockable: row.get("is_stockable"),
        inventory_asset_account_code: row.get("inventory_asset_account_code"),
        expense_account_code: row.get("expense_account_code"),
        cost_of_goods_sold_account: row.get("cost_of_goods_sold_account"),
        revenue_account_code: row.get("revenue_account_code"),
        image_url: row.get("image_url"),
        barcode: row.get("barcode"),
        supplier_item_codes: row.try_get("supplier_item_codes").unwrap_or(serde_json::json!([])),
        is_active: row.get("is_active"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_subinventory(row: &sqlx::postgres::PgRow) -> Subinventory {
    Subinventory {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        inventory_org_id: row.get("inventory_org_id"),
        code: row.get("code"),
        name: row.get("name"),
        description: row.get("description"),
        subinventory_type: row.get("subinventory_type"),
        asset_subinventory: row.get("asset_subinventory"),
        quantity_tracked: row.get("quantity_tracked"),
        location_code: row.get("location_code"),
        is_active: row.get("is_active"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_locator(row: &sqlx::postgres::PgRow) -> Locator {
    Locator {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        subinventory_id: row.get("subinventory_id"),
        code: row.get("code"),
        description: row.get("description"),
        picker_order: row.get("picker_order"),
        is_active: row.get("is_active"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_on_hand(row: &sqlx::postgres::PgRow) -> OnHandBalance {
    OnHandBalance {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        inventory_org_id: row.get("inventory_org_id"),
        item_id: row.get("item_id"),
        subinventory_id: row.get("subinventory_id"),
        locator_id: row.get("locator_id"),
        lot_number: row.get("lot_number"),
        serial_number: row.get("serial_number"),
        revision: row.get("revision"),
        quantity: row.get("quantity"),
        reserved_quantity: row.get("reserved_quantity"),
        available_quantity: row.get("available_quantity"),
        unit_cost: row.get("unit_cost"),
        total_value: row.get("total_value"),
        last_transaction_date: row.get("last_transaction_date"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_txn_type(row: &sqlx::postgres::PgRow) -> InventoryTransactionType {
    InventoryTransactionType {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        code: row.get("code"),
        name: row.get("name"),
        description: row.get("description"),
        transaction_action: row.get("transaction_action"),
        source_type: row.get("source_type"),
        is_system: row.get("is_system"),
        is_active: row.get("is_active"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_transaction(row: &sqlx::postgres::PgRow) -> InventoryTransaction {
    InventoryTransaction {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        transaction_number: row.get("transaction_number"),
        transaction_type_id: row.get("transaction_type_id"),
        transaction_type_code: row.get("transaction_type_code"),
        transaction_action: row.get("transaction_action"),
        source_type: row.get("source_type"),
        source_id: row.get("source_id"),
        source_number: row.get("source_number"),
        source_line_id: row.get("source_line_id"),
        item_id: row.get("item_id"),
        item_code: row.get("item_code"),
        item_description: row.get("item_description"),
        from_inventory_org_id: row.get("from_inventory_org_id"),
        from_subinventory_id: row.get("from_subinventory_id"),
        from_locator_id: row.get("from_locator_id"),
        to_inventory_org_id: row.get("to_inventory_org_id"),
        to_subinventory_id: row.get("to_subinventory_id"),
        to_locator_id: row.get("to_locator_id"),
        quantity: row.get("quantity"),
        uom: row.get("uom"),
        unit_cost: row.get("unit_cost"),
        total_cost: row.get("total_cost"),
        lot_number: row.get("lot_number"),
        serial_number: row.get("serial_number"),
        revision: row.get("revision"),
        transaction_date: row.get("transaction_date"),
        accounting_date: row.get("accounting_date"),
        reason_id: row.get("reason_id"),
        reason_name: row.get("reason_name"),
        reference: row.get("reference"),
        reference_type: row.get("reference_type"),
        notes: row.get("notes"),
        is_posted: row.get("is_posted"),
        posted_at: row.get("posted_at"),
        journal_entry_id: row.get("journal_entry_id"),
        status: row.get("status"),
        approved_by: row.get("approved_by"),
        approved_at: row.get("approved_at"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_cycle_count(row: &sqlx::postgres::PgRow) -> CycleCountHeader {
    CycleCountHeader {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        count_number: row.get("count_number"),
        name: row.get("name"),
        description: row.get("description"),
        inventory_org_id: row.get("inventory_org_id"),
        subinventory_id: row.get("subinventory_id"),
        count_date: row.get("count_date"),
        status: row.get("status"),
        count_method: row.get("count_method"),
        tolerance_percent: row.get("tolerance_percent"),
        total_items: row.get("total_items"),
        counted_items: row.get("counted_items"),
        matched_items: row.get("matched_items"),
        mismatched_items: row.get("mismatched_items"),
        approved_by: row.get("approved_by"),
        approved_at: row.get("approved_at"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_cycle_count_line(row: &sqlx::postgres::PgRow) -> CycleCountLine {
    CycleCountLine {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        cycle_count_id: row.get("cycle_count_id"),
        line_number: row.get("line_number"),
        item_id: row.get("item_id"),
        item_code: row.get("item_code"),
        item_description: row.get("item_description"),
        subinventory_id: row.get("subinventory_id"),
        locator_id: row.get("locator_id"),
        lot_number: row.get("lot_number"),
        revision: row.get("revision"),
        system_quantity: row.get("system_quantity"),
        count_quantity_1: row.get("count_quantity_1"),
        count_quantity_2: row.get("count_quantity_2"),
        count_quantity_3: row.get("count_quantity_3"),
        count_date_1: row.get("count_date_1"),
        count_date_2: row.get("count_date_2"),
        count_date_3: row.get("count_date_3"),
        counted_by_1: row.get("counted_by_1"),
        counted_by_2: row.get("counted_by_2"),
        counted_by_3: row.get("counted_by_3"),
        approved_quantity: row.get("approved_quantity"),
        variance_quantity: row.get("variance_quantity"),
        variance_percent: row.get("variance_percent"),
        is_matched: row.get("is_matched"),
        status: row.get("status"),
        adjustment_transaction_id: row.get("adjustment_transaction_id"),
        notes: row.get("notes"),
        metadata: row.try_get("metadata").unwrap_or(serde_json::json!({})),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_txn_reason(row: &sqlx::postgres::PgRow) -> TransactionReason {
    TransactionReason {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        code: row.get("code"),
        name: row.get("name"),
        description: row.get("description"),
        applicable_actions: row.try_get("applicable_actions").unwrap_or(serde_json::json!([])),
        is_active: row.get("is_active"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

#[async_trait]
impl InventoryRepository for PostgresInventoryRepository {
    // ========================================================================
    // Inventory Organizations
    // ========================================================================

    async fn create_inventory_org(
        &self,
        org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        org_type: &str, location_code: Option<&str>, address: Option<serde_json::Value>,
        default_subinventory_code: Option<&str>,
        default_currency_code: &str,
        requires_approval_for_issues: bool,
        requires_approval_for_transfers: bool,
        enable_lot_control: bool,
        enable_serial_control: bool,
        enable_revision_control: bool,
        created_by: Option<Uuid>,
    ) -> AtlasResult<InventoryOrganization> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.inventory_organizations
                (organization_id, code, name, description, org_type, location_code, address,
                 default_subinventory_code, default_currency_code,
                 requires_approval_for_issues, requires_approval_for_transfers,
                 enable_lot_control, enable_serial_control, enable_revision_control, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            ON CONFLICT (organization_id, code) DO UPDATE
                SET name = $3, description = $4, org_type = $5, location_code = $6,
                    address = $7, is_active = true, updated_at = now()
            RETURNING *
            "#,
        )
        .bind(org_id).bind(code).bind(name).bind(description)
        .bind(org_type).bind(location_code).bind(address)
        .bind(default_subinventory_code).bind(default_currency_code)
        .bind(requires_approval_for_issues).bind(requires_approval_for_transfers)
        .bind(enable_lot_control).bind(enable_serial_control).bind(enable_revision_control)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_inventory_org(&row))
    }

    async fn get_inventory_org(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<InventoryOrganization>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.inventory_organizations WHERE organization_id = $1 AND code = $2 AND is_active = true"
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_inventory_org(&r)))
    }

    async fn list_inventory_orgs(&self, org_id: Uuid) -> AtlasResult<Vec<InventoryOrganization>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.inventory_organizations WHERE organization_id = $1 AND is_active = true ORDER BY code"
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_inventory_org).collect())
    }

    async fn delete_inventory_org(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        sqlx::query(
            "UPDATE _atlas.inventory_organizations SET is_active = false, updated_at = now() WHERE organization_id = $1 AND code = $2"
        )
        .bind(org_id).bind(code)
        .execute(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Item Categories
    // ========================================================================

    async fn create_item_category(
        &self, org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        parent_category_id: Option<Uuid>, track_as_asset: bool, created_by: Option<Uuid>,
    ) -> AtlasResult<ItemCategory> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.item_categories
                (organization_id, code, name, description, parent_category_id, track_as_asset, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (organization_id, code) DO UPDATE
                SET name = $3, description = $4, parent_category_id = $5, track_as_asset = $6, is_active = true, updated_at = now()
            RETURNING *
            "#,
        )
        .bind(org_id).bind(code).bind(name).bind(description)
        .bind(parent_category_id).bind(track_as_asset).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_item_category(&row))
    }

    async fn get_item_category(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<ItemCategory>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.item_categories WHERE organization_id = $1 AND code = $2 AND is_active = true"
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_item_category(&r)))
    }

    async fn list_item_categories(&self, org_id: Uuid) -> AtlasResult<Vec<ItemCategory>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.item_categories WHERE organization_id = $1 AND is_active = true ORDER BY code"
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_item_category).collect())
    }

    // ========================================================================
    // Items
    // ========================================================================

    async fn create_item(
        &self,
        org_id: Uuid, item_code: &str, name: &str, description: Option<&str>,
        long_description: Option<&str>, category_id: Option<Uuid>, category_code: Option<&str>,
        item_type: &str, uom: &str, secondary_uom: Option<&str>,
        weight: Option<&str>, weight_uom: Option<&str>,
        volume: Option<&str>, volume_uom: Option<&str>,
        list_price: &str, standard_cost: &str,
        min_order_quantity: Option<&str>, max_order_quantity: Option<&str>,
        lead_time_days: i32, shelf_life_days: Option<i32>,
        is_lot_controlled: bool, is_serial_controlled: bool, is_revision_controlled: bool,
        is_perishable: bool, is_hazardous: bool,
        is_purchasable: bool, is_sellable: bool, is_stockable: bool,
        inventory_asset_account_code: Option<&str>, expense_account_code: Option<&str>,
        cost_of_goods_sold_account: Option<&str>, revenue_account_code: Option<&str>,
        barcode: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<Item> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.items
                (organization_id, item_code, name, description, long_description,
                 category_id, category_code, item_type, uom, secondary_uom,
                 weight, weight_uom, volume, volume_uom,
                 list_price, standard_cost, min_order_quantity, max_order_quantity,
                 lead_time_days, shelf_life_days,
                 is_lot_controlled, is_serial_controlled, is_revision_controlled,
                 is_perishable, is_hazardous, is_purchasable, is_sellable, is_stockable,
                 inventory_asset_account_code, expense_account_code,
                 cost_of_goods_sold_account, revenue_account_code,
                 barcode, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,
                    $11,$12,$13,$14,$15,$16,$17,$18,$19,$20,
                    $21,$22,$23,$24,$25,$26,$27,$28,$29,$30,$31,$32,$33,$34)
            ON CONFLICT (organization_id, item_code) DO UPDATE
                SET name = $3, description = $4, long_description = $5,
                    category_id = $6, category_code = $7, item_type = $8, uom = $9,
                    list_price = $15, standard_cost = $16, is_active = true, updated_at = now()
            RETURNING *
            "#,
        )
        .bind(org_id).bind(item_code).bind(name).bind(description).bind(long_description)
        .bind(category_id).bind(category_code).bind(item_type).bind(uom).bind(secondary_uom)
        .bind(weight).bind(weight_uom).bind(volume).bind(volume_uom)
        .bind(list_price).bind(standard_cost).bind(min_order_quantity).bind(max_order_quantity)
        .bind(lead_time_days).bind(shelf_life_days)
        .bind(is_lot_controlled).bind(is_serial_controlled).bind(is_revision_controlled)
        .bind(is_perishable).bind(is_hazardous).bind(is_purchasable).bind(is_sellable).bind(is_stockable)
        .bind(inventory_asset_account_code).bind(expense_account_code)
        .bind(cost_of_goods_sold_account).bind(revenue_account_code)
        .bind(barcode).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_item(&row))
    }

    async fn get_item(&self, id: Uuid) -> AtlasResult<Option<Item>> {
        let row = sqlx::query("SELECT * FROM _atlas.items WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_item(&r)))
    }

    async fn get_item_by_code(&self, org_id: Uuid, item_code: &str) -> AtlasResult<Option<Item>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.items WHERE organization_id = $1 AND item_code = $2 AND is_active = true"
        )
        .bind(org_id).bind(item_code)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_item(&r)))
    }

    async fn list_items(&self, org_id: Uuid, category_code: Option<&str>, item_type: Option<&str>) -> AtlasResult<Vec<Item>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM _atlas.items
            WHERE organization_id = $1 AND is_active = true
              AND ($2::text IS NULL OR category_code = $2)
              AND ($3::text IS NULL OR item_type = $3)
            ORDER BY item_code
            "#,
        )
        .bind(org_id).bind(category_code).bind(item_type)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_item).collect())
    }

    async fn update_item_status(&self, id: Uuid, is_active: bool) -> AtlasResult<Item> {
        let row = sqlx::query(
            "UPDATE _atlas.items SET is_active = $2, updated_at = now() WHERE id = $1 RETURNING *"
        )
        .bind(id).bind(is_active)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_item(&row))
    }

    // ========================================================================
    // Subinventories
    // ========================================================================

    async fn create_subinventory(
        &self, org_id: Uuid, inventory_org_id: Uuid, code: &str, name: &str,
        description: Option<&str>, subinventory_type: &str,
        asset_subinventory: bool, quantity_tracked: bool,
        location_code: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<Subinventory> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.subinventories
                (organization_id, inventory_org_id, code, name, description,
                 subinventory_type, asset_subinventory, quantity_tracked,
                 location_code, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (organization_id, inventory_org_id, code) DO UPDATE
                SET name = $4, description = $5, subinventory_type = $6,
                    asset_subinventory = $7, quantity_tracked = $8, is_active = true, updated_at = now()
            RETURNING *
            "#,
        )
        .bind(org_id).bind(inventory_org_id).bind(code).bind(name).bind(description)
        .bind(subinventory_type).bind(asset_subinventory).bind(quantity_tracked)
        .bind(location_code).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_subinventory(&row))
    }

    async fn get_subinventory(&self, id: Uuid) -> AtlasResult<Option<Subinventory>> {
        let row = sqlx::query("SELECT * FROM _atlas.subinventories WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_subinventory(&r)))
    }

    async fn list_subinventories(&self, inventory_org_id: Uuid) -> AtlasResult<Vec<Subinventory>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.subinventories WHERE inventory_org_id = $1 AND is_active = true ORDER BY code"
        )
        .bind(inventory_org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_subinventory).collect())
    }

    // ========================================================================
    // Locators
    // ========================================================================

    async fn create_locator(
        &self, org_id: Uuid, subinventory_id: Uuid, code: &str,
        description: Option<&str>, picker_order: i32,
    ) -> AtlasResult<Locator> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.locators (organization_id, subinventory_id, code, description, picker_order)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (subinventory_id, code) DO UPDATE
                SET description = $4, picker_order = $5, is_active = true, updated_at = now()
            RETURNING *
            "#,
        )
        .bind(org_id).bind(subinventory_id).bind(code).bind(description).bind(picker_order)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_locator(&row))
    }

    async fn list_locators(&self, subinventory_id: Uuid) -> AtlasResult<Vec<Locator>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.locators WHERE subinventory_id = $1 AND is_active = true ORDER BY code"
        )
        .bind(subinventory_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_locator).collect())
    }

    // ========================================================================
    // On-Hand Balances
    // ========================================================================

    async fn get_on_hand_balance(
        &self, org_id: Uuid, inventory_org_id: Uuid, item_id: Uuid,
        subinventory_id: Uuid, locator_id: Option<Uuid>,
        lot_number: Option<&str>, serial_number: Option<&str>, revision: Option<&str>,
    ) -> AtlasResult<Option<OnHandBalance>> {
        let row = sqlx::query(
            r#"
            SELECT * FROM _atlas.on_hand_balances
            WHERE organization_id = $1 AND inventory_org_id = $2 AND item_id = $3
              AND subinventory_id = $4 AND COALESCE(locator_id, '00000000-0000-0000-0000-000000000000'::uuid) = COALESCE($5, '00000000-0000-0000-0000-000000000000'::uuid)
              AND COALESCE(lot_number, '') = COALESCE($6, '')
              AND COALESCE(serial_number, '') = COALESCE($7, '')
              AND COALESCE(revision, '') = COALESCE($8, '')
            "#,
        )
        .bind(org_id).bind(inventory_org_id).bind(item_id).bind(subinventory_id)
        .bind(locator_id).bind(lot_number).bind(serial_number).bind(revision)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_on_hand(&r)))
    }

    async fn upsert_on_hand_balance(
        &self, org_id: Uuid, inventory_org_id: Uuid, item_id: Uuid,
        subinventory_id: Uuid, locator_id: Option<Uuid>,
        lot_number: Option<&str>, serial_number: Option<&str>, revision: Option<&str>,
        quantity_delta: &str, unit_cost: &str,
    ) -> AtlasResult<OnHandBalance> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.on_hand_balances
                (organization_id, inventory_org_id, item_id, subinventory_id, locator_id,
                 lot_number, serial_number, revision, quantity, reserved_quantity,
                 available_quantity, unit_cost, total_value, last_transaction_date)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9::numeric, 0, $9::numeric, $10::numeric, $9::numeric * $10::numeric, now())
            ON CONFLICT (organization_id, inventory_org_id, item_id, subinventory_id, locator_id, lot_number, serial_number, revision)
            DO UPDATE SET
                quantity = on_hand_balances.quantity + $9::numeric,
                available_quantity = on_hand_balances.available_quantity + $9::numeric,
                unit_cost = $10::numeric,
                total_value = (on_hand_balances.quantity + $9::numeric) * $10::numeric,
                last_transaction_date = now(),
                updated_at = now()
            RETURNING *
            "#,
        )
        .bind(org_id).bind(inventory_org_id).bind(item_id).bind(subinventory_id).bind(locator_id)
        .bind(lot_number).bind(serial_number).bind(revision)
        .bind(quantity_delta).bind(unit_cost)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_on_hand(&row))
    }

    async fn list_on_hand_balances(&self, org_id: Uuid, item_id: Option<Uuid>, inventory_org_id: Option<Uuid>) -> AtlasResult<Vec<OnHandBalance>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM _atlas.on_hand_balances
            WHERE organization_id = $1
              AND ($2::uuid IS NULL OR item_id = $2)
              AND ($3::uuid IS NULL OR inventory_org_id = $3)
            ORDER BY item_id
            "#,
        )
        .bind(org_id).bind(item_id).bind(inventory_org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_on_hand).collect())
    }

    // ========================================================================
    // Transaction Types
    // ========================================================================

    async fn create_transaction_type(
        &self, org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        transaction_action: &str, source_type: &str, is_system: bool,
        created_by: Option<Uuid>,
    ) -> AtlasResult<InventoryTransactionType> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.inventory_transaction_types
                (organization_id, code, name, description, transaction_action, source_type, is_system, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (organization_id, code) DO UPDATE
                SET name = $3, description = $4, transaction_action = $5, source_type = $6, is_active = true, updated_at = now()
            RETURNING *
            "#,
        )
        .bind(org_id).bind(code).bind(name).bind(description)
        .bind(transaction_action).bind(source_type).bind(is_system).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_txn_type(&row))
    }

    async fn get_transaction_type(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<InventoryTransactionType>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.inventory_transaction_types WHERE organization_id = $1 AND code = $2 AND is_active = true"
        )
        .bind(org_id).bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_txn_type(&r)))
    }

    async fn list_transaction_types(&self, org_id: Uuid) -> AtlasResult<Vec<InventoryTransactionType>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.inventory_transaction_types WHERE organization_id = $1 AND is_active = true ORDER BY code"
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_txn_type).collect())
    }

    // ========================================================================
    // Inventory Transactions
    // ========================================================================

    async fn create_transaction(
        &self,
        org_id: Uuid, transaction_number: &str,
        transaction_type_id: Option<Uuid>, transaction_type_code: Option<&str>,
        transaction_action: &str, source_type: &str,
        source_id: Option<Uuid>, source_number: Option<&str>, source_line_id: Option<Uuid>,
        item_id: Uuid, item_code: Option<&str>, item_description: Option<&str>,
        from_inventory_org_id: Option<Uuid>, from_subinventory_id: Option<Uuid>, from_locator_id: Option<Uuid>,
        to_inventory_org_id: Option<Uuid>, to_subinventory_id: Option<Uuid>, to_locator_id: Option<Uuid>,
        quantity: &str, uom: &str, unit_cost: &str, total_cost: &str,
        lot_number: Option<&str>, serial_number: Option<&str>, revision: Option<&str>,
        transaction_date: chrono::DateTime<chrono::Utc>,
        reason_id: Option<Uuid>, reason_name: Option<&str>,
        notes: Option<&str>, status: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<InventoryTransaction> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.inventory_transactions
                (organization_id, transaction_number,
                 transaction_type_id, transaction_type_code, transaction_action, source_type,
                 source_id, source_number, source_line_id,
                 item_id, item_code, item_description,
                 from_inventory_org_id, from_subinventory_id, from_locator_id,
                 to_inventory_org_id, to_subinventory_id, to_locator_id,
                 quantity, uom, unit_cost, total_cost,
                 lot_number, serial_number, revision,
                 transaction_date, reason_id, reason_name,
                 notes, status, created_by)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22,$23,$24,$25,$26,$27,$28,$29,$30,$31)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(transaction_number)
        .bind(transaction_type_id).bind(transaction_type_code).bind(transaction_action).bind(source_type)
        .bind(source_id).bind(source_number).bind(source_line_id)
        .bind(item_id).bind(item_code).bind(item_description)
        .bind(from_inventory_org_id).bind(from_subinventory_id).bind(from_locator_id)
        .bind(to_inventory_org_id).bind(to_subinventory_id).bind(to_locator_id)
        .bind(quantity).bind(uom).bind(unit_cost).bind(total_cost)
        .bind(lot_number).bind(serial_number).bind(revision)
        .bind(transaction_date).bind(reason_id).bind(reason_name)
        .bind(notes).bind(status).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_transaction(&row))
    }

    async fn get_transaction(&self, id: Uuid) -> AtlasResult<Option<InventoryTransaction>> {
        let row = sqlx::query("SELECT * FROM _atlas.inventory_transactions WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_transaction(&r)))
    }

    async fn list_transactions(
        &self, org_id: Uuid, item_id: Option<Uuid>,
        transaction_action: Option<&str>, status: Option<&str>,
    ) -> AtlasResult<Vec<InventoryTransaction>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM _atlas.inventory_transactions
            WHERE organization_id = $1
              AND ($2::uuid IS NULL OR item_id = $2)
              AND ($3::text IS NULL OR transaction_action = $3)
              AND ($4::text IS NULL OR status = $4)
            ORDER BY transaction_date DESC
            "#,
        )
        .bind(org_id).bind(item_id).bind(transaction_action).bind(status)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_transaction).collect())
    }

    async fn update_transaction_status(&self, id: Uuid, status: &str, approved_by: Option<Uuid>) -> AtlasResult<InventoryTransaction> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.inventory_transactions
            SET status = $2,
                approved_by = COALESCE($3, approved_by),
                approved_at = CASE WHEN $3 IS NOT NULL THEN now() ELSE approved_at END,
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(status).bind(approved_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_transaction(&row))
    }

    // ========================================================================
    // Transaction Reasons
    // ========================================================================

    async fn create_transaction_reason(
        &self, org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        applicable_actions: serde_json::Value, created_by: Option<Uuid>,
    ) -> AtlasResult<TransactionReason> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.transaction_reasons
                (organization_id, code, name, description, applicable_actions, created_by)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (organization_id, code) DO UPDATE
                SET name = $3, description = $4, applicable_actions = $5, is_active = true, updated_at = now()
            RETURNING *
            "#,
        )
        .bind(org_id).bind(code).bind(name).bind(description)
        .bind(applicable_actions).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_txn_reason(&row))
    }

    async fn list_transaction_reasons(&self, org_id: Uuid) -> AtlasResult<Vec<TransactionReason>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.transaction_reasons WHERE organization_id = $1 AND is_active = true ORDER BY code"
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_txn_reason).collect())
    }

    // ========================================================================
    // Cycle Counts
    // ========================================================================

    async fn create_cycle_count(
        &self, org_id: Uuid, count_number: &str, name: &str, description: Option<&str>,
        inventory_org_id: Uuid, subinventory_id: Option<Uuid>, count_date: chrono::NaiveDate,
        count_method: &str, tolerance_percent: &str, created_by: Option<Uuid>,
    ) -> AtlasResult<CycleCountHeader> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.cycle_count_headers
                (organization_id, count_number, name, description,
                 inventory_org_id, subinventory_id, count_date,
                 count_method, tolerance_percent, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(count_number).bind(name).bind(description)
        .bind(inventory_org_id).bind(subinventory_id).bind(count_date)
        .bind(count_method).bind(tolerance_percent).bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_cycle_count(&row))
    }

    async fn get_cycle_count(&self, id: Uuid) -> AtlasResult<Option<CycleCountHeader>> {
        let row = sqlx::query("SELECT * FROM _atlas.cycle_count_headers WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_cycle_count(&r)))
    }

    async fn list_cycle_counts(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<CycleCountHeader>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM _atlas.cycle_count_headers
            WHERE organization_id = $1 AND ($2::text IS NULL OR status = $2)
            ORDER BY count_date DESC
            "#,
        )
        .bind(org_id).bind(status)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_cycle_count).collect())
    }

    async fn update_cycle_count_status(&self, id: Uuid, status: &str) -> AtlasResult<CycleCountHeader> {
        let row = sqlx::query(
            "UPDATE _atlas.cycle_count_headers SET status = $2, updated_at = now() WHERE id = $1 RETURNING *"
        )
        .bind(id).bind(status)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_cycle_count(&row))
    }

    async fn update_cycle_count_summary(
        &self, id: Uuid, total_items: i32, counted_items: i32,
        matched_items: i32, mismatched_items: i32,
    ) -> AtlasResult<CycleCountHeader> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.cycle_count_headers
            SET total_items = $2, counted_items = $3, matched_items = $4, mismatched_items = $5, updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(total_items).bind(counted_items).bind(matched_items).bind(mismatched_items)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_cycle_count(&row))
    }

    // ========================================================================
    // Cycle Count Lines
    // ========================================================================

    async fn create_cycle_count_line(
        &self, org_id: Uuid, cycle_count_id: Uuid, line_number: i32,
        item_id: Uuid, item_code: Option<&str>, item_description: Option<&str>,
        subinventory_id: Option<Uuid>, locator_id: Option<Uuid>,
        lot_number: Option<&str>, revision: Option<&str>,
        system_quantity: &str,
    ) -> AtlasResult<CycleCountLine> {
        let row = sqlx::query(
            r#"
            INSERT INTO _atlas.cycle_count_lines
                (organization_id, cycle_count_id, line_number,
                 item_id, item_code, item_description,
                 subinventory_id, locator_id, lot_number, revision,
                 system_quantity)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING *
            "#,
        )
        .bind(org_id).bind(cycle_count_id).bind(line_number)
        .bind(item_id).bind(item_code).bind(item_description)
        .bind(subinventory_id).bind(locator_id).bind(lot_number).bind(revision)
        .bind(system_quantity)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_cycle_count_line(&row))
    }

    async fn list_cycle_count_lines(&self, cycle_count_id: Uuid) -> AtlasResult<Vec<CycleCountLine>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.cycle_count_lines WHERE cycle_count_id = $1 ORDER BY line_number"
        )
        .bind(cycle_count_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_cycle_count_line).collect())
    }

    async fn update_cycle_count_line_count(
        &self, id: Uuid, count_number: i32, count_quantity: &str, counted_by: Option<Uuid>,
    ) -> AtlasResult<CycleCountLine> {
        let qty_col = format!("count_quantity_{}", count_number);
        let date_col = format!("count_date_{}", count_number);
        let by_col = format!("counted_by_{}", count_number);
        let sql = format!(
            r#"UPDATE _atlas.cycle_count_lines
               SET {} = $2, {} = now(), {} = $3, status = 'counted', updated_at = now()
               WHERE id = $1 RETURNING *"#,
            qty_col, date_col, by_col
        );
        let row = sqlx::query(&sql)
            .bind(id).bind(count_quantity).bind(counted_by)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_cycle_count_line(&row))
    }

    async fn approve_cycle_count_line(&self, id: Uuid, approved_quantity: &str) -> AtlasResult<CycleCountLine> {
        let row = sqlx::query(
            r#"
            UPDATE _atlas.cycle_count_lines
            SET approved_quantity = $2,
                variance_quantity = $2::numeric - system_quantity,
                variance_percent = CASE WHEN system_quantity = 0 THEN 0 ELSE (($2::numeric - system_quantity) / system_quantity * 100) END,
                is_matched = ($2::numeric = system_quantity),
                status = 'approved',
                updated_at = now()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id).bind(approved_quantity)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| atlas_shared::AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_cycle_count_line(&row))
    }
}
