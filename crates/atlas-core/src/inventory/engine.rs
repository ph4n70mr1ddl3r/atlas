//! Inventory Management Engine
//!
//! Manages the full lifecycle of inventory:
//! - Inventory organizations (warehouses, stores)
//! - Item categories and items
//! - Subinventories and locators
//! - On-hand balance tracking
//! - Inventory transactions (receive, issue, transfer, adjust)
//! - Transaction types and reasons
//! - Cycle counting with variance analysis
//! - Dashboard summary
//!
//! Oracle Fusion Cloud ERP equivalent: SCM > Inventory Management

use atlas_shared::{
    InventoryOrganization, ItemCategory, Item, Subinventory, Locator,
    OnHandBalance, InventoryTransactionType, InventoryTransaction,
    CycleCountHeader, CycleCountLine, TransactionReason,
    InventoryDashboardSummary,
    AtlasError, AtlasResult,
};
use super::InventoryRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Valid organization types
#[allow(dead_code)]
const VALID_ORG_TYPES: &[&str] = &[
    "warehouse", "store", "distribution_center", "manufacturing", "other",
];

/// Valid item types
#[allow(dead_code)]
const VALID_ITEM_TYPES: &[&str] = &[
    "inventory", "non_inventory", "service", "expense", "capital",
];

/// Valid transaction actions
#[allow(dead_code)]
const VALID_TRANSACTION_ACTIONS: &[&str] = &[
    "receive", "issue", "transfer", "adjustment", "return_to_vendor",
    "return_to_customer", "cycle_count_adjustment", "misc_receipt", "misc_issue",
];

/// Valid subinventory types
#[allow(dead_code)]
const VALID_SUBINVENTORY_TYPES: &[&str] = &[
    "storage", "receiving", "staging", "inspection", "packing", "other",
];

/// Valid transaction statuses
#[allow(dead_code)]
const VALID_TXN_STATUSES: &[&str] = &[
    "pending", "approved", "processed", "cancelled",
];

/// Valid cycle count statuses
#[allow(dead_code)]
const VALID_CYCLE_COUNT_STATUSES: &[&str] = &[
    "draft", "in_progress", "completed", "cancelled",
];

/// Valid cycle count methods
#[allow(dead_code)]
const VALID_COUNT_METHODS: &[&str] = &[
    "full", "abc", "random", "by_category",
];

/// Inventory Management Engine
pub struct InventoryEngine {
    repository: Arc<dyn InventoryRepository>,
}

impl InventoryEngine {
    pub fn new(repository: Arc<dyn InventoryRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Inventory Organizations
    // ========================================================================

    /// Create a new inventory organization
    pub async fn create_inventory_org(
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
        if code.is_empty() || name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Organization code and name are required".to_string(),
            ));
        }
        if !VALID_ORG_TYPES.contains(&org_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid org type '{}'. Must be one of: {}", org_type, VALID_ORG_TYPES.join(", ")
            )));
        }

        info!("Creating inventory organization '{}' for org {}", code, org_id);

        self.repository.create_inventory_org(
            org_id, code, name, description, org_type, location_code, address,
            default_subinventory_code, default_currency_code,
            requires_approval_for_issues, requires_approval_for_transfers,
            enable_lot_control, enable_serial_control, enable_revision_control,
            created_by,
        ).await
    }

    /// Get an inventory organization by code
    pub async fn get_inventory_org(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<InventoryOrganization>> {
        self.repository.get_inventory_org(org_id, code).await
    }

    /// List all inventory organizations
    pub async fn list_inventory_orgs(&self, org_id: Uuid) -> AtlasResult<Vec<InventoryOrganization>> {
        self.repository.list_inventory_orgs(org_id).await
    }

    /// Delete (deactivate) an inventory organization
    pub async fn delete_inventory_org(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        info!("Deactivating inventory organization '{}' in org {}", code, org_id);
        self.repository.delete_inventory_org(org_id, code).await
    }

    // ========================================================================
    // Item Categories
    // ========================================================================

    /// Create an item category
    pub async fn create_item_category(
        &self, org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        parent_category_id: Option<Uuid>, track_as_asset: bool, created_by: Option<Uuid>,
    ) -> AtlasResult<ItemCategory> {
        if code.is_empty() || name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Category code and name are required".to_string(),
            ));
        }
        info!("Creating item category '{}' for org {}", code, org_id);
        self.repository.create_item_category(
            org_id, code, name, description, parent_category_id, track_as_asset, created_by,
        ).await
    }

    /// Get an item category by code
    pub async fn get_item_category(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<ItemCategory>> {
        self.repository.get_item_category(org_id, code).await
    }

    /// List all item categories
    pub async fn list_item_categories(&self, org_id: Uuid) -> AtlasResult<Vec<ItemCategory>> {
        self.repository.list_item_categories(org_id).await
    }

    // ========================================================================
    // Items
    // ========================================================================

    /// Create a new item
    pub async fn create_item(
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
        if item_code.is_empty() || name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Item code and name are required".to_string(),
            ));
        }
        if !VALID_ITEM_TYPES.contains(&item_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid item type '{}'. Must be one of: {}", item_type, VALID_ITEM_TYPES.join(", ")
            )));
        }
        if uom.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Unit of measure is required".to_string(),
            ));
        }

        info!("Creating item '{}' for org {}", item_code, org_id);

        self.repository.create_item(
            org_id, item_code, name, description, long_description,
            category_id, category_code, item_type, uom, secondary_uom,
            weight, weight_uom, volume, volume_uom,
            list_price, standard_cost, min_order_quantity, max_order_quantity,
            lead_time_days, shelf_life_days,
            is_lot_controlled, is_serial_controlled, is_revision_controlled,
            is_perishable, is_hazardous, is_purchasable, is_sellable, is_stockable,
            inventory_asset_account_code, expense_account_code,
            cost_of_goods_sold_account, revenue_account_code,
            barcode, created_by,
        ).await
    }

    /// Get item by ID
    pub async fn get_item(&self, id: Uuid) -> AtlasResult<Option<Item>> {
        self.repository.get_item(id).await
    }

    /// Get item by code
    pub async fn get_item_by_code(&self, org_id: Uuid, item_code: &str) -> AtlasResult<Option<Item>> {
        self.repository.get_item_by_code(org_id, item_code).await
    }

    /// List items with optional filters
    pub async fn list_items(
        &self, org_id: Uuid, category_code: Option<&str>, item_type: Option<&str>,
    ) -> AtlasResult<Vec<Item>> {
        self.repository.list_items(org_id, category_code, item_type).await
    }

    /// Deactivate an item
    pub async fn deactivate_item(&self, id: Uuid) -> AtlasResult<Item> {
        info!("Deactivating item {}", id);
        self.repository.update_item_status(id, false).await
    }

    // ========================================================================
    // Subinventories
    // ========================================================================

    /// Create a subinventory
    pub async fn create_subinventory(
        &self, org_id: Uuid, inventory_org_id: Uuid, code: &str, name: &str,
        description: Option<&str>, subinventory_type: &str,
        asset_subinventory: bool, quantity_tracked: bool,
        location_code: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<Subinventory> {
        if code.is_empty() || name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Subinventory code and name are required".to_string(),
            ));
        }
        if !VALID_SUBINVENTORY_TYPES.contains(&subinventory_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid subinventory type '{}'. Must be one of: {}",
                subinventory_type, VALID_SUBINVENTORY_TYPES.join(", ")
            )));
        }

        // Verify inventory org exists
        let _inv_org = self.repository.get_inventory_org(org_id, code).await?;
        // We don't require it to exist for flexibility, but log if not found

        info!("Creating subinventory '{}' in org {}", code, org_id);

        self.repository.create_subinventory(
            org_id, inventory_org_id, code, name, description,
            subinventory_type, asset_subinventory, quantity_tracked,
            location_code, created_by,
        ).await
    }

    /// List subinventories for an inventory org
    pub async fn list_subinventories(&self, inventory_org_id: Uuid) -> AtlasResult<Vec<Subinventory>> {
        self.repository.list_subinventories(inventory_org_id).await
    }

    // ========================================================================
    // Locators
    // ========================================================================

    /// Create a locator
    pub async fn create_locator(
        &self, org_id: Uuid, subinventory_id: Uuid, code: &str,
        description: Option<&str>, picker_order: i32,
    ) -> AtlasResult<Locator> {
        if code.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Locator code is required".to_string(),
            ));
        }
        info!("Creating locator '{}' for subinventory {}", code, subinventory_id);
        self.repository.create_locator(org_id, subinventory_id, code, description, picker_order).await
    }

    /// List locators for a subinventory
    pub async fn list_locators(&self, subinventory_id: Uuid) -> AtlasResult<Vec<Locator>> {
        self.repository.list_locators(subinventory_id).await
    }

    // ========================================================================
    // On-Hand Balances
    // ========================================================================

    /// Get on-hand balance for a specific item at a specific location
    pub async fn get_on_hand_balance(
        &self, org_id: Uuid, inventory_org_id: Uuid, item_id: Uuid,
        subinventory_id: Uuid, locator_id: Option<Uuid>,
        lot_number: Option<&str>, serial_number: Option<&str>, revision: Option<&str>,
    ) -> AtlasResult<Option<OnHandBalance>> {
        self.repository.get_on_hand_balance(
            org_id, inventory_org_id, item_id, subinventory_id,
            locator_id, lot_number, serial_number, revision,
        ).await
    }

    /// List on-hand balances with optional filters
    pub async fn list_on_hand_balances(
        &self, org_id: Uuid, item_id: Option<Uuid>, inventory_org_id: Option<Uuid>,
    ) -> AtlasResult<Vec<OnHandBalance>> {
        self.repository.list_on_hand_balances(org_id, item_id, inventory_org_id).await
    }

    // ========================================================================
    // Transaction Types
    // ========================================================================

    /// Create a transaction type
    pub async fn create_transaction_type(
        &self, org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        transaction_action: &str, source_type: &str, is_system: bool,
        created_by: Option<Uuid>,
    ) -> AtlasResult<InventoryTransactionType> {
        if code.is_empty() || name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Transaction type code and name are required".to_string(),
            ));
        }
        if !VALID_TRANSACTION_ACTIONS.contains(&transaction_action) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid transaction action '{}'. Must be one of: {}",
                transaction_action, VALID_TRANSACTION_ACTIONS.join(", ")
            )));
        }
        info!("Creating transaction type '{}' for org {}", code, org_id);
        self.repository.create_transaction_type(
            org_id, code, name, description, transaction_action, source_type, is_system, created_by,
        ).await
    }

    /// List transaction types
    pub async fn list_transaction_types(&self, org_id: Uuid) -> AtlasResult<Vec<InventoryTransactionType>> {
        self.repository.list_transaction_types(org_id).await
    }

    // ========================================================================
    // Inventory Transactions
    // ========================================================================

    /// Process a receipt transaction (goods received into inventory)
    pub async fn receive_item(
        &self,
        org_id: Uuid,
        item_id: Uuid,
        item_code: Option<&str>,
        item_description: Option<&str>,
        to_inventory_org_id: Uuid,
        to_subinventory_id: Uuid,
        to_locator_id: Option<Uuid>,
        quantity: &str,
        uom: &str,
        unit_cost: &str,
        lot_number: Option<&str>,
        serial_number: Option<&str>,
        revision: Option<&str>,
        source_type: &str,
        source_id: Option<Uuid>,
        source_number: Option<&str>,
        reason_id: Option<Uuid>,
        reason_name: Option<&str>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<InventoryTransaction> {
        let qty: f64 = quantity.parse().map_err(|_| AtlasError::ValidationFailed(
            "Quantity must be a valid number".to_string(),
        ))?;
        if qty <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Quantity must be positive for receipt".to_string(),
            ));
        }
        let cost: f64 = unit_cost.parse().map_err(|_| AtlasError::ValidationFailed(
            "Unit cost must be a valid number".to_string(),
        ))?;
        if cost < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Unit cost must be non-negative".to_string(),
            ));
        }
        let total_cost = format!("{:.2}", qty * cost);
        let txn_number = format!("RCV-{}", Uuid::new_v4().to_string()[..8].to_uppercase());

        info!("Receiving {} {} of item {} into org {}", quantity, uom, item_id, to_inventory_org_id);

        // Create the transaction
        let txn = self.repository.create_transaction(
            org_id, &txn_number,
            None, None,
            "receive", source_type,
            source_id, source_number, None,
            item_id, item_code, item_description,
            None, None, None,
            Some(to_inventory_org_id), Some(to_subinventory_id), to_locator_id,
            quantity, uom, unit_cost, &total_cost,
            lot_number, serial_number, revision,
            chrono::Utc::now(),
            reason_id, reason_name,
            notes, "processed",
            created_by,
        ).await?;

        // Update on-hand balance
        self.repository.upsert_on_hand_balance(
            org_id, to_inventory_org_id, item_id, to_subinventory_id, to_locator_id,
            lot_number, serial_number, revision,
            quantity, unit_cost,
        ).await?;

        Ok(txn)
    }

    /// Process an issue transaction (goods issued from inventory)
    pub async fn issue_item(
        &self,
        org_id: Uuid,
        item_id: Uuid,
        item_code: Option<&str>,
        item_description: Option<&str>,
        from_inventory_org_id: Uuid,
        from_subinventory_id: Uuid,
        from_locator_id: Option<Uuid>,
        quantity: &str,
        uom: &str,
        unit_cost: &str,
        lot_number: Option<&str>,
        serial_number: Option<&str>,
        revision: Option<&str>,
        source_type: &str,
        source_id: Option<Uuid>,
        source_number: Option<&str>,
        reason_id: Option<Uuid>,
        reason_name: Option<&str>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<InventoryTransaction> {
        let qty: f64 = quantity.parse().map_err(|_| AtlasError::ValidationFailed(
            "Quantity must be a valid number".to_string(),
        ))?;
        if qty <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Quantity must be positive for issue".to_string(),
            ));
        }

        // Check on-hand balance
        let balance = self.repository.get_on_hand_balance(
            org_id, from_inventory_org_id, item_id, from_subinventory_id,
            from_locator_id, lot_number, serial_number, revision,
        ).await?;

        let available: f64 = balance
            .as_ref()
            .map(|b| b.available_quantity.parse::<f64>().unwrap_or(0.0))
            .unwrap_or(0.0);

        if available < qty {
            return Err(AtlasError::ValidationFailed(format!(
                "Insufficient on-hand quantity. Available: {}, Requested: {}", available, qty
            )));
        }

        let cost: f64 = unit_cost.parse().unwrap_or(0.0);
        let total_cost = format!("{:.2}", qty * cost);
        let txn_number = format!("ISS-{}", Uuid::new_v4().to_string()[..8].to_uppercase());

        info!("Issuing {} {} of item {} from org {}", quantity, uom, item_id, from_inventory_org_id);

        let status = "processed";

        // Create transaction
        let txn = self.repository.create_transaction(
            org_id, &txn_number,
            None, None,
            "issue", source_type,
            source_id, source_number, None,
            item_id, item_code, item_description,
            Some(from_inventory_org_id), Some(from_subinventory_id), from_locator_id,
            None, None, None,
            quantity, uom, unit_cost, &total_cost,
            lot_number, serial_number, revision,
            chrono::Utc::now(),
            reason_id, reason_name,
            notes, status,
            created_by,
        ).await?;

        // Update on-hand (negative delta)
        let neg_qty = format!("-{}", quantity);
        self.repository.upsert_on_hand_balance(
            org_id, from_inventory_org_id, item_id, from_subinventory_id, from_locator_id,
            lot_number, serial_number, revision,
            &neg_qty, unit_cost,
        ).await?;

        Ok(txn)
    }

    /// Process a transfer transaction (move goods between subinventories/orgs)
    pub async fn transfer_item(
        &self,
        org_id: Uuid,
        item_id: Uuid,
        item_code: Option<&str>,
        item_description: Option<&str>,
        from_inventory_org_id: Uuid,
        from_subinventory_id: Uuid,
        from_locator_id: Option<Uuid>,
        to_inventory_org_id: Uuid,
        to_subinventory_id: Uuid,
        to_locator_id: Option<Uuid>,
        quantity: &str,
        uom: &str,
        unit_cost: &str,
        lot_number: Option<&str>,
        serial_number: Option<&str>,
        revision: Option<&str>,
        reason_id: Option<Uuid>,
        reason_name: Option<&str>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<InventoryTransaction> {
        let qty: f64 = quantity.parse().map_err(|_| AtlasError::ValidationFailed(
            "Quantity must be a valid number".to_string(),
        ))?;
        if qty <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Quantity must be positive for transfer".to_string(),
            ));
        }

        // Check source balance
        let balance = self.repository.get_on_hand_balance(
            org_id, from_inventory_org_id, item_id, from_subinventory_id,
            from_locator_id, lot_number, serial_number, revision,
        ).await?;

        let available: f64 = balance
            .as_ref()
            .map(|b| b.available_quantity.parse::<f64>().unwrap_or(0.0))
            .unwrap_or(0.0);

        if available < qty {
            return Err(AtlasError::ValidationFailed(format!(
                "Insufficient on-hand quantity for transfer. Available: {}, Requested: {}", available, qty
            )));
        }

        let cost: f64 = unit_cost.parse().unwrap_or(0.0);
        let total_cost = format!("{:.2}", qty * cost);
        let txn_number = format!("XFR-{}", Uuid::new_v4().to_string()[..8].to_uppercase());

        info!("Transferring {} {} of item {} from subinv {} to {}",
            quantity, uom, item_id, from_subinventory_id, to_subinventory_id);

        let txn = self.repository.create_transaction(
            org_id, &txn_number,
            None, None,
            "transfer", "manual",
            None, None, None,
            item_id, item_code, item_description,
            Some(from_inventory_org_id), Some(from_subinventory_id), from_locator_id,
            Some(to_inventory_org_id), Some(to_subinventory_id), to_locator_id,
            quantity, uom, unit_cost, &total_cost,
            lot_number, serial_number, revision,
            chrono::Utc::now(),
            reason_id, reason_name,
            notes, "processed",
            created_by,
        ).await?;

        // Decrease source balance
        let neg_qty = format!("-{}", quantity);
        self.repository.upsert_on_hand_balance(
            org_id, from_inventory_org_id, item_id, from_subinventory_id, from_locator_id,
            lot_number, serial_number, revision,
            &neg_qty, unit_cost,
        ).await?;

        // Increase destination balance
        self.repository.upsert_on_hand_balance(
            org_id, to_inventory_org_id, item_id, to_subinventory_id, to_locator_id,
            lot_number, serial_number, revision,
            quantity, unit_cost,
        ).await?;

        Ok(txn)
    }

    /// Process an adjustment transaction (correct on-hand quantity)
    pub async fn adjust_item(
        &self,
        org_id: Uuid,
        item_id: Uuid,
        item_code: Option<&str>,
        item_description: Option<&str>,
        inventory_org_id: Uuid,
        subinventory_id: Uuid,
        locator_id: Option<Uuid>,
        quantity_delta: &str,
        uom: &str,
        unit_cost: &str,
        lot_number: Option<&str>,
        serial_number: Option<&str>,
        revision: Option<&str>,
        reason_id: Option<Uuid>,
        reason_name: Option<&str>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<InventoryTransaction> {
        let delta: f64 = quantity_delta.parse().map_err(|_| AtlasError::ValidationFailed(
            "Quantity delta must be a valid number".to_string(),
        ))?;
        if delta == 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Quantity delta cannot be zero for adjustment".to_string(),
            ));
        }

        let cost: f64 = unit_cost.parse().unwrap_or(0.0);
        let total_cost = format!("{:.2}", delta * cost);
        let txn_number = format!("ADJ-{}", Uuid::new_v4().to_string()[..8].to_uppercase());

        info!("Adjusting item {} by {} in org {}", item_id, quantity_delta, inventory_org_id);

        let txn = self.repository.create_transaction(
            org_id, &txn_number,
            None, None,
            "adjustment", "manual",
            None, None, None,
            item_id, item_code, item_description,
            None, None, None,
            Some(inventory_org_id), Some(subinventory_id), locator_id,
            quantity_delta, uom, unit_cost, &total_cost,
            lot_number, serial_number, revision,
            chrono::Utc::now(),
            reason_id, reason_name,
            notes, "processed",
            created_by,
        ).await?;

        // Update on-hand balance
        self.repository.upsert_on_hand_balance(
            org_id, inventory_org_id, item_id, subinventory_id, locator_id,
            lot_number, serial_number, revision,
            quantity_delta, unit_cost,
        ).await?;

        Ok(txn)
    }

    /// List transactions with optional filters
    pub async fn list_transactions(
        &self, org_id: Uuid, item_id: Option<Uuid>,
        transaction_action: Option<&str>, status: Option<&str>,
    ) -> AtlasResult<Vec<InventoryTransaction>> {
        if let Some(a) = transaction_action {
            if !VALID_TRANSACTION_ACTIONS.contains(&a) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid action '{}'. Must be one of: {}", a, VALID_TRANSACTION_ACTIONS.join(", ")
                )));
            }
        }
        if let Some(s) = status {
            if !VALID_TXN_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_TXN_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_transactions(org_id, item_id, transaction_action, status).await
    }

    /// Get a transaction by ID
    pub async fn get_transaction(&self, id: Uuid) -> AtlasResult<Option<InventoryTransaction>> {
        self.repository.get_transaction(id).await
    }

    // ========================================================================
    // Transaction Reasons
    // ========================================================================

    /// Create a transaction reason
    pub async fn create_transaction_reason(
        &self, org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        applicable_actions: serde_json::Value, created_by: Option<Uuid>,
    ) -> AtlasResult<TransactionReason> {
        if code.is_empty() || name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Reason code and name are required".to_string(),
            ));
        }
        info!("Creating transaction reason '{}' for org {}", code, org_id);
        self.repository.create_transaction_reason(
            org_id, code, name, description, applicable_actions, created_by,
        ).await
    }

    /// List transaction reasons
    pub async fn list_transaction_reasons(&self, org_id: Uuid) -> AtlasResult<Vec<TransactionReason>> {
        self.repository.list_transaction_reasons(org_id).await
    }

    // ========================================================================
    // Cycle Counts
    // ========================================================================

    /// Create a cycle count
    pub async fn create_cycle_count(
        &self,
        org_id: Uuid, name: &str, description: Option<&str>,
        inventory_org_id: Uuid, subinventory_id: Option<Uuid>,
        count_date: chrono::NaiveDate,
        count_method: &str, tolerance_percent: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CycleCountHeader> {
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Cycle count name is required".to_string(),
            ));
        }
        if !VALID_COUNT_METHODS.contains(&count_method) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid count method '{}'. Must be one of: {}",
                count_method, VALID_COUNT_METHODS.join(", ")
            )));
        }

        let count_number = format!("CC-{}", Uuid::new_v4().to_string()[..8].to_uppercase());

        info!("Creating cycle count '{}' for org {}", count_number, org_id);

        self.repository.create_cycle_count(
            org_id, &count_number, name, description,
            inventory_org_id, subinventory_id, count_date,
            count_method, tolerance_percent, created_by,
        ).await
    }

    /// Get a cycle count by ID
    pub async fn get_cycle_count(&self, id: Uuid) -> AtlasResult<Option<CycleCountHeader>> {
        self.repository.get_cycle_count(id).await
    }

    /// List cycle counts
    pub async fn list_cycle_counts(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<CycleCountHeader>> {
        self.repository.list_cycle_counts(org_id, status).await
    }

    /// Add an item line to a cycle count
    pub async fn add_cycle_count_line(
        &self,
        org_id: Uuid,
        cycle_count_id: Uuid,
        item_id: Uuid,
        item_code: Option<&str>,
        item_description: Option<&str>,
        subinventory_id: Option<Uuid>,
        locator_id: Option<Uuid>,
        lot_number: Option<&str>,
        revision: Option<&str>,
    ) -> AtlasResult<CycleCountLine> {
        let cc = self.repository.get_cycle_count(cycle_count_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Cycle count {} not found", cycle_count_id)
            ))?;

        if cc.status == "completed" || cc.status == "cancelled" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot add lines to cycle count in '{}' status", cc.status
            )));
        }

        // Get system quantity from on-hand
        let system_qty = if let Some(sub_id) = subinventory_id {
            let balance = self.repository.get_on_hand_balance(
                org_id, cc.inventory_org_id, item_id, sub_id,
                locator_id, lot_number, None, revision,
            ).await?;
            balance.map(|b| b.quantity).unwrap_or_else(|| "0".to_string())
        } else {
            "0".to_string()
        };

        let existing_lines = self.repository.list_cycle_count_lines(cycle_count_id).await?;
        let line_number = (existing_lines.len() as i32) + 1;

        info!("Adding line {} to cycle count {}", line_number, cc.count_number);

        let line = self.repository.create_cycle_count_line(
            org_id, cycle_count_id, line_number,
            item_id, item_code, item_description,
            subinventory_id, locator_id, lot_number, revision,
            &system_qty,
        ).await?;

        // Update summary
        let all_lines = self.repository.list_cycle_count_lines(cycle_count_id).await?;
        self.repository.update_cycle_count_summary(
            cycle_count_id,
            all_lines.len() as i32,
            cc.counted_items,
            cc.matched_items,
            cc.mismatched_items,
        ).await?;

        Ok(line)
    }

    /// Record a count for a cycle count line
    pub async fn record_count(
        &self,
        line_id: Uuid,
        count_number: i32,
        count_quantity: &str,
        counted_by: Option<Uuid>,
    ) -> AtlasResult<CycleCountLine> {
        if !(1..=3).contains(&count_number) {
            return Err(AtlasError::ValidationFailed(
                "Count number must be 1, 2, or 3".to_string(),
            ));
        }
        let qty: f64 = count_quantity.parse().map_err(|_| AtlasError::ValidationFailed(
            "Count quantity must be a valid number".to_string(),
        ))?;
        if qty < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Count quantity must be non-negative".to_string(),
            ));
        }

        let line = self.repository.update_cycle_count_line_count(
            line_id, count_number, count_quantity, counted_by,
        ).await?;

        // Update cycle count header summary
        let all_lines = self.repository.list_cycle_count_lines(line.cycle_count_id).await?;
        let counted = all_lines.iter().filter(|l| l.status != "pending").count() as i32;
        self.repository.update_cycle_count_summary(
            line.cycle_count_id,
            all_lines.len() as i32,
            counted,
            0, 0, // matched/mismatched determined on approval
        ).await?;

        Ok(line)
    }

    /// Approve a cycle count line and generate adjustment if needed
    pub async fn approve_cycle_count_line(
        &self,
        line_id: Uuid,
        approved_quantity: &str,
    ) -> AtlasResult<CycleCountLine> {
        let approved: f64 = approved_quantity.parse().map_err(|_| AtlasError::ValidationFailed(
            "Approved quantity must be a valid number".to_string(),
        ))?;
        if approved < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Approved quantity must be non-negative".to_string(),
            ));
        }

        let line = self.repository.approve_cycle_count_line(line_id, approved_quantity).await?;

        // If there's a variance, generate an adjustment transaction
        if !line.is_matched {
            if let (Some(_variance_qty), Some(ref variance_str)) = (line.variance_quantity.as_ref(), line.variance_quantity.as_ref()) {
                let variance: f64 = variance_str.parse().unwrap_or(0.0);
                if variance.abs() > 0.001 {
                    let cc = self.repository.get_cycle_count(line.cycle_count_id).await?
                        .ok_or_else(|| AtlasError::EntityNotFound(
                            format!("Cycle count {} not found", line.cycle_count_id)
                        ))?;

                    let _cc_id = line.cycle_count_id.to_string();
                    let _adj_tx = self.adjust_item(
                        cc.organization_id,
                        line.item_id,
                        line.item_code.as_deref(),
                        line.item_description.as_deref(),
                        cc.inventory_org_id,
                        line.subinventory_id.unwrap_or_else(Uuid::nil),
                        line.locator_id,
                        variance_str,
                        "EA",
                        "0",
                        line.lot_number.as_deref(),
                        None,
                        line.revision.as_deref(),
                        None,
                        Some(&format!("Cycle count adjustment: {}", cc.count_number)),
                        Some(&format!("Variance of {} for item {}", variance_str, line.item_code.as_deref().unwrap_or("unknown"))),
                        None,
                    ).await?;

                    // TODO: update cycle count line with adjustment_transaction_id
                }
            }
        }

        // Update header summary
        let all_lines = self.repository.list_cycle_count_lines(line.cycle_count_id).await?;
        let matched = all_lines.iter().filter(|l| l.is_matched).count() as i32;
        let mismatched = all_lines.iter().filter(|l| !l.is_matched && l.status == "approved").count() as i32;
        self.repository.update_cycle_count_summary(
            line.cycle_count_id,
            all_lines.len() as i32,
            all_lines.iter().filter(|l| l.status != "pending").count() as i32,
            matched,
            mismatched,
        ).await?;

        Ok(line)
    }

    /// List cycle count lines
    pub async fn list_cycle_count_lines(&self, cycle_count_id: Uuid) -> AtlasResult<Vec<CycleCountLine>> {
        self.repository.list_cycle_count_lines(cycle_count_id).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get inventory dashboard summary
    pub async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<InventoryDashboardSummary> {
        let all_items = self.repository.list_items(org_id, None, None).await?;
        let all_orgs = self.repository.list_inventory_orgs(org_id).await?;
        let all_txns = self.repository.list_transactions(org_id, None, None, None).await?;
        let all_on_hand = self.repository.list_on_hand_balances(org_id, None, None).await?;
        let cycle_counts = self.repository.list_cycle_counts(org_id, Some("in_progress")).await?;

        let total_items = all_items.len() as i32;
        let active_items = all_items.iter().filter(|i| i.is_active).count() as i32;

        let total_on_hand_value: f64 = all_on_hand.iter()
            .map(|b| b.total_value.parse::<f64>().unwrap_or(0.0))
            .sum();

        let pending_txns = all_txns.iter().filter(|t| t.status == "pending").count() as i32;
        let processed_txns = all_txns.iter().filter(|t| t.status == "processed").count() as i32;

        // Items by type
        let mut by_type: std::collections::HashMap<String, i32> = std::collections::HashMap::new();
        for i in &all_items {
            *by_type.entry(i.item_type.clone()).or_insert(0) += 1;
        }
        let items_by_type: serde_json::Value = by_type.into_iter()
            .map(|(k, v)| serde_json::json!({"type": k, "count": v}))
            .collect();

        // Items by category
        let mut by_cat: std::collections::HashMap<String, i32> = std::collections::HashMap::new();
        for i in &all_items {
            let key = i.category_code.clone().unwrap_or_else(|| "uncategorized".to_string());
            *by_cat.entry(key).or_insert(0) += 1;
        }
        let items_by_category: serde_json::Value = by_cat.into_iter()
            .map(|(k, v)| serde_json::json!({"category": k, "count": v}))
            .collect();

        // Transactions by action
        let mut by_action: std::collections::HashMap<String, i32> = std::collections::HashMap::new();
        for t in &all_txns {
            *by_action.entry(t.transaction_action.clone()).or_insert(0) += 1;
        }
        let transactions_by_action: serde_json::Value = by_action.into_iter()
            .map(|(k, v)| serde_json::json!({"action": k, "count": v}))
            .collect();

        // Top items by on-hand value
        let mut by_item_value: std::collections::HashMap<Uuid, (String, f64)> = std::collections::HashMap::new();
        for b in &all_on_hand {
            let entry = by_item_value.entry(b.item_id).or_insert(("".to_string(), 0.0));
            entry.1 += b.total_value.parse::<f64>().unwrap_or(0.0);
        }
        let mut top_items: Vec<_> = by_item_value.into_iter()
            .map(|(id, (_, v))| serde_json::json!({"item_id": id.to_string(), "value": format!("{:.2}", v)}))
            .collect();
        top_items.sort_by(|a, b| {
            let va: f64 = a["value"].as_str().unwrap_or("0").parse().unwrap_or(0.0);
            let vb: f64 = b["value"].as_str().unwrap_or("0").parse().unwrap_or(0.0);
            vb.partial_cmp(&va).unwrap_or(std::cmp::Ordering::Equal)
        });
        top_items.truncate(10);

        Ok(InventoryDashboardSummary {
            total_items,
            active_items,
            total_organizations: all_orgs.len() as i32,
            total_on_hand_value: format!("{:.2}", total_on_hand_value),
            total_pending_transactions: pending_txns,
            total_processed_transactions: processed_txns,
            items_by_type,
            items_by_category,
            transactions_by_action,
            top_items_by_value: serde_json::Value::Array(top_items),
            pending_cycle_counts: cycle_counts.len() as i32,
            low_stock_items: 0, // Would need min/max from items to calculate
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_org_types() {
        assert!(VALID_ORG_TYPES.contains(&"warehouse"));
        assert!(VALID_ORG_TYPES.contains(&"store"));
        assert!(VALID_ORG_TYPES.contains(&"distribution_center"));
        assert!(VALID_ORG_TYPES.contains(&"manufacturing"));
        assert!(VALID_ORG_TYPES.contains(&"other"));
    }

    #[test]
    fn test_valid_item_types() {
        assert!(VALID_ITEM_TYPES.contains(&"inventory"));
        assert!(VALID_ITEM_TYPES.contains(&"non_inventory"));
        assert!(VALID_ITEM_TYPES.contains(&"service"));
        assert!(VALID_ITEM_TYPES.contains(&"expense"));
        assert!(VALID_ITEM_TYPES.contains(&"capital"));
    }

    #[test]
    fn test_valid_transaction_actions() {
        assert!(VALID_TRANSACTION_ACTIONS.contains(&"receive"));
        assert!(VALID_TRANSACTION_ACTIONS.contains(&"issue"));
        assert!(VALID_TRANSACTION_ACTIONS.contains(&"transfer"));
        assert!(VALID_TRANSACTION_ACTIONS.contains(&"adjustment"));
        assert!(VALID_TRANSACTION_ACTIONS.contains(&"return_to_vendor"));
        assert!(VALID_TRANSACTION_ACTIONS.contains(&"cycle_count_adjustment"));
        assert!(VALID_TRANSACTION_ACTIONS.contains(&"misc_receipt"));
        assert!(VALID_TRANSACTION_ACTIONS.contains(&"misc_issue"));
    }

    #[test]
    fn test_valid_subinventory_types() {
        assert!(VALID_SUBINVENTORY_TYPES.contains(&"storage"));
        assert!(VALID_SUBINVENTORY_TYPES.contains(&"receiving"));
        assert!(VALID_SUBINVENTORY_TYPES.contains(&"staging"));
        assert!(VALID_SUBINVENTORY_TYPES.contains(&"inspection"));
    }

    #[test]
    fn test_valid_transaction_statuses() {
        assert!(VALID_TXN_STATUSES.contains(&"pending"));
        assert!(VALID_TXN_STATUSES.contains(&"approved"));
        assert!(VALID_TXN_STATUSES.contains(&"processed"));
        assert!(VALID_TXN_STATUSES.contains(&"cancelled"));
    }

    #[test]
    fn test_valid_cycle_count_methods() {
        assert!(VALID_COUNT_METHODS.contains(&"full"));
        assert!(VALID_COUNT_METHODS.contains(&"abc"));
        assert!(VALID_COUNT_METHODS.contains(&"random"));
        assert!(VALID_COUNT_METHODS.contains(&"by_category"));
    }
}
